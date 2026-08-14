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

Provenance
----------
`implementation_name` is the first word of `elementsd --version`, and
`implementation_version` is that first line verbatim, so the report carries
the binary's own version string including any `-dirty` marker. Elements
embeds the revision it was built from in that string; when it does,
`upstream_revision` is that embedded revision, because that is the revision
that actually executed. `--upstream-repo` may name the upstream checkout, and
its `git rev-parse HEAD` is used only as a fallback when the binary embeds no
revision -- a checkout's HEAD is not evidence about a binary built earlier.

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
  Arithmetic opcode error                                unmapped (see below)
  unknown error                                          unmapped (see below)
  Push value size limit exceeded                         unmapped
  Stack size limit exceeded                              unmapped
  Stack size must be exactly one after execution         unmapped
  Script is too big                                      unmapped
  Operation limit exceeded                               unmapped
  Script failed an OP_VERIFY operation                   unmapped
  Script failed an OP_EQUALVERIFY operation              unmapped
  Script failed an OP_CHECKSIGVERIFY operation           unmapped
  Script failed an OP_NUMEQUALVERIFY operation           unmapped
  Invalid Schnorr signature size                         unmapped
  Invalid Schnorr signature hash type                    unmapped
  Invalid Taproot control block size                     unmapped

Two of those unmapped entries are deliberate and load-bearing. Elements
answers both an overflowing 64-bit operation and a zero divisor with the
single string "Arithmetic opcode error", so `arithmetic_overflow` and
`division_by_zero` are not distinguishable here; and a malformed or oversized
script number raises `scriptnum_error`, which becomes `SCRIPT_ERR_UNKNOWN_ERROR`
and the string "unknown error", so `malformed_script_number` and
`script_number_range_exceeded` are not distinguishable either. Reporting
either pair as one of its members would be a guess presented as an
observation.

What consensus does not see
---------------------------
Because a policy-only rejection is retried as a block, three of the table's
entries were measured to be unreachable through this adapter, and are kept
only so that a future Elements making them consensus would be classified
rather than silently unmapped:

  malformed_push          a nonminimal push is standardness in tapscript,
                          not consensus, and is accepted at consensus;
  leaf_version_rejected   an unrecognised leaf version is unconditionally
                          valid at consensus under BIP-341, and is accepted;
  unknown_opcode          an OP_SUCCESSx byte makes the script succeed at
                          consensus. Only a byte that is neither defined nor
                          OP_SUCCESSx -- inside an executed branch -- is
                          rejected as an unknown opcode.

Measured against `v28.99.0-7110a84bb1fe`: leaf versions `0xc0` and `0xc2`,
`OP_SUCCESS` byte `0x50`, and a nonminimal one-byte push were all accepted at
consensus while the mempool refused them on policy grounds. A fixture whose
reviewed expectation is a rejection in one of those classes is stating a
policy requirement, and no node verdict will confirm it.
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

# The protocol revision this adapter speaks. It must match
# NATIVE_PROTOCOL_SCHEMA in the conformance package.
NATIVE_PROTOCOL_SCHEMA = 1

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
    # Deliberately unmapped: Elements collapses two harness classes into one
    # string in each of these cases. See the module docstring.
    "Arithmetic opcode error": None,
    "unknown error": None,
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


def parse_fixture(raw: object) -> dict:
    """Decodes one fixture strictly, naming any field this adapter cannot read.

    The `expected` member is checked for shape and then discarded. This
    adapter never reads what the fixture expects: an executor that consults
    the expectation is comparing a fixture with itself (Guide-9 section 10.6).
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
            "script",
            "initial_stack",
            "context",
            "expected",
        ),
        "fixture",
    )
    check_expected_shape(fixture["expected"])
    return {
        "execution_domain": fixture["execution_domain"],
        "leaf_version": require_int(fixture["leaf_version"], "fixture.leaf_version"),
        "script": require_bytes(fixture["script"], "fixture.script"),
        "initial_stack": require_byte_vectors(fixture["initial_stack"], "fixture.initial_stack"),
        "context": parse_context(fixture["context"]),
    }


def check_expected_shape(raw: object) -> None:
    """Validates the expectation's shape without reading its content."""
    expected = require_object(raw, "fixture.expected")
    if len(expected) != 1:
        raise AdapterError("fixture.expected is not a single-variant enum")
    for name, body in expected.items():
        if name == "accept":
            require_keys(
                require_object(body, "fixture.expected.accept"),
                ("final_stack", "final_altstack"),
                "fixture.expected.accept",
            )
        elif name == "reject":
            require_keys(
                require_object(body, "fixture.expected.reject"),
                ("class",),
                "fixture.expected.reject",
            )
        else:
            raise AdapterError("unknown field: fixture.expected.%s" % name)


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
        "outpoint_txid": require_bytes(value["outpoint_txid"], path + ".outpoint_txid"),
        "outpoint_index": require_int(value["outpoint_index"], path + ".outpoint_index"),
        "spent_asset": require_bytes(value["spent_asset"], path + ".spent_asset"),
        "spent_value": require_bytes(value["spent_value"], path + ".spent_value"),
        "spent_program": require_bytes(value["spent_program"], path + ".spent_program"),
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
        "asset": require_bytes(value["asset"], path + ".asset"),
        "value": require_bytes(value["value"], path + ".value"),
        "nonce": require_bytes(value["nonce"], path + ".nonce"),
        "program": require_bytes(value["program"], path + ".program"),
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
        "control": require_bytes(value["control"], "fixture.context.script_path.control"),
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

    def prime(self) -> None:
        """Locates the chain's free-coin output and confirms one block."""
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
                return
        raise FatalAdapterError("the chain carries no anyone-can-spend free-coin output")

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

    def raw_output(self, asset_field: bytes, value_field: bytes, nonce_field: bytes, program: bytes):
        """Builds one output from declared field bytes, verbatim."""
        messages = self.messages
        out = messages.CTxOut()
        out.nAsset = messages.CTxOutAsset(asset_field)
        out.nValue = messages.CTxOutValue()
        out.nValue.vchCommitment = value_field
        out.nNonce = messages.CTxOutNonce(nonce_field)
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
        return self.judge(transaction)

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
        if declared_path["script"] and declared_path["script"] != fixture["script"]:
            raise AdapterError(
                "fixture.context.script_path.script disagrees with fixture.script"
            )
        if declared_path["control"]:
            raise AdapterError(
                "fixture.context.script_path.control is declared, and this adapter "
                "derives the control block from the leaf it built"
            )

        funded = []
        for position, declared in enumerate(inputs):
            self.refuse_unmaterialisable_input(declared, position == index)
            amount = explicit_amount(declared["spent_value"], declared["path"] + ".spent_value")
            spend_program = program if position == index else self.anyone_can_spend
            funded.append((declared, self.fund(spend_program, amount), amount))

        transaction = messages.CTransaction()
        transaction.version = context["version"]
        transaction.nLockTime = context["locktime"]
        for declared, funding_txid, _amount in funded:
            transaction.vin.append(
                messages.CTxIn(
                    messages.COutPoint(txid_to_internal_int(funding_txid), 0),
                    nSequence=declared["sequence"],
                )
            )
        for declared in context["outputs"]:
            transaction.vout.append(
                self.raw_output(
                    declared["asset"], declared["value"], declared["nonce"], declared["program"]
                )
            )
        for position, (declared, _txid, _amount) in enumerate(funded):
            witness = messages.CTxInWitness()
            if position == index:
                witness.scriptWitness.stack = list(fixture["initial_stack"]) + [
                    leaf_script,
                    control,
                ]
            else:
                witness.scriptWitness.stack = list(declared["witness"])
            transaction.wit.vtxinwit.append(witness)
        return transaction

    def refuse_unmaterialisable_input(self, declared: dict, is_current: bool) -> None:
        """Names every declared input field this adapter cannot honour."""
        path = declared["path"]
        if any(declared["outpoint_txid"]):
            raise AdapterError(
                "%s.outpoint_txid names a transaction this adapter cannot create; "
                "leave it all-zero to have the executor supply the real outpoint" % path
            )
        if declared["outpoint_index"] != 0:
            raise AdapterError(
                "%s.outpoint_index is declared nonzero, and this adapter supplies "
                "the real outpoint" % path
            )
        if declared["issuance"] is not None:
            raise AdapterError("%s.issuance is not materialisable by this adapter" % path)
        if self.policy_asset_field != declared["spent_asset"]:
            raise AdapterError(
                "%s.spent_asset names an asset this adapter cannot issue on the "
                "disposable chain" % path
            )
        if is_current and declared["spent_program"]:
            raise AdapterError(
                "%s.spent_program is declared, and this adapter supplies the taproot "
                "program committing to the fixture's leaf" % path
            )

    # -- verdict ----------------------------------------------------------

    def judge(self, transaction) -> dict:
        """Submits the spending transaction and classifies what the node said."""
        raw = transaction.serialize().hex()
        answer = self.node.call("testmempoolaccept", json.dumps([raw]))
        if not isinstance(answer, list) or len(answer) != 1:
            raise AdapterError("the node did not answer testmempoolaccept with one result")
        result = answer[0]
        if result.get("allowed") is True:
            return {"verdict": "accepted", "observed_failure": None}
        reason = result.get("reject-reason")
        if not isinstance(reason, str):
            raise AdapterError("the node rejected without naming a reason")
        if reason.startswith(CONSENSUS_SCRIPT_PREFIX):
            return rejection(reason[len(CONSENSUS_SCRIPT_PREFIX) : -1])
        if reason.startswith(POLICY_SCRIPT_PREFIX):
            return self.judge_at_consensus(raw, reason[len(POLICY_SCRIPT_PREFIX) : -1])
        raise AdapterError("the node refused the transaction for a reason that is not "
                           "a script verdict: %s" % reason)

    def judge_at_consensus(self, raw: str, policy_error: str) -> dict:
        """Retries a policy-only rejection as a block, which is consensus.

        Standardness is not consensus, and the reviewed contract is about
        consensus. A transaction the mempool refuses on policy grounds is
        offered to `generateblock`, which validates it the way a block does.
        """
        try:
            self.node.call("generateblock", "raw(%s)" % ANYONE_CAN_SPEND_HEX, json.dumps([raw]))
        except AdapterError as error:
            if CONSENSUS_SCRIPT_PREFIX in error.note:
                start = error.note.index(CONSENSUS_SCRIPT_PREFIX) + len(CONSENSUS_SCRIPT_PREFIX)
                end = error.note.index(")", start)
                return rejection(error.note[start:end])
            raise AdapterError(
                "the node refused the transaction on policy grounds (%s) and refused "
                "the block for a reason that is not a script verdict: %s"
                % (policy_error, error.note)
            )
        return {"verdict": "accepted", "observed_failure": None}


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


def resources_for(fixture) -> dict:
    """Reports the observations this executor can actually make.

    A node exposes no interpreter stack, so the peak depths and the largest
    element are unobserved. The protocol's resource type has no null for
    them, so they are reported as zero and this adapter does not declare the
    resource-observation capability; a zero here means "not observed", and
    the schema cannot yet say so.
    """
    return {
        "script_bytes": len(fixture["script"]) if fixture else 0,
        "initial_stack_items": len(fixture["initial_stack"]) if fixture else 0,
        "peak_stack_items": 0,
        "peak_altstack_items": 0,
        "maximum_element_bytes": 0,
        "validation_budget_used": None,
    }


def implementation_provenance(elementsd: str, upstream_repo):
    """Reads the node's own version line and the revision it embeds."""
    completed = subprocess.run(
        [elementsd, "--version"], stdin=subprocess.DEVNULL, capture_output=True, text=True,
        check=False,
    )
    if completed.returncode != 0:
        raise FatalAdapterError("the node binary did not report a version")
    first_line = completed.stdout.strip().splitlines()[0].strip()
    name = first_line.split(" ")[0] if first_line else "unknown"
    match = re.search(r"-([0-9a-f]{7,40})(-dirty)?$", first_line)
    revision = match.group(1) if match else None
    if revision is None and upstream_repo is not None:
        head = subprocess.run(
            ["git", "-C", upstream_repo, "rev-parse", "HEAD"],
            stdin=subprocess.DEVNULL, capture_output=True, text=True, check=False,
        )
        if head.returncode == 0:
            revision = head.stdout.strip()
    return name, first_line, revision


def serve(arguments) -> int:
    """Runs the whole exchange, and destroys the node whatever happens."""
    framework_path, messages, script = load_framework(arguments.framework)
    log("framework loaded from %s" % framework_path)
    name, version, revision = implementation_provenance(
        arguments.elementsd, arguments.upstream_repo
    )

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
                "implementation_name": name,
                "implementation_version": version,
                "upstream_revision": revision,
                "supported_domains": ["tapscript"],
                "supported_leaf_versions": [TAPSCRIPT_LEAF_VERSION],
                "capabilities": ["failure_class_reporting", "transaction_context"],
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
            "resources": resources_for(fixture),
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
        "--upstream-repo",
        default=None,
        help="upstream checkout, consulted for a revision only when the binary "
        "embeds none",
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
