#!/bin/sh
# Adapter smoke test for the target-native Elements executor.
#
# Boots scripts/elements-native-executor.sh through its documented
# environment boundary, sends a handshake and four hand-written execution
# requests, and asserts the typed responses. This is a test of the adapter,
# not the conformance matrix: it proves the adapter boots a real elementsd,
# builds a real taproot script-path spend, and reports what the node said.
#
# It needs the same configuration the adapter needs (see
# elements-native-executor.sh). Without it there is nothing to execute
# against, and the test exits 77 -- the conventional "skipped" status -- so
# that a missing node is never mistaken for a passing adapter.
#
# Usage: test-elements-native-executor.sh <repository-root>
set -eu

repository_root="${1:-.}"
repository_root="$(cd "$repository_root" && pwd)"
launcher="$repository_root/scripts/elements-native-executor.sh"

if [ ! -x "$launcher" ]; then
  echo "test-elements-native-executor.sh: no executable launcher at $launcher" >&2
  exit 1
fi

for name in ELEMENTS_NATIVE_EXECUTOR_ELEMENTSD \
            ELEMENTS_NATIVE_EXECUTOR_ELEMENTS_CLI \
            ELEMENTS_NATIVE_EXECUTOR_FRAMEWORK; do
  eval "value=\${$name:-}"
  if [ -z "$value" ]; then
    echo "==> skipped: $name is unset, so there is no node to execute against" >&2
    exit 77
  fi
done

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT INT TERM

requests="$work/requests.ndjson"
responses="$work/responses.ndjson"

echo "==> writing the handshake and four execution requests" >&2
python3 - "$requests" <<'WRITE_REQUESTS'
import json
import sys

SCHEMA = 1
TAPSCRIPT_LEAF_VERSION = 0xC4
ZERO_32 = [0] * 32


def case(ordinal):
    return {"group": "instruction_encoding", "opcode": None, "ordinal": ordinal}


def fixture(ordinal, script, stack, expected, context=None):
    return {
        "case": case(ordinal),
        "target_contract_version": 1,
        "network_id": ZERO_32,
        "genesis_id": ZERO_32,
        "execution_domain": "tapscript",
        "leaf_version": TAPSCRIPT_LEAF_VERSION,
        "script": list(script),
        "initial_stack": [list(item) for item in stack],
        "context": context,
        "expected": expected,
    }


ACCEPTS = {"accept": {"final_stack": [[1]], "final_altstack": []}}


def rejects(name):
    return {"reject": {"class": name}}


# A context whose current input names a transaction that does not exist on
# any chain the adapter can build. The adapter must refuse it by name rather
# than approximate it, and refusal is an infrastructure error, never a
# verdict.
UNMATERIALISABLE_CONTEXT = {
    "version": 2,
    "locktime": 0,
    "current_input_index": 0,
    "inputs": [
        {
            "outpoint_txid": [7] * 32,
            "outpoint_index": 0,
            "spent_asset": [1] + ZERO_32,
            "spent_value": [1] + [0] * 8,
            "spent_program": [],
            "sequence": 4294967294,
            "issuance": None,
            "witness": [],
        }
    ],
    "outputs": [],
    "script_path": {"leaf_version": TAPSCRIPT_LEAF_VERSION, "script": [], "control": []},
}

# The elementsregtest policy asset, as an explicit asset field: the 0x01
# prefix followed by the identifier in internal byte order. A public chain
# constant, not fixture-specific material.
POLICY_ASSET_FIELD = [1] + list(
    bytes.fromhex("b2e15d0d7a0c94e4e2ce0fe6e8691b9e451377f6e46e8045a86f7c4b5d4f0f23")[::-1]
)


def explicit_value(satoshis):
    return [1] + list(satoshis.to_bytes(8, "big"))


# A context the adapter can materialise: one input whose all-zero outpoint
# asks the executor to supply the real one, and two explicit outputs that
# conserve the input's value.
MATERIALISABLE_CONTEXT = {
    "version": 2,
    "locktime": 0,
    "current_input_index": 0,
    "inputs": [
        {
            "outpoint_txid": ZERO_32,
            "outpoint_index": 0,
            "spent_asset": POLICY_ASSET_FIELD,
            "spent_value": explicit_value(100000),
            "spent_program": [],
            "sequence": 4294967294,
            "issuance": None,
            "witness": [],
        }
    ],
    "outputs": [
        {
            "asset": POLICY_ASSET_FIELD,
            "value": explicit_value(99000),
            "nonce": [0],
            "program": [0x51],
        },
        {
            "asset": POLICY_ASSET_FIELD,
            "value": explicit_value(1000),
            "nonce": [0],
            "program": [],
        },
    ],
    "script_path": {"leaf_version": TAPSCRIPT_LEAF_VERSION, "script": [], "control": []},
}

REQUESTS = [
    # OP_TRUE: one push of true, which leaves a true top of stack.
    fixture(0, b"\x51", [], ACCEPTS),
    # OP_0: pushes an empty element, which is false.
    fixture(1, b"\x00", [], rejects("evaluated_false")),
    # OP_ADD with nothing beneath it.
    fixture(2, b"\x93", [], rejects("stack_underflow")),
    # A context the adapter cannot materialise.
    fixture(3, b"\x51", [], ACCEPTS, UNMATERIALISABLE_CONTEXT),
    # A context the adapter can materialise, spent through the same leaf.
    fixture(4, b"\x51", [], ACCEPTS, MATERIALISABLE_CONTEXT),
]

with open(sys.argv[1], "w", encoding="utf-8") as stream:
    stream.write(json.dumps({"schema": SCHEMA}) + "\n")
    for entry in REQUESTS:
        stream.write(
            json.dumps({"schema": SCHEMA, "case": entry["case"], "fixture": entry}) + "\n"
        )
WRITE_REQUESTS

echo "==> running the adapter against a real elementsd" >&2
"$launcher" < "$requests" > "$responses"

echo "==> asserting the typed responses" >&2
python3 - "$responses" <<'CHECK_RESPONSES'
import json
import sys

EXPECTED = [
    (0, "accepted", None),
    (1, "rejected", "evaluated_false"),
    (2, "rejected", "stack_underflow"),
    (3, "infrastructure_error", None),
    (4, "accepted", None),
]

failures = []
with open(sys.argv[1], encoding="utf-8") as stream:
    lines = [line for line in stream if line.strip()]

if len(lines) != len(EXPECTED) + 1:
    print("FAIL: expected %d lines, read %d" % (len(EXPECTED) + 1, len(lines)))
    for line in lines:
        print("  " + line.rstrip())
    sys.exit(1)

handshake = json.loads(lines[0])
print("handshake: " + json.dumps(handshake, sort_keys=True))
for field, want in (
    ("protocol_schema", 1),
    ("supported_domains", ["tapscript"]),
    ("supported_leaf_versions", [196]),
):
    if handshake.get(field) != want:
        failures.append("handshake.%s is %r, wanted %r" % (field, handshake.get(field), want))
if "elements" not in str(handshake.get("implementation_name", "")).lower():
    failures.append("handshake.implementation_name does not name Elements")
if not handshake.get("upstream_revision"):
    failures.append("handshake.upstream_revision is empty")

for line, (ordinal, verdict, failure) in zip(lines[1:], EXPECTED):
    answer = json.loads(line)
    print("case %d: " % ordinal + json.dumps(answer, sort_keys=True))
    if answer.get("case", {}).get("ordinal") != ordinal:
        failures.append("case %d answered out of order" % ordinal)
    if answer.get("verdict") != verdict:
        failures.append("case %d verdict is %r, wanted %r" % (ordinal, answer.get("verdict"), verdict))
    if answer.get("observed_failure") != failure:
        failures.append(
            "case %d failure is %r, wanted %r" % (ordinal, answer.get("observed_failure"), failure)
        )
    if answer.get("final_stack") is not None or answer.get("final_altstack") is not None:
        failures.append("case %d reported a stack the node cannot expose" % ordinal)

for failure in failures:
    print("FAIL: " + failure)
sys.exit(1 if failures else 0)
CHECK_RESPONSES

echo "==> adapter smoke test passed" >&2
