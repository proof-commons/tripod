#!/usr/bin/env python3
"""Project sealed final native captures; run only on the evidence node.

No executable content from an archive is evaluated. One archive is loaded at a
time. LZMA-alone matches the existing keyed-scaling-run-of-record fixture.
"""
import argparse
import hashlib
import json
import lzma
from pathlib import Path
import time


def digest(data):
    return hashlib.sha256(data).hexdigest()


def compact(value):
    return json.dumps(value, ensure_ascii=False, separators=(",", ":")).encode()


def sealed(path, root):
    seal = root / "retention.sha256"
    entries = {}
    for line in seal.read_text().splitlines():
        checksum, name = line.split(maxsplit=1)
        entries[name.lstrip("*")] = checksum
    candidates = (str(path.relative_to(root)), "./" + str(path.relative_to(root)), str(path))
    checksum = next((entries[name] for name in candidates if name in entries), None)
    if checksum is None:
        raise ValueError(f"file absent from seal: {path}")
    data = path.read_bytes()
    if digest(data) != checksum:
        raise ValueError(f"seal mismatch: {path}")
    return data, checksum


def readback(value):
    if value is None:
        return None
    return {"transaction_id": value["transaction_id"],
            "witness_transaction_id": value["witness_transaction_id"],
            "block_height": value["block_height"],
            "raw_transaction": bytes(value["raw_transaction"]).hex()}


def policy(value):
    # start_arguments contains ephemeral RPC ports and run paths.
    return {key: item for key, item in value.items() if key != "start_arguments"}


def project(path, root, run):
    started = time.monotonic()
    data, checksum = sealed(path, root)
    archive = json.loads(data)
    del data
    manifest = archive["manifest"]
    expected = bytes(archive["manifest_digest"]).hex()
    if digest(compact(manifest)) != expected:
        raise ValueError("manifest digest mismatch")
    for row in manifest["rows"]:
        submitted = bytes(row["submitted"])
        if digest(submitted) != bytes(row["digest"]).hex():
            raise ValueError("submitted digest mismatch")
        row["submitted"] = submitted.hex()
    if archive["native_clause_dimensions"] is not None:
        raise ValueError("unexpected native clause measurements")
    sessions = []
    for index, session in enumerate(archive["sessions"]):
        parents, observations = [], []
        for request, response in session["exchanges"]:
            subject = request["subject"]
            if "transaction_bytes" not in subject:
                if response["funded_outputs"]:
                    parents.append(readback(response["mined_readback"]))
                continue
            submitted = bytes(subject["transaction_bytes"])
            detail = json.loads(response["observed_detail"])
            inner = detail.get("detail", {})
            observations.append({
                "digest": digest(submitted), "step": request["case"]["step"],
                "layer": response["observed_layer"],
                "route": detail["route"], "relay_allowed": detail["relay_allowed"],
                "relay_detail": detail.get("relay_detail"),
                "verdict_reason": inner.get("verdict_reason"),
                "block_script_error": inner.get("block_script_error"),
                "template_member": inner.get("template", {}).get("member"),
                "inclusion_in_produced": inner.get("inclusion_in_produced"),
                "policy": policy(detail["policy"]),
                "resources": response["resources"],
                "accepted_txid": response["accepted_txid"],
                "readback": readback(response["mined_readback"]),
            })
            if observations[-1]["readback"] is not None:
                if observations[-1]["readback"]["raw_transaction"] != submitted.hex():
                    raise ValueError("readback differs from submitted bytes")
                # Equality was checked above; one copy suffices in the projection.
                observations[-1]["readback"].pop("raw_transaction")
        diagnostic = path.parent / f"work/key-{index}/diagnostics/executor-child-output.txt"
        diagnostic_bytes, diagnostic_digest = sealed(diagnostic, root)
        sessions.append({"key": session["key"], "provenance": session["provenance"],
                         "handshake": session["handshake"], "parents": parents,
                         "observations": observations,
                         "diagnostic_digest": diagnostic_digest,
                         "diagnostic": "\n".join(line for line in diagnostic_bytes.decode().splitlines()
                                                    if "TestBlockValidity failed:" in line
                                                    or "Balance check failed" in line)})
    result = {"run": run, "archive": str(path.relative_to(root)),
              "archive_digest": checksum, "manifest_digest": expected,
              "manifest": manifest, "sessions": sessions}
    print(f"project {run}/{root.name}/{result['archive']} rows={len(manifest['rows'])} wall={time.monotonic()-started:.6f}s", flush=True)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence_root", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    campaign = "20261114t0850z-campaign-ad"
    envelope = "20261114t1500z-envelope-campaign"
    census = {"schema": 1, "files": []}
    for target in ("birth", "continuation", "commitment", "cost", "funding", "scaling"):
        run = campaign if target not in ("funding", "scaling") else envelope
        root = args.evidence_root / run / target
        paths = [root / "capture.json"] if run == campaign else [
            root / f"case-{n}/capture.json" for n in range(24 if target == "funding" else 8)]
        compressed = bytearray()
        case_census = []
        for path in paths:
            case = project(path, root, run)
            raw = compact(case) + b"\n"
            member = lzma.compress(raw, format=lzma.FORMAT_ALONE, preset=6)
            case_census.append({"offset": len(compressed), "bytes": len(member),
                                "decoded_bytes": len(raw), "decoded_sha256": digest(raw),
                                "archive_digest": case["archive_digest"],
                                "manifest_digest": case["manifest_digest"],
                                "rows": len(case["manifest"]["rows"])})
            compressed.extend(member)
        name = target + ".json.lzma"
        if len(compressed) > 4_000_000:
            raise ValueError(f"fixture exceeds 4 MB: {name} {len(compressed)}")
        (args.output / name).write_bytes(compressed)
        census["files"].append({"name": name, "bytes": len(compressed),
                                "sha256": digest(compressed), "cases": case_census})
        print(f"fixture {name} bytes={len(compressed)} decoded={sum(c['decoded_bytes'] for c in case_census)}", flush=True)
    census_bytes = compact(census) + b"\n"
    total = len(census_bytes) + sum(row["bytes"] for row in census["files"])
    if total > 16_000_000:
        raise ValueError(f"fixture total exceeds 16 MB: {total}")
    (args.output / "CENSUS.json").write_bytes(census_bytes)
    print(f"fixture-total bytes={total}", flush=True)


if __name__ == "__main__":
    main()
