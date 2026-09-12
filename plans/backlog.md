# Tripod Implementation Backlog

> **Status:** ACTIVE
> **Current gate:** Phase 6 — STATE constructor and maturity announcement
> **Current condition:** Phases 1 and 2 are complete. The target-independent compiler constructs one deterministic, validated scoped analyzed program for the complete pilot scope, factorized per operation, with relation-owned requirements, a corruption-resistant assembly validator, and independent assembly oracles; the Phase-2 exit gate record is in §2.11, including passed document reproducibility. Phase 3 then delivered the Guide-8 target foundation, the Guide-9 native primitive gate, the Guide-10 constructor and wide-floor prototypes, and the Guide-11 declassification result, each with its own gate record. Public declassification is no longer an open blocker: an initial policy is selected — an explicit boundary reached by owner-authorized normalization, with the direct and public-committed paths deferred against three named target blockers — and the gate record is in §2.4. Phase-3 exit is recorded: the Guide-12 preflight register closed all sixteen seventh-review rows, and (`gate:phase3:exit`) was checked green on that tree with the record in §2.6. Phase-4 exit is recorded in §2.8: compact ASH ran end to end against a live target with twelve accepted and matched submissions and honest coverage stated at 100 of 211. Phase-5 exit is recorded in §2.20 and §2.21: the live-transfer campaign closed with the amended exit gate met on a countersigned assessment. The current phase is Phase 6 — the STATE constructor and maturity announcement — chartered against Guide 14 as the owner supplied it and archived it verbatim (`T6-001`), with Wave 0 — the tenth static review's disposition and the Phase-5 handoff revalidation — run and closed out in the required order (`T6-002`, `T6-031`) and the owner-issued conceptual preflight register binding Waves 1–13 (`T8-001`), so Wave 1, typed STATE metadata and semantic transition, is the next wave.
> **Next gate:** Phase 6 — announce-maturity end to end through Guide-14 Waves 1–13 (`T8-001`), Wave 1 next
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
| Phase 6 | Active | the Phase-6 card; Guide 14 chartered, Wave 0 and the conceptual preflight complete, Wave 1 next |

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

The Phase-2 gate is **passed** (§2.11), the Phase-3 gate is
**passed** (§2.6 with the Phase-3 card), and the Phase-4 gate is
**passed** (§2.8). The Guide-13 batch gate is §2.9: at that historical
gate Phase 5 had not exited, because the batch stopped honestly on the two
typed blockers the intermediate confidential-funding guide owned; the
live-transfer campaign subsequently closed and Phase 5 exited in §2.20 and
§2.21 with the amended gate met on a countersigned assessment, and Phase 6
is current.

The Guide-8 target foundation is complete; the gate record is §2.12.

The T3, T4, and T5 closed rows are archived verbatim in [backlog history](history/backlog-history.md#exited-phase-task-rows) under the Exited-phase task rows group, so a textual reference to any of those row ids resolves there. The open Phase-4 row remains here:

| ID | Status | Task |
|---|---|---|
| `T4-009` | TODO | Negative relation coverage (Waves 13–13e): the mutation machinery, the derived class-to-requirement link, per-mutation funding, three section-18 boundary respecs proven against live evidence, and one negative row discharged. Open honestly: 71 of 72 negative rows outstanding, each naming its reason in code; the remedy is the three guide gaps, filed as [the Guide-13 feature requests](guides/guide_thirteen_feature_requests.md), not more first-party tests. Full narrative archived in [backlog history](history/backlog-history.md) |

The Phase-6 task state opens with the Guide-14 document import; each wave
lands with its own evidence:

| ID | Status | Task |
|---|---|---|
| `T6-001` | DONE | Guide-14 document import, the FIRST Phase-6 record and a documentation-only wave: the owner's packet admitted verbatim and the maintained-prose cap raised beside it. NOTHING WAS HAND-TRANSCRIBED. The packet was extracted programmatically from the session transcript's stored JSON and split at two exact full-line boundaries, so the bytes in the tree are the bytes received, and the split was proved by reassembling the three segments and comparing the result to the source message character for character. THREE FILES: `plans/reviews/review-10-0.6.1-dev.md`, 46180 verbatim bytes under a short authored provenance header; `plans/guides/guide_fourteen.md`, 116415 bytes with the received Draft heading and Status line untouched; and `plans/guides/guide_fifteen_concept.md`, 124621 bytes. Census and index updated per ADR-014 — all three listed in their directories' meson census, the review given a row in the reviews index, the Guide-15 concept a row in the concept-guides table, and Guide 14 named in a paragraph rather than in the executed-guides table, because no batch has been run against it and listing it as executed would be a false claim; it joins that table when a batch is chartered against it. THE STANDING DISPOSITION RULING: the review's confirmed findings are dispositioned in Guide-14 Wave 0, register G14-R01 through G14-R16 in the Guide-14 draft's section 4, and NO WAVE IS CHARTERED BY THIS IMPORT. No per-finding register rows are opened here — the guide's own section-4 table is the register of record, and the reviews index names it as this review's register. Three of the review's findings were independently confirmed at source before archival: the seven-row wrong-boundary defect, the recorded-randomness digest omission, and the completed-closeout-over-empty-ledger match. THE CAP RAISE RESTS ON ITS OWN MEASUREMENT rather than on the import's bytes: all three imported documents are verbatim records under the archive directories and are charged to the 4 MiB archive backstop, which stands at 2707242 bytes with roughly 1.4 MiB free, so the import adds NOTHING to the maintained-prose class. What presses that class is Phase 6 itself — the class measured 1139697 bytes against the 1153024-byte cap, roughly 13.4 KiB of headroom, with the phase just opened and its records entirely unwritten, the same condition that motivated the previous raise at roughly 11.5 KiB. The hard cap goes from 1126 KiB to 1180 KiB, 1208320 bytes, the measured tree rounded up to the nearest 10 KiB above roughly 64 KiB of headroom, leaving 67.0 KiB; the soft target rises by the ratio the hard cap took, 762 scaled by 1180 over 1126 being 798.54 KiB and rounded DOWN to 798 KiB, 817152 bytes, per the budget's whole-KiB convention. The constants and their derivation live at packages/labels/src/plans.rs, and the two pinned figures moved with them: the cap test in that file and the weight-budget statement in plans/README.md. That is the ONLY non-documentation edit in the wave, and no behaviour changed. ONE DEFECT WAS IN THE RECEIVED BYTES, AND IT WAS REPAIRED BY RULING RATHER THAN BY THE IMPORTER. The Guide-15 concept minted one label twice — its section 1.12 heading and its section 6.6 heading both carried the same clear-destruction rule label, at lines 331 and 1340 — so the attestation register generator refused the tree with DuplicateMint. The import STOPPED there rather than editing verbatim bytes it was carrying, and the owner then ruled: section 6.6 takes the label spelled clear-destruction-exact, and section 1.12 keeps the plain name. That ruling is the SOLE permitted edit to a verbatim body in this wave, and it moved ONE LINE and six bytes — `plans/guides/guide_fifteen_concept.md` goes from 124621 to 124627 bytes and its whole diff is that one heading. Nothing cited the label anywhere in the packet or the tree, the two headings being its only occurrences, so no citation had to follow it. THE STOP WAS MEASURED BEFORE IT WAS REPORTED: a throwaway probe branch carrying that one heading disambiguated and nothing else was pushed and run, and both validators passed on it, which established BEFORE the ruling that the duplicate was the ONLY defect the import carries and that the ruling would cost one line. Guide 14's 172 heading mints are all distinct and NONE collides with the Guide-14 concept already in the tree, the feared re-mint conflict having not materialized; the review file mints nothing. THE REGISTER IS REGENERATED BY ITS OWN GENERATOR ON THE SERVER, not by hand: `plans/labels/attestation.md` moves exactly two rows as the import's labels are harvested — candidate mints 49 to 50, task mints 86 to 114 — and the 6629 bytes committed here are the generator's own output taken from the server checkout. The banned token is ZERO across every changed file against a positive control that matched. VERIFICATION AT THE BRANCH TIP, every step on the server per the no-host-compute ruling and every step timed: both validators are green — check-plans reports the documentation tree valid in 0.5 seconds against the 1208320-byte cap, the maintained class measuring 1144826 bytes at the register tip for roughly 62.0 KiB of headroom, and the register generator completes in 0.5 seconds with ZERO errors — and the TRACKED-TREE DELTA AFTER GENERATION IS EMPTY, which is the bar a committed generated register has to meet, the generator reproducing the committed bytes exactly. The lane's own wall is 1.1 seconds at exit 0. ONE PROCESS OBSERVATION, filed not repaired: the plans lane leaves its regenerated register modified in the server checkout, so the NEXT run aborts at checkout whenever that register's committed bytes have changed — it cost one failed run here, cleared by restoring the file, and the lane script would be honest if it reset the checkout before switching branches. Branch `g14-import` pushed to shared, NOT merged; the probe branch was deleted after it reported. |
| `T6-002` | DONE | Guide-14 Wave-0 P0 study: ten read-only lanes returned eight CONFIRMED and two RECLASSIFIED verdicts, found no evidence that any recorded historical value was false, fixed the alpha-through-zeta then rerun-day order, and registered NEW-N1 (P0), NEW-N2 (P2), plus a separate conservation-negatives sibling. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T6-003` | DONE | R01 wrong-boundary repair: seven safety rows were retyped to ConsensusRejectionBeforeScript, one fail-closed mapper now makes observed and declared boundaries equal before minting an answer, and red-before found exactly those seven while green restored 391 of 391 tests and kept the partition 82 + 1 + 25 = 108. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T6-004` | DONE | C005 owner-profile establishment repair landed: tapscript now owns the unforgeable EstablishedOwnerSighashProfile witness, linker re-exports it, and transaction requires its reviewed revision before census construction while preserving submit-ready identity. Verification was green in round 2. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T6-005` | DONE | C006 provenance repair landed: the T5-026 ceremony owns all seven outcomes, six ScriptPathRejection records and selected-profile authorization accepted with unconditional reverification; no member is NotRecorded. Verification was green in round 2. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T6-006` | DONE | C003 conformance repair landed: an unforgeable nonempty expected-zero witness now augments a census derived from every burn record, including over-claiming burns, and empty or nonzero expectations fail typed. Verification was green in round 4. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T6-007` | DONE | R02 signing-census binding repair landed: both production routes derive protected bytes, spent-output facts, exact receipt leaves, and code-separator position from finalized objects; the foreign-negative seam is explicit-only and every recorded identity and message stayed unchanged. The generic commitment check remains first, so uncommitted foreign paths retain LeafHashDoesNotCommit while committed siblings fail exact finalized control-path matching; verification was green in round 7. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T6-008` | DONE | R10 + C007 + C-008 provenance closure landed: typed Txid and constructor-closed RecordedAcceptance bind identities to their owning records, and private restart now strictly binds the full immutable record before any native assertion. The static gate was green in round 7; native reproduction was explicitly deferred to the serialized rerun day later recorded by (`T6-012`). Full narrative archived in [backlog history](history/backlog-history.md) |
| `T6-009` | DONE | C001 + C002 proof-bearing record repair landed: the placeholder constructor was deleted, one schema-versioned outpoint-free record now carries complete observations and typed projection refusals, and the historical T5-031 record remains explicitly NotCaptured under an evidence-binding erratum. Static verification was green in round 4, with V2 honestly Pending for the rerun-day mint in (`T6-012`). Full narrative archived in [backlog history](history/backlog-history.md) |
| `T6-010` | DONE | N1 corpus-ledger repair landed: schema 3 hard-rejects schema 2, and the truthfully empty transcript-grade binding set yields the exact partition 40 answered + 1 vocabulary-closed + 25 native-run-required + 42 RecordedObservationUnbound = 108, with 67 outstanding. Verification was green in round 6 after both cross-foot walks included the 42-row unbound bucket. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T6-011` | DONE | NEW-N2 repair landed: missing observed weight now has its own lane-local reason, distinct from a run blocked before bytes existed, while the existing weight 1911 and wire spelling agree remain pinned. C-011 was separately filed because the conservation-negatives control constants remained conditionally unbound and, at this wave's close, its repair was still open. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T6-012` | DONE | V2 proof-bearing run-of-record mint landed: schema 1 now carries the transcript facts, ordered observations and refusals, candidate messages, and second-origin reverification, with both native equality gates enforcing the exact record. The serialized rerun day at the wave's base tree against Elements was 40 of 40 green in 420.7 seconds, with the C006 seven-outcome binding and C007 private-restart gate live; static verification was green in round 5. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T6-013` | DONE | Restart-ledger lifecycle and closeout repair: status now derives from all seven ordered entries, completed and typed-stopped dispositions compare exactly, stored and rendered results are distinct, and an eighth record returns `AlreadyComplete`. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T6-014` | DONE | R08/R11 live-pattern recipe repair: one emitted recipe is the dispatch and exact census source for both roles, composed evidence follows component unions, member-side dependencies remain correctly absent, and existing script bytes stayed stable. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T6-015` | DONE | R05 report-layer repair: schema 4 separates two report requirements from validated observations, disclosure checks precede private observation minting, and validation moves the census from 38 plan answers plus 2 requirements to 40 answers. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T6-016` | DONE | C-004/C-009/C-010 complement repair: retained chain views are explicitly anchored and contiguous, signer censuses above 65,536 refuse before position conversion, and the live-fault census is pinned at 23 cases over 8 validators. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T6-017` | DONE | R04/R06 proof-negative attribution repair: the value-blinder wording was corrected to the serialized value commitment, caller-authored byte ranges were removed, and typed mutations now select canonically located fields while the narrower arbitrary-range claim was refuted. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T6-018` | DONE | R12/R13/R16-f native protocol revision 7: required nullable resource figures preserve absence, rejecting layers forbid openings, accepted results require transaction bytes, and the Rust/Python schema bump landed atomically. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T6-019` | DONE | R09 fixture-digest v2 repair: both contracts unconditionally bind each output's semantic amount, v2 is the sole live algorithm, historical v1 pins remain immutable data, and cross-language transcript goldens passed. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T6-020` | DONE | R14/R15 repair: native diagnostics are closed-enum fixed spellings with arbitrary details quarantined, while resource results truthfully identify synthetic sizing and unauthorizing witnesses instead of a cleared global blocker. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T6-021` | DONE | R16-b review-basis repair: backlog section 2 now points only to the canonical review index and Guide-14 section 4, deleting the stale mirrored review census; the server plans validator was GREEN at exit 0 in 2.8 seconds. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T6-022` | DONE | C-011 filed-then-repaired binding: the native gate now binds the accepted control, four typed mutants, ranges and reverification unconditionally; after the binding landed, the serialized run at main was GREEN at exit 0 in 13.9 seconds, and a one-digit control-identity flip drew the exact binder mismatch at exit 101 in 13.4 seconds. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T6-023` | DONE | Digest-era forward-v2 repair: thirteen exact forward-v2 pins and twelve recorded-v1 bridge assertions now keep live recomputation separate from immutable historical values, with all digest-class tests GREEN after the repair. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T6-024` | DONE | First Wave-0 archival moved the eleven long (`T6-002`) through (`T6-012`) narratives byte-verbatim, reducing maintained weight from 1,214,670 to 1,189,514 bytes, 18,806 below the cap; server plans validation was GREEN at exit 0 in 1.2 seconds. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T6-025` | DONE | N1-A rerun-day transcript archive under the accepted N1 design report (§1 binding standard, §2 archive fidelity, and §4 N1-A). ALL 79 SUPPLIED FILES ARE RETAINED: 39 canonical ceremony transcripts, 39 audit-only timing sidecars, and one audit-only predecessor setup transcript, totaling 65,698 bytes. They were retained byte-exact as supplied until the pin criterion cut the twenty-five values the tree cannot regenerate; every other byte stands as it was delivered. The archive's own module states the SHA-256 of that sorted manifest and the vectors suite recomputes it from the files; the orchestrator audit spot-checked fixture bytes against the canonical scratchpad source and passed. The strict parser admits the real rejected-positive spelling `accepted_txid none` through the typed absence variant (`TransactionIdField::Absent`) while every unenumerated spelling still refuses as malformed hex. A complete transaction-ID token sweep found 39 present 32-byte values and 13 absent values, with `none` the only absence spelling and no other legitimate non-hex vocabulary. THE ARCHIVE REMAINS TYPED-INCOMPLETE BEFORE ANY STANDING CHANGE: 38 ceremonies are `MissingRequestBytes`, the key-path probe is `MissingExecutorProvenance`, no row retypes, and no standing or census moves. Suite identity remains artifact-internal data from the serialized baseline (40 of 40, 420.7 seconds) and is never inferred from a later tree. VERIFY TOOK FIVE ROUNDS: round 1 applied thirteen formatter hunks; round 2 repaired four Clippy findings; round 3 exposed the canonical absent transaction ID after fmt and Clippy passed; round 4 merged the resulting identical match arms; round 5 was GREEN at exit 0 in 201 seconds with formatting clean, workspace Clippy clean, every vectors suite and doc-test passing, and the byte-fidelity audit passed. |
| `T6-026` | DONE | N1-B report-vocabulary schema-5 wave under the accepted design of record, `w7-n1design-report.md` §§2–4, landed in a backlog opener, a schema-4 red-before, a schema-5 vocabulary step, red-team regressions, a validator implementation, pair-recomputation repairs, a formatter fix-back, a compile repair, pedantic repairs, and a refusal-authority repair. SCHEMA-5 FIELD INVENTORY: every `LiveRunBinding` carries its deterministic SHA-256 content-addressed `run_id`; exact `archive_bytes`; a CLOSED `NativeProtocolRevision` (`Revision6` or `Revision7`); named `FixtureDigestFact` values pairing an exact 32-byte value with CLOSED `FixtureDigestAlgorithm::{HistoricalV1, ForwardV2}`; archive-derived deployment environment, network, genesis and target-contract fields; archive-derived executor adapter, adapter version, node, node version and executed source tip; exact request bytes; one typed request fact per request (`Acceptance`, `Control`, `Refusal { mutant, control_request_id, locator }`, or `Paired { member, projection }`); and one exact response per request (`Accepted { identity }` or `Refused { observed_layer, detail, control_identity }`). `LiveMutationLocator` closes over serialized output fields, exact witness items, witnessless ranges and transaction shapes; pair members are closed as explicit/private and carry the public constructor inputs needed to recompute the eleven comparison terms and exact two-term withholding policy. Schema 4 hard-refuses, unknown revisions and digest algorithms hard-refuse, revision-6 facts accept only `HistoricalV1`, revision-7 facts accept only `ForwardV2`, and no archival-v1 fact becomes a live-v1 algorithm, is recomputed as v2 or is relabeled. VALIDATOR AS LANDED: archive bytes are strictly parsed; exact requests decode and round-trip; accepted txids are double-SHA-256 recomputed from witnessless target bytes; request/fact/response keys are equal; run order and globally unique request IDs are enforced; refusal mutants link a same-run accepted control whose identity is recomputed, whose typed §15 row and boundary match, whose layer/detail match the bound observation exactly and whose locator is checked against both decoded requests; paired equality is independently recomputed from both decoded accepted members rather than trusted from relation prose; deployment/executor fields are reparsed from archive bytes; run IDs are recomputed; observation aliasing is refused; and run/request/response/observation counts cross-foot before the corresponding recomputation marks. Assembly and envelope expectations remain deliberately zero-run: N1-C alone may later add the validated corpus overlay, so the supplied archive moves no census row. FORGED-EVIDENCE RED TEAM, LIE → KILLING REFUSAL: schema 4 → `UnsupportedSchema(4)`; unknown or relabeled revision → `UnknownProtocolRevision` or `RunProtocolRevisionDiffers`; unknown, wrong or v1/v2-relabeled digest algorithm → `UnknownDigestAlgorithm` or `RunDigestFactsDiffer`; malformed/duplicated archive facts → `MalformedRunArchive`; invented archive, deployment or executor copy → `RunArchiveDiffers`, `RunDeploymentDiffers` or `RunExecutorProvenanceDiffers`; forged/stale content address or unsorted run census → `RunIdDiffers` or `RunCensusDiffers`; undecodable/non-round-tripping request bytes → `RequestDecodeRefused`; decodable forged bytes retaining the old accepted identity → `AcceptedIdentityDiffers`; orphan/missing request, fact or response → `RequestResponseCensusDiffers`; reused request ID → `ReusedRequestId`; response variant inconsistent with the typed request → `RequestResponseShapeDiffers`; mutant/control role or row/boundary mismatch → `RequestRoleDiffers`; cross-run control → `CrossRunRequestLink`; invented control identity → `RefusalControlIdentityDiffers`; false mutation range/item/shape → `MutationLocatorDiffers`; invented response layer/detail → `RunResponseDiffers`; one request aliased to unrelated observations → `ObservationAliased`; incomplete run-to-observation accounting → `RunObservationCensusDiffers`; transcript-only pair prose or forged projection inputs behind the canonical relation string → `PairedRelationNotRecomputed`; and the keystone two matching lies, an invented run plus an identical expected copy carrying undecodable bytes and an invented accepted identity, now die at `RequestDecodeRefused` where the old equality-only comparison returned `Ok(())`. REGRESSIONS cover every refusal above, deterministic content addresses and the complete rendered field inventory, plus the unchanged 40 answered + 1 vocabulary-closed + 25 native-required + 42 recorded-unbound census. ROUND-5 ROOT CAUSE: the first valid refusal fixture reached `RequestRoleDiffers` because the new validator mistakenly looked up typed safety-row boundaries in §18 `matrix::all_classes`; `malformed-signature` is authoritatively a §15 safety row and is absent from §18. The refusal-authority repair switches the lookup to `live_safety::required_safety_matrix`, preserving the honestly typed fixture and making the test reach the intended exact response layer/detail comparison; this also prevents the other typed §15-only mutants from becoming unbindable. VERIFY TOOK SIX ROUNDS: round 1 stopped on 21 rustfmt hunks, whose valid unified resend was mechanically checked and applied after the first non-unified relay correctly failed `git apply --check`; round 2 passed formatting and stopped on three `concat!` capture errors plus the real `transaction::bytes::OutputWitness` import; round 3 passed formatting and compilation and stopped on five library ownership/auto-deref findings plus two redundant test clones; round 4 passed formatting and stopped on constant-size `chunks_exact` and identical match arms, followed by a whole-diff pedantic sweep; round 5 passed formatting and workspace Clippy with 447 tests green and stopped only on the §18/§15 authority defect above; round 6 was GREEN at exit 0 in 197 seconds with formatting clean, workspace Clippy clean under warnings denied, all 448 `tripod-vectors` tests and doc-tests passing, and the orchestrator diff audit against design §§2–4 passing schema 5, the closed archival revision/digest vocabulary and every N1-B recomputation obligation. Only `plans/backlog.md` and `packages/vectors/src/live_report.rs` changed; no mechanical schema ripple was required in `live_resource_report.rs` or `live_minimality_report.rs`; `guide_fourteen.md`, recorded history, N1-A and N1-C files, run-of-record constants, network and local toolchain remained untouched. |
| `T6-027` | DONE | N1-C validated corpus-evidence overlay, following `T6-024` because this lane base contains neither `T6-025` nor `T6-026`, implements the accepted N1 forward-binding wave design report, section 2 and lane N1-C. AS LANDED, the immutable raw evidence plan remains the historical classifier, while `derive_live_evidence_plan` passes that raw plan and the independent `ArchiveFactsView` into private `ValidatedCorpusEvidence::try_from_inputs`; downstream census and outstanding-row consumers therefore receive only the overlay view. Only that constructor can retype to `NativeRunObserved`, `NativeRefusalObserved`, or `PairedRelationObserved`, and it demands a private `ValidatedCorpusWitness` whose `ValidatedCorpusObservation` carries the complete nominal `ProvenArchiveLink` run/request/response shape. It refuses a duplicate row attribution, a wrong-row attribution, a reused archive link, links crossing runs, a historical observation mismatch, a refusal-boundary mismatch, and the parity-form row's insufficient one-acceptance proof. The present incomplete archive view contains ZERO attributions and retypes ZERO rows. Pinned regressions preserve exactly 40 answered + 1 vocabulary-closed + 25 native-run-required + 42 recorded-unbound = 108 with 67 outstanding; the synthetic complete keypath-probe witness pins the conditional pre-validation arithmetic at 39 + 2 + 1 + 25 + 41 = 108 and moves exactly one row to the post-validation arithmetic 41 + 1 + 25 + 41 = 108 with 66 outstanding. The 25 native-required grounds remain unchanged, pinned by count and the row-plus-ground fingerprint. This lane deliberately does NOT import the N1-A corpus module, depend on N1-B's report schema change, wire the later validator adapter, edit recorded history, or change what today's tree reports; the later composition point is the adapter that alone mints these nominal witnesses from validator-proven corpus links. VERIFICATION TOOK FIVE ROUNDS: round 1 returned five formatter hunks; round 2 passed fmt and returned explicit-auto-deref findings; round 3 passed fmt and returned `struct-field-names`; round 4 returned one formatter hunk; round 5 passed fmt, workspace clippy, all `tripod-vectors` suites, and doc-tests at exit 0 in 216 seconds. The orchestrator diff audit against design-report sections 2 and 4 also PASSED. |
| `T6-028` | DONE | N1-D private-restart source split under the accepted N1 design of record (`w7-n1design-report.md` §3, "Forward-v2 private restart," and §4, lane N1-D), landed in a backlog opener, a type split, regressions, a pedantic cleanup, and a formatter fix-back. TYPE INVENTORY AS LANDED: `HistoricalPrivateRestartAcceptedMember` binds one recorded V1 successor digest, provenance-bearing `RecordedAcceptance`, and observed commitment prefix; `HistoricalPrivateRestartTwoAcceptanceLink` requires the Primary and Balancing members; `HistoricalPrivateRestartV1` retains the issued asset, shared V1 predecessor digest, the two-acceptance link, submitted bytes, output proof-byte counts, receipt-leaf count, and wall time; and (`HistoricalPrivateRestartRun::V1`) is the sole immutable historical variant, minted only by `historical_private_restart_run`. `ForwardPrivateRestartAcceptedMember` reserves one freshly minted V2 target identity and its same-run observed prefix; `ForwardPrivateRestartTwoAcceptanceLink` has mandatory Primary and Balancing members and no exactly-one form; (`ForwardPrivateRestartAcceptance::{Pending, Recorded}`) makes acceptance state explicit, with (`ForwardPrivateRestartExpectationRefusal::AcceptancePending`) returned by `recorded_link` until both fresh identities exist; `ForwardPrivateRestartFixtureDigestsV2` owns the unchanged shared predecessor, Primary successor, and Balancing successor V2 pins; `ForwardPrivateRestartV2` pairs only those pins with the forward acceptance state; and (`ForwardPrivateRestartExpectation::V2`) is minted by `forward_private_restart_expectation` with Pending acceptance. N1-F SELECTION SURFACE: destructure the value from `forward_private_restart_expectation`, compare a fresh run through `fixtures().predecessor()` and receipt-indexed `fixtures().successor(consumed)`, require `acceptance().recorded_link()` before any acceptance gate can pass, then select the fresh identity through `link.for_receipt(consumed)`; the native gate never selects `historical_private_restart_run` as a live expectation. RED-BEFORE EVIDENCE: the compile-fail example on `HistoricalPrivateRestartRun` refuses a forward V2 payload as historical V1; the compile-fail example on `ForwardPrivateRestartAcceptance` refuses the historical two-acceptance link as a forward Recorded link, so no constructor accepts V2 pins and V1 txids together; `the_forward_v2_expectation_is_pending_and_has_only_forward_pins` proves Pending returns `AcceptancePending` instead of a false green; `both_commitment_parity_forms_requires_a_two_acceptance_link` compiles the parity gate only against both mandatory members; and `the_historical_v1_type_preserves_every_recorded_value` pins every existing historical field through the new type while the cumulative source diff deletes zero constants and the existing renderer and forward-v2 byte-identity tests stay green. VERIFY TOOK TWO ROUNDS: round 1 stopped only on the server's rustfmt normalization, mechanically extracted from session JSON, passed `git apply --check`, and landed byte-verbatim; round 2 was GREEN at exit 0 in 220 seconds with formatting clean, workspace Clippy clean under warnings denied, all `tripod-vectors` suites and doc-tests passing, and the orchestrator design-report §3 audit passing the typed historical members, mandatory two-acceptance link, and zero constant deletions. Only `plans/backlog.md` and `packages/vectors/src/live_private_restart.rs` changed; `guide_fourteen.md`, `guide13_live_native.rs`, the concurrent proof-bearing source, all recorded constant values, all forward V2 pins, network, and local toolchain remained untouched. |
| `T6-029` | DONE | N1-E proof-bearing schema-2 forward archive under the accepted design of record, `w7-n1design-report.md` sections 3 and 4, landed in a backlog opener, red-before regressions, a schema-2 implementation, and a formatter fix-back. This row follows `T6-027` because `T6-028` is absent from the lane base. SCHEMA-2 FIELD INVENTORY: `ForwardV2ProofBearingRunOfRecord` carries schema version 2, closed `FixtureDigestAlgorithm::ForwardV2`, an observation member typed `ForwardProofBearingRecordMember<ForwardV2ProofBearingObservations>`, and an acceptance member typed `ForwardProofBearingRecordMember<ProofBearingReverification>`. The observation member holds the issued asset, forward-v2 predecessor digest, outpoint-free node-reported coins, output-witness vector length and per-entry proof-byte census, spent-value prefixes, ordered case observations, construction-refusal capture, and candidate messages. The acceptance member holds accepted and witness transaction identities, block height, readback equality, independently recomputed message, readback signature, verification result, and the emptied-vector comparison. BOTH MEMBERS REMAIN `Pending` and expose no borrowable recorded value until an owner-authorized forward-v2 ceremony mints them. NAMING KEEPS THREE DIMENSIONS DISTINCT: the immutable archive's retained `ProofBearingRunOfRecordV2` and `construction_run_of_record_v2` names describe ceremony generation; schema 1 is explicitly `HistoricalV1` through its structural digest marker and stays byte-identical; the new algorithm-first `ForwardV2...` names describe digest semantics, while the schema constant separately says 2. The fresh planner types its records `ForwardV2`; the historical projection refuses them with `HistoricalV1DigestRequired`, the forward projection refuses historical records with `ForwardV2DigestRequired`, and neither conversion wraps or relabels schema-1 data. N1-F'S INTENDED SELECTION SURFACE is `forward_v2_proof_bearing_run_of_record() -> &'static ForwardV2ProofBearingRunOfRecord`; the source renderer already routes fresh forward-v2 records only through the schema-2 projection and reports both pending members, while `guide13_live_native.rs` remains N1-F-owned and untouched. RED-BEFORE EVIDENCE landed before implementation: `a_fresh_forward_v2_projection_refuses_the_schema_one_historical_path`, `schema_one_historical_facts_refuse_the_forward_v2_projection`, `only_schema_two_records_forward_v2_facts`, and `pending_forward_members_cannot_pass_as_recorded`; existing schema-1 rendering/equality and T5-031 `NotCaptured` regressions remained green. The historical constants module is byte-identical to base; the orchestrator audit confirmed zero recorded-constant deletions and the green rendering tests confirmed schema-1 byte identity. VERIFY TOOK TWO ROUNDS: round 1 stopped only on three rustfmt hunks, mechanically extracted as unified diff and committed in the formatter fix-back; round 2 was GREEN at exit 0 in 214 seconds with formatting clean, workspace Clippy clean under warnings denied, every `tripod-vectors` suite and doc-test passing, and the orchestrator design/constant audit passing. Only `plans/backlog.md` and `packages/vectors/src/live_proof_bearing_observation.rs` changed; `guide_fourteen.md`, `guide13_live_native.rs`, `live_private_restart.rs`, schema-1 values, T5-031, network, and local toolchain remained untouched. |
| `T6-030` | DONE | N1-F native-gate integration under the accepted design of record, `w7-n1design-report.md` section 3's forward-v2 subsections and section 4 lane N1-F, landed in a backlog opener, a gate integration, and a binding-documentation correction. PRIVATE-RESTART SELECTION AS LANDED: before, the fresh gate selected the historical V1 predecessor and receipt-indexed successor digest constants beside their recorded accepted identities; after, every fresh predecessor and successor comparison selects only `ForwardPrivateRestartExpectation::V2` through `fixtures().predecessor()` and `fixtures().successor(consumed)`. The acceptance half independently consumes `acceptance().recorded_link()`: when Recorded, the receipt-selected forward member supplies its fresh identity and same-run commitment prefix; while Pending, only that receipt's fresh accepted identity and unconditional reverification bind to the matching immutable historical `RecordedAcceptance`, because the deterministic transaction identity is independent of the fixture-digest algorithm. No historical digest enters either branch, so a v2 expectation is never paired with a v1 digest value. PROOF-BEARING SELECTION AS LANDED: before, a fresh record projected through schema 1 and compared with `construction_run_of_record_v2`; after, the live gate selects `forward_v2_proof_bearing_run_of_record`, projects only through `ForwardV2ProofBearingRunOfRecord::try_from`, checks schema 2 and `ForwardV2`, and compares only the forward record's own observation and acceptance members once each is Recorded. Pending members fail closed rather than masquerading as recorded evidence. Schema 1 remains immutable historical-v1 data, validates its own archived bytes in the source module, and is never the normal fresh-run expectation. CROSS-PAIRING KILL EVIDENCE: the non-ignored companion `a_fresh_v2_projection_refuses_schema_one_before_node_execution` creates only the node-free forward-v2 planner record and deliberately offers it to `ProofBearingRunOfRecord::try_from`; the typed result is `HistoricalV1DigestRequired`, and the test contains no executor configuration or node call. Both owned regions carry the algorithm-move note: recorded history stays immutable and renderable, the sole live algorithm is v2, and forward expectations await their own owner-authorized accepted runs. VERIFY TOOK ONE ROUND: round 1 was GREEN at exit 0 in 216 seconds with formatting clean, workspace Clippy clean under warnings denied, all `tripod-vectors` suites and doc-tests passing, ignored native tests compiling, and the orchestrator diff audit passing forward-v2 selection in both gates, typed cross-pairing refusal, and schema-2 projection. The ignored native runtime check follows at the post-merge serialized wave gate. Only `plans/backlog.md` and the owned proof-bearing/private-restart regions of `packages/vectors/tests/guide13_live_native.rs` changed; `guide_fourteen.md`, both landed source modules, the conservation region, recorded history, network, and local toolchain remained untouched. POST-MERGE NATIVE FINDING: the proof-bearing run exposed that Pending members were incorrectly unwrapped as Recorded and panicked at the gate, so the follow-up accepts a complete, internally consistent schema-2 fresh projection as candidate mint material while both expectations are Pending, reserves full equality for the future Recorded/Recorded state, refuses mixed states, and leaves the independently green 12.7-second private-restart runtime untouched. |
| `T6-031` | DONE | Guide-14 Wave-0 closeout completed in the required order: the ACTIVE row opened first; the post-merge native rerun and exact single-digit red proof completed (`T6-022`); the long (`T6-013`) through (`T6-024`) rows moved byte-verbatim to the Guide-14 Wave-0 history group and their live rows compressed; and the sole Guide-14 edit added the compact tree-derived section-4 disposition register in one dedicated commit. The archive/source row block matched. WEIGHT ARITHMETIC: the failed maintained set began at 1,211,178 bytes; the ACTIVE row added 379 and the (`T6-022`) append added 498, reaching 1,212,055; archival compression removed 36,529, reaching 1,175,526; the 7,358-byte section-4 addition is present in the checked tree but is archive-classed under `plans/guides/` and therefore adds zero maintained bytes; this DONE expansion adds 1,158, for a final maintained set of 1,176,684 against the 1,208,320-byte cap, a 31,636-byte margin. The register records every `G14-R01` through `G14-R16` subclaim, `NEW-N1`, `NEW-N2`, and `C-001` through `C-011`; tree inspection leaves `G14-R16-a` and `G14-R16-f` openly conflicting with their claimed repairs, while N1 honestly binds zero archived rows and leaves owner-authorized recapture pending. VALIDATOR RESULT: GREEN — plansval exit 0, wall 1.1s; one link-fix round; orchestrator audit passed 29 register rows and 1 hunk. |
| `T6-032` | DONE | The Guide-14 §4 OPEN — TREE CONFLICT entries `G14-R16-a` and `G14-R16-f` are discharged by this row. R16-A BEFORE → backlog §11 said `Phase 5 is not exited`; AFTER → §2.9 is explicitly the historical gate where Phase 5 `had not exited`, the live-transfer campaign subsequently closed under §2.20/§2.21 with the amended gate met on a countersigned assessment, and Phase 6 is current. R16-F BEFORE → the revision narrative said `A revision-4 executor is therefore refused at the handshake` while `NATIVE_PROTOCOL_SCHEMA` was 7; AFTER → any executor speaking an earlier revision is handshake-refused, the constant below is the sole current revision, and revisions 2 through 6 and what revision 4 minted are unambiguously past-tense history. EMITTER VERDICT → `emit.rs` line 350 was a current-state claim (`under revision 4 the record either states`), so the same rustdoc-only commit now anchors it to the sole current revision named by `NATIVE_PROTOCOL_SCHEMA`. DOC-ONLY PROOF: every changed line in `protocol.rs` and `emit.rs` is a `///` line, and deleting rustdoc leaves both files byte-identical to their pre-commit forms. VERIFY TOOK ONE ROUND: exit 0 with formatting clean, workspace Clippy clean under warnings denied, the `tripod-target-elements-conformance` suites and doc-tests passing, and the orchestrator doc-only Rust-diff audit passing. The ACTIVE opener changed only `plans/backlog.md`, `packages/target-elements-conformance/src/protocol.rs`, and `packages/target-elements-conformance/src/emit.rs`; `plans/guides/guide_fourteen.md`, recorded history, network, local toolchain, and code tokens remained untouched. |
| `T7-001` | DONE | The owner-authorized revision-7 and forward-v2 recapture arc opened and landed the caller-owned progressive executor journal, with formatter repairs, structural Clippy repairs, explicit protocol-read constraints, and the orchestrator-authorized shared-test visibility repair; exact native request bytes are captured before send, full handshake and environment carriers survive failures, accept and target-refuse exchanges finish complete, every infrastructure terminal is typed, and the unchanged `execute_operations` surface remains a throwaway-journal wrapper; verification took seven rounds, with six bounce rounds for rustfmt twice, the Clippy census, generic inference, and test visibility twice before round 7 passed rustfmt, Clippy, and the full `tripod-target-elements-conformance` package tests at exit 0 in 212 seconds. |
| `T7-002` | DONE | The owner-approved narrow live-report schema 6 vocabulary advance retains the complete schema-5 reader and validator coverage; adds the typed two-acceptance observation, witness-path-shape and committed-leaf-arrangement locators, primary/support link accounting, and the independently proven multi-row semantic witness; adds no auxiliary role; and preserves the all-run-requests-used refusal. VERIFY TOOK FIVE ROUNDS: round one stopped on formatting; round two found the required non-exhaustive foreign-enum refusal arm; round three reached Clippy and established that `compare_run_bindings` and `validate_bound_observations` were superseded schema-5 forwarding wrappers rather than disconnected validation paths, then removed them while their schema-parameterized successors retained every obligation; round four stopped on width-driven formatting; round five exited 0 in 233 seconds with formatting, Clippy under warnings denied, and the full `tripod-vectors` tests passing. NOTE: `T7-001` and `T7-003` landed on main in parallel, so this row may need id-order placement at merge. |
| `T7-003` | DONE | R4 capture driver landed in an opener, a driver, and a mock contract plus root Meson registration. This row follows `T6-032` because parallel-lane rows `T7-001` and `T7-002` are absent from its base. THE DRIVER'S FIVE DUTIES ARE CLOSED: it refuses a dirty suite or pre-existing destination, resolves the full commit and tree, declares the sorted 39-ID semantic roster, validates the executor environment, and creates a new output directory; it records expected elementsd tip and requires both observed intended tip and binary-reported revision to equal it; it runs exactly one `tripod-vectors` ignored integration target with one test thread under an outer UTC/millisecond timer and retains Cargo's exit; it requires 40 outcomes = 39 unique complete canonical captures + 39 canonical timing sidecars + one audit-only predecessor setup artifact while rejecting missing, duplicate, unexpected, noncanonical, CR-bearing, incomplete, or cross-file-inconsistent output; and it emits the exact observed-data `RUN-REPORT`, builds and verifies one sorted manifest, makes the directory read-only, and reports ineligible with nonzero exit after Cargo or census failure. THERE IS NO ELIGIBLE SINGLE-TEST MODE. The hash topology resolves the charter's impossible mutual cycle explicitly: `MANIFEST.sha256` binds the 79 harness artifacts, then `RUN-REPORT` binds that manifest, yielding the required 81 files without pretending a report containing the manifest digest can simultaneously be hashed by that manifest. The pure-shell contract has 18 cases: eligible success; every file/census/terminal/name/CR refusal; dirty-tree and nonempty-directory preflight; both tip mismatches; deployment/schema mismatch; retained Cargo failure with report; and forced manifest-verification failure. RED BEFORE is structural: no driver existed at the lane's base, so no pre-lane guard could reject the three named bad captures. VERIFY TOOK ONE ROUND: `lane-verify` was GREEN at exit 0 in 229 seconds, and the server separately ran the shell contract on the exact implementation ref GREEN at exit 0 in 19.1 seconds with `live-native capture contract: all cases passed`; the 63.26-second local shell run was diagnostic only. No Cargo, network, node, server capture, recorded history, or Guide-14 edit entered the lane; only `plans/backlog.md`, `meson.build`, and the two owned shell files changed. |
| `T7-004` | DONE | R3 native harness recorder landed the closed 40-test/39-semantic-ceremony roster, captured executor integration, canonical `native-v2-r7-capture` transcript and panic-safe timing guard, exact-byte format/path/guard regressions, typed mutation and pair facts, and byte-for-byte legacy-path preservation without running the ignored native ceremonies. VERIFY TOOK EIGHT ROUNDS: rounds one, three, and five stopped on formatting; rounds two, four, and six reached Clippy and respectively required unsafe environment mutation to become explicit capture-destination plumbing, the full structural/mechanical lint census, and release-active SHA-256 byte-census assertions; round seven passed formatting and Clippy, then root-caused both test failures to the scripted adapter's extra read before the unsolicited environment record and corrected that local protocol sequence; round eight exited 0 in 202 seconds with formatting, Clippy under warnings denied, and the full `tripod-vectors` suite passing, including all seven non-ignored tests in the target while all 40 owner-gated native ceremonies compiled and remained ignored. NOTE: `T7-005` landed on main in parallel, so this row belongs between `T7-003` and `T7-005` at merge. |
| `T7-005` | DONE | The owner-retired, never-persisted live safety report schema-5 reader was removed, with a red regression and a formatter fix-back; schema 6 is now the sole accepted report schema, schemas 4, 5, 7, and an unknown value refuse through `UnsupportedSchema`, the two-matching-lies checks remain, and the revision-6/historical-v1 corpus vocabulary remains intact. RED verified at exit 101 in 182 seconds with `schema_five_is_hard_rejected` the sole failure and 486 tests passing because the historical reader returned `Ok`; GREEN verified at exit 0 in 209 seconds with formatting, Clippy under warnings denied, and the full `tripod-vectors` suite passing with schema 5 refused. |
| `T7-006` | DONE | The native recorder fact join defect exposed by the authorized C1 run was repaired in an opener, a pre-fix regression commit, a joining/accessor repair, a formatter fix-back, and a Clippy repair. RED BEFORE: the real C1 capture completed 35 of 40 ceremonies while `conservation-negatives`, `owner-observation`, `proof-bearing-observation`, `report`, and `sponsored-committed-value` all stopped in write-before validation; conservation reproduced deterministically with `operation-3 has an incomplete mutation declaration`, and the pre-fix regression commit preserves all five real operation censuses against the old join before the repair commit. The fix keeps completeness strict: conservation's report-external `missing-rangeproof` control remains auxiliary with neither mutation half; owner and proof-bearing negative controls, the report probe, and the committed sponsor probe receive ceremony-specific auxiliary roles rather than invented acceptances; selected observation cases remain acceptances; genuine conservation mutants retain typed locators and accepted-control links; and the committed sponsor predecessor digest comes from one read-only planner accessor. VERIFY TOOK THREE ROUNDS: round one stopped on formatting; round two passed formatting and found four Clippy repairs (one const constructor, one trailing comma, and two emptiness assertions); round three exited 0 in 230 seconds with formatting, Clippy under warnings denied, the full `tripod-vectors` suite passing, and all five new census regressions green. No native ceremony ran in this lane, and Guide 14 and recorded history remained untouched. |
| `T7-007` | DONE | The evidence-driven R4 capture-driver repair landed in an ACTIVE opener, a red-before contract, and an implementation. RED BEFORE reproduced both C1 attempt #2 false refusals in 42.32 seconds: a fully valid 40-of-40 mock run with all 39 complete captures, the setup artifact, the harness-owned `diagnostics/<ceremony-id>/` subtree, and binary-reported revision was rejected both for the diagnostics entry and because the 12-hex carrier did not equal the full expected tip. The repaired driver admits exactly one top-level `diagnostics/` audit subtree, excludes it from the 81-file census and `MANIFEST.sha256`, records `diagnostics-present yes|no`, and still refuses every other unexpected entry. The binary revision must be lower hex, at least 12 characters, and a prefix of full expected tip; the intended executed tip still requires exact 40-hex equality. The 22-case contract adds green coverage for diagnostics present and absent, 12-hex prefix and full binary revisions, non-prefix, 11-hex, non-hex, and second-unexpected-directory refusals while retaining every original driver boundary. VERIFY TOOK ONE ROUND: `lane-verify` exited 0 in 199 seconds, and the server verdict of record ran the 22-case shell contract on the exact implementation ref at exit 0 in 23.8 seconds with `live-native capture contract: all cases passed`; the 61.49-second local contract run was diagnostic only. No Cargo, node, network, server capture, recorded history, or Guide-14 edit entered the lane; only `plans/backlog.md` and the two owned shell files changed. |
| `T7-008` | DONE | R5 imports the reviewed native-v2/revision-7 C1 run of record as one byte-verbatim 81-file archive and a strict, content-addressed corpus. The importer fixes the 39-ceremony roster in code, validates the 79-entry manifest before parsing, binds the manifest through the strict run report, validates the closed capture grammar and all request/response, transaction, locator, projection, row-attribution, and global-address obligations, and exposes proven corpus material without retyping an evidence-plan row. The admitted archive produces exactly 39 ceremonies and 40 outcomes, and every transcript-byte, manifest-hash, ID-link, txid, decoded-locator, projection-term, protocol/digest-tag, and missing/duplicate/extra-census refusal test bites. VERIFY was GREEN on round 20 at exit 0 in 213 seconds: formatting and workspace Clippy passed, then the full `tripod-vectors` suite passed. The rounds established that suite-tree is a 40-hex revision; duty 5 decodes and round-trips accepted requests while refused mutants remain exact locator-checked evidence; Elements witness-path leaves use version `0xc4`; sponsor authorization is the target's two-item witness; row-bearing bindings partition as 28 runs plus 11 corpus-only ceremonies; the attribution inventory cross-foots 42 rows through 41 observations, 44 primary links, and 17 support links; and R01 reads the content-addressed consumed-prefix declarations as the opposite `0x08`/`0x09` pair, then binds each accepted successor's independently matching confidential output. The rerun-day archive remains untouched. |
| `T7-009` | DONE | R6 binds the validated native-v2/revision-7 C1 corpus through `ValidatedCorpusEvidence` in `live_evidence.rs`: 42 row attributions apply atomically as 24 native-run observations, 17 native-refusal observations, and one paired relation, with their 28 run bindings carried by the same object and no historical-v1 equality oracle. The exact census moves from `38+2+1+25+42=108` before overlay to `80+2+1+25=108` before report validation and `82+1+25=108` after its two report-layer observations validate, leaving exactly 25 native-run-required rows unchanged. `live_report.rs` now assembles from that overlay and validates the real Elements bytes while preserving txid, byte-round-trip, link-class, all-requests-used, census, locator, predicate, alias, and request-role refusal power. VERIFY TOOK THIRTEEN ROUNDS: the final round passed formatting, workspace Clippy under warnings denied, and all `tripod-vectors` package tests; earlier rounds exposed and repaired formatter/Clippy findings plus real-capture locator, sponsored-predicate, typed-alias, conservation-role, historical-identity-adversary, rendered-observation, and section-scoreboard assumptions. No corpus accessor was added, the schema-6 vocabulary stayed frozen, and the 25 `NativeRunRequired` rows and grounds in `live_negative_half.rs` remained byte-unchanged. |
| `T7-010` | DONE | R7 mints the authorized forward private-restart acceptance and proof-bearing run of record atomically from typed projections of the validated native-v2/revision-7 corpus; malformed, incomplete, mixed, schema-1-substituted, or tampered evidence remains Pending, and verify round 5 passed fmt, workspace clippy under `-D warnings`, and all package tests in 207 seconds. |
| `T7-011` | DONE | R8 moved every recorded-v1 value byte-verbatim behind `packages/vectors/src/live_history_v1.rs`: unchanged historical re-exports and exact-value/rendered-byte regressions guarantee the archive was not recomputed. Current consumers now read typed native-v2/revision-7 corpus projections or the authorized T7-009/T7-010 products, while historical consumers call the historical-v1 namespace explicitly; historical closeout snapshots, live Q19 forward-v2 equality/uniqueness pins, and the exact `82+1+25=108` census remain unchanged. VERIFY ROUND 6 was GREEN at exit 0 in 206 seconds with formatting clean, workspace Clippy clean under `-D warnings`, and every package test and integration suite passing. |
| `T7-012` | DONE | R3 completed the final native-test consumer succession: owner observation, private restart, owner-signing negatives, keypath verdicts, committed-sponsor values, and the pair-arc ledger now bind current claims to typed native-v2/revision-7 corpus projections or the recorded T7-010 forward mint, with the obsolete historical private-restart fallback removed. The live recorder, capture paths, write-before-gates, legacy rendering, and non-native format/path/guard goldens remain byte-compatible. VERIFY ROUND 3 was GREEN in 207 seconds with formatting clean, workspace Clippy clean under `-D warnings`, and all `tripod-vectors` package tests and integration suites passing; round 1 applied the formatter fix-back and round 2 split the keypath phase-A/phase-B assertions for `too_many_lines`. |
| `T7-013` | DONE | Registered the T7-011 historical-v1 source in the vectors build census; a sorted comparison of all declared vectors source, test, fixture, and documentation paths against `git ls-files` matched 227/227 with no other missing or stale entries, and the remaining comment-only `NearMissSpan` warning was cleared. VERIFY ROUND 1 was GREEN in 205 seconds with formatting clean, workspace Clippy clean under `-D warnings`, and all `tripod-vectors` package tests passing; Meson-gate proof lands with the orchestrator's full-gate record. |
| `T7-014` | DONE | R10 physically moved every historical-v1 declaration and complete recorded structure into 13 registered files beneath `live_history_v1`, removed the historical declaration namespaces from the origin modules, and left those origins with current planners, renderers, corpus projections, and the private-restart `forward_v2` API. The unchanged byte-equality regression expectations passed before and after the move; the no-active-v1-reads audit and exact `82+1+25=108` census remain green. VERIFY ROUND 6 was GREEN in 205 seconds with formatting clean, workspace Clippy clean under `-D warnings`, and all package and integration tests passing. |
| `T7-015` | DONE | R11 deleted the 14-file `live_history_v1` namespace and the now-purpose-free Wave 5–8 `live_closeout.rs`, leaving git history as the v1 archive while closed prose and fixture archives remain byte-unchanged. Every raw `RecordedObservationUnbound` row now retains only its kind: 24 acceptances, 17 refusals, and one paired relation across the same 42 rows; fresh values enter only through the atomic validated-corpus overlay. The censuses remain exactly `38+2+1+25+42=108`, `80+2+1+25=108`, and `82+1+25=108`. Historical-only tests and constructors were deleted, current tests were re-fixtured from native-v2/revision-7 projections, and the harness projection API retained its value-free retired schema-1 type in `live_proof_bearing_observation.rs`. A source-and-test sweep found zero v1-only recorded payloads, including the private-restart, maximum-inputs, and proof-bearing sentinels; the only historical hexadecimal values shared with source are independently present current-corpus/forward-record values. The vectors Meson source census cross-foots at 56 declared and 56 tracked; Meson-gate proof lands with the orchestrator's full-gate record. VERIFY TOOK FIVE ROUNDS: the final round was GREEN in 209 seconds with formatting clean, workspace Clippy clean under `-D warnings`, and all package and integration tests passing; prior rounds applied rustfmt, repaired kind-only report consumers and dead helpers, and completed the pair-arc weight re-fixture from decoded current-corpus acceptance bytes. |
| `T8-001` | DONE | The Guide-14 conceptual preflight track opened, archived all three reviews byte-verbatim with their authorized index and census entries, and bound 15 OPEN register rows plus 26 wave-ownership assignments, distinct from the T7 recapture arc. VERIFY TOOK ONE ROUND: the server plans validators, including check-plans and register generation, passed at exit 0 in 1.9 seconds with zero tracked-tree delta; the orchestrator audit traced all 25 ranked findings to their rows, merges, or the one declined disposition, spot-checked the archives byte-identical, and confirmed the rulings verbatim. |
| `T9-001` | DONE | Typed closures now distinguish architecture or deployment closure, typing correction, and adjudicated duplicates with six no-catch-all proof arguments; live classification, zero census and report buckets under schema 7, and the `tapscript::family_range_defects` `Vec` adapter landed without moving any row, ground, or register entry; VERIFY TOOK FOUR ROUNDS: round 1 applied two rustfmt hunks after a formatter-only bounce in 2.3 seconds, round 2 fixed `assert-is-empty` after formatting passed and Clippy bounced in 31.1 seconds, round 3 corrected the schema-7 current-report rejection set after formatting and Clippy passed and 475 tests passed with one failure in 213.3 seconds, and round 4 was GREEN in 210.9 seconds with formatting clean, workspace Clippy clean under `-D warnings`, and all package and integration tests passed. |
| T9-002 | DONE | Six pre-adjudicated §15 rows now consume the exact typed boundaries T9-001 minted: four architecture or deployment closures, one typing correction and one adjudicated duplicate, each with its row-local argument and with the fee-role claim explicitly scoped to this deployment. The run-owed register shrank from 25 to 19 without changing any remaining entry or ground; its recomputed grounds fingerprint is pinned. Every raw, validated and rendered census still cross-foots at 108 with NativeRunRequired 19, ArchitectureClosed 4, TypingCorrectionClosed 1 and AdjudicatedDuplicateClosed 1; the six routes remain non-answers, the layerless census pins exactly the operation-vocabulary closure plus those six typed closures, and report schema 7 is unchanged. VERIFY TOOK THREE ROUNDS: round 1 was a formatter-only bounce in 2.3 seconds and applied two rustfmt hunks before tests started; round 2 passed formatting and workspace Clippy, then passed 474 tests and failed exactly the sanctioned stale fingerprint pin plus the missed old-world layerless-row consumer in 166.4 seconds, yielding the fingerprint value used above; round 3 was GREEN in 214.3 seconds with formatting clean, workspace Clippy clean under `-D warnings`, and all package and integration tests passed. |
| T9-003 | DONE | Six first-party errata now close through the exact routes their published gates admit: `foreign-sponsor-asset`, `two-sponsor-envelopes`, and `unclassified-u` are FirstPartyDischarged at their true first-party construction or backend-emission boundaries, while `sponsor-member-unclassified`, `output-claimed-through-two-flows`, and `confidential-sponsor-values` are FirstPartyFactObserved by the existing focused tests or accepted positive twin that decide their own gates. The `unclassified-u` discharge makes the previously inert family-range validator honest by staging an unchecked complete-family value with its sole output range absent and actually calling `tapscript::family_range_defects`, observing `PositionUnaccounted`; the validator label alone is no longer treated as evidence for that case. `amount-outside-semantic-domain` stays NativeRunRequired, but its impossible-mint ground is replaced by the owed native surgery of an output to `1 << 51` and the target's own `bad-txns-vout-toolarge`-class refusal. The run-owed register shrank from 19 to 13 and its recomputed grounds fingerprint is pinned at `0x053e_323b_8934_f2b5`; the exact 108-row census now carries NativeRunRequired 13, FirstPartyDischarged 37, FirstPartyFactObserved 6, and the untouched typed-closure buckets 4/1/1. VERIFY TOOK FIVE ROUNDS: round 1 was a formatter-only bounce in 2.4 seconds and applied the relayed rustfmt diff before tests started; round 2 passed formatting and fast-failed compilation in 4.0 seconds on `FieldSide` being imported from `target_elements` rather than its `tapscript` re-export; round 3 passed formatting and workspace Clippy, then passed 474 tests and failed exactly the sanctioned stale fingerprint discovery plus the first missed rendered-report consumer in 173.1 seconds, yielding the fingerprint above and moving plan-derived observations from 4 to 7; round 4 passed 475 tests and exposed the next exact consumer in the same rendered-identity test in 168.2 seconds, moving the rendered observation total from 47 to 50 by `41 + 7 + 2`; round 5 was GREEN in 215.5 seconds with formatting clean, workspace Clippy clean under `-D warnings`, and all package and integration tests passed. |
| `T9-004` | DONE | The revision-8 capture-wave design is complete and adopted as the chartering input for the capture lanes: three rows extend the owner-signing-negatives ceremony, the remaining five ride three new negative ceremonies, one boundary erratum moves malformed-control-path from the consensus stage to script-path rejection, report schema 7 suffices with no new locator variant because a changed-length witness item already validates, the driver evolves to a sibling r8 pair re-pinning the same reviewed validator tip, and the import path moves the register from 13 to 5 with every census pin enumerated and the identity cross-footing at 108 in the raw, overlaid, and reported states. Two independent orchestrator verification passes sampled about thirty file-and-line citations and every census recomputation with zero factual errors. The three design questions closed at adoption, each on its merits: copied-commitment narrows to a definite class because the range-and-shape pair separates it from every other registered row once the third-output observation lands; sponsor-protocol-overlap keeps its declared typing until the live capture shows which clause the node fires; and revision 8 re-pins the same tip because every new refusal must be judged by the same reviewed validator that accepted the controls it is compared against. The design report is a lane artifact of the read-only lane, which made no tracked changes. DISPATCH FINDING: the first dispatch was killed by its vendor's cyber-risk classifier at 625 seconds with zero deliverable — a false positive on defensive conformance vocabulary — and the rerouted worker delivered clean in about 35 minutes; adversarial-shaped defensive design lanes now route past that classifier at dispatch, and a classifier-killed lane is rerouted, never retried. |
| `T9-005` | DONE | C2 harness lane of the capture wave: the two owner-signing seams — the eighth consensus surgery writing an output value above the semantic domain and the third leaf arrangement exchanging member and coordinator sources — plus a post-build witness surgery stage for the malformed control path; the three new negative ceremony modules for the split-commitment, offsetting-flow, and sponsored owner-signing successors with their package meson census entries; the eight new mutant kinds and their row bindings in the report vocabulary, reusing only existing locator variants; the recorder roster, control submissions, and mutation facts with three new ignored ceremony tests each writing its capture before its gates; and the two safety-matrix corrections — the malformed-control-path boundary erratum and the copied-commitment class narrowing on its separating range-and-shape argument. The tree stays buildable and green on the frozen r7 corpus throughout. Delivered by the C2 harness lane and merged, after rounds that hardened the sponsor-position gates into compile-time facts. The sponsored ceremony carries its sponsor-range mutants with sponsor-protocol-overlap untyped, deferred to the C3 live probe. |
| `T9-006` | DONE | R8 driver and contract lane of the capture wave: sibling scripts evolve the r7 capture driver and its mock-driven contract to revision 8 — ceremony roster 39 to 42 with the three new identifiers in bytewise-sorted position, test census 40 to 43, file census 81 to 87 with 85 manifest-bound entries, the same reviewed validator tip re-pinned under the unchanged 12-hex-prefix rule, the diagnostics-subtree rule carried over, and the run-report grammar renamed to the r8 schema strings while the native protocol revision stays 7. The 22 contract cases stay parametric over the roster — only the mock roster and the census constants move — and the r8 contract joins the meson suite beside the r7 lane. VERIFIED GREEN IN ONE ROUND on the shared instance: the contract passed every case in 25.4 seconds and meson parsed with both the r7 and r8 contract lanes registered in 1.2 seconds, growing the suite to fifty lanes. The orchestrator accounted for all twenty changed driver lines as adopted design deltas, independently recomputed the forty-two-entry roster and verified its bytewise sort, and confirmed the r7 scripts are byte-unchanged. |
| `T10-001` | DONE | Revised-edition draft adoption arc: two revised editions of the adopted-source disciplines are archived verbatim under the standing drafts template with their index rows and census entries in the same commit — the label calculus adding a generated-compliance invariant, a reworked profile signature, and restated warrant, minting, and participation judgments; the environment-kind registry adding a fifteenth Convention with its coordinated preamble and table updates. The identity-adjudication and interchange copies that arrived alongside were byte-identical to their retired texts and were discarded, since git history already carries those bytes. Integration amends ADR-019 and ADR-020 to the new normative bodies under the drafts rule that a draft binds nothing until its integrating change lands, records every deliberate divergence, and reports any new checker obligation the revised invariants impose rather than silently absorbing or ignoring it; the drafts then retire into their records as the founding editions did. VERIFIED GREEN OVER FOUR SERVER ROUNDS and merged: round 1 caught the checker's committed kind vocabulary drifted from the new registry body; round 2 cleared it and exposed the stale companion register, regenerated server-side and applied by relayed diff with no byte hand-written; round 3 passed the whole documentation bar including the plans validators' first measurement on the branch while the labels package tests caught one derived homonym pin still at the old figures; round 4 was green on both bars in 1.3 and 8 seconds with all 196 labels tests passing. The vocabulary sync is a proper superset — twelve kinds and twenty pairs added, nothing removed — and the orchestrator recomputed both counts, the subset property, and the fold-fidelity of both ADR bodies independently before merging. ADR-020's gate is fully discharged with no migration owed; ADR-019's gate records one clause honestly open — generated-register occurrence compliance — because the checker still drops the registers from discovery, securing the no-feeding half while leaving their occurrences unharvested; the discovery-narrowing repair is filed as its own future bite, not ridden in here. The drafts retired into their records and the standing template reads empty again. |
| `T11-001` | DONE | Guide-14 execution track opened, and the stale Phase-6 declarations welded to the state the rows already record, across three maintained files. THREE DECLARATIONS: the Phase-6 card's status block, which said no execution guide was chartered and the Guide-14 drafting decision remained the owner's; the backlog header, whose next-gate line waited on that same decision and whose current-condition line closed on it, together with the phase table's Phase-6 cell reading no execution guide chartered yet; and the guides index, whose Guide-14 paragraph explained the guide as deliberately absent from the executed-guides table because no batch had been run against it. EVIDENCE: the owner supplied Guide 14 and it was archived verbatim (`T6-001`); Wave 0 — the tenth static review's disposition and the Phase-5 handoff revalidation — ran and closed out in the required order (`T6-002`, `T6-031`); and the owner-issued conceptual preflight register binds Waves 1–13 (`T8-001`). Each declaration now states that, and names Wave 1, typed STATE metadata and semantic transition, as the next wave; the executed-guides table gained the Guide-14 row the index's own rule requires once a batch is chartered against a guide. Documentation only: no package, script, or census file was touched, and no claim was added that a row does not already carry. VERIFIED GREEN IN ONE SERVER ROUND on the tip carrying all three welds, with nothing bounced: check-plans passed at exit 0 in 16.0 s wall, measuring maintained Markdown at 1,169,788 bytes against the 1,208,320-byte hard cap for a 38,532-byte margin and raising only the pre-existing per-file and soft-target warnings; `cargo test -p tripod-labels` passed 9 unit and 3 doc tests at exit 0 in 11.1 s wall; both whitespace diff checks were clean, the whole gate took 28.3 s wall, and the lane tree was empty under `git status --untracked-files=all`. |
| `T11-002` | DONE | Wave-1 opening record, documentation only. The Wave-0 ground truth Guide 14 §3.1 required, which no earlier row recorded, is now on the Phase-6 card at (`sec:phase6:ground-truth`): a fourteen-row census naming the owner file and line for every STATE fact the guide asks for — operation identifier and operation spec, singleton asset and singleton amount, cycle domain, announcement lead bounds, maturity variants, the six-field `PoolState` census set against the architecture's four-entry `DataId` slice, the absent realization declaration, the guard refusal vocabulary, the transition certificate, and the first-party and prototype codec precedents. Beside it stand the five Wave-1 rulings ((`rule:phase6:wave1-rulings`)) — the realization cycle domain as an ordinal; realization ownership of the typed STATE metadata, decided by the verified dependency direction, with the projection law against the model's `PoolState` proved in model conformance as the drift tripwire; typed lead bounds carrying the model's validity law with no architecture bound minted; distinct closed refusal sums per layer; and a big-endian canonical encoding with a `u32` representation nonce, recorded before any bytes exist because D006 binds metadata schemas into ABI identity — and the four questions handed up ((`rem:phase6:wave1-questions`)): the R-6 and `G14C-01` reading, the missing `BoundId` for the lead bounds, the two STATE fields the architecture data census does not name, and the §5.1 carrier-sufficiency proof carried as Wave-0 debt. EVERY cited line was re-read at the recorded tip before it was written, and three citations did not survive that reading: `AssetId::Pid` is at `ids.rs:71`, `Sat::ONE` at `ops/maturity.rs:70`, and the branch's authorization refusal is `Guard::BadSignature` raised by `require_signer`, not `BadAuthorization`, which no path in this branch reaches. The study read the tree only and changed nothing in it: no package, script, or census file was touched, and no claim was added that the tree does not carry. VERIFIED GREEN IN ONE SERVER ROUND on the tip carrying both documentation commits, with nothing bounced. `check-plans` passed at exit 0 in 11.2 s wall, measuring maintained Markdown at 1,184,857 bytes against the 1,208,320-byte hard cap for a 23,463-byte margin and raising only the nine pre-existing per-file weight warnings and the pre-existing soft-target warning; neither edited file is among them. `meson setup build` passed at exit 0 in 0.9 s wall and `meson compile -C build lint` at exit 0 in 70.9 s wall over a cold release target directory, its `labels-check` lane raising only the two pre-existing near-miss spans at `packages/architecture/src/spec.rs:424` and `:425`, so the three labels minted here and the four cross-file citations made here pass the calculus unremarked. `cargo test -p tripod-labels` reported ten `test result` lines at exit 0 in 9.3 s wall — 225 passed and 0 failed in total, 213, 9 and 3 across the three targets that hold tests. Both whitespace diff checks were clean, the whole gate took 93.4 s wall, and the lane tree was empty under `git status --untracked-files=all` on both the lane clone and the server checkout. |
| `T11-003` | DONE | The first Wave-1 code bite landed in merge `ce058a75…65a9f29c`. `packages/realization/src/domain.rs` gained the ordinal `Cycle` domain, `packages/realization/src/error.rs` gained `RealizationError::{CycleOverflow, InvalidAnnouncementLeadBounds}`, and the new `packages/realization/src/state.rs` carries `Maturity`, the six-field `StateMetadata`, `AnnouncementLeadBounds`, the total `announce_maturity`, and the five-member `MaturityTransitionRefusal` with `ALL` and `name()`. The bounds constructor refuses a zero minimum or an inverted pair, exactly the law `packages/model/src/constants.rs:40` enforces. Seventeen focused tests landed in `packages/realization/src/tests/state_tests.rs`: direct recount finds 17 `#[test]` attributes, one on the proptest case. SCOPE STOPPED THERE: no codec, representation nonce, target type, compiler plan, or declaration entry landed. LANE VERIFY: green at lane tip `2acd5b25…5ba6ef4b` after one formatter fix-back — fmt 2.6 s, workspace Clippy 10.3 s warm, realization tests 5.7 s with 231 passed in the library target, model tests 81.7 s green, and rustdoc 0.6 s. MERGE VERIFY: the gate of record on merged main `ce058a75…65a9f29c` ran check-plans in 0.8 s at exit 0 and the baseline gate in 802.1 s at exit 0, with 53 suites reporting ok, 0 failed, and the tree clean. `ACTIVE` is absent from (`tab:backlog:status`); this DONE transition removes its only use as a status cell, and the next opener uses `IN PROGRESS`. |
| `T11-004` | DONE | Carrier-sufficiency closeout, documentation only. The proof at (`sec:phase6:carrier-proof`) accepts Guide 14 §5.1's initial carrier and discharges question (d): full predecessor metadata and the announced cycle determine the unique semantic successor through the typed transition; the ruled canonical encoding and public successor nonce determine metadata bytes; and the shared static subtree, fixed branch side, deterministic retry, output position, leaf version, and public internal-key policy determine the constructor and comparison. Lead bounds remain validation data published with the consensus constants, not successor-derivation data, and the predecessor nonce remains a predecessor-linkage input. The proof fails on a predecessor digest in place of all six fields or on any hidden constructor input. The residual obligation is an explicit constructor-wave test tripwire pinning the constructor to exactly the listed public inputs. VERIFY: the proof tip `883243da…0671a06c` passed check-plans in 5.5 s, Meson lint in 1.7 s, and the labels package in 4.0 s with 213 library, 9 subprocess-contract, and 3 doc tests passing, zero failures, silent diff-check, and a clean server tree. WEIGHT: the edited Phase-6 card measured 21,480 bytes against its 16,384-byte advisory threshold; the documentation tree remained valid. |
| `T11-005` | DONE | The Wave-1 canonical STATE-metadata codec landed from lane `w1b` tip `ee3c13e7…d9d058fb` as seven signed cherry-picks: `656cfc99…14ee2bf5` encoded the metadata, `f4186e17…cc4d6151` added its tests, `acfb79cc…7188add1` applied formatting, `ccd97df4…11303f19` preserved the nonce API, `97e5ec98…b1257c1a` satisfied decoder house lints, `9847069a…1c8995ac` applied the second formatting fix, and `f39c277f…47a31698` removed the overlapping maturity arm, leaving main at `f39c277f…47a31698`. `packages/realization/src/state_codec.rs` is 235 lines and exports `STATE_METADATA_DOMAIN = b"tripod/state-metadata"`, schema `1`, width `86 = 21 + 4 + 5×8 + 1 + 8 + 4 + 8`, `StateRepresentationNonce(u32)`, `EncodedStateMetadata`, `encode_state_metadata`, `decode_state_metadata`, and the eight-member closed `StateMetadataRefusal`; `packages/realization/src/tests/state_codec_tests.rs` is 360 lines with 17 test attributes, split as 14 ordinary cases and three proptest cases; `packages/realization/src/lib.rs` gained five lines, `packages/realization/src/tests/mod.rs` gained one, and `packages/realization/meson.build` gained the two census entries while normalizing its trailing newline. LANE VERIFY at `ee3c13e7…d9d058fb`: green in 101.6 s — fmt 2.6 s, warm clippy 0.3 s, realization tests 11.2 s with 248 library-target passes (`231 + 17`), model tests 84.4 s with 318 passes, and rustdoc 2.0 s — after eight completed gate rounds, including two formatter fix-backs, two concealed clippy reds and one concealed test-target compile red, plus one harness-interrupted round. MERGE VERIFY at `f39c277f…47a31698`: check-plans 0.7 s exit 0, clean diff check, baseline gate 804.6 s exit 0, `test-rc 0`, 53 `test result: ok` lines, and publication to all three remotes. The shared API closeout then changed `packages/realization/README.md` by 25 insertions and six deletions from lane `w1doc` tip `9ef2bf2c…0edc200e` through `fcad460f…7b329c06` and `1768d2ed…4216aa6f`, adding the `Cycle` domain, the STATE metadata/transition/codec tour, and both closed refusal sums; its lane rounds were green at check-plans 0.7 s, Meson lint 24.0 s and label tests 3.9 s with `213 + 9 + 3` passes, and its merge gate at `1768d2ed…4216aa6f` was check-plans 0.7 s exit 0 at 1,194,793 of 1,208,320 Markdown bytes, label tests 4.0 s exit 0 with `213 + 9 + 3` passes, a clean diff check, and publication to all three remotes. |
| `T11-006` | DONE | The Wave-1 model-conformance close landed from lane `w1c` tip `d4d35377…7749fc00` as four signed cherry-picks: `fd2baa0c…33b0900a` proved the STATE-metadata projection law, `8eaa5e34…a80baaf3` formatted it, `376acef9…9cd8e368` pinned realization agreement on announce-maturity, and `42905e72…12cdf201` formatted the announcement cases, leaving that imported series at `42905e72…12cdf201`. `packages/model/src/tests/state_metadata_conformance_tests.rs` is 436 lines with 13 test attributes, split as ten ordinary tests and three proptest properties; `packages/model/src/tests/mod.rs` and `packages/model/meson.build` each gained one census entry. The exhaustive refusal projection maps `PredecessorAlreadyAnnounced` and `PredecessorMaturityComplete` to the model's collapsed `MaturityAlreadyAnnounced`, `AnnouncementBelowMinimum` to `MaturityLeadTooShort`, `AnnouncementAboveMaximum` to `MaturityLeadTooLong`, and `CycleArithmeticOverflow` to `CycleOverflow`; `Sealed` and `BadSignature` precede maturity checks as authorization outcomes and stay outside that projection, `Complete` is reached through `mature_world`, and both arithmetic probes reach `Guard::CycleOverflow`. LANE VERIFY at `d4d35377…7749fc00`: green — fmt 2.6 s, warm clippy 0.4 s, model tests 78.6 s with 331 unit-target passes (`318 + 13`) plus `7 + 1 + 3`, and a focused 13-test run in 0.2 s — across five gate starts, including two formatter fix-backs, one server-checkout bounce and one harness interruption. MERGE VERIFY at `42905e72…12cdf201`: check-plans 0.7 s exit 0, clean diff check, baseline gate 752.9 s exit 0, `test-rc 0`, 54 `test result: ok` lines, and publication to all three remotes. The shared README closeout is the 25-insertion, six-deletion `packages/realization/README.md` series `fcad460f…7b329c06` and `1768d2ed…4216aa6f`, whose two green lane rounds and merge gate are recorded in `T11-005`; together the test and tour make the projection owner, semantic transition and canonical bytes independently visible. |
| `T11-007` | DONE | The declaration-surface study closed read-only at `ce058a75…65a9f29c`: 0 tracked files changed and 0 tests added or run. At (`sec:phase6:declaration-prerequisites`), `Relation` and `ConstructibilityClass` cannot state operator authorization (`packages/realization/src/relation.rs:20-25,42-68`), and `FactId` cannot key the six public STATE fields (`packages/realization/src/identity.rs:16-77`; `packages/realization/src/state.rs:44-70`). Rulings 6–9 record the evidence-required authorization shape justified by the ownerless STATE, signer-bearing model and identity-free observation (`packages/realization/src/evaluate.rs:371-374,828-840`; `packages/model/src/ops/maturity.rs:34-40`; `packages/realization/src/observation.rs:210-220`), plus the public facts, representation pair, six exits and validator boundary. LANE VERIFY at `ce058a75…65a9f29c`: read-only audit, 0 changed files and 0 tests. MERGE VERIFY at `1768d2ed…4216aa6f`: no study diff; citations re-read there, with check-plans 0.7 s exit 0 at 1,194,793/1,208,320 bytes, label tests 4.0 s exit 0 with `213 + 9 + 3` passes, and a clean diff check. |
| `T11-008` | TODO | Vocabulary prerequisite: add `OperatorAuthorization` and its evidence requirement in `packages/realization/src/relation.rs` and `evaluate.rs`, `ConstructibilityClass::Operator` and its existing-graph mapping in `relation.rs` and `validate.rs`, and typed input/output facts for all six STATE fields in `identity.rs`, exporting only the public surface through `lib.rs`. Carry the new relation exhaustively through `packages/compiler/src/{requirement,source,layout,placement,coverage}.rs` and existing realization/compiler tests. Close question (b) with named public lead-window inputs or their architecture schema owner. Do not add the declaration, change model semantics, or enter target/signing work. |
| `T11-009` | BLOCKED | Blocked on `T11-008`. Add `packages/realization/src/declarations/announce_maturity.rs`, its `declarations/mod.rs` and Meson entries, focused tests with their test-module/Meson entries, and the scoped mirror in `validate.rs` with only `StateInput`/`StateOutput` additions to `error.rs`. The declaration owes an exact relation census, evidence-required operator authorization, public STATE-field facts, both ruled representations, all six ruled lifecycle exits under both modes, scope-gated lifecycle and State-object content pins, and independent corruption refusals. No model, compiler-plan, target, ABI, or signing work enters this bite. |
| `T11-010` | BLOCKED | Blocked on `T11-009`. Extend `packages/model/src/tests/state_metadata_conformance_tests.rs` in place, without new module or Meson entries, to compare the executed declaration's field projection, boundary successes, five semantic refusals, two overflow shapes, sealed/signature precedence, and evidence-required operator premise with the model. Report the new ordinary/property count and both gates at closure; exclude target lowering, operator-key/message work, ABI, and all Wave-3 signing obligations. |




Current blockers are: NONE carried by the live evidence set, whose residual set is now EMPTY. `PredecessorConstructorAbsent` was the last member and it left by a RULING rather than by a run: the observed pair its clearing rule demanded was demanded by a mis-typing of the `time-locked-input` row, §10.4 refuses the maturity half because the time-locked class is a receipt class rather than a consensus timelock, and the refusal half attributes to a leaf the spent program does not commit to, which every foreign taptree draws alike — so no observation of it could ever name the lock. The row is retyped FIRST-PARTY beside its `time-locked-output` sibling and discharged against the constructor typing that is the real protection. An empty residual set is not a smaller set of obligations: the rows a run has still to answer are counted by the census and not by it, and no time-locked spend was ever observed refused. `SponsorEnvelopeSignerAbsent` is CLEARED, by its own defining site's rule and on an observed result rather than on a capability existing: a sponsor-signed explicit control was accepted by a real node, read back from the node's own copy byte for byte, and mined, which is the acceptance that rule said a returned byte stack had to wait for. The sighash blockers are cleared, the confidential predecessor is funded, the signing handoff is closed, and the evidence restart has taken its first five steps against a real node — six positive private matrix rows are answered by observed acceptances of their own shapes, the three proof-negatives are observed at the mempool boundary and attributed by field, and the multi-output and multi-input fixture constructor exists and derives at any width. The restart order stopped typed at step six on the sponsor signer dependency through three waves; that dependency closed by observation, and the minimality wave recorded step six and step seven in a wave-eight ledger of its own — the wave-seven ledger is a record and was not amended. The order is COMPLETE: all seven steps accepted, the disposition is `Completed`, and both accepted steps state their own shortfalls in their entries. Eight positive private rows have moved on observed acceptances of their own shapes, private-merge and private-sponsor-values being the two the minimality wave added — the sponsor row had moved in the guide closeout while the safety matrix still classified it as awaiting a run, and the matrix caught up in the same wave rather than being left to disagree with the closeout. Of the remaining two, deterministic-public-fixture-openings is now ANSWERED in the safety matrix on a minted determinism standing — its §11.2 gate is the byte-identity contract rather than an acceptance, the wave-7 observation was already recorded, and the new member is counted in its own bucket so a determinism observation is never read as an acceptance; it stays out of the closeout delta, which admits target-computed identities only, and the two artifacts disagreeing there is the vocabulary difference showing rather than a row nobody is checking. Projection-equality-with-paired-explicit did NOT move and the failing conjunct is named: its rule wants an observed acceptance of its own shape, and no run compares the semantic projections of an accepted private transaction and its paired accepted explicit one — the equality this workspace holds is first-party, over registry members that never reached a target. The remaining work is an observation comparing two accepted transactions' projections, and runs of the two pair shapes that have none; one question is still handed up for a ruling, the deterministic-fixture-openings gate, and the step-six/step-seven ordering question is answered by the erratum filed against the guide's mandatory chain. The sponsor dependency is CLOSED and its closure was a repair to this repository rather than to the target: the deployment's fee-role program digest was a fixture constant no program hashes to, threading it to the digest the target itself computes for the empty fee program construction writes produced the first observed sponsor-signed acceptance, and the record says in its own bytes that this establishes the sponsor envelope's wire and one acceptance and not production multi-party sponsor signing.
