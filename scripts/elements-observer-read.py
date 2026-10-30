#!/usr/bin/env python3
"""Acquire one public transaction through a fresh node over a retained chain."""

import argparse
import importlib.util
import json
import sys
import time


def locator(value):
    if not isinstance(value, dict) or set(value) != {"chain_name", "genesis_id", "transaction_id"}:
        raise ValueError("closed public locator required")
    if value["chain_name"] != "elementsregtest":
        raise ValueError("unsupported chain")
    for field in ("genesis_id", "transaction_id"):
        text = value[field]
        if not isinstance(text, str) or len(text) != 64 or any(c not in "0123456789abcdef" for c in text):
            raise ValueError("noncanonical public identity")
    return value


def acquire(node, public):
    chain = node.call("getblockchaininfo")["chain"]
    genesis = node.call("getblockhash", "0")
    transaction = node.call("getrawtransaction", public["transaction_id"], "true")
    block_hash = transaction["blockhash"]
    block = node.call("getblock", block_hash)
    height = block["height"]
    before = node.call("getblockhash", str(height))
    exact = node.call("getrawtransaction", public["transaction_id"], "true", block_hash)
    after = node.call("getblockhash", str(height))
    floors = node.call("getmempoolinfo", amount_strings=True)
    if any(arg.startswith(("-minrelaytxfee=", "-blockmintxfee=")) for arg in node.start_arguments):
        raise ValueError("unexpected fee override")
    return {
        "chain_name": chain, "genesis_id": genesis,
        "block_hash": block_hash, "block_height": height,
        "active_before": before, "active_after": after,
        "transaction_id": exact["txid"], "witness_identity": exact["hash"],
        "raw_transaction": exact["hex"], "members": block["tx"],
        "relay_floor": floors["minrelaytxfee"], "mempool_floor": floors["mempoolminfee"],
        "block_floor": "0.00000100", "block_provenance": "pinned-source-no-override",
        "fee_overrides": [],
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--adapter", required=True)
    parser.add_argument("--elementsd", required=True)
    parser.add_argument("--elements-cli", required=True)
    parser.add_argument("--chain-directory", required=True)
    args = parser.parse_args()
    started = time.monotonic()
    public = locator(json.load(sys.stdin))
    spec = importlib.util.spec_from_file_location("observer_adapter", args.adapter)
    adapter = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = adapter
    spec.loader.exec_module(adapter)
    node = adapter.DisposableNode(args.elementsd, args.elements_cli, public["chain_name"], 120,
                                  enable_wallet=False, datadir=args.chain_directory,
                                  fee_policy="node-defaults")
    try:
        node.start()
        answer = acquire(node, public)
    finally:
        node.stop()
        print("acquisition-wall-ms %d" % int((time.monotonic() - started) * 1000), file=sys.stderr)
    print(json.dumps(answer, sort_keys=True, separators=(",", ":")))


if __name__ == "__main__":
    main()
