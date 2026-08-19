#!/usr/bin/env python3
"""Why a complete taproot signature can still be an invalid one.

This is the recorded diagnosis behind `G11-W11-06`, kept runnable so the
finding is a thing that can be re-derived rather than a paragraph
somebody has to believe.

# The finding

This target's taproot digest commits the transaction's OUTPUT WITNESS
vector, and it hashes that vector at whatever length the vector happens
to have -- not at one entry per output. Two facts then collide:

  * a transaction carrying no witness at all deserializes with an EMPTY
    output-witness vector, so a signer hashes the empty string there;
  * serializing a transaction that has any witness grows that vector to
    one entry per output, and deserializing it again keeps that length,
    so consensus hashes one empty witness per output.

A wallet asked to sign an unblinded transaction therefore signs a digest
that the same bytes can never produce once they are on the wire. The
wallet reports the signing COMPLETE and the target refuses the spend as
an invalid Schnorr signature.

# What this script does

It boots one disposable node, creates an explicit taproot output the
wallet owns, and spends it three ways:

  A  fully explicit                     no output witnesses
  B  one confidential output            blinding cannot balance against
                                        an explicit input, so the node
                                        declines and quietly returns the
                                        transaction still explicit
  C  two confidential outputs           blinding balances, so the
                                        outputs really are confidential

For each it records what the wallet said, what the target said, and --
the part that makes this a diagnosis rather than an observation -- which
of two candidate digests the produced signature actually verifies
against: the one computed with the output-witness vector empty, or the
one computed with it grown to match the outputs.

A and B are expected to sign the empty-vector digest and be refused. C
is the control: it is expected to sign the grown-vector digest and be
accepted, which is what establishes that the digests are being modelled
correctly here rather than merely differing.

Stdout is the JSON record. Stderr is diagnostics. Exit 0 when the
diagnosis holds, 1 when it does not, 2 when the run could not be made.

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

FEE_SATOSHIS = 1000
COIN_SATOSHIS = 100000000


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
    sys.stderr.write("diagnose-taproot-output-witness-digest: %s\n" % message)
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
        "unsigned_outputs": output_representations(node, unsigned),
        "blinding_requested": blind,
    }
    staged = unsigned
    if blind:
        # Asked for with the ignore-failure default LEFT ON, because the
        # quiet return is the behaviour this row exists to show.
        staged = node.call("blindrawtransaction", unsigned, wallet=wallet)
        record["blinding_changed_the_bytes"] = staged != unsigned
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


def which_digest(framework_directory, record, spent, genesis_hex):
    """Which of the two candidate digests this signature verifies against."""
    sys.path.insert(0, framework_directory)
    from test_framework.messages import (
        CTransaction, CTxOut, CTxOutAsset, CTxOutValue,
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

    grown = TaprootSignatureHash(
        transaction, [spent_output], hash_type, int(genesis_hex, 16), input_index=0
    )
    emptied = copy.deepcopy(transaction)
    emptied.wit.vtxoutwit = []
    empty = TaprootSignatureHash(
        emptied, [spent_output], hash_type, int(genesis_hex, 16), input_index=0
    )
    return {
        "output_witnesses_as_deserialized": len(transaction.wit.vtxoutwit),
        "output_count": len(transaction.vout),
        "digest_with_vector_grown": grown.hex(),
        "digest_with_vector_empty": empty.hex(),
        "digests_differ": grown != empty,
        "signature_verifies_against_grown": verify_schnorr(output_key, signature, grown),
        "signature_verifies_against_empty": verify_schnorr(output_key, signature, empty),
    }


def main(argv):
    parser = argparse.ArgumentParser(prog="diagnose-taproot-output-witness-digest")
    parser.add_argument("--elementsd", required=True)
    parser.add_argument("--elements-cli", required=True)
    parser.add_argument("--framework", required=True)
    parser.add_argument("--boot-timeout", type=float, default=120.0)
    arguments = parser.parse_args(argv)

    scripts_directory = os.path.dirname(os.path.abspath(__file__))
    adapter = load_adapter(scripts_directory)
    adapter.load_framework(arguments.framework)

    wallet = "guide11-w1106-diagnosis"
    node = adapter.DisposableNode(
        arguments.elementsd, arguments.elements_cli, "elementsregtest",
        arguments.boot_timeout, enable_wallet=True,
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
        ownership = {
            "spendable": coin.get("spendable"),
            "solvable": coin.get("solvable"),
            "amountblinder": coin.get("amountblinder"),
            "assetblinder": coin.get("assetblinder"),
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
            attempt(adapter, node, wallet, "one_confidential_output", outpoint,
                    COIN_SATOSHIS, [one], blind=True),
            attempt(adapter, node, wallet, "two_confidential_outputs", outpoint,
                    COIN_SATOSHIS, [one, two], blind=True),
        ]
        genesis = node.call("getblockhash", "0")
    finally:
        node.stop()

    for row in rows:
        row["digest"] = which_digest(arguments.framework, row, spent, genesis)
        note(
            "%-26s complete=%s allowed=%s signed=%s"
            % (
                row["row"], row["wallet_reported_complete"], row["target_allowed"],
                "grown" if row["digest"]["signature_verifies_against_grown"]
                else "empty" if row["digest"]["signature_verifies_against_empty"]
                else "neither",
            )
        )

    by_row = {row["row"]: row for row in rows}
    holds = (
        by_row["fully_explicit"]["wallet_reported_complete"] is True
        and by_row["fully_explicit"]["target_allowed"] is False
        and by_row["fully_explicit"]["digest"]["signature_verifies_against_empty"]
        and not by_row["fully_explicit"]["digest"]["signature_verifies_against_grown"]
        and by_row["one_confidential_output"]["blinding_actually_blinded"] is False
        and by_row["one_confidential_output"]["target_allowed"] is False
        and by_row["two_confidential_outputs"]["blinding_actually_blinded"] is True
        and by_row["two_confidential_outputs"]["target_allowed"] is True
        and by_row["two_confidential_outputs"]["digest"][
            "signature_verifies_against_grown"
        ]
    )

    json.dump(
        {
            "schema": "tripod-taproot-output-witness-digest-1",
            "genesis_id": genesis,
            "spent_object": spent,
            "spent_object_ownership": ownership,
            "rows": rows,
            "diagnosis_holds": holds,
        },
        sys.stdout,
        indent=2,
        sort_keys=True,
    )
    sys.stdout.write("\n")
    note("diagnosis holds" if holds else "the diagnosis did NOT reproduce")
    return 0 if holds else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
