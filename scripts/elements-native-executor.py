#!/usr/bin/env python3
"""Target-native executor adapter for the Guide-9 conformance protocol.

This program speaks the newline-delimited JSON executor protocol owned by
`tripod-target-elements-conformance` on its stdin and stdout, and
answers each case by executing the fixture's exact script bytes through a real
Elements node: a disposable `elementsregtest` `elementsd` instance that this
process boots, drives over JSON-RPC, and destroys. Nothing here interprets a
script; the verdict is the node's.

No-arguments-from-harness contract
----------------------------------
The conformance harness spawns the selected executor with no arguments at all
and never reads its stderr (Guide-9 section 11.1, section 11.2). This script
therefore has to be reached through a launcher that supplies its
configuration; `elements-native-executor.sh` beside it is that launcher. The
configuration this script requires -- the `elementsd` and `elements-cli`
paths, and the `--framework` path -- is the executor's own, established
outside the first-party interface, and carries no credential of any kind
(ADR-015, and Guide-9 section 17.6, which bans `--rpc-user`, `--rpc-password`,
`--cookie`, `--token`, `--wallet`, and `--private-key`).

Everything this script writes on stdout is protocol data. Diagnostics go to
stderr, which the harness nulls.

Disposable datadir, and who owns the cookie
-------------------------------------------
The node runs in a fresh `mkdtemp` directory that this process creates, owns,
and deletes when the exchange ends -- on end of stdin, on error, and on
SIGINT/SIGTERM. `elementsd` writes its RPC cookie inside that directory and
`elements-cli` resolves it from the same `-datadir`; the cookie is never read
by this script, never placed on a command line, never logged, and never
crosses the protocol boundary. It is disposable test-network material under
ADR-015's rule with the heading "Disposable test-network material": generated
for a regtest chain whose assets have no value, confined to disposable test
state, destroyed with its environment, and never derived from or reused as
production key material. The internal taproot key is the published BIP-341
NUMS constant, which is a public test constant with no known discrete log.

The harness supervises this whole tree as one process group and stops it
gracefully first, precisely so that the cleanup below is reachable on a
timeout: the SIGTERM handler is what removes the datadir and the cookie in it
(Guide-10 section 5.8). Two residuals stay honest about their limits. A
forceful kill -- the harness's own second step, once the bounded cleanup
interval has passed, or any kill -9 from elsewhere -- cannot be handled, so the
node and the datadir are then reclaimed by the group signal and by the host's
temporary-directory policy rather than by this script. And a datadir left by a
hard-killed run is not adopted by a later one: each run mkdtemps its own.

Provenance
----------
Five roles, kept apart, because one revision string cannot answer five
questions and a schema offering one invites whichever answer is easiest to
obtain (ADR-018).

  adapter_name/version       this script, which builds the transactions
  framework_revision         the upstream functional-test framework it
                             builds them with
  node_name/node_version     the first word, and the whole first line, of
                             `elementsd --version`
  binary_reported_revision   the revision the binary itself embeds in that
                             line, and nothing else
  intended_executed_tip      what the operator says they meant to run
  upstream_base              the upstream base that tip derives from
  included_local_topics      the local topic branches folded into it

A checkout's `git rev-parse HEAD` is never substituted for
`binary_reported_revision`. A checkout identifies intended source; it says
nothing about a binary built at some earlier time from some other state, and
the fallback that once filled the field this way was a misattribution rather
than a best effort. A binary embedding no revision reports `None`, and the
report records that this run establishes no workspace provenance.

The last three are explicit operator declarations, passed as arguments. They
are recorded as declarations and are never derived from a working tree.

Environment
-----------
The adapter states the environment it actually ran on, after the node is up:

  genesis_id    `getblockhash 0` from the node that executed, byte order as
                the node prints it
  chain_name    the chain the node was configured to run
  network_id    the identity the deployment binding names for that chain,
                passed as `--network-id` and restated only once the genesis
                above has been read from the booted node

The harness compares all of it with the validated binding before any case
executes, so a run cannot inherit a caller's label for a chain the adapter
never looked at. A dishonest adapter can still lie; what this removes is the
honest one labelling one chain with another chain's identity.

Execution model
---------------
For each case the adapter builds two real transactions on the disposable
chain:

  funding   spends an anyone-can-spend output and creates a taproot output
            whose single-leaf tree carries the fixture's exact script bytes
            at the fixture's leaf version;
  spending  spends that taproot output through the script path, with the
            witness stack set to the fixture's exact initial stack followed
            by the leaf script and the control block.

The funding transaction is broadcast and confirmed with `generateblock`, so
that the spending transaction is never subject to mempool ancestor policy --
an ancestor-limit rejection would otherwise be indistinguishable from a
script rejection. The spending transaction is offered to `testmempoolaccept`.
A policy-only rejection is retried at consensus level by asking
`generateblock` to include the raw transaction in a block, because the
contract is about consensus behaviour and standardness is not consensus.

`final_stack` and `final_altstack` are always null: `elementsd` exposes no
interpreter stack, and synthesising one would be fabricating evidence.

What the fixture states, and what this adapter supplies
-------------------------------------------------------
Several fixture fields are optional, and an absent one is the fixture saying
that only the executor can know the value: an outpoint names a funding output
this adapter created, the program of the input under validation commits to
the very script the fixture carries, and which asset a development network
issues is the network's own fact. Those are supplied here. A field that *is*
stated is a requirement, and every requirement this adapter cannot meet is
refused by name -- never approximated, because an approximated context makes
the target answer a question the fixture did not ask.

Input amounts are the one supplied value with a rule attached. The declared
outputs fix the total the inputs must carry, and no fee or change output may
be added, since an added output would move every output index a fixture
states. So the inputs the fixture leaves unstated share exactly what the
stated ones leave, and the materialised transaction conserves value with the
outputs the fixture asked for and no others.

Resource observations
---------------------
`script_bytes` and `initial_stack_items` are restated from the fixture, so a
harness comparing them catches an executor that ran something else.
`transaction_weight` is read back from `decoderawtransaction`. The peak
depths, the largest element, and the validation budget are null: a validating
node exposes none of them, and a zero would report "not observed" as a
measurement.

Verdict mapping
---------------
`testmempoolaccept` reports an allowed result, or a `reject-reason`. Elements
formats a script failure as `mandatory-script-verify-flag-failed (TEXT)` at
consensus level and `non-mandatory-script-verify-flag (TEXT)` at policy
level, where TEXT is `ScriptErrorString` from `src/script/script_error.cpp`.
Anything else -- a missing input, a fee-rate refusal, a value-conservation
failure -- is this adapter failing to build a transaction, not the target
reaching a verdict, and is reported as `infrastructure_error`.

TEXT is mapped to the harness's `ObservedFailureClass` by the explicit table
below. A TEXT with no entry, and a TEXT whose entry is `None` because the
node does not distinguish the classes the harness names, produce a rejection
with a null `observed_failure`: an unmapped class is reported as unmapped and
never as a plausible neighbour.

  Elements ScriptErrorString                             ObservedFailureClass
  ---------------------------------------------------- ----------------------
  Script evaluated without error but finished with a     evaluated_false
    false/empty top stack element
  Operation not valid with the current stack size        stack_underflow
  Operation not valid with the current altstack size     stack_underflow
  Arithmetic opcodes expect 8 bytes operands             invalid_operand_width
  Opcode missing or not understood                       unknown_opcode
  Attempted to use a disabled opcode                     unknown_opcode
  Data push larger than necessary                        malformed_push
  Negative locktime                                      negative_timelock
  Locktime requirement not satisfied                     unsatisfied_timelock
  Invalid Sha256 context object read                     hash_context_load
  Invalid Sha256 context object write                    hash_context_write
  Introspection opcode used without correct evaluation   introspection_context_unavailable
    context
  Introspection index out of bounds                      introspection_index_out_of_range
  EC scalar mult verify fail                             invalid_curve_relation
  Invalid Schnorr signature                              invalid_signature
  Signature must be zero for failed CHECK(MULTI)SIG      invalid_signature
    operation
  Public key is neither compressed or uncompressed       invalid_public_key_encoding
  Too much signature validation relative to witness      validation_budget_exhausted
    weight
  Taproot version reserved for soft-fork upgrades        leaf_version_rejected
  OP_CHECKMULTISIG(VERIFY) is not available in           unsupported_execution_domain
    tapscript
  Stack size must be exactly one after execution         non_singleton_final_stack
  Push value size limit exceeded                         malformed_push
  unknown error                                          malformed_script_number
  Invalid Schnorr signature size                         invalid_signature
  Script failed an OP_CHECKSIGVERIFY operation           empty_signature
  Arithmetic opcode error                                fixed_width_conversion_refused
  Stack size limit exceeded                              unmapped
  Script is too big                                      unmapped
  Operation limit exceeded                               unmapped
  Script failed an OP_VERIFY operation                   unmapped
  Script failed an OP_EQUALVERIFY operation              unmapped
  Script failed an OP_NUMEQUALVERIFY operation           unmapped
  Invalid Schnorr signature hash type                    unmapped
  Invalid Taproot control block size                     unmapped

Four of those entries are worth naming, because they were settled by reading
the interpreter rather than by guessing at a string:

  "unknown error" is what a script-number exception becomes. Minimal encoding
  is enforced by a relay flag and not at consensus, so the only script-number
  exception a consensus run can raise is an operand wider than the primitive
  reads -- which is exactly the harness's malformed-script-number class. In
  the relay lane a nonminimal operand raises the same exception, and the same
  class covers it.

  "Invalid Schnorr signature size" and "Invalid Schnorr signature" are two
  target codes under one contract cause: the reviewed failure census admits
  the offered signature being refused, and does not split refusal by width
  from refusal by verification.

  "Script failed an OP_CHECKSIGVERIFY operation" is reachable in tapscript
  only for an empty signature -- a non-empty signature that does not verify
  errors earlier with its own code -- so the generic string is precise.

  "Arithmetic opcode error" is genuinely coarse and is reported as such. It
  is raised for a fixed-width conversion whose operand is the wrong width and
  for one whose result will not fit a script number, and the reviewed
  contract names those as separate causes. The harness has a class that means
  exactly "the target refused a fixed-width conversion without saying which",
  and that is what is reported; a fixture whose contract cause is one of the
  two admits it alongside.

What consensus does not see
---------------------------
Two of the table's entries were measured to be unreachable in the consensus
lane, and are kept so that a fixture stating them at the relay layer -- where
they are real -- is classified rather than silently unmapped:

  malformed_push          a nonminimal push is standardness in tapscript and
                          is accepted at consensus. The oversized form is a
                          consensus rule and does reach this class;
  leaf_version_rejected   an unrecognised leaf version is unconditionally
                          valid at consensus under BIP-341, and is accepted.

`unknown_opcode` is reachable: an OP_SUCCESSx byte makes the script succeed
at consensus, but a byte that is neither defined nor OP_SUCCESSx is rejected
there, and so is a push whose payload runs off the end of the script -- the
target answers a malformed push and an undefined byte with one code.

Measured against `v28.99.0-6f43e3ffe730`.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import signal
import socket
import subprocess
import sys
import tempfile
import time

COMMAND_NAME = "elements-native-executor"

# This adapter's own version, which is provenance for the transactions it
# builds and is not the node's version.
ADAPTER_VERSION = "2.0.0"

# The protocol revision this adapter speaks. It must match
# NATIVE_PROTOCOL_SCHEMA in the conformance package.
NATIVE_PROTOCOL_SCHEMA = 2

# The reviewed tapscript leaf version.
TAPSCRIPT_LEAF_VERSION = 0xC4

# The published BIP-341 NUMS point, used as the taproot internal key so that
# the key path is unspendable. A public constant, not key material.
NUMS_INTERNAL_KEY_HEX = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0"

# The anyone-can-spend program the disposable chain's free coins sit behind,
# and which this adapter reuses for its own change and block rewards.
ANYONE_CAN_SPEND_HEX = "51"

# Explicit-amount prefix for an Elements asset, value, or nonce field.
EXPLICIT_PREFIX = 0x01

# Satoshis committed to each case's taproot output, and the fee each
# adapter-built transaction pays. Both are far above any relay threshold on a
# chain configured with -minrelaytxfee=0.
CASE_FUNDING_SATOSHIS = 100_000
ADAPTER_FEE_SATOSHIS = 1_000

# The boundary between a lock time counted in blocks and one counted in
# seconds, and the bit layout of a sequence field's relative lock. All four
# are the target's own constants, restated here because the adapter has to
# mature a chain far enough for the declared transaction to be final -- and a
# transaction refused as non-final is not a script verdict.
LOCKTIME_HEIGHT_THRESHOLD = 500_000_000
SEQUENCE_DISABLE_FLAG = 0x8000_0000
SEQUENCE_TIME_MODE_FLAG = 0x0040_0000
SEQUENCE_AGE_MASK = 0x0000_FFFF
SEQUENCE_TIME_GRANULARITY_BITS = 9

# The median a relative time lock is compared against is taken over this many
# blocks, so advancing it means mining that many at the later time.
MEDIAN_TIME_BLOCKS = 11

# The descriptor the maturity blocks pay to: the same anyone-can-spend
# program the chain's free coins sit behind, which keeps every coin this
# adapter creates spendable by the adapter and by nobody who cares.
MINING_DESCRIPTOR = "raw(%s)" % ANYONE_CAN_SPEND_HEX

# Prefixes Elements puts in front of a script error in a rejection reason.
CONSENSUS_SCRIPT_PREFIX = "mandatory-script-verify-flag-failed ("
POLICY_SCRIPT_PREFIX = "non-mandatory-script-verify-flag ("

# The mapping documented in the module docstring. A key absent from this
# table, and a key whose value is None, both mean "this adapter did not
# classify the rejection".
FAILURE_CLASS_BY_SCRIPT_ERROR = {
    "Script evaluated without error but finished with a false/empty top stack element": "evaluated_false",
    "Operation not valid with the current stack size": "stack_underflow",
    "Operation not valid with the current altstack size": "stack_underflow",
    "Arithmetic opcodes expect 8 bytes operands": "invalid_operand_width",
    "Opcode missing or not understood": "unknown_opcode",
    "Attempted to use a disabled opcode": "unknown_opcode",
    "Data push larger than necessary": "malformed_push",
    "Negative locktime": "negative_timelock",
    "Locktime requirement not satisfied": "unsatisfied_timelock",
    "Invalid Sha256 context object read": "hash_context_load",
    "Invalid Sha256 context object write": "hash_context_write",
    "Introspection opcode used without correct evaluation context": "introspection_context_unavailable",
    "Introspection index out of bounds": "introspection_index_out_of_range",
    "EC scalar mult verify fail": "invalid_curve_relation",
    "Invalid Schnorr signature": "invalid_signature",
    "Signature must be zero for failed CHECK(MULTI)SIG operation": "invalid_signature",
    "Public key is neither compressed or uncompressed": "invalid_public_key_encoding",
    "Too much signature validation relative to witness weight": "validation_budget_exhausted",
    "Taproot version reserved for soft-fork upgrades": "leaf_version_rejected",
    "OP_CHECKMULTISIG(VERIFY) is not available in tapscript": "unsupported_execution_domain",
    # The final stack was not the single item the reviewed domain requires.
    "Stack size must be exactly one after execution": "non_singleton_final_stack",
    # A literal the target refuses outright, which is what the harness's
    # malformed-push class names.
    "Push value size limit exceeded": "malformed_push",
    # The generic code a script-number exception becomes. At consensus the
    # only script-number exception is an operand wider than the primitive
    # reads: minimal encoding is a relay rule and is not enforced there at
    # all, so this string has one meaning in the consensus lane and the
    # same one in the relay lane, where a nonminimal operand joins it.
    "unknown error": "malformed_script_number",
    # A signature of the wrong width, which the reviewed contract classes
    # with a signature that does not verify: both are the target refusing
    # the offered signature, and the contract's failure census draws no
    # line between them.
    "Invalid Schnorr signature size": "invalid_signature",
    # The verifying forms reach this only for an empty signature: a
    # non-empty signature that does not verify errors earlier with its own
    # code, so the generic verify-failure string is precise here.
    "Script failed an OP_CHECKSIGVERIFY operation": "empty_signature",
    # Deliberately coarse, and named as such. Elements answers a conversion
    # whose operand is the wrong width and one whose result will not fit a
    # script number with this one string, and the reviewed contract names
    # those as separate causes. Reporting either would name a cause this
    # adapter did not observe.
    "Arithmetic opcode error": "fixed_width_conversion_refused",
}


class AdapterError(Exception):
    """A case this adapter could not run, with a note naming why.

    Every instance becomes one `infrastructure_error` response. It is never
    a target verdict: the two are kept apart on purpose (Guide-9 section 11.6).
    """

    def __init__(self, note: str) -> None:
        super().__init__(note)
        self.note = note


class FatalAdapterError(Exception):
    """A condition that ends the exchange rather than one case."""


def log(message: str) -> None:
    """Writes one diagnostic line to stderr, which the harness nulls."""
    print("%s: %s" % (COMMAND_NAME, message), file=sys.stderr, flush=True)


# --------------------------------------------------------------------------
# Strict wire decoding
# --------------------------------------------------------------------------


def require_object(value: object, path: str) -> dict:
    """Returns value as an object, or names the path that was not one."""
    if not isinstance(value, dict):
        raise AdapterError("field is not an object: %s" % path)
    return value


def require_keys(value: dict, allowed: tuple, path: str) -> None:
    """Refuses any field this adapter does not know at `path`.

    A parallel wave may extend the fixture schema. An unknown field must be
    loud here, so that a drift between the harness's fixture type and this
    adapter's reading of it surfaces as a named infrastructure error rather
    than as a silently ignored difference.
    """
    for key in value:
        if key not in allowed:
            raise AdapterError("unknown field: %s.%s" % (path, key))
    for key in allowed:
        if key not in value:
            raise AdapterError("missing field: %s.%s" % (path, key))


def require_int(value: object, path: str) -> int:
    """Returns value as an integer, or names the path that was not one."""
    if isinstance(value, bool) or not isinstance(value, int):
        raise AdapterError("field is not an integer: %s" % path)
    return value


def require_bytes(value: object, path: str) -> bytes:
    """Returns a serde byte sequence -- a JSON array of octets -- as bytes."""
    if not isinstance(value, list):
        raise AdapterError("field is not a byte array: %s" % path)
    out = bytearray()
    for index, item in enumerate(value):
        octet = require_int(item, "%s[%d]" % (path, index))
        if octet < 0 or octet > 0xFF:
            raise AdapterError("byte out of range: %s[%d]" % (path, index))
        out.append(octet)
    return bytes(out)


def require_byte_vectors(value: object, path: str) -> list:
    """Returns a JSON array of byte arrays as a list of bytes."""
    if not isinstance(value, list):
        raise AdapterError("field is not an array of byte arrays: %s" % path)
    return [require_bytes(item, "%s[%d]" % (path, index)) for index, item in enumerate(value)]


def require_optional_bytes(value: object, path: str):
    """Returns an optional serde byte sequence: bytes, or None where absent.

    An absent field is the fixture saying the executor supplies the value.
    It is never read as an empty one: empty bytes are a statement, and
    silence is not.
    """
    if value is None:
        return None
    return require_bytes(value, path)


def require_optional_int(value: object, path: str):
    """Returns an optional integer, or None where the fixture states none."""
    if value is None:
        return None
    return require_int(value, path)


def require_string(value: object, path: str) -> str:
    """Returns value as a string, or names the path that was not one."""
    if not isinstance(value, str):
        raise AdapterError("field is not a string: %s" % path)
    return value


def parse_fixture(raw: object) -> dict:
    """Decodes one fixture strictly, naming any field this adapter cannot read.

    The `expected` member is checked for shape and then discarded. This
    adapter never reads what the fixture expects: an executor that consults
    the expectation is comparing a fixture with itself (Guide-9 section 10.6).

    Four members are validated and then discarded for a different reason:
    `leaf_version_status`, `script_source`, and `expected_resources` describe
    how the fixture came to be stated and what the harness will compare, none
    of which changes what a node is asked; and `enforcement_layer` is checked
    against the one layer this adapter answers at rather than ignored, since
    answering a relay question with a consensus verdict would be a different
    answer wearing the right shape.
    """
    fixture = require_object(raw, "fixture")
    require_keys(
        fixture,
        (
            "case",
            "target_contract_version",
            "network_id",
            "genesis_id",
            "execution_domain",
            "leaf_version",
            "leaf_version_status",
            "enforcement_layer",
            "script_source",
            "script",
            "initial_stack",
            "context",
            "expected",
            "expected_resources",
        ),
        "fixture",
    )
    check_expected_shape(fixture["expected"])
    check_resource_expectation_shape(fixture["expected_resources"])
    check_enumeration(
        fixture["leaf_version_status"],
        ("reviewed", "unreviewed"),
        "fixture.leaf_version_status",
    )
    check_enumeration(
        fixture["script_source"],
        ("typed_program", "deliberately_malformed"),
        "fixture.script_source",
    )
    layer = check_enumeration(
        fixture["enforcement_layer"],
        ("consensus", "relay_policy"),
        "fixture.enforcement_layer",
    )
    return {
        "enforcement_layer": layer,
        "execution_domain": fixture["execution_domain"],
        "leaf_version": require_int(fixture["leaf_version"], "fixture.leaf_version"),
        "script": require_bytes(fixture["script"], "fixture.script"),
        "initial_stack": require_byte_vectors(fixture["initial_stack"], "fixture.initial_stack"),
        "context": parse_context(fixture["context"]),
    }


def check_enumeration(raw: object, admitted: tuple, path: str) -> str:
    """Returns one of the admitted spellings, or names the one that was not."""
    text = require_string(raw, path)
    if text not in admitted:
        raise AdapterError("unknown value at %s: %s" % (path, text))
    return text


def check_expected_shape(raw: object) -> None:
    """Validates the expectation's shape without reading its content."""
    expected = require_object(raw, "fixture.expected")
    if len(expected) != 1:
        raise AdapterError("fixture.expected is not a single-variant enum")
    for name, body in expected.items():
        if name == "accept":
            require_keys(
                require_object(body, "fixture.expected.accept"),
                ("static_final_stack", "static_final_altstack"),
                "fixture.expected.accept",
            )
        elif name == "reject":
            require_keys(
                require_object(body, "fixture.expected.reject"),
                ("classes", "static_final_stack", "static_final_altstack"),
                "fixture.expected.reject",
            )
        else:
            raise AdapterError("unknown field: fixture.expected.%s" % name)


# The rows a fixture's resource expectation carries, in the order the
# harness declares them. Each is either an exact figure or the
# recorded-only marker; this adapter validates the shape and reports its
# own observations, and never reads the expectation into an answer.
RESOURCE_EXPECTATION_ROWS = (
    "script_bytes",
    "initial_stack_items",
    "peak_stack_items",
    "peak_altstack_items",
    "maximum_element_bytes",
    "validation_budget_used",
    "transaction_weight",
)


def check_resource_expectation_shape(raw: object) -> None:
    """Validates the resource expectation's shape, without reading a figure."""
    expected = require_object(raw, "fixture.expected_resources")
    require_keys(expected, RESOURCE_EXPECTATION_ROWS, "fixture.expected_resources")
    for row in RESOURCE_EXPECTATION_ROWS:
        path = "fixture.expected_resources.%s" % row
        entry = expected[row]
        if entry == "recorded_only":
            continue
        body = require_object(entry, path)
        if len(body) != 1 or "exact" not in body:
            raise AdapterError("unknown resource expectation at %s" % path)
        require_int(body["exact"], path + ".exact")


def parse_context(raw: object):
    """Decodes the generic transaction context, or None where there is none."""
    if raw is None:
        return None
    context = require_object(raw, "fixture.context")
    require_keys(
        context,
        ("version", "locktime", "current_input_index", "inputs", "outputs", "script_path"),
        "fixture.context",
    )
    inputs = context["inputs"]
    outputs = context["outputs"]
    if not isinstance(inputs, list) or not isinstance(outputs, list):
        raise AdapterError("fixture.context inputs and outputs must be arrays")
    return {
        "version": require_int(context["version"], "fixture.context.version"),
        "locktime": require_int(context["locktime"], "fixture.context.locktime"),
        "current_input_index": require_int(
            context["current_input_index"], "fixture.context.current_input_index"
        ),
        "inputs": [
            parse_input(item, "fixture.context.inputs[%d]" % index)
            for index, item in enumerate(inputs)
        ],
        "outputs": [
            parse_output(item, "fixture.context.outputs[%d]" % index)
            for index, item in enumerate(outputs)
        ],
        "script_path": parse_script_path(context["script_path"]),
    }


def parse_input(raw: object, path: str) -> dict:
    """Decodes one fixture input."""
    value = require_object(raw, path)
    require_keys(
        value,
        (
            "outpoint_txid",
            "outpoint_index",
            "spent_asset",
            "spent_value",
            "spent_program",
            "sequence",
            "issuance",
            "witness",
        ),
        path,
    )
    issuance = value["issuance"]
    if issuance is not None:
        require_keys(
            require_object(issuance, path + ".issuance"),
            ("asset_amount", "inflation_keys_amount", "entropy", "blinding_nonce"),
            path + ".issuance",
        )
    return {
        "outpoint_txid": require_optional_bytes(
            value["outpoint_txid"], path + ".outpoint_txid"
        ),
        "outpoint_index": require_optional_int(
            value["outpoint_index"], path + ".outpoint_index"
        ),
        "spent_asset": require_optional_bytes(value["spent_asset"], path + ".spent_asset"),
        "spent_value": require_optional_bytes(value["spent_value"], path + ".spent_value"),
        "spent_program": require_optional_bytes(
            value["spent_program"], path + ".spent_program"
        ),
        "sequence": require_int(value["sequence"], path + ".sequence"),
        "issuance": issuance,
        "witness": require_byte_vectors(value["witness"], path + ".witness"),
        "path": path,
    }


def parse_output(raw: object, path: str) -> dict:
    """Decodes one fixture output."""
    value = require_object(raw, path)
    require_keys(value, ("asset", "value", "nonce", "program"), path)
    return {
        "asset": require_optional_bytes(value["asset"], path + ".asset"),
        "value": require_bytes(value["value"], path + ".value"),
        "nonce": require_bytes(value["nonce"], path + ".nonce"),
        "program": require_optional_bytes(value["program"], path + ".program"),
        "path": path,
    }


def parse_script_path(raw: object) -> dict:
    """Decodes the declared script path."""
    value = require_object(raw, "fixture.context.script_path")
    require_keys(
        value, ("leaf_version", "script", "control"), "fixture.context.script_path"
    )
    return {
        "leaf_version": require_int(
            value["leaf_version"], "fixture.context.script_path.leaf_version"
        ),
        "script": require_bytes(value["script"], "fixture.context.script_path.script"),
        "control": require_optional_bytes(
            value["control"], "fixture.context.script_path.control"
        ),
    }


# --------------------------------------------------------------------------
# The disposable node
# --------------------------------------------------------------------------


def free_loopback_port() -> int:
    """Picks a loopback port the node can bind."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as probe:
        probe.bind(("127.0.0.1", 0))
        return probe.getsockname()[1]


class DisposableNode:
    """One `elementsd` regtest instance in a directory this process owns.

    The directory is created here, holds the node's cookie, and is deleted in
    `stop`. Nothing outside this class ever names it, and no credential
    leaves it: `elements-cli` is given the same `-datadir` and resolves the
    cookie itself.
    """

    def __init__(self, elementsd: str, elements_cli: str, chain: str, boot_timeout: float) -> None:
        self.elementsd = elementsd
        self.elements_cli = elements_cli
        self.chain = chain
        self.boot_timeout = boot_timeout
        self.datadir = tempfile.mkdtemp(prefix="tripod-native-executor-")
        self.rpc_port = free_loopback_port()
        self.process = None
        self.node_log = None

    def start(self) -> None:
        """Boots the node and waits for its RPC interface."""
        self.node_log = open(os.path.join(self.datadir, "node.out"), "wb")
        arguments = [
            self.elementsd,
            "-datadir=" + self.datadir,
            "-chain=" + self.chain,
            "-daemon=0",
            "-server=1",
            "-listen=0",
            "-discover=0",
            "-dnsseed=0",
            "-rpcbind=127.0.0.1",
            "-rpcallowip=127.0.0.1",
            "-rpcport=%d" % self.rpc_port,
            "-disablewallet=1",
            "-validatepegin=0",
            "-minrelaytxfee=0",
            "-blockmintxfee=0",
            "-initialfreecoins=2100000000000000",
            "-anyonecanspendaremine=1",
            "-printtoconsole=1",
        ]
        self.process = subprocess.Popen(
            arguments,
            stdin=subprocess.DEVNULL,
            stdout=self.node_log,
            stderr=subprocess.STDOUT,
        )
        deadline = time.monotonic() + self.boot_timeout
        while True:
            if self.process.poll() is not None:
                raise FatalAdapterError(
                    "the node exited during startup with status %d" % self.process.returncode
                )
            try:
                self.call("getblockchaininfo")
                return
            except AdapterError:
                if time.monotonic() >= deadline:
                    raise FatalAdapterError("the node did not answer RPC before the boot timeout")
                time.sleep(0.2)

    def call(self, method: str, *arguments: str):
        """Issues one JSON-RPC call through `elements-cli`."""
        command = [
            self.elements_cli,
            "-datadir=" + self.datadir,
            "-chain=" + self.chain,
            "-rpcport=%d" % self.rpc_port,
            method,
        ]
        command.extend(arguments)
        try:
            completed = subprocess.run(
                command, stdin=subprocess.DEVNULL, capture_output=True, text=True, check=False
            )
        except OSError as error:
            raise AdapterError("could not run the node client: %s" % error.strerror)
        if completed.returncode != 0:
            raise AdapterError("rpc %s failed: %s" % (method, one_line(completed.stderr)))
        text = completed.stdout.strip()
        if text == "":
            return None
        try:
            return json.loads(text)
        except json.JSONDecodeError:
            return text

    def stop(self) -> None:
        """Terminates the node and deletes the directory it owned."""
        if self.process is not None and self.process.poll() is None:
            try:
                self.call("stop")
            except AdapterError:
                pass
            try:
                self.process.wait(timeout=20)
            except subprocess.TimeoutExpired:
                self.process.terminate()
                try:
                    self.process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    self.process.kill()
                    self.process.wait(timeout=10)
        self.process = None
        if self.node_log is not None:
            self.node_log.close()
            self.node_log = None
        shutil.rmtree(self.datadir, ignore_errors=True)


def one_line(text: str) -> str:
    """Collapses a client's message to one line for a stderr diagnostic."""
    return " ".join(text.split())[:400]


# --------------------------------------------------------------------------
# Transaction construction
# --------------------------------------------------------------------------


def load_framework(framework: str):
    """Imports the upstream functional-test framework from an explicit path.

    The path is supplied as `--framework`; it is never guessed, never read
    from the environment, and never defaulted. The framework provides the
    reference taproot tweak, control-block, and Elements transaction
    serialisation used below, so that this adapter does not reimplement any
    of them.
    """
    candidate = os.path.abspath(framework)
    if os.path.basename(candidate) == "test_framework":
        candidate = os.path.dirname(candidate)
    if not os.path.isdir(os.path.join(candidate, "test_framework")):
        raise FatalAdapterError("the framework path holds no test_framework package")
    sys.path.insert(0, candidate)
    try:
        from test_framework import messages, script
    except ImportError as error:
        raise FatalAdapterError("the framework did not import: %s" % error)
    return candidate, messages, script


def txid_to_internal_int(txid_hex: str) -> int:
    """Converts a displayed txid into the integer an outpoint carries."""
    return int.from_bytes(bytes.fromhex(txid_hex), "big")


def explicit_amount(field: bytes, path: str) -> int:
    """Reads an explicit Elements value field, or refuses a confidential one."""
    if len(field) == 9 and field[0] == EXPLICIT_PREFIX:
        return int.from_bytes(field[1:], "big")
    raise AdapterError(
        "confidential or malformed value field, which this adapter cannot "
        "materialise deterministically: %s" % path
    )


class CaseExecutor:
    """Executes fixtures against one disposable node."""

    def __init__(self, node: DisposableNode, messages, script) -> None:
        self.node = node
        self.messages = messages
        self.script = script
        self.internal_key = bytes.fromhex(NUMS_INTERNAL_KEY_HEX)
        self.anyone_can_spend = bytes.fromhex(ANYONE_CAN_SPEND_HEX)
        self.change = None
        self.policy_asset_field = None
        self.mining_descriptor = None
        self.mock_time = 0

    def prime(self) -> None:
        """Locates the chain's free-coin output and confirms one block."""
        # The descriptor the maturity blocks are mined to, checksummed by the
        # node so that a hand-written checksum cannot drift from the script.
        self.mining_descriptor = self.node.call(
            "getdescriptorinfo", MINING_DESCRIPTOR
        )["descriptor"]
        self.node.call("generateblock", "raw(%s)" % ANYONE_CAN_SPEND_HEX, "[]")
        genesis_hash = self.node.call("getblockhash", "0")
        genesis = self.node.call("getblock", genesis_hash, "2")
        for transaction in genesis["tx"]:
            for output in transaction["vout"]:
                program = output["scriptPubKey"]["hex"]
                if program != ANYONE_CAN_SPEND_HEX:
                    continue
                amount = int(round(float(output["value"]) * 100_000_000))
                if amount <= 0:
                    continue
                asset = bytes.fromhex(output["asset"])[::-1]
                self.policy_asset_field = bytes([EXPLICIT_PREFIX]) + asset
                self.change = {
                    "txid": transaction["txid"],
                    "vout": output["n"],
                    "amount": amount,
                }
                self.mock_time = self.node.call("getblockchaininfo")["mediantime"]
                return
        raise FatalAdapterError("the chain carries no anyone-can-spend free-coin output")

    # -- chain maturity ---------------------------------------------------

    def mine(self, count: int) -> None:
        """Mines empty blocks to the adapter's own program."""
        if count > 0:
            self.node.call("generatetodescriptor", str(count), self.mining_descriptor)

    def mature(self, context: dict, funded: list) -> None:
        """Advances the chain until the declared transaction can be final.

        A fixture states a lock time and a sequence per input, and both are
        chain requirements as well as script data: a transaction whose lock
        time is above the tip, or whose input is younger than its own
        relative lock, is refused as non-final. That refusal is not a script
        verdict, and reporting it as one would make every introspection case
        look like a target rejection.

        So the chain is grown to meet what the fixture declared, rather than
        the declaration being trimmed to what an unmatured chain accepts. The
        rules restated here are the target's: an absolute lock time below the
        threshold counts blocks and above it counts seconds; a relative lock
        counts either, is measured from the block that funded the input, and
        is disabled outright by the flag bit.
        """
        required_height = 0
        required_time = 0

        # An absolute lock binds only while some input is non-final.
        if context["locktime"] != 0 and any(
            declared["sequence"] != 0xFFFF_FFFF for declared in context["inputs"]
        ):
            if context["locktime"] < LOCKTIME_HEIGHT_THRESHOLD:
                required_height = max(required_height, context["locktime"])
            else:
                required_time = max(required_time, context["locktime"] + 1)

        # Relative locks bind only from the version that introduced them.
        if context["version"] >= 2:
            for entry in funded:
                sequence = entry["sequence"]
                if sequence & SEQUENCE_DISABLE_FLAG:
                    continue
                age = sequence & SEQUENCE_AGE_MASK
                if sequence & SEQUENCE_TIME_MODE_FLAG:
                    required_time = max(
                        required_time,
                        entry["parent_median_time"] + (age << SEQUENCE_TIME_GRANULARITY_BITS) + 1,
                    )
                else:
                    required_height = max(required_height, entry["coin_height"] + age - 1)

        self.advance(required_height, required_time)

    def advance(self, required_height: int, required_time: int) -> None:
        """Grows the chain to one height and one median time."""
        info = self.node.call("getblockchaininfo")
        if required_time > info["mediantime"]:
            # The median is taken over a window, so the clock is moved once
            # and the window is refilled at the later time. The clock only
            # ever moves forward: a chain whose time went backwards would
            # refuse the very blocks meant to advance it.
            self.mock_time = max(self.mock_time + 1, required_time)
            self.node.call("setmocktime", str(self.mock_time))
            self.mine(MEDIAN_TIME_BLOCKS)
            info = self.node.call("getblockchaininfo")
            if info["mediantime"] < required_time:
                raise AdapterError(
                    "the chain's median time reached %d, and the declared "
                    "relative timelock needs %d"
                    % (info["mediantime"], required_time)
                )
        if required_height > info["blocks"]:
            self.mine(required_height - info["blocks"])

    # -- taproot ----------------------------------------------------------

    def taproot_for(self, script_bytes: bytes, leaf_version: int):
        """Builds the single-leaf taproot commitment to one script."""
        leaf_script = self.script.CScript(script_bytes)
        info = self.script.taproot_construct(
            self.internal_key, [("leaf", leaf_script, leaf_version)]
        )
        leaf = info.leaves["leaf"]
        control = bytes([leaf.version + info.negflag]) + info.internal_pubkey + leaf.merklebranch
        return info.scriptPubKey, bytes(leaf.script), control

    # -- primitives -------------------------------------------------------

    def output(self, amount: int, program: bytes, asset_field=None, nonce_field=None):
        """Builds one explicit Elements output."""
        messages = self.messages
        asset = asset_field if asset_field is not None else self.policy_asset_field
        out = messages.CTxOut(
            nValue=messages.CTxOutValue(amount),
            scriptPubKey=program,
            nAsset=messages.CTxOutAsset(asset),
        )
        if nonce_field is not None:
            out.nNonce = messages.CTxOutNonce(nonce_field)
        return out

    def raw_output(self, declared: dict):
        """Builds one output from declared field bytes, verbatim.

        A field the fixture leaves unstated is the executor's to supply, and
        only two are: the asset, because which asset a development network
        issues is the network's fact, and the program, because a fixture that
        named one would be describing a deployment. Both are supplied with
        the chain's own -- the policy asset and the anyone-can-spend program
        -- and everything the fixture does state is written unchanged.
        """
        messages = self.messages
        asset_field = declared["asset"]
        if asset_field is None:
            asset_field = self.policy_asset_field
        program = declared["program"]
        if program is None:
            program = self.anyone_can_spend
        out = messages.CTxOut()
        out.nAsset = messages.CTxOutAsset(asset_field)
        out.nValue = messages.CTxOutValue()
        out.nValue.vchCommitment = declared["value"]
        out.nNonce = messages.CTxOutNonce(declared["nonce"])
        out.scriptPubKey = program
        return out

    def fund(self, program: bytes, amount: int) -> str:
        """Creates and confirms one output paying `amount` to `program`.

        The funding transaction is confirmed by mining it directly, rather
        than broadcast and left in the mempool. `generateblock` mines exactly
        the transactions it is handed and pulls nothing from the mempool, so
        an unconfirmed parent would be invisible both to the consensus retry
        in `judge_at_consensus` and to a block built later; and an
        ever-growing mempool chain would eventually meet the ancestor limit,
        whose rejection is not a script verdict but would arrive looking like
        one.
        """
        messages = self.messages
        source = self.change
        if source is None:
            raise AdapterError("the adapter has no spendable change output")
        remainder = source["amount"] - amount - ADAPTER_FEE_SATOSHIS
        if remainder < 0:
            raise AdapterError("the adapter's change output cannot fund this case")
        transaction = messages.CTransaction()
        transaction.version = 2
        transaction.vin.append(
            messages.CTxIn(
                messages.COutPoint(txid_to_internal_int(source["txid"]), source["vout"]),
                nSequence=0xFFFFFFFE,
            )
        )
        transaction.vout.append(self.output(amount, program))
        transaction.vout.append(self.output(remainder, self.anyone_can_spend))
        transaction.vout.append(self.output(ADAPTER_FEE_SATOSHIS, b""))
        self.node.call(
            "generateblock",
            "raw(%s)" % ANYONE_CAN_SPEND_HEX,
            json.dumps([transaction.serialize().hex()]),
        )
        txid = transaction.rehash()
        self.change = {"txid": txid, "vout": 1, "amount": remainder}
        return txid

    # -- one case ---------------------------------------------------------

    def execute(self, fixture: dict) -> dict:
        """Runs one fixture and returns its verdict body."""
        if fixture["execution_domain"] != "tapscript":
            raise AdapterError(
                "unsupported execution domain: %s" % fixture["execution_domain"]
            )
        script_bytes = fixture["script"]
        leaf_version = fixture["leaf_version"]
        program, leaf_script, control = self.taproot_for(script_bytes, leaf_version)
        context = fixture["context"]
        if context is None:
            transaction = self.build_default_spend(program, leaf_script, control, fixture)
        else:
            transaction = self.build_context_spend(
                program, leaf_script, control, fixture, context
            )
        raw = transaction.serialize().hex()
        body = self.judge(raw, fixture["enforcement_layer"])
        body["transaction_weight"] = self.weight_of(raw)
        return body

    def weight_of(self, raw: str):
        """The materialised transaction's weight, as the node computes it.

        The one resource figure a validating node can actually be asked for.
        It is read back from the node rather than computed here so that the
        observation is the target's own accounting; when a node reports none,
        the answer is that it was not observed, never a number this adapter
        derived.
        """
        try:
            decoded = self.node.call("decoderawtransaction", raw)
        except AdapterError:
            return None
        if not isinstance(decoded, dict):
            return None
        weight = decoded.get("weight")
        if isinstance(weight, bool) or not isinstance(weight, int):
            return None
        return weight

    def build_default_spend(self, program, leaf_script, control, fixture):
        """Builds the minimal transaction that exercises the leaf."""
        messages = self.messages
        funding_txid = self.fund(program, CASE_FUNDING_SATOSHIS)
        transaction = messages.CTransaction()
        transaction.version = 2
        transaction.vin.append(
            messages.CTxIn(
                messages.COutPoint(txid_to_internal_int(funding_txid), 0), nSequence=0xFFFFFFFE
            )
        )
        transaction.vout.append(
            self.output(CASE_FUNDING_SATOSHIS - ADAPTER_FEE_SATOSHIS, self.anyone_can_spend)
        )
        transaction.vout.append(self.output(ADAPTER_FEE_SATOSHIS, b""))
        witness = messages.CTxInWitness()
        witness.scriptWitness.stack = list(fixture["initial_stack"]) + [leaf_script, control]
        transaction.wit.vtxinwit.append(witness)
        return transaction

    def build_context_spend(self, program, leaf_script, control, fixture, context):
        """Materialises a declared generic transaction context.

        Only what a disposable regtest chain can be made to say is
        materialised. A declaration this adapter cannot honour is refused by
        name rather than approximated: an approximated context would make the
        target answer a question the fixture did not ask.
        """
        messages = self.messages
        inputs = context["inputs"]
        index = context["current_input_index"]
        if not inputs:
            raise AdapterError("fixture.context declares no inputs")
        if index >= len(inputs):
            raise AdapterError(
                "fixture.context.current_input_index names no declared input"
            )
        declared_path = context["script_path"]
        # Exact equality, in both directions. Tolerating an empty declared
        # path let a context that stated no leaf script through, and the
        # adapter would then have chosen the leaf the fixture failed to
        # state. Canonical construction cannot produce that shape, but the
        # protocol is what stands between this adapter and a sender.
        if declared_path["script"] != fixture["script"]:
            raise AdapterError(
                "fixture.context.script_path.script disagrees with fixture.script"
            )
        if declared_path["control"] is not None:
            raise AdapterError(
                "fixture.context.script_path.control is stated, and this adapter "
                "derives the control block from the leaf it built"
            )

        # Every unmaterialisable declaration is named before any amount is
        # computed, so that a refusal reports the field it could not honour
        # rather than an arithmetic consequence of it.
        for declared in inputs:
            self.refuse_unmaterialisable_input(declared)
        amounts = self.input_amounts(context)
        funded = []
        for position, declared in enumerate(inputs):
            amount = amounts[position]
            spend_program = program if position == index else self.anyone_can_spend
            # Read before funding: the block that will carry the funding
            # output is the next one, so this tip is exactly the block a
            # relative time lock measures from.
            before = self.node.call("getblockchaininfo")
            funded.append(
                {
                    "declared": declared,
                    "sequence": declared["sequence"],
                    "txid": self.fund(spend_program, amount),
                    "coin_height": before["blocks"] + 1,
                    "parent_median_time": before["mediantime"],
                }
            )

        transaction = messages.CTransaction()
        transaction.version = context["version"]
        transaction.nLockTime = context["locktime"]
        for entry in funded:
            transaction.vin.append(
                messages.CTxIn(
                    messages.COutPoint(txid_to_internal_int(entry["txid"]), 0),
                    nSequence=entry["sequence"],
                )
            )
        for declared in context["outputs"]:
            transaction.vout.append(self.raw_output(declared))
        for position, entry in enumerate(funded):
            witness = messages.CTxInWitness()
            if position == index:
                witness.scriptWitness.stack = list(fixture["initial_stack"]) + [
                    leaf_script,
                    control,
                ]
            else:
                witness.scriptWitness.stack = list(entry["declared"]["witness"])
            transaction.wit.vtxinwit.append(witness)

        self.mature(context, funded)
        return transaction

    def input_amounts(self, context: dict) -> list:
        """How much each declared input carries.

        Value conservation is the chain's rule, not the fixture's. The
        declared outputs fix the total the inputs must carry, and an input
        whose value the fixture leaves unstated is one the executor funds --
        so the unstated inputs share whatever the stated ones leave. That is
        what lets the fixture's transaction be materialised exactly: an
        adapter that funded arbitrary amounts would have to add a change or
        fee output, which would move every output index the fixture states.
        """
        outputs = context["outputs"]
        inputs = context["inputs"]
        total = 0
        for declared in outputs:
            if declared["asset"] is not None and declared["asset"] != self.policy_asset_field:
                raise AdapterError(
                    "%s.asset names an asset this adapter cannot issue on the "
                    "disposable chain" % declared["path"]
                )
            total += explicit_amount(declared["value"], declared["path"] + ".value")

        stated = {}
        for position, declared in enumerate(inputs):
            if declared["spent_value"] is not None:
                stated[position] = explicit_amount(
                    declared["spent_value"], declared["path"] + ".spent_value"
                )
        unstated = [position for position in range(len(inputs)) if position not in stated]
        remainder = total - sum(stated.values())

        if not unstated:
            if remainder != 0:
                raise AdapterError(
                    "the declared inputs and outputs do not conserve value, and no "
                    "input is left unstated for the executor to fund"
                )
            return [stated[position] for position in range(len(inputs))]
        if remainder < len(unstated):
            raise AdapterError(
                "the declared outputs leave too little value for the %d input(s) "
                "the executor funds" % len(unstated)
            )
        share = remainder // len(unstated)
        amounts = dict(stated)
        for position in unstated[:-1]:
            amounts[position] = share
        amounts[unstated[-1]] = remainder - share * (len(unstated) - 1)
        return [amounts[position] for position in range(len(inputs))]

    def refuse_unmaterialisable_input(self, declared: dict) -> None:
        """Names every declared input field this adapter cannot honour.

        A field stated as absent is the fixture leaving the value to the
        executor, which is the case this adapter serves. A field stated with
        a value is a requirement, and every requirement it cannot meet is
        refused by name rather than approximated.
        """
        path = declared["path"]
        if declared["outpoint_txid"] is not None:
            raise AdapterError(
                "%s.outpoint_txid names a transaction this adapter cannot create; "
                "state it absent to have the executor supply the real outpoint" % path
            )
        if declared["outpoint_index"] is not None:
            raise AdapterError(
                "%s.outpoint_index is stated, and this adapter supplies the real "
                "outpoint" % path
            )
        if declared["issuance"] is not None:
            raise AdapterError("%s.issuance is not materialisable by this adapter" % path)
        if (
            declared["spent_asset"] is not None
            and declared["spent_asset"] != self.policy_asset_field
        ):
            raise AdapterError(
                "%s.spent_asset names an asset this adapter cannot issue on the "
                "disposable chain" % path
            )
        if declared["spent_program"] is not None:
            raise AdapterError(
                "%s.spent_program is stated, and this adapter supplies the programs "
                "it funds -- the taproot commitment to the fixture's leaf for the "
                "input under validation, and an anyone-can-spend program for the "
                "rest" % path
            )

    # -- verdict ----------------------------------------------------------

    def judge(self, raw: str, layer: str) -> dict:
        """Asks the node the question the fixture actually stated.

        The two layers are different rules and are asked in different ways.
        Consensus is what a block enforces, so a consensus fixture is judged
        by offering the transaction to `generateblock`; relay is what the
        mempool enforces, so a relay fixture is judged by
        `testmempoolaccept`. Answering either question with the other's
        machinery is how a harness reports an unrelayable but perfectly
        valid spend as an invalid one, or the reverse.

        The previous shape -- mempool first, retried as a block when the
        refusal was a policy script failure -- collapsed the two, and it
        could only retry a refusal that named a script. A policy rule that
        refuses a transaction *without* running the script, such as an
        oversized witness element, then looked like the adapter failing to
        build a transaction rather than like standardness.
        """
        if layer == "relay_policy":
            return self.judge_in_mempool(raw)
        return self.judge_in_block(raw)

    def judge_in_block(self, raw: str) -> dict:
        """Judges at consensus, which is what block validation enforces."""
        try:
            self.node.call("generateblock", "raw(%s)" % ANYONE_CAN_SPEND_HEX, json.dumps([raw]))
        except AdapterError as error:
            if CONSENSUS_SCRIPT_PREFIX in error.note:
                start = error.note.index(CONSENSUS_SCRIPT_PREFIX) + len(CONSENSUS_SCRIPT_PREFIX)
                end = error.note.index(")", start)
                return rejection(error.note[start:end])
            raise AdapterError(
                "the node refused the block for a reason that is not a script "
                "verdict: %s" % error.note
            )
        return {"verdict": "accepted", "observed_failure": None}

    def judge_in_mempool(self, raw: str) -> dict:
        """Judges at relay, which is what the mempool enforces."""
        answer = self.node.call("testmempoolaccept", json.dumps([raw]))
        if not isinstance(answer, list) or len(answer) != 1:
            raise AdapterError("the node did not answer testmempoolaccept with one result")
        result = answer[0]
        if result.get("allowed") is True:
            return {"verdict": "accepted", "observed_failure": None}
        reason = result.get("reject-reason")
        if not isinstance(reason, str):
            raise AdapterError("the node rejected without naming a reason")
        for prefix in (POLICY_SCRIPT_PREFIX, CONSENSUS_SCRIPT_PREFIX):
            if reason.startswith(prefix):
                return rejection(reason[len(prefix) : -1])
        raise AdapterError(
            "the node refused the transaction for a reason that is not a script "
            "verdict: %s" % reason
        )


def rejection(script_error: str) -> dict:
    """Builds a rejection body, naming any script error the table cannot map."""
    text = script_error.strip()
    failure = FAILURE_CLASS_BY_SCRIPT_ERROR.get(text)
    if failure is None:
        log("rejected, with a script error this adapter does not classify: %s" % text)
    return {"verdict": "rejected", "observed_failure": failure}


# --------------------------------------------------------------------------
# Protocol
# --------------------------------------------------------------------------


def write_message(value: dict) -> None:
    """Writes one NDJSON protocol line on stdout."""
    sys.stdout.write(json.dumps(value, separators=(",", ":")) + "\n")
    sys.stdout.flush()


def resources_for(fixture, weight) -> dict:
    """Reports the observations this executor can actually make.

    Two figures are the fixture's own and are restated exactly, so that a
    harness comparing them catches an executor that ran something other than
    what it was handed. The transaction's weight is a real observation, read
    back from the node.

    Everything else is null, and null is the point: a node exposes no
    interpreter stack, so the peak depths, the largest element, and the
    validation budget are not measured here. Reporting them as zero -- which
    is what this adapter had to do before the schema admitted absence --
    turned "not observed" into a measurement of nothing.
    """
    return {
        "script_bytes": len(fixture["script"]) if fixture else 0,
        "initial_stack_items": len(fixture["initial_stack"]) if fixture else 0,
        "peak_stack_items": None,
        "peak_altstack_items": None,
        "maximum_element_bytes": None,
        "validation_budget_used": None,
        "transaction_weight": weight,
    }


def node_provenance(elementsd: str):
    """Reads the node's own version line and the revision it embeds.

    The revision returned is the *binary's* own, or None. There is no
    checkout fallback: a working tree's HEAD identifies intended source, and
    reporting it here would attribute one program's identity to another.
    """
    completed = subprocess.run(
        [elementsd, "--version"], stdin=subprocess.DEVNULL, capture_output=True, text=True,
        check=False,
    )
    if completed.returncode != 0:
        raise FatalAdapterError("the node binary did not report a version")
    first_line = completed.stdout.strip().splitlines()[0].strip()
    name = first_line.split(" ")[0] if first_line else "unknown"
    match = re.search(r"-([0-9a-f]{7,40})(-dirty)?$", first_line)
    return name, first_line, (match.group(1) if match else None)


def framework_revision(framework_path):
    """The revision of the transaction framework this adapter builds with.

    Read from the framework checkout, which is exactly what this field is
    about: the framework is source this adapter imports and runs in-process,
    so its checkout state *is* what executed. That is not true of the node
    binary, whose field is left alone.
    """
    if framework_path is None:
        return None
    completed = subprocess.run(
        ["git", "-C", str(framework_path), "rev-parse", "HEAD"],
        stdin=subprocess.DEVNULL, capture_output=True, text=True, check=False,
    )
    return completed.stdout.strip() if completed.returncode == 0 else None


def identifier(text: str, role: str) -> bytes:
    """One 32-byte public identifier, from 64 hex digits."""
    try:
        value = bytes.fromhex(text)
    except ValueError:
        raise FatalAdapterError("the %s identifier is not hex" % role) from None
    if len(value) != 32:
        raise FatalAdapterError("the %s identifier must be 32 bytes" % role)
    return value


def serve(arguments) -> int:
    """Runs the whole exchange, and destroys the node whatever happens."""
    framework_path, messages, script = load_framework(arguments.framework)
    log("framework loaded from %s" % framework_path)
    name, version, revision = node_provenance(arguments.elementsd)
    network_id = identifier(arguments.network_id, "network")
    topics = [topic for topic in (arguments.included_local_topic or []) if topic]

    request_line = sys.stdin.readline()
    if request_line == "":
        raise FatalAdapterError("the harness closed stdin before the handshake")
    handshake = json.loads(request_line)
    if not isinstance(handshake, dict) or handshake.get("schema") != NATIVE_PROTOCOL_SCHEMA:
        raise FatalAdapterError("the harness spoke a protocol revision this adapter does not")

    node = DisposableNode(
        arguments.elementsd, arguments.elements_cli, arguments.chain, arguments.boot_timeout_seconds
    )
    closing = {"done": False}

    def close(*_ignored):
        if not closing["done"]:
            closing["done"] = True
            node.stop()

    for received in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
        try:
            signal.signal(received, lambda *_a: (close(), os._exit(1)))
        except (ValueError, OSError):
            pass

    try:
        started = time.monotonic()
        node.start()
        executor = CaseExecutor(node, messages, script)
        executor.prime()
        log("node ready in %.1fs" % (time.monotonic() - started))

        write_message(
            {
                "protocol_schema": NATIVE_PROTOCOL_SCHEMA,
                "adapter_name": COMMAND_NAME,
                "adapter_version": ADAPTER_VERSION,
                "framework_revision": framework_revision(framework_path),
                "node_name": name,
                "node_version": version,
                # The binary's own revision, or nothing. Never a checkout.
                "binary_reported_revision": revision,
                "intended_executed_tip": arguments.intended_executed_tip,
                "upstream_base": arguments.upstream_base,
                "included_local_topics": sorted(set(topics)),
                "supported_domains": ["tapscript"],
                "supported_leaf_versions": [TAPSCRIPT_LEAF_VERSION],
                "capabilities": [
                    "failure_class_reporting",
                    "resource_observation",
                    "transaction_context",
                ],
            }
        )

        # What this run actually executed on, read from the node that is
        # now up. The genesis is the node's answer; the chain name is what
        # it was configured to run; the network identity is the name the
        # binding gives that chain, restated only now that its genesis has
        # been observed.
        genesis = identifier(node.call("getblockhash", "0"), "genesis")
        write_message(
            {
                "schema": NATIVE_PROTOCOL_SCHEMA,
                "environment": "development",
                "chain_name": arguments.chain,
                "network_id": list(network_id),
                "genesis_id": list(genesis),
                "active_domains": ["tapscript"],
                "active_leaf_versions": [TAPSCRIPT_LEAF_VERSION],
            }
        )

        for line in sys.stdin:
            if line.strip() == "":
                continue
            answer_case(executor, line)
    finally:
        close()
    return 0


def answer_case(executor: CaseExecutor, line: str) -> None:
    """Answers exactly one execution request."""
    request = json.loads(line)
    if not isinstance(request, dict):
        raise FatalAdapterError("the harness sent a request that is not an object")
    for key in request:
        if key not in ("schema", "case", "fixture"):
            raise FatalAdapterError("the harness sent a request field named %s" % key)
    case = request.get("case")
    # The case identity is echoed verbatim, so that the harness correlates
    # against exactly what it sent. Without one there is nothing to answer,
    # and answering the wrong case would be worse than not answering.
    if not isinstance(case, dict):
        raise FatalAdapterError("the harness sent a request naming no case")
    fixture = None
    body = None
    try:
        if request.get("schema") != NATIVE_PROTOCOL_SCHEMA:
            raise AdapterError("the request carries a protocol revision this adapter does not")
        fixture = parse_fixture(request.get("fixture"))
        started = time.monotonic()
        body = executor.execute(fixture)
        log("case answered in %.2fs" % (time.monotonic() - started))
    except AdapterError as error:
        log("infrastructure error: %s" % error.note)
        body = {"verdict": "infrastructure_error", "observed_failure": None}
    write_message(
        {
            "schema": NATIVE_PROTOCOL_SCHEMA,
            "case": case,
            "verdict": body["verdict"],
            "final_stack": None,
            "final_altstack": None,
            "observed_failure": body["observed_failure"],
            "resources": resources_for(fixture, body.get("transaction_weight")),
        }
    )


def parse_arguments(argv):
    """Reads this adapter's own explicit, credential-free configuration."""
    parser = argparse.ArgumentParser(
        prog=COMMAND_NAME,
        description=(
            "Execute Guide-9 primitive fixtures through a disposable Elements "
            "regtest node, speaking the conformance executor protocol on stdio."
        ),
    )
    parser.add_argument("--elementsd", required=True, help="path to the elementsd binary")
    parser.add_argument("--elements-cli", required=True, help="path to the elements-cli binary")
    parser.add_argument(
        "--framework",
        required=True,
        help="path to the upstream functional-test framework (its test_framework "
        "package, or the directory holding it)",
    )
    parser.add_argument(
        "--network-id",
        required=True,
        help="the public development network identity of the chain this adapter "
        "boots, as 64 hex digits; restated as an observation only once the "
        "chain's genesis has been read from the node",
    )
    parser.add_argument(
        "--intended-executed-tip",
        default=None,
        help="the integration tip the operator intended to run (ADR-018); a "
        "declaration, never derived from a working tree",
    )
    parser.add_argument(
        "--upstream-base",
        default=None,
        help="the upstream base that tip derives from (ADR-018)",
    )
    parser.add_argument(
        "--included-local-topic",
        action="append",
        default=None,
        help="one local topic branch folded into that tip; repeatable (ADR-018)",
    )
    parser.add_argument("--chain", default="elementsregtest", help="the disposable chain name")
    parser.add_argument(
        "--boot-timeout-seconds", type=float, default=120.0, help="how long to wait for RPC"
    )
    return parser.parse_args(argv)


def main(argv) -> int:
    """Entry point."""
    arguments = parse_arguments(argv)
    try:
        return serve(arguments)
    except FatalAdapterError as error:
        log("fatal: %s" % error)
        return 1
    except json.JSONDecodeError:
        log("fatal: the harness sent a line that is not JSON")
        return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
