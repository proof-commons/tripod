#!/usr/bin/env python3
"""Drives the Guide-11 section 8.4 conservation matrix through the adapter.

This runner speaks the same newline-delimited JSON executor protocol the
conformance harness speaks, spawns the selected executor with NO arguments
exactly as the harness does, and records what each row observed. It exists
because the conservation lane's evidence has to come from the reviewed
adapter driving a real node, and it is deliberately thin: every fixture it
sends comes from `emit-conservation-matrix`, so nothing here re-derives the
fixture language.

What it does NOT do is gate. It records observations and the comparison of
each row's expected layer against its observed one; whether a run is
evidence is a question for the typed report and its gate, and this runner
makes no such claim.

  --executor        the zero-argument executor to spawn
  --matrix          the JSON `emit-conservation-matrix` produced
  --report          where to write the run record

Nothing here accepts a credential, and none may be added.
"""
import argparse
import json
import sys
import time

from executor_supervision import (
    DEFAULT_TOTAL_DEADLINE_SECONDS,
    Deadline,
    SupervisedExecutor,
)


def main(argv):
    parser = argparse.ArgumentParser(prog="run-conservation-matrix")
    parser.add_argument("--executor", required=True)
    parser.add_argument("--matrix", required=True)
    parser.add_argument("--report", required=True)
    parser.add_argument("--expect-network-id", default=None)
    parser.add_argument("--expect-genesis-id", default=None)
    parser.add_argument(
        "--total-deadline-seconds",
        type=float,
        default=DEFAULT_TOTAL_DEADLINE_SECONDS,
        help="the bound on the whole run, not on any single row",
    )
    arguments = parser.parse_args(argv)

    with open(arguments.matrix) as handle:
        matrix = json.load(handle)
    rows = {row["id"]["name"]: row for row in matrix["rows"]}

    started = time.monotonic()
    deadline = Deadline(arguments.total_deadline_seconds)
    # Every path out of this block kills the executor's group and reaps
    # it, including the refusals below and any exception a row raises
    # (G12-R15).
    with SupervisedExecutor([arguments.executor], deadline) as executor:
        return run_matrix(arguments, matrix, rows, executor, deadline, started)


def run_matrix(arguments, matrix, rows, executor, deadline, started):
    """Drives the whole matrix through one supervised executor."""
    handshake, environment = executor.handshake(matrix["schema"])

    if "confidential_conservation" not in handshake.get("capabilities", []):
        raise executor.refuse(
            "the executor advertised no confidential-conservation capability; "
            "the launcher must pass --enable-wallet"
        )

    # The environment is checked before any row runs, on the harness's own
    # reasoning: a run must not label one chain with another chain's
    # identity, and a mismatch is not something a later row can repair.
    def as_hex(values):
        return bytes(values).hex()

    observed_network = as_hex(environment["network_id"])
    observed_genesis = as_hex(environment["genesis_id"])
    for expected, observed, role in (
        (arguments.expect_network_id, observed_network, "network"),
        (arguments.expect_genesis_id, observed_genesis, "genesis"),
    ):
        if expected is not None and expected.lower() != observed:
            raise executor.refuse(
                "the executor ran on a chain whose %s identity is %s, and the run "
                "was declared against %s" % (role, observed, expected)
            )

    results = []
    for request in matrix["requests"]:
        name = request["case"]["name"]
        row = rows[name]
        row_started = time.monotonic()
        executor.send(request)
        response = executor.read("response for %s" % name)
        elapsed = time.monotonic() - row_started

        observed = response["observed_layer"]
        expected = row["expected_layer"]
        # A layer that is not a target verdict can never agree with an
        # expectation, whatever it happens to be called: the target was
        # never asked.
        verdict = "not-target-evidence"
        if observed in (
            "consensus_rejection_before_script",
            "script_path_rejection",
            "relay_policy_rejection",
            "accepted",
        ):
            verdict = "agrees" if observed == expected else "disagrees"

        results.append({
            "ordinal": row["id"]["ordinal"],
            "name": name,
            "defect": row["defect"],
            "expected_layer": expected,
            "observed_layer": observed,
            "observed_detail": response.get("observed_detail"),
            "verdict": verdict,
            "seconds": round(elapsed, 2),
            "transaction_bytes_hex": (
                bytes(response["transaction_bytes"]).hex()
                if response.get("transaction_bytes") else None
            ),
            "observed_value_commitments": [
                bytes(value).hex()
                for value in response.get("observed_value_commitments", [])
            ],
            "observed_asset_commitments": [
                bytes(value).hex()
                for value in response.get("observed_asset_commitments", [])
            ],
            # The openings the target reported for what it created. These
            # are what the commitment oracle predicts from; without them
            # the three-way comparison has nothing to meet at.
            "observed_openings": response.get("observed_openings", []),
        })
        print("  %-2d %-42s %-34s %s"
              % (row["id"]["ordinal"], name, observed, verdict), flush=True)

    # The orderly shutdown. The context manager guarantees the child is
    # gone either way, so this is the well-behaved path rather than the
    # only one.
    executor.close()

    deferred = [
        {
            "ordinal": row["id"]["ordinal"],
            "name": row["id"]["name"],
            "deferral": row["deferral"],
        }
        for row in matrix["rows"]
        if row.get("deferral") is not None
    ]

    record = {
        "handshake": handshake,
        "environment": {
            "chain_name": environment["chain_name"],
            "network_id": observed_network,
            "genesis_id": observed_genesis,
            "environment": environment["environment"],
        },
        "rows": results,
        "deferred": deferred,
        "wall_seconds": round(time.monotonic() - started, 1),
    }
    with open(arguments.report, "w") as handle:
        json.dump(record, handle, indent=2, sort_keys=True)
        handle.write("\n")
    print("wrote %s in %.1fs" % (arguments.report, record["wall_seconds"]), flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
