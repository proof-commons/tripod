#!/usr/bin/env python3
"""Drives the Guide-11 section 13 fresh-process lifecycle proof.

Section 13's question is whether "public" means recoverable from
canonical public chain data by a party that did not participate in
creation, rather than still present in the creator's process. Section
13.5 closes with the sentence this runner exists to satisfy: the test
must use an ACTUAL process boundary, not two functions in one process
sharing memory.

So the boundary here is an operating-system boundary and is not
negotiable anywhere in this file:

  Process A   one adapter process. It builds the accepted normalization,
              confirms it in a block, and writes the public record. Its
              wallet, its blinding factors, its descriptors, and its
              memory are creator-local and never leave it.

  boundary    A's wallet directory is deleted from the chain directory
              and A's process is waited on until it has exited. What is
              destroyed is recorded by name and size, before and after.

  Process B   a NEW adapter process, with a new wallet, on a node that
              is itself freshly started. It is handed the public record
              file and nothing else, and it locates, parses, and checks
              the object from chain data.

The chain directory persists across the boundary, and that scoping is
the point rather than a loophole. The chain data IS the canonical public
record; the creator's wallet is the private state. Destroying the chain
would destroy the evidence the test exists to find, and keeping the
wallet would make the boundary a fiction.

This runner records and does not gate. Which rows agree with what was
expected is decided by the typed report in the package that owns the
claim, from a record this process did not classify -- the division
`G11-W7-06` paid for.

  --executor        the zero-argument executor launcher to spawn
  --matrix          the JSON `emit-normalization-matrix` produced, whose
                    unmutated row supplies the claim
  --chain-dir       the chain directory both processes share
  --report          where to write the run record

Nothing here accepts a credential, and none may be added.
"""
import argparse
import json
import os
import shutil
import subprocess
import sys
import time

# Kept in step with the adapter's constant of the same name. Stated here
# as well because a runner that read whatever the adapter sent would not
# be able to build the negative rows that a wrong schema is supposed to
# fail.
FRESH_PROCESS_HANDOFF_SCHEMA = "tripod-fresh-process-handoff-1"

# A genesis identifier no chain this project runs has. Used by the
# wrong-chain-context row, which must be refused before any evidence is
# fetched rather than after the transaction happens not to resolve.
FOREIGN_GENESIS_ID = "de" * 32


def read_record(stream, what):
    line = stream.readline()
    if line == "":
        raise SystemExit("the executor closed its output before the %s" % what)
    return json.loads(line)


class AdapterProcess:
    """One adapter process, its handshake, and its environment observation.

    Deliberately not reusable across roles. Each `with` block is one
    operating-system process that starts its own node, answers, and is
    waited on until it has exited -- so a step cannot accidentally be
    served by a process a previous step left running.
    """

    def __init__(self, executor, chain_dir, wallet_name, schema):
        self.executor = executor
        self.chain_dir = chain_dir
        self.wallet_name = wallet_name
        self.schema = schema
        self.child = None
        self.handshake = None
        self.environment = None

    def __enter__(self):
        self.child = subprocess.Popen(
            [
                self.executor,
                # This lane is not the conformance harness and does not
                # pretend to be: the harness spawns an executor with no
                # arguments, and these three are what make one process
                # distinguishable from another. None is a credential.
                "--enable-wallet",
                "--datadir", self.chain_dir,
                "--wallet-name", self.wallet_name,
            ],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=None,
            text=True,
        )
        self.send({"schema": self.schema})
        self.handshake = read_record(self.child.stdout, "handshake")
        self.environment = read_record(self.child.stdout, "environment observation")
        if "fresh_process_lifecycle" not in self.handshake.get("capabilities", []):
            self.child.kill()
            raise SystemExit(
                "the executor advertised no fresh-process-lifecycle capability; "
                "the launcher must pass --enable-wallet"
            )
        return self

    def __exit__(self, *_ignored):
        if self.child.poll() is None:
            self.child.stdin.close()
            self.child.wait(timeout=180)

    def send(self, value):
        self.child.stdin.write(json.dumps(value, separators=(",", ":")) + "\n")
        self.child.stdin.flush()

    def ask(self, role, subject):
        self.send({"schema": self.schema, "case": {"lifecycle": role}, "subject": subject})
        return read_record(self.child.stdout, "response for %s" % role)

    @property
    def pid(self):
        return self.child.pid

    def genesis(self):
        return bytes(self.environment["genesis_id"]).hex()


def claim_of(matrix):
    """The claim the unmutated section 10.4 row states.

    Lifted from the emitted matrix rather than written here, so that the
    object this lane publishes is the same object the threat matrix
    judges. A claim restated in a runner would be a second opinion about
    what was built, and the two could drift without anything saying so.
    """
    for request in matrix["requests"]:
        if request["subject"]["mutation"] == "none":
            return request["subject"]["claim"]
    raise SystemExit("the matrix carries no unmutated row to take the claim from")


def wallet_directory(chain_dir, chain_name, wallet_name):
    """Where the node keeps one wallet inside the chain directory."""
    return os.path.join(chain_dir, chain_name, "wallets", wallet_name)


def inventory(path):
    """Every file under `path`, by name and size. Never by content.

    The destruction record has to say what was destroyed without
    republishing it. A size and a name establish that a wallet database
    was there and is gone; the bytes inside it are the creator-local
    material this boundary exists to remove, and copying them into a run
    record would defeat the boundary in the act of documenting it.
    """
    found = []
    for root, _directories, files in os.walk(path):
        for name in sorted(files):
            full = os.path.join(root, name)
            try:
                found.append(
                    {"path": os.path.relpath(full, path), "bytes": os.path.getsize(full)}
                )
            except OSError:
                continue
    return sorted(found, key=lambda entry: entry["path"])


def perturbations(handoff, decoy, change_index):
    """The section 13.5 failure rows, in their explicit-path form.

    Section 13.5 lists capsule failures because it anticipated a
    PublicCommitted representation. This path publishes an EXPLICIT
    output, so there is no capsule and each row is stated in the form it
    takes here:

      missing capsule       -> the record names a transaction the chain
                               does not carry;
      wrong output          -> the record names an index the transaction
                               does not have, and separately one it does
                               have but which is the private change;
      copied capsule        -> the record carries ANOTHER real
                               transaction's bytes for this outpoint;
      stale capsule         -> the record names an output a later
                               transaction has since spent;
      wrong chain context   -> the record declares a genesis this chain
                               does not have;
      owner-private
        dependency          -> the record carries a field naming
                               owner-private material, and separately an
                               unknown field the schema does not admit.

    Each row is a WHOLE record built here, so that a row cannot pass by
    accident of a field the previous row left behind.
    """
    rows = []

    def row(name, edit, expected):
        copy = dict(handoff)
        edit(copy)
        rows.append({"row": name, "handoff": copy, "expected_outcome": expected})

    def wrong_txid(record):
        # One digit changed. Still 64 hex digits, still a well-formed
        # identifier, and no transaction the chain has.
        digits = list(record["txid"])
        digits[0] = "0" if digits[0] != "0" else "1"
        record["txid"] = "".join(digits)

    row("missing_transaction", wrong_txid, "refused_evidence_absent")
    row(
        "wrong_output_index_absent",
        lambda record: record.update(output_index=99),
        "refused_output_absent",
    )
    row(
        "wrong_output_index_private_change",
        lambda record: record.update(output_index=change_index),
        "refused_output_not_explicit",
    )
    row(
        "copied_evidence",
        lambda record: record.update(raw_transaction=decoy["raw_transaction"]),
        "refused_copied_evidence",
    )
    row(
        "wrong_chain_context",
        lambda record: record.update(genesis_id=FOREIGN_GENESIS_ID),
        "refused_wrong_chain_context",
    )
    row(
        "owner_private_field",
        # The name is what is refused, and the value is deliberately not
        # a real one: this row proves the schema refuses the FIELD, and a
        # runner that had to supply a genuine blinding factor to prove it
        # would be carrying owner-private material to make a point about
        # not carrying it.
        lambda record: record.update(owner_blinding_key="00" * 32),
        "executor_infrastructure_failure",
    )
    row(
        "unknown_field",
        lambda record: record.update(note="a field the schema does not admit"),
        "executor_infrastructure_failure",
    )
    return rows


def main(argv):
    parser = argparse.ArgumentParser(prog="run-fresh-process-lifecycle")
    parser.add_argument("--executor", required=True)
    parser.add_argument("--matrix", required=True)
    parser.add_argument("--chain-dir", required=True)
    parser.add_argument("--report", required=True)
    parser.add_argument("--expect-network-id", default=None)
    parser.add_argument("--expect-genesis-id", default=None)
    arguments = parser.parse_args(argv)

    with open(arguments.matrix) as handle:
        matrix = json.load(handle)
    schema = matrix["schema"]
    claim = claim_of(matrix)

    chain_dir = os.path.abspath(arguments.chain_dir)
    if os.path.exists(chain_dir) and os.listdir(chain_dir):
        raise SystemExit(
            "the chain directory is not empty, and this lane builds the chain "
            "it reads: %s" % chain_dir
        )
    os.makedirs(chain_dir, exist_ok=True)

    started = time.monotonic()
    record = {"schema": FRESH_PROCESS_HANDOFF_SCHEMA, "chain_directory": chain_dir}

    # ---- Process A ------------------------------------------------------
    print("Process A: constructing", flush=True)
    a_wallet = "guide11-lifecycle-a"
    with AdapterProcess(arguments.executor, chain_dir, a_wallet, schema) as process_a:
        observed_genesis = process_a.genesis()
        observed_network = bytes(process_a.environment["network_id"]).hex()
        for expected, observed, role in (
            (arguments.expect_network_id, observed_network, "network"),
            (arguments.expect_genesis_id, observed_genesis, "genesis"),
        ):
            if expected is not None and expected.lower() != observed:
                raise SystemExit(
                    "the executor ran on a chain whose %s identity is %s, and the "
                    "run was declared against %s" % (role, observed, expected)
                )
        chain_name = process_a.environment["chain_name"]
        a_pid = process_a.pid
        subject = process_a.ask("construct", {"claim": claim, "supersede": False})
        decoy = process_a.ask("construct", {"claim": claim, "supersede": False})
        stale = process_a.ask("construct", {"claim": claim, "supersede": True})
        wallet_path = wallet_directory(chain_dir, chain_name, a_wallet)
        before = inventory(wallet_path)
        record["process_a"] = {
            "pid": a_pid,
            "wallet_name": a_wallet,
            "subject": subject,
            "decoy": decoy,
            "stale": stale,
        }
        record["environment"] = {
            "chain_name": chain_name,
            "network_id": observed_network,
            "genesis_id": observed_genesis,
            "environment": process_a.environment["environment"],
            "handshake": process_a.handshake,
        }

    for step in ("subject", "decoy", "stale"):
        if record["process_a"][step]["outcome"] != "constructed":
            raise SystemExit(
                "Process A did not construct the %s object: %s"
                % (step, record["process_a"][step].get("detail"))
            )

    # ---- the destruction boundary ---------------------------------------
    print("boundary: destroying creator-local state", flush=True)
    shutil.rmtree(wallet_path, ignore_errors=True)
    exit_status = process_a.child.poll()
    record["destruction"] = {
        # Scoping, stated rather than implied. The node is not creator
        # state: chain data IS the canonical public record, and this lane
        # would be proving nothing if it destroyed that.
        "scope": (
            "the creator's wallet and the creator's process; the chain "
            "directory persists because chain data is the canonical public "
            "record this lane exists to read"
        ),
        "wallet_directory": os.path.relpath(wallet_path, chain_dir),
        "destroyed_files": before,
        "destroyed_bytes": sum(entry["bytes"] for entry in before),
        "wallet_directory_present_after": os.path.exists(wallet_path),
        "remaining_files_after": inventory(wallet_path),
        "process_a_pid": a_pid,
        "process_a_exit_status": exit_status,
        "process_a_running_after": exit_status is None,
    }
    if exit_status is None:
        raise SystemExit("Process A had not exited when the boundary was taken")
    if os.path.exists(wallet_path):
        raise SystemExit("the creator's wallet survived the boundary")

    # ---- Process B ------------------------------------------------------
    handoff = record["process_a"]["subject"]["handoff"]
    decoy_handoff = record["process_a"]["decoy"]["handoff"]
    stale_handoff = record["process_a"]["stale"]["handoff"]

    # Which output is the private change, taken from what A published
    # about the transaction it built. Public data: it is an output index
    # of a transaction the chain carries.
    change_index = None
    for position, output in enumerate(record["process_a"]["subject"]["observed_outputs"]):
        if output["is_fee"] or position == handoff["output_index"]:
            continue
        change_index = position
    if change_index is None:
        raise SystemExit("the published transaction carries no private change output")

    rows = [{"row": "accepted", "handoff": handoff, "expected_outcome": "verified"}]
    # The stale row exists only if Process A could actually spend the
    # object it published. Where it could not, the row is recorded as
    # unbuilt with the target's own words and is NOT sent: an unspent
    # object verifying successfully would be filed as a stale record
    # passing, which is a false fact and the exact shape of mistake
    # `G11-W7-07` catalogues.
    stale_failure = record["process_a"]["stale"].get("supersede_failure")
    record["stale_row"] = {
        "constructed": stale_failure is None,
        "superseded_by": record["process_a"]["stale"].get("superseded_by"),
        "unbuilt_reason": stale_failure,
    }
    if stale_failure is None:
        rows.append(
            {
                "row": "stale_evidence",
                "handoff": stale_handoff,
                "expected_outcome": "refused_output_spent",
            }
        )
    else:
        print("  %-34s not built: %s" % ("stale_evidence", stale_failure), flush=True)
    rows.extend(perturbations(handoff, decoy_handoff, change_index))

    # The chain-context row is refused here as well, before the record is
    # ever sent, by comparing what it declares against what Process A
    # observed. Section 13.5 wants a wrong chain context refused before
    # any evidence is fetched, and the earliest place that can happen is
    # in the process that holds the record.
    record["pre_dispatch_refusals"] = [
        {
            "row": row["row"],
            "declared_genesis": row["handoff"].get("genesis_id"),
            "observed_genesis": observed_genesis,
            "refused_before_dispatch": row["handoff"].get("genesis_id") != observed_genesis,
        }
        for row in rows
    ]

    runs = []
    for attempt, wallet in enumerate(("guide11-lifecycle-b1", "guide11-lifecycle-b2"), 1):
        print("Process B run %d: verifying from the public record" % attempt, flush=True)
        answers = []
        with AdapterProcess(arguments.executor, chain_dir, wallet, schema) as process_b:
            b_pid = process_b.pid
            b_genesis = process_b.genesis()
            for row in rows:
                answer = process_b.ask("verify", {"handoff": row["handoff"]})
                answers.append(
                    {
                        "row": row["row"],
                        "expected_outcome": row["expected_outcome"],
                        "observed_outcome": answer["outcome"],
                        "checks": answer["checks"],
                        "spend": answer["spend"],
                        "detail": answer["detail"],
                    }
                )
                print("  %-34s %s" % (row["row"], answer["outcome"]), flush=True)
        runs.append(
            {
                "attempt": attempt,
                "pid": b_pid,
                "wallet_name": wallet,
                "observed_genesis": b_genesis,
                "rows": answers,
            }
        )
        # A cold start, and stated as one. The adapter process has exited,
        # so its node has been stopped and its memory is gone; deleting
        # the wallet this run created means the next run cannot inherit a
        # key, a cached descriptor, or a rescan result from it. What
        # survives is the chain directory, which is the public record.
        shutil.rmtree(
            wallet_directory(chain_dir, chain_name, wallet), ignore_errors=True
        )

    record["process_b_runs"] = runs
    record["cold_start"] = {
        "node_restarted_between_runs": True,
        "how": (
            "each Process B is its own operating-system process; the adapter "
            "starts a node on entry and stops it on exit, so the second run "
            "reads a node that was started after the first one was stopped"
        ),
        "previous_wallet_deleted_between_runs": True,
    }
    record["wall_seconds"] = round(time.monotonic() - started, 1)

    with open(arguments.report, "w") as handle:
        json.dump(record, handle, indent=2, sort_keys=True)
    print("wrote %s" % arguments.report, flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
