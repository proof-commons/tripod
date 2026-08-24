#!/usr/bin/env python3
"""Is the issuance-rangeproof term the output-witness hazard's twin?

The owner-sighash concept charters this question rather than answering
it. This script is the answer, run by the recorded diagnosis's own
method so that the two results are comparable rather than merely
adjacent.

# The question

This target's taproot message carries two terms whose value depends on
the LENGTH of a witness vector rather than on one entry per input or
per output:

  * the output-witness term, over `witness.vtxoutwit`;
  * the issuance-rangeproof term, over `witness.vtxinwit`.

Both helpers iterate their vector and hash whatever entries are there.
Both vectors are cleared on deserialization and resized to the
transaction's own cardinality only when the witness flag is set, and
both are grown again on serialization. On the source alone the two
terms are the same shape, so the recorded `G11-W11-06` hazard --- a
signer hashing an EMPTY vector where consensus hashes a grown one ---
should have a twin on the input side.

# What this script establishes

It boots one disposable node, creates an explicit taproot output the
wallet owns, and spends it two ways:

  A  fully explicit                     the recorded diagnosis's case A
  C  two confidential outputs           the accepted control

For each produced signature it computes FOUR candidate messages over
the same transaction and reports which one the signature actually
verifies against:

  both_grown        both vectors as the wire form carries them
  outputs_emptied   output-witness vector emptied, input side grown
  inputs_emptied    input-witness vector emptied, output side grown
  both_emptied      the naive signer that works from witnessless bytes

Two vectors, two states each, so the four candidates separate the two
terms instead of confounding them. `both_grown` is what consensus
forms. `outputs_emptied` is the recorded diagnosis's empty-vector
candidate. `inputs_emptied` exists only to prove the input-side term is
length-dependent at all: if it equals `both_grown` the term could not
carry a hazard, and the question would be closed the other way.

# The expectation, taken from source and not from this run

The wallet resizes the input-witness vector to the input count before
any signing begins, and nothing resizes the output-witness vector until
serialization. So case A is expected to sign `outputs_emptied` --- the
output-witness term wrong, the issuance-rangeproof term already at its
consensus length --- and to be refused. The twin is expected to be
structural rather than observed in this path.

That is a refutation of the hazard for this signer, not of the length
dependence: `inputs_emptied` is expected to differ from `both_grown`,
which is what makes the term hazardous for any OTHER signer that works
from witnessless bytes.

# One probe rides along

The selected profile refuses the single-output dimension, and a refusal
ought to be a claim with content rather than caution. The last row asks
the wallet for that hash type over the explicit row's own bytes and
records what the target does. It is recorded, never asserted: the
expectation above does not depend on it, because this arc never reaches
that branch.

Stdout is the JSON record. Stderr is this script's own diagnostics. The
adapter code it imports writes to neither: it writes to the files under
the directory announced on stderr when the run starts.

Exit 0 when the expectation above holds, 1 when it does not, 2 when the
run could not be made. A run that exits 1 is a finding, not a fault:
the record says which candidate each signature matched.

Nothing here accepts a credential. Every value it prints is test
material from a chain this process created and destroyed.
"""
import argparse
import copy
import importlib.util
import io
import json
import os
import sys
import tempfile

FEE_SATOSHIS = 1000
COIN_SATOSHIS = 100000000

CANDIDATES = ("both_grown", "outputs_emptied", "inputs_emptied", "both_emptied")


def load_adapter(scripts_directory):
    """The adapter module, for its node and its framework loader.

    Imported by path rather than by name because the file is a script
    with a hyphenated name, and because a second implementation of
    "boot a disposable node" would be a second thing to keep true.
    """
    spec = importlib.util.spec_from_file_location(
        "elements_native_executor",
        os.path.join(scripts_directory, "elements-native-executor.py"),
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def note(message):
    sys.stderr.write("diagnose-taproot-issuance-rangeproof-digest: %s\n" % message)
    sys.stderr.flush()


def output_representations(node, raw):
    """Explicit or confidential, per output, as the target decodes it."""
    decoded = node.call("decoderawtransaction", raw)
    return [
        "confidential" if entry.get("valuecommitment") else "explicit"
        for entry in decoded["vout"]
    ]


def attempt(adapter, node, wallet, label, outpoint, value, destinations, blind):
    """One spend shape, from construction to the target's verdict."""
    remaining = value - FEE_SATOSHIS
    share = remaining // len(destinations)
    declared = []
    for position, destination in enumerate(destinations):
        piece = remaining - share * (len(destinations) - 1) if position == 0 else share
        declared.append('{"%s":%s}' % (destination, adapter.satoshis_to_amount(piece)))
    declared.append('{"fee":%s}' % adapter.satoshis_to_amount(FEE_SATOSHIS))

    unsigned = node.call(
        "createrawtransaction", json.dumps([outpoint]), "[" + ",".join(declared) + "]"
    )
    record = {
        "row": label,
        "unsigned_transaction": unsigned,
        "unsigned_outputs": output_representations(node, unsigned),
        "blinding_requested": blind,
    }
    staged = unsigned
    if blind:
        staged = node.call("blindrawtransaction", unsigned, wallet=wallet)
        record["blinded_outputs"] = output_representations(node, staged)
        record["blinding_actually_blinded"] = "confidential" in record["blinded_outputs"]

    signed = node.call("signrawtransactionwithwallet", staged, wallet=wallet)
    record["wallet_reported_complete"] = signed.get("complete")
    record["wallet_errors"] = signed.get("errors")
    verdict = node.call("testmempoolaccept", json.dumps([signed["hex"]]))
    record["target_allowed"] = verdict[0].get("allowed")
    record["target_reject_reason"] = verdict[0].get("reject-reason")
    record["signed_transaction"] = signed["hex"]
    return record


def probe_single_output(adapter, node, wallet, unsigned, outputs):
    """What the target does when asked for the refused single-output type.

    The selected owner profile refuses this dimension, so this arc never
    reaches the branch. The probe exists because a refusal ought to be a
    claim with content: the branch reads the output-witness vector at
    the signing input's position after bounds-checking that position
    against the OUTPUT list only, and the two are not the same length in
    a transaction that is being signed rather than verified.

    Whatever happens here is recorded rather than asserted. A node that
    survives says the branch is defended somewhere this reading did not
    look; a node that does not says the refusal is worth more than
    caution.
    """
    record = {"row": "single_output_probe", "outputs": outputs}
    try:
        signed = node.call(
            "signrawtransactionwithwallet", unsigned, "[]", "SINGLE", wallet=wallet
        )
        record["wallet_reported_complete"] = signed.get("complete")
        record["wallet_errors"] = signed.get("errors")
        record["signed_transaction"] = signed["hex"]
        record["client_refused"] = False
    except adapter.AdapterError as error:
        record["client_refused"] = True
        record["client_refusal"] = str(error)
        record["signed_transaction"] = None
    try:
        record["node_still_answers"] = node.call("getblockcount") is not None
    except adapter.AdapterError:
        record["node_still_answers"] = False
    # The node's own exit status, which is the difference between a
    # refusal the target chose and a process that stopped existing. A
    # negative status is the signal that ended it.
    status = node.process.poll()
    record["node_process_status"] = status
    record["node_ended_by_signal"] = status is not None and status < 0
    return record


def four_candidates(framework_directory, record, spent, genesis_hex):
    """Which of four candidate messages this signature verifies against.

    The two witness vectors vary independently, so the record separates
    the two length-dependent terms rather than reporting one verdict
    about both at once.
    """
    sys.path.insert(0, framework_directory)
    from test_framework.messages import (
        CTransaction, CTxInWitness, CTxOut, CTxOutAsset, CTxOutValue,
    )
    from test_framework.script import TaprootSignatureHash
    from test_framework.key import verify_schnorr

    program = bytes.fromhex(spent["script"])
    if len(program) != 34 or program[0] != 0x51 or program[1] != 0x20:
        raise SystemExit("the spent output is not a taproot v1 program")
    output_key = program[2:]
    spent_output = CTxOut(
        nValue=CTxOutValue(spent["value"]),
        scriptPubKey=program,
        nAsset=CTxOutAsset(b"\x01" + bytes.fromhex(spent["asset"])[::-1]),
    )

    transaction = CTransaction()
    transaction.deserialize(io.BytesIO(bytes.fromhex(record["signed_transaction"])))
    stack = transaction.wit.vtxinwit[0].scriptWitness.stack
    if len(stack) != 1 or len(stack[0]) not in (64, 65):
        raise SystemExit("the spend is not the key-path authorization this reads")
    signature = stack[0][:64]
    hash_type = stack[0][64] if len(stack[0]) == 65 else 0

    def message(drop_output_witnesses, drop_input_witnesses):
        variant = copy.deepcopy(transaction)
        if drop_output_witnesses:
            variant.wit.vtxoutwit = []
        if drop_input_witnesses:
            variant.wit.vtxinwit = []
        return TaprootSignatureHash(
            variant, [spent_output], hash_type, int(genesis_hex, 16), input_index=0
        )

    digests = {
        "both_grown": message(False, False),
        "outputs_emptied": message(True, False),
        "inputs_emptied": message(False, True),
        "both_emptied": message(True, True),
    }
    verifies = {
        name: verify_schnorr(output_key, signature, digest)
        for name, digest in digests.items()
    }
    matched = [name for name in CANDIDATES if verifies[name]]

    # The naive signer's input-side term, written out so the record
    # says what the bytes are and not only that they differ. A
    # default-constructed input witness contributes two length prefixes
    # of zero, one per issuance rangeproof.
    default_entry = CTxInWitness()
    per_entry = default_entry.serialize_issuance_proofs()

    return {
        "input_witnesses_as_deserialized": len(transaction.wit.vtxinwit),
        "output_witnesses_as_deserialized": len(transaction.wit.vtxoutwit),
        "input_count": len(transaction.vin),
        "output_count": len(transaction.vout),
        "issuance_proofs_per_default_entry": per_entry.hex(),
        "issuance_proofs_bytes_per_default_entry": len(per_entry),
        "issuance_rangeproofs_are_all_empty": all(
            not inwit.vchIssuanceAmountRangeproof
            and not inwit.vchInflationKeysRangeproof
            for inwit in transaction.wit.vtxinwit
        ),
        "digests": {name: digest.hex() for name, digest in digests.items()},
        "verifies": verifies,
        "signed_candidate": matched[0] if len(matched) == 1 else None,
        "candidates_matched": len(matched),
        "input_side_term_is_length_dependent": (
            digests["inputs_emptied"] != digests["both_grown"]
        ),
        "output_side_term_is_length_dependent": (
            digests["outputs_emptied"] != digests["both_grown"]
        ),
        "all_four_candidates_distinct": len(set(digests.values())) == 4,
    }


def main(argv):
    parser = argparse.ArgumentParser(prog="diagnose-taproot-issuance-rangeproof-digest")
    parser.add_argument("--elementsd", required=True)
    parser.add_argument("--elements-cli", required=True)
    parser.add_argument("--framework", required=True)
    parser.add_argument("--boot-timeout", type=float, default=120.0)
    parser.add_argument("--adapter-diagnostics-directory", default=None)
    # A directory the CALLER owns, kept after the run. The probe row can
    # end the node's process, and when it does the node's own account of
    # why is the only account there is; a disposable directory takes it
    # away with the chain. Off by default, so an ordinary run still
    # destroys everything it made.
    parser.add_argument("--node-datadir", default=None)
    arguments = parser.parse_args(argv)

    scripts_directory = os.path.dirname(os.path.abspath(__file__))
    adapter = load_adapter(scripts_directory)
    adapter.load_framework(arguments.framework)

    diagnostics_directory = arguments.adapter_diagnostics_directory or tempfile.mkdtemp(
        prefix="diagnose-taproot-issuance-rangeproof-digest-"
    )
    os.makedirs(diagnostics_directory, exist_ok=True)
    adapter_output = os.path.join(diagnostics_directory, "adapter-diagnostics.txt")
    adapter_elements_output = os.path.join(
        diagnostics_directory, "adapter-child-output.txt"
    )
    note("the adapter's diagnostics go to %s" % diagnostics_directory)
    typed_handle = open(adapter_output, "a", encoding="utf-8", errors="replace")
    child_handle = open(
        adapter_elements_output, "a", encoding="utf-8", errors="replace"
    )
    adapter.STREAMS = adapter.DiagnosticStreams(typed_handle, child_handle)

    wallet = "sighash-w1-issuance-rangeproof"
    single = None
    node = adapter.DisposableNode(
        arguments.elementsd, arguments.elements_cli, "elementsregtest",
        arguments.boot_timeout, enable_wallet=True,
        datadir=arguments.node_datadir,
    )
    node.start()
    try:
        node.call("createwallet", wallet)
        node.call("generatetoaddress", "1", node.call("getnewaddress", wallet=wallet))
        node.call("rescanblockchain", wallet=wallet)

        fresh = node.call("getnewaddress", "", "bech32m", wallet=wallet)
        info = node.call("getaddressinfo", fresh, wallet=wallet)
        explicit_address = info["unconfidential"]
        note("the object under test is %s" % explicit_address)

        txid = node.call(
            "sendtoaddress", explicit_address,
            adapter.satoshis_to_amount(COIN_SATOSHIS), wallet=wallet,
        )
        node.call("generatetoaddress", "1", node.call("getnewaddress", wallet=wallet))
        coin = None
        for candidate in node.call("listunspent", "1", "9999999", wallet=wallet):
            if candidate["txid"] == txid and candidate.get("address") == explicit_address:
                coin = candidate
                break
        if coin is None:
            raise SystemExit("the wallet does not report the explicit taproot object")
        spent = {
            "script": info["scriptPubKey"],
            "value": COIN_SATOSHIS,
            "asset": coin["asset"],
        }

        outpoint = {"txid": txid, "vout": coin["vout"]}
        plain = node.call(
            "getaddressinfo", node.call("getnewaddress", "", "bech32m", wallet=wallet),
            wallet=wallet,
        )["unconfidential"]
        one = node.call("getnewaddress", "", "bech32m", wallet=wallet)
        two = node.call("getnewaddress", "", "bech32m", wallet=wallet)

        rows = [
            attempt(adapter, node, wallet, "fully_explicit", outpoint, COIN_SATOSHIS,
                    [plain], blind=False),
            attempt(adapter, node, wallet, "two_confidential_outputs", outpoint,
                    COIN_SATOSHIS, [one, two], blind=True),
        ]
        genesis = node.call("getblockhash", "0")
        # Last, because it is the row that may leave nothing behind it
        # to ask. The refused dimension is probed on the explicit row's
        # own unsigned bytes, so the only difference from that row is
        # the requested hash type.
        single = probe_single_output(
            adapter, node, wallet, rows[0]["unsigned_transaction"],
            len(rows[0]["unsigned_outputs"]),
        )
        note(
            "single_output_probe          refused=%s node_alive=%s"
            % (single.get("client_refused"), single.get("node_still_answers"))
        )
    finally:
        node.stop()

    for row in rows:
        row["digest"] = four_candidates(arguments.framework, row, spent, genesis)
        note(
            "%-26s complete=%s allowed=%s signed=%s"
            % (
                row["row"], row["wallet_reported_complete"], row["target_allowed"],
                row["digest"]["signed_candidate"],
            )
        )

    by_row = {row["row"]: row for row in rows}
    explicit = by_row["fully_explicit"]["digest"]
    control = by_row["two_confidential_outputs"]["digest"]
    holds = (
        by_row["fully_explicit"]["wallet_reported_complete"] is True
        and by_row["fully_explicit"]["target_allowed"] is False
        and explicit["signed_candidate"] == "outputs_emptied"
        and explicit["input_side_term_is_length_dependent"] is True
        and explicit["output_side_term_is_length_dependent"] is True
        and explicit["all_four_candidates_distinct"] is True
        and by_row["two_confidential_outputs"]["target_allowed"] is True
        and control["signed_candidate"] == "both_grown"
    )

    json.dump(
        {
            "schema": "tripod-taproot-issuance-rangeproof-digest-1",
            "genesis_id": genesis,
            "spent_object": spent,
            "rows": rows,
            "refused_dimension_probe": single,
            "verdict": {
                "input_side_term_is_length_dependent": explicit[
                    "input_side_term_is_length_dependent"
                ],
                "wallet_signed_candidate": explicit["signed_candidate"],
                "hazard_observed_in_this_signer": explicit["signed_candidate"]
                in ("inputs_emptied", "both_emptied"),
            },
            "expectation_holds": holds,
        },
        sys.stdout,
        indent=2,
        sort_keys=True,
    )
    sys.stdout.write("\n")
    note("expectation holds" if holds else "the expectation did NOT hold")
    return 0 if holds else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
