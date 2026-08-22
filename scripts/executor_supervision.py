"""Bounded, group-supervised executor processes for the experimental runners.

The experimental Python runners drive the same executor protocol the Rust
supervisor drives, and used to do it with none of the supervisor's
guarantees. This module is where those guarantees are stated once, so the
three runners cannot drift from each other or from the reviewed
supervisor (G12-R15).

Four properties, each of which the Rust supervisor already has and each
of which was absent here:

  bounded records   a record is read with at most `limit + 1` units taken
                    from the child. Without a bound, an executor that
                    writes and never emits a newline makes the runner
                    allocate until the host intervenes, which turns a
                    typed protocol failure into a resource failure of the
                    process that was supposed to report it;

  process group     the child leads its own session, so the executor and
                    the node it starts can be signalled as one. A runner
                    that killed only the child it spawned would leave the
                    node behind, still holding the datadir and the RPC
                    port, and the next run would inherit them;

  total deadline    the whole run is bounded, not just the individual
                    waits. A row loop with a per-call timeout and no
                    total has no bound at all: every row can be just
                    under its own limit;

  cleanup always    the child is killed and reaped on EVERY exit path.
                    The runners used to clean up on the two handshake
                    refusals only, so a failure inside the row loop --
                    the common case -- left the child unkilled and
                    unreaped.

The bound is stated once here rather than per runner, and matches
`ProtocolLimits::DEFAULT` in the conformance package. The streams are
text, so the bound counts characters where the Rust side counts bytes;
for the ASCII JSON this protocol carries the two coincide, and where they
would not the character bound is the tighter of the two.

Nothing here accepts a credential, and none may be added.
"""
import json
import os
import signal
import subprocess
import time

# The record bounds, matching ProtocolLimits::DEFAULT in the conformance
# package. Stated as one mapping so a phase this module does not know is
# a lookup failure rather than a silently unbounded read.
MAXIMUM_RECORD_UNITS = {
    "handshake": 64 * 1024,
    "environment": 64 * 1024,
    "response": 4 * 1024 * 1024,
    "trailing": 4 * 1024,
}

# How long the executor group has to exit after its stdin closes before it
# is killed.
#
# Deliberately NOT the Rust supervisor's DEFAULT_EXECUTOR_CLEANUP_GRACE of
# five seconds. That value is sized for an executor that has only itself
# to shut down; these executors stop a real node, wait for it, and delete
# the datadir they own. Killing that group after five seconds would leave
# the datadir behind AND record the creator as killed rather than exited
# -- and the lifecycle lane's destruction record states exactly that exit
# status as evidence the process boundary held. The grace matches what
# the runners already allowed for the same wait.
CLEANUP_GRACE_SECONDS = 180.0

# How long a group that has been sent SIGKILL gets to be reaped. Short,
# because nothing is being waited for here except the kernel.
REAP_TIMEOUT_SECONDS = 10.0

# The default total a run may take. Deliberately generous: these lanes
# boot a real node and mine real blocks. It is a bound, not a budget.
DEFAULT_TOTAL_DEADLINE_SECONDS = 3600.0


class Deadline:
    """The whole run's bound, consulted before every blocking step.

    A per-call timeout does not bound a loop, because every iteration may
    sit just under it. This is the bound over the run.
    """

    def __init__(self, seconds: float) -> None:
        self.seconds = seconds
        self.expires_at = time.monotonic() + seconds

    def remaining(self) -> float:
        """What is left, never negative."""
        return max(0.0, self.expires_at - time.monotonic())

    def check(self, what: str) -> None:
        """Refuses where the run has already overrun."""
        if self.remaining() <= 0.0:
            raise SystemExit(
                "the run exceeded its total deadline of %.0fs, reached at: %s"
                % (self.seconds, what)
            )


def read_bounded(stream, what: str, phase: str, deadline: "Deadline" = None):
    """Reads one newline-delimited record under an explicit bound.

    `readline` is given a size, so at most `limit + 1` units are taken
    before the record is refused. A record that reaches the bound without
    a newline is a protocol failure this runner states, rather than an
    allocation it grows until the host stops it.
    """
    if deadline is not None:
        deadline.check("reading the %s" % what)
    limit = MAXIMUM_RECORD_UNITS[phase]
    line = stream.readline(limit + 1)
    if line == "":
        raise SystemExit("the executor closed its output before the %s" % what)
    if not line.endswith("\n"):
        raise SystemExit(
            "the executor's %s reached the %d-unit bound without ending a record"
            % (what, limit)
        )
    return json.loads(line)


def diagnostic_paths(directory: str, label: str):
    """The two destinations one supervised executor writes to.

    Derived from the label rather than fixed, because a lane may run two
    adapters at once -- the section 13 lifecycle lane runs a creator and
    a reader -- and two processes appending to one file would produce a
    record neither of them could be held to.
    """
    slug = "".join(
        character if character.isalnum() else "-" for character in label.lower()
    ).strip("-")
    while "--" in slug:
        slug = slug.replace("--", "-")
    slug = slug or "executor"
    return (
        os.path.join(directory, "%s-diagnostics.txt" % slug),
        os.path.join(directory, "%s-child-output.txt" % slug),
    )


class SupervisedExecutor:
    """One executor process, its group, and its guaranteed cleanup.

    Used as a context manager. The child is killed and reaped when the
    block ends, whatever ended it -- a return, a refusal, an exception
    from any row, or the deadline. That is the property the runners
    lacked: cleanup that happens on the failure paths and not only on the
    two the author remembered.

    # Where the child's diagnostics go

    Stated here rather than per runner, for the reason everything else in
    this class is stated here: the three runners drive the same protocol
    and must spawn an executor the same way. The two destinations are
    appended to whatever arguments the caller gave, exactly as the Rust
    supervisor appends them, and the directory is created if it is not
    there -- an executor refuses a run it cannot write, and that refusal
    should not be the way a runner discovers a missing directory.
    """

    def __init__(
        self,
        arguments,
        deadline: "Deadline",
        diagnostics_directory: str,
        label: str = "executor",
        cleanup_grace: float = CLEANUP_GRACE_SECONDS,
    ) -> None:
        os.makedirs(diagnostics_directory, exist_ok=True)
        output, elements_output = diagnostic_paths(diagnostics_directory, label)
        self.output = output
        self.elements_output = elements_output
        self.arguments = list(arguments) + [
            "--output", output,
            "--elements-output", elements_output,
        ]
        self.deadline = deadline
        self.label = label
        self.cleanup_grace = cleanup_grace
        self.child = None
        # Kept after the child is released, because a caller may need to
        # state how the process ended AFTER the block that owned it --
        # the lifecycle lane's destruction record is exactly that, and
        # its boundary claim rests on the creator having exited.
        self.exit_status = None
        self.was_killed = False

    def __enter__(self):
        self.deadline.check("spawning the %s" % self.label)
        self.child = subprocess.Popen(
            self.arguments,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            # Routed to the null device, exactly as the Rust supervisor
            # has it: arbitrary child bytes never become first-party
            # diagnostics. It used to be None, which INHERITS this
            # runner's stderr -- so the bytes the comment said were not
            # captured went straight to the operator's terminal. The
            # executor has two files of its own to write to instead.
            stderr=subprocess.DEVNULL,
            text=True,
            # The child leads its own session, so it and the node it
            # starts form one group this runner can signal as a whole.
            start_new_session=True,
        )
        return self

    def __exit__(self, *_ignored):
        self.close()
        return False

    @property
    def pid(self):
        """The child's identifier, which is also its group's."""
        return self.child.pid

    def send(self, value) -> None:
        """Writes one protocol record."""
        self.deadline.check("writing to the %s" % self.label)
        self.child.stdin.write(json.dumps(value, separators=(",", ":")) + "\n")
        self.child.stdin.flush()

    def read(self, what: str, phase: str = "response"):
        """Reads one protocol record, bounded and under the deadline."""
        return read_bounded(self.child.stdout, what, phase, self.deadline)

    def handshake(self, schema):
        """Performs the opening exchange and returns both records."""
        self.send({"schema": schema})
        opening = self.read("handshake", "handshake")
        environment = self.read("environment observation", "environment")
        return opening, environment

    def kill_group(self) -> None:
        """Signals the whole group, not just the process spawned.

        The executor starts a node, and killing only the executor leaves
        that node holding its datadir and its RPC port. A group kill is
        what makes the next run's environment actually fresh.
        """
        if self.child is None or self.child.poll() is not None:
            return
        self.was_killed = True
        try:
            os.killpg(os.getpgid(self.child.pid), signal.SIGKILL)
        except (ProcessLookupError, PermissionError):
            # Already gone, or never ours. Either is the state the kill
            # was asking for rather than a failure to reach it.
            pass

    def close(self) -> None:
        """Ends the exchange and guarantees the child is reaped.

        Closing stdin is how this protocol says "no more requests", and a
        well-behaved executor exits on it and deletes the datadir it
        owns. The grace is what that shutdown gets; after it, the group
        is killed. Either way the child is waited on, because a child
        that is killed but never reaped is still a process.
        """
        if self.child is None:
            return
        try:
            if self.child.stdin is not None and not self.child.stdin.closed:
                try:
                    self.child.stdin.close()
                except OSError:
                    pass
            try:
                self.child.wait(timeout=self.cleanup_grace)
            except subprocess.TimeoutExpired:
                self.kill_group()
                try:
                    # A short wait, because this one is reaping a process
                    # that has been sent SIGKILL rather than waiting for
                    # an orderly shutdown.
                    self.child.wait(timeout=REAP_TIMEOUT_SECONDS)
                except subprocess.TimeoutExpired:
                    # Reported rather than swallowed: a child this
                    # runner could not reap is a fact about the run.
                    print(
                        "warning: the %s did not exit after a group kill" % self.label,
                        flush=True,
                    )
        finally:
            # Recorded before the child is released. A caller that has to
            # state how the process ended cannot ask a child that is no
            # longer held, and "the creator had exited" is a claim this
            # lane's boundary rests on rather than a detail.
            self.exit_status = self.child.poll()
            self.child = None

    def refuse(self, note: str):
        """Kills the group and ends the run with a stated reason.

        The kill happens before the exit rather than being left to the
        context manager, because a refusal of the handshake means no
        orderly shutdown is coming.
        """
        self.kill_group()
        return SystemExit(note)
