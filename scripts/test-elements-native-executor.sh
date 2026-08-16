#!/bin/sh
# Adapter smoke test for the target-native Elements executor.
#
# Boots scripts/elements-native-executor.sh through its documented
# environment boundary, sends a handshake and a series of hand-written
# execution requests, and asserts the typed responses. This is a test of the
# adapter, not the conformance matrix: it proves the adapter boots a real
# elementsd, builds a real taproot script-path spend, and reports what the
# node said.
#
# Both request records are exercised. The primitive ones cover the fixture
# shapes the adapter has always answered; the compound-prototype ones cover
# the two transaction shapes a stated construction can require -- one
# successor output for the constructor relation, and no output of any role
# for the wide-floor relation -- with one accepting and one refusing case
# each. Their scripts are trivial, because what is under test is the
# materialisation and the verdict path rather than either prototype's own
# program.
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

# The adapter states an observed environment, and the network identity of
# the chain it boots is explicit configuration rather than something a node
# can be asked for. This is an arbitrary nonzero development value: it names
# nothing, authorizes nothing, and is not secret material.
ELEMENTS_NATIVE_EXECUTOR_NETWORK_ID="${ELEMENTS_NATIVE_EXECUTOR_NETWORK_ID:-\
1111111111111111111111111111111111111111111111111111111111111111}"
export ELEMENTS_NATIVE_EXECUTOR_NETWORK_ID

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT INT TERM

requests="$work/requests.ndjson"
responses="$work/responses.ndjson"

echo "==> writing the handshake, the primitive and the compound requests" >&2
python3 - "$requests" "$ELEMENTS_NATIVE_EXECUTOR_FRAMEWORK" <<'WRITE_REQUESTS'
import json
import os
import sys

SCHEMA = 2
TAPSCRIPT_LEAF_VERSION = 0xC4
ZERO_32 = [0] * 32


def case(ordinal):
    return {"group": "instruction_encoding", "opcode": None, "ordinal": ordinal}


def resource_expectation(script, stack):
    return {
        "script_bytes": {"exact": len(script)},
        "initial_stack_items": {"exact": len(stack)},
        "peak_stack_items": "recorded_only",
        "peak_altstack_items": "recorded_only",
        "maximum_element_bytes": "recorded_only",
        "validation_budget_used": "recorded_only",
        "transaction_weight": "recorded_only",
    }


def fixture(ordinal, script, stack, expected, context=None):
    if context is not None:
        # Exact equality, both directions: the adapter no longer tolerates
        # a context that leaves its leaf script unstated.
        context = dict(context)
        context["script_path"] = dict(context["script_path"], script=list(script))
    return {
        "case": case(ordinal),
        "target_contract_version": 1,
        "network_id": ZERO_32,
        "genesis_id": ZERO_32,
        "execution_domain": "tapscript",
        "leaf_version": TAPSCRIPT_LEAF_VERSION,
        "leaf_version_status": "reviewed",
        "enforcement_layer": "consensus",
        "script_source": "typed_program",
        "script": list(script),
        "initial_stack": [list(item) for item in stack],
        "context": context,
        "expected": expected,
        "expected_resources": resource_expectation(script, stack),
    }


ACCEPTS = {"accept": {"static_final_stack": [[1]], "static_final_altstack": []}}


def rejects(name):
    return {
        "reject": {
            "classes": [name],
            "static_final_stack": None,
            "static_final_altstack": None,
        }
    }


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
            "outpoint_index": None,
            "spent_asset": None,
            "spent_value": None,
            "spent_program": None,
            "sequence": 4294967294,
            "issuance": None,
            "witness": [],
        }
    ],
    "outputs": [],
    "script_path": {"leaf_version": TAPSCRIPT_LEAF_VERSION, "script": [], "control": None},
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
            "outpoint_txid": None,
            "outpoint_index": None,
            "spent_asset": POLICY_ASSET_FIELD,
            "spent_value": explicit_value(100000),
            "spent_program": None,
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
    "script_path": {"leaf_version": TAPSCRIPT_LEAF_VERSION, "script": [], "control": None},
}

# A context whose outputs the executor supplies entirely: no asset and no
# program are stated, and no input value is, so the executor funds the one
# input with exactly what the outputs demand. This is the shape the canonical
# census uses, and it is the shape that proves no fee or change output is
# added behind the fixture's back.
SUPPLIED_CONTEXT = {
    "version": 2,
    "locktime": 0,
    "current_input_index": 0,
    "inputs": [
        {
            "outpoint_txid": None,
            "outpoint_index": None,
            "spent_asset": None,
            "spent_value": None,
            "spent_program": None,
            "sequence": 4294967294,
            "issuance": None,
            "witness": [],
        }
    ],
    "outputs": [
        {
            "asset": None,
            "value": explicit_value(70000),
            "nonce": [0],
            "program": None,
        },
        {
            "asset": None,
            "value": explicit_value(30000),
            "nonce": [0],
            "program": None,
        },
    ],
    "script_path": {"leaf_version": TAPSCRIPT_LEAF_VERSION, "script": [], "control": None},
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
    # A context whose asset, programs, and input amount the executor supplies.
    fixture(5, b"\x51", [], ACCEPTS, SUPPLIED_CONTEXT),
]


# -- the compound-prototype requests ----------------------------------------
#
# A compound fixture states a complete taproot construction, and every value
# in it must be the one the internal key and the tree actually determine --
# the adapter refuses anything else rather than approximating it. So the
# stated values are computed here from the same upstream helper the adapter
# builds with, which is what makes this a test of the adapter's
# materialisation rather than of a hand-copied constant.
#
# The scripts are trivial on purpose. This is an adapter smoke test: it
# proves the two transaction shapes are built and judged, and it establishes
# nothing about either prototype's own program, whose rows are the
# conformance matrix's work.

framework = os.path.abspath(sys.argv[2])
if os.path.basename(framework) == "test_framework":
    framework = os.path.dirname(framework)
sys.path.insert(0, framework)
from test_framework import script as framework_script  # noqa: E402

# The published BIP-341 NUMS point, which is what both matrices state as
# their internal key. A public test constant with no known discrete log.
NUMS_INTERNAL_KEY = bytes.fromhex(
    "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0"
)

ACCEPTING_SCRIPT = b"\x51"
REJECTING_SCRIPT = b"\x00"
# A second leaf, so that a two-leaf tree is a real branch with a real
# control path rather than a single leaf wearing the word "tree".
SIBLING_SCRIPT = b"\x6a"


def leaf(program):
    return {"leaf": {"version": TAPSCRIPT_LEAF_VERSION, "script": list(program)}}


def branch(left, right):
    return {"branch": {"left": left, "right": right}}


def constructed(items, executing):
    """The program and control block the stated key and tree determine."""
    info = framework_script.taproot_construct(NUMS_INTERNAL_KEY, items)
    built = info.leaves[executing]
    control = (
        bytes([built.version + info.negflag])
        + bytes(info.internal_pubkey)
        + bytes(built.merklebranch)
    )
    return bytes(info.scriptPubKey), control


def item(name, program):
    return (name, framework_script.CScript(program), TAPSCRIPT_LEAF_VERSION)


def compound(relation, name, program, tree, items, outputs, expected):
    built_program, control = constructed(items, "executing")
    return {
        "case": {"relation": relation, "name": name},
        "claims": ["wide_floor_exact_division_observed"]
        if relation == "wide_floor_relation"
        else ["successor_program_observed"],
        "target_contract_version": 1,
        "script": list(program),
        "initial_stack": [],
        "construction": {
            "internal_key": list(NUMS_INTERNAL_KEY),
            "tree": tree,
            "executing_leaf": leaf(program),
            "control": list(control),
            "predecessor_program": list(built_program),
            "outputs": outputs,
        },
        "expected": expected,
        "expected_resources": resource_expectation(program, []),
    }


def wide_floor_shaped(name, program, expected):
    """A relation requiring no output of any role: one leaf, no successor."""
    return compound(
        "wide_floor_relation",
        name,
        program,
        leaf(program),
        [item("executing", program)],
        [],
        expected,
    )


def constructor_shaped(name, program, expected):
    """A relation requiring one successor output, spent from a two-leaf tree."""
    successor, _control = constructed([item("only", ACCEPTING_SCRIPT)], "only")
    return compound(
        "metadata_constructor_continuity",
        name,
        program,
        branch(leaf(program), leaf(SIBLING_SCRIPT)),
        [item("executing", program), item("sibling", SIBLING_SCRIPT)],
        [{"role": "successor", "program": list(successor)}],
        expected,
    )


COMPOUND_REQUESTS = [
    wide_floor_shaped("adapter_smoke_accepting", ACCEPTING_SCRIPT, "accepted"),
    wide_floor_shaped("adapter_smoke_rejecting", REJECTING_SCRIPT, "rejected"),
    constructor_shaped("adapter_smoke_accepting", ACCEPTING_SCRIPT, "accepted"),
    constructor_shaped("adapter_smoke_rejecting", REJECTING_SCRIPT, "rejected"),
]

with open(sys.argv[1], "w", encoding="utf-8") as stream:
    stream.write(json.dumps({"schema": SCHEMA}) + "\n")
    for entry in REQUESTS:
        stream.write(
            json.dumps({"schema": SCHEMA, "case": entry["case"], "fixture": entry}) + "\n"
        )
    for entry in COMPOUND_REQUESTS:
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
    (5, "accepted", None),
    # The two compound shapes, one accepting and one refusing each. A
    # compound case is named by a relation and a name rather than by an
    # ordinal, which is the field the adapter routes on.
    (("wide_floor_relation", "adapter_smoke_accepting"), "accepted", None),
    (
        ("wide_floor_relation", "adapter_smoke_rejecting"),
        "rejected",
        "evaluated_false",
    ),
    (
        ("metadata_constructor_continuity", "adapter_smoke_accepting"),
        "accepted",
        None,
    ),
    (
        ("metadata_constructor_continuity", "adapter_smoke_rejecting"),
        "rejected",
        "evaluated_false",
    ),
]

# Every figure a validating node cannot see. The schema admits absence, and
# an adapter reporting zero for one of these would be reporting a
# measurement it never made.
UNOBSERVED_RESOURCES = (
    "peak_stack_items",
    "peak_altstack_items",
    "maximum_element_bytes",
    "validation_budget_used",
)

failures = []
with open(sys.argv[1], encoding="utf-8") as stream:
    lines = [line for line in stream if line.strip()]

# The handshake and the environment observation both precede the answers.
if len(lines) != len(EXPECTED) + 2:
    print("FAIL: expected %d lines, read %d" % (len(EXPECTED) + 2, len(lines)))
    for line in lines:
        print("  " + line.rstrip())
    sys.exit(1)

handshake = json.loads(lines[0])
environment = json.loads(lines[1])
print("environment: " + json.dumps(environment, sort_keys=True))
for field, want in (
    ("schema", 2),
    ("environment", "development"),
    ("active_domains", ["tapscript"]),
    ("active_leaf_versions", [196]),
):
    if environment.get(field) != want:
        failures.append(
            "environment.%s is %r, wanted %r" % (field, environment.get(field), want)
        )
# The genesis is the node's own answer, so the test asserts its shape rather
# than a value: a fixed expectation here would be this script declaring what
# the chain is instead of reading it.
genesis = environment.get("genesis_id")
if not isinstance(genesis, list) or len(genesis) != 32 or not any(genesis):
    failures.append("environment.genesis_id is not an observed 32-byte identity")
if not environment.get("chain_name"):
    failures.append("environment.chain_name is empty")
print("handshake: " + json.dumps(handshake, sort_keys=True))
for field, want in (
    ("protocol_schema", 2),
    ("supported_domains", ["tapscript"]),
    ("supported_leaf_versions", [196]),
    # Compared as a set: the harness reads these into one, and asserting an
    # order here would fail an adapter that listed the same capabilities
    # differently. Both tree-bearing capabilities are required, because the
    # compound cases below are exactly the work they gate -- an adapter
    # whose framework offers no taproot_construct cannot run them, and
    # saying so here is more use than the requests being refused one by one.
    (
        "capabilities",
        sorted(
            [
                "compound_prototype_fixtures",
                "failure_class_reporting",
                "resource_observation",
                "transaction_context",
                "tree_materialization",
            ]
        ),
    ),
):
    observed = handshake.get(field)
    if field == "capabilities" and isinstance(observed, list):
        observed = sorted(observed)
    if observed != want:
        failures.append("handshake.%s is %r, wanted %r" % (field, observed, want))
if "elements" not in str(handshake.get("node_name", "")).lower():
    failures.append("handshake.node_name does not name Elements")
if not handshake.get("adapter_name"):
    failures.append("handshake.adapter_name is empty")
# The binary's own revision, and nothing standing in for it. A binary that
# embeds none reports null, and the report then records that this run
# establishes no workspace provenance rather than borrowing a checkout's.
revision = handshake.get("binary_reported_revision", "missing")
if revision != "missing" and revision is not None and not str(revision).strip():
    failures.append("handshake.binary_reported_revision is blank rather than absent")

def answered_case(answer):
    """The identity in one answer, in the shape its request stated it."""
    case = answer.get("case")
    if not isinstance(case, dict):
        return None
    if "relation" in case:
        return (case.get("relation"), case.get("name"))
    return case.get("ordinal")


for line, (identity, verdict, failure) in zip(lines[2:], EXPECTED):
    answer = json.loads(line)
    label = "case %s" % (identity,)
    print(label + ": " + json.dumps(answer, sort_keys=True))
    if answered_case(answer) != identity:
        failures.append("%s answered out of order" % label)
    if answer.get("verdict") != verdict:
        failures.append("%s verdict is %r, wanted %r" % (label, answer.get("verdict"), verdict))
    if answer.get("observed_failure") != failure:
        failures.append(
            "%s failure is %r, wanted %r" % (label, answer.get("observed_failure"), failure)
        )
    if answer.get("final_stack") is not None or answer.get("final_altstack") is not None:
        failures.append("%s reported a stack the node cannot expose" % label)
    resources = answer.get("resources")
    if not isinstance(resources, dict):
        failures.append("%s reported no resource observation" % label)
        continue
    for row in UNOBSERVED_RESOURCES:
        if resources.get(row) is not None:
            failures.append(
                "%s reported %s as %r, which the node does not expose"
                % (label, row, resources.get(row))
            )
    weight = resources.get("transaction_weight")
    if verdict == "infrastructure_error":
        if weight is not None:
            failures.append("%s reported a weight for a transaction it never built" % label)
    elif not isinstance(weight, int) or isinstance(weight, bool) or weight <= 0:
        failures.append("%s reported no transaction weight" % label)

for failure in failures:
    print("FAIL: " + failure)
sys.exit(1 if failures else 0)
CHECK_RESPONSES

echo "==> adapter smoke test passed" >&2
