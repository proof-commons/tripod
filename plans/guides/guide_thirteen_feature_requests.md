# Guide-13 Feature Requests: Closing the Guide-12 Negative Half

Filed 2026-08-22, from the Guide-12 handoff. The first three are the guide
gaps the [Guide-12 completion record](../history/guide-12-completion-report.md)
names as its remedy for the undischarged negative evidence layer, filed
here as feature requests against the guide series. Each is a gap in the
guide rather than a defect in the code: Waves 13 through 13e built the
whole negative-coverage machinery and ran it against a live target, and
in each case the guide is the reason the results cannot be counted
before the evidence is. Coverage ended at 100 of 211 with 71 of 72
negative rows outstanding, each naming its reason in code rather than
in prose (backlog item `T4-009`; full narrative in
[the backlog history](../history/backlog-history.md)). No amount of
first-party test writing substitutes for these three changes: a later
guide that wants the negative half discharged has to close all three
first, and inventing any of the missing links locally would be the
discharge-by-intent the coverage plan exists to prevent.

The addressee is the author of the numbered Guide 13 — or of whichever
later guide takes up the negative half. The
[Guide-13 concept](guide_thirteen_concept.md) already enumerates its
own negative cases the same way Guide 12 did, as lists of case names
under (`sec:guide13:coverage`), so each request below states what the
numbered guide should carry to avoid re-importing the gap.

## Request 1: a relation, a mutation class, and a boundary per negative row

Guide 12's section-18 mutation tables, from
(`tab:guide12-exec:cardinality-vectors`) through
(`tab:guide12-exec:resource-vectors`), are lists of names only: no row
names the relation it violates, the semantic mutation class its change
falls in, or the boundary where the refusal should be observed. The
live campaign measured what that costs. Of the eight staged mutation
arms, only two could be resolved to a coverage row at all, each by
reading its own class name in the relation vocabulary —
`two-ash-outputs` to ASH-output cardinality above maximum, and
`successor-one-below-the-sum` to closed-asset conservation. Two arms
have no member in the negative mutation vocabulary at all
(`noncanonical-ash-ordering`, `witness-item-reorder`), a gap between
two authorities rather than a defect in either; two fit two published
classes equally well (`ordinary-wallet-u-output`,
`shorten-successor-and-grow-another-output`). And three boundary
claims taken from the matrix were proven wrong against the live target
and respecified on evidence: `wrong-sequence` and
`wrong-transaction-version` moved to the constructor's boundary because
no emitted program reads either field and no signature covers them, and
`successor-one-below-the-sum` moved to consensus-before-script because
the target checks per-asset conservation before it runs any script.

**Request.** Every negative row in a coverage table names three things:
the relation it violates, the mutation class that stages it, and the
boundary where the refusal is observed. With those three the link from
an executed arm to a coverage row is read from the guide; without them
the link must be invented, and Wave 13d declined to invent it — which
is why six script-path refusals discharged exactly one row. The
numbered Guide 13 should carry all three per row from the start,
including for the authorization seams
(`rule:guide13:authorization-coverage`) its concept lists by name.

## Request 2: a first-party discharge condition for the negative half

Guide 12's section 19.1 (`rule:guide12-exec:positive-coverage`) lets
positive coverage of a compiler-static or backend-structural relation
use typed structural evidence instead of inventing target execution.
Section 19.2 (`rule:guide12-exec:negative-coverage`) states no such
rule for the negative half, and its discharge conditions — a complete
mutated target transaction, an executed carrier, an observed target
rejection — are three things a compiler-static relation cannot have.
The 18 first-party negative rows (10 compiler-analysis, 8
emitted-structure) are therefore undischargeable as written: the guide
is the reason before the evidence is. A boundary being first-party is
not the same claim as a first-party test discharging the row.

**Request.** Either state a first-party discharge condition for the
negative half — which typed refusal, driven at which layer, under what
independence requirement, counts as the row's evidence — or declare
the 18 rows out of scope explicitly. The evidence underneath is
independently thin (`T4-009` enumerates it per class: live typed
refusals no compiler-level test drives, a lifecycle exit refused only
at another layer, refusals made unrepresentable rather than observable,
predicates with no error at all), so a stated condition would also
demand new tests. The request is for the rule, without which no such
test can count.

## Request 3: the ABI-validation entry point

Guide 12's section 16.5 (`rule:guide12-exec:report-roles`) names an
ABI-validation result among the report roles, but no ABI-validation
entry point exists, and no coverage requirement is indexed at the
constructor's boundary. The live campaign produced two arms that
belong exactly there: `wrong-sequence` and `wrong-transaction-version`
expect a refusal before any target sees the bytes — no emitted program
reads either field, no signature covers the sponsorless form, and
consensus admits that form at either version — so their refusals are
the constructor's, and they have no row to land on. What actually pins
both fields, the constructor writing the ABI's values against a
request carrying no field to argue with, is tested; what is missing is
the guide-side place for that evidence to count.

**Request.** Specify the ABI-validation entry point: the
constructor-boundary classification through which a pre-target refusal
is reported, and the indexing of pre-target negative requirements at
that boundary. Section 16.5 already reserves the report role; the
entry point it names should exist, and the rows that can only be
answered there should be indexed there.

## Request 4: absence-preserving executor resource figures

Wave 12's live-transfer resource study found a protocol defect below the typed study: operation responses make no script or initial-stack measurement, yet the native executor serializes `"script_bytes": 0` and `"initial_stack_items": 0` in the ordinary operation writer (`scripts/elements-native-executor.py:5806-5808`) and again when the step did not happen (`scripts/elements-native-executor.py:5843-5845`). Those zeros are wire facts, so a first-party decoder cannot distinguish no figure from a measured zero; that is the absent-as-zero substitution section 18.4 forbids when it says no absent observation is read as zero or as agreement.

**Request.** Make each executor resource figure that may be unavailable preserve absence on the wire, as JSON null or an omitted field under a specified protocol rule, and decode it into a presence-bearing first-party type rather than a numeric default. `script_bytes` and `initial_stack_items` remain numbers only when the step actually measured them; an operation step that made no measurement and an infrastructure failure carry absence, so neither can be counted as a zero observation or agreement.

## What the first three unblock

The 72 negative rows classify as 48 target-executable (one discharged),
18 first-party, and 6 unreachable because their evidence is an external
report nobody has written. Request 1 unblocks the target-executable
column, Request 2 settles the first-party column one way or the other,
and Request 3 gives the pre-target arms their landing place. The
external-report column is out of any guide's scope and is listed only
so the census stays honest.
