#!/usr/bin/env python3
"""Subprocess tests for the executor adapter's two boundaries (CI-009).

`scripts/test-executor-classification.py` covers what the adapter's pure
functions decide. This file covers what crosses its edges, which needs the
adapter running as an actual child process: what it writes, where, and what
it refuses to read.

Node-free by construction. Both subjects are reachable before any node is
up -- the diagnostic split shows itself the first time `elements-cli` is
run at all, and every request-framing case is posed at the handshake, which
this adapter reads before it boots anything. The `elementsd` and
`elements-cli` the tests configure are two-line stand-ins, and the upstream
functional-test framework is a stub package holding the two module names
the adapter imports.

What the marker test proves
---------------------------
The configured child writes one unique string on its stderr. The adapter
must file that string in the `--elements-output` quarantine and nowhere
else: not on its own stderr, which must be empty; not on stdout, which is
protocol data; and not in `--output`, which carries typed facts only.

Asserting the absences is the whole point, and it is why the marker is a
string nothing else could produce. A test that only checked the quarantine
would pass just as well for an adapter that wrote the child's bytes to
every stream it had.

What the framing tests prove
----------------------------
The harness enforces a strict bounded framing on the adapter's answers;
the adapter enforces the same framing on the requests. Five endings are
told apart -- a clean end at a record boundary, a stream that ended inside
a record, a record that reached the byte bound with no newline, a blank
record, and a well-framed record that is not one JSON object -- and each
must produce its own fixed spelling in the diagnostic file.

They are asserted as five distinct outcomes rather than five nonzero
exits. A reader of the diagnostic stream has a different next step for
each, and a framing that reported "bad record" for all of them would
report a harness that stopped writing and a harness that wrote four
megabytes without a newline as the same event.

The bound itself is asserted against the harness's own constant, read out
of `protocol.rs`. A bound only one side held would be a framing only one
side had.

Run standalone (`python3 scripts/test-executor-boundaries.py`) or as the
`executor-boundaries` meson test.
"""

from __future__ import annotations

import importlib.util
import io
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time

REPOSITORY_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
EXECUTOR_PATH = os.path.join(REPOSITORY_ROOT, "scripts", "elements-native-executor.py")
PROTOCOL_PATH = os.path.join(
    REPOSITORY_ROOT, "packages", "target-elements-conformance", "src", "protocol.rs"
)

# The string the stand-in client writes on its stderr.
#
# Deliberately unmistakable and deliberately not a real message: what the
# assertions are about is where a byte went, so the byte has to be one that
# could not have arrived by any other route. It names nothing and is
# test-fixture material under `(´[ADR015-rule:security:test-material]´)`.
MARKER = "ELEMENTS_CLI_STDERR_MARKER_THAT_MUST_ONLY_BE_QUARANTINED"

# A public development identifier, as the adapter spells them.
NETWORK_ID = "1111111111111111111111111111111111111111111111111111111111111111"

# The adapter's protocol revision, read from the adapter rather than
# restated: a test that pinned its own number would keep passing through a
# revision bump by testing a revision nobody speaks.
SCHEMA = None

# A request-controlled line-shaped value. It is not itself a diagnostic:
# the tests below place it where only the quarantine may retain it.
INJECTED_LINE_MARKER = "INJECTED_DIAGNOSTIC_LINE_THAT_MUST_NOT_EXIST"


class Failures:
    """Collected assertion failures, so one run reports every problem."""

    def __init__(self) -> None:
        self.notes = []

    def check(self, condition: bool, note: str) -> None:
        if not condition:
            self.notes.append(note)

    def equal(self, actual, expected, note: str) -> None:
        self.check(
            actual == expected,
            "%s: expected %r, observed %r" % (note, expected, actual),
        )


class Fixture:
    """One disposable configuration for the adapter under test.

    Holds a stub framework package, a stand-in `elementsd`, a stand-in
    `elements-cli`, and the two diagnostic destinations. Everything lives
    in one temporary directory that the fixture removes.
    """

    def __init__(self, client_body: str, node_seconds: float) -> None:
        self.root = tempfile.mkdtemp(prefix="tripod-executor-boundaries-")
        framework = os.path.join(self.root, "framework", "test_framework")
        os.makedirs(framework)
        # The two module names `load_framework` imports, and a third it
        # treats as optional. Empty: nothing in these tests reaches a
        # framework helper, because nothing here builds a transaction.
        for name in ("__init__.py", "messages.py", "script.py", "key.py"):
            open(os.path.join(framework, name), "w").close()
        self.framework = os.path.dirname(framework)

        # A node that answers `--version` and then stays up for a bounded
        # while. It has to outlive the boot attempt: an `elementsd` that
        # exited first would be reported as a node that died during
        # startup, and the client would never be run at all.
        self.elementsd = self.script(
            "elementsd",
            'if [ "$1" = "--version" ]; then\n'
            '  echo "Elements Core Daemon version v23.99.0-0000000000000000000000000000000000000000"\n'
            "  exit 0\n"
            "fi\n"
            "sleep %.2f\n" % node_seconds,
        )
        self.elements_cli = self.script("elements-cli", client_body)
        self.output = os.path.join(self.root, "executor-diagnostics.txt")
        self.elements_output = os.path.join(self.root, "executor-child-output.txt")

    def script(self, name: str, body: str) -> str:
        """Writes one executable stand-in binary."""
        path = os.path.join(self.root, name)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write("#!/bin/sh\n" + body)
        os.chmod(path, 0o755)
        return path

    def argv(self, boot_timeout: float):
        return [
            sys.executable,
            EXECUTOR_PATH,
            "--output",
            self.output,
            "--elements-output",
            self.elements_output,
            "--elementsd",
            self.elementsd,
            "--elements-cli",
            self.elements_cli,
            "--framework",
            self.framework,
            "--network-id",
            NETWORK_ID,
            "--boot-timeout-seconds",
            "%.2f" % boot_timeout,
        ]

    def read(self, path: str) -> str:
        if not os.path.exists(path):
            return ""
        with open(path, encoding="utf-8", errors="replace") as handle:
            return handle.read()

    def diagnostics(self) -> str:
        return self.read(self.output)

    def quarantine(self) -> str:
        return self.read(self.elements_output)

    def close(self) -> None:
        shutil.rmtree(self.root, ignore_errors=True)


class Run:
    """What one adapter process did."""

    def __init__(self, status, stdout: bytes, stderr: bytes, seconds: float) -> None:
        self.status = status
        self.stdout = stdout
        self.stderr = stderr
        self.seconds = seconds


def drive(fixture: Fixture, stdin: bytes, boot_timeout: float = 0.3) -> Run:
    """Runs the adapter as a child, with the given bytes on its stdin.

    The bytes are handed over exactly as given, without a trailing newline
    added: whether a record was terminated is precisely what several of
    these tests are about, so nothing here may terminate one on the
    caller's behalf.
    """
    started = time.monotonic()
    completed = subprocess.run(
        fixture.argv(boot_timeout),
        input=stdin,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
        timeout=120,
    )
    return Run(
        completed.returncode,
        completed.stdout,
        completed.stderr,
        time.monotonic() - started,
    )


def adapter_schema() -> int:
    """The protocol revision the adapter declares, read from its source."""
    with open(EXECUTOR_PATH, encoding="utf-8") as handle:
        for line in handle:
            match = re.match(r"^NATIVE_PROTOCOL_SCHEMA = (\d+)", line)
            if match:
                return int(match.group(1))
    raise SystemExit("the adapter declares no protocol revision")


def executor_module():
    """Loads the adapter for node-free tests of its diagnostic boundary."""
    spec = importlib.util.spec_from_file_location(
        "elements_native_executor_boundary_subject", EXECUTOR_PATH
    )
    if spec is None or spec.loader is None:
        raise SystemExit("the executor module could not be loaded")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def memory_diagnostics(module):
    """Configures one in-memory typed stream and quarantine."""
    output = io.StringIO()
    quarantine = io.StringIO()
    module.STREAMS = module.DiagnosticStreams(output, quarantine)
    return output, quarantine


def adapter_bound(name: str) -> int:
    """One request bound, read from the adapter's own source."""
    with open(EXECUTOR_PATH, encoding="utf-8") as handle:
        for line in handle:
            match = re.match(r"^%s = (.+)$" % re.escape(name), line)
            if match:
                return int(eval_bound(match.group(1)))
    raise SystemExit("the adapter states no %s" % name)


def harness_bound(name: str) -> int:
    """The same bound, read from the harness's typed contract."""
    with open(PROTOCOL_PATH, encoding="utf-8") as handle:
        for line in handle:
            match = re.match(
                r"^pub const %s: usize = (.+);$" % re.escape(name), line
            )
            if match:
                return int(eval_bound(match.group(1)))
    raise SystemExit("the harness states no %s" % name)


def eval_bound(text: str) -> int:
    """Reads a `A * B * C` product of integers, and nothing else.

    Deliberately not an expression evaluator. The two files state their
    bounds as a product of literals, and a reader that accepted more than
    that would be running whatever a source file happened to contain.
    """
    total = 1
    for piece in text.split("*"):
        total *= int(piece.strip().rstrip("_usize"))
    return total


def handshake(extra=None) -> bytes:
    """One handshake request record, optionally carrying an extra field."""
    record = {"schema": SCHEMA}
    if extra is not None:
        record.update(extra)
    return (json.dumps(record, separators=(",", ":")) + "\n").encode("utf-8")


# --------------------------------------------------------------------------
# G13-R06: the marker never leaves the quarantine
# --------------------------------------------------------------------------


def test_child_stderr_reaches_only_the_quarantine(failures) -> int:
    """The configured client's stderr appears in one file and no other.

    The client fails every call, so the adapter cannot bring its node up
    and refuses the run -- which is the shortest path to a real
    `elements-cli` failure with real child bytes on a real stderr. What
    the run produced afterwards is what the assertions are about.
    """
    fixture = Fixture(
        client_body='echo "%s" >&2\nexit 1\n' % MARKER,
        node_seconds=2.0,
    )
    try:
        run = drive(fixture, handshake())
        diagnostics = fixture.diagnostics()
        quarantine = fixture.quarantine()

        failures.check(run.status != 0, "a run whose node never answered is refused")
        # The four absences, which are the substance of the row.
        failures.equal(
            run.stderr, b"", "the adapter writes nothing at all on its own stderr"
        )
        failures.check(
            MARKER.encode("utf-8") not in run.stdout,
            "the marker reached stdout, which is protocol data only",
        )
        failures.check(
            MARKER not in diagnostics,
            "the marker reached --output, which carries typed facts only",
        )
        # And the one presence.
        failures.check(
            MARKER in quarantine,
            "the marker did not reach --elements-output, so it was dropped "
            "rather than quarantined",
        )
        # The correlation the two files are supposed to support: a typed
        # line naming a record number, and that record present next door.
        numbers = re.findall(r"elements-output record (\d+)", diagnostics)
        failures.check(
            bool(numbers),
            "no typed line named an elements-output record, so nothing "
            "correlates the two files",
        )
        for number in numbers:
            failures.check(
                ("record %s:" % number) in quarantine,
                "typed line names elements-output record %s, which is absent"
                % number,
            )
        # The quarantine's header is this adapter's own sentence, built
        # from typed facts, and states the method the call belonged to.
        failures.check(
            "getblockchaininfo" in quarantine,
            "the quarantine entry does not name the method it belongs to",
        )
        return 7 + len(numbers)
    finally:
        fixture.close()


def test_no_configuration_path_reaches_either_file(failures) -> int:
    """Neither diagnostic file names a path out of this operator's argv.

    The adapter used to write the framework path into a diagnostic. A
    diagnostic file travels -- it is kept as a run artifact beside a
    report -- so an operator's directory layout is no more at home there
    than in a first-party record.
    """
    fixture = Fixture(
        client_body='echo "%s" >&2\nexit 1\n' % MARKER,
        node_seconds=2.0,
    )
    try:
        drive(fixture, handshake())
        for role, text in (
            ("--output", fixture.diagnostics()),
            ("--elements-output", fixture.quarantine()),
        ):
            failures.check(
                fixture.framework not in text,
                "%s names the configured framework path" % role,
            )
            failures.check(
                fixture.elements_cli not in text,
                "%s names the configured client path" % role,
            )
        return 4
    finally:
        fixture.close()


# --------------------------------------------------------------------------
# G13-R18: five framing endings, told apart
# --------------------------------------------------------------------------


def framing_case(failures, stdin: bytes, expected: str, note: str) -> int:
    """Drives one malformed handshake and asserts the typed refusal.

    The client here never runs: every case is refused while reading the
    handshake, which the adapter reads before it boots anything.
    """
    fixture = Fixture(client_body="exit 1\n", node_seconds=0.1)
    try:
        run = drive(fixture, stdin)
        diagnostics = fixture.diagnostics()
        failures.check(
            run.status != 0, "%s: a framing failure is a nonzero exit" % note
        )
        failures.equal(run.stderr, b"", "%s: stderr stays empty" % note)
        failures.check(
            expected in diagnostics,
            "%s: expected the %r spelling, and --output holds %r"
            % (note, expected, diagnostics),
        )
        return 3
    finally:
        fixture.close()


def test_a_stream_that_ends_before_the_handshake(failures) -> int:
    """Nothing at all is a clean end that arrived in the wrong phase."""
    return framing_case(failures, b"", "clean_eof", "an empty request stream")


def test_a_blank_record_is_fatal_rather_than_skipped(failures) -> int:
    """A record that says nothing is refused, not stepped over."""
    return framing_case(
        failures, b"\n", "blank_record", "a blank record"
    ) + framing_case(
        failures, b"   \t  \n", "blank_record", "a whitespace-only record"
    )


def test_a_malformed_record_is_named_as_one(failures) -> int:
    """Well framed, and not one JSON object."""
    return framing_case(
        failures, b"not json at all\n", "malformed_record", "a record that is not JSON"
    ) + framing_case(
        failures, b"[1,2,3]\n", "malformed_record", "a record that is not an object"
    )


def test_a_record_past_the_bound_is_refused_without_unbounded_reading(
    failures,
) -> int:
    """The bound is reached and the record refused, whatever follows it.

    The record here is one byte past the bound and IS terminated, so an
    adapter that read to the newline before judging the size would see a
    well-formed line. It must refuse on the bound instead.
    """
    bound = adapter_bound("MAXIMUM_HANDSHAKE_REQUEST_BYTES")
    oversized = b"{\"schema\":" + b"0" * (bound + 1) + b"}\n"
    return framing_case(
        failures, oversized, "oversized_record", "a record past the bound"
    )


def test_a_stream_that_ends_inside_a_record(failures) -> int:
    """A harness that stopped writing part-way is its own fault.

    The bytes are a complete, valid handshake in every respect but the
    newline, so the only thing wrong is where the stream ended -- which is
    exactly the distinction this case exists to keep.
    """
    return framing_case(
        failures,
        handshake().rstrip(b"\n"),
        "unterminated_record",
        "a record the stream ended inside",
    )


def test_an_extra_handshake_field_is_refused(failures) -> int:
    """Revision 4 is a field census, mirroring `deny_unknown_fields`.

    An executor that ignored an unknown member would be speaking a
    revision the harness is not, quietly. The refusal is what keeps one
    revision meaning one schema.
    """
    fixture = Fixture(client_body="exit 1\n", node_seconds=0.1)
    try:
        run = drive(fixture, handshake({"extra_member": True}))
        diagnostics = fixture.diagnostics()
        failures.check(run.status != 0, "an extra handshake field is refused")
        failures.equal(run.stderr, b"", "stderr stays empty on a census refusal")
        failures.check(
            "field census" in diagnostics,
            "the refusal is not named as a census failure: %r" % diagnostics,
        )
        # The field's own name is the harness's text, so it is quarantined
        # rather than written into a typed line.
        failures.check(
            "extra_member" not in diagnostics,
            "the unknown field's name reached --output, which carries typed "
            "facts only",
        )
        failures.check(
            "extra_member" in fixture.quarantine(),
            "the unknown field's name was dropped rather than quarantined",
        )
        return 5
    finally:
        fixture.close()


def test_an_extra_execution_field_cannot_inject_a_typed_line(failures) -> int:
    """A request field name is quarantined and has one fixed typed outcome."""
    module = executor_module()
    output, quarantine = memory_diagnostics(module)
    field = "extra\r\n%s" % INJECTED_LINE_MARKER
    request = {
        "schema": SCHEMA,
        "case": {"group": "boundary", "ordinal": 1},
        "subject": {},
        "construction": None,
        field: True,
    }
    try:
        try:
            module.answer_case(object(), request)
        except module.FatalAdapterError as error:
            module.report_fatal(error)
        else:
            failures.check(False, "an extra execution field was accepted")

        diagnostics = output.getvalue()
        quarantined = quarantine.getvalue()
        failures.equal(
            diagnostics.splitlines(),
            [
                "elements-native-executor: fatal: execution request failed its "
                "field census; detail is elements-output record 1"
            ],
            "the execution census emits exactly one fixed typed line",
        )
        failures.check(
            INJECTED_LINE_MARKER not in diagnostics,
            "the request-controlled marker became a typed line",
        )
        failures.check(
            field in quarantined,
            "the request-controlled field name was not quarantined exactly",
        )
        return 4
    finally:
        module.STREAMS = None


def test_the_typed_sink_rejects_line_bearing_content(failures) -> int:
    """CR or LF content is dropped and replaced by one fixed fatal line."""
    module = executor_module()
    output = io.StringIO()
    quarantine = io.StringIO()
    streams = module.DiagnosticStreams(output, quarantine)
    injected = "unrecognized\r\n%s" % INJECTED_LINE_MARKER
    try:
        streams.typed(injected)
    except module.FatalAdapterError:
        pass
    else:
        failures.check(False, "the typed sink accepted line-bearing content")

    failures.equal(
        output.getvalue().splitlines(),
        ["elements-native-executor: fatal: typed diagnostic content rejected"],
        "the sink replaces line-bearing content with its fixed fatal outcome",
    )
    failures.check(
        INJECTED_LINE_MARKER not in output.getvalue(),
        "the typed sink wrote the injected line",
    )
    failures.equal(
        quarantine.getvalue(),
        "",
        "the typed sink retained content it was required to drop",
    )
    return 4


def test_an_unknown_diagnostic_outcome_is_fatal(failures) -> int:
    """An unrecognized form is quarantined but never written through."""
    module = executor_module()
    output = io.StringIO()
    quarantine = io.StringIO()
    streams = module.DiagnosticStreams(output, quarantine)
    marker = "UNKNOWN_DIAGNOSTIC_OUTCOME_MARKER"
    try:
        streams.typed(marker)
    except module.FatalAdapterError:
        pass
    else:
        failures.check(False, "an unknown diagnostic outcome was accepted")

    failures.equal(
        output.getvalue().splitlines(),
        [
            "elements-native-executor: fatal: unrecognized diagnostic outcome; "
            "detail is elements-output record 1"
        ],
        "the unknown outcome takes the fixed fatal path",
    )
    failures.check(marker not in output.getvalue(), "the unknown form was written through")
    failures.check(marker in quarantine.getvalue(), "the unknown form was not quarantined")
    return 4


def test_zk_paths_and_loader_exceptions_reach_neither_file(failures) -> int:
    """Materializer diagnostics retain neither a path nor exception text."""
    module = executor_module()
    output, quarantine = memory_diagnostics(module)
    marker = "/operator/ZK_PATH_AND_EXCEPTION_MARKER/libzk.so"

    class Operations:
        materializer = None

    class Executor:
        operations = Operations()

    original = module.ConfidentialMaterializer
    try:
        module.ConfidentialMaterializer = lambda _library: object()
        module.initialize_confidential_materializer(Executor(), marker)

        def fail_initialization(_library):
            raise OSError(marker)

        module.ConfidentialMaterializer = fail_initialization
        module.initialize_confidential_materializer(Executor(), marker)

        failures.equal(
            output.getvalue().splitlines(),
            [
                "elements-native-executor: confidential materializer ready",
                "elements-native-executor: confidential materializer initialization failed",
            ],
            "materializer outcomes use only their fixed spellings",
        )
        failures.check(marker not in output.getvalue(), "the ZK path reached --output")
        failures.check(
            marker not in quarantine.getvalue(),
            "the ZK path or loader exception reached --elements-output",
        )
        return 3
    finally:
        module.ConfidentialMaterializer = original
        module.STREAMS = None


def test_a_wrong_revision_is_refused_before_any_node(failures) -> int:
    """A revision this adapter does not speak ends the exchange."""
    fixture = Fixture(client_body="exit 1\n", node_seconds=0.1)
    try:
        run = drive(fixture, json.dumps({"schema": SCHEMA + 1}).encode("utf-8") + b"\n")
        failures.check(run.status != 0, "an unknown revision is refused")
        failures.equal(run.stderr, b"", "stderr stays empty on a revision refusal")
        failures.check(
            "protocol revision" in fixture.diagnostics(),
            "the refusal does not name the revision as the reason",
        )
        return 3
    finally:
        fixture.close()


# --------------------------------------------------------------------------
# The cross-language agreement the bounds rest on
# --------------------------------------------------------------------------


def test_the_request_bounds_agree_across_the_two_implementations(failures) -> int:
    """Both sides state the same figures, read from both sources.

    Not a restatement here of what either says: the numbers are read out
    of the adapter's source and out of the harness's typed contract, and
    compared. A bound only one side held would let the harness believe it
    had tightened the framing while the adapter went on reading whatever
    arrived.
    """
    checked = 0
    for name in ("MAXIMUM_HANDSHAKE_REQUEST_BYTES", "MAXIMUM_REQUEST_BYTES"):
        failures.equal(
            adapter_bound(name),
            harness_bound(name),
            "%s differs between the adapter and the harness" % name,
        )
        checked += 1
    return checked


# --------------------------------------------------------------------------
# The two mandatory destinations
# --------------------------------------------------------------------------


def test_both_destinations_are_mandatory(failures) -> int:
    """Neither argument has a default, and neither may be omitted."""
    fixture = Fixture(client_body="exit 1\n", node_seconds=0.1)
    checked = 0
    try:
        for dropped in ("--output", "--elements-output"):
            argv = fixture.argv(0.1)
            index = argv.index(dropped)
            del argv[index : index + 2]
            completed = subprocess.run(
                argv,
                input=b"",
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
                timeout=60,
            )
            failures.check(
                completed.returncode != 0,
                "a run without %s must be refused" % dropped,
            )
            checked += 1
        return checked
    finally:
        fixture.close()


def test_one_file_for_two_roles_is_refused(failures) -> int:
    """Two destinations that name one file are not two destinations.

    A separation a configuration mistake can undo is not a separation, and
    the refusal is the exit status alone: there is no honest place to
    explain a failure to establish the places where explanations go.
    """
    fixture = Fixture(client_body="exit 1\n", node_seconds=0.1)
    try:
        argv = fixture.argv(0.1)
        argv[argv.index("--elements-output") + 1] = fixture.output
        completed = subprocess.run(
            argv,
            input=b"",
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            timeout=60,
        )
        failures.check(
            completed.returncode != 0,
            "one file named for both roles must be refused",
        )
        failures.equal(
            completed.stderr,
            b"",
            "even the refusal of the destinations stays off stderr",
        )
        return 2
    finally:
        fixture.close()


TESTS = (
    ("child stderr reaches only the quarantine", test_child_stderr_reaches_only_the_quarantine),
    ("no configuration path reaches either file", test_no_configuration_path_reaches_either_file),
    ("a stream that ends before the handshake", test_a_stream_that_ends_before_the_handshake),
    ("a blank record is fatal rather than skipped", test_a_blank_record_is_fatal_rather_than_skipped),
    ("a malformed record is named as one", test_a_malformed_record_is_named_as_one),
    ("a record past the bound is refused", test_a_record_past_the_bound_is_refused_without_unbounded_reading),
    ("a stream that ends inside a record", test_a_stream_that_ends_inside_a_record),
    ("an extra handshake field is refused", test_an_extra_handshake_field_is_refused),
    ("an extra execution field cannot inject a typed line", test_an_extra_execution_field_cannot_inject_a_typed_line),
    ("the typed sink rejects line-bearing content", test_the_typed_sink_rejects_line_bearing_content),
    ("an unknown diagnostic outcome is fatal", test_an_unknown_diagnostic_outcome_is_fatal),
    ("ZK paths and loader exceptions reach neither file", test_zk_paths_and_loader_exceptions_reach_neither_file),
    ("a wrong revision is refused before any node", test_a_wrong_revision_is_refused_before_any_node),
    ("the request bounds agree across implementations", test_the_request_bounds_agree_across_the_two_implementations),
    ("both destinations are mandatory", test_both_destinations_are_mandatory),
    ("one file for two roles is refused", test_one_file_for_two_roles_is_refused),
)


def main() -> int:
    global SCHEMA
    SCHEMA = adapter_schema()
    print("adapter protocol revision: %d" % SCHEMA)
    failed = 0
    started = time.monotonic()
    for name, test in TESTS:
        failures = Failures()
        checked = test(failures)
        if failures.notes:
            failed += 1
            print("FAIL %s" % name)
            for note in failures.notes:
                print("     %s" % note)
        else:
            print("ok   %s (%d assertions)" % (name, checked))
    elapsed = time.monotonic() - started
    print(
        "%d tests %s in %.1fs"
        % (len(TESTS), "passed" if failed == 0 else "FAILED", elapsed)
    )
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
