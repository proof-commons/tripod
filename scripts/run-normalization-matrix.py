#!/usr/bin/env python3
"""Drives the Guide-11 section 10.4 normalization matrix through the adapter.

This runner speaks the same newline-delimited JSON executor protocol the
conformance harness speaks, spawns the selected executor with NO arguments
exactly as the harness does, and records what each row observed. Every
request it sends comes from `emit-normalization-matrix`, so nothing here
re-derives the claim or the matrix.

It is deliberately thinner than it looks. It does NOT compare a claim with
a transaction, does NOT decide which layer refused a row, and does NOT
gate: three of these rows are answered by the report layer rather than by
the target, and that judgement belongs to the typed report in the package
that owns the claim. This runner is transport and a run record.

  --executor        the zero-argument executor to spawn
  --matrix          the JSON `emit-normalization-matrix` produced
  --report          where to write the run record

Nothing here accepts a credential, and none may be added.
"""
import argparse
import json
import subprocess
import sys
import time


def read_record(stream, what):
    line = stream.readline()
    if line == "":
        raise SystemExit("the executor closed its output before the %s" % what)
    return json.loads(line)


def main(argv):
    parser = argparse.ArgumentParser(prog="run-normalization-matrix")
    parser.add_argument("--executor", required=True)
    parser.add_argument("--matrix", required=True)
    parser.add_argument("--report", required=True)
    parser.add_argument("--expect-network-id", default=None)
    parser.add_argument("--expect-genesis-id", default=None)
    arguments = parser.parse_args(argv)

    with open(arguments.matrix) as handle:
        matrix = json.load(handle)

    started = time.monotonic()
    child = subprocess.Popen(
        [arguments.executor],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=None,
        text=True,
    )

    def send(value):
        child.stdin.write(json.dumps(value, separators=(",", ":")) + "\n")
        child.stdin.flush()

    send({"schema": matrix["schema"]})
    handshake = read_record(child.stdout, "handshake")
    environment = read_record(child.stdout, "environment observation")

    if "owner_authorized_normalization" not in handshake.get("capabilities", []):
        child.kill()
        raise SystemExit(
            "the executor advertised no owner-authorized-normalization "
            "capability; the launcher must pass --enable-wallet"
        )

    # The environment is checked before any row runs: a run must not label
    # one chain with another chain's identity, and a mismatch is not
    # something a later row can repair.
    def as_hex(values):
        return bytes(values).hex()

    observed_network = as_hex(environment["network_id"])
    observed_genesis = as_hex(environment["genesis_id"])
    for expected, observed, role in (
        (arguments.expect_network_id, observed_network, "network"),
        (arguments.expect_genesis_id, observed_genesis, "genesis"),
    ):
        if expected is not None and expected.lower() != observed:
            child.kill()
            raise SystemExit(
                "the executor ran on a chain whose %s identity is %s, and the run "
                "was declared against %s" % (role, observed, expected)
            )

    responses = []
    for request in matrix["requests"]:
        name = request["case"]["normalization"]
        row_started = time.monotonic()
        send(request)
        response = read_record(child.stdout, "response for %s" % name)
        elapsed = time.monotonic() - row_started
        responses.append(response)
        # Printed for a watching operator only. The layer named here is the
        # TARGET's, never the row's verdict: what refused a row is the
        # typed report's finding and is not computed in this process.
        print("  %-36s target-layer %-34s %.1fs"
              % (name, response["observed_layer"], elapsed), flush=True)

    child.stdin.close()
    child.wait(timeout=180)

    record = {
        "handshake": handshake,
        "environment": {
            "chain_name": environment["chain_name"],
            "network_id": observed_network,
            "genesis_id": observed_genesis,
            "environment": environment["environment"],
        },
        "responses": responses,
        "wall_seconds": round(time.monotonic() - started, 1),
    }
    with open(arguments.report, "w") as handle:
        json.dump(record, handle, indent=2, sort_keys=True)
        handle.write("\n")
    print("wrote %s in %.1fs" % (arguments.report, record["wall_seconds"]), flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
