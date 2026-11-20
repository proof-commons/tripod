# Tripod Implementation Backlog

> **Status:** ACTIVE
> **Current gate:** Phase 7 — Burn, ASH, and clear
> **Current condition:** Phases 1 and 2 are complete. The target-independent compiler constructs one deterministic, validated scoped analyzed program for the complete pilot scope, factorized per operation, with relation-owned requirements, a corruption-resistant assembly validator, and independent assembly oracles; the Phase-2 exit gate record is in §2.11, including passed document reproducibility. Phase 3 then delivered the Guide-8 target foundation, the Guide-9 native primitive gate, the Guide-10 constructor and wide-floor prototypes, and the Guide-11 declassification result, each with its own gate record. Public declassification is no longer an open blocker: an initial policy is selected — an explicit boundary reached by owner-authorized normalization, with the direct and public-committed paths deferred against three named target blockers — and the gate record is in §2.4. Phase-3 exit is recorded: the Guide-12 preflight register closed all sixteen seventh-review rows, and (`gate:phase3:exit`) was checked green on that tree with the record in §2.6. Phase-4 exit is recorded in §2.8: compact ASH ran end to end against a live target with twelve accepted and matched submissions and honest coverage stated at 100 of 211. Phase-5 exit is recorded in §2.20 and §2.21: the live-transfer campaign closed with the amended exit gate met on a countersigned assessment. Phase 6 — the STATE constructor and maturity announcement — was chartered against Guide 14 as the owner supplied it and archived it verbatim (`T6-001`), with Wave 0 — the tenth static review's disposition and the Phase-5 handoff revalidation — run and closed out in the required order (`T6-002`, `T6-031`) and the owner-issued conceptual preflight register binding Waves 1–13 (`T8-001`), with Wave 13 closed under `T11-147` and the gate record at (`sec:phase6:gate-record`) reading §23 NOT PASSED on five conditional conjuncts and the register fully discharged. Phase-6 exit is recorded in §2.26 and on the Phase-6 card's exit assessment at (`sec:phase6:exit-assessment`): the gate met on its amended text. The current phase is Phase 7 — burn, public ASH and permissionless clear — with Guide 15 chartered at `plans/guides/guide_fifteen.md`; Wave 0 is closed at (`rec:backlog:wave0-baseline`) and Wave 1 at (`rec:w1b16:wave1-exit`). Wave 2's typed burn and clear realization has its bounded execution contract adopted under (`task:guide15-exec:wave2`); no later-wave delivery is claimed.
> **Next gate:** Phase 7 — Wave 2 typed burn and clear realization (`gate:guide15-exec:wave2-gate`); the bounded sequence is adopted at (`rec:backlog:wave2-closure`) under (`task:guide15-exec:wave2`) after the recorded Wave-1 exit.
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

### 2.22 Forward-recapture gate (archived)

Moved verbatim to [backlog history](history/backlog-history.md#later-closed-gate-records).

### 2.23 V1-descent epilogue (archived)

Moved verbatim to [backlog history](history/backlog-history.md#later-closed-gate-records).

### 2.24 Typed-closure and erratum gate (archived)

Moved verbatim to [backlog history](history/backlog-history.md#later-closed-gate-records).

### 2.25 Tripod positioning wave (archived)

Moved verbatim to [backlog history](history/backlog-history.md#later-closed-gate-records).

### 2.26 Phase-6 exit gate (archived)

Moved verbatim to [backlog history](history/backlog-history.md#later-closed-gate-records).

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
|Attestation derivation profiling|TODO|Profile the roughly threefold attestation-derivation regression between `0.7.96-dev` and the bite-1.2 candidate. Retain both runs, w20b1-keyed-cost-20261110T171100Z and w20b1-keyed-default-20261110T183600Z, at (`rec:w1b12:cost-results`) and (`rec:w1b12:default-cost`); compare only within each campaign because identical-code baseline medians moved. No numerical threshold or causal explanation is adopted. The wave-baseline effect is disposed at (`rec:w1b16:baseline`): no delivery failure is attributed to the final default-consumer campaign's 7.239723 s maximum, so the delivery-regression reopen trigger is not met; profiling remains open.|
|Runner cgroup identity retention|TODO|Before any future build-cost comparison, correct `scripts/keyed-cost-kranich.py` to retain each counter sample's cgroup path and inode and establish that start/end counters share a scope. Review the correction before measurement. Current build CPU/stall deltas remain unadmitted, never clamped; both campaign records retain the raw defect. A comparison against `0.7.96-dev` also needs equal external dependency sets, which the runner requires (`scripts/keyed-cost-kranich.py:251`–`252`) and which the dependency-line moves no longer provide.|
|Bite-1.3 concrete-member controls|DONE|Model controls at (`rec:w1b13:closed`), serialized counterparts at (`rec:w1b14:reproduce`) and final native twins/complete histories at (`rec:w1b16:clauses`) and (`rec:w1b16:admit`) cover every member and one changed member beside controls. Exact-byte joins distinguish relations from native execution; an unchanged member in a refused whole transaction is not separately claimed to execute.|
|Cryptographic report assumptions|DONE|The key-bound `Cryptographic` header carries conditional NUMS/discrete-log, preimage, second-preimage/collision and reviewed-target premises, with changed-condition/key/absence controls and independent review (`rec:w1b16:admit`). The typed-form obligation closes; the assumptions are carried, never proven.|
|Closing review B2 — printed Rust totals|TODO|Runner follow-up: print executed/ignored/failed totals from this run's selected journals, joined to its Meson lane census, excluding stale unsplit streams. This would expose denominator movement without reconstructing an archive and keep harness-free targets separate; no runner change is built here.|
|Gate-of-record journal retention|DONE|The loop-owned gate now copies each run’s per-test journals into its evidence directory as `cargo-test-json.tar.gz`, retaining digest, file count and size in the gate log; the `0.7.123-dev` journals were saved before replacement. This preserves future passing durations; the first `0.7.121-dev` timing gap remains explicitly unavailable under (`rec:w1b15:gate-review`).|
|History phase tracing|TODO|Convert test-only printed phase timers to scoped tracing spans while preserving flushed diagnostics when a child times out; the current exception and its reason are recorded in `packages/vectors/TRACING.md` and (`rec:w1b15:gate-review`).|
|Loaded keyed-family hang guards and confidential transport wall|TODO|Vectors test maintenance: read the unchanged external guards in packages/vectors/tests/keyed_history, keyed_final, keyed_bridge, keyed_reproduction and keyed_final_capture against their own loaded-gate walls. The confidential-test correction in `0.7.165-dev` does not calibrate those guards. Retain passing phase walls: the confidential transport's full wall is unknown because its old guard stopped it at 40.6 s and passing printed phase walls are not retained in gate evidence. A passing verdict supplies no missing duration; do not infer headroom or widen guards from it.|
|Activated relation object sets|TODO|Before the first Wave-3 consumer enables the activated bodies, review and pin relation_objects in packages/compiler/src/engine_attestation/requirement.rs. Its provisional sets cover SideRepresentation, DataOutputs, BurnBinding, ClearClamp, ClearAssignment, ClearFloor, ClearFlow, ClearBinding and ClearCanonicalDeltaPolicy. Current requirement lowering refuses them before producing plan/source tables; activation and declaration dispatch do not settle these sets' semantics.|
|Fixed-form S5 review test follow-ups F1–F5|TODO|Extend existing test owners when scheduled; none is claimed executed at S5. F1: member-binder subjects, first OfflineSubjectMismatch to check assertion/context binding, then WrongMember, NotConsumedHandle, UnknownInput, DuplicateMember, DuplicateInput, DuplicateOutpoint, MissingInputIndex, NoRuntimeAssignments, NoConsumedMembers and UnexpectedRelation, to complete named refusal coverage beside accepted controls. F2: literal root, tweak, output key, program and control for the record's control triple beside the independent implementation comparison, to pin the commitment values directly. F3: current input zero born at zero accepted and born at one refused, plus ProgramFact::Other successor refused at instruction 33, to complete the offline birth/non-witness matrix. F4: Extension(code) refused as UnknownParameter and an out-of-range PlacementSelection refused, to cover the linker extension and selection boundaries. F5: multiple members with non-zero other witness bytes, to exercise both terms of transaction_weight beyond the existing one-member and overflow controls. Owners are the existing tapscript/linker keyed_fixed_bundle tests and vectors keyed_fixed_template test under (`step:kffwork:s5`); this row grants no new test mechanism.|
|Fixed-form sibling consolidation|TODO|At the first edition’s next authorized, independently reviewed re-freeze, assess sharing source_evidence, proof_records, generic read_shells, owner-generic restoration and owner-independent public vocabulary. S9 carries the cost of two compiling owner families accepted by ruling 208 at (`rule:phase7:fixed-form-rulings`); no shared mechanism or re-pin is authorized solely by this follow-up. Preserve edition isolation and supply the source/history/evidence disposition under (`rule:vectors:keyed-refreeze`).|
|Frozen relation documentation|TODO|At the next independently reviewed re-pin of packages/realization/src/relation.rs under (`rule:vectors:keyed-refreeze`), add the deferred documentation for ClearCanonicalDeltaPolicy's conditional residual policy. The comment was declined in the previous packet because this is frozen source. It is not a 2.4/2.6 edit or a reason by itself to re-pin; carry it with that source's next authorized re-pin and its source/history/evidence record.|
|Test-directory collision|REPAIRED|The shared vectors allocator uses exclusive temporary-directory creation with collision retry; an eight-thread, 512-directory same-label control checks distinct empty owned directories (`rec:w1b16:test-directory`). The `0.7.124-dev` gate remains RED; the repair's covering gate is separate evidence.|
|Keyed freeze identity admission|DONE|The provenance class is admitted at (`rec:identities:keyed-capture-freeze`): owned canonical grammar preserves the existing recipe and historical bytes; Dependencies binds whole manifest bytes. The present-class prerequisite is satisfied for both dependent re-pins under (`rule:vectors:keyed-refreeze`); each still supplies its own controls and evidence disposition. Ruling 210 at (`rule:phase7:fixed-form-rulings`) places this admission ahead of S2; ruling 211 assigns the distinct fixed-form capture-freeze admission to S5, with S8's consumer migration before capture. S2 specifies the realization recipe and records pre-admission. (`rec:backlog:fixed-form-decisions`). |
|Digest census gaps and adjudication|TODO|Walk (`[ADR021-alg:identity:adjudication]`) under (`rule:backlog:identity-admission`) for `ValidatedKeyedManifest::binding` in `packages/architecture/src/keyed.rs`: SHA-256 of the schema-2 keyed manifest’s domain, schema/profile/key, paper/version/anchor set and sorted object/operation/clause rows. The same debt covers further computed families found by source search: `ConfidentialFixtureRegistry::register_with_source` in `packages/target-elements-conformance/src/confidential_fixture.rs` hashes its framed manifest/openings transcript; `digest` in `packages/transaction/src/operator_right.rs` tags candidate bytes; `manifest_digest` in `packages/vectors/src/keyed_final_capture.rs` hashes capture-manifest JSON; `content_address_run` in `packages/vectors/src/live_report.rs` hashes framed archive/request/fact/response material. Also walk archive/census/file/retention/provenance digests produced or checked by `packages/vectors/tools/extract-keyed-final.py`, `packages/vectors/src/keyed_final_corpus.rs`, the other vectors corpus readers, `scripts/keyed-final-capture/`, `scripts/elements-observer-capture.py`, `scripts/live-native-*-capture.sh`, `scripts/compact-native-capture.sh` and `scripts/keyed-cost-kranich.py`; first-party subject/program byte bindings in `packages/vectors/src/keyed_final_native.rs`, `keyed_final_manifest.rs`, `maturity_continuity.rs`, `maturity_history.rs`, `maturity_adapter.rs` and reader/stack helpers; and derived test-material values such as `test_scalar` in `packages/target-elements-conformance/src/conservation.rs` and fixture derivation helpers. Establish exact family membership and existing-stop applicability, then decide admission with every required fact or a named stop; source locations and hash-citation coverage are not admission. No binding is adjudicated by this row and no new completeness claim is made. (`tab:identities:census`).|
|Keyed value consequences|TODO|A later-position covenant output is unspendable; its value has no record unless SOURCE data and a published reader rule expose it (`[A-prop:kpc-model:single-birth]`), (`[A-rem:kpc-interface:record-evidence]`). A live handle's value above the positive explicit consensus floor can go to its extender: the clauses check neither value nor authority (`[A-def:kpc-model:first-output]`); the relay floor is observed in `plans/research/state-constructor.md`'s keyed funding record. Add both consequences to the paper's model and Guide-15 execution exclusions at paper revision. Adopted receiver: S1 paper, S0/S9 both guide exclusions, S6/S8 malformed SOURCE and surplus controls. (`rec:backlog:fixed-form-closure`). |
|Keyed reading cost and chain witness|TODO|Reading needs complete dependency closure (`[A-sec:kpc:interface]`); later gathering enlarges only future handles' closure. A membership chain witness grows with extensions and omits other ancestry (`[A-thm:kpc-invariants:unique-successor]`). Costs remain open (`[A-sec:kpc:limitations]`). State them in paper reconstruction or limitations; decide whether the vectors reader offers chain-witness reading. Adopted receiver: S1 closure/path explanation; S7/S8 measured full reading; chain-witness API remains deferred. (`rec:backlog:fixed-form-closure`). |
|Live-handle contention limits|TODO|Retry after a competing extension gathers a superset; SOURCE consumes no handle (`[A-thm:kpc-invariants:set-membership-growth]`), (`[A-op:kpc-operations:source]`). Lost-race count/cost, replacement or first-seen policy, producer discretion and confirmed-to-final depth lack bounds or facts. State limits in the paper and measure conflict policy with deployment facts (`tab:w1b16:exit-receivers`). Adopted receiver: S1 limits and S8 bounded default-policy conflict/retry; no finality or fairness closure; deployment-fact measurement stays at (`tab:w1b16:exit-receivers`). (`rec:backlog:fixed-form-closure`). |
|Extender-chosen successor form|TODO|An extender chooses output 0's asset/value form because only its program is compared (`[A-def:kpc-model:first-output]`). Only explicit policy-asset successors were built or observed (`sec:w1b16:packet-domain`); another party's ability to spend an extender-chosen form is open. Name it in packet domain limits and answer before non-explicit admission. Adopted receiver: S3–S8 fixed-form predicate/native evidence, S9 scoped closure; legacy fully-confidential question stays unproved; stated in the maintained vectors contract in place of the archived packet limits, which stay unchanged. (`rec:backlog:fixed-form-closure`). |
|Keyed template document of record|TODO|Leaf bytes/version, internal key, tags and control layout exist in code and plan records (`rec:w1b13:resources`), (`rec:w1b13:constructor-surface`), which have no semantic authority; the paper assumes no mechanism (`[A-def:kpc-model:keyed-membership]`). Add a provenance-cited keyed realization note in `docs/`, like `docs/attestation/realization.md`, for independent byte comparison. Adopted receiver: S2 DOC owner, S5/S8 byte comparison, S9 publication join. (`rec:backlog:fixed-form-closure`). |
|Native observation environment|TODO|Runs used one Elements Core development build, daemon v28.99.0, fresh isolated `elementsregtest` nodes and harness blocks under the default challenge without block signing (`rec:w1b16:packet`); no release, Liquid or signed-block production was observed. State this in packet limits (`sec:w1b16:packet-domain`) and `packages/vectors/README.md`'s Wave-1 packet; reconcile the two required-tip descriptions in `lint/hash-citation-families.tsv` and the plans. Adopted receiver: S2 maintained vectors contract/README clarification, S8 effective environment, S9 tip-description reconciliation; stated in the maintained vectors contract in place of the archived packet limits, which stay unchanged. (`rec:backlog:fixed-form-closure`). |
|Attributable destruction placement|TODO|MERGE and no-handle events expose no record (`[A-op:kpc-operations:merge]`), (`[A-def:kpc-operations:classification]`). Attributed destruction must be in its record's SOURCE event; MERGE-content reading lacks SOURCE-only monotonicity (`[A-rem:kpc-interface:record-evidence]`). State this at paper revision. Adopted receiver: S1 SOURCE sentence, S7 attribution controls, S9 join. (`rec:backlog:fixed-form-closure`). |
|Burn and Clear separation in the paper|TODO|Wave-2 adoption separates keyed/shared-engine identity from attestation semantics in `plans/backlog.md`; Burn/Clear (`rule:phase7:wave2-rulings`), (`sec:realization:burn-clear`) adds no keyed destruction. The paper's operations section (`[A-sec:kpc:operations]`) lacks the sentence urged by `plans/drafts/wave2-route-study.md`; add it at revision. Adopted receiver: S1 operations sentence, S3/S4 isolation controls, S9 join. (`rec:backlog:fixed-form-closure`). |
|Graceful process-group signal control|TODO|`packages/target-elements-conformance/tests/executor_supervision.rs` proves eventual descendant exit, but leader-only graceful signalling would pass before forceful group cleanup (`rule:guide10:process-group`). Add a child ending only on the graceful signal and observe exit before force.|
|Unadopted child group cleanup|TODO|`UnadoptedChild::drop` kills only its direct child in `packages/target-elements-conformance/src/executor.rs`; a descheduled thread allows a pre-adoption fork. Signal the group or record, with a control, why that is unnecessary (`rule:guide10:process-group`).|
|Harness timing follow-ups|TODO|`keyed-final-capture-wrappers` has 300/127.842789 ≈ 2.35-fold headroom; the fourfold rule (`rec:w1b15:wide-bound`) needs a split-or-report decision (`plans/drafts/wave1-bite-1-6-plan.md`). `cargo-lane-contracts`, `vectors-target-census`, `compact-native-capture-contract` and `compact-native-funding-contract` lack budgets/tolerances beside `test(` in `meson.build` and `scripts/meson.build`; measure and record them. `scripts/ci-timing-report.py` reports every wall; decide on budget/tolerance comparisons as reports, never failures (`sec:test-performance:tracking`).|

### Wave-1 final packet evidence (archived)

The verbatim packet receipt is in [backlog history](history/backlog-history.md#closed-wave-records).

**Wave-1 status** · `tab:backlog:wave1-exit`

| Bite or wave | Status | Exit evidence |
|---|---|---|
| Bite 1.0 | DONE | Provisional six-family measurement packet and decisions: (`rec:phase7:wave1-bite-1-0-admission`); final repetitions belong to bite 1.6. |
| Bite 1.1 | DONE | Validated stable declarations and architecture weld: (`rec:phase7:wave1-declarations`). |
| Bite 1.2 | DONE | Shared compiler plans, pins and measured consumer comparison: (`rec:w1b12:closed`). |
| Bite 1.3 | DONE | Default emission and model member controls: (`rec:w1b13:closed`). |
| Bite 1.4 | DONE | Linking, commitment, public construction and typed envelope refusal: (`rec:w1b14:closed`). |
| Bite 1.5 | DONE | Reader, premise form and six report families: (`rec:w1b15:closed`). |
| Bite 1.6 | DONE | Final native subjects, admission, reviewed packet, full gate and separate closure: (`rec:w1b16:closed`). |
| Wave 1 | CLOSED | Three criteria and exact domain: (`rec:w1b16:wave1-exit`), ruling 181 at (`rule:phase7:wave1-exit-ruling`). |
| Wave 2 | CLOSED | Closure imported and gated as `0.7.189-dev`; frozen semantic delivery at (`rec:backlog:wave2-frozen`); later-wave obligations remain open. |

The realized domain is one once-reviewed template with conditional authentication, observed under two representative public keys. Every 32-byte key is data; commitment succeeds or returns its reviewed typed refusal. Per-key construction, reading and premises remain necessary, but a key-specific campaign is not a precondition of use under unchanged frozen conditions. There is no deployment activation/finality, calibration, universal-security or constant-native-cost claim (`sec:w1b16:packet-domain`). All seven follow-ups above have named dispositions at (`tab:w1b16:packet-followups`); profiling, runner counter identity and history phase tracing remain open with their reasons.

### Wave-2 adopted sequence and closure · `rec:backlog:wave2-closure`

Under (`rule:phase7:wave2-rulings`), (`sec:realization:burn-clear`) owns the clause/identity/graph/witness/six-interface census. The twelve original packets in (`sec:wave2:execution`), with the path/consumer corrections and added bite 2.7a below under ruling 213, govern execution. This record adopts the drafts without rewriting them. That adoption opened code; the frozen delivery judgment is (`rec:backlog:wave2-frozen`).

The join covers thirteen packet gates, eight deliverables, seven entry rows, dispositions 1/7/10/12, two compact reopenings, ruling 133's three subjects, six ledgers, comparable baselines, full gate logs and absent dimensions. Wave-3 plan proofs, Wave-4 carrier measurements, linked constructor equality, Wave-7 STATE generation, target signatures, native execution and Waves-10–12 reports remain owed. Results update implementation standing, not historical preflight findings.

**Table (Wave-2 task packets)** · `tab:backlog:wave2-tasks`

| Task reference | Status | Closing criterion |
|---|---|---|
| (`step:wave2:bite-2-0`) | DONE | Adopted with review corrections as `0.7.147-dev`; (`gate:wave2:bite-2-0`) is met: every deliverable, preflight and obligation row has its producer and acceptance boundary at (`tab:backlog:wave2-join`); all six eight-item interface ledgers and witness obligations are adopted at (`tab:realization:burn-clear-interfaces`) and (`tab:realization:burn-clear-witnesses`); plans/labels/census lanes passed in the full repository gate on `0.7.151-dev` (receipt below). Adoption and clause census remain at (`sec:realization:burn-clear`) and (`tab:backlog:wave2-paths`). |
| (`step:wave2:bite-2-1`) | DONE | Typed Clear assignment and exhaustive refusal/field tests; (`gate:wave2:bite-2-1`). Landed as `0.7.151-dev` after independent review, required corrections and re-read; full repository gate green at 2026-11-16T08:41:23Z, 2,844s. Receipt below. |
| (`step:wave2:bite-2-2`) | DONE | Bound internal projections and independent record-credit verdict; (`gate:wave2:bite-2-2`). Landed as `0.7.153-dev`; full repository gate on `0.7.154-dev` green at 2026-11-16T10:20:03Z, 3,280s. Receipt below. |
| (`step:wave2:bite-2-3a`) | DONE | Evaluated law bodies and frozen vocabulary design; (`gate:wave2:bite-2-3a`). Landed as `0.7.156-dev`; full repository gate on `0.7.157-dev` green at 2026-11-16T11:43:37Z, 3,589s. Receipt below. |
| (`step:wave2:bite-2-3b`) | DONE | Activated vocabulary, injective engine-key codes and compiler/vector compatibility; (`gate:wave2:bite-2-3b`). Landed as `0.7.166-dev` after independent review; full repository gate green at 2026-11-16T19:49:18Z, 4,265s. Receipt below. |
| (`step:wave2:bite-2-4`) | DONE | Complete Burn declaration and weld; (`gate:wave2:bite-2-4`). Landed as `0.7.170-dev` after independent review; full repository gate green at 2026-11-17T08:41:11Z, 3,287s. Receipt below. |
| (`step:wave2:bite-2-5`) | DONE | Independent Burn conformance; (`gate:wave2:bite-2-5`). Landed as `0.7.176-dev` after independent review; full repository gate on `0.7.177-dev` green at 2026-11-17T10:17:38Z, 1,788s. Receipt below. |
| (`step:wave2:bite-2-6`) | DONE | Complete Clear declaration/weld and immediate ruling-133/138 reopenings; (`gate:wave2:bite-2-6`). Clear will collide with CompactAsh and Burn on shared lifecycle nodes/edges and operation-independent disclosure bounds, and with AnnounceMaturity on STATE lifecycle nodes. Record and pin these assembly refusals as 2.4 did, without changing shared owners; TransferLive or Clear alone supplies the reaching control. Landed as `0.7.180-dev` after independent review; full repository gate on `0.7.182-dev` green at 2026-11-17T14:05:38Z, 2,559s. Receipt below. |
| (`step:wave2:bite-2-7`) | DONE | Independent Clear conformance and six-field comparison; selected design at (`sec:backlog:wave2-clear-adapter-design`); (`gate:wave2:bite-2-7`). Consumed-code dispositions from the 2.5 review: read event omega/Y from ClearProjection, X from the certificate's U/Recon destruction, residual from its created ASH outputs in the after-world, and successor from after-world STATE; never recompute them with realization clear_ash. Read accounting evidence from the certificate. Burn's order/fresh-ASH token formula does not carry to Clear; bind Clear's own certificate and STATE identities. Reuse the generic record_projection completion-event helper. Keep the Burn adapter's field decision: common retains its protected model provenance; extension/event/accounting are public comparison values, not jointly authenticated fields. Supply both `AshBatchMax` and `FeeSponsorInputMax` in accepted observations; test a missing ASH maximum at the actual evaluation boundary, not by assuming the staged-law error. Landed as `0.7.183-dev` after independent review; full repository gate green at 2026-11-17T14:56:37Z, 988s. Receipt below. |
| (`step:backlog:wave2-bite-2-7a`) | DONE | Shared operation-graph union; (`gate:backlog:wave2-bite-2-7a`). Landed as `0.7.185-dev` after independent review; full repository gate green at 2026-11-17T17:12:01Z, 3,106s, also covering `0.7.184-dev`. Receipt below. |
| (`step:wave2:bite-2-8`) | DONE | Joint region/projection/public-dependency evidence; (`gate:wave2:bite-2-8`); selected design at (`sec:backlog:wave2-joint-design`). Landed as `0.7.186-dev` after independent review; full repository gate green at 2026-11-17T17:41:48Z, 950s. Supplies the compact comparison producer and substituted-region subject. Raw Burn/Clear tokens remain incomparable across operations. Receipt below. |
| (`step:wave2:bite-2-9`) | DONE | Candidate-bound dispositions at (`rec:backlog:wave2-candidate-dispositions`); landed as `0.7.188-dev` after independent review. Full repository gate green at 2026-11-17T18:53:06Z, 1,198s, also covering `0.7.187-dev`. Receipt below; (`gate:wave2:bite-2-9`). |
| (`step:wave2:bite-2-10`) | DONE | Closing packet landed as `0.7.189-dev` after independent review and corrections; its own full repository gate green at 2026-11-18T09:24:05Z, 3,218s. Both tripod:labels-check and tripod:plans-check passed in CI and the real-document subset. (`gate:wave2:closing`) is met; receipt and resources at (`rec:history:wave2-closure-import`). |

Bite 2.9 receipt: `0.7.188-dev` retains the five candidate-bound dispositions in (`rec:backlog:wave2-candidate-dispositions`). Independent review confirmed that compact_clear_declaration_reopenings_keep_both_keys asserts the validator's actual returned values and that the two emitted-Clear keys are two coverage identities over one comparison, not independent observations. The landing includes narrower records and explicit compact tree, archive and STATE digest evidence. Full repository gate green at 2026-11-17T18:53:06Z, 1,198s, also covering `0.7.187-dev`.

### Frozen Wave-2 closing packet · `rec:backlog:wave2-frozen`

The implementation is frozen at `0.7.188-dev`. Wave 2 and bite 2.10 are closed on the reviewed closing packet `0.7.189-dev`, whose own plans/labels import condition passed; (`rec:history:wave2-closure-import`) retains its full-gate receipt and the wall comparison. The gate (`gate:guide15-exec:wave2-gate`) is MET for complete typed declarations and independent semantic conformance, with the bounded evidence below. (`gate:wave2:closing`) joins that result without claiming compiler plans, emitted Clear, target authentication or a new protocol. The full gate and baseline comparison are (`rec:history:wave2-frozen-gate`); the already-adopted Wave-3 assumption is (`assum:guide15-exec:wave3-inherited-inputs`). This is the current consuming result, not a rewrite of preflight history. Closed implementation receipts/designs below move verbatim to history; live gates, interface contracts, consumers and follow-ups remain maintained.

**Table (Delivered semantic items)** · `tab:backlog:wave2-delivered`

Standing legend: MET meets the named bounded Wave-2 item; MET semantically excludes target execution; MET internally excludes an external report; MET at a semantic/declaration boundary excludes the later compiler, constructor and authentication evidence named in that row. None closes an undischarged receiver.

Test names below resolve in packages/realization/src/tests/{burn_tests.rs,clear_tests.rs,burn_clear_projection_tests.rs} (R), packages/model/src/tests/{burn_realization_conformance_tests.rs,clear_realization_conformance_tests.rs,burn_clear_composition_tests.rs,realization_conformance_tests.rs} (M), and packages/vectors/src/{first_party.rs,maturity_mutants/native_staging.rs} (V). Each cited suite is included in the frozen full gate; a test name is evidence of its stated claim only.

| Deliverable | Landing and evidence | Standing |
|---|---|---|
| 1 Complete Burn | `0.7.170-dev`; R relation_bodies_and_proofs_match_the_adopted_literal_rows, architecture_weld_refuses_each_changed_coordinate_and_duplicates, every_shell_requirement_refuses_omission_and_permutation_is_accepted | MET: complete shell, exact weld and omission refusals. |
| 2 Complete Clear | `0.7.180-dev`; R clear_shell_matches_plan_literals, clear_weld_refuses_changed_coordinates, clear_shell_refuses_omission_and_accepts_permutation; M clear_floor_uses_actual_successor_supply | MET: exact assignment/clamp and complete shell; independent floor evidence is direct staged-law evaluation. ClearFloor cannot fail in the complete Clear report: failed assignments block it, and passed clamp/assignments imply it. Wave 7 must keep this distinction. |
| 3 Event/accounting projections | `0.7.153-dev`, `0.7.176-dev`, `0.7.183-dev`, `0.7.186-dev`; M seven_projection_roles_keep_separate_results, matrix below | MET internally: seven distinct roles, no report producer. |
| 4 Exact Recon tag | `0.7.180-dev`, `0.7.183-dev`; M clear_destruction_coordinates_refuse_changes; R review_clear_binding_rejects_second_leg_and_issuance | MET semantically: exact U/Recon amount/count/association; no native destruction claim. |
| 5 Residual rule | `0.7.151-dev`, `0.7.183-dev`; clear_state_tests.rs each_limiter_and_tie_full_and_partial; M clear_residual_coordinates_refuse_changes | MET semantically: positive B−X yields one ASH, zero yields none; no zero placeholder. Wave-7 residual emission remains owed. |
| 6 Public ASH availability | `0.7.170-dev`, `0.7.180-dev`, `0.7.186-dev`; R constructor_facts_and_witnesses_are_exact_and_public_ash_has_no_creator_secret; M joint_public_dependencies_match_literal_obligations, public_compact_and_clear_refuse_protocol_signers | MET as dependency/lifecycle declaration evidence; constructor bytes and public recovery remain later work. |
| 7 Architecture weld | `0.7.170-dev`, `0.7.180-dev`; R Burn/Clear coordinate and coherent-removal tests above | MET per operation, not architecture-wide completeness. |
| 8 Independent conformance | `0.7.176-dev`, `0.7.183-dev`, `0.7.186-dev`; M accepted_burn_matrix_agrees_with_complete_declaration, clear_model_matrix_conforms, clear_successor_six_fields_match_both_branches, sequential_burn_compact_clear_preserves_credit_identity | MET: execution-derived observations and independent model laws for Explicit representation only; substrate discharge is model-only. Private change and PublicCommitted outputs have declaration admission, not model-conformance evidence. |

**Table (Seven entry consumers)** · `tab:backlog:wave2-entry-results`

| Entry condition | Evidence | Wave-2 standing / remaining receiver |
|---|---|---|
| (`cond:guide15:operation-scope`) | `0.7.170-dev`/`0.7.180-dev` dispatch and weld; V realization_operation_census_retains_only_the_announcement_state_transition | MET for Burn/Clear; other unsupported operations stay unsupported. |
| (`cond:guide15:validated-relations`) | Both complete shell tests and independent matrices above | MET semantically; compiler/target proofs remain open. |
| (`cond:guide15:target-independent`) | packages/realization/Cargo.toml and src/burn_clear_observation.rs: primitive domain carriers, BurnAddress bytes; no target adapter in this owner | MET at semantic boundary; target obligations remain separate. |
| (`cond:guide15:typed-records`) | `0.7.153-dev` R records_accept_repeated_addresses_maximum_and_contiguous_ordinals; `0.7.176-dev` model credit | MET for typed internal records; Wave 10 reports remain owed. |
| (`cond:guide15:state-owner`) | `0.7.151-dev` state.rs and `0.7.183-dev` six-field comparison on both branches | MET: omega, y_l, y_t, q, cycle, maturity; no codec/generation change. |
| (`cond:guide15:clear-assignment`) | M clear_successor_six_fields_match_both_branches, clear_copied_fields_refuse_changes, clear_floor_uses_actual_successor_supply | MET: only y_l changes, Y is derived; Wave 7 lowering remains owed. |
| (`cond:guide15:recon-tag`) | M clear_destruction_coordinates_refuse_changes | MET semantically; Wave 7 execution remains owed. |

**Table (Four disposition results)** · `tab:backlog:wave2-disposition-results`

| Disposition | Evidence | Standing |
|---|---|---|
| 1 Model census is input | Both declaration welds plus independently executed matrices, `0.7.170-dev` through `0.7.183-dev` | MET; model support alone was not credited as dispatch. |
| 7 Stored/derived distinction | M six-field comparison and copied-field subjects, `0.7.183-dev` | MET; Y is not another stored write. |
| 10 Seven roles | `0.7.186-dev` computed matrix below | MET separately; event equality discharges neither accounting nor STATE succession. |
| 12 Internal projections | src/burn_clear_observation.rs and src/burn_clear_projection.rs; (`tab:realization:burn-clear-interfaces`) | MET as internal types; external interchange requires its own ADR-022 decision. |

**Table (Six interface evidence joins)** · `tab:backlog:wave2-interface-results`

This joins column 8 of (`tab:realization:burn-clear-interfaces`) to the frozen `0.7.188-dev` tree. R and M use the file legend above; additional files are named explicitly. MET here is the ledger's bounded semantic contract, not every possible permutation. Named broader follow-ups are not silently counted as executed controls.

| Ledger | Listed controls and landed evidence | Standing and receiver |
|---|---|---|
| Clear transition/STATE | Limiters/ties and residual branches: realization clear_state_tests.rs each_limiter_and_tie_full_and_partial; zero/overflow: every_refusal_is_reached_and_named and each_refusal_has_an_accepted_control_one_coordinate_away; Sealed versus overflowing B: sealed_precedes_ash_overflow_and_structural_guards_precede_sealed. M clear_successor_six_fields_match_both_branches, clear_copied_fields_refuse_changes and clear_floor_uses_actual_successor_supply cover each field and direct independent floor. | MET, `0.7.151-dev`/`0.7.183-dev`; no stored-field/codec change. Wave 7 owes native assignments/floor and preserves blocked-versus-direct-law evidence. |
| Relations/engine keys | Coherent omission: R every_shell_requirement_refuses_omission_and_permutation_is_accepted and clear_shell_refuses_omission_and_accepts_permutation. Wrong/duplicate IDs: realization burn_clear_relation_tests.rs activated_census_checks_every_identity_body_and_multiplicity. Absent facts: structural_binding_refuses_duplicate_recon_wrong_association_and_missing_facts in that file. Namespace collision: compiler engine_keyed_tests.rs keyed_binding_refuses_an_attestation_scope_with_its_numeric_collision; compiler relation_tests.rs duplicate_relation_rejects. Typed unsupported plans: compiler tests/public_api.rs declared_burn_reaches_unsupported_plan_at_input_binding and declared_clear_reaches_unsupported_plan_at_input_binding. | MET through `0.7.180-dev`; old pilots/keyed identities stay separate. Wave 3 owns actual plan activation and provisional relation_objects; this join grants no plan proof. |
| Bound extension | Missing extension and wrong certificate/transaction/snapshot: R binding_refuses_wrong_region_certificate_transaction_and_snapshot. Well-formed wrong region: valid_foreign_region_selection_cannot_reuse_an_extension; foreign object/ASH: foreign_payload_references_refuse_in_a_well_formed_snapshot and structural_refusals_have_one_coordinate_controls. Wrong STATE: M clear_context_and_state_association_refuse_changes; maturity control: R state_fields_are_exhaustive_operation_qualified_and_never_defaulted; legacy scope entry: realization burn_clear_relation_tests.rs public_bound_entry_refuses_undeclared_operations. | MET, `0.7.153-dev` through `0.7.186-dev`; internal bound carrier only. Clear-specific drop/replace source-entry and event-source permutations remain 2.7 F3, not additional executed evidence. Target authentication remains later work. |
| Model adapters | Accepted controls: M accepted_burn_matrix_agrees_with_complete_declaration and clear_model_matrix_conforms. Substituted execution/source/payload: binding_and_projection_refusals_keep_their_boundaries; Clear context/STATE and projection-role subjects. Equal-total split: equal_total_changed_split_passes_standalone_and_disagrees_with_model. Substituted region: substituted_region_and_foreign_payload_refuse at 2.8, not 2.5. | MET, `0.7.176-dev`/`0.7.183-dev`/`0.7.186-dev`; Explicit only, independent model laws and model-only substrate evidence. Owner swap 2.5 F2 and Clear source permutations 2.7 F3 remain deferred. |
| Data outputs | Empty and beyond-domain overclaim: M accepted_burn_matrix_agrees_with_complete_declaration; max/repeated addresses: R records_accept_repeated_addresses_maximum_and_contiguous_ordinals; over-bound/bad ordinal/tag/association: structural_refusals_have_one_coordinate_controls. Asset/count/split destruction: M clear_destruction_coordinates_refuse_changes; R clear_carrier_refusals_reach_independent_boundaries and review_clear_binding_rejects_second_leg_and_issuance; realization burn_clear_relation_tests.rs structural_binding_refuses_duplicate_recon_wrong_association_and_missing_facts. | MET, `0.7.153-dev` through `0.7.183-dev`; structure, credit and canonical destruction remain distinct. Native carrier/destruction and reports remain Waves 4/6/7/10. |
| Burn representation | Explicit input/private change: R complete_burn_evaluates_with_private_change_and_independent_overclaim; private input despite union: private_input_is_not_admitted_by_the_input_change_mode_union. Retained private exit: realization live_transfer_tests.rs burn_dispatch_preserves_the_deferred_private_receipt_exit. The former direct-case refusal is replaced by compiler case_tests.rs direct_case_entry_accepts_complete_burn_and_refuses_changed_case at 3.2b. | MET, `0.7.166-dev`/`0.7.170-dev`; unioned lifecycle modes do not widen input admission. Mixed-scope realization pin 2.7a B1 and compiler reaching subject below remain Wave-3 follow-ups; no private-route discharge. |

**Table (Frozen semantic counters)** · `tab:backlog:wave2-semantic-counters`

Counted from declaration constructors and independently enumerated literal tests, not inferred from the proposal. Burn's shell_census_has_literal_identities_counts_and_disclosure_result checks each collection; Clear's clear_shell_matches_plan_literals enumerates each endpoint; compact_ash.rs supplies the unchanged control. Nodes/edges are collection entries per operation before the shared union. Only disclosure graphs have seeds; constructibility and lifecycle have no seed field.

| Operation | Relations / dependencies | Constructibility nodes / edges | Lifecycle nodes / edges | Disclosure nodes / edges / seeds | Stored assignments |
|---|---|---|---|---|---|
| Burn | 32 / 42 | 13 / 12 | 9 / 10 | 15 / 16 / 2 | 0 |
| CompactAsh | 23 / 24 | 5 / 4 | 4 / 4 | 6 / 4 / 2 | 0 |
| Clear | 44 / 55 | 15 / 14 | 12 / 16 | 32 / 40 / 4 | 6 |

Recount method: compact has 24 literal dep calls, five constructor nodes/four edges and four fact plus two relation disclosure nodes. Clear has 13 facts (four counts, ASH amount, two bounds, six input fields), one operation and one witness; lifecycle is ASH 4/4 plus STATE 8/12. Clear dependencies expand as 3+12+3+18+9+10=55; disclosure nodes are 13 relations+13 facts+6 output fields=32, edges 13+3+1+6+2+3+6+6=40. The six stored assignment identities are omega, y_l, y_t, q, cycle and maturity; Y remains derived. Complete supported dispatch is five operations: AnnounceMaturity, Burn, Clear, CompactAsh, TransferLive, as pinned by realization_operation_census_retains_only_the_announcement_state_transition. The separate role/premise census remains (`tab:backlog:wave2-role-results`): seven roles; Burn/compact/Clear applicable counts 4/3/4 and one operation-qualified Lbtc substrate premise each.

**Table (Model control and negative counters)** · `tab:backlog:wave2-model-counters`

Counting units are explicit matrix rows and changed subjects, not assertions, test functions or unique histories. A subject checked at several boundaries counts once. These are bounded denominators: adapter matrices and their focused negative tests, plus the joint role matrix; additional composition/region tests remain named evidence, not silently folded into these counts.

| Source and operation | Accepted matrix controls | Focused negative subjects and derivation |
|---|---|---|
| M burn_realization_conformance_tests.rs | 7: empty, underclaim, exact, multiple inputs/change, repeated-address overclaim, beyond-domain overclaim, sponsored | 12: binding/projection function 6; same-owner equal-total split 1; entry function 5 (extension, scope, three removed bounds). Overclaim controls are accepted operations, not operation negatives. |
| M clear_realization_conformance_tests.rs | 8: full, partial, B/y_l tie, B/floor tie, three-way tie, three ASH, sponsored, descending request | 29: floor 1; copied fields 5; destruction 5; residual 5 (two event-value branches, one duplicate count, two accounting); context/STATE 5; independent roles 3; entry 4 (two bounds, extension, scope); model ZeroProgress 1. The two six-field successor branch controls are separately checked, not additional matrix rows. |
| M seven_projection_roles_keep_separate_results, Burn | 2: exact-credit, overclaim | 2: changed event offer and duplicate ordinal |
| Same joint matrix, CompactAsh | 1 | 1: inserted ClearEvent; ProjectionPolicy fails and Constructibility/Representation/lifecycle dependents block |
| Same joint matrix, Clear | 1 | 3: event, accounting and STATE offers |

Thus the two adapter matrices have 15 controls and their focused functions 41 subjects; the joint matrix has 4 controls/6 subjects. These sets overlap in purpose and are not summed into a distinct-history or whole-model coverage claim. The compact row is the unchanged operation's control, not a new compact adapter law or a new native refusal.

**Table (Witness evidence join)** · `tab:backlog:wave2-witness-results`

The exact identities and external evidence names remain (`tab:realization:burn-clear-witnesses`); these nine distinct witness rows include shared B/C rows rather than counting each twice.

| Witness | Landed semantic evidence | Undischarged boundary |
|---|---|---|
| CanonicalDelta B/C | Burn complete evaluation (`0.7.170-dev`); Clear destruction matrix (`0.7.183-dev`) | Target substrate and Wave-3 plans/Wave-7 destruction. |
| AshLineage B/C | Literal lifecycle products and union (`0.7.185-dev`); M clear_residual_does_not_become_burn_or_residue (`0.7.186-dev`). The separate sequential_burn_compact_clear_preserves_credit_identity ends in a full Clear with no residual | LinkedConstructorEquality and PublicRecovery, Waves 5/8/11. |
| ReceiptOwnerRouting B | M equal_total_changed_split_passes_standalone_and_disagrees_with_model (`0.7.176-dev`): amount split under one owner, CommonObservation/BurnEvent disagreement | Owner-swap subject remains 2.5 F2 at (`tab:backlog:wave2-design-followups`); FinalizedBurnAuthorization signatures, Wave 9. |
| ReceiptClassClosure B | R private_input_is_not_admitted_by_the_input_change_mode_union (`0.7.170-dev`) | Target forms, Wave 4; private input route not discharged. |
| AttestationAuthentication B | R event_carried_fields_are_compared_with_independent_literals and bound context subjects (`0.7.153-dev`) | AuthenticatedBurnCarrier, Waves 6/9/10. |
| ValueFlowClosure B/C | M disjoint_regions_preserve_sponsored_controls and substituted_region_and_foreign_payload_refuse (`0.7.186-dev`) | Compiler coverage and target substrate. |
| StateSuccession C | M six-field and state-association subjects (`0.7.183-dev`) | LinkedStateSuccession/generation, Wave 7. |
| FloorNondecrease C | M clear_floor_uses_actual_successor_supply (`0.7.183-dev`) | Direct staged-law evaluation supplies the independent false case; the complete report blocks ClearFloor behind failed assignments and cannot report its failure. Wave 7 retains this distinction alongside TargetFloorNondecrease and authenticated operands. |
| NativeFeeAuction C | Clear public/sponsored controls (`0.7.183-dev`/`0.7.186-dev`), sponsor amounts unread | Economic/fee premise, Waves 7/11/12. |

**Table (Five candidate reopenings)** · `tab:backlog:wave2-reopening-results`

All rows are re-disposed on `0.7.188-dev`; (`rec:backlog:wave2-candidate-dispositions`) retains each old/current premise, control, subject and boundary verbatim. Identity support is the_fixture_pin_is_the_output_key_this_tree_derives (target-elements-conformance reference_oracle_tests.rs), current_archive_replays_sixteen_rows_after_their_paired_controls (vectors compact_corpus.rs) and every_staged_subject_matches_its_pin (V native_staging.rs), not the step tag alone.

| Subject | Evidence | Standing |
|---|---|---|
| Sponsorless emitted-Clear key | V compact_clear_declaration_reopenings_keep_both_keys: actual Ok / MissingClearDeclaration | ObservedDeclarationRefusal retained; no emitted-path credit. |
| Sponsored emitted-Clear key | Same test asserts equal control, subject and refusal | Second key over the same comparison, not another observation. |
| Ruling 133 announced-state premise | V candidate_leaf_and_transition_censuses_have_no_announced_state_spend; realization announce_maturity's two transition refusals | Candidate still lacks announced-STATE spender; declaration dispatch is a different census. |
| 16.6 / wrong-control-recipe | Candidate leaf census plus maturity_mutants.rs no_announcement_byte_carries_the_successor_recipe | Final absent-coordinate standing retained, not native refusal. |
| Conjunct 5 successor-recipe part | Same no-position evidence; no offered successor recipe | RETIRED on this candidate; two observable constructions remain owed. |

**Table (Separate result dimensions)** · `tab:backlog:wave2-result-matrix`

| Dimension | Evidence and present result | Not claimed |
|---|---|---|
| Burn | Complete declaration + model matrix; accepted overclaim remains a Burn with false credit | Target authorization or private-input route. |
| Compact | Existing common adapter preserved; sequence/silence and two-key reopening pass | Emitted Clear exit. |
| Clear | Full/partial, ties, copied fields, clamp and six-field successor comparisons pass | Native successor or migration. Strictly limiting y_l is unreachable in accepted history and compared only by the branch-local function probe. |
| Representation | Explicit-only Burn inputs; permitted private change; public ASH modes declared | Model conformance is Explicit-only; adapters project every object Explicit. Private change/PublicCommitted declaration admission is not model-conformance evidence or target public-commitment implementation. |
| Constructor | Exact declaration dependency/witness census and scope union | Linked constructor equality or complete lifecycle. |
| Event | Separate Burn/Clear comparisons and structural binding | Signature authentication or external report. |
| Accounting | Execution-derived totals, residue and decrement checks; independent model credit | Target substrate; synthetic catalog model provenance. |
| Public construction | Literal public keys/reasons; Authorization refuses protocol signer, Constructibility blocked (217) | Native secrecy, global token uniqueness or economic evidence. |
| Coverage | Per-operation counts/premises and typed unsupported compiler plans | Wave-3 plan/carrier/layout coverage proof. |
| Resources | Frozen gate costs at (`rec:history:wave2-frozen-gate`); inherited resource subjects retained | New Burn carrier size, expanded STATE bytes or speedup. |

**Table (Joint role controls and subjects)** · `tab:backlog:wave2-role-results`

Recounted from M seven_projection_roles_keep_separate_results at `0.7.186-dev`: P = Passed, NA = NotApplicable, NR = NotReached. Credit booleans are the model indexer's own verdict compared with realization, not operation acceptance. Every row is computed by the harness. The last column names typed refusal rather than treating absence as success.

| Control / subject | Operation | Burn event | Record | Compact silence | Clear event | Accounting | STATE |
|---|---|---|---|---|---|---|---|
| Exact-credit Burn | P | P | true | NA | NA | P | NA |
| Overclaim Burn | P | P | false | NA | NA | P | NA |
| Compact | P | NA | NA | P | NA | P | NA |
| Clear | P | NA | NA | NA | P | P | P |
| Changed Burn offer | P | BurnEvent | true | NA | NA | P | NA |
| Duplicate record ordinal | NR | NR | NR | NA | NA | NR | NA |
| Changed Clear offer | P | NA | NA | NA | ClearEvent | P | P |
| Changed accounting offer | P | NA | NA | NA | P | Accounting | P |
| Changed STATE offer | P | NA | NA | NA | P | P | StateSuccession |
| Compact with ClearEvent inserted | ProjectionPolicy failed | NA | NA | CompactSilence | NA | Accounting | NA |

Duplicate ordinal refuses RecordOrdinal at binding. In the final row, Constructibility is blocked by ProjectionPolicy, Representation(Ash) by Constructibility, and both ASH lifecycle exits by Representation; these are not reached failures of their own predicates. Burn has 32 relations / 4 applicable roles / {SubstrateConservation(Burn,Lbtc)}; compact 23 / 3 / {SubstrateConservation(CompactAsh,Lbtc)}; Clear 44 / 4 / {SubstrateConservation(Clear,Lbtc)}. union_scope_preserves_operation_verdicts pins these literal counts/sets against each operation's own declaration and singleton/combined reports. Synthetic-catalog conformance is realization-only evidence and carries no model-established substrate premise.

**Table (Full repository gate join)** · `tab:backlog:wave2-gate-receipts`

These are full repository receipts, not lane-only gates. Earlier receipts remain in backlog history; the frozen log is independently retained at the hub ref named in (`rec:history:wave2-frozen-gate`). Times are UTC terminal success; walls are seconds.

| Bite | Landing / covering gate tag | Green time | Wall |
|---|---|---|---|
| 2.0 | `0.7.147-dev` / `0.7.151-dev` | 2026-11-16T08:41:23Z | 2844 |
| 2.1 | `0.7.151-dev` | 2026-11-16T08:41:23Z | 2844 |
| 2.2 | `0.7.153-dev` / `0.7.154-dev` | 2026-11-16T10:20:03Z | 3280 |
| 2.3a | `0.7.156-dev` / `0.7.157-dev` | 2026-11-16T11:43:37Z | 3589 |
| 2.3b | `0.7.166-dev` | 2026-11-16T19:49:18Z | 4265 |
| 2.4 | `0.7.170-dev` | 2026-11-17T08:41:11Z | 3287 |
| 2.5 | `0.7.176-dev` / `0.7.177-dev` | 2026-11-17T10:17:38Z | 1788 |
| 2.6 | `0.7.180-dev` / `0.7.182-dev` | 2026-11-17T14:05:38Z | 2559 |
| 2.7 | `0.7.183-dev` | 2026-11-17T14:56:37Z | 988 |
| 2.7a | `0.7.185-dev` (also covers `0.7.184-dev`) | 2026-11-17T17:12:01Z | 3106 |
| 2.8 | `0.7.186-dev` | 2026-11-17T17:41:48Z | 950 |
| 2.9 | `0.7.188-dev` (also covers `0.7.187-dev`) | 2026-11-17T18:53:06Z | 1198 |
| 2.10 closing packet import | `0.7.189-dev`, own full repository gate, including plans/labels | 2026-11-18T09:24:05Z | 3218 |

No wave gate item above lacks its bounded evidence. The untouched later-wave obligations are not met by this closure. Ruling 133 remains open for Cycle, Redeem, AdmitDeposits and ReceiptRelabel STATE exits; later realization reopens its premise. An absent candidate position is a census-supported finding with a register label, not a refusal. Shared follow-ups remain choices unless a product defect appears. The frozen draft stays a proposal record, and no accepted ruling is rewritten.

**Closed bites 2.7a and 2.8 receipts (archived)**

The verbatim record, including its labels, is in [closed Wave-2 implementation records](history/backlog-history.md#closed-wave-2-implementation-records).

**Closed Bite 2.1 receipt (archived)**

The verbatim closed record is in [backlog history](history/backlog-history.md#closed-wave-records).

**Closed bites 2.4–2.7 receipts (archived)**

The verbatim record, including its labels, is in [closed Wave-2 implementation records](history/backlog-history.md#closed-wave-2-implementation-records).

**Closed Clear declaration and adapter designs (archived)**

The verbatim record, including its labels, is in [closed Wave-2 implementation records](history/backlog-history.md#closed-wave-2-implementation-records).

### Bite 2.7a — Shared operation-graph union · `step:backlog:wave2-bite-2-7a`

Ruling 213 adds this small packet after the complete Burn and Clear declarations at 2.4/2.6 and before 2.8; 2.7 remains independently owned. The frozen strategy is unchanged. The assembler in packages/realization/src/derive.rs owns cross-operation union: identical lifecycle nodes and edges are shared; operation-independent disclosure facts are shared only when their initial visibility agrees. Conflicting declarations and duplicates within one operation still refuse. Preserve the graph builders' duplicate guards, operation-qualified identities, declaration validation, private-mode admission and public-dependency meaning; no new walker, public type or interface is selected. packages/realization/src/declassification.rs is the disclosure owner for any merge support; packages/realization/src/lifecycle.rs remains read-only because its single-list duplicate check is correct.

The 2.6 review fixes the exact overlap packet: `CompactAsh`, Burn and Clear share four ASH lifecycle nodes and four `RequiresExit` edges; `AnnounceMaturity` and Clear share eight STATE nodes and twelve edges. `BoundValue(AshBatchMax)` is public in `CompactAsh` and Clear, and `BoundValue(FeeSponsorInputMax)` is public in Burn and Clear. Those bound facts have no incoming edge, so joint reasons are unions of standalone reasons; the Burn/Clear scope gives the fee bound Clear's `PermissionlessConstructibility` reason although Burn alone has no seed reaching it. Clear's corrected disclosure rows supply the independent expected unions. After node collisions, lifecycle edge collisions precede duplicate disclosure facts in the current assembler. Complete Clear census can refuse a within-declaration duplicate before a graph guard, so use the selected pilot fixture for graph-guard subjects.

Files, all relative to packages/realization/: src/derive.rs, src/declassification.rs, src/tests/derivation_tests.rs, src/tests/lifecycle_tests.rs, src/tests/disclosure_tests.rs, src/tests/burn_tests.rs, src/tests/clear_tests.rs, README.md and TRACING.md. These are existing receivers by execution, with no new module or registration. Use the existing derivation and graph test harnesses; replace the Burn and Clear limitation pins with acceptance controls at this step, retaining their prior refusal as history.

**Gate (Bite 2.7a packet)** · `gate:backlog:wave2-bite-2-7a`

Retain independent expected unions for overlapping Burn/pilot and Clear/CompactAsh/Burn/AnnounceMaturity scopes and the combined declared scope, alongside unchanged standalone projections. Accepted controls share lifecycle nodes, identical edges and equal-visibility disclosure facts; one-coordinate subjects retain within-operation duplicate and conflicting-visibility refusals. No scope membership discharges the private receipt-to-Burn route. The packet meets (`def:wave2:packet`), its lane gate, independent review and full repository gate before advancement; 2.8 consumes this union and 2.10 joins its receipt with the other packet gates. A required additional owner or changed identity/interface stops for a revised packet.

**Closed union, joint-evidence and candidate designs (archived)**

The verbatim record, including its labels, is in [closed Wave-2 implementation records](history/backlog-history.md#closed-wave-2-implementation-records).

**Table (Selected-design follow-ups)** · `tab:backlog:wave2-design-followups`

These are deferred choices under (`rule:guide15-exec:wave2-settings`), not added landing tests. A discovered product defect still requires disposition; no deferral waives either gate.

| Bite / further case | Status and what it would show |
|---|---|
| Closing review B1 — compiler private exit | DEFER to Wave 3: the reaching subject at (`rec:backlog:wave2-wave3-receiver`) must distinguish Burn scope membership from usable PrivateCommitted receipt exit evidence; no guard is built in this records step. |
| 2.9 review B1 — per-key route literals | DEFER: a literal route for each of the 74 compact manifest keys would make exact route retention a claim of `compact_negative_manifest_matches_every_published_negative` alone; the selected tests currently split route evidence across manifest, ledger and corpus checks. |
| 2.9 review B2 — standing selection | DEFER: adding `no_announcement_byte_carries_the_successor_recipe` and the three candidate-identity pin tests to the focused selection would directly rerun the absent-position and unchanged-candidate support whenever the candidate changes; this bite names their full-gate evidence without enlarging the selected eleven. |
| 2.7a review B1–B6 | DEFER by choice: B1 pins Burn's Explicit-only input SideRepresentation inside Burn+TransferLive, showing required private exit is not admission. B2 binds/analyzes CompactAsh with Clear, AnnounceMaturity with Clear and TransferLive with Burn, comparing standalone compiler outputs except inherited shared-bound reasons; compiler-owned later packet. B3 checks pilot per-declaration lifecycle closure, showing company cannot supply missing rows. B4 pins or guards no seeds on shared bounds and no edges into them, showing reasons cannot cross operations; 2.8 preserved this property but did not add the guard. B5 supplies full-field singleton goldens to catch declaration/shared-builder drift. B6 traces the two assembly DEBUG events with literal post-merge counts under a packet owning tracing tests. None is asserted executed here. |
| 2.8 review B1–B5 | DEFER by choice: B1 compares equal-valued same-base Clears, showing identical normalized common/payload/tokens and no outpoint distinction at this boundary. B2 checks address-level indexer query credit after burn/overclaim/compact/Clear against a literal, beyond per-burn verdicts. B3 follows partial Clear with residual compact and another Clear, showing no extra Burn count in a longer residual chain. B4 retains 2.7a B4's shared-bound edge/seed guard as open, rather than treating empty newly_disclosed as a guard. B5 traces one completion event in the new compact producer's own span under a tracing-path packet. |
| 2.5 — deferred review subjects F1–F4, N1, N2, N5, N7 | DEFER by choice: F1 changes only a well-formed record address, showing binding/conformance/credit still pass while independent event comparison refuses. F2 swaps equal-valued change owners, showing standalone acceptance but bound-projection disagreement. F3 substitutes fresh_ash or removes BurnRecordMax, showing AshBinding or RecordBound at binding rather than an evaluation MissingBoundValue. F4 adds observe_burn traced/untraced equivalence in packages/model/src/tests/tracing_boundary_tests.rs, a separately scoped follow-up. N1 executes a real 9/11 split and binds it to a 10/10 request, showing RequestBindingMismatch and projection disagreement instead of only malformed-history shape. N2 preserves certificate/region tokens in the equal-total split subject, isolating the split coordinate. N5 examines certificate txid/common token rules and same-base executions sharing order/txid, showing the distinction needed at 2.8 without carrying Burn's fresh-ASH token formula into Clear. N7 puts world/owner/change/record/sponsor inputs and independent model records_accepted credit expectations in a data-table harness, making row comparisons explicit; the landed sponsored credit literal already closes that omission. These are further subjects or harness work, not claims of executed tests. |
| 2.6 — broader arithmetic/guard permutations | DEFER to 2.7's independent model conformance: overflow, batch/distinct/lookup precedence and varied six-field successors compare independently accepted model executions rather than duplicate the transition suite here. |
| 2.6 — exhaustive same-length substitutions across every shell collection | DEFER: would detect count-only completeness implementations beyond the selected literal census and reaching copy/public-requirement/exit subjects; no generic mutation framework is selected. |
| 2.6 — sponsor and disjoint-region combinations | DEFER to 2.8: would show public dependencies and foreign-source isolation under combined regions beyond the complete declaration's basic full/partial controls. |
| 2.6 review F1–F10 | DEFER F1–F6 and F8–F10; F7 DONE in the realization README. F1 changes each STATE and ASH object coordinate to show `StateObject` and `AshOutput` weld refusals. F2 changes each remaining operation coordinate to show its exact architecture field. F3 asserts the literal STATE succession effect in residual controls. F4 changes Q, Cycle, Maturity and YT copies through the complete declaration to show each copy refusal. F5 shares one ASH `ObjectSpec` literal across compact, Burn and Clear welds to show the same object expectation without repeated copies. F6 enumerates disclosure edges directly from the plan and frozen graph, making the literal oracle easier to audit. F7 explains that the omega `ClearAssignment` mismatch identity anchors the expression collection even when another field differs. F8 reads declared `RootPolicy` identities in the vectors STATE-declarer pin, making its positive assertion speak directly for declaration dispatch. F9 refuses a RESV root effect beside an accepted residual control. F10 removes `AshBatchMax` through complete evaluation to confirm the input-ASH Cardinality `MissingBoundValue` boundary recorded for 2.7. |
| 2.7 review F1–F5 | DEFER by choice: F1 coherently changes the common Recon leg and extension X to X−1, showing binding can pass while the independent clamp rejects event/accounting and the reached law statuses name the disagreement. F2 uses an admitted announced predecessor with pairwise-distinct fields, nonzero q/cycle and Announced maturity to expose conversion transpositions. F3 drops/replaces an extension ASH source (AshBinding) and independently an event source (ClearEvent), showing source binding at each role. F4 adds MissingClearStateEdge and MissingClearDestruction to traced_variant_names_are_snake_case in packages/model/src/tests/tracing_capture.rs under a packet owning that path. F5 removes the Recon leg or splits X across two U flows, showing the zero-leg and split-destruction boundaries beside an accepted control. |
| 2.7 — broader adapter permutations and tracing | DEFER by choice: randomized accepted histories, exhaustive same-length payload substitutions and a traced/untraced observe_clear subject would broaden coordinate and instrumentation confidence; existing function-equivalence tests remain, and a new tracing-test path needs a revised packet. |
| 2.7 — cross-operation token and region subjects | DEFER to 2.8: independently executed same-base alternatives, foreign-region sources and Burn/compact/Clear composition would show isolation beyond single-operation context binding. No token-only uniqueness or lifecycle claim follows from 2.7. |
| 2.8 — broader composition corpus | DEFER by choice: randomized histories, all operation permutations, many simultaneous regions and longer residual chains would broaden noninterference and scaling evidence beyond the selected finite sequence and disjoint controls. No performance claim is selected. |
| 2.8 — unified token API and tracing expansion | DEFER: a common token encoding/API would simplify cross-operation tooling but requires its own compatibility decision; raw encodings are not compared here. Additional traced/untraced producer subjects would check instrumentation equivalence under a packet owning the tracing test paths. Neither is a landing condition absent a product defect. |
| 2.7a — every subset and randomized orderings | DEFER: would broaden subset/order stability beyond the collision pairs, combined scope and singleton checks, which already exercise the union identity rules. |
| 2.7a — union scaling and disclosure-worklist profiling | DEFER: would measure assembly cost on larger synthetic scopes; disclosure analysis is reused and no performance claim or benchmark harness is selected. |

**Wave-3 receiver** · `rec:backlog:wave2-wave3-receiver`

The guide already adopts (`assum:guide15-exec:wave3-inherited-inputs`); this packet supplies its evidence, not a new assumption or plan proof. Before claiming discharge, Wave 3 must meet these receivers from the union/joint/candidate reviews and (`tab:backlog:wave2-consumers`).

| Receiver | What the consuming packet must establish |
|---|---|
| Private receipt-to-Burn (191) | Closing review B1: add a reaching compiler subject with Burn in scope proving ReceiptLive/PrivateCommitted → Burn is not counted usable, beside the 2.7a B1 realization-side pin. Explicit-only admission does not discharge this route. packages/compiler/src/engine_lifecycle.rs:211–215 chooses AvailableInCompilerScope solely from records.scope.contains(operation); a Burn plan must distinguish scope membership from representation-specific usable exit evidence. |
| Per-operation scope and reasons | Read each operation's own declaration, not a merged row supplied by company. Shared-bound reasons can mix operations; preserve equal-visibility union and explicitly decide any incoming edge/seed on a shared bound. Compiler mixed-scope controls and provisional relation_objects review remain named follow-ups before activation. |
| Joint evidence | Retain (`tab:backlog:wave2-role-results`) and operation-specific premises. Synthetic catalogs remain realization-only; original model evidence stays with its execution. Raw adapter tokens and accidental same-base equality prove no cross-operation identity. |
| Refusal ownership (217) | Use Authorization's UnexpectedProtocolAuthorization and blocked Constructibility for public protocol-signer insertion; propagate this distinction to Waves 5/7/8/11. |
| Candidate changes | Rerun the leaf census, successor-recipe witness, compact tree/archive pins and STATE digest pins named in (`tab:backlog:wave2-reopening-results`). Re-dispose both keys over their actual comparison and all three ruling-133 subjects; labels alone establish no absence. |
| Later construction | Compiler plans, carrier/layout/coverage oracles and ASH constructor closure remain owed. Carrier size/policy at Wave 4 may reopen declarations/plans under ruling 157; native successor, signatures and report production remain their later owners' work. |

**Table (Delivery and entry join)** · `tab:backlog:wave2-join`

| Adopted rows | Producer and acceptance boundary | Retained receiver |
|---|---|---|
| All eight rows of (`tab:wave2:deliverables`) | Producers/closing evidence: 2.1 transition, 2.2 projections, 2.3a/2.3b vocabulary, 2.4/2.6 complete declarations/welds, 2.5/2.7 independent adapters, 2.8 joint evidence. | Each row's missing compiler, target, constructor and report evidence remains later work. |
| All seven rows of (`tab:wave2:preflight`) | Adopt every Wave-2 packet and unchanged/later boundary; preserve historical findings. | Typed-record reports remain Wave 10; assignment/destruction execution remains Wave 7. |
| Every row of (`tab:wave2:obligations`) | Adopt dispositions 1/7/10/12, rulings 133/138/157, consume-pipeline and baseline/gate obligations and producers, with these six ledgers and expanded census. | No model evidence replaces declaration acceptance; no typed projection replaces a report, native evidence or measured carrier. |
| Ruling 138's two missing-clear-exit-emitted keys | Reopen separately at 2.6; 2.9 retains sponsorless and sponsored control/removal subjects, observed boundary and all 74 compact identities. | No emitted Clear credit; later plan/emission closure remains owed. |
| Ruling 133's three subjects | Reopen at 2.6; 2.9 separately disposes announced-state premise, 16.6 / wrong-control-recipe finality and conjunct-5 RETIRED successor recipe. Retain both named candidate/dispatch tests, old/new premises, controls, boundary and standing under (`cond:wave0:baseline-freeze`). | Declaration dispatch is not candidate-leaf admission; no automatic finality or retirement. |
| Witnesses and interfaces | (`tab:realization:burn-clear-witnesses`) and (`tab:realization:burn-clear-interfaces`) replace the draft proposals. | Each external requirement and later receiver remains owed. |

At 2.10 recount the same identities under (`tab:wave2:baseline`): Safety 206/177/29; observed/final/retired/waiting 177/27/2/0; classes 81/12/39/15/16/14; relation links 14/108/84 and reasons 55/9/9/0/11; conjunct parts 10 MET/9 RETIRED/0 WAITING. Keep T11-064/T11-063/T4-009/T11-110 on their own criteria; dispose reopened regressions. Enumerate new declarations, relations, graphs, fields, roles, controls and premises separately. Retain logs/exits/walls/load/own CPU/cgroup scope/deltas and absent dimensions; runner/profiling/tracing follow-ups stay at (`tab:backlog:wave1-followups`).

**Table (Verified bite path lists)** · `tab:backlog:wave2-paths`

The implementation allocation was 84 distinct paths (71 adoption-existing and 13 deliberately new); all 13 new files are now present. Bite 2.10 adds the existing history receiver, giving 85 distinct paths (72 existing + 13 allocated new), with no new module or contract. The historical derivation is retained verbatim in [backlog history](history/backlog-history.md#closed-wave-2-path-count-derivation). Files paragraphs and adopted corrections below remain the exact edit lists; inclusion alone grants no edit authority.

| Bite / source list | Existing / new | New files or earlier creating bite |
|---|---|---|
| (`step:wave2:bite-2-0`) | 7 / 0 | Four governing records, both drafts and plans/drafts/README.md; the last three expand the draft's “these drafts for adoption records”. |
| (`step:wave2:bite-2-1`) | 8 / 1 | `packages/realization/src/tests/clear_state_tests.rs` |
| (`step:wave2:bite-2-2`) | 7 / 3 | `packages/realization/src/burn_clear_observation.rs`; `packages/realization/src/burn_clear_projection.rs`; `packages/realization/src/tests/burn_clear_projection_tests.rs` |
| (`step:wave2:bite-2-3a`) | 6 / 4 | `packages/realization/src/burn_clear_laws.rs`; `packages/realization/src/tests/burn_clear_relation_tests.rs`; plus two created in 2.2. |
| (`step:wave2:bite-2-3b`) | 33 / 4 | Four previously allocated in 2.2/2.3a. Existing paths include the adopted `packages/compiler/src/case.rs` and `packages/compiler/src/tests/case_tests.rs`, plus `packages/vectors/src/keyed_final_manifest/pins.rs`, `packages/vectors/tests/keyed_final/freeze_history.rs` and `plans/packages/vectors.md` for the frozen relation source under (`rule:vectors:keyed-refreeze`); `packages/compiler/src/engine_attestation/constructibility.rs` replaces the StateField cast with the shared state_field_code encoder required by (`tab:realization:burn-clear-vocabulary`); `packages/compiler/src/tests/tracing_boundary_tests.rs` recounts the new typed refusal's tracing name, and `packages/compiler/src/engine_attestation/coverage.rs` propagates a typed mutation refusal instead of hiding it as empty coverage or a panic. |
| (`step:wave2:bite-2-4`) | 14 / 4 | New here: `packages/realization/src/declarations/burn.rs`; `packages/realization/src/tests/burn_tests.rs`. Add `packages/compiler/README.md` for the reachable bind_input refusal; add the 2.3a-created `packages/realization/src/burn_clear_laws.rs` and `packages/realization/src/tests/burn_clear_relation_tests.rs` for the law-to-law edge census and its controls. The draft's other paths remain; details below. |
| (`step:wave2:bite-2-5`) | 7 / 1 | `packages/model/src/tests/burn_realization_conformance_tests.rs` |
| (`step:wave2:bite-2-6`) | 14 / 4 | New here: `packages/realization/src/declarations/clear.rs`; `packages/realization/src/tests/clear_tests.rs`. Add `packages/compiler/README.md` for Clear's reachable refusal and the 2.3a-created `packages/realization/src/burn_clear_laws.rs` and `packages/realization/src/tests/burn_clear_relation_tests.rs` to extend and test the same edge census before Clear dispatch. Add `plans/backlog.md` and `plans/phases/07-burn-and-clear.md` for immediate ruling-133/138 reopening records; accepted rulings and historical standing remain unchanged. |
| (`step:wave2:bite-2-7`) | 8 / 1 | `packages/model/src/tests/clear_realization_conformance_tests.rs` |
| (`step:backlog:wave2-bite-2-7a`) | 7 / 2 | Exact nine-path list above; burn_tests.rs and clear_tests.rs were allocated by 2.4/2.6. Four additional union members, no new allocation. |
| (`step:wave2:bite-2-8`) | 7 / 3 | `packages/model/src/tests/burn_clear_composition_tests.rs`; plus two created in 2.2/2.3a. Add packages/model/src/conformance.rs, src/lib.rs, README.md and TRACING.md to the original six paths for the compact comparison adapter. Exact ten-path list at (`sec:backlog:wave2-joint-design`); all ten now exist. |
| (`step:wave2:bite-2-9`) | 8 / 0 | No additions: the recounted eight existing paths at (`sec:backlog:wave2-candidate-design`) cover the selected harness and records; six code/test owners and two planning records, no new file or registration. |
| (`step:wave2:bite-2-10`) | 5 / 0 | Add existing plans/history/backlog-history.md for verbatim closed packets and the frozen gate record; four original record owners remain. No new file or contract. |

The 2.8 correction adds four existing entries already allocated by 2.5/2.7, not new union members. Its ten paths and all thirteen deliberately new implementation files are now present; the history receiver alone raises the closing allocation to 85.

The 2.3b disposition/reaching-subject packet adds packages/compiler/src/case.rs and packages/compiler/src/tests/case_tests.rs; the row and union now count both. The re-freeze correction adds only the three record paths named in the row: changing the frozen packages/realization/src/relation.rs requires its source digest and retained-capture entry in pins.rs, the freeze-history row, and the plan record beside (`tab:vectors:dependency-refreeze`) in the same reviewed gate-passing landing. The re-pin classification and evidence statement under (`rule:vectors:keyed-refreeze`) is: behaviour change confined to attestation vocabulary; keyed relation types unchanged; reopened: source identity; native observations not reopened provided every keyed byte pin passes unchanged. The shared-encoder correction also adds packages/compiler/src/engine_attestation/constructibility.rs, whose StateField fact-key cast must use the same state_field_code encoder as subject keys under (`tab:realization:burn-clear-vocabulary`). The tracing and coverage correction adds packages/compiler/src/tests/tracing_boundary_tests.rs and packages/compiler/src/engine_attestation/coverage.rs: the former must count the typed UnsupportedOperationPlan tracing name, and the latter must carry the typed refusal from relation_mutations when that function becomes fallible. These corrections name omitted owners, without adding an interface or changing a contract. Checking every expanded Files list for bites 2.4–2.10 against all 35 source paths in the pins.rs freeze inventory finds no other frozen source; no other bite receives a re-freeze path. Other census-only paths retain no-change/later-wave dispositions; a necessary edit to such a path is a required additional path and stops the bite for a revised packet under (`rule:guide15-exec:wave2-settings`). Lexical discovery authorizes no target algorithm, interface or seventh ledger.

**Table (Adopted consumer census and sweep)** · `tab:backlog:wave2-consumers`

The post-2.3b packet correction below applies to (`step:wave2:bite-2-4`) and (`gate:wave2:bite-2-4`), with the explicit Clear extension in 2.6. The strategy and route study remain frozen proposals, including their status tables; this adopted record owns current status and coordinates. Accepted ruling text is unchanged.

| Consumer / decision | Reason and disposition |
|---|---|
| packages/model/src/conformance.rs; src/tests/burn_realization_conformance_tests.rs (paths relative to packages/model/) | ADAPTER 2.5: Bite 2.5 consumer dispositions from the bite-2.4 review, section 9: use the Burn-only realization and evaluate_operation_with_burn_clear; the legacy entry returns MissingExtension and an undeclared bound operation returns OperationOutsideScope. Credit is absent from the evaluation report: compare evaluate_record_credit separately, retaining overclaim as a valid Burn. Changed structural record facts (ordinal, tag, zero amount, count and association) refuse at validate_burn_clear_observation binding. A changed address or amount in a well-formed record can bind; independent event comparison catches either, and accounting also catches changed amounts. Supply BurnInputMax, BurnChangeMax and FeeSponsorInputMax from model parameters or the result is MissingBoundValue. SubstrateConservation is EvidenceRequired: conformance is not evidence completeness, and any model-only discharge keeps its model-only label. Private change amounts come from the execution; an equal-total changed split remains valid standalone and disagrees only at bound-request comparison. Build observations independently from model execution, not realization's private fixture. These dispose consumed code facts under the existing 2.5 packet, adding no acceptance requirement or path. |
| packages/model/src/conformance.rs; src/lib.rs; README.md; TRACING.md | EDIT 2.8: add the independent compact comparison producer beside the common-only adapter, preserving the old API. This narrowly revises the draft's tests-only packet; model acceptance, realization validators and existing token encodings remain unchanged. No compact context producer currently exists; the added owner avoids a test-only substitute for a missing production projection. |
| packages/compiler/README.md | EDIT 2.4, extend at 2.6: the bind_input failure list must name UnsupportedOperationPlan for a declared Burn/Clear requested in compiler scope. Pair the new reaching subject with the existing incomplete-scope refusal; no compiler plan or new error is needed. |
| packages/realization/TRACING.md | EDIT 2.4, already in its path row: replace “Either public evaluation method” with all public evaluation methods, including evaluate_operation_with_burn_clear alongside evaluate_operation and evaluate_operation_with_operator_membership. The same evaluation boundary owns the event; no new per-law trace boundary. |
| packages/realization/src/burn_clear_laws.rs; packages/realization/src/tests/burn_clear_relation_tests.rs | EDIT 2.4: put the law-to-law edge census beside validate_burn_clear_relation_census and consume it from the complete Burn declaration and validator. The current activated_declaration/activated_graph fixture alone states these edges; an evaluator's blocked-versus-error result must not depend on fixture-only wiring. Use existing RelationDependencyDeclaration/StaticRequirement and existing graph validation, not a second graph walker or public interface. Burn input SideRepresentation→BurnBinding→DataOutputs belongs here; complete family/authorization/conservation dependencies remain the declaration weld's responsibility under (`tab:realization:burn-clear-graphs`). Retain independent literal expectations and omission/changed-edge refusals, so sharing construction does not become self-comparison. Extend this same owner and its tests for Clear before 2.6 dispatch, including ClearClamp→ClearCanonicalDeltaPolicy from the accepted activation: sealed/no-progress clamps block the delta; missing bound remains an error. No new relation edge kind or changed law is authorized. |
| packages/realization/README.md; src/declarations/mod.rs; src/tests/derivation_tests.rs (paths relative to packages/realization/) | EDIT 2.4, already scoped: update the declared-operation list, add complete Burn dispatch, replace undeclared_burn_scope_rejects_for_now with supported Burn and an actually unsupported operation. README's pilot-only Burn absence and two-operation example remain correct. Reconcile existing live-transfer private-exit declarations/tests without claiming the direct-private route discharged. |
| packages/vectors/src/maturity_mutants/native_staging.rs | EDIT 2.4, already scoped: add Burn to the non-STATE list at :2866 and supported-operation set at :2988–2992. Preserve the explicit new-body match at :2876–2925. Current dispatch test is realization_operation_census_retains_only_the_announcement_state_transition at :2968–2994 (old :2958–2985); candidate_leaf_and_transition_censuses_have_no_announced_state_spend remains :2793. These are the current coordinates for the strategy's 2.4/2.6/2.9 packet and ruling 189's retained citations. Clear's candidate/dispatch reopening remains 2.6/2.9, not Burn's new STATE claim. |
| packages/compiler/tests/public_api.rs | EDIT 2.4/2.6, already scoped: scope_outside_the_realization_is_incomplete is at :211–230 (old :211–228). Keep the pilot-only refusal and add the complete Burn/Clear reaching refusal beside it. Bite 2.5 needs no change here. |
| packages/realization/src/lib.rs; src/declarations/mod.rs; src/tests/mod.rs; README.md; TRACING.md; meson.build (paths relative to packages/realization/) | SHARED REGISTRATION: S3 registers its sibling owners here under (`tab:backlog:fixed-form-consumers`). Reconcile shared files at landing and retain both owner suites. The existing public burn_clear_laws module makes a new lib.rs export unnecessary for 2.4; do not add that path for speculative registration. Other listed registration paths already belong to 2.4, but are not its exclusive files. |
| packages/realization/src/derive.rs; src/declassification.rs; src/tests/{derivation_tests.rs,lifecycle_tests.rs,disclosure_tests.rs,burn_tests.rs,clear_tests.rs}; README.md; TRACING.md (paths relative to packages/realization/) | EDIT 2.7a: assembler-owned cross-operation union and its existing disclosure owner/tests; complete declarations expose shared lifecycle and disclosure identities. No shared-owner edit at 2.4/2.6. |
| packages/realization/src/{scope.rs,constructibility.rs,lifecycle.rs}; src/tests/{property_graph_tests.rs,tracing_tests.rs,tracing_evaluation_tests.rs} | NO CHANGE: retain scoped controls and generic graph validation. lifecycle.rs keeps its duplicate guard; the assembler unions across declarations only after preserving within-declaration refusals. |
| packages/compiler/src/input.rs; src/engine_attestation.rs; src/tests/analyzed_program_tests.rs (paths relative to packages/compiler/) | NO CHANGE: input revalidates, checks membership, then require_supported_plan refuses Burn/Clear. The analyzed-program Burn refusal deliberately uses phase1_realization and remains IncompleteRealizationScope. Existing selector/forwarder dispositions below retain Wave-3 ownership; dispatch introduces no compiler lowering algorithm. |
| packages/realization/src/relation.rs; packages/compiler/src/engine_attestation/requirement.rs | DEFER: the frozen-source comment and provisional relation_objects review have their follow-up rows at (`tab:backlog:wave1-followups`); neither grants a 2.4 edit. |

The targeted sweep read the 2.4 Files list and searched realization/compiler/vectors Rust and Markdown for Burn/Clear, UnsupportedOperationDeclaration, supported-operation lists, literal len/count assertions, exhaustive matches and bind_input callers. Besides the named additions, it found the README declared-operation list, the already-scoped undeclared-Burn test and the vectors non-STATE list; pilot-only counts and missing-scope callers retain their scopes as disposed above. The graph owner is added to 2.4 now because its first complete declaration consumes those dependencies, rather than leaving a fixture as semantic authority until Clear. This refines the existing graph census, not an interface or contract change. New paths still stop a future bite under (`rule:guide15-exec:wave2-settings`).

Sweep command: `rg -n '\b(Relation|RelationKind|RelationSubject|RelationEdge|ExpressionNode|FactId|StateField|StateMetadata|OperationObservation|ConstructibilityClass|RepresentationMode)\b' packages --glob '*.rs'`: rerun on `0.7.166-dev`, 3,991 reference lines in 187 files (includes constructors/imports, not a semantic-match count; the 2.3a correction had 3,687 lines in the same file count). The broader inventory includes the inspected match/let/matches! sites, Self, local Kind/Subject aliases and wildcards via defining/importing files, notably identity.rs, expression.rs, validate.rs and compiler fold.rs. Each file's disposition covers all sites; inclusion grants no edit authority.

packages/realization/src/declarations/mod.rs now dispatches Burn; Clear remains UnsupportedOperationDeclaration until 2.6. The landed packages/compiler/src/input.rs revalidates the owner and checks membership; public Burn/Clear remain refused at the binding seam. Packet 2.3b installed direct case and source refusals for then-unsupported bodies. Packet 3.2b replaces those subjects with complete internal Burn controls in packages/compiler/src/tests/case_tests.rs and source_tests.rs, including the changed-case census and exact source-table refusal. Case and relation modules remain private. The Burn public reaching subject landed at 2.4; Clear waits for 2.6, while pilot-only missing-scope controls stay unchanged.

Other silent selectors remain in compiler carrier.rs (Cardinality), case.rs (conservation, sponsor, representation), sponsor_region.rs (envelope/open flow), operation_plan.rs and the operation-specific live_transfer_plan.rs and maturity_announcement_plan.rs. Existing-operation selectors supply no Burn/Clear plans. Fact/STATE consumers include realization declassification.rs and expression.rs, compiler disclosure.rs and maturity_announcement_requirements.rs, vectors maturity_report.rs field rendering and compiler engine_attestation/case_facts.rs's ConstructibilityClass selector. No new expression variant was needed: compiler fold.rs remains unchanged. Realization validate.rs/evaluate.rs now own and collect ClearAssignment expressions; engine_attestation/foundation.rs explicitly refuses the unsupported body. Legacy evaluation through derive.rs reaches the missing-extension refusal in evaluate.rs; that evaluation path needs no derive.rs edit, but 2.7a owns its separate cross-operation graph union. The vectors non-STATE list and exhaustive STATE match have their current coordinates and 2.4 disposition above. Preserve these vocabularies with regressions; later plan meaning remains outside this packet.

Dispositions cover all sites, wildcards and tests. REFREEZE: 2.3b paired source pin and retained capture, history, and plan record under (`rule:vectors:keyed-refreeze`); the Markdown plan record is an explicit non-Rust consumer outside the lexical sweep. ACTIVE: 2.3b bodies/keys and unsupported-plan refusals; 2.4/2.6 dispatch pins. CASE: 2.3b case disposition/reaching subject; Wave 3 meaning. ADAPTER: 2.5/2.7 bound observations; raw model unchanged. SILENT: 2.3b direct-entry/refusal disposition; Wave 3 meaning. COMPILER: 2.3b preserves scoped selectors/forwarding with explicit refusal; Wave 3 plans. CONDITIONAL: vocabulary unchanged; 2.3a names expression-change paths before 2.3b. OWNER: 2.1–2.4/2.6 declaration/transition owners preserve constructors. TEST: preserve controls/scopes; 2.3b exhaustive matches, 2.9 candidates. TRACING: 2.3b recounts the compiler's stable error-name census for UnsupportedOperationPlan. COVERAGE: 2.3b propagates relation_mutations' typed refusal through attestation coverage, preserving the refusal boundary instead of constructing empty coverage or panicking. EVIDENCE: 2.3b preserves scoped fields/reports/fixtures; 2.9 reopens premises. DOWNSTREAM: 2.3b preserves observations/STATE/representation/fixtures; target work later. SEPARATE: preserve keyed/shared-engine identities and 2.3b controls, not attestation semantics. SUPPLEMENT: 2.3b reviews unchanged graph/proof/value/side vocabulary, errors/forwarding; input.rs owns admission refusal, declarations/mod.rs 2.4/2.6 dispatch; value.rs depends on expression changes; architecture/frontend homonyms stay unchanged.

| Disposition | Directory prefix | File stems (append stem + .rs unless an extension is shown) |
|---|---|---|
| REFREEZE | `packages/vectors/src/keyed_final_manifest/` | pins |
| REFREEZE | `packages/vectors/tests/keyed_final/` | freeze_history |
| REFREEZE | `plans/packages/` | vectors.md |
| ACTIVE | `packages/compiler/src/` | coverage, engine_attestation, layout, placement |
| ACTIVE | `packages/compiler/src/engine_attestation/` | constructibility, foundation, requirement, source |
| ACTIVE | `packages/realization/src/` | evaluate, expression, identity, relation, validate |
| ACTIVE | `packages/vectors/src/` | maturity_safety |
| ACTIVE | `packages/vectors/src/maturity_mutants/` | native_staging |
| COVERAGE | `packages/compiler/src/engine_attestation/` | coverage |
| ADAPTER | `packages/model/src/` | conformance |
| CASE | `packages/compiler/src/` | case |
| COMPILER | `packages/compiler/src/` | carrier, coverage_graph, error, expression, lifecycle, live_transfer_plan, maturity_announcement_plan, maturity_announcement_requirements, operation_plan, proof, relation, requirement, source, sponsor_region |
| COMPILER | `packages/compiler/src/engine_attestation/` | identity, operation, placement |
| DOWNSTREAM | `packages/linker/src/` | carrier, live_carrier, state_bundle |
| DOWNSTREAM | `packages/tapscript/src/` | bundle, maturity_assessment, pattern, state_announcement, state_constructor, upstream |
| DOWNSTREAM | `packages/target-elements-conformance/src/bin/` | check-target-elements-prototypes |
| DOWNSTREAM | `packages/target-elements/src/` | transaction_form |
| DOWNSTREAM | `packages/transaction/src/` | state_abi, state_construct, state_signing, state_view |
| EVIDENCE | `packages/vectors/src/` | compact_campaign, compact_campaign_second, first_party, live_safety, maturity_branch, maturity_closure, maturity_continuity, maturity_continuity_report, maturity_evidence, maturity_first_party, maturity_fixture, maturity_native, maturity_nonce_corpus, maturity_operator, maturity_recovery, maturity_recovery_report, maturity_refinement, maturity_report, plan, violation |
| OWNER | `packages/realization/src/` | domain, error, lib, lifecycle, observation, state, state_codec |
| OWNER | `packages/realization/src/declarations/` | announce_maturity, compact_ash, transfer_live |
| SEPARATE | `packages/compiler/src/` | engine_coverage_graph, engine_coverage_join, engine_foundation, engine_requirement, keyed_plan |
| SEPARATE | `packages/compiler/src/engine_keyed/program/` | refusal_tests, restore |
| SEPARATE | `packages/compiler/src/tests/` | engine_coverage_graph_tests, engine_foundation_tests, engine_keyed_tests |
| SEPARATE | `packages/realization/src/tests/` | keyed_profile_tests |
| SEPARATE | `packages/vectors/src/` | keyed_history_reader |
| SEPARATE | `packages/vectors/src/keyed_evidence/` | families, family_execution, family_resources, family_types |
| SEPARATE | `packages/vectors/src/keyed_final_corpus/` | history |
| SEPARATE | `packages/vectors/src/keyed_history_reader/` | authentication, evidence |
| SEPARATE | `packages/vectors/tests/keyed_final_capture/` | cost |
| SEPARATE | `packages/vectors/tests/keyed_history/` | families, reader, reader_review |
| SILENT | `packages/compiler/src/engine_attestation/` | lifecycle, proof |
| TRACING | `packages/compiler/src/tests/` | tracing_boundary_tests |
| SUPPLEMENT | `packages/architecture/src/` | ids, validate |
| SUPPLEMENT | `packages/architecture/src/tests/` | validation_tests |
| SUPPLEMENT | `packages/compiler/src/` | analyzed, analyzed_operation, analyzed_validate, constructibility, engine_cases, engine_identity, engine_keyed, engine_records, foundation, input, instrumentation, lib, target |
| SUPPLEMENT | `packages/compiler/src/engine_attestation/` | cases, coverage_join, program |
| SUPPLEMENT | `packages/compiler/src/engine_keyed/` | program |
| SUPPLEMENT | `packages/compiler/src/engine_keyed/program/` | assignment_tests, tests |
| SUPPLEMENT | `packages/compiler/src/tests/` | engine_cases_tests, engine_program_tests, engine_proof_tests, guide13_reproductions, target_tests, tracing_capture |
| SUPPLEMENT | `packages/compiler/tests/` | keyed_plan |
| SUPPLEMENT | `packages/flatten-latex-main/src/tests/` | mod |
| SUPPLEMENT | `packages/linker/src/` | backend, bundle, state_carrier, state_error |
| SUPPLEMENT | `packages/linker/src/tests/` | tracing_tests |
| SUPPLEMENT | `packages/model/src/` | lib, manifest |
| SUPPLEMENT | `packages/model/src/tests/` | tracing_boundary_tests, tracing_capture |
| SUPPLEMENT | `packages/realization/src/` | binding, scope, value |
| SUPPLEMENT | `packages/realization/src/declarations/` | keyed, mod |
| SUPPLEMENT | `packages/realization/src/tests/` | mod, tracing_evaluation_tests, tracing_tests |
| SUPPLEMENT | `packages/tapscript/src/` | keyed_bundle, lib |
| SUPPLEMENT | `packages/tapscript/src/keyed_bundle/` | tests |
| SUPPLEMENT | `packages/target-elements-conformance/src/` | declassification |
| SUPPLEMENT | `packages/transaction/src/` | abi, live_request, operator_signing |
| SUPPLEMENT | `packages/vectors/src/` | compact_corpus, error, fixture, keyed_cost, live_evidence, live_minimality_report, maturity_mutants, projection, report |
| TEST | `packages/compiler/src/tests/` | analyzed_corruption_tests, analyzed_operation_tests, analyzed_program_oracle_tests, analyzed_program_tests, carrier_tests, case_tests, compact_validation_tests, constructibility_tests, coverage_graph_tests, coverage_oracle_tests, coverage_tests, disclosure_tests, expression_tests, fold_tests, layout_tests, lifecycle_tests, live_transfer_plan_tests, maturity_announcement_corruption_tests, maturity_announcement_oracle_tests, maturity_announcement_plan_tests, maturity_announcement_requirements_tests, operation_plan_tests, oracle_tests, pilot_coverage_tests, pilot_placement_tests, pilot_program_tests, placement_oracle_tests, placement_tests, proof_tests, relation_tests, requirement_tests, source_tests |
| TEST | `packages/compiler/tests/` | public_api |
| TEST | `packages/linker/src/tests/` | carrier_tests, mod, state_bundle_tests, state_carrier_tests, state_taptree_tests |
| TEST | `packages/model/src/tests/` | realization_conformance_tests, state_metadata_conformance_tests |
| TEST | `packages/realization/src/tests/` | announce_maturity_tests, compact_ash_tests, constructibility_tests, derivation_tests, disclosure_tests, expression_tests, guide11_reproductions, lifecycle_tests, live_transfer_tests, open_flow_tests, operator_vocabulary_tests, relation_identity_tests, root_policy_tests, state_codec_tests, state_tests, tracing_refusal_tests, type_tests |
| TEST | `packages/realization/tests/` | public_api |
| TEST | `packages/tapscript/src/tests/` | abstract_oracle_tests, compact_validation_tests, maturity_assessment_tests, state_announcement_tests, state_constructor_tests |
| TEST | `packages/target-elements-conformance/src/tests/` | constructor_tests |
| TEST | `packages/target-elements/src/tests/` | transaction_form_tests |
| TEST | `packages/transaction/src/tests/` | state_abi_validation_tests, state_construct_tests, state_support, state_view_tests |
| TEST | `packages/vectors/tests/` | maturity_operator |
| CONDITIONAL | `packages/compiler/src/` | disclosure, fold |
| CONDITIONAL | `packages/compiler/src/engine_attestation/` | case_facts, constructibility |
| CONDITIONAL | `packages/realization/src/` | constructibility, declassification, derive |

Supplement: `rg -n '\b(ExprId|ExpressionRole|SemanticType|ExpressionValue|ProtocolAmount|Count|ObservedSide|TransactionSide|ObservedObjectRef|ObservedObjectKind|ObservedValue|ObservedRootEffectKind|ConformanceProjectionError|RealizationError|CompileError|ExternalEvidenceRequirement|RelationStatus|RelationFailure|ProofKind|ProofAlternativeId|AvailabilityClass|ConstructibilityNodeId|ConstructibilityEdgeRole|RequirementStrength|WitnessRole|LifecycleNodeId|LifecycleEdge|DisclosureNodeId|DisclosureEdge|DisclosureReason)\b' packages --glob '*.rs'`: 5,377 reference lines in 210 files, 145 shared and 65 additional, hence 245 distinct files. Shared files inherit their primary disposition. Error/diagnostic/tracing Self matches stay with scoped error.rs/conformance.rs owners. Graph/proof/side/value/external-evidence members stay unchanged; expression additions require 2.3a activation.

SUPPLEMENT lists the 65 additional files; the 145 shared files retain primary dispositions.

The implementation packet has 75 paths (original 71 plus two case paths and two compiler tracing/coverage consumers): 62 existing, 13 new. Census preservation/later-receiver dispositions authorize no downstream edits. Clamp-driven changes to value.rs, fold.rs or other conditional consumers stop 2.3a for a revised exact path packet before 2.3b.

Six ledgers cover the lexical superset, separate keyed/engine identities and unchanged downstream constructors/common observations, without migration. Test public and in-crate case entries at their actual boundaries; wildcard fallthrough/planned guards prove no refusal. Bite 2.3b atomically integrates kind/body/subject matches with injectivity/old-code controls. New interfaces, undefined laws, lifecycle-contract changes or unsupported ruling evidence stop the owning packet.

Divergences are recorded at (`tab:drafts:index`): expanded consumers, admission/case distinctions, two added 2.3b paths, exact identities/codes, witness/graph expansions and interface clarifications. Draft proposals and historical references stay untouched. Adoption supplies no code result, target evidence or completed Wave-2 gate.


### Wave-3 adoption and execution · `rec:backlog:wave3-adoption`

Packet 3.0 adopts rulings 221–234 at (`rule:phase7:wave3-rulings`) from the corrected drafts on `0.7.193-dev`. It opens on the closed Wave-2 receiver (`rec:backlog:wave2-wave3-receiver`), preserving its evidence limits. The two pre-code readings are confirmed at (`rec:w3:precode`); no compiler implementation or wave gate is credited here.

Implementation reads the live packet order/file-package limits at (`tab:guide15-exec:wave3-bites`), exact paths at (`tab:guide15-exec:wave3-paths`), selected tests at (`sec:guide15-exec:wave3-execution`), receiver accounting at (`tab:guide15-exec:wave3-receivers`) and settings at (`rule:guide15-exec:wave3-settings`). Packet 3.5c follows vocabulary before actual Burn lowering. Detailed completed receipts append to plans/history/wave3-compiler-records.md; current contracts remain (`sec:compiler:wave3-plans`) and (`sec:realization:wave3-admission`).

Packet 3.1a's operation-local choice and input-admission work is recorded at (`rec:w3:packet-3-1a`); its node gate is green, while independent review and the full import gate remain pending. Packet 3.1b's shared-bound guard is recorded at (`rec:w3:packet-3-1b`); its node gate is green, while independent review and the full import gate remain pending. Packet 3.2a's vocabulary is recorded at (`rec:w3:3-2a`); its node gate is green, while independent review and the full import gate remain pending. Packet 3.5c's qualified keys and side projections are recorded at (`rec:w3:3-5c`); its original code gate is green and the carried-form gate is pending. Packet 3.2b's Burn lowering and report boundary are recorded at (`rec:w3:3-2b`); its node gate is green, while independent review and the full import gate remain pending. The frozen wave gate, full baseline comparison and Wave-4 handoff remain outstanding.

Packet 3.2c's dormant Clear body, quantity, carrier, layout and four report-anchor tables are recorded at (`rec:w3:3-2c`). The three selected tests and records-inclusive node gate passed; the independent full import gate remains pending. Public Clear still refuses binding.

Clear facade packet 3.5a stopped at the complete-placement candidate cap; its staged facade is not validated evidence. Adopted rulings 238–243 (`rule:phase7:clear-placement`) require prerequisite packets 3.2d/3.2e at (`sec:guide15-exec:wave3-placement`), preserving exact choices as factors with active-plan equivalence and the review at (`rule:phase7:wave3-review-placement`) before resumption. The code recount and staged-work disposition are (`sec:clear-placement:study`). Adoption supplies no implementation or activation evidence.

Packet 3.2d — CODE GREEN, IMPORT PENDING (`rec:w3:3-2d`). The placement owner has a checked canonical factor product, single-member validation and explicitly bounded expansion, exercised by two literal controls. The old enumerator and every production route remain unchanged; Clear stays refused. Compiler gate and preservation details are in the receipt. The full import gate remains pending.

Packet 3.2e — CODE GATE GREEN, JOINT REVIEW PENDING (`rec:w3:3-2e`). Ruling 244 restores ClearFloor's runtime AmountMismatch negative without changing the shared census. Clear's checked factors validate completely; active and keyed complete members and public facade figures are preserved. The granted exact-path amendment updates the tracing-kind census for the packet's one typed error. Joint independent review and full import gate remain pending; public Clear remains refused.

Packet 3.4's compact revalidation is recorded at (`rec:w3:3-4`): unchanged compact, linker, vectors and keyed pins passed the node gate; its event-silence corruption is refused. The full import gate remains pending.

Packet 3.3a's dormant explicit Burn facade, independent plan validator and selected corruptions are recorded at (`rec:w3:3-3a`). Public Burn binding remains refused; the unfiltered node code gate passed after one correction round. The final records-tree gate and full import gate remain pending. The mixed private-change alternative and public activation belong to later packets.

Packet 3.3b — CODE GREEN, IMPORT PENDING (`rec:w3:3-3b`). A source-based finite Burn oracle preceded the public guard change. Explicit Burn now binds and plans publicly; Clear retains its input guard. The TransferLive+Burn control publishes Explicit Burn as implemented, PrivateCommitted Burn as unavailable, and Redeem as outstanding; a changed common closure is refused. The scope-derived live contract was already implemented by 3.3a's correction and needed no further production edit here. Compiler node code gate passed after one lint correction; its records-tree round and independent review/full import gate remain pending. The mixed Burn alternative stays with 3.5d–f.

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
| Phase 7 | Active | Waves 0 and 1 are closed at (`rec:backlog:wave0-baseline`) and (`rec:w1b16:wave1-exit`). The independently reviewed corrected packet `0.7.142-dev` and its full repository gate support the separate bite-1.6 closure (`rec:w1b16:closed`) and ruling 181 (`rule:phase7:wave1-exit-ruling`). The result is one parameterized realization with conditional authentication and per-instance evidence, observed under two representative keys; commitment may succeed or return its typed refusal, and new keys require no template review or key-specific campaign under unchanged conditions. Funding's final-byte acceptance condition is met; the reader rule's deployment condition remains outside the closure scope. Earlier bite closures remain indexed at (`tab:backlog:wave1-exit`); historical probe refusals and provisional measurements retain their own meanings. Open receivers are (`tab:w1b16:exit-receivers`). Wave 2 — typed burn and clear realization — has its bounded execution contract adopted under (`task:guide15-exec:wave2`), while the remaining Phase-7 deliverables and phase exit stay open. |

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

### 5.10 Phase-5 closing-arc adversarial review (archived)

The completed review and remediation register moved verbatim to [backlog history](history/backlog-history.md#phase-5-closing-review). Its label and row identifiers resolve there.

### Guide-11 execution wave records (archived)

The three Guide-11 execution wave records — CT fixtures and the
conservation matrix, candidate dispositions and normalization, and the
fresh-process lifecycle — are archived verbatim, with their numbering
and their labels, in [backlog history](history/backlog-history.md);
a reference to any of their task labels resolves there. All three
closed DONE at the Guide-11 gate (section 2.4).

### T6 — Validate capabilities for external-evidence obligations

**Priority:** P1
**Status:** DONE

The completed repair narrative moved verbatim to [backlog history](history/backlog-history.md#closed-t6-t9-repair-records).

### T7 — Constrain every representation-sensitive proof choice

**Priority:** P1
**Status:** DONE

The completed repair narrative moved verbatim to [backlog history](history/backlog-history.md#closed-t6-t9-repair-records).

### T8 — Weld every current-phase declaration

**Priority:** P2
**Status:** DONE

The completed repair narrative moved verbatim to [backlog history](history/backlog-history.md#closed-t6-t9-repair-records).

### T9 — Refresh compiler status documentation

**Priority:** P2
**Status:** DONE

The completed repair narrative moved verbatim to [backlog history](history/backlog-history.md#closed-t6-t9-repair-records).

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

### 6.2 Completed foundation (archived)

The verbatim closed record is in [backlog history](history/backlog-history.md#closed-wave-records).

### P2-007 — Repair and reaccept exact proof planning

**Status:** DONE

The verbatim closed record is in [backlog history](history/backlog-history.md#closed-wave-records).

### P2-010 — Derive execution-case placement and layout requirements

**Status:** DONE

The verbatim closed record is in [backlog history](history/backlog-history.md#closed-wave-records).

### P2-011 — Derive relation-indexed coverage requirements

**Status:** DONE

The verbatim closed record is in [backlog history](history/backlog-history.md#closed-wave-records).

### P2-012 — Analyze both pilots end to end

**Status:** DONE

The verbatim closed record is in [backlog history](history/backlog-history.md#closed-wave-records).

### P2-013 — Phase-2 evidence and exit

**Status:** DONE

The verbatim closed record is in [backlog history](history/backlog-history.md#closed-wave-records).

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

### 7.2 C1-008 — Proof-plan search reacceptance (archived)

The verbatim closed record is in [backlog history](history/backlog-history.md#closed-wave-records).

## 8. Dependency posture · `sec:backlog:dependencies`

### 8.1 Current and deferred dependencies · `tab:backlog:dependencies`

| Dependency | Class | Status | Role |
|---|---|---|---|
| `petgraph = 0.8.3` | Pinned | Adopted and reviewed; `serde-1` only | Graph storage and standard algorithms |
| `num-bigint = 0.5` | Tracked | Existing | Exact arbitrary-size integers |
| `num-integer` | Tracked | Existing | Exact integer helpers |
| `num-traits` | Tracked | Existing | Numeric traits |
| `proptest` | Tracked | Existing | Property and independent-oracle testing |
| `sha2 = 0.11` | Tracked | Existing | tagged SHA-256 for taproot leaf, branch and tweak commitments |
| `toml = 1` | Tracked | Existing | Architecture TOML publication rendering and parsing |
| `lzma-rust2 = 0.25.0` | Tracked | Reviewed; decoder-only `std`, no default features | Test-only bounded scaling archive admission; (`rec:backlog:scaling-decoder-review`) |
| `num-rational` | — | Deferred | Future exact-rational consumer |
| `faer` | — | Deferred | Future certified numerical diagnostics |
| `fixedbitset` | — | Deferred | Dense local coverage sets if measured |
| Elements libraries (`elements = 0.27.0`, `secp256k1-zkp = 0.11.2`) | Slow | Phase-3 review | Target transaction and consensus types |
| `bitcoin_hashes = 0.14.101` | Slow | Existing; reviewed at (`rec:w1b14:construct`) | Production sponsor HASH160 on the 0.14 copy that `bitcoin`, `secp256k1` and `base58ck` resolve; `elements` resolves its own copy on the 1.1 line |
| SAT/LP/MILP solver | — | Deferred | Larger exact planning only after measured need |
| `salsa` | — | Deferred | Incremental compiler queries |
| `egg` | — | Deferred | Equality saturation |
| `rayon` | — | Removed and deferred | Requires measured need and schedule-independent results |

The other workspace entries are in the tracked class and are not listed here: each names one release line and resolves within it as that class states; whether each line is still the newest is the wave-boundary re-measurement of (`rule:backlog:dependency-currency`).

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

### 8.3 Dependency-currency rule · `rule:backlog:dependency-currency`

An admitted dependency stays on a current release line, because the floating resolution of (`[ADR011-rule:toolchain:locked]`) only surfaces an outgrown dependency when the manifest names the dependency's current release line. Every workspace entry belongs to one of three classes.

1. **Tracked.** The requirement names the dependency's current release line, which is its major version, or its minor version while the major is 0, and resolves to the newest release of that line that the declared Rust floor admits: under `resolver = "3"` Cargo prefers such a release and takes a newer one only when the line has none. When a newer line is published, the next maintenance step moves the workspace to it and adapts the call sites. A bare `"0"` is not a line: under a zero major each minor is a breaking line, so such a requirement does not say which API the code was written against.
2. **Slow.** `elements`, `secp256k1-zkp` and the Rust Bitcoin crates they resolve, `bitcoin_hashes` among them, change only together with a renewed review of record, because the target conformance evidence was examined against exactly these versions and an upgrade reopens that examination. `bitcoin_hashes` is held on its 0.14 line because the resolved graph carries two copies: `bitcoin`, `secp256k1` and `base58ck` resolve the 0.14 copy, and the production sponsor HASH160 shares that copy with them, as (`rec:w1b14:construct`) records; `elements` requires the 1.1 line and resolves its own copy, which no first-party edge names. Moving the direct edge would compute the sponsor HASH160 with the copy `elements` uses, so it moves only with those crates, under a renewed review.
3. **Pinned.** Any other entry that is not on its newest line, or that is pinned exactly, carries a comment beside its line in `Cargo.toml` naming the concrete reason and what would lift it. A new line that needs a newer compiler than the declared floor is pinned for that reason unless the floor is raised deliberately, because (`[ADR011-rule:toolchain:msrv]`) does not let a dependency update raise it incidentally.

A move to a new line keeps the review a reviewed entry carries: the entry's record is renewed against the new release in the same change as its range, as (`rec:backlog:scaling-decoder-review`) requires of its own range. A move that would change a pinned digest, fixture, generated publication or census count is a finding about the upgrade and stops that move. A move that brings a crate into the resolved graph names it with its licence in the move's record, because (`[ADR011-rule:toolchain:licenses]`) admits only permissive licences into the first-party build graph and leaves that check to dependency review.

At each wave boundary every entry is re-measured: the registry's newest stable release against the workspace line and the node's resolution. A tracked entry behind its newest line becomes the next maintenance step, and a slow or pinned entry is checked against the reason its comment states.

The first moves under this rule (`lzma-rust2` 0.25, `num-bigint` 0.5, `toml` 1, `sha2` 0.11) brought these crates into the node's resolution: `block-buffer` 0.12.1, `crypto-common` 0.2.2, `digest` 0.11.3, `hybrid-array` 0.4.15, `num-bigint` 0.5.1, `serde_spanned` 1.1.3, `sha2` 0.11.1, `toml` 1.2.1+spec-1.1.0 and `toml_writer` 1.2.0+spec-1.1.0 under MIT OR Apache-2.0, `const-oid` 0.10.3 under Apache-2.0 OR MIT, and `lzma-rust2` 0.25.0 under Apache-2.0; and removed: `block-buffer` 0.10.4, `cpufeatures` 0.2.17, `crypto-common` 0.1.7, `digest` 0.10.7, `generic-array` 0.14.7, `lzma-rust2` 0.24.0, `num-bigint` 0.4.8, `serde_spanned` 0.6.9, `sha2` 0.10.9, `toml` 0.8.23, `toml_datetime` 0.6.11, `toml_edit` 0.22.27, `toml_write` 0.1.2, `version_check` 0.9.5 and `winnow` 0.7.15. Each entering licence is permissive. The entering crates declare Rust versions from 1.60 to 1.88, with `sha2` 0.11.1 at exactly 1.88 and every other at most 1.85, all within the workspace's 1.88 floor; this was checked from declared metadata, not by an additional compiler run at that floor, since no gate lane builds with Rust 1.88.

| Item | Status | Required next evidence |
|---|---|---|
| Dependency-currency re-measurement | TODO | At each wave boundary, the newest stable registry release, the workspace line and the node's resolution for every workspace entry, with each tracked entry behind its line moved or pinned with its reason. The row returns to TODO after each boundary's measurement is recorded. |

### Scaling archive decoder review · `rec:backlog:scaling-decoder-review`

Purpose and home: `packages/vectors/src/keyed_scaling_corpus.rs` is a test-only admission reader for a 164,375,392-byte native capture retained as a compressed original. `lzma-rust2` is declared in workspace dependencies and inherited only by vectors' dev-dependencies; the module is under `cfg(test)`, so neither the codec nor its types enter the public library API. A maintained LZMA implementation is a substantial algorithm and is not clearer as new first-party code. The independent encoder is XZ Utils 5.8.2 on kranich, with `xz --format=lzma --threads=1 -6 -c`; LZMA-Alone carries no timestamp or filename. The independently pinned original SHA-256 is the round-trip oracle. Capture framing and wire JSON are unchanged.

Source and maintained status: the reviewed crates.io release is `lzma-rust2` 0.25.0 from [hasenbanck/lzma-rust2](https://github.com/hasenbanck/lzma-rust2), a maintained continuation of lzma-rust; its [release history](https://github.com/hasenbanck/lzma-rust2/blob/master/CHANGELOG.md) records ongoing decoder hardening and bounded worker fixes. The published crate's `Cargo.toml`, `src/lib.rs` and `src/lzma_reader.rs` were read, rather than relying on its description. The Cargo range is `0.25.0`, allowing patch updates within that minor under (`[ADR011-rule:toolchain:locked]`); no lockfile is committed. Future feature or range changes require renewed review.

Renewal from 0.24.0 to 0.25.0: the two published crates were compared file by file. The manifest differs only in its version: features, the optional SHA-2 dependency of the disabled XZ feature, Rust 1.85 and edition 2024 are as reviewed, and `forbid(unsafe_code)` still applies without `optimization`. The LZ decoder, range decoder and LZMA decoder modules are unchanged. In `src/lzma_reader.rs` the `std` build of `LzmaReader`, including `new_mem_limit` and its memory check before allocation, `set_relaxed_end_condition` and `read`, is unchanged; the `no_std` refill now retries an interrupted read, and the push-style `LzmaStream`, which this module does not call, gains `memory_usage`, `mem_limit` and `set_mem_limit` and no longer fails permanently on a memory-limit error. The remaining changes are in the XZ and lzip paths, which are disabled, and in the LZMA2 reader and stream, which this module does not call. The call sites, bounds and pinned digests are unchanged, so the admission evidence below carries over. Advisory review on 2026-11-15: a search of the RustSec advisory database found no lzma-rust2 entry at any version, so no advisory affects 0.25.0; the gate's cargo-audit lane remains the check of the resolved graph.

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

The Phase-7 task state retains the Guide-15 import and Wave-0 closures below; (`tab:backlog:wave1-exit`) records the completed second-contract wave and its bite evidence. Wave 2's execution contract is adopted at (`rec:backlog:wave2-closure`); no compiler plan or implementation is supplied by adoption.

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

**Record (Wave 0 region refit, archived)**

Moved verbatim to [backlog history](history/backlog-history.md#wave-0-closed-evidence-records).

**Record (Wave 0 closure-exact audit, archived)**

Moved verbatim to [backlog history](history/backlog-history.md#wave-0-closed-evidence-records).

**Record (Wave 0 closure elision, archived)**

Moved verbatim to [backlog history](history/backlog-history.md#wave-0-closed-evidence-records).

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

### Sponsored construction (archived)

The verbatim closed record is in [backlog history](history/backlog-history.md#closed-wave-records).

### Sponsored observation (archived)

The verbatim closed record is in [backlog history](history/backlog-history.md#closed-wave-records).

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

### Native observation (archived)

The verbatim closed record is in [backlog history](history/backlog-history.md#closed-wave-records).

### Independent observation (archived)

The verbatim closed record is in [backlog history](history/backlog-history.md#closed-wave-records).

### Report and relations (archived)

The verbatim closed record is in [backlog history](history/backlog-history.md#closed-wave-records).

### Relation dispositions (archived)

The verbatim closed record is in [backlog history](history/backlog-history.md#closed-wave-records).

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

### ABI-entry Safety comparisons (archived)

The verbatim closed record is in [backlog history](history/backlog-history.md#closed-wave-records).

### Combined projection (archived)

The verbatim closed record is in [backlog history](history/backlog-history.md#closed-wave-records).

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

### Native route construction (archived)

The verbatim closed record is in [backlog history](history/backlog-history.md#closed-wave-records).

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

**Closed **Record (Confidential sponsor capture driver) (archived)**

The verbatim closed record is in [backlog history](history/backlog-history.md#closed-wave-records).

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

### Adapter request comparisons · archived record

The verbatim closed record is in [backlog history](history/backlog-history.md#closed-wave-records).

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

### Test performance · archived record

The verbatim closed record is in [backlog history](history/backlog-history.md#closed-wave-records).

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

**Record (Wave 0 preflight semantic census, archived)**

The completed packet, including its carrier and Clear-schedule tables, is retained at [the historical preflight semantic census](history/wave0-preflight-semantic-census.md). Current execution standing remains below and at (`rec:backlog:wave2-closure`).

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

Further closed Phase-6 rows moved unchanged to [later Phase-6 task history](history/backlog-history.md#later-phase-6-closed-task-rows). Rows still read by open work, rulings or package documentation remain below.

| ID | Status | Task |
|---|---|---|
| `T11-049` | DONE | The plans-weight sweep landed through `0.6.187-dev`: the 103 closed Phase-6 rows of section 11, 141,507 bytes, moved verbatim with their identifiers into [backlog history](history/backlog-history.md#phase-6-closed-task-rows) under the Phase-6 closed task rows group behind a 233-byte pointer paragraph, and the maintained-prose soft target rose from 1,062,912 to 1,228,800 bytes. The maintained tree measures 883,525 bytes where it measured 1,024,799 before the move, leaving 345,275 bytes of advisory room under an unchanged 1,572,864-byte hard cap, and the move carried the preservation audit that the live-only task-status and duplicate-row checks cannot supply. Ruling 42 at (`rule:phase6:wave6-rulings`) states the decision and finding 45 at (`sec:phase6:wave6-findings`) records its arithmetic. |
| `T11-062` | DONE | Extension-point notes at the four sites where a second constructor generation would enter. The guide records migration as outstanding and states that no migration between static subtrees is implemented, so the code points that embody that rule say so where a reader would otherwise reach for the wrong mechanism. DONE criterion, checkable in the diff: (1) `StateConstructorGeneration` in `packages/tapscript/src/state_constructor.rs` carries a doc paragraph stating that a generation is a distinct recipe identity rather than a metadata change and that the census is single because of the continuity rule, citing the guide's outstanding migration status by label; (2) `ReferenceClass` in `packages/linker/src/graph.rs`, and the STATE graph's class enum once ruling 45 lands, carries a doc paragraph stating that a reference to a previously linked bundle is a backward, acyclic edge — a resolution class and not a cut — so that `AuthenticatedWitnessedRootContinuity` is not mistaken for it; (3) the §17.3 continuity equality is one named function with a doc paragraph saying it is the no-migration rule and the single assertion a migration relation would replace; (4) `MetadataAuthentication` in `packages/tapscript/src/state_announcement.rs` carries a doc paragraph stating that, recognition naming no predecessor under ruling 41, generation support would enter through this component's authentication against the prior generation's recipe. Each note states the current shape's reason and cites the guide, and none sketches a generations design. Carried by `T11-052` for sites 1 and 4, `T11-055` for site 2 and `T11-059` for site 3; if any site's bite slips the row stays open into the next wave, so the Phase-6 closure cannot close over it. Sites 1 and 4's reason paragraphs landed as `0.6.189-dev`, site 2's STATE half on the binding-time enum as `0.6.193-dev` and site 3's continuity note as `0.6.196-dev`; this step lands the two the closure record found missing, `ReferenceClass`'s paragraph in `packages/linker/src/graph.rs` and the §17.3 citation in `MetadataAuthentication`'s, so all four sites state the current shape's reason and cite the guide. |
| `T11-063` | DONE | Audit the compact and live generations against the closure-exact principle. Their region counts are required for closure on their own terms — compact ASH aggregates over a fixed batch, and a live transfer admits split, merge and redistribution, so its output count is a second free axis its multi-destination logic reads — while their sponsor-region checks may not be (`[ADR024-rem:closure:consequences]`). Gate: each retained check names what closure needs it for, and each surplus check is removed with its census recount. Scheduled after the Phase-6 closure unless a Wave-6 link depends on it. Classification is recorded at (`rec:backlog:wave0-closure-audit`); `T12-003` retains destination elision and its byte-dependent revalidation, with regional count and issuance residuals named. Elision and byte-dependent revalidation are closed by `T12-003` and (`rec:backlog:wave0-closure-elision`), with the named regional residuals retaining their scope. |
| `T11-064` | DONE | The region-scoping refit of the realization under ruling 52: an operation's transaction-side relations quantify over the region the operation claims, region membership enters the observation, and a transaction is the union of its operations' regions, with `AllowedObjectFamilies`, `SponsorIsolation`, `CanonicalDeltaPolicy`, `OpenFlowPolicy` and `SponsorEnvelopeMultiplicity` re-scoped or retired. Gate: model conformance over a composed observation — an announcement beside a second family's operation — is accepted, and a foreign object inside the announcement's own region is refused. Wave-6 scope only if a Wave-6 link depends on a closed carrier for those relations; otherwise the first bite of Phase 7's composition work, because the first composed operation is what exercises it. The discharge table classes ten relation identities MODEL-SCOPE — `AllowedObjectFamilies` on each transaction side, the plain-L-BTC cardinality and recognition relations on each side, `SponsorIsolation` and `SponsorEnvelopeMultiplicity` on the sponsor subject, and `OpenFlowPolicy` and `CanonicalDeltaPolicy` on the operation subject — of which this refit's scope is the six identities its five relations are declared over, the family closure being declared once per transaction side; the remaining four lost their carrier with the retired partition fragment and are model-scope for that reason rather than for this one. Its structural portion is closed by `T12-002` at (`rec:backlog:wave0-region-refit`); the admitted S1/S2 region binding and retained obligations are recorded at (`rec:backlog:wave0-close-safety`), whose historical bounded-consolidation census is retained beside the current reconciliation at (`result:backlog:close-safety-reconciliation`). The remaining obligations close with `T12-005` at (`result:backlog:bite-0-5-gate-pass`): the composed observation and foreign-object refusal are recorded at (`result:backlog:region-composition`); ruling 130 revisits ruling 58 before C1 (`rule:phase7:wave0-rulings`); C1 construction, C2 observation and C3 retyping are recorded at (`rec:backlog:wave0-sponsored-construction`), (`rec:backlog:wave0-sponsored-observation`) and (`rec:backlog:wave0-sponsored-retyping`); S1/S2 positive-floor acceptances are recorded at (`rec:backlog:wave0-sponsored-observation`), the eight semantic forms at (`rec:backlog:wave0-native-observation`) and the combined projection at (`rec:backlog:wave0-combined-projection`), with confidential agreement at (`rec:backlog:wave0-confidential-route-admission`) and exact region binding at (`tab:backlog:conjunct-dispositions`). |
| `T11-098` | DONE | The accepted step: after a separately ruled route and its prerequisite implementation, one run supplying an Accepted submission identity, exact mined readback and a validated continuity comparison over those bytes, admitted as immutable evidence. Ruling 70 at (`rule:phase6:wave9-rulings`) records this dependency and selects neither route. Gate: the current accepted-response validation and an exact readback match. Blocked on the route track's accepted evidence at `T11-108` under ruling 79 at (`rule:phase6:wave9-rulings`); no node run is scheduled by this record. Supplied on `0.6.245-dev`: the accepted identity, the exact mined readback and the accepted-byte continuity comparison admitted under `T11-108`; the row classification over those bytes remains with `T11-099`. |
| `T11-099` | DONE | The Wave-9 closure or precisely partial stopping record: recounts, dispositions on gate evidence, the register clauses actually advanced and the residuals, with no unconditional first-exit claim without accepted readback. Gate: closes `T11-091`. Depends on `T11-097` and on either `T11-098` or an explicit conditional-closeout ruling. Closed on the label gate, this record. |
| `T11-100` | DONE | The route record: ADR-025, ruling 79 at (`rule:phase6:wave9-rulings`), the two archived inputs and these ten rows establish the constant-elision route at (`[ADR025-dec:relay-witness:constant-elision]`). DONE when the route closes under `T11-109`. Gate: documentation, census and label checks. Closure depends on `T11-109`. Closed on `0.6.245-dev` by the route-closure record under `T11-109`. |
| `T11-108` | DONE | Obtain and immutably admit the fresh node run: fund a predecessor under the new leaf, settle the three exchanges through `scripts/live-native-maturity-capture.sh` and the existing executor, and admit Accepted identity plus exact mined bytes, txid and wtxid into the second corpus with its Meson entries; retain and diagnose any refusal and disclose the fee-free environment. DONE when accepted-byte continuity validates and the old corpus still admits. Gate: all three exchanges settle, submission is Accepted, exact readback validates, and both corpus admissions pass at (`[ADR025-gate:relay-witness:adoption]`). Supplies `T11-098`; depends on `T11-106` and `T11-107`. The run's four files are admitted on `0.6.245-dev` under `packages/vectors/fixtures/maturity-variable-run-of-record/`, and the live carrier and corpus pins landed on `0.6.244-dev`. |
| `T11-109` | DONE | Close the route track with measured values, evidence identities, row dispositions, remaining obligations and `T11-098`'s evidenced status. DONE when those dispositions close `T11-100` without attributing acceptance to absent evidence. Gate: the full gate plus documentation, label and census checks, against the adoption gate at (`[ADR025-gate:relay-witness:adoption]`). Depends on `T11-108`. Closed on the label gate, this record. |
| `T11-110` | DONE | Test-performance measurement and the first scheduling remedy are recorded in [test-performance.md](research/test-performance.md) and (`rec:backlog:wave0-test-performance`): the dated register retains the comparable before/after pair, ruling 129 is implemented, and the warm wall falls with equal census and attribution. Gate: `scripts/check-plans.sh`, Meson `lint`, and `tripod-labels` tests. Dependencies: none. |
| `T11-112` | DONE | The tagged-hash premise test in `packages/target-elements-conformance/src/tests/constructor_tests.rs` pins each collision and second-preimage premise to its exact leaf, branch and tweak domain, so the recorded premise count of `tweak_evidence_recomputes_both_sides` is carried by six named premise-to-domain pairings rather than a count; no production change. Gate: each pairing's test fails when its domain or premise is perturbed. Depends on nothing beyond the current tree; scheduled beyond the wave. DONE on `0.6.249-dev`: `commitment_premises_name_constructor_hash_domains` and `tweak_evidence_recomputes_both_sides` in `packages/vectors/src/maturity_continuity.rs`, with `a_tag_separates_domains` in `packages/target-elements-conformance/src/tests/constructor_tests.rs`, pin the premise-to-domain association. |
| `T11-114` | DONE | The typed edge sequence in new `packages/vectors/src/maturity_history.rs`, with `packages/vectors/src/lib.rs`, `packages/vectors/meson.build` and the continuity module's test sources made crate-visible: `StateRootEdge` carries the two outpoints, `OperationId` and a `MaturityTransitionCertificate` projected from a validated continuity rather than accepted from a caller; `validate_state_root_history` checks outpoint uniqueness over the whole sequence before it reads any edge, then applies §17.1's clauses and `state_bundle_continuity` edge by edge, returning the advanced cursor only after every edge validates, while `MaturityRootHistoryRefusal` retains the clause, position and operands in eight variants. Eighteen tests comprise one accepted-archive edge, sixteen one-axis §16.8 refusals and one cursor-return test; the Meson census gained one entry and no existing test count moved. DONE on `0.6.257-dev`: each clause and fault refuses on its own axis, the three-edge sequence whose final successor equals the honest one refuses at position 1 under clause 4, the accepted archive's one edge advances the cursor to its successor at output 0, and §17.3 calls the linker's `state_bundle_continuity`. This establishes edge ordering over supplied bytes, with no report, row standing, census change or chain observation. Depends on `T11-113`. |
| `T11-115` | DONE | The checkpoint and registry in `packages/vectors/src/maturity_history.rs`, `packages/vectors/src/maturity_evidence.rs` and `packages/vectors/src/maturity_corpus.rs`: `MaturityRootCheckpoint` binds §17.2's ten facts in eleven private fields, six carried by the admitted archive, one read from the public bytes, one derived from the bytes and the locator, and two `MaturityDeclaredPremise` values carrying the new `MaturityPremiseProvenance::DeploymentDeclaration`; its sole constructor `bind` refuses an outstanding acceptance and checks the readback's byte digest once, and `binds` asks three questions (identity, successor, predecessor), each `MaturityRootCheckpointRefusal` retaining both operands; `ModelledBranchPrefix` is admitted by `anchored`, with `from_checkpoint` delegating to it under the continuity's branch identifier, extends and rewinds into new prefixes, and reprojects into a `MaturityRootObservation` or a typed stale refusal raised after the foreign-prefix refusal, while `MaturityModelledBranchRefusal` has five variants with `ZeroPrefixIdentifier` first; `MaturityMutationRegistry` replaces the outstanding-named type, its `registered` constructor carries no visibility modifier and is therefore module-private, it derives the sixteen §16.8 entries from the matrix rows whose locator is `RootHistoryEdge`, and `census()` reads the standing from the list. Six tests, three in history, two in evidence and one in corpus, moved history tests 18→21, evidence 19→21 and corpus 41→42; registry mutations moved 0→16, `MaturityRegistryStanding` members 1→2 and the retired outstanding-reason members 1→0. DONE on `0.6.259-dev`: all ten facts distinguish checked bindings from premises, the sixteen entries are set-equal to the matrix rows in both directions, no public registry constructor exists, a rewound or changed block stales the observation and another prefix refuses reuse, and no evidence-plan census bucket moved. This establishes neither chain freshness nor row standing. Depends on `T11-114`. |
| `T11-116` | DONE | The public handoff in new `packages/vectors/src/maturity_recovery.rs` and new `packages/vectors/tests/maturity_recovery.rs`, with `packages/vectors/src/maturity_continuity.rs` (three items made crate-visible with their documentation), `packages/vectors/src/lib.rs` and `packages/vectors/meson.build`: `PublicAnnouncementHandoff` carries §14.7's three public inputs beside five published deployment parameters, its static subtree being the one-leaf announcement subtree rebuilt from witness item seven under the reviewed target, because the linker substitutes the announcement leaf before it builds the subtree; `recover_public_successor` takes that value alone and returns `RecoveredSuccessor` or one of eleven `MaturityRecoveryRefusal` variants over nine §16.13 rows through a total row mapping; the integration binary imports neither the linker bundle, the deployment identity nor either planner. Thirteen unit tests cover nine §16.13 mutations and four type-level construction proofs, and none reaches `SuccessorCommitmentRefused`: the accepted successor's witnessed representation nonce is zero, a value the landing records rather than pins, so no lower candidate exists, and no public candidate leaf refuses the branch side; two integration tests cover accepted-archive output-0 equality and the complete import list; the Meson census gained two entries and no existing test count moved. DONE on `0.6.261-dev`: the handoff carries exactly its eight public inputs, the recovery function has no second argument, every mutation refuses on its own axis, the four private-dependency exclusions are asserted over the type, the import list is pinned, and the accepted archive's reconstructed program equals output 0. This establishes reconstructibility for that transaction, with no report, row standing, freshness, origin or relayability claim. Depends on `T11-115`. |
| `T11-117` | DONE | The two validated reports and their routing, landed on `0.6.263-dev` (the root-history report in new `packages/vectors/src/maturity_history_report.rs`), `0.6.264-dev` (the public-recovery report in new `packages/vectors/src/maturity_recovery_report.rs`, with the binary's third test and three pinned imports in `packages/vectors/tests/maturity_recovery.rs`), each with `packages/vectors/src/lib.rs` and `packages/vectors/meson.build`, and `0.6.265-dev` (the boundaries, the retyped rows and the recount in `packages/vectors/src/matrix.rs`, `packages/vectors/src/maturity_safety.rs`, `packages/vectors/src/maturity_evidence.rs`, `packages/vectors/src/maturity_report.rs` and `packages/vectors/src/observed_boundary.rs`, with `packages/vectors/src/maturity_negative_half.rs` unedited): `MaturityRootHistoryReport` (twelve fields, nineteen recomputed items, six refusals) and `MaturityPublicRecoveryReport` (fifteen fields, twenty-six items, six refusals and a fourteen-row registry answering one row by recovery, nine by their own refusals and four by construction proofs) have separate assemble-and-validate pairs, roles, residuals, censuses and canonical bytes, the history report rendering the three synthetic-origin residuals and the recovery report those three followed by `witnessed-nonce-without-leastness`; `EvidenceBoundary` gains `RootHistoryReportRejection` and `PublicRecoveryReportRejection`, and `classify` enters `RootHistoryObserved` or `PublicRecoveryObserved` only when the matching validated report observes the row. The sixteen §16.8 rows and the fourteen public-recovery rows, thirty in all, take those boundaries; the two displaced typed reasons leave `MaturityRowBoundary`. Counts moved: boundaries 13→15, row-boundary members 7→5, typed non-answers 65→35, answered 41→71, root-history observed 0→16, public-recovery observed 0→14, and the partition six→eight buckets summing to 206; `STILL_REQUIRED` stays 42, evidence tests moved 21→22, safety tests stay 20, and Meson gained two entries. DONE across the three tags: both reports validate and render their ordered residuals over the accepted archive, reject changed claims at named items, keep their role bytes separate, route exactly the thirty named rows, refuse the evidence derivation through its own refusal member when the accepted replay, the checkpoint or either report refuses rather than leaving rows outstanding, recompute the 206-row partition and the waiting register's set equality, and leave neither retired reason in `packages/`. The eleven acceptance-awaiting positive rows keep their separate question; neither report establishes freshness, chain observation, archive origin, artifact promotion or the other's evidence. Depends on `T11-114` through `T11-116`. |
| `T11-122` | DONE | Branch realizations, the invalidation event and the thread's semantic history for `G14C-11` in new `packages/vectors/src/maturity_branch.rs`, `packages/vectors/src/lib.rs` and `packages/vectors/meson.build`, as bite 4a of the study fixes them: `MaturitySemanticEdge`, `MaturityRealization`, `MaturityBranchView` with `realize` and `rewind`, `MaturityRealizationSuffixInvalidation`, `MaturityIntendedStep`, `MaturityThread` with `record` and `project`, `MaturityThreadProjection` and `MaturityBranchRefusal`. DONE when a fork after the accepted predecessor realizes the accepted and a host-built competing successor on their own branches while their union refuses as a reused predecessor, a rewind of one branch names exactly its removed realization and leaves the other branch and the thread's semantic history unchanged, and a fresh planner replay re-realizes byte-identical bytes into an identical view. Gate: the bite's readings — four module tests, no `Option` field, no production panic path and exactly the three files changed. Depends on `T11-118`, no code of `T11-119` to `T11-121` being read, and lands after `T11-121` in the study's order. DONE on `0.6.272-dev`: the four module tests pin competing branches, suffix invalidation, preserved semantic history and byte-identical reproduction. |
| `T11-125` | DONE | The stale comments of findings 171 and 172 in `packages/vectors/src/maturity_safety.rs`, as bite 6 of the study fixes them: the documentation of `AcceptanceAwaitsRelayAdmissibility`, the sponsorless row's comment and the standings test's comment, comments only. DONE when each comment states what its code does at the tip. Gate: every changed line a comment, twenty tests unchanged and exactly the one file changed. Depends on `T11-124`. DONE on `0.6.275-dev`: the three comments state their code's landed standing, with assertions, tests and census figures unchanged. |
| `T11-129` | DONE | Bite 1 in `packages/transaction/src/tests/state_construct_tests.rs`: `each_request_form_is_built_or_refused_by_name` walks `RequestedForm` crossed with `SponsorChangeRequest` over one validated view and ABI. It pins four distinct forms in order: sponsorless without change builds one input and one output; sponsorless with change returns `SponsorChangeWithoutSponsoredForm` from `MaturityAnnouncementRequest::new`; sponsored without and with change each return `SponsoredMaturityFormHasNoCarrier` from `construct_over`. The file's tests move 12 → 13, `SponsoredMaturityFormHasNoCarrier` occurrences move from 3 to at least 4, and no production line changes. DONE when the four dispositions are asserted at those boundaries. Gate: 13 `#[test]` markers, at least four refusal occurrences, and exactly this one file changed. The host refusal creates no sponsored byte and reaches no node. Depends on `T11-128`. DONE on `0.6.278-dev`: the four-form test pins the named construction and request boundaries. |
| `T11-130` | DONE | Bite 2 in `packages/realization/src/tests/open_flow_tests.rs`: `sponsorship_moves_no_announcement_relation_outside_the_sponsor_family` compares the sponsorless and sponsored observations on all 20 of the declaration's 26 identities outside its six sponsor-family identities, and `a_state_corruption_fails_the_same_state_relation_with_or_without_a_sponsor` makes the output STATE asset fail recognition on both observations while sponsor values remain `SponsorOpaque`. Tests move 14 → 16 in this file; the 22 announcement-maturity tests remain unchanged, with no production line changed. DONE when both assertions hold. Gate: those two test counts and exactly this one file changed. These model observations are not transactions, answer no target-mutation row, and decide no refit of sponsor-family relations. Depends on `T11-128`. DONE on `0.6.279-dev`: the two model tests pin the non-sponsor relation comparison and equal STATE failure. |
| `T11-131` | DONE | Bite 3 adds `the_canonical_bytes_publish_no_sponsor_key` inside each test module of `packages/vectors/src/maturity_continuity_report.rs`, `packages/vectors/src/maturity_report.rs`, `packages/vectors/src/maturity_history_report.rs`, and `packages/vectors/src/maturity_recovery_report.rs`. Each test renders its validated report, uses the public `FORBIDDEN_KEYS` and `forbidden_key_in` vocabulary, catches every forbidden key staged as an appended rendered line, and permits a forbidden word in a value. Test counts move 31 → 32, 20 → 21, 10 → 11, and 12 → 13 respectively; no production line or census figure moves. DONE when each rendering excludes the keys and each staged key is detected. Gate: those four counts, no `forbidden_key_in` use above each test module, and exactly the four files changed. The check concerns keys in sponsorless report bytes; it answers neither `report-publishes-*` row and establishes no sponsored byte. Depends on `T11-128`. DONE on `0.6.281-dev`: the four report tests pin forbidden-key exclusion in each validated rendering. |
| `T11-132` | DONE | Bite 4 in `packages/vectors/src/maturity_safety.rs`: `MaturityRowBoundary` gains documented `SponsoredControlRefusedAtConstruction`; the two §16.1 sponsored positives and nine §16.10 rows take it, while `SponsorArmAwaitsRegionScoping` retains `two-sponsor-envelopes`, `sponsor-member-unclassified`, and `zero-valued-sponsor-member-under-exact-role-structure` with documentation limited to the relations `T11-064` re-scopes or retires. The standings test, its exhaustive match and the section comments state 9 acceptance-awaiting, 11 construction-refused, 3 region-scoping, 11 totality, and 1 build-property rows; tests remain 20, typed non-answers 35, rows 206, answered 71, relation standings 14/102/90, and every census bucket unchanged. DONE when each touched row remains a typed non-answer on its named reason and no canonical byte moves. Gate: the three reason-use readings 9, 11 and 3; 20 tests; `the_census_figures_are_unmoved_by_the_binding_readers`; and exactly this one file changed. It answers no row. Depends on `T11-129`; `T11-125` is landed. DONE on `0.6.280-dev`: the standings test pins the reason split without moving a row answer, census bucket or canonical byte. |
| `T11-133` | DONE | The Wave-11 closure record at (`sec:phase6:wave11-findings`): recount the landed tests, reason standings, census, report keys and register, and dispose `T11-129` through `T11-132` on their gate evidence. It states the three exits against the study's expected CONDITIONALLY MET, MET and MET, with §23's eighth conjunct following the third; retypes the Wave-10 closure's eleven acceptance-awaiting rows where bite 4 moves them to nine; and states C1, the sponsored STATE builder, conditional on `T11-064` and a ruling revisiting ruling 58, C2, the sponsored native ceremony, conditional on C1 and an Elements-provisioned compute node, and C3, the sponsor rows' boundary retyping, conditional on C1 for eleven rows and `T11-064` for three, without minting their rows. The wave owns no register row and changes no status cell. DONE when the evidenced disposition closes `T11-128`, names the remaining sponsored-byte, node, row, guide-text, §16-matrix and positive-fee-floor limits, and leaves the register recount intact. Gate: the closure's documentation, label and census checks over the landed evidence. Depends on `T11-132`. DONE on this row's label gate: findings 199–203, the row dispositions, exits and obligations close Wave 11 on the landed evidence. |
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
| `T11-152` | DONE | Bite 4 admits both new mutant-run-of-record fixture directories under `packages/vectors/fixtures/`, adds eight census lines to `packages/vectors/meson.build`, and edits `packages/vectors/src/maturity_corpus.rs`, `packages/vectors/src/maturity_evidence.rs`, `packages/vectors/src/maturity_negative_half.rs` and `packages/vectors/src/maturity_report.rs` for `MaturityCaptureTarget`, `MaturityMutantCorpusPins`, `admit_maturity_mutant_corpus`, both run accessors, roster replay, row-bound refusal standing, recomputed census and schema-2 `native_refusal` rendering. It restates the run-of-record and row-standing sentences in `packages/vectors/README.md` made stale by the admitted runs, and any swept document's Meson lane count changed by `T11-150`. Eight new tests cover both pins and replay, changed-byte refusal, earlier-run admission, nonbinding phase walls, seven row bindings, unbound refusals, the rendered refusal block and altered-line validation. DONE when both mutant runs and both earlier runs admit, each bound refusal stands `NativeRefusalObserved`, and the census, register, report and swept prose state the admitted figures. Gate: corpus tests 42 → 46, evidence 22 → 24, report 21 → 23, negative half stays 7; 206 rows with seven bound refusals imply native refusal observed 0 → 7, native run required 42 → 35, answered 71 → 78 and `STILL_REQUIRED` 42 → 35, otherwise the run's figures govern; schema 2, eight fixture census entries, the full gate and exactly the files named by this bite. Depends on `T11-151`. |




Current blockers are: NONE carried by the live evidence set, whose residual set is now EMPTY. `PredecessorConstructorAbsent` was the last member and it left by a RULING rather than by a run: the observed pair its clearing rule demanded was demanded by a mis-typing of the `time-locked-input` row, §10.4 refuses the maturity half because the time-locked class is a receipt class rather than a consensus timelock, and the refusal half attributes to a leaf the spent program does not commit to, which every foreign taptree draws alike — so no observation of it could ever name the lock. The row is retyped FIRST-PARTY beside its `time-locked-output` sibling and discharged against the constructor typing that is the real protection. An empty residual set is not a smaller set of obligations: the rows a run has still to answer are counted by the census and not by it, and no time-locked spend was ever observed refused. `SponsorEnvelopeSignerAbsent` is CLEARED, by its own defining site's rule and on an observed result rather than on a capability existing: a sponsor-signed explicit control was accepted by a real node, read back from the node's own copy byte for byte, and mined, which is the acceptance that rule said a returned byte stack had to wait for. The sighash blockers are cleared, the confidential predecessor is funded, the signing handoff is closed, and the evidence restart has taken its first five steps against a real node — six positive private matrix rows are answered by observed acceptances of their own shapes, the three proof-negatives are observed at the mempool boundary and attributed by field, and the multi-output and multi-input fixture constructor exists and derives at any width. The restart order stopped typed at step six on the sponsor signer dependency through three waves; that dependency closed by observation, and the minimality wave recorded step six and step seven in a wave-eight ledger of its own — the wave-seven ledger is a record and was not amended. The order is COMPLETE: all seven steps accepted, the disposition is `Completed`, and both accepted steps state their own shortfalls in their entries. Eight positive private rows have moved on observed acceptances of their own shapes, private-merge and private-sponsor-values being the two the minimality wave added — the sponsor row had moved in the guide closeout while the safety matrix still classified it as awaiting a run, and the matrix caught up in the same wave rather than being left to disagree with the closeout. Of the remaining two, deterministic-public-fixture-openings is now ANSWERED in the safety matrix on a minted determinism standing — its §11.2 gate is the byte-identity contract rather than an acceptance, the wave-7 observation was already recorded, and the new member is counted in its own bucket so a determinism observation is never read as an acceptance; it stays out of the closeout delta, which admits target-computed identities only, and the two artifacts disagreeing there is the vocabulary difference showing rather than a row nobody is checking. Projection-equality-with-paired-explicit did NOT move and the failing conjunct is named: its rule wants an observed acceptance of its own shape, and no run compares the semantic projections of an accepted private transaction and its paired accepted explicit one — the equality this workspace holds is first-party, over registry members that never reached a target. The remaining work is an observation comparing two accepted transactions' projections, and runs of the two pair shapes that have none; one question is still handed up for a ruling, the deterministic-fixture-openings gate, and the step-six/step-seven ordering question is answered by the erratum filed against the guide's mandatory chain. The sponsor dependency is CLOSED and its closure was a repair to this repository rather than to the target: the deployment's fee-role program digest was a fixture constant no program hashes to, threading it to the digest the target itself computes for the empty fee program construction writes produced the first observed sponsor-signed acceptance, and the record says in its own bytes that this establishes the sponsor envelope's wire and one acceptance and not production multi-party sponsor signing.


### Fixed-form Keyed Covenant adoption and closure · `rec:backlog:fixed-form-closure`

Adopt the named track (`task:guide15-exec:fixed-form`) and rulings 193–212 at (`rule:phase7:fixed-form-rulings`), with the present-ruling ledger (`tab:phase7:fixed-form-legacy-rulings`). The proposals remain historical inputs. The corrected packets below govern S0–S9 under (`def:kffwork:packet`); S1 adds a small labels census code change and publication regressions to documents/publication, so requires C/D/P. Adoption establishes future direction, not implemented predicates, proofs or native outcomes. READ source evidence supports the census; FOLLOWS choices below state their reasons; OPEN work remains assigned to its producing step.

**Table (Fixed-form task packets)** · `tab:backlog:fixed-form-tasks`

| Task reference | Status | Closing reading and gate |
|---|---|---|
| (`step:kffwork:s0`) | ADOPTED; implementation open | Exact paths, sweep, ledger and decisions in this record; candidate-bound D receipt accompanies the S0 submission; independent authority/layout review and full repository gate at import. |
| (`step:kffwork:s1`) | DONE | Complete fixed-form paper and publication consumer regressions; C/D/P. Landed as `0.7.158-dev`; full repository gate green at 2026-11-16T12:39:31Z, 1,337s. Receipt below. |
| (`step:kffwork:s2`) | DONE | Realization recipe, environment/envelope/floor and paper obligations at (`[DOC-sec:keyed-fixed:scope]`); property-by-property pre-admission at (`rec:identities:keyed-fixed-capture-stop`). Landed as `0.7.164-dev`; full repository gate green at 2026-11-16T17:52:01Z, 1,413s. Ruling 211 leaves capture-freeze admission to S5 and activation to S8; S2 mints no identity. D with build-registration verification; receipt and timing-test correction below. |
| (`step:kffwork:s3`) | DONE | Complete declarations/member/manifest identities, unchanged legacy and attestation controls, paired namespace re-pin; C/D. Landed as `0.7.168-dev` after independent review; full repository gate green at 2026-11-16T22:10:05Z, 5,429s. Receipt below. |
| (`step:kffwork:s4`) | DONE | Public sealed compiler plans, full projections/re-derivation, cross-owner refusals; C. Landed as `0.7.169-dev` after independent review; full repository gate green at 2026-11-17T07:42:03Z, 3,847s. Receipt below. |
| (`step:kffwork:s5`) | DONE | Independently checked transcript/104-byte leaf/linker/commitment/resources and separate typed freeze, owned encoding, inventory and ADR-021 admission under ruling 211, recording S8's deadline-bound consumer migration; C/D. Design question for this program producer: should a typed authenticated program bind the program to its triple, instead of leaving that binding solely to the caller? This records an option, not additional assigned implementation. Landed as `0.7.174-dev` after independent review; full repository gate on `0.7.175-dev` green at 2026-11-17T09:47:13Z, 2,476s. Receipt below. |
| (`step:kffwork:s6`) | DONE | Selected implementation/tests: (`sec:backlog:fixed-form-s6-design`). Public construction, exact member authentication, amount/envelope bounds and arbitrary merge surplus; C. Under ruling 212, consume the target-owned stated-amount bound: an accepted control at its maximum and a subject one unit above must produce the typed too-large refusal at the constructor. When S6 edits (`sec:transaction:fixed-form`), name reviewed_stated_amount_bound() there as the bound owner. S5 consumer dispositions (facts, not new requirements): (1) LinkedKeyedFixedBundle::bytes() is the complete leaf; relation fragments are coverage material with their own H pushes and are not programs. (2) Recipe offers come independently from the request's authenticated triple; echoing the emission's recipe supplies no binding check. (3) The offline vectors constructor supplies root, tweak, output key, parity, program and control comparison material; the record fixes their reconstruction. (4) Member receipts cover supplied roles, not parent authentication; the binder accepts mixed recipes, so S6's existing exact-member authentication includes program Pq, birth zero and explicit A/v, and agreement with the successor's Pq. (5) The 46 selected sources become pinned at S5 landing, including whole manifests and the shared model helper; subsequent authorized edits carry pins/history/dispositions under (`rule:vectors:keyed-refreeze`), while S6 sources join at S8's re-freeze. (6) transaction_weight takes complete stripped bytes once, other witnesses and 143 bytes per member; 307 marginal WU is predicted, None maximum is unadmitted, and typed N+1/value ceiling remain S6's existing bound work. (7) The leaf drops outpoint flags; issuance/peg-in remain excluded by the constructor envelope, with peg-in classification/evidence left to S7/S8. Landed as `0.7.178-dev` after independent review; full repository gate green at 2026-11-17T12:03:36Z, 2,429s. Receipt below. |
| (`step:kffwork:s7`) | DONE | Selected implementation/tests: (`sec:backlog:fixed-form-s7-design`). Full byte-reader/model agreement, filters, SOURCE-only attribution, missing-branch and malformed-payment controls; C/D. Review disposition: classify present-nonce and non-policy-asset exact-pair members independently of S6's narrower construction scope; classify a foreign triple's handle offered as collateral as a nonmember, not a valid chain spend; authenticate txids over stripped bytes including the zero flag byte. Landed as `0.7.179-dev` after independent review; full repository gate green at 2026-11-17T13:22:24Z, 1,498s. Receipt below. |
| (`step:kffwork:s8`) | TODO | Selected sub-steps, tests, operator packet and questions: (`sec:backlog:fixed-form-s8-design`). Activate S5's admitted fixed-form freeze consumers and verify its migration before any fixed-form capture under ruling 211; reviewed native subjects, capture admission, resource/reading/contention results and measured test budgets; C/N/D. Review disposition: retain the effective node weight bound beside the 400,000-WU target ceiling and 72-byte sponsor bound, and measure 1299/1300 endpoints. Give asset identity, present nonce, confidential form and M−1 floor subjects declared structural routes because S6's typed path refuses them. Re-freeze the shared commitment/transaction-byte owners, including `keyed_curve.rs`, `taproot.rs`, `bytes.rs` and consumed parts of `construct.rs` and `sponsor.rs`, as well as the new S6 owners. S7 review dispositions: supply every triple whose handles can appear in a capture; keep captured events and retrieved parents within the shared decoder's domain, because a proof-bearing blinded-asset parent, issuance or peg-in refuses the whole read; set inclusion and retrieval assertions from actual evidence; claim no finality without a published depth; reproduce the fixed linear and branching workload shapes in `packages/vectors/tests/keyed_fixed_history.rs` or choose their library owner before measurement. |
| (`step:kffwork:s9`) | TODO | Entry includes the one-identity four-family campaign of record at (`rec:history:fixed-form-s8a-campaign`) and S8b/S8c's separate admitted observations; independently join rulings/follow-ups, legacy preservation, external facts and absent dimensions; C/D/P. |

S1 receipt: `~/loops/evidence/wave2/record-0.7.158-dev/{manifest.txt,gate.log}` on kranich names `0.7.158-dev` as the gated tree and records the complete papers build, including the fixed-form 2.0.0 PDF. All nine batteries exited 0; `ci.sh` passed 116 Meson lanes with zero failures, and the real-TeX documentation/generated run passed five checks with zero failures. The green rerun ended 2026-11-16T12:39:31Z, wall 1,337s, chain rc 0, with a clean tree after the run.

S2 receipt: `0.7.164-dev` publishes the DOC-owned public recipe and build registration, conditional authentication, paper-obligation mapping, source-derived floor and envelope, and explicit implementation/native evidence debts. Package consumers point to that record. Its fixed-form capture-freeze walk remains pre-admission; ruling 211 assigns encoder/inventory/admission to S5 and consumer activation before capture to S8, without changing ruling 210. The loop's full repository gate receipt is green at 2026-11-16T17:52:01Z, wall 1,413s. An earlier run was RED on the confidential agreement test's wall-dependent verdict under node load although the candidate changed only documents. The correction landed separately as `0.7.165-dev`: child guards become hang guards, the confidential family wall assertion and final-capture wall assertions are removed, and printed timings remain. Its full repository gate is green at 2026-11-16T18:20:11Z, wall 1,498s. The earlier failure is not erased; unchanged keyed guards and the unknown full transport wall remain at (`tab:backlog:wave1-followups`).

Paper release 2.1.0 clarifies that SOURCE consumes no handles of its own triple while allowing ordinary funding inputs (`[A-op:kpcfixed-operations:source]`), makes both branches and the empty-case reason explicit in the existing recurrence (`[A-def:kpcfixed-operations:classification]`), and identifies additional meanings as reader folds (`[A-rem:kpcfixed-operations:reader-folds]`). Membership, record sets and theorem premises are unchanged; SOURCE-only attribution and dependency-closure requirements remain in force. The versioned PDF and flat publication names follow the paper version. OPEN: the release's independent reading and full repository gate, including the real paper build, require import receipts; the S1 receipt above remains evidence for its published edition.

S3 receipt: `0.7.168-dev` adds the sibling fixed-form vocabulary and semantic manifest, positive symbolic value and member predicate, SOURCE/MERGE declarations and identities, with unchanged legacy and attestation controls. The program-to-triple binding remains the caller's obligation; the typed-program option is recorded at S5. The frozen shared profile source is re-pinned with retained capture identity and history controls; no fixed-form freeze is admitted here. Ruling 212 places the target ceiling at S6. Independent review preceded landing; full repository gate green at 2026-11-16T22:10:05Z, wall 5,429s.

S4 receipt: `0.7.169-dev` publishes sealed fixed-form compiler input and immutable SOURCE/MERGE plans through the existing shared kernels, with full independent projections, re-derivation, namespace and cross-owner refusals. SOURCE has no runtime clause; MERGE carries born-first, own qualification and successor qualification. Legacy compiler sources and common kernels remain unchanged; key/program authentication stays with the program producer and reader. Independent review preceded landing; full repository gate green at 2026-11-17T07:42:03Z, wall 3,847s. The first run ended red on battery time budgets alone under concurrent node load: release build 5,452s against 5,400s. That evidence is retained; the green rerun on the same commit is the receipt.

S5 receipt: `0.7.174-dev` supplies the fixed-form transcript, emitted leaf, linker and commitment path, bounded offline stack comparison and a separate typed capture freeze with owned encoding and identity admission. Independent review rebuilt the transcript, pair digest, leaf, internal key and freeze identity from the realization record alone. Legacy frozen sources and byte pins remain preserved. S8 still owns consumer activation and the expanded source inventory before fixed-form capture; no native execution or constructor/reader completion is claimed. Full repository gate on `0.7.175-dev` green at 2026-11-17T09:47:13Z, wall 2,476s.

S6 receipt: `0.7.178-dev` provides the sibling fixed-form constructor through authenticated parents, checked SOURCE/MERGE requests, structural construction, policy admission and ordinary sponsor signing. The envelope remains ruling 214's policy-asset, empty-nonce parent construction scope; reader membership is broader. Recounting the landed formula gives 4 × (239 + CompactSizeLen(n + 1) + 41n) + 117 + 143n: 1,299 members weigh 399,878 units, while 1,300 weigh 400,185 and exceed the 400,000 ceiling. This is a construction-policy bound, not a covenant count law. The target-owned value ceiling, aggregate funding and positive-fee limitations remain separate. Independent review reproduced the formula, member bound and commitment with its own serializer. Legacy/frozen sources are unchanged; native evidence remains S8. Full repository gate green at 2026-11-17T12:03:36Z, wall 2,429s.

S7 receipt: `0.7.179-dev` reads fixed-form membership and SOURCE-singleton/MERGE-union record sets from authenticated retrieved bytes for the public triple, with an independent graph-model comparison. Missing evidence, invalid histories, an unknown triple and finality remain distinct. Reader predicates select a subset of SOURCE records without changing membership or recurrence. Independent review found the recurrence right on every history it tried; the landing corrects member-spend annex handling, all-or-nothing fold acceptance and an unsupplied queried event previously reported as a negative answer. Present-nonce/non-policy-asset membership is not narrowed to S6's construction envelope. Legacy sources and frozen pins remain unchanged; S8 retains native evidence and workload measurement. Full repository gate green at 2026-11-17T13:22:24Z, wall 1,498s.

Each step consumes its predecessor’s accepted contract; preparation may overlap but cannot claim its gate. The old-leaf copy subject may prepare after S0. The complete observation table (`tab:kffwork:observations`) is adopted: opening-holder and public-only copy controls on the legacy leaf; fixed-form SOURCE/MERGE acceptance under multiple triples; separately balanced asset, value, asset-form and value-form refusals; malformed SOURCE membership/input guard and later-position birth controls; policy-asset empty-nonce M−1/M dust edge and rate/nonce qualifications; weights, bounds and surplus; conflict/retry and full-closure reading. Refusals name their exact boundary, not impossibility. Independent proof, declaration/compiler, byte/linker/constructor, reader and pre/post-capture readings precede the corresponding dependent claim; S9 joins their candidate-bound journals and full import gates. Wave-2 delivery, production finality and unresolved legacy fully-confidential continuation are not closed here.

**Record (S0 decisions and later facts)** · `rec:backlog:fixed-form-decisions`

Ruling 214 selects policy-asset triples and empty-nonce member parents in the one-P2WPKH-sponsor, three-output, 72-byte worst-signature, 400,000-WU fixed-form constructor envelope. Other-asset triples and present-nonce exact-pair outputs remain members for readers but outside S6 construction. Ruling 212's maximum is checked-request admission only: a positive explicit fee makes a complete maximum-valued successor exceed the target's aggregate output bound.

Ruling 212 reads the S3 range sentence as two boundaries: realization owns symbolic value and typed zero refusal; S6 construction consumes the sole target-owned ceiling and supplies the bound/above-bound typed refusal controls. No target dependency, copied ceiling or caller-selected ceiling enters realization.

Layout (a) and namespace route (i) are adopted: sibling bodies, one architecture-owned profile code 3, no edits to legacy semantic operation/object unions, and one additive frozen-body exception in semantic.rs. Source inspection supports unchanged explicit codes, not an executed equivalence result. New local vocabulary lives under keyed_fixed; unpinned sealed registrations and shared compiler Source/LoweringError arms admit the new owner without another engine. Package receivers are (`sec:realization:fixed-form`), (`sec:compiler:fixed-form`), (`sec:tapscript:fixed-form`), (`sec:linker:fixed-form`), (`sec:transaction:fixed-form`) and (`sec:vectors:fixed-form`). Architecture’s README is its actual contract owner, assigned to S3; S0 records its decision here within planning confinement, rather than inventing plans/packages/architecture.md.

Landing order: S3’s semantic.rs addition waits for the accepted Wave-2 bite 2.3b vocabulary/identity integration and relation.rs re-pin landing, before the next overlapping declaration-registration landing if both are ready. That re-pin requires a revised Wave-2 packet adding packages/vectors/src/keyed_final_manifest/pins.rs and packages/vectors/tests/keyed_final/freeze_history.rs before 2.3b starts: neither path is in its adopted path/consumer lists, and (`rule:guide15-exec:wave2-settings`) stops a bite needing an additional path. This track records the dependency; Wave 2 owns that packet correction, which this adoption does not make. S4 follows that accepted compiler input baseline. S1/S2 and old-copy preparation have no Wave-2 completion dependency. This short integration edge avoids concurrent edits to inherited vocabulary and freeze history; it does not wait for Wave-2 closure. At each landing merge the accepted base, reconcile the shared registrations and rerun both owner suites; 2.4/2.6 dispatch edits and subsequent S3/S4 corrections serialize whichever packet is ready first. Wave 2 retains its architecture-read-only setting: S3 alone owns the explicit exception and its compatibility evidence.

Freeze decision: every inventoried re-pin by either track stales the working legacy freeze identity; none replaces capture-time provenance. Under accepted ruling 210 at (`rule:phase7:fixed-form-rulings`), the present class is admitted by a vectors code step ahead of the documents-only S2 packet at (`rec:identities:keyed-capture-freeze`), with whole-file Dependencies binding and an owned encoding preserving the historical bytes. This satisfies the legacy admission prerequisite for Wave 2’s packages/realization/src/relation.rs and S3’s packages/architecture/src/semantic.rs: either re-pin may proceed under (`rule:vectors:keyed-refreeze`) once its other packet and landing dependencies are met. Preserve the recipe and capture-time digests; the source/pin/history landing supplies its own before/after controls and affected-evidence disposition, and an additive source edit is not editorial. S2 specifies the fixed-form realization recipe and records capture-freeze pre-admission; ruling 211 assigns that distinct identity's admission to S5's typed freeze/encoding/inventory owner, with S8's deadline-bound consumers active before fixed-form capture. S3 and S4 neither read nor re-pin a fixed-form freeze; the legacy admission does not select the new recipe. OPEN: each re-pin's new values and affected-evidence disposition require its own equivalence packet; fixed-form native observations require S8 capture.

Choose empty nonce for the canonical constructor envelope because the value is explicit and this minimizes serialized size. The leaf binds the asset/value prefixes and contents, not nonce: a consensus-valid present-nonce output with the exact pair is still a member. Readers cannot reject it as nonmember solely for leaving the canonical envelope; constructors and relay claims state their narrower policy scope. With standardness required, the derived policy-asset minimum is M(R,S)=max(1,CFeeRate(R).GetFee(S+32+4+1+floor(107/WITNESS_SCALE_FACTOR)+4)), where R defaults to DUST_RELAY_TX_FEE, S=78 for empty nonce and S=110 for present nonce. At R=100 the minima are 15 and 18; recommend 4M for the declared envelope, never a selected deployment value. Explicit non-policy assets skip dust policy, leaving consensus positivity and funding/fees as separate premises (`calc:kffroute:dust-minimum`). S2 owns the cited derivation and standardness qualification at (`[DOC-calc:keyed-fixed:floor]`); S8 records the null-nonce edge only; native present-nonce qualification remains OPEN under amended ruling 218.

Choose papers/lineage-fixed-form/ for paper 2.0.0, final filename Tripod_Keyed_Provenance_Covenant_v2.0.0.pdf, areas kpcfixed, kpcfixed-interface, kpcfixed-model, kpcfixed-operations, kpcfixed-invariants, kpcfixed-verification and kpcfixed-symbols under the existing A owner. Its versioned filename names the paper, not manifest schema 3. S1 freezes exact anchor names in the complete manuscript; no old label is redirected. Choose docs/lineage/realization.md under DOC with area keyed-fixed, registered through docs/lineage/meson.build, docs/meson.build and docs/README.md. It owns the recipe and environment once, with explicit legacy and new-edition rows; RZ remains attestation-only. S3 implements ruling 209’s schema/domain/profile triple; the schema counter is shared across editions, next allocation 4.

OPEN external facts remain exactly (`tab:kffwork:external`): deployment K,A,v with reference policy/nonce envelope; availability of the selected capture infrastructure; and any deployment inclusion/finality policy/depth to be claimed. The canonical constructor envelope above does not select a deployment’s answer. S1 proofs/anchors are discharged by the S1 DONE row and its landing receipt above. S3 regression/digests and S4 compiler evidence are recorded above. OPEN evidence remains S5 distinct fixed-form capture-freeze admission and final resources, S6 input bound and S8 consumer activation, native edge, costs/contention/environment; none is supplied by adoption or the legacy admission.

S2's realization contract is (`[DOC-sec:keyed-fixed:scope]`); the property-by-property pre-admission record is (`rec:identities:keyed-fixed-capture-stop`). Ruling 211 corrects the ordering: admission belongs with S5's typed validated subject, complete owned encoding and inventory, recording S8's capture binding as a deadline-bound consumer migration under (`[ADR021-req:identity:admission-record]`). S3 and S4 consume no fixed-form freeze; S3's frozen semantic.rs edit belongs to the admitted legacy class. S8 verifies the admission and activates its consumers before any fixed-form capture. The documents record claims no new digest or native result.

### S6 — Construction design and focused selection · `sec:backlog:fixed-form-s6-design`

This selection serves req:dispatch:design-names-focused-tests and prin:environment:testing-serves-the-product: the implementation brief carries these checks without adding a battery. It implements (`step:kffwork:s6`) and (`sec:transaction:fixed-form`), with the seven S5 consumer dispositions in (`tab:backlog:fixed-form-tasks`). Paths below are relative to packages/transaction/ unless qualified. New src/keyed_fixed_abi.rs names construction obligations; keyed_fixed_taproot.rs commits only LinkedKeyedFixedBundle::bytes() and authenticates the supplied predecessor program; keyed_fixed_inputs.rs decodes canonical parent bytes, verifies their content address and selected output, and classifies exact triple/form/birth membership before sealing inputs. keyed_fixed_request.rs seals SOURCE/MERGE requests from those inputs and independently supplied recipe offers; keyed_fixed_construct.rs materializes ordered outputs/witnesses, consumes reviewed_stated_amount_bound(), checks aggregate/funding/fee limits and derives the envelope-specific member bound; keyed_fixed_sponsor.rs binds ordinary sponsor authority to the final transaction. Relation fragments are coverage, never leaf programs; neither the S5 member receipt nor its program comparison authenticates a parent.

The public sibling surface is KeyedFixedCommitment and AuthenticatedKeyedFixedPredecessor; KeyedFixedInputOffer/BoundKeyedFixedInput; KeyedFixedSourceRequest/KeyedFixedMergeRequest; KeyedFixedConstructionRequest, KeyedFixedOutputOffer, KeyedFixedStructuralTransaction, KeyedFixedPolicy and KeyedFixedAdmittedTransaction, with typed commitment/input/request/construction/sponsor refusals. These names allocate the already planned sibling interface, not a new interchange. Request and construction owners consume the sealed stages; vectors reproduction, S7 reading and S8 capture consume their public bytes and descriptors. Reuse TargetTransaction encoding/decoding, generic curve/tagged hashing and ordinary sponsor capability/signature primitives unchanged; production transaction code never calls the vectors/conformance oracle. All needed crate dependencies already exist. Register sibling exports and tests in the listed lib.rs/Meson receivers, and describe ownership/tracing in README.md/TRACING.md. No change to a frozen legacy constructor is selected.

**Table (S6 focused tests)** · `tab:backlog:fixed-form-s6-tests`

New names below are implementation allocations, not tests already run. F is tests/keyed_fixed_surface.rs; R is packages/vectors/tests/keyed_fixed_reproduction.rs. Each refusal has a progressing accepted control and an exact typed error at the named boundary; do not accept any-error or a later unrelated refusal. Expected bytes come from (`[DOC-sec:keyed-fixed:recipe]`) or an independent serializer/hash/curve route, never the producer being checked.

| Test and file | Command | Claim |
|---|---|---|
| fixed_commitment_binds_each_coordinate — F | `cargo test -p tripod-transaction --profile gate --test keyed_fixed_surface fixed_commitment_binds_each_coordinate` | The independently offered triple commits the sole linked leaf to the record's root/tweak/program/control; changing K, A or v changes the commitment, and a mismatched recipe or control refuses at commitment/authentication. |
| fixed_parent_authentication_and_request_roles — F | `cargo test -p tripod-transaction --profile gate --test keyed_fixed_surface fixed_parent_authentication_and_request_roles` | Canonical parent bytes, hash and vout establish SOURCE/MERGE roles; beside valid requests, wrong hash, later birth, wrong pair/form, foreign program/version and false collateral designation refuse at the corresponding input/request boundary. Issuance/peg-in remain unsupported envelope forms, not extra membership clauses. |
| fixed_value_bound_and_aggregate_limits — F | `cargo test -p tripod-transaction --profile gate --test keyed_fixed_surface fixed_value_bound_and_aggregate_limits` | Ruling 212: the constructor's checked-request amount admission accepts the target-owned maximum and refuses exactly maximum+1 with its typed amount-range error. Separately, a funded valid transaction and adjacent aggregate/fee-shortage subjects distinguish individual admission from complete-transaction validity; no duplicated ceiling or overflow wrapping. |
| fixed_source_merge_bytes_and_surplus — F | `cargo test -p tripod-transaction --profile gate --test keyed_fixed_surface fixed_source_merge_bytes_and_surplus` | SOURCE and a two-parent MERGE produce independently expected output zero and script/control witnesses with empty nonce; release to either of two valid extra destinations is allowed, while malformed SOURCE/version/extra-witness requests refuse. No private opening or covenant signer is needed. |
| fixed_sponsor_authority_is_local — F | `cargo test -p tripod-transaction --profile gate --test keyed_fixed_surface fixed_sponsor_authority_is_local` | An ordinary sponsor signs its retained inputs/change/fee and exact final bytes; changing the bound bytes or offering member authority as collateral refuses, without granting the sponsor authority over covenant membership. |
| fixed_envelope_weight_and_n_plus_one — F | `cargo test -p tripod-transaction --profile gate --test keyed_fixed_surface fixed_envelope_weight_and_n_plus_one` | Rulings 163/180: independently serialize the declared SOURCE/MERGE envelope including count prefixes and worst allowed sponsor witness, derive its new N, accept N and refuse N+1 as construction policy before bytes; do not inherit 1564 or the old weight formula. |
| fixed_floor_uses_supplied_policy — F | `cargo test -p tripod-transaction --profile gate --test keyed_fixed_surface fixed_floor_uses_supplied_policy` | The empty-nonce policy-asset floor follows the record's supplied rate/standardness calculation (14/15 at the illustrative default), distinct from positivity, fee rate and non-policy-asset treatment; the 4M recommendation never becomes a default triple. |
| fixed_public_source_merge_reproduce_in_fresh_process — R | `cargo test -p tripod-vectors --profile gate --test keyed_fixed_reproduction fixed_public_source_merge_reproduce_in_fresh_process` | A child process reconstructs SOURCE/MERGE from public triple/parents and ordinary sponsor material for multiple triples, matches the independent vectors commitment, and cannot silently migrate an existing handle to a changed triple. |

Ledger coverage is complete for this step: only 163 and 180 reopen at S6 in (`tab:kffwork:legacy-rulings`); the envelope row supplies derived bounds and typed endpoints here, while S8 still owes measured SOURCE/MERGE endpoints and worst-witness/native evidence. N+1 is construction policy, never consensus or a finite bound on symbolic OneOrMore. Ruling 212's maximum/above-maximum pair is not deferred. (`[DOC-sec:keyed-fixed:envelope]`) and (`[DOC-calc:keyed-fixed:floor]`) govern empty nonce, authenticated ordinary parents, amount/funding/fee separation and source-qualified floor; valid present-nonce members remain in the broader leaf domain.

Ruling 214 closes the sponsor/output/witness envelope open point and the maximum-value shape open point. The inherited narrow shape yields N by complete serialization; no valid complete transaction carries the target's maximum explicit successor alongside a positive explicit fee, so the maximum control is checked-request admission and aggregate refusal is separate. S5's 46 pinned sources remain under (`rule:vectors:keyed-refreeze`) if any becomes unavoidable; no speculative re-pin or deployment triple is selected. Effective node policy and measured endpoints remain S8 obligations.

### S7 — Reader design and focused selection · `sec:backlog:fixed-form-s7-design`

Implement (`step:kffwork:s7`) and (`sec:vectors:fixed-form`) within packages/vectors/. New src/keyed_fixed_history_reader.rs owns read/Input/Reading and exports its evidence/refusal types; children identity.rs, authentication.rs, fold.rs, evidence.rs and refusal.rs respectively own canonical byte/address/dependency indexing, triple/program/asset/value/form/birth qualification and clause checks, complete SOURCE-singleton/MERGE-union recurrence, typed evidence/query distinctions, and exact invalid-boundary errors. src/keyed_fixed_history_model.rs is the independent explicit graph recurrence oracle; src/keyed_fixed_evidence.rs carries reading/acceptance reports and named external premises for S8. Public Input, Handle, Membership, Query, Missing, Reading and Refusal are scoped to the new reader module; the evidence owner exposes Request/Report for internal vectors consumers, not a new external schema. S6's bytes and independent retrieved parent bytes feed the reader; S8 consumes its reports and fixed workload inputs.

Reuse unchanged transaction decoding/serialization, content hashing, standard collections and generic graph facilities where their existing typed boundary admits the new owner. The old reader's index and fold are private and key-specific: do not export or modify them merely to share code, and do not build a generic replacement traversal framework. The planned sibling index/fold applies the same canonical graph discipline to exact triple-qualified membership; its oracle must not call the reader. Preserve existing legacy replay/negative tests unchanged in the ordinary vectors gate. Tests stay in tests/keyed_fixed_history.rs; lib.rs, Meson and README/TRACING receivers register and describe the new modules. Progress and flushed timeout diagnostics use tracing, not the printed history-phase exception.

**Table (S7 focused tests)** · `tab:backlog:fixed-form-s7-tests`

H is packages/vectors/tests/keyed_fixed_history.rs. Names allocate new tests. One small fixture containing two SOURCEs, a branch join and a later continuation is enough for the shared basic controls; expected record sets and graph edges are independent literals from (`[A-thm:kpcfixed-interface:membership-correctness]`), never reader output.

| Test and file | Command | Claim |
|---|---|---|
| fixed_reader_matches_independent_graph — H | `cargo test -p tripod-vectors --profile gate --test keyed_fixed_history fixed_reader_matches_independent_graph` | Byte reconstruction equals the independent model and literal SOURCE/union sets; MERGE/no-handle events add no record and extending the history leaves earlier readings unchanged. |
| fixed_reader_authenticates_exact_membership — H | `cargo test -p tripod-vectors --profile gate --test keyed_fixed_history fixed_reader_authenticates_exact_membership` | Correct first-output pair/program is a handle, including a valid present nonce; wrong asset/value/form, later birth and malformed SOURCE are not handles with no record. Foreign/version substitutions at a consumed-handle claim refuse at the authenticated successor clause; fully classified unrelated inputs are not mistaken for missing evidence. |
| fixed_reader_requires_all_retrieval_fields_and_branches — H | `cargo test -p tripod-vectors --profile gate --test keyed_fixed_history fixed_reader_requires_all_retrieval_fields_and_branches` | A complete join reads; removing one branch, its parent fields or an inclusion/retrieval assertion yields the exact Incomplete/Missing result, never a SOURCE or default set. A valid path witnesses one record but is not a complete-reading API. |
| fixed_reader_refuses_invalid_identity_and_dependencies — H | `cargo test -p tripod-vectors --profile gate --test keyed_fixed_history fixed_reader_refuses_invalid_identity_and_dependencies` | Beside a consistent acyclic control, conflicting byte/address identities, duplicate consumption and a cycle are invalid with exact refusals, distinct from missing evidence. |
| fixed_reader_filters_are_source_folds — H | `cargo test -p tripod-vectors --profile gate --test keyed_fixed_history fixed_reader_filters_are_source_folds` | Ruling 155: fixed SOURCE-content predicates preserve accepted growth; changing an amount/attribution/eligibility filter may change acceptance but not membership/record sets. Destruction data at MERGE or no-handle events cannot be attributed as a new SOURCE record. |
| fixed_reader_reports_premises_without_finality — H | `cargo test -p tripod-vectors --profile gate --test keyed_fixed_history fixed_reader_reports_premises_without_finality` | Inclusion/Retrieval remain supplied premises; absent deployment finality cannot become a final report. Complete, incomplete, not-handle and invalid outcomes remain distinct at the public evidence surface. |
| fixed_reader_workloads_and_path_boundary — H | `cargo test -p tripod-vectors --profile gate --test keyed_fixed_history fixed_reader_workloads_and_path_boundary` | Fix deterministic linear and branching fixture shapes/counts for S8, recording events/edges/bytes through tracing; the branch-cut control proves path evidence cannot replace dependency closure. No partial-chain interface or runtime/complexity claim is introduced. |

Only ruling 155 reopens at S7 in (`tab:kffwork:legacy-rulings`); its filter row distinguishes the constant deposit from reader credit and fixed from changing predicates. Ruling 159 continues unchanged: the oracle stays vectors-owned. (`[A-def:kpcfixed-interface:membership-reading]`), (`[A-thm:kpcfixed-interface:membership-correctness]`) and (`[A-rem:kpcfixed-interface:dependency-closure]`) govern exact membership and complete reconstruction; (`[A-rem:kpcfixed-operations:reader-folds]`) permits application meaning as a reader-chosen fold, not a new covenant law. S7 supplies no Burn/Clear semantics, deployment finality depth or chain-witness API. S8 measurement uses the selected workload shapes; its actual scale, cold/warm run conditions and budgets remain measured evidence, not S7 performance assertions.

Both designs retain the steps' full closing readings and C/D gates as assigned, (`tab:backlog:fixed-form-paths`), the consumer census and (`rule:guide15-exec:fixed-form-settings`). First-edition frozen sources, bundles, references, pins and captures stay byte-identical. S5 has now landed: changing any of its 46 pinned whole sources needs the paired pin, real history row, complete diff/classification and affected-evidence disposition under (`rule:vectors:keyed-refreeze`); the initial synthetic stale subject is no substitute. No test selects deployment K,A,v or a finality policy. Every selected refusal names its exact error beside an accepted control; every expected byte comes from the record or a second route. Additional cases below are choices for later work, not automatic implementation scope; a discovered product defect still needs disposition. The implementation brief carries this selection without relaxing independent review or the import gate.

### S8 — Native evidence design and focused selection · `sec:backlog:fixed-form-s8-design`

This design serves req:dispatch:design-names-focused-tests and prin:environment:testing-serves-the-product under (`rule:guide15-exec:wave2-settings`). It selects implementation work for (`step:kffwork:s8`), not a native result or permission for an implementation lane to start a node. Ruling 215 at (`rule:phase7:fixed-form-rulings`) settles Q1–Q7 at (`tab:backlog:fixed-form-s8-questions`) as recommended; ruling 216 removes only Q6's separate raised-rate sessions while retaining its node, image, defaults and baseline rule. The implementation briefs carry the selected tests without enlarging the battery. Accepted rulings 193–214 retain their text. In particular, 211's migration deadline precedes the first pilot as well as the campaign, and 214's N = 1299 is a derived construction-policy bound awaiting measurement.

**Table (S8 landings and actors)** · `tab:backlog:fixed-form-s8-steps`

| Sub-step / actor | Deliverable and green boundary |
|---|---|
| S8a — implementation lane | DONE: pre-capture binding, structural subjects, harness, inventory/history transition and admitted maximum. Landed through `0.7.190-dev`, `0.7.192-dev` and `0.7.195-dev`; full gates green at 2026-11-18T09:49:24Z (1,487 s), 11:42:21Z (1,299 s) and 13:47:53Z (1,775 s). Independent core, whole-step, pilot and campaign reviews precede the campaign receipt (`rec:history:fixed-form-s8a-campaign`). The sealed pilot remains under its own identity; the first campaign sealed policy/histories but stopped clauses/scaling; the second sealed clauses but stopped scaling. Those packets remain retained at their capture-time identities, not the campaign of record. All four families then sealed from `0.7.195-dev` under one identity; the independent verdict is CAMPAIGN SOUND WITH RECORDED LIMITS. |
| S8r — independent reviewer, then loop import | Review actual S8a sources, freeze migration, one-coordinate subjects, balanced collateral, isolation and timing/retention contract; run the full import gate and record the exact reviewed tree. No capture from an unreviewed correction. |
| S8p — loop operator, then independent reviewer | Run a separately retained pilot on the available pinned node/image; review measurements and seals, including writer and post-transport wall. Derive campaign guards within the ruled budget; a required code correction returns through S8a/S8r. |
| S8n — loop operator | Execute reviewed capture families from that tree, seal settled outputs and retain failures as well as successes; no implementation worker launches these rounds. |
| S8b — implementation lane, then independent reviewer/import | IMPLEMENTED; corrected-scope gate passed before carry (`rec:phase7:fixed-form-s8b-admission`). Literal model credits: ValueMinus/ValuePlus 12, WrongProgram/WrongKey/DisplacedSuccessor 34, MalformedPaymentValue 21, LaterPosition 26; OPEN 0. Independent review/import remain; S9 is the whole-track closure. |
| S8c — separate sponsor implementation and capture under rulings 218 and 220 | Design and independently review the sponsor extension, re-freeze, then run the second capture. Ruling 220 moves the timing subscriber and watcher obligations into the campaign repair; S8c consumes their reviewed behavior. Give member-funding rows a distinct supporting census role and record per-kind subject counts in `CENSUS.json`, whose S8a form names kinds only; recheck `prepare.log` before the preparation watcher reports a unit that ended without its marker. These changes receive S8c's own re-freeze. |

S8a can exercise the complete admission API with node-free controls, while S8b adds evidence rather than leaving pre-capture tests dependent on nonexistent archives. A separate landing solely for freeze activation would freeze the capture sources before they exist and require another transition; the reviewed S8a landing binds the first capture source set once. S8c is the distinct sponsor extension and second capture selected by ruling 218; ruling 220 discharges ruling 219's timing and watcher obligations during the campaign repair. Pilots are evidence, not a way around pre-capture review.

#### S8a implementation and selection · `sec:backlog:fixed-form-s8a`

Within packages/vectors/, src/keyed_fixed_native.rs owns the frozen structural subject, signing-request/completion equality and expected/observed provenance comparison; src/keyed_fixed_capture.rs owns the finite subject census, control/subject bindings, full journal admission and serialization of the packet; src/keyed_fixed_corpus.rs supplies capture-time identity admission before capture, with retained fixture loading added only at S8b. Current capture/native binding requires the current per-triple ValidatedFreeze identity; retained corpus admission requires the recorded capture-time identity reconstructed from verified history. None, unequal and legacy values fail at each consumer, beside a progressing matched control. No archive hash replaces this comparison. New src/keyed_fixed_corpus/history.rs remains an S8b receiver.

src/keyed_fixed_manifest.rs and its pins.rs/encoding.rs, tests/keyed_fixed_freeze_history.rs, plans/registers/identities.md and plans/packages/vectors.md carry the transition: retain the S5 ordered inventory/encoding evidence, add whole-source pins and computed inventory, admit Some(1299) for the declared envelope, and append an actual history row with old/new identities, path additions, source diffs, editorial/behavioural classification and affected-evidence disposition. None remains unadmitted. An inventory change changes the value; changing framing/domain/projection requires a distinct admitted recipe, not an unnoticed v1 change. Historical capture identities are never replaced by working pins. There is no fixed-form native evidence to retain from S5; legacy observations and all legacy pins remain unchanged.

Recommended inventory additions are the six S6 transaction owners keyed_fixed_abi/request/inputs/construct/taproot/sponsor.rs; whole shared transaction sources keyed_curve.rs, taproot.rs, bytes.rs, construct.rs and sponsor.rs; the fixed capture/native owners, adapter and three wrapper sources; and the S7 reader/evidence/model with all five reader children. Pin whole files, including the consumed weight/program and signing facts in construct.rs/sponsor.rs, rather than inventing a fragment-digest grammar. Retain the existing 46 source entries, pair every changed source (including the manifest owner) with its new pin/history row, and sort/deduplicate additions; the implementation records the exact resulting inventory count and transitive-child census. The pin table represents itself through the inventory and must not hash itself. The corpus's future fixture constants are evidence consumers, not capture-input pins; actual corpus-reader source provenance is recorded separately. Resolved lock, binary, framework, zk library, toolchain and image digests remain separate provenance. Q3 below rules this explicit selected closure; it is not a claim to freeze the entire workspace.

Reuse unchanged the production constructor, target amount/weight owners, serializer/decoder, ordinary sponsor signing and public commitment arithmetic, S7 reader/model, and conformance executor/exchange transport. Use the existing confidential fixture and commitment oracle to construct real proofs. In keyed_fixed_native.rs provide a vectors-only structural subject route using TargetTransaction and explicit parent/proof/signing material for cases outside S6 admission; do not weaken S6 or pass off its refusal as native evidence. A confidential-asset output can be balanced against explicit funding with a valid surjection proof; avoid proof-bearing blinded-asset parents in complete-reader histories. If this cannot use the existing oracle, the named confidential_fixture/funding and commitment_oracle owners receive only the demonstrated extension with its focused proof control; no second prover or validator. Present-nonce/non-policy-asset subjects use structural construction, and all triples appearing in the reader's domain are supplied. Issuance/peg-in remain excluded and unsupported by the shared decoder.

scripts/keyed-fixed-capture/{capture-prepare.sh,capture-unit.sh,run-one.sh} are sibling entry wrappers following the existing keyed-final-capture lifecycle: clean reviewed checkout, uniquely selected Cargo test artifact, immutable image, offline build, isolated network-none node, separate retained deadline, verified writer/container exit and atomic seal. They select the new keyed_fixed_capture integration target, never the legacy maturity_native target. Extend elements-native-executor.py only at the existing request/readback/policy routes needed by the declared structural subjects, with the three existing executor contract tests. Preserve legacy request meanings and archive replay. Register new sources/tests and targets in the listed crate/root/script Meson and lib receivers; add tracing spans/events and flushed timeout diagnostics in README/TRACING receivers. No printed-timer exception, new extractor, generic walker or external schema is selected.

**Table (S8a focused tests)** · `tab:backlog:fixed-form-s8a-tests`

Names below allocate new tests, not completed checks. F = packages/vectors/tests/keyed_fixed_capture.rs; H = packages/vectors/tests/keyed_fixed_freeze_history.rs; W = scripts/test-keyed-fixed-capture-wrappers.py. Every refusal compares a progressing accepted control and the exact typed boundary; expected bytes/sites use the realization record, independent serialization and stack model, not the producer under test. Node-free mocks never count as observations.

| Name / file | Command | One claim |
|---|---|---|
| fixed_capture_freeze_consumers — F | `cargo test -p tripod-vectors --profile gate --test keyed_fixed_capture fixed_capture_freeze_consumers` | All three consumers reject missing, unequal and cross-edition freeze identities and bind the matched per-triple value. |
| fixed_capture_refreeze_history — H | `cargo test -p tripod-vectors --profile gate --test keyed_fixed_freeze_history fixed_capture_refreeze_history` | The real S5-to-S8 inventory transition and admitted bound replay exactly; stale source/inventory/envelope offers refuse without rewriting the old encoding. |
| fixed_capture_subject_bytes — F | `cargo test -p tripod-vectors --profile gate --test keyed_fixed_capture fixed_capture_subject_bytes` | Multiple supplied triples' SOURCE/MERGE subjects match public construction and independently predicted script/control/serialized bytes. |
| fixed_capture_structural_pairs — F | `cargo test -p tripod-vectors --profile gate --test keyed_fixed_capture fixed_capture_structural_pairs` | The explicit other-asset and v−1/v+1 model subjects balance per asset and reach the declared output site; malformed explicit parents and later birth reach their input/birth sites. Proof-bearing asset/value-form subjects remain deferred at the named proof and sponsor owners, so this model test makes no claim that their earlier native obligations pass. |
| fixed_capture_proofs_and_sponsor — F | `cargo test -p tripod-vectors --profile gate --test keyed_fixed_capture fixed_capture_proofs_and_sponsor` | Real confidential proof and tally pairs and an explicit-asset, explicit-value, null-nonce ordinary sponsor signing pair are checked separately; the combined proof-and-sponsor claim moves to S8c under ruling 218. |
| fixed_capture_policy_envelope — F | `cargo test -p tripod-vectors --profile gate --test keyed_fixed_capture fixed_capture_policy_envelope` | Default-rate M−1/M and 1299/1300 routes keep typed, relay and consensus boundaries distinct, including count-prefix and 72-byte sponsor accounting. |
| fixed_capture_journal_binding — F | `cargo test -p tripod-vectors --profile gate --test keyed_fixed_capture fixed_capture_journal_binding` | Exact request/response/parent/readback/provenance joins reject missing or substituted rows and never infer an instruction offset from generic node error text. |
| fixed_capture_workload_shapes — F | `cargo test -p tripod-vectors --profile gate --test keyed_fixed_capture fixed_capture_workload_shapes` | The native workload's graph reproduces S7's linear and branching recurrence while its own funded byte counts are retained independently. |
| wrapper lifecycle controls — W | `python3 scripts/test-keyed-fixed-capture-wrappers.py` | Stand-ins demonstrate exactly-one ignored-test selection, drift/reuse refusal, retained deadline, failed-run sealing and no successful marker while a writer survives. |
| executor structural request contracts — scripts/test-executor-request-comparisons.py, test-executor-classification.py, test-executor-boundaries.py | `python3 scripts/test-executor-request-comparisons.py`; `python3 scripts/test-executor-classification.py`; `python3 scripts/test-executor-boundaries.py` (separate commands) | The unchanged revision-nine adapter preserves wire comparison and stage classification under ruling 216; the node verdict at import is authoritative. |

S8a defers retained-native truth and measured budgets to S8p/S8n/S8b, not to mock pass bits. Exhaustive refusal permutations, arbitrary confidential collateral envelopes and generalized workload libraries remain in (`tab:backlog:fixed-form-design-followups`). Obligations still applying: rulings 193–201 and 208–212 for exact bytes, independence, both-end membership, cryptographic premises, edition separation and provenance; 163/180/214 for the declared weight envelope; (`rule:vectors:keyed-refreeze`) for every changed pin; C/D and independent pre-capture/full import gates under ruling 207. Timing assertions are report-only; a hang guard is a resource boundary, not evidence of product speed.

#### Operator capture packet · `sec:backlog:fixed-form-s8-capture`

The command contract below is implemented by the sibling wrappers, subject to S8r review before an operator uses it. Rulings 215, 216 and 218 settle its environment, default policy and staged families. The operator runs on kranich; CHECKOUT is the absolute path to a clean reviewed checkout, REVIEWED_COMMIT is its full 40-hex HEAD commit id, RUN is a fresh lower-case run name, and IMAGE is the immutable native image identity verified by preparation against the first-edition retained provenance. These are supplied provenance locators, not protocol identities. Set them explicitly in the operator dispatch; no moving branch or image tag is an expected identity.

```sh
cd "$CHECKOUT"
time systemd-run --user --quiet --collect --same-dir --unit="tripod-$RUN-prepare" --property=RuntimeMaxSec=2100 bash "$PWD/scripts/keyed-fixed-capture/capture-prepare.sh" "$REVIEWED_COMMIT" "$RUN"
time timeout 2100 bash -ec 'until grep -q "^KFF_PREPARE_DONE " "$1/prepare.log" 2>/dev/null; do state=$(systemctl --user show "tripod-$2-prepare" --property=ActiveState --value); case "$state" in active|activating|deactivating) sleep 1;; inactive|failed) echo "preparation unit ended before marker: $state" >&2; exit 74;; *) echo "unknown preparation unit state: $state" >&2; exit 74;; esac; done; grep -Fx "KFF_PREPARE_DONE run=$2 exit=0" "$1/prepare.log"; test -f "$1/retention.sha256"' bash "/var/home/tripod/loops/evidence/fixed-form/$RUN" "$RUN"
time systemd-run --user --quiet --collect --unit="tripod-$RUN-$FAMILY" --property=RuntimeMaxSec="$UNIT_SECONDS" bash "/var/home/tripod/loops/evidence/fixed-form/$RUN/capture-unit" "/var/home/tripod/loops/evidence/fixed-form/$RUN/$FAMILY" "$FAMILY" "$GUARD_SECONDS" "$IMAGE"
time bash "/var/home/tripod/loops/evidence/fixed-form/$RUN/capture-unit" --wait "/var/home/tripod/loops/evidence/fixed-form/$RUN/$FAMILY" "$FAMILY" "$GUARD_SECONDS" "$IMAGE"
```

Run these as separate commands from `CHECKOUT`, a clean checkout whose HEAD is `REVIEWED_COMMIT`; preparation checks both the commit id and cleanliness. The detached preparation unit inherits that directory through `--same-dir`, while its foreground marker watcher waits for a successful seal. The detached family unit and its foreground `capture-unit --wait` watcher run once per FAMILY in order: clauses, policy, scaling, histories. Pilot uses a separate RUN and the pilot family, never relabelled as campaign. The old-leaf legacy-copy family moves to S8c at the sponsor owner's `UnsupportedForm` boundary for outputs outside explicit asset, explicit value and null nonce. The provisional pilot/clauses/policy/histories GUARD_SECONDS = 1800 and UNIT_SECONDS = 1920; scaling = 3600/3720. run-one invokes `/capture-build/keyed_fixed_capture --ignored --exact native_$FAMILY --nocapture --test-threads=1` (S8c will separately register legacy-copy after its sponsor review). The ignored native tests are not part of ordinary Cargo/Meson gates. Preparation retains its exact Cargo artifact-selection/build command and actual resolved lock; the mounted source is read-only during capture. The preparation and capture markers are respectively `KFF_PREPARE_DONE run=... exit=0` and `KFF_CAPTURE_DONE target=... exit=0`; the watcher must also verify container absence and the final seal before declaring success. On transport interruption reconnect to the same unit/run and wait; never relaunch or renew a deadline. The foreground family watcher remains attached through completion; --wait seals after writers are absent, including failed runs.

The pilot measured 194 s for preparation and 47 s for its sealed unit: 771 exchanges and 257 subjects, including a relayed 46,727-byte 252-member MERGE. These are observations of this pilot, not estimates for the larger campaign. The first campaign run `20261118t0949z-fixed-campaign` under capture-time identity `6591fbb37cfc1fbcf1ffe14a5212a14bc919416f937dc51d314818070028c063` prepared in 154 s; policy and histories sealed in 5 s and 33 s under that identity. Clauses stopped after 27 subject records were written and retained, with none admitted and 84 exchanges: the member key-path attempt was refused with `Invalid Schnorr signature`, while the mixed sponsor input made the pinned adapter return `script_path_rejection` rather than the predicted key-path layer. Scaling stopped after 509 s and 12,052 exchanges with 4,017 subject rows written: 4,008 member-funding `PublicSource(42)` rows were absent from its admission census, and its first such row refused at `JournalRefusal::Census`. The stopped packets remain evidence of the stops and are never relabelled. The second run `20261118t1143z-fixed-campaign2` under `da23379053f2f1b7af68b7ea896cf04790ab0a5d020fd37edcc4110c89b0d6d3` sealed clauses with 32 subjects in 41 s; scaling stopped after 600.22 s, 9446 exchanges and 3148 subject records when its 600-second executor watchdog terminated the adapter and closed the planner worker's response channel. Its 7467252693-byte timing file held 24778316 lines because a process-wide DEBUG subscriber included construction events from other crates. The next pass runs clauses, policy, scaling and histories again from one reviewed commit under one new identity, so the campaign of record has per-exchange timing for every family. Policy and histories from the first run and clauses from the second stay retained as sealed evidence under their own capture-time identities; stopped packets remain stopped evidence. The next run's timing subscriber selects DEBUG and higher only for `vectors::native_test`, `vectors::keyed_fixed_capture` and `target_elements_conformance::executor`, retaining one `fixed-form native exchange retained` line per exchange plus transport/admission walls, family events, executor events and worker-side `capture freeze identity bound` lines (one per submission); admission itself emits its wall and two family events. Scaling's complete 12052 exchanges and 4017 subject rows should produce roughly 16300 lines and a few megabytes; `run-one.sh` refuses more than 2 lines per exchange plus 32, or more than 2048 bytes per exchange with an 8192-byte minimum. At 12052 exchanges those ceilings are 24136 lines and 24682496 bytes, far below the stopped file. The scaling executor's whole-run inner bound becomes 2040 s: four times the first pass's measured 509 s is 2036 s, rounded up to a minute, inside the unchanged 3600-second family guard. The first pass completed at 509 s; the second hit 600.22 s while synchronous DEBUG output reached 7.47 GB. Load was about 67–72 during preparation and clauses and 17–23 near scaling's end, so neither pass is a controlled performance baseline. Preparation's 2100-second guard exceeds four times both 154 s and 253 s; the other families' 600-second executor bound exceeds four times policy's 5 s, histories' 33 s, clauses' 5/41 s and pilot's 47 s. Their 1800-second family guards remain unchanged. The provisional S8a wall allowance after ruling 218 is 2100 s preparation + 1920 s pilot + three 1920 s families + 3720 s scaling + 1500 s sealing/inspection = 15000 s (4 h 10 m), excluding review/queue time, not a measured runtime. S8c's legacy-copy session accounts for the remaining 1920 s of ruling 216's 16920 s allowance. The original 16920-second arithmetic omitted the two separately configured raised-rate sessions even though Q6 selected them; pricing those omitted sessions at 1920 s each would have made the all-session allowance 20760 s. Ruling 216 removes both, a 3840-second reduction from that corrected all-session allowance, leaving the published family arithmetic at 16920 s. Without legacy-copy it is 15000 s. Q4 rules these planning bounds. Pilot measures preparation, each subject phase, transport, packet/archive writing, admission and seal separately, with bytes and load/cgroup scope. Scaling cannot inherit the first edition's 1082-second/3600-second fourfold-headroom failure as adequate margin: confirm the current writer path, measure a representative high-count pilot, and split bands before campaign if fourfold headroom fails. Do not silently increase a guard. New offline tests likewise retain subject/family diagnostics; measure representative isolated and loaded runs, put expected wall and report-only tolerance beside each new Meson declaration, then independently review the split before campaign. S8b publishes actual results and remaining failures of headroom.

**Table (S8 native subjects)** · `tab:backlog:fixed-form-s8-subjects`

Each negative joins a preceding relay-accepted control of its declared kind from the same preparation on disjoint unspent parents; controls are shared by kind. Value negatives and surplus destinations use the one-member MERGE control, and all acceptance credit requires `relay-and-template` with `relay_allowed=true`. The v=14/15 floor pair has separate triple freezes with the same key and asset and adjacent values. Retain all balancing changes as collateral. Expectations below are predictions to test; a different result remains evidence requiring disposition, never a regenerated expectation.

| Subject / submitted bytes | Expected outcome and refusal stage |
|---|---|
| Two test triples: public SOURCE, one-member MERGE and two-member MERGE; two ordinary surplus destinations | Relay/mining acceptance and exact readback; one retained v and `(n−1)v` available before fees, no new MERGE record or covenant destination restriction. |
| Asset identity: same program and value, other explicit output asset with separately balanced additional funding/change | Deferred from the revision-nine packet: `fund` gives no raw readback for an issued asset, and `fund_confidential` is fixture-bound; the packet cannot bind the arbitrary explicit issued-asset funding parent needed to exclude a tally refusal. |
| Value: MERGEs spending a member of the same program/asset with v−1 and v+1 at a non-minimum binding, rebalanced change | Native script equality after O; v−1 ≥ M and adequate fee prevent dust/fee pre-emption. A wrong-value SOURCE executes no leaf. |
| Asset form: valid confidential asset commitment/surjection proof for the same intended A,v, balanced collateral | Deferred from the revision-nine packet: its confidential funding arm is restricted to earlier issued assets and fixed fixture outputs, while the selected triple names the policy asset; no selected proof owner produces a surjection proof for a structural policy-asset output. |
| Value form: valid confidential value/range proof for the same intended A,v, balanced change | S8c under ruling 218: the pinned sponsor owner refuses any output outside explicit asset, explicit value and null nonce. A MERGE must spend a member to reach equality after O. |
| Malformed first-output payments: wrong explicit value and valid confidential-value form; opening-holder spend attempts with correct successor | The wrong explicit-value payment can succeed as a nonmember and its spend is predicted to refuse at equality after I with O passing. The confidential-value payment is deferred at the pinned sponsor owner's `UnsupportedForm` refusal before a signed byte exists; the explicit and confidential asset-form payments remain deferred for the issued-parent and surjection-proof limits above. Keep proof-bearing subjects outside the S7 complete-reader input when its decoder rejects them; report that boundary explicitly. |
| Exact-pair later-position payment, then attempted continuation | Creation can succeed but creates no handle/record; spend refuses at born-first equality. First-position control passes. |
| Wrong continuation program/key and displaced successor; malformed control and attempted key-path spend | Wrong program changes the version-one successor payload, wrong key selects the other key's successor program while retaining the spending leaf, and displaced successor leaves the exact explicit pair at output zero under a different version-one program while placing the true successor at output one. Thus O passes and the continuation payload check is the predicted site. Malformed control predicts a script-path refusal with witness-program-hash-mismatch detail. The member key-path attempt has a separate semantic claim, but its ordinary sponsor input makes the pinned adapter classify the node's mandatory Invalid Schnorr signature refusal as `script_path_rejection`; credit binds both that layer and the exact reason class. A failed key-path attempt does not prove absence of a discrete logarithm. Retain conditional NUMS and hash assumptions. |
| Floor: separately bound v=M−1 and v=M SOURCEs, policy asset, empty nonce, one dust candidate, positive adequate fee, require_standard enabled | At default R=100 predict v=14 refused by relay for dust and then accepted through the adapter's direct-block route (`relay_allowed=false`, `route=direct-block`); v=15 relays. The retained `relay_detail` supplies a reject reason, not an opcode or debug stage. The zero-fee v=14 probe omits the explicit fee output, so no zero-valued output can supply a second dust reason; it has the same adapter route and is credited only as a fee-policy relay refusal. |
| Qualified floor controls: present-nonce values 17/18; explicit non-policy asset v=1 with policy-asset fee funding | The present-nonce pair is OPEN and deferred beyond S8c under amended ruling 218: sendrawtransaction refuses this explicit-value form before broadcast; the source calculation stands. The non-policy-asset subject remains deferred because the adapter cannot return exact raw issued-asset funding bytes. Raised-rate sessions remain deferred under ruling 216. |
| Weight range in scaling: one zero-member SOURCE control and MERGE at 1,2,251,252,253,650,1299,1300 members; same declared three-output envelope and bounded sponsor | SOURCE weighs at most 1,077 WU under a 72-byte sponsor signature. Through 1299 the typed constructor builds the SOURCE and MERGEs; independently serialized weights agree with the formula and node, and 1299 is expected to relay. The family also journals ordinary `PublicSource(42)` transactions that fund each MERGE member; these are census subjects but not extra weight controls. At 1300 the typed constructor refuses before bytes with its construction-policy refusal. One independently serialized structural 1300 yields the node's expected standard-weight relay refusal and its isolated direct-block consensus acceptance (`relay_allowed=false`, `route=direct-block`). The constructor, relay and consensus verdicts remain separate. Retain actual sponsor signature length and worst-72-byte upper bound, never pad an invalid signature. |
| Contention: competing spends, replacement and retry | Deferred under ruling 218: the pinned adapter mines each accepted submission before returning, so S8b's `fixed_corpus_contention` has no S8a evidence source pending an adapter revision. |
| Full-closure histories: funded S7 four-event linear and branch/join/extension shapes, then larger fixed repetitions; cold/warm reads and branch omission | Native accepted-byte histories read equal to model/literal sets; omitted branch is Incomplete. Record events/edges/unique bytes/records and read wall, separate retrieval/decode/fold/report spans. No finality without supplied policy/depth. |
| Old-leaf copy controls (Q2): explicit asset/confidential value, opening-holder continuation and separate public-only process copying script/commitment/range proof with its own sponsor | S8c under ruling 218, with its own freeze and archive route. The fixed-form sponsor owner must verify its ordinary sponsor; the first edition's pinned sponsor source stays unchanged. The current owner refuses any output outside explicit asset, explicit value and null nonce. |

All four successor coordinate negatives share the same equality instruction after O. Node OP_EQUALVERIFY text contains no offset: attribute only with paired independently balanced subjects, the bounded stack model and exclusion of earlier/sponsor failures; otherwise mark OPEN. Do not promote bad-txns-in-ne-out, dust, missing inputs or double-spend failures to script evidence. S7's decoder refuses issuance, peg-in and proof-bearing blinded-asset parents for the whole read; preserve those native packets but do not claim full reading over unsupported bytes. Exact-pair present-nonce and non-policy-asset readable subjects remain members regardless of S6's narrower envelope.

Retain full submitted/witness and parent/proof bytes, txid/wtxid and block/readback bytes, triples, leaf/control/descriptors, sponsor request/signature and capability separation, each RPC request/response and debug-stage evidence, fee/weight, startup/configuration and effective settings with observed-versus-source-default origins. Retain clean source snapshot/revision, compiled-in per-triple freeze encoding/identity, expected and observed provenance, actual binary and resolved Cargo.lock, Rust/Cargo/Python, adapter/protocol, Elements executable/source/framework/configuration, zk-library and immutable image identities. Each family seals `observed-provenance.json` with the in-container file digests and the zk library's loading status, `executor-handshake.json` with the typed adapter report, and `phase-timing.jsonl` with transport and admission walls in microseconds and family and executor events; filtered-subscriber sessions add one worker-side `capture freeze identity bound` line per submission, while admission itself emits its wall and two family events. Policy and histories sealed under the first campaign identity above retain ruling 219's calling-thread timing scope: their per-exchange timing comes from sub-second exchange file times in pax archives. Ruling 220 gives later runs a process-wide subscriber reaching the planner worker; the subscriber selects only native-test, capture-admission and conformance-executor targets at DEBUG and higher, and the wrapper requires one per-exchange timing line per retained exchange plus bounded timing bytes and lines. The four runtime file digests are computed once per family and reused for every journal row. `debug.log` and `node.out` are copied from the isolated regtest directory into the family seal after container exit; `node-log-copies.json` identifies their source paths. The original regtest directory stays outside the seal. Seal raw journals, completed exchanges, manifest/census, stdout/stderr, exits, start/end/deadline, phase/resource samples and exact commands; retain failed/timeout packets too. Binding joins expected identity to the binary's self-report and the journal, not proof that a self-reported executable ran. Cgroup counters label actual sampled path and whether container inclusion was verified against `container-cgroup.txt`. The container's isolated HOME may acquire a default `liquidv1` folder from the adapter's `elementsd --version` probe, which runs without a datadir argument; it is not the configured regtest node and carries no chain log. Native interpreter stack peaks, per-opcode charge/CPU/stalls and production finality remain absent unless separately instrumented; whole-process time supplies none of them.

#### S8b implementation and selection · `sec:backlog:fixed-form-s8b`

S8b admits only campaign3 at (`rec:history:fixed-form-s8a-campaign`), from retained bytes without a node. This live design replaces the pre-campaign selection under (`rule:guide15-exec:fixed-form-settings`); its dated reference packet is (`sec:s8b:study`) in [the S8b admission study](drafts/keyed-fixed-s8b-admission-study.md). That archive-class study records the measured fixture sizing, grammar, complete join table, attribution predictions, exact 26-path audit and implementation budget; this selection adopts those details without making the archive an independent authority. No production contract changes, new extraction tool or native run are selected. Earlier pilot/pass packets remain separately retained outside the corpus at their original identities, never relabelled as campaign3.

Extend keyed_fixed_corpus.rs and its new history.rs child with bounded indexed-LZMA fixtures preserving exact retained file bytes. Eight fixtures replace the former CENSUS/subjects allocation: CENSUS.json, clauses/policy/scaling/histories/root/preparation/sources.json.lzma, explicitly enumerated at (`tab:s8b:fixtures`). The measured payload candidate is 21,219,940 bytes plus at most 32 KiB of index; scaling is 19,997,541 compressed bytes, 251,202,490 decoded bytes across 32 members, largest 16,797,132. Keep the original six digest-addressed archives outside the tree, including duplicated transport aggregates and four logs per family; preserve exact seals and enumerate omitted bytes without claiming they were rehashed. Register exact census/content and capture-time identity subjects in the existing lint registry, no directory exemption. Bound decoding to 32,000,000 bytes per member and the existing 16 MiB LZMA memory limit.

The retained encoding decoder extends keyed_fixed_manifest/encoding.rs, with authenticated historical complete-freeze reconstruction in keyed_fixed_manifest.rs. Rebuild the descriptor from its triple/recipe, validate framing/domain/inventory/codes/envelope, re-encode exactly and bind the independently reviewed capture identity and historical source bytes. Keep current validation distinct. These two pinned sources require paired pins, old/new history, retained pre-change sources and independent review under (`rule:vectors:keyed-refreeze`); this is necessary to feed historical ValidatedFreeze to the unchanged journal API, not permission to alter capture behavior. No legacy pin changes. Current/history validation equivalence and consumer admission reopen; native observations remain only if submitted bytes and premises stay unchanged. The full grammar and changed-byte controls are (`sec:s8b:decoder`).

Before credit, perform every comparison at (`tab:s8b:joins`): seal/preparation, each triple, actual census counts, wire/parent/readback bytes, sponsor message/signature and fund/sign/submit adjacency, preparation provenance, separately resolved earlier same-family/run controls with disjoint inputs, declared subject/control relation, key-path member witness, outcome, history and measurement joins. Reuse bind_journal unchanged after corpus checks. CENSUS kinds are not counts: derive all 4220 records and 12667 exchanges, including 4008 scaling supporting SOURCE rows and two clauses payments outside the subject total. Floor controls validate each side's own 14/15 freeze. KeyPathAttempt requires its authenticated version-one member parent paying the frozen program and exactly one 64-byte witness item; generic script-layer/Schnorr RPC text alone supplies no witness shape.

Rebuild WrongKey's K=43/v=60 foreign program from the recipe; replay typed 1300 from its retained member parents and exchange 12049 sponsor, beside typed 1299; keep structural 1300 relay refusal/direct-block acceptance separate. Rebuild all six history graphs and literal SOURCE memberships using unchanged S7 reader/model, with ordinary funding closure and evidence-only inclusion/retrieval, then repeat the missing-event-2 Incomplete result. Seven opcode-only refusals receive exact equality credit only if the bounded model uniquely attributes the retained paired bytes; predictions and OPEN fallback are (`tab:s8b:rebuild`). An RPC opcode class never becomes a native instruction trace.

**Table (S8b focused tests)** · `tab:backlog:fixed-form-s8b-tests`

F is new packages/vectors/tests/keyed_fixed_corpus.rs, deliberately separate from the pinned capture test. Each command is `cargo test -p tripod-vectors --profile gate --test keyed_fixed_corpus NAME`, substituting the exact name below. Tests are offline; mutation controls reach the owning inner boundary after acquisition so an outer digest failure is not credited as a join test. Small table entries stay within these five tests; further subjects remain follow-ups.

| Name / file | Accepted control | Changed subject and owning refusal |
|---|---|---|
| fixed_corpus_seal_and_identity — F | Four exact campaign encodings round-trip and admit after S8b re-pin; original bounded fixture/prepare seal passes | Changed compressed byte → acquisition digest; changed encoded source byte → historical Inventory; bounded framing/domain/code/descriptor/envelope entries → decoder/manifest refusal; current validation still refuses historical pins |
| fixed_corpus_exact_native_pairs — F | The 220 retained complete rows re-admit once, all 4,000 parent-only bytes authenticate, two payment submissions and the funded-coin census agree, and paired controls are evaluated by the bounded model | Changed parent byte, control step, provenance digest/image, reported sponsor witness, refused txid, payment relay flag, key-path witness or census kind → owning corpus refusal; seven retained equality subjects reach their uniquely attributed model sites or remain OPEN |
| fixed_corpus_history_closure — F | Six byte-derived graphs agree with literal SOURCE sets and independent S7 model; complete four-event branch | Remove event 2 bytes while retaining association → Incomplete, never complete membership; no failed candidate becomes an event |
| fixed_corpus_resources — F | Typed 1299 construction, independently serialized weights and retained floor 15 relay acceptance | Add member 1300 → existing typed policy refusal before bytes; retained structural 1300 and floor 14 independently retain relay refusal/direct-block acceptance, not consensus refusal |
| fixed_corpus_reading_measurements — F | Retained exchange timing prefixes, line totals and cgroup scope agree; history samples and edge counts are checked by fixed_corpus_history_closure | Remove one exchange timing prefix → corpus measurement-completeness refusal; no speed assertion or universal complexity credit |

The five receipt limits are dispositioned individually at (`tab:s8b:limits`): preparation comparison narrows the execution-provenance limit but cannot prove which executable ran; model replay may discharge individual equality sites; fresh readings/admissions discharge reproducibility but do not invent original traces; RPC-only refusal evidence remains RPC-only; scaling's failed fourfold capture headroom remains a limit for the next capture. The sponsor/confidential/present-nonce/other-asset, legacy-copy, raised-rate and contention obligations remain with rulings 216/218/220 and S8c; fixed_corpus_contention is not implemented without winner/retry evidence. S9 remains independent closure.

Implementation uses exactly (`tab:s8b:paths`), 16 existing and ten new paths, with 1020–1620 authored changed lines estimated. Five offline tests are expected to take 170–290 s per profile, anchored by the recorded 142.34 s scaling admission, with a 900 s full-admission child guard and existing 1800 s Meson target guard; measure actual wall/peak rather than treating estimates as passes. Full admission runs once, small negative controls do not repeat scaling. The gate needs all fixtures, current/history/legacy freeze checks, formatting/Clippy, selected vectors tests and full import gate plus independent review, no node/network. The dated study contains precise registrations and the two-source re-pin reason; any additional pinned edit or uncovered contract requires its own reviewed amendment.

No new ruling blocks this design. If model attribution fails, recommend retaining that site OPEN rather than expanding the model or native scope; if a future capture must meet fourfold scaling headroom, recommend splitting measured loaded-run bands rather than silently raising guards. These conditional questions and their reasons are recorded at (`tab:s8b:limits`). Random DAGs, adversarial scaling, additional refusal permutations, application folds, chain-witness API, broader sponsor envelopes and production finality remain in (`tab:backlog:fixed-form-design-followups`).

**S8b retention ruling and implementation receipt** · `rec:backlog:fixed-form-s8b-retention`. Ruling 237 selects the bytes needed by the joins for permanent checkout retention. Scaling keeps 17 complete records (the weight control, seven MERGEs, structural 1300 and eight supporting SOURCE exemplars), 52 exchanges including the typed-1300 funding exchange, and 4,000 further supporting SOURCE transaction files as member-parent bytes. The 4,000 remaining records and 12,000 exchanges stay at the checked archive digest and are named as archive-addressed by the pinned census and whole seal; they are not claimed as secondarily re-admitted. The admission test checks 220 complete records across all families and authenticates all 4,000 parent-only bytes against retained MERGE parents/outpoints. This preserves the node campaign's earlier 4,220-record admission without keeping repeated JSON arrays in every clone. The 4,000 omitted funding rows repeat one shape, were admitted on the node and independently checked from their bytes, authenticate themselves as MERGE parents, and would otherwise live in every clone. A join requiring an omitted file reopens ruling 237. The fixture loader is test-only at packages/vectors/src/keyed_fixed_corpus/fixture.rs, an unpinned child added by the implementation amendment; the library corpus module keeps only the capture-freeze binding. Preset 7 failed the reader's 16 MiB bound; preset 6 is the next strongest candidate, with the final bounded verdict and results recorded at (`rec:phase7:fixed-form-s8b-admission`). The line allocation is raised by the same amendment; report its final measured figure.

#### S8c sponsor and second-capture design · `sec:backlog:fixed-form-s8c`

Under rulings 218/220/237, S8c selects the dated [sponsor study](drafts/keyed-fixed-s8c-sponsor-study.md), (`sec:s8c:study`), as this live design's reference detail. It specifies the product boundary, source-history mechanism, exact 36-path list, joins, six focused tests and operator lifecycle. The study is archive-class design input; results remain in this live selection and the capture/admission receipts. No adapter, shared decoder, first-edition sponsor or production reader change is selected.

keyed_fixed_sponsor.rs admits explicit-asset outputs with committed or explicit values and null or present nonces, while keeping the sponsor spent output explicit/null-nonce P2WPKH, canonical stripped bytes and existing SIGHASH_ALL/DER/low-S/key checks. Existing serialization supplies hashOutputs unchanged. Output range proofs are not signed: proof/tally verification remains separate from signature verification. Replay S8b's 220 retained complete records and supporting payment signatures bit-identically; their archive-addressed omissions receive no new replay claim. The selected submitted-signature mutation now exercises InvalidSignature because this step changes the sponsor's admitted domain. Keep that owner unchanged after the nonce failure: independent digest/valid-signature controls now cover all four value/nonce combinations, with a one-coordinate nonce mutation refusing InvalidSignature. This credits signing only, not submission eligibility; the leaf still ignores nonce and its canonical null-nonce floor is not a universal relay promise.

The nine pinned edits land with their old bytes, paired pins/history and independent review. Replace the historical route's two-path/current-byte assumption with reviewed epoch source maps and per-triple anchor lists, resolved by (path,digest) snapshots and current bytes only on exact digest equality. Parameterize decoding by the trusted epoch count, preserving grammar and campaign3 identities. Keep S8c's three fixed-form identities separate from campaign3 and from the legacy semantic identity. Inventory stays 98; the next reviewed source transition adds data without rebuilding the resolver. Source identity, sponsor behavior, capture lifecycle and historical equivalence reopen; old observations stand only with successful replay and unchanged premises. Details: (`sec:s8c:history`).

The adopted corrected capture is nine records/27 fund-sign-submit exchanges: sponsor six/18 (three primary, two supporting member SOURCEs and one supporting confidential payment), legacy-copy three/nine. Remove PresentNonceFloor(18)/(17) without substitution: the reference sendrawtransaction rejects the explicit-value/present-nonce form before broadcast. A fresh disjoint accepted MERGE still controls the two confidential-value refusals. The source-derived present-nonce dust calculation stands, but its native adjacent pair remains OPEN; null-nonce 14/15 from campaign3 and a committed-value payment cannot supply it. The nonce disposition at (`sec:s8c:nonce-disposition`) implements amended ruling 218. Rerun fresh preparation and both families whole; failed run 20261119t1608z-sponsor-campaign is diagnostic evidence only, never a prefix admitted into the new corpus. Census counts/roles, refusal txids, accepted readbacks and same-family controls remain required; (`tab:s8c:capture`) and (`tab:s8c:joins`) now describe the reduced selection.

One detached campaign unit owns preparation and the two families sequentially through capture-unit.sh's selected --campaign/--wait-campaign modes, with a foreground waiter to its final marker. It rechecks prepare.log after observing terminal state before declaring a missing marker, preserves the original deadline across reconnect and awaiting-start, and seals only after writers/containers are absent. Ruling 220's filtered worker tracing and proportional timing limits remain. Expected warm wall is 185–405 s; retained bounds are preparation 2100 s, each family executor 600 s/container 1800 s plus 120 s cleanup, and 120 s final seal, totaling 6060 s. Report measured fourfold headroom; never silently raise a guard. The operator invocation and seal contract are (`sec:s8c:driver`).

Ruling 237 retains only bytes consumed by offline joins, using S8b's test-only indexed-LZMA loader and preset 6. Four new fixtures are CENSUS.json, sponsor.json.lzma, legacy-copy.json.lzma and preparation.json.lzma under packages/vectors/fixtures/keyed-fixed-sponsor/; update only the old source-history fixture/index for new snapshots. Successful fixture budgets total 572 KiB; the correction allows up to 128 KiB incremental source snapshots above the measured existing history fixture. Full archives, verbose logs, transport aggregates and executable remain digest-addressed. Measure exact final sizes and omissions; no second hash claim for omitted files. Budgets and byte-to-join inventory are (`tab:s8c:fixtures`).

Six focused tests at (`tab:s8c:tests`) cover output-form admission, the combined proof/tally/signature boundary, generic epoch source resolution, live packet identity/census controls, exact offline admission including submitted-signature/copy mutations, and one-unit marker/timing lifecycle. Existing campaign3 and freeze-history tests remain regressions. Allocate 1600–2400 authored changed lines; expect 20–40 s total warmed offline Rust work per profile including old replay, plus existing wrapper stand-ins. Drain child tracing while waiting. The full implementation/import gate and two reviews remain required; this design's document gate is not evidence those tests pass.

Review placement follows (`rule:phase7:wave3-review-placement`): one independent implementation review after its gate and before capture covers product, re-freeze and harness together; one after capture/offline admission covers evidence, projection and lasting claims. No third standing review is justified; S9 remains separate closure. Amended ruling 218 adopts the nonce disposition. Unexpected native boundaries or a required shared proof-decoder extension return to the owning review/ruling boundary with the evidence retained, not an automatic expansion. Asset/other-asset funding, raised-rate, contention, broader sponsor forms and further proof/signature permutations remain follow-ups. An outside-track contract change stops for renewed adoption.

The nonce correction retains each typed response and timing immediately after receipt, before case/shape/weight/identity comparisons; a mismatch still fails the family. Its four pinned edits, immutable failed/corrected epochs, retained old source bytes, exact path delta and focused controls are at (`sec:s8c:nonce-disposition`). Require paired re-freeze/history and a focused return to the independent implementation reviewer before recapture under (`rule:phase7:wave3-review-placement`); the capture premise changed, so the earlier review plus a loop-only diff read is insufficient. The evidence/admission review remains after successful recapture. No shared-adapter, sponsor production, leaf, constructor or reader change is selected.

**S8c pre-capture implementation status** · `rec:backlog:fixed-form-s8c-pre-capture`. The fixed-form sponsor extension, nine-source re-freeze, epoch resolver, two-family producer, live identity/census checks, one-unit driver and test-only packet join are implemented for independent review before capture. The four new sponsor fixtures and `fixed_corpus_sponsor_admission` await the reviewed node run; no new native row is claimed here. The source and gate figures are recorded at (`rec:vectors:keyed-fixed-s8c-refreeze`) and (`rec:phase7:fixed-form-s8c-pre-capture`). The selected follow-ups and limits remain those at (`tab:s8c:tests`) and (`sec:s8c:budget`).

The implementation review's five corrections are in the pre-capture tree: the payment adjacency step and full mocked eleven-record packet control, immutable Sponsor epoch map and paired review re-freeze, four computed offline joins, retained payment-signature verification and corrected legacy identity sentence. The producer's payment check reads its input outpoint index. The review transition figures and remaining second-capture work are at (`rec:vectors:keyed-fixed-s8c-refreeze`) and (`rec:phase7:fixed-form-s8c-review-correction`).

**S8c nonce-disposition correction status** · `rec:backlog:fixed-form-s8c-nonce-correction`. The failed `20261119t1608z-sponsor-campaign` run is diagnostic only. The producer now selects six sponsor records/18 exchanges and three legacy-copy records/nine exchanges; both present-nonce floor submissions are removed. Every typed response and timing is retained before comparison. Four paired source re-pins and a distinct corrected epoch preserve the failed epoch and every earlier inventory. The mocked producer packet admits six and three records through the checked loader. The corrected source and receipt are at (`rec:vectors:keyed-fixed-s8c-refreeze`) and (`rec:phase7:fixed-form-s8c-nonce-correction`); independent implementation review and a fresh full capture remain next. The present-nonce adjacent floor is OPEN.

The review's producer-census finding is corrected before recapture: the native writer derives all five counts from the six selected sponsor subjects, its checker compares all five, and the gated mocked packet uses that writer and checker before offline admission. One paired source re-pin and its identity movement are at (`rec:vectors:keyed-fixed-s8c-refreeze`); the receipt is (`rec:phase7:fixed-form-s8c-census-correction`). The two live README claims now say nine rows and keep the native present-nonce floor OPEN. A fresh full capture remains pending.

**S8c capture admission status** · `rec:backlog:fixed-form-s8c-admission`. The reviewed second unit `20261120t0650z-sponsor-campaign2` sealed nine records/27 exchanges; the checked projection admits six sponsor and three legacy-copy rows. The exact archive addresses, per-row outcomes, fixture bytes, qualified claims and offline gate are at (`rec:phase7:fixed-form-s8c-admission`) and (`rec:history:fixed-form-s8c-admission`). The present-nonce adjacent floor stays OPEN; evidence review and import remain next.

**Table (S8 questions settled by ruling 215)** · `tab:backlog:fixed-form-s8-questions`

| Question answerable in one sentence | Recommendation and reason |
|---|---|
| Q1 — Which test triples shall the capture use? | Use K = 32 bytes of 42 and 32 bytes of 43, the observed policy asset A, v=60 and v=1000 respectively for ordinary cases; derive separate floor triples at M−1/M and a structurally funded non-policy-asset triple for qualification. They vary K/v and exercise A separately without selecting deployment defaults. |
| Q2 — Do old-leaf copy controls run here or in a separate capture? | Run a separate legacy-copy family in this operator packet with its own freeze/provenance/archives and isolated public process; keep it outside fixed-form admission but finish the observation-table obligation before S9. |
| Q3 — Shall the re-freeze pin the whole shared files and capture/reader instruments listed in S8a? | Yes: whole-file pins fit the existing grammar and expose shared byte/commitment drift; record corpus-consumer source separately to avoid fixture/self-identity cycles. Any additional consumed source found by the explicit census is a packet correction before capture. |
| Q4 — Accept the provisional 16920-second total allowance and 1800/3600-second family guards, subject to pilot review and fourfold measured headroom? | Yes as planning limits only; split expensive families/bands after measurement instead of silently widening a guard, and retain breached pilot results. Before measurements there is no justified numeric regression tolerance; use report-only comparison and record load. |
| Q5 — Reproduce S7 shapes inside the capture owner at 4/16/64 events, one cold and five warm reads, rather than create a shared workload library? | Yes: it consumes the existing S7 reader/model unchanged, bounds the initial experiment and keeps native funded bytes distinct from offline seed bytes. |
| Q6 — May the operator use the available first-edition node/image with pinned defaults, fresh isolated regtest, and separate raised-dust-rate sessions? | Yes, after matching daemon/framework/zk-library/image provenance to the retained first-edition archives and recording actual toolchain/adapter drift; require standardness for floor subjects, observe default replacement and effective weight policy, and stop for review if the baseline differs. Availability is supplied by this brief, not inferred from the tree. |
| Q7 — Shall structural asset-form subjects use explicit funding with valid confidential outputs, and exclude unsupported parents from full-reader histories? | Yes: existing proof/serialization owners avoid widening S6 and prevent the shared decoder's unsupported-form refusal from masquerading as a complete reading; unsupported subjects retain separate native evidence. |

No missing deployment triple or finality policy blocks this design: ruling 215 selects test subjects, and all readings remain nonfinal. Actual daemon/image equality, effective policy and walls are measured prerequisites for capture, not facts invented by the design.

**Table (Fixed-form design follow-ups)** · `tab:backlog:fixed-form-design-followups`

| Step / deferred case | What it would show |
|---|---|
| S6 — exhaustive request/member refusal permutations | Would broaden boundary precedence and coordinate coverage beyond the selected authentic-parent and typed-value controls; the S5 F1/F4 follow-up remains at (`tab:backlog:wave1-followups`). |
| S6 — additional literal commitment triples and weight combinations | Would broaden independent commitment and multi-member arithmetic examples beyond the selected record triple and endpoint envelope; S5 F2/F5 remain choices, not prerequisites added here. |
| S6/S8 — native dust, amount, N/N+1 and worst-witness endpoints | S8 owes observations on final bytes with effective policy and adequate funding; structural construction and derived bounds here do not establish native acceptance or consensus impossibility. |
| S8 — raised dust rates 413/414 at v=60 | Would show natively that the floor follows a supplied dust rate; the constructor's calculation test already shows the arithmetic only. Requires a new adapter protocol revision, its own request-comparison fixture and contract, and migrated consumers before capture. Deferred by ruling 216; the shared revision-nine adapter stays pinned. |
| S8 — competing spends, replacement and winner retry | Would show the effective mempool replacement/fee settings and a retry against the observed winner, with branch-superset comparison. The pinned adapter mines every accepted `submit` before answering, so it cannot retain two competing spends in the mempool or observe replacement; these subjects need a reviewed adapter revision and matching fixture/contract/consumer migration. The histories family still captures complete linear and branch/join/extension graphs. |
| S8 — explicit other-asset and blinded-asset subjects | Would show native asset equality and the other-asset dust bypass with independently balanced funding and valid surjection proofs. Revision-nine `fund` has no raw readback for an arbitrary issued-asset parent, `fund_confidential` is fixture-bound to an earlier issued asset, and the selected proof owner only materializes hybrid confidential values; a reviewed adapter revision and asset-proof owner are needed before these rows can be credited. The deferred subjects are wrong output asset, confidential output asset, malformed payment asset and asset form, and other-asset floor. |
| S8 — proof-bearing output with ordinary sponsor and old-leaf copy | Selected by (`sec:backlog:fixed-form-s8c`): combined proof/tally/signature controls and separately admitted old-leaf public copy. Broader asset and sponsor-spent forms remain outside this extension. |
| S8c — sponsor extension and second capture under rulings 218 and 220 | Selected live design at (`sec:backlog:fixed-form-s8c`), reference joins/tests/paths at (`sec:s8c:study`). The corrected capture selects nine records/27 exchanges; the present-nonce adjacent floor remains OPEN, and asset/adapter and contention subjects remain deferred. |
| S8c — additional cryptographic/proof permutations | Hash flags, malformed-DER permutations, arbitrary assets/surjection proofs and more proof mutations remain follow-ups; the submitted-signature mutation is selected at (`tab:s8c:tests`). |
| S8b review — stopped-packet admission control | Would turn representative retained clauses and scaling stop rows into fixtures and check the repaired census and layer joins against their exact bytes, including the member input's key-path witness shape, after capture-time freeze reconstruction. It adds no native observation and does not relabel either stopped family. |
| S8 review — node-free adapter-answer replay | Would exercise each native entry against canned accepted relay/template, accepted direct-block, script-path, key-path and consensus answers, exposing prediction/construction contradictions before a node pilot. It is separate from the selected ten tests. |
| S8 review — non-current historical freeze reconstruction | Would show `reconstruct_capture_history` admitting a retained mapping and matching bytes after a later re-freeze while rejecting substituted history; the current test covers only the current state. |
| S8 review B1–B7 | Would isolate the remaining journal shape arms, retained baseline drift and watcher refusal, binary digest, one-coordinate envelope offers, literal descriptor values and workload literal comparisons; they are review follow-ups outside the selected ten tests. |
| S6/S8 — additional sponsor, asset and confidential collateral envelopes | Would establish supported construction and funding beyond the declared canonical envelope; unrelated covenant-valid outputs alone do not establish constructor support. |
| S7 — random DAGs, all permutations and adversarial size scaling | Would broaden recurrence/order robustness and measure costs beyond the focused join fixture; no randomized framework or performance gate is selected. |
| S7/S8 — cold/warm linear and branching measurement | S8 owes counts/bytes/timings for the fixed S7 workloads and measured budgets; S7 correctness does not claim bounded complexity or measured throughput. |
| S7 — application folds beyond the selected SOURCE predicates | Would exercise other reader-chosen meanings without changing membership or record sets; no new application semantics are assigned. |
| S7 — chain-witness API | Remains deferred by (`step:kffwork:s7`); a future proposal would specify membership-witness versus complete-closure claims and resource bounds before any interface is selected. |
| S7 review B1 — remaining refusal subjects | Later birth, malformed parent pair, wrong successor pair, scriptSig, extra argument, leaf/control mismatches, unassociated bytes, missing output, malformed or unsupported bytes and invalid triple would each show its typed boundary beside a valid control. |
| S7 review B2 — invalid model subjects | Exercising the independent model's eight refusal arms would compare graph and clause failures as well as successful sets. |
| S7 review B3 — missing membership/event | A handle parent supplied only by retrieval and an associated parent event without bytes would isolate `Missing::Membership` and propagated `Missing::Event`. |
| S7 review B4 — present-nonce MERGE | A present-nonce parent and successor would show nonce ignored at both ends of a consumption. |
| S7 review B5 — cross-event duplicate spend | Two events spending one outpoint would complement the within-event duplicate-consumption control. |
| S7 review B6 — workload library owner | A shared owner for fixed linear and branching bytes would let S8 consume them directly; choose its placement against S8's capture owner before adding a mechanism. |
| S7 review B7 — decoder-domain subjects | A member issuance input and a proof-bearing blinded-asset funding parent would demonstrate `UnsupportedForm` at the shared decoder boundary. |
| S6 review B1 — literal commitment | A record-route program/control literal would pin the surface triple and two-member witness independently of the vectors constructor. |
| S6 review B2 — confidential asset | A confidential-asset parent would show `WrongMemberForm` at the asset arm rather than accepting a form the leaf refuses. |
| S6 review B3 — parent edges | Noncanonical-parent and missing-output subjects would show both authentication refusals at the input boundary. |
| S6 review B4 — distinct dust rate | An admission subject with dust and fee rates differing would show that the output floor follows the dust rate. |
| S6 review B5 — varied reproduction triple | Reproduction triples varying A and v as well as K would broaden independent coordinate coverage. |
| S6 review B6 — adjacent aggregate bound | Balanced inputs at the aggregate maximum and one above would isolate the exact running-total edge. |
| S6 review B7 — foreign handle collateral | A structural spend of another triple's handle as collateral would show structural acceptance beside admission's envelope refusal. |
| S6 review B8 — signature binding | A signer echoing other bytes would show the `SignatureBinding` refusal distinct from invalid ECDSA. |
| S6 review B9 — shared sponsor verifier | Delegating the copied verifier to the first edition's pinned owner would make fixes shared, while introducing a dependency on that frozen source. |

**Table (Verified fixed-form step path lists)** · `tab:backlog:fixed-form-paths`

S8c correction: (`tab:s8c:paths`) is the exact 36-path implementation receiver list at `0.7.207-dev` (31 existing, five new), including nine pinned sources with paired history. It supersedes S8c portions of the historical S8 union below; no global historical count is presented as a current union census. The separate design-document receivers are listed in (`sec:s8c:study`).

S8b design correction: (`tab:s8b:paths`) is the exact writable subset for offline admission, superseding S8b allocations in the historical S8 union below. Replacing subjects.json.lzma with seven named payloads and adding the separate offline test adds seven paths to that union; its historical 250 entries/178 distinct become 257/185, with 90 new allocations/95 existing at adoption and the E-role union unchanged. The four design-document receivers are separately named in the study and do not authorize implementation edits.

S8 design correction: its 37 paths become 46 (34 E, 12 N) by adding the existing backlog, vectors plan, phase-7 engine-history and phase-7 ruling receivers, four new sibling wrapper/contract-test files, the capture test native child, and packages/vectors/Cargo.toml. The whole sequence adds nine entries and five distinct paths: 250 entries, 178 distinct paths, 83 new allocations and 95 existing at adoption; the E-role union is 102. The manifest correction moves the already declared `elements` workspace dependency from dev-only to the vectors library so the existing reference serializer can bind proof-bearing structural parents; it creates no new interface or contract under (`rule:guide15-exec:fixed-form-settings`). The phase-7 ruling receiver records rulings 215–216 and 218; the native child holds the selected operator-only family entries. These are writable receivers, not the source-pin inventory: shared construction/commitment/byte and S7 reader sources are read-only inputs to the re-freeze, not authorization to edit them. All nine paths are assigned in (`sec:backlog:fixed-form-s8-design`); no registration mechanism is added. The separate legacy-copy archive moves to S8c under ruling 218.

The expanded list has 250 entries and 178 distinct paths: S6 adds phase-card and realization-record receivers without a new distinct path or E-role member; 83 new allocations and 95 existing at adoption. S4 adds the backlog receiver for this correction, engine_attestation/program.rs for the compile-forced foreign-lowering refusal arm, and three sibling assignment/refusal test files; these preserve the shared kernels and attestation policy. S4 also adds tests/public_api.rs to extend the existing private-owner surface guard for the new binding-error re-export; its fixed-form-only checks are disjoint from Wave-2 Burn declarations. S4 adds src/tests/engine_cases_tests.rs to extend the existing complete eligibility comparison owner under ruling 178, with no new interface or kernel change. S3 includes plans/packages/vectors.md for the source re-pin record required by (`rule:vectors:keyed-refreeze`); this corrects a packet census omission and adds no distinct path. S3 also records ruling 212 through the backlog, guide and phase card; those three existing receivers add three entries and no distinct paths. S6 already includes keyed_fixed_construct.rs and tests/keyed_fixed_surface.rs for the ceiling controls, so needs no additional path. The E-role census has 102 distinct paths: the two docs/lineage Markdown files are N at S2 and later E receivers, and the fixed-form manifest, pins, encoding and freeze-history test are N at S5 and E at S8. S2 includes plans/registers/identities.md for its pre-admission record, plans/backlog.md for packet corrections, and the phase card, guide and plans/packages/realization.md for ruling 211 and its adopted ordering. S5/S8 include the identity register, owned encoding and freeze-history test for admission and consumer activation; these extend the existing vectors freeze owner, not a new interface or contract. S5 inventories the fixed-form architecture/declarations, compiler, tapscript/linker and vectors freeze/constructor sources produced through S5, together with their shared recipe dependencies, as an explicit source list; it does not claim later construction or capture sources. S8 owns the reviewed pre-capture re-freeze that adds S6's construction and commitment sources and any later sources used by the capture binding, updates the computed inventory and pins, and supplies history controls and affected-evidence dispositions under (`rule:vectors:keyed-refreeze`). Its packet includes the manifest and pins owners as well as the register and history test. S5 adds plans/backlog.md for its packet correction and plans/packages/vectors.md for the admission-standing receiver; neither adds a distinct path or E-role member, interface or contract. S5 also includes root meson.build to register both new vectors integration targets in the existing exact target census; this adds one entry and no distinct path, interface or contract. S5 adds packages/vectors/src/keyed_stack_core.rs solely to document the existing rows() error contract exposed by the new model's ProgramFact re-export. It is absent from the 35-source legacy freeze and already in the separate 46-source inventory; its implementation and interface are unchanged. This adds one existing path and one E-role member, with no new contract. S6/S7 add root meson.build for the existing vectors integration-target census, plans/backlog.md for the selected-design disposition, and their package records (transaction.md at S6, vectors.md at S7). These six existing-path entries add no distinct path or E-role member and no new interface. S5's child encoding/tests.rs is allocated for the encoder grammar controls. S1 has 17 existing and 15 new paths, including the paper's `.latexmkrc`. Append each basename to its prefix; E means an existing file at adoption or an explicitly earlier-created receiver, N a deliberately new file allocated here. At adoption, all existing paths were checked as files; new paths were checked absent. The packet corrections rechecked all E-only paths as files and the new encoding, encoder-test and freeze-history-test paths as absent. S5/S8/S9’s docs/lineage receivers are created at S2. No selected path moved. There is no plans/packages/architecture.md; architecture’s existing README receives S3. No draft body is edited by adoption. These lists replace approximate counts and “as needed” registrations, without granting permission to change every audited consumer. New test files at tests/*.rs use Cargo discovery; src/tests/mod.rs, crate roots, new parent modules and Meson own the other registrations. No new Cargo dependency or public interchange is selected. S8’s oracle/helper paths may remain unchanged if structural subjects suffice. Source histories and capture artifacts are never regenerated to satisfy a new expectation.

| Step | Standing | Prefix | Exact basenames |
|---|---|---|---|
| S0 | E | `plans/` | `backlog.md` |
| S0 | E | `plans/labels/` | `attestation.md` |
| S0 | E | `plans/history/` | `README.md`, `meson.build` |
| S0 | N | `plans/history/` | `phase-7-engine-records.md` |
| S0 | E | `plans/drafts/` | `README.md` |
| S0 | E | `plans/guides/` | `guide_fifteen.md` |
| S0 | E | `plans/packages/` | `compiler.md`, `linker.md`, `realization.md`, `tapscript.md`, `transaction.md`, `vectors.md` |
| S0 | E | `plans/phases/` | `07-burn-and-clear.md` |
| S1 | E | `./` | `README.md`, `meson.build` |
| S1 | E | `packages/execwrap/` | `TRACING.md` |
| S1 | E | `packages/flatten-latex-main/` | `TRACING.md` |
| S1 | E | `packages/document-stamps/` | `TRACING.md` |
| S1 | E | `packages/labels/` | `README.md`, `meson.build` |
| S1 | E | `packages/labels/src/` | `census.rs`, `tests.rs` |
| S1 | E | `packages/labels/tests/` | `subprocess_contract.rs` |
| S1 | E | `plans/labels/` | `README.md`, `attestation.md`, `specification.md` |
| S1 | E | `scripts/` | `census-args.sh`, `meson.build`, `test-attestation-stamps.sh`, `test-meson-mock.sh` |
| S1 | N | `papers/lineage-fixed-form/` | `.latexmkrc`, `main.tex`, `macros_lineage_fixed.tex`, `references.bib`, `stamps.tex.in`, `meson.build`, `meson.options` |
| S1 | N | `papers/lineage-fixed-form/sections/` | `00_title.tex`, `01_interface.tex`, `02_model.tex`, `03_operations.tex`, `04_invariants.tex`, `10_limitations.tex`, `A1_verification.tex`, `B1_symbols.tex` |
| S2 | E | `docs/` | `README.md`, `meson.build` |
| S2 | E | `plans/` | `backlog.md` |
| S2 | E | `plans/registers/` | `identities.md` |
| S2 | E | `plans/guides/` | `guide_fifteen.md` |
| S2 | E | `plans/phases/` | `07-burn-and-clear.md` |
| S2 | E | `packages/vectors/` | `README.md` |
| S2 | E | `plans/packages/` | `realization.md`, `target-elements.md`, `vectors.md` |
| S2 | N | `docs/lineage/` | `README.md`, `realization.md`, `meson.build` |
| S3 | E | `lint/` | `hash-citation-families.tsv` |
| S3 | E | `plans/packages/` | `vectors.md` |
| S3 | E | `plans/` | `backlog.md` |
| S3 | E | `plans/guides/` | `guide_fifteen.md` |
| S3 | E | `plans/phases/` | `07-burn-and-clear.md` |
| S3 | E | `packages/architecture/` | `README.md`, `TRACING.md`, `meson.build` |
| S3 | E | `packages/architecture/src/` | `lib.rs`, `semantic.rs`, `validate.rs` |
| S3 | E | `packages/architecture/src/tests/` | `mod.rs` |
| S3 | E | `packages/realization/` | `README.md`, `TRACING.md`, `meson.build` |
| S3 | E | `packages/realization/src/declarations/` | `mod.rs` |
| S3 | E | `packages/realization/src/` | `lib.rs` |
| S3 | E | `packages/realization/src/tests/` | `mod.rs` |
| S3 | E | `packages/vectors/` | `README.md`, `TRACING.md`, `meson.build` |
| S3 | E | `packages/vectors/src/keyed_final_manifest/` | `pins.rs` |
| S3 | E | `packages/vectors/src/` | `lib.rs` |
| S3 | E | `packages/vectors/tests/keyed_final/` | `freeze_history.rs` |
| S3 | N | `packages/architecture/src/` | `keyed_fixed.rs` |
| S3 | N | `packages/architecture/src/keyed_fixed/` | `vocabulary.rs` |
| S3 | N | `packages/architecture/src/tests/` | `keyed_fixed_profile_tests.rs` |
| S3 | N | `packages/realization/src/` | `keyed_fixed.rs`, `keyed_fixed_relation.rs` |
| S3 | N | `packages/realization/src/declarations/` | `keyed_fixed.rs` |
| S3 | N | `packages/realization/src/tests/` | `keyed_fixed_profile_tests.rs` |
| S4 | E | `packages/compiler/` | `README.md`, `TRACING.md`, `meson.build` |
| S4 | E | `packages/compiler/src/` | `engine_identity.rs`, `engine_program.rs`, `engine_records.rs`, `error.rs`, `input.rs`, `lib.rs` |
| S4 | E | `packages/compiler/src/input/` | `owner.rs` |
| S4 | N | `packages/compiler/src/` | `engine_keyed_fixed.rs`, `keyed_fixed_plan.rs` |
| S4 | N | `packages/compiler/src/engine_keyed_fixed/` | `program.rs` |
| S4 | N | `packages/compiler/src/engine_keyed_fixed/program/` | `restore.rs`, `refusal.rs`, `stage_refusal.rs`, `tests.rs`, `assignment_tests.rs`, `negative_tests.rs`, `refusal_tests.rs` |
| S4 | E | `packages/compiler/src/engine_attestation/` | `program.rs` |
| S4 | E | `plans/` | `backlog.md` |
| S4 | E | `packages/compiler/tests/` | `public_api.rs` |
| S4 | E | `packages/compiler/src/tests/` | `engine_cases_tests.rs` |
| S4 | N | `packages/compiler/tests/` | `keyed_fixed_plan.rs` |
| S5 | E | `docs/lineage/` | `README.md`, `realization.md` |
| S5 | E | `./` | `meson.build` |
| S5 | E | `lint/` | `hash-citation-families.tsv` |
| S5 | E | `packages/linker/` | `README.md`, `TRACING.md`, `meson.build` |
| S5 | E | `packages/linker/src/` | `lib.rs` |
| S5 | E | `packages/tapscript/` | `README.md`, `TRACING.md`, `meson.build` |
| S5 | E | `packages/tapscript/src/` | `lib.rs` |
| S5 | E | `packages/vectors/` | `README.md`, `TRACING.md`, `meson.build` |
| S5 | E | `packages/vectors/src/` | `lib.rs`, `keyed_stack_core.rs` |
| S5 | E | `plans/` | `backlog.md` |
| S5 | E | `plans/packages/` | `linker.md`, `vectors.md` |
| S5 | E | `plans/registers/` | `identities.md` |
| S5 | N | `packages/tapscript/src/` | `keyed_fixed_pattern.rs`, `keyed_fixed_constructor.rs`, `keyed_fixed_bundle.rs`, `keyed_fixed_resources.rs`, `keyed_fixed_members.rs` |
| S5 | N | `packages/tapscript/tests/` | `keyed_fixed_bundle.rs` |
| S5 | N | `packages/linker/src/` | `keyed_fixed_bundle.rs`, `keyed_fixed_reference.rs` |
| S5 | N | `packages/linker/tests/` | `keyed_fixed_bundle.rs` |
| S5 | N | `packages/vectors/src/` | `keyed_fixed_stack_model.rs`, `keyed_fixed_constructor.rs`, `keyed_fixed_manifest.rs` |
| S5 | N | `packages/vectors/src/keyed_fixed_manifest/` | `pins.rs`, `encoding.rs` |
| S5 | N | `packages/vectors/src/keyed_fixed_manifest/encoding/` | `tests.rs` |
| S5 | N | `packages/vectors/tests/` | `keyed_fixed_template.rs`, `keyed_fixed_freeze_history.rs` |
| S6 | E | `./` | `meson.build` |
| S6 | E | `plans/` | `backlog.md` |
| S6 | E | `plans/phases/` | `07-burn-and-clear.md` |
| S6 | E | `plans/packages/` | `transaction.md` |
| S6 | E | `docs/lineage/` | `realization.md` |
| S6 | E | `packages/transaction/` | `README.md`, `TRACING.md`, `meson.build` |
| S6 | E | `packages/transaction/src/` | `lib.rs` |
| S6 | E | `packages/vectors/` | `README.md`, `TRACING.md`, `meson.build` |
| S6 | E | `packages/vectors/src/` | `lib.rs` |
| S6 | N | `packages/transaction/src/` | `keyed_fixed_abi.rs`, `keyed_fixed_request.rs`, `keyed_fixed_inputs.rs`, `keyed_fixed_construct.rs`, `keyed_fixed_taproot.rs`, `keyed_fixed_sponsor.rs` |
| S6 | N | `packages/transaction/tests/` | `keyed_fixed_surface.rs` |
| S6 | N | `packages/vectors/tests/` | `keyed_fixed_reproduction.rs` |
| S7 | E | `./` | `meson.build` |
| S7 | E | `plans/` | `backlog.md` |
| S7 | E | `plans/packages/` | `vectors.md` |
| S7 | E | `packages/vectors/` | `README.md`, `TRACING.md`, `meson.build` |
| S7 | E | `packages/vectors/src/` | `lib.rs` |
| S7 | N | `packages/vectors/src/` | `keyed_fixed_history_reader.rs`, `keyed_fixed_history_model.rs`, `keyed_fixed_evidence.rs` |
| S7 | N | `packages/vectors/src/keyed_fixed_history_reader/` | `authentication.rs`, `evidence.rs`, `fold.rs`, `identity.rs`, `refusal.rs` |
| S7 | N | `packages/vectors/tests/` | `keyed_fixed_history.rs` |
| S8 | E | `docs/lineage/` | `README.md`, `realization.md` |
| S8 | E | `lint/` | `hash-citation-families.tsv` |
| S8 | E | `./` | `meson.build` |
| S8 | E | `packages/target-elements-conformance/` | `README.md`, `TRACING.md`, `meson.build` |
| S8 | E | `packages/target-elements-conformance/src/commitment_oracle/` | `commitment.rs`, `curve.rs`, `generator.rs`, `mod.rs`, `vector.rs` |
| S8 | E | `packages/target-elements-conformance/src/` | `confidential_fixture.rs`, `confidential_funding.rs`, `lib.rs` |
| S8 | E | `packages/vectors/` | `README.md`, `TRACING.md`, `meson.build` |
| S8 | E | `packages/vectors/` | `Cargo.toml` |
| S8 | E | `packages/vectors/src/` | `lib.rs`, `keyed_fixed_manifest.rs` |
| S8 | E | `packages/vectors/src/keyed_fixed_manifest/` | `encoding.rs`, `pins.rs` |
| S8 | E | `packages/vectors/tests/` | `keyed_fixed_freeze_history.rs` |
| S8 | E | `plans/registers/` | `identities.md` |
| S8 | E | `plans/` | `backlog.md` |
| S8 | E | `plans/packages/` | `vectors.md` |
| S8 | E | `plans/history/` | `phase-7-engine-records.md` |
| S8 | E | `plans/phases/` | `07-burn-and-clear.md` |
| S8 | E | `scripts/` | `elements-native-executor.py`, `meson.build`, `test-executor-request-comparisons.py`, `test-executor-classification.py`, `test-executor-boundaries.py` |
| S8 | N | `scripts/` | `test-keyed-fixed-capture-wrappers.py` |
| S8 | N | `scripts/keyed-fixed-capture/` | `capture-prepare.sh`, `capture-unit.sh`, `run-one.sh` |
| S8 | N | `packages/vectors/src/` | `keyed_fixed_capture.rs`, `keyed_fixed_native.rs`, `keyed_fixed_corpus.rs` |
| S8 | N | `packages/vectors/src/keyed_fixed_corpus/` | `history.rs` |
| S8 | N | `packages/vectors/tests/` | `keyed_fixed_capture.rs` |
| S8 | N | `packages/vectors/tests/keyed_fixed_capture/` | `native.rs` |
| S8b | N | `packages/vectors/fixtures/keyed-fixed-native/` | `CENSUS.json`, `clauses.json.lzma`, `policy.json.lzma`, `scaling.json.lzma`, `histories.json.lzma`, `root.json.lzma`, `preparation.json.lzma`, `sources.json.lzma`; exact S8b subset and pin audit at (`tab:s8b:paths`); legacy-copy stays S8c |
| S8b | N | `packages/vectors/tests/` | `keyed_fixed_corpus.rs`; separate offline target avoids editing pinned capture tests |
| S9 | E | `docs/lineage/` | `README.md`, `realization.md` |
| S9 | E | `lint/` | `hash-citation-families.tsv` |
| S9 | E | `packages/vectors/` | `README.md` |
| S9 | E | `plans/` | `backlog.md` |
| S9 | E | `plans/guides/` | `guide_fifteen.md` |
| S9 | E | `plans/packages/` | `compiler.md`, `linker.md`, `realization.md`, `tapscript.md`, `target-elements.md`, `transaction.md`, `vectors.md` |
| S9 | E | `plans/phases/` | `07-burn-and-clear.md` |

**Table (Literal publication and keyed-owner sweep)** · `tab:backlog:fixed-form-sweep`

READ — S0 searched packages/, scripts/, lint/ and all Meson definitions for papers/docs paths, paper names/versions/artifacts, keyed manifest domains/schema constants, literal keyed source paths and their include/module consumers. The rows group repeated literals within one owner; fixture-only examples are distinguished from registries. Receiving steps include these paths in addition to their writable list, with read-only disposition where no extension is needed. “Added” means a deliberate correction to the drafts, not a new contract or parser. The existing label/publication mechanisms remain the owners. S0 regenerates plans/labels/attestation.md through generate-attestation-register after moving record locations. This generated output is also part of S1 when the new paper lands.

| File and line | Enumerated subject | Receiver / disposition | Draft coverage |
|---|---|---|---|
| packages/labels/src/census.rs:148–184,272–298; packages/labels/src/tests.rs:690–727; packages/labels/tests/subprocess_contract.rs:466 | Two literal paper directories and verification against declared census; discovery/unreadable and CLI controls | S1 extend discovery to the third paper and test declared/discovered agreement and omissions; C/D/P | Added code and regressions; drafts named only publication registration |
| meson.build:56–69,231,600–636; papers/lineage/meson.build:2–21,39–45,137,156–182 | Subprojects, A-role inputs, exclusions, rendered/flat names, mirrors and papers alias | S1 add sibling registrations and final versioned outputs; preserve legacy subproject | Root/build already named; exact seams resolved |
| scripts/census-args.sh:72–103 | Shell argument census recognizes only attestation main/sections, not companion papers | S1 explicitly include both legacy and fixed-form companion main/sections in the existing A role, excluding their macros/stamps; retain existing argument grammar | Added; also corrects omitted legacy companion census on this helper path |
| packages/labels/src/latex.rs:12–38,189; packages/labels/src/bin/check-labels.rs:74–92; generate-label-registers.rs:51–57; generate-attestation-register.rs:76–85 in the same bin directory | Harvest/parser consumes supplied paper files, assigning A, without a directory-name switch | S1 audit unchanged; expanded census feeds the existing parser, no new owner or CLI schema | Existing parser reuse; paths resolved |
| packages/labels/src/adoption.rs:279–317; packages/labels/src/census.rs:209–212,706–725 | Partition description retains attestation special cases; generic docs discovery carries other Markdown | S1/S2 audit unchanged: current operational TeX harvest uses the A census; DOC includes the new realization note, RZ stays special | Existing owner premise; paths resolved |
| scripts/test-attestation-stamps.sh:32–82; scripts/test-meson-mock.sh:89–97,147–172,229–235 | Two-paper stamp isolation and literal legacy versioned artifacts/mirror repair/failure independence | S1 extend controls for the fixed-form paper, preserving legacy cases; register through scripts/meson.build | Added tests; same stamp/publication contract |
| packages/execwrap/TRACING.md:9; packages/flatten-latex-main/TRACING.md:9 | Literal Attestation/Lineage build-caller lists | S1 add the fixed-form paper as the third caller; retain the existing caller scopes | Added tracing-document receivers |
| packages/document-stamps/TRACING.md:7; papers/lineage/meson.build:39–45 | Tracing record names only the attestation caller although lineage also invokes stamps | S1 reconcile the existing lineage caller and add the fixed-form caller | Added tracing-document receiver |
| packages/document-stamps/src/lib.rs:577–650,952–1003; packages/document-stamps/src/tests/mod.rs:968–1049; packages/document-stamps/tests/subprocess_contract.rs:67–97 | Generic tree/input request and attestation fixture paths, not a production directory allocator | S1 reuse unchanged; sibling Meson passes its own tree/macro data; shell tests above exercise it | Generic stamp machinery already selected |
| scripts/check-document-reproducibility.sh:74–75,110–122 | Manual attestation-specific byte reproducibility probe | S1 audit unchanged; do not claim this probe covers the new paper. P still requires its real build; extra reproducibility is not a track acceptance premise | Scope clarification, no new contract |
| packages/flatten-latex-main/src/tests/mod.rs:626–698; packages/artifacts/src/tests/mod.rs:32–39; packages/artifacts/tests/subprocess_contract.rs:113–144,244 | Synthetic named paper paths; flatten and generated-publication controls | S1 audit unchanged generic mechanisms, not a complete-paper registry | Existing publication machinery; paths resolved |
| packages/architecture/src/keyed.rs:12,57–58,133; packages/architecture/src/validate.rs:251–278; packages/architecture/src/tests/keyed_profile_tests.rs:27–29,165–167 | Legacy schema 2/domain/paper and symbolic clauses | S3 preserve legacy constants/tests; new keyed_fixed owner and test encode schema 3/profile 3/domain and S1 anchors; shared validator admits the new type | Sibling route already named |
| packages/architecture/src/semantic.rs:10–13,33–56; packages/compiler/src/engine_identity.rs:12–16,52–58 | Profile namespace and legacy object/operation encodings | S3 additive profile; S4 consumes it; no changed legacy encoding | Explicit exception already named |
| packages/vectors/src/keyed_final_manifest/pins.rs:10–320,330–339; packages/vectors/tests/keyed_final/census.rs:6–9,69–127; freeze_history.rs:26–369,505 onwards in the same test directory | 35 literal sources, child-module census, retained-capture overrides and history | S3 paired semantic.rs re-pin/history/override; S5 separate fixed-form inventory, no added child under a legacy pinned parent | Already named, test census resolved |
| packages/vectors/src/keyed_final_native.rs:718,790; packages/vectors/src/keyed_final_corpus.rs:7–41 | Legacy handshake comparison and freeze recipe, six immutable corpus artifact paths | Ruling 210 discharges present-class admission ahead of S2; ruling 211 assigns fixed-form capture-freeze admission to S5 and consumers to S8 before capture; S3 retains legacy source-history mapping; S8 new capture/native/corpus siblings, existing owners replay unchanged | Already covered by identity and sibling decisions |
| packages/vectors/tools/extract-keyed-final.py:132–170 | Historical campaign/run names and six-family extraction | S8 audit unchanged historical extractor; new capture owner writes its own admitted subjects and census, no second extraction tool or historical rerender | Added audit path, no new mechanism |
| packages/vectors/tests/keyed_history/reading_reports.rs:317–319; family_controls.rs:161–169 in the same directory | Legacy paper clauses consumed as reader-test evidence | S7 preserve them and add fixed-form paper checks in keyed_fixed_history.rs | Sibling tests already named |
| packages/compiler/tests/measurement/engine_cost.rs:145,180; scripts/keyed-cost-kranich.py:28–32 | Legacy measurement fixture/harness and keyed subject names | S4/S8 audit unchanged historical measurement owners; new scoped measurements cannot relabel those baselines | Existing baseline obligation, paths resolved |
| lint/hash-citation-families.tsv:104–105,118,202,204–211 | Environment tips, legacy manifest and frozen/capture identities | S3/S5/S8 add exact new identity families when minted; S9 reconciles legacy environment descriptions without withdrawing historical families | S9 named; earlier same-landing identity registration added |
| packages/architecture/meson.build:10–20; packages/realization/meson.build:10–37; packages/compiler/meson.build:40–49,98,138; packages/tapscript/meson.build:11–18,48–49,87; packages/linker/meson.build:12–14,68; packages/transaction/meson.build:10–19,88; packages/vectors/meson.build:21–68,160–166,426,441–484 | Literal source/test/fixture registrations | S3–S8 add precisely the sibling paths in their packet, preserve legacy registrations; lib.rs and src/tests/mod.rs receivers are listed there | Registration points already named; exact lists resolved |

**Table (Shared Wave-2 consumers)** · `tab:backlog:fixed-form-consumers`

READ — Every basename below resolves to an existing Rust file; compare (`tab:backlog:wave2-consumers`) and (`tab:kffwork:wave2`). Append .rs to each stem. Audit-only rows are read-only unless a corrected packet expressly adopts a justified edit; no shared-kernel semantic change is authorized. Rerun both owner families after shared registration merges.

| Prefix | Exact stems | Fixed-form receiver / Wave-2 integration |
|---|---|---|
| packages/compiler/src/ | input, input/owner, error, lib, engine_records, engine_program, engine_identity | S4 registration/typed source/error/namespace helpers; input overlaps 2.3b under SUPPLEMENT and error under COMPILER; lib, engine_records and engine_identity are SUPPLEMENT; input/owner and engine_program are outside (`tab:backlog:wave2-consumers`) and are this track’s additional shared-seam audit |
| packages/compiler/src/ | engine_foundation, engine_requirement, engine_cases, engine_coverage_graph, engine_coverage_join, engine_operation | S4 audit/revalidation only, common kernels retained; engine_foundation, engine_requirement and the two coverage kernels are Wave-2 SEPARATE, engine_cases is SUPPLEMENT, and engine_operation is outside (`tab:backlog:wave2-consumers`) |
| packages/compiler/src/ | engine_attestation, engine_attestation/foundation, engine_attestation/requirement, engine_attestation/source, coverage, layout, placement, case, fold | S4 audit only; preserve 2.3b active and conditional expression behavior, no new attestation policy |
| packages/compiler/src/ | engine_attestation/program | S4 compile-forced test-only foreign-lowering forwarding; Wave-2 SUPPLEMENT, no attestation policy change |
| packages/compiler/tests/ | public_api | S4 fixed-form public surface controls; Wave-2 2.4/2.6 edit receiver, disjoint from the Burn/Clear scope controls |
| packages/compiler/src/tests/ | engine_cases_tests | S4 complete fixed-form eligibility controls through the existing canonical comparison; Wave-2 SUPPLEMENT |
| packages/compiler/src/tests/ | case_tests, coverage_tests, coverage_oracle_tests, analyzed_program_oracle_tests, source_tests | S4 existing regression consumers, no semantic rewrites to pass a new owner |
| packages/realization/src/ | relation, identity, expression, evaluate, validate, error | S3 audit only; 2.3b owns active bodies; its relation.rs re-pin awaits the revised Wave-2 packet recorded at (`rec:backlog:fixed-form-decisions`); fixed-form relation stays sibling |
| packages/realization/src/ | lib, declarations/mod, tests/mod | S3 exports/trait impl/test census, serialized with 2.4/2.6 declaration registrations |
| packages/architecture/src/ | semantic, validate, lib, tests/mod | S3 sole additive pinned exception plus sealed admission/export/test registration; inherited Wave-2 boundary, scheduled after 2.3b |
| packages/vectors/src/ | keyed_final_manifest/pins | S3 namespace mapping/history, serialized after Wave-2 relation re-pin; S5 new inventory separate |
| packages/vectors/src/ | keyed_stack_core | S5 documentation-only error contract for the existing rows method re-exported through the new model; implementation unchanged, outside the legacy freeze and Wave-2 bodies; refresh only the new freeze pin |
| packages/vectors/tests/keyed_final/ | freeze_history, census | S3 history/override controls; preserve current census’s non-test child rule |
| packages/vectors/src/ | maturity_safety, maturity_mutants/native_staging | S3/S4/S8 audit unchanged Wave-2 candidate evidence, not receivers for keyed expectations |

Common README/TRACING/Meson and root registrations are enumerated in the step lists and sweep. They merge against the accepted base, not stale copies. Shared plans/backlog.md, plans/phases/07-burn-and-clear.md and plans/guides/guide_fifteen.md serialize adoption/closure edits. S2's additional documentation consumers are plans/registers/identities.md (vectors-owned pre-admission, read by re-freeze decisions), plans/backlog.md (packet census and disposition), and the phase card/guide (ruling 211's ordering). S5 registers the two vectors integration targets in root meson.build, the existing exact Cargo-target census receiver. S5 updates plans/packages/vectors.md as the existing admission-standing receiver and plans/backlog.md for the packet correction, serialized with planning edits. S5 owns packages/vectors/src/keyed_fixed_manifest/encoding.rs and packages/vectors/tests/keyed_fixed_freeze_history.rs, with Cargo discovery and the existing vectors Meson/module registrations; S8 consumes those owners and updates the identity register for capture activation. These packet additions supply no new shared kernel. S1’s publication files and S2’s new realization record can proceed beside Wave-2 code; S3/S4 meet only the named vocabulary/entry/history seams; S5–S8 sibling bodies are disjoint from Burn/Clear bodies.

S0 weight maintenance: the two completed engine experiment records (`rec:phase7:wave1-engine-probe`), (`rec:phase7:wave1-engine-refit`) moved with their record bodies byte-for-byte to plans/history/phase-7-engine-records.md, omitting only the card’s final inter-section blank separator at the destination’s EOF, with an unlabeled source stub and history index/census. Searches of packages/, scripts/ and lint/ found no executable consumer of either record or a literal Phase-7 card path; the generic phase-status/index reader retains its header in place. Hash-family descriptions cite the preserved labels, not a file parser. No backlog table moved: its executable readers in compact_corpus.rs, maturity_first_party.rs, maturity_evidence.rs and maturity_mutants/native_staging.rs retain their subjects. Byte equality of the extracted record bodies and destination was checked after excluding that single separator byte, not inferred from a diff summary.
