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

## Errata

**Section 12.3 count.** A Wave 9 brief said the guide's may-not-select list had thirteen items; the authoritative guide lists fourteen, and the workspace implements all fourteen. The erratum is the brief's count, not the guide or implementation.

**Section 15.4, wrong constructor schema.** The row should read: wrong constructor leaf schema — a constructor-derivation refusal, raised when a leaf names another representation, when a required coordinator or member leaf is absent, or when a leaf serves no admitted shape. Empty-leaf-set evidence stays classified under key-path escape, which is its own row. Section 15 states the row as a name only; the boundary came from the executable transcription, which filed it at the linker. That was wrong on arrival: the static leaf schema is validated by tapscript's constructor derivation, and the linker receives sealed constructor and bundle values whose only construction path has already checked it. Preserving the linker boundary would have required exposing an unchecked constructor, admitting test-only corruption as canonical input, or revalidating a state the public types make impossible — the first weakens the seam, the second fails the canonical-malformed-input requirement, and the third creates two authorities. The workspace mints a constructor-derivation boundary and a static-constructor-schema mutation layer rather than reusing the ABI-construction boundary, whose wording covers the constructor but names a stage three layers downstream.

**Section 15.4, mixed operation program.** The row asks for a program mixing this operation with another, and no value of this architecture names one: compiler projection fixes the operation to transfer-live before a program is planned, the leaf-role vocabulary admits coordinator and member roles of one transfer representation and nothing else, the constructor and the linker consume only that role type, and the request cannot select a program at all. It is an operation-vocabulary closure invariant owned by compiler projection and backend constructor typing, not a stageable refusal, so it is outside section 4.2's staged-refusal denominator and is not evidence of any kind. Foreign-operation effects on a target remain the business of the issuance and destruction rows of section 15.5 and the root and specialized-event rows of section 15.7. A future guide should add a concrete linker-negative row here only if a future constructor deliberately admits a sum vocabulary containing several operation roles — following the mixed-representation model, where both alternatives are nameable and the refusal is real.

**Section 15.5, amount outside semantic domain.** Ruling: the ceiling is enforced by the blockchain, the same class as amount-in equals amount-out conservation, and the row's boundary is the target's rather than the request path's. The protocol amount domain is exclusive above two to the fifty-first; the reviewed target refuses a stated amount above twenty-one million coins in its transaction check, before any script runs, so every amount outside the protocol domain is above the target's own bound. The transaction layer's protocol value type deliberately gains no ceiling check: the ruling settles who enforces the bound, not whether the enforcement has been observed, and the row stays typed-blocked behind the absent accepting control like every other target-negative.

**Section 5.3 / Section 15.2, the private-merge shape is unconstructible on the confidential lane.** The split-and-merge rule (`rule:guide13-exec:split-merge`) states merge as inputs at least two and outputs exactly one, and Section 15.2's private safety matrix carries a private-merge row asking that at least one valid private merge accept. On the confidential-committed lane those two are unsatisfiable together as the reference registry is built. **The ground recorded here on arrival was too strong, and is corrected.** It read that a single-output confidential transaction has no free blinder to balance. It has no free blinder, which is true, and it needs none: the Pedersen tally requires only that the output blinders sum to the input blinders, so where there is exactly one output that output's blinder is forced to the input blinder sum — and a forced value is a solved value, not an impossible one. Consensus admits the shape. What refuses it is the registry's construction model, which solves one balancing output against at least one freely chosen other and so has nothing to solve against when there is no other. That is a property of the model, and this erratum should not have stated it as a property of confidential balance. The derivation, the pinned tally rows, and the removal path are recorded in the shape-possibility register (`rule:shapes:two-output-floor`). The reference fixture registry states exactly that floor: `ConfidentialFixtureRegistry::register` refuses any manifest with fewer than two outputs (`RegistrationRefusal::OutputSetTooSmall`), on the ground the code gives that a balance needs at least two and that exactly one must be balancing — which restates the construction model rather than the balance rule, and is the same conflation this correction removes. A private merge is one output, so it cannot be registered, materialized, or submitted, and the private-merge row cannot move by construction; it stays unmoved and typed. The scope of this erratum is the merge-predicate-versus-registry conflict only. Neither rule is at fault and neither is relaxed: the outputs-equal-one predicate is a correct property of the explicit split/merge lane, where a merge with no confidential balancing output is expressible, and the two-output floor is a sound property of the reference registry's construction model, which is what it is a property of. A future guide has three options rather than the two first recorded here. It should scope the private-merge row to a confidential merge that still carries a balancing output — inputs at least two and outputs at least two, one of them balancing — or record the private-merge row as inapplicable to the confidential-committed representation and satisfied on the explicit lane alone, or, now that the ground is corrected, extend the registry with a single-output fully-solved balancing form: a manifest whose one output is balancing and carries no freely chosen blinder, taking the input blinder sum directly, with the parity search degenerating to a well-formedness check and the two-output case left bit-for-bit as it is. That third option admits the private merge as the guide's own predicate states it, and it is the only one of the three that does. It carries a condition the other two do not: **the forced blinder can be zero, and a zero blinder hides nothing.** A merge's single output takes the sum of the consumed coins' blinders, so merging the two halves of this workspace's inverse-pair dual-parity predecessor — whose blinders cancel by construction, which is what makes it an inverse pair — forces that sum to zero, and the output commitment is then exactly the value times the value generator: a point anyone can recompute from a guessed amount, carrying a blinded output's form and none of its hiding. The tally still balances and the transaction is still valid; what fails is confidentiality, silently. A predecessor whose blinders do not cancel avoids it, so the third option must either require a non-canceling predecessor or refuse a solved zero blinder outright. Nothing above is implemented and neither rule is relaxed by this correction; the private-merge row stays unmoved and typed exactly as recorded.

**Confidential-funding guide §10.5 step 4, proof-negatives attributed "each to its own layer".** The fourth restart step of the confidential-funding execution guide (`task:guide-ctf-exec:restart-order`) asks for the wrong-blinder, missing-rangeproof, and malformed-rangeproof cases "each attributed to its own layer". That layer attribution is undischargeable from the reviewed target, because the target emits one identical refusal for all three. The balance check the pinned tip queues at src/confidential_validation.cpp:364 ("// Check balance", a CBalanceCheck) runs before the range-proof loop at :368, both inside VerifyAmounts, and the mempool path (src/validation.cpp:1097 passing pvChecks=nullptr) runs each queued check inline in source order and returns at the first failure. Whichever check fails, VerifyAmounts returns false and src/consensus/tx_verify.cpp:250-251 reports the single string bad-txns-in-ne-out / "value in != value out" as TX_CONSENSUS: the internal SCRIPT_ERR_PEDERSEN_TALLY and SCRIPT_ERR_RANGEPROOF codes are discarded inside VerifyAmounts and never leave it. The first-party observed-layer vocabulary mirrors this with a single member, `ObservedOutcomeLayer::ConsensusRejectionBeforeScript`, covering all three. Attribution is therefore by the mutated construction field — the value-commitment field for the wrong blinder, the range-proof bytes for the two range-proof cases — and not by any target layer the three cases could differ on. The scope of this erratum is the step's layer over-specification only. The step-3 ordering claim — that the balance rule is checked before the range-proof rule, so a wrong-blinder mutant of a balance-valid control is refused at the balance layer — is correct at the pinned tip and is not the subject; a future guide should replace step 4's per-layer attribution with per-field attribution against a single consensus-before-script layer.
