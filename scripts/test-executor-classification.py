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
import os
import sys

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


TESTS = [
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
