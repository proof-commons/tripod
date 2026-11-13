# Tripod Implementation Backlog

> **Status:** ACTIVE
> **Current gate:** Phase 7 — Burn, ASH, and clear
> **Current condition:** Phases 1 and 2 are complete. The target-independent compiler constructs one deterministic, validated scoped analyzed program for the complete pilot scope, factorized per operation, with relation-owned requirements, a corruption-resistant assembly validator, and independent assembly oracles; the Phase-2 exit gate record is in §2.11, including passed document reproducibility. Phase 3 then delivered the Guide-8 target foundation, the Guide-9 native primitive gate, the Guide-10 constructor and wide-floor prototypes, and the Guide-11 declassification result, each with its own gate record. Public declassification is no longer an open blocker: an initial policy is selected — an explicit boundary reached by owner-authorized normalization, with the direct and public-committed paths deferred against three named target blockers — and the gate record is in §2.4. Phase-3 exit is recorded: the Guide-12 preflight register closed all sixteen seventh-review rows, and (`gate:phase3:exit`) was checked green on that tree with the record in §2.6. Phase-4 exit is recorded in §2.8: compact ASH ran end to end against a live target with twelve accepted and matched submissions and honest coverage stated at 100 of 211. Phase-5 exit is recorded in §2.20 and §2.21: the live-transfer campaign closed with the amended exit gate met on a countersigned assessment. Phase 6 — the STATE constructor and maturity announcement — was chartered against Guide 14 as the owner supplied it and archived it verbatim (`T6-001`), with Wave 0 — the tenth static review's disposition and the Phase-5 handoff revalidation — run and closed out in the required order (`T6-002`, `T6-031`) and the owner-issued conceptual preflight register binding Waves 1–13 (`T8-001`), with Wave 13 closed under `T11-147` and the gate record at (`sec:phase6:gate-record`) reading §23 NOT PASSED on five conditional conjuncts and the register fully discharged. Phase-6 exit is recorded in §2.26 and on the Phase-6 card's exit assessment at (`sec:phase6:exit-assessment`): the gate met on its amended text. The current phase is Phase 7 — burn, public ASH and permissionless clear — with Guide 15 chartered at `plans/guides/guide_fifteen.md`; Wave 0 opens on its recorded entry obligations and closes inherited debt and preflight before new semantics begin.
> **Next gate:** Phase 7 — Wave 0 inherited debt closure and preflight (`gate:guide15-exec:wave0`) under the chartered execution guide at `plans/guides/guide_fifteen.md`
> **Authority:** Current execution queue only. The specification, the realization document, typed architecture, implemented ADRs, accepted decisions, package contracts, phase cards, and accepted research results take precedence.

This file contains only:

- the current repository and readiness state;
- the latest static-review basis and open findings;
- compact historical milestones and gate records;
- the active Phase-2 compiler queue;
- immediate algorithm preparation;
- dependency posture;
- verification and clean-tree requirements.

Detailed historical implementation narratives belong to Git history, annotated
tags, phase cards, ADRs, accepted decisions, and focused evidence records—not
to the active backlog.

---

## 1. Backlog contract · `sec:backlog:contract`

### 1.1 Authority · `rule:backlog:authority`

This backlog sequences work. It is not protocol, realization, compiler, target,
ABI, evidence, deployment, or release input.

When artifacts disagree:

| Subject | Owner |
|---|---|
| abstract economic interface | the Attestation specification |
| realization meaning, invariants, obligations, and residuals | the realization document |
| finite registries and stable architecture IDs | typed architecture |
| current executable reference behavior | executable model |
| repository engineering policy | implemented ADRs |
| accepted cross-package implementation direction | planning decisions |
| package boundary | package contract |
| phase entry, deliverables, and exit | phase card |
| unresolved design question | research note |
| current task ordering | this backlog |

A lower owner never overrides an upper owner on the upper owner’s subject.

### 1.2 Status vocabulary · `tab:backlog:status`

| Status | Meaning |
|---|---|
| **TODO** | Ready when its named dependencies are complete. |
| **IN PROGRESS** | Actively being implemented, reviewed, or verified. |
| **BLOCKED** | A named dependency or correctness defect prevents safe progress. |
| **PARKED** | Deliberately inactive until a concrete consumer or measured need exists. |
| **DONE** | Implementation, focused evidence, required gates, documentation, and clean-tree evidence are recorded. |
| **DROPPED** | Deliberately not implemented; rationale and replacement are recorded. |
| **HISTORICAL** | Immutable evidence about an earlier revision; not a claim about the current checkout. |

Code resembling an intended result is not sufficient for `DONE`.

A static-review finding remains open until one of these occurs:

1. the finding is reproduced and fixed with focused coverage;
2. a typed proof shows the reported state is unconstructible;
3. reproduction shows the finding is false;
4. the owning policy or assurance claim is deliberately corrected.

“Existing tests pass” does not close a finding unless a named test reaches the
reported path.

### 1.3 Priority vocabulary · `tab:backlog:priority`

| Priority | Meaning |
|---|---|
| **P0** | Can manufacture false release, deployment, provenance, or semantic evidence. |
| **P1** | Trusted semantic/evidence boundary defect or active phase blocker. |
| **P2** | Required correctness, determinism, publication, identity, or documentation work before phase exit. |
| **P3** | Maintainability or evidence-quality work required by the active gate. |
| **POST** | Later-phase work that does not block the current gate. |

### 1.4 Definition of done · `rule:backlog:done`

An implementation task is `DONE` only when it records:

1. implementing source files;
2. focused positive and negative tests;
3. affected ADRs, decisions, package contracts, phase cards, or research notes;
4. exact verification commands and outcomes;
5. generated-publication and label impact;
6. semantic, identity, schema, and migration impact;
7. dependency impact;
8. final clean-tree output.

A review finding additionally records whether reproduction confirmed,
refuted, or reclassified it.

A dependency task additionally records:

- selected source and exact version;
- enabled features;
- transitive graph;
- licence;
- MSRV;
- unsafe boundary;
- determinism and parallelism implications;
- advisory status;
- lockfile impact.

A research task additionally records:

- exact question and constraints;
- prototype and tool versions;
- accepted and rejected candidates;
- positive and negative evidence;
- measurements;
- permanent implementation handoff.

### 1.5 Identity admission · `rule:backlog:identity-admission`

No new digest or digest-bearing field enters merely because an object is
important.

A proposed identity must satisfy
(`[ADR021-req:identity:admission-record]`) and name:

- the complete typed object or exact bytes;
- owner and producer;
- a present consumer;
- the exact decision the consumer makes;
- assurance class;
- canonical projection and encoding;
- domain separator and recipe identifier;
- stale conditions;
- migration behavior;
- explicit non-claims.

No named consumer means no digest. No distinct decision means no digest.

---

## 2. Review basis and current evidence · `sec:backlog:review-basis`

### 2.1 Canonical review record · `tab:backlog:review-basis`

The [static-review index](reviews/README.md) is the review record of record
and always names the latest indexed review. The per-finding disposition
register for the Guide-14 arc lives in
[Guide 14 §4](guides/guide_fourteen.md)
((`tab:guide14-exec:preflight`)). Review facts — the reviewed tree, file
census, exclusions, and verdict — live in the indexed review texts, not
here. A review's findings are evidence about its exact tree.

### 2.2 Earlier review basis

Every earlier reviewed tree is listed in the same canonical
[static-review index](reviews/README.md). Historical reviews are evidence
about their exact trees, not current-checkout evidence.

### 2.3 Historical gate records

The completed gate records for Phase 1, the T1–T5 remediation, the P2-004
compiler-input batch, and Guides 2 through 10 stood here as §2.3–§2.14.
They are archived verbatim, with their original numbering and labels, in
[backlog history](history/backlog-history.md), so a reference to §2.8 or
§2.11 anywhere below resolves to the heading of that number there.

### 2.4 Guide-11 declassification gate (archived)

Closed and archived verbatim, with its label, in
[backlog history](history/backlog-history.md) beside the earlier
records; §2.4 references below resolve to the heading of that number
there.

### 2.5 Draft-promotion gate · `gate:backlog:draft-promotion`

The 2026-08-19 batch executed the user's promotion ruling: the
identity-adjudication and interchange drafts take priority as the
disciplines, ADR-016 is deleted rather than kept superseded, every
overlap is deleted rather than restated, and ADR-010 stays untouched as
a different scope. Wave 1 minted [ADR-021](../adr/021-identity-adjudication.md)
carrying only adoption data — the single local recipe convention, the
current-identities table, the separation migration verbatim, the
ADR-011 amendment linkage, and three recorded divergences — and
retargeted all ten imported citations of the retired owner; the
released realization contract took one prose sentence with a three-way
proof no identity moved. Wave 2 minted
[ADR-022](../adr/022-interchange-conventions.md) adopting the
fourth-edition interchange conventions wholesale, stated the ADR-010
boundary one-directionally, restated the executor wire-frame stop from
the identities register with its revisit condition intact, and
discharged DI-005 — the integration queue of §13 is closed. CI-005 was
recorded in the same batch under the user's wall-time ruling.

Gate, content-scoped per the cadence ruling and CI-005: the light lane
registry passed 9 lanes with the two test lanes skipped by design, 331
cumulative seconds inside the runner, 5 m 31 s wall; the corpus checker
oracle passed all four checkers in 5.6 s on the merged tree; the full
workspace suite ran green once, inside Wave 1, immediately after the
batch's only Rust-touching edits, and the delta since is markdown only.
No identity moved: the generated manifest is byte-identical and both
registers regenerate byte-identically. Weight: this record's room came
from archiving the closed Guide-11 gate record to history per the
CI-004 pattern, 9470 bytes net. Lane wall-times, recorded per the
wall-time ruling: Wave 1 two stints (interrupted by a harness restart,
resumed with work intact), final stint 9 m; Wave 2 two stints, final
stint 20 m 19 s; both workers' reports carry their own lane timings.
Found in passing and owed to CI-005: the golden-capture script prints
each checker's exit status but exits 0 itself, so a chained commit can
proceed past a red checker — it did once here, caught on the next run;
the archival also taught that a block moved into history must have its
relative links re-based, which the checker catches as broken links.

### 2.6 Guide-12 preflight and Phase-3 exit gate (archived)

### 2.7 Guide-12 build boundary (archived)

### 2.8 Guide-12 Phase-4 exit gate (archived)

The three Guide-12 gate records are archived verbatim, with their
labels, in [backlog history](history/backlog-history.md) under the
Guide-12 records group; a reference to §2.6, §2.7, or §2.8 below
resolves to the heading of that number there. Phase 4 exited at §2.8:
twelve accepted and matched live submissions, honest coverage 100 of
211, the section-29 audit at 94 of 105, the local gate 47 of 47 with
byte-identical document reproduction, and the runtime half of
`G12-R04` discharged from the wave transcripts. Its conditional re-audit still fires when the conservation, normalization, or lifecycle lanes next run live; the intermediate confidential-funding guide's evidence-restart wave now owns that expected firing point.

### 2.9 Guide-13 batch gate (archived)

The Guide-13 batch gate record is archived verbatim, with its label, in [backlog history](history/backlog-history.md) under the Guide-13 records group, beside the nineteen Phase-5 wave narratives whose live rows compress to their evidence; a reference to §2.9 below resolves to the heading of that number there. The batch closed 2026-08-24 with fourteen waves merged and the full repository suite green on the shared instance at the batch tip, 48 of 48 meson lanes in 13 m 12 s. The Phase-5 result is the honest stopped one the phase card records: the entire remaining distance to the Phase-5 exit is the two typed blockers — the owner sighash and confidential predecessor funding — both owned by the chartered intermediate confidential-funding guide.

### 2.10 Confidential-funding batch gate (archived)

The confidential-funding batch gate record is archived verbatim, with its label, in [backlog history](history/backlog-history.md); a reference to §2.10 resolves to the heading of that number there. The batch closed 2026-08-25, 48 of 48 lanes green in 567.1 seconds at the batch tip, with both Guide-13 blockers this guide was chartered to own resolved and the executed guide archived in [the guides directory](guides/guide_confidential_funding.md).

### 2.11 Interlock resolution and handoff gate (archived)

The interlock-resolution gate record is archived verbatim, with its label, in [backlog history](history/backlog-history.md); a reference to §2.11 resolves to the heading of that number there. The step-3/step-4 interlock was ruled unwarranted and discharged with real evidence 2026-08-25, 48 of 48 lanes green in 579.9 seconds, the restart order then typed-stopped at step five.

### 2.12 Shape-wave closing gate · `gate:backlog:ctf-shapes`

The confidential-funding restart's step-five stop was cleared 2026-08-25 (T5-039): split, many-to-many representative, and several-distinct-owners were built as multi-output and multi-input fixtures and accepted by the pinned node, moving six of the ten positive private matrix rows onto observed acceptances of their own shapes; the registry parity search, which no fixture wider than two outputs could pass, was repaired with the two-output rule preserved. The one-output shapes stay typed at the registry floor, the merge-predicate conflict is filed as a guide erratum, and the order now stops typed at step six on the sponsor signer dependency. Two handed-up questions were ruled: the step-six/step-seven chain rule is affirmed strict, and the deterministic-fixture-openings row awaits a determinism-standing member rather than a guide repair. Full suite green at the merge tip: 48 of 48 meson lanes, zero failures, 490.8 seconds of wall time, both validators clean and the register byte-stable. The Phase-5 exit remains unreachable — the explicit positive table is answered in zero of sixteen rows — waiting on the explicit-lane runs, the sponsor dependency, and the minimality pairs behind it. The full narrative is [the wave-seven closeout](history/ctf-guide-wave-7-closeout.md).

### 2.13 Shape-census gate · `gate:backlog:shape-census`

The consensus shape-possibility register landed 2026-08-25 (T5-040) on the discipline that the exact consensus-possible shapes be tracked and every limitation explicitly labeled, pinned, explained, and eventually structurally removed. Eight blinded shapes are censused with separate consensus and first-party verdicts — four observed-accepted on run-of-record identities, three consensus-possible but refused by first-party convention with named removal paths filed as T5-041 and T5-042, and fee-only consensus-impossible by the Pedersen tally — with the consensus half a computed predicate held against every recorded verdict and every refusal recomputed against the live registry. The register records that the two-output floor guards consensus only incidentally, and the private-merge erratum's overstated ground is corrected. The plans weight caps were raised in the same wave — hard cap to 1 MiB, soft target to 693 KiB by the same ratio, the motivating measurement recorded at the constant — and a latent fixture-sizing defect the raise exposed was repaired. Full suite green at the merge tip: 48 of 48 meson lanes, zero failures, 514.7 seconds of wall time, both validators clean and the register byte-stable. The register is [the shape-possibility reference](reference/shape-possibility.md).

### 2.14 Sponsor-arc gate · `gate:backlog:sponsor-arc`

The sponsor arc closed 2026-08-25 across two waves (T5-043, T5-044) with the first observed sponsor-signed acceptance on the live lane. The reserve asset and the fee-role program digest are runtime-threaded beside the protocol asset — the demonstration deployment keeps its literals and its identities, proven byte-identical where an intact baseline existed — and a sponsor-signed explicit control was accepted at a real node with the relay boundary crossed for the first time: the mempool allowed it before the mine, and the node read the bytes back equal. Three refusals were peeled to get there, each typed and each a first-party defect rather than a target property: the unauthorizing signature, the owner census that had no route to a sponsored message, and the fee-role digest that was a fixture constant no program hashes to. The never-raised live sponsor reserve-asset guard is now enforced, SponsorEnvelopeSignerAbsent is cleared at its defining site by its own rule leaving PredecessorConstructorAbsent the one carried residual, the step-7 ordering erratum is filed sharpened by the outcome, and no matrix row moved — the acceptance is a positive explicit control, which answers neither the sponsor-authorization negative nor the confidential sponsor-values row. One acceptance establishes the envelope wire, not production multi-party signing. Full suite green at the arc tip: 48 of 48 meson lanes, zero failures, 459.0 seconds of wall time, both validators clean and the register byte-stable.

### 2.15 Structural-removals gate (archived)

### 2.16 Explicit-table and fee-vocabulary gate (archived)

### 2.17 Sponsor-arc completion gate (archived)

### 2.18 Fee-completion and crossing gate (archived)

### 2.19 Closure-arc gate (archived)

### 2.20 Negative-half gate (archived)

### 2.21 Handoff-and-repair gate (archived)

The seven closed gate records of the section-15 shape, fee, negative-half
and handoff arcs (T5-040 through T5-072) are archived verbatim, with
their labels, in [backlog history](history/backlog-history.md) under the
live-transfer arc records group; a reference to §2.15 through §2.21 below
resolves to the heading of that number there. The arcs closed 2026-08-26
and 2026-08-27; the partition at their tip was 108 = 82 answered + 1
vocabulary-closed + 25 still required, each of the twenty-five a final
typed stop against a named gap, and every closing suite green on the
shared instance.

### 2.22 Forward-recapture gate · `gate:backlog:forward-recapture`

The native v2/revision-7 forward recapture closed 2026-08-28 across thirteen waves (`T7-001` through `T7-013`) under the owner authorization that chartered it end to end: run a native revision-7 capture with an enhanced recorder, let that capture become the run of record, and let every recorded-v1 value descend to history as immutable data — moved verbatim, never recomputed, because recomputing a recorded observation is backdating. Two further rulings rode the arc: the narrow schema-6 concept was approved after worked examples (a composite two-acceptance observation for the restart parity pair, witness-path-shape and committed-leaf-arrangement locators, a primary/support link split, and the multi-row semantic witness) and schema 5 was retired outright rather than kept historical, verified against the fact that no committed artifact ever persisted a schema-5 document. `T7-001` gave the executor a capture journal that clones exact request bytes before send and survives errors; `T7-002` built schema 6; `T7-003` the capture driver with its five duties and RUN-REPORT grammar; `T7-004` the enhanced recorder writing every capture before its gates run, with the legacy rendering byte-preserved beside it; `T7-005` retired schema 5. The capture itself took three attempts, each refusal a real catch: attempt one was refused by the recorder's own write-time completeness validator against five multi-operation ceremonies (fact-joining repaired as `T7-006`); attempt two passed 40 of 40 but was refused by two driver false refusals — the diagnostics subtree and a full-tip equality the binary cannot satisfy because elementsd reports a twelve-hex short revision (repaired as `T7-007` under the prefix rule, flagged to the owner without veto); attempt three is ELIGIBLE at the run-of-record suite, forty of forty outcomes, and its eighty-one files stand byte-verbatim in `packages/vectors/fixtures/native-v2-r7-run-of-record/` (`T7-008`, with the strict importer whose eight duties re-decode every byte). `T7-009` bound the corpus through the only retyping path: all forty-two recorded-observation-unbound rows moved atomically as twenty-four native-run, seventeen native-refusal, and one paired-relation observation, census `38+2+1+25+42=108` before, `82+1+25=108` after report validation, outstanding exactly twenty-five, with no v1 value as an equality oracle; its thirteen-round verify walk aligned the report validators to real Elements bytes family by family — leaf version 0xc4, the one-item Schnorr key-path witness with no control block, the real sponsored shape, the R01 0x08/0x09 declared-parity rule, and the exact authorized aliasing forms — always preserving refusal power. `T7-010` minted the two authorized Pending successors from typed corpus projections alone: the private-restart two-acceptance link and the atomic proof-bearing schema-2 record, whose one substantive walk stop was a false equality the corpus refuted — the issued asset is distinct from the predecessor coins' asset. `T7-011` performed the descent: `live_history_v1` became the sole historical namespace through re-exports that resolve to the original declarations, so byte preservation holds by construction; every consumer's current claim moved to typed corpus projections, every historical claim names history explicitly, the no-active-v1-reads audit passes, Q19 forward pins stayed active while the v1-inequality assertions descended — with the recorded fact that the fresh capture reproduces the historical maximum-inputs txid exactly, so v1-versus-v2 inequality is not an oracle and none was invented. `T7-012` finished the native harness's own succession: its six seams all proved to be current claims, the private-restart pending fallback died with the mint Recorded, and the recorder machinery stayed byte-compatible. `T7-013` repaired what only the full gate could see: the descent's new file was outside the ADR-014 declared census (a package-scoped verify is structurally blind to the meson census — filed as the arc's process finding), registered with a 227-declared-to-227-tracked empty diff, alongside one near-miss span rewording. THE CLOSING GATE IS GREEN: forty-nine of forty-nine meson lanes, zero failures, 883.7 seconds of wall time, the register unchanged at twenty-five, and the twenty-five still-required grounds fingerprint-pinned. The rows `T7-001` through `T7-013` flip DONE under this record.

### 2.23 V1-descent epilogue · `gate:backlog:v1-descent-epilogue`

The recapture arc's epilogue closed 2026-08-28 on one strategy — re-point, then delete to git history — across two waves. `T7-014` made the physical location match the naming: every v1 literal declaration moved byte-verbatim from its origin module into thirteen registered `live_history_v1` child files, the bridging re-exports deleted, the origin modules left v2-only with the forward records at `live_private_restart::forward_v2`; the raw classifier's fifty-two historical reads were re-pointed path-only, proven by reproducing the parent file byte-for-byte when the new prefixes are reversed, and the byte-equality regression blocks stayed identical to their pre-move bytes. `T7-015` then deleted the record from the working tree: the fifteen-file history namespace including `live_closeout.rs` (whose only content was the Wave 5–8 reconstructions), fifty-five archival tests including the three Q19-divergence archives, and every stranded v1-era helper; the raw classifier's `RecordedObservationUnbound` standings now carry kind facts only — names and the 24/17/1 split with no identities, details, or relation strings — while all three census identities hold byte-for-byte (`38+2+1+25+42`, `80+2+1+25`, `82+1+25`, each `=108`); `ProofBearingRunOfRecord` relocated type-only into the current module; four current tests re-fixtured from the validated corpus, the pair-arc width now decoding each acceptance's real transaction weight; and the zero-v1-values sweep checked all sixty-two unique hex payloads of the former namespace absent from source and tests, the two surviving strings carrying independent v2 provenance only. The keep set — rerun-day bytes and parser, the corpus `HistoricalV1`/`Revision6` arms, the forward records, the byte-unchanged live harness, closed plans prose — is green throughout, and the meson census stands at fifty-six declared equal to fifty-six tracked. Net movement: 524 insertions against 7,143 deletions. THE CLOSING GATE IS GREEN: forty-nine of forty-nine meson lanes, zero failures, 483.0 seconds of wall time. Git history is the sole archive of the v1 record; the native v2/revision-7 corpus is the only run of record in code. The rows `T7-014` and `T7-015` flip DONE under this record.

### 2.24 Typed-closure and erratum gate · `gate:backlog:typed-closure-errata`

The no-capture half of the negative-half closure program closed 2026-08-29 across three waves (`T9-001` through `T9-003`), chartered end to end by rulings settled before any wave opened: the four register adjudications — the member-then-coordinator leaf arrangement assigned to `member-coordinator-leaf-exchange` with the arrangement itself as separator, `wrong-coordinator` read as an arrangement fault and therefore an adjudicated duplicate, the §15.6 confidential-sponsor-values fault listing struck as the guide's own error against its §15.2 positive twin, and the ABI-validation row ruled mis-typed against a status vocabulary whose only member is `Candidate` — plus a batch erratum approval covering every boundary and ground retyping, each argued on the merits at its site with adjudication provenance in the landing commits. `T9-001` built the vocabulary: three typed-closure boundary kinds beside the operation-vocabulary closure — architecture closure with four no-catch-all proof arguments, typing correction, adjudicated duplicate — three standings outside the run-owed denominator with zero census buckets, report schema 7 whose rejection set now pins the historical 6 and the tripwire 8, and the `family_range_defects` fault validator registered unwired; census-neutral, both register pins untouched. `T9-002` consumed it: six rows left the run-owed register through their adjudicated members — the ASH destination closed by the constructor-table key, the surjection proof by the mandatory hybrid form, the fee-role carrier by the demonstration shape set with its deployment scoping stated, the witness reorder by the ABI order constant, the ABI-validation row by typing correction, and wrong-coordinator as the duplicate — register 25 to 19, buckets 4+1+1, the fingerprint re-pinned through the sanctioned discovery round, and the layerless set pinned at exactly seven named rows. `T9-003` closed six more through the first-party routes: foreign-sponsor-asset, two-sponsor-envelopes and unclassified-u discharged — the last by a genuine staged `family_range_defects` call demanding the exact `PositionUnaccounted` defect, repairing the real hazard that the discharge dispatcher copies the validator label without dispatching on it — and sponsor-member-unclassified, output-claimed-through-two-flows and confidential-sponsor-values recorded as first-party facts on their existing deciding tests; the retained amount row's false no-chain-mints-such-a-coin ground was struck for the honest `MutantBuilderOwed`; register 19 to 13, discharge census 34 to 37, fact census 3 to 6, answered 82 to 88, the fingerprint re-pinned to 0x053e_323b_8934_f2b5, every census identity crossing to 108 throughout. The verify walks ran 4, 3 and 5 server rounds; the two missed report consumers each round caught were repaired by recomputation, never by copying the observed value. The full gate first REFUSED the record: the three evidence rows pushed the maintained plans class 271 bytes over its hard cap, answered by the standing archival ritual — the seven settled live-transfer arc records (§2.15–§2.21) moved verbatim to history, reclaiming roughly 18.9 KiB. THE CLOSING GATE IS GREEN: forty-nine of forty-nine meson lanes, zero failures, 635.3 seconds of wall time. The register stands at THIRTEEN, every departure a typed closure or a first-party answer, and the thirteen remaining grounds are fingerprint-pinned; the capture half — eight drivable rows riding one revision-8 capture — is the arc's second act.

### 2.25 Tripod positioning wave · `gate:backlog:tripod-positioning`

The repository's self-description inverts: Tripod, a contract closure compiler, is the product, and the attestation contract is the exemplar that proves its build-out — the specification keeps its authority over the contract's meaning, and the record the contract manufactures stays the contract's product. The wave lands in ordered, gate-coherent commits: (1) positioning prose; (2) the imported-citation prefixes follow the documents they name — the specification's consumer prefix becomes `A-` (the paper's title carries the name) and the realization document's becomes `RZ-`, a version-free prefix that survives version movement; anchor names are hashed prefix-stripped, so neither hash nor the anchor-set pin moves. (3) the version- and frame-bearing filenames retire — `docs/attestation/realization.md`, `plans/labels/realization.md`, `plans/labels/specification.md` (a version in a filename would churn census, fixtures, and partition on every version movement), every path reference moving in the same commit. (4) THE ATOMIC RE-PIN: the specification releases at v1.0.0 (title *Attestation*, changelog comments removed under the released policy, PDF `Tripod_Attestation_v1.0.0.pdf`); the two own-division label areas become `attestation` and `realization`, so two anchor names move and the anchor-set hash is re-pinned by the recorded ceremony (retired value reproduced before the new one was taken); the manifest binding renames to `specification` (a hashed key, so the semantic hash moves in the same commit); and `realization_version` becomes the tracked binding `0.6.0-dev` under the rewritten law — the binding gate derives the expected value from the workspace version, the denotation gate pins the behavioural hash, unchanged across the wave and its regression witness. The gate test carries the scheme-migration record; the identity register carries DI-008. The behavioural hash is the wave's regression witness and must not move; the semantic and anchor-set hashes are expected to move exactly once, in the atomic commit, by the recorded re-pin ceremony.

### 2.26 Phase-6 exit gate · `gate:backlog:phase6-exit`

Phase 6 — the STATE constructor and maturity announcement — opened 2026-08-27 when Phase 5 exited, ran Guide 14's Wave 0 and Waves 1 through 13 under the conceptual preflight register (`T8-001`), and recorded its gate at the Phase-6 card's gate record (`sec:phase6:gate-record`) over `0.6.305-dev`, landed on `0.6.306-dev` under `T11-147`, with Guide 14's §23 NOT PASSED on five conditional conjuncts — 5, 7, 11, 13 and 14 — and the conceptual register fully discharged, fifteen rows CLOSED and none OPEN. It exited 2026-09-26 on the card's exit assessment (`sec:phase6:exit-assessment`), with its exit gate (`gate:phase6:exit`) met on its amended text: four of the gate's ten conditions — the bypass, wrong-code-subtree, resource and relation-coverage conditions — are amended in place, each narrowing its condition to what the gate record evidences and adding no evidence, the other six standing as written. `every_required_row_is_answered` is false, with 128 of the Safety matrix's 206 rows outstanding and ninety unlinked to a published relation, each stated as its own figure beside the amended gate. The residuals stay with their open rows: `T11-064` carries the region-scoping refit, §23 row 14 and Wave 11's C1, `T11-063` the closure-exact audit, `T4-009` the compact-ASH negative half and `T11-110` test-performance measurement; Phase 7 inherits the handoff as carried candidate evidence.

### The verification harness's two standing hazards · `rem:backlog:verification-harness`

A verdict is read from the report wrapper's own report line, never from a pipeline's shell status: the wrapper propagates its exit code faithfully, and a pipe to a filter truncates the status to the last stage's. And a single shared build-target directory is poisoned when checkouts of two different commits build the same crates into it, so a lane that moves between commits pins a target directory of its own.

### Publication metadata · `rem:backlog:publication-metadata`

The published documents carry their attribution and their terms: the attribution position names Proof Commons, a commons rather than a company, and the document set is dedicated to the public domain. A specification exists to be reproduced, cited and implemented, and an attribution condition taxes exactly that use, so the attribution that matters is the one the title page carries. The rights line states the dedication, the licence-URL field names the canonical deed for machine readers, and the document licence file holds the legal code verbatim.

### The crates carry the product's name · `rem:backlog:crate-naming`

Each crate manifest and the build system's project name the product, so the product the repository describes is the product its artifacts are named after. The name feeds nothing that hashes, exports or gates: the versioning derivation reads the version alone, the binary and library target names are their own, and the generated model artifacts never carried it.

### The realization's fixed body states its law · `rem:backlog:law-delimitation`

The realization's fixed body states the law without restating the era. The versioning law leaves its worked example to the masthead, which prints the live binding; the hash pins defer to the algorithm the attached envelope names; and the schema discriminants are read at their carriers — the envelope for the architecture manifest, the canonical query context for the wire, the bound profile for deployment. The workspace authors field names the publishing organization.

### The shared crate's acknowledgement · `rem:backlog:cli-common-thanks`

The shared command-line crate's README thanks the developers whose work inspired the tooling. Gratitude for inspiration, stated once, naming no repository and claiming no lineage: the crate's own manual remains the authority on what it does.

### The tip's tests assert their own consistency · `rem:backlog:tip-convergence`

The release profile's identity test derives its expected value from the profile itself rather than comparing against a hand-pinned constant, so the tree asserts its own internal consistency instead of a value a reader would have to take on trust. The paper is Attestation, the product is Tripod, and the protocol object is the attestation contract.

### The licence terms and their citation families · `rem:backlog:licence-convergence`

The code licence is stated once, in the licence file, and the manifest key, the toolchain licence rule and the dependency comments agree with it rather than restating it a second way. The dependency-review clauses name what a dropped feature pulled in and which rule refuses it, which holds under any workspace licence, and the toolchain record says why a copyleft workspace strengthens rather than relaxes the permissive-dependency rule. Documents stay under the public-domain dedication.

### The hash-citation audit the lint suite runs · `rem:backlog:hash-citation-audit`

The audit holds every published hexadecimal value to the identity rule, as a labels binary in the lint suite wired exactly as the forbidden-text check is: it asks git for the complete tracked set and requires every value to be described by a rule in the citation-families table, which names what each kind of value measures and the program that writes it. The audit places rather than recomputes — whether a digest is right belongs to the test that owns it — and what it establishes is that every value has an owner on the record.

### The cut the settled records carry · `rem:backlog:record-cut`

A planning record earns its place by what it establishes, so the passages that spent themselves on a document's own corrections are gone and what survives argues the result. A citation is a promise that a reader can fetch what it names, so the sites that cited tags no hub carries name the phase cards and the archived record as the durable record, with nothing hash-shaped substituted. The archive index states its rule for when a settled record may be edited at all, and states it there rather than here: a wave record that paraphrases a rule it does not own carries a second copy of it, and the copy stops being true the moment the rule is sharpened.

### The discipline every published value follows · `rem:backlog:hash-citation-discipline`

Every hexadecimal value the tree publishes declares how a reader establishes it: recomputable from bytes the tree carries under a named algorithm, resolvable to a referent a reader can fetch, or defined by something other than its author. A value that nothing regenerates, nothing resolves and nothing defines stands nowhere, including inside an encoded rendering, which a decode-and-assert check reads. The cascade above the captures recomputes from the bytes it is taken over, and each chosen constant is a derivation from a string published beside it.

### The separators spell the product's name · `rem:backlog:separator-convergence`

A domain separator is hashed input that identifies a recipe, and the product's name is part of that identity. Each separator spells it at the version it carries, because no projection, encoding or algorithm moved. The identities that follow those bytes are re-taken by the tree's own generator rather than edited, so the typed pin, the generated artifacts, the realization's envelope and the versioning gate carry one result.

## 3. Current repository state · `sec:backlog:state`

### 3.1 Implemented areas · `tab:backlog:implemented`

| Area | Current source state |
|---|---|
| Specification | Released *Attestation* paper, version owned by the paper source |
| Realization document | Attestation Realization with final architecture appendix and tracked version binding |
| `architecture` | Typed architecture, validation, semantic/behavioural hashes, deployment-profile scaffolding |
| `model` | Executable state machine, invariants, property/corruption suites, indexer and accounting projections |
| `realization` | Target-independent typed semantics for compact ASH and live receipt transfer |
| `compiler` input | Owner-revalidated architecture/realization binding, explicit scope, strict policy |
| `compiler` graphs | Scoped relation and expression DAGs with stable typed projections |
| `compiler` folding | Conservative checked constant folding with independent oracle |
| `compiler` source analysis | Typed operands, authenticatable source requirements, sponsor-value read rejection |
| `compiler` constructibility | Authorization-case analysis and source/constructibility weld |
| `compiler` disclosure | Inherited and added disclosure with typed reasons |
| `compiler` lifecycle | Representation choices and required-exit analysis |
| `compiler` proof search | Exact deterministic feasible-plan enumeration; fixed external-evidence requirements retained, representation modes static; reaccepted in §2.8 |
| `compiler` placement | Typed execution cases, discharge classification, carrier eligibility, exact feasible placement sets, layout requirements, independent placement oracle; gate record §2.9 |
| `compiler` coverage | Relation-indexed coverage requirements, typed mutation catalogue, coverage dependency graph with forbidden-cycle SCC policy, independent coverage oracle; gate record §2.10 |
| `compiler` analyzed program | Relation-indexed requirement bundles, factorized operation analysis, scoped analyzed programs with a corruption-resistant validator and independent assembly oracles; gate record §2.11 |
| `artifacts` | Generated-publication derivation, writer/checker, realization-document weld |
| `labels` | Owner-aware Markdown/Rust label graph, census, plan checks, register rendering |
| `cli-common` | ADR-010 streams, diagnostics, checker report/stamp publication, batch publication |
| `target-elements` | Reviewed typed target contract, capability and evidence-role adapter, typed instruction core, welded development binding |
| `tapscript` | Compact-ASH proof patterns, abstract stack and failure-state validator, static ASH constructor over nine shapes, candidate relocatable bundle |
| `linker` | Typed symbols, references, and relocations; deterministic taptree; candidate linked bundle |
| `transaction` | Candidate compact-ASH transaction and witness ABI with the safe construction pipeline |
| `vectors` | Canonical fixtures, evidence plan, negative-mutation machinery, live target execution, and the candidate resource study |
| `target-elements-conformance` | Native-executor protocol and supervision, constructor and commitment oracles, conformance reports |
| Meson | Explicit source census, stamp-backed checks, mocked document graph |
| Security policy | Public-data interfaces and external execution-environment boundary |
| Path policy | Central tracked-mode audit, lexical output roles, explicit host-filesystem non-claims |

### 3.2 Current published identities · `tab:backlog:identities`

Exact hash and version values are deliberately not duplicated here. Read each
identity from its authority:

| Identity | Authority |
|---|---|
| Specification version | `papers/attestation/main.tex` and `sections/00_title.tex` |
| Architecture schema, semantic/behavioural algorithms and hashes, anchor-set hash | `packages/model/generated/architecture.json`, checked against typed architecture |
| Realization version (tracked binding) | `docs/attestation/realization.md` masthead |
| Attestation wire schema | typed model/architecture constants |
| Deployment-profile schema | typed architecture constants |

The generated architecture publication is authoritative. Planning prose must
not become a competing identity source.

Architecture finality does not imply:

- complete realization scope;
- compiler completeness;
- target support;
- linked bundle or ABI existence;
- deployment evidence;
- production readiness.

### 3.3 Not implemented · `tab:backlog:not-implemented`

```text
public complete compiler-analysis API beyond the target-requirement boundary
compiler-plan identity
direct authenticated opening and public-committed representation
public opening capsule
production target-native evidence

tripod-release

independent deployment observers
production deployment
production signer or wallet
production key management
```

### 3.4 Readiness statement · `rem:backlog:readiness`

```text
Attestation specification:          published
Realization contract:              published
Typed architecture:                final and pinned
Executable model:                  implemented
Typed realization pilots:          implemented
Compiler input/graph/folding:       implemented internally
Compiler source/disclosure/lifecycle:
                                    implemented internally
Compiler exact planning:            implemented and reaccepted
Compiler placement/layout:          implemented internally, section 2.9
Compiler coverage:                  implemented internally, section 2.10
Complete analyzed pilots:           implemented internally, section 2.11
Compiler target-requirement boundary:
                                    public and validated, section 2.12
Typed Elements target contract:     reviewed and welded, section 2.13
Capability and evidence adapter:    implemented, section 2.13
Typed instruction core:             implemented, section 2.13
Development target-native evidence: recorded, section 2.13
Production target evidence:        absent
Backend patterns/constructor:       implemented (candidate), section 2.8
Linker and transaction ABI:        implemented (candidate), section 2.8
Vectors and live target execution: recorded, section 2.8
Independent deployment evidence:   absent
Production deployment:             absent
Phase-2 exit gate:                 passed and recorded, section 2.11
Phase-3 exit gate:                 passed and recorded, section 2.6
Phase-4 exit gate:                 passed and recorded, section 2.8
```

Current packages are public-data tools. They do not legitimately accept private
keys, seed material, signing nonces, blinding factors, private openings,
credentials, or production authority.

---

### Wave-1 follow-ups · `tab:backlog:wave1-followups`

|Item|Status|Required next evidence|
|---|---|---|
|Attestation derivation profiling|TODO|Profile the roughly threefold attestation-derivation regression between `0.7.96-dev` and the bite-1.2 candidate. Retain both runs, w20b1-keyed-cost-20261110T171100Z and w20b1-keyed-default-20261110T183600Z, at (`rec:w1b12:cost-results`) and (`rec:w1b12:default-cost`); compare only within each campaign because identical-code baseline medians moved. No numerical threshold or causal explanation is adopted.|
|Runner cgroup identity retention|TODO|Before any future build-cost comparison, correct `scripts/keyed-cost-kranich.py` to retain each counter sample's cgroup path and inode and establish that start/end counters share a scope. Review the correction before measurement. Current build CPU/stall deltas remain unadmitted, never clamped; both campaign records retain the raw defect.|
|Bite-1.3 concrete-member controls|DISCHARGED IN MODEL CONTEXTS|`packages/vectors/src/keyed_stack_model/members.rs::production_bundle_checks_both_clauses_for_every_member_and_a_single_changed_member` checks both clauses for every member and refuses a single wrong member beside its control (`rec:w1b13:closed`). Serialized-transaction and final-byte counterparts remain bites 1.4 and 1.6; symbolic quantification alone was not substituted for these subjects.|
|Cryptographic report assumptions|TODO|Add an assumption kind for conditional NUMS/discrete-log, preimage, second-preimage/collision and target-authentication premises in the keyed report form, with binding/refusal controls and independent review. They remain carried in the records, not discharged or encoded by current headers (`rec:w1b15:gate-review`).|
|Gate-of-record journal retention|DONE|The loop-owned gate now copies each run’s per-test journals into its evidence directory as `cargo-test-json.tar.gz`, retaining digest, file count and size in the gate log; the `0.7.123-dev` journals were saved before replacement. This preserves future passing durations; the first `0.7.121-dev` timing gap remains explicitly unavailable under (`rec:w1b15:gate-review`).|
|History phase tracing|TODO|Convert test-only printed phase timers to scoped tracing spans while preserving flushed diagnostics when a child times out; the current exception and its reason are recorded in `packages/vectors/TRACING.md` and (`rec:w1b15:gate-review`).|
|Test-directory collision|REPAIRED|The shared vectors allocator uses exclusive temporary-directory creation with collision retry; an eight-thread, 512-directory same-label control checks distinct empty owned directories (`rec:w1b16:test-directory`). The `0.7.124-dev` gate remains RED; the repair's covering gate is separate evidence.|

## 4. Compact historical record · `sec:backlog:history`

### 4.1 Completed phases · `tab:backlog:completed-phases`

| Phase | Status | Durable record |
|---|---|---|
| Phase 0 | HISTORICAL | the recorded baseline and identities on [the Phase-0 card](phases/00-baseline.md) |
| Phase 1 | HISTORICAL | the completion evidence on [the Phase-1 card](phases/01-realization.md), and the gate record in [the backlog archive](history/backlog-history.md) §2.3 |
| Phase 2 | HISTORICAL | Guide-4 through Guide-7 gate records, §2.8–§2.11 |
| Phase 3 | HISTORICAL | exit recorded in §2.6, the Phase-3 card, and the [closed task rows](history/backlog-history.md#exited-phase-task-rows) |
| Phase 4 | HISTORICAL | exit recorded in §2.8, the Phase-4 card, [the Guide-12 completion record](history/guide-12-completion-report.md), and the [closed task rows](history/backlog-history.md#exited-phase-task-rows) |
| Phase 5 | HISTORICAL | exit recorded in §2.20 and §2.21, the countersigned assessment on the Phase-5 card, [the review archive](reviews/review-9-0.5.17-dev.md), and the [closed task rows](history/backlog-history.md#exited-phase-task-rows) |
| Phase 6 | HISTORICAL | exit recorded in §2.26, the exit assessment on the Phase-6 card, the gate record at (`sec:phase6:gate-record`), and the [closed task rows](history/backlog-history.md#phase-6-closed-task-rows) |
| Phase 7 | Active | Wave 0 has its completed baseline at (`rec:backlog:wave0-baseline`), and Wave-1 realization routes are adopted as rulings 158 to 165 at (`rule:phase7:wave1-realization-rulings`), with other selections awaiting their stated acceptance conditions; the semantic-home probe refuted its candidate at (`rec:phase7:wave1-profile-probe`), so that candidate accepted no home and left bite 1.1 open; rulings 166 to 170 govern the second engine probe at (`rule:phase7:wave1-engine-rulings`), accepting no home, route or realization. Current-input outpoint inspection is accepted as the born-first realization mechanism because the four admitted subjects and the pinned index/flag reading meet ruling 160's acceptance condition (`rec:phase7:wave1-birth-admission`); this admits no issuance or peg-in coverage, native stack metrics or later measurement, and that individual admission did not close the bite-1.0 gate. Authenticated current-program equality with a sole key-committing public leaf is accepted as the continue-first realization mechanism because the admitted measurements (b) and (c), including the full leaf/key-path census and stated conditional premises, meet ruling 161's acceptance condition (`rec:phase7:wave1-keyed-admission`); those premises retain their conditional standing, no native stack metrics, issuance or peg-in coverage, funding floor, cost, scaling or later measurement, or unconditional key-path impossibility is admitted, and that individual admission did not close the bite-1.0 gate. Route B stands for the probe subjects at (`rec:phase7:wave1-engine-probe`), with code retained as evidence only pending the required refit and no home, stable types or realization accepted. Measurement (d)'s cost packet, isolated-continuation weight and witness-standardness refusal are admitted at (`rec:state-constructor:keyed-cost`), with native interpreter metrics and the consensus push-size boundary unadmitted and the bite-1.0 gate still open at that admission. Measurement (e) funding observations are admitted at (`rec:state-constructor:keyed-funding`); ruling 162 still requires repetition on final bytes. Measurement (f)'s admitted measured range and worst-witness evidence meet ruling 163's acceptance condition, so the 1,564-keyed-handle MERGE bound is accepted as construction policy derived from relay standardness for the recorded envelope (`rec:state-constructor:keyed-scaling`); a changed witness, funding or output envelope reopens it, ruling 180's owning-constructor typed N+1 assignment is discharged as construction policy at (`rec:w1b14:closed`), and no covenant count law, other envelope or bite-1.0 closure follows. The complete provisional packet now meets both bite-1.0 gates at (`rec:phase7:wave1-bite-1-0-admission`), which checks the dispositions of (a) (`rec:state-constructor:keyed-birth`), (b) (`rec:state-constructor:keyfree-continuation`) and (`rec:state-constructor:keyed-continuation`), (c) (`rec:state-constructor:keyed-constructor`), (d) (`rec:state-constructor:keyed-cost`), (e) (`rec:state-constructor:keyed-funding`) and (f) (`rec:state-constructor:keyed-scaling`). Its provisional recommendations are explicit policy-asset funding with a supported 10,000-satoshi handle and fees against each subject's reported floor, and at most 1,564 keyed handles in the recorded envelope; final-byte repetition and ruling 162's exact recipe remain open; the typed N+1 assignment is discharged at (`rec:w1b14:closed`), with no paper-level realization claim. The reviewed refit closes the required code findings, including both search tracing contracts and the typed layout-refusal boundary, at (`rec:phase7:wave1-engine-refit`); the refit is imported at `0.7.94-dev`, with per-subject and compile-time cost measures still open. Rulings 171 to 180 at (`rule:phase7:wave1-home-rulings`) accept the explicit semantic-profile home only, assign the typed N+1 construction-policy refusal to the bite-1.4 transaction constructor under ruling 180, and assign the stable declaration packet (`proposal:w1home:packet`). That packet now meets both bite-1.1 declaration gates at (`rec:phase7:wave1-declarations`), with independent version-2 binding pins, the nine-row paper weld and exact attestation regressions; production keyed analysis and the remaining compiler obligations stay with bite 1.2, and no realization or deployment is accepted. Ruling 172 now accepts opaque per-position keys for their structural guarantee, with the measured growth and retained lookup invariants recorded at (`rec:phase7:wave1-keys-disposition`); production keyed plans and cost measurement remain with bite 1.2. The default keyed-plan packet, final consumer measurement and clause-by-clause gate evidence meet the bite-1.2 gates at (`rec:w1b12:closed`), with governing evidence at (`rec:phase7:wave1-keyed-plans`) and (`rec:w1b12:gate`); compiler feature removal is complete. Bite 1.3 has discharged the multi-member and single-wrong-member subjects in model contexts at (`rec:w1b13:closed`); serialized reproduction and N+1 constructor refusal are now discharged by bite 1.4; native final-byte evidence remains with bite 1.6. Profiling and runner-accounting follow-ups are explicit at (`tab:backlog:wave1-followups`). The bite-1.3 default emission packet meets its gates and is closed at (`rec:phase7:wave1-keyed-emission`) and (`rec:w1b13:closed`); the bite-1.4–1.6 obligations remain at (`rec:w1b13:members`) and the explicit closing handoff, while (`tab:backlog:wave1-followups`) retains profiling, runner accounting and the model-control disposition. The bite-1.4 public construction gate is closed at (`rec:phase7:wave1-keyed-construction`) and (`rec:w1b14:closed`), including ruling 180's construction-policy assignment; parent facts remain per-instance external, bite 1.5 supplies reader premises/relation evidence, and bite 1.6 records joint relation-plus-native-execution discharge and final-byte observations. The bite-1.5 six-family evidence packet is closed at (`rec:phase7:wave1-keyed-evidence`) and (`rec:w1b15:closed`); conditional parent/finality premises and final-byte native/policy observations remain explicit, and the existing follow-ups table is retained. |

### 4.2 Historical finding families · `tab:backlog:historical-findings`

| Family | Status | Scope |
|---|---|---|
| `F1` | HISTORICAL | Phase-1 remediation |
| `F2` | HISTORICAL | Post-Phase-1 boundary remediation |
| `F3-001`–`F3-010` | HISTORICAL | Meson, publication, projection, ownership, and stamp remediation |
| `F4-001`–`F4-005` | DONE or DROPPED | Predicate, lifecycle, and labels findings |
| `A17-001`–`A17-005` | DONE | ADR-017 implementation |
| `R1`–`R6` | DONE | Path, ADR status, dependency, CI, recipe, and compiler-status review |
| `S1`–`S7` | DONE | Semantic-boundary review and status weld |
| `T1` | DONE | Bound conformance observations to executed requests |
| `T2` | DONE | Typed substrate-conservation evidence requirement |
| `T3` | DONE | Batch-staged multi-output publication |
| `T4` | DONE | Runtime-bound conformance in the global invariant |
| `T5` | DONE | Removed duplicated volatile identities from planning prose |

### 4.3 Identity work · `tab:backlog:identity-work`

| ID | Status | Result |
|---|---|---|
| `I1-001` | DONE | ADR-016 adopted |
| `I1-002` | DONE | Current identity inventory |
| `I1-003` | DONE | Future immediate-edge identity DAG |
| `I1-004` | PARKED | Typed evidence envelopes; waits for a persistent report consumer |
| `I1-005` | BLOCKED | Deployment-profile migration; waits for bundle, ABI, and evidence types |
| `I1-006` | BLOCKED | Release root; waits for the release package |

The identity freeze is lifted only under
(`[ADR021-req:identity:admission-record]`). Phase 2 still mints
no public realization or compiler digest without a real consumer.

---

## 5. Current static-review findings · `sec:backlog:findings`

### 5.1 Summary · `tab:backlog:findings-current`

| ID | Priority | Status | Finding |
|---|---:|---|---|
| `T6` | P1 | DONE | External-evidence relations bypassed abstract capability validation during proof planning. |
| `T7` | P1 | DONE | Representation-relation proof choices were not constrained by the selected representation mode. |
| `T8` | P2 | DONE | Current-phase declarations disagreed and the plan checker did not detect all copies. |
| `T9` | P2 | DONE | Compiler status documentation materially understated implemented internal analysis. |

All four findings were reproduced or directly corrected, repaired, and
verified in the Guide-4 batch; the gate record is in §2.8. Their evidence
records follow. Implementation detail beyond these records belongs to Git
history. The later two-pass review recorded in §2.1 opened the SR2 findings
in §5.2.

### 5.2 Closed review registers

The finding registers of the second, third, fourth, and fifth static
reviews stood here as §5.2–§5.5. Every row was remediated and closed, in
the Guide-7, Guide-8, Guide-9, and Guide-10 preflight waves
respectively. They are archived verbatim, with their original numbering
and labels, in [backlog history](history/backlog-history.md), so a
reference to §5.3 below resolves to the heading of that number there.


### 5.6 Sixth-review findings

The sixth static review’s finding register stood here. Every row was
remediated and closed in the Guide-11 preflight and declassification
waves. It is archived verbatim, with its original numbering and its
label, in [backlog history](history/backlog-history.md), so a reference
to §5.6 below resolves to the heading of that number there.


### 5.7 Confidential-value target review · `tab:backlog:findings-ct-review`

The Guide-11 §5 target review read the confidential-value machinery at
merged tip `78499c2`, upstream base `b7fc5d0`, and landed its accepted
facts as typed Rust in `packages/target-elements/src/confidential.rs`
with human provenance in (`tab:elements-ref:review-ct`). The twenty
questions the guide sets are all answered from source.

The review's own numbering note: the preflight arc in §5.6 already
spends the label "Wave 5" on the row that closed `G11-H01`. This record
belongs to the guide's §5 execution, a different arc, and is filed by
content rather than by wave number.

Three dispositions come out of it, and one is a negative result that the
guide asks not to be softened.

| ID | Priority | Status | Finding |
|---|---:|---|---|
| `G11-C01` | P1 | DONE | The confidential encodings record whether the y coordinate is a quadratic residue, while the curve-checking primitives accept only the compressed public-key prefixes, which record whether y is odd. The two conventions select different points, so no pattern may be carried from one to the other by analogy. Typed as `PointParityConvention`. |
| `G11-C02` | P1 | DONE | The commitment relation is the conceptual one with both terms positive: the blinder multiplies the base point and the amount multiplies the asset generator. The opening scalar is thirty-two bytes big-endian, refused at or above the group order, admitted at zero. Typed as `CommitmentRelation`. |
| `G11-C03` | P0 | OPEN | An authenticated public opening has no complete on-script form under the reviewed revision. Three independent blockers are named and typed as `OpeningBlocker`; the capability stays `Unsupported` for that reason. This is the §11 disposition input and it is a review result, not a scope decision. |

`G11-C03` is the row every later declassification candidate must be
judged against. The blockers are that the asset generator cannot be
derived on-script, that the confidential and curve-primitive encoding
domains do not meet, and that a witness-supplied parity byte is bound to
nothing, so a relation a program could verify holds for a point or its
negation. Candidate C in the guide names parity as load-bearing; this
review supplies the source reason.

Two questions are recorded as OPTIONAL follow-ups rather than as
findings, because neither changes a typed fact.

| ID | Priority | Status | Item |
|---|---:|---|---|
| `G11-C04` | OPTIONAL | OPEN | The capability contract calls commitment equality `Unsupported`, which is exact for a primitive: no reviewed instruction compares two commitments. Byte equality over the exposed canonical items is nonetheless a composition a backend might build, and the guide's disposition vocabulary has a name for that state which the typed enum does not carry. Deciding whether `ConfidentialCapabilityState` should gain the backend-pattern and named-blocker states belongs to a wave that owns the contract shape, since the states are part of the versioned schema. |
| `G11-C05` | OPTIONAL | OPEN | Value introspection presents an absent value as an explicit zero, prefix `0x01` over eight zero bytes, so a program cannot distinguish an absent field from a zero one at that primitive. No current pattern depends on the difference. Recorded so that one which does is not written in the belief that the distinction survives. |

The typed facts are deliberately not wired into `TargetDefinition`.
Growing that schema is a versioned change under
(`rule:guide11-exec:target-version-honesty`), and this review states
facts rather than revising a contract shape; the contract version did
not move.

### 5.8 Dependency decision and commitment oracle (archived)

The Guide-11 dependency decision and the independent commitment
oracle record are archived verbatim, with their labels, in [backlog
history](history/backlog-history.md); references resolve there. The
decision to add nothing was later revised by the user's 2026-08-20
ruling recorded in section 2.7, which adopted the reference crates
as testing oracles.

### 5.9 Seventh-review preflight findings

The seventh static review’s preflight register stood here. Its sixteen
rows are the Guide-12 preflight identifiers `G12-R01` through
`G12-R16`; fifteen are CONFIRMED and DONE and `G12-R08` is
RECLASSIFIED and DONE, so the blocking rule
(`gate:guide12-exec:preflight`) is satisfied and no confirmed P0 or P1
row remains open. Wave 0 adjudicated and reproduced, and Waves 1a, 1b,
and 2 repaired: each row’s witness was written to assert its defect and
was flipped by the wave that fixed it. The register is archived
verbatim, with its numbering and its label, in [backlog
history](history/backlog-history.md), so a reference to §5.9 or to its
label resolves there. Guide-12 preflight dispositions themselves are
owned by the guide’s own table.

### 5.10 Phase-5 closing-arc adversarial review · `tab:backlog:findings-r5`

An external adversarial review of the phase-5 closing arc, archived at [plans/reviews/review-9-0.5.17-dev.md](reviews/review-9-0.5.17-dev.md). It performed no build, no test and no node run; its evidence is the tree it read.

Its central verdict was that the exit gate could not be signed: two of the fifteen rows asserted more than the tree supports. The owner accepted both of its gate amendments, and the `T5-071` gate-repair wave executed them and remediated every finding. EVERY ROW BELOW WAS RECOMPUTED AT THE WAVE'S OWN BASE BEFORE ANY REPAIR — a review is evidence about the tree it names, not an instruction — and the recomputation moved two figures the review reported, because the handoff arc that landed between the reviewed tip and the wave's base answered two more rows.

| ID | Finding | Priority | Verified at source | Status |
|---|---|---:|---|---|
| `R5-001` | Exit-gate row 4 asserted "complete safety evidence" and "eighty-one observed or first-party-established" beside a canonical completeness predicate the classifier reports as FALSE. | P0 | YES | DONE |
| `R5-002` | Exit-gate row 12 asserted that mixed-program vectors reject, where the architecture can state no such vector and no layer ever produced a verdict. | P0 | YES | DONE |
| `R5-003` | The native owner-signing lane did not bind the recorded refusal details, the control's acceptance, or the run-of-record constants. | P1 | YES | DONE |
| `R5-004` | The two typed leaf-partner grounds predicted the exact verdict words an unbuilt mutant would draw, which no in-repo source establishes. | P1 | YES | DONE |
| `R5-005` | Comments claimed seven distinct field ranges where two rows share one and separate by shape, and a non-claim called the bare-u mutant the only script-path verdict. | P2 | YES | DONE |
| `R5-006` | Section 2.20's filed process finding named the wrong separating fact for the eighth same-verdict row. | P2 | YES | DONE |
| `R5-007` | The gate record said nine waves closed while five of them still read ACTIVE. | P2 | YES | DONE |
| `R5-008` | The closing assessment re-minted the exit-gate colon-label in prose instead of citing it, which the register validator refuses as a duplicate mint. | P2 | YES | DONE (hotfixed) |
| `R5-009` | The orchestrator's exit-time record claimed both plans validators clean at the reviewed tip while validator 2 was in fact RED there with the `R5-008` duplicate mint — a false green in the records. | P1 | YES | DONE |
| `R5-010` | Section 15.4's `key-path-escape` row declared a script-path boundary a key-path spend can never reach. Recorded as an ERRATUM rather than a review finding. | P1 | YES | DONE |

**`R5-001` and `R5-002` are the two amendments.** Both NARROW what the gate claims; neither adds evidence, and no row of the matrix moved to make either true. Row 4 now states the explicit disposition standard in the row itself and says in the same breath that this is not section 13.5's answered-bar, which is false and honestly so. Row 12 now states operation-vocabulary closure with its three architectural grounds and records that nothing was ever submitted and nothing ever refused. The exit assessment is re-recorded against both.

**The corrected arithmetic**, recomputed at the wave's base rather than taken from the review: the 108-row matrix is 82 ANSWERED (34 first-party discharged, 24 native-run observed, 17 native-refusal observed, 1 determinism, 1 paired relation, 3 first-party fact, 2 report-layer) + 1 VOCABULARY-CLOSED, which `is_answered` reports false for and which section 4.2 puts outside the coverage denominator, + 25 STILL REQUIRED. The review computed 80 + 1 + 27 at the reviewed tip; the difference is the handoff arc, which minted the paired-relation standing and answered the key-path row. The claim the review refuted — "eighty-one observed or first-party-established" — was wrong on both sides of the ruling: it counted a vocabulary-closed row as established, and it is superseded by a recomputation rather than repeated with a new total.

**`R5-003`** was the review's sharpest finding and the repair only ADDS binding. The lane accepted any observed layer for the bare-u mutant and the control, checked the control's readback only where a reverification happened to be present, and checked the consensus and leaf mutants without reference to the recorded detail, the accepted identity, or the recorded separators — so it would have stayed green with a REJECTED control, which makes every refusal in the run unattributable. Each verdict the ceremony's `run_of_record` constants carry is now asserted against them, with row-set equality both ways. No constant moved and no existing assertion weakened. The reason the lane had left them unasserted — that a lane asserting a verdict fails rather than reports when the honest answer changes — is answered by the order of operations: the transcript is written to disk before the first assertion runs.

**`R5-004`** is the honest half of a typed stop. The in-repo covenant establishes that the two partners' arrangements carry a SECOND failing input and therefore have no separating fact of their own against the partner already driven; it does not establish which of the two failures a target reports for a multi-input candidate, and the register said it did. The prediction is withdrawn and the typing is unchanged. CORRECTED by the binding-closeout wave, whose re-review found this sentence's count too strong: the `T5-071` repair reached FOUR sites and this record said those four were all that carried it. TWO more did — `LeafArrangement::typed_partner` in the owner-signing ceremony and the paired-leaf evidence test's own comment, each still saying an unbuilt mutant would draw the SAME verdict at the SAME clause. Six sites carried it; four were repaired then and the remaining two are repaired here, all six now resting on the non-separation the covenant does establish. The finding stands as filed and the withdrawal is complete; only the count is corrected.

**`R5-009` is the orchestrator's own finding against its own records**, filed here rather than left in a report. The exit-time record asserted both plans validators clean at the reviewed tip; validator 2 was RED there, refusing the duplicate mint `R5-008` names. Two independent reproductions stand: `T5-069` reproduced it byte-identically at that tip, and a direct server run of the validator against the reviewed tip reproduced it again. The lesson is filed with the finding — a validator reported green without a run to point at is a claim, and the exit record made one.

### Guide-11 execution wave records (archived)

The three Guide-11 execution wave records — CT fixtures and the
conservation matrix, candidate dispositions and normalization, and the
fresh-process lifecycle — are archived verbatim, with their numbering
and their labels, in [backlog history](history/backlog-history.md);
a reference to any of their task labels resolves there. All three
closed DONE at the Guide-11 gate (section 2.4).

### T6 — Validate capabilities for external-evidence obligations · `task:review:external-evidence-capability`

**Priority:** P1
**Status:** DONE

Reproduction: with every capability available except whole-transaction value
conservation, production planning returned feasible plans for both pilots
(two compact-ASH and four live-transfer candidates) that carried the
substrate-conservation evidence requirement while omitting the conservation
capability entirely.

Repair: the external-evidence obligation class now retains its approved proof
alternative, required capabilities, and source requirements. Classification
validates that the realization approves exactly one substrate-conservation
alternative, with a typed error for any other shape. Enumeration tests the
fixed capability set against the capability view before the search begins and
seeds every candidate from the fixed requirements. The evidence requirement
remains unresolved, and no sponsor amount enters any fixed requirement.

Focused tests, in the compiler proof suite:

- `a_missing_whole_transaction_capability_blocks_substrate_conservation`
- `adding_the_whole_transaction_capability_restores_feasibility`
- `external_evidence_carries_its_capability_and_source_into_every_candidate`
- `fixed_external_requirements_name_no_sponsor_amount`
- `an_unexpected_external_evidence_alternative_is_rejected`

### T7 — Constrain every representation-sensitive proof choice · `task:review:representation-proof-compatibility`

**Priority:** P1
**Status:** DONE

Reproduction: unconstrained live-transfer planning returned four candidates,
two of which paired a selected mode with a contradictory arithmetic proof on
the representation relation itself (private committed with public arithmetic,
and explicit with confidential conservation).

Repair: representation relations are declared relation-only in both pilot
declarations, with the approved mode sets unchanged, and the compiler
classifies them as statically validated alongside lifecycle exits. The
conservation relation keeps exact proof/mode compatibility. Live transfer now
yields exactly one candidate per approved mode, and the selected mode remains
in each candidate's representations for later placement and backend work.

Focused tests, in the compiler proof suite:

- `representation_relations_carry_no_proof_variable`
- `live_transfer_conservation_agrees_with_every_selected_mode`
- `compact_ash_keeps_both_modes_under_public_arithmetic`
- `every_pilot_relation_is_classified_exactly_once`, extended

The independent oracle no longer calls production obligation classification.
It derives the substrate-conservation capability and its typed external
source row itself, creates no proof variable for representation relations,
and validates fixed requirements before enumeration. New oracle regressions
cover the missing whole-transaction capability across all scopes, one
candidate per live mode, and random capability views over all thirteen
capabilities, while still distinguishing infeasibility from unexpected
errors.

### T8 — Weld every current-phase declaration · `task:review:phase-declaration-drift`

**Priority:** P2
**Status:** DONE

Repair: the plans README declares Phase 2 in the fixed declaration form. The
plan checker now compares the backlog current gate, the roadmap current
phase, the plans README current phase, and exactly one active numbered phase
card, normalized to the numeric phase. Diagnostics are order-stable and name
the stale file, the observed phase, and the expected phase; malformed or
missing declarations fail explicitly.

Focused tests: ten phase-weld tests in the labels plans module covering
agreement, each stale copy, zero and two active cards, malformed and missing
declarations, a gate without a phase card, and traversal-order independence.

### T9 — Refresh compiler status documentation · `task:review:compiler-status-drift`

**Priority:** P2
**Status:** DONE

Repair: the compiler crate Rustdoc lists the implemented internal stages and
the remaining absent work, and keeps the boundary statement that no public
value can be mistaken for a completed analysis. The compiler package
contract and the packages index carry an accurate status and milestone
states, and the phase card separates delivered-analysis requirements from
the open placement, coverage, and pilot deliverables. The stale-phrase sweep
found two current-state matches; both were corrected.

Verification: compiler package tests, a warning-free Rustdoc build, and the
plans check all passed.

---

## 6. Phase-2 implementation queue · `sec:backlog:phase2`

### 6.1 Summary · `tab:backlog:phase2`

| ID | Priority | Status | Deliverable |
|---|---:|---|---|
| `P2-001` | P1 | DONE | Immutable, canonical, ownership-validated realization boundary |
| `P2-002` | P2 | DONE | Petgraph dependency and lockfile review |
| `P2-003` | P1 | DONE | Compiler crate boundary and typed error root |
| `P2-004` | P1 | DONE | Bind architecture, realization, policy, and explicit scope |
| `P2-005` | P1 | DONE | Canonical compiler relation and expression DAGs |
| `P2-006` | P1 | DONE | Checked constant folding |
| `P2-007` | P1 | DONE | Exact proof planning; reaccepted after T6 and T7 closed, gate record §2.8 |
| `P2-008` | P1 | DONE | Disclosure, source, and constructibility analysis |
| `P2-009` | P1 | DONE | Representation lifecycle analysis |
| `P2-010` | P1 | DONE | Execution-case placement and layout requirements; gate record §2.9 |
| `P2-011` | P1 | DONE | Relation-indexed coverage requirements; gate record §2.10 |
| `P2-012` | P1 | DONE | Compact-ASH and live-transfer analyzed pilots; gate record §2.11 |
| `P2-013` | Gate | DONE | Complete Phase-2 evidence and exit; gate record §2.11 |

### 6.2 Completed foundation

The following are implemented internally and remain subject to their package
tests and owner validation:

```text
P2-001  realization boundary
P2-002  Petgraph review
P2-003  compiler crate and error root
P2-004  compiler input binding
P2-005  relation/expression DAGs
P2-006  checked constant folding
P2-007  exact proof planning, reaccepted
P2-008  source, constructibility, and disclosure analysis
P2-009  representation lifecycle
P2-010  execution-case placement and layout requirements
P2-011  relation-indexed coverage requirements
```

No complete analyzed-program public API is exposed yet. This is deliberate:
partial analyses must not be mistaken for a completed compiler result.

### P2-007 — Repair and reaccept exact proof planning · `task:phase2:proof-planning`

**Priority:** P1
**Status:** DONE
**Depends on:** T6, T7, P2-005, P2-006
**Blocks:** P2-010 through P2-013

Reaccepted after T6 and T7 closed; the gate record is in §2.8.

Reacceptance required, and the recorded evidence establishes:

1. every in-scope relation classified exactly once;
2. every approved proof alternative retained or rejected for a typed reason;
3. external-evidence capabilities validated;
4. proof/representation compatibility applied completely;
5. capability and source requirements consistent;
6. hard constraints applied before objective or candidate retention;
7. exact feasible set compared with an independently derived exhaustive oracle;
8. no partial result on complexity exhaustion;
9. deterministic candidate ordering and equality under declaration
   permutations.

The search retains the exact feasible set; it does not silently select one
candidate under an unstated target policy.

### P2-010 — Derive execution-case placement and layout requirements · `task:phase2:placement`

**Priority:** P1
**Status:** DONE
**Depends on:** accepted P2-007, P2-008, P2-009, C1-009
**Blocks:** P2-011 and P2-012

Delivered in the Guide-5 batch; the gate record and evidence are in §2.9.

Relations are classified per execution case on independent axes — discharge
boundary, semantic scope, activation, and carrier multiplicity — rather than
one local/global/conditional/duplicated category. Execution cases expand only
the sponsor dimension; the representation is fixed by each feasible proof
plan. Every active runtime relation case has an eligible abstract carrier;
carrier eligibility, exact feasible placement sets, and target-independent
layout requirements are validated against an independent exhaustive oracle.
Complexity limits are explicit typed parameters and exhaustion returns no
partial result. No target bytes or concrete target positions enter this
stage.

### P2-011 — Derive relation-indexed coverage requirements · `task:phase2:coverage`

**Priority:** P1
**Status:** DONE
**Depends on:** P2-010, C1-010, C1-013
**Blocks:** P2-012

Delivered in the Guide-6 batch; the gate record and evidence are in §2.10.

Exact equality holds among the realization relation scope, the compiler
relation scope, the relation-case plan scope, and the coverage scope. Every
active relation-case carries positive and negative requirements at each of
its boundaries, a carrier requirement when runtime-carried, and an accepted
semantic-projection requirement; inactive relation-cases carry an explicit
inactive-valid requirement; conditional relations satisfy the
inactive-valid, active-valid, active-invalid triplet across the case set,
and representation modes are covered across the complete feasible plan
set. A broad operation test does not substitute for relation coverage.

### P2-012 — Analyze both pilots end to end · `task:phase2:pilots`

**Priority:** P1
**Status:** DONE
**Depends on:** T6, T7, P2-005 through P2-011
**Blocks:** P2-013

Delivered in the Guide-7 batch; the gate record and evidence are in §2.11.

Both pilots analyze end to end into one validated scoped analyzed program
per scope: the acceptance relation lists are the exact relation censuses,
every requirement in the acceptance matrices holds, rejected proof and
representation pairings are absent from the plan set, repeated analysis
from equal typed inputs produces equal stable projections, and the
combined two-pilot scope stores per-operation factors whose product
equals the retired global enumeration exactly. The analyzed value claims
no deployment lifecycle completeness: the pilots retain their future
target and operation obligations explicitly.

### P2-013 — Phase-2 evidence and exit · `gate:backlog:phase2`

**Priority:** Gate
**Status:** DONE
**Depends on:** T6–T9, P2-004 through P2-012, C1-005, C1-008,
C1-009, C1-010, C1-013

Passed; the complete exit evidence is the Guide-7 gate record in §2.11,
including the declared-MSRV and current-stable lanes, the meson-required
CI run, the explicit product regressions, and passed document byte
reproducibility. Every exit condition held: findings closed, censuses
exact, capabilities failing closed, deterministic pilot analysis,
independent oracles agreeing, gates recorded, current-phase declarations
moving together, and a clean final tree.

Phase completion does not itself justify a persistent compiler digest;
none was minted.

---

## 7. Immediate algorithm preparation · `sec:backlog:algorithms`

### 7.1 Current status · `tab:backlog:algorithms`

| ID | Status | Deliverable |
|---|---|---|
| `C1-001` | DONE | Compiler/linker/mathematics/solver research notes |
| `C1-002` | DONE | Direct Petgraph decision |
| `C1-003` | DONE | Exact/certified mathematics decision |
| `C1-004` | DONE | Petgraph dependency review |
| `C1-005` | DONE | Canonical direct-Petgraph compiler graph prototype |
| `C1-006` | PARKED | Exact keyed linear systems until a consumer exists |
| `C1-007` | PARKED | Certified numerical analysis until a consumer exists |
| `C1-008` | DONE | Exact proof-plan search; reaccepted after T6/T7, gate record §2.8 |
| `C1-009` | DONE | Execution-case-aware placement; gate record §2.9 |
| `C1-010` | DONE | Typed symbol resolution and SCC policy in coverage scope; gate record §2.10 |
| `C1-011` | BLOCKED | Structured relocation; linker phase |
| `C1-012` | BLOCKED | Deterministic bounded-depth target tree; linker phase |
| `C1-013` | DONE | Independent small-instance oracles; placement oracle §2.9, coverage/collateral/SCC oracles §2.10 |
| `C1-014` | DONE | Preparation review and Phase-2 handoff; gate record §2.11 |

### 7.2 C1-008 — Proof-plan search reacceptance

Reaccepted in the Guide-4 batch, gate record §2.8. The recorded evidence
establishes each former condition:

- external-evidence capability requirements are retained;
- representation-sensitive proofs are checked against selected modes;
- candidate capabilities and sources agree;
- the independent oracle derives those rules independently;
- infeasibility and complexity exhaustion remain distinct;
- no partial result is returned.

### 7.3 C1-009 — Execution-case-aware placement

Delivered in the Guide-5 batch with P2-010; the gate record is in §2.9.
Every active relation case is covered by an eligible carrier, production
search is compared with an independent exhaustive assignment oracle on
small instances and both pilot scopes, and the adversarial case list —
optional-carrier defects, global-on-local defects, source unavailability,
duplicate enforcement, permutation equality, and both budget exhaustions —
is retained as focused regressions.

### 7.4 C1-010 — Typed symbols and SCC policy

Delivered in the Guide-6 batch within the coverage scope; the gate record is
in §2.10. Two-pass typed resolution runs a complete definition census before
complete reference resolution; duplicate keys, unknown references, and
cross-operation references fail; SCC members and diagnostics are normalized
by stable key and every current cycle is rejected. SCC membership does not
authorize a semantic cycle — an accepted cycle would require a new typed
resolution strategy and policy review. This work prepares linker policy but
pulled no linker or target types into compiler core.

### 7.5 C1-013 — Independent algorithm oracles

| Production analysis | Independent oracle |
|---|---|
| canonical topology | valid-order enumeration plus least-key rule |
| SCC | mutual-reachability equivalence |
| expression interning | non-interned evaluator |
| dependency closure | repeated complete scan |
| proof selection | exhaustive candidate enumeration with independently derived hard constraints |
| placement | exhaustive carrier subsets |
| relation census | direct set equality |
| constant folding | non-folded evaluator |
| stable projection | declaration-order permutation |
| coverage | direct relation × case matrix equality |

The proof-selection oracle must not call the same obligation-classification or
compatibility helper as production for the property it is meant to verify.

Delivered across the Guide-5 and Guide-6 batches: the placement oracle in
§2.9; the relation-by-case coverage oracle, the repeated-scan collateral
oracle, and the mutual-reachability SCC oracle in §2.10. Each restates its
hard predicates independently of the production helper it verifies.

---

## 8. Dependency posture · `sec:backlog:dependencies`

### 8.1 Current and deferred dependencies · `tab:backlog:dependencies`

| Dependency | Status | Role |
|---|---|---|
| `petgraph = 0.8.3` | Adopted and reviewed; `serde-1` only | Graph storage and standard algorithms |
| `num-bigint` | Existing | Exact arbitrary-size integers |
| `num-integer` | Existing | Exact integer helpers |
| `num-traits` | Existing | Numeric traits |
| `proptest` | Existing | Property and independent-oracle testing |
| `sha2` | Existing | tagged SHA-256 for taproot leaf, branch and tweak commitments |
| `lzma-rust2 = 0.24.0` | Reviewed; decoder-only `std`, no default features | Test-only bounded scaling archive admission; (`rec:backlog:scaling-decoder-review`) |
| `num-rational` | Deferred | Future exact-rational consumer |
| `faer` | Deferred | Future certified numerical diagnostics |
| `fixedbitset` | Deferred | Dense local coverage sets if measured |
| Elements libraries | Phase-3 review | Target transaction and consensus types |
| SAT/LP/MILP solver | Deferred | Larger exact planning only after measured need |
| `salsa` | Deferred | Incremental compiler queries |
| `egg` | Deferred | Equality saturation |
| `rayon` | Removed and deferred | Requires measured need and schedule-independent results |

The latest static review excluded `Cargo.lock`; no lockfile claim is made by
that review.

### 8.2 Dependency-entry rule · `rule:backlog:dependency-entry`

A deferred dependency enters only when:

1. a concrete consumer exists;
2. current implementation demonstrates missing functionality;
3. simpler exact first-party code is insufficient;
4. source, version, licence, MSRV, unsafe boundary, transitive graph,
   determinism, and advisories are reviewed;
5. public API leakage is considered;
6. focused tests and an independent oracle exist;
7. lockfile changes are reviewed;
8. required gates remain green and clean.

Unused dependencies are not added to advertise intent.

### Scaling archive decoder review · `rec:backlog:scaling-decoder-review`

Purpose and home: `packages/vectors/src/keyed_scaling_corpus.rs` is a test-only admission reader for a 164,375,392-byte native capture retained as a compressed original. `lzma-rust2` is declared in workspace dependencies and inherited only by vectors' dev-dependencies; the module is under `cfg(test)`, so neither the codec nor its types enter the public library API. A maintained LZMA implementation is a substantial algorithm and is not clearer as new first-party code. The independent encoder is XZ Utils 5.8.2 on kranich, with `xz --format=lzma --threads=1 -6 -c`; LZMA-Alone carries no timestamp or filename. The independently pinned original SHA-256 is the round-trip oracle. Capture framing and wire JSON are unchanged.

Source and maintained status: the reviewed crates.io release is `lzma-rust2` 0.24.0 from [hasenbanck/lzma-rust2](https://github.com/hasenbanck/lzma-rust2), a maintained continuation of lzma-rust; its [release history](https://github.com/hasenbanck/lzma-rust2/blob/master/CHANGELOG.md) records ongoing decoder hardening and bounded worker fixes. The published crate's `Cargo.toml`, `src/lib.rs` and `src/lzma_reader.rs` were read, rather than relying on its description. The Cargo range is `0.24.0`, allowing patch updates within that minor under (`[ADR011-rule:toolchain:locked]`); no lockfile is committed. Future feature or range changes require renewed review.

License, graph and unsafe boundary: Apache-2.0 is permissive and compatible with this repository's AGPL-3.0-only distribution. `default-features = false, features = ["std"]` excludes `encoder`, `optimization`, `xz` and `lzip`; the optional SHA-2 dependency belongs to the disabled XZ feature, so this selected graph adds no transitive crates, build script, C library or native compiler. The crate applies `forbid(unsafe_code)` when `optimization` is disabled. Only its single-threaded `LzmaReader` is called; its encoder and unsafe assembly paths are not enabled. The external compression tool is not linked into the Rust graph.

Determinism, bounds and MSRV: identical compressed bytes deterministically produce the original bytes; acquisition pins bind both representations and the original manifest. A compressed digest mismatch is refused before decoder creation; decoder dictionary/model memory is capped at 16,384 KiB and output at exactly 164,375,392 bytes, with a one-byte overrun probe never appended to the output. A different output digest, invalid completion or oversized output refuses admission. The release declares Rust 1.85 and edition 2024, below the workspace's 1.88 MSRV; this review is not an additional MSRV compiler run. No wall time is an admission input. Successful decompression and replay are cached once per test process, and tests change the compressed byte, output digest, output bound, neighbour class/reason, node weight and response census beside the same accepted archive.

Advisory review on 2026-11-09: the [RustSec database](https://rustsec.org/advisories/) search identified no lzma-rust2 advisory; the gate's cargo-audit lane remains the check of the resolved graph, not a permanent clean bill. Zstandard/ruzstd and gzip candidates were considered but not selected: the chosen feature surface actually excludes compression and unsafe optimization code and needs no transitive dependency. This review grants no general decompression or untrusted-archive service; only the pinned, bounded test archive is admitted.

---

## 9. Mathematical and algorithmic laws · `sec:backlog:laws`

### 9.1 Exactness · `rule:backlog:exactness`

Semantic, conservation, authorization, identity, calibration, and release
claims use:

- checked bounded integers;
- arbitrary-precision integers;
- reduced exact rationals;
- exact finite search;
- independently checked certificates;
- target-native execution where the claim concerns the target.

A floating residual is diagnostic evidence, not exact equality.

### 9.2 Handles and identity · `rule:backlog:handles`

Always distinguish:

```text
local handle:
    process-local graph, arena, matrix, or solver position

stable key:
    complete typed semantic identity

digest:
    optional domain-separated commitment, admitted per the
    identity adjudication procedure
```

Never use as semantic identity:

- Petgraph indices;
- matrix positions;
- solver variable numbers;
- insertion or traversal order;
- source path or line;
- pivot order;
- floating-point bits;
- thread schedule;
- temporary path.

### 9.3 Complexity failure · `rule:backlog:complexity`

An analysis exceeding its explicit budget returns a typed complexity error.

It must not:

- drop a relation;
- weaken authorization;
- increase disclosure silently;
- remove a lifecycle exit;
- switch to an undocumented greedy fallback;
- claim optimality from incomplete search;
- accept the best partial result seen before exhaustion.

### 9.4 Evidence separation · `rule:backlog:evidence-separation`

Keep separate:

- typed validation;
- capability availability;
- witness/source availability;
- model execution;
- realization conformance;
- compiler completeness;
- backend pattern evidence;
- linked-bundle execution;
- target consensus/policy results;
- event projection;
- attestation query;
- receipt accounting;
- calibration;
- release validation.

A report or status from one class never silently satisfies another.

In particular:

```text
target capability exists
≠
required evidence was produced

sponsor role structure is valid
≠
substrate conservation was evidenced

model relation is conformant
≠
deployment is ready
```

---

## 10. Verification matrix · `sec:backlog:verification`

### 10.1 Working cadence

For Rust changes:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Do not run the full Meson/release surface after every small edit.

Documentation-only changes do not require the Rust lane, but they still require
the relevant documentation and census checks.

### 10.2 Focused commands · `tab:backlog:focused-tests`

| Area | Command |
|---|---|
| architecture/profile | `cargo test -p tripod-architecture` |
| realization | `cargo test -p tripod-realization` |
| compiler | `cargo test -p tripod-compiler` |
| compiler planning | `cargo test -p tripod-compiler proof` |
| compiler oracle | `cargo test -p tripod-compiler oracle` |
| model conformance | `cargo test -p tripod-model realization_conformance` |
| model bounds | `cargo test -p tripod-model bound_conformance` |
| artifacts | `cargo test -p tripod-artifacts` |
| labels/plans | `cargo test -p tripod-labels` |
| checker report/stamps | `cargo test -p cli-common` |
| document stamps | `cargo test -p tripod-document-stamps` |
| flattener | `cargo test -p flatten-latex-main` |
| process wrapper | `cargo test -p execwrap` |
| mocked Meson graph | `meson test -C build meson-mock-contract` |
| executor classification | `meson test -C build executor-classification` |

Focused filters supplement but never replace complete package and workspace
runs. Every gate lane is a Meson test (CI-009), so any of them narrows the
same way: `scripts/ci.sh --suite lint`, or a lane by name.

### 10.3 Full gate

After a completed batch:

```sh
scripts/ci.sh
meson compile -C build
meson test -C build --print-errorlogs
```

The first line is the whole lane surface with the TeX toolchain mocked; the
second and third add the real document build, which the mocked run cannot
cover. `scripts/ci.sh` is a shim over `meson test` and forwards its arguments,
and `scripts/ci-timing-report.py` reads the finished run's logs for the
per-suite, per-lane, and per-test timing.

The canonical build directory is `build/`.

Use:

```sh
meson setup build
```

only if `build/` does not exist.

For a complete release-oriented result, additionally run the required MSRV and
stable lanes and the separate byte-reproducibility check:

```sh
scripts/check-document-reproducibility.sh
```

### 10.4 Documentation and census

```sh
scripts/check-plans.sh
meson compile -C build lint
git diff --check
git diff --cached --check
```

Every new tracked subject must join its nearest `meson.build` census.

### 10.5 Dependency and advisory evidence

```sh
cargo tree -e features
cargo metadata
cargo audit
```

A missing `cargo-audit` is recorded as skipped, never passed.

### 10.6 Clean repository

The final check is:

```sh
git status --porcelain=v1 --untracked-files=all
```

It must be empty.

### 10.7 Execution trust

Repository source, tests, Cargo manifests, Meson definitions, scripts, TeX,
and `.latexmkrc` are executable.

Untrusted contributions run only in an externally established,
credential-free isolated environment under ADR-015. A clean-tree result is a
correctness check, not malicious-code containment.

---

## 11. Current gate · `gate:backlog:current`

The Phase-2 gate is **passed** (§2.11), the Phase-3 gate is **passed** (§2.6 with the Phase-3 card), and the Phase-4 gate is **passed** (§2.8). The Guide-13 batch gate is §2.9: at that historical gate Phase 5 had not exited, because the batch stopped honestly on the two typed blockers the intermediate confidential-funding guide owned; the live-transfer campaign subsequently closed and Phase 5 exited in §2.20 and §2.21 with the amended gate met on a countersigned assessment. Phase 6 exited in §2.26 with its exit gate met on its amended text at the Phase-6 card's exit assessment (`sec:phase6:exit-assessment`), and Phase 7 is current. Guide 15 is chartered as the Phase-7 execution guide at `plans/guides/guide_fifteen.md`, with inherited debt and preflight in Wave 0, the Keyed Provenance Covenant in Wave 1 (`[A-abst:kpc:abstract]`), and the remaining phase delivery in Waves 2–14.

The Phase-7 task state opens with the Guide-15 document import; each wave lands with its own evidence, and Wave 0 opens only when its entry obligations, including the pinned Elements target provisioned on the node of record by the loop's own account, are recorded here.

| ID | Status | Task |
|---|---|---|
| `T12-001` | DONE | Bite 0.1 records verified pinned-target provenance and binding, the reproduced accepted sponsorless maturity control, all ten repository-entry fields, and the reproduced inherited censuses (`rec:backlog:wave0-entry`). It supplies the target prerequisite for `T11-064` and the native portions of `T12-005`; it closes neither row, the region-scoping refit, sponsored construction, positive-floor relayability, the outstanding Safety obligations, nor Wave-0 preflight. |
| `T12-002` | DONE | The operation-region refit and exact relation-link recount are recorded at (`rec:backlog:wave0-region-refit`); this closes the structural portion of `T11-064` and supplies conjunct 14's structural evidence. Positive-floor and sponsored-form obligations remain with `T12-005`. |
| `T12-003` | DONE | The classification at (`rec:backlog:wave0-closure-audit`) and elision at (`rec:backlog:wave0-closure-elision`) close `T11-063` with linked positive/fault pairs and producing-candidate historical admission: compact symbols/relocations/sites move 22/61/66 → 20/55/60, live 42/218/247 → 40/200/229, compact unresolved/linked bytes 3939/3975 → 3861/3861, while carriers 28/30/28 and coverage 209/221/223 retain the classification record's scope and regional residuals. |
| `T12-004` | DONE | The current compact native campaign at (`rec:backlog:wave0-compact-campaign`) observes or formally closes every one of its 48 native requirements under (`gate:guide15-exec:bite-0-4-gate`): 27 observed across two admitted archives (16 in the first, 11 in the second, none counted twice), 21 formally closed (1 by absent sponsor authorization coordinate, 2 discharged by sponsor authorization, 18 by absence of coordinate) and 0 unresolved. Its 48-subject roster repeats the historically discharged output-cardinality requirement because that observation covers only its producing candidate. The first-party 2 discharged / 18 retained and external 6 retained rows carry the classification record's standing and are not closed here. |
| `T12-005` | DONE | Bite 0.5 closes through (`result:backlog:bite-0-5-gate-pass`) under (`gate:guide15-exec:bite-0-5-gate`) and (`gate:wave0:bite-0-5-packet`), with the non-answer and native readings accepted by ruling 135 (`rule:phase7:wave0-rulings`). The census home (`result:backlog:close-safety-reconciliation`) records 206 / 177 / 29, observed/final/retired/waiting (177, 27, 2, 0), typed final/waiting 27 / 0, first-party discharged/required 81 / 0, native acceptance/refusal 12 / 39, report required/observed 0 / 15, root history/public recovery 16 / 14, relations 14 / 108 / 84 and non-link reasons 55 / 9 / 9 / 0 / 11, with supplementary [41, 2, 0]. The thirty-five charter non-answers map to fifteen observed and twenty final rows; seven additional coordinate finals give the current twenty-seven typed finals. The four Safety acquisition archives supplied selected S1/S2 positive-floor acceptances and sponsor refusals (`rec:backlog:wave0-sponsored-observation`), declared-boundary refusals and eight semantic positive-floor acceptances (`rec:backlog:wave0-native-observation`), one semantic refusal answer and two-successor acceptance supporting finality without an acceptance answer (`rec:backlog:wave0-native-route-capture-driver`), and confidential agreement supporting finality without an observed answer (`rec:backlog:wave0-confidential-route-admission`). Independent edge and successor acquisition is retained at (`rec:backlog:wave0-independent-observation`); separate diagnostic attribution is 24 agreements / 1 difference / 0 absent claims at (`rec:backlog:wave0-diagnostic-route-admission`). The five conjuncts have nineteen parts: ten MET, nine RETIRED and zero WAITING (`tab:backlog:conjunct-dispositions`). The parsed first-party, closure, existing-entry and native-staging spans retain 43 / 7 / 14 / 42 rows. Construction and retyping history, first-party and ABI comparisons, report and relation dispositions, bounded consolidation (`rec:backlog:wave0-close-safety`), combined projection and native settlement remain at their cited records; the historical 206 / 78 / 128 census and NOT PASSED conjunction remain at (`sec:phase6:gate-record`). A shared diagnostic class supplies no unique instruction site, and the absent-coordinate, undecided-coordinate and confidential-value premises reopen under (`cond:wave0:baseline-freeze`). This closes the remaining `T11-064` obligations through (`rec:guide15-exec:bite-0-5-closure-carrier`); preflight execution exits remain for bite 0.7. |
| `T12-006` | DONE | Bite 0.6 closes through `T11-110`: the first scheduling remedy has a lower comparable warm wall with equal census and retained attribution. |
| `T12-007` | DONE | Bite 0.7 closes the nine conditions of (`gate:guide15-exec:wave0`) and (`gate:guide15:preflight`) under the semantic-entry split. Twelve dispositions and their typed authorities are recorded at (`rec:backlog:wave0-preflight-census`); all 65 execution exits and acceptance clauses are joined at (`rec:backlog:wave0-preflight-exits`), retaining 50 HOLDS / 13 UNDECIDED / 2 DOES NOT HOLD findings and 25 consuming-wave obligations. The completed-tree gate at `0.7.61-dev`, retained under `evidence/wave0/baseline/20261103T131720Z-completed-tree/`, is recorded at (`rec:backlog:wave0-baseline`): both mocked and real-document Meson runs Ok 63 / Fail 0, workspace tests 5,996 passed / 0 failed / 55 ignored, fmt and clippy successful, clean before and after, chain rc 0, wall 5,132 s with own CPU and shared-load context. Consolidation under (`rec:guide15-exec:bite-0-7-closure-carrier`) retains `T11-064` DONE through (`rec:backlog:wave0-region-refit`) and (`result:backlog:bite-0-5-gate-pass`), `T11-063` DONE through (`rec:backlog:wave0-closure-audit`) and (`rec:backlog:wave0-closure-elision`), `T4-009` DONE through (`rec:backlog:wave0-compact-inherited-closure`), and `T11-110` DONE through (`rec:backlog:wave0-test-performance`) and ruling 129. Safety remains 206 / 177 / 29 with categories 177 / 27 / 2 / 0, relations 14 / 108 / 84, and nineteen conjunct parts 10 MET / 9 RETIRED / 0 WAITING. Compact retains 74 requirements, 51 observed / 23 formal, no unresolved requirement. The baseline publishes before/after counters and exact subjects without changing historical findings, the original NOT PASSED verdict or later implementation obligations. Cold after is unavailable and gate-wall regression standing stays open; no deployment, release, calibration, production or byte-reproducibility claim is added. DONE takes effect through this record landing's own import gate. |
| `T12-008` | DONE | Charter import: `plans/guides/guide_fifteen.md` supersedes the unchanged concept as execution direction, registers the controlling entry table and fifteen waves, reconciles current status, and admits the second contract inside Phase 7 under (`sec:phase7:change-statement`); no implementation wave is claimed executed. |

**Record (Wave 0 region refit)** · `rec:backlog:wave0-region-refit`

Explicit object claims and owned flows define each operation region; their disjoint union is the transaction's claimed portion, while unclaimed catalog objects contribute to no operation's relations. Projection obtains claims from exact consumed and created certificate objects; evaluation and fact selection share that region.

**Table (Relation dispositions)** · `tab:backlog:region-relations`

| Relation | Former domain | New domain | Reason | Consumers changed |
|---|---|---|---|---|
| `AllowedObjectFamilies` | Every object on either transaction side | Claimed objects on the selected side | Other operations retain their own families | Evaluation, shared selectors, projection, fixtures |
| `SponsorIsolation` | All sponsor flows and ordinary L-BTC objects | Selected region's fee-sponsor flows and ordinary L-BTC members | Flow membership remains distinct from family membership | Evaluation, projection, omission coverage, vector links |
| `CanonicalDeltaPolicy` | Whole canonical partition | Selected region's canonical partition and its exact membership | Other regions own their asset flows | Evaluation, projection, vector links |
| `OpenFlowPolicy` | All transaction open flows | Selected region's owned open flows | Other operations declare their own admitted flows | Evaluation, projection, fixtures |
| `SponsorEnvelopeMultiplicity` | All fee-sponsor flows | Selected region's fee-sponsor flow count, maximum unchanged | Operation-local resource policy | Evaluation, projection, vector link |

`SponsorEnvelopeMultiplicity` remains MODEL-SCOPE resource policy enforced by no leaf, under ruling 52 at (`rule:phase6:wave6-rulings`).

**Result (Composed observation accepted and foreign object refused)** · `result:backlog:region-composition`

`announcement_beside_compact_ash_is_conformant` and `foreign_object_claimed_by_announcement_is_refused` in `packages/model/src/tests/realization_conformance_tests.rs` ran green in `gate-w16b2-20261024T222051Z.log`: independently executed announcement and compact-ASH facts combine in disjoint regions; claiming an otherwise unclaimed ASH object for the announcement fails its family closure while the compact verdict stays unchanged. This establishes realization conformance over combined facts, without a combined state-machine transition or target evidence.

**Result (Relation links recounted)** · `result:backlog:region-links`

One new exact coverage class, `SponsorMemberUnclaimed`, is instantiated once per sponsor case.

Before: 14 / 102 / 90, with unlinked reasons 55 / 9 / 9 / 17; after: 14 / 108 / 84, with reasons 55 / 9 / 9 / 0 / 11. Six rows link exactly; eleven take `SubjectOutsideTheRelationsRegion`. The census retains every keyed row and checks 14 + 108 + 84 = 206 in `packages/vectors/src/maturity_safety.rs`.

| Row | Disposition and domain |
|---|---|
| `unaffected-field-omega-changed-alone` | Outside: metadata copy-through is not the region's canonical asset partition |
| `unaffected-field-y-l-changed-alone` | Outside: metadata copy-through is not the region's canonical asset partition |
| `unaffected-field-y-t-changed-alone` | Outside: metadata copy-through is not the region's canonical asset partition |
| `unaffected-field-q-changed-alone` | Outside: metadata copy-through is not the region's canonical asset partition |
| `unaffected-field-cycle-changed-alone` | Outside: metadata copy-through is not the region's canonical asset partition |
| `semantic-field-changes-compensated-by-another-field` | Outside: metadata compensation is not the region's canonical asset partition |
| `introduce-canonical-u-ent-or-dist-ctl-flow` | `CanonicalDeltaPolicy`: `UnexpectedCanonicalDeltaFamily` in the operation region |
| `state-sponsor-overlap` | `SponsorIsolation`: `SponsorProtocolOverlap` in the operation region |
| `two-sponsor-envelopes` | `SponsorEnvelopeMultiplicity`: `SponsorEnvelopeMultiplicityExceeded` in the operation region |
| `missing-sponsor-authorization` | `SponsorIsolation`: `MissingSponsorAuthorization` in the operation region |
| `sponsor-change-at-state-output-0` | Outside: absolute position is not sponsor flow membership |
| `state-successor-in-sponsor-range` | `SponsorIsolation`: `SponsorProtocolOverlap`, with STATE claimed by the sponsor flow |
| `fee-change-substitution` | Outside: economic allocation is not value-blind sponsor isolation |
| `sponsor-member-unclassified` | `SponsorIsolation`: `SponsorMemberUnclaimed`, exact membership omission |
| `empty-sponsor-offer-for-sponsored-request` | Outside: request/offer presence is not observed sponsor membership |
| `zero-valued-sponsor-member-under-exact-role-structure` | Outside: erased amount zero is not a sponsor role fault |
| `state-and-sponsor-positions-exchanged` | Outside: position ordering is not explicit sponsor membership |

**Caveat (Evidence boundary)**

No leaf changes or native observations are supplied; relation links do not answer sponsored construction or execution rows. Positive-floor and sponsored-form obligations remain with bite 0.5 and `T12-005`.

**Caveat (Remaining carrier scope)**

The four other MODEL-SCOPE identities, plain-L-BTC cardinality and recognition on both sides, retain their missing-carrier standing from the retired partition fragment; the six identities of this refit supply only `T11-064`'s structural portion and conjunct 14's structural evidence.

**Record (Wave 0 closure-exact audit)** · `rec:backlog:wave0-closure-audit`

The fixed compact batch needs its aggregate count; live redistribution independently needs its destination count. Neither establishes a sponsor restriction. The 47 declared relations retain their stated reasons; two emitted destination equalities are surplus under (`[ADR024-rem:closure:consequences]`), with elision remaining under `T12-003`. No semantic restriction is added.

**Table (Check census)** · `tab:backlog:closure-check-census`

The identifier names the declaration field; its input/output component names the side. The layer column follows declaration, compiler discharge, emitted fragment and linked carrier. Runtime entries are carried once per applicable case; structural and external entries do not acquire runtime carriers.

| Check | Generation | Layer and carrier | Closure law served | Disposition | Reason |
|---|---|---|---|---|---|
| `compact_ash.input_cardinality` | compact | declaration → runtime → `cardinality_fragment` → coordinator | (`op:realization:compact-ash`) | retained | Closure needs the fixed aggregate's bounded input family; regional requirement; transaction-wide residual below |
| `compact_ash.output_cardinality` | compact | declaration → runtime → `cardinality_fragment` → coordinator | (`op:realization:compact-ash`) | retained | Closure requires one aggregate successor; regional requirement; transaction-wide residual below |
| `compact_ash.sponsor_input_cardinality` | compact | declaration → runtime → `cardinality_fragment` → coordinator | (`sec:realization:domains`) | retained | Closure needs the declared sponsor input region's bound; regional requirement; transaction-wide residual below |
| `compact_ash.sponsor_output_cardinality` | compact | declaration → runtime → `cardinality_fragment` → coordinator | (`sec:realization:domains`) | retained | Closure permits at most one sponsor change role; regional requirement; transaction-wide residual below |
| `compact_ash.input_recognition` | compact | declaration → runtime → `ash_input_recognition_fragment` → each protocol input | (`op:realization:compact-ash`) | retained | Closure needs authenticated ASH input membership; declared region and role |
| `compact_ash.output_recognition` | compact | declaration → runtime → `successor_recognition_fragment` → coordinator | (`op:realization:compact-ash`) | retained | Closure needs an ASH aggregate successor; declared region and role |
| `compact_ash.sponsor_input_recognition` | compact | declaration → runtime → `sponsor_isolation_fragment` → coordinator | (`sec:realization:domains`) | retained | Closure requires ordinary L-BTC sponsor inputs; declared region and role |
| `compact_ash.sponsor_output_recognition` | compact | declaration → runtime → `output_census_fragment` → coordinator | (`sec:realization:domains`) | retained | Closure requires ordinary L-BTC sponsor change; declared region and role |
| `compact_ash.conservation` | compact | declaration → runtime → `explicit_sum_fragment` → coordinator | (`op:realization:compact-ash`) | retained | Closure preserves the aggregate's U amount; declared region and role |
| `compact_ash.permissionless` | compact | declaration → BackendStructural → `secret-free candidate` → no runtime carrier | (`op:realization:compact-ash`) | retained | Closure requires an ownerless execution path; declared region and role |
| `compact_ash.input_closure` | compact | declaration → runtime → `output_census_fragment` → coordinator | (`sec:realization:domains`) | retained | Closure excludes undeclared input families within the region; declared region and role |
| `compact_ash.output_closure` | compact | declaration → runtime → `output_census_fragment` → coordinator | (`sec:realization:domains`) | retained | Closure excludes undeclared output families within the region; declared region and role |
| `compact_ash.sponsor` | compact | declaration → runtime → `sponsor_isolation_fragment` → coordinator | (`rule:realization:derive`) | retained | Closure requires disjoint sponsor claims and owned flows; declared region and role |
| `compact_ash.sponsor_multiplicity` | compact | declaration → ModelScope → `none` → model_scope(), two obligations | (`rule:compiler:relations`) | retained | Closure records a resource policy with no leaf enforcement; resource policy, no leaf |
| `compact_ash.substrate_conservation` | compact | declaration → ExternalEvidence → `none` → external obligation | (`pkg:realization:contract`) | retained | Closure needs target conservation evidence for L-BTC; declared region and role |
| `compact_ash.open_flow_policy` | compact | declaration → runtime → `output_census_fragment` → coordinator | (`sec:realization:domains`) | retained | Closure permits only the declared regional open flows; declared region and role |
| `compact_ash.canonical_delta_policy` | compact | declaration → runtime → `output_census_fragment` → coordinator | (`op:realization:compact-ash`) | retained | Closure requires an ownerless lateral canonical partition; declared region and role |
| `compact_ash.roots` | compact | declaration → runtime → `output_census_fragment` → coordinator | (`op:realization:compact-ash`) | retained | Closure excludes undeclared root effects; declared region and role |
| `compact_ash.projections` | compact | declaration → runtime → `output_census_fragment` → coordinator | (`pkg:realization:contract`) | retained | Closure requires exactly the declared projections; declared region and role |
| `compact_ash.constructibility` | compact | declaration → CompilerStatic → `none` → no runtime carrier | (`op:realization:compact-ash`) | retained | Closure needs a constructible aggregate successor; declared region and role |
| `compact_ash.representation` | compact | declaration → CompilerStatic / BackendStructural → `structural candidate` → no runtime carrier | (`pkg:realization:contract`) | retained | Closure requires an authenticated supported representation; declared region and role |
| `compact_ash.compact_lifecycle` | compact | declaration → CompilerStatic / BackendStructural → `structural candidate` → no runtime carrier | (`op:realization:compact-ash`) | retained | Closure preserves the compact exit; declared region and role |
| `compact_ash.clear_lifecycle` | compact | declaration → CompilerStatic / BackendStructural → `structural candidate` → no runtime carrier | (`op:realization:compact-ash`) | retained | Closure preserves the clear exit; declared region and role |
| `transfer_live.input_cardinality` | live | declaration → runtime → `live_cardinality_fragment` → coordinator | (`op:realization:live-transfer`) | retained | Closure needs the bounded live input family; regional requirement; transaction-wide residual below |
| `transfer_live.output_cardinality` | live | declaration → runtime → `live_cardinality_fragment` → coordinator | (`[ADR024-rem:closure:consequences]`) | retained | Closure needs an independent destination count for redistribution; regional requirement; transaction-wide residual below |
| `transfer_live.sponsor_input_cardinality` | live | declaration → runtime → `live_cardinality_fragment` → coordinator | (`sec:realization:domains`) | retained | Closure needs the declared sponsor input region's bound; regional requirement; transaction-wide residual below |
| `transfer_live.sponsor_output_cardinality` | live | declaration → runtime → `live_cardinality_fragment` → coordinator | (`sec:realization:domains`) | retained | Closure permits at most one sponsor change role; regional requirement; transaction-wide residual below |
| `transfer_live.input_recognition` | live | declaration → runtime → `local_recognition_fragment` → each protocol input | (`op:realization:live-transfer`) | retained | Closure needs authenticated live input membership; declared region and role |
| `transfer_live.output_recognition` | live | declaration → runtime → `destination_closure_fragment` → coordinator | (`op:realization:live-transfer`) | retained | Closure requires live-class destinations; declared region and role |
| `transfer_live.sponsor_input_recognition` | live | declaration → runtime → `live_sponsor_isolation_fragment` → coordinator | (`sec:realization:domains`) | retained | Closure requires ordinary L-BTC sponsor inputs; declared region and role |
| `transfer_live.sponsor_output_recognition` | live | declaration → runtime → `live_sponsor_isolation_fragment` → coordinator | (`sec:realization:domains`) | retained | Closure requires ordinary L-BTC sponsor change; declared region and role |
| `transfer_live.authorization` | live | declaration → runtime → `owner_authorization_fragment` → each protocol input | (`op:realization:live-transfer`) | retained | Closure requires every consumed receipt's owner authorization; declared region and role |
| `transfer_live.input_closure` | live | declaration → runtime → `destination_closure_fragment` → coordinator | (`sec:realization:domains`) | retained | Closure excludes undeclared input families within the region; declared region and role |
| `transfer_live.output_closure` | live | declaration → runtime → `destination_closure_fragment` → coordinator | (`sec:realization:domains`) | retained | Closure excludes undeclared output families within the region; declared region and role |
| `transfer_live.conservation` | live | declaration → runtime explicit; external private → `explicit_conservation_fragment` → coordinator | (`op:realization:live-transfer`) | retained | Closure preserves U across split, merge and redistribution; declared region and role |
| `transfer_live.sponsor` | live | declaration → runtime → `live_sponsor_isolation_fragment` → coordinator | (`rule:realization:derive`) | retained | Closure requires disjoint sponsor claims and owned flows; declared region and role |
| `transfer_live.sponsor_multiplicity` | live | declaration → ModelScope → `none` → model_scope(), two obligations | (`rule:compiler:relations`) | retained | Closure records a resource policy with no leaf enforcement; resource policy, no leaf |
| `transfer_live.substrate_conservation` | live | declaration → ExternalEvidence → `none` → external obligation | (`pkg:realization:contract`) | retained | Closure needs target conservation evidence for L-BTC; declared region and role |
| `transfer_live.open_flow_policy` | live | declaration → runtime → `destination_closure_fragment` → coordinator | (`sec:realization:domains`) | retained | Closure permits only the declared regional open flows; declared region and role |
| `transfer_live.canonical_delta_policy` | live | declaration → runtime → `destination_closure_fragment` → coordinator | (`op:realization:live-transfer`) | retained | Closure requires the lateral canonical partition; declared region and role |
| `transfer_live.roots` | live | declaration → runtime → `destination_closure_fragment` → coordinator | (`op:realization:live-transfer`) | retained | Closure excludes undeclared root effects; declared region and role |
| `transfer_live.projections` | live | declaration → runtime → `destination_closure_fragment` → coordinator | (`pkg:realization:contract`) | retained | Closure requires exactly the declared projections; declared region and role |
| `transfer_live.constructibility` | live | declaration → CompilerStatic → `none` → no runtime carrier | (`op:realization:live-transfer`) | retained | Closure needs constructible live successors; declared region and role |
| `transfer_live.representation` | live | declaration → CompilerStatic / BackendStructural → `structural candidate` → no runtime carrier | (`pkg:realization:contract`) | retained | Closure requires an authenticated supported representation; declared region and role |
| `transfer_live.transfer_exit` | live | declaration → CompilerStatic / BackendStructural → `structural candidate` → no runtime carrier | (`op:realization:live-transfer`) | retained | Closure preserves the transfer exit; declared region and role |
| `transfer_live.burn_exit` | live | declaration → CompilerStatic / BackendStructural → `structural candidate` → no runtime carrier | (`op:realization:live-transfer`) | retained | Closure preserves the burn exit; declared region and role |
| `transfer_live.redeem_exit` | live | declaration → CompilerStatic / BackendStructural → `structural candidate` → no runtime carrier | (`op:realization:live-transfer`) | retained | Closure preserves the redemption exit; declared region and role |
| `compact.change_destination_equality` | compact | emission `output_census_fragment` → change symbols → linked coordinator | (`[ADR024-rem:closure:consequences]`) | surplus | Elision deferred: the sponsor's all-inputs-all-outputs signature fixes the destination; `flow_role_is_exact` states no destination equality. |
| `live.change_destination_equality` | live | emission `live_sponsor_isolation_fragment` → change symbols → linked coordinator | (`[ADR024-rem:closure:consequences]`) | surplus | Elision deferred for the same signature guarantee in both explicit and private construction; ordinary L-BTC recognition and exact roles remain required. |

**Result (Before and after census)** · `result:backlog:closure-census`

All before figures name the base `0.7.14-dev`; arrows are the resource-policy reclassification, not leaf elision. Relations and cases stay 23/46 compact and 24/48 live; feasible placements stay 216 per plan. Each plan gains two explicit ModelScope obligations outside runtime carriers.

| Generation / representation | Runtime carriers | Coverage requirements | Layout requirements | Graph nodes / edges | Symbols / relocations / pushed sites |
|---|---|---|---|---|---|
| compact explicit | 30 → 28 | 213 → 209 | 69 → 63 | 346/375 → 338/355 | 22/61/66 unchanged |
| live explicit | 32 → 30 | 225 → 221 | 69 → 63 | 364/501 → 356/479 | 42/218/247 unchanged |
| live private committed | 30 → 28 | 227 → 223 | 61 → 55 | 361/487 → 353/465 | 42/218/247 unchanged |

Compiler pins: `every_pilot_factor_is_exactly_its_measured_census` in `pilot_program_tests.rs`, `the_published_censuses_have_the_measured_pilot_sizes` in `live_transfer_plan_tests.rs`, and `packages/compiler/tests/public_api.rs`. Assessment requirements move 79 → 73 and complete patterns 74 → 68. Carrier pins: `the_three_censuses_are_one_set_and_every_case_stays_reachable`, `every_case_of_this_candidate_is_carried_by_exactly_one_site`, and `the_two_plans_differ_by_exactly_the_conservation_of_the_protocol_asset`.

Emission pins: `the_compact_demonstration_has_exactly_sixty_six_pushed_symbol_sites`, `each_live_demonstration_has_exactly_two_hundred_forty_seven_pushed_symbol_sites` (both representations, also 42 symbols and 218 relocations), `the_measured_shape_of_the_candidate_is_exactly_this`, and `the_census_splits_by_who_settles_each_symbol`. Compact unresolved/linked leaf bytes stay 3939/3975. STATE stays 6 symbols/15 sites, variable metadata 7/16, pinned by `the_composed_consumer_census_is_exactly_six_symbols_over_fifteen_sites`; 11/45 is the intermediate ruling-41 figure, not this baseline.

| Reconciled evidence count | Landed at base | After classification | Pinning test |
|---|---|---|---|
| total / positive / negative | 213/139/74 | 209/135/74 | `every_coverage_requirement_has_exactly_one_row` |
| target / first-party / external negatives | 50/18/6 | 48/20/6 | `the_negative_half_splits_into_three_kinds_that_sum_to_it`; `compact_negative_manifest_matches_every_published_negative` |
| missing entries | 22 = 16 + 6 | 24 = 18 + 6 | `compact_negative_manifest_pins_missing_owners` |
| dispositions: total = discharged / out-of-scope / retained | 18 = 2/0/16 | 20 = 2/0/18 | `compact_first_party_dispositions_retain_all_twenty_subjects` |
| first-party name bases, each in two sponsor cases | 9 | 10 | `compact_first_party_dispositions_retain_all_twenty_subjects` |

At classification, the two multiplicity rows moved from target execution to compiler analysis because their discharge is ModelScope; `OwnerEntryMissing::CompilerSponsorMultiplicity` named `required: compiler::validate_sponsor_envelope_multiplicity` and both rows remained retained. They now have observed comparisons at (`rec:backlog:wave0-compact-inherited-closure`). The historical subset moved 48 → 46 target rows, outstanding 47 → 45; current outstanding moved 49 → 47 at that classification. Inherited pre-region totals 211/72/48 and base 213/74/50 differed by the two `SponsorMemberUnclaimed` rows; the classification before-figures remain above.

**Result (Regressions)** · `result:backlog:closure-regressions`

Positive linking is pinned by `the_sound_strategy_links_and_owes_no_equality` and `the_demonstration_link_completes_over_both_representations`. Focused refusals are pinned by `removing_one_coordinator_leaf_breaks_a_uniquely_carrying_program`, `a_tree_with_no_member_leaf_leaves_a_member_carried_case_unreachable`, and `a_plan_whose_own_tree_lacks_a_carrier_starves_rather_than_borrowing`; `compact_negative_manifest_pins_missing_owners` and `compact_first_party_dispositions_retain_all_twenty_subjects` prevent missing multiplicity evidence from becoming a discharge.

The six-crate gate covers compact and live positive linking, role/asset refusals, carrier disjointness, manifest totality, dispositions and archive admission. Leaf schedules, definitions, ABI readers, constructors and golden fixtures remain byte-identical to the base; only comments qualify the transaction-wide fragments. The reproduced maturity control (`rec:backlog:wave0-entry`), Phase-4 compact positive and prepared compact negative inventory keep their original byte premises and evidence scope. STATE remains unchanged; archived captures admit as before through the unchanged current ABI. The reclassified multiplicity rows carry no new execution evidence.

Sponsor signing is `SighashProfile::AllInputsAllOutputs` in `construct.rs::collect_sponsor_signatures`, `live_construct.rs::complete_live_transfer` and `live_construct.rs::private_sponsor_witnesses`. Seven readers remain unchanged: `live_sponsored_private.rs`, `live_fault_discharge.rs`, `live_measurements.rs`, `live_pairs.rs`, `live_sponsor_shapes.rs`, and transaction tests `abi_tests.rs` and `live_abi_tests.rs`. Defaults remain construction data; the current leaf still repeats the destination equality pending elision.

**Caveat (Regional residuals and evidence boundary)**

Transaction-wide cardinalities and the live all-issuance sweep remain broader than the regional contract: their fragment comments name the first composed operation's replacement proof. They are residuals, not newly justified restrictions. The redundant compact self-comparison is dead instruction bytes, not an admitted-subject restriction. No native run or new acceptance is claimed; the compact negative campaign remains with bite 0.4.

**Caveat (Deferred elision and admission dependency)**

The preserved elision work starts from measured compact/live symbols 22 → 20 / 42 → 40 and pushed sites 66 → 60 / 247 → 229; compact relocations move 61 → 55 and live relocation recount remains part of that bite. Its linked alternate-destination controls and retained asset/role faults must accompany the removal. Revalidation reaches digest pins in `live_multi_shapes.rs` and `live_private_restart.rs`, measurements in `live_measurements.rs`, and the golden fixture. Historical admission depends on `live_corpus_native_operator::run_of_record` through `maturity_operator.rs` and the current live ABI; an earlier capture must admit against its producing candidate as historical, never as acceptance of changed bytes. Destination-refusal comments in unchanged `live_sponsor_shapes.rs`, `live_sponsored_private.rs` and `live_measurements.rs` require reconciliation with that elision.

**Record (Wave 0 closure elision)** · `rec:backlog:wave0-closure-elision`

The compact `output_census_fragment` and live `live_sponsor_isolation_fragment` omit only the sponsor-change program/version equalities classified as surplus at (`rec:backlog:wave0-closure-audit`). An all-inputs-all-outputs sponsor signature fixes the chosen destination; `flow_role_is_exact` requires its asset and role without destination equality. Regional semantics, the 47 declared relations and all other leaf instructions retain their classification under (`[ADR024-rem:closure:consequences]`).

Construction getters remain defaults. An explicit ordinary destination is honored; absent capability data selects the default. Emission, symbol disclosure, relocations, linked bundles and ABI evidence remove the two unused leaf literals. Every moved current pin retains its previous value beside the assertion; historical comparisons retain separate producing-candidate pins. Existing tests retain their obligations, with changed comparisons separating current derivation from historical acceptance.

**Table (Byte census)** · `tab:backlog:elision-byte-census`

| Pin | File | Before | After | Reason |
|---|---|---|---|---|
| `the_symbol_table_is_exactly_the_guide_list_and_every_relocation_is_backed` | `packages/tapscript/src/tests/bundle_tests.rs` | 22 symbols | 20 | Two destination literals leave the leaves |
| `the_compact_demonstration_has_exactly_sixty_pushed_symbol_sites` | `packages/tapscript/src/tests/bundle_tests.rs` | 66 pushed sites | 60 | Six sites leave three coordinator leaves |
| `every_resolved_value_appears_at_exactly_the_recorded_sites` | `packages/linker/src/tests/relocation_tests.rs` | 61 relocations | 55 | Removed literal sites |
| `each_live_demonstration_has_exactly_two_hundred_twenty_nine_pushed_symbol_sites` | `packages/tapscript/src/tests/live_bundle_tests.rs` | 42 symbols / 218 relocations / 247 sites | 40 / 200 / 229 | Both representations retain the explicit symbol and relocation assertions |
| `the_measured_shape_of_the_candidate_is_exactly_this` | `packages/tapscript/src/tests/bundle_tests.rs` | 3939 total bytes; change leaves 419/497/575; change coefficient 64 | 3861; 393/471/549; 38 | Compact emission loses only the equalities |
| `the_linked_script_bytes_are_the_pre_link_ones_moved_by_the_resolved_widths` | `packages/linker/src/tests/link_tests.rs` | 3975 linked bytes | 3861 | Also pinned by `total_script_bytes_is_the_exact_sum_of_linked_leaf_charges` |
| `the_census_splits_by_who_settles_each_symbol` | `packages/linker/src/tests/symbol_tests.rs` | 21 definitions: 6 deployment / 15 bundle | 19: 4 / 15 | Construction defaults have no leaf definition |
| `the_resource_formulas_are_refitted_and_predict_every_linked_measurement` | `packages/linker/src/tests/link_tests.rs` | Pre-link and linked models differ | Models agree | The removed literal supplied the only width change |
| `PINNED_PROGRAM` | `packages/vectors/src/bundle.rs` | pinned array; before value in the comment beside it | pinned array; before value in the comment beside it | Compact constructor commitment |
| `FIXTURE_MERKLE_ROOT` | `packages/target-elements-conformance/src/reference.rs` | pinned array; before value in the comment beside it | pinned array; before value in the comment beside it | Independent root pin |
| `FIXTURE_REFERENCE_OUTPUT_KEY` | `packages/target-elements-conformance/src/reference.rs` | pinned array; before value in the comment beside it | pinned array; before value in the comment beside it | Independent output-key pin; parity remains even |
| `SPLIT_SUCCESSOR_DIGEST` | `packages/vectors/src/live_multi_shapes.rs` | pinned constant; before value in the comment beside it | pinned constant; before value in the comment beside it | Current constructor |
| `MANY_TO_MANY_SUCCESSOR_DIGEST` | `packages/vectors/src/live_multi_shapes.rs` | pinned constant; before value in the comment beside it | pinned constant; before value in the comment beside it | Current constructor |
| `SEVERAL_OWNERS_SUCCESSOR_DIGEST` | `packages/vectors/src/live_multi_shapes.rs` | pinned constant; before value in the comment beside it | pinned constant; before value in the comment beside it | Current constructor |
| `STRICT_ONE_TO_ONE_SUCCESSOR_DIGEST` | `packages/vectors/src/live_multi_shapes.rs` | pinned constant; before value in the comment beside it | pinned constant; before value in the comment beside it | Current constructor |
| `FEE_BEARING_SUCCESSOR_DIGEST` | `packages/vectors/src/live_multi_shapes.rs` | pinned constant; before value in the comment beside it | pinned constant; before value in the comment beside it | Current constructor |
| `MERGE_SUCCESSOR_DIGEST` | `packages/vectors/src/live_multi_shapes.rs` | pinned constant; before value in the comment beside it | pinned constant; before value in the comment beside it | Current constructor |
| `EXIT_CROSSING_SUCCESSOR_DIGEST` | `packages/vectors/src/live_multi_shapes.rs` | pinned constant; before value in the comment beside it | pinned constant; before value in the comment beside it | Current constructor |
| `ENTRY_CROSSING_SUCCESSOR_DIGEST` | `packages/vectors/src/live_multi_shapes.rs` | pinned constant; before value in the comment beside it | pinned constant; before value in the comment beside it | Current constructor |
| `PURE_SPLIT_SUCCESSOR_DIGEST` | `packages/vectors/src/live_multi_shapes.rs` | pinned constant; before value in the comment beside it | pinned constant; before value in the comment beside it | Current constructor |
| `PAIRED_ONE_TO_ONE_SUCCESSOR_DIGEST` | `packages/vectors/src/live_multi_shapes.rs` | pinned constant; before value in the comment beside it | pinned constant; before value in the comment beside it | Current constructor |
| `PREDECESSOR_DIGEST` | `packages/vectors/src/live_private_restart.rs` | pinned constant; before value in the comment beside it | pinned constant; before value in the comment beside it | Current constructor |
| `SUCCESSOR_DIGEST` | `packages/vectors/src/live_private_restart.rs` | pinned constant; before value in the comment beside it | pinned constant; before value in the comment beside it | Current constructor |
| `PARITY_SUCCESSOR_DIGEST` | `packages/vectors/src/live_private_restart.rs` | pinned constant; before value in the comment beside it | pinned constant; before value in the comment beside it | Current constructor |
| `the_measured_table_is_the_one_this_study_reports` | `packages/vectors/src/live_measurements.rs` | Change-present member: weight 1904 / vsize 476 / depth 5 / peak 7 / budget 50 | 1866 / 467 / 5 / 7 / 50 | All 17 rows recounted; the other 16 retain every figure |
| `closure_leaf_bytes_and_roots_match_the_fixture` | `packages/vectors/fixtures/closure-compact-live.golden` | 135998 bytes, 133 lines; digest held by the regenerating test's fixture comparison | 133034 bytes, 133 lines; digest held by the regenerating test's fixture comparison | One current golden; the before snapshot precedes elision |
| `OPERATOR_SUCCESSOR_DIGEST` | `packages/vectors/fixtures/historical-candidate-0.7.15-dev-operator-successor.leaves` | Absent | 29 retained leaves; `OPERATOR_SUCCESSOR_DIGEST` | Replay material registered in `meson.build`; count pinned by `the_candidate_commits_twenty_nine_leaves_in_each_representation` in `packages/linker/src/tests/live_taptree_tests.rs` |

The digest assertions are `the_pre_wave_shapes_register_under_their_forward_v2_digests`, `the_shapes_this_wave_added_carry_digests_of_their_own`, `the_fee_bearing_current_digest_is_pinned` and `current_fixtures_register_under_current_forward_v2_digests`. Historical shape and restart comparisons use separate revision-seven constants and typed archive standing. Runtime carriers remain 28/30/28 and coverage 209/221/223 under `every_case_of_this_candidate_is_carried_by_exactly_one_site` and `every_coverage_requirement_has_exactly_one_row`; STATE stays 6 symbols/15 sites and variable metadata 7/16 under `the_composed_consumer_census_is_exactly_six_symbols_over_fifteen_sites`.

**Result (Admission)** · `result:backlog:elision-admission`

`CandidateStanding::HistoricalCandidate` binds deployment identities, target contract and program commitments separately from source provenance. Operator captures retain their report/capture suite revision; revision-seven captures explicitly have no source-revision field. Matching provenance alone establishes no current acceptance. Retained operator leaves are digest-bound before independent reconstruction, with deployment-asset relocation confined to complete literal pushes; exact subject equality, signature settlement, row comparison, message recomputation and mined readback remain required.

The admission roots `live_corpus_native_operator.rs::run_of_record` and `live_corpus_native_v2_r7.rs::run_of_record` admit an earlier candidate only as historical evidence for that candidate. Revision-seven imports retain their validated transcript and program commitments with `NotReplayableForCurrentCandidate`; they do not silently substitute current construction. Omitting replay would remove the independent subject check, and a production emitter compatibility mode would preserve obsolete covenant behavior; both alternatives are rejected. The retained fixture supplies replay material without changing current emission.

| Reader | Former standing | Standing after elision |
|---|---|---|
| `maturity_operator.rs::replay_recorded` | Current successor reconstruction | Retained historical successor; current construction remains separate |
| `maturity_corpus.rs` | Historical maturity runs borrowing operator deployment | Typed historical maturity programs and source; deployment agreement establishes no fresh acceptance |
| `live_multi_shapes.rs` | Current derivations compared with archive pins | Separate current pins and historical archive comparisons |
| `live_conservation_negatives.rs` | Current module pins compared with archive | Separate historical pins; fresh unequal subjects still refused |
| `live_native.rs` | Archive measurements in current transcript | Explicit historical transcript standing |
| `live_resource_report.rs` | Current sizing beside archived target measurements | Current host sizing and typed historical target observations |
| `tests/guide13_live_native.rs` | Exact comparisons against a current-named corpus | Historical comparison scope; changed subjects still fail equality |
| `live_private_restart.rs` | Recorded forward pair without candidate standing | Historical pair and candidate standing; current pins remain distinct |
| `live_evidence.rs` | Archive-derived evidence plan | Historical standing retained on corpus-derived evidence; synthetic overlays remain distinct |
| `packages/target-elements-conformance/src/tests/protocol_tests.rs` | Historical parser compatibility | Historical parser plus explicit absent-source and replay-limit assertions |
| `tests/maturity_operator.rs` | Admitted operator observations | Producing-candidate historical standing explicitly selected |
| `tests/sighash_verdict.rs` | Recorded owner-observation verdicts | Historical candidate explicitly selected |
| `live_explicit_shapes.rs` | Recorded shape acceptances | Historical producing-candidate identities explicitly acknowledged |
| `live_pairs.rs` | Recorded pair acceptances | Historical producing-candidate identities explicitly acknowledged |
| `live_pair_arc.rs` | Recorded sponsor prerequisite | Historical subject prerequisite, no fresh acceptance |
| `live_proof_bearing_observation.rs` | Recorded observation/acceptance pair | Both members retain the producing candidate's historical scope |
| `live_roles.rs` | Recorded role attribution | Historical accepted subject scope explicitly acknowledged |
| `shape_census.rs` | Recorded verdict and refusal identities | Historical subject scope for both accepted and refused observations |

The transitive maturity readers in `maturity_{branch,continuity,continuity_report,evidence,history,history_report,measurements,mutants,nonce_corpus,recovery,recovery_report,refinement,resources}.rs` and `tests/maturity_{continuity,recovery}.rs` explicitly select historical candidate standing from the same admitted runs; both validated maturity corpus types expose that standing. Independent STATE construction remains separate from archive observation. No byte under any `-run-of-record` directory changes.

**Result (Regressions)** · `result:backlog:elision-regressions`

`compact_linked_change_destination_pair` and `live_linked_change_destination_pairs` interpret the linked instructions: alternate ordinary payload and version are accepted, while wrong asset, missing role and extra role are refused. Compiler-plan checks alone cannot establish this result. The byte golden separately pins current emission and roots. The gate at (`gate:guide15-exec:bite-0-3-gate`) requires formatting, clippy and tests over eight crates, plan checks, the documentation suite and label tests.

The executable failure census first exposes three stale compact root/key assertions, then six vector failures after producing-candidate admission: one golden, one measurement table and four constructor-digest/current-versus-historical comparisons. No replay refusal remains in that run. The earlier 266-failure aggregate is not reproduced as an executable baseline and supplies no new count here. Test changes retain exact checks, separate historical comparisons and recount current pins; no test is removed.

**Caveat (Historical acceptance and prepared subjects)**

The reproduced maturity control retains its historical acceptance and unchanged STATE bytes; its live construction path changes. The Phase-4 compact positive remains acceptance of its original leaf bytes. The compact negative inventory retains 74 rows, its target-executable recipes and ABI boundary premises; classification counts remain those of (`rec:backlog:wave0-closure-audit`). `T12-005` must reproduce acceptance of the changed candidate on the pinned target; this elision supplies no native acceptance. The regional residuals named in the classification record retain their scope.

Revisions are named by tag, with `0.7.15-dev` as the before candidate. The record and the classification record jointly carry closure; `T12-004` and `T12-005` retain their obligations.

**Record (Wave 0 compact native campaign)** · `rec:backlog:wave0-compact-campaign`

The selected compact candidate is `0.7.16-dev` under ruling 128 (`rule:phase7:wave0-rulings`), with `FIXTURE_MERKLE_ROOT`, `FIXTURE_REFERENCE_OUTPUT_KEY` and `PINNED_PROGRAM` as its byte premises. The elision comparison retains 20 symbols / 55 relocations / 60 pushed sites and 3861 linked bytes; leaf bytes, compiler censuses and manifest keys are unchanged. The native roster is 48 rather than 47 because the historical output-cardinality-above-maximum discharge covers only its producing candidate and is repeated for the current candidate.

`packages/vectors/fixtures/compact-run-of-record/` contains exactly four immutable files: the report ceremony capture, its `compact-phases-1` timing sidecar, `MANIFEST.sha256` and `RUN-REPORT`. `compact_corpus::run_of_record` admits them once through closed grammar, manifest and report pins, schedule address recomputation, exact request matching and settlement replay. `CandidateStanding::CurrentCandidate` binds the selected pins and reconstructed issued-asset programs; historical access remains typed and every historical assertion retains its scope.

Control families A2/A3/A4 have two, three or four ASH inputs without a sponsor; N2/N3/N4 add one sponsor without change; C2/C3/C4 add one sponsor with change. Every row observed in this archive refers to it and to `current_archive_replays_sixteen_rows_after_their_paired_controls`; the shared schedule reconstructs the safe control, compares the recorded mutant coordinates, retains the intended and collateral requirements, settles signing, and checks accepted bytes against mined readback. Credit follows the paired control and `matches_boundary(expected, observed)`; a failed control suspends its family and a different refusal boundary earns no credit.

`packages/vectors/fixtures/compact-run-of-record-2/` contains exactly four immutable files of the second schedule, admitted once by `compact_corpus::second_run_of_record` through the same closed grammar, its own pins (`COMPACT_SECOND_MANIFEST_SHA256`, `COMPACT_SECOND_REPORT_SHA256` and `COMPACT_SECOND_RUN_ADDRESS` over the `v2` address input set) and settlement replay through the shared schedule, with `CandidateStanding::CurrentCandidate` bound to the same three pins and the same reconstructed program. Each of its eleven subjects changes its row's own coordinate (an ASH input's asset or program, an input under a foreign program, the sponsor input's asset, the successor's asset, the change asset, or a second sponsor input), is made consensus-valid by the smallest balancing edit (a second issued asset with one outlet output, or one added input), and names that edit's violations as collateral in its row; its paired control shares the funded packet except the replaced or added coin, and both records carry their exact prevouts. Every row observed there refers to `second_archive_replays_eleven_rows_after_their_paired_controls`. `compact_corpus::joined_campaign` requires equal key sets and equal pins, counts a row once if either archive observes it, and fills the remaining keys from the tested formal closures.

**Table (Disposition census)** · `tab:backlog:compact-dispositions`

| Subject | Control family | Boundary expected | Boundary observed | Disposition | Pinning test |
|---|---|---|---|---|---|
| `Absent/Cardinality/ObjectFamily { side: Input, object: Ash }/RuntimeCarrier/FocusedReject(CardinalityBelowMinimum)` | A2 | ScriptPathRejection | ScriptPathRejection | Observed; compact-run-of-record | `current_archive_replays_sixteen_rows_after_their_paired_controls` |
| `Absent/Cardinality/ObjectFamily { side: Input, object: Ash }/RuntimeCarrier/FocusedReject(CardinalityAboveMaximum { ceiling: Bound(AshBatchMax) })` | A4 | ScriptPathRejection | ScriptPathRejection | Observed; compact-run-of-record | `current_archive_replays_sixteen_rows_after_their_paired_controls` |
| `Present/Cardinality/ObjectFamily { side: Input, object: Ash }/RuntimeCarrier/FocusedReject(CardinalityBelowMinimum)` | N2 | ScriptPathRejection | ScriptPathRejection | Observed; compact-run-of-record | `current_archive_replays_sixteen_rows_after_their_paired_controls` |
| `Present/Cardinality/ObjectFamily { side: Input, object: Ash }/RuntimeCarrier/FocusedReject(CardinalityAboveMaximum { ceiling: Bound(AshBatchMax) })` | N4 | ScriptPathRejection | ScriptPathRejection | Observed; compact-run-of-record | `current_archive_replays_sixteen_rows_after_their_paired_controls` |
| `Present/Cardinality/ObjectFamily { side: Input, object: PlainLbtc }/RuntimeCarrier/FocusedReject(CardinalityAboveMaximum { ceiling: Bound(FeeSponsorInputMax) })` | N2 | ScriptPathRejection | ScriptPathRejection | Observed; compact-run-of-record-2; collateral: compiled transaction-wide input count | `second_archive_replays_eleven_rows_after_their_paired_controls` |
| `Absent/Cardinality/ObjectFamily { side: Output, object: Ash }/RuntimeCarrier/FocusedReject(CardinalityBelowMinimum)` | A2 | ScriptPathRejection | ScriptPathRejection | Observed; compact-run-of-record | `current_archive_replays_sixteen_rows_after_their_paired_controls` |
| `Absent/Cardinality/ObjectFamily { side: Output, object: Ash }/RuntimeCarrier/FocusedReject(CardinalityAboveMaximum { ceiling: Declared(Count(1)) })` | A2 | ScriptPathRejection | ScriptPathRejection | Observed; compact-run-of-record; historical observation retains its original scope | `current_archive_replays_sixteen_rows_after_their_paired_controls`; `the_five_script_path_refusals_discharge_exactly_the_one_linked_row` |
| `Present/Cardinality/ObjectFamily { side: Output, object: Ash }/RuntimeCarrier/FocusedReject(CardinalityBelowMinimum)` | N2 | ScriptPathRejection | ScriptPathRejection | Observed; compact-run-of-record | `current_archive_replays_sixteen_rows_after_their_paired_controls` |
| `Present/Cardinality/ObjectFamily { side: Output, object: Ash }/RuntimeCarrier/FocusedReject(CardinalityAboveMaximum { ceiling: Declared(Count(1)) })` | N2 | ScriptPathRejection | ScriptPathRejection | Observed; compact-run-of-record | `current_archive_replays_sixteen_rows_after_their_paired_controls` |
| `Present/Cardinality/ObjectFamily { side: Output, object: PlainLbtc }/RuntimeCarrier/FocusedReject(CardinalityAboveMaximum { ceiling: Declared(Count(1)) })` | C2 | ScriptPathRejection | ScriptPathRejection | Observed; compact-run-of-record | `current_archive_replays_sixteen_rows_after_their_paired_controls` |
| `Absent/Recognition/ObjectFamily { side: Input, object: Ash }/RuntimeCarrier/FocusedReject(WrongRecognizedAsset)` | A2 | ScriptPathRejection | ScriptPathRejection | Observed; compact-run-of-record-2; collateral: output cardinality; output closure; U aggregate conservation; canonical partition | `second_archive_replays_eleven_rows_after_their_paired_controls` |
| `Absent/Recognition/ObjectFamily { side: Input, object: Ash }/RuntimeCarrier/FocusedReject(WrongRecognizedObject)` | A2 | ScriptPathRejection | ScriptPathRejection | Observed; compact-run-of-record-2; collateral: input recognition; input closure; authenticated family membership | `second_archive_replays_eleven_rows_after_their_paired_controls` |
| `Present/Recognition/ObjectFamily { side: Input, object: Ash }/RuntimeCarrier/FocusedReject(WrongRecognizedAsset)` | N2 | ScriptPathRejection | ScriptPathRejection | Observed; compact-run-of-record-2; collateral: output cardinality; output closure; U aggregate conservation; canonical partition | `second_archive_replays_eleven_rows_after_their_paired_controls` |
| `Present/Recognition/ObjectFamily { side: Input, object: Ash }/RuntimeCarrier/FocusedReject(WrongRecognizedObject)` | N2 | ScriptPathRejection | ScriptPathRejection | Observed; compact-run-of-record-2; collateral: input recognition; input closure; authenticated family membership | `second_archive_replays_eleven_rows_after_their_paired_controls` |
| `Present/Recognition/ObjectFamily { side: Input, object: PlainLbtc }/RuntimeCarrier/FocusedReject(WrongRecognizedAsset)` | N2 | ScriptPathRejection | ScriptPathRejection | Observed; compact-run-of-record-2; collateral: input cardinality; output cardinality; output closure; sponsor isolation | `second_archive_replays_eleven_rows_after_their_paired_controls` |
| `Present/Recognition/ObjectFamily { side: Input, object: PlainLbtc }/RuntimeCarrier/FocusedReject(WrongRecognizedObject)` | N2 | ScriptPathRejection | none | Formally closed (discharged by sponsor authorization): `SponsorAuthorizationCoversCoordinate` | `sponsor_authorization_covers_uninspected_programs` |
| `Absent/Recognition/ObjectFamily { side: Output, object: Ash }/RuntimeCarrier/FocusedReject(WrongRecognizedAsset)` | A2 | ScriptPathRejection | ScriptPathRejection | Observed; compact-run-of-record-2; collateral: input cardinality; input closure; output cardinality; output closure; U aggregate conservation; canonical partition | `second_archive_replays_eleven_rows_after_their_paired_controls` |
| `Absent/Recognition/ObjectFamily { side: Output, object: Ash }/RuntimeCarrier/FocusedReject(WrongRecognizedObject)` | A2 | ScriptPathRejection | ScriptPathRejection | Observed; compact-run-of-record | `current_archive_replays_sixteen_rows_after_their_paired_controls` |
| `Present/Recognition/ObjectFamily { side: Output, object: Ash }/RuntimeCarrier/FocusedReject(WrongRecognizedAsset)` | N2 | ScriptPathRejection | ScriptPathRejection | Observed; compact-run-of-record-2; collateral: input cardinality; input closure; output cardinality; output closure; U aggregate conservation; canonical partition | `second_archive_replays_eleven_rows_after_their_paired_controls` |
| `Present/Recognition/ObjectFamily { side: Output, object: Ash }/RuntimeCarrier/FocusedReject(WrongRecognizedObject)` | N2 | ScriptPathRejection | ScriptPathRejection | Observed; compact-run-of-record | `current_archive_replays_sixteen_rows_after_their_paired_controls` |
| `Present/Recognition/ObjectFamily { side: Output, object: PlainLbtc }/RuntimeCarrier/FocusedReject(WrongRecognizedAsset)` | C2 | ScriptPathRejection | ScriptPathRejection | Observed; compact-run-of-record-2; collateral: input cardinality; input closure; output cardinality; output closure; sponsor isolation | `second_archive_replays_eleven_rows_after_their_paired_controls` |
| `Present/Recognition/ObjectFamily { side: Output, object: PlainLbtc }/RuntimeCarrier/FocusedReject(WrongRecognizedObject)` | C2 | ScriptPathRejection | none | Formally closed (discharged by sponsor authorization): `SponsorAuthorizationCoversCoordinate` | `sponsor_authorization_covers_uninspected_programs` |
| `Absent/Conservation/Asset { asset: U }/RuntimeCarrier/FocusedReject(AmountMismatch)` | A2 | ScriptPathRejection | ScriptPathRejection | Observed; compact-run-of-record | `current_archive_replays_sixteen_rows_after_their_paired_controls` |
| `Present/Conservation/Asset { asset: U }/RuntimeCarrier/FocusedReject(AmountMismatch)` | N2 | ScriptPathRejection | ScriptPathRejection | Observed; compact-run-of-record | `current_archive_replays_sixteen_rows_after_their_paired_controls` |
| `Absent/AllowedObjectFamilies/TransactionSide { side: Input }/RuntimeCarrier/FocusedReject(UndeclaredObjectFamily)` | A2 | ScriptPathRejection | ScriptPathRejection | Observed; compact-run-of-record-2; collateral: input recognition; input closure; authenticated family membership | `second_archive_replays_eleven_rows_after_their_paired_controls` |
| `Present/AllowedObjectFamilies/TransactionSide { side: Input }/RuntimeCarrier/FocusedReject(UndeclaredObjectFamily)` | N2 | ScriptPathRejection | ScriptPathRejection | Observed; compact-run-of-record-2; collateral: input recognition; input closure; authenticated family membership | `second_archive_replays_eleven_rows_after_their_paired_controls` |
| `Absent/AllowedObjectFamilies/TransactionSide { side: Output }/RuntimeCarrier/FocusedReject(UndeclaredObjectFamily)` | A2 | ScriptPathRejection | ScriptPathRejection | Observed; compact-run-of-record | `current_archive_replays_sixteen_rows_after_their_paired_controls` |
| `Present/AllowedObjectFamilies/TransactionSide { side: Output }/RuntimeCarrier/FocusedReject(UndeclaredObjectFamily)` | N2 | ScriptPathRejection | ScriptPathRejection | Observed; compact-run-of-record | `current_archive_replays_sixteen_rows_after_their_paired_controls` |
| `Absent/CanonicalDeltaPolicy/Operation/RuntimeCarrier/FocusedReject(MissingCanonicalDeltaFamily)` | A2 | ScriptPathRejection | none | Formally closed (absence of coordinate): `NoIndependentDeltaFamilyCoordinate` | `compact_delta_family_has_no_independent_carrier_coordinate` |
| `Absent/CanonicalDeltaPolicy/Operation/RuntimeCarrier/FocusedReject(UnexpectedCanonicalDeltaFamily)` | A2 | ScriptPathRejection | none | Formally closed (absence of coordinate): `NoIndependentDeltaFamilyCoordinate` | `compact_delta_family_has_no_independent_carrier_coordinate` |
| `Absent/CanonicalDeltaPolicy/Operation/RuntimeCarrier/FocusedReject(DuplicateCanonicalSourceOrDestination)` | A2 | ScriptPathRejection | none | Formally closed (absence of coordinate): `NoIndependentCanonicalMembershipCoordinate` | `compact_canonical_membership_is_derived_once_per_position` |
| `Present/CanonicalDeltaPolicy/Operation/RuntimeCarrier/FocusedReject(MissingCanonicalDeltaFamily)` | N2 | ScriptPathRejection | none | Formally closed (absence of coordinate): `NoIndependentDeltaFamilyCoordinate` | `compact_delta_family_has_no_independent_carrier_coordinate` |
| `Present/CanonicalDeltaPolicy/Operation/RuntimeCarrier/FocusedReject(UnexpectedCanonicalDeltaFamily)` | N2 | ScriptPathRejection | none | Formally closed (absence of coordinate): `NoIndependentDeltaFamilyCoordinate` | `compact_delta_family_has_no_independent_carrier_coordinate` |
| `Present/CanonicalDeltaPolicy/Operation/RuntimeCarrier/FocusedReject(DuplicateCanonicalSourceOrDestination)` | N2 | ScriptPathRejection | none | Formally closed (absence of coordinate): `NoIndependentCanonicalMembershipCoordinate` | `compact_canonical_membership_is_derived_once_per_position` |
| `Absent/OpenFlowPolicy/Operation/RuntimeCarrier/FocusedReject(UndeclaredOpenFlow)` | A2 | ScriptPathRejection | none | Formally closed (absence of coordinate): `NoIndependentOpenFlowClaimCoordinate` | `compact_open_flow_claims_have_no_carrier_encoding` |
| `Present/OpenFlowPolicy/Operation/RuntimeCarrier/FocusedReject(UndeclaredOpenFlow)` | N2 | ScriptPathRejection | none | Formally closed (absence of coordinate): `NoIndependentOpenFlowClaimCoordinate` | `compact_open_flow_claims_have_no_carrier_encoding` |
| `Absent/SponsorIsolation/Sponsor/RuntimeCarrier/FocusedReject(SponsorProtocolOverlap)` | A2 | ScriptPathRejection | none | Formally closed (absence of coordinate): `EmptySponsorClaimDomain` | `compact_absent_sponsor_claim_domain_is_empty` |
| `Absent/SponsorIsolation/Sponsor/RuntimeCarrier/FocusedReject(SponsorMemberUnclaimed)` | A2 | ScriptPathRejection | none | Formally closed (absence of coordinate): `EmptySponsorClaimDomain` | `compact_absent_sponsor_claim_domain_is_empty` |
| `Absent/SponsorIsolation/Sponsor/RuntimeCarrier/FocusedReject(MissingSponsorAuthorization)` | A2 | ScriptPathRejection | none | Formally closed: `NoSponsorAuthorizationCoordinate` | `no_sponsor_authorization_coordinate_covers_a2_a3_a4` |
| `Present/SponsorIsolation/Sponsor/RuntimeCarrier/FocusedReject(SponsorProtocolOverlap)` | N2 | ScriptPathRejection | none | Formally closed (absence of coordinate): `NoIndependentSponsorMembershipCoordinate` | `compact_sponsor_membership_is_fixed_by_disjoint_positions` |
| `Present/SponsorIsolation/Sponsor/RuntimeCarrier/FocusedReject(SponsorMemberUnclaimed)` | N2 | ScriptPathRejection | none | Formally closed (absence of coordinate): `NoIndependentSponsorMembershipCoordinate` | `compact_sponsor_membership_is_fixed_by_disjoint_positions` |
| `Present/SponsorIsolation/Sponsor/RuntimeCarrier/FocusedReject(MissingSponsorAuthorization)` | N2 | ScriptPathRejection | ScriptPathRejection | Observed; compact-run-of-record | `current_archive_replays_sixteen_rows_after_their_paired_controls` |
| `Absent/RootPolicy/Operation/RuntimeCarrier/FocusedReject(WrongRootEffect)` | A2 | ScriptPathRejection | none | Formally closed (absence of coordinate): `NoIndependentRootEffectCoordinate` | `compact_root_effects_have_no_independent_carrier_coordinate` |
| `Present/RootPolicy/Operation/RuntimeCarrier/FocusedReject(WrongRootEffect)` | N2 | ScriptPathRejection | none | Formally closed (absence of coordinate): `NoIndependentRootEffectCoordinate` | `compact_root_effects_have_no_independent_carrier_coordinate` |
| `Absent/ProjectionPolicy/Operation/RuntimeCarrier/FocusedReject(MissingRequiredProjection)` | A2 | ScriptPathRejection | none | Formally closed (absence of coordinate): `DerivedTransitionCertificateHasNoRemovalCoordinate` | `compact_transition_certificate_is_derived_not_serialized` |
| `Absent/ProjectionPolicy/Operation/RuntimeCarrier/FocusedReject(ForbiddenProjectionPresent)` | A2 | ScriptPathRejection | none | Formally closed (absence of coordinate): `NoIndependentSpecializedProjectionCoordinate` | `compact_specialized_projection_has_no_carrier_coordinate` |
| `Present/ProjectionPolicy/Operation/RuntimeCarrier/FocusedReject(MissingRequiredProjection)` | N2 | ScriptPathRejection | none | Formally closed (absence of coordinate): `DerivedTransitionCertificateHasNoRemovalCoordinate` | `compact_transition_certificate_is_derived_not_serialized` |
| `Present/ProjectionPolicy/Operation/RuntimeCarrier/FocusedReject(ForbiddenProjectionPresent)` | N2 | ScriptPathRejection | none | Formally closed (absence of coordinate): `NoIndependentSpecializedProjectionCoordinate` | `compact_specialized_projection_has_no_carrier_coordinate` |
| `permissionless-private-dependency` | sponsorless + sponsored (2) | dependency owner | exact dependency refusal | First-party discharged | `the_permissionless_private_dependency_row_is_discharged_end_to_end` |
| `constructibility-witness-unavailable` | sponsorless + sponsored (2) | analysis | supporting `PermissionlessPrivateDependency`; no requested-class observation; before: none | First-party formal; before: retained subject only, no closure | `compact_first_party_dispositions_retain_all_twenty_subjects` |
| `unsupported-representation` | sponsorless + sponsored (2) | analysis | `TargetPlanRepresentationMismatch`; before: none | First-party observed; before: retained subject only, no closure | `compact_first_party_dispositions_retain_all_twenty_subjects` |
| `sponsor-multiplicity-exceeded` | sponsorless + sponsored (2) | analysis | `SponsorEnvelopeMultiplicity`; before: none | First-party observed; before: retained subject only, no closure | `compact_first_party_dispositions_retain_all_twenty_subjects` |
| `missing-compact-exit-analysis` | sponsorless + sponsored (2) | analysis | `MissingLifecyclePath` for Compact; before: none | First-party observed; before: retained subject only, no closure | `compact_first_party_dispositions_retain_all_twenty_subjects` |
| `missing-clear-exit-analysis` | sponsorless + sponsored (2) | analysis | `MissingLifecyclePath` for Clear; before: none | First-party observed; before: retained subject only, no closure | `compact_first_party_dispositions_retain_all_twenty_subjects` |
| `unexpected-protocol-secret` | sponsorless + sponsored (2) | emitted structure | `UnexpectedProtocolSecret`; before: none | First-party observed; before: retained subject only, no closure | `compact_first_party_dispositions_retain_all_twenty_subjects` |
| `unauthenticated-representation` | sponsorless + sponsored (2) | emitted structure | `MissingExplicitValueAuthentication`; before: none | First-party observed; before: retained subject only, no closure | `compact_first_party_dispositions_retain_all_twenty_subjects` |
| `missing-compact-exit-emitted` | sponsorless + sponsored (2) | emitted structure | `MissingCompactExit`; before: none | First-party observed; before: retained subject only, no closure | `compact_first_party_dispositions_retain_all_twenty_subjects` |
| `missing-clear-exit-emitted` | sponsorless + sponsored (2) | emitted structure | `MissingClearDeclaration`; before: none | First-party declaration observed; before: retained subject only, no closure | `compact_first_party_dispositions_retain_all_twenty_subjects` |
| `ExternalEvidenceMissing` | sponsorless + sponsored (2) | external report | `ExternalEvidenceMissing`; before: none | External report observed; before: retained, no closure | `substrate_closure_admits_six_observed_and_zero_retained_report_rows` |
| `ExternalEvidenceFailed` | sponsorless + sponsored (2) | external report | `ExternalEvidenceFailed`; before: none | External report observed; before: retained, no closure | `substrate_closure_admits_six_observed_and_zero_retained_report_rows` |
| `ExternalEvidenceIdentityMismatch` | sponsorless + sponsored (2) | external report | `ExternalEvidenceIdentityMismatch`; before: none | External report observed; before: retained, no closure | `substrate_closure_admits_six_observed_and_zero_retained_report_rows` |

`NoSponsorAuthorizationCoordinate` closes only the absent-sponsor authorization-removal requirement: A2, A3 and A4 contain only ASH script paths, so removing their script material changes an ASH path while no sponsor authorization coordinate exists to remove.

Discharge by sponsor authorization (`SponsorAuthorizationCoversCoordinate`) closes the two wrong-object sponsor rows: a sponsor's all-inputs-all-outputs signature binds its own input and change programs, so the covenant inspects neither and an alternate ordinary destination is accepted by design. Absence of coordinate closes the eighteen semantic rows: a relation whose coordinate has no independent carrier encoding cannot be mutated in isolation on this candidate, because every leaf takes zero witness data items, the coordinator inspects programs only at the ASH positions, the successor and the fee, and the layouts claim every position exactly once, so no delta family, membership list, open-flow claim, sponsor claim, root effect or projection set is serialized. Each closure test binds its exact keys, the expected boundary, the three pins, the twelve programs, the nine layouts, the 3861 linked bytes and every retained instruction; a closure is never counted as an observation.

**Result (Campaign)** · `result:backlog:compact-campaign`

One ceremony executes 92 operations: 1 issuance + 1 constructor probe + 1 sponsor funding batch + 25 ASH funding batches + 23 signings + 41 submissions, pinned by `recorded_operation_census_is_the_pinned_funding_and_signing_schedule`. The submissions comprise nine initial controls, sixteen mutants and sixteen paired controls. Nine control families accept, all sixteen mutants refuse at ScriptPathRejection, zero refuse elsewhere, zero mutants accept, and zero families are suspended. The eligible report and the four-file archive are bound by `COMPACT_REPORT_SHA256`, `COMPACT_MANIFEST_SHA256` and `COMPACT_RUN_ADDRESS`; the archive supplies current positive controls without promoting a historical positive pass.

Attempt 1 completes the same operation census but remains ineligible because the summary shares the test-status prefix; a leading newline makes the summary independently readable. Attempt 2 is eligible and supplies the archive; its total capture wall is 6563 ms, including invocation 6470 ms, and its sidecar separately records staging, execution and total durations. The preliminary probe took 5492 ms in total, including 3903 ms of execution; these measurements supply no observation credit. The failed attempt and scratch realization remain outside the archive. `every_file_is_bound_before_replay` and `rebound_subjects_signatures_readback_and_wrong_boundaries_cannot_gain_credit` pin the admission boundary.

**Result (Second campaign)** · `result:backlog:compact-campaign-second`

One ceremony under the second schedule executes 87 operations: 2 issuances + 1 constructor probe + 1 sponsor funding batch of 14 coins + 30 funding batches + 22 signings + 31 submissions, derived from the second roster by `compact_campaign_second::operation_census` and pinned by `second_schedule_settles_offline_to_its_derived_census`. The submissions comprise nine initial controls, eleven mutants and eleven paired controls. Nine control families accept, all eleven mutants refuse at ScriptPathRejection with the script verifier's equal-verify failure, zero refuse elsewhere, zero mutants accept, and zero families are suspended. The first record-mode attempt is eligible and supplies the archive; its total capture wall is 6720 ms, including invocation 5355 ms, and its sidecar records staging 1093 ms, execution 3935 ms and total 5178 ms. The preliminary probe of the same schedule took 6606 ms in total, including 3889 ms of execution; it stays outside the archive, and these measurements supply no observation credit. `second_archive_replays_eleven_rows_after_their_paired_controls` pins the second admission boundary, including every file's binding and the refusal of these files under the first archive's specification.

**Result (Closure)** · `result:backlog:compact-closure`

At native-campaign closure, 74 = 27 observed + 21 formally closed + 0 unresolved + 2 first-party discharged + 18 first-party retained + 6 external retained. Current closure is 74 = 27 native observed + 21 native formal + 18 first-party observed + 2 first-party formal + 6 report observed, with zero retained or unresolved, at (`rec:backlog:wave0-compact-inherited-closure`).

The 21 native formal closures are 1 by absent sponsor authorization coordinate, 2 discharged by sponsor authorization and 18 by absence of coordinate. Native standing moved from 16 observed / 1 formally closed / 31 unresolved to 27 / 21 / 0, independently of the historical discharge. `CompactAshEvidencePlan::current_campaign_census` and `joined_campaign_counts_each_row_once_across_both_archives` pin [27, 21, 0, 0]; `record_dispositions_bind_exactly_to_the_current_manifest` binds all 48 native table rows. At native closure, first-party standing was 2 discharged / 0 formal / 18 retained and external standing 6 retained; (`T12-004`) closed only the native half. (`T4-009`) closes separately at (`rec:backlog:wave0-compact-inherited-closure`) after ruling 136 and the nonnative evidence.

**Caveat (Manifest classification questions)**

The twenty native classification questions concerned two `WrongRecognizedObject` sponsor requirements discharged by sponsor authorization and eighteen absent-coordinate requirements. The eighteen semantic rows (`CanonicalDeltaPolicy` 6, `OpenFlowPolicy` 2, `SponsorIsolation` overlap and unclaimed 4, `RootPolicy` 2, `ProjectionPolicy` 4) name metadata whose coordinate never reaches the carrier, so a script-path refusal cannot be their expected boundary. They retain `ScriptPathRejection` as the inherited expected-boundary field, with no observed script refusal claimed; ruling 136 supplies the final reading below.

Ruling 136 resolves these twenty classification questions at the selected candidate: the published manifest identities and expected-boundary fields remain, with `ScriptPathRejection` naming the inherited classification rather than an observable refusing coordinate or an observed refusal; the typed formal-closure reasons supply the answers (`rule:phase7:wave0-rulings`). The two sponsor-authorization closures remain distinct from the eighteen absent-coordinate closures, partitioned 6 / 2 / 4 / 2 / 4; compiler coverage, archive bytes, key sets, closure tests and 48 / 20 / 6 remain unchanged. This ruling alone did not close the then-retained first-party and external obligations; their separate closure is recorded at (`rec:backlog:wave0-compact-inherited-closure`).

**Caveat (Retained obligations and timing)**

The guide's 72-row count was stale against the manifest's 74 at native closure; ruling 139 reconciled it (`rule:phase7:wave0-rulings`). Earlier retained timing sidecars carried no durations, so they supported no numeric wall estimate; an action without a reported duration is a process defect. The then-retained eighteen first-party rows and six external rows supplied no closure by retention; performed comparisons and the distinct formal arguments now close them at (`rec:backlog:wave0-compact-inherited-closure`). The second archive's replay and the twenty closure tests extended admission without weakening a production constructor or removing a test.

**Record (Wave 0 Safety first-party dispositions)** · `rec:backlog:wave0-safety-first-party`

The supplementary ledger disposes the 43 first-party-required keys under (`step:wave0:bite-0-5-first-party`) and (`rule:guide15-exec:observable-retention`), preserving each inherited mechanism and relation link. The keys come from `maturity_first_party_cases`, not the numerically equal native-observation-reachable figure in `maturity_safety.rs`. This is a disposition step with remaining obligations, not closure of (`gate:guide15-exec:bite-0-5-first-party-gate`).

The initial five comparisons remain distinct from the eight subsequent existing-entry comparisons at (`rec:backlog:wave0-first-party-entries`). Retained entries require their exact offered input and validation route; a native or report refusal at another boundary supplies no first-party comparison.

`MaturityConstructorComparisonTestStaging` retains normally derived constructor pairs for continuity; `MaturitySymbolResolutionTestStaging` retains complete and single-definition-omitted censuses for resolution. The same entry accepts the valid object and refuses its malformed sibling with the exact class. Continuity supplies no claim that derivation rejects a different well-formed tree. No production validator changes.

**Table (First-party Safety rows)** · `tab:backlog:safety-first-party-rows`

| Subject | Owning validator | Relation checked | Disposition | Pinning test |
|---|---|---|---|---|
| `16.2 / malformed-cycle-encoding` | `MaturityAnnouncementRequest::decode_announced_cycle` | Accepted control and 1 exact changed offer; `CycleWidth` | comparison | `entry_pair_cycle_decoding_reads_the_bound_writer_item` |
| `16.2 / request-cycle-differs-from-successor` | `OperatorAuthorizedMaturityAnnouncement::check_offered` | Authorized request-derived successor; only its offered announced cycle changes; MaturitySuccessorMetadataChangedAfterSigning | comparison | `entry_pair_successor_cycle` |
| `16.3 / correct-semantic-metadata-with-noncanonical-representation-nonce` | `CandidateStateConstructor::compare_offered_representation` | Accepted control and 1 exact changed offer; `NoncanonicalRepresentationNonce` | comparison | `entry_pair_nonce_comparison` |
| `16.5 / wrong-static-subtree` | `CandidateStateConstructor::continuity` | Retained versus changed static root; ConflictingLeaf | comparison | `supplementary_comparisons_retain_exact_objects_and_refusals` |
| `16.5 / metadata-leaf-missing` | `PublicMaturityStateView::validate` | Complete committed program; offered program commits the same static subtree without the metadata branch; MaturityViewProgramNotReconstructed | comparison | `entry_pair_metadata_omission` |
| `16.5 / extra-escape-leaf` | `CandidateStateConstructor::continuity` | Retained root versus root with extra leaf; ConflictingLeaf | comparison | `supplementary_comparisons_retain_exact_objects_and_refusals` |
| `16.7 / caller-supplied-branch-order` | `StateStaticSubtree::new` | Equal root, leaf hashes and paths; topology and leaf changes excluded | absence of coordinate | `absence_of_coordinate_is_only_unchanged_content_ordering` |
| `16.7 / source-order-dependent-tree` | `assemble_state_static` | Equal canonical assembly and paths; topology and leaf changes excluded | absence of coordinate | `absence_of_coordinate_is_only_unchanged_content_ordering` |
| `16.9 / add-live-receipt-input-or-output` | `transaction::validate_maturity_abi_classification` | Accepted control and 2 exact changed offers; `RelationVerdicts(UndeclaredObjectFamily)` plus CanonicalDeltaPolicy | comparison | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.9 / add-time-locked-receipt` | `transaction::validate_maturity_abi_classification` | Accepted control and 1 exact changed offer; `RelationVerdicts(UndeclaredObjectFamily)` plus CanonicalDeltaPolicy | comparison | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.9 / add-ash` | `transaction::validate_maturity_abi_classification` | Accepted control and 1 exact changed offer; `RelationVerdicts(UndeclaredObjectFamily)` plus CanonicalDeltaPolicy | comparison | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.9 / add-entitlement` | `transaction::validate_maturity_abi_classification` | Accepted control and 1 exact changed offer; `RelationVerdicts(UndeclaredObjectFamily)` plus CanonicalDeltaPolicy | comparison | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.9 / add-request` | `transaction::validate_maturity_abi_classification` | Accepted control and 1 exact changed offer; `RelationVerdicts(UndeclaredObjectFamily)` | comparison | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.9 / add-distribution-control-or-vault` | `transaction::validate_maturity_abi_classification` | Accepted control and 2 exact changed offers; `RelationVerdicts(UndeclaredObjectFamily)` plus CanonicalDeltaPolicy | comparison | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.9 / add-resv-input-or-output` | `transaction::validate_maturity_abi_classification` | Accepted control and 2 exact changed offers; `RelationVerdicts(UndeclaredObjectFamily)` | comparison | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.9 / add-pace` | `transaction::validate_maturity_abi_classification` | Accepted control and 1 exact changed offer; `RelationVerdicts(UndeclaredObjectFamily)` | comparison | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.9 / add-authority-object` | `transaction::validate_maturity_abi_classification` | Accepted control and 2 exact changed offers; `RelationVerdicts(UndeclaredObjectFamily)` | comparison | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.9 / add-issuance` | `transaction::validate_maturity_abi_classification` | Accepted control and 1 exact changed offer; `IssuanceOffered` | comparison | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.9 / add-destruction` | `transaction::validate_maturity_abi_classification` | Accepted control and 1 exact changed offer; `UnadmittedOutputRole(Destruction)` | comparison | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.9 / add-burn-record` | `transaction::validate_maturity_abi_classification` | Accepted control and 1 exact changed offer; `UnadmittedOutputRole(Burn)` | comparison | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.9 / add-clear-event` | `transaction::validate_maturity_abi_classification` | Accepted control and 1 exact changed offer; `UnadmittedOutputRole(Recon)` | comparison | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.9 / add-residue-projection` | `transaction::validate_maturity_abi_classification` | Accepted control and 1 exact changed offer; `RelationVerdicts(ProjectionPolicy)` | comparison | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.9 / introduce-canonical-u-ent-or-dist-ctl-flow` | `transaction::validate_maturity_abi_classification` | Accepted control and 3 exact changed offers; `RelationVerdicts(CanonicalDeltaPolicy)` with both closure failures | comparison | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.11 / wrong-coordinator` | `transaction::validate_maturity_abi_roles` | Accepted control and 1 exact changed offer; `CoordinatorIsNotTheValidatedPredecessor` | comparison | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.11 / duplicate-coordinator` | `transaction::validate_maturity_abi_roles` | Accepted control and 1 exact changed offer; `CoordinatorRepeated` | comparison | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.11 / no-coordinator` | `transaction::validate_maturity_abi_roles` | Accepted control and 1 exact changed offer; `CoordinatorMissing` | comparison | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.11 / state-and-sponsor-positions-exchanged` | `transaction::validate_maturity_abi_roles` | Accepted control and 1 exact changed offer; `StateAndSponsorPositionsExchanged` | comparison | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.11 / wrong-input-count` | `transaction::validate_maturity_abi_roles` | Accepted control and 1 exact changed offer; `Cardinality(Input)` | comparison | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.11 / wrong-output-count` | `transaction::validate_maturity_abi_roles` | Accepted control and 1 exact changed offer; `Cardinality(Output)` | comparison | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.11 / unresolved-metadata-schema-symbol` | `resolve_state_census` | Consumed definitions versus omitted MetadataSchema; MissingDefinition(MetadataSchema) | comparison | `supplementary_comparisons_retain_exact_objects_and_refusals` |
| `16.11 / unresolved-lead-bound-symbol` | `resolve_state_census` | Consumed definitions versus omitted MaturityLeadMin; MissingDefinition(MaturityLeadMin) | comparison | `supplementary_comparisons_retain_exact_objects_and_refusals` |
| `16.11 / unresolved-operator-symbol` | `resolve_state_census` | Consumed definitions versus omitted CommittedOperatorKey; MissingDefinition(CommittedOperatorKey) | comparison | `supplementary_comparisons_retain_exact_objects_and_refusals` |
| `16.11 / conflicting-leaf-weight` | `TaptreeInput<StateLeafRole>::new` | Unique weighted declaration; add the same role with a conflicting weight; DuplicateStateTreeLeaf(Announcement) | comparison | `entry_pair_conflicting_weight` |
| `16.11 / candidate-outside-bounds` | `StateLinkedTaptree::bind` | Complete tree under its accepting cap; lower only that cap to the positive static depth; StateTreeDepthExceeded | comparison | `entry_pair_depth_cap` |
| `16.12 / blank-request` | `RequestReader.read` | Nonempty object record versus two spaces, with newline retained; `blank_record` | comparison | `adapter_request_comparisons_bind_exact_pairs` |
| `16.12 / oversized-request` | `RequestReader.read` | Object record versus the same object with 4194303 trailing spaces and final newline; `oversized_record` | comparison | `adapter_request_comparisons_bind_exact_pairs` |
| `16.12 / unterminated-request` | `RequestReader.read` | Object record versus the same bytes without the final newline; `unterminated_record` | comparison | `adapter_request_comparisons_bind_exact_pairs` |
| `16.12 / malformed-json` | `RequestReader.read_object` | Object record versus replacement of its closing delimiter by a closing array delimiter; `malformed_record` | comparison | `adapter_request_comparisons_bind_exact_pairs` |
| `16.12 / unknown-request-field` | `require_execution_request_fields` | Declared schema, case and subject fields versus one additional unknown field; `EXECUTION_REQUEST_FIELD_CENSUS_FAILED` | comparison | `adapter_request_comparisons_bind_exact_pairs` |
| `16.12 / wrong-schema` | `compare_environment` | Matching environment observation; change only schema; UnsupportedProtocolSchema | comparison | `entry_pair_schema` |
| `16.12 / wrong-environment` | `compare_environment` | Matching environment observation; change only network identifier; EnvironmentBindingMismatch | comparison | `entry_pair_environment` |
| `16.12 / wrong-provenance` | `ExecutionTranscript::provenance_agreement` | Matching independently stated expectation; add one reported topic, reconstructing both transcripts through run_protocol; TopicCensusIsNotTheExpectedCensus | comparison | `entry_pair_provenance` |
| `16.12 / signing-response-bound-to-other-bytes` | `bind_script_path_response` | Shape-valid accepted signing response; change one byte of signature_bound_to alone; ScriptPathTransactionMismatch | comparison | `entry_pair_signature_binding` |

**Result (First-party census)** · `result:backlog:safety-first-party`

`supplementary_safety_keys_and_census_are_exact` pins the current partition recorded at (`rec:backlog:wave0-adapter-comparisons`). The initial 5 / 2 / 36 stage remains reconstructed separately at (`rec:backlog:wave0-close-safety`). It also pins the inherited standings at 0 deferred owners and 43 claims with no offered public input, retaining the distinct typed variants and original reasons.

`supplementary_dispositions_join_required_keys_without_promoting_historical_answers` pins the historical Safety census at 206 / 78 / 128 and its first-party 40 / 43, native 35, report-layer 15 and typed-non-answer 35 buckets. The initial five comparison discharges are reconciled by the bounded consolidation; eight later comparisons are recorded at (`rec:backlog:wave0-first-party-entries`). The ABI and adapter admissions follow at (`rec:backlog:wave0-abi-entries`) and (`rec:backlog:wave0-adapter-comparisons`). None changes the identity of the 78 historical answers. The historical 83 = 40 + 43 partition remains unchanged.

`supplementary_comparisons_retain_exact_objects_and_refusals` pins two constructor pairs and three distinct missing-symbol pairs, the existing-entry pairs and five source-bound Python pairs. `supplementary_admission_refuses_copied_pairs_and_wrong_bindings` refuses copied objects, invalid controls, accepted changed inputs and mismatched row or symbol bindings. `safety_disposition_record_matches_the_typed_ledger` binds all table keys, entries and dispositions to the ledger.

The two formal closures are absence of coordinate: canonical assembly erases offered order, leaving equal roots, leaf hashes and paths. `absence_of_coordinate_is_only_unchanged_content_ordering` pins both keys and excludes topology and leaf changes; an observable refusal at another boundary cannot use this argument.

**Caveat (Retained observable obligations)**

`retained_obligations_have_valid_subjects_and_named_boundaries` now pins no retained adapter subject. The fifteen classification rows, six role rows, cycle decoder and nonce comparison have performed comparisons; the five Python request comparisons are admitted at (`rec:backlog:wave0-adapter-comparisons`) at their framing, JSON and field entries. A refusal from another boundary supplies none of that admission.

At the initial first-party admission, the 35 native rows, sponsored construction and positive-floor observations, independent edge and successor observations, fifteen report-layer rows, relation reconciliation and five conjunct dispositions remained with `T12-005`. That historical step supplied no target observation or archive amendment; subsequent adapter admission is recorded at (`rec:backlog:wave0-adapter-comparisons`) and the current census at (`result:backlog:close-safety-reconciliation`).

**Record (Wave 0 Safety native staging)** · `rec:backlog:wave0-safety-native-staging`

The native staging sorts every §16 row a native run is still owed on the maturity candidate of record into one of four places, and stages the one-coordinate subjects that a later conformance run can submit to observe the project's own program refusing them or, where the covenant does not decide the coordinate, accepting them. The candidate of record is the accepted variable-schedule maturity control's deployment. The keys come from `maturity_first_party_cases`'s native-run-required partition and the register `STILL_REQUIRED`; the subjects depart from that control, which the planner rebuilds byte-equal to the archived submission. This is a staging step with remaining obligations, not a run: nothing is submitted, no target runs, and no fixture or evidence directory is written.

Each staged subject moves exactly one coordinate and, where that coordinate is a signed field, is re-signed under the published operator key so authorization stays out of the confound set. Each distinct subject is pinned by the digest of its spend-time serialization in a `PIN_*` constant, recomputed from the admitted control; no production validator changes.

**Table (Native Safety subjects)** · `tab:backlog:safety-native-subjects`

The table carries current construction standings; the historical result below remains the observation baseline. The two added constructions are recorded at (`rec:backlog:wave0-native-route-construction`); their admitted observations and removal of both construction waits are recorded at (`rec:backlog:wave0-native-route-capture-driver`).

| Subject | Mutation | Control route | Declared boundary | Standing | Pinning test |
|---|---|---|---|---|---|
| `16.2 / successor-remains-unannounced` | output 0 to the predecessor semantic at its first admissible nonce; shared subject with `16.6 / successor-from-predecessor-metadata-unchanged` | sponsorless announcement | ScriptPathRejection | staged; refusal expected | `every_staged_subject_matches_its_pin` and `each_row_standing_is_pinned_by_key` |
| `16.2 / successor-becomes-complete` | output 0 to the same semantic with the completed status | sponsorless announcement | ScriptPathRejection | staged; refusal expected | `every_staged_subject_matches_its_pin` and `each_row_standing_is_pinned_by_key` |
| `16.3 / metadata-from-another-state-object` | witness item 4 to another state object's record | sponsorless announcement | ScriptPathRejection | staged; refusal expected | `every_staged_subject_matches_its_pin` and `each_row_standing_is_pinned_by_key` |
| `16.4 / empty-signature` | witness item 6 emptied | sponsorless announcement | ScriptPathRejection | staged; refusal expected | `every_staged_subject_matches_its_pin` and `each_row_standing_is_pinned_by_key` |
| `16.4 / malformed-signature` | witness item 6 truncated by one byte | sponsorless announcement | ScriptPathRejection | staged; refusal expected | `every_staged_subject_matches_its_pin` and `each_row_standing_is_pinned_by_key` |
| `16.4 / wrong-operator` | witness item 6 the first signer over the honest message; shared subject with `16.4 / valid-signature-under-another-key` | sponsorless announcement | ScriptPathRejection | staged; refusal expected | `every_staged_subject_matches_its_pin` and `each_row_standing_is_pinned_by_key` |
| `16.4 / stale-operator` | no superseded key | sponsorless announcement | ScriptPathRejection | unstageable: no superseded key exists; any other key is the wrong-operator subject | `each_row_standing_is_pinned_by_key` |
| `16.4 / valid-signature-under-another-key` | witness item 6 the first signer over the honest message; shared subject with `16.4 / wrong-operator` | sponsorless announcement | ScriptPathRejection | staged; refusal expected | `every_staged_subject_matches_its_pin` and `each_row_standing_is_pinned_by_key` |
| `16.4 / valid-signature-over-another-candidate` | witness item 6 an operator signature over a lock-time-one candidate | sponsorless announcement | ScriptPathRejection | staged; refusal expected | `every_staged_subject_matches_its_pin` and `each_row_standing_is_pinned_by_key` |
| `16.4 / successor-metadata-changed-after-signing` | witness items 0 to 2 and output 0 from the latest-lead announcement; signature kept | sponsorless announcement | ScriptPathRejection | staged; refusal expected | `every_staged_subject_matches_its_pin` and `each_row_standing_is_pinned_by_key` |
| `16.4 / successor-program-changed-after-signing` | output 0 at the successor's next admissible nonce; signature kept | sponsorless announcement | ScriptPathRejection | staged; refusal expected | `every_staged_subject_matches_its_pin` and `each_row_standing_is_pinned_by_key` |
| `16.4 / sponsor-input-added-after-signing` | a sponsor input the sponsorless shape cannot add | sponsorless announcement | ScriptPathRejection | waiting on C1 | `each_row_standing_is_pinned_by_key` |
| `16.4 / fee-output-changed-after-signing` | a fee output that exists only in a sponsored form | sponsorless announcement | ScriptPathRejection | waiting on C1 | `each_row_standing_is_pinned_by_key` |
| `16.5 / wrong-state-asset` | a coin of another asset spent, output 0 asset follows | sponsorless announcement | ScriptPathRejection | staged; refusal expected | `every_staged_subject_matches_its_pin` and `each_row_standing_is_pinned_by_key` |
| `16.5 / wrong-singleton-amount` | a coin one unit over the singleton, output 0 value follows | sponsorless announcement | ScriptPathRejection | staged; refusal expected | `every_staged_subject_matches_its_pin` and `each_row_standing_is_pinned_by_key` |
| `16.5 / wrong-predecessor-program` | a coin at the predecessor program at a later nonce, item 8 re-committed | sponsorless announcement | ScriptPathRejection | staged; refusal expected | `every_staged_subject_matches_its_pin` and `each_row_standing_is_pinned_by_key` |
| `16.5 / predecessor-metadata-reconstructs-another-program` | witness item 4 the predecessor record at a later admissible nonce | sponsorless announcement | ScriptPathRejection | staged; refusal expected | `every_staged_subject_matches_its_pin` and `each_row_standing_is_pinned_by_key` |
| `16.5 / wrong-leaf-version` | the leaf-version byte of witness item 8 changed; a generic commitment verdict | sponsorless announcement | ScriptPathRejection | staged; generic verdict expected | `every_staged_subject_matches_its_pin` and `each_row_standing_is_pinned_by_key` |
| `16.6 / successor-from-wrong-semantic-metadata` | output zero commits the first-admissible-nonce successor with q increased by one; items 0 and 1 follow, item 6 re-signed, honest request and predecessor retained; (`rec:backlog:wave0-native-route-construction`) | sponsorless announcement | ScriptPathRejection | staged; refusal expected | `foreign_quantity_changes_only_the_successor_semantic_commitment` and `each_row_standing_is_pinned_by_key` |
| `16.6 / successor-from-predecessor-metadata-unchanged` | output 0 to the predecessor semantic unchanged; shared subject with `16.2 / successor-remains-unannounced` | sponsorless announcement | ScriptPathRejection | staged; refusal expected | `every_staged_subject_matches_its_pin` and `each_row_standing_is_pinned_by_key` |
| `16.6 / wrong-announcement-cycle` | output 0 announces the next cycle; item 2 kept | sponsorless announcement | ScriptPathRejection | staged; refusal expected | `every_staged_subject_matches_its_pin` and `each_row_standing_is_pinned_by_key` |
| `16.6 / wrong-representation-nonce` | witness item 1 the honest nonce plus one | sponsorless announcement | ScriptPathRejection | staged; refusal expected | `every_staged_subject_matches_its_pin` and `each_row_standing_is_pinned_by_key` |
| `16.6 / later-admissible-nonce-instead-of-first` | output 0 at the next admissible nonce, re-signed | sponsorless announcement | ScriptPathRejection | staged; acceptance expected: nonce leastness is a stated leaf residual | `every_staged_subject_matches_its_pin` and `each_row_standing_is_pinned_by_key` |
| `16.6 / wrong-leaf-version` | output 0 committed under another leaf version; item 0 matched | sponsorless announcement | ScriptPathRejection | staged; refusal expected | `every_staged_subject_matches_its_pin` and `each_row_standing_is_pinned_by_key` |
| `16.6 / wrong-control-recipe` | the successor control recipe | sponsorless announcement | ScriptPathRejection | unstageable: no announcement position offers the recipe | `each_row_standing_is_pinned_by_key` |
| `16.6 / arbitrary-caller-supplied-output-program` | output 0 a version-one program of the first signer's key | sponsorless announcement | ScriptPathRejection | staged; refusal expected | `every_staged_subject_matches_its_pin` and `each_row_standing_is_pinned_by_key` |
| `16.6 / no-state-successor` | output 0 a null-data burn of the singleton | sponsorless announcement | ScriptPathRejection | staged; refusal expected | `every_staged_subject_matches_its_pin` and `each_row_standing_is_pinned_by_key` |
| `16.6 / two-state-successors` | honest input and output zero plus a second same-asset coin and successor at amount one, both inputs signed over the complete census; (`rec:backlog:wave0-native-route-construction`) | sponsorless announcement | ScriptPathRejection | staged; acceptance expected: the covenant reads no output count | `two_state_successors_balance_and_authorize_the_complete_census` and `each_row_standing_is_pinned_by_key` |
| `16.6 / extra-state-like-output` | a stand-in key-path coin of another asset funds an extra state-like output of that asset | sponsorless announcement | ScriptPathRejection | staged; acceptance expected: the covenant reads no output count | `every_staged_subject_matches_its_pin` and `each_row_standing_is_pinned_by_key` |
| `16.11 / wrong-transaction-version` | the transaction version moved to the other admitted value, re-signed | sponsorless announcement | ConsensusRejectionBeforeScript | staged; acceptance expected: consensus admits either version and the covenant reads none | `every_staged_subject_matches_its_pin` and `each_row_standing_is_pinned_by_key` |
| `16.11 / wrong-sequence` | the input sequence a non-final BIP-68 value | sponsorless announcement | ConsensusRejectionBeforeScript | staged; refusal expected | `every_staged_subject_matches_its_pin` and `each_row_standing_is_pinned_by_key` |
| `16.11 / witness-item-reorder` | witness items 1 and 2 exchanged | sponsorless announcement | ScriptPathRejection | staged; refusal expected | `every_staged_subject_matches_its_pin` and `each_row_standing_is_pinned_by_key` |
| `16.11 / predecessor-and-successor-metadata-witnesses-exchanged` | the two metadata witnesses exchanged | sponsorless announcement | ScriptPathRejection | unstageable: successor metadata is derived, never witnessed | `each_row_standing_is_pinned_by_key` |
| `16.11 / control-block-from-another-program` | witness item 8 a control block from another program; a generic commitment verdict | sponsorless announcement | ScriptPathRejection | staged; generic verdict expected | `every_staged_subject_matches_its_pin` and `each_row_standing_is_pinned_by_key` |
| `16.11 / target-bytes-changed-after-abi-validation` | the lock time set to one, inert under final sequences | sponsorless announcement | ScriptPathRejection | staged; refusal expected | `every_staged_subject_matches_its_pin` and `each_row_standing_is_pinned_by_key` |
| `16.5 / wrong-internal-key` | a second coin at the foreign-key program; items 5 and 8 written, re-signed | sponsorless announcement | ScriptPathRejection | inherited admission | `inherited_admissions_keep_their_mutation_control_and_boundary` |
| `16.5 / wrong-control-block` | predecessor parity item 5 flipped | sponsorless announcement | ScriptPathRejection | inherited admission | `inherited_admissions_keep_their_mutation_control_and_boundary` |
| `16.5 / metadata-leaf-selected-for-execution` | witness replaced with the metadata leaf and its recipe | sponsorless announcement | ScriptPathRejection | inherited admission | `inherited_admissions_keep_their_mutation_control_and_boundary` |
| `16.5 / key-path-spend-attempt` | witness replaced with one 64-byte item | sponsorless announcement | KeyPathRejection | inherited admission | `inherited_admissions_keep_their_mutation_control_and_boundary` |
| `16.6 / successor-under-another-static-subtree` | output 0 under the second subtree; items 0 and 1 written, re-signed | sponsorless announcement | ScriptPathRejection | inherited admission | `inherited_admissions_keep_their_mutation_control_and_boundary` |
| `16.6 / wrong-internal-key` | output 0 at the foreign output key; item 0 written, re-signed | sponsorless announcement | ScriptPathRejection | inherited admission | `inherited_admissions_keep_their_mutation_control_and_boundary` |
| `16.6 / wrong-parity` | successor parity item 0 flipped | sponsorless announcement | ScriptPathRejection | inherited admission | `inherited_admissions_keep_their_mutation_control_and_boundary` |

**Result (Native Safety staging)** · `result:backlog:safety-native-staging`

`native_staging_ledger_partitions_the_thirty_five_native_rows` pins 35 = 25 staged with a refusal expected + 3 staged with acceptance expected + 2 waiting on C1 + 5 unstageable by typed reason, with the four standings' key sets whose union equals the register `STILL_REQUIRED` and the classifier's recomputed `outstanding_rows`. The region-refit term the earlier census carried for comparability is zero on this base by construction and is not a standing here. The seven inherited admissions stand beside that figure, preserved by `inherited_admissions_keep_their_mutation_control_and_boundary` at six script-path boundaries and one key-path boundary.

`every_staged_subject_matches_its_pin` recomputes each distinct subject's spend-time digest from the admitted variable control and holds it to its `PIN_*` constant, from `PIN_UNANNOUNCED` through `PIN_EXTRA_OUTPUT`; `each_subject_moves_only_its_coordinate` holds every subject to one changed coordinate, its input and output counts and its named collateral; `staged_subjects_are_pairwise_distinct` holds all twenty-six to distinct bytes; `shared_pair_keys_map_to_one_subject` holds each shared pair's two keys to one subject; and `re_signed_subjects_verify_under_the_published_operator_key` verifies each of the thirteen operator-re-signed subjects' signatures against its own spent-output census. Twenty-six `PIN_*` constants cover the twenty-eight staged keys.

`native_staging_leaves_the_historical_census_unchanged` reconstructs the historical Safety census at 206 / 78 / 128, the native-run-required figure at 35, and the first-party ledger at 43 = 5 + 2 + 36. `each_row_standing_is_pinned_by_key` pins each row's standing and typed reason, and `safety_native_record_matches_the_typed_register` binds this table's forty-two keys to the register. No classifier, register or plan code is edited; the native staging ledger is supplementary, as the first-party ledger was.

The original staging partition is historical; settlement established three absent-coordinate finalities and left two construction waits at (`rec:backlog:wave0-native-route-settlement`). Subsequent admission at (`rec:backlog:wave0-native-route-capture-driver`) removes both construction waits. `historical_unstageable_subjects_keep_their_partition_after_scope_settlement` checks that finality belongs to the Safety boundary while these subjects remain historically unstageable. At (`rec:backlog:wave0-native-route-construction`) the table carries current standings: `native_staging_ledger_partitions_the_thirty_five_native_rows`, `staged_subjects_are_pairwise_distinct` and `re_signed_subjects_verify_under_the_published_operator_key` now pin [26, 4, 2, 3], twenty-eight constants over thirty keys and fifteen operator plus two key-path signature verifications, while `historical_unstageable_subjects_keep_their_partition_after_scope_settlement` holds [25, 3, 2, 5] by key.

**Caveat (The unstageable stale-operator row and the shared pairs)**

`16.4 / stale-operator` has no superseded key to spend: no superseded key exists, and any other key is the wrong-operator subject while rewriting the key in the leaf script draws the program-generic commitment verdict, so it is unstageable rather than a shared subject of `16.4 / wrong-operator`. Two staged pairs share one subject because their bytes are identical: `16.2 / successor-remains-unannounced` with `16.6 / successor-from-predecessor-metadata-unchanged`, and `16.4 / wrong-operator` with `16.4 / valid-signature-under-another-key`, each held by `shared_pair_keys_map_to_one_subject`. The two item-4 subjects, `16.3 / metadata-from-another-state-object` and `16.5 / predecessor-metadata-reconstructs-another-program`, both write the predecessor metadata record and exercise the one metadata-reconstruction clause.

**Caveat (Wrong register grounds)**

These rows carry a gap or ground in `STILL_REQUIRED` this ledger contradicts, left for the register's own revision rather than retyped here: `16.4 / fee-output-changed-after-signing` is a sponsored-form wait rather than a staged write; `16.4 / stale-operator` has no superseded key, and any other key is the wrong-operator subject; `16.5 / wrong-predecessor-program`, `16.5 / wrong-leaf-version`, `16.6 / arbitrary-caller-supplied-output-program`, `16.6 / wrong-leaf-version` and `16.6 / no-state-successor` stage from public constructors despite a derivation or program-generic gap; `16.6 / wrong-control-recipe`, `16.6 / two-state-successors` and `16.11 / predecessor-and-successor-metadata-witnesses-exchanged` are unstageable for a position or coin the candidate lacks rather than for want of a published relation; and `16.6 / later-admissible-nonce-instead-of-first`, `16.6 / extra-state-like-output` and `16.11 / wrong-transaction-version` stage with acceptance expected where the register read them as waiting or walled.

**Caveat (Declared-boundary classification questions)**

The three acceptance-expected rows are declared-boundary classification questions for the observation step: the covenant decides none of their coordinates, so a run is expected to accept them. Output count (`16.6 / extra-state-like-output`) is unread because the STATE program introspects only position 0; nonce leastness (`16.6 / later-admissible-nonce-instead-of-first`) is a stated leaf residual; and for the transaction version (`16.11 / wrong-transaction-version`) consensus has no rule between the two admitted values and the covenant reads no version, so the coordinate is undecided on this candidate.

**Caveat (Stand-in outpoints)**

The second-coin subjects `16.5 / wrong-state-asset`, `16.5 / wrong-singleton-amount` and `16.5 / wrong-predecessor-program` spend a test-fixed stand-in outpoint in place of the state coin. `16.6 / extra-state-like-output` funds its extra output from a stand-in key-path second input whose census claims the second deployment's asset, the control's funded amount and a version-one program of the first published test signer; the extra output carries the same asset at a state-like program. Its input witness is that signer's key-path signature over `transaction::candidate_key_path_message`, a candidate-scoped, unreviewed construction its own documentation names; a node computing a different key-path message would refuse input 1 for a reason other than the row's fault, which the observation step classifies. The observation step restages all four over funded outpoints and compares by replay; this step observes nothing and writes no fixture or evidence directory. The base is `0.7.21-dev`.

**Record (Wave 0 sponsored ruling)** · `rec:backlog:wave0-sponsored-ruling`

Ruling 130 selects the typed sponsored STATE arm, with and without change, under region-scoped `SponsorIsolation`; construction, funded observations and evidence-backed retyping remain ordered as C1, C2 and C3 (`rule:phase7:wave0-rulings`); (`pre:guide15-exec:sponsored-order`). The fourteen Safety rows and two native-staging waits retain their present bounds until their required evidence is admitted. Each input keeps its own signer; one capability may serve several inputs within one sponsor flow.

### Sponsored ruling rows · `sec:backlog:sponsored-ruling-rows`

**Table (Sponsored ruling rows)** · `tab:backlog:sponsored-ruling-rows`

| Row | Present bound | Form making it observable | Declared boundary C1 or C2 reaches | Step that retypes it |
|---|---|---|---|---|
| `16.1 / sponsored-without-change` | `SponsoredControlRefusedAtConstruction` | Funded sponsored control without change | C1 construction; C2 positive-floor acceptance and readback | C3 |
| `16.1 / sponsored-with-change-where-supported` | `SponsoredControlRefusedAtConstruction` | Funded sponsored control with change | C1 construction; C2 positive-floor acceptance and readback | C3 |
| `16.10 / state-sponsor-overlap` | `SponsoredControlRefusedAtConstruction` | Sponsored control with STATE claimed as sponsor | C1 construction disjointness and region-scoped `SponsorIsolation` | C3 |
| `16.10 / two-sponsor-envelopes` | `SponsorArmAwaitsRegionScoping` | Sponsored control with two region-owned fee-sponsor flows | C1 region-scoped `SponsorEnvelopeMultiplicity`; no leaf-count claim | C3 |
| `16.10 / foreign-sponsor-asset` | `SponsoredControlRefusedAtConstruction` | Sponsored control with one foreign-asset sponsor member | C1 sponsor recognition and model recognition | C3 |
| `16.10 / missing-sponsor-authorization` | `SponsoredControlRefusedAtConstruction` | Sponsored control missing one input member's authorization | C1 completion and region-scoped authorization; C2 sponsor-input execution where submitted | C3 |
| `16.10 / sponsor-change-at-state-output-0` | `SponsoredControlRefusedAtConstruction` | Sponsored change replaces successor at zero | C1 layout; C2 STATE script boundary with unrelated signatures valid | C3 |
| `16.10 / state-successor-in-sponsor-range` | `SponsoredControlRefusedAtConstruction` | STATE successor claimed by sponsor flow | C1 region-scoped `SponsorIsolation`; positional target variant distinguished separately | C3 |
| `16.10 / fee-change-substitution` | `SponsoredControlRefusedAtConstruction` | Sponsored fee/change allocation changed | C1 offer/freeze comparison; C2 retained-signature refusal, with any reauthorized allocation assessed separately | C3 |
| `16.10 / sponsor-member-unclassified` | `SponsorArmAwaitsRegionScoping` | Sponsored control omitting one member's flow classification | C1 region-scoped exact-membership boundary | C3 |
| `16.10 / empty-sponsor-offer-for-sponsored-request` | `SponsoredControlRefusedAtConstruction` | Sponsored request with empty offer | C1 construction refusal against a nonempty control | C3 |
| `16.10 / balanced-state-corruption-compensated-by-sponsor-change` | `SponsoredControlRefusedAtConstruction` | Funded sponsored control with the proposed compensating change | C1 exact construction/model comparison; C2 conservation or STATE script boundary as the exact subject permits | C3 |
| `16.10 / zero-valued-sponsor-member-under-exact-role-structure` | `SponsorArmAwaitsRegionScoping` | Exact sponsor membership retaining a zero-valued member | C1 value-blind model acceptance control; C2 exact target value/policy boundary | C3 |
| `16.10 / confidential-sponsor-value-where-the-selected-target-policy-claims-support` | `SponsoredControlRefusedAtConstruction` | Balanced confidential sponsor control with required proofs | C1 proof finalization/signing; C2 selected-policy acceptance or precise refusal | C3 |
| `16.4 / sponsor-input-added-after-signing` | `ScriptPathRejection`; native staging waits on C1 | Funded signed sponsored control plus an input | C1 protected-region refusal; C2 declared STATE script boundary | C3 |
| `16.4 / fee-output-changed-after-signing` | `ScriptPathRejection`; native staging waits on C1 | Funded signed sponsored control with changed fee output | C1 protected-region refusal; C2 declared STATE script boundary | C3 |

**Result (Sponsored ruling)** · `result:backlog:sponsored-ruling`

Ruling 130 revisits ruling 58: the reduced STATE leaf retains its local checks and singleton premises, while the completed refit supplies the announcement operation region for `SponsorIsolation`. C1 builds the selected sponsored arm and its declared negatives, C2 supplies funded positive-floor observations with exact fees, policy, subjects and readback, and C3 preserves each former reason beside the admitted disposition keyed by section and row name. The nine `AcceptanceAwaitsRelayAdmissibility` controls remain C2 obligations alongside the sponsored forms; this decision supplies no construction or target observation and leaves conjunct 14's binding to exact operation regions open (`rule:phase7:wave0-rulings`); (`gate:guide15-exec:bite-0-5-conjunct-7`); (`gate:guide15-exec:bite-0-5-conjunct-14`); (`gate:guide15-exec:bite-0-5-non-answer-gate`).

**Caveat (Model scope and distinct boundaries)**

`SponsorEnvelopeMultiplicity` remains model-scope resource policy enforced by no leaf; the other region-scoped relations and the four plain-L-BTC cardinality and recognition identities retain their stated carrier limits (`rec:backlog:wave0-region-refit`). Value-blind membership does not decide conservation, fee policy or target authorization: exact zero-valued membership is a model acceptance control with target value and policy behavior classified separately. C1 and C2 must declare the compensated-corruption subject's exact reachable boundary and preserve any earlier conservation refusal. At the sponsored ruling landing, an explicit-only first construction left the balanced confidential form and proofs owed. Their construction and selected-policy agreement are now admitted at (`rec:backlog:wave0-confidential-route-admission`). No refusal at another boundary answers an observable obligation, and removal of the selected form requires a replacement route (`rule:guide15-exec:observable-retention`); (`rec:backlog:wave0-safety-native-staging`).

### Sponsored construction · `sec:backlog:sponsored-construction`

**Record (Wave 0 sponsored construction)** · `rec:backlog:wave0-sponsored-construction`

The typed sponsored STATE request constructs with and without explicit change under ruling 130 at (`rule:phase7:wave0-rulings`). The coordinator remains at input zero, canonical sponsor inputs begin at one, and the authenticated successor remains at output zero. Positive change occupies output one before the final fee; absent or zero change leaves the fee at one. The complete spent-output census reaches operator signing, and each sponsor input receives its own request over the frozen transaction through the sponsor capability. The operator construction-right path remains intact.

**Table (Sponsored construction rows)** · `tab:backlog:sponsored-construction-rows`

Tests named below reside in `packages/transaction/src/tests/state_construct_tests.rs`, `state_finalize_tests.rs` and `state_signing_tests.rs`. Model subjects use the constructed positions and retained census with erased sponsor values. Runtime relation failures and observation-normalization refusals retain their distinct boundaries.

| Row | Present bound | C1 construction or refusal reaching the boundary | Test pin | What C2 still owes |
|---|---|---|---|---|
| `16.1 / sponsored-without-change` | `SponsoredControlRefusedAtConstruction` | Explicit control completes; successor zero, fee one, complete census and per-input witnesses | `sponsored_without_change_reaches_submit_ready` | Funded positive-floor acceptance and exact readback |
| `16.1 / sponsored-with-change-where-supported` | `SponsoredControlRefusedAtConstruction` | Explicit control completes; successor zero, change one, fee two | `sponsored_with_change_reaches_submit_ready` | Funded positive-floor acceptance and exact readback |
| `16.10 / state-sponsor-overlap` | `SponsoredControlRefusedAtConstruction` | Construction `OverlappingOutpoint`; model `SponsorIsolation` failure | `state_input_cannot_join_sponsor_offer`; `state_input_and_successor_cannot_join_sponsor_flow` | No target claim from these boundaries |
| `16.10 / two-sponsor-envelopes` | `SponsorArmAwaitsRegionScoping` | Second disjoint owned flow normalizes and fails `SponsorEnvelopeMultiplicity` | `second_sponsor_envelope_fails_region_multiplicity` | No leaf-count claim |
| `16.10 / foreign-sponsor-asset` | `SponsoredControlRefusedAtConstruction` | Construction `SponsorInputCarriesForeignAsset`; model `ObjectRecognition` failure | `sponsor_views_are_required_and_foreign_assets_refuse`; `foreign_sponsor_asset_fails_model_recognition` | No target claim from recognition |
| `16.10 / missing-sponsor-authorization` | `SponsoredControlRefusedAtConstruction` | Missing member signer fails isolation; missing response fails completion | `each_sponsor_member_requires_its_own_model_authorization`; `each_sponsor_member_requires_its_own_response_and_exact_echo` | Sponsor-input execution where submitted |
| `16.10 / sponsor-change-at-state-output-0` | `SponsoredControlRefusedAtConstruction` | Exchanged output positions fail finalized output comparison | `sponsor_change_cannot_replace_successor_zero` | STATE script boundary with unrelated signatures valid |
| `16.10 / state-successor-in-sponsor-range` | `SponsoredControlRefusedAtConstruction` | Successor flow membership fails `SponsorIsolation` | `state_input_and_successor_cannot_join_sponsor_flow` | Distinguish any positional target variant |
| `16.10 / fee-change-substitution` | `SponsoredControlRefusedAtConstruction` | Reallocation differs from frozen outputs; changed fee is named after signing | `fee_change_reallocation_differs_from_frozen_offer`; `sponsored_fee_mutation_after_signing_is_named` | Retained-signature refusal; assess reauthorized allocation separately |
| `16.10 / sponsor-member-unclassified` | `SponsorArmAwaitsRegionScoping` | Missing flow membership fails isolation; duplicate and cross-region references refuse normalization | `missing_sponsor_membership_fails_isolation`; `surplus_sponsor_membership_refuses_at_observation_boundary` | No target claim from membership |
| `16.10 / empty-sponsor-offer-for-sponsored-request` | `SponsoredControlRefusedAtConstruction` | `EmptySponsorOffer` against a nonempty constructive control | `malformed_and_empty_offers_refuse_against_a_nonempty_control` | No target claim from offer admission |
| `16.10 / balanced-state-corruption-compensated-by-sponsor-change` | `SponsoredControlRefusedAtConstruction` | Changed STATE amount fails recognition and frozen-output comparison; distinct asset balances do not compensate | `sponsor_change_does_not_compensate_state_asset_corruption`; `state_amount_corruption_fails_model_recognition_with_sponsorship` | Exact conservation or STATE script boundary; preserve an earlier conservation refusal |
| `16.10 / zero-valued-sponsor-member-under-exact-role-structure` | `SponsorArmAwaitsRegionScoping` | Exact erased membership still passes isolation with a zero-valued source | `zero_sponsor_value_does_not_erase_membership` | Exact target value and policy boundary |
| `16.10 / confidential-sponsor-value-where-the-selected-target-policy-claims-support` | `SponsoredControlRefusedAtConstruction` | Explicit route returns `MaturitySponsorValueRequiresProofFinalization`; no closure | `confidential_sponsor_value_requires_balanced_proof_route` | Balanced confidential control, proofs and selected-policy verdict |
| `16.4 / sponsor-input-added-after-signing` | `ScriptPathRejection`; native staging waits on C1 | Distinct appended input fails `MaturitySponsorInputAddedAfterSigning` | `sponsored_input_extension_after_signing_is_named` | Declared STATE script boundary |
| `16.4 / fee-output-changed-after-signing` | `ScriptPathRejection`; native staging waits on C1 | Existing fee mutation fails `MaturityFeeRoleChangedAfterSigning` | `sponsored_fee_mutation_after_signing_is_named` | Declared STATE script boundary |

**Result (Sponsored construction)** · `result:backlog:sponsored-construction`

Both explicit sponsored forms construct, retain the full signing census and collect distinct sponsor responses by position. Construction refuses malformed funding context, a reserve alias of the STATE asset and an unindexable sponsor suffix; finalization and signing also compare the settled output witnesses. The tests `declared_reserve_cannot_alias_the_state_asset`, `sponsor_suffix_position_overflow_refuses_before_signing` and `sponsored_output_witness_mutation_refuses_at_both_boundaries` pin these boundaries against accepted controls. Model tests exercise exact membership, recognition, member authorization and region-owned multiplicity. The request cross-product preserves sponsorless-change refusal, and the public finalized facts `SponsorRegionAbsent` and `FeeRoleAbsent` become `SponsorRegion` and `FeeRole` within the unchanged eleven-member census. These are construction and model findings for (`gate:guide15-exec:bite-0-5-conjunct-14`), with target evidence and retyping ordered after them.

**Caveat (Declared boundaries and remaining evidence)**

No target observation is supplied. Safety bounds and census pins remain unchanged; former construction-refusal reasons remain legible pending C3. The confidential-value row stays open because explicit-only construction supplies no balanced confidential form or proofs. Duplicate membership yields `DuplicateObservedReference` and cross-region membership yields `UnknownObservedObject` during observation normalization; neither is an isolation verdict. Sponsor response echoes establish binding rather than target signature validity. The reserve remains caller-declared, so rejecting a STATE-asset alias does not establish the target's reserve-asset identity. No refusal substitutes for another declared boundary (`rule:guide15-exec:observable-retention`).

### Sponsored observation · `sec:backlog:sponsored-observation`

**Record (Wave 0 sponsored observation)** · `rec:backlog:wave0-sponsored-observation`

The explicit sponsored observations are retained; the confidential obligation was later discharged by admitted agreement at (`rec:backlog:wave0-confidential-route-admission`). The executor selects either the historical zero overrides or the node's default fee policy; selecting defaults omits both fee overrides and supplies no measured floor by itself. Relay acceptance, ordinary block-template selection and direct-block acceptance are separate observations. The relay and mempool floors are reported by the node through its mempool readout. The block floor is declared by the pinned source, no override: `DEFAULT_BLOCK_MIN_TX_FEE` is 100 smallest units per kvB, represented as `0.00000100 BTC/kvB`, and the retained start arguments must show that neither floor was overridden. The archive admission distinguishes these provenance markers. The qualifying four-file sponsored archive is admitted (`step:wave0:bite-0-5-c2`); (`gate:guide15-exec:bite-0-5-conjunct-7`).

The control families are S0, a funded variable-metadata sponsorless announcement; S1, a funded sponsored announcement without change; and S2, a funded sponsored announcement with change. Each required acceptance must bind the submitted bytes, transaction identity, witness identity, mined bytes and block height. Sponsored controls require relay acceptance, membership by transaction identity in a non-producing assembly read with its total weight, and mining without an explicit transaction list. If assembly reading is unavailable, only the recorded isolated-production fallback may establish selection. Sponsorless controls require the relay outcome followed by separately recorded direct-block acceptance. The observations below are derived by exact replay; construction-only dispositions retain their separate boundaries.

**Table (Sponsored observation rows)** · `tab:backlog:sponsored-observation-rows`

| Row | Control family | Expected boundary | Observed outcome with route | Disposition for C3 | Pinning test |
|---|---|---|---|---|---|
| `16.1 / sponsored-without-change` | S1 against S0 | Relay acceptance and ordinary block selection | Relay accepted; assembly member; ordinary mined readback; fee 128 units | Acceptance and exact readback supplied | `sponsored_admission_binds_bytes_provenance_census_routes_and_readback` |
| `16.1 / sponsored-with-change-where-supported` | S2 against S1 | Relay acceptance and ordinary block selection | Relay accepted; assembly member; ordinary mined readback; fee 141, change 999859 units | Acceptance and exact readback supplied | `sponsored_admission_binds_bytes_provenance_census_routes_and_readback` |
| `16.1 / minimum-valid-lead` | S0 at minimum lead | Accepted transaction at inclusive lower endpoint | Relay refused: `min relay fee not met`; direct-block accepted with exact readback | Positive-floor direct-block observation supplied; no template-route claim | `sponsored_admission_binds_bytes_provenance_census_routes_and_readback` |
| `16.1 / maximum-valid-lead` | S0 at maximum lead | Accepted transaction at inclusive upper endpoint | Relay refused: `min relay fee not met`; direct-block accepted with exact readback | Positive-floor direct-block observation supplied; no template-route claim | `sponsored_admission_binds_bytes_provenance_census_routes_and_readback` |
| `16.1 / representative-interior-lead` | S0 at interior lead | Accepted transaction inside the lead window | Relay refused: `min relay fee not met`; direct-block accepted with exact readback | Positive-floor direct-block observation supplied; no template-route claim | `sponsored_admission_binds_bytes_provenance_census_routes_and_readback` |
| `16.1 / smallest-current-cycle` | S0 with current cycle zero | Accepted transaction with derived announcement cycle | Relay refused: `min relay fee not met`; direct-block accepted with exact readback | Positive-floor direct-block observation supplied; no template-route claim | `sponsored_admission_binds_bytes_provenance_census_routes_and_readback` |
| `16.1 / current-cycle-near-checked-upper-domain` | S0 at last complete window | Accepted transaction at the checked domain edge | Relay refused: `min relay fee not met`; direct-block accepted with exact readback | Positive-floor direct-block observation supplied; no template-route claim | `sponsored_admission_binds_bytes_provenance_census_routes_and_readback` |
| `16.1 / nontrivial-unaffected-fields` | S0 with distinct unaffected values | Acceptance with unchanged unaffected fields | Relay refused: `min relay fee not met`; direct-block accepted with exact readback | Positive-floor direct-block observation supplied; no template-route claim | `sponsored_admission_binds_bytes_provenance_census_routes_and_readback` |
| `16.1 / representation-nonce-zero` | S0 with constructor-selected zero nonce | Acceptance with least-first search evidence | Relay refused: `min relay fee not met`; direct-block accepted with exact readback | Positive-floor direct-block observation supplied; no template-route claim | `sponsored_admission_binds_bytes_provenance_census_routes_and_readback` |
| `16.1 / representation-nonce-nonzero` | S0 with constructor-selected nonzero nonce | Acceptance with every earlier retryable refusal retained | Relay refused: `min relay fee not met`; direct-block accepted with exact readback | Positive-floor direct-block observation supplied; no template-route claim | `sponsored_admission_binds_bytes_provenance_census_routes_and_readback` |
| `16.1 / accepted-target-transaction-with-semantic-constructor-root-and-recovery-projections-all-matching` | S0 with all four projection checks | Acceptance and agreement of all four projections | Relay refused: `min relay fee not met`; direct-block accepted with exact readback | Positive-floor direct-block observation supplied; no template-route claim | `sponsored_admission_binds_bytes_provenance_census_routes_and_readback` |
| `16.10 / state-sponsor-overlap` | S2 with STATE input claimed as sponsor | Construction disjointness and model isolation | No target carrier for membership | Inherit construction evidence only | `state_input_cannot_join_sponsor_offer`; `state_input_and_successor_cannot_join_sponsor_flow` |
| `16.10 / two-sponsor-envelopes` | S2 with second disjoint owned flow | Model multiplicity | No leaf-count carrier | Inherit model evidence only | `second_sponsor_envelope_fails_region_multiplicity` |
| `16.10 / foreign-sponsor-asset` | S2 with foreign sponsor asset | Construction and model recognition | No target claim from recognition | Inherit construction evidence only | `sponsor_views_are_required_and_foreign_assets_refuse`; `foreign_sponsor_asset_fails_model_recognition` |
| `16.10 / missing-sponsor-authorization` | S2 with one witness removed | Sponsor-input script execution | `ScriptPathRejection`: witness program hash mismatch at relay validation | Sponsor-input refusal supplied against accepted S2 | `sponsored_admission_binds_bytes_provenance_census_routes_and_readback` |
| `16.10 / sponsor-change-at-state-output-0` | S2 with successor/change positions exchanged | STATE script execution with fresh unrelated signatures | `ScriptPathRejection`: OP_EQUALVERIFY at relay validation; fresh unrelated signatures | Positional STATE refusal supplied against accepted S2 | `sponsored_admission_binds_bytes_provenance_census_routes_and_readback` |
| `16.10 / state-successor-in-sponsor-range` | S2 with successor claimed as sponsor member | Model isolation | No target membership carrier; exchanged positions separately reach `ScriptPathRejection` | Preserve membership boundary; no target membership credit | `sponsored_admission_binds_bytes_provenance_census_routes_and_readback` |
| `16.10 / fee-change-substitution` | S2 with fee/change allocation moved at fixed total | Retained-STATE-signature refusal; reauthorized allocation separately assessed | Retained signature: `ScriptPathRejection`; reauthorized fee 142 and change 999858 accepted through relay, assembly and ordinary mining | Refusal and separate allocation acceptance supplied | `sponsored_admission_binds_bytes_provenance_census_routes_and_readback` |
| `16.10 / sponsor-member-unclassified` | S2 with one flow membership omitted | Model exact membership | No target carrier for membership | Inherit model evidence only | `missing_sponsor_membership_fails_isolation` |
| `16.10 / empty-sponsor-offer-for-sponsored-request` | S1 with empty offer | Construction admission | No transaction to submit from the refused offer | Inherit construction evidence only | `malformed_and_empty_offers_refuse_against_a_nonempty_control` |
| `16.10 / balanced-state-corruption-compensated-by-sponsor-change` | S2 with STATE amount corrupted and reserve change adjusted | Conservation or STATE script, as reached | `ConsensusRejectionBeforeScript`: `bad-txns-in-ne-out`, also refused by direct block | Earlier conservation refusal retained; no STATE-script credit | `sponsored_admission_binds_bytes_provenance_census_routes_and_readback` |
| `16.10 / zero-valued-sponsor-member-under-exact-role-structure` | S2 with one member value zero | Funding validity before spend execution | Zero-valued funding refused: `ConsensusRejectionBeforeScript`, `bad-txns-in-ne-out` | Funding boundary supplied; zero-valued spend unreachable | `sponsored_admission_binds_bytes_provenance_census_routes_and_readback` |
| `16.10 / confidential-sponsor-value-where-the-selected-target-policy-claims-support` | S2 with confidential value and required proofs | Balanced confidential construction, then selected policy | Explicit control and confidential subject both accept through relay and ordinary template selection with exact readback | Final build-property reason bound to admitted agreement (`rec:backlog:wave0-confidential-route-admission`) | `confidential_admission_binds_bytes_provenance_census_route_and_readback`; `confidential_acceptance_binds_final_build_property_standing` |
| `16.4 / sponsor-input-added-after-signing` | S2 with distinct input appended after STATE signing | STATE script execution | `ScriptPathRejection`: invalid Schnorr signature; added funding and balancing allocation, both sponsor signatures refreshed | STATE signing refusal supplied against accepted S2 | `sponsored_admission_binds_bytes_provenance_census_routes_and_readback` |
| `16.4 / fee-output-changed-after-signing` | S2 with fee changed after STATE signing | STATE script execution | `ScriptPathRejection`: invalid Schnorr signature; fee/change total fixed and sponsor signature refreshed | STATE signing refusal supplied; shares allocation subject | `sponsored_admission_binds_bytes_provenance_census_routes_and_readback` |

**Result (Sponsored observation packet)** · `result:backlog:sponsored-observation`

The archive `packages/vectors/fixtures/maturity-sponsored-run-of-record/` binds the ceremony, timing sidecar, `MANIFEST.sha256` and `RUN-REPORT` through `MATURITY_SPONSORED_CEREMONY_SHA256`, `MATURITY_SPONSORED_MANIFEST_SHA256` and `MATURITY_SPONSORED_RUN_ADDRESS`. Its report states relay and mempool floors of `0.00000100 BTC/kvB`, reported by the node, and a block floor of `0.00000100 BTC/kvB`, declared by the pinned source, no override. Exact replay yields 69 operations, 35 funding steps, 17 signings and 17 submissions: 12 acceptances and five refusals. Zero-valued funding contributes one separate conservation refusal among the funding steps. The nine sponsorless subjects use direct-block acceptance after relay refusal; S1, S2 and the separately reauthorized allocation use observed assembly membership followed by ordinary mining. Measured signed weights of 2544 and 2809 units, each with an eight-unit signature margin, yield fees of 128 and 141 units by rounding upward at twice the largest floor. S2 funding is 1000000 units and its change is 999859; the independently reauthorized allocation pays 142 and returns 999858. Every accepted submission binds transaction identity, witness identity, exact submitted/mined bytes and block height. At sponsored admission, C3 inherited these observations, the funding refusal and the six non-target dispositions; the confidential row and the STATE-script obligation behind the conservation refusal remained open. The subsequent native observation at (`rec:backlog:wave0-native-observation`) supplied the script-boundary refusal, and admitted confidential agreement supplied the final disposition at (`rec:backlog:wave0-confidential-route-admission`). Safety bounds and census pins remain unchanged.

The disposable node retains `-listen=0`, `-dnsseed=0` and `-connect=0`, preventing incoming connections and automatic connection attempts; its private data directory and RPC endpoint isolate its mempool and block production. Each probe nevertheless retains its own transaction identity, current floors, five-minute load, mempool size and minimum fee, assembly membership and total weight. Confirmation uses a transaction lookup followed by its returned block hash, with inclusion outside the produced block recorded separately. Client calls have a 120-second budget; mutating methods have one attempt, while reads permit one 240-second retry. An inclusion found after an expired mutation remains undecided with its block hash and `inclusion_in_produced=false`, including in the legacy handlers. An expired wallet spend queries the fresh destination's wallet records and retains any found effects without repeating the spend or crediting a verdict. Missing assembly membership is undecided by displacement, never a refusal. Load and concurrent-state counts supply no verdict or admission precondition.

**Caveat (Boundaries and unresolved evidence)**

Fresh attempts restart the negative family and credit only subjects paired with that attempt's accepted control. Abandoned exchanges remain history and contribute no accepted or refused credit. Subsequent captures and reports carry `sponsored-waiting` with each unresolved row and retained reason; admission compares both copies with replay. An unresolved table entry is typed waiting with that reason, never refused. The immutable archive omits this optional field and replays to an empty waiting list; omission cannot admit a waiting row.

The six construction/model-only dispositions supply no target refusal credit. At this capture the confidential-value row remained open because the explicit builder supplied no balanced confidential form or proofs; the subsequent construction, capture and admission are recorded at (`rec:backlog:wave0-confidential-route-capture-driver`) and (`rec:backlog:wave0-confidential-route-admission`). A zero-valued funding refusal cannot establish execution of a zero-valued input. Input extension and allocation mutations declare their balance-preserving collateral; a literal single-field mutation refused by conservation leaves the intended STATE-script obligation open. Any observation at another boundary receives no credit for the declared boundary. Direct-block acceptance establishes neither relay acceptance nor ordinary block-template selection. The caller's branch context establishes no current-root freshness.

### Sponsored retyping · `sec:backlog:sponsored-retyping`

**Record (Wave 0 sponsored retyping)** · `rec:backlog:wave0-sponsored-retyping`

Fourteen sponsored Safety rows receive dispositions from the admitted observations or their own pre-target ABI-layout claims. The sponsored fold reads `maturity_sponsored_run_of_record()` beside the mutant fold; the archive's exact replay, pinned by `MATURITY_SPONSORED_MANIFEST_SHA256` and `MATURITY_SPONSORED_RUN_ADDRESS`, supplies credited submissions paired with their own accepted control. `submission_row` reads exact case, label and ordinal components into section-and-row keys; funding, signing, the nine positive-floor controls, membership-only claims and the two operator rows receive no key in this fold. Acceptance requires the relay-and-template route in the response itself and carries the schedule recorded by the planner's link, identity, mined height and provenance-bearing fee floors. Carrier execution of a refusal is read from its observed layer; the row's declared locator and carrier are compared before the shared binder runs. Refusals enter only through `from_native_refusal` and the shared `matches_boundary` rule. The former reasons remain documented in `MaturityRowBoundary` beside their replacements (`rec:backlog:wave0-sponsored-construction`); (`rec:backlog:wave0-sponsored-observation`).

**Table (Sponsored retyping ledger)** · `tab:backlog:sponsored-retyping-ledger`

Every row is pinned by `sponsored_retyping_ledger_pins_all_fourteen_rows` in `packages/vectors/src/maturity_evidence.rs`. The native entries additionally retain `sponsored_admission_binds_bytes_provenance_census_routes_and_readback`; the named construction and region tests reside in `packages/transaction/src/tests/state_construct_tests.rs`. `N` denotes the exact ordinal regenerated during replay. Final reasons retain `OutstandingUnderTypedNonAnswer`; finality does not manufacture an observed standing. The carrier column names the former and resulting carrier and the reason for any change.

| Row key | Former reason | Observation | Standing after | Finality | Evidence reference | Pinning test | Carrier change |
|---|---|---|---|---|---|---|---|
| `16.1 / sponsored-without-change` | `SponsoredControlRefusedAtConstruction` | Relay accepted, assembly member, ordinary mined readback; fee 128 | `NativeAcceptanceObserved`; `Layer(AcceptedTransaction)` | answered | Sponsored pins; `sponsored-without-change:control:N` | `sponsored_acceptances_carry_the_admitted_pins_schedule_and_readback` | `CoordinatorStructure` → `Target`: accepted transaction |
| `16.1 / sponsored-with-change-where-supported` | `SponsoredControlRefusedAtConstruction` | Relay accepted, assembly member, ordinary mined readback; fee 141, change 999859 | `NativeAcceptanceObserved`; `Layer(AcceptedTransaction)` | answered | Sponsored pins; `sponsored-with-change-where-supported:control:N` | `sponsored_acceptances_carry_the_admitted_pins_schedule_and_readback` | `CoordinatorStructure` → `Target`: accepted transaction |
| `16.10 / state-sponsor-overlap` | `SponsoredControlRefusedAtConstruction` | No target membership carrier; construction and isolation evidence retained | `OutstandingUnderTypedNonAnswer(SponsorConstructionAnsweredByLandedEvidence)` | final | Pre-target ABI-layout disjointness claim answered by C1 | `state_input_cannot_join_sponsor_offer`; `state_input_and_successor_cannot_join_sponsor_flow` | `CoordinatorStructure` → `CoordinatorStructure`: unchanged |
| `16.10 / two-sponsor-envelopes` | `SponsorArmAwaitsRegionScoping` | No leaf-count carrier; region multiplicity evidence retained | `OutstandingUnderTypedNonAnswer(SponsorRegionAnsweredByLandedEvidence)` | final | Pre-target ABI-layout multiplicity claim answered by C1 | `second_sponsor_envelope_fails_region_multiplicity` | `CoordinatorStructure` → `CoordinatorStructure`: unchanged |
| `16.10 / foreign-sponsor-asset` | `SponsoredControlRefusedAtConstruction` | No target recognition claim; construction and recognition evidence retained | `OutstandingUnderTypedNonAnswer(SponsorConstructionAnsweredByLandedEvidence)` | final | Pre-target ABI-layout offer/recognition claim answered by C1 | `sponsor_views_are_required_and_foreign_assets_refuse`; `foreign_sponsor_asset_fails_model_recognition` | `CoordinatorStructure` → `CoordinatorStructure`: unchanged |
| `16.10 / missing-sponsor-authorization` | `SponsoredControlRefusedAtConstruction` | Witness program hash mismatch at relay validation | `NativeRefusalObserved`; `Layer(ScriptPathRejection)` | answered | Sponsored pins; `sponsored-with-change-where-supported:missing-sponsor-authorization:N`; accepted S2 | `sponsored_refusal_requires_its_own_row_control_site_and_carrier` | `CoordinatorStructure` → `Target`: sponsor-input authorization executes |
| `16.10 / sponsor-change-at-state-output-0` | `SponsoredControlRefusedAtConstruction` | OP_EQUALVERIFY at relay validation with fresh unrelated signatures | `NativeRefusalObserved`; `Layer(ScriptPathRejection)` | answered | Sponsored pins; `sponsored-with-change-where-supported:sponsor-change-at-state-output-0:N`; accepted S2 | `sponsored_retyping_ledger_pins_all_fourteen_rows` | `CoordinatorStructure` → `AnnouncementLeaf`: positional STATE enforcement executes |
| `16.10 / state-successor-in-sponsor-range` | `SponsoredControlRefusedAtConstruction` | No target membership carrier; positional refusal supplies no membership credit | `OutstandingUnderTypedNonAnswer(SponsorRegionAnsweredByLandedEvidence)` | final | Pre-target ABI-layout membership claim answered by C1 | `state_input_and_successor_cannot_join_sponsor_flow` | `CoordinatorStructure` → `CoordinatorStructure`: unchanged |
| `16.10 / fee-change-substitution` | `SponsoredControlRefusedAtConstruction` | Retained STATE signature refuses; separately reauthorized fee 142/change 999858 accepted through relay, assembly and ordinary mining | `NativeRefusalObserved`; `Layer(ScriptPathRejection)` | answered | Sponsored pins; `sponsored-with-change-where-supported:fee-change-substitution:N`; accepted S2 | `sponsored_refusal_requires_its_own_row_control_site_and_carrier` | `CoordinatorStructure` → `AnnouncementLeaf`: retained STATE signature executes |
| `16.10 / sponsor-member-unclassified` | `SponsorArmAwaitsRegionScoping` | No target membership carrier; region isolation evidence retained | `OutstandingUnderTypedNonAnswer(SponsorRegionAnsweredByLandedEvidence)` | final | Pre-target ABI-layout membership claim answered by C1 | `missing_sponsor_membership_fails_isolation`; `surplus_sponsor_membership_refuses_at_observation_boundary` | `CoordinatorStructure` → `CoordinatorStructure`: unchanged |
| `16.10 / empty-sponsor-offer-for-sponsored-request` | `SponsoredControlRefusedAtConstruction` | No transaction from the refused offer | `OutstandingUnderTypedNonAnswer(SponsorConstructionAnsweredByLandedEvidence)` | final | Pre-target ABI-layout offer admission answered by C1 | `malformed_and_empty_offers_refuse_against_a_nonempty_control` | `CoordinatorStructure` → `CoordinatorStructure`: unchanged |
| `16.10 / balanced-state-corruption-compensated-by-sponsor-change` | `SponsoredControlRefusedAtConstruction` | `ConsensusRejectionBeforeScript`, `bad-txns-in-ne-out`; also refused by direct block | `NativeRefusalAtUnexpectedBoundary`; `Layer(ScriptPathRejection)` | contradicted | Sponsored pins; `sponsored-with-change-where-supported:balanced-state-corruption-compensated-by-sponsor-change:N`; accepted S2 | `sponsored_retyping_ledger_pins_all_fourteen_rows` | `CoordinatorStructure` → `AnnouncementLeaf`: intended STATE enforcement, explicitly `NotReached` |
| `16.10 / zero-valued-sponsor-member-under-exact-role-structure` | `SponsorArmAwaitsRegionScoping` | Funding refused with `bad-txns-in-ne-out`; no zero-valued input executed | `OutstandingUnderTypedNonAnswer(ZeroSponsorRoleStructureAnsweredByLandedEvidence)` | final | Pre-target ABI-layout role claim: erased zero value preserves membership; no submission key | `zero_sponsor_value_does_not_erase_membership`; `sponsored_fold_preserves_relay_operator_and_funding_obligations` | `CoordinatorStructure` → `CoordinatorStructure`: unchanged |
| `16.10 / confidential-sponsor-value-where-the-selected-target-policy-claims-support` | `SponsoredControlRefusedAtConstruction` | Explicit control and balanced confidential subject accept through relay and ordinary template selection with exact readback | `OutstandingUnderTypedNonAnswer(ConfidentialSponsorValueUndecidedByCandidate)` | final | Confidential archive and admitted equivalent-coin agreement (`rec:backlog:wave0-confidential-route-admission`) | `confidential_acceptance_binds_final_build_property_standing` | `CoordinatorStructure` → `CoordinatorStructure`: unchanged |

At sponsored retyping, the nine positive-floor rows retained `AcceptanceAwaitsRelayAdmissibility`, their standings and their count of nine. Their C2 direct-block acceptances after relay refusal established neither relay acceptance nor ordinary template selection; the observe-remaining step owned their acceptance-awaiting shapes (`step:wave0:bite-0-5-observe-remaining`). Eight subsequently received native acceptance at (`rec:backlog:wave0-native-observation`) and the combined row was discharged at (`rec:backlog:wave0-combined-projection`), with 8 + 1 = 9; the denominator remained 206, with no row split.

**Result (Sponsored retyping census)** · `result:backlog:sponsored-retyping`

At sponsored retyping, the fourteen dispositions were five answered, seven final, one waiting and one contradicted. Two sponsored positives and four submitted sponsor faults left the typed non-answer census: 35 → 29, layer-bound 171 → 177, reachable 43 → 49 and unreachable 128 → 128. The original reason counts 9/11/3/11/1 became 9/0/0/11/1; construction-final, region-final, zero-role-final and confidential-waiting contributed 3/3/1/1. The eleven former construction rows became two acceptances, three matching refusals, one unexpected refusal, three construction finals, one region final and one confidential wait. The three former region rows became two region finals and one zero-role final. Seven finals joined the existing twelve: finality 12 final / 23 waiting → 19 final / 10 waiting; the unexpected-boundary obligation was separately contradicted. Both supplemental finality counts were subsets of the 29 typed non-answers and never re-entered the row partition.

At sponsored retyping, evidence answered moved 78 → 83: acceptance 1 → 3 from the two sponsored positives, matching refusal 7 → 10 from missing authorization, output-zero substitution and retained-signature fee substitution. Balanced corruption changed unexpected refusal 0 → 1. Typed outstanding moved 35 → 29; total outstanding moved 128 → 123. Native-run-required stayed 35, first-party-required 43, report-layer-required 15, first-party-discharged 40, root-history-observed 16 and public-recovery-observed 14. Native-declared-boundary-observed, constructor-continuity-observed, report-layer-observed, infrastructure-blocked and experimental remained zero. Before: 40 + 43 + 35 + 1 + 7 + 0 + 15 + 16 + 14 + 35 = 206. After: 40 + 43 + 35 + 3 + 10 + 1 + 15 + 16 + 14 + 29 = 206. The row/layer product changed 1197 → 1239 pairs, 43 → 49 diagonal hits and 42 → 46 refusal-bindable rows: six additional layer rows included two positive controls. The report carried 7 → 11 native refusal lines, including the contradiction, while its witnessed native refusals moved 7 → 10; its waiting rows and register remained set-equal at 35. Rows admitting no refusal changed 37 → 33 because missing authorization, output-zero substitution, fee substitution and balanced corruption gained declared layers; the two sponsored positives remained their own controls. Removing one rendered refusal then left 10 of 11 lines instead of 6 of 7. Supplemental finality rendering added two keys, 16 → 18 census fields and 39 → 41 total report keys, without changing partition arithmetic.

**Caveat (Declared claims and retained observable obligations)**

The seven final rows declare pre-target ABI-layout mutations. Their construction or region answers are held by the named transaction tests, not by the vectors first-party validator registry; moving those claims into that registry is a later-step option, not a missing answer. No target observation is those rows' obligation. Zero-valued funding refusal establishes no zero-valued-input execution, and that execution question is not a row of this matrix. Membership and multiplicity are not positional or leaf-count target claims. At sponsored retyping the confidential row retained the balanced confidential route, proofs and selected-policy observation; admitted agreement subsequently supplied its final disposition at (`rec:backlog:wave0-confidential-route-admission`).

At sponsored retyping, balanced STATE corruption adjusted reserve change across distinct assets, so per-asset conservation refused before the STATE script. Its declared script boundary therefore remained contradicted at an observed boundary; the report rendered failed completeness and the contradiction standing, never ordinary waiting. The STATE-script obligation stayed with observe-remaining and needed a per-asset-balanced mutation (`step:wave0:bite-0-5-observe-remaining`); (`rule:guide15-exec:observable-retention`). The two §16.4 rows, `sponsor-input-added-after-signing` and `fee-output-changed-after-signing`, retained their script bounds, sponsorless control derivation and register entries. Their matrix control was the sponsorless announcement while their C2 observations were taken against the sponsored S2 control, so their credit needed a control ruling or a sponsorless re-observation, which the observe-remaining step owned (`step:wave0:bite-0-5-observe-remaining`); their C2 observations were held here, not folded. Subsequent native observation supplied the balanced-corruption script-boundary refusal and both operator-row observations at (`rec:backlog:wave0-native-observation`), while the sponsored archive retains its historical contradiction. Reauthorized allocation acceptance supplies no refusal credit for the retained-signature negative. No archive bytes or target observations were added by retyping.

### Native observation · `sec:backlog:native-observation`

**Record (Wave 0 native observation)** · `rec:backlog:wave0-native-observation`

The subject inventory retains the thirty-five keys and twenty-six archived-context digests of (`rec:backlog:wave0-safety-native-staging`). Exact replay rebuilds the four stand-in subjects over funded outpoints and adapts the maximum-lead subject to its live context. Each negative precedes its consuming control; credit requires acceptance of that family's control. The serial schedule runs under the node defaults without numerical overrides. The eight semantic positives use the deployed S2 form; at this admission the combined projection row kept its waiting reason, subsequently discharged at (`rec:backlog:wave0-combined-projection`). The confidential construction remains outside this observation.

**Table (Native observation row ledger)** · `tab:backlog:native-observation-rows`

| Row | Control family | Declared boundary | Observed outcome with route or reason none was obtained | Standing after | Pinning test |
| --- | --- | --- | --- | --- | --- |
| `16.2 / successor-remains-unannounced` | SponsorlessAnnouncement; window | `ScriptPathRejection` | script_path_rejection; refused; direct-block validation after relay refusal | `NativeRefusalObserved` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.2 / successor-becomes-complete` | SponsorlessAnnouncement; window | `ScriptPathRejection` | script_path_rejection; refused; direct-block validation after relay refusal | `NativeRefusalObserved` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.3 / metadata-from-another-state-object` | SponsorlessAnnouncement; metadata | `ScriptPathRejection` | script_path_rejection; refused; direct-block validation after relay refusal | `NativeRefusalObserved` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.4 / empty-signature` | SponsorlessAnnouncement; operator | `ScriptPathRejection` | script_path_rejection; refused; direct-block validation after relay refusal | `NativeRefusalObserved` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.4 / malformed-signature` | SponsorlessAnnouncement; operator | `ScriptPathRejection` | script_path_rejection; refused; direct-block validation after relay refusal | `NativeRefusalObserved` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.4 / wrong-operator` | SponsorlessAnnouncement; operator | `ScriptPathRejection` | script_path_rejection; refused; direct-block validation after relay refusal | `NativeRefusalObserved` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.4 / stale-operator` | SponsorlessAnnouncement; unavailable | `ScriptPathRejection` → `NativeCoordinateAbsentFromCandidate` | No operator-key migration relation; ruling 131 at (`rec:backlog:wave0-native-route-settlement`) | `NativeCoordinateAbsentFromCandidate; Final` | `absent_native_coordinates_have_exact_final_membership` |
| `16.4 / valid-signature-under-another-key` | SponsorlessAnnouncement; operator | `ScriptPathRejection` | script_path_rejection; refused; direct-block validation after relay refusal | `NativeRefusalObserved` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.4 / valid-signature-over-another-candidate` | SponsorlessAnnouncement; operator | `ScriptPathRejection` | script_path_rejection; refused; direct-block validation after relay refusal | `NativeRefusalObserved` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.4 / successor-metadata-changed-after-signing` | SponsorlessAnnouncement; operator | `ScriptPathRejection` | script_path_rejection; refused; direct-block validation after relay refusal | `NativeRefusalObserved` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.4 / successor-program-changed-after-signing` | SponsorlessAnnouncement; operator | `ScriptPathRejection` | script_path_rejection; refused; direct-block validation after relay refusal | `NativeRefusalObserved` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.4 / sponsor-input-added-after-signing` | SponsorlessAnnouncement; added-sponsor | `ScriptPathRejection` | script_path_rejection; refused | `NativeRefusalObserved` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.4 / fee-output-changed-after-signing` | SponsoredAnnouncement; interior S2 | `ScriptPathRejection` | script_path_rejection; refused | `NativeRefusalObserved` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.5 / wrong-state-asset` | SponsorlessAnnouncement; predecessor | `ScriptPathRejection` | script_path_rejection; refused; direct-block validation after relay refusal | `NativeRefusalObserved` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.5 / wrong-singleton-amount` | SponsorlessAnnouncement; predecessor | `ScriptPathRejection` | script_path_rejection; refused; direct-block validation after relay refusal | `NativeRefusalObserved` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.5 / wrong-predecessor-program` | SponsorlessAnnouncement; predecessor | `ScriptPathRejection` | script_path_rejection; refused; direct-block validation after relay refusal | `NativeRefusalObserved` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.5 / predecessor-metadata-reconstructs-another-program` | SponsorlessAnnouncement; predecessor | `ScriptPathRejection` | script_path_rejection; refused; direct-block validation after relay refusal | `NativeRefusalObserved` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.5 / wrong-leaf-version` | SponsorlessAnnouncement; abi | `ScriptPathRejection` | script_path_rejection; refused; direct-block validation after relay refusal | `NativeRefusalObserved` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.6 / successor-from-wrong-semantic-metadata` | SponsorlessAnnouncement; same-coin retained control | `ScriptPathRejection` | script_path_rejection; refused; retained archive | `NativeRefusalObserved` | `retained_semantic_refusal_binds_its_exact_row_and_same_coin_control` |
| `16.6 / successor-from-predecessor-metadata-unchanged` | SponsorlessAnnouncement; window | `ScriptPathRejection` | script_path_rejection; refused; direct-block validation after relay refusal | `NativeRefusalObserved` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.6 / wrong-announcement-cycle` | SponsorlessAnnouncement; successor | `ScriptPathRejection` | script_path_rejection; refused; direct-block validation after relay refusal | `NativeRefusalObserved` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.6 / wrong-representation-nonce` | SponsorlessAnnouncement; successor | `ScriptPathRejection` | script_path_rejection; refused; direct-block validation after relay refusal | `NativeRefusalObserved` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.6 / later-admissible-nonce-instead-of-first` | SponsorlessAnnouncement; later-nonce | `ScriptPathRejection → typed final reason` | accepted; direct-block | `NativeCoordinateUndecidedByCandidate; Final` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.6 / wrong-leaf-version` | SponsorlessAnnouncement; successor | `ScriptPathRejection` | script_path_rejection; refused; direct-block validation after relay refusal | `NativeRefusalObserved` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.6 / wrong-control-recipe` | SponsorlessAnnouncement; unavailable | `ScriptPathRejection` → `NativeCoordinateAbsentFromCandidate` | No operation spends an announced state; no successor control position exists; ruling 133 at (`rec:backlog:wave0-native-route-settlement`) | `NativeCoordinateAbsentFromCandidate; Final` | `candidate_leaf_and_transition_censuses_have_no_announced_state_spend` |
| `16.6 / arbitrary-caller-supplied-output-program` | SponsorlessAnnouncement; successor | `ScriptPathRejection` | script_path_rejection; refused; direct-block validation after relay refusal | `NativeRefusalObserved` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.6 / no-state-successor` | SponsorlessAnnouncement; successor | `ScriptPathRejection` | script_path_rejection; refused; direct-block validation after relay refusal | `NativeRefusalObserved` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.6 / two-state-successors` | SponsorlessAnnouncement; equivalent-coin retained control | `ScriptPathRejection → typed final reason` | accepted; direct-block; matched readback; retained archive | `NativeCoordinateUndecidedByCandidate; Final` | `retained_two_successor_acceptance_binds_final_undecided_standing` |
| `16.6 / extra-state-like-output` | SponsorlessAnnouncement; extra-output | `ScriptPathRejection → typed final reason` | accepted; direct-block | `NativeCoordinateUndecidedByCandidate; Final` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.11 / wrong-transaction-version` | SponsorlessAnnouncement; version | `ConsensusRejectionBeforeScript → typed final reason` | accepted; direct-block | `NativeCoordinateUndecidedByCandidate; Final` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.11 / wrong-sequence` | SponsorlessAnnouncement; successor | `ConsensusRejectionBeforeScript` | consensus_rejection_before_script; refused; direct-block validation after relay refusal | `NativeRefusalObserved` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.11 / witness-item-reorder` | SponsorlessAnnouncement; abi | `ScriptPathRejection` | script_path_rejection; refused; direct-block validation after relay refusal | `NativeRefusalObserved` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.11 / predecessor-and-successor-metadata-witnesses-exchanged` | SponsorlessAnnouncement; unavailable | `ScriptPathRejection` → `NativeCoordinateAbsentFromCandidate` | Successor metadata is derived, never a free witness; ruling 132 at (`rec:backlog:wave0-native-route-settlement`) | `NativeCoordinateAbsentFromCandidate; Final` | `absent_coordinates_bind_the_candidate_witness_and_generation_censuses` |
| `16.11 / control-block-from-another-program` | SponsorlessAnnouncement; abi | `ScriptPathRejection` | script_path_rejection; refused; direct-block validation after relay refusal | `NativeRefusalObserved` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.11 / target-bytes-changed-after-abi-validation` | SponsorlessAnnouncement; abi | `ScriptPathRejection` | script_path_rejection; refused; direct-block validation after relay refusal | `NativeRefusalObserved` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.1 / minimum-valid-lead` | TheRowIsTheControl; funded S2 | `AcceptedTransaction`; former `AcceptanceAwaitsRelayAdmissibility` | Relay admission, assembly membership, ordinary mining and exact readback | `NativeAcceptanceObserved`; `Target` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.1 / maximum-valid-lead` | TheRowIsTheControl; funded S2 | `AcceptedTransaction`; former `AcceptanceAwaitsRelayAdmissibility` | Relay admission, assembly membership, ordinary mining and exact readback | `NativeAcceptanceObserved`; `Target` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.1 / representative-interior-lead` | TheRowIsTheControl; funded S2 | `AcceptedTransaction`; former `AcceptanceAwaitsRelayAdmissibility` | Relay admission, assembly membership, ordinary mining and exact readback | `NativeAcceptanceObserved`; `Target` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.1 / smallest-current-cycle` | TheRowIsTheControl; funded S2 | `AcceptedTransaction`; former `AcceptanceAwaitsRelayAdmissibility` | Relay admission, assembly membership, ordinary mining and exact readback | `NativeAcceptanceObserved`; `Target` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.1 / current-cycle-near-checked-upper-domain` | TheRowIsTheControl; funded S2 | `AcceptedTransaction`; former `AcceptanceAwaitsRelayAdmissibility` | Relay admission, assembly membership, ordinary mining and exact readback | `NativeAcceptanceObserved`; `Target` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.1 / nontrivial-unaffected-fields` | TheRowIsTheControl; funded S2 | `AcceptedTransaction`; former `AcceptanceAwaitsRelayAdmissibility` | Relay admission, assembly membership, ordinary mining and exact readback | `NativeAcceptanceObserved`; `Target` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.1 / representation-nonce-zero` | TheRowIsTheControl; funded S2 | `AcceptedTransaction`; former `AcceptanceAwaitsRelayAdmissibility` | Relay admission, assembly membership, ordinary mining and exact readback | `NativeAcceptanceObserved`; `Target` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.1 / representation-nonce-nonzero` | TheRowIsTheControl; funded S2 | `AcceptedTransaction`; former `AcceptanceAwaitsRelayAdmissibility` | Relay admission, assembly membership, ordinary mining and exact readback | `NativeAcceptanceObserved`; `Target` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.1 / accepted-target-transaction-with-semantic-constructor-root-and-recovery-projections-all-matching` | TheRowIsTheControl; admitted S2 | `AcceptedTransaction`; former `AcceptanceAwaitsRelayAdmissibility` | One relay-and-template acceptance with semantic, constructor, root-history and public-recovery agreement over identical bytes | `NativeAcceptanceObserved`; `Target` | `combined_projection_binds_the_selected_acceptance_and_all_four_terms` |
| `16.10 / balanced-state-corruption-compensated-by-sponsor-change` | SponsoredAnnouncement; interior S2 | `ScriptPathRejection` | Refused at STATE output recognition; `OP_EQUALVERIFY`; both asset totals exact | `NativeRefusalObserved` | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |

**Result (Native observation evidence and census)** · `result:backlog:native-observation`

The four immutable files in `packages/vectors/fixtures/maturity-native-run-of-record/` are admitted once through `MATURITY_NATIVE_MANIFEST_SHA256`, `MATURITY_NATIVE_RUN_ADDRESS` and the ceremony pin. `native_observation_admission_binds_bytes_policy_census_and_replay` checks the archive, policy, census and exact replay; `native_replay_refuses_changed_requests_and_missing_control_readback` checks request and control integrity. The run reports relay and mempool floors of `0.00000100 BTC/kvB`; the block floor is `0.00000100 BTC/kvB`, declared by the pinned source with no override. Its schedule has 137 operations: 63 fundings, 27 sponsor signings, 47 submissions, 21 acceptances and 26 refusal observations. The report records preflight 35 ms, Cargo 316921 ms, census 282 ms, manifest 1083 ms and elapsed 318334 ms at report emission. No C2-based wall estimate exists because its timing sidecar carries no duration.

The denominator remains 206. Native-run-required moves 35 → 0: twenty-seven rows gain declared-boundary refusals, three accepted departures gain the final coordinate reason, and five unavailable subjects gain the waiting construction reason. Acceptance moves 3 → 11 through the eight relay-and-template positives. Refusal moves 10 → 38 through those twenty-seven native rows and balanced corruption, which also removes the sole unexpected-boundary standing, 1 → 0. Answered moves 83 → 119 and outstanding 123 → 87. Typed non-answers remain 29: final 19 → 22 and waiting 10 → 7, because eight positive waits leave and five staging waits enter. `STILL_REQUIRED` becomes empty; `NATIVE_STAGING_HISTORY` retains all thirty-five keys, including the unavailable subjects. The historical staging partition remains 25 / 3 / 2 / 5 and its twenty-six digests remain pinned. The report renders thirty-eight refusal lines and computes partial completeness. This supplies no native or Safety closure; the close-safety step inherits the ledger and all eighty-seven outstanding obligations.

Two subject observations each support two row claims: `window:successor-remains-unannounced` supports both §16.2's unannounced successor and §16.6's unchanged predecessor metadata; `operator:wrong-operator` supports both wrong operator and a valid signature under another key. Each row independently passes the control, locator, carrier and declared-boundary binder. Six accepted control observations are also shared across row claims: window, operator, predecessor, successor, ABI and interior S2. The metadata, added-sponsor and classification controls each support one row. The interior S2 acceptance is both its positive-row observation and the control for the fee and balanced-corruption subjects. The §16.4 fee row changes control because the sponsorless form has no fee output. Its fresh mutation raises the fee by one unit and lowers change by one, retains the STATE signature and refreshes sponsor authorization. The §16.10 fee row retains its C3 credit: the two rows share a mutation, not an observation. Balanced corruption exchanges output-zero and change asset/value pairs, preserves their programs and the fee, and refreshes both authorizations before the honest S2 control consumes the inputs.

The preceding observation census remains historical; the separately admitted retained observation is recorded at (`rec:backlog:wave0-native-route-capture-driver`) and current finality at (`result:backlog:close-safety-reconciliation`).

**Caveat (Boundaries and remaining constructions)**

The five subjects unavailable in this historical capture now comprise three absent coordinates and two constructions admitted through the separate retained archive at (`rec:backlog:wave0-native-route-capture-driver`); no universal impossibility is claimed. Three accepted departures establish a property of this candidate, not negative refusal credit or positive-row credit. Every observed refusal reaches its declared boundary in this archive; relay fee or witness-policy refusals alone receive no credit, and the later direct-block validation supplies the recorded consensus or script outcome. A refusal elsewhere or infrastructure failure would answer nothing at the row's declared boundary. The sponsorless controls and classification acceptances use direct block after fee refusal; that route establishes no positive-floor acceptance. Twenty-two sponsorless refusal observations, carrying twenty-four row claims, establish the script boundary through the direct-block verdict `mandatory-script-verify-flag-failed` alone, which does not name the refusing check. The close-safety step (`step:wave0:bite-0-5-close-safety`) inherits the finding that the script error from direct-block validation must be retained in the capture so the refusing check is named.

**Caveat (Combined projections and confidential value)**

The combined projection formerly retained `AcceptanceAwaitsRelayAdmissibility`: sponsorless bytes failed the fee floor and sponsored bytes failed the readers' one-input-one-output restriction. Positional readers now read past that restriction for role-validated layouts over one admitted S2 acceptance (`rec:backlog:wave0-combined-projection`), without broadening a validator. The confidential-value row is final under `ConfidentialSponsorValueUndecidedByCandidate`, bound to admitted agreement at (`rec:backlog:wave0-confidential-route-admission`). Retained evidence is named by the loop's `evidence/wave0/native/<run-address>/` tier, while admitted evidence is cited by its fixture directory and pins.

### Independent observation · `sec:backlog:independent-observation`

**Record (Wave 0 independent observation)** · `rec:backlog:wave0-independent-observation`

The separate observer receives a closed public manifest and answers acquired through the node interface. Its five imports and forbidden input names are pinned by `observer_imports_and_input_surface_are_pinned`; it reads standard input and writes standard output. The producer exits before acquisition, its wallet directory is removed, and a fresh node opens the retained chain directory. The coordinator awaits each child separately and removes the chain directory at completion. No producer response, archive, fixture or temporary file supplies the observer's semantic input. The manifest declares the variable schedule, lead bounds, static root, internal policy, contract revision, STATE output index and explicit candidate and ABI projections; the locator identifies the public transaction.

**Table (Independent observation readings)** · `tab:backlog:independent-observation-readings`

| Reading | What was supplied | What was read from the node | Outcome | Standing or disposition | Pinning test |
| --- | --- | --- | --- | --- | --- |
| `16.13 / missing-transaction` | Manifest without its transaction locator | Unaltered acquisition answers | `missing-public-input:transaction_id` | Refused before public recovery | `closed_manifest_refuses_every_missing_and_private_field` |
| (`rule:guide14-exec:checkpoint`) | Altered declared genesis | Actual genesis and chain name | `checkpoint-genesis_id-disagrees` | Checkpoint refusal; no Safety reclassification | `boundary_replay_checks_seven_outcomes_and_separate_report_roles` |
| `16.13 / copied-transaction-bytes` | Offered complete bytes changed in one position | Exact transaction including witness | `copied-transaction-bytes` | Refused at the acquired-byte binding | `boundary_replay_checks_seven_outcomes_and_separate_report_roles` |
| (`rule:guide14-exec:checkpoint`) | Altered offered block hash | Containing block and active block at its height | `checkpoint-block-disagrees` | Checkpoint refusal; no Safety reclassification | `boundary_replay_checks_seven_outcomes_and_separate_report_roles` |
| (`rule:guide14-exec:checkpoint`) | Altered offered height | Containing block height | `checkpoint-height-disagrees` | Checkpoint refusal; no Safety reclassification | `boundary_replay_checks_seven_outcomes_and_separate_report_roles` |
| `equal-final-cursor / altered-predecessor-edge-refused` | Changed predecessor edge followed by the honest edge; equal final cursor | Honest transaction and containing block | `PredecessorIsNotTheCursor` at the first edge | Refused despite equal final cursors | `boundary_replay_checks_seven_outcomes_and_separate_report_roles` |
| (`gate:guide15-exec:bite-0-5-conjunct-11`) | Public manifest and the single acquired edge | Complete bytes, identity, containing block, height, membership and active-block answers | Validated history report with one edge and a node-bound checkpoint | Observable edge acquisition and validation MET; retained non-claims below | `observer_archive_replays_reports_and_preserves_each_boundary` |
| (`gate:guide15-exec:bite-0-5-conjunct-13`) | Observer-built public handoff | Complete bytes and the STATE output program | Validated public-recovery report; `programs_agree` true | Observable public recovery and program comparison MET; retained non-claims below | `observer_archive_replays_reports_and_preserves_each_boundary` |

**Result (Independent acquisition and conjunct dispositions)** · `result:backlog:independent-observation`

The four recaptured files in `packages/vectors/fixtures/maturity-observer-run-of-record/`, carrying the namespace probe diagnostic and separate measured child walls, are admitted once by `maturity_observer_run_of_record`, under `MATURITY_OBSERVER_MANIFEST_SHA256`, `MATURITY_OBSERVER_RUN_ADDRESS` and `MATURITY_OBSERVER_CEREMONY_SHA256`. Admission checks the manifest and ordered report ceremony (preflight, build, producer, acquisition, observer, census, manifest and coordinator walls, followed by their summed elapsed wall and the three child exits), binds each reported child exit to its recorded result, replays all seven experiment outcomes and both reports, binds the recorded process exits and isolation evidence, and requires the submitted and independently acquired bytes to equal the variable archive's 1,701-byte subject exactly, witness included. `observer_archive_refuses_each_changed_file` pins all four file boundaries. The capture records the actual chain name and genesis, containing block and height, membership and active-block readings, and both recomputed transaction identities. Relay and mempool floors are node answers of `0.00000100 BTC/kvB`; the block floor is `0.00000100 BTC/kvB` from the pinned source with no override. The actual route is relay refusal followed by direct-block inclusion.

Conjunct 11 is MET for independent acquisition of the exact announcement, its one validated root edge and its bound checkpoint; equality of the final cursor alone is explicitly refused by `equal-final-cursor / altered-predecessor-edge-refused`, related to `16.8 / successor-cursor-restored-after-invalid-intermediate-edge`. Conjunct 13 is MET for recovery from the acquired public handoff and agreement of the reconstructed program with the observed output. These are dispositions by parts under (`crit:guide15-exec:observer-independence`) and (`crit:guide15-exec:conjunct-observation`), not revisions of the historical CONDITIONALLY MET entries at (`sec:phase6:gate-record`). The history and recovery report roles stay separate, retaining conjunct 15's distinction. Only a checked observation selects the acquisition residuals: the history report uses `single-node-acquisition-only`; the recovery report preserves every structural validator line and appends `acquisition_residual single-node-acquisition-only replaces no-chain-observation`. Cursor freshness and archive-origin residuals remain and full-artifact authentication remains unclaimed. A source tag alone supplies no observation credit.

The census stays 206 / 119 / 87 / 11 / 38 / 29: denominator, answered, outstanding, native acceptances, native refusals and typed non-answers respectively. The existing root-history and public-recovery buckets remain 16 and 14. This archive adds independent acquisition evidence without reclassifying a Safety row. The close-safety step (`step:wave0:bite-0-5-close-safety`) inherits both conjunct dispositions by parts, the six recorded refusals and all retained non-claims, alongside the eighty-seven outstanding obligations. Retained evidence belongs to `evidence/wave0/observer/<run-address>/`; admitted evidence is named by the fixture directory and its pins.

**Caveat (Acquisition boundary and chain lifetime)**

Construction is the isolation boundary in force: the pin covers the observer target and `maturity_observer`, `maturity_history`, `maturity_history_report`, `maturity_recovery`, `maturity_recovery_report`, `maturity_continuity`, `maturity_closure`, `maturity_safety`, `maturity_report` and `maturity_negative_half` along its call path. The scanned set is closed over callable imports with a named allowlist; the one archive-bearing callable, `maturity_evidence::derive_maturity_evidence_plan_with`, is imported only by `maturity_negative_half` for `outstanding_rows` in the scanned set, and the name refusal in scanned non-test regions and archive-free callable bodies is mechanical; only the call graph beyond the allowlisted bodies remains a reading. The path also uses the linker's continuity rule and the transaction, tapscript, realization, architecture and target crates, plus `serde_json` and `target_elements_conformance`. The observer has no filesystem APIs, no archive imports, no inherited environment and no file argument. The admitted probe diagnostic records that a filesystem namespace is unavailable: `bwrap` is absent and `unshare` refuses with `Operation not permitted`. Preflight records the excluded paths as unnamed in observer arguments and environment. Both processes use the same retained chain through separate node lifetimes; this establishes independent acquisition and validation of the exact subject from one node of record, with no second chain view, deployment or archival-availability claim (`cav:wave0:observer-node-limit`).

**Caveat (Fee-floor condition)**

The exact sponsorless form has no fee output and is included directly after its relay refusal. This establishes the observed subject needed by the one-input, one-output continuity and recovery readers, not positive-floor relayability. Ruling 115's fee-floor condition and conjunct 7 remain unchanged at (`sec:phase6:gate-record`).

**Caveat (Checkpoint premises and retained limits)**

Each checkpoint fact carries its own provenance: node answers for genesis, block, height, membership and active block; recomputation from acquired bytes for transaction identity and the two outpoints; public declaration bound to the reviewed contract; and explicit public projections for the chain identifier, candidate and ABI. Admission compares the declared chain identifier with the variable archive's chain identity. No archived caller-supplied branch context is reused. Active-block readings bind this acquisition to its containing chain prefix, but do not authenticate later cursor freshness. Full candidate and ABI artifacts, archive origin beyond exact admission, another node's view and future availability remain outside the claim.

### Report and relations · `sec:backlog:report-and-relations`

**Record (Wave 0 report consumers and relation dispositions)** · `rec:backlog:wave0-report-and-relations`

The report-and-relations step (`step:wave0:bite-0-5-report-and-relations`) supplies the report and relation readings of (`gate:guide15-exec:bite-0-5-report-gate`) and (`gate:guide15-exec:bite-0-5-relation-gate`). The fifteen report subjects are six metadata, two sponsor and seven protocol-report rows. Their accepted controls and refused subjects run through the consumers in `packages/vectors/src/maturity_report.rs`; the evidence fold in `packages/vectors/src/maturity_evidence.rs` consumes only the campaign's privately constructed admission object. No row is retyped.

**Table (Report consumer readings)** · `tab:backlog:report-consumer-readings`

All pairs use `ReportSemanticProjectionRejection` and retain the matrix's expected projection. The five individual field changes and compensated change compare the complete offered tuple against the successor recomputed from the admitted predecessor and request. Publication comparisons first validate structured fields against the source plan, matrix and register, then compare offered bytes with the canonical projection. Typed refusals below are variants of `MaturityReportConsumerRefusal`; `Structured` retains the underlying report refusal.

| Section / row | Accepted control | Changed subject | Validated consumer and exact refusal | Standing after | Pinning test |
|---|---|---|---|---|---|
| `16.3 / unaffected-field-omega-changed-alone` | Admitted nontrivial semantic successor accepted | Omega + 1 | `validate_report_semantic_projection`: `SemanticTermDiffers(Omega)` | `ReportLayerObserved` | `report_pair_metadata_omega` |
| `16.3 / unaffected-field-y-l-changed-alone` | Admitted nontrivial semantic successor accepted | YL + 1 | `validate_report_semantic_projection`: `SemanticTermDiffers(YL)` | `ReportLayerObserved` | `report_pair_metadata_y_l` |
| `16.3 / unaffected-field-y-t-changed-alone` | Admitted nontrivial semantic successor accepted | YT + 1 | `validate_report_semantic_projection`: `SemanticTermDiffers(YT)` | `ReportLayerObserved` | `report_pair_metadata_y_t` |
| `16.3 / unaffected-field-q-changed-alone` | Admitted nontrivial semantic successor accepted | Q + 1 | `validate_report_semantic_projection`: `SemanticTermDiffers(Q)` | `ReportLayerObserved` | `report_pair_metadata_q` |
| `16.3 / unaffected-field-cycle-changed-alone` | Admitted nontrivial semantic successor accepted | Cycle + 1 | `validate_report_semantic_projection`: `SemanticTermDiffers(Cycle)` | `ReportLayerObserved` | `report_pair_metadata_cycle` |
| `16.3 / semantic-field-changes-compensated-by-another-field` | Admitted nontrivial semantic successor accepted | Omega + 1 and YL − 1, preserving their sum | `validate_report_semantic_projection`: `SemanticTermDiffers(Omega)` | `ReportLayerObserved` | `report_pair_metadata_compensation` |
| `16.10 / report-publishes-sponsor-amount` | Validated source report and canonical bytes accepted | Append sponsor_amount | `validate_maturity_safety_publication`: `ForbiddenPublication(sponsor_amount)` | `ReportLayerObserved` | `report_pair_sponsor_amount` |
| `16.10 / report-publishes-sponsor-opening` | Validated source report and canonical bytes accepted | Append sponsor_opening | `validate_maturity_safety_publication`: `ForbiddenPublication(sponsor_opening)` | `ReportLayerObserved` | `report_pair_sponsor_opening` |
| `16.12 / caller-authored-evidence-standing` | Validated source report and canonical bytes accepted | Claim one extra report-layer observation | `validate_maturity_safety_publication`: `Structured(CensusBucketDiffers(report_layer_observed))` | `ReportLayerObserved` | `report_pair_authored_standing` |
| `16.12 / report-summary-edited` | Validated source report and canonical bytes accepted | Claim one extra answered row | `validate_maturity_safety_publication`: `Structured(AnsweredOrOutstandingDiffers(answered))` | `ReportLayerObserved` | `report_pair_summary` |
| `16.12 / failed-row-removed` | Validated source report and canonical bytes accepted | Remove a native refusal row | `validate_maturity_safety_publication`: `Structured(NativeRefusalCountDiffers)` | `ReportLayerObserved` | `report_pair_removed_failure` |
| `16.12 / duplicate-passing-row` | Validated source report and canonical bytes accepted | Duplicate a witnessed passing row | `validate_maturity_safety_publication`: `Structured(WitnessedRefusalCountDiffers)` | `ReportLayerObserved` | `report_pair_duplicate_pass` |
| `16.12 / observed-target-layer-differs-from-declared-row-boundary` | Validated source report and canonical bytes accepted | Falsify the retained observed layer | `validate_maturity_safety_publication`: `Structured(NativeRefusalDiffers)` | `ReportLayerObserved` | `report_pair_falsified_boundary` |
| `16.12 / canonical-report-includes-wall-time` | Validated source report and canonical bytes accepted | Append wall_time | `validate_maturity_safety_publication`: `PublicationDiffers(first surplus line)` | `ReportLayerObserved` | `report_pair_wall_time` |
| `16.12 / typed-diagnostic-contains-a-caller-path-or-injected-line` | Validated source report and canonical bytes accepted | Append a diagnostic containing a path and an extra line | `validate_maturity_safety_publication`: `PublicationDiffers(first surplus line)` | `ReportLayerObserved` | `report_pair_diagnostic` |

The boundary-layer pair refuses a falsified observation, not an honestly reported contradiction. `truthful_unexpected_boundary_stays_failed_and_only_the_falsification_refuses` accepts the truthful structured report with completeness `Failed`, then refuses its falsified layer. `publication_bytes_refuse_altered_surplus_duplicate_and_diagnostic_lines` independently pins offered byte changes, including a path-only diagnostic and an injected extra line.

**Result (Report admission and relation reconciliation)** · `result:backlog:report-and-relations`

The source and folded census is 206 / 119 / 87 / 15 / 0 / 29 → 206 / 134 / 72 / 0 / 15 / 29: denominator, answered, outstanding, report-layer-required, report-layer-observed and typed non-answers. Exactly fifteen admitted report pairs account for the movement; native observations and typed non-answers do not move. Typed finality remains 22 final and 7 waiting. `report_campaign_moves_only_its_fifteen_rows` compares every keyed row before and after, and the existing census tests retain the denominator and bucket partition.

The two-way join required by (`verif:wave0:campaign-join`) lives in `admit_report_readings`. Manifest membership comes from matrix boundaries. Every required section-and-name key must have its exact accepted control, refused subject, boundary and expected projection; every output receiving credit must belong to that manifest. `report_campaign_joins_every_manifest_key_in_both_directions` pins equality; `report_join_refuses_missing`, `report_join_refuses_duplicate`, `report_join_refuses_surplus`, `report_join_refuses_wrong_section` and `report_join_refuses_changed_subject_and_projection` pin the refusals. Derivation stages the source plan, report campaign and admission, evidence fold, then final report assembly and validation. Neither a test name nor a rendering result grants standing.

Relations remain 14 / 108 / 84 before and after this step, with reasons 55 / 9 / 9 / 0 / 11. The dispositions are recorded in (`tab:backlog:relation-dispositions`). `the_whole_transaction_scoped_rows_are_exactly_the_recounted_set` reconstructs the historical 14 / 102 / 90 and 55 / 9 / 9 / 17 by moving the six named exact links back into the prior seventeen-row scope bucket with the eleven exclusions. `every_non_link_has_a_keyed_semantic_disposition` pins all eighty-four remaining keys. This step adds no relation links.

**Caveat (Report authority)**

These are report-layer observations under (`tab:wave0:report-consumers`). Rendering supplies neither target evidence nor policy acceptance. The semantic consumer compares admitted semantic material; the publication consumer rejects discrepancies in a report's claims and bytes. Neither broadens a target validator.

**Caveat (Combined-projection wait)**

The combined-projection row formerly retained `AcceptanceAwaitsRelayAdmissibility` because sponsorless bytes failed the fee floor and sponsored bytes failed the readers' count restriction. The requirement for one exact transaction satisfying acceptance and every projection is discharged by the positional readers and selected S2 acceptance (`rec:backlog:wave0-combined-projection`); the earlier native observation remains historical evidence.

**Caveat (Inherited closure obligations)**

The close-safety step (`step:wave0:bite-0-5-close-safety`) inherited at that admission seventy-two outstanding rows, the combined-projection wait, the balanced confidential route, the remaining conjunct dispositions and all acquisition and authentication residuals recorded at (`rec:backlog:wave0-independent-observation`). The current census and retained obligations are recorded at (`result:backlog:close-safety-reconciliation`). Neither (`gate:wave0:bite-0-5-packet`) nor the close-safety gate closes here. At that admission, `T12-005` remained TODO and `T11-064` retained its obligations; their closure is recorded at (`result:backlog:bite-0-5-gate-pass`).


### Relation dispositions · `sec:backlog:relation-dispositions`

The table disposes the ninety historical non-links: eighty-four retained non-links and six exact links already established by (`rec:backlog:wave0-region-refit`).

**Table (Relation dispositions)** · `tab:backlog:relation-dispositions`

| Section / row | Reason or prior scope | Deciding semantic fact or exact relation / subject / class | Disposition after | Pinning test |
|---|---|---|---|---|
| `16.2 / equal-to-current-cycle-where-outside-the-lead` | NoPublishedRelationNamesIt | The pinned request coordinate is a State transition/window law; no expression predicate is published. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.2 / lower-bound-addition-overflow` | NoPublishedRelationNamesIt | The pinned request coordinate is a State transition/window law; no expression predicate is published. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.2 / malformed-cycle-encoding` | NoPublishedRelationNamesIt | The pinned request coordinate is a State transition/window law; no expression predicate is published. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.2 / one-above-maximum` | NoPublishedRelationNamesIt | The pinned request coordinate is a State transition/window law; no expression predicate is published. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.2 / one-below-minimum` | NoPublishedRelationNamesIt | The pinned request coordinate is a State transition/window law; no expression predicate is published. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.2 / request-cycle-differs-from-successor` | NoPublishedRelationNamesIt | The pinned request coordinate is a State transition/window law; no expression predicate is published. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.2 / upper-bound-addition-overflow` | NoPublishedRelationNamesIt | The pinned request coordinate is a State transition/window law; no expression predicate is published. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.3 / correct-semantic-metadata-with-noncanonical-representation-nonce` | NoPublishedRelationNamesIt | The pinned codec coordinate is not representation mode or authentication; the published State representation relation names only those. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.3 / duplicate-one-field` | NoPublishedRelationNamesIt | The pinned codec coordinate is not representation mode or authentication; the published State representation relation names only those. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.3 / noncanonical-enum-tag` | NoPublishedRelationNamesIt | The pinned codec coordinate is not representation mode or authentication; the published State representation relation names only those. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.3 / nonzero-reserved-field` | NoPublishedRelationNamesIt | The pinned codec coordinate is not representation mode or authentication; the published State representation relation names only those. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.3 / omit-one-field` | NoPublishedRelationNamesIt | The pinned codec coordinate is not representation mode or authentication; the published State representation relation names only those. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.3 / reorder-fields` | NoPublishedRelationNamesIt | The pinned codec coordinate is not representation mode or authentication; the published State representation relation names only those. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.3 / semantic-field-changes-compensated-by-another-field` | SubjectOutsideTheRelationsRegion | Offered successor metadata is outside the empty canonical asset-flow delta set; compensation does not turn it into an asset flow. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.3 / trailing-bytes` | NoPublishedRelationNamesIt | The pinned codec coordinate is not representation mode or authentication; the published State representation relation names only those. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.3 / unaffected-field-cycle-changed-alone` | SubjectOutsideTheRelationsRegion | Offered successor metadata is outside the empty canonical asset-flow delta set; compensation does not turn it into an asset flow. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.3 / unaffected-field-omega-changed-alone` | SubjectOutsideTheRelationsRegion | Offered successor metadata is outside the empty canonical asset-flow delta set; compensation does not turn it into an asset flow. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.3 / unaffected-field-q-changed-alone` | SubjectOutsideTheRelationsRegion | Offered successor metadata is outside the empty canonical asset-flow delta set; compensation does not turn it into an asset flow. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.3 / unaffected-field-y-l-changed-alone` | SubjectOutsideTheRelationsRegion | Offered successor metadata is outside the empty canonical asset-flow delta set; compensation does not turn it into an asset flow. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.3 / unaffected-field-y-t-changed-alone` | SubjectOutsideTheRelationsRegion | Offered successor metadata is outside the empty canonical asset-flow delta set; compensation does not turn it into an asset flow. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.3 / unknown-schema` | NoPublishedRelationNamesIt | The pinned codec coordinate is not representation mode or authentication; the published State representation relation names only those. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.3 / wrong-domain-separator` | NoPublishedRelationNamesIt | The pinned codec coordinate is not representation mode or authentication; the published State representation relation names only those. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.6 / later-admissible-nonce-instead-of-first` | NoPublishedRelationNamesIt | The pinned representation nonce is not operator witness availability or an external report identity. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.6 / wrong-representation-nonce` | NoPublishedRelationNamesIt | The pinned representation nonce is not operator witness availability or an external report identity. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.7 / nonce-search-exceeds-bound` | NoPublishedRelationNamesIt | Nonce enumeration and minimality are outside the operator constructibility relation. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.7 / nonce-skips-an-admissible-value` | NoPublishedRelationNamesIt | Nonce enumeration and minimality are outside the operator constructibility relation. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.7 / nonce-starts-at-one` | NoPublishedRelationNamesIt | Nonce enumeration and minimality are outside the operator constructibility relation. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.7 / repeated-hashing-used-as-fixed-point-search` | NoPublishedRelationNamesIt | Nonce enumeration and minimality are outside the operator constructibility relation. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.9 / add-ash` | SeveralPublishedRelationsFit | Both input and output family closures exclude this family; the row leaves the side unspecified. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.9 / add-authority-object` | SeveralPublishedRelationsFit | Both input and output family closures exclude this family; the row leaves the side unspecified. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.9 / add-burn-record` | NoPublishedRelationNamesIt | The event is not a declared canonical asset-flow family; no family or side is inferred from its name. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.9 / add-clear-event` | NoPublishedRelationNamesIt | The event is not a declared canonical asset-flow family; no family or side is inferred from its name. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.9 / add-destruction` | NoPublishedRelationNamesIt | The event is not a declared canonical asset-flow family; no family or side is inferred from its name. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.9 / add-distribution-control-or-vault` | SeveralPublishedRelationsFit | Both input and output family closures exclude this family; the row leaves the side unspecified. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.9 / add-entitlement` | SeveralPublishedRelationsFit | Both input and output family closures exclude this family; the row leaves the side unspecified. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.9 / add-issuance` | NoPublishedRelationNamesIt | The event is not a declared canonical asset-flow family; no family or side is inferred from its name. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.9 / add-live-receipt-input-or-output` | SeveralPublishedRelationsFit | Both input and output family closures exclude this family; the row leaves the side unspecified. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.9 / add-pace` | SeveralPublishedRelationsFit | Both input and output family closures exclude this family; the row leaves the side unspecified. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.9 / add-request` | SeveralPublishedRelationsFit | Both input and output family closures exclude this family; the row leaves the side unspecified. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.9 / add-resv-input-or-output` | SeveralPublishedRelationsFit | Both input and output family closures exclude this family; the row leaves the side unspecified. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.9 / add-time-locked-receipt` | SeveralPublishedRelationsFit | Both input and output family closures exclude this family; the row leaves the side unspecified. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.9 / introduce-canonical-u-ent-or-dist-ctl-flow` | Historical whole-transaction scope | CanonicalDeltaPolicy / Operation / UnexpectedCanonicalDeltaFamily | Declared (retained from region refit) | `the_whole_transaction_scoped_rows_are_exactly_the_recounted_set` |
| `16.10 / confidential-sponsor-value-where-the-selected-target-policy-claims-support` | NoPublishedRelationNamesIt | Confidential value support belongs to target policy; substrate conservation does not declare that support. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.10 / empty-sponsor-offer-for-sponsored-request` | SubjectOutsideTheRelationsRegion | Request/offer agreement is outside observed sponsor membership. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.10 / fee-change-substitution` | SubjectOutsideTheRelationsRegion | Fee/change value is outside value-blind sponsor isolation. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.10 / missing-sponsor-authorization` | Historical whole-transaction scope | SponsorIsolation / Sponsor / MissingSponsorAuthorization | Declared (retained from region refit) | `the_whole_transaction_scoped_rows_are_exactly_the_recounted_set` |
| `16.10 / report-publishes-sponsor-amount` | TheReportIsTheSubject | Report carrier, ReportField locator and report semantic rejection boundary; no target relation is inferred. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.10 / report-publishes-sponsor-opening` | TheReportIsTheSubject | Report carrier, ReportField locator and report semantic rejection boundary; no target relation is inferred. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.10 / sponsor-change-at-state-output-0` | SubjectOutsideTheRelationsRegion | Absolute output position is outside role membership. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.10 / sponsor-member-unclassified` | Historical whole-transaction scope | SponsorIsolation / Sponsor / SponsorMemberUnclaimed | Declared (retained from region refit) | `the_whole_transaction_scoped_rows_are_exactly_the_recounted_set` |
| `16.10 / state-sponsor-overlap` | Historical whole-transaction scope | SponsorIsolation / Sponsor / SponsorProtocolOverlap | Declared (retained from region refit) | `the_whole_transaction_scoped_rows_are_exactly_the_recounted_set` |
| `16.10 / state-successor-in-sponsor-range` | Historical whole-transaction scope | SponsorIsolation / Sponsor / SponsorProtocolOverlap | Declared (retained from region refit) | `the_whole_transaction_scoped_rows_are_exactly_the_recounted_set` |
| `16.10 / two-sponsor-envelopes` | Historical whole-transaction scope | SponsorEnvelopeMultiplicity / Sponsor / SponsorEnvelopeMultiplicityExceeded | Declared (retained from region refit) | `the_whole_transaction_scoped_rows_are_exactly_the_recounted_set` |
| `16.10 / zero-valued-sponsor-member-under-exact-role-structure` | SubjectOutsideTheRelationsRegion | Zero amount can preserve exact role structure; membership supplies no amount claim. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.11 / candidate-outside-bounds` | NoPublishedRelationNamesIt | The pinned ABI, witness or relocation coordinate is not the published operator constructibility subject or violation. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.11 / duplicate-coordinator` | NoPublishedRelationNamesIt | The pinned ABI, witness or relocation coordinate is not the published operator constructibility subject or violation. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.11 / no-coordinator` | NoPublishedRelationNamesIt | The pinned ABI, witness or relocation coordinate is not the published operator constructibility subject or violation. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.11 / predecessor-and-successor-metadata-witnesses-exchanged` | NoPublishedRelationNamesIt | The pinned ABI, witness or relocation coordinate is not the published operator constructibility subject or violation. Scope finality is recorded at (`rec:backlog:wave0-native-route-settlement`). | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.11 / relocation-applied-twice` | NoPublishedRelationNamesIt | The pinned ABI, witness or relocation coordinate is not the published operator constructibility subject or violation. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.11 / relocation-omitted` | NoPublishedRelationNamesIt | The pinned ABI, witness or relocation coordinate is not the published operator constructibility subject or violation. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.11 / state-and-sponsor-positions-exchanged` | SubjectOutsideTheRelationsRegion | Absolute state/sponsor ordering is outside region membership. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.11 / target-bytes-changed-after-abi-validation` | NoPublishedRelationNamesIt | The pinned ABI, witness or relocation coordinate is not the published operator constructibility subject or violation. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.11 / unresolved-lead-bound-symbol` | NoPublishedRelationNamesIt | The pinned ABI, witness or relocation coordinate is not the published operator constructibility subject or violation. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.11 / unresolved-metadata-schema-symbol` | NoPublishedRelationNamesIt | The pinned ABI, witness or relocation coordinate is not the published operator constructibility subject or violation. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.11 / unresolved-operator-symbol` | NoPublishedRelationNamesIt | The pinned ABI, witness or relocation coordinate is not the published operator constructibility subject or violation. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.11 / witness-item-reorder` | NoPublishedRelationNamesIt | The pinned ABI, witness or relocation coordinate is not the published operator constructibility subject or violation. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.11 / wrong-coordinator` | NoPublishedRelationNamesIt | The pinned ABI, witness or relocation coordinate is not the published operator constructibility subject or violation. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.11 / wrong-sequence` | NoPublishedRelationNamesIt | The pinned ABI, witness or relocation coordinate is not the published operator constructibility subject or violation. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.11 / wrong-transaction-version` | NoPublishedRelationNamesIt | The pinned ABI, witness or relocation coordinate is not the published operator constructibility subject or violation. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.12 / accepted-submission-without-identity` | NoPublishedRelationNamesIt | Request/response framing and field coherence are not published operation-relation subjects. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.12 / accepted-submission-without-mined-readback-where-required` | NoPublishedRelationNamesIt | Request/response framing and field coherence are not published operation-relation subjects. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.12 / blank-request` | NoPublishedRelationNamesIt | Request/response framing and field coherence are not published operation-relation subjects. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.12 / caller-authored-evidence-standing` | TheReportIsTheSubject | Report carrier, ReportField locator and report semantic rejection boundary; no target relation is inferred. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.12 / canonical-report-includes-wall-time` | TheReportIsTheSubject | Report carrier, ReportField locator and report semantic rejection boundary; no target relation is inferred. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.12 / conservation-rejection-carrying-accepted-only-openings` | NoPublishedRelationNamesIt | Request/response framing and field coherence are not published operation-relation subjects. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.12 / duplicate-passing-row` | TheReportIsTheSubject | Report carrier, ReportField locator and report semantic rejection boundary; no target relation is inferred. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.12 / failed-row-removed` | TheReportIsTheSubject | Report carrier, ReportField locator and report semantic rejection boundary; no target relation is inferred. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.12 / infrastructure-response-carrying-target-observation` | NoPublishedRelationNamesIt | Request/response framing and field coherence are not published operation-relation subjects. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.12 / malformed-json` | NoPublishedRelationNamesIt | Request/response framing and field coherence are not published operation-relation subjects. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.12 / observed-target-layer-differs-from-declared-row-boundary` | TheReportIsTheSubject | Report carrier, ReportField locator and report semantic rejection boundary; no target relation is inferred. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.12 / oversized-request` | NoPublishedRelationNamesIt | Request/response framing and field coherence are not published operation-relation subjects. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.12 / rejected-signing-response-carrying-witness` | NoPublishedRelationNamesIt | Request/response framing and field coherence are not published operation-relation subjects. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.12 / rejected-submission-carrying-accepted-identity` | NoPublishedRelationNamesIt | Request/response framing and field coherence are not published operation-relation subjects. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.12 / report-summary-edited` | TheReportIsTheSubject | Report carrier, ReportField locator and report semantic rejection boundary; no target relation is inferred. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.12 / typed-diagnostic-contains-a-caller-path-or-injected-line` | TheReportIsTheSubject | Report carrier, ReportField locator and report semantic rejection boundary; no target relation is inferred. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.12 / unknown-request-field` | NoPublishedRelationNamesIt | Request/response framing and field coherence are not published operation-relation subjects. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.12 / unterminated-request` | NoPublishedRelationNamesIt | Request/response framing and field coherence are not published operation-relation subjects. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.12 / wrong-environment` | NoPublishedRelationNamesIt | Request/response framing and field coherence are not published operation-relation subjects. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.12 / wrong-provenance` | NoPublishedRelationNamesIt | Request/response framing and field coherence are not published operation-relation subjects. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |
| `16.12 / wrong-schema` | NoPublishedRelationNamesIt | Request/response framing and field coherence are not published operation-relation subjects. | Unlinked | `every_non_link_has_a_keyed_semantic_disposition` |

### Close safety · `sec:backlog:close-safety`

**Record (Wave 0 bounded Safety consolidation)** · `rec:backlog:wave0-close-safety`

This is the bounded consolidation of (`step:wave0:bite-0-5-close-safety`), not the closing verdict of (`gate:guide15-exec:bite-0-5-gate`) or (`gate:wave0:bite-0-5-packet`). The matrix has 205 constructor calls plus one direct row literal, retaining 206 section-and-name keys. The historical 206 / 78 / 128 partition and its 43 / 35 / 15 / 35 outstanding buckets remain the baseline at (`sec:phase6:exit-assessment`). The current 206 / 134 / 72 partition is the before state of this admission, not a replacement historical baseline.

The supplementary ledger at (`rec:backlog:wave0-safety-first-party`) contributes exactly five performed comparison discharges and two typed absence-of-coordinate proofs. Production derives the validated ledger once; a test-held reference check compares an offered ledger’s complete keys and retained evidence with a recomputation in both directions. Missing, duplicate, surplus and misbound entries refuse in `supplementary_join_refuses_missing_duplicate_surplus_and_misbound_evidence`. The evidence fold checks the required keys and prior reasons again before applying the ledger; `supplementary_admission_refuses_an_already_closed_plan` refuses a second application. Only the seven rows in (`tab:backlog:safety-closure-ledger`) change standing; no target observation, matrix boundary, relation link or fixture byte changes.

`supplementary_dispositions_join_required_keys_without_promoting_historical_answers` compares all 206 keys and complete unchanged row values, retains the original first-party evidence objects, and checks the fixed 78-key historical answer manifest. It reconstructs 78 / 43 / 35 / 15 / 35 and requires one supplementary ledger entry per changed standing. The fifteen report-consumer pairs retain their separate before-and-after test, `report_campaign_moves_only_its_fifteen_rows`.

**Result (Safety categories and retained partition)** · `result:backlog:close-safety`

Before: 206 / 134 / 72, first-party discharged / required 40 / 43. After: 206 / 139 / 67, first-party 45 / 36, retired 2. Native acceptance / refusal stays 11 / 38, report required / observed stays 0 / 15, root history / public recovery stays 16 / 14, and typed non-answers stay 29 with 22 final and 7 waiting. The five comparisons alone add five answers; retirement adds no answer. At that admission, the category split was observed 139 + final 22 + retired 2 + waiting 43 = 206; `bounded_consolidation_plan` reconstructs the original seven supplementary movements while replaying subsequent native boundaries, so its intermediate census is distinct from this historical admission. The current full census is recorded at (`result:backlog:close-safety-reconciliation`). At that admission, waiting comprised 36 first-party obligations and seven typed waits. Its outstanding figure means not answered: 67 = 22 final + 2 retired + 43 waiting, not 67 unperformed experiments. Completeness remains partial.

The Safety report schema advances from 2 to 3 because its grammar adds five keys: the detailed `retired` bucket and `disposition_observed`, `disposition_final`, `disposition_retired` and `disposition_waiting`. The aggregate keys are separate from the detailed buckets and native waiting-row lines. At that admission, category publication and its refusals had a detailed bucket sum of 206 and an outstanding-side sum of 67; the current publication and its census are recorded at (`result:backlog:close-safety-reconciliation`). The native protocol schema is unchanged. The relation partition remains 14 / 108 / 84 with reasons 55 / 9 / 9 / 0 / 11; (`tab:backlog:relation-dispositions`) retains all ninety former non-links, six now declared and eighty-four still justified non-links.

**Result (Current Safety census and non-answer reconciliation)** · `result:backlog:close-safety-reconciliation`

Following admitted confidential agreement at (`rec:backlog:wave0-confidential-route-admission`), the current Safety census is 206 rows, 177 answered and 29 outstanding, with observed, final, retired and waiting categories (177, 27, 2, 0). `the_census_figures_are_unmoved_by_the_binding_readers` and `closure_categories_partition_without_turning_finality_into_observation` pin 177 + 29 = 206, 177 + 27 + 2 + 0 = 206 and 27 + 2 + 0 = 29. The census test also pins report required/observed 0 / 15 and root history/public recovery 16 / 14. Typed non-answers are 27 = 27 final + 0 waiting, pinned by `each_typed_non_answer_stands_on_exactly_the_rows_its_reason_names` and `sponsored_retyping_ledger_pins_all_fourteen_rows`. Native acceptance/refusal is 12 / 39, totaling 51, and first-party discharged/required is 81 / 0, totaling 81, pinned by the census test and `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals`. `supplementary_safety_keys_and_census_are_exact` pins [41, 2, 0], with 41 + 2 + 0 = 43. The reason-membership test pins 179 layer-bound rows, with 179 + 27 = 206; `the_observed_product_reaches_each_row_at_its_own_boundary_only` pins 51 reachable + 128 unreachable = 179. `from_native_refusal_classifies_every_declared_row_at_every_observed_layer` pins 179 × 7 = 1,253 row/observed-layer pairs, 51 diagonal matches and 39 refusal-bindable rows. `the_whole_transaction_scoped_rows_are_exactly_the_recounted_set` pins the relation partition 14 + 108 + 84 = 206 and the remaining non-link reasons 55 + 9 + 9 + 0 + 11 = 84. `closure_category_publication_is_exact_and_refuses_each_changed_count` pins report schema 3 and the exact published categories. Finality and retirement add no observed answer; outstanding denotes not answered, and completeness remains partial. Diagnostic admission at (`rec:backlog:wave0-diagnostic-route-admission`) leaves every figure and standing in this result unchanged. Later admissions update this result as the home of the current census.

The re-read of (`gate:guide15-exec:bite-0-5-non-answer-gate`) retains every key of the charter inventory at (`data:guide15-exec:non-answer-census`). The current standings below are pinned by the named tests in `packages/vectors/src/maturity_safety.rs` and `packages/vectors/src/maturity_evidence.rs`; the zero-role comparison is in `packages/transaction/src/tests/state_construct_tests.rs`. The balanced-corruption row's current refusal is supplied by the later native archive at (`rec:backlog:wave0-native-observation`); the sponsored archive retains its historical contradiction at (`tab:backlog:sponsored-retyping-ledger`).

| Safety key | Standing now | Pin |
|---|---|---|
| `16.1 / repeated-equal-construction-producing-equal-candidate-bytes` | Final: `PropertyOfTheBuildRatherThanAFixture` | `each_typed_non_answer_stands_on_exactly_the_rows_its_reason_names` (membership counts and section placement); `MATURITY_SAFETY_ROWS` declarations (key identity) |
| `16.7 / nonce-starts-at-one` | Final: `TotalityAnsweredByLandedNonceEvidence` | `each_typed_non_answer_stands_on_exactly_the_rows_its_reason_names` (membership counts and section placement); `MATURITY_SAFETY_ROWS` declarations (key identity) |
| `16.7 / nonce-skips-an-admissible-value` | Final: `TotalityAnsweredByLandedNonceEvidence` | `each_typed_non_answer_stands_on_exactly_the_rows_its_reason_names` (membership counts and section placement); `MATURITY_SAFETY_ROWS` declarations (key identity) |
| `16.7 / nonce-search-exceeds-bound` | Final: `TotalityAnsweredByLandedNonceEvidence` | `each_typed_non_answer_stands_on_exactly_the_rows_its_reason_names` (membership counts and section placement); `MATURITY_SAFETY_ROWS` declarations (key identity) |
| `16.7 / invalid-internal-key-retried` | Final: `TotalityAnsweredByLandedNonceEvidence` | `each_typed_non_answer_stands_on_exactly_the_rows_its_reason_names` (membership counts and section placement); `MATURITY_SAFETY_ROWS` declarations (key identity) |
| `16.7 / malformed-metadata-retried` | Final: `TotalityAnsweredByLandedNonceEvidence` | `each_typed_non_answer_stands_on_exactly_the_rows_its_reason_names` (membership counts and section placement); `MATURITY_SAFETY_ROWS` declarations (key identity) |
| `16.7 / missing-leaf-retried` | Final: `TotalityAnsweredByLandedNonceEvidence` | `each_typed_non_answer_stands_on_exactly_the_rows_its_reason_names` (membership counts and section placement); `MATURITY_SAFETY_ROWS` declarations (key identity) |
| `16.7 / zero-tweak-rejected-merely-for-zero` | Final: `TotalityAnsweredByLandedNonceEvidence` | `each_typed_non_answer_stands_on_exactly_the_rows_its_reason_names` (membership counts and section placement); `MATURITY_SAFETY_ROWS` declarations (key identity) |
| `16.7 / tweak-at-or-above-group-order-accepted` | Final: `TotalityAnsweredByLandedNonceEvidence` | `each_typed_non_answer_stands_on_exactly_the_rows_its_reason_names` (membership counts and section placement); `MATURITY_SAFETY_ROWS` declarations (key identity) |
| `16.7 / identity-result-accepted` | Final: `TotalityAnsweredByLandedNonceEvidence` | `each_typed_non_answer_stands_on_exactly_the_rows_its_reason_names` (membership counts and section placement); `MATURITY_SAFETY_ROWS` declarations (key identity) |
| `16.7 / target-and-oracle-parity-disagree` | Final: `TotalityAnsweredByLandedNonceEvidence` | `each_typed_non_answer_stands_on_exactly_the_rows_its_reason_names` (membership counts and section placement); `MATURITY_SAFETY_ROWS` declarations (key identity) |
| `16.7 / repeated-hashing-used-as-fixed-point-search` | Final: `TotalityAnsweredByLandedNonceEvidence` | `each_typed_non_answer_stands_on_exactly_the_rows_its_reason_names` (membership counts and section placement); `MATURITY_SAFETY_ROWS` declarations (key identity) |
| `16.1 / minimum-valid-lead` | Observed acceptance | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.1 / maximum-valid-lead` | Observed acceptance | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.1 / representative-interior-lead` | Observed acceptance | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.1 / smallest-current-cycle` | Observed acceptance | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.1 / current-cycle-near-checked-upper-domain` | Observed acceptance | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.1 / nontrivial-unaffected-fields` | Observed acceptance | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.1 / representation-nonce-zero` | Observed acceptance | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.1 / representation-nonce-nonzero` | Observed acceptance | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.1 / accepted-target-transaction-with-semantic-constructor-root-and-recovery-projections-all-matching` | Observed acceptance | `combined_projection_binds_the_selected_acceptance_and_all_four_terms` |
| `16.1 / sponsored-without-change` | Observed acceptance | `sponsored_retyping_ledger_pins_all_fourteen_rows` |
| `16.1 / sponsored-with-change-where-supported` | Observed acceptance | `sponsored_retyping_ledger_pins_all_fourteen_rows` |
| `16.10 / state-sponsor-overlap` | Final: `SponsorConstructionAnsweredByLandedEvidence` | `sponsored_retyping_ledger_pins_all_fourteen_rows` |
| `16.10 / foreign-sponsor-asset` | Final: `SponsorConstructionAnsweredByLandedEvidence` | `sponsored_retyping_ledger_pins_all_fourteen_rows` |
| `16.10 / empty-sponsor-offer-for-sponsored-request` | Final: `SponsorConstructionAnsweredByLandedEvidence` | `sponsored_retyping_ledger_pins_all_fourteen_rows` |
| `16.10 / missing-sponsor-authorization` | Observed script-path refusal | `sponsored_retyping_ledger_pins_all_fourteen_rows` |
| `16.10 / sponsor-change-at-state-output-0` | Observed script-path refusal | `sponsored_retyping_ledger_pins_all_fourteen_rows` |
| `16.10 / fee-change-substitution` | Observed script-path refusal | `sponsored_retyping_ledger_pins_all_fourteen_rows` |
| `16.10 / state-successor-in-sponsor-range` | Final: `SponsorRegionAnsweredByLandedEvidence` | `sponsored_retyping_ledger_pins_all_fourteen_rows` |
| `16.10 / balanced-state-corruption-compensated-by-sponsor-change` | Observed script-path refusal in the later native archive | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| `16.10 / confidential-sponsor-value-where-the-selected-target-policy-claims-support` | Final: `ConfidentialSponsorValueUndecidedByCandidate` | `confidential_acceptance_binds_final_build_property_standing` |
| `16.10 / two-sponsor-envelopes` | Final: `SponsorRegionAnsweredByLandedEvidence` | `sponsored_retyping_ledger_pins_all_fourteen_rows` |
| `16.10 / sponsor-member-unclassified` | Final: `SponsorRegionAnsweredByLandedEvidence` | `sponsored_retyping_ledger_pins_all_fourteen_rows` |
| `16.10 / zero-valued-sponsor-member-under-exact-role-structure` | Final: `ZeroSponsorRoleStructureAnsweredByLandedEvidence` | `sponsored_retyping_ledger_pins_all_fourteen_rows`; `zero_sponsor_value_does_not_erase_membership` |

`each_typed_non_answer_stands_on_exactly_the_rows_its_reason_names` pins the membership counts and section placement of all thirteen reasons in `packages/vectors/src/maturity_safety.rs`; `MaturityRowBoundary::finality` supplies their typed finality.

| Reason | Finality | Members |
|---|---|---:|
| `AcceptanceAwaitsRelayAdmissibility` | Waiting | 0 |
| `NativeCoordinateUndecidedByCandidate` | Final | 4 |
| `NativeSubjectAwaitsConstructionRoute` | Waiting | 0 |
| `NativeCoordinateAbsentFromCandidate` | Final | 3 |
| `SponsoredControlRefusedAtConstruction` | Waiting | 0 |
| `SponsorArmAwaitsRegionScoping` | Waiting | 0 |
| `TotalityAnsweredByLandedNonceEvidence` | Final | 11 |
| `PropertyOfTheBuildRatherThanAFixture` | Final | 1 |
| `SponsorConstructionAnsweredByLandedEvidence` | Final | 3 |
| `SponsorRegionAnsweredByLandedEvidence` | Final | 3 |
| `ZeroSponsorRoleStructureAnsweredByLandedEvidence` | Final | 1 |
| `ConfidentialSponsorAwaitsBalancedProofRoute` | Waiting | 0 |
| `ConfidentialSponsorValueUndecidedByCandidate` | Final | 1 |

Final membership sums to 4 + 3 + 11 + 1 + 3 + 3 + 1 + 1 = 27; waiting membership sums to 0 + 0 + 0 + 0 + 0 = 0. `sponsored_retyping_ledger_pins_all_fourteen_rows` also pins typed 27 = 27 final + 0 waiting.

The charter inventory is 11 totality + 1 build-property + 9 relay + 11 sponsored-control + 3 sponsor-arm = 35 keys. Its twelve existing finals remain final; its twenty-three former waits now comprise fifteen observed rows and eight finals on their admitted grounds. Thus the charter keys total 15 observed + 20 final + 0 retired + 0 waiting = 35. The current typed set is 20 charter finals + 7 coordinate finals = 27: `16.4 / stale-operator`, `16.6 / wrong-control-recipe` and `16.11 / predecessor-and-successor-metadata-witnesses-exchanged` take the absent-coordinate reason; `16.6 / later-admissible-nonce-instead-of-first`, `16.6 / two-state-successors`, `16.6 / extra-state-like-output` and `16.11 / wrong-transaction-version` take the undecided-coordinate reason. Those seven were native rows at charter, while fifteen of the charter's thirty-five typed rows are observed now. For the twelve charter-final rows, `each_typed_non_answer_stands_on_exactly_the_rows_its_reason_names` pins membership counts and section placement; their key identities are declared in `MATURITY_SAFETY_ROWS`, exposed by `rows()`, in `packages/vectors/src/maturity_safety.rs`. The keyed native ledger (`tab:backlog:native-observation-rows`), `sponsored_retyping_ledger_pins_all_fourteen_rows`, `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` and `combined_projection_binds_the_selected_acceptance_and_all_four_terms` retain these identities; `retained_two_successor_acceptance_binds_final_undecided_standing` pins the later two-successor final and `confidential_acceptance_binds_final_build_property_standing` pins the confidential final. The map sums 9 relay + 6 sponsored = 15 observed and 12 retained + 3 construction + 3 region + 1 zero-role + 1 confidential = 20 final, with 3 absent + 4 undecided = 7 additional coordinate finals.

Of the twenty-three waiting rows named by the gate, fifteen were discharged through observation and eight are final on their grounds under (`def:guide15-exec:non-answer-finality`), seven of them retyped on landed transaction tests at (`rec:backlog:wave0-sponsored-retyping`); ruling 135(a) accepts this discharge and records the gate MET (`rule:phase7:wave0-rulings`); (`result:backlog:bite-0-5-gate-pass`).

No final reason names a missing node, absent provisioning or an unperformed observable experiment. `NativeCoordinateAbsentFromCandidate` states that the retained candidate has no coordinate for operator migration, an independent successor-metadata witness or a successor control position; `NativeCoordinateUndecidedByCandidate` states that its covenant does not read nonce leastness, output count or transaction version; `ConfidentialSponsorValueUndecidedByCandidate` states that it does not read the sponsor value commitment, with selected support witnessed by admitted explicit/confidential agreement. These are properties of the retained candidate, and their premises and archive compatibility reopen when the candidate, target, ABI, constructor or selected policy changes under (`cond:wave0:baseline-freeze`). The zero-role row claims pre-target ABI-layout membership, answered by `zero_sponsor_value_does_not_erase_membership`; the funding refusal establishes no zero-valued-input execution, and that execution question is outside the row.

Ruling 134 adopts diagnostic attribution as remaining Wave 0 work under (`req:guide15-exec:position-acceptance`) on the merits of the native gate (`rule:phase7:wave0-rulings`); (`judg:w0route:recapture-wave`). The executor, reader and expected-class table are landed under (`spec:w0route:recapture-executor`); capture and separate admission at (`rec:backlog:wave0-diagnostic-route-admission`) complete (`step:wave0:bite-0-5-diagnostic-recapture`) under (`spec:w0route:recapture-admission`) and (`req:w0route:recapture-constraints`). Ruling 135(b) accepts the native gate's (`gate:guide15-exec:bite-0-5-native-gate`) attribution reading (`rule:phase7:wave0-rulings`). The recapture amends no admitted native archive. Existing boundary credit, standings, census and pins remain unchanged. The non-answer (`gate:guide15-exec:bite-0-5-non-answer-gate`) and native (`gate:guide15-exec:bite-0-5-native-gate`) gates are satisfied under ruling 135; (`result:backlog:bite-0-5-gate-pass`) records the bite (`gate:guide15-exec:bite-0-5-gate`) and packet (`gate:wave0:bite-0-5-packet`) gates MET and closes `T12-005` and the remaining `T11-064` obligations through (`rec:guide15-exec:bite-0-5-closure-carrier`). This result remains the single home of the current census.

**Caveat (Thirty-six first-party comparisons at bounded consolidation)**

At this admission, all thirty-six retained rows remained first-party-required with valid subjects but no invented malformed sibling. Subsequent existing-entry movements are recorded separately at (`rec:backlog:wave0-first-party-entries`), (`rec:backlog:wave0-abi-entries`) and (`rec:backlog:wave0-adapter-comparisons`); the list below preserves the obligations at bounded consolidation. The required ABI-classification and ABI-role entries are obligations, not existing validators; model conformance or a native refusal at another boundary cannot answer them. The comparisons then missing, keyed exactly as in (`tab:backlog:safety-first-party-rows`), were:

- `16.2 / malformed-cycle-encoding`: accepted typed cycle decoding versus malformed encoding through the required decoder.
- `16.2 / request-cycle-differs-from-successor`: the request-derived successor versus the changed offered successor through `check_offered`.
- `16.3 / correct-semantic-metadata-with-noncanonical-representation-nonce`: selected nonce versus the earlier admissible nonce in the canonical search comparison.
- `16.5 / metadata-leaf-missing`: complete program reconstruction versus omission of the metadata leaf.
- `16.9 / add-live-receipt-input-or-output`: admitted ABI families versus a live receipt on each side.
- `16.9 / add-time-locked-receipt`: admitted ABI families versus a time-locked receipt.
- `16.9 / add-ash`: admitted ABI families versus ASH.
- `16.9 / add-entitlement`: admitted ABI families versus entitlement.
- `16.9 / add-request`: admitted ABI families versus a request object.
- `16.9 / add-distribution-control-or-vault`: admitted ABI families versus distribution control or vault.
- `16.9 / add-resv-input-or-output`: admitted ABI families versus RESV on each side.
- `16.9 / add-pace`: admitted ABI families versus PACE.
- `16.9 / add-authority-object`: admitted ABI families versus an authority object.
- `16.9 / add-issuance`: issuance-free ABI classification versus offered issuance.
- `16.9 / add-destruction`: destruction-free ABI classification versus offered destruction.
- `16.9 / add-burn-record`: burn-free ABI classification versus an offered burn record.
- `16.9 / add-clear-event`: clear-free ABI classification versus an offered clear event.
- `16.9 / add-residue-projection`: admitted ABI projection versus a forbidden residue projection.
- `16.9 / introduce-canonical-u-ent-or-dist-ctl-flow`: admitted canonical flow versus unexpected canonical flow at the ABI-classification boundary.
- `16.11 / wrong-coordinator`: fixed coordinator versus another coordinator at the required ABI-role entry.
- `16.11 / duplicate-coordinator`: unique coordinator versus a duplicated role at that entry.
- `16.11 / no-coordinator`: required coordinator versus an absent role at that entry.
- `16.11 / state-and-sponsor-positions-exchanged`: declared positions versus exchanged positions at that entry, not a membership comparison.
- `16.11 / wrong-input-count`: admitted input cardinality versus a surplus input at that entry.
- `16.11 / wrong-output-count`: admitted output cardinality versus a surplus output at that entry.
- `16.11 / conflicting-leaf-weight`: unique weighted declaration versus conflicting weights through `TaptreeInput::new`.
- `16.11 / candidate-outside-bounds`: complete linked tree inside its declared cap versus an exceeded cap through `StateLinkedTaptree::bind`.
- `16.12 / blank-request`: valid framing versus a blank record through `RequestReader.read`.
- `16.12 / oversized-request`: bounded framing versus an oversized record through that reader.
- `16.12 / unterminated-request`: terminated framing versus a truncated record through that reader.
- `16.12 / malformed-json`: accepted request JSON versus malformed JSON.
- `16.12 / unknown-request-field`: declared request fields versus an unknown field through `require_execution_request_fields`.
- `16.12 / wrong-schema`: required revision versus a different schema through protocol environment binding.
- `16.12 / wrong-environment`: declared environment versus a mismatched observation through environment binding.
- `16.12 / wrong-provenance`: expected provenance versus a mismatched report through `ExecutionTranscript::provenance_agreement`.
- `16.12 / signing-response-bound-to-other-bytes`: requested bytes versus a different `signature_bound_to` through `bind_script_path_response`.

**Caveat (Admitted confidential and native routes)**

Settlement at (`rec:backlog:wave0-native-route-settlement`) established three absent coordinates as final build properties. The two retained constructions now have admitted observations at (`rec:backlog:wave0-native-route-capture-driver`): wrong semantic metadata refuses at the script path against an accepted same-coin control, while two STATE successors accept with matched readback and support the final undecided-coordinate reason. No construction wait remains. The recipe's chain reading is refuted by the candidate's transition and leaf census; this is not a missing-construction impossibility argument.

`16.10 / confidential-sponsor-value-where-the-selected-target-policy-claims-support` is final under `ConfidentialSponsorValueUndecidedByCandidate`, bound to admitted agreement at (`rec:backlog:wave0-confidential-route-admission`). The former waiting reason has zero members. The explicit route's proof-finalization refusal supplies none of those observations. The combined-projection row formerly waited because the sponsorless bytes failed the fee floor and the sponsored bytes failed the sponsorless readers' one-input, one-output restriction. Its one-transaction conjunction is now discharged by positional readers (`rec:backlog:wave0-combined-projection`); different transactions still cannot supply the conjunction. The zero typed waits and twenty-seven final reasons are pinned by `each_typed_non_answer_stands_on_exactly_the_rows_its_reason_names`, `sponsored_retyping_ledger_pins_all_fourteen_rows` and `confidential_acceptance_binds_final_build_property_standing`. Adapter comparisons are admitted at (`rec:backlog:wave0-adapter-comparisons`).

**Caveat (Admitted diagnostic attribution retains instruction-site limits)**

The immutable native archive at (`rec:backlog:wave0-native-observation`) contains twenty-two sponsorless refusal observations supporting twenty-four row claims. Its bare `mandatory-script-verify-flag-failed` verdict establishes the script boundary; the refusing check is not identified by this capture. The record-only disposition preserved boundary credit and supplied no opcode attribution; the admitted baseline now carries 24 agreements, one difference and zero absent claims, with the shared-site non-claim of ruling 134 (`rule:phase7:wave0-rulings`). The separate diagnostic admission at (`rec:backlog:wave0-diagnostic-route-admission`) binds each exact submitted subject to its accepted control, direct-block result and recorded script-error key. A class shared by several checks supplies no unique instruction-site attribution. No existing capture is amended. The retained semantic refusal adds one subject and claim to the current diagnostic obligation: twenty-five claims on twenty-three subjects (`rec:backlog:wave0-diagnostic-route-capture-driver`).

The affected claims are `16.2 / successor-remains-unannounced` and `successor-becomes-complete`; `16.3 / metadata-from-another-state-object`; `16.4 / empty-signature`, `malformed-signature`, `wrong-operator`, `valid-signature-under-another-key`, `valid-signature-over-another-candidate`, `successor-metadata-changed-after-signing` and `successor-program-changed-after-signing`; `16.5 / wrong-state-asset`, `wrong-singleton-amount`, `wrong-predecessor-program`, `predecessor-metadata-reconstructs-another-program` and `wrong-leaf-version`; `16.6 / successor-from-wrong-semantic-metadata`, `successor-from-predecessor-metadata-unchanged`, `wrong-announcement-cycle`, `wrong-representation-nonce`, `wrong-leaf-version`, `arbitrary-caller-supplied-output-program` and `no-state-successor`; and `16.11 / witness-item-reorder`, `control-block-from-another-program` and `target-bytes-changed-after-abi-validation`. The admitted recapture strengthens their diagnostic attribution with 24 agreements and one finding, and creates no new answers (`rec:backlog:wave0-diagnostic-route-admission`).

**Caveat (Conditional claims and retained limits)**

The NUMS judgment retains (`assum:guide15-exec:discrete-log`), (`assum:guide15-exec:sha256-preimage`) and (`fact:guide15-exec:nums-derivation`). `StateInternalKeyPolicy::new` recomputes SHA-256 over the uncompressed generator, compares the offered key with that result and `STATE_NUMS_KEY`, and checks the point. `nums_is_recomputed_from_public_uncompressed_generator` and `key_path_policy_retains_the_residual_without_claiming_impossibility` remain unchanged. A finite refusal proves no impossibility of a usable secret.

The sponsored region comparison reads admitted S1/S2 submission and mined bytes and matches each input to its retained funding outpoint. It projects exact STATE and sponsor positions into the six refitted relation identities, with opaque sponsor values. Owner identifiers and the sponsor signer set are labels assigned by the test, so the isolation owner clause passes by construction, backed only by asserted target acceptance of the exact bytes. The MET covers membership, family, asset and erasure, not an independent check of that owner clause. It refuses displaced membership and a foreign family inside the announcement region. It supplies no new chain observation, authorization theorem, combined state-machine transition or broader deployment claim. The independent observer's six refusals, separate report roles, cursor-freshness and archive-origin residuals, full-artifact-authentication non-claim and single-node limit (`cav:wave0:observer-node-limit`) remain in force. The confidential observation is admitted and its row final at (`rec:backlog:wave0-confidential-route-admission`); adapter comparisons are admitted at (`rec:backlog:wave0-adapter-comparisons`), and diagnostic attribution is admitted separately at (`rec:backlog:wave0-diagnostic-route-admission`), with its native-gate reading accepted by ruling 135(b) (`rule:phase7:wave0-rulings`). `T12-005`, `T11-064`, the packet and the close-safety gate close at (`result:backlog:bite-0-5-gate-pass`), retaining these non-claims.

**Result (Bite 0.5 gate pass)** · `result:backlog:bite-0-5-gate-pass`

Ruling 135 accepts the non-answer and native readings for (`task:guide15-exec:bite-0-5`) (`rule:phase7:wave0-rulings`). The gate table records each condition against its admitted evidence. The non-answer gate is MET by fifteen observations and eight admitted finals among the twenty-three former waits, retaining the twelve charter finals. The native gate is SATISFIED at the declared layers; diagnostic attribution records twenty-four agreements, one difference and zero absent claims without adding an answer or changing a standing.

| Gate or condition | Standing | Evidence and retained limit |
|---|---|---|
| (`task:guide15-exec:bite-0-5-deliverables`): dispose 128 outstanding, preserve 78 answers and all 206 identities | MET | (`result:backlog:close-safety-reconciliation`): 99 additional answers + 27 finals + 2 retired = 128; `supplementary_dispositions_join_required_keys_without_promoting_historical_answers` preserves historical admission. |
| (`pre:guide15-exec:sponsored-order`): ruling before C1 | MET | Ruling 130 (`rule:phase7:wave0-rulings`); (`rec:backlog:wave0-sponsored-construction`), (`rec:backlog:wave0-sponsored-observation`) and (`rec:backlog:wave0-sponsored-retyping`). |
| (`crit:guide15-exec:conjunct-observation`); (`crit:guide15-exec:observer-independence`) | MET | (`result:backlog:independent-observation`): separate acquisition process, exact bytes, checkpoint and environment bindings, separate report roles; single-node and authentication limits remain. |
| (`gate:guide15-exec:bite-0-5-first-party-gate`) | MET | (`rec:backlog:wave0-safety-first-party`), (`rec:backlog:wave0-first-party-entries`), (`rec:backlog:wave0-abi-entries`) and (`rec:backlog:wave0-adapter-comparisons`): supplementary 43 = 41 discharged + 2 out-of-contract; first-party 81 / 0. |
| (`gate:guide15-exec:bite-0-5-native-gate`) | SATISFIED under ruling 135(b) | (`result:backlog:native-observation`), (`rec:backlog:wave0-native-route-settlement`), (`rec:backlog:wave0-native-route-capture-driver`) and (`rec:backlog:wave0-diagnostic-route-admission`): exact controls and declared layers, admitted coordinate finals, diagnostic 24 / 1 / 0. |
| (`gate:guide15-exec:bite-0-5-report-gate`) | MET | (`rec:backlog:wave0-report-and-relations`); (`result:backlog:report-and-relations`): fifteen validated consumer dispositions, zero report-required rows. |
| (`gate:guide15-exec:bite-0-5-non-answer-gate`) | MET under ruling 135(a) | (`result:backlog:close-safety-reconciliation`): 35 = 15 observed + 20 final, with 27 current typed finals and zero waits. |
| (`gate:guide15-exec:bite-0-5-relation-gate`) | MET | (`result:backlog:region-links`); (`tab:backlog:relation-dispositions`): 14 / 102 / 90 → 14 / 108 / 84; non-links 55 / 9 / 9 / 0 / 11. |
| (`gate:guide15-exec:bite-0-5-conjunct-5`) | MET by parts | (`tab:backlog:conjunct-dispositions`): three MET, two RETIRED. Native escape refusals, conditional NUMS derivation and static-subtree comparison retained; announcement recipe coordinate and successor recipe construction retired with exact non-claims. |
| (`gate:guide15-exec:bite-0-5-conjunct-7`) | MET by parts | (`tab:backlog:conjunct-dispositions`): two MET, one RETIRED. S1/S2 and eight semantic forms have positive-floor evidence; broader deployment, sponsorless positive-floor relay and whole-schedule relay remain unclaimed. |
| (`gate:guide15-exec:bite-0-5-conjunct-11`) | MET by parts | (`tab:backlog:conjunct-dispositions`): one MET, three RETIRED. Exact independent edge observed; later freshness, origin and broader authentication and view guarantees unclaimed. |
| (`gate:guide15-exec:bite-0-5-conjunct-13`) | MET by parts | (`tab:backlog:conjunct-dispositions`): one MET, two RETIRED. Public successor reconstructed; future availability and broader origin and authentication unclaimed. |
| (`gate:guide15-exec:bite-0-5-conjunct-14`) | MET by parts | (`tab:backlog:conjunct-dispositions`): three MET, one RETIRED. Composed region, exact sponsored region binding and ABI entries; no leaf envelope-count enforcement or unsupported expansion claim. Owner and signer-label limitation remains. |
| (`gate:guide15-exec:bite-0-5-gate`) | MET | (`tab:backlog:conjunct-dispositions`): nineteen parts = ten MET + nine RETIRED + zero WAITING; historical NOT PASSED table preserved at (`sec:phase6:gate-record`). |
| (`gate:wave0:bite-0-5-packet`) | MET | (`tab:wave0:bite-0-5-steps`): all twenty-one step predicates accounted for below, keyed reconciliation retained, closure carrier linked. |
| `T12-005` closing condition | MET | (`rec:guide15-exec:bite-0-5-closure-carrier`); (`result:backlog:close-safety-reconciliation`); (`tab:backlog:conjunct-dispositions`): no forbidden final ground and every observable conjunct remainder performed. |
| `T11-064` owned obligations | MET | (`result:backlog:region-composition`); (`rec:guide15-exec:bite-0-5-closure-carrier`): region composition plus ruling on 58, C1, C2, C3 and positive floors, with their individual records below. |

| Step predicates | Closing evidence |
|---|---|
| (`step:wave0:bite-0-5-first-party`); (`step:wave0:bite-0-5-first-party-entries`) | (`rec:backlog:wave0-safety-first-party`), (`rec:backlog:wave0-first-party-entries`) and (`rec:backlog:wave0-adapter-comparisons`): retained exact comparisons and typed out-of-contract proofs. |
| (`step:wave0:bite-0-5-native-staging`); (`step:wave0:bite-0-5-observe-remaining`) | (`rec:backlog:wave0-safety-native-staging`), (`rec:backlog:wave0-native-observation`), (`rec:backlog:wave0-native-route-settlement`) and (`rec:backlog:wave0-native-route-capture-driver`): subjects and controls for observable coordinates; exact final reasons for absent or undecided coordinates under rulings 131–133, retained by ruling 135(a). The historical staging inventory stays intact. |
| (`step:wave0:bite-0-5-rule-sponsored`); (`step:wave0:bite-0-5-c1`); (`step:wave0:bite-0-5-c2`); (`step:wave0:bite-0-5-c3`) | Ruling 130 (`rule:phase7:wave0-rulings`); (`rec:backlog:wave0-sponsored-construction`), (`rec:backlog:wave0-sponsored-observation`) and (`rec:backlog:wave0-sponsored-retyping`). |
| (`step:wave0:bite-0-5-independent-observer`) | (`rec:backlog:wave0-independent-observation`); (`result:backlog:independent-observation`): exact edge and actual successor, with six refusal experiments. |
| (`step:wave0:bite-0-5-report-and-relations`) | (`rec:backlog:wave0-report-and-relations`); (`tab:backlog:relation-dispositions`): fifteen consumers and all old non-links dispositioned. |
| (`step:wave0:bite-0-5-close-safety`) | (`result:backlog:close-safety-reconciliation`); (`tab:backlog:conjunct-dispositions`): final reconciliation and 10 / 9 / 0 conjunct partition under ruling 135 (`rule:phase7:wave0-rulings`). |
| (`step:wave0:bite-0-5-abi-entries`); (`step:wave0:bite-0-5-abi-classification-entry`); (`step:wave0:bite-0-5-abi-role-entry`); (`step:wave0:bite-0-5-cycle-decoder-and-nonce-comparison`); (`step:wave0:bite-0-5-abi-comparisons-and-record`) | (`rec:backlog:wave0-abi-entries`); (`result:backlog:abi-entries`): twenty-three row comparisons, twenty-one classification subjects, six role subjects, decoder and canonical nonce comparisons; historical 28 → 5 required and 53 → 76 discharged preserved. |
| (`step:wave0:bite-0-5-native-routes`) | (`rec:backlog:wave0-native-route-settlement`), (`rec:backlog:wave0-native-route-construction`) and (`rec:backlog:wave0-native-route-capture-driver`): three absent-coordinate dispositions, semantic refusal and two-successor finality evidence. |
| (`step:wave0:bite-0-5-residual-routes`); (`step:wave0:bite-0-5-confidential-sponsor-route`); (`step:wave0:bite-0-5-combined-projection-route`) | (`rec:backlog:wave0-confidential-route-admission`); (`rec:backlog:wave0-combined-projection`): confidential final on admitted agreement; combined projection observed over one transaction. |
| (`step:wave0:bite-0-5-diagnostic-recapture`) | (`rec:backlog:wave0-diagnostic-route-admission`): completed separate admission, twenty-five claims, 24 / 1 / 0 attribution, unchanged standings. |

The nineteen conjunct parts are ten MET, nine RETIRED and zero WAITING (`tab:backlog:conjunct-dispositions`). Every observable remainder has its performed evidence and every retired part retains its exact non-claim. The historical NOT PASSED conjunction remains unchanged at (`sec:phase6:gate-record`). Thus (`gate:guide15-exec:bite-0-5-gate`) and (`gate:wave0:bite-0-5-packet`) are MET, closing `T12-005` and the remaining `T11-064` obligations through (`rec:guide15-exec:bite-0-5-closure-carrier`).

(`result:backlog:close-safety-reconciliation`) remains the single current census home. Earlier landing records retain their historical figures and pre-pass status, including the protected first-party-entry record; this result supplies their subsequent gate disposition. The first-party, closure, existing-entry and native-staging parsed spans retain 43, seven, fourteen and forty-two rows respectively. No archive, expected-class table, subject pin, row identity or standing changes in this gate pass.

For (`cond:guide15-exec:wave0-exit-condition-9`), the former blanket "no opcode attribution" non-claim is retired in favor of the admitted 24 / 1 / 0 attribution ledger and ruling 134's shared-site non-claim. The absent-coordinate, undecided-coordinate and confidential-value final premises reopen with their compatibility work under (`cond:wave0:baseline-freeze`). The retained conjunct limits, including NUMS assumptions, single-node acquisition, owner-label premises and deployment exclusions, remain in force.

Bite 0.7 inherits the execution-exit joins for all preflight rows, the twelve question dispositions, the remaining numbered position acceptances, the candidate audit and the clean-tree full repository baseline gate. Bite 0.5 supplies exit conditions 2, 3, 4 and 5, condition 1 through `T11-064`, and its figures and non-claims for condition 9. (`T4-009`), previously TODO and blocking consolidation and never an unfinished bite 0.5 obligation, closes on its own criteria at (`rec:backlog:wave0-compact-inherited-closure`); (`T12-007`) retains consolidation (`rec:guide15-exec:bite-0-7-closure-carrier`).

### ABI-entry Safety comparisons · `sec:backlog:abi-entries`

**Record (ABI-entry Safety comparisons)** · `rec:backlog:wave0-abi-entries`

The four landed entries are called over retained accepted controls and twenty-nine changed offers for twenty-three Safety rows. One staging object retains each row's exact nonempty subject inventory and each subject's expected refusal; a row discharges only after its control accepts and every distinct subject refuses exactly. The classification and role subjects retain their matrix control, pre-target boundary and carrier; the exchanged-positions comparison uses its constructed S1 control. No production validator changes in this comparison step.

**Table (ABI-entry comparisons)** · `tab:backlog:abi-entry-comparisons`

| Key | Accepted control | Changed coordinate | Owning entry and exact refusal | Standing after | Pinning test |
|---|---|---|---|---|---|
| `16.9 / add-live-receipt-input-or-output` | Sponsorless announcement | Live receipt added at input 1 and, separately, output 1; truthful family claims | `transaction::validate_maturity_abi_classification`: Input/output `AllowedObjectFamilies: UndeclaredObjectFamily`; each also `CanonicalDeltaPolicy` | First-party comparison; 2 subjects | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.9 / add-time-locked-receipt` | Sponsorless announcement | Time-locked receipt at output 1 | `transaction::validate_maturity_abi_classification`: Output closure `UndeclaredObjectFamily` plus `CanonicalDeltaPolicy` | First-party comparison; 1 subject | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.9 / add-ash` | Sponsorless announcement | ASH at output 1 | `transaction::validate_maturity_abi_classification`: Output closure `UndeclaredObjectFamily` plus `CanonicalDeltaPolicy` | First-party comparison; 1 subject | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.9 / add-entitlement` | Sponsorless announcement | ENT at output 1 | `transaction::validate_maturity_abi_classification`: Output closure `UndeclaredObjectFamily` plus `CanonicalDeltaPolicy` | First-party comparison; 1 subject | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.9 / add-request` | Sponsorless announcement | Request at output 1 | `transaction::validate_maturity_abi_classification`: Output closure `UndeclaredObjectFamily` | First-party comparison; 1 subject | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.9 / add-distribution-control-or-vault` | Sponsorless announcement | Distribution control and, separately, vault at output 1 | `transaction::validate_maturity_abi_classification`: Output closure `UndeclaredObjectFamily` plus `CanonicalDeltaPolicy`, for each alternative | First-party comparison; 2 subjects | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.9 / add-resv-input-or-output` | Sponsorless announcement | RESV at input 1 and, separately, output 1 | `transaction::validate_maturity_abi_classification`: Input/output closure `UndeclaredObjectFamily` | First-party comparison; 2 subjects | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.9 / add-pace` | Sponsorless announcement | PACE at output 1 | `transaction::validate_maturity_abi_classification`: Output closure `UndeclaredObjectFamily` | First-party comparison; 1 subject | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.9 / add-authority-object` | Sponsorless announcement | Entitlement authority and, separately, distribution authority at output 1 | `transaction::validate_maturity_abi_classification`: Output closure `UndeclaredObjectFamily`, for each alternative | First-party comparison; 2 subjects | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.9 / add-issuance` | Sponsorless announcement | Input 0 issuance flag and checked complete 74-byte issuance body | `transaction::validate_maturity_abi_classification`: `IssuanceOffered` | First-party comparison; 1 subject | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.9 / add-destruction` | Sponsorless announcement | Nonzero STATE-asset data-carrier output 1, claimed destruction | `transaction::validate_maturity_abi_classification`: `UnadmittedOutputRole { position: 1, claim: Destruction }` | First-party comparison; 1 subject | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.9 / add-burn-record` | Sponsorless announcement | Zero-value output 1 carrying `tag-burn` | `transaction::validate_maturity_abi_classification`: `UnadmittedOutputRole { position: 1, claim: DataOutput(Some(Burn)) }` | First-party comparison; 1 subject | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.9 / add-clear-event` | Sponsorless announcement | Zero-value output 1 carrying the clear tag | `transaction::validate_maturity_abi_classification`: `UnadmittedOutputRole { position: 1, claim: DataOutput(Some(Recon)) }` | First-party comparison; 1 subject | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.9 / add-residue-projection` | Sponsorless announcement | Output 1 carrying `tag-distribution-residue`, claimed projection | `transaction::validate_maturity_abi_classification`: `RelationVerdicts` with `ProjectionPolicy` | First-party comparison; 1 subject | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.9 / introduce-canonical-u-ent-or-dist-ctl-flow` | Sponsorless announcement | Three alternatives: U, ENT and DIST_CTL, each with input 1, output 1 and a lateral canonical flow | `transaction::validate_maturity_abi_classification`: `RelationVerdicts` with `CanonicalDeltaPolicy` and both `UndeclaredObjectFamily` closures, for each alternative | First-party comparison; 3 subjects | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.11 / wrong-coordinator` | Sponsorless announcement | Invented coordinator outpoint at input 0 | `transaction::validate_maturity_abi_roles`: `CoordinatorIsNotTheValidatedPredecessor { expected, offered }` | First-party comparison; 1 subject | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.11 / duplicate-coordinator` | Sponsorless announcement | Predecessor repeated at input 1 | `transaction::validate_maturity_abi_roles`: `CoordinatorRepeated { positions: [0, 1] }` | First-party comparison; 1 subject | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.11 / no-coordinator` | Sponsorless announcement | Input 0 replaced by a reserve sponsor member | `transaction::validate_maturity_abi_roles`: `CoordinatorMissing` | First-party comparison; 1 subject | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.11 / state-and-sponsor-positions-exchanged` | Sponsored S1, with two encoded witness-program reserve inputs and no change | STATE and sponsor positions 0 and 1 exchanged with their census and claims | `transaction::validate_maturity_abi_roles`: `StateAndSponsorPositionsExchanged { state: 1, sponsor: 0 }` | First-party comparison; 1 subject | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.11 / wrong-input-count` | Sponsorless announcement | Distinct surplus STATE input 1 | `transaction::validate_maturity_abi_roles`: `Cardinality { relation: input STATE cardinality, failure: CardinalityAboveMaximum }` | First-party comparison; 1 subject | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.11 / wrong-output-count` | Sponsorless announcement | Surplus STATE output 1 | `transaction::validate_maturity_abi_roles`: `Cardinality { relation: output STATE cardinality, failure: CardinalityAboveMaximum }` | First-party comparison; 1 subject | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects` |
| `16.2 / malformed-cycle-encoding` | Signing and binding chain's actual cycle item | The bound signing writer's requested-cycle stack item truncated from eight bytes to seven | `MaturityAnnouncementRequest::decode_announced_cycle`: `CycleWidth { declared: 8, offered: 7 }` | First-party comparison; 1 subject | `entry_pair_cycle_decoding_reads_the_bound_writer_item` |
| `16.3 / correct-semantic-metadata-with-noncanonical-representation-nonce` | Selected nonce, returning the search's whole evidence | The retained semantic at the first admitted nonce above its retained least nonce, inside the default budget | `CandidateStateConstructor::compare_offered_representation`: `NoncanonicalRepresentationNonce { offered, least }` | First-party comparison; 1 subject | `entry_pair_nonce_comparison` |

The seven family subjects with canonical-delta collateral are both live-receipt sides, time-locked receipt, ASH, entitlement, distribution control and distribution vault. The canonical-flow row separately retains both closure failures for each of its three assets. All six role subjects first classify; their input/output STATE cardinality statuses are respectively passed/passed, above-maximum/passed, below-minimum/passed, passed/passed, above-maximum/passed and passed/above-maximum. The changed-position S1 control is built locally over admitted witness programs and public reserve funding, without sponsor signing.

**Result (ABI-entry census)** · `result:backlog:abi-entries`

At ABI-entry admission, first-party required moves 28 → 5 and discharged 53 → 76; the supplementary partition moves [13, 2, 28] → [36, 2, 5]. The Safety census was 206 / 170 / 36: 170 observed, 22 final, 2 retired and 12 waiting, with waiting split into five adapter comparisons and seven typed waits. The exported `MaturityAbiValidationRow` index then had twenty-eight section/name keys, twenty-three `AbiFieldRefused` readings over twenty-nine subjects and five `TargetWasNotAsked` adapter readings. `UnsafeRawMutationExists` is a finding for a changed offer the entry accepts; invalid controls, unconstructible subjects and wrong refusal classes remain errors. The obligations fold refuses failed staging. `maturity_index_pins_every_key_subject_and_ledger_reading` pins the index against the ledger, and `classification_rows_require_every_subject_and_its_exact_refusal` rejects copied, missing, accepted and wrong-class alternatives, including the second and third subject of a row. The subsequent adapter comparisons and current index reading are recorded at (`rec:backlog:wave0-adapter-comparisons`).

The bounded-consolidation baseline remains unchanged. Its historical eight entry movements and these twenty-three ABI movements are disjoint; that step's preservation test compared their union of thirty-one and pinned 175 other rows unchanged. The fourteen-key entry record then had nine movements in that union's intersection, including the nonce. The original seven-movement reconstruction excludes all entry discharges. At that admission, native acceptance/refusal remained 11 / 38, report-layer required/observed 0 / 15, typed non-answers 29 and relation links 14 / 108 / 84; no report key changes and the schema remains 3. Those retained adapter requests are now admitted at (`rec:backlog:wave0-adapter-comparisons`). At that admission, the confidential observation and broader gates kept `T12-005` open; its closure is recorded at (`result:backlog:bite-0-5-gate-pass`).

**Caveat (ABI-entry limits)** · `cav:backlog:abi-entry-limits`

Claim independence is established by the transaction tests' alternative claims over the same bytes; the vectors comparisons pin the truthful claims and exact verdict sets. Claims never substitute for byte authentication. The sponsor maximum remains the observed, uncalibrated count retained by classification, not a calibrated bound supplied to roles. Issuance, destruction, burn and clear rows retain ABI-layout refusals without borrowing relation identities. All relation links and reasons remain unchanged.

Placement acceptance does not establish the presence rules for sponsor change only with a sponsor region or fees where required. The change-only transaction test retains that residual and pins the sponsor projection's recognition and cardinality statuses, including blocked isolation. The same-ABI-and-view premise of the role entry is explicit; this step does not broaden its checks. The decoder control is the signing writer's actual bound item, located by role identity; only truncation is staged here, with extension retained in the transaction tests. Nonce leastness is host leastness within the default search budget; the comparison adds no covenant leastness clause or new target observation.

### Combined projection · `sec:backlog:combined-projection`

**Record (One accepted transaction and four projections)** · `rec:backlog:wave0-combined-projection`

The combined §16.1 row is one conjunction over `sponsored-with-change-where-supported:control:52` in `packages/vectors/fixtures/maturity-sponsored-run-of-record/`. Admission binds submitted and mined bytes, both transaction identities, relay admission, ordinary template membership and block height 49. The archived funding census is joined by outpoint in input order. Classification and roles use one ABI and validated view; their authenticated claims establish positions while blocked sponsor relation verdicts remain blocked. The semantic transition uses the request decoder; both constructors retain host leastness; the root edge binds the same readback checkpoint; public recovery reconstructs the successor from the same bytes and public spent outputs. Four distinct term agreements and acceptance must share exact bytes and one transaction identity. The ten row-keyed positive controls comprise two sponsored and eight native controls; the separately reauthorized fee-change acceptance makes eleven relay-and-template acceptances in those archives. Only the selected S2 control supplies this conjunction.

**Result (Combined-projection census)**

At combined-projection admission, the row moved from `AcceptanceAwaitsRelayAdmissibility` to `Layer(AcceptedTransaction)` and stood `NativeAcceptanceObserved`. The Safety census moved 206 / 170 / 36 → 206 / 171 / 35: observed 171, final 22, retired 2 and waiting 11. Typed non-answers moved 29 → 28, with 22 final and six waiting; native acceptance moved 11 → 12 and native refusal stayed 38. First-party 76 / 5, supplementary [36, 2, 5], report-layer 0 / 15, relations 14 / 108 / 84 and schema 3 remained unchanged. The admitted sponsored archive recorded twelve acceptances and five refusals; assigning another row to an existing acceptance adds no archive observation. The intermediate test plans replay current native folds, while earlier recorded census results remain historical. `combined_projection_binds_the_selected_acceptance_and_all_four_terms`, `combined_fold_refuses_terms_from_two_accepted_transactions` and the census pins check the movement. The current first-party standing is recorded at (`rec:backlog:wave0-adapter-comparisons`).

**Caveat (Scope of the conjunction)**

The sponsorless readers and their input/output-count refusals remain unchanged. No target validator, fixture, archive or relation changes. Host leastness is not a covenant leastness claim, and the checkpoint supplies no current-chain freshness guarantee. The subsequent native-route dispositions are recorded at (`rec:backlog:wave0-native-route-settlement`) and (`rec:backlog:wave0-native-route-capture-driver`); adapter comparisons follow at (`rec:backlog:wave0-adapter-comparisons`). The balanced confidential observation is admitted at (`rec:backlog:wave0-confidential-route-admission`). Diagnostic recapture is complete at (`rec:backlog:wave0-diagnostic-route-admission`), and ruling 135(b)'s native-gate (`gate:guide15-exec:bite-0-5-native-gate`) reading is recorded at (`result:backlog:bite-0-5-gate-pass`). The one-or-four question remains open in `plans/drafts/accepted-byte-classification-study.md`; the denominator stays 206 under (`step:wave0:bite-0-5-combined-projection-route`). A follow-up can fold the sponsorless continuity, public-handoff derivation and recovery byte checks onto shared positional cores once their byte-identity constraints no longer bind the copies.


### Native route settlement · `sec:backlog:native-route-settlement`

**Record (Native coordinates on the admitted candidate)** · `rec:backlog:wave0-native-route-settlement`

The present variable-schedule maturity candidate is retained under rulings 131–133 (`rule:phase7:wave0-rulings`). Its seven ABI witness roles contain predecessor metadata but no successor-metadata witness; the announcement census's sixth member is a derived intermediate. Its constructor generation has one member and no migration relation. The linked static subtree has exactly the announcement leaf and zero support leaves, while the separate metadata commitment leaf always aborts. The only realized STATE transition is `announce_maturity`: it maps unannounced to announced and refuses announced and complete predecessors. Codec support and `with_maturity` can represent complete metadata but establish no transition to it. The leaf checks predecessor maturity against zero, so the proposed second announcement has no accepted control. The announcement module also declares cycle, clear, redeem, admit-deposits and receipt-relabel STATE exits that the dispatch leaves unrealized. Realizing any of these exits, including the maturity-plus-clear expansion retained as owed by ruling 131, reopens this premise and the recipe row's finality. The recipe row's declared constructibility relation is unchanged. `absent_coordinates_bind_the_candidate_witness_and_generation_censuses`, `candidate_leaf_and_transition_censuses_have_no_announced_state_spend` and `realization_operation_census_retains_only_the_announcement_state_transition` pin these premises.

**Table (Native route dispositions)** · `tab:backlog:native-route-settlement`

| Row and governing label | Former standing | Settlement | Standing now | Construction owed or ruling |
|---|---|---|---|---|
| `16.4 / stale-operator`; (`case:wave0:generation-preferred`) | `NativeSubjectAwaitsConstructionRoute; Waiting` | No operator-key migration relation; another signer remains wrong-operator | `NativeCoordinateAbsentFromCandidate; Final` | Ruling 131 (`rule:phase7:wave0-rulings`); `absent_native_coordinates_have_exact_final_membership` |
| `16.6 / successor-from-wrong-semantic-metadata`; (`step:wave0:bite-0-5-native-routes`) | `NativeSubjectAwaitsConstructionRoute; Waiting` | Distinct semantic census is composable through `link_for_metadata` | `NativeRefusalObserved` | Constructed and admitted; accepted same-coin control and observed script-path refusal recorded at (`rec:backlog:wave0-native-route-construction`): q increased by one under the same policy, first-admissible-nonce successor against the honest predecessor/request with refreshed authorization and proved semantic/program inequalities; honest control, observed `ScriptPathRejection` |
| `16.6 / wrong-control-recipe`; (`gate:guide15-exec:bite-0-5-conjunct-5`) | `NativeSubjectAwaitsConstructionRoute; Waiting` | No operation on the candidate spends an announced state; no successor control position exists | `NativeCoordinateAbsentFromCandidate; Final` | Ruling 133 (`rule:phase7:wave0-rulings`); `candidate_leaf_and_transition_censuses_have_no_announced_state_spend` |
| `16.6 / two-state-successors`; (`step:wave0:bite-0-5-native-routes`) | `NativeSubjectAwaitsConstructionRoute; Waiting` | Balanced construction below `construct_maturity_candidate` at `TargetTransaction` | `NativeCoordinateUndecidedByCandidate; Final` | Constructed and admitted; equivalent-coin control and matched direct-block acceptance recorded at (`rec:backlog:wave0-native-route-construction`): honest STATE input/output at zero plus an independently spendable same-asset coin and second successor, each amount 1, both authorizations over the full census, honest control and one issuance with reserve payments; declared `ScriptPathRejection`, observed acceptance taking `NativeCoordinateUndecidedByCandidate` |
| `16.11 / predecessor-and-successor-metadata-witnesses-exchanged`; (`step:wave0:bite-0-5-native-routes`) | `NativeSubjectAwaitsConstructionRoute; Waiting` | Successor metadata is derived, never an independent free witness | `NativeCoordinateAbsentFromCandidate; Final` | Ruling 132 (`rule:phase7:wave0-rulings`); `absent_coordinates_bind_the_candidate_witness_and_generation_censuses` |

At settlement the census remained 206 / 171 / 35: observed 171, final 25, retired 2 and waiting 8 = five first-party obligations + three typed waits. Typed non-answers remained 28 = 25 final + three waiting; construction-wait membership moved 5 → 2 and the absent-coordinate reason had three members. The undecided-coordinate reason retained three members. Native acceptance/refusal 12 / 38, first-party 76 / 5, relations 14 / 108 / 84, supplementary [36, 2, 5], historical staging [25, 3, 2, 5], all twenty-six subject digests and report schema 3 were unchanged. No observation was added or credited. `absent_native_coordinates_have_exact_final_membership` pins each boundary by key, and `historical_unstageable_subjects_keep_their_partition_after_scope_settlement` checks each evidence standing while separating current finality from the historical staging facts. Subsequent retained admission is recorded at (`rec:backlog:wave0-native-route-capture-driver`). Later adapter admission is recorded at (`rec:backlog:wave0-adapter-comparisons`).

The two constructions, their capture and separate admission are complete at (`rec:backlog:wave0-native-route-construction`) and (`rec:backlog:wave0-native-route-capture-driver`). The manifest precedes capture and whole-archive admission precedes each family's fold. Adapter comparisons are admitted at (`rec:backlog:wave0-adapter-comparisons`). Confidential agreement is admitted at (`rec:backlog:wave0-confidential-route-admission`) without changing the candidate. Diagnostic recapture is complete at (`rec:backlog:wave0-diagnostic-route-admission`), and ruling 135(b)'s native-gate (`gate:guide15-exec:bite-0-5-native-gate`) reading is recorded at (`result:backlog:bite-0-5-gate-pass`). The three scope settlements contribute to the non-answer gate's (`gate:guide15-exec:bite-0-5-non-answer-gate`) discharge under ruling 135(a), and (`result:backlog:bite-0-5-gate-pass`) records the native (`gate:guide15-exec:bite-0-5-native-gate`) and packet (`gate:wave0:bite-0-5-packet`) gates satisfied and `T12-005` closed; the current census remains at (`result:backlog:close-safety-reconciliation`). The generation-policy source entry in (`tab:guide15-exec:preflight`) retains its open expanded-generation execution exit; its ruling-131 reference is owed in the join at (`step:wave0:bite-0-7-join-entry-exits`). Each final premise is rechecked if the candidate changes under (`cond:wave0:baseline-freeze`).

### Native route construction · `sec:backlog:native-route-construction`

**Record (Retained native route constructions)** · `rec:backlog:wave0-native-route-construction`

The two constructions of (`step:wave0:bite-0-5-native-routes`) are staged without a target observation. `SuccessorFromWrongSemanticMetadata` changes only q by checked addition of one through `link_for_metadata`, derives the foreign successor at its first admissible nonce under the same policy and offers its program at output zero with the honest predecessor metadata and request retained. Prefix and nonce follow that program and authorization is refreshed; successor reconstruction is the expected refusing check. `TwoStateSuccessors` keeps honest STATE input and output zero and adds a separately spendable same-asset coin and successor at amount one, signing both inputs over the full spent-output census. Its declared `ScriptPathRejection` remains separate from expected acceptance under `OutputCountUnread`.

The twenty-eight subjects cover thirty staged keys with the two existing shared pairs unchanged. The current partition is [26, 4, 2, 3]; `historical_unstageable_subjects_keep_their_partition_after_scope_settlement` reconstructs [25, 3, 2, 5] by the five historical keys and checks three absent-coordinate finals separately from the two subsequently admitted constructions, preserving their historical staging classification. The historical result at (`result:backlog:safety-native-staging`) remains unchanged. `native_staging_leaves_the_historical_census_unchanged` reconstructs the historical denominator and supplementary partition, while `native_observation_preserves_the_denominator_and_staging_history` preserves the current census. Both new byte strings differ from every inherited admission and every other staged subject; the extra-output pin is unchanged.

The retained-routes planner and `replay_retained_routes` form a separate schedule from historical native replay. The semantic refusal is submitted first on the bootstrap predecessor coin, followed by its honest consuming control on that same coin. Only the two-successor family accepts its control before funding a fresh equivalent predecessor from the issuance reserve: its compatibility argument under (`pre:wave0:accepted-control`) keeps predecessor program, amount, asset, constructor, request and policy identical while outpoint, transaction identity, item 6 and the added key-path signature differ. The second STATE coin uses another reserve payment at amount one to the first published signer's key-path program. Funding validates asset, amount, program and distinct outpoint. The families are independent: a refused control retains the responses as diagnostic evidence and stops only that family; the schedule continues with the next family, and completion reports stopped families without admitting their exchanges. Changed requests, incomplete schedules and invalid readbacks refuse replay. Scripted responses test the schedule without claiming target acceptance.

**Table (Retained native construction measurements)** · `tab:backlog:native-route-construction`

| Subject and governing label | Spend-time pin | Control bytes / weight units | Subject bytes / weight units |
|---|---|---|---|
| `16.6 / successor-from-wrong-semantic-metadata`; (`step:wave0:bite-0-5-native-routes`) | `PIN_WRONG_SEMANTIC_METADATA` in `packages/vectors/src/maturity_mutants/native_staging.rs` | 1701 / 2091 | 1701 / 2091 |
| `16.6 / two-state-successors`; (`step:wave0:bite-0-5-native-routes`) | `PIN_TWO_STATE_SUCCESSORS` in `packages/vectors/src/maturity_mutants/native_staging.rs` | 1701 / 2091 | 1891 / 2638 |

Construction established the historical census 206 / 171 / 35 and categories 171 / 25 / 2 / 8, with typed non-answers 25 final plus three waiting. Subsequent retained capture and admission at (`rec:backlog:wave0-native-route-capture-driver`) bind both accepted controls, exact subjects, earlier funding and node-default policy: semantic metadata now has an observed script-path refusal; two successors has the final undecided-coordinate reason. The driver record states the admitted census; construction pins and report schema 3 remain unchanged. No production validator changes. At that retained landing, capture ran separately from the then-owed diagnostic recapture (`alg:wave0:campaign-manifest`); (`proto:wave0:native-admission`).

**Record (Retained native capture driver)** · `rec:backlog:wave0-native-route-capture-driver`

At retained native admission the census was 206 / 172 / 34: observed 172, final 26, retired two and waiting six = five first-party obligations plus one typed wait. Typed non-answers were 27 = 26 final plus one waiting; construction-wait membership was zero and the absent-coordinate reason had three members. The undecided-coordinate reason had four members. Native acceptance/refusal was 12 / 39; first-party 76 / 5, relations 14 / 108 / 84, supplementary [36, 2, 5], historical staging [25, 3, 2, 5], all inherited subject pins and report schema 3 remained unchanged. The retained archive added one refusal answer; its two-successor acceptance supports finality without adding an acceptance answer. `retained_semantic_refusal_binds_its_exact_row_and_same_coin_control` and `retained_two_successor_acceptance_binds_final_undecided_standing` pin those observations. `historical_unstageable_subjects_keep_their_partition_after_scope_settlement` separates current standing from historical staging facts. Later adapter admission is recorded at (`rec:backlog:wave0-adapter-comparisons`) and the current census at (`result:backlog:close-safety-reconciliation`).

The diagnostic executor recording, compatible reader and expected-class table have landed under ruling 134 (`rule:phase7:wave0-rulings`); this driver's admitted capture remains unchanged, and diagnostic capture and admission are separately complete at (`rec:backlog:wave0-diagnostic-route-admission`).

The `maturity_retained` target selects the ignored test `the_retained_maturity_routes_run_against_a_real_target` through `scripts/live-native-maturity-retained-capture.sh`, checked by `scripts/test-live-native-maturity-retained-capture.sh`. Its wrapper binds two `campaign-row` records inside the evidence payload and RUN-REPORT: typed claims, candidate and ABI, deployment, policy, constructor, mutation, declared boundary, control, funded outpoints, native fold and reverse observation-to-claim mapping under (`alg:wave0:campaign-manifest`); the unchanged retained schedule supplies the exchanges. The wrapper freezes intent before execution and binds funding before submission. Eligibility establishes structural completion and replay consistency, including contradictory outcomes and stopped-control diagnostics; admission requires the whole archive with no stopped family, agreement for both expectations, accepted controls with matched readback and complete replay; the fold then assigns each family's standing from its own observation under (`pre:wave0:accepted-control`) and (`proto:wave0:native-admission`). The capture of record `w16x1-retained-20261101T134825Z` is admitted byte-for-byte at `packages/vectors/fixtures/maturity-retained-run-of-record/`. Admission binds all twenty-four items below to the capture, timing, manifest and replayed campaign; policy-less records and stopped, absent or contradicting families refuse with the family and failed binding retained. Eligibility alone assigns no row credit. The retained parser consumes the following closed grammar; encoded fields use byte length and lower hexadecimal, JSON records have closed keys, and the four-file archive binds capture and timing through MANIFEST.sha256 and that manifest through RUN-REPORT. Items 16 to 21 bind an eligible report; an ineligible report carries the `missing` sentinel for an absent or duplicated retained field and every `campaign-row` the evidence record held.

1. `run-report-schema native-maturity-run-report 1`.
2. `capture-format-schema native-maturity-capture 1`.
3. `suite-commit`, `suite-tree`, `suite-clean yes`.
4. `rust-test-target maturity_retained`.
5. Encoded `rust-test-name` naming the retained test.
6. Encoded `cargo-argv`: pinned cargo, `test -p tripod-vectors --test maturity_retained -- --ignored --exact the_retained_maturity_routes_run_against_a_real_target --test-threads=1`.
7. Expected/observed test and ceremony counts one; setup counts zero.
8. `diagnostics-present yes`; passed/failed/ignored counts one/zero/zero; cargo exit zero.
9. Expected Elements tip, binary-reported revision, intended executed tip.
10. Adapter name/version and node name/version; adapter version `2.1.0`.
11. `protocol-revision 8`, `fixture-digest-algorithm forward-v2`.
12. Deployment environment, network, genesis and target contract.
13. `capture-profile retained`.
14. Four `phase-wall-ms` lines: preflight, cargo, census, manifest, each unsigned.
15. Unsigned `elapsed-wall-ms`.
16. Encoded `retained-policy`: object keys `relay`, `mempool`, `block`, `unit`, `block_identifier`, `relay_provenance`, `block_provenance`, `start_arguments`.
17. Encoded `retained-families`: semantic family then two-successor family; each record has keys `family`, `claim`, `control`, `subject`, `stopped`, `expectation`, `control_accepted`. Each claim has `section`, `name`; each non-null control or subject has `step`, `layer`, `route`, `readback`.
18. Encoded `retained-stopped`: ordered subset of the two families.
19. `campaign-manifest-schema 1`.
20. `campaign-manifest-count 2`.
21. Two encoded `campaign-row` records in family order, validated against typed intent and exchanges at admission; each row has keys `claim`, `operation`, `family`, `candidate`, `abi`, `target`, `deployment`, `branch`, `policy_requirement`, `policy`, `constructor`, `mutation`, `declared_boundary`, `expected_layer`, `control_id`, `compatibility`, `control`, `observation`, `expected_consumer`. Each claim has `section`, `name`; candidate has `schedule`, `subject_pin`; deployment has `network`, `genesis`; branch has `identifier`, `checkpoint`; each non-null control or observation has `step`, `digest`, `funded_outpoints`, `claims`; each funded-outpoint entry has `funding_step`, `output`, with the protocol's funded-output keys; policy uses the validated policy object from item 16.
22. `ceremony-roster begin`, one `ceremony report` digest, `ceremony-roster end`.
23. `manifest-sha256` digest.
24. `eligible yes` or `eligible no`, then end of file; eligibility assigns no row credit.

**Record (Diagnostic maturity capture driver)** · `rec:backlog:wave0-diagnostic-route-capture-driver`

The separate `scripts/live-native-maturity-diagnostic-capture.sh` selects `maturity_diagnostic` and its ignored test `the_diagnostic_maturity_routes_run_against_a_real_target`; `scripts/test-live-native-maturity-diagnostic-capture.sh` checks the driver without a node. The driver and Rust campaign implement (`step:wave0:bite-0-5-diagnostic-recapture`) under ruling 134 (`rule:phase7:wave0-rulings`). At the driver landing, capture of record was to follow import and gating, alone on the pinned target; capture, archive admission and attribution findings were then unperformed. That driver landing changed no fixture, historical archive, executor source, staged pin, Safety standing or census. Capture and admission are now recorded at (`rec:backlog:wave0-diagnostic-route-admission`).

`DiagnosticRoutesPlanner` composes the native planner's first six ranges and the retained planner's semantic family, stopping before either next family's funding. Its nominal census is 48 exchanges, 17 fundings, zero signings, 31 submissions, seven accepted controls and 24 refusals: 23 script-path refusals plus the consensus-layer `wrong-sequence`. Inner names and ordinals remain unchanged, with the retained phase starting at zero under its own family name. A refused control or an unexpectedly accepted negative that breaks the same-coin sequence stops only that family; its actual responses remain, later families continue, and no accepted response is fabricated.

The manifest freezes twenty-five claims on twenty-three subject rows before execution. Each row has one or two `claims`; the unannounced successor and wrong operator supply the two shared pairs. `wrong-sequence` has no campaign row or attribution consumer. `expected_class` and `expectation_reason` are copied from `DIAGNOSTIC_EXPECTED_CLASSES` at freeze time, shared claims must agree, and binding checks claim-set equality without duplicates. The class partition is 11 curve relations, seven invalid signatures, one empty signature, three unequal operands and three unnamed expectations. The re-signed semantic-metadata subject expects `InvalidCurveRelation` at successor reconstruction's `TweakVerify`; a class cannot distinguish all instruction sites sharing it.

Funding is retained before submission. Most subjects use the control's coin; wrong asset, wrong amount and wrong predecessor program use a separately retained coin with exactly their declared coordinate changed. The amount mutation is plus one; other unchanged coordinates must agree. Exact replay reconstructs the submitted transaction. A single live campaign policy binds both phases; historical phase replays keep their original start arguments and cannot turn two historical policies into one live capture.

The closed report grammar follows (`rec:backlog:wave0-native-route-capture-driver`) items 1 through 15 and 22 through 24, replacing the target, test, encoded Cargo argv and profile with the diagnostic names. It retains protocol revision 8, adapter version 2.1.0, one report ceremony, one selected test, zero setups, forward-v2 fixture digests, the clean suite identity, deployment, target revision, phase walls and elapsed wall. Its profile-specific fields, in order, are:

1. Encoded `diagnostic-policy`, with the retained policy's eight keys and node-default fee premises, without a fee override.
2. Encoded `diagnostic-families`, seven ordered records: window, metadata, operator, predecessor, successor, abi and wrong-semantic-metadata. Each has `family`, `control`, `control_accepted`, `stopped`, `expectation` and `subjects`. Each subject has `subject`, `claims`, `observation` and `expectation`; claims have `section` and `name`, and non-null observations have `step`, `layer`, `route` and `readback`. Claim sections belong to subjects, not families. The consensus-only subject remains in this schedule record.
3. Encoded `diagnostic-stopped`, an ordered subset of the seven families.
4. `campaign-manifest-schema 1`, then `campaign-manifest-count 23` and 23 encoded `campaign-row` records in schedule order. Each row has `subject`, `claims`, `family`, `operation`, `candidate`, `abi`, `target`, `deployment`, `branch`, `policy_requirement`, `policy`, `constructor`, `mutation`, `declared_boundary`, `expected_layer`, `expected_class`, `expectation_reason`, `control_id`, `compatibility`, `control`, `observation` and `expected_consumer`. Candidate, deployment, branch and bound submissions use the retained record's nested shapes; each bound submission carries all its claims. `control_id` names the family control independent of its run ordinal; the actual ordinal is retained in the bound control's `step` and checked by replay. The consumer is `diagnostic-attribution-ledger`.

The evidence payload begins `maturity-diagnostic-evidence-schema 1`, binds deployment and branch, then carries the policy, family and manifest records and every exact exchange, with any planner or manifest refusal retained. Capture and timing are written before outcome assertions. The shell decodes each exchange's observation detail and inspects its nested `detail`: every script-path refusal carries `block_script_error`; consensus and key-path refusals may carry it because the executor writes the member before classifying any direct-block refusal; acceptances and fundings do not. Rust reads the closed vocabulary. Layer agreement remains separate from expected-class comparison, so a class mismatch is a finding and never a capture-eligibility refusal.

The nominal census does not discard shorter stopped-family diagnostics: all 23 frozen rows remain, with absent observations explicit. Structural eligibility and replay consistency assign no row credit. Whole-archive admission at (`rec:backlog:wave0-diagnostic-route-admission`) refuses stopped, absent, contradicting or uncontrolled families and binds every observation to its own accepted control before producing the supplementary attribution ledger. Capture of record belongs under `evidence/wave0/diagnostic/`; its four files are capture, timing, MANIFEST.sha256 and RUN-REPORT. The manifest binds capture and timing, and RUN-REPORT binds that manifest. No archive pins were minted in the driver landing.

The retained archive's missing named registry family is closed by `native-retained-content-pins`, matching `MATURITY_RETAINED_(MANIFEST_SHA256|RUN_ADDRESS|CEREMONY_SHA256)` in `lint/hash-citation-families.tsv`; the diagnostic archive's `native-diagnostic-content-pins` row is registered with its admission at (`rec:backlog:wave0-diagnostic-route-admission`). The current census remains (`result:backlog:close-safety-reconciliation`).

**Record (Diagnostic maturity archive admission)** · `rec:backlog:wave0-diagnostic-route-admission`

The four immutable files in `packages/vectors/fixtures/maturity-diagnostic-run-of-record/` admit the diagnostic capture from `evidence/wave0/diagnostic/w17e1-diagnostic-20261102T103745Z/`, relative to the execution directory resolved by (`rec:backlog:wave0-entry`). Source and destination digests were equal at transfer; fixture admission preserves those bytes. Census, sizes, manifest, original report address, ceremony digest, selected target and exact test, Cargo argv, adapter and protocol, pinned node revision, deployment and timing bind before exact replay (`proto:wave0:native-admission`). The diagnostic profile has fourteen capabilities and four funding fields, matching the confidential advertisement; its Cargo path and node-default fee premises inherit the native pins. Relay and mempool floors are reported by the node; the block floor is declared by the pinned source without an override. All three are `0.00000100 BTC/kvB`.

| Artifact or content pin | Reading |
|---|---|
| `ae053ac43a.report.capture` | 2,321,183 bytes |
| `ae053ac43a.report.capture.timing` | 49 bytes |
| `MANIFEST.sha256` | 191 bytes; binds capture and timing |
| `RUN-REPORT` | 151,862 bytes; binds manifest, campaign and one report ceremony |
| `MATURITY_DIAGNOSTIC_CEREMONY_SHA256` | `f27b3f710e495cf8e193ff062ca7b5304f3f6236c9bfadda268d91ace7caffe2` |
| `MATURITY_DIAGNOSTIC_MANIFEST_SHA256` | `f041eb5d02b95f71829acbe966bf4eef4dbe9a738514f3cf09f10f545765faee` |
| `MATURITY_DIAGNOSTIC_RUN_ADDRESS` | `a7159fdc16c5a2a7bb8f9ce5011fe014377d481023e14e2acb714461a47a6cbc` |

The admitted census is 48 exchanges: 17 fundings, zero signings and 31 submissions, comprising seven accepted controls, 23 script-path refusals and one consensus refusal. All seven families agree at their declared layers, every control has exact mined readback, and the stopped list is empty. The 23 frozen campaign rows bind all 25 claims to their controls and subject observations. The unannounced-successor and wrong-operator subjects each carry two claims. There are 24 recorded diagnostic members: the 23 script-path refusals and the consensus-only successor subject `wrong-sequence` at step 35, whose recorded key is `unclassified` and which has no attribution-ledger entry.

| Family | Subjects | Accepted control step |
|---|---:|---|
| window | 2 | 4 |
| metadata | 1 | 8 |
| operator | 6 | 17 |
| predecessor | 4 | 27 |
| successor | 6 | 36 |
| abi | 4 | 43 |
| wrong-semantic-metadata | 1 | 3 |

`replay_diagnostic_routes` reconstructs the complete schedule, and `DiagnosticCampaignManifest::validate` rebuilds the frozen campaign and every funded binding. Whole-archive admission refuses any stopped, absent, contradicting or uncontrolled family. Structural capture eligibility alone supplies none of that admission. `maturity_diagnostic_run_of_record` retains the admitted result once. The `diagnostic-attribution-ledger` consumer derives 25 entries once through typed planner observations and `recorded_block_script_error`, preserving each raw recorded key, expected class and reason, control and subject step, and comparison. A test recomputes both directions. The ledger takes no Safety row and returns no standing.

The ledger has 24 agreements, one difference and zero absent claims. Eleven curve-relation claims, seven invalid-signature claims, one empty-signature claim and three unequal-operand claims agree by named class; two commitment claims agree by recorded `unclassified`. Historical absence remains distinct from that sentinel. A named/unnameable mismatch in either direction is a Difference, never an absent observation.

FINDING — `16.11 / witness-item-reorder`: the expectation remains `None` / `UnnameableFirstCheck`. The admitted observation records `Operation not valid with the current stack size`, read as `StackUnderflow`. The node's first refusing check reports a stack-size failure that the expectation could not name. The ledger records Difference beside the unchanged `NativeRefusalObserved` standing at the script-path boundary; no expectation, staged pin or census changes. Ruling 134 (`rule:phase7:wave0-rulings`) makes a different recorded class a finding beside the unchanged standing.

AGREEMENT — `16.5 / wrong-leaf-version` and `16.11 / control-block-from-another-program` each record the key `unclassified`, read as `Recorded(None)`, agreeing with `None` / `UnmappedCommitment`. These are recorded observations, not historical absence. `16.6 / successor-from-wrong-semantic-metadata` records `EC scalar mult verify fail` / `InvalidCurveRelation`, agreeing with its expectation. The class alone does not establish a unique instruction site.

Safety remains 206 / 177 / 29, categories (177, 27, 2, 0), typed final/waiting (27, 0), native acceptance/refusal 12 / 39, first-party discharged/required 81 / 0, supplementary [41, 2, 0] and staging [26, 4, 2, 3]. Every staged pin, historical archive pin and size, executor source pin and parsed Safety table row stays unchanged. The historical `NotRecorded` test is untouched; the diagnostic archive has its own compatibility load and recording assertions. The contract retains a `key-path-member` subject as `eligible yes`, and admission refuses its wrong-layer family (`diagnostic_families_refuse_each_uncontrolled_or_wrong_boundary_subject`).

The registry row `native-diagnostic-content-pins` matches `MATURITY_DIAGNOSTIC_(MANIFEST_SHA256|RUN_ADDRESS|CEREMONY_SHA256)` in `lint/hash-citation-families.tsv`; each named pin binds its own artifact under exact whole-archive admission. `packages/vectors/meson.build` registers all four fixture files. The recorded capture walls are preflight 41 ms, Cargo 160506 ms, census 127 ms and manifest 398 ms, with elapsed 161087 ms. Installed admission verification passed formatting and workspace Clippy; the whole vectors target passed 1407 tests with 52 ignored, the documentation, lint and integration suites passed 4, 13 and 13 tests, and labels passed 240, all with zero failures. The plans and hash-citation checks passed. The installed readings confirmed exact fixture bytes, unchanged historical and staged pins, unchanged parsed Safety rows, the supplementary ledger and the required record and contract registrations.

This admission completes (`step:wave0:bite-0-5-diagnostic-recapture`). Ruling 135(b) accepts the attribution, finding and remaining instruction-site limits for the native gate (`gate:guide15-exec:bite-0-5-native-gate`) (`rule:phase7:wave0-rulings`). No standing or observed-answer count moves. The broader conjunct, bite (`gate:guide15-exec:bite-0-5-gate`) and packet (`gate:wave0:bite-0-5-packet`) closure is recorded at (`result:backlog:bite-0-5-gate-pass`); ruling 134 is unchanged.

**Record (Confidential sponsor capture driver)** · `rec:backlog:wave0-confidential-route-capture-driver`

Balanced construction and the separate driver supplied the capture admitted at (`rec:backlog:wave0-confidential-route-admission`) under (`step:wave0:bite-0-5-confidential-sponsor-route`). Its evidence home is `evidence/wave0/confidential/w17a1-confidential-20261101T194031Z/`, relative to the entry evidence home (`rec:backlog:wave0-entry`); the four-file archive is `packages/vectors/fixtures/maturity-confidential-run-of-record/`.

The planner freezes one typed Safety claim, candidate source and ABI, deployment, branch, constructor, expected acceptance, control identity, compatibility and future final build-property consumer before issuance (`alg:wave0:campaign-manifest`). Fourteen successful exchanges are `issue`, `control-predecessor`, `fund-template-sponsor`, `sign-control-template`, `measure-control`, `fund-sponsor`, `sign-control`, `control`, `subject-predecessor`, `fund-confidential-sponsor-reserve`, `sign-confidential-template`, `measure-confidential`, `sign-confidential` and the confidential claim submission. Templates are never submitted. Subject funding and construction follow only relay-and-template control acceptance with exact readback (`pre:wave0:accepted-control`).

The checked compatibility argument keeps issued asset, predecessor program, singleton amount, metadata, request, reserve asset, sponsor program, candidate, ABI, operator, sponsor key identity and policy equal. Both reserve inputs carry 1250 units. Outpoints, transaction and witness identities, signatures, representation, measured fee and change differ. Each submission records its weight, fee, positive change and accepted readback when present. The historical million-unit S2 control retains weight 2809, fee 141 and change 999859.

The narrow reserve reader resolves the catalogued handle and digest, requires both derived openings and borrows the first primary coin's own blinder. Change nonce input and proof seed retain their tagged derivations. No opening, blinder, nonce input, proof seed or material object is serialized or formatted; only public commitments, output nonce commitments and proofs enter the candidate and records. The real adapters materialize and independently check commitments. Signed-template measurement uses twice the largest default floor over rounded-up virtual size, eight witness weight units and proof growth to the admitted maximum of 5134 bytes including framing. The final signed weight must fit the fee; checked subtraction must leave positive change. Failure reports a public fee-fit refusal, with no retry or invented amount. Re-amounting requires a new handle on both sides.

The ignored target `packages/vectors/tests/maturity_confidential.rs` selects `the_confidential_maturity_route_runs_against_a_real_target` through the unchanged shared executor. A supervised zero-operation handshake probe requires confidential funding, confidential sponsor authorization and selected profiles before the family; the real handshake is checked again. `scripts/live-native-maturity-confidential-capture.sh` selects profile `confidential` with one Cargo invocation, one test, one report ceremony and no setup artifact. Its header requires the explicit readable library setting, node-default policy, maturity observation, wallet and disk-backed temporary root. Library configuration, readability and digest are public readings without a pathname. The capture is written before assertions. `scripts/test-live-native-maturity-confidential-capture.sh` supplies the integration contract without a node or actual library.

Agreement is eligible. A refused or direct-block-only control stops the family, produces no subject and is ineligible. A refused subject after the qualified control is a contradiction: structurally eligible diagnostic evidence, but refused by whole-archive admission (`proto:wave0:native-admission`). A subject accepted only by direct block is likewise an eligible contradiction, with its readback retained and whole-archive admission refused. A readback mismatch retains the raw response and is ineligible; infrastructure failure is retained and ineligible. Admitted agreement supports the final build-property reason recorded at (`rec:backlog:wave0-confidential-route-admission`): the covenant reads no sponsor value commitment and the selected support holds under defaults. The fault row cannot take an accepted-transaction layer. Contradiction cannot justify this final reason; archive admission and the Safety retype land together. A refusal at any step after the qualified control reports its refusal coordinate, not a stopped family; expectation is pending until the subject outcome settles.

The report has twenty-five grouped items: twenty-three unchanged or renamed items, one new preflight item and terminal eligibility. Items 1–3, 7–12, 14–15, 19 and 22–23 retain the retained driver's grammar. Items 4–6 and 13 select the new target and profile; 16–18 and 20–21 carry one confidential family and committed funding. Item 24 is new; eligibility moves to item 25. Encoded fields use byte length and lower hexadecimal. The four files remain capture, timing, MANIFEST.sha256 and RUN-REPORT; diagnostics stay outside the manifest.

1. `run-report-schema native-maturity-run-report 1`.
2. `capture-format-schema native-maturity-capture 1`.
3. `suite-commit`, `suite-tree`, `suite-clean yes`.
4. `rust-test-target maturity_confidential`.
5. Encoded `rust-test-name` naming the confidential test.
6. Encoded `cargo-argv`: selected cargo, `test -p tripod-vectors --test maturity_confidential -- --ignored --exact the_confidential_maturity_route_runs_against_a_real_target --test-threads=1`.
7. Expected/observed test and ceremony counts one; setup counts zero.
8. `diagnostics-present yes`; passed/failed/ignored one/zero/zero; cargo exit zero.
9. Expected Elements tip, binary-reported revision, intended executed tip.
10. Adapter name/version and node name/version.
11. `protocol-revision 8`, `fixture-digest-algorithm forward-v2`.
12. Deployment environment, network, genesis and target contract.
13. `capture-profile confidential`.
14. Four `phase-wall-ms` lines: preflight, cargo, census, manifest.
15. Unsigned `elapsed-wall-ms`.
16. Encoded `confidential-policy`: relay/mempool/block/unit/block_identifier/relay_provenance/block_provenance/start_arguments.
17. Encoded `confidential-families`: one record with family/claim/control/subject/stopped/expectation/control_accepted/eligible/refusal; expectation is agreement, contradiction, stopped or pending.
18. Encoded `confidential-stopped`: empty or the sole family.
19. `campaign-manifest-schema 1`.
20. `campaign-manifest-count 1`.
21. One encoded `campaign-row`: claim/operation/family/candidate/abi/target/deployment/branch/policy_requirement/policy/constructor/mutation/declared_boundary/expected_layer/control_id/compatibility/control/observation/expected_consumer. Claim has section/name, candidate has schedule, deployment has network/genesis, branch has identifier/checkpoint, compatibility has kind/same/different. Submissions have step/digest/funded_outpoints/claims/fee/change/weight/readback; funding entries have funding_step/output. Explicit outputs retain protocol keys; committed outputs have outpoint/explicit_asset/value_commitment/nonce/script/output_witness_index/surjection_proof/rangeproof and no amount field. Readback preserves transaction and witness identities, block hash, height and raw bytes.
22. `ceremony-roster begin`, one `ceremony report` digest, `ceremony-roster end`.
23. `manifest-sha256` digest.
24. Encoded `confidential-preflight`: library_configured/library_readable/library_sha256/probe/capture; each handshake reading has confidential_value_test_funding/confidential_value_sponsor_authorization/advertisement_matches/profiles. Profiles have representation/custody/materializer/reproducibility_contract.
25. `eligible yes` or `eligible no`, then end of file; eligibility assigns no row credit.

At the confidential driver landing, twelve scripted scenario tests and fifteen wrapper-guard tests cover the settled labels, pending subject and eligibility, direct-block control and subject, campaign projection, readback mismatch, acceptance-boundary panic, changed requests/funding/controls at replay, material confinement and fee fit. Module tests pin proof-growth measurement and final-weight fit. Reader tests cover accepted material, changed digest, shape and missing openings. The shell contract retains framing guards and adds missing or unreadable library, either capability missing, advertisement and profile mismatch, committed-output grammar, family count, stopped control and mismatched preflight reading, capture-only reading mismatch and an existing mode-zero library file (skipped when readable despite its mode). No fixture or archive is added. Safety stayed 206 / 177 / 29, categories (177, 26, 2, 1), typed 27 = 26 final + one waiting, native 12 / 39, confidential membership one, layer-bound 179, reachable/unreachable 51 / 128 and reachable relation pairs/bound rows 1253 / 39. The finality and census tests pinned these values at that landing; the current census is recorded at (`result:backlog:close-safety-reconciliation`). At that landing the waiting reason and NoPublishedRelationNamesIt disposition remained unchanged. The later admission retypes only the boundary cell at (`rec:backlog:wave0-confidential-route-admission`).


**Record (Confidential sponsor admission)** · `rec:backlog:wave0-confidential-route-admission`

The four immutable files in `packages/vectors/fixtures/maturity-confidential-run-of-record/` admit the capture named by the entry record (`rec:backlog:wave0-entry`). Census, sizes, manifest, report address, ceremony digest, selected target and exact test, cargo argv, adapter and protocol, node revision, deployment and timing bind before exact replay (`proto:wave0:native-admission`). The closed family and campaign records bind to the replayed planner (`alg:wave0:campaign-manifest`); the closed preflight records bind the library digest as a well-formed reading and require equal probe and capture profiles, without pinning a self-built library identity. Structural eligibility alone never admits a contradiction, pending expectation or stopped family.

The fourteen exchanges contain six funding operations, six signing operations and two submissions. The explicit control at step 7 pays fee 141 and change 1109 from reserve 1250 and is mined at height 7; the confidential subject at step 13 pays fee 409 and change 841 from reserve 1250 and is mined at height 10. Both qualify through relay and ordinary template selection with exact readback (`pre:wave0:accepted-control`). Equivalent coins preserve issued asset, predecessor and sponsor programs, singleton amount, candidate, ABI and selected default policy; value representation is the intended difference, with separately recorded outpoints, identities, signatures, fee and change.

The sole confidential fault row is final under `ConfidentialSponsorValueUndecidedByCandidate`: the covenant reads no sponsor value commitment, and admitted agreement witnesses selected support under defaults. Its carrier and `NoPublishedRelationNamesIt` disposition are unchanged; no accepted-transaction layer or refusal answer is assigned. The former `ConfidentialSponsorAwaitsBalancedProofRoute` remains documented with zero members, and the new reason has one. The planner admits exactly those two boundary reasons so replay remains valid across admission. `confidential_acceptance_binds_final_build_property_standing` binds the final reason to the admitted report, agreement and step-13 readback.

Safety remains 206 / 177 / 29. Categories move (177, 26, 2, 1) → (177, 27, 2, 0), and typed final/waiting moves (26, 1) → (27, 0), with typed 27, native acceptance/refusal 12 / 39, first-party discharged/required 81 / 0, supplementary [41, 2, 0], layer-bound 179, reachable/unreachable 51 / 128 and relation pairs/bound rows 1253 / 39 unchanged. The sponsored ledger partitions fourteen rows as seven ABI finals, one confidential final, two acceptances, three matching refusals and one contradiction. The bounded consolidation reconstructed against the current table moves categories (141, 26, 2, 37) → (141, 27, 2, 36), while its 141 answered and 65 outstanding rows remain fixed; historical landing results retain their original figures.

The installed admission gate passed the formatter, vectors Clippy and gate-profile tests, plans validation, documentation and lint suites, labels tests and the fixture census. The four archive files retain their admitted sizes and digests, and the census and standing tests pass.

Close-safety reconciliation and the non-answer gate's re-read are recorded at (`result:backlog:close-safety-reconciliation`). Diagnostic recapture (`step:wave0:bite-0-5-diagnostic-recapture`) is complete at (`rec:backlog:wave0-diagnostic-route-admission`); the four bite 0.5 gates (`gate:guide15-exec:bite-0-5-non-answer-gate`), (`gate:guide15-exec:bite-0-5-native-gate`), (`gate:guide15-exec:bite-0-5-gate`), (`gate:wave0:bite-0-5-packet`) named here are satisfied at (`result:backlog:bite-0-5-gate-pass`). This admission completes the confidential route sub-step and supplies no broader packet closure.


### Conjunct dispositions · `sec:backlog:conjunct-dispositions`

**Table (Bounded conjunct dispositions by part)** · `tab:backlog:conjunct-dispositions`

The historical table at (`sec:phase6:gate-record`) remains NOT PASSED in its original scope and vocabulary. These nineteen parts, ten MET, nine RETIRED and zero WAITING, do not rewrite it. RETIRED is limited by (`def:guide15-exec:retired-residual`); the observable remainder is separate.

| Gate and part | Prior condition | Evidence | Standing | Residual or non-claim | Pin |
|---|---|---|---|---|---|
| (`gate:guide15-exec:bite-0-5-conjunct-5`) admitted escape refusals | Narrowed native conditions | Seven predecessor/successor mutant admissions at their declared boundaries; historical gate row 5 | MET | No secret-impossibility or unoffered-recipe claim | `each_admitted_mutant_refusal_answers_its_row_at_its_declared_boundary` |
| (`gate:guide15-exec:bite-0-5-conjunct-5`) NUMS derivation | Key-path residual | (`judg:guide15-exec:nums-standing`); exact public derivation check | MET | Conditional on both named hardness assumptions | `nums_is_recomputed_from_public_uncompressed_generator` |
| (`gate:guide15-exec:bite-0-5-conjunct-5`) exact announcement recipe coordinate | No successor recipe position in these bytes | Historical alternative reading and the actual announcement witness | RETIRED | No successor-recipe refusal was observed; announcement-byte coordinate alone | `no_announcement_byte_carries_the_successor_recipe` |
| (`gate:guide15-exec:bite-0-5-conjunct-5`) successor recipe construction | No operation on the candidate spends an announced state | Ruling 133 and (`rec:backlog:wave0-native-route-settlement`) | RETIRED | No operation on the candidate spends an announced state; no successor control position exists | `candidate_leaf_and_transition_censuses_have_no_announced_state_spend` |
| (`gate:guide15-exec:bite-0-5-conjunct-5`) static subtree comparison | Carried first-party case | Retained and changed constructors; continuity refuses `ConflictingLeaf` | MET | First-party continuity only; no native static-subtree refusal claimed | `supplementary_comparisons_retain_exact_objects_and_refusals` |
| (`gate:guide15-exec:bite-0-5-conjunct-7`) selected sponsored forms | Zero-floor inherited control | S1/S2 relay acceptance, assembly membership and ordinary mined readback at positive floors; (`tab:backlog:sponsored-observation-rows`) | MET | Exact fees, subjects and policy only | `sponsored_admission_binds_bytes_provenance_census_routes_and_readback` |
| (`gate:guide15-exec:bite-0-5-conjunct-7`) eight semantic forms | Acceptance-awaiting forms | Eight S2 positive-floor acceptances; (`tab:backlog:native-observation-rows`) | MET | The combined projection is observed over one admitted S2 transaction; (`rec:backlog:wave0-combined-projection`) | `native_observation_credits_only_exact_rows_and_never_acceptances_as_refusals` |
| (`gate:guide15-exec:bite-0-5-conjunct-7`) broader policy | Historical ruling 115 | Exact selected-form policy provenance retained | RETIRED | No production deployment is constructed or observed; the evidence binds only the declared deployment form on the pinned target; no sponsorless positive-floor relay claim because that form has no fee output; no whole-schedule relay claim because its 86-byte metadata item exceeds the 80-byte relay bound | `sponsored_acceptances_carry_the_admitted_pins_schedule_and_readback` |
| (`gate:guide15-exec:bite-0-5-conjunct-11`) independent exact edge | Archived checkpoint-bound edge | Independent node-interface acquisition, checkpoint and environment; (`tab:backlog:independent-observation-readings`) | MET | Equal final cursors alone are refused | `observer_archive_replays_reports_and_preserves_each_boundary`; `boundary_replay_checks_seven_outcomes_and_separate_report_roles` |
| (`gate:guide15-exec:bite-0-5-conjunct-11`) later cursor freshness | Checkpoint premise | Active-block reading binds the acquired prefix | RETIRED | No authentication of later cursor freshness | `boundary_replay_checks_seven_outcomes_and_separate_report_roles` |
| (`gate:guide15-exec:bite-0-5-conjunct-11`) archive origin | Admission premise | Exact immutable archive admission | RETIRED | No archive-origin guarantee beyond admission | `observer_archive_refuses_each_changed_file` |
| (`gate:guide15-exec:bite-0-5-conjunct-11`) broader chain and artifact claims | Single node and declared public projections | (`cav:wave0:observer-node-limit`) | RETIRED | No second chain view or full-artifact authentication | `observer_archive_replays_reports_and_preserves_each_boundary` |
| (`gate:guide15-exec:bite-0-5-conjunct-13`) public successor | Archived recovery handoff | Separate process reconstructs from acquired public data and compares the observed program; (`tab:backlog:independent-observation-readings`) | MET | Pinned import boundary; recovery role stays separate | `observer_archive_replays_reports_and_preserves_each_boundary` |
| (`gate:guide15-exec:bite-0-5-conjunct-13`) future availability | Bounded acquisition | Exact acquired subject and completed reconstruction | RETIRED | No archival or deployment availability guarantee | `boundary_replay_checks_seven_outcomes_and_separate_report_roles` |
| (`gate:guide15-exec:bite-0-5-conjunct-13`) origin and authentication | Public declarations and one node | Independent acquisition record and its retained residuals | RETIRED | No broader archive-origin, artifact-authentication, cursor-freshness or second-view claim | `observer_archive_replays_reports_and_preserves_each_boundary` |
| (`gate:guide15-exec:bite-0-5-conjunct-14`) composed model region | Whole-transaction scope | (`result:backlog:region-composition`); disjoint announcement and compact-ASH facts accepted, foreign object refused | MET | Structural evidence only | `announcement_beside_compact_ash_is_conformant`; `foreign_object_claimed_by_announcement_is_refused` |
| (`gate:guide15-exec:bite-0-5-conjunct-14`) exact sponsored region binding | Model/native join owed | Admitted S1/S2 bytes and funding bound to six region identities, with displaced and foreign membership subjects | MET | Membership, family, asset and erasure only; its owner clause rests on test-assigned owner and signer labels backed only by asserted target acceptance of the exact bytes; no new native observation | `observed_sponsored_compositions_bind_exact_operation_regions` |
| (`gate:guide15-exec:bite-0-5-conjunct-14`) carrier and expansion limits | Model-only multiplicity and four missing carriers | (`rec:backlog:wave0-region-refit`) | RETIRED | No leaf envelope-count enforcement, unsupported operation expansion or combined state-machine claim | `the_whole_transaction_scoped_rows_are_exactly_the_recounted_set` |
| (`gate:guide15-exec:bite-0-5-conjunct-14`) ABI classification and roles | Observable entry routes missing | Fifteen classification comparisons and six role comparisons, with exact subjects and accepted controls at (`tab:backlog:abi-entry-comparisons`) | MET | Claim independence, uncalibrated sponsor maximum and presence residual remain at (`cav:backlog:abi-entry-limits`); no construction admission or target observation | `abi_entry_pairs_cover_twenty_three_rows_and_twenty_nine_subjects`; `retained_obligations_have_valid_subjects_and_named_boundaries` |

### Safety closure ledger · `sec:backlog:safety-closure-ledger`

**Table (Supplementary Safety standing movements)** · `tab:backlog:safety-closure-ledger`

Only these seven keys move. The other 199 keep their complete standing and evidence identity, including all 78 historical answers. Every row is pinned by `supplementary_dispositions_join_required_keys_without_promoting_historical_answers`; comparison and normalization pins below retain the exact objects.

| Key | Old standing or reason | Admitted evidence | New category | Pin |
|---|---|---|---|---|
| `16.5 / wrong-static-subtree` | FirstPartyRequired; owner has no retained object to compare the offer against | Retained versus changed static root; continuity `ConflictingLeaf` | observed | `supplementary_comparisons_retain_exact_objects_and_refusals` |
| `16.5 / extra-escape-leaf` | FirstPartyRequired; owner has no retained object to compare the offer against | Retained root versus extra-leaf root; continuity `ConflictingLeaf` | observed | `supplementary_comparisons_retain_exact_objects_and_refusals` |
| `16.7 / caller-supplied-branch-order` | FirstPartyRequired; offered order normalized | Actual reversed children, equal roots, leaf hashes and paths | retired | `absence_of_coordinate_is_only_unchanged_content_ordering` |
| `16.7 / source-order-dependent-tree` | FirstPartyRequired; offered order normalized | Reordered declarations, equal canonical assembly and paths | retired | `absence_of_coordinate_is_only_unchanged_content_ordering` |
| `16.11 / unresolved-metadata-schema-symbol` | FirstPartyRequired; census resolved by its owning entry | Complete versus MetadataSchema-omitted census; `MissingDefinition(MetadataSchema)` | observed | `supplementary_comparisons_retain_exact_objects_and_refusals` |
| `16.11 / unresolved-lead-bound-symbol` | FirstPartyRequired; census resolved by its owning entry | Complete versus MaturityLeadMin-omitted census; `MissingDefinition(MaturityLeadMin)` | observed | `supplementary_comparisons_retain_exact_objects_and_refusals` |
| `16.11 / unresolved-operator-symbol` | FirstPartyRequired; census resolved by its owning entry | Complete versus CommittedOperatorKey-omitted census; `MissingDefinition(CommittedOperatorKey)` | observed | `supplementary_comparisons_retain_exact_objects_and_refusals` |

### Adapter request comparisons · `sec:backlog:adapter-comparisons`

**Record (Exact Python request comparisons)** · `rec:backlog:wave0-adapter-comparisons`

The five request-entry comparisons complete (`step:wave0:bite-0-5-first-party-entries`). The Python check executes the adapter source bytes it reads and hashes, drives both operands through the same entry over `io.BytesIO`, and compares its five rendered readings byte for byte with `packages/vectors/fixtures/adapter-request-comparisons/COMPARISONS`. Its default mode refuses any differing line; `--emit` produces the fixture for deliberate re-admission. The Rust admission embeds that fixture, constructs the same operands independently, and checks every identity, length, digest, boundary, entry, outcome and refusal before constructing a retained comparison. Library derivation reads no filesystem and starts no interpreter or subprocess. A source-file test independently hashes the adapter through the manifest directory. Returned readings are data validated against the Rust-constructed operands; a copied success flag supplies no comparison.

| Identity field | Admitted reading |
|---|---|
| Adapter | `elements-native-executor` |
| Adapter version / protocol revision | `2.1.1` / `8` |
| Adapter source SHA-256 | `e60999d8427b129b6caf82e2fefae553ec6ceb321327de726f08fc831bedc596` |
| Interpreter version | `3.14.10`; carried as a reading, not a Rust identity pin |

Every adapter source change requires deliberate re-pinning of `ADAPTER_SOURCE_SHA256` and the Python source pin, followed by a fresh `--emit` and both admission checks. An interpreter reading change also requires refreshing the fixture after a successful comparison; no interpreter patch version is an operand-validity premise. The source identity belongs to this admission and does not rewrite historical capture provenance.

The five comparisons are re-admitted on the node of record's gate interpreter under this record's interpreter rule. Adapter `2.1.1` sends every node-client RPC argument through `-stdin`, one argument per line, instead of placing transaction hex on the process command line; the offline executor boundary suite checks intact delivery of a 512 KiB argument, empty arguments and trailing spaces, and refusal of embedded line feeds before launch. Protocol revision `8` and the request/response schema are unchanged; historical captures retain their original executable provenance.

All reader calls use maximum 4194304 and phase `request`. Bytes below are ASCII with a final line-feed where stated. The field comparison decodes each record with `RequestReader.read_object` and then uses the allowed tuple `schema`, `case`, `subject`. These controls establish acceptance at their named entries, not transaction execution or acceptance by later request semantics.

| Key | Accepted bytes | One changed coordinate | Entry and exact refusal | Boundary |
|---|---|---|---|---|
| `16.12 / blank-request` | Empty JSON object followed by line-feed: `{}\n` | Replace the object body with two spaces, retaining line-feed | `RequestReader.read`; `FRAMING_BLANK_RECORD` gives `blank_record` | RequestFraming |
| `16.12 / oversized-request` | `{}\n` | Insert 4194303 spaces between the object and line-feed; the first 4194305 bytes contain no newline | `RequestReader.read`; `FRAMING_OVERSIZED_RECORD` gives `oversized_record` | RequestFraming |
| `16.12 / unterminated-request` | `{}\n` | Remove only the final line-feed, leaving `{}` | `RequestReader.read`; `FRAMING_UNTERMINATED_RECORD` gives `unterminated_record` | RequestFraming |
| `16.12 / malformed-json` | `{}\n` | Replace the closing object delimiter with the closing array delimiter: `{]\n` | `RequestReader.read_object`; `FRAMING_MALFORMED_RECORD` gives `malformed_record` | RequestJson |
| `16.12 / unknown-request-field` | `{"schema":8,"case":{},"subject":{}}\n` | Append the object field `"unknown":0`, producing `{"schema":8,"case":{},"subject":{},"unknown":0}\n` | `require_execution_request_fields`; `EXECUTION_REQUEST_FIELD_CENSUS_FAILED` | RequestFields |

The three request-side boundaries refine the existing `SemanticRequestRejection` layer. All five matrix declarations, carriers, prior reasons and unlinked relation dispositions remain unchanged. `MaturityRetainedComparison::PythonRequest` retains the language, entry, request bound or allowed-field tuple, source pin and validated interpreter reading beside both operands; the oversized operand is constructed and hashed, then retained by length and digest. The fixture stores lengths and digests rather than its large padding. `adapter_request_comparisons_bind_exact_pairs`, `adapter_admission_refuses_each_changed_reading` and `adapter_pairs_refuse_copied_reversed_and_misbound_operands` reject copied or reversed operands, changed bindings, invalid controls, accepted subjects, wrong refusals, missing, duplicate or surplus lines, malformed encoding and altered digests. Admission failure retains the attempted typed failure rather than discharging a row. `adapter_source_matches_the_admitted_digest` binds the source independently.

The fixture grammar is exactly five lines in the table's order: `comparison E(key) boundary E(entry) accepted refused E(refusal) control_length control_sha256 subject_length subject_sha256 E(adapter_name) E(adapter_version) protocol source_sha256 E(interpreter_version)`. Each `E(text)` is two space-separated fields: canonical decimal UTF-8 byte length and lowercase hexadecimal bytes. Lengths have no redundant leading zero; digests are 64 lowercase hexadecimal digits. Boundary and outcome words are closed tokens. Every line ends in one line-feed; carriage returns, missing final line-feed, extra fields and surplus bytes refuse. The key includes `16.12 / ` and the exact row name. This grammar is separate from the unchanged Safety report schema 3.

The existing `executor-boundaries` suite poses framing faults in the handshake phase under the 64 KiB handshake bound and routes its field fault through the handshake census. It corroborates neither the 4 MiB request bound nor `require_execution_request_fields`. The new `executor-request-comparisons` lint test supplies exact-entry evidence and fixture equality without a node daemon. Neither changes an adapter validator or the process-level suite. Rust response-framing mirrors read responses and supply no request-side comparison.

At the adapter admission landing, the Safety census was 206 / 177 / 29, with first-party discharged / required 81 / 0 and categories observed 177 / final 26 / retired 2 / waiting 1. The supplementary partition was 43 = 41 comparisons + 2 typed out-of-contract dispositions + 0 retained, or [41, 2, 0]. Against the preceding admission, answered rose by five from 172, outstanding fell by five from 34, and first-party discharged rose by five from 76. Waiting fell from six to the one confidential observation. The five comparisons added observed answers; they added neither native observations nor final reasons. Typed non-answers remained 27 = 26 final + 1 waiting, native acceptance/refusal 12 / 39, report required/observed 0 / 15, relation links 14 / 108 / 84, both staging partitions and report schema 3 remained unchanged. Completeness remains partial. The subsequent confidential admission moves final/waiting to 27 / 0 at (`rec:backlog:wave0-confidential-route-admission`).

`entry_movements_preserve_every_other_row_and_historical_answer` checks 36 cumulative movements and 170 unchanged rows, preserving the 78 historical answers and original evidence. The original eight-entry and subsequent twenty-three ABI-entry movements remain disjoint from these five. The fourteen-entry table now records fourteen observed rows, including the nonce comparison supplied by (`rec:backlog:wave0-abi-entries`). The ABI index keeps 28 keys and now records 23 `AbiFieldRefused`, 5 `AdapterRequestRefused` and 0 unasked readings over 34 changed subjects. The initial 40 discharges, original seven-movement reconstruction and two retirements retain their separate historical pins.

The five Python request comparisons retain both operands and their exact checked entry relations; source-bound readings admit only an accepted control and the named refusal of its changed subject. Together with the existing comparisons and typed out-of-contract proofs, this supplies every inherited row's obligation under (`gate:guide15-exec:bite-0-5-first-party-gate`): copied success flags cannot answer a row. Every considered existing-entry row has an admitted exact comparison; the nonce comparison is recorded by the ABI-entries step and adapter admission is supplied here. The confidential selected-policy observation is admitted at (`rec:backlog:wave0-confidential-route-admission`); detailed diagnostic recapture is complete at (`rec:backlog:wave0-diagnostic-route-admission`), while close-safety reconciliation and non-answer gate re-reading are recorded at (`result:backlog:close-safety-reconciliation`); `T12-005` and the broader packet close at (`result:backlog:bite-0-5-gate-pass`).

### First-party entries · `sec:backlog:first-party-entries`

**Record (Wave 0 existing first-party entries)** · `rec:backlog:wave0-first-party-entries`

The original bounded admission under (`step:wave0:bite-0-5-first-party-entries`) began with 36 close-safety rows across fourteen entry-point groups: 22 rows named absent entries (one request-cycle decoder, fifteen ABI-classification and six ABI-role rows), and fourteen named existing entries. Schema and environment shared one entry. That step added eight exact comparisons to the supplementary ledger and left six considered rows required. Following (`rec:backlog:wave0-abi-entries`), nine of those fourteen keys were observed and five remained required. Adapter admission at (`rec:backlog:wave0-adapter-comparisons`) then supplied the remaining comparisons and completed this step, while (`gate:wave0:bite-0-5-packet`) remained open.

`MaturityEntryComparisonTestStaging` retains exact accepted and changed offers beside each typed refusal. The same entry accepts the control and refuses the subject. `entry_pairs_refuse_copied_objects_invalid_controls_and_wrong_classes` refuses copied operands, reversed control/subject pairs and a refusal attributed to a different change; the weighted case also refuses credit for an accepted change of a unique weight. The complete-program subject is derived by committing the retained static root without its metadata branch, not by corrupting arbitrary program bytes. The depth pair retains one complete tree under two positive caps. Scripted provenance transcripts use independent expected topics and are both reconstructed through `run_protocol`; they supply no native observation.

**Table (Existing first-party entry movements)** · `tab:backlog:first-party-entry-movements`

All fourteen considered keys are retained below. The original eight movements change only first-party-required standings; the subsequent nonce comparison is the ninth observed row in this table. The five adapter rows now have admitted exact refusals at (`rec:backlog:wave0-adapter-comparisons`), making all fourteen considered keys observed.

| Section and row name | Previous standing | Exact entry | Control and changed coordinate | Typed refusal | New standing or category | Pin |
|---|---|---|---|---|---|---|
| `16.2 / request-cycle-differs-from-successor` | FirstPartyRequired | `OperatorAuthorizedMaturityAnnouncement::check_offered` | Authorized request-derived successor; only its offered announced cycle changes | `MaturitySuccessorMetadataChangedAfterSigning` | observed | `entry_pair_successor_cycle` |
| `16.3 / correct-semantic-metadata-with-noncanonical-representation-nonce` | FirstPartyRequired | `CandidateStateConstructor::compare_offered_representation` | Retained semantic instance at its selected nonce versus its next admitted nonce inside the default budget | `NoncanonicalRepresentationNonce` | observed | `entry_pair_nonce_comparison` |
| `16.5 / metadata-leaf-missing` | FirstPartyRequired | `PublicMaturityStateView::validate` | Complete committed program; offered program commits the same static subtree without the metadata branch | `MaturityViewProgramNotReconstructed` | observed | `entry_pair_metadata_omission` |
| `16.11 / candidate-outside-bounds` | FirstPartyRequired | `StateLinkedTaptree::bind` | Retained static subtree plus one support leaf, using the extra-escape-leaf comparison's constructor; lower only the accepting cap to the positive static depth | `StateTreeDepthExceeded` | observed | `entry_pair_depth_cap` |
| `16.11 / conflicting-leaf-weight` | FirstPartyRequired | `TaptreeInput<StateLeafRole>::new` | Unique weighted declaration; add the same role with a conflicting weight | `DuplicateStateTreeLeaf(Announcement)` | observed | `entry_pair_conflicting_weight` |
| `16.12 / blank-request` | FirstPartyRequired | `RequestReader.read` | Nonempty object record versus two spaces, with newline retained | `blank_record` | observed | `adapter_request_comparisons_bind_exact_pairs` |
| `16.12 / malformed-json` | FirstPartyRequired | `RequestReader.read_object` | Object record versus replacement of its closing delimiter by a closing array delimiter | `malformed_record` | observed | `adapter_request_comparisons_bind_exact_pairs` |
| `16.12 / oversized-request` | FirstPartyRequired | `RequestReader.read` | Object record versus the same object with 4194303 trailing spaces and final newline | `oversized_record` | observed | `adapter_request_comparisons_bind_exact_pairs` |
| `16.12 / signing-response-bound-to-other-bytes` | FirstPartyRequired | `bind_script_path_response` | Shape-valid accepted signing response; change one byte of signature_bound_to alone | `ScriptPathTransactionMismatch` | observed | `entry_pair_signature_binding` |
| `16.12 / unknown-request-field` | FirstPartyRequired | `require_execution_request_fields` | Declared schema, case and subject fields versus one additional unknown field | `EXECUTION_REQUEST_FIELD_CENSUS_FAILED` | observed | `adapter_request_comparisons_bind_exact_pairs` |
| `16.12 / unterminated-request` | FirstPartyRequired | `RequestReader.read` | Object record versus the same bytes without the final newline | `unterminated_record` | observed | `adapter_request_comparisons_bind_exact_pairs` |
| `16.12 / wrong-environment` | FirstPartyRequired | `compare_environment` | Matching environment observation; change only network identifier | `EnvironmentBindingMismatch` | observed | `entry_pair_environment` |
| `16.12 / wrong-provenance` | FirstPartyRequired | `ExecutionTranscript::provenance_agreement` | Matching independently stated expectation; add one reported topic, reconstructing both transcripts through run_protocol | `TopicCensusIsNotTheExpectedCensus` | observed | `entry_pair_provenance` |
| `16.12 / wrong-schema` | FirstPartyRequired | `compare_environment` | Matching environment observation; change only schema | `UnsupportedProtocolSchema` | observed | `entry_pair_schema` |

**Result (Eight entry comparisons and retained partitions)** · `result:backlog:first-party-entries`

`closure_categories_partition_without_turning_finality_into_observation` and `closure_category_publication_is_exact_and_refuses_each_changed_count` pin the current categories and their exact publication at (`result:backlog:close-safety-reconciliation`). The eight-entry result below remains historical. The depth comparison binds the retained static subtree plus one support leaf, using the extra-escape-leaf comparison's constructor so that the lowered cap remains positive.

The historical entry movement was n = 8. Supplementary comparison / retired / retained moves 5 / 2 / 36 → 13 / 2 / 28; first-party discharged / required moves 45 / 36 → 53 / 28. The census moves 206 / 139 / 67 → 206 / 147 / 59. Categories move observed 139 / final 22 / retired 2 / waiting 43 → 147 / 22 / 2 / 35, summing to 206; outstanding 59 = 22 final + 2 retired + 35 waiting. Waiting is 28 first-party obligations plus seven typed waits. Only performed first-party comparisons add answers; neither retirement nor a final non-answer adds one. Completeness remains partial.

`entry_movements_preserve_every_other_row_and_historical_answer` compares the bounded consolidation with the current plan in both directions, pins the disjoint union of eight original, twenty-three ABI-entry and five adapter movements with 170 unchanged rows, and preserves the 78 historical answers and existing first-party evidence by identity. The original entry step changed eight keys with 198 unchanged rows. `supplementary_dispositions_join_required_keys_without_promoting_historical_answers` still reconstructs the original seven movements against the original closure ledger. Each ledger parser stops at its own section boundary. `first_party_entry_record_matches_all_fourteen_considered_keys` checks this table against the typed ledger: its intersection with the cumulative movements contains all fourteen keys, including the nonce and five adapter comparisons.

At the eight-entry admission, typed non-answers remained 29 (22 final / 7 waiting), native acceptance / refusal 11 / 38, report required / observed 0 / 15, root history / public recovery 16 / 14, and relations 14 / 108 / 84. That admission left the Safety report grammar and native protocol revision unchanged, changed no capture or fixture, and added no target acceptance or native refusal. The subsequent adapter fixture is recorded at (`rec:backlog:wave0-adapter-comparisons`) and the current Safety census at (`result:backlog:close-safety-reconciliation`).

**Caveat (Six considered rows and their remaining routes)**

The nonce row was retained by the original entry step because its search accepted no offered nonce; the subsequent typed canonical-offer comparison now answers it beside the request-cycle decoder and twenty-one ABI rows. The five adapter rows subsequently receive exact Python comparisons and validated cross-language admission at (`rec:backlog:wave0-adapter-comparisons`) under (`step:wave0:bite-0-5-first-party-entries`). The adapter's documented Rust framing mirrors read responses; their refusals do not answer request-side rows. No retained row is retired or credited through another layer.

**Caveat (Guide package authority for the typed entry routes)**

The guide's files setting (`setup:guide15-exec:bite-0-5-files-and-packages`) now names the transaction ABI and request-decoder routes, the vectors three-way index and the tapscript nonce comparison, each with its tests. The entries have landed; (`rec:backlog:wave0-abi-entries`) records their performed comparisons. The historical eight-entry arithmetic above remains the result of that earlier step, not the current census.

**Caveat (Public comparison entries and unchanged validation)** · `cav:backlog:first-party-entry-visibility`

In `packages/target-elements-conformance/src/executor.rs`, `run_protocol` and `compare_environment` change from crate-private to public and `bind_script_path_response` changes from private to public, with documentation stating their scope. Their validation bodies are unchanged. No raw transcript constructor becomes public. These are production API changes allowing exact first-party comparisons. `run_protocol` retains the trust declared by the caller's configuration; scripted streams do not force Mock trust. These entry pairs explicitly pass Mock trust, and their scripted exchanges establish no native execution, authentication or deployment guarantee. Native constructions and combined projection subsequently received their dispositions at (`rec:backlog:wave0-native-route-capture-driver`) and (`rec:backlog:wave0-combined-projection`); adapter comparison admission follows at (`rec:backlog:wave0-adapter-comparisons`). Before the bite 0.5 gate pass, balanced confidential observation was admitted at (`rec:backlog:wave0-confidential-route-admission`) and diagnostic recapture was complete at (`rec:backlog:wave0-diagnostic-route-admission`), while `T12-005` stayed TODO; its closure is recorded at (`result:backlog:bite-0-5-gate-pass`).

### Test performance · `sec:backlog:test-performance`

**Record (Wave 0 test performance)** · `rec:backlog:wave0-test-performance`

The baseline at `0.7.20-dev`, scheduling change and comparable warm result satisfy the three steps of bite 0.6 (`step:wave0:bite-0-6-baseline-measurement`); (`step:wave0:bite-0-6-rule-and-land-remedy`); (`step:wave0:bite-0-6-measure-and-close`). Ruling 129 adds priority 1 to exactly `executor-boundaries`, `live-native-v2-r7-capture-contract` and `live-native-v2-r8-capture-contract` in root `meson.build`, preserving all 56 checks, arguments, dependencies, timeouts, profiles, failure propagation, timing attribution and the final clean-tree check (`rule:phase7:wave0-rulings`).

The executor check creates fresh temporary framework, executable and diagnostic fixtures. Each capture check uses its own temporary synthetic repository, mock Cargo and output directory. None shares package JSON outputs or the Cargo target with a package check or serial Cargo check. Imported Python bytecode is an incidental ignored cache, not fixture state. Serial Cargo checks retain their ordering; the signing fixture and its 2.0 s stand-in lifetime remain unchanged.

**Result (Before/after pair)** · `result:backlog:test-performance`

The retained baseline under `evidence/wave0/test-performance/w16d1-measure-20261025T202808Z/` records 1662.2 s cold and 891.8 s warm, then 689.3 s isolated debug and 77.9 s isolated release. The after sequence under `evidence/wave0/test-performance/w16d1-measure-20261025T220347Z/` records 731.2 s warm, 658.4 s isolated debug and 66.3 s isolated release: a 160.6 s (18.0%) lower comparable warm wall. Both warm observations reuse compiled artifacts on sojka in the same target directory; the after target is not cleared. Cold after is unavailable. Each full run passes the same 56 checks; each isolated suite passes 16. The retained `analysis.log` reconciles every package's test identity, binary ordinal and outcome against the baseline, with no new failure or skip and no completed event lacking timing (`tab:test-performance:register`).

Executed and ignored counts below are equal before and after in each profile, including isolated suites; totals include unit, integration and documentation tests. All suite summaries reconcile with the event streams.

| Package | Executed per profile | Ignored per profile |
|---|---:|---:|
| architecture | 166 | 0 |
| artifacts | 12 | 0 |
| cli-common | 76 | 0 |
| compiler | 812 | 3 |
| document-stamps | 45 | 0 |
| execwrap | 52 | 0 |
| flatten-latex-main | 41 | 0 |
| labels | 240 | 0 |
| linker | 258 | 0 |
| model | 360 | 0 |
| realization | 363 | 0 |
| tapscript | 634 | 0 |
| target-elements | 256 | 0 |
| target-elements-conformance | 679 | 0 |
| transaction | 471 | 0 |
| vectors | 1114 | 49 |
| Total | 5579 | 52 |

Achieved full-gate parallelism rises from 2.428903 to 3.111391 overlapping check-equivalents. Each moved check overlaps all 32 package checks; clean-tree remains last. After warm debug vectors, conformance and compiler leave residuals of 2.829079 s, 2.690823 s and 2.067020 s after subtracting successful sequential suite walls, not overlapping individual-test sums. These residuals combine unseparated work and waiting.

The initial after observation under `evidence/wave0/test-performance/w16d1-measure-20261025T213511Z/` takes 933.8 s, then 698.8 s debug and 68.3 s release. It compiles vectors because tests embed the edited backlog; it is retained as incremental compilation and excluded from the warm pair. Repeating the sequence without tracked changes or target clearing supplies the compilation-free pair. Final record edits change no selected embedded safety or compact disposition table; the measured test subject remains unchanged.

Holding durations and other scheduling offsets fixed, moving only the three checks removes 155.768517 s of exposed work and models a 736.031483 s warm wall. The model retains 12.680301 s after documentation and 1.502826 s after attestation for checks that do not move. Its longest moved check is 138.698715 s within the observed 688.835804 s initial phase. This is a model, not a measured after result; additional contention can change those durations.

**Caveat (Performance scope and unavailable dimensions)**

A wall is not calibration; one comparable pair does not estimate variance or assign contention (`cav:guide15-exec:performance-scope`). The node exposes 61 available processing units on a host with 128 physical cores and 256 logical CPUs. The retained loads accompany each command. Separate Cargo-lock durations, doctest compilation, fixture waiting, scheduler delay, CPU work and individual shell or Python case durations are unavailable. The mocked gate does not establish real document rendering or byte reproducibility.

The baseline isolated debug wall of 689.3 s exceeds the historical 480 s tolerance on a different reference host; executor boundaries occupies 138.698715 s against its stale 20 s tolerance. Both remain within their actual timeouts. Budget comments remain unchanged. The excluded attempt consumed 69 minutes, including unnecessary target-directory checking; environment initialization selected the wrong target. Its observations remain explicitly excluded under `evidence/wave0/test-performance/w16d1-measure-20261025T192056Z/`, and cannot supply the accepted pair.

**Record (Wave 0 compact inherited closure)** · `rec:backlog:wave0-compact-inherited-closure`

Compact inherited closure under rulings 136–139 comprises 74 = 27 native observed + 21 native formal + 18 first-party observed + 2 first-party formal + 6 report observed, or 51 observed and 23 formal, with zero retained or unresolved requirements (`rule:phase7:wave0-rulings`); (`gate:guide15-exec:bite-0-4-gate`). The eighteen first-party observations include two Clear declaration observations; no total claims 74 observed refusals (`rec:guide15-exec:bite-0-4-closure-carrier`).

First-party standing is 20 closed / 0 retained, comprising 18 observed + 2 formal, before 2 discharged / 18 retained; external standing is 6 observed / 0 retained, before 0 observed / 6 retained; missing entries are 0, before 24 = 18 first-party + 6 external. Earlier before-figures remain 18 first-party = 2 discharged + 16 retained and 22 = 16 + 6 missing entries. The manifest's supplied-entry mapping establishes missing-entry standing, the first-party disposition comparisons establish 20 / 0, and explicit ledger admission joins all 26 nonnative keys exactly once.

The unchanged figures are 74 negatives, partition 48 / 20 / 6, 209 = 135 positive + 74 negative requirements, two derivation-only discharges, native roster 48, joined native census [27, 21, 0, 0], compact record reader 48 and Safety record spans 43 / 42 / 7 / 14. Earlier population figures remain 72 / 48 / 18 / 6, the intermediate 50 / 18 / 6 partition and 213 = 139 + 74. Explicit closure admission adds no discharge to fresh derivation and changes no archive-admission predicate.

The two unavailable-witness identities are formally closed under ruling 137 because Compact's permissionless authorization selects `PermissionlessPrivateDependency`, making `ConstructibilityWitnessUnavailable` unreachable under that authorization; the supporting observed refusal receives no credit as the requested class. The two emitted-Clear identities observe acceptance of the outstanding declaration and refusal of its removal, with no emitted Clear path claimed. Clear's realization landing in Wave 2 reopens those dispositions and evidence compatibility under ruling 138 and (`cond:wave0:baseline-freeze`).

The ledger and exact-key, pair, restoration and disposition tests reside in `packages/vectors/src/first_party.rs`, including `stage_compact_closure_readings`, `admit_compact_closure_readings`, `compact_closure_joins_all_twenty_six_keys_and_the_native_forty_eight` and `compact_first_party_dispositions_retain_all_twenty_subjects`. The supplied-entry mapping and missing-entry pins reside in `packages/vectors/src/compact_negative_manifest.rs`; report controls, three refusal pairs for both sponsor identities and the six-observation pin reside in `packages/vectors/src/report.rs`. Native evidence remains in `packages/vectors/fixtures/compact-run-of-record/` and `packages/vectors/fixtures/compact-run-of-record-2/`, joined and record-bound in `packages/vectors/src/compact_corpus.rs`.

The compiler comparisons reside in `packages/compiler/src/constructibility.rs`, `operation_plan.rs`, `lifecycle.rs` and `sponsor_region.rs`, with focused tests in `packages/compiler/src/tests/compact_validation_tests.rs`. The emitted comparisons reside in `packages/tapscript/src/pattern.rs` and `bundle.rs`, with focused tests in `packages/tapscript/src/tests/compact_validation_tests.rs`. The three guide requests retain the derived manifest, exact first-party comparisons and formal arguments, and the typed ABI entry in `packages/transaction/src/abi.rs` with comparisons in `packages/vectors/src/abi_validation.rs`; ABI's two classes and four sponsor-case identities remain outside the compiler-derived denominator.

Sixteen of the twenty-six nonnative keys share sponsor-identical pairs because the landed entries are sponsor-agnostic: dependency, unavailable witness, representation, both compiler lifecycle exits, multiplicity, recognition and emitted Clear. Separate requirement identities do not establish sponsor-specific evidence. The neighbouring emitted member omission returns `ConstructorStructureMismatch` at the tapscript entry; the vectors neighbouring-omission test instead establishes `PairMismatch` at admission. The dependency disposition route is selected by the `PermissionlessPrivateDependency` mutation rather than the manifest owner; the two selections currently agree, and the two `Discharged` rows remain pinned.

Each landing's full gate of record runs at import on the imported candidate; a red candidate does not land. This record's own full gate is established the same way, so its DONE marking takes effect through the landing's gate. Verification comprises the plans check and the documentation, lint and label suites, with no real-document rendering claim beyond those suites.

| Landing | Package or documentation verification | Full gate of record and vectors target wall |
|---|---|---|
| Rulings and documents, `0.7.53-dev` | Plans check, documentation, lint and label suites green | Ok 63 / Fail 0; wall 828 s; vectors gate 417.8 s / release 511.3 s |
| Compiler comparisons, `0.7.54-dev` | Compiler 824 passed; vectors 1,407 passed; twelve focused tests | Ok 63 / Fail 0; wall 1,258 s; vectors gate 436.9 s / release 760.3 s; before: 417.8 s / 511.3 s |
| Tapscript comparisons, `0.7.55-dev` | Tapscript 646 passed; compiler 824 passed; vectors 1,407 passed; seven focused tests | Ok 63 / Fail 0; wall 1,278 s; vectors gate 513.4 s / release 799.9 s; before: 436.9 s / 760.3 s |
| Report comparisons, `0.7.56-dev` | Vectors 1,419 passed; compiler 824 passed; tapscript 646 passed; twelve tests | Ok 63 / Fail 0; wall 1,093 s; vectors gate 479.0 s / release 694.3 s; before: 513.4 s / 799.9 s |
| Closure ledger, `0.7.57-dev` | Vectors 1,432 passed; compiler 824 passed; tapscript 646 passed; thirteen tests; vectors package wall 389.5 s, before 472.3 s and 398.2 s | Ok 63 / Fail 0; wall 1,042 s; vectors gate 467.1 s / release 686.3 s; before: 479.0 s / 694.3 s |
| Closure record | Plans check and documentation, lint and label suites | Established at import as this landing's full gate of record |

This documentation landing adds one backlog record label and no source, dependency, schema, identity, migration, emission or acceptance change. The affected package contracts are (`pkg:compiler:contract`), (`pkg:tapscript:contract`) and (`pkg:vectors:contract`); across the five landings no dependency, schema, identity or migration changed, the only build-description changes being one test registration line in each of the compiler and tapscript Meson files; each Ok 63 / Fail 0 result includes the final clean-tree check. The label and generated-publication checks, final formatting check and final clean-tree output are retained with the closure-record gate evidence.

Recorded gate results establish the bounded closure of (`T4-009`). (`T12-004`) retains its completed native-only scope. (`T12-007`) remains TODO for consolidation, preflight and the full clean-tree Wave-0 baseline; this record does not complete that obligation.

**Record (Wave 0 compact negative manifest)** · `rec:backlog:wave0-compact-negative-manifest`

The manifest derives every negative key, relation, mutation and collateral obligation from the validated compact plan. Native owners retain linked carrier sites and alternatives; the previously missing first-party and report entries now have supplied comparisons or the ruled formal disposition at (`rec:backlog:wave0-compact-inherited-closure`). No positive compact result is promoted.

The current manifest partition is 48 / 20 / 6, with 209 = 135 positive + 74 negative requirements and 0 missing entries, before 24 = 18 + 6; first-party standing is 20 closed / 0 retained, comprising 18 observed + 2 formal, before 2 discharged / 18 retained. Earlier before-figures remain 50 target-executable / 18 first-party / 16 retained first-party, 213 = 139 + 74 requirements and 22 = 16 + 6 missing entries. These transitions are pinned by `compact_negative_manifest_matches_every_published_negative`, `compact_negative_manifest_pins_missing_owners`, `every_coverage_requirement_has_exactly_one_row`, `the_negative_half_splits_into_three_kinds_that_sum_to_it` and `compact_first_party_dispositions_retain_all_twenty_subjects`. Rulings 136–139 and the performed comparisons supply the distinct observed and formal dispositions at (`rec:backlog:wave0-compact-inherited-closure`).

**Table (Negative classes)** · `tab:backlog:compact-negative-classes`

| Class | Inherited count | Current count | Closed dispositions | Comparisons | Outstanding | Pinning test |
|---|---:|---:|---|---|---:|---|
| target-executable | 48 | 48 (before scope-only multiplicity: 50) | 27 observed + 21 formal; before: 1 historical at its exact candidate scope / 0 new | 48 current native subjects; before: 0 new and four linked attempts retained | 0; before: 47 inherited / 49 then-current | `compact_negative_manifest_matches_every_published_negative`; `compact_negative_manifest_has_unique_routes_and_preserves_collateral`; `joined_campaign_counts_each_row_once_across_both_archives`; `record_dispositions_bind_exactly_to_the_current_manifest` |
| first-party | 18 | 20 (before scope-only multiplicity: 18) | 18 observed + 2 formal = 20 closed; before: 2 at the dependency entry | 20 exact identities; before: 2 identities sharing one public/private pair | 0 retained; before closure: 18 retained; before scope-only multiplicity: 16 | `compact_first_party_dispositions_retain_all_twenty_subjects`; `compact_closure_joins_all_twenty_six_keys_and_the_native_forty_eight` |
| unreachable | 6 | 6 | 6 observed report refusals; before: 0 | 6 report comparisons; before: 0 and report consumer absent | 0 retained; before: 6 | `substrate_closure_admits_six_observed_and_zero_retained_report_rows`; `compact_negative_manifest_pins_missing_owners` |

`every_coverage_requirement_has_exactly_one_row` pins 209 = 135 positive + 74 negative, before 213 = 139 + 74 and inherited 211 = 139 + 72. `the_negative_half_splits_into_three_kinds_that_sum_to_it` pins 48 / 20 / 6, before 50 / 18 / 6 and inherited 48 / 18 / 6. The historical 71 of 72 outstanding statement remains scoped to its original evidence; derivation still discharges only two first-party identities.

ABI boundary rows: 2, outside the compiler-derived denominator, with four identities: `wrong-sequence-sponsorless`, `wrong-sequence-sponsored`, `wrong-transaction-version-sponsorless`, and `wrong-transaction-version-sponsored`. `supplemental_abi_rows_have_four_staged_sponsor_case_identities` pins the two classes, four names and exact changed fields. Sponsor authorization-shaped test data establishes construction only.

**Table (First-party rows)** · `tab:backlog:compact-first-party-rows`

| Row | Disposition | Comparison subject or retained argument | Test |
|---|---|---|---|
| `permissionless-private-dependency-sponsorless` | discharged | Published public dependency versus ASH-owner-private availability; `compiler::validate_required_dependency` returns the exact class | `the_permissionless_private_dependency_row_is_discharged_end_to_end` |
| `permissionless-private-dependency-sponsored` | discharged | Same retained pair, sponsored requirement identity | `the_permissionless_private_dependency_row_is_discharged_end_to_end` |
| `constructibility-witness-unavailable-sponsorless` | formal (before: retained) | Compact permissionless authorization selects `PermissionlessPrivateDependency`; ruling 137 closes the unreachable requested class without relabelling the supporting refusal | `compact_first_party_dispositions_retain_all_twenty_subjects`; `compact_closure_joins_all_twenty_six_keys_and_the_native_forty_eight`; `the_same_case_does_not_discharge_the_neighbouring_class` |
| `constructibility-witness-unavailable-sponsored` | formal (before: retained) | Compact permissionless authorization selects `PermissionlessPrivateDependency`; ruling 137 closes the unreachable requested class without relabelling the supporting refusal | `compact_first_party_dispositions_retain_all_twenty_subjects`; `compact_closure_joins_all_twenty_six_keys_and_the_native_forty_eight`; `another_operations_authorization_cannot_answer_compact_witness_coverage` |
| `unsupported-representation-sponsorless` | observed (before: retained) | Explicit control versus private-committed selection at `validate_compact_representation` | `compact_first_party_dispositions_retain_all_twenty_subjects`; `compact_closure_joins_all_twenty_six_keys_and_the_native_forty_eight` |
| `unsupported-representation-sponsored` | observed (before: retained) | Explicit control versus private-committed selection at `validate_compact_representation` | `compact_first_party_dispositions_retain_all_twenty_subjects`; `compact_closure_joins_all_twenty_six_keys_and_the_native_forty_eight` |
| `missing-compact-exit-analysis-sponsorless` | observed (before: retained) | Retained lifecycle projection versus removal of its Compact `RequiresExit` edge at `validate_compact_lifecycle_projection` | `compact_first_party_dispositions_retain_all_twenty_subjects`; `compact_closure_joins_all_twenty_six_keys_and_the_native_forty_eight` |
| `missing-compact-exit-analysis-sponsored` | observed (before: retained) | Retained lifecycle projection versus removal of its Compact `RequiresExit` edge at `validate_compact_lifecycle_projection` | `compact_first_party_dispositions_retain_all_twenty_subjects`; `compact_closure_joins_all_twenty_six_keys_and_the_native_forty_eight` |
| `missing-clear-exit-analysis-sponsorless` | observed (before: retained) | Retained lifecycle projection versus removal of its Clear `RequiresExit` edge at `validate_compact_lifecycle_projection`; outside-scope status remains | `compact_first_party_dispositions_retain_all_twenty_subjects`; `compact_closure_joins_all_twenty_six_keys_and_the_native_forty_eight` |
| `missing-clear-exit-analysis-sponsored` | observed (before: retained) | Retained lifecycle projection versus removal of its Clear `RequiresExit` edge at `validate_compact_lifecycle_projection`; outside-scope status remains | `compact_first_party_dispositions_retain_all_twenty_subjects`; `compact_closure_joins_all_twenty_six_keys_and_the_native_forty_eight` |
| `unexpected-protocol-secret-sponsorless` | observed (before: retained) | Protocol leaf versus inserted `CheckSig` at `validate_compact_secret_free_path` | `compact_first_party_dispositions_retain_all_twenty_subjects`; `compact_closure_joins_all_twenty_six_keys_and_the_native_forty_eight` |
| `unexpected-protocol-secret-sponsored` | observed (before: retained) | Protocol leaf versus inserted `CheckSig` at `validate_compact_secret_free_path` | `compact_first_party_dispositions_retain_all_twenty_subjects`; `compact_closure_joins_all_twenty_six_keys_and_the_native_forty_eight` |
| `unauthenticated-representation-sponsorless` | observed (before: retained) | Input-zero recognition fragment versus removal of explicit-value authentication at `validate_compact_emitted_representation` | `compact_first_party_dispositions_retain_all_twenty_subjects`; `compact_closure_joins_all_twenty_six_keys_and_the_native_forty_eight` |
| `unauthenticated-representation-sponsored` | observed (before: retained) | Input-zero recognition fragment versus removal of explicit-value authentication at `validate_compact_emitted_representation` | `compact_first_party_dispositions_retain_all_twenty_subjects`; `compact_closure_joins_all_twenty_six_keys_and_the_native_forty_eight` |
| `missing-compact-exit-emitted-sponsorless` | observed (before: retained) | Keyed emitted programs versus removal of the sponsor-matched coordinator leaf at `validate_compact_emitted_lifecycle` | `compact_first_party_dispositions_retain_all_twenty_subjects`; `compact_closure_joins_all_twenty_six_keys_and_the_native_forty_eight` |
| `missing-compact-exit-emitted-sponsored` | observed (before: retained) | Keyed emitted programs versus removal of the sponsor-matched coordinator leaf at `validate_compact_emitted_lifecycle` | `compact_first_party_dispositions_retain_all_twenty_subjects`; `compact_closure_joins_all_twenty_six_keys_and_the_native_forty_eight` |
| `missing-clear-exit-emitted-sponsorless` | declaration observed (before: retained) | Outstanding Clear declaration versus its removal at `validate_compact_emitted_lifecycle`; no emitted Clear path; ruling 138 reopens in Wave 2 | `compact_first_party_dispositions_retain_all_twenty_subjects`; `compact_closure_joins_all_twenty_six_keys_and_the_native_forty_eight` |
| `missing-clear-exit-emitted-sponsored` | declaration observed (before: retained) | Outstanding Clear declaration versus its removal at `validate_compact_emitted_lifecycle`; no emitted Clear path; ruling 138 reopens in Wave 2 | `compact_first_party_dispositions_retain_all_twenty_subjects`; `compact_closure_joins_all_twenty_six_keys_and_the_native_forty_eight` |
| `sponsor-multiplicity-exceeded-sponsorless` | observed (before closure: retained; before scope-only multiplicity: target class) | One owned fee-sponsor envelope versus two at `validate_sponsor_envelope_multiplicity`; ModelScope | `compact_first_party_dispositions_retain_all_twenty_subjects`; `compact_closure_joins_all_twenty_six_keys_and_the_native_forty_eight` |
| `sponsor-multiplicity-exceeded-sponsored` | observed (before closure: retained; before scope-only multiplicity: target class) | One owned fee-sponsor envelope versus two at `validate_sponsor_envelope_multiplicity`; ModelScope | `compact_first_party_dispositions_retain_all_twenty_subjects`; `compact_closure_joins_all_twenty_six_keys_and_the_native_forty_eight` |

**Result (Manifest complete)** · `result:backlog:compact-manifest`

Request 1: `packages/vectors/src/compact_negative_manifest.rs`, checked by `compact_negative_manifest_matches_every_published_negative`, `compact_negative_manifest_has_unique_routes_and_preserves_collateral` and `compact_negative_manifest_pins_missing_owners`. Request 2: `packages/vectors/src/first_party.rs`, checked by `compact_first_party_dispositions_retain_all_twenty_subjects` and `compact_closure_joins_all_twenty_six_keys_and_the_native_forty_eight`. Request 3: `packages/transaction/src/abi.rs`, consumed by `packages/vectors/src/abi_validation.rs` and checked by `supplemental_abi_rows_have_four_staged_sponsor_case_identities` and `abi_test_staging_refuses_other_faults_and_wrong_shape_counts`. Missing entries are 0, before 24 = 18 first-party + 6 external and earlier 22 = 16 + 6; exact closure and gates are recorded at (`rec:backlog:wave0-compact-inherited-closure`); zero missing entries does not assert executable-owner completeness.

**Result (Boundaries staged)** · `result:backlog:compact-staging`

`CompactConstructorTestStaging` retains a constructed input selection and offers empty and duplicate selections to `CompactAshRequest::new`; `compact_constructor_test_staging_observes_only_constructor_refusals` checks both typed refusals without native credit. `CompactDependencyTestStaging` retains the published public node and private-availability sibling for `compiler::validate_required_dependency`; the disposition test drives both sponsor identities. `CompactAbiTestStaging` retains each constructed transaction and changes only sequence or version; `validate_compact_abi_fields` accepts the control and names the exact malformed field. Constructor pinning remains checked. No production validator is weakened. The sixteen then-retained analysis/emitted rows, eighteen retained after scope-only multiplicity, now have fourteen direct observations, two declaration observations and two formal dispositions in addition to the two original dependency observations at (`rec:backlog:wave0-compact-inherited-closure`).

**Caveat (Native evidence and candidate scope)**

At manifest admission no native observation was supplied; the one historical sponsorless ASH-output-cardinality discharge retained its original candidate scope, leaving 47 inherited / 49 then-current outstanding rows, including the two `SponsorMemberUnclaimed` additions. The current 48 native rows subsequently closed as 27 observed + 21 formal at (`rec:backlog:wave0-compact-campaign`). An existing conservation mutant answered before script execution still supplies no runtime-carrier discharge.

**Caveat (Retained observable obligations)**

No observable first-party obligation is removed: the earlier 18 = 2 discharged / 0 out-of-scope / 16 retained became 20 = 2 discharged / 0 out-of-scope / 18 retained after scope-only multiplicity and now 20 closed / 0 retained = 18 observed + 2 formal. Ruling 137 supplies the two argued candidate-specific formal dispositions. The six external-report rows now have six report-consumer observations, before six retained with the consumer absent. Native and scoped-entry closure is recorded at (`rec:backlog:wave0-compact-inherited-closure`); the ABI entry retains its separate denominator.

At manifest admission the native arithmetic was 47 inherited / 49 then-current outstanding identities; the two membership additions preserved the inherited denominator and historical evidence scope. Current native closure is 27 observed + 21 formal with zero unresolved at (`rec:backlog:wave0-compact-campaign`).

ABI requirements remain outside the compiler-derived denominator, now 209 requirements, before 213: two ABI classes and four sponsor-case identities. The typed ABI entry and comparison evidence are recorded at (`rec:backlog:wave0-compact-inherited-closure`).

The sixteen first-party obligations with private or missing entries, eighteen after scope-only multiplicity, now resolve through the supplied-entry mapping and ruled formal disposition in `packages/vectors/src/compact_negative_manifest.rs`. The additive compiler and tapscript entries and exact comparisons are named at (`rec:backlog:wave0-compact-inherited-closure`).

Compact authorization selects `PermissionlessPrivateDependency`; that refusal still does not observe the requested unavailable-witness class. Ruling 137 formally closes the two identities on that exact candidate premise at (`rec:backlog:wave0-compact-inherited-closure`).

Both emitted Clear rows now observe retention and removal of the outstanding declaration, before retaining the obligation without an accepting emitted control. No accepting emitted Clear path is invented; ruling 138 reopens the disposition when Clear is realized in Wave 2.

Completeness means exact key equality at 74, the preserved partition, unique concrete links and retained collateral. Missing entries are now 0, before 24 = 18 + 6 and earlier 22 = 16 + 6, pinned by `compact_negative_manifest_pins_missing_owners`.

The manifest landing established manifest completeness, the ABI entry, constructor and dependency staging, and first-party dispositions 2 / 0 / 16, leaving analysis/emitted staging and the native half outstanding. Subsequent scope-only multiplicity gave 2 / 0 / 18; native closure is recorded at (`rec:backlog:wave0-compact-campaign`) and inherited closure at (`rec:backlog:wave0-compact-inherited-closure`).

The manifest is a registered vectors module derived from validated coverage, without a second key inventory. Constructor-pinning tests remain while ABI-refusal assertions now require a driven malformed subject. The implementation remains bounded to the three implementing units and their records.

**Record (Wave 0 preflight semantic census)** · `rec:backlog:wave0-preflight-census`

This is the semantic census and acceptance packet for bite 0.7, under (`task:guide15-exec:disposition-1`). It records inherited executable law and selected future implementation contracts separately. The four inherited debts retain their own closures; the compact closure at (`rec:backlog:wave0-compact-inherited-closure`) is not reopened. The execution-register join, completed-tree baseline and complete preflight verdict remain owed by (`T12-007`), which stays TODO. Package-contract boundaries are unchanged; no package contract or research note needs an amendment.

Consuming waves use the shifted execution numbering at (`conv:guide15-exec:later-wave-numbering`); the referenced execution-wave sections retain their own deliverables and gates.

**Table (Twelve-question index)**:

| Question | Source | Disposition and acceptance | Retained execution obligation |
|---|---|---|---|
| 1. Semantic census | (`q:guide15:semantic-census`) | Typed owners below; ruling 140 | Burn and Clear declarations and validation in Wave 2 |
| 2. ASH representation | (`q:guide15:ash-representation`) | Explicit-only backend boundary; ruling 141 | Selected target representation in Wave 4 (`task:guide15-exec:wave4`) |
| 3. Private burn route | (`q:guide15:private-burn`) | Normalized-first with private change; ruling 141 | Private-route disposition in Wave 4 (`task:guide15-exec:wave4`) |
| 4. Constructor continuity | (`q:guide15:ash-constructor`) | Equal ASH constructor requirement; ruling 142, retaining ruling 128 | Fresh, compacted and residual equality proof at the Wave-5 gate (`obl:guide15-exec:disposition-4-proof`), with Wave-8 cross-operation closure (`task:guide15-exec:wave8`) |
| 5. Record carrier | (`q:guide15:burn-record-carrier`) | Proposed encoding and signing packet; ruling 144 | Wave-4 carrier measurements under (`cond:guide15-exec:carrier-reopening`); size or policy refusal requires a Wave-4 ruling before declarations; assessment in Wave 4 (`task:guide15-exec:wave4`); carrier in Wave 6 (`task:guide15-exec:wave6`); signing in Wave 9 (`task:guide15-exec:wave9`) |
| 6. Record law | (`q:guide15:burn-record-rule`) | Raw burn validity separate from credit; ruling 143 | Record law in Waves 2 and 10 (`task:guide15-exec:wave2`), (`task:guide15-exec:wave10`); carrier in Wave 6 (`task:guide15-exec:wave6`) |
| 7. Clear assignment | (`q:guide15:clear-assignment`) | Stored `y_l`, derived Y; (`rule:guide15-exec:disposition-7`) | Wave-2 declaration and Wave-7 lowering |
| 8. Clear arithmetic | (`q:guide15:clear-arithmetic`) | Checked bounded schedule; ruling 145 | Emitted schedule execution in Wave 7 |
| 9. STATE generation | (`q:guide15:state-generation`) | Synthetic new origin, no crossing or migration; rulings 131 and 146 | Expanded static subtree and maturity revalidation in Wave 7 |
| 10. Events and accounting | (`q:guide15:event-independence`) | Separate producers and claims; ruling 147 | Wave-10 event evidence and Waves 10–12 evidence producers (`task:guide15-exec:wave10`), (`task:guide15-exec:wave11`), (`task:guide15-exec:wave12`) |
| 11. Public construction | (`q:guide15:permissionless`) | Fresh-process contract; ruling 148 | Fresh-to-compact-to-clear construction in Waves 5 and 11 |
| 12. Query boundary | (`q:guide15:attestation-query`) | Internal projection default; (`rule:guide15-exec:disposition-12`) | Internal projection retained under (`rule:guide15-exec:disposition-12`); event inputs in Waves 10 and 12; any external consumer needs separate admission |

**Table (Question 1 typed owners)**:

| Source label | Architecture identifier | Model owner | Realization owner or absence | Stored / derived / evidence | Inherited test | Acceptance clause | Consuming wave | Reopening premise |
|---|---|---|---|---|---|---|---|---|
| (`q:guide15:semantic-census`) | `packages/architecture/src/ids.rs`: `OperationId::{Burn,CompactAsh,Clear}`; `packages/architecture/src/spec.rs`: their `OperationSpec` entries | `packages/model/src/ops/burn.rs`: `BurnReceipts::apply`; `packages/model/src/ops/ash.rs`: `CompactAsh::apply`, `ClearAsh::apply` | `packages/realization/src/operation.rs`: `OperationRealization`; `packages/realization/src/declarations/mod.rs`: `derive_operation` dispatches only AnnounceMaturity, CompactAsh and TransferLive; Burn and Clear return `UnsupportedOperationDeclaration` | Requests and declaration graphs are stored inputs; transitions and certification derive successors and projections; no absent declaration receives validation credit | `packages/model/src/tests/realization_conformance_tests.rs`: `compact_ash_model_transition_satisfies_realization`, `binding_rejects_a_successor_from_another_compact_ash` | Ruling 140; (`cond:guide15:operation-scope`) and (`cond:guide15:validated-relations`) retain DOES NOT HOLD | Wave 2 | Changed operation, field, relation or dispatch boundary |

`OperationRealization` in `packages/realization/src/operation.rs` stores `operation`, `expressions`, `relations`, `relation_dependencies`, `constructibility_nodes`, `constructibility_edges`, `lifecycle_nodes`, `lifecycle_edges`, `disclosure_nodes`, `disclosure_edges`, `disclosure_seeds`. Compact's `derive` supplies no expressions and supplies the relation and dependency graphs below; `OperationRealization::project` derives its stable operation row and sorted disclosure seeds. Neither stored declaration data nor a stable projection proves target execution.

The exact compact declaration population is 23. The names below are the `Ids` fields consumed by `relation_declarations` in `packages/realization/src/declarations/compact_ash.rs`; each denotes its complete `RelationId`, not a second free-form inventory. `RelationKind`, transaction side, object and asset qualify repeated kinds. `compact_ash_weld_accepts_the_published_architecture` and `compact_ash_negative_observations_name_the_load_bearing_relation` in `packages/realization/src/tests/compact_ash_tests.rs` retain the inherited weld and refusal scope. No corresponding Burn or Clear declaration list exists yet.

| Declaration name | Relation and subject |
|---|---|
| `input_cardinality` | Cardinality, ASH input, 2 through `AshBatchMax` |
| `output_cardinality` | Cardinality, ASH output, exactly 1 |
| `sponsor_input_cardinality` | Cardinality, PlainLbtc input, 0 through `FeeSponsorInputMax` |
| `sponsor_output_cardinality` | Cardinality, PlainLbtc output, 0 through 1 |
| `input_recognition` | Recognition, ASH input, U |
| `output_recognition` | Recognition, ASH output, U |
| `sponsor_input_recognition` | Recognition, PlainLbtc input, Lbtc |
| `sponsor_output_recognition` | Recognition, PlainLbtc output, Lbtc |
| `permissionless` | PermissionlessAuthorization, operation |
| `input_closure` | AllowedObjectFamilies, input, Ash and PlainLbtc |
| `output_closure` | AllowedObjectFamilies, output, Ash and PlainLbtc |
| `conservation` | AmountConservation, U, ASH inputs and output |
| `sponsor` | SponsorIsolation, sponsor region |
| `substrate_conservation` | SubstrateConservation, Lbtc; external evidence requirement |
| `sponsor_multiplicity` | SponsorEnvelopeMultiplicity, maximum 1; model scope |
| `open_flow_policy` | OpenFlowPolicy, FeeSponsor only |
| `canonical_delta_policy` | CanonicalDeltaPolicy, U OwnerlessLateral, no destruction tag |
| `roots` | RootPolicy, every root forbidden |
| `projections` | ProjectionPolicy, TransitionCertificate required, every other projection forbidden |
| `constructibility` | Constructibility, PublicPermissionless |
| `representation` | Representation, Ash, Explicit and PublicCommitted semantic modes |
| `compact_lifecycle` | LifecycleExit, Ash to CompactAsh |
| `clear_lifecycle` | LifecycleExit, Ash to Clear; declaration obligation, not emitted Clear execution |

**Table (Question 2 typed owners)**:

| Source label | Architecture identifier | Model owner | Realization owner or absence | Stored / derived / evidence | Inherited test | Acceptance clause | Consuming wave | Reopening premise |
|---|---|---|---|---|---|---|---|---|
| (`q:guide15:ash-representation`) | `packages/architecture/src/spec.rs`: `ObjectId::Ash` lifecycle | `packages/model/src/object.rs`: `Utxo { asset, value, meta }`, `Meta::Ash` | `packages/realization/src/domain.rs`: `RepresentationMode`; `packages/realization/src/declarations/compact_ash.rs`: `Relation::Representation` | Model semantic value is stored without a confidential coordinate; backend explicit-only selection is policy, not a restriction of the compact representation relation's semantic {Explicit, PublicCommitted} set | `packages/model/src/tests/realization_conformance_tests.rs`: `phase1_declassification_matches_model_owned_pilot_rows`; `packages/realization/src/tests/lifecycle_tests.rs`: `an_extra_graph_representation_is_rejected` | Ruling 141, (`cond:guide15:public-revalidation`) | Wave 4 (`task:guide15-exec:wave4`) | Changed reviewed target or authenticated opening evidence |

The revalidation is a source-boundary reading against (`sec:public-opening:result`), not a new opening experiment. PublicCommitted and direct authenticated private opening remain deferred on `GeneratorNotDerivableOnScript`, `EncodingDomainMismatch` and `SuppliedParityUnbound`; a scalar attached to a commitment does not authenticate an opening.

**Table (Question 3 typed owners)**:

| Source label | Architecture identifier | Model owner | Realization owner or absence | Stored / derived / evidence | Inherited test | Acceptance clause | Consuming wave | Reopening premise |
|---|---|---|---|---|---|---|---|---|
| (`q:guide15:private-burn`) | `packages/architecture/src/spec.rs`: Burn uses `PermissionClass::ReceiptOwners`, ReceiptLive inputs and change, one Ash output | `packages/model/src/ops/burn.rs`: `BurnReceipts { receipts, signers, ash_value, change, records, fee_envelope }` | Burn dispatch absent; semantic private-opening input absent | Request stores references and semantic values; checked live-input total equals fresh ASH plus live change; normalization and private change are backend policy | `packages/model/src/tests/burn_attestation_tests.rs`: `genuine_burn_creates_one_burn_projection`; no extra test needed for route selection | Ruling 141, (`cond:guide15:private-route`) | Wave 4 (`task:guide15-exec:wave4`) | Direct-private admission or changed normalization policy |

**Table (Question 4 typed owners)**:

| Source label | Architecture identifier | Model owner | Realization owner or absence | Stored / derived / evidence | Inherited test | Acceptance clause | Consuming wave | Reopening premise |
|---|---|---|---|---|---|---|---|---|
| (`q:guide15:ash-constructor`) | `packages/architecture/src/spec.rs`: Ash allocators Burn, CompactAsh and Clear | `packages/model/src/ops/burn.rs` and `packages/model/src/ops/ash.rs`: all three transitions emit `Meta::Ash` | `packages/realization/src/declarations/compact_ash.rs`: `compact_lifecycle`, `clear_lifecycle`, `lifecycle_declarations`; no Burn/Clear constructors | Semantic class is shared; target constructor bytes and cross-operation equality are evidence still owed | `packages/realization/src/tests/lifecycle_tests.rs`: `the_derived_pilots_satisfy_the_lifecycle_weld`, `a_missing_required_edge_is_rejected_with_both_nodes_present` | Ruling 142, (`task:guide15-exec:disposition-4`), (`cond:guide15:ash-compatibility`); ruling 128 retained | Wave-5 equality gate (`obl:guide15-exec:disposition-4-proof`) and Wave-8 cross-operation closure (`task:guide15-exec:wave8`) | New class, recipe, target, deployment, representation or linked bundle |

The constructor recipe is fixed by the selected compact `CandidateLinkedBundle` and `LinkedConstructor` in `packages/linker/src/bundle.rs`: target contract, leaf version, representation, explicit-value policy, internal-key bytes and policy, key-path policy, exact linked programs, deterministic tree and resolved deployment definitions. `packages/transaction/src/taproot.rs` supplies `commit_tree`, `PinnedAshInstance::check_against` and `output_script`: commit the exact leaf programs and tree, retain the public pin's output key and parity, verify the pin against the bundle and use the witness-version-one output script. The pin and tree alone do not prove a newly implemented Burn or Clear constructor computes the same key. Ruling 128 supplies the inherited compact subject; no new byte compatibility is inferred from `Meta::Ash`.

| ASH class | Semantic constructor | Required target constructor equality | Evidence standing |
|---|---|---|---|
| Fresh | Burn emits one positive `Asset::U`, `Meta::Ash` | Same linked recipe, output program and public reconstruction as compacted ASH under one target, deployment, representation and bundle | Wave-5 equality gate (`obl:guide15-exec:disposition-4-proof`); Wave-8 cross-operation closure (`task:guide15-exec:wave8`), not a result |
| Compacted | CompactAsh emits summed `Asset::U`, `Meta::Ash` | Inherited selected compact recipe is the comparison subject | Existing compact evidence only at its recorded scope |
| Clear-residual | Clear emits `Asset::U`, `Meta::Ash` only for positive residual | Same linked recipe and output program as fresh and compacted ASH | Wave-5 equality gate (`obl:guide15-exec:disposition-4-proof`); Wave-8 cross-operation closure (`task:guide15-exec:wave8`), not a result; no residual output when zero |

**Table (Question 5 typed owners)**:

| Source label | Architecture identifier | Model owner | Realization owner or absence | Stored / derived / evidence | Inherited test | Acceptance clause | Consuming wave | Reopening premise |
|---|---|---|---|---|---|---|---|---|
| (`q:guide15:burn-record-carrier`) | `packages/architecture/src/spec.rs`: `DataOutputSpec`, BurnRecord, `TagId::Burn`, asset None, minimum 0, `BurnRecordMax` | `packages/model/src/object.rs`: `DataOutput::BurnRecord`; `packages/model/src/history.rs`: `BurnRecord { record_index, address, amount }`; `packages/model/src/certify.rs`: `derive_burn_projection` | No Burn target carrier codec or realization declaration | Raw payload stored; `BurnProjection { ash_output, ash_value, records }` derived; target encoding below is proposed, not measured | `packages/model/src/tests/burn_attestation_tests.rs`: `genuine_burn_creates_one_burn_projection`, `bare_burn_tag_without_burn_projection_attests_nothing` | Ruling 144, (`task:guide15-exec:disposition-5`), (`cond:guide15:signing-profiles`) | Wave-4 carrier measurements under (`cond:guide15-exec:carrier-reopening`); Wave-4 assessment (`task:guide15-exec:wave4`); Wave-6 carrier (`task:guide15-exec:wave6`); Wave-9 signing (`task:guide15-exec:wave9`) | Size or policy refusal under the Wave-4 carrier measurements requires a Wave-4 ruling before declarations |

**Table (Proposed carrier packet)** · `tab:backlog:preflight-carrier`

| Component | Exact proposed encoding or rule | Evidence and boundary |
|---|---|---|
| Payload offsets | Bytes 0–7: ASCII `tag-burn`; byte 8: schema 1; bytes 9–12: unsigned four-byte little-endian record ordinal; bytes 13–44: the 32 `AttestationAddress` bytes in stored order; bytes 45–52: unsigned eight-byte little-endian record amount | 53 payload bytes; `packages/model/src/scalar.rs`: `AttestationAddress`; `packages/model/src/history.rs`: `BurnRecord`; no existing target codec claimed |
| Script | `OP_RETURN`, the direct 53-byte push opcode, then exactly the 53-byte payload; no other instruction, alternative push encoding or trailing byte | 55 script bytes; unspendable chain-public data-output proposal |
| Outer asset and value | Deployment's explicit L-BTC asset, explicit zero value, null nonce, empty surjection and range proofs; amount is in the payload, not the output's spendable value; recognition is by family (exact script, tag and placement), never by zero value; the record output is not a PLAIN_LBTC object and joins no open flow or sponsor region | The explicit L-BTC field is a target encoding necessity with no protocol asset or value allocation; architecture data asset None retains that boundary; this zero-value target envelope adds no U destruction and is subject to target/policy measurement |
| Placement | Fresh ASH at output zero; live change in request order; a contiguous record block in increasing ordinal order; optional sponsor change next; explicit fee output last | Record count zero means an empty record block; all non-record output roles are authenticated before the record block is accepted; this is the proposed Burn ABI, not existing implementation |
| Parsing | Exact script, tag, schema, widths and endianness; ordinals 0 through count minus one with no gap or duplicate; count at most configured `BurnRecordMax`; each amount positive and below `2^51`; address is the opaque 32-byte identity, with repeats allowed | Refuse wrong tag/schema, truncation, trailing bytes, alternate push, noncanonical ordinal, wrong placement, wrong asset/value/nonce/proof form, zero/out-of-domain amount and surplus records at the carrier-recognition boundary; raw chain inclusion alone supplies no record acceptance |
| Identity and recovery | Recover the record identity as enclosing authenticated Burn transaction identity plus record ordinal; recover fresh ASH outpoint from that transaction's unique authenticated ASH output; recompute event order from authenticated chain context | Authenticate spent ReceiptLive inputs, no ASH input, exact output families, value balance and applicable signatures before credit; copied payload without authenticated Burn recognition attests nothing; no payload-supplied transaction identity, ASH pointer or sampled Clear identifier |
| Credit | Sum valid individual amounts for comparison with the fresh ASH value; zero records permitted; underclaim donates; overclaim yields no credit for the whole record set without invalidating the raw burn | `packages/model/src/ledger.rs`: `BurnTransaction::records_accepted`; aggregate need not fit `Sat`; ruling 143 unchanged |
| Serialization | Each record output: 33 explicit asset bytes + 9 explicit value bytes + 1 null nonce byte + 1 script-length byte + 55 script bytes = 99 stripped bytes; a witnessed transaction adds two empty proof-length bytes per record output | Derived from `packages/transaction/src/bytes.rs`: `encode_asset_field`, `encode_value_field`, `TargetOutput::encode_at`, `OutputWitness::encode`, `TargetTransaction::serialize_with_locator`; analytic cost, not a serialized candidate measurement |
| All-output cost | Let N be records, O the count of other outputs, C(n) the CompactSize width, and S the sum of serialized other-output sizes. Output-vector stripped size is C(O+N) + S + 99N; relative to zero records the increment is 99N + C(O+N) − C(O). In a witnessed explicit form the output-witness vector is 2(O+N) bytes; record increment is 2N, and record weight increment is 398N + 4(C(O+N) − C(O)) | C(n) is 1 below 253, 3 through 65535, 5 through the unsigned four-byte limit, otherwise 9; at default N=64, record bodies alone are 6336 stripped bytes and 128 witness bytes, weight 25472 before any count-width change. Confidential change adds its actual commitment, nonce and proof sizes; inputs, input witnesses, header and lock time are additional. Neither the 55-byte script nor this output subtotal is a whole-transaction policy claim |
| Measurement reopen | Wave 4 must measure complete supported attestation carrier transactions, recording all outputs, proofs, weight, fees and policy outcomes under (`cond:guide15-exec:carrier-reopening`); required subjects retain the multiple-data-output standardness premise and exact zero-value 53-byte N-output form. Wave-1 measurements concern the two-clause covenant and do not discharge these subjects; refusal of size or policy reopens selection by a Wave-4 ruling before declarations | Proposed encoding until measured; no codec implementation, relay pass or target observation supplied here |

**Table (Carrier signing roles and protected bytes)**:

| Role | Selected protection | Existing evidence and future obligation |
|---|---|---|
| Each applicable receipt input | Finalized all-inputs/all-outputs authorization over the complete input census and spent-output facts, version, sequences, lock time, fresh ASH, ordered live change, record block, sponsor change and fee; retain executing script-path and deployment binding | `packages/transaction/src/live_finalize.rs`: `FinalizedLiveTransfer`, `protected_preimage`; `packages/transaction/src/live_signing.rs`: `authorize_live_transfer` supply the inherited finalization discipline, not a Burn signing implementation |
| Each sponsor input | `SighashProfile::AllInputsAllOutputs`, complete finalized non-witness transaction; no sponsor capability crosses into protocol authorization | `packages/transaction/src/sponsor.rs`: `SponsorSigningRequest`, `SignerRole::SponsorSuffixMember`; version-zero sponsor signatures do not bind output proof bytes, unlike the selected receipt script-path profile |
| Finalization and assembly | Freeze all protected output bytes and applicable proof vector before signatures; collect input witnesses afterwards; reject any protected-field mutation or mismatched response and preserve exact role bindings | Private-change receipt authorization must bind its output-witness proofs as `protected_preimage` does for the private live form; sponsor proof exclusion is retained explicitly; actual Burn finalization, multi-owner signing, profile validation and post-signing mutation refusal remain Wave 9 (`task:guide15-exec:wave9`) |

**Table (Question 6 typed owners)**:

| Source label | Architecture identifier | Model owner | Realization owner or absence | Stored / derived / evidence | Inherited test | Acceptance clause | Consuming wave | Reopening premise |
|---|---|---|---|---|---|---|---|---|
| (`q:guide15:burn-record-rule`) | `packages/architecture/src/spec.rs`: `BoundId::BurnRecordMax`, unique fresh Ash output | `packages/model/src/ops/burn.rs`: `BurnReceipts::apply`; `packages/model/src/kernel.rs`: `TxBuilder::emit_burn_record`; `packages/model/src/certify.rs`: `derive_burn_records`; `packages/model/src/ledger.rs`: `BurnTransaction::records_accepted`, `validate_burn_records` | Burn declaration absent | Stored contiguous zero-based indices and positive individual `Sat` amounts; derived aggregate credit uses checked `u128` and early overclaim refusal of credit, not burn validity; default maximum 64 is configuration, not immutable law | `packages/model/src/tests/burn_attestation_tests.rs`: `overclaiming_records_do_not_invalidate_burn`, `burn_record_overclaim_beyond_sat_domain_keeps_burn_valid`, `underclaiming_records_is_a_valid_donation`, `duplicate_burn_record_indices_are_rejected`, `noncanonical_burn_record_index_is_rejected` | Ruling 143, (`task:guide15-exec:disposition-6`) | Waves 2 and 10 record law (`task:guide15-exec:wave2`), (`task:guide15-exec:wave10`); Wave-6 carrier (`task:guide15-exec:wave6`) | Changed record or accounting authority |

**Table (Question 7 typed owners)**:

| Source label | Architecture identifier | Model owner | Realization owner or absence | Stored / derived / evidence | Inherited test | Acceptance clause | Consuming wave | Reopening premise |
|---|---|---|---|---|---|---|---|---|
| (`q:guide15:clear-assignment`) | `packages/architecture/src/ids.rs`: `OperationId::Clear`, `TagId::Recon`; Clear `OperationSpec` in `packages/architecture/src/spec.rs` | `packages/model/src/pool.rs`: `PoolState`, `y`, `is_sealed`; `packages/model/src/ops/ash.rs`: `ClearAsh::apply` | `packages/realization/src/state.rs`: `StateMetadata`; `packages/realization/src/state_codec.rs`: `EncodedStateMetadata`, `StateRepresentationNonce`; Clear dispatch absent | `omega`, `y_l`, `y_t`, `q`, `cycle`, `maturity` stored; only `y_l` changes by X; Y is derived as `y_l + y_t`, so Y decreases by X without another field write; encoding nonce is nonsemantic | `packages/model/src/tests/ash_clear_tests.rs`: `clear_changes_only_the_stored_y_l_and_the_derived_total` at `0.7.59-dev` exhaustively destructures both states, checks the five unchanged fields, stored drop, ASH drop, residual, derived total and sealed status | (`rule:guide15-exec:disposition-7`); ruling 140 records the census | Waves 2 and 7 | Changed STATE field set, assignment or representation boundary |

The field identity weld is `StateField::data_id` in `packages/realization/src/identity.rs`, resolving to `DataId` in `packages/architecture/src/ids.rs`. Both semantic maturity enums, in `packages/model/src/asset.rs` and `packages/realization/src/state.rs`, distinguish Unannounced, Announced with cycle, and Complete. The model/realization conformance functions `project_then_embed_is_identity`, `embed_then_project_is_identity` and `both_censuses_have_six_fields` in `packages/model/src/tests/state_metadata_conformance_tests.rs` retain the six-field agreement required by (`inv:guide15-exec:clear-census`).

| Architecture data identifier | Model `PoolState` field | Realization `StateMetadata` field and `StateField` | Clear assignment |
|---|---|---|---|
| `DataId::StateOmega` | `omega: Sat` | `omega: ProtocolAmount`, Omega | Stored, unchanged |
| `DataId::StateYLive` | `y_l: Sat` | `y_l: ProtocolAmount`, YL | Stored, subtract X exactly once |
| `DataId::StateYTimeLocked` | `y_t: Sat` | `y_t: ProtocolAmount`, YT | Stored, unchanged |
| `DataId::StateQ` | `q: Sat` | `q: ProtocolAmount`, Q | Stored, unchanged |
| `DataId::StateCycle` | `cycle: Cycle` | `cycle: Cycle`, Cycle | Stored, unchanged |
| `DataId::StateMaturity` | `maturity: Maturity` | `maturity: Maturity`, Maturity | Stored, unchanged |
| `DataId::ClearY` | No stored Y; `PoolState::y()` | No stored Y; derived successor projection | Y = `y_l+y_t`; Y′ = Y−X |

The destruction balance is separate evidence: `ClearAsh::apply` in `packages/model/src/ops/ash.rs` declares `DestructionLeg { tag: Tag::Recon, amount: clear_amount }`; `derive_clear_projection` in `packages/model/src/certify.rs` requires exactly one U/Recon destruction and checks its amount against the stored `y_l` drop, while `derive_canonical_partition` checks source = destination + destruction. The existing `clear_decrements_y_l_and_ash_equally`, `clear_never_reaches_zero_total_supply` and `oversized_single_ash_partially_clears_with_residual` in `packages/model/src/tests/ash_clear_tests.rs` execute that certified path. The new field-assignment test is not credited with an independent destruction-tag amount assertion.

**Table (Question 8 typed owners)**:

| Source label | Architecture identifier | Model owner | Realization owner or absence | Stored / derived / evidence | Inherited test | Acceptance clause | Consuming wave | Reopening premise |
|---|---|---|---|---|---|---|---|---|
| (`q:guide15:clear-arithmetic`) | `packages/architecture/src/spec.rs`: Clear `OperationSpec` has empty named-quantity reads/writes, STATE root succession and `BoundId::AshBatchMax` | `packages/model/src/scalar.rs`: `Sat`; `packages/model/src/ops/ash.rs`: `ClearAsh::apply` | `packages/realization/src/domain.rs`: `ProtocolAmount`; no Clear arithmetic declaration | Authenticated ASH values and stored STATE quantities are operands; B, Y, X, residual and successor quantities derived; target schedule below is assessed, not emitted | `packages/model/src/tests/ash_clear_tests.rs`: `zero_progress_clear_is_rejected`, `clear_never_reaches_zero_total_supply`, `oversized_single_ash_partially_clears_with_residual` | Ruling 145, (`task:guide15-exec:disposition-8`) | Wave 7 | Changed bounds, operand encoding or opcode contract |

**Table (Checked Clear schedule)** · `tab:backlog:preflight-clear-schedule`

| Stage | Authenticated operands and checked action | Bound and refusal contract |
|---|---|---|
| Domain | Authenticate each ASH input's U asset, constructor and positive explicit amount; authenticate STATE metadata and singleton; decode amount operands as nonnegative integers below `2^51` | `Sat::new` and `ProtocolAmount::new` define the amount domain; STATE wire integers are big-endian and target arithmetic uses exactly eight-byte signed little-endian operands, so conversion must preserve the authenticated value. No arbitrary arithmetic witness substitutes for decoding |
| Accumulation | Start B at zero; for each recognized ASH amount b compute B+b with Add64, immediately verify its success flag, and require the new B below `2^51`; independently add stored `y_l+y_t` with the same checks to derive Y | Every addition has inputs below `2^51`, hence mathematical sum below `2^52`, safely below signed-64 overflow; semantic-domain rejection still applies at every accumulation, not merely at the end |
| Supply floor | Require Y positive, then compute Y−1 with Sub64 and immediately verify success; require the result in the nonnegative amount domain | Zero Y refuses before subtraction; cycle and maturity are copied, not cast to amount operands |
| First minimum | Set a=B and b=`y_l`; `DuplicateTwo` preserves the authenticated pair for `LessThanOrEqual64`. Apply `Size` then `RemoveSecond` to its boolean: the width is exactly c in {0,1}, typed as a script number; convert with `ScriptNumToLe64`, then `Neg64` and immediate `Verify` of its success flag yield mask=−c. Select T=b XOR ((a XOR b) AND mask), using `BitwiseXor`, `BitwiseAnd`, then `BitwiseXor` on eight-byte operands; apply `Substring` at 0 and 8 to give T an exact eight-byte type admitted as a signed-64 operand by `packages/tapscript/src/stack.rs` | T = min(B, `y_l`); true produces an all-ones mask and selects a, false an all-zero mask and selects b; equality chooses B deterministically; no witness chooses T |
| Second minimum | Set a=T and b=Y−1; preserve the pair with `DuplicateTwo`, compare with `LessThanOrEqual64`, apply `Size` then `RemoveSecond` to obtain the boolean width c in {0,1} as a script number, convert with `ScriptNumToLe64`, form mask with `Neg64` and consume its success flag with `Verify`; select X=b XOR ((a XOR b) AND mask) with `BitwiseXor`, `BitwiseAnd`, `BitwiseXor`; apply `Substring` at 0 and 8 to give X an exact eight-byte type admitted as a signed-64 operand by `packages/tapscript/src/stack.rs` | X = min(T, Y−1); require X positive using `GreaterThan64` and `Verify`; minimum comparisons are consumed by selection, domain and positivity comparisons by `Verify`, never confused with arithmetic success flags |
| Residual and successor | Compute R=B−X and `y_l′=y_l−X` with Sub64; consume each success flag immediately and check each result nonnegative and below `2^51` | X ≤ B and X ≤ `y_l`; the residual output shape asserts R>0 with `GreaterThan64` and `Verify`, while the residual-free shape asserts R=0 with `EqualVerify`; these are verified shape choices, not script branches; copy `omega`, `y_t`, `q`, `cycle`, `maturity` unchanged |
| Derived successor | Compute Y′=`y_l′+y_t` with checked Add64 and domain check; compute Y−X with checked Sub64 and domain check | X ≤ Y−1 implies Y′≥1; Y′ is derived, not a second stored write |
| Three balances | Checked recomputation and equality checks enforce B=X+R, `y_l=y_l′+X`, and Y′=Y−X; consume every success flag and every equality result | RHS sums equal authenticated quantities below `2^51`; destruction U/Recon amount is X; no multiplication, division or wide arithmetic required |

The opcode contract is `arithmetic_opcodes` and `comparison_opcodes` in `packages/target-elements/src/opcode.rs`: Add64 and Sub64 consume two signed-64 operands and return result plus success on success; overflow retains operands and pushes false, so immediate verification is mandatory before any result is used. Wrong width and stack underflow abort. LessThanOrEqual64 consumes its operands and returns one comparison boolean; false is a successful comparison result, not overflow. Both minima consume the comparison through `Size`, `RemoveSecond`, `ScriptNumToLe64`, checked `Neg64` and bytewise selection, then re-width the final `BitwiseXor` result with `Substring` at 0 and 8; no conditional opcode is used. `conversion_opcodes`, `stack_opcodes`, `verification_opcodes` and `byte_string_opcodes` in that file supply the conversion, stack, verification and selection contracts recorded in `packages/target-elements/README.md`. The admitted duplication and reordering primitives are `Duplicate`, `DuplicateTwo`, `CopyOver`, `Swap`, `Rotate`, `RemoveSecond` and `Tuck`; disposal uses `Drop` or `DropTwo`, boolean refusal uses `Verify`, and byte equality can use `EqualVerify`. The abstract validator in `packages/tapscript/src/stack.rs` requires the typing bridges: `Size` pushes the boolean width c in {0,1} as a script number, `RemoveSecond` removes the boolean before `ScriptNumToLe64`, and `Substring` at 0 and 8 gives the untyped selection bytes an exact eight-byte type accepted in signed-64 positions, following the post-bitwise slicing idiom in `packages/tapscript/src/state_announcement.rs`. Wave-7 scheduling must meet those registries and their three-item reach bound: no primitive reads below the third item, so at most two computed values may remain above the next witness; operands that this bound forces Wave 7 to re-derive must be re-authenticated from the admitted inputs, never supplied by a witness. For the selector frame written bottom to top as [a,b,mask], `Rotate`, `Rotate`, `Duplicate`, `Rotate`, `BitwiseXor`, `Rotate`, `BitwiseAnd`, `BitwiseXor` leave exactly [b XOR ((a XOR b) AND mask)]; every read stays within the top three items. Neither conditional control flow, indexed copying nor an alternate stack is admitted. This is a bounded mathematical schedule using admitted primitives; its complete linear stack schedule, byte-order conversion tests, target execution, limits and resource measurements remain Wave 7 (`task:guide15-exec:wave7`).

**Table (Question 9 typed owners)**:

| Source label | Architecture identifier | Model owner | Realization owner or absence | Stored / derived / evidence | Inherited test | Acceptance clause | Consuming wave | Reopening premise |
|---|---|---|---|---|---|---|---|---|
| (`q:guide15:state-generation`) | `packages/architecture/src/ids.rs`: `ObjectId::State`, `OperationId::{AnnounceMaturity,Clear}` | `packages/model/src/pool.rs`: `PoolState`; generation is not a stored semantic field | `packages/realization/src/state.rs`: `StateMetadata`; `packages/realization/src/state_codec.rs`: `STATE_METADATA_LAYOUT`, `encode_state_metadata`, `decode_state_metadata` | Semantic fields stored; nonce and codec are representation; exact static subtree plus constructor policy determines generation subject, not enum alone | `packages/model/src/tests/state_metadata_conformance_tests.rs`: `both_censuses_have_six_fields`, `successor_projects_to_realization_successor`; metadata round trips retain all six fields | Rulings 131 and 146, (`task:guide15-exec:disposition-9`), (`cond:guide15:generation-policy`), (`cond:guide15:origin-nonclaims`) | Wave 7 | Changed static leaf set or proposed migration |

**Table (Generation manifest)**:

| Subject | Exact recipe and scope | Evidence or obligation |
|---|---|---|
| Retained predecessor | `packages/linker/src/state_bundle.rs`: `linked_subtree` makes a single `StateStaticNode::Leaf` with identity 0 from the linked Announcement program; `CandidateLinkedMaturityBundle::static_subtree` retains it; `packages/tapscript/src/state_constructor.rs`: `CandidateStateConstructor`, `StateBranchSide::MetadataLeftStaticRight`, `StateConstructorGeneration::CanonicalMetadataV1` | Maturity-only static subtree; metadata commitment is the separate outer child. Exact inherited subject remains the accepted variable-metadata control recorded at (`rec:backlog:wave0-entry`), not an arbitrary Announcement program |
| Metadata codec | `packages/realization/src/state_codec.rs`: domain `tripod/state-metadata`, schema 1, fixed 86 bytes; `STATE_METADATA_LAYOUT` has domain 0–20, schema 21–24, omega 25–32, `y_l` 33–40, `y_t` 41–48, q 49–56, cycle 57–64, maturity 65–73, nonce 74–77, zero reserved bytes 78–85; integers big-endian | `EncodedStateMetadata` separates `semantic` from `representation`; strict decoding rejects wrong domain/schema/width/discriminant/payload/reserved/trailing bytes; metadata is not a generation-crossing permission |
| Proposed expanded static set | Maturity and Clear execution leaves under a shared static subtree, with metadata commitment retained separately; exact linked bytes, paths and complete selected shape set await Wave-7 construction | No deployed-generation claim and no completed leaf-set implementation; `CanonicalMetadataV1` alone cannot identify both old and new static sets. Synthetic new origin, public reconstruction and maturity revalidation are required |
| Generation isolation | Old maturity-only output cannot use an uncommitted Clear leaf; new constructor cannot inherit an old acceptance by matching semantic metadata alone | Wave-7 uncommitted-leaf refusal and continuity tests owed; no migration relation or old-to-new spend is admitted by ruling 131 |

**Table (Question 10 typed owners)**:

| Source label | Architecture identifier | Model owner | Realization owner or absence | Stored / derived / evidence | Inherited test | Acceptance clause | Consuming wave | Reopening premise |
|---|---|---|---|---|---|---|---|---|
| (`q:guide15:event-independence`) | `packages/architecture/src/ids.rs`: `ProjectionId::{TransitionCertificate,BurnEvent,ClearEvent,DistributionResidue}`; `packages/architecture/src/spec.rs`: `QuantityId::{AttestationDelta,AttestationMap}` | `packages/model/src/history.rs`: `TransitionCertificate`, `BurnProjection`, `ClearProjection`; `packages/model/src/ledger.rs`: `RecognizedAttestationEvent`, `compare_attestation_events`, `compare_attestation_query`; `packages/model/src/audit.rs`: `ReceiptAccountingAuditProjection`, `receipt_accounting_audit` | `packages/realization/src/observation.rs`: `OperationObservation`; compact projection policy requires only TransitionCertificate | Certificate and events retain derived evidence; raw records, event recognition, record credit, query and accounting are separate claims, not independent mutable authority | `packages/model/src/tests/burn_attestation_tests.rs`: `compacted_ash_never_creates_another_burn_event`, `bare_burn_tag_without_burn_projection_attests_nothing`; `packages/model/src/tests/indexer_conformance_tests.rs`: `query_mismatch_with_equal_event_snapshot_is_detected`, `missing_clear_with_empty_query_is_detected` | Ruling 147, (`cond:guide15:separate-evidence`) | Waves 10–12 (`task:guide15-exec:wave10`), (`task:guide15-exec:wave11`), (`task:guide15-exec:wave12`) | Merged role or changed observation binding |

The certificate field census in `packages/model/src/history.rs` is `txid`, `order`, `branch`, `consumed`, `created`, `state_edge`, `resv_edge`, `pace_edge`, `entitlement_authority_edge`, `distribution_authority_edge`, `canonical_partition`, `open_flows`, `chain_fee`, `burn`, `clear`, `distribution_residue`. Root edges use `RootEdge`; the canonical partition retains certified flows, their source/destination partition and destructions. Burn projection retains `ash_output`, `ash_value`, `records`; Clear projection retains successor `omega` and derived `y`. These are certification outputs, not new STATE fields.

`BurnTransaction` in `packages/model/src/ledger.rs` retains `txid`, `block_hash`, `order`, `ash_value`, `records`; `ClearEntry` retains `clear_id`, `block_hash`, `order`, `omega`, `y`. `RecognizedAttestationEvent` distinguishes GenesisClear, Burn and Clear. `ReceiptAccountingAuditProjection` in `packages/model/src/audit.rs` derives `residue_events`, `historical_live_residue`, `historical_time_locked_residue`, `current_y_live`, `current_y_time_locked`, `circulating_live`, `circulating_time_locked`, `ash`, `distribution_live`, `distribution_time_locked`. None of these historical or audit totals is a new accumulator in STATE. Operation acceptance, raw event equality, query equality, accounting equality, root history, continuity, safety and resources keep separate evidence producers; Phase-7 producer delivery remains Waves 10–12 work.

**Table (Question 11 typed owners)**:

| Source label | Architecture identifier | Model owner | Realization owner or absence | Stored / derived / evidence | Inherited test | Acceptance clause | Consuming wave | Reopening premise |
|---|---|---|---|---|---|---|---|---|
| (`q:guide15:permissionless`) | `packages/architecture/src/spec.rs`: CompactAsh and Clear `PermissionClass::Permissionless` | `packages/model/src/ops/ash.rs`: `CompactAsh`, `ClearAsh` carry ASH references and fee envelope, no ASH creator secret | `packages/realization/src/constructibility.rs`: `ConstructibilityAuthorization`, `AvailabilityClass`; `packages/realization/src/relation.rs`: `ConstructibilityClass::PublicPermissionless`; `packages/realization/src/declarations/compact_ash.rs`: constructibility graph | Public construction inputs and sponsor-local capability are distinct; no hidden protocol secret is admitted | `packages/realization/src/tests/constructibility_tests.rs`: `permissionless_private_dependency_fails`, `permissionless_private_dependency_cannot_be_laundered_through_public_fact`, `sponsor_local_dependency_must_stay_on_sponsor_edge` | Ruling 148, (`task:guide15-exec:disposition-11`), (`cond:guide15:public-ash`) | Waves 5 and 11 | Hidden dependency or changed constructor input |

**Table (Fresh-process construction input contract)**:

| Input or dependency | Admitted source and role | Required test reading |
|---|---|---|
| Chain objects | Authenticated public transactions, ASH outpoints/assets/explicit values, spent-output census and STATE singleton metadata | Start a fresh process with no prior construction cache; recover and authenticate every selected object from supplied public evidence |
| Program and deployment | Exact reviewed target, deployment assets and parameters, selected linked bundle, ABI, public internal-key policy, leaf scripts, paths, output key/parity and representation nonce where applicable | Reconstruct the same constructor and required control paths; missing or inconsistent public material refuses, rather than requesting an ASH creation secret |
| Operation parameters | Public ASH selection, bound parameters, Clear predecessor STATE and public fee policy | Derive B, Y, X, residual and successor; no secret or witness chooses the clamp |
| Sponsor capability | Optional isolated fee-sponsor offer, funding and per-input signing capability available only to the sponsor construction boundary | No sponsor input authorizes protocol ASH spending; sponsor secrets and openings do not become protocol witnesses or retained public evidence |
| Forbidden dependency | ASH creation signing key, private ASH opening, earlier-process memory, hidden nonce cache or private constructor state | Remove all such material and still construct from the admitted public inputs; any required dependency fails the contract |
| Retained result | Exact reconstructed transaction and witness bindings, public recovery inputs, explicit dependency census and separate target result if executed | Test both sponsorless and selected sponsored forms; fresh-to-compact-to-clear success remains owed in Waves 5 and 11, not observed by this packet |

**Table (Question 12 typed owners)**:

| Source label | Architecture identifier | Model owner | Realization owner or absence | Stored / derived / evidence | Inherited test | Acceptance clause | Consuming wave | Reopening premise |
|---|---|---|---|---|---|---|---|---|
| (`q:guide15:attestation-query`) | `packages/architecture/src/spec.rs`: AttestationDelta and AttestationMap are derived/interface quantities without operation writer | `packages/model/src/ledger.rs`: `ReferenceIndexer`, `AttestationContext`, `AttestationTerm`, `AttestationQueryResult`, `compare_attestation_events`, `compare_attestation_query` | No external-consumer realization declaration | Query derived from ordered recognized events and checkpoint context; no sampled Clear identifier stored in BurnRecord | `packages/model/src/tests/attestation_reorg_tests.rs`: `same_raw_burn_reprojects_across_clear_boundary`, `reorg_leaves_the_raw_burn_record_unchanged_in_both_directions`; `packages/model/src/tests/indexer_conformance_tests.rs`: `identical_event_sets_produce_identical_query` | (`rule:guide15-exec:disposition-12`) | Internal projection retained; event inputs in Waves 10 and 12 (`task:guide15-exec:wave10`), (`task:guide15-exec:wave12`); no separate later query wave assigned | Admitted external consumer or changed context/query semantics |

`AttestationContext` retains network, genesis, architecture-manifest identity, checkpoint block and height, and schema version; `AttestationTerm` derives `clear_id`, `clear_order`, `aggregate_burn_amount`, `omega`, `y`; `AttestationQueryResult` contains context, address and terms. The independent event and query comparisons retain separate premises even when both projections are empty. Internal projection is the default; no new interchange identity is allocated.

This packet establishes the code census, source-level schedule argument, proposed carrier grammar and accepted design boundaries. It supplies no new native observation, baseline gate pass, fresh/compacted/residual byte compatibility, deployed generation or complete preflight verdict. Missing Burn/Clear declaration validation remains Wave 2; Waves 5/8 constructor equality, Wave-7 arithmetic and generation execution, internal query projections over Waves 10 and 12 event inputs and Waves 10–12 evidence producers remain explicit obligations. The register join and final baseline record must retain those distinctions.

**Record (Wave 0 preflight execution exits)** · `rec:backlog:wave0-preflight-exits`

This join binds each source entry in (`tab:guide15-exec:preflight`) to its Wave-0 execution exit and retains its historical finding, evidence class and evidence scope under (`rule:guide15-exec:register-exits`). Numbered acceptance clauses refer to (`rule:phase7:wave0-rulings`). RECORDED means the stated preflight record exists at its own scope; BASELINE PENDING names an exit still conditional on the completed-tree gate and its record. No row is promoted: UNDECIDED remains UNDECIDED beside its accepted position, and DOES NOT HOLD remains beside its Wave-2 obligation. Subsequent records reconcile historical scope wording without rewriting the original finding. (`T12-007`) remains TODO; no exit claims an observation, gate pass or verdict before its evidence exists.

| Register label | Wave-0 execution exit | Acceptance clause | Retained evidence class | Retained consuming obligation |
|---|---|---|---|---|
| (`cond:guide15:phase6-exited`) | RECORDED: Amended exit plus Safety’s five-conjunct reconciliation. (`sec:phase6:exit-assessment`); (`result:backlog:bite-0-5-gate-pass`) | Ruling 135: non-answer and native gate readings at (`gate:guide15-exec:bite-0-5-non-answer-gate`) and (`gate:guide15-exec:bite-0-5-native-gate`). | inherited policy | — |
| (`cond:guide15:clean-tree`) | RECORDED: Entry classification plus fresh baseline clean-tree result. (`rec:backlog:wave0-entry`); (`gate:guide15-exec:baseline-repository`); (`rec:backlog:wave0-baseline`) | No new position ruling required; inherited evidence retained. | structural implementation | — |
| (`cond:guide15:operation-scope`) | RECORDED: Exact identifiers and missing dispatch recorded; finding stays DOES NOT HOLD. (`rec:backlog:wave0-preflight-census`) | Ruling 140: (`task:guide15-exec:disposition-1`). | new Phase-7 choice | Wave 2: Implement missing Burn and Clear declaration dispatch and architecture weld. |
| (`cond:guide15:validated-relations`) | RECORDED: Complete model-law census and absent declaration validation recorded; finding stays DOES NOT HOLD. (`rec:backlog:wave0-preflight-census`) | Ruling 140: (`task:guide15-exec:disposition-1`). | new Phase-7 choice | Wave 2: Validate Burn and Clear declarations against architecture and independent model conformance. |
| (`cond:guide15:sponsor-opacity`) | RECORDED: Region refit and sponsored closure. (`rec:backlog:wave0-region-refit`); (`result:backlog:bite-0-5-gate-pass`) | Ruling 130: region-scoped sponsored construction at (`rec:backlog:wave0-sponsored-ruling`); Ruling 135: non-answer and native gate readings at (`gate:guide15-exec:bite-0-5-non-answer-gate`) and (`gate:guide15-exec:bite-0-5-native-gate`). | new Phase-7 choice | Waves 3, 6, 7: Prove operation-specific sponsor opacity for burn, compact and clear. |
| (`cond:guide15:target-independent`) | RECORDED: Census confirms semantic-only realization boundary. (`rec:backlog:wave0-preflight-census`) | Ruling 140: (`task:guide15-exec:disposition-1`). | inherited policy | Wave 2: Keep target-specific types outside the realization declaration boundary. |
| (`cond:guide15:lifecycle-obligations`) | RECORDED: Exact exit declarations and incomplete constructors recorded. (`rec:backlog:wave0-preflight-census`) | Ruling 140: (`task:guide15-exec:disposition-1`); Ruling 142: (`task:guide15-exec:disposition-4`) and (`cond:guide15:ash-compatibility`). | inherited policy | Waves 5, 11: Prove actual constructor and lifecycle closure. |
| (`data:guide15:starting-revision`) | RECORDED: Entry’s actual `0.7.9-dev`, workspace `0.7.0-dev`. (`rec:backlog:wave0-entry`) | No new position ruling required; inherited evidence retained. | structural implementation | — |
| (`data:guide15:phase6-revision`) | RECORDED: Entry’s retained Phase-6 evidence and amended closeout references. (`rec:backlog:wave0-entry`) | No new position ruling required; inherited evidence retained. | observed candidate evidence | — |
| (`data:guide15:architecture-values`) | RECORDED: Schema 18 and generated registered fields cited by path. (`rec:backlog:wave0-entry`); (`rec:backlog:wave0-preflight-census`) | No new position ruling required; inherited evidence retained. | structural implementation | — |
| (`data:guide15:realization-version`) | RECORDED: Derived `0.7.0-dev`; no new identity. (`rec:backlog:wave0-entry`) | No new position ruling required; inherited evidence retained. | structural implementation | — |
| (`data:guide15:clean-result`) | RECORDED: Fresh baseline result, not copied entry pass. (`gate:guide15-exec:baseline-repository`); (`rec:backlog:wave0-baseline`) | No new position ruling required; inherited evidence retained. | observed candidate evidence | — |
| (`cond:guide15:public-revalidation`) | RECORDED: Recorded explicit-only boundary revalidated. (`rec:backlog:wave0-preflight-census`) | Ruling 141: (`cond:guide15:public-revalidation`) and (`cond:guide15:private-route`). | new Phase-7 choice | — |
| (`cond:guide15:ash-selection`) | RECORDED: Explicit-only selection recorded. (`rec:backlog:wave0-preflight-census`) | Ruling 141: (`cond:guide15:public-revalidation`) and (`cond:guide15:private-route`); (`rule:guide15-exec:disposition-2`). | inherited policy | Wave 4: Assess the actual selected Phase-7 representation forms. |
| (`cond:guide15:authenticated-opening`) | RECORDED: Unsupported opening and refusal of scalar substitutes retained. (`rec:backlog:wave0-preflight-census`) | Ruling 141: (`cond:guide15:public-revalidation`) and (`cond:guide15:private-route`). | inherited policy | — |
| (`cond:guide15:public-value`) | RECORDED: Chain-public value authority retained. (`rec:backlog:wave0-preflight-census`) | Ruling 141: (`cond:guide15:public-revalidation`) and (`cond:guide15:private-route`). | inherited policy | — |
| (`cond:guide15:value-introspection`) | RECORDED: Reviewed explicit asset/value capabilities recorded. (`rec:backlog:wave0-preflight-census`); (`pkg:target-elements:contract`) | Ruling 141: (`cond:guide15:public-revalidation`) and (`cond:guide15:private-route`). | structural implementation | Wave 4: Assess explicit U/value introspection for each selected form. |
| (`data:guide15:public-policy`) | RECORDED: Explicit-only, normalized-first, private-change policy recorded. (`rec:backlog:wave0-preflight-census`) | Ruling 141: (`cond:guide15:public-revalidation`) and (`cond:guide15:private-route`). | inherited policy | — |
| (`cond:guide15:private-route`) | RECORDED: Normalized-first selection. (`rec:backlog:wave0-preflight-census`) | Ruling 141: (`cond:guide15:public-revalidation`) and (`cond:guide15:private-route`). | new Phase-7 choice | Wave 4: Assess the normalized-first route; direct-private remains separately admitted only. |
| (`cond:guide15:compact-build`) | RECORDED: Selected compact subject rebuilt by completed-tree baseline. (`rec:backlog:wave0-compact-inherited-closure`); (`gate:guide15-exec:baseline-repository`); (`rec:backlog:wave0-baseline`) | No new position ruling required; inherited evidence retained. | new Phase-7 choice | — |
| (`cond:guide15:compact-regression`) | RECORDED: Current compiler/pattern/linker/ABI/vector/resource regressions in baseline. (`rec:backlog:wave0-compact-inherited-closure`); (`gate:guide15-exec:baseline-repository`); (`rec:backlog:wave0-baseline`) | No new position ruling required; inherited evidence retained. | new Phase-7 choice | — |
| (`cond:guide15:ash-compatibility`) | RECORDED: Exact-equality intent and proof boundary recorded. (`rec:backlog:wave0-preflight-census`) | Ruling 142: (`task:guide15-exec:disposition-4`) and (`cond:guide15:ash-compatibility`). | new Phase-7 choice | Waves 5, 8: Prove exact constructor equality on actual linked subjects. |
| (`data:guide15:compact-revision`) | RECORDED: Selected candidate pins per ruling 128 and compatibility joined. (`rec:backlog:wave0-compact-inherited-closure`) | Ruling 128: (`data:guide15:compact-revision`). | new Phase-7 choice | — |
| (`cond:guide15:operator-signing`) | RECORDED: Finalized-byte boundary retained. (`rec:backlog:wave0-preflight-census`) | Ruling 144: (`task:guide15-exec:disposition-5`) and (`cond:guide15:signing-profiles`). | structural implementation | Wave 9: Prove Burn signatures bind every finalized byte role. |
| (`cond:guide15:signing-profiles`) | RECORDED: Exact selected profiles and protected roles. (`rec:backlog:wave0-preflight-census`) | Ruling 144: (`task:guide15-exec:disposition-5`) and (`cond:guide15:signing-profiles`). | new Phase-7 choice | Waves 4, 9: Assess selected profiles and execute Phase-7 signing and mutation refusals. |
| (`cond:guide15:typed-records`) | RECORDED: Model record/event law recorded. (`rec:backlog:wave0-preflight-census`) | Ruling 140: (`task:guide15-exec:disposition-1`); Ruling 143: (`task:guide15-exec:disposition-6`); Ruling 147: (`cond:guide15:separate-evidence`). | structural implementation | Waves 2, 10: Deliver typed record/event declarations and independently bound reports. |
| (`cond:guide15:state-owner`) | RECORDED: Six stored fields and codec ownership recorded. (`rec:backlog:wave0-preflight-census`) | Ruling 140: (`task:guide15-exec:disposition-1`). | structural implementation | Wave 2: Validate complete metadata ownership against independent model conformance. |
| (`cond:guide15:clear-assignment`) | RECORDED: Only stored `y_l` changes; Y derived. (`rec:backlog:wave0-preflight-census`) | Ruling 140: (`task:guide15-exec:disposition-1`); (`rule:guide15-exec:disposition-7`). | structural implementation | Wave 2: Validate the stored-versus-derived assignment against independent model conformance. |
| (`cond:guide15:recon-tag`) | RECORDED: Exact `TagId::Recon` ownership recorded. (`rec:backlog:wave0-preflight-census`) | Ruling 140: (`task:guide15-exec:disposition-1`). | structural implementation | Waves 2, 7: Bind declarations and destruction to the exact U/Recon tag. |
| (`cond:guide15:candidate-limits`) | RECORDED: Bound subject and non-calibration status retained. (`rec:backlog:wave0-compact-inherited-closure`); (`rec:backlog:wave0-preflight-census`); (`sec:phase6:exit-assessment`) | No new position ruling required; inherited evidence retained. | inherited policy | — |
| (`cond:guide15:target-definition`) | RECORDED: Reviewed V2 definition bound and validated in baseline. (`rec:backlog:wave0-entry`); (`gate:guide15-exec:baseline-repository`); (`rec:backlog:wave0-baseline`) | No new position ruling required; inherited evidence retained. | structural implementation | — |
| (`cond:guide15:development-binding`) | RECORDED: Entry’s exact development binding retained. (`rec:backlog:wave0-entry`) | No new position ruling required; inherited evidence retained. | structural implementation | — |
| (`cond:guide15:fixed-arithmetic`) | RECORDED: Bounded checked schedule accepted. (`rec:backlog:wave0-preflight-census`) | Ruling 145: (`task:guide15-exec:disposition-8`). | structural implementation | Wave 7: Emit and execute the composed bounded checked minimum schedule. |
| (`cond:guide15:target-conservation`) | RECORDED: External target dependency distinguished from semantic proof. (`rec:backlog:wave0-preflight-census`) | Ruling 140: (`task:guide15-exec:disposition-1`). | inherited policy | Wave 3: Retain target conservation as external evidence, separate from semantic proof. |
| (`cond:guide15:execution-domain`) | RECORDED: Entry/Safety domain and leaf-version observations retained. (`rec:backlog:wave0-entry`); (`result:backlog:bite-0-5-gate-pass`) | No new position ruling required; inherited evidence retained. | observed candidate evidence | — |
| (`cond:guide15:relay-separation`) | RECORDED: Positive-floor evidence and relay/block distinctions retained through Safety. (`result:backlog:bite-0-5-gate-pass`) | No new position ruling required; inherited evidence retained. | inherited policy | — |
| (`data:guide15:target-revision`) | RECORDED: Entry’s target/deployment/provenance fields joined. (`rec:backlog:wave0-entry`) | No new position ruling required; inherited evidence retained. | structural implementation | — |
| (`cond:guide15:maturity-green`) | RECORDED: Entry and Safety evidence plus current baseline regression. (`rec:backlog:wave0-entry`); (`result:backlog:bite-0-5-gate-pass`); (`gate:guide15-exec:baseline-repository`); (`rec:backlog:wave0-baseline`) | No new position ruling required; inherited evidence retained. | new Phase-7 choice | Wave 7: Revalidate maturity under the expanded constructor generation. |
| (`cond:guide15:constructor-reconstruction`) | RECORDED: Safety’s constructor and recovery evidence retained. (`result:backlog:bite-0-5-gate-pass`) | No new position ruling required; inherited evidence retained. | observed candidate evidence | — |
| (`cond:guide15:generation-identity`) | RECORDED: Exact maturity-only subtree and constructor manifest recorded. (`rec:backlog:wave0-preflight-census`) | Ruling 146: (`task:guide15-exec:disposition-9`), (`cond:guide15:generation-policy`) and (`cond:guide15:origin-nonclaims`). | structural implementation | — |
| (`cond:guide15:clear-subtree`) | RECORDED: Distinct maturity-plus-clear static set recorded. (`rec:backlog:wave0-preflight-census`) | Ruling 146: (`task:guide15-exec:disposition-9`), (`cond:guide15:generation-policy`) and (`cond:guide15:origin-nonclaims`). | inherited policy | Wave 7: Implement and validate the expanded static subtree. |
| (`cond:guide15:generation-policy`) | RECORDED: Explicit synthetic-origin acceptance. (`rec:backlog:wave0-preflight-census`) | Ruling 146: (`task:guide15-exec:disposition-9`), (`cond:guide15:generation-policy`) and (`cond:guide15:origin-nonclaims`); Ruling 131: (`task:guide15-exec:disposition-9`). | new Phase-7 choice | — |
| (`cond:guide15:committed-leaf`) | RECORDED: No-generation-crossing rule recorded. (`rec:backlog:wave0-preflight-census`) | Ruling 146: (`task:guide15-exec:disposition-9`), (`cond:guide15:generation-policy`) and (`cond:guide15:origin-nonclaims`). | inherited policy | Wave 7: Prove refusal of uncommitted Clear leaves and generation crossing. |
| (`cond:guide15:origin-nonclaims`) | RECORDED: Explicit no-migration/no-old-generation-spend clauses. (`rec:backlog:wave0-preflight-census`) | Ruling 146: (`task:guide15-exec:disposition-9`), (`cond:guide15:generation-policy`) and (`cond:guide15:origin-nonclaims`). | new Phase-7 choice | — |
| (`cond:guide15:constructor-hashing`) | RECORDED: Existing hash domains and tweak/parity requirements retained under rulings 74 and 112 and the gate record. (`rule:phase6:wave9-rulings`); (`rule:phase6:wave13-rulings`); (`sec:phase6:gate-record`). New-subtree proof boundary recorded at (`rec:backlog:wave0-preflight-census`) | Ruling 146: (`task:guide15-exec:disposition-9`), (`cond:guide15:generation-policy`) and (`cond:guide15:origin-nonclaims`). | structural implementation | Wave 7: Prove hashing and tweak requirements for the expanded committed subtree. |
| (`data:guide15:state-generation`) | RECORDED: Predecessor and proposed generation manifests joined. (`rec:backlog:wave0-preflight-census`) | Ruling 146: (`task:guide15-exec:disposition-9`), (`cond:guide15:generation-policy`) and (`cond:guide15:origin-nonclaims`); Ruling 131: (`task:guide15-exec:disposition-9`). | structural implementation | — |
| (`cond:guide15:edge-history`) | RECORDED: Independent edge evidence and checkpoint limits joined through Safety. (`result:backlog:bite-0-5-gate-pass`) | No new position ruling required; inherited evidence retained. | observed candidate evidence | — |
| (`cond:guide15:exact-bindings`) | RECORDED: Entry and admitted observation bindings retained. (`rec:backlog:wave0-entry`); (`result:backlog:bite-0-5-gate-pass`) | No new position ruling required; inherited evidence retained. | observed candidate evidence | — |
| (`cond:guide15:compact-silence`) | RECORDED: Complete inherited projection/lifecycle census. (`rec:backlog:wave0-preflight-census`) | Ruling 147: (`cond:guide15:separate-evidence`); (`rule:guide15-exec:disposition-10`). | structural implementation | Wave 11: Reproduce the complete compact event and lifecycle projection. |
| (`cond:guide15:native-eligibility`) | RECORDED: Real-target admission boundary retained; Safety. (`result:backlog:bite-0-5-gate-pass`) | No new position ruling required; inherited evidence retained. | inherited policy | — |
| (`cond:guide15:validated-plans`) | RECORDED: Validated-plan construction and exact row disposition retained; Safety. (`result:backlog:bite-0-5-gate-pass`) | No new position ruling required; inherited evidence retained. | structural implementation | — |
| (`cond:guide15:validated-coverage`) | RECORDED: Validated wrappers and recounted categories retained; Safety. (`result:backlog:bite-0-5-gate-pass`) | No new position ruling required; inherited evidence retained. | structural implementation | — |
| (`cond:guide15:observed-refusals`) | RECORDED: Exact subjects and observed layers retained. (`result:backlog:bite-0-5-gate-pass`) | Ruling 134: diagnostic attribution at (`req:w0route:recapture-constraints`); Ruling 135: non-answer and native gate readings at (`gate:guide15-exec:bite-0-5-non-answer-gate`) and (`gate:guide15-exec:bite-0-5-native-gate`). | structural implementation | — |
| (`cond:guide15:exact-refusals`) | RECORDED: Matching-boundary credit and wrong-boundary non-credit retained; Safety. (`result:backlog:bite-0-5-gate-pass`) | No new position ruling required; inherited evidence retained. | inherited policy | — |
| (`cond:guide15:observed-evidence`) | RECORDED: Fifteen report-consumer dispositions joined; no target-observation promotion. (`result:backlog:bite-0-5-gate-pass`) | No new position ruling required; inherited evidence retained. | observed candidate evidence | — |
| (`cond:guide15:accepted-controls`) | RECORDED: Accepted family controls and exact associations retained; Safety. (`result:backlog:bite-0-5-gate-pass`) | No new position ruling required; inherited evidence retained. | structural implementation | — |
| (`cond:guide15:rejection-artifacts`) | RECORDED: Validated refusal response shapes retained; Safety. (`result:backlog:bite-0-5-gate-pass`) | No new position ruling required; inherited evidence retained. | structural implementation | — |
| (`cond:guide15:response-shape`) | RECORDED: Protocol-8 response validation joined to admitted evidence. (`result:backlog:bite-0-5-gate-pass`); (`rec:backlog:wave0-entry`) | No new position ruling required; inherited evidence retained. | structural implementation | — |
| (`cond:guide15:retained-transcripts`) | RECORDED: Complete admitted generic transcripts retained; Safety. (`result:backlog:bite-0-5-gate-pass`) | No new position ruling required; inherited evidence retained. | structural implementation | — |
| (`cond:guide15:separate-evidence`) | RECORDED: Separate role/producer/consumer contract. (`rec:backlog:wave0-preflight-census`) | Ruling 147: (`cond:guide15:separate-evidence`). | new Phase-7 choice | Waves 10–12: Deliver separate validated Phase-7 evidence producers. |
| (`data:guide15:executor-revision`) | RECORDED: Protocol 8 and adapter binding from entry; historical revisions preserved. (`rec:backlog:wave0-entry`) | No new position ruling required; inherited evidence retained. | structural implementation | — |
| (`cond:guide15:public-successor`) | RECORDED: Independently acquired public recovery evidence and import boundary; Safety. (`result:backlog:bite-0-5-gate-pass`) | No new position ruling required; inherited evidence retained. | observed candidate evidence | — |
| (`cond:guide15:public-ash`) | RECORDED: Public-only fresh-process test contract. (`rec:backlog:wave0-preflight-census`) | Ruling 148: (`task:guide15-exec:disposition-11`) and (`cond:guide15:public-ash`). | new Phase-7 choice | Waves 5, 11: Prove the complete public-only fresh-to-compact-to-clear lifecycle. |
| (`cond:guide15:compact-authorization`) | RECORDED: Compact authorization absence revalidated. (`rec:backlog:wave0-compact-inherited-closure`) | Ruling 128: (`data:guide15:compact-revision`). | structural implementation | Wave 11: Revalidate permissionless compact execution and separate optional sponsor authorization. |
| (`cond:guide15:candidate-distinction`) | RECORDED: Candidate-only, no deployment/release/calibration claims. (`rec:backlog:wave0-preflight-census`); (`sec:phase6:exit-assessment`) | No new position ruling required; inherited evidence retained. | inherited policy | — |

The join records 65 exits, with 40 rows naming bite 0.7 and 25 consuming-wave obligations retained. Finding counts remain 50 HOLDS / 13 UNDECIDED / 2 DOES NOT HOLD; the register retains fourteen columns, its label order, evidence classes and evidence scopes.

**Record (Wave 0 baseline)** · `rec:backlog:wave0-baseline`

The completed tree at `0.7.61-dev` passes (`gate:guide15-exec:baseline-repository`). This record closes (`gate:guide15-exec:wave0`) on the nine conditions below, consolidating the inherited closures without changing their subjects or promoting formal dispositions into observations. The subject manifest follows (`def:guide15-exec:baseline-subject`); historical TODO and pending statements in earlier records retain their original scope and are superseded for consolidation by this record. This record-only landing takes effect through its own import gate; the completed-tree gate cited here precedes the record.

| Subject field | Exact binding and evidence |
|---|---|
| Target | Reviewed `TargetContractVersion::V2`, `packages/target-elements/src/definition.rs`, bound by development validation in `packages/target-elements/src/deployment.rs`; executable and source verified against the selected toolchain's `PROVENANCE.txt` in (`rec:backlog:wave0-entry`); (`data:backlog:entry-target-revision`). |
| Deployment | Inherited development deployment bound by the capture's `deployment-genesis-id` and `deployment-target-contract`; `elementsregtest`, active tapscript, leaf version 196. The entry's handshake node, executable, intended-tip and framework fields retain the exact binding; no deployment is introduced (`rec:backlog:wave0-entry`); (`data:backlog:entry-target-revision`). |
| Policy | Explicit-only backend boundary, normalized-first private burn with private change; PublicCommitted and authenticated direct-private opening deferred on `GeneratorNotDerivableOnScript`, `EncodingDomainMismatch` and `SuppliedParityUnbound`. Carrier and signing forms remain proposed until measured; rulings 141 and 144 and (`rec:backlog:wave0-preflight-census`) retain those boundaries. Native observations keep their individual fee-floor and representation premises rather than borrowing the repository gate's outcome. |
| Candidate | Completed repository tree `0.7.61-dev`, workspace `0.7.0-dev`, as `subject.describe` and `subject.workspace_version` in `manifest.txt`. Compact is ruling 128's selected `0.7.16-dev` program, pinned by `FIXTURE_MERKLE_ROOT`, `FIXTURE_REFERENCE_OUTPUT_KEY` and `PINNED_PROGRAM`, with closure at (`rec:backlog:wave0-compact-inherited-closure`); (`rule:phase7:wave0-rulings`). Maturity retains the separate variable-metadata control at (`rec:backlog:wave0-entry`), not the compact pins. |
| Protocol | `NATIVE_PROTOCOL_SCHEMA = 8`, `elements-native-executor` version `2.1.0`, with adapter and handshake binding at (`data:backlog:entry-executor-revision`); archived captures retain their producing revisions. Safety report schema 3 remains a separate identity at (`result:backlog:close-safety-reconciliation`). |
| Constructor generation | Retained maturity-only `CandidateLinkedMaturityBundle::static_subtree`, single Announcement leaf identity 0, `StateBranchSide::MetadataLeftStaticRight`, `StateConstructorGeneration::CanonicalMetadataV1`; metadata domain `tripod/state-metadata`, schema 1, fixed 86-byte codec. The maturity-plus-Clear static set is proposed, starts synthetically and has no deployed-generation or migration claim; its linked bytes, uncommitted-leaf refusal and maturity revalidation remain Wave 7 work (`rec:backlog:wave0-preflight-census`), rulings 131 and 146. |
| Evidence corpus | Admitted archives and exact homes resolved through (`rec:backlog:wave0-entry`), with Safety closure at (`result:backlog:bite-0-5-gate-pass`) and the two compact archives at (`rec:backlog:wave0-compact-inherited-closure`). Baseline home, relative to the execution directory identified by (`rec:backlog:wave0-entry`): `evidence/wave0/baseline/20261103T131720Z-completed-tree/`. `gate.log`, `manifest.txt`, retained `ci-meson-testlog.json`, `baseline-20261103T131720Z-completed-tree-build-testlog.json`, and `baseline-20261103T131720Z-completed-tree-build-testlog.txt`. The gate log prints sixteen-hexadecimal digest prefixes of `manifest.txt` and the three retained Meson logs; the gate writes no digest of `gate.log` or of its gate script. The tracked tree carries no digest of any of them, because nothing in the tree can regenerate or resolve one. |
| Commands | `baseline-gate.sh`; exact command surface in the battery table and resolved paths in `manifest.txt`. Toolchain: rustc and Cargo `1.101.0-nightly`, Meson `1.12.1`, Ninja `1.11.1`, XeTeX `3.141592653-2.6-0.999998` from TeX Live 2026, biber `2.22`, latexmk `4.88`. The shared clone was reset and cleaned, the shared Cargo cache warm, and both Meson directories cold; each battery runs under `chrt --idle 0 nice -n 19 ionice -c3`, or under `nice -n 19 ionice -c3` when the idle-class probe is refused; the log does not record which form ran. |
| Clean-tree state | `subject.clean_tree_before=0 entries`; `git diff --check` records `diff-check ok`; before and after status checks record `clean-tree-before ok` and `clean-tree-after ok`; `== chain-rc=0` and `== baseline green`. The baseline subject is the completed tree, not an uncommitted record draft (`manifest.txt`; `gate.log`). |
| Measured dimensions | Whole gate wall 5,132 s; own CPU user 22,550.5 s / system 988.4 s; 61 available threads. Gate start `20261103T131723Z`, first battery start `20261103T131724Z`, end `20261103T144255Z`; node load averages (1/5/15 minutes) 71.25/78.04/68.05 → 26.13/41.38/42.77. Per-battery wall, own CPU, shared-cgroup samples and deltas are in `gate.log`; no separate clean-check timing or exclusive-node measurement is present. |

The command notation `$B` denotes the clean document build directory selected by `baseline-gate.sh` and fully resolved in `manifest.txt`; archive and generated outputs are redirected below it. Commands below retain the script's arguments, with the directory represented by that variable to keep machine paths out of this record. The shared cgroup carried other concurrent workloads: own CPU is process accounting for the gate and its children, whereas cgroup deltas measure node load. All start/end CPU, pressure and memory samples remain in the retained log. Clean-tree checks are predicates within the gate, with no independently timed battery; their successful empty readings are not invented zero-second measurements. The batch's build-directory compile and test pair ran as ADR-011's document lane in a clean build directory with real TeX, formatting ran in check mode (`cargo fmt --all --check`), and the reproducibility and MSRV and stable lanes, which answer the release question, were not run.

| Battery | Command | Exit / result | Wall s | Own CPU user / system s | Node load average 1 min, start → end; shared CPU usage delta µs |
|---|---|---|---:|---|---|
| Clean-tree before | `git status --porcelain=v1 --untracked-files=all`; `git diff --check` | Successful empty status; diff-check ok; chain rc 0 | Not separately measured | Not separately measured | No separate sample; whole-gate context below |
| ci.sh | `sh scripts/ci.sh` | 0; mocked TeX; Ok 63 / Fail 0 | 1,128 | 4,746.6 / 456.9 | 69.15 → 24.61; +19,881,800,574 |
| Document setup | `meson setup "$B" . -Dpublication_archive_root="$B/archive" -Dartifact_generation_output_dir="$B/generated"` | 0; fresh real-document build | 1 | 0.8 / 0.2 | 24.61 → 24.61; +3,822,656 |
| Document compile | `meson compile -C "$B" attestation` | 0; rendered attestation 182,858 bytes, identical at its three logged output paths | 32 | 27.9 / 3.1 | 24.61 → 27.84; +323,919,955 |
| Document test | `meson test -C "$B" --print-errorlogs` | 0; real TeX; Ok 63 / Fail 0 | 510 | 3,560.8 / 289.4 | 27.84 → 61.72; +11,879,278,616 |
| fmt | `cargo fmt --all --check` | 0 | 5 | 3.9 / 0.2 | 61.72 → 64.87; +132,336,823 |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | 0; targets fresh, `Finished` in 0.33 s | 0 at whole-second resolution | 0.2 / 0.3 | 64.87 → 64.87; +25,685,574 |
| Cargo test workspace | `cargo test --workspace` | 0; 93 result lines: 5,996 passed / 0 failed / 55 ignored; compile 1 min 15 s; largest suite 1,225 passed in 739.49 s | 3,455 | 14,209.3 / 237.2 | 64.87 → 27.79; +32,369,623,370 |
| Clean-tree after | `git status --porcelain=v1 --untracked-files=all` | Successful empty status; clean-tree-after ok; chain rc 0 | Not separately measured | Not separately measured | No separate sample; whole-gate context below |

The log's three rendered paths below `$B` are `papers/attestation/main.pdf`, `papers/attestation/Tripod_Attestation_v1.0.0.pdf` and `archive/rendered/Tripod_Attestation_v1.0.0.pdf`, with equal recorded sizes and digest prefixes. The clippy battery reused targets already checked by the ci.sh lane on the same cache; its rounded wall does not mean that no command ran. Cargo reports unused workspace dependencies `artifacts`, `execwrap` and `flatten-latex-main`, and the clippy output also reports unused dependency `nix`. These are Cargo manifest lints at warn level, outside the compiler's `-D warnings`; they are recorded observations, not failed batteries. The script runs every battery and records completion rather than treating a returned shell as a passing gate.

| Whole-gate resource reading | Recorded value |
|---|---|
| Scope and completion | Shared cgroup; 5,132 s; own CPU 22,550.5 / 988.4 s; `== chain-rc=0`; `== baseline green` |
| Shared CPU deltas, µs | usage +64,685,353,716; user +54,296,820,174; system +10,388,533,541 |
| Shared pressure total deltas, µs, some / full | CPU +111,718,760 / +21,455,352; memory +1,405,145 / +1,137,175; IO +7,482,606 / +6,486,242 |
| Shared throttling and idle deltas | core-scheduling force-idle, periods, throttled periods, throttled time, bursts and burst time all zero |

The before/after counters preserve their separate denominators under (`reg:guide15-exec:baseline-inventory`). Earlier category partitions below are reconstructed from the cited keyed census; none converts a final or retired claim into an observed refusal. A repository regression pass rechecks executable comparisons and admitted archives; it does not recapture a native experiment.

| Figure | Before | Wave-0 baseline | Evidence |
|---|---|---|---|
| Safety total / answered / outstanding | 206 / 78 / 128 | 206 / 177 / 29; completeness remains partial because outstanding means not answered | (`sec:phase6:exit-assessment`); (`result:backlog:close-safety-reconciliation`) |
| Categories: observed / final / retired / waiting | 78 / 12 / 0 / 116, reconstructed from the inherited twelve finals and twenty-three typed waits | 177 / 27 / 2 / 0; 177 + 27 + 2 + 0 = 206 | (`result:backlog:close-safety-reconciliation`); (`result:backlog:bite-0-5-gate-pass`) |
| Inherited partition, keyed reconciliation | 78 answered; outstanding 43 first-party / 35 native / 15 report / 35 typed non-answers | 78 historical answers retained; 43 → 41 observed + 2 retired; 35 native → 28 observed + 7 final; 15 report → 15 observed; 35 typed → 15 observed + 20 final. No key dropped; 78 + 41 + 28 + 15 + 15 = 177 | (`rec:backlog:wave0-close-safety`); (`result:backlog:close-safety-reconciliation`); `packages/vectors/src/maturity_evidence.rs`: `assert_historical_partition` |
| Answer evidence classes: first-party / native acceptance / native refusal / report / root history / public recovery | 40 / 1 / 7 / 0 / 16 / 14 | 81 / 12 / 39 / 15 / 16 / 14; first-party required 0, report required 0, typed final / waiting 27 / 0 | (`sec:phase6:exit-assessment`); (`result:backlog:close-safety-reconciliation`) |
| Relation links: every / declared / unlinked | 14 / 102 / 90; non-link reasons 55 / 9 / 9 / 17 | 14 / 108 / 84; reasons 55 / 9 / 9 / 0 / 11; all ninety former non-links dispositioned | (`result:backlog:region-links`); (`tab:backlog:relation-dispositions`); (`result:backlog:close-safety-reconciliation`) |
| Five conjuncts 5 / 7 / 11 / 13 / 14 | Historical NOT PASSED retained | MET by parts: respectively 3/2, 2/1, 1/3, 1/2, 3/1 MET/RETIRED; nineteen parts = 10 MET + 9 RETIRED + 0 WAITING | (`sec:phase6:gate-record`); (`tab:backlog:conjunct-dispositions`); (`result:backlog:bite-0-5-gate-pass`) |
| Four inherited carriers | `T11-064`, `T11-063`, `T4-009`, `T11-110` open at inherited handoff | All four DONE on their own criteria; consolidated by `T12-007` through (`rec:guide15-exec:bite-0-7-closure-carrier`) | (`rec:backlog:wave0-region-refit`); (`result:backlog:bite-0-5-gate-pass`); (`rec:backlog:wave0-closure-audit`); (`rec:backlog:wave0-closure-elision`); (`rec:backlog:wave0-compact-inherited-closure`); (`rec:backlog:wave0-test-performance`) |
| Compact negative population | Historical 72, partition 48 / 18 / 6; intermediate 74, partition 50 / 18 / 6 | 74, partition 48 / 20 / 6; 51 observed + 23 formal, zero retained or unresolved; native [27, 21, 0, 0] | (`result:backlog:closure-census`); (`rec:backlog:wave0-compact-inherited-closure`) |
| Compact first-party / report | First-party 2 discharged / 18 retained after classification; report 0 observed / 6 retained | First-party 20 closed / 0 retained = 18 observed + 2 formal; report 6 observed / 0 retained; two derivation-only discharges unchanged | (`rec:backlog:wave0-compact-inherited-closure`) |
| Missing compact entries | 22 = 16 + 6 before classification; 24 = 18 + 6 after classification | 0 after explicit admission of all 26 nonnative keys | (`result:backlog:closure-census`); (`rec:backlog:wave0-compact-inherited-closure`) |
| Compact structural pins | Runtime carriers 30; coverage 213 = 139 + 74; layout 69; graph 346/375; symbols / relocations / pushed sites 22/61/66; linked bytes 3,975 | Runtime carriers 28; coverage 209 = 135 + 74; layout 63; graph 338/355; symbols / relocations / pushed sites 20/55/60; linked bytes 3,861; compact relations / cases remain 23/46 | (`result:backlog:closure-census`); (`rec:backlog:wave0-closure-elision`); ruling 128 (`rule:phase7:wave0-rulings`) |
| Questions and entries | Twelve questions; 65 OPEN execution rows at entry; 59 RECORDED and 6 BASELINE PENDING at the completed-tree join | Twelve dispositioned; 65 RECORDED, zero OPEN and zero BASELINE PENDING; fourteen columns; 40 rows name bite 0.7 and 25 consuming-wave obligations remain; findings stay 50 HOLDS / 13 UNDECIDED / 2 DOES NOT HOLD | (`rec:backlog:wave0-preflight-census`); (`rec:backlog:wave0-preflight-exits`); (`tab:guide15-exec:preflight`) |
| Performance pair | Cold 1,662.2 s; warm 891.8 s; isolated debug / release 689.3 / 77.9 s | Comparable warm 731.2 s, decrease 160.6 s (18.0%); isolated debug / release 658.4 / 66.3 s; cold after unavailable; 56 checks and 5,579 executed / 52 ignored per profile unchanged within that pair | (`result:backlog:test-performance`); (`rec:backlog:wave0-test-performance`), ruling 129 |
| Completed-tree gate dimensions | Performance pair and earlier gates retain their distinct subjects and cache states | 63/0 mocked Meson, 63/0 real-document Meson, 5,996/0/55 workspace tests; full wall 5,132 s with shared-load context; no direct speedup comparison against the earlier 56-check pair | This record's subject and battery tables; `gate.log`; (`open:guide15-exec:gate-wall-baseline`) |

| Wave-0 exit condition | Evidence and disposition |
|---|---|
| (`cond:guide15-exec:wave0-exit-condition-1`) | MET: `T11-064` closes through region composition and the sponsored, positive-floor and conjunct packet at (`result:backlog:bite-0-5-gate-pass`); `T11-063` through (`rec:backlog:wave0-closure-audit`) and (`rec:backlog:wave0-closure-elision`); `T4-009` through (`rec:backlog:wave0-compact-inherited-closure`) under rulings 136–139; `T11-110` through ruling 129's landed scheduling remedy and (`rec:backlog:wave0-test-performance`). Their own criteria, not preparatory DONE rows, supply consolidation under (`rec:guide15-exec:bite-0-7-closure-carrier`). |
| (`cond:guide15-exec:wave0-exit-condition-2`) | MET: all 206 keys reconciled as 177 observed, 27 final, 2 retired, 0 waiting; no final ground is missing provisioning or an unperformed observable experiment (`result:backlog:close-safety-reconciliation`); (`result:backlog:bite-0-5-gate-pass`). |
| (`cond:guide15-exec:wave0-exit-condition-3`) | MET: the complete 78 / 43 / 35 / 15 / 35 keyed partition remains, with the before/after movements above and unchanged historical answer identities (`rec:backlog:wave0-close-safety`); (`result:backlog:close-safety-reconciliation`). |
| (`cond:guide15-exec:wave0-exit-condition-4`) | MET: 14 / 102 / 90 → 14 / 108 / 84; each remaining non-link has its exact reason, six old non-links now declared and eleven region-external, retaining all ninety identities (`result:backlog:region-links`); (`tab:backlog:relation-dispositions`). |
| (`cond:guide15-exec:wave0-exit-condition-5`) | MET: five conjuncts individually disposed through nineteen parts, ten MET / nine RETIRED / zero WAITING; every observable remainder performed and exact non-claims retained; original NOT PASSED verdict unchanged (`tab:backlog:conjunct-dispositions`); (`result:backlog:bite-0-5-gate-pass`). |
| (`cond:guide15-exec:wave0-exit-condition-6`) | MET: twelve dispositions and exact typed censuses at (`rec:backlog:wave0-preflight-census`), 65 explicit exits at (`rec:backlog:wave0-preflight-exits`), and clean completed-tree checks in this record pass (`gate:guide15:preflight`) under (`def:guide15-exec:semantic-entry-split`). Selected representation, private route, carrier/value rules, stored Clear assignment, bounded reviewed arithmetic, generation policy, separate event roles and public-only input contract are recorded; no production secret interface is introduced. The two DOES NOT HOLD findings retain Wave-2 declaration validation rather than becoming new implementation claims. |
| (`cond:guide15-exec:wave0-exit-condition-7`) | MET: the seven assigned dispositions and nine position-bearing UNDECIDED entries have numbered acceptance at (`rule:phase7:wave0-rulings`), joined clause by clause in (`rec:backlog:wave0-preflight-exits`) and grounded by (`rec:backlog:wave0-preflight-census`). Rulings 128 and 131 remain; 140–148 complete the packet. The baseline gate accepts no new position. |
| (`cond:guide15-exec:wave0-exit-condition-8`) | MET: completed tree `0.7.61-dev`, exact command surface, zero battery failures, both 63/0 Meson runs including real document rendering, workspace test totals and clean before/after results are retained in this record's evidence home; (`gate:guide15-exec:baseline-repository`) is green on that subject, and (`gate:guide15:batch`) is green on the stated equivalent surface. |
| (`cond:guide15-exec:wave0-exit-condition-9`) | MET: this record publishes the exact subject, separate counters, measured dimensions and retained non-claims; (`rec:backlog:wave0-preflight-census`) and (`rec:backlog:wave0-preflight-exits`) retain the future proof boundaries. Every later wave compares with this baseline under (`obl:guide15-exec:baseline-comparison`) and (`cond:wave0:baseline-freeze`). |

Candidate status remains: no production key or opening custody, deployment, release, final calibration, universal security or general theorem beyond the finite admitted corpus is claimed (`cav:guide15-exec:production-non-claims`); (`sec:phase6:exit-assessment`). Cold after remains unavailable in the accepted performance pair. Gate-wall regression standing stays open, measured without a threshold (`open:guide15-exec:gate-wall-baseline`). Real document rendering is established; byte reproducibility remains a separate release-oriented check, not a Wave-0 condition. No new native capture, Burn or Clear declaration dispatch, cross-operation constructor equality, measured carrier encoding, deployed expanded generation or complete public lifecycle follows from this gate. This record does not bind the evidence bytes: its figures are transcribed from the files in the evidence home, and no value in this tree lets a reader check them against particular bytes.

Every later wave publishes comparable figures beside this baseline; a regression needs an explicit disposition before advancement, never a silently changed denominator (`obl:guide15-exec:baseline-comparison`); (`def:guide15-exec:baseline-standings`); (`tab:guide15-exec:baseline-comparisons`). The twenty-five consuming obligations in (`rec:backlog:wave0-preflight-exits`) remain. Changed target, deployment, policy, constructor, ABI or protocol requires an explicit compatibility argument and rerun of affected evidence (`cond:guide15-exec:baseline-revalidation`). Under (`cond:wave0:baseline-freeze`), candidate-affecting landings stopped before the completed-tree gate; this record changes no evidence subject. Subsequent target, ABI, constructor or policy changes reopen compatibility and verification before publication. Each numbered ruling retains its own candidate-audit revision trigger; the Wave-4 carrier measurements under (`cond:guide15-exec:carrier-reopening`) still reopen a refused carrier by a Wave-4 ruling, and Wave-2 Clear realization reopens the retained compact declaration observations. The historical findings and original Phase-6 verdict remain unchanged.


**Record (Wave 0 entry)** · `rec:backlog:wave0-entry`

Bite 0.1 verifies the provisioned pinned Elements target, reproduces the accepted variable-schedule sponsorless maturity control, and reproduces the inherited typed censuses under (`gate:guide15-exec:bite-0-1-gate`) and (`gate:wave0:bite-0-1-packet`). The retained evidence home is `/workspace/loops/tripod/evidence/wave0/entry/20261024T194326Z-maturity-control`; its `RUN-REPORT` binds `MANIFEST.sha256`, which binds the report capture and timing sidecar. Five of five installed binary digests match the selected toolchain's adjacent `PROVENANCE.txt`; the executable version matches its version-string field, the source checkout matches its tip field, and the functional framework is present. The provenance retains the Release build configuration and SQLite-only descriptor-wallet restriction; the caveats below delimit environment comparison and inherited scope.

The native-route retained evidence home is under the dedicated execution directory identified by the absolute entry evidence location above, at `evidence/wave0/native/w16x1-retained-20261101T134825Z/`; its four evidence files are admitted at `packages/vectors/fixtures/maturity-retained-run-of-record/`, and its `RUN-REPORT` retains the suite provenance. The strategy and route study resolve their proposed homes through this record under (`conv:wave0:artifact-homes`); admitted archives retain their separate fixture paths and pins (`req:guide15-exec:wave0-entry-record`). The confidential retained evidence home is `evidence/wave0/confidential/w17a1-confidential-20261101T194031Z/`, under the same execution directory; its four files are admitted at `packages/vectors/fixtures/maturity-confidential-run-of-record/` (`rec:backlog:wave0-confidential-route-admission`). The diagnostic evidence home is `evidence/wave0/diagnostic/w17e1-diagnostic-20261102T103745Z/`, under that execution directory; its four files were copied with equal source and destination digests and are admitted byte-for-byte at `packages/vectors/fixtures/maturity-diagnostic-run-of-record/` (`rec:backlog:wave0-diagnostic-route-admission`).

**Data (Starting revision)** · `data:backlog:entry-starting-revision`

The starting revision for (`data:guide15:starting-revision`) is the clean entry tree at tag `0.7.9-dev`, workspace version `0.7.0-dev`, containing the completion strategy (`sec:wave0:bite-0-1`), after the Guide-15 formal rewrite and admission study; the exact entry source is retained in the capture's `RUN-REPORT` fields `suite-commit` and `suite-tree`, with `suite-clean yes`. This identifies the actual entry tree rather than the admission study's older starting tree.

**Data (Phase-6 result revision)** · `data:backlog:entry-phase6-revision`

The Phase-6 result for (`data:guide15:phase6-revision`) retains evidence base `0.6.305-dev`, gate-record landing `0.6.306-dev`, and amended closeout carried at `0.7.0-dev` (`sec:phase6:gate-record`); (`sec:phase6:exit-assessment`). Its accepted candidate and conditional evidence retain their recorded scope.

**Data (Architecture schema and semantic hashes)** · `data:backlog:entry-architecture-values`

The architecture values for (`data:guide15:architecture-values`) are schema 18 in `packages/architecture/src/spec.rs`, with `architecture_schema_version`, `semantic_hash`, `semantic_hash_algorithm`, `behavioural_hash`, and `behavioural_hash_algorithm` in `packages/model/generated/architecture.json` under (`reg:identities:ownership`). No architecture value changes in this entry.

**Data (Realization version)** · `data:backlog:entry-realization-version`

The realization version for (`data:guide15:realization-version`) is `0.7.0-dev`, derived from the workspace version in `Cargo.toml` and retained by `packages/architecture/src/spec.rs` and the `realization_version` field of `packages/model/generated/architecture.json`; this entry allocates no new realization identity.

**Data (Target contract revision)** · `data:backlog:entry-target-revision`

The target for (`data:guide15:target-revision`) is reviewed `TargetContractVersion::V2` from `packages/target-elements/src/definition.rs`, bound through the development validation in `packages/target-elements/src/deployment.rs`. The report capture's deployment identity fields, `deployment-genesis-id`, and `deployment-target-contract` select the inherited development deployment; `environment-chain` decodes to `elementsregtest`, with active tapscript and leaf version 196. Its `handshake-node-version`, `handshake-binary-reported-revision`, `handshake-intended-executed-tip`, and `handshake-framework-revision` bind the verified executable and source to the selected toolchain's `PROVENANCE.txt`; each matches the inherited capture's corresponding field.

**Data (Executor protocol revision)** · `data:backlog:entry-executor-revision`

The executor revision for (`data:guide15:executor-revision`) is `NATIVE_PROTOCOL_SCHEMA = 8` in `packages/target-elements-conformance/src/protocol.rs`, with `elements-native-executor` version `2.1.0` in the report capture's handshake. `scripts/elements-native-executor.sh` supplies the existing adapter configuration and preserves its two diagnostic destinations; `scripts/live-native-maturity-capture.sh` invokes the ignored `maturity_native` target serially with its documented environment. Archived revisions retain their own scope.

**Data (Compact-ASH candidate revision)** · `data:backlog:entry-compact-revision`

The compact candidate for (`data:guide15:compact-revision`) is PROPOSED: the `0.4.1-dev` candidate named by `plans/history/guide-12-completion-report.md`, with its deterministic twelve-leaf bundle of nine coordinator leaves and three recomputation-shared member leaves. The completion record admits no bundle identity without separate admission. Selection and acceptance remain a numbered ruling before the first compact control of bite 0.4; no new position ruling is needed for the inherited maturity control (`judg:wave0:early-rulings`).

**Data (STATE-constructor generation)** · `data:backlog:entry-state-generation`

The constructor generation for (`data:guide15:state-generation`) is the inherited maturity-only candidate in `packages/tapscript/src/state_constructor.rs` and (`sec:phase6:gate-record`), with the variable-metadata witness schedule retained by the accepted control. No maturity-plus-clear generation is accepted here; its distinct generation and origin disposition remain with (`task:guide15-exec:disposition-9`).

**Data (Selected public-value policy)** · `data:backlog:entry-public-policy`

The policy for (`data:guide15:public-policy`) remains explicit-only at the boundary, normalized-first with private change, with public-committed opening deferred (`sec:public-opening:result`). The maturity control reproduces that inherited candidate scope and supplies no compact, burn, or clear acceptance.

**Data (Clean-tree result)** · `data:backlog:entry-clean-result`

The base result for (`data:guide15:clean-result`) is the supplied gate of record `/workspace/loops/tripod/evidence/gate-side-20261024T183000Z.log` over the entry base `0.7.9-dev` (workspace version `0.7.0-dev`): Ok 54, Fail 0, chain-rc 0, wall 893.7 s. The capture independently records a clean entry source in `RUN-REPORT`; this field preserves the base result and does not substitute for the full clean-tree baseline required by bite 0.7.

**Result (Accepted control reproduced)** · `result:backlog:entry-control`

The clean base was published and captured before any entry-record edit or commit; the captured tree is the entry tree without a code difference, and the later change is this record. In `/workspace/loops/tripod/evidence/wave0/entry/20261024T194326Z-maturity-control`, `RUN-REPORT` records one test passed, zero failed or ignored, one ceremony, zero setup artifacts, cargo exit 0, and `eligible yes`. The test `the_maturity_announcement_runs_against_a_real_target` accepts singleton issuance, predecessor funding, and the sponsorless submission; its exact replay and mined-readback assertions bind the variable-metadata submission bytes to the accepted identity. The manifest-listed report capture retains operation 2's `request-id`, `request-bytes`, `request-subject`, `response-id`, `response-request-id`, and `response-verdict`, and `legacy-rendering` retains the accepted schedule, identity and mined readback. Request bytes and the deployment identities, chain, node revision, framework revision, adapter and protocol match `packages/vectors/fixtures/maturity-variable-run-of-record/`, whose acceptance is admitted by `variable_archive_admits_established_identity_and_exact_mined_readback` in `packages/vectors/src/maturity_corpus.rs`; fresh block provenance remains in the new capture rather than replacing the inherited fixture.

**Result (Inherited censuses reproduced)** · `result:backlog:entry-censuses`

At the published clean entry base, `the_census_figures_are_unmoved_by_the_binding_readers`, `the_whole_transaction_scoped_rows_are_exactly_the_recounted_set`, and `maturity_negative_half::tests::every_outstanding_row_is_registered` each passed. Their 31 assertion sites are `packages/vectors/src/maturity_evidence.rs:3838-3863`, `packages/vectors/src/maturity_safety.rs:4427-4461`, and `packages/vectors/src/maturity_negative_half.rs:621-632`: they retain the source claims of 206 rows, 78 answered and the required buckets 43 / 35 / 15 / 35, the relation partition 14 / 102 / 90 and unlinked reasons 55 / 9 / 9 / 17, and exact outstanding-register equality. The prescribed filters also selected `live_negative_half::tests::every_outstanding_row_is_registered`; the invocation passed four tests with zero failures, and that additional pass is not a maturity census claim. These executions reproduce inherited classifications without adding row evidence (`verif:guide15-exec:entry-census-check`).

**Caveat (Fee-floor scope)** · `cav:backlog:entry-fee-floors`

The control's acceptance was observed under zero relay and block floors, as the inherited evidence was: `scripts/elements-native-executor.py` sets `-minrelaytxfee=0` and `-blockmintxfee=0`, beside `-validatepegin=0`. The retained `diagnostics/executor-diagnostics.txt` and `diagnostics/executor-child-output.txt` record startup and completion; the capture header carries no fee-floor fields. Positive-floor relayability stays with bite 0.5 and `T12-005`, and this acceptance does not discharge it.

**Caveat (Single-node scope)** · `cav:backlog:entry-single-node`

This evidence is one disposable development node executing the inherited sponsorless control. It establishes neither production readiness, independent observer acquisition, current-root freshness, nor sponsored composition; the caller branch context and declared-boundary scope remain those of `packages/vectors/tests/maturity_native.rs`.

**Caveat (Inherited evidence remains scoped)** · `cav:backlog:entry-inherited-evidence`

At the entry admission, the inherited 128 outstanding Safety rows and five conditional Guide-14 conjuncts retained their Phase-6 standing (`sec:phase6:exit-assessment`); the typed census tests reproduced their source classifications, not fresh observations for those obligations. The compact candidate remained proposed, and `T11-064`, `T12-005`, and the twelve-question preflight remained open beyond the target prerequisite supplied there. The current Safety reading is recorded at (`result:backlog:close-safety-reconciliation`).

**Caveat (Execution-environment differences)** · `cav:backlog:entry-environment-differences`

The source provenance and evidence home differ from the inherited capture; the declared deployment identities, chain, active domain, leaf version, advertised capabilities, executable version, framework revision, adapter, protocol and zero fee policy match. The current provenance records SQLite-only descriptor wallets and Python 3.12.3; the inherited header does not establish those build-environment details. Current diagnostics retain two transient startup RPC failures followed by node readiness in 0.5 s and report the confidential materializer unavailable; the accepted explicit control requires no confidential materialization. No external capture retry occurred, and no inherited environment fact is inferred from an absent field.

The Guide-8 target foundation is complete; the gate record is §2.12.

The T3, T4, and T5 closed rows are archived verbatim in [backlog history](history/backlog-history.md#exited-phase-task-rows) under the Exited-phase task rows group, so a textual reference to any of those row ids resolves there. The open Phase-4 row remains here:

| ID | Status | Task |
|---|---|---|
| `T4-009` | DONE | Compact inherited closure is recorded at (`rec:backlog:wave0-compact-inherited-closure`) under rulings 136–139 (`rule:phase7:wave0-rulings`) and (`gate:guide15-exec:bite-0-4-gate`): 74 = 27 native observed + 21 native formal + 18 first-party observed + 2 first-party formal + 6 report observed, giving 51 observed and 23 formal with zero retained or unresolved. The two unavailable-witness identities are formal candidate-specific dispositions under ruling 137; their supporting private-dependency refusal receives no credit as the requested class. The two emitted-Clear observations concern declaration retention and removal, with no emitted Clear path claimed; Clear's realization landing in Wave 2 reopens them under ruling 138. First-party standing moves from 2 discharged / 18 retained to 20 closed / 0 retained and missing entries from 24 = 18 + 6 to zero; 74, 48 / 20 / 6, 209 = 135 + 74, two derivation-only discharges and native [27, 21, 0, 0] remain. The record names the ledger, exact comparison tests, both native archives and recorded package, plans, documentation, lint, label and full-gate results. (`T12-004`) keeps its native-only DONE scope; (`T12-007`) remains TODO for consolidation and the full clean-tree baseline. |

The Phase-6 task state opens with the Guide-14 document import; each wave
lands with its own evidence:

The T6 through T11 closed rows are archived verbatim in [backlog history](history/backlog-history.md#phase-6-closed-task-rows) under the Phase-6 closed task rows group, so a textual reference to any of those row ids resolves there.

| ID | Status | Task |
|---|---|---|
| `T11-049` | DONE | The plans-weight sweep landed through `0.6.187-dev`: the 103 closed Phase-6 rows of section 11, 141,507 bytes, moved verbatim with their identifiers into [backlog history](history/backlog-history.md#phase-6-closed-task-rows) under the Phase-6 closed task rows group behind a 233-byte pointer paragraph, and the maintained-prose soft target rose from 1,062,912 to 1,228,800 bytes. The maintained tree measures 883,525 bytes where it measured 1,024,799 before the move, leaving 345,275 bytes of advisory room under an unchanged 1,572,864-byte hard cap, and the move carried the preservation audit that the live-only task-status and duplicate-row checks cannot supply. Ruling 42 at (`rule:phase6:wave6-rulings`) states the decision and finding 45 at (`sec:phase6:wave6-findings`) records its arithmetic. |
| `T11-050` | DONE | ADR-024, closure-exact emission and linking, adopted through `0.6.188-dev` with its index row and census entry: an emitted and linked program carries exactly the checks the contract's closure requires and nothing more, and a check beyond closure is a defect of the same kind as a missing one (`[ADR024-dec:closure:exact-linking]`). For the STATE generation it supersedes the three §10.1 census entries and §10.2's exact-count, sponsor-suffix, change-role, fee-role and no-unclassified-position clauses, names the five realization relations whose scope it changes, and stays decided but unimplemented until the reduction bite of `T11-052` meets its adoption gate (`[ADR024-gate:closure:adoption]`). |
| `T11-051` | DONE | The Guide-14 Wave-6 opening record: rulings 40–52 at (`rule:phase6:wave6-rulings`), findings 38–47 at (`sec:phase6:wave6-findings`), these sixteen rows, and `R-7 CLOSURE-EXACT` in the conceptual register carrying the §10.1 and §10.2 supersession for the STATE generation with its discharge table and its MODEL-SCOPE class. Gate: the repository documentation and label gate over the three edited plan files, with both minted card labels resolving. Closed by the Wave-6 closure record under `T11-061`. |
| `T11-052` | DONE | The reduction bite. Ruling 41 drops the predecessor-program literal, `MetadataAuthentication` discharging predecessor identity, and ruling 40 drops the exact input and output counts, the sponsor-suffix asset sweep, the committed change program and version, the fee-program digest, the sponsor-maximum literal and the per-input issuance sweep, keeping self-position, input zero's asset, amount and script version, output zero's asset, amount and authenticated successor program, the semantic components and operator authorization, and carrying no count bound. It publishes the 26-row discharge table — relation to positional check, self-position, consensus conservation, named external evidence, semantic component or MODEL-SCOPE — recounts symbols and sites against finding 39's baseline, re-runs the golden fixture, refreshes the closure tests and the record-backed assessment, and carries extension-point note sites 1 and 4 of `T11-062`. Gate: absent semantic authentication rejects; a wrong actual predecessor fails the authenticated equation; no predecessor literal survives; a transaction carrying extra inputs and outputs of any asset but the singleton is accepted; the singleton at any position other than input zero or output zero is refused. Depends on `T11-050`. Landed as `0.6.189-dev`. |
| `T11-053` | DONE | The typed upstream and deployment bridge: validated maturity plan, architecture-keyed lead bounds, target revision, candidate identity and established operator binding, together with the two external evidence roles finding 41 names — the singleton's non-reissuable declaration and its issuance under the constructor — recorded beside `SubstrateConservation`. Sources stay typed and validated rather than an untyped value map, and the fixture lead bounds are declared as fixtures rather than calibrated defaults. Rulings 44 and 49 at (`rule:phase6:wave6-rulings`). Gate: a missing or unvalidated source refuses by type, and both external roles appear in the deployment binding. Depends on `T11-052`. Landed as `0.6.190-dev`. |
| `T11-054` | DONE | The STATE symbol and refusal census, `state_symbol.rs` and `state_error.rs`: closed stable keys, values, origins and expected types resolved in two passes and derived from the family enums rather than by extending compact or live, with no reserved owner-sighash entry and no unused symbol. Rulings 43 and 44. Gate: every definition key matches an actual record site or constructor-field consumer before normalization, a constructor-policy consumer counting even without a script push, and the census closed over is the reduced one. Depends on `T11-052` and `T11-053`. Landed as `0.6.192-dev`. |
| `T11-055` | DONE | The authenticated graph, `state_graph.rs` as a narrow extension of `state_constructor_graph.rs`: typed binding times, witnessed-root evidence, validated cuts removed, a residual acyclic graph and a canonical dependency order within the shared 64-reference bound, with the extension note that a reference to a previously linked bundle is a backward acyclic edge carried on its reference-class enum as site 2 of `T11-062`. Ruling 45. Gate: closes `G14C-08` and `G14C-09` only on ruling 50's criteria — a residual-DAG proof with a component carrying two edge-disjoint cycles where one authenticated cut is insufficient and the linker refuses, and the refusal of a literal static root beneath itself with exact witnessed-root authentication. Depends on `T11-054`. Landed as `0.6.193-dev`. |
| `T11-056` | DONE | The relocation and resource sweep, `state_relocate.rs` and `state_resource.rs`: one relocation record per mandatory occurrence carrying symbol, expected type, component and leaf role, site identity, pre-value, linked value and checked resource delta; shared component aliases counted once; one pristine rebuild reparsed and abstractly revalidated; and checked linked totals beside the abstract schedule, the final-stack walk and explicit absent native observations, without inheriting the saturating diagnostic projection. Rulings 46 and 51. Gate: contributes `G14C-10` on ruling 50's stable-identity, permutation and simultaneous-rebuild criteria. Depends on `T11-055`. Landed as `0.6.194-dev`. |
| `T11-057` | DONE | The STATE tree, `state_taptree.rs`: the ruled singleton static tree and the fixed metadata and static outer pair, with declaration sequences, exact identity, content, role and version checks, unit weights, the shared checked-cost engine and `SubsetOracleOnly`. Ruling 47. Gate: static cost is zero and the unit-weight complete tree costs two, the evidence binds to the exact linked leaf bytes, and a deployment depth cap is enforced rather than assumed. Depends on `T11-052`. Landed as `0.6.191-dev`. |
| `T11-058` | DONE | The carrier comparison, `state_carrier.rs`: the discharge table formalized as checked three-sided closure over the compiler, emitted and linked sides, with an exact outstanding ABI-role contract that Wave 7 answers against the same keys. Ruling 48. Gate: representation, execution case, relation, discharge boundary and selected alternative compared; vacuity and external requirements preserved; actual component ranges verified inside the committed leaf; duplicate, missing, extra and unreachable assignments refused in both directions; and the MODEL-SCOPE rows naming `T11-064`. Depends on `T11-056` and `T11-057`. Landed as `0.6.195-dev`. |
| `T11-059` | DONE | The candidate bundle, `state_bundle.rs`: `CandidateLinkedMaturityBundle` carrying the exact plan and reviewed target, validated deployment and operator binding, constructor policy and exact static subtree, linked record and programs, placements and control recipes, definitions, graph and cut proof, relocations, checked resources, carrier closure, ABI obligations, evidence and lifecycle obligations and candidate status, with a fallible constructor application over the fixed linked subtree and no content digest. Ruling 49. Gate: the §17.3 continuity equality is one named check, which is extension-point note site 3 of `T11-062`, and supplied concrete instances are retained only with their exact metadata. Depends on `T11-058`. Landed as `0.6.196-dev`. |
| `T11-060` | DONE | Independent closure tests in `packages/vectors` over linked bytes rather than over the linker's own structures: the discharge table checked rather than restated, the reduced leaf's golden evidence re-run, and the accepted and refused transactions of the adoption gate exercised end to end (`[ADR024-gate:closure:adoption]`). Rulings 49 through 51. Gate: a test reading only linked bytes fails if a removed check returns or a kept check is absent. Depends on `T11-059`. Landed as `0.6.197-dev` and `0.6.198-dev`. |
| `T11-061` | DONE | The Wave-6 closure and handoff record: recounted symbols, sites and resources against finding 39's baseline, deliverables and exits dispositioned on gate evidence, conceptual rows closed only on ruling 50's criteria, and the exact ABI, carrier, resource and native obligations handed to Waves 7, 8 and 12. Gate: marks `T11-062` DONE only with four diff citations, and closes `T11-051`. Depends on `T11-060`. |
| `T11-062` | DONE | Extension-point notes at the four sites where a second constructor generation would enter. The guide records migration as outstanding and states that no migration between static subtrees is implemented, so the code points that embody that rule say so where a reader would otherwise reach for the wrong mechanism. DONE criterion, checkable in the diff: (1) `StateConstructorGeneration` in `packages/tapscript/src/state_constructor.rs` carries a doc paragraph stating that a generation is a distinct recipe identity rather than a metadata change and that the census is single because of the continuity rule, citing the guide's outstanding migration status by label; (2) `ReferenceClass` in `packages/linker/src/graph.rs`, and the STATE graph's class enum once ruling 45 lands, carries a doc paragraph stating that a reference to a previously linked bundle is a backward, acyclic edge — a resolution class and not a cut — so that `AuthenticatedWitnessedRootContinuity` is not mistaken for it; (3) the §17.3 continuity equality is one named function with a doc paragraph saying it is the no-migration rule and the single assertion a migration relation would replace; (4) `MetadataAuthentication` in `packages/tapscript/src/state_announcement.rs` carries a doc paragraph stating that, recognition naming no predecessor under ruling 41, generation support would enter through this component's authentication against the prior generation's recipe. Each note states the current shape's reason and cites the guide, and none sketches a generations design. Carried by `T11-052` for sites 1 and 4, `T11-055` for site 2 and `T11-059` for site 3; if any site's bite slips the row stays open into the next wave, so the Phase-6 closure cannot close over it. Sites 1 and 4's reason paragraphs landed as `0.6.189-dev`, site 2's STATE half on the binding-time enum as `0.6.193-dev` and site 3's continuity note as `0.6.196-dev`; this step lands the two the closure record found missing, `ReferenceClass`'s paragraph in `packages/linker/src/graph.rs` and the §17.3 citation in `MetadataAuthentication`'s, so all four sites state the current shape's reason and cite the guide. |
| `T11-063` | DONE | Audit the compact and live generations against the closure-exact principle. Their region counts are required for closure on their own terms — compact ASH aggregates over a fixed batch, and a live transfer admits split, merge and redistribution, so its output count is a second free axis its multi-destination logic reads — while their sponsor-region checks may not be (`[ADR024-rem:closure:consequences]`). Gate: each retained check names what closure needs it for, and each surplus check is removed with its census recount. Scheduled after the Phase-6 closure unless a Wave-6 link depends on it. Classification is recorded at (`rec:backlog:wave0-closure-audit`); `T12-003` retains destination elision and its byte-dependent revalidation, with regional count and issuance residuals named. Elision and byte-dependent revalidation are closed by `T12-003` and (`rec:backlog:wave0-closure-elision`), with the named regional residuals retaining their scope. |
| `T11-064` | DONE | The region-scoping refit of the realization under ruling 52: an operation's transaction-side relations quantify over the region the operation claims, region membership enters the observation, and a transaction is the union of its operations' regions, with `AllowedObjectFamilies`, `SponsorIsolation`, `CanonicalDeltaPolicy`, `OpenFlowPolicy` and `SponsorEnvelopeMultiplicity` re-scoped or retired. Gate: model conformance over a composed observation — an announcement beside a second family's operation — is accepted, and a foreign object inside the announcement's own region is refused. Wave-6 scope only if a Wave-6 link depends on a closed carrier for those relations; otherwise the first bite of Phase 7's composition work, because the first composed operation is what exercises it. The discharge table classes ten relation identities MODEL-SCOPE — `AllowedObjectFamilies` on each transaction side, the plain-L-BTC cardinality and recognition relations on each side, `SponsorIsolation` and `SponsorEnvelopeMultiplicity` on the sponsor subject, and `OpenFlowPolicy` and `CanonicalDeltaPolicy` on the operation subject — of which this refit's scope is the six identities its five relations are declared over, the family closure being declared once per transaction side; the remaining four lost their carrier with the retired partition fragment and are model-scope for that reason rather than for this one. Its structural portion is closed by `T12-002` at (`rec:backlog:wave0-region-refit`); the admitted S1/S2 region binding and retained obligations are recorded at (`rec:backlog:wave0-close-safety`), whose historical bounded-consolidation census is retained beside the current reconciliation at (`result:backlog:close-safety-reconciliation`). The remaining obligations close with `T12-005` at (`result:backlog:bite-0-5-gate-pass`): the composed observation and foreign-object refusal are recorded at (`result:backlog:region-composition`); ruling 130 revisits ruling 58 before C1 (`rule:phase7:wave0-rulings`); C1 construction, C2 observation and C3 retyping are recorded at (`rec:backlog:wave0-sponsored-construction`), (`rec:backlog:wave0-sponsored-observation`) and (`rec:backlog:wave0-sponsored-retyping`); S1/S2 positive-floor acceptances are recorded at (`rec:backlog:wave0-sponsored-observation`), the eight semantic forms at (`rec:backlog:wave0-native-observation`) and the combined projection at (`rec:backlog:wave0-combined-projection`), with confidential agreement at (`rec:backlog:wave0-confidential-route-admission`) and exact region binding at (`tab:backlog:conjunct-dispositions`). |
| `T11-065` | DONE | The Wave-7 opening record: rulings 53 through 61 at (`rule:phase6:wave7-rulings`), findings 68 onward at (`sec:phase6:wave7-findings`), these eleven rows and the register's Wave-7 paragraph, with every figure and citation relocated in the tree at the tip the record opens from rather than carried from the opening study. Gate: the repository documentation and label gate over the three edited plan files, with both minted card labels resolving. Closed by the Wave-7 closure record under `T11-075`. Landed as `0.6.201-dev`. |
| `T11-066` | DONE | Parameterize the vectors' maturity sources in `packages/vectors/src/maturity_closure.rs` by singleton asset, committed operator key and lead window as supplied values, and add a third deployment committing the third published test signer handle's x-only key with a fixture singleton, every closure test running over three deployments. Ruling 54. Gate: the golden test passes unchanged on the two existing deployments, whose six golden roots stay bit-identical, and the third deployment links. Depends on `T11-065`. Landed as `0.6.202-dev`. |
| `T11-067` | DONE | The public current-STATE view in `packages/transaction/src/state_view.rs`: the ten entries of (`def:guide14-exec:state-view`), the duplicate and no-overwrite refusals lifted from the compact view, the first-party reconstruction check, and the freshness half of the link obligation named as a residual rather than assumed. Ruling 55. Gate: a supplied metadata and nonce that do not reproduce the supplied program refuse by name, and a second statement about one outpoint refuses. Depends on `T11-065`. Landed as `0.6.205-dev`. |
| `T11-068` | DONE | The typed request in `packages/transaction/src/state_request.rs`: the requested cycle, the requested form and the inherited sponsor-change request, with the unselectable fields of (`def:guide14-exec:typed-request`) and (`def:guide14-exec:request`) structurally absent under a census test on the live generation's facet pattern. Ruling 58. Gate: the sponsored form is representable and the census walks every facet. Depends on `T11-065`. Landed as `0.6.203-dev`. |
| `T11-069` | DONE | The relay figure in the reviewed target `packages/target-elements`: the eighty-byte tapscript stack-item relay limit stated beside the relay conditions with its provenance named, and the announcement form's relay admission stated there. Ruling 53. Gate: the vectors test reads the figure from the target rather than from its own constant. Depends on `T11-065`. Landed as `0.6.204-dev`. |
| `T11-070` | DONE | The candidate ABI in `packages/transaction/src/state_abi.rs`: a derivation over a prototype bundle giving input and output layouts with no shape index, seven witness-role records carrying the seven attributes, widths and order of (`rule:guide14-exec:witness`) read from the record with consumers read from the carrier map, the typed relay verdict, candidate status, and the inherited link obligations partitioned into discharged and carried. Rulings 53, 55, 59 and 60. Gate: a bundle claiming more than the prototype artifact status refuses, the partition covers the five obligations exactly, and the verdict is refused relay with direct submission named as the route; the guide's suggested commit rendered in the house form is `feat(transaction): derive the maturity-announcement ABI`. Depends on `T11-067`, `T11-068`, `T11-069`. Landed as `0.6.206-dev`. |
| `T11-071` | DONE | Construction in `packages/transaction/src/state_construct.rs`: the ordered steps of (`rule:guide14-exec:successor-search`) — validate the request, derive the successor metadata through the realization's transition, apply the bundle's constructor, hold the continuity equality, fix output zero, refuse the sponsored arm by name, freeze the protected bytes and create the operator signing request — with a caller-supplied nonce result or output program unrepresentable. Rulings 56 and 58. Gate: an exhausted search arrives as the linker's refusal wrapped, distinguishable from a wrong nonce and from a later-admissible one. Depends on `T11-070`. Landed as `0.6.207-dev`. |
| `T11-072` | DONE | Finalization in `packages/transaction/src/state_finalize.rs`: the protected transaction and its bytes, the finalized output census, the spent-output census that (`def:guide14-exec:signing-request`) binds, the settled-fact census, the exact-byte offered comparison, and the signing request built through the script-path census so that all nine terms of (`rule:guide14-exec:census-binding`) derive from one finalized candidate. Gate: the finalized bytes survive the strict decoder's round trip. Depends on `T11-071`. Landed as `0.6.208-dev`. |
| `T11-073` | DONE | Signing states and mutation refusals in `packages/transaction/src/state_signing.rs`: signing-started, authorized and submit-ready over the production route, the twelve-region census with an ordered complete list, every protected operation after signing-started refusing by the region it names, the four post-signing mutations of (`rule:guide14-exec:signing-faults`) named, and those faults reached by name through a wrapping STATE root. Rulings 57 and 60. Gate: every region's mutation after signing rejects by name. Depends on `T11-072`. Landed as `0.6.209-dev`. |
| `T11-074` | DONE | The node-free submit-ready candidate in `packages/vectors/tests/maturity_abi.rs`: over the third deployment, view, request, construction, finalization and signing with the handle's own material, a submit-ready candidate whose signature verified, the wrong-key control, the strict-decoder round trip of the finalized bytes, and determinism — equal complete input tuples give equal candidate bytes — closing `G14C-07`, with the vectors' refusal root given a closed-root walk if the module is touched. Rulings 54 and 61. Gate: the wave's first exit is met, and `G14C-07` closes only on the determinism property as a test. Depends on `T11-066`, `T11-073`. The landing touched no module carrying the vectors' refusal root, so the conditional closed-root walk was not owed here and stays the named follow-on of finding 66. Landed as `0.6.210-dev`. |
| `T11-075` | DONE | The Wave-7 closure and handoff record: recounts, dispositions on gate evidence, `G14C-07` and the `G14C-03` host-policy clause closed on their tests, the wave's second exit recorded as conditionally met with its condition stated, the residuals — the relay restructure, the freshness half of the split link obligation, and the duplicated curve stubs of finding 50 if they remain unabsorbed — and the native obligations handed to Wave 8. Gate: closes `T11-065`. Depends on `T11-074`. Landed as `0.6.211-dev`. |
| `T11-076` | DONE | The Wave-8 opening record: rulings 62 through 69 at (`rule:phase6:wave8-rulings`), findings 96 onward at (`sec:phase6:wave8-findings`), these fifteen rows, the register's Wave-8 paragraph, and the two in-place corrections of the Wave-7 handoff clause and ruling 14's dependency citation, with every figure and citation relocated in the tree at the tip the record opens from rather than carried from the opening study. Gate: the repository documentation and label gate over the three edited plan files, with both minted card labels resolving. Closed by the Wave-8 closure record under `T11-090`. |
| `T11-077` | DONE | The closed-root walk over `MaturityClosureRefusal` in `packages/vectors/src/maturity_closure.rs` and `packages/vectors/tests/maturity_closure.rs`: an exhaustive match with no wildcard over the twenty-five variants, each reached by name by a test or declared unreachable with its reason, on the pattern the linker's own walk sets in `packages/linker/src/tests/state_symbol_tests.rs`. Finding 94's follow-on, and the module the wave's other bites read from. Gate: a variant added to the root stops the crate compiling until it is censused. Depends on `T11-076` and can start at once. |
| `T11-078` | DONE | The canonical semantic fixture registry in `packages/vectors/src/maturity_fixture.rs`: a fixture identity binding architecture, realization, operation, initial semantic state, request and order context while excluding target bytes; a positive census over the positive rows at (`tab:guide14-exec:positive`); expected successors taken through the realization's transition; and the executable model and the typed projection adapter recorded as explicitly outstanding producers rather than silently absent. Ruling 69. Gate: no fixture carries a target program, control block, transaction byte or signature. Depends on `T11-076`. |
| `T11-079` | DONE | The STATE safety matrix in `packages/vectors/src/maturity_safety.rs`: §16's thirteen tables transcribed as typed rows carrying all nine per-row facts, reusing the generation-neutral boundary and layer types of `packages/vectors/src/matrix.rs`, with a section enum, a row-boundary enum whose non-answer members are typed, and a census asserted from the tables rather than from prose. Rulings 63, 65 and 67. Gate: the row count equals the tables' recount of 206 and every section's count is asserted. Depends on `T11-076`. |
| `T11-080` | DONE | Relation and boundary links in `packages/vectors/src/maturity_safety.rs`: a relation standing and a row resolver against the compiler's validated announcement plan, resolving each row to exactly one published requirement or failing loudly, with ruling 52's model-scope relations carried as a stated unlinked reason rather than a guessed link. Gate: every linkable row resolves and every unlinked row states why. Depends on `T11-079`. |
| `T11-081` | DONE | Positive class witnesses in `packages/vectors/src/maturity_fixture.rs`: one executable typed predicate per class of (`rule:guide14-exec:class-witnesses`) with a STATE subject — minimum lead, maximum lead, single-field transition and nonzero representation nonce — with the sponsored predicate carried as out-of-wave under ruling 58 and the public-recovery predicate with the recovery deliverables at (`task:guide14-exec:wave10`). Gate: each predicate is exercised against the registry. Depends on `T11-078` and `T11-079`. |
| `T11-082` | DONE | First-party refusal evidence in `packages/vectors/src/maturity_first_party.rs`: a validator census over the STATE chain's entry points in `packages/transaction`, one case per pre-target row, a two-run discharge running the owning entry point once honest and once with the case's one stated change and concluding nothing if the control did not pass, and a validated evidence type with a private constructor, on the live half's pattern in `packages/vectors/src/live_first_party.rs`. Gate: every pre-target row has a case and no case passes without its control. Depends on `T11-077` and `T11-079`. |
| `T11-083` | DONE | The evidence plan and its standings in `packages/vectors/src/maturity_evidence.rs`: `MaturityAnnouncementEvidencePlan` with private fields, no public constructor and one derivation from the eight inputs of (`def:guide14-exec:evidence-plan`), two of them structurally non-empty and explicitly outstanding; the STATE standing vocabulary with the unexpected-boundary and infrastructure-blocked members excluded from an answered standing; the closed three-class observation vocabulary and the class-3 poison marker; and a census with one bucket per standing. Rulings 62, 66 and 67. Gate: no ordinary standing accepts the marker, by type. Depends on `T11-078`, `T11-079`, `T11-080` and `T11-082`. |
| `T11-084` | DONE | The negative half's register in `packages/vectors/src/maturity_negative_half.rs`: every row a run has not answered, each carrying the named gap that keeps it waiting, held against the classifier by set equality in both directions on the live register's pattern. Gate: the register and the census agree both ways. Depends on `T11-083`. |
| `T11-085` | DONE | The validated safety report in `packages/vectors/src/maturity_report.rs`: a report role, a three-state completeness, the volatile-field exclusions of (`rule:guide14-exec:report-canonicality`), assembly, validation and rendering, the `G14C-02` clause the report witnesses, and canonical bytes that exclude wall time. Ruling 68. Gate: two runs of one plan render identical canonical bytes. Depends on `T11-083` and `T11-084`. The guide's suggested subject rendered in the house form is `feat(vectors): complete maturity-announcement safety evidence`. |
| `T11-086` | DONE | The exact-subject binding sweep over `packages/vectors/src/maturity_safety.rs` and `packages/vectors/src/maturity_evidence.rs`: every row's locator names exactly one subject, every control identity on a standing came from an accepted control of the row's own shape, and the layer-against-boundary product is walked as `observed_boundary`'s own test walks it. Ruling 67. Gate: the wave's fourth exit at (`task:guide14-exec:wave8`), controls and mutants exact-subject bound. Depends on `T11-079` and `T11-083`. |
| `T11-087` | DONE | The fifth case's planner in `packages/vectors/src/maturity_native.rs`: a target operation planner for the announcement — issue the singleton, fund the predecessor at the exact candidate program, submit the submit-ready candidate's bytes — with the positive row's boundary declared as ruling 64 fixes it. Gate: the planner's declared layers agree with the matrix's rows. Depends on `T11-083`. |
| `T11-088` | DONE | The fifth case's run, a node-gated test on the pattern of the four-shape run, driven by the live-run variables and writing a capture: the run on the Elements toolchain the compute node carries, with its observation compared against the boundary the row declared. Ruling 64. Gate: the capture's observed layer is `relay_policy_rejection` and the row is answered as such, while the accepted row is recorded outstanding with its two routes named. The run's capture is committed by the admission under `T11-089` on `0.6.227-dev`. Depends on `T11-087`. |
| `T11-089` | DONE | The capture's strict admission in `packages/vectors/src/maturity_corpus.rs`, over a capture committed under `packages/vectors/fixtures/maturity-run-of-record/` on `0.6.227-dev`: a fixed file census, pinned sizes, a pinned manifest and run addresses, and a typed import refusal per admission layer, on the pattern of the live operator run of record in `packages/vectors/src/live_corpus_native_operator.rs`. Gate: the committed capture is admitted by the strict importer and a moved byte is refused. Depends on `T11-088`. |
| `T11-090` | DONE | The Wave-8 closure and handoff record: recounts, dispositions on gate evidence, each of the four exits met or conditionally met with its condition named, `G14C-12`'s closed half stated as the half that closed, `G14C-02` recorded against the clause the report witnesses, the seventh conjunct of (`gate:guide14-exec:exit`) recorded as met or outstanding, and the residuals named rather than left for a later study to rediscover. Gate: closes `T11-076`. Depends on `T11-086` and `T11-089`. |
| `T11-091` | DONE | The Wave-9 opening record: rulings 70 through 78 at (`rule:phase6:wave9-rulings`), findings 143 onward at (`sec:phase6:wave9-findings`), these nine rows and the register's Wave-9 paragraph, with figures recounted and citations relocated in the opening tree. Gate: the repository documentation and label gate over the three edited plan files, with both minted card labels resolving. Closed by the Wave-9 closure record under `T11-099`. Closed on `0.6.229-dev` by the Wave-9 closure record under `T11-099`. |
| `T11-092` | DONE | The constructor characterization sweep in `packages/tapscript/src/tests/state_constructor_tests.rs` and `packages/linker/src/tests/state_bundle_tests.rs`: both semantic sides under real arithmetic, a nonzero-search example with lower-candidate refusals, normalized branch order characterized, full retained-tree equality distinguished from root equality and derived control recipes compared; no production change beyond what a test needs to reach. Gate: each test fails when its semantic side, nonce prefix, static descriptor or control path is perturbed, with each test carrying its perturbed arm. Depends on `T11-091` and can start at once. Landed on `0.6.230-dev`. |
| `T11-093` | DONE | The byte projections in new `packages/vectors/src/maturity_continuity.rs`, with `lib.rs` and the Meson census: private validated records retaining byte source, branch context, predecessor funding evidence, codec metadata and nonce, both constructor parameters, the realization's expected successor, exact input and output comparisons and unmet acceptance; existing target decoding and retained recipe context are inputs, and the projector refuses inconsistent retained predecessor data. Rulings 70, 71 and 77 at (`rule:phase6:wave9-rulings`). Gate: exact decode and re-encode, the correct funded outpoint and program, two-sided program reconstruction and independent semantic and static failure controls. Depends on `T11-092`. Landed on `0.6.231-dev`. |
| `T11-094` | DONE | Constructor evidence and negatives in `packages/vectors/src/maturity_continuity.rs` with targeted additions to `packages/vectors/src/maturity_fixture.rs`: checked leastness-prefix evidence, separate tweak/parity and predecessor-observed/successor-derived control comparisons, and coherent cross-bundle and stale-constructor cases, including repository recomputation of findings 143 and 152. Rulings 75 and 78 at (`rule:phase6:wave9-rulings`). Gate: equal metadata with different static content rejects, a wrong semantic successor with the same static tree rejects, a valid control masks neither, and every mutant records its actual carrier and original matrix boundary. Depends on `T11-093`. Landed on `0.6.232-dev`. |
| `T11-095` | DONE | The validated continuity report in new `packages/vectors/src/maturity_continuity_report.rs`, with module and census wiring: assembly derives both projections and separate comparison results; validation recomputes every source binding, field, hash, control, class and branch, the acceptance disposition, residuals and completeness; rendering takes the validated wrapper alone with STATE's eleven volatile exclusions. Rulings 72, 74 and 76 at (`rule:phase6:wave9-rulings`). Gate: mutation of each claimed field is detected, volatile changes leave canonical bytes unchanged, and missing acceptance yields partial completeness. Depends on `T11-094`. Landed on `0.6.235-dev`. |
| `T11-096` | DONE | Evidence integration without boundary change in `packages/vectors/src/maturity_evidence.rs`, `packages/vectors/src/maturity_report.rs` and `packages/vectors/src/maturity_native.rs`, with `packages/vectors/src/maturity_negative_half.rs` only if the required set changes: typed constructor material-present/absent input, the history registry kept outstanding, validated comparison records associated with exact rows and `ConstructorContinuityObserved` reserved for accepted readback; every test pinning a census, register or report figure moved by this bite is in scope, whichever module holds it. Ruling 73 at (`rule:phase6:wave9-rulings`). Gate: the recomputed partition, zero false target discharges, unchanged boundary declarations and set equality with the register. Depends on `T11-095`. Landed on `0.6.238-dev`. |
| `T11-097` | DONE | Both concrete sources end to end in new `packages/vectors/tests/maturity_continuity.rs`, with its census entry: the node-free submit-ready candidate of `0.6.210-dev` and the archived submission admitted by `0.6.227-dev`, reconstructed with distinct funding and branch context through public builders and the admitted corpus, with announcement 10 derived from actual inputs. The suggested subject at (`task:guide14-exec:wave9`) rendered in the house form is `feat(vectors): verify STATE constructor continuity`. Gate: deterministic validated report bytes, exact source reconstruction, independent negative axes and explicit unmet acceptance. Depends on `T11-093` through `T11-096`. Landed on `0.6.247-dev`. |
| `T11-098` | DONE | The accepted step: after a separately ruled route and its prerequisite implementation, one run supplying an Accepted submission identity, exact mined readback and a validated continuity comparison over those bytes, admitted as immutable evidence. Ruling 70 at (`rule:phase6:wave9-rulings`) records this dependency and selects neither route. Gate: the current accepted-response validation and an exact readback match. Blocked on the route track's accepted evidence at `T11-108` under ruling 79 at (`rule:phase6:wave9-rulings`); no node run is scheduled by this record. Supplied on `0.6.245-dev`: the accepted identity, the exact mined readback and the accepted-byte continuity comparison admitted under `T11-108`; the row classification over those bytes remains with `T11-099`. |
| `T11-099` | DONE | The Wave-9 closure or precisely partial stopping record: recounts, dispositions on gate evidence, the register clauses actually advanced and the residuals, with no unconditional first-exit claim without accepted readback. Gate: closes `T11-091`. Depends on `T11-097` and on either `T11-098` or an explicit conditional-closeout ruling. Closed on the label gate, this record. |
| `T11-100` | DONE | The route record: ADR-025, ruling 79 at (`rule:phase6:wave9-rulings`), the two archived inputs and these ten rows establish the constant-elision route at (`[ADR025-dec:relay-witness:constant-elision]`). DONE when the route closes under `T11-109`. Gate: documentation, census and label checks. Closure depends on `T11-109`. Closed on `0.6.245-dev` by the route-closure record under `T11-109`. |
| `T11-101` | DONE | Classify the canonical layout rows in `packages/realization/src/state_codec.rs` as constant with value or variable, with derived variable-region and canonical-rebuild helpers; encoding and strict decoding stay unchanged. DONE when classification and encoding agree byte for byte and existing strict refusals remain. Gate: agreement tests and unchanged strict-decoder refusals under the adoption gate at (`[ADR025-gate:relay-witness:adoption]`). Depends on `T11-100`. Landed on `0.6.234-dev`. |
| `T11-102` | DONE | Retain the witness schedule in the composed record in `packages/tapscript/src/state_program.rs` and carry it through the planner, closure and corpus loader in `packages/vectors/src/maturity_native.rs`, `packages/vectors/src/maturity_closure.rs` and `packages/vectors/src/maturity_corpus.rs`. DONE when historical replay is exact, historical selection for new deployment refuses, and cross-schedule replay refuses. Gate: all three historical requests byte-identical and both refusals observed, preserving replay at (`[ADR025-dec:relay-witness:constant-elision]`). Depends on `T11-101`. Landed on `0.6.236-dev`. |
| `T11-103` | DONE | Deliver policy, legalization, prologue, symbol and loader atomically: the target policy in `packages/target-elements`, two typed lowering refusals, the generated metadata-authentication prologue in `packages/tapscript/src/state_announcement.rs`, width-checked `MetadataHeader` resolution in `packages/linker/src/state_symbol.rs`, variable-region population in `packages/transaction/src/state_signing.rs`, and derived ABI with per-schedule form review in `packages/transaction/src/state_abi.rs` and `packages/target-elements/src/transaction_form.rs`. DONE when the seven-item map carries 53 bytes at index 4, every initial item is at most 80, and the prologue restores the historical entry stack. Gate: stack equivalence, unchanged semantic outcomes, wrong/short/long variable-item refusals, strict round trips and complete-tuple determinism at (`[ADR025-gate:relay-witness:adoption]`). Depends on `T11-102`. Landed on `0.6.239-dev`. |
| `T11-104` | DONE | Add initial-argument admission accounting beside the 520-byte execution bound in `packages/linker/src/state_resource.rs`, recomputing script, instruction, initial-stack and peak-stack figures while retaining historical refusal. DONE when the two bounds are measured separately without filling absent dimensions. Gate: 80 passes and 81 fails the initial-item comparator, script/control are excluded, and deliberate absences remain absent at (`[ADR025-gate:relay-witness:adoption]`). Depends on `T11-103`. Landed on `0.6.240-dev`. |
| `T11-105` | DONE | Sweep constructor, relocation and golden fixtures by schedule: recompute all seven goldens and both semantic sides' selected nonces with real arithmetic and independent framing, recount sixteen relocation sites, and retain old/new-tree continuity refusal. DONE when both sides reconstruct and every rejected lower nonce is explained. Gate: independent reconstruction, continuity negatives and seven pushed symbols at sixteen sites at (`[ADR025-gate:relay-witness:adoption]`). Depends on `T11-104`. Landed on `0.6.241-dev`. |
| `T11-106` | DONE | Establish the accepted capture/admission contract in `packages/vectors/src/maturity_native.rs` and `packages/vectors/src/maturity_corpus.rs`: acceptance carries schedule, identity and readback, with a separate pinned four-file corpus admission path and the historical loader preserved. DONE when scripted acceptance exercises identity/readback validation without acquiring native provenance. Gate: wrong txid, wtxid or bytes, recipe swaps, incomplete exchanges and refused responses carrying accepted fields all refuse at (`[ADR025-gate:relay-witness:adoption]`). Depends on `T11-105`. Landed on `0.6.242-dev`. |
| `T11-107` | DONE | Integrate continuity: `packages/vectors/src/maturity_continuity.rs` reconstructs through legalization from the retained schedule with observed width as a check, `packages/vectors/src/maturity_safety.rs` annotates the `wrong-domain-separator` and `nonzero-reserved-field` carrier moves with ABI construction rejection unchanged, and the continuity report reads the schedule. DONE when both schedules project without deleting codec or leaf-binding evidence. Gate: wrong-schedule bytes refuse and the historical map still projects at (`[ADR025-gate:relay-witness:adoption]`). Depends on `T11-106`, `T11-095` and `T11-096`. Landed on `0.6.243-dev`. |
| `T11-108` | DONE | Obtain and immutably admit the fresh node run: fund a predecessor under the new leaf, settle the three exchanges through `scripts/live-native-maturity-capture.sh` and the existing executor, and admit Accepted identity plus exact mined bytes, txid and wtxid into the second corpus with its Meson entries; retain and diagnose any refusal and disclose the fee-free environment. DONE when accepted-byte continuity validates and the old corpus still admits. Gate: all three exchanges settle, submission is Accepted, exact readback validates, and both corpus admissions pass at (`[ADR025-gate:relay-witness:adoption]`). Supplies `T11-098`; depends on `T11-106` and `T11-107`. The run's four files are admitted on `0.6.245-dev` under `packages/vectors/fixtures/maturity-variable-run-of-record/`, and the live carrier and corpus pins landed on `0.6.244-dev`. |
| `T11-109` | DONE | Close the route track with measured values, evidence identities, row dispositions, remaining obligations and `T11-098`'s evidenced status. DONE when those dispositions close `T11-100` without attributing acceptance to absent evidence. Gate: the full gate plus documentation, label and census checks, against the adoption gate at (`[ADR025-gate:relay-witness:adoption]`). Depends on `T11-108`. Closed on the label gate, this record. |
| `T11-110` | DONE | Test-performance measurement and the first scheduling remedy are recorded in [test-performance.md](research/test-performance.md) and (`rec:backlog:wave0-test-performance`): the dated register retains the comparable before/after pair, ruling 129 is implemented, and the warm wall falls with equal census and attribution. Gate: `scripts/check-plans.sh`, Meson `lint`, and `tripod-labels` tests. Dependencies: none. |
| `T11-111` | DONE | Accepted-byte row classification in `packages/vectors/src/maturity_evidence.rs`: the standing classifier enters `NativeAcceptanceObserved` for the sponsorless positive row from the archive admitted at `MATURITY_VARIABLE_RUN_ADDRESS`, whose native evidence stands answered at its declared boundary and carries an established obligation under the variable schedule with a readback identity equal to the accepted identity; the standing carries run address, schedule, identity, block height and the disclosed zero relay and block fee floors as a declared environment premise. It displaces the historical declared-boundary standing, since one standing stands per row and both are answered. `ConstructorContinuityObserved` stands on no row: a validated continuity report's entries are attributed to the sponsorless row alone, its mutants are experimental by type, and §16.1's four-term row is answered by no single report under §1.14 at (`rule:guide14-exec:evidence-separation`). The classifier recounts `is_answered`, the census and register set equality without changing boundary declarations; it reads the declared-boundary comparison from `expected_layer` rather than restating it. The projector verifies a stated acceptance claim, and the report joins its sources' verified obligations. Ruling 73 at (`rule:phase6:wave9-rulings`) reserved the accepted-byte standing, and the closure at (`sec:phase6:wave9-findings`) stated its evidence boundary. Gate: the acceptance standing arises only from the admitted archive's established obligation; refused, outstanding and node-free projections enter neither answered standing; the continuity bucket stays zero; and the recomputed partition and register equality hold. Designed on `0.6.250-dev`; landed on `0.6.251-dev` (the projector's verified claim), `0.6.252-dev` (the standing and the census recount) and `0.6.253-dev` (the report's join). |
| `T11-112` | DONE | The tagged-hash premise test in `packages/target-elements-conformance/src/tests/constructor_tests.rs` pins each collision and second-preimage premise to its exact leaf, branch and tweak domain, so the recorded premise count of `tweak_evidence_recomputes_both_sides` is carried by six named premise-to-domain pairings rather than a count; no production change. Gate: each pairing's test fails when its domain or premise is perturbed. Depends on nothing beyond the current tree; scheduled beyond the wave. DONE on `0.6.249-dev`: `commitment_premises_name_constructor_hash_domains` and `tweak_evidence_recomputes_both_sides` in `packages/vectors/src/maturity_continuity.rs`, with `a_tag_separates_domains` in `packages/target-elements-conformance/src/tests/constructor_tests.rs`, pin the premise-to-domain association. |
| `T11-113` | DONE | The Wave-10 opening record: rulings 80 through 90 at (`rule:phase6:wave10-rulings`), findings 154 onward at (`sec:phase6:wave10-findings`), these six rows and the register's Wave-10 paragraph, with figures recounted and citations relocated in the opening tree. Gate: the documentation and label gate over the three edited plan files, with both minted card labels resolving. Landed as `0.6.256-dev`; closed by the closure record under `T11-118`. |
| `T11-114` | DONE | The typed edge sequence in new `packages/vectors/src/maturity_history.rs`, with `packages/vectors/src/lib.rs`, `packages/vectors/meson.build` and the continuity module's test sources made crate-visible: `StateRootEdge` carries the two outpoints, `OperationId` and a `MaturityTransitionCertificate` projected from a validated continuity rather than accepted from a caller; `validate_state_root_history` checks outpoint uniqueness over the whole sequence before it reads any edge, then applies §17.1's clauses and `state_bundle_continuity` edge by edge, returning the advanced cursor only after every edge validates, while `MaturityRootHistoryRefusal` retains the clause, position and operands in eight variants. Eighteen tests comprise one accepted-archive edge, sixteen one-axis §16.8 refusals and one cursor-return test; the Meson census gained one entry and no existing test count moved. DONE on `0.6.257-dev`: each clause and fault refuses on its own axis, the three-edge sequence whose final successor equals the honest one refuses at position 1 under clause 4, the accepted archive's one edge advances the cursor to its successor at output 0, and §17.3 calls the linker's `state_bundle_continuity`. This establishes edge ordering over supplied bytes, with no report, row standing, census change or chain observation. Depends on `T11-113`. |
| `T11-115` | DONE | The checkpoint and registry in `packages/vectors/src/maturity_history.rs`, `packages/vectors/src/maturity_evidence.rs` and `packages/vectors/src/maturity_corpus.rs`: `MaturityRootCheckpoint` binds §17.2's ten facts in eleven private fields, six carried by the admitted archive, one read from the public bytes, one derived from the bytes and the locator, and two `MaturityDeclaredPremise` values carrying the new `MaturityPremiseProvenance::DeploymentDeclaration`; its sole constructor `bind` refuses an outstanding acceptance and checks the readback's byte digest once, and `binds` asks three questions (identity, successor, predecessor), each `MaturityRootCheckpointRefusal` retaining both operands; `ModelledBranchPrefix` is admitted by `anchored`, with `from_checkpoint` delegating to it under the continuity's branch identifier, extends and rewinds into new prefixes, and reprojects into a `MaturityRootObservation` or a typed stale refusal raised after the foreign-prefix refusal, while `MaturityModelledBranchRefusal` has five variants with `ZeroPrefixIdentifier` first; `MaturityMutationRegistry` replaces the outstanding-named type, its `registered` constructor carries no visibility modifier and is therefore module-private, it derives the sixteen §16.8 entries from the matrix rows whose locator is `RootHistoryEdge`, and `census()` reads the standing from the list. Six tests, three in history, two in evidence and one in corpus, moved history tests 18→21, evidence 19→21 and corpus 41→42; registry mutations moved 0→16, `MaturityRegistryStanding` members 1→2 and the retired outstanding-reason members 1→0. DONE on `0.6.259-dev`: all ten facts distinguish checked bindings from premises, the sixteen entries are set-equal to the matrix rows in both directions, no public registry constructor exists, a rewound or changed block stales the observation and another prefix refuses reuse, and no evidence-plan census bucket moved. This establishes neither chain freshness nor row standing. Depends on `T11-114`. |
| `T11-116` | DONE | The public handoff in new `packages/vectors/src/maturity_recovery.rs` and new `packages/vectors/tests/maturity_recovery.rs`, with `packages/vectors/src/maturity_continuity.rs` (three items made crate-visible with their documentation), `packages/vectors/src/lib.rs` and `packages/vectors/meson.build`: `PublicAnnouncementHandoff` carries §14.7's three public inputs beside five published deployment parameters, its static subtree being the one-leaf announcement subtree rebuilt from witness item seven under the reviewed target, because the linker substitutes the announcement leaf before it builds the subtree; `recover_public_successor` takes that value alone and returns `RecoveredSuccessor` or one of eleven `MaturityRecoveryRefusal` variants over nine §16.13 rows through a total row mapping; the integration binary imports neither the linker bundle, the deployment identity nor either planner. Thirteen unit tests cover nine §16.13 mutations and four type-level construction proofs, and none reaches `SuccessorCommitmentRefused`: the accepted successor's witnessed representation nonce is zero, a value the landing records rather than pins, so no lower candidate exists, and no public candidate leaf refuses the branch side; two integration tests cover accepted-archive output-0 equality and the complete import list; the Meson census gained two entries and no existing test count moved. DONE on `0.6.261-dev`: the handoff carries exactly its eight public inputs, the recovery function has no second argument, every mutation refuses on its own axis, the four private-dependency exclusions are asserted over the type, the import list is pinned, and the accepted archive's reconstructed program equals output 0. This establishes reconstructibility for that transaction, with no report, row standing, freshness, origin or relayability claim. Depends on `T11-115`. |
| `T11-117` | DONE | The two validated reports and their routing, landed on `0.6.263-dev` (the root-history report in new `packages/vectors/src/maturity_history_report.rs`), `0.6.264-dev` (the public-recovery report in new `packages/vectors/src/maturity_recovery_report.rs`, with the binary's third test and three pinned imports in `packages/vectors/tests/maturity_recovery.rs`), each with `packages/vectors/src/lib.rs` and `packages/vectors/meson.build`, and `0.6.265-dev` (the boundaries, the retyped rows and the recount in `packages/vectors/src/matrix.rs`, `packages/vectors/src/maturity_safety.rs`, `packages/vectors/src/maturity_evidence.rs`, `packages/vectors/src/maturity_report.rs` and `packages/vectors/src/observed_boundary.rs`, with `packages/vectors/src/maturity_negative_half.rs` unedited): `MaturityRootHistoryReport` (twelve fields, nineteen recomputed items, six refusals) and `MaturityPublicRecoveryReport` (fifteen fields, twenty-six items, six refusals and a fourteen-row registry answering one row by recovery, nine by their own refusals and four by construction proofs) have separate assemble-and-validate pairs, roles, residuals, censuses and canonical bytes, the history report rendering the three synthetic-origin residuals and the recovery report those three followed by `witnessed-nonce-without-leastness`; `EvidenceBoundary` gains `RootHistoryReportRejection` and `PublicRecoveryReportRejection`, and `classify` enters `RootHistoryObserved` or `PublicRecoveryObserved` only when the matching validated report observes the row. The sixteen §16.8 rows and the fourteen public-recovery rows, thirty in all, take those boundaries; the two displaced typed reasons leave `MaturityRowBoundary`. Counts moved: boundaries 13→15, row-boundary members 7→5, typed non-answers 65→35, answered 41→71, root-history observed 0→16, public-recovery observed 0→14, and the partition six→eight buckets summing to 206; `STILL_REQUIRED` stays 42, evidence tests moved 21→22, safety tests stay 20, and Meson gained two entries. DONE across the three tags: both reports validate and render their ordered residuals over the accepted archive, reject changed claims at named items, keep their role bytes separate, route exactly the thirty named rows, refuse the evidence derivation through its own refusal member when the accepted replay, the checkpoint or either report refuses rather than leaving rows outstanding, recompute the 206-row partition and the waiting register's set equality, and leave neither retired reason in `packages/`. The eleven acceptance-awaiting positive rows keep their separate question; neither report establishes freshness, chain observation, archive origin, artifact promotion or the other's evidence. Depends on `T11-114` through `T11-116`. |
| `T11-118` | DONE | The Wave-10 precisely partial stopping record at (`sec:phase6:wave10-findings`): it recounts the landed tests, boundaries, standings, census and register at `0.6.265-dev`; disposes of `T11-114` through `T11-117` on their tags and the tests that pin each claim; states the four exits, one valid edge MET with §23's eleventh conjunct CONDITIONALLY MET on the disclosed zero-fee environment and on the edge's recomputation through the creator-side projection, the hidden-invalid-edge exit MET, output-0 reconstruction CONDITIONALLY MET on a function type and a pinned import list, and creator-private state MET for recovery and NOT MET for root history; records `R-5 RECOVERY` ADOPTED with its condition, only the `G14C-02`, `G14C-11`, `G14C-12` and `G14C-13` clauses actually advanced, and the register exit NOT PASSED on `G14C-11`, `G14C-13` and Wave 10's clause of `G14C-12`; and names the residuals for chain observation, cursor freshness, archive origin, the declared checkpoint premises and the edge's creator-side projection, the eleven acceptance-awaiting rows, fee-floor relayability, resources and artifact promotion, with findings 163 to 172. DONE: the evidenced disposition closes `T11-113` and treats neither the modelled branch nor the public-byte reconstruction as a chain or origin proof. Gate: the closure's documentation, label and census checks over the landed evidence. Depends on `T11-117`. Closed on the label gate, this record. |
| `T11-119` | DONE | The root-history checkpoint's premises, findings 164 and 165 at (`sec:phase6:wave10-findings`), in `packages/vectors/src/maturity_history.rs`, `packages/vectors/src/maturity_history_report.rs`, `packages/vectors/src/maturity_evidence.rs`, `packages/vectors/src/maturity_corpus.rs` and the cross-role test of `packages/vectors/src/maturity_recovery_report.rs`, as bite 1 of `plans/drafts/wave10-register-closure-study.md` fixes them: the derivation declares the bundle the accepted continuity was projected under and the ABI over that bundle's view through a crate-private `checkpoint_premises_of`; `MaturityRootCheckpoint::bind` and `binds` take the public handoff in place of the continuity, answer identity, successor and predecessor from its bytes, and compare each premise's published parameters with the handoff's, `MaturityRootCheckpointRefusal` moving from five variants to ten and `bind` losing its `unreachable!` and `# Panics` section; the history report validates against the handoff, gains `CheckpointPremiseProvenance` (nineteen items to twenty) and renders the carried provenance through a new `MaturityPremiseProvenance::name`. DONE when the declared ABI's and bundle's schedules equal the handoff's in the retyped checkpoint test, the plan's whole-metadata premises refuse the binding at the schedule, a premise carrying executor arguments refuses at the provenance item, and the accepted archive's report renders both premise lines with `deployment-declaration` and the one added `recomputed` line. Gate: the bite's readings in section 5 of the study — history tests 22 and history-report tests 10, every other count unchanged, no continuity named by the report above its tests, the census figures unmoved and exactly the five files changed. Depends on `T11-118`. DONE on `0.6.268-dev`: the handoff binds both declared premises, and the report checks and renders their provenance. |
| `T11-120` | DONE | The root edge built from the public handoff alone, finding 166, in `packages/vectors/src/maturity_recovery.rs`, `packages/vectors/src/maturity_history.rs`, `packages/vectors/src/maturity_evidence.rs` and `packages/vectors/tests/maturity_recovery.rs`, as bite 2 of the study fixes it: `RecoveredSuccessor` keeps the two outpoints recovery reads, `StateRootEdge::from_public_handoff` takes the handoff by value and refuses through `MaturityRecoveryRefusal`, and the evidence derivation validates and reports that edge, with `MaturityEvidenceRefusal::RootHistoryEdgeRefused` as its twenty-first member. DONE when the handoff-built edge agrees field by field with the projected edge, static roots compared, validates from its predecessor to its successor at output 0, and the binary builds and validates it from its own handoff with 31 pinned imports. Gate: the bite's readings — history tests 23, binary tests 4, recovery and evidence counts unchanged, no production `StateRootEdge::from_continuity` in the derivation and exactly the four files changed. Depends on `T11-119`. DONE on `0.6.271-dev`: the handoff-built edge validates from its public predecessor to its public successor and agrees field by field with the projected edge. |
| `T11-121` | DONE | `G14C-13`'s non-claims in `packages/vectors/src/maturity_recovery_report.rs`, as bite 3 of the study fixes them: `MaturityPublicRecoveryResidual` gains `NoIndependentImplementation`, `NoTrustAnchor`, `NoInclusionProof`, `NoCanonicalChainAuthentication` and `NoStaticSubtreeDataAvailability` (four members to nine), each with a positional recomputed item, and the census gains `static_leaves` with `CensusStaticLeaves` (twenty-six items to thirty-two). DONE when the accepted handoff's report validates and renders nine residuals in canonical order and `static_leaves 1`, none of the five new residuals appears in the history report's bytes, and a changed census or a removed or reordered residual refuses at its named item. Gate: the bite's readings — recovery-report tests 12, the two roster widths, the five names present in the recovery report and absent from the history report, and exactly the one file changed. Depends on `T11-119`. DONE on `0.6.270-dev`: the validated report renders nine ordered residuals and the one-leaf census beside the five new non-claims. |
| `T11-122` | DONE | Branch realizations, the invalidation event and the thread's semantic history for `G14C-11` in new `packages/vectors/src/maturity_branch.rs`, `packages/vectors/src/lib.rs` and `packages/vectors/meson.build`, as bite 4a of the study fixes them: `MaturitySemanticEdge`, `MaturityRealization`, `MaturityBranchView` with `realize` and `rewind`, `MaturityRealizationSuffixInvalidation`, `MaturityIntendedStep`, `MaturityThread` with `record` and `project`, `MaturityThreadProjection` and `MaturityBranchRefusal`. DONE when a fork after the accepted predecessor realizes the accepted and a host-built competing successor on their own branches while their union refuses as a reused predecessor, a rewind of one branch names exactly its removed realization and leaves the other branch and the thread's semantic history unchanged, and a fresh planner replay re-realizes byte-identical bytes into an identical view. Gate: the bite's readings — four module tests, no `Option` field, no production panic path and exactly the three files changed. Depends on `T11-118`, no code of `T11-119` to `T11-121` being read, and lands after `T11-121` in the study's order. DONE on `0.6.272-dev`: the four module tests pin competing branches, suffix invalidation, preserved semantic history and byte-identical reproduction. |
| `T11-123` | DONE | The branch-bound compare-and-swap token for `G14C-11` in `packages/vectors/src/maturity_branch.rs`, as bite 4b of the study fixes it: `MaturityBranchView::live_cursor`, `MaturityBranchViewToken`, `MaturityThread::token` and `fence`, and `MaturityFencingRefusal` with four variants. DONE when a token fences its own view and one extended by an empty block without refusal and refuses a moved cursor, another branch, a removed checkpoint block and a changed one. Gate: the bite's readings — five module tests, no production caller of the fence and exactly the one file changed. Depends on `T11-122`. DONE on `0.6.273-dev`: the token test pins the passing views and the branch, checkpoint and cursor refusals. |
| `T11-124` | DONE | Wave 10's class-2 clause of `G14C-12` in `packages/vectors/src/maturity_branch.rs`, as bite 5 of the study fixes it: `MaturityThreadStanding`, `MaturityEnvironmentTransition` with its class fixed to the modelled-unintended member, `MaturityEnvironmentRealization`, `MaturityThreadObservation` and `MaturityThread::observe`. DONE when a competing spend loses the thread and fences its token, a same-key instance contests it without moving its cursor, a reorganization-surfaced realization loses it only beside the invalidation event that names the removed realization, and every transition is class 2 under the quantifier with no field able to hold a poison marker. Gate: the bite's readings — nine module tests, no poison-marker type above the tests and exactly the one file changed. Depends on `T11-123`. DONE on `0.6.274-dev`: the three environment tests and the taxonomy test pin the class-two thread standings. |
| `T11-125` | DONE | The stale comments of findings 171 and 172 in `packages/vectors/src/maturity_safety.rs`, as bite 6 of the study fixes them: the documentation of `AcceptanceAwaitsRelayAdmissibility`, the sponsorless row's comment and the standings test's comment, comments only. DONE when each comment states what its code does at the tip. Gate: every changed line a comment, twenty tests unchanged and exactly the one file changed. Depends on `T11-124`. DONE on `0.6.275-dev`: the three comments state their code's landed standing, with assertions, tests and census figures unchanged. |
| `T11-126` | DONE | The closure over `T11-119` to `T11-125` in `plans/phases/06-state-and-maturity.md`, `plans/guides/guide_fourteen_conceptual_register.md` and these rows: findings for each bite, the Status cells of `G14C-11`, `G14C-12` and `G14C-13` with evidence pointers stating that the branches are modelled, the competing successor is never submitted, the same-key comparison reads keys from projected continuities and no chain is observed, `G14C-13`'s non-claim count corrected from four to five in its Status cell, finding 169 and the register-exit paragraph, the renamed tests' citations retyped, §23's eleventh conjunct and the fourth exit restated on the landed tags, and the register exit decided on that evidence. DONE when every clause's disposition cites the tag and test that pin it. Gate: the documentation, label and census checks over the edited plan files. Depends on `T11-125`. DONE on this row's label gate: the seven findings, three CLOSED cells and restated exits cite the landed tags and tests. |
| `T11-127` | DONE | The labels crate's subprocess test `check_forbidden_text_omits_untrusted_git_stderr` in `packages/labels/tests/subprocess_contract.rs` writes a fake git script and execs it under a parallel test binary; another test's child can hold the freshly written script open across the exec window, the spawn fails with the executable-busy error, and the checker reports the spawn-error text where the test expects the exit-status text (one red on a full gate at the vectors tip, green on rerun). DONE when the test either writes and marks the stub before any concurrent spawn in the binary or retries the spawn once on `io::ErrorKind::ExecutableFileBusy`, and the text match stays exact (never widened to accept both paths). Gate: labels tests green under the release profile ten runs in a row on the server. Depends on nothing. DONE on `0.6.283-dev`: one static lock per test binary, a stub writer helper and serialized-command helpers cover writing and spawning without a retry, preserve the exact text match, and passed ten consecutive release-profile labels runs. |
| `T11-128` | DONE | The Wave-11 opening record: rulings 91 through 97 at (`rule:phase6:wave11-rulings`), findings 180 onward at (`sec:phase6:wave11-findings`), these six rows, the register's Wave-11 paragraph and the card's status line, with figures recounted and citations relocated in the opening tree. Gate: the documentation and label gate over the three edited plan files, with both minted card labels resolving. Closed by the Wave-11 closure record under `T11-133`. DONE on `0.6.277-dev`: the opening record's rulings, findings, six rows, register paragraph and card status line passed its documentation and label gate. |
| `T11-129` | DONE | Bite 1 in `packages/transaction/src/tests/state_construct_tests.rs`: `each_request_form_is_built_or_refused_by_name` walks `RequestedForm` crossed with `SponsorChangeRequest` over one validated view and ABI. It pins four distinct forms in order: sponsorless without change builds one input and one output; sponsorless with change returns `SponsorChangeWithoutSponsoredForm` from `MaturityAnnouncementRequest::new`; sponsored without and with change each return `SponsoredMaturityFormHasNoCarrier` from `construct_over`. The file's tests move 12 → 13, `SponsoredMaturityFormHasNoCarrier` occurrences move from 3 to at least 4, and no production line changes. DONE when the four dispositions are asserted at those boundaries. Gate: 13 `#[test]` markers, at least four refusal occurrences, and exactly this one file changed. The host refusal creates no sponsored byte and reaches no node. Depends on `T11-128`. DONE on `0.6.278-dev`: the four-form test pins the named construction and request boundaries. |
| `T11-130` | DONE | Bite 2 in `packages/realization/src/tests/open_flow_tests.rs`: `sponsorship_moves_no_announcement_relation_outside_the_sponsor_family` compares the sponsorless and sponsored observations on all 20 of the declaration's 26 identities outside its six sponsor-family identities, and `a_state_corruption_fails_the_same_state_relation_with_or_without_a_sponsor` makes the output STATE asset fail recognition on both observations while sponsor values remain `SponsorOpaque`. Tests move 14 → 16 in this file; the 22 announcement-maturity tests remain unchanged, with no production line changed. DONE when both assertions hold. Gate: those two test counts and exactly this one file changed. These model observations are not transactions, answer no target-mutation row, and decide no refit of sponsor-family relations. Depends on `T11-128`. DONE on `0.6.279-dev`: the two model tests pin the non-sponsor relation comparison and equal STATE failure. |
| `T11-131` | DONE | Bite 3 adds `the_canonical_bytes_publish_no_sponsor_key` inside each test module of `packages/vectors/src/maturity_continuity_report.rs`, `packages/vectors/src/maturity_report.rs`, `packages/vectors/src/maturity_history_report.rs`, and `packages/vectors/src/maturity_recovery_report.rs`. Each test renders its validated report, uses the public `FORBIDDEN_KEYS` and `forbidden_key_in` vocabulary, catches every forbidden key staged as an appended rendered line, and permits a forbidden word in a value. Test counts move 31 → 32, 20 → 21, 10 → 11, and 12 → 13 respectively; no production line or census figure moves. DONE when each rendering excludes the keys and each staged key is detected. Gate: those four counts, no `forbidden_key_in` use above each test module, and exactly the four files changed. The check concerns keys in sponsorless report bytes; it answers neither `report-publishes-*` row and establishes no sponsored byte. Depends on `T11-128`. DONE on `0.6.281-dev`: the four report tests pin forbidden-key exclusion in each validated rendering. |
| `T11-132` | DONE | Bite 4 in `packages/vectors/src/maturity_safety.rs`: `MaturityRowBoundary` gains documented `SponsoredControlRefusedAtConstruction`; the two §16.1 sponsored positives and nine §16.10 rows take it, while `SponsorArmAwaitsRegionScoping` retains `two-sponsor-envelopes`, `sponsor-member-unclassified`, and `zero-valued-sponsor-member-under-exact-role-structure` with documentation limited to the relations `T11-064` re-scopes or retires. The standings test, its exhaustive match and the section comments state 9 acceptance-awaiting, 11 construction-refused, 3 region-scoping, 11 totality, and 1 build-property rows; tests remain 20, typed non-answers 35, rows 206, answered 71, relation standings 14/102/90, and every census bucket unchanged. DONE when each touched row remains a typed non-answer on its named reason and no canonical byte moves. Gate: the three reason-use readings 9, 11 and 3; 20 tests; `the_census_figures_are_unmoved_by_the_binding_readers`; and exactly this one file changed. It answers no row. Depends on `T11-129`; `T11-125` is landed. DONE on `0.6.280-dev`: the standings test pins the reason split without moving a row answer, census bucket or canonical byte. |
| `T11-133` | DONE | The Wave-11 closure record at (`sec:phase6:wave11-findings`): recount the landed tests, reason standings, census, report keys and register, and dispose `T11-129` through `T11-132` on their gate evidence. It states the three exits against the study's expected CONDITIONALLY MET, MET and MET, with §23's eighth conjunct following the third; retypes the Wave-10 closure's eleven acceptance-awaiting rows where bite 4 moves them to nine; and states C1, the sponsored STATE builder, conditional on `T11-064` and a ruling revisiting ruling 58, C2, the sponsored native ceremony, conditional on C1 and an Elements-provisioned compute node, and C3, the sponsor rows' boundary retyping, conditional on C1 for eleven rows and `T11-064` for three, without minting their rows. The wave owns no register row and changes no status cell. DONE when the evidenced disposition closes `T11-128`, names the remaining sponsored-byte, node, row, guide-text, §16-matrix and positive-fee-floor limits, and leaves the register recount intact. Gate: the closure's documentation, label and census checks over the landed evidence. Depends on `T11-132`. DONE on this row's label gate: findings 199–203, the row dispositions, exits and obligations close Wave 11 on the landed evidence. |
| `T11-134` | DONE | The Wave-12 opening record: rulings 98 through 104 at (`rule:phase6:wave12-rulings`), findings 204 onward at (`sec:phase6:wave12-findings`), these seven rows and the `T11-127` disposition, the register's Wave-12 paragraph and the card's status line, with figures recounted and citations relocated in the opening tree. DONE when those rulings, findings, rows and dispositions state the baseline and both minted labels resolve. Gate: the documentation and label checks over the three edited plan files. Closed by the Wave-12 closure record under `T11-140`. DONE on `0.6.285-dev`: the opening record's rulings, findings, seven rows, register paragraph and card status line passed its documentation and label gate. |
| `T11-135` | DONE | Bite 1 in `packages/linker/src/state_error.rs`, `packages/linker/src/state_resource.rs` and `packages/linker/src/tests/state_resource_tests.rs`: `StateLinkRefusal::ResourceProjectionDisagreement` carries `checked: Option<u64>`, and crate-private `projection_disagreement` admits an equal measured total or a saturating-maximum projection over a measured total while refusing an unmeasured projected dimension with `checked: None`; `an_unmeasured_projected_dimension_refuses_with_no_checked_figure` pins zero and maximum projections of `WitnessBytes`, the measured `ScriptBytes` maximum and the record's own projection, while `every_resource_refusal_is_reached_or_declared` retains its constructed `Some(1)`. Tests move 16 → 17 in `state_resource_tests.rs`; the 14 `state_symbol_tests.rs` tests stay unchanged. DONE when the refusal preserves absence and the four cases are asserted. Gate: 17 test markers, zero `unwrap_or_default` uses in `state_resource.rs` from one, one `checked: Option<u64>` in `state_error.rs`, and exactly the three named files changed. The production refusal remains unreachable, reaches no target and changes no measured figure. Depends on `T11-134`. DONE on `0.6.286-dev`: the projection comparison preserves unmeasured absence with seventeen resource tests and its three-file gate. |
| `T11-136` | DONE | Bite 2 adds `packages/vectors/src/maturity_resources.rs` (new), its `pub mod maturity_resources;` line in `packages/vectors/src/lib.rs` and its census line in `packages/vectors/meson.build`: `MaturityResourceField` names §14.8's 21 fields in order and the eleven-dimension roster; `MaturitySubmission`, `MaturityWitnessPosition`, `MaturityPrediction`, `MaturityObservationStanding`, `MaturityUnobservedReason`, `MaturityResourceFailure` and `MaturitySubmissionResources` keep figures, the reviewed zero, per-role values, verdict layers and typed absence separate; `compare_submission` compares one prediction with one admitted response, and `archived_submission_resources` reads both corpora. Six new tests — `the_fields_are_section_fourteen_eight_in_order`, `the_roster_stands_six_measured_three_elsewhere_one_unemitted_one_subjectless`, `the_archived_submissions_predict_their_own_bytes`, `the_target_observed_one_dimension_and_it_agrees`, `a_mismatch_records_both_figures_and_fails_by_type` and `no_absent_prediction_is_a_zero` — pin 21 fields, two noncanonical fields, six leaf totals, three other host predictions, one unemitted and one subjectless dimension, the archived widths and weights, two weight agreements and a typed synthetic mismatch; the corpus and native modules remain at 42 and 32 tests. DONE when transaction weight agrees with the target's own response for both archived submissions and every other observation has its named standing. Gate: six new test markers, one Meson census entry, no `LiveInfrastructureBlocker` in the new module, and exactly the three named files changed. It establishes no observation beyond those two node-reported weights, no verdict for an unjudged candidate and no report-level consequence of the synthetic mismatch. Depends on `T11-134`. DONE on `0.6.287-dev`: six resource tests pin the field and roster vocabulary, archived predictions, two node-weight agreements and typed absences. |
| `T11-137` | DONE | Bite 3 edits `packages/vectors/src/maturity_native.rs` and adds `packages/vectors/src/maturity_measurements.rs` (new), its `pub mod maturity_measurements;` line in `packages/vectors/src/lib.rs` and its census line in `packages/vectors/meson.build`: `MaturityLead` gives `MaturityAnnouncementPlanner::at_lead` minimum, interior and maximum choices while `new` retains the interior rule; `MaturityResourceCase` names §18.2's fourteen cases, `MaturityCaseStanding` types ten measured cases, one corpus reading, two construction refusals and one subjectless sponsor shape, and `MaturityMeasuredCandidate` with `measure_maturity_cases` builds six candidates over the two archives' funding. Six new tests — `the_fourteen_cases_are_section_eighteen_two_in_order`, `each_lead_announces_its_window_edge`, `the_interior_lead_is_the_archived_submission`, `every_candidate_of_one_schedule_costs_the_same`, `both_sponsored_cases_are_refused_where_wave_eleven_pins_them` and `the_largest_metadata_witness_is_the_replay_only_schedule` — pin case order, leads 9/10/11, archived interior bytes, within-schedule figures, both sponsor refusals and the 86-byte replay-only witness against the 53-byte variable witness; native and corpus tests remain 32 and 42. DONE when all six candidates and fourteen standings have those asserted boundaries. Gate: six new test markers, 42 corpus test markers, one Meson census entry, and exactly the four named files changed. No target judged the four added candidates; their target observations stand absent, and width equality describes this generation's encoding. Depends on `T11-136`. DONE on `0.6.288-dev`: six measurements tests pin all fourteen cases, six candidates, three leads, both sponsored refusals and the replay-only width boundary. |
| `T11-138` | DONE | Bite 4 adds `packages/vectors/src/maturity_nonce_corpus.rs` (new), its `pub mod maturity_nonce_corpus;` line in `packages/vectors/src/lib.rs` and its census line in `packages/vectors/meson.build`: `MaturityNonceDerivation` names each archive, constructor side, lead, selected nonce, attempt count and lower-candidate refusals; `MaturityNonceCorpus` carries eight real-curve derivations under the reviewed 4,096-attempt budget and names the greatest; `NONCE_CORPUS_RESIDUAL` carries the landed later-admissible-nonce residual. Five new tests — `the_corpus_is_eight_named_derivations`, `every_selection_is_the_least_admitted_nonce`, `the_archived_derivations_are_the_archives_own`, `the_greatest_attempt_is_stated_beside_the_budget` and `the_residual_claims_no_totality` — pin independent leastness, attempts equal selection plus one, the archived whole pair's 2 and 1 attempts, the greatest first reading and exhaustion one attempt short. DONE when the bounded corpus and residual provide `G14C-03`'s Wave-12 evidence without a totality claim. Gate: five new test markers, one Meson census entry and exactly the three named files changed. Host leastness is not target leastness, and eight derivations say nothing about metadata outside the corpus. Depends on `T11-137`. DONE on `0.6.289-dev`: five nonce-corpus tests pin eight named derivations, independent leastness, the greatest attempt and the residual. |
| `T11-139` | DONE | Bite 5 adds `packages/vectors/src/maturity_resource_report.rs` (new), its `pub mod maturity_resource_report;` line in `packages/vectors/src/lib.rs` and its census line in `packages/vectors/meson.build`: `MATURITY_RESOURCE_SCHEMA_ID`, `MaturityResourceReportRole`, `CandidateMaturityResourceReport` and `ValidatedCandidateMaturityResourceReport` make a separate validated resource document; `MaturityResourceResult` names the one observed dimension over two submissions; `MaturityCalibrationResult` and seven `MaturityCandidateParameter` members state no final calibration; `MaturityResourceDiagnostics` separately renders caller-measured construction time, nonce attempts and archived sidecar status with absent duration; `MaturityResourceReportRefusal` retains both mismatch figures, and `assemble_maturity_resource_report`, `validate_maturity_resource_report` and `render_maturity_resource_report` validate before canonical rendering. Six new tests — `the_report_states_one_result_and_its_non_calibration`, `a_mismatch_fails_validation_before_any_result`, `no_absent_figure_renders_as_zero`, `the_canonical_bytes_carry_no_timing_or_forbidden_key`, `every_absence_names_a_live_reason_and_no_stale_blocker` and `the_canonical_bytes_are_stable_across_renderings` — pin the schema, typed mismatch, absence reasons, diagnostics separation, blocker census and byte stability. DONE when validation refuses a mismatch before any result and the canonical bytes state the narrow agreement and no numeric absence or calibration claim. Gate: six new test markers, no `wall_clock` or `elapsed` above the test module, one Meson census entry, and exactly the three named files changed. The one result covers two archived submissions and one observed dimension; host timing supplies no target execution duration or relayability claim. Depends on `T11-135` through `T11-138`. DONE on `0.6.290-dev`: six resource-report tests pin validation before rendering, one weight-agreement result, typed absence, separate diagnostics and stable canonical bytes. |
| `T11-140` | DONE | The Wave-12 closure record at (`sec:phase6:wave12-findings`): recount the landed measurements, comparisons, cases, nonce derivations, diagnostics, test counts and register; dispose `T11-135` through `T11-139` on gate evidence; state the three exits against their opening NOT MET standings, with the first MET for transaction weight over the two archived submissions after `T11-136` and rendered by `T11-139`, the second MET after `T11-139`, and the third MET after `T11-135` and `T11-139`; and decide §23's fifteenth, seventeenth and nineteenth conjuncts at those exact scopes. The record applies ruling 100's closing sentence to `G14C-03` on `T11-138`'s corpus evidence, restates Wave 6's handoff as six measured and five absent dimensions, and names N1 target runs of added leads and N2 archived execution durations under an Elements-provisioned compute-node condition without minting their rows. DONE when the evidenced disposition closes `T11-134`, accounts for all three exits and register status, and keeps positive-fee-floor relayability with a fee-paying form under Wave 11's C1 and deployment. Gate: the closure's documentation, label and census checks over the landed evidence. Depends on `T11-139`. DONE on this row's label gate: findings 215–220, the row dispositions, exits and obligations close Wave 12 on the landed evidence. |
| `T11-141` | DONE | Rewrite the root `README.md` whole, as one cross-file design step scheduled with the Phase-6 close-out and landed before the exit gate at (`gate:phase6:exit`) is read. The file was last revised when Phase 6 opened and its content predates the phase's work: its layout names fourteen of the sixteen workspace crates and omits `packages/vectors/` and `packages/document-stamps/`; it scopes the realization to compact ASH and live receipt transfer and the transaction crate to the compact-ASH ABI, while the tree carries the STATE constructor, the maturity announcement, its candidate reports and the run-of-record corpus; every section after the layout was written against an earlier tree. A patch-by-patch repair would keep the earlier document's structure, which does not follow the tree's, so the rewrite starts from the tree as it stands at the close-out — the crate roster, each crate's own README and the package contracts under `plans/packages/`, the phase record, the ADRs, the gate's lane list, the tag and version rule — and states what the repository is, what it carries, how it is built and gated, and what its evidence establishes and does not, in the tree's own vocabulary, restating no plan and claiming nothing the tree does not hold. DONE when every crate under `packages/` is named with a one-line contract agreeing with its own README and, where one exists, its contract document; every command shown runs against the tree; the version registry and CI sections agree with `Cargo.toml`, `meson.build` and the gate's lane list; no sentence scopes a crate to fewer operations than it carries; and the documentation and label gate, with the forbidden-text and hash-citation checks, is green over the new file. |
| `T11-142` | DONE | The Wave-13 opening record: rulings 105 through 117 at (`rule:phase6:wave13-rulings`), findings 221 onward at (`sec:phase6:wave13-findings`), these six rows, the register's Wave-13 paragraph and the card's status line, with figures recounted and citations relocated at the opening tree. DONE when those rulings, findings, rows and statements give the five bites their baseline, both minted card labels resolve, and the register's four OPEN and eleven CLOSED cells remain unchanged. Gate: the documentation and label gate over the three edited plan files. Closed by the Wave-13 gate record under `T11-147`. |
| `T11-143` | DONE | Bite 1 edits `packages/labels/src/plans.rs`, `packages/labels/src/tests.rs` and `packages/labels/README.md`: `check_plans` calls `verify_gate_record` after `verify_unique_row_ids`; its pure `gate_record_failures` scan reads the Phase-6 card's gate-record section, stays dormant when the section is absent and the card reads `Active`, and fails without the section for any other status. `PHASE_SIX_CARD`, `HANDOFF_NON_CLAIMS`, `FORBIDDEN_FINAL_ARTIFACTS`, `REPORT_TEMPLATE_VERDICTS` and `CONJUNCT_STANDINGS` bind the four register clauses, six final names, five §25 verdicts and three row standings. The scan requires cited conjunct rows 1–23 and R and the five keys `release-complete`, `final calibration claim`, `production key claim`, `Phase-6 verdict` and `§23 verdict`; it refuses six promotions: release completeness, final calibration, production key custody, an outside verdict, a final artifact name and PASSED over any row other than MET. Thirteen stated-text tests in `tests.rs` cover the complete record, each omitted clause, missing evidence, missing section under a non-Active card and each promotion; `the_gate_record_rule_runs_under_check_plans` in `plans.rs` pins integration. Tests move 157 → 170 and 37 → 38. DONE when the structural rule and its refusals pass while the Active card has no gate-record section. Gate: 170 and 38 test markers, a valid plans lane over that dormant state, and exactly the three named files changed. The bite proves its rule over one card, not the gate record or claims outside its structured keys and phrases. Depends on `T11-142`. |
| `T11-144` | DONE | Bite 2 edits tests only in `packages/vectors/src/maturity_recovery_report.rs` and `packages/vectors/src/maturity_resource_report.rs`: each gains `independent_assemblies_render_identical_bytes` on the history report's form. Two independently built accepted handoffs each assemble and validate a recovery report against their own inputs before their canonical bytes compare; two fresh resource derivations each assemble and validate before their canonical bytes compare. Tests move 13 → 14 and 6 → 7. DONE when the recovery bytes agree with `programs_agree true` and the resource bytes agree from `schema 1` after independent validation. Gate: 14 and 7 test markers and exactly the two named files changed. The comparison is within one build and toolchain, commits no golden bytes and claims nothing for a later schema. Depends on `T11-143`. |
| `T11-145` | DONE | Bite 3 adds `packages/vectors/src/maturity_refinement.rs` (new), its `pub mod maturity_refinement;` line in `packages/vectors/src/lib.rs`, its census line in `packages/vectors/meson.build`, and restates two documentation sentences in `packages/vectors/src/maturity_report.rs`. `MaturityBoundEnvironment`, `MaturityConcreteState` and `abstract_state` state a typed concrete-to-semantic relation with decode, reconstruction and program-mismatch refusals; `MaturityStepStanding`, `simulate_forward` and `MaturityTheoremClause` check predecessor, `announce_maturity` and successor over the admitted accepted step and map safety, continuity, history and recovery reports to clauses. `MaturityRepresentabilityAssumption` names six bounds, form, subtree, window, schedule and signer assumptions; `check_supported_step_completeness` covers both archives' requests at cycles 9, 10 and 11 and refuses 8 and 12; `MaturityRefinementNonClaim` names four limits. Six new tests — `the_accepted_step_forward_simulates_to_exactly_one_semantic_successor`, `the_abstraction_is_a_function_of_the_concrete_state`, `the_historical_step_is_host_projected_and_not_accepted`, `every_request_inside_the_window_is_constructible_under_the_named_assumptions`, `each_report_witnesses_exactly_one_clause` and `the_non_claims_state_what_a_finite_corpus_cannot` — pin those boundaries. The safety report retains 21 tests; continuity retains 58 and measurements 6. DONE when the one accepted-step witness and finite window check carry `G14C-02`'s scoped evidence without a general-theorem, outside-corpus, chain-observation or sponsored-form claim. Gate: six new test markers, one Meson census entry, 21 safety-report markers and exactly the four named files changed. Depends on `T11-144`. |
| `T11-146` | DONE | Bite 4 sweeps fifteen files: `packages/architecture/README.md`, `packages/linker/README.md`, `packages/realization/README.md`, `packages/target-elements/README.md`, `packages/transaction/README.md`, `packages/vectors/README.md`; `plans/packages/compiler.md`, `plans/packages/linker.md`, `plans/packages/realization.md`, `plans/packages/tapscript.md`, `plans/packages/target-elements.md`, `plans/packages/transaction.md`, `plans/packages/vectors.md`; and `packages/transaction/Cargo.toml` and `packages/vectors/Cargo.toml`. It restates the four stale README openings, realization's Phase-1 scope, vectors' run and row sentences, architecture's four Phase-6 identifiers and schema 18, target-elements' announcement form and relay bound, seven contract status and direct-dependency headers, and two manifest descriptions; touched legacy transaction paragraphs use one-line paragraph form. No test count or plans file count moves. DONE when each swept README, contract and manifest description states the crate's carried scope and each contract header equals its manifest's direct dependencies. Gate: zero `compact-ASH` matches in both descriptions, no `Phase-1 scope` line in the realization README, at least one `TAPSCRIPT_STACK_ITEM_RELAY_LIMIT` in the target-elements README, equal contract dependency sets, the full gate including plans, forbidden-text and hash-citation lanes, and exactly fifteen named files changed. The Phase-6 sections below those openings retain their existing scope; `T11-141` carries the root `README.md` after this sweep. Depends on `T11-145`. |
| `T11-147` | DONE | Bite 5 edits `plans/phases/06-state-and-maturity.md`, `plans/guides/guide_fourteen_conceptual_register.md` and `plans/backlog.md`. The card mints the gate-record label under bite 1's rule and records §23's twenty-three conjuncts plus the register conjunct, the card gate's ten conditions on their literal text, §25's five keys and its blocks as the complete result matrix with all 135 outstanding rows by name, the four identity, schema, security and dependency impact tables, the four `G14C-15` non-claims, and §26's handoff with the named construction guarantee and qualifications. It names the open rows and their carriers, lists the owner-reviewed closeout's moves as unperformed, disposes the Wave-13 rows on their landing tags, and states three exits while the card remains `Active`. The register closes `G14C-06`, `G14C-14` and `G14C-15` on their evidence and `G14C-02` on ruling 116's scoped cell; the backlog marks `T11-112` DONE on `0.6.249-dev` and corrects its condition and next-gate lines. DONE when that evidenced record closes `T11-142` under bite 1's rule, cites the gate of record over the tip below it, gives the register conjunct its evidenced standing, and preserves §23's NOT PASSED verdict wherever conjuncts 5, 7, 11, 13 and 14 remain CONDITIONALLY MET. Gate: the plans lane with the rule active, the §23 verdict reading NOT PASSED under those conditions, an Active card and exactly the three named files changed; the record's own text passes its documentation gate. The closeout alone moves the welded phase declarations and decides OQ4; this bite moves no conjunct by prose and performs no exit. Depends on `T11-146`, on `T11-141`, which lands under its own row after the sweep, and on `T11-152`. |
| `T11-148` | DONE | Bite 1 adds `packages/vectors/src/maturity_mutants.rs` (new), its `pub mod maturity_mutants;` line in `packages/vectors/src/lib.rs` and its census line in `packages/vectors/meson.build`, and widens `fixed_projection` and `prefix` in `packages/vectors/src/maturity_continuity.rs` to `pub(crate)`. `MaturityMutantRow` names the seven staged rows with their section, matrix name, step, site and declared layer; `MaturityMutantTable` partitions predecessor and successor rows; `FOREIGN_INTERNAL_KEY`, `foreign_key_predecessor_program` and `stage_maturity_mutant` produce exact bytes from a signed honest submission and re-sign changed signed fields through `OperatorSigningRequest::freeze`. Nine new tests pin declared positions and host clauses, re-signed verification, public-constructor foreign-key refusal, metadata-leaf commitment, key-path shape, matrix declarations, determinism and absence of a successor-recipe position. DONE when each stage changes exactly its declared positions and the host projector or commitment recomputation reaches its row's clause. Gate: nine new module tests, 58 unchanged continuity tests, one Meson census entry and no changed file under `packages/tapscript/`, `packages/linker/` or `packages/transaction/`. Depends on this amendment under `T11-142`. |
| `T11-149` | DONE | Bite 2 adds `packages/vectors/src/maturity_mutant_ceremony.rs` (new), its module line in `packages/vectors/src/lib.rs` and census line in `packages/vectors/meson.build`, `packages/vectors/tests/maturity_predecessor_mutants.rs` and `packages/vectors/tests/maturity_successor_mutants.rs` (new) with census lines, and `MaturityNativePlanRefusal::MutantStage` in `packages/vectors/src/maturity_native.rs`. `PREDECESSOR_MUTANT_STEPS`, `SUCCESSOR_MUTANT_STEPS`, `MaturityMutantCeremonyPlanner`, matrix-derived `declared_layer`, the closed carrier table and `MaturityMutantEvidence::from_transcript` give every offer a row-bearing `MaturityNativeRefusal`; each test target has one ignored real-target test and five scripted tests. DONE when scripted runs settle both rosters, declared-layer refusals bind, other-layer and unexpected-detail refusals remain recorded without binding, and no mutant step has a live mutation fact. Gate: eight new module tests, six per target with one ignored per target, three Meson census entries, 32 unchanged `maturity_native.rs` tests and the variable run still admitting. Depends on `T11-148`. |
| `T11-150` | DONE | Bite 3 adds `scripts/live-native-maturity-mutant-capture.sh` and `scripts/test-live-native-maturity-mutant-capture.sh` (new) and wires `live-native-maturity-capture-contract` and `live-native-maturity-mutant-capture-contract` in `meson.build`. The new driver takes `predecessor-constructor` or `successor-constructor` and a new capture directory, retains the maturity driver's environment contract and pinned tip, selects one target and test with a one-test, one-ceremony, no-setup census, and writes `phase-wall-ms` lines in its `RUN-REPORT`. DONE when both contracts pass as lanes and `scripts/live-native-maturity-capture.sh` remains byte-identical to its base. Gate: both lanes green, Meson lane count 52 → 54, and empty diffs for the existing driver and contract. Depends on `T11-149`. |
| `T11-151` | DONE | The ceremony runs both driver profiles on an Elements-provisioned compute node from a clean suite worktree at `T11-150`'s landing tag under the study's section 5 environment contract. It delivers `packages/vectors/fixtures/maturity-predecessor-mutant-run-of-record/` and `packages/vectors/fixtures/maturity-successor-mutant-run-of-record/` (new): two capture directories of four files each, concluding `eligible yes`, copied to the host and reviewed as text; `T11-152` is their landing. DONE when both directories exist, every offer's layer and detail have been checked against D3's declared table, and each control has accepted mined readback. Gate: both drivers' concluding `eligible yes` and the capture review; an accepted mutant or unexpected detail is recorded as a finding before admission. Depends on `T11-150`. |
| `T11-152` | DONE | Bite 4 admits both new mutant-run-of-record fixture directories under `packages/vectors/fixtures/`, adds eight census lines to `packages/vectors/meson.build`, and edits `packages/vectors/src/maturity_corpus.rs`, `packages/vectors/src/maturity_evidence.rs`, `packages/vectors/src/maturity_negative_half.rs` and `packages/vectors/src/maturity_report.rs` for `MaturityCaptureTarget`, `MaturityMutantCorpusPins`, `admit_maturity_mutant_corpus`, both run accessors, roster replay, row-bound refusal standing, recomputed census and schema-2 `native_refusal` rendering. It restates the run-of-record and row-standing sentences in `packages/vectors/README.md` made stale by the admitted runs, and any swept document's Meson lane count changed by `T11-150`. Eight new tests cover both pins and replay, changed-byte refusal, earlier-run admission, nonbinding phase walls, seven row bindings, unbound refusals, the rendered refusal block and altered-line validation. DONE when both mutant runs and both earlier runs admit, each bound refusal stands `NativeRefusalObserved`, and the census, register, report and swept prose state the admitted figures. Gate: corpus tests 42 → 46, evidence 22 → 24, report 21 → 23, negative half stays 7; 206 rows with seven bound refusals imply native refusal observed 0 → 7, native run required 42 → 35, answered 71 → 78 and `STILL_REQUIRED` 42 → 35, otherwise the run's figures govern; schema 2, eight fixture census entries, the full gate and exactly the files named by this bite. Depends on `T11-151`. |




Current blockers are: NONE carried by the live evidence set, whose residual set is now EMPTY. `PredecessorConstructorAbsent` was the last member and it left by a RULING rather than by a run: the observed pair its clearing rule demanded was demanded by a mis-typing of the `time-locked-input` row, §10.4 refuses the maturity half because the time-locked class is a receipt class rather than a consensus timelock, and the refusal half attributes to a leaf the spent program does not commit to, which every foreign taptree draws alike — so no observation of it could ever name the lock. The row is retyped FIRST-PARTY beside its `time-locked-output` sibling and discharged against the constructor typing that is the real protection. An empty residual set is not a smaller set of obligations: the rows a run has still to answer are counted by the census and not by it, and no time-locked spend was ever observed refused. `SponsorEnvelopeSignerAbsent` is CLEARED, by its own defining site's rule and on an observed result rather than on a capability existing: a sponsor-signed explicit control was accepted by a real node, read back from the node's own copy byte for byte, and mined, which is the acceptance that rule said a returned byte stack had to wait for. The sighash blockers are cleared, the confidential predecessor is funded, the signing handoff is closed, and the evidence restart has taken its first five steps against a real node — six positive private matrix rows are answered by observed acceptances of their own shapes, the three proof-negatives are observed at the mempool boundary and attributed by field, and the multi-output and multi-input fixture constructor exists and derives at any width. The restart order stopped typed at step six on the sponsor signer dependency through three waves; that dependency closed by observation, and the minimality wave recorded step six and step seven in a wave-eight ledger of its own — the wave-seven ledger is a record and was not amended. The order is COMPLETE: all seven steps accepted, the disposition is `Completed`, and both accepted steps state their own shortfalls in their entries. Eight positive private rows have moved on observed acceptances of their own shapes, private-merge and private-sponsor-values being the two the minimality wave added — the sponsor row had moved in the guide closeout while the safety matrix still classified it as awaiting a run, and the matrix caught up in the same wave rather than being left to disagree with the closeout. Of the remaining two, deterministic-public-fixture-openings is now ANSWERED in the safety matrix on a minted determinism standing — its §11.2 gate is the byte-identity contract rather than an acceptance, the wave-7 observation was already recorded, and the new member is counted in its own bucket so a determinism observation is never read as an acceptance; it stays out of the closeout delta, which admits target-computed identities only, and the two artifacts disagreeing there is the vocabulary difference showing rather than a row nobody is checking. Projection-equality-with-paired-explicit did NOT move and the failing conjunct is named: its rule wants an observed acceptance of its own shape, and no run compares the semantic projections of an accepted private transaction and its paired accepted explicit one — the equality this workspace holds is first-party, over registry members that never reached a target. The remaining work is an observation comparing two accepted transactions' projections, and runs of the two pair shapes that have none; one question is still handed up for a ruling, the deterministic-fixture-openings gate, and the step-six/step-seven ordering question is answered by the erratum filed against the guide's mandatory chain. The sponsor dependency is CLOSED and its closure was a repair to this repository rather than to the target: the deployment's fee-role program digest was a fixture constant no program hashes to, threading it to the digest the target itself computes for the empty fee program construction writes produced the first observed sponsor-signed acceptance, and the record says in its own bytes that this establishes the sponsor envelope's wire and one acceptance and not production multi-party sponsor signing.
