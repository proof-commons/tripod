#!/usr/bin/env python3
"""Node-free tests for the executor's script-error classification (CI-009).

`scripts/elements-native-executor.py` is 4400 lines of adapter, and until
this file existed none of it was covered by any lane: CI-007 found that the
repository had no Python test surface at all, and the ruling that closed it
sends the missing lane here rather than into a driver's registry.

The subject is the part of the adapter that decides what a target's refusal
MEANS: `script_error_in`, which lifts the script error out of the node's
wrapper, and `rejection`, which maps that text to a member of the harness's
typed failure vocabulary. Both are pure string functions over module-level
data, so the whole lane runs without a node, without a network, and without
a Rust or TeX toolchain -- which is why it can be an ordinary meson test
rather than a live-node lane.

This file also drives `submit_maturity` over a scripted in-memory node.

What this proves, and why it is worth a lane
--------------------------------------------
The G12-R07 repair replaced a parser that took the substring up to the
FIRST `)` after the wrapper opened with one that closes the wrapper by
matching the parenthesis it opened. Two of the mapped messages carry a
parenthesis of their own, so under the old parser each truncated to a
string the class table does not hold, the lookup missed, and a rejection
the target had classified precisely arrived carrying no class at all.

That failure was SILENT. A lost class is indistinguishable from a target
that refused for a reason this adapter does not classify, so nothing in
any report said a class had been dropped. A repair whose failure mode is
silence needs an oracle that is exhaustive rather than illustrative: this
file therefore asserts every one of the table's messages round-trips, not
just the two that regressed, so a future message carrying a parenthesis --
or any other character the parser might mishandle -- is caught when it is
added rather than when someone re-reads a report.

The negative direction matters equally. `script_error_in` returns None for
a text that is not a script verdict at all and for one whose wrapper never
closes, and both callers turn that None into a raised AdapterError rather
than into a rejection. A parser that guessed where an unbalanced message
ended would manufacture a consensus fact out of a malformed string, so the
refusal of both shapes is asserted directly.

Run standalone (`python3 scripts/test-executor-classification.py`) or as
the `executor-classification` meson test.
"""

from __future__ import annotations

import importlib.util
import json
import os
import sys
from types import SimpleNamespace

REPOSITORY_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
EXECUTOR_PATH = os.path.join(REPOSITORY_ROOT, "scripts", "elements-native-executor.py")


def load_executor():
    """Import the adapter by path; its file name is not an identifier.

    Importing runs module-level definitions only -- the adapter's entry
    point is behind a `__main__` guard -- so no node is started and no
    subprocess is spawned by the import itself.
    """
    spec = importlib.util.spec_from_file_location("elements_native_executor", EXECUTOR_PATH)
    if spec is None or spec.loader is None:
        raise SystemExit("could not load %s" % EXECUTOR_PATH)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class Failures:
    """Collected assertion failures, so one run reports every problem."""

    def __init__(self) -> None:
        self.notes = []

    def check(self, condition: bool, note: str) -> None:
        if not condition:
            self.notes.append(note)

    def equal(self, actual, expected, note: str) -> None:
        self.check(
            actual == expected,
            "%s: expected %r, observed %r" % (note, expected, actual),
        )


def test_every_mapped_message_survives_both_wrappers(executor, failures) -> int:
    """Every table message is recovered verbatim from both wrappers.

    Both prefixes are exercised for every message rather than for a sample:
    the relay judgement tries the policy wrapper and then the consensus one,
    the consensus judgement reads the consensus wrapper alone, and the two
    spellings of the rule were unified by G12-R07 precisely so they cannot
    disagree. Asserting one wrapper would leave the other free to drift.
    """
    prefixes = (executor.CONSENSUS_SCRIPT_PREFIX, executor.POLICY_SCRIPT_PREFIX)
    checked = 0
    for message in executor.FAILURE_CLASS_BY_SCRIPT_ERROR:
        for prefix in prefixes:
            # The shape the node actually emits: the wrapper around the
            # message, with trailing text after the closing parenthesis,
            # which the parser must not include and must not stop short of.
            text = prefix + message + ") extra trailing detail"
            failures.equal(
                executor.script_error_in(text, prefix),
                message,
                "message %r under wrapper %r" % (message, prefix),
            )
            checked += 1
    return checked


def test_every_mapped_message_classifies(executor, failures) -> int:
    """`rejection` yields a rejected verdict with a class for every message.

    A key present in the table but mapped to None would read, in every
    report, exactly like a message the adapter never knew about; the table's
    own docstring says both mean "not classified". This asserts the table
    holds no such entry.
    """
    for message, expected in executor.FAILURE_CLASS_BY_SCRIPT_ERROR.items():
        body = executor.rejection(message)
        failures.equal(body.get("verdict"), "rejected", "verdict for %r" % message)
        failures.equal(body.get("observed_failure"), expected, "class for %r" % message)
        failures.check(
            expected is not None and expected != "",
            "table entry %r maps to no class, which reports as unclassified" % message,
        )
    return len(executor.FAILURE_CLASS_BY_SCRIPT_ERROR)


def test_parenthesised_messages_are_not_truncated(executor, failures) -> int:
    """The two messages that carry a parenthesis, named explicitly.

    These are the G12-R07 regression itself. The exhaustive test above
    already covers them; they are also asserted by name so a future reader
    of a failure sees the original defect rather than a count.
    """
    regressed = [
        "Signature must be zero for failed CHECK(MULTI)SIG operation",
        "OP_CHECKMULTISIG(VERIFY) is not available in tapscript",
    ]
    for message in regressed:
        failures.check(
            message in executor.FAILURE_CLASS_BY_SCRIPT_ERROR,
            "the parenthesised message %r left the class table; if it was renamed, "
            "rename it here too rather than dropping the regression case" % message,
        )
        text = executor.CONSENSUS_SCRIPT_PREFIX + message + ") and more"
        recovered = executor.script_error_in(text, executor.CONSENSUS_SCRIPT_PREFIX)
        failures.equal(recovered, message, "parenthesised message %r" % message)
        # The defect was silent because the truncated text simply missed the
        # table. Assert the class arrives, which is what a report shows.
        failures.check(
            executor.rejection(recovered or "")["observed_failure"] is not None,
            "parenthesised message %r recovered without a class" % message,
        )
    return len(regressed)


def test_negative_shapes_are_refused(executor, failures) -> int:
    """The two shapes that are not a readable verdict both yield None.

    Shape one: the wrapper is absent, so the text is not a script verdict.
    Shape two: the wrapper opens and never closes, so where the message
    ended is unknown. Guessing either would put a fabricated class into a
    report, so both must refuse rather than return a best effort.
    """
    consensus = executor.CONSENSUS_SCRIPT_PREFIX
    policy = executor.POLICY_SCRIPT_PREFIX
    absent = [
        ("bad-txns-inputs-missingorspent", consensus),
        ("", consensus),
        ("mandatory-script-verify-flag-failed", consensus),  # opener never reached
        # The other layer's wrapper is not this layer's wrapper: a consensus
        # read of a policy refusal must decline, which is what makes the
        # relay judgement's two-prefix loop meaningful.
        (policy + "Negative locktime)", consensus),
    ]
    unclosed = [
        (consensus + "Negative locktime", consensus),
        (consensus + "OP_CHECKMULTISIG(VERIFY) is not available", consensus),
        (policy + "Script is too big", policy),
    ]
    for text, prefix in absent:
        failures.equal(
            executor.script_error_in(text, prefix),
            None,
            "absent wrapper in %r" % text,
        )
    for text, prefix in unclosed:
        failures.equal(
            executor.script_error_in(text, prefix),
            None,
            "unclosed wrapper in %r" % text,
        )
    return len(absent) + len(unclosed)


def test_unmapped_message_is_reported_unclassified(executor, failures) -> int:
    """A well-formed wrapper naming an unknown message yields no class.

    This is the behaviour a lost class was mistaken for, so it is asserted
    to be the ONLY thing that produces it: a rejection body with a null
    class, never a fabricated one and never a raised error.
    """
    unknown = "a script error no reviewed target has ever emitted"
    failures.check(
        unknown not in executor.FAILURE_CLASS_BY_SCRIPT_ERROR,
        "the fixture message was added to the class table; choose another",
    )
    recovered = executor.script_error_in(
        executor.CONSENSUS_SCRIPT_PREFIX + unknown + ")",
        executor.CONSENSUS_SCRIPT_PREFIX,
    )
    failures.equal(recovered, unknown, "unmapped message recovery")
    body = executor.rejection(unknown)
    failures.equal(body.get("verdict"), "rejected", "unmapped verdict")
    failures.equal(body.get("observed_failure"), None, "unmapped class")
    return 1


def test_a_mempool_conservation_reason_is_not_upgraded_to_a_script_verdict(
    executor, failures
) -> int:
    """A precise mempool reason is not overridden by the block's text.

    The block runs the amount checks in the same queue as the script
    checks, so an imbalance reaches it wearing a mandatory-script error.
    The mempool already said `bad-txns-in-ne-out`, which is not a script
    verdict, and that has to decide the layer -- otherwise a
    conservation failure is attributed to an opening script that never
    ran, and a negative coverage row is discharged on a carrier that
    never executed.

    The one case where the block's text may still speak is the
    transaction the mempool did not refuse at all.
    """

    class BlockSaidScript:
        client_detail = (
            executor.CONSENSUS_SCRIPT_PREFIX + "unknown error) while connecting block"
        )

    class Bare:
        """Enough of the executor for the classifier under test."""

    judge = executor.OperationExecutor.refused_at_consensus
    checks = 0

    overridden = judge(Bare(), BlockSaidScript(), mempool_reason="bad-txns-in-ne-out")
    failures.equal(
        overridden["observed_layer"],
        "consensus_rejection_before_script",
        "a mempool conservation reason must settle the layer",
    )
    failures.equal(
        overridden["observed_detail"],
        "bad-txns-in-ne-out",
        "the mempool's own reason is what leaves the classifier",
    )
    checks += 2

    # A mempool reason that IS a script verdict never reaches here, but a
    # non-script one that is not conservation must behave the same way.
    other = judge(Bare(), BlockSaidScript(), mempool_reason="bad-txns-inputs-missingorspent")
    failures.equal(
        other["observed_layer"],
        "consensus_rejection_before_script",
        "any non-script mempool reason settles the layer",
    )
    checks += 1

    # Nothing from the mempool to contradict: the block's text is the
    # only evidence there is, and is read.
    silent = judge(Bare(), BlockSaidScript())
    failures.equal(
        silent["observed_layer"],
        "script_path_rejection",
        "a relay-accepted transaction a block refuses on a script error",
    )
    checks += 1

    return checks


def maturity_submission(executor, block_error=None, relay_reason="min relay fee not met"):
    """Drive the real submission and classifier over a scripted node, without RPC."""
    txid = "11" * 32
    block_hash = "22" * 32

    class Block:
        vtx = []

        def deserialize(self, stream):
            assert stream.read() == b"\x00"

        def get_weight(self):
            return 400

    class Node:
        fee_policy = "node-defaults"
        start_arguments = []

        def __init__(self):
            self.call_observations = []
            self.methods = []

        def call(self, method, *args, **kwargs):
            self.methods.append(method)
            self.call_observations.append({"method": method})
            if method == "getmempoolinfo":
                return {"size": 0, "minrelaytxfee": "0.00000100",
                        "mempoolminfee": "0.00000100"}
            if method == "decoderawtransaction":
                return {"txid": txid, "vin": [{"txinwitness": ["01", "02", "03"]}]}
            if method == "testmempoolaccept":
                return [{"txid": txid, "allowed": False, "reject-reason": relay_reason}]
            if method == "getnewblockhex":
                return "00"
            if method == "getrawtransaction":
                raise executor.AdapterError("not included")
            if method == "generateblock":
                if block_error is not None:
                    raise executor.AdapterError("block refused", client_detail=block_error)
                return {"hash": block_hash}
            raise AssertionError("unexpected scripted call: " + method)

    node = Node()
    operations = executor.OperationExecutor.__new__(executor.OperationExecutor)
    operations.executor = SimpleNamespace(
        node=node, messages=SimpleNamespace(CBlock=Block), weight_of=lambda raw: 400)
    operations.read_back_confirmed = lambda mined, identity: {
        "block_hash": block_hash, "transaction_id": identity}
    return operations.submit_maturity({"transaction_bytes": b"\x00"}), node.methods


def test_direct_block_records_only_vocabulary_keys(executor, failures):
    """The direct-block branch retains keys, not client text or class values."""
    checked = 0
    for key in executor.FAILURE_CLASS_BY_SCRIPT_ERROR:
        client = ("CLIENT_PREFIX TestBlockValidity failed: "
                  + executor.CONSENSUS_SCRIPT_PREFIX + key + ") CLIENT_SUFFIX")
        body, methods = maturity_submission(executor, client)
        detail = json.loads(body["observed_detail"])
        failures.equal(detail["detail"]["block_script_error"], key, "recorded key")
        failures.equal(body["observed_layer"], "script_path_rejection", "layer retained")
        failures.equal(detail["detail"]["verdict_reason"],
                       "mandatory-script-verify-flag-failed", "bare verdict retained")
        failures.equal(body["transaction_weight"], 400, "weight retained")
        failures.check("CLIENT_PREFIX" not in body["observed_detail"]
                       and "CLIENT_SUFFIX" not in body["observed_detail"], "client text absent")
        failures.equal(methods.count("generateblock"), 1, "one direct-block attempt")
        checked += 6
    for key in ("Witness program hash mismatch", "arbitrary unrecognized client text"):
        body, _ = maturity_submission(executor, "TestBlockValidity failed: "
                                     + executor.CONSENSUS_SCRIPT_PREFIX + key + ")")
        detail = json.loads(body["observed_detail"])
        failures.equal(detail["detail"]["block_script_error"], "unclassified", "unmapped key")
        failures.check(key not in body["observed_detail"], "unmapped client text absent")
        checked += 2
    body, _ = maturity_submission(executor, "TestBlockValidity failed: bad-txns-inputs-missingorspent")
    failures.equal(json.loads(body["observed_detail"])["detail"]["block_script_error"],
                   "unclassified", "no script wrapper")
    return checked + 1


def test_direct_block_preserves_other_routes(executor, failures):
    """Acceptance, relay-only refusal, waiting and mempool precedence stay distinct."""
    body, methods = maturity_submission(executor)
    detail = json.loads(body["observed_detail"])
    failures.equal(body["observed_layer"], "accepted", "accepted direct block")
    failures.equal(detail["route"], "direct-block", "accepted route")
    failures.check("block_script_error" not in detail["detail"], "accepted recording absent")
    failures.equal(methods.count("generateblock"), 1, "accepted block attempted")
    relay = executor.CONSENSUS_SCRIPT_PREFIX + "Invalid Schnorr signature)"
    body, methods = maturity_submission(executor, relay_reason=relay)
    detail = json.loads(body["observed_detail"])
    failures.equal(body["observed_layer"], "script_path_rejection", "relay refusal")
    failures.equal(detail["detail"]["verdict_reason"], relay, "relay reason retained")
    failures.check("block_script_error" not in detail["detail"], "relay recording absent")
    failures.check("generateblock" not in methods, "relay refusal never submits a block")
    body, _ = maturity_submission(executor, "not a validation verdict")
    detail = json.loads(body["observed_detail"])
    failures.equal(body["observed_layer"], "executor_infrastructure_failure", "waiting layer")
    failures.equal(detail["route"], "undecided", "waiting route")
    failures.equal(detail["detail"]["reason"], "block-call-without-validation-verdict",
                   "waiting reason")
    failures.check("block_script_error" not in detail["detail"], "waiting recording absent")
    failures.equal(body["transaction_weight"], None, "waiting has no weight verdict")
    body, _ = maturity_submission(executor, "TestBlockValidity failed: " + relay,
                                  "bad-txns-in-ne-out")
    detail = json.loads(body["observed_detail"])
    failures.equal(body["observed_layer"], "consensus_rejection_before_script", "mempool wins")
    failures.equal(detail["detail"]["verdict_reason"], "bad-txns-in-ne-out", "mempool reason wins")
    failures.equal(detail["detail"]["block_script_error"], "Invalid Schnorr signature",
                   "recording does not decide the layer")
    return 16


TESTS = [
    ("direct block records only vocabulary keys", test_direct_block_records_only_vocabulary_keys),
    ("direct block preserves other routes", test_direct_block_preserves_other_routes),
    ("every mapped message survives both wrappers", test_every_mapped_message_survives_both_wrappers),
    ("every mapped message classifies", test_every_mapped_message_classifies),
    ("parenthesised messages are not truncated", test_parenthesised_messages_are_not_truncated),
    ("negative shapes are refused", test_negative_shapes_are_refused),
    ("an unmapped message is reported unclassified", test_unmapped_message_is_reported_unclassified),
    (
        "a mempool conservation reason is not upgraded to a script verdict",
        test_a_mempool_conservation_reason_is_not_upgraded_to_a_script_verdict,
    ),
]


def main() -> int:
    """Run every test, reporting each one's assertion count and outcome."""
    executor = load_executor()
    table_size = len(executor.FAILURE_CLASS_BY_SCRIPT_ERROR)
    # The class table is the census this lane is exhaustive over, so its
    # size is stated rather than assumed: a message added or removed
    # without a thought for this lane changes the number the run prints.
    print("class table: %d mapped messages" % table_size)

    failed = 0
    for name, body in TESTS:
        failures = Failures()
        assertions = body(executor, failures)
        if failures.notes:
            failed += 1
            print("FAIL %s (%d assertions)" % (name, assertions))
            for note in failures.notes:
                print("     %s" % note)
        else:
            print("ok   %s (%d assertions)" % (name, assertions))

    if failed:
        print("%d of %d tests failed" % (failed, len(TESTS)))
        return 1
    print("%d tests passed" % len(TESTS))
    return 0


if __name__ == "__main__":
    sys.exit(main())
