# Tripod Implementation Backlog

> **Status:** ACTIVE
> **Current gate:** Phase 5 — Live receipt transfer
> **Current condition:** Phases 1 and 2 are complete. The target-independent compiler constructs one deterministic, validated scoped analyzed program for the complete pilot scope, factorized per operation, with relation-owned requirements, a corruption-resistant assembly validator, and independent assembly oracles; the Phase-2 exit gate record is in §2.11, including passed document reproducibility. Phase 3 then delivered the Guide-8 target foundation, the Guide-9 native primitive gate, the Guide-10 constructor and wide-floor prototypes, and the Guide-11 declassification result, each with its own gate record. Public declassification is no longer an open blocker: an initial policy is selected — an explicit boundary reached by owner-authorized normalization, with the direct and public-committed paths deferred against three named target blockers — and the gate record is in §2.4. Phase-3 exit is recorded: the Guide-12 preflight register closed all sixteen seventh-review rows, and (`gate:phase3:exit`) was checked green on that tree with the record in §2.6. Phase-4 exit is recorded in §2.8: compact ASH ran end to end against a live target with twelve accepted and matched submissions and honest coverage stated at 100 of 211. The current work is Phase 5 — Guide 13's owner-authorized live receipt transfer — which consumes the Phase-4 pipeline rather than redesigning it and begins implementation only when the Guide-13 preflight register closes.
> **Next gate:** Phase 5 — live-transfer end-to-end pipeline
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

### 2.1 Latest static review · `tab:backlog:review-basis`

The current review is the sixth static review, performed in two independent
passes over the supplied concatenation of the tree:

```text
tree:
   

selected files:
    220
```

The supplied filter excluded 352 files, among them:

```text
Cargo.lock
the paper sources
most unit and integration tests
most compiler implementation files
most executable-model implementation files
the label-tool implementation
several build/tooling packages and scripts
```

No Cargo, Meson, TeX, the native executor, advisory, or reproducibility
command was run as part of this review.

Therefore:

- the review makes no current green-build claim;
- the excluded packages and test directories received no content review;
- lockfile checksums and resolved features were not independently verified;
- advisory status was not checked;
- licence compatibility was not independently checked;
- historical gate records remain historical evidence only.

Its findings are recorded in §5.6. Its central verdict is that the native
evidence layer still does not bind evidence to its subject: an execution
transcript retains neither the requested subjects nor the exact fixtures
sent to the child, so it can be rebound to a census, matrix, target, or
deployment that was never executed; and the claim a fixture files under is
caller-authored, so claim-bearing metadata can be attached to an unrelated
or trivially true script. Both passes rank those as release-blocking for
that subsystem, and no public-declassification prototype begins while they
remain open. The review text is archived at
[plans/reviews/review-6-0.3.4-dev.md](reviews/review-6-0.3.4-dev.md).

### 2.2 Earlier review basis

The repository retains seven earlier reviewed trees as historical context:

```text
initial reviewed tree:
   

follow-up reviewed tree:
   

proof-planning reviewed tree:
   

second-review tree:
   

third-review tree:
   

fourth-review tree:
   

fifth-review tree:
   
```

Those reviews and their findings are evidence about their exact trees. They are
not current-checkout execution evidence. The second through sixth review
texts are archived under [plans/reviews/](reviews/README.md).

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

### 2.9 Guide-13 batch gate · `gate:backlog:guide13-batch`

The Guide-13 batch closed 2026-08-24 with fourteen waves merged — T5-001 through T5-019 in §11 — and the full repository suite green on the shared instance at the batch tip: 48 of 48 meson lanes passed, zero failed, 13 m 12 s wall through the run-report wrapper, after one honest bounce whose cause was the batch's own archiving — the generated label register went stale when the executed guide, the eighth review, and the imported Guide-14 concept landed their mints, was regenerated by the meson target on the server, and the two census bumps were reviewed as a diff and committed before the rerun. Per the no-host-compute ruling every verification of the batch ran on the server: worker lanes for each wave, the native live-transfer lane against the server-built target binary with the run of record byte-identical across three runs at three tips, check-plans on every plans-touching commit, and this gate. The Phase-5 result is the honest stopped one the phase card records: the candidate pipeline is complete, the first-party half of the safety matrix is discharged twenty-five of twenty-five, and the entire remaining distance to the Phase-5 exit is the two typed blockers — the owner sighash and confidential predecessor funding — both owned by the chartered intermediate confidential-funding guide, which must close before Guide 14 is drafted. The batch also delivered by user ruling: the four adopted drafts retired into ADR-019 through ADR-022 with the amendment-listing cleanup recorded open as DI-F07, the root-ADR weight class, the drafts directory restored as a standing template, and the intermediate guide's four charter decisions recorded as accepted in its concept.

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
| Phase 3 | HISTORICAL | exit recorded in §2.6 and the Phase-3 card |
| Phase 4 | HISTORICAL | exit recorded in §2.8, the Phase-4 card, and [the Guide-12 completion record](history/guide-12-completion-report.md) |
| Phase 5 | Active | current backlog, Guide 13, and the Phase-5 card |

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
**passed** (§2.8). No Phase-5 charter exists yet: the next gate arrives
with Guide 13.

The Guide-8 target foundation is complete; the gate record is §2.12. The
Phase-3 task state is:

| ID | Status | Task |
|---|---|---|
| `T3-001` | DONE | Typed target package boundary |
| `T3-002` | DONE | Reviewed initial target definition |
| `T3-003` | DONE | Encoding and opcode contracts |
| `T3-004` | DONE | Development deployment binding |
| `T3-005` | DONE | Compiler capability adapter |
| `T3-006` | DONE | Target-native primitive conformance, gate record §2.13 |
| `T3-007` | DONE | Typed tapscript instruction foundation, gate record §2.13 |
| `T3-008` | DONE | STATE-constructor prototype accepted, gate record §2.14 |
| `T3-009` | DONE | Exact wide floor-arithmetic prototype accepted, gate record §2.14 |
| `T3-010` | DONE | Public-declassification initial policy selected, gate record §2.4 |

The Phase-4 task state follows Guide 12's wave plan, from
(`task:guide12-exec:wave4`) onward; each wave lands with its own
evidence and the batch gate record extends §2.6:

| ID | Status | Task |
|---|---|---|
| `T4-001` | DONE | Validated compact-ASH target operation plan — compiler `operation_plan` public boundary, projections recomputed against the analysis (23 relations, 2 cases, 46 relation-cases, 69 layout requirements), Guide-11 representation stated, nine corruption oracles |
| `T4-002` | DONE | Target transaction-form and dependency review — target-elements `transaction_form`, fee role recognized by form, both forms consensus-admitted, sponsor asset explicit and value uninspected, first-party substrate selected awaiting dependency ratification, three requirements deferred to transaction evidence |
| `T4-003` | DONE | Static target assessment and typed tapscript proof patterns — tapscript `shape`, `policy`, `pattern`, `operation_assessment`; `CompactAshShape` minted and the (`rule:guide12-exec:plan-selection`) tie-break stated in `CompactAshBackendPolicy`; eight pattern identities earned by machine-checked schedules; all 79 plan requirements assessed (74 complete-pattern, 3 structural, 2 external evidence) with emission admissible |
| `T4-004` | DONE | Candidate relocatable bundle — tapscript `bundle`; static ASH constructor over 9 shapes as 9 coordinator leaves and 3 recomputation-shared member leaves, 3939 exact encoded bytes, 61 relocations located by rebuilding each program with one symbol replaced plus 9 introspection references read from the emitted instructions, candidate status held by construction refusing a release-complete plan and by a read-only artifact status |
| `T4-005` | DONE | Linker foundation — `packages/linker`; two-pass resolution over 22 typed symbols of which 21 are defined, frozen reference graph whose one strongly connected component is the ASH constructor's self-commitment, 61 relocations applied by structured substitution to 3975 exact encoded bytes, deterministic 12-leaf taptree checked against an exact subset oracle that is itself checked against literal enumeration, `ControlPathDepth` settled at 4, all 30 relation-cases uniquely carried; the demonstration link refused without an explicit authenticated strategy for the self-commitment, which is the finding this wave was for |
| `T4-005a` | DONE | Self-commitment resolved by identity introspection — the recognition fragments read the constructor's program off the input the leaf is spending instead of pushing a literal for it, which is the resolution the Wave-8 refusal named; `AshConstructorProgramVersion` retired, `AshConstructorProgram` rebound `ReadFromTargetAtSpendTime` and carried by an introspection-reference census, the sound strategy now links with no equality outstanding, and every emitted program shrank (coordinator model base 116 to 85 and per-ash-input 109 to 78, member 102 to 64) |
| `T4-006` | DONE | Candidate transaction ABI — `packages/transaction`; the ABI states every item Guide-12 section 15.4 lists over the 9 admitted shapes, with a first-party explicit-field encoder and decoder checked against hand-written byte strings (193 sponsorless, 374 sponsored) and control blocks over a committed tree checked against digests computed outside the crate; the three whole-transaction dimensions settle per constructed transaction; the taproot output key is pinned rather than recomputed and the pin-against-tree equality is carried as an outstanding obligation |
| `T4-007` | DONE | Canonical semantic and target fixtures — `packages/vectors`; the complete Guide-12 section-18 matrix transcribed as 153 named classes each naming its polarity, mutation layer, and expected section-1.5 boundary; canonical and experimental standing separated by a type whose canonical constructor is crate-private and has no promotion path; the section-16.4 evidence plan recomputing every census two ways and comparing them (23 relations, 2 cases, 46 relation-cases of which 4 vacuous, 211 coverage requirements split 139 positive and 72 negative), the section-1.3 relation closure checked in both directions; 14 positive semantic cases carrying no target vocabulary, with the successor derived from the realization layer's own checked sum, and 9 of them materialized to exact byte-stable target transactions; no coverage row discharged, every row naming why it is outstanding |
| `T4-008` | DONE | Real target execution (Waves 11 and 12): the section 16.2 edge end to end, twelve vectors — eight sponsorless, four sponsored under the test-scoped authorization capability — accepted and mined at the pinned genesis, all twelve matched on the thirteen section 17.4 terms, coverage 99 of 211, three byte-identical runs per wave. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T4-012` | DONE | Issuance bisect: the refusal `bad-txns-in-ne-out` traced to the witness-section guard ahead of `VerifyIssuanceAmount` — a balance diagnostic for a non-balance condition — and fixed first-party by funding through the witness-carrying program form; filed upstream as EG-021. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T4-013` | DONE | Kept and reclassified: the `maximum` fixture stands at the protocol's 51-bit bound, `vectors::divergence` derives the row's standing against the reviewed stated-amount bound, and the divergent row was refused live with excess while the other eight submitted. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T4-009` | TODO | Negative relation coverage (Waves 13–13e): the mutation machinery, the derived class-to-requirement link, per-mutation funding, three section-18 boundary respecs proven against live evidence, and one negative row discharged. Open honestly: 71 of 72 negative rows outstanding, each naming its reason in code; the remedy is the three guide gaps, filed as [the Guide-13 feature requests](guides/guide_thirteen_feature_requests.md), not more first-party tests. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T4-010` | DONE | Candidate resource study (Wave 14): all thirty-six section 20.2 assignments measured, twelve live weights matched against prediction, and the finding that linking — seven of thirty-six under the sixteen-leaf oracle budget — is the candidate ceiling, not any target bound. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T4-011` | DONE | Phase-4 gate and handoff (Wave 15): the section-29 audit dispositioned 105 items (94 PASS, 5 QUALIFIED, 1 honest FAIL, 5 deferred), the completion report filed at [the Guide-12 completion record](history/guide-12-completion-report.md), and the deferred lanes discharged by the §2.8 gate run locally as the verdict of record. Full narrative archived in [backlog history](history/backlog-history.md) |

The Phase-5 task state follows Guide 13's wave plan, from its Wave 0
preflight onward; each wave lands with its own evidence:

| Task | Status | Evidence |
|---|---|---|
| `T5-001` | DONE | Guide-13 Wave 0 preflight reproduction: all eighteen review rows dispositioned. Sixteen CONFIRMED with committed reproductions in six packages — a confirmed defect is an ignored test stating the property the repair must establish, so the repair un-ignores or replaces it and nothing rounds up. `G13-R09` closed by the phase-state reconciliation. `G13-R06` RECLASSIFIED at reproduction, then ruled a repair 2026-08-22: the executor's diagnostics move to a mandatory `--output` file argument carrying typed facts only, raw child material moves to a mandatory `--elements-output` file, and nothing is written to stderr — so the channel the scope debate was about no longer exists; chartered with the `G13-R18` framing lane. `G13-R18` CONFIRMED on all four sub-claims: blank records skipped, no byte bound, the handshake accepts unknown fields, and exactly one of the five framing cases the Rust side distinguishes exists in the reader. Two couplings the review did not draw: the live lane publishes its coverage through the forgeable discharge API, so `G13-R01` and `G13-R12` repair together, and one caller-authored tuple was measured discharging forty-two rows. Integration lane green on the shared instance: clippy 7.5s, the workspace suite 339.9s |
| `T5-002` | DONE | Guide-13 Wave 0 preflight repairs: all eighteen rows closed across six serial-then-parallel lanes, each merged only after diff review and a server-side integration lane. Transaction: superfluous-witness refusal with the universal re-encode law, form exactness with the empty-offer refusal and the shape's form a fourth conjunct of selection, all duplicates refused. Tapscript: u16 index domain, fallible shape sets, settled-comparison truth recorded without fabricating canonical bytes. Vectors: coverage discharge transcript-derived behind a validated operation report matched on submitted bytes, weight mismatch refuses the plan, the live lane renders from the validated report without wall time, sponsor-change presence represented end to end. Linker: one u128 checked domain with the sixteen-leaf budget promoted to an enforced precondition, duplicate leaves refused, program role derived. Conformance: zero tweak admitted per the target rule with an independent pure-Python recomputation, response validators exhaustive over role and outcome. Compiler and neighbours: thirty enum censuses generated from one declaration. Python executor: the ruled mandatory `--output` and `--elements-output` files with nothing on stderr, marker-proof quarantine, strict bounded request framing with contract-side bounds. Repair lanes surfaced and fixed eight defects beyond the register. Residuals carried: saturating arithmetic in the linker's graph reference counts and bundle resource folds; the vectors planner transcript's public Default; the ADR-018 provenance seam in the validated report; the phase-index table outside the check-plans weld; twenty-one hand-maintained censuses in conformance, vectors, transaction, and labels; the census macro's three-crate triplication; the lifecycle response validator having no production Rust caller; and the live lane plus the change-present sponsored row awaiting a real node |
| `T5-003` | DONE | Two residuals named in the Wave-0 completion record discharged by the first Codex worker lanes (gpt-5.5, read-back gated, edits reviewed and committed by the orchestrator). Linker: reference-edge site counts accumulate with checked arithmetic and refuse instead of saturating, the zero-count fallback became an invariant-stating expectation, and the script-byte total sums exactly under a proven construction bound of eighty-four million bytes, with four focused tests reaching the repaired branches. Labels: the phase-index table joined the check-plans weld — a card without an index row, a row linking a missing card, a status disagreeing with the card's own declaration, a duplicate row, and a missing section or table now fail with nine focused tests; on arrival the weld caught real drift, the index calling phases 03 and 04 Complete while both cards declare Exited, repaired in the same commit. Both lanes gated on the shared instance: the first pass bounced on formatting and two documentation lints because the lane design carries no toolchain, fixed at the gate; the second pass is green with a forced re-check |
| `T5-004` | DONE | Guide-13 Wave 1, the evidence handoff: every canonical negative vector states its whole declaration as typed data — source fixture requirement, intended relation, mutation class, changed target fields, semantic change, evidence boundary, carrier, collateral policy, representation and sponsor case — with the changed fields and semantic change recomputed from the mutated bytes rather than asserted, which corrected two guessed declarations; plan derivation requires exactly one matching requirement and a contradicted declaration is a hard refusal, honest blockage staying a typed pre-resolution reason. The first-party negative policy landed with its proving instance: the permissionless-private-dependency rows discharged end-to-end through the exact owning compiler validator, extracted so an evidence package drives the analysis's own check and never a mirror. The withheld pre-target classes land at a typed ABI-validation boundary whose indexed set is proven the exact complement of the submittable set, with both rows observed safe-constructor-refused over all nine materialized vectors. The ADR-018 provenance seam closed the smaller way: the transcript retains its configured expectation and conformance publishes the comparison, never the operands, so no consumer can compare a report against itself; a failed comparison refuses the whole report and the rendered schema moved to revision two so an old reader refuses rather than conflates. Four resolutions and twelve typed blockages, unchanged from the earlier census — the machinery grew and the honest number did not. Merged after full diff review and a green server gate; the ruled follow-up wiring the discharge into plan derivation is its own record |
| `T5-005` | DONE | The ruled first-party discharge wired into derivation: derive_evidence_plan builds the canonical permissionless-private-dependency case, validates it through the sole validator, and discharges exactly the two first-party rows through the existing path, refusing with a typed error when either half goes missing; reported coverage moves to 102 of 211 — ninety-nine positive rows outstanding for want of a target, one negative runtime row, two first-party rows discharged — with the fresh-plan controls restated at the new baseline and double-counting impossible because coverage rows overwrite rather than increment. Residual carried by name: a first-party validation refusal rides the borrowed unclassifiable-negative-requirement variant, unreachable today and awaiting a named variant in an error-surface wave |
| `T5-006` | DONE | Guide-13 Wave 2, the validated live-transfer target plan: a new compiler module derives every clause of the semantic contract from the typed realization and checks it against the guide's fixed value, so a drifted realization is noticed and drift is never published as contract, with nineteen named clauses and a both-directions exactly-one rule per clause. Two representations planned side by side — the split drawn where the equivalence rule draws it, measured first against the analysis — with both named deferrals derived under trip-wires that refuse when a future realization would widen them silently. The forbidden-class census derived by complement over all object families, the owner quantifier grounded as the agreement of two relations over one family, candidate cardinality bounds carried unresolved, and the roots and certificate projections minted where the wave list dropped a clause. The validator covers every corruption row including the capability and evidence censuses; the compact-ASH validator's omission of those same two checks is a confirmed substrate defect dispatched to its own lane. Merged after diff review with the trust pattern verified and a green server gate: six hundred twelve library tests, twenty-eight boundary tests, formatting and workspace lints clean |
| `T5-007` | DONE | The compact-ASH plan validator gains the aggregate-census checks the live-transfer wave found missing: capabilities and external evidence are re-derived from the relation rows through the two helpers the constructor itself now calls — one authored derivation on both sides — refusing with the variants the live-transfer plan minted, and two corruption tests reach both refusals. First lane run under the worker verify loop: the lane's uncommitted worktree was snapshotted through a temporary index, verified on the shared instance before the diff was reviewed, and came back green in one round, retiring the gate-fix pass the first three toolchain-less lanes each cost |
| `T5-008` | DONE | Guide-13 Wave 3, signature and CT target closure: the confidential-conservation evidence requirement follows the selected proof alternative — committed conservation loses its runtime carrier and lands as external target evidence, splitting the representations' measured censuses (private sixty-one layout and two hundred twenty-five coverage rows against explicit sixty-nine and two hundred twenty-three) with one shared visibility decision read by placement and coverage alike. The owner sighash profile is selected in the adapter from the protocol's fifteen protected data against the target's dimension vocabulary, fail-closed over unconsidered dimensions and honestly assessed review-incomplete. The test-only signing capability reproduces the published BIP-340 vectors byte for byte from their published secret keys under the test-material rule, signs only messages it is handed, and carries no production shape. Sixteen owner-authorization cases each name the protected datum they disturb, with the narrowing guard proven firing per case and every case carrying its residual; the mixed-representation refusal is exercised by hand in both orderings. Merged after diff review and a green server gate over five packages. Findings carried: the SDK and server clippy now diverge in both directions; the rustdoc bar fails on pre-existing links in tapscript and conformance, dispatched to its own lane; the conformance census residual text is stale in one respect, left for the native-signature wave |
| `T5-009` | DONE | Three residual discharges by parallel Codex lanes under the verify loop, merged serially after review. The operation transcript is buildable only by its planner: the public Default that contradicted the type's own claim is gone, a crate-private constructor replaces it, and the preflight control strengthens to no transcript of either kind being constructible outside the crate. The first-party refusal names its own cause: a typed variant carries the first-party module's refusal through both derivation sites, retiring the borrowed variant whose name described a different failure. The rustdoc bar passes for tapscript and conformance: five pre-existing link defects repaired, and the verify loop gained a documentation stage that builds with warnings denied. Model evaluation rode the first lane: gpt-5.6-sol matched gpt-5.5 on read-back fidelity and finished its work step faster than every gpt-5.5 baseline, with one house-lint bounce repaired in one round |
| `T5-010` | DONE | Guide-13 Wave 4, the candidate live-receipt constructor: canonical owner metadata derived from the target's own encoding registry — the guide fixes that one canonical encoding exists, the target fixes which — with the no-secret rule carried by the encoding domain, the only check that separates an approved public key from the scalar a private key is written as, and its ordering before the class gate proven by test. No key-path escape holds as a property three ways: one-variant inherited policies, a spending-route census derived from the leaf set, and construction refusing an empty transfer leaf set, the escape reached by omission. Two disjoint per-representation leaf sets foreclose in-script representation dispatch structurally; the one-to-one shape contributes no member leaf, derived from the position rule rather than assumed. The lifecycle is structurally release-incomplete and the candidate-ABI binding is recorded outstanding rather than invented, the section-7.1 and wave-order disagreement recorded as a guide erratum. Seventeen mutation cases published as executable data: eleven built, run, and refused by the variant that names each; six carry residuals naming the wave they wait for. Eight fail-closed guards over the validated plan are named unreachable-today in the report. Merged after diff review with the pattern-docs collision against the parallel doc lane resolving identically; one server gate-lint fix (assert_ne over assert-not-empty), second gate green including the documentation bar |
| `T5-011` | DONE | Guide-13 Wave 5, recognition and owner authorization: the abstract walk learned to say what section 1.8 asks — the target's two signature successes leave identical stacks, so the walk now reports which successful form each signature primitive can reach beside the states, and the forward-compatibility path became a computable predicate rather than an argument. The owner key is never a witness item: the constructor-committed key is pushed by the program at the approved encoding's exact width, and the walk reports the unverified form unreachable; the six mandated negative cases run as executable mutants with registry-derived widths, the two abstractly indistinguishable ones each naming the first-party oracle that separates them. The curve-point-membership residual is discharged in conformance over the existing curve arithmetic with no new dependency, the emitting adapter deliberately kept free of the checker's opinion. Recognition is proven local by construction — an owner-parameterized constructor forecloses the compact-ASH cross-input comparison, recorded with the mutual-fixed-point argument — and reads no amount, which is what lets one fragment serve the private plan. Coordinator and member placement carries an honest eleven-check census: one established, three partial with named remainders, seven typed-outstanding pointing at the exact later-wave patterns. The empty-key census defect found and fixed: section 1.8's empty key had been mapped onto the missing-signature behaviour, a different target fault. The sighash-review residual is a computed disposition required on every signing pattern, and the sighash profile stays a link-time symbol with the reasons recorded. Merged after diff review with the six load-bearing tests recomputed locally; server gate green in one round including the documentation bar |
| `T5-012` | DONE | Guide-13 Wave 6, the explicit live-transfer plan: the coordinator gains its global checks — destination closure reading the target's own output fields against five newly consumed link-time symbols, exact aggregate conservation whose every addition is followed immediately by the verify that consumes its success flag (proven over every admitted shape, with an executed negative showing the overflow residue an unconsumed flag leaves behind), sponsor isolation that never introspects a value field, and issuance absence tested on every input including sponsors. The eleven-check census flips to seven established and four partial owing exactly two named remainders: private conservation for Wave 7, and the destination-constructor identity — unreachable in script because a destination's constructor is owner-parameterized, so the Wave-5 predecessor residual is re-scoped rather than discharged, the same residual now carried at both ends of an induction stated in three parts where recognition documents itself. Family ranges classify every position of every shape into exactly one authenticated family with four executable refusals, and the explicit programs assemble into a candidate relocatable bundle whose emission runs the final-stack and family-range oracles per leaf and refuses on either. One substrate defect fixed at its root: the abstract walk's byte equality was blind to operands of provably disjoint widths, which made issuance absence unverifiable; the narrowing removes only successful forms, never aborts, and settles nothing for untaught types. Two findings recorded for later work: the compact-ASH bundle's symbol-width table disagrees with its own relocations about leaf-program width, documented at the live symbol table; and the walk treats a per-path operand type mismatch as a whole-program error, foreclosing a class of branch pruning. A permanently empty introspection census was removed under the no-unconsumed-field rule rather than carried as dead weight. Merged after diff review with ten load-bearing tests recomputed locally; server gate green in one round over four packages including the documentation bar |
| `T5-013` | DONE | Guide-13 Wave 7, the private live-transfer plan — delivered, not deferred, with no new dependency and no confidential fixture, because a value's form is a registry prefix and nothing in the plan ever needs an opening. The private coordinator settles its own destination value form and carries every representation-independent global check, while confidential conservation lands as external target evidence through a third placement set and a status that says so — the outstanding-pattern promise the guide forbids is now structurally inexpressible, and the six-line soundness argument is a total census with the conservation row's fragment set asserted empty in both directions. Six of the seven opacity prohibitions are checked properties over the emitted programs, the seventh a named non-claim asserted to be the only one. The candidate bundle accepts both representations under the same per-leaf oracles with the leaf sets still disjoint. The wave's headline is a substrate repair: the confidential value encoding declares two prefixes, one per commitment parity, and the pattern helper compared only the first — so private recognition as merged would have refused about half of all valid confidential receipts, for a parity their owner neither controls nor observes; repaired by an exact masked membership test derived from the registry that refuses rather than widens when a prefix set is not precisely what one mask admits, the explicit plan's bytes unchanged. A second overclaim corrected: destruction absence cited the explicit arithmetic the private coordinator does not carry and now names the external requirement. Both confidential prefixes are flagged for target-native exercise since the abstract walk cannot separate them. Merged after diff review with the full tapscript suite and the named opacity, soundness, and mask tests recomputed locally; server gate green in one round over four packages including the documentation bar |
| `T5-014` | DONE | Guide-13 Wave 8, the linked live-transfer candidate: the linker's tree construction becomes generic over one leaf trait so one Huffman, one exact cost arithmetic, and one oracle pair serve both vocabularies with compact ASH's behavior unchanged, and a closed-form equal-weight optimum joins the search oracles because the candidate commits twenty-nine leaves — past the exhaustive budget — with the closed form cross-checked against both oracles everywhere they run and the committed tree reaching the proven optimum of three leaves at depth four and twenty-six at depth five. The fifteen guide-listed symbol roles land typed, the constructor symbol parameterized by owner and representation, the sighash profile entering from the reviewed contract with no deployment parameter so the exactness ruling cannot be weakened by passing a different value, and role completeness is a checked census scoped to the carried plans. Carrier closure compares required, emitted, and reachable per representation against each plan's own tree — made possible by a plan-neutral case key after finding the existing key embeds the representation, which made every cross-plan comparison a tautology — measuring thirty-two explicit and thirty private carriers with exactly two explicit-only, both the conservation rows the private plan carries as external evidence, and a starved plan is a typed refusal naming itself. The induction's link-time end is established as one linked constructor placement per owner and representation while the output key and destination table stay honestly outstanding, the recognition residual re-scoped on both ends. Five mutation-case flips retire the linker residual and name the ABI dependency it stood in front of; the bundle width disagreement is fixed at its root; a four-axis live resource model replaces the three-axis one that could not express output freedom. Merged after diff review with both suites recomputed and the twenty-nine-leaf optimum verified by hand; one server gate bounce on the emptiness-assert lint at three test sites, fixed typed, second gate green over four packages with the linker joining the documentation bar |
| `T5-015` | DONE | Guide-13 Wave 9, the candidate transaction ABI and signing flow: the typed request carries the eight selectable facets while the fourteen the guide forbids are absent as fields rather than checked as rules, each absence naming where the refused choice actually comes from; canonical input layout with the three rejections landing before the sort, request-order outputs with the semantic comparison on the multiset, and the sponsored form's five-way equivalence tested arrow by arrow with an empty capability proven to refuse rather than downgrade. The finalization boundary holds twice — structurally, since the finalized form has no mutable route and signing requests are constructible only from it, and by typed refusal, since an offered transaction is compared against an independently held census and the mutation named. Owner responses arrive as position-response pairs so a duplicate can exist to be refused, all ten multi-owner rejections carry executable negatives with the wrong-owner and wrong-bytes rows also proven against real published-vector signatures, and the three authorization levels stay distinct types. Both linker handoff obligations discharge: destination owners select linked constructors from a typed table closing the request end of the induction with the predecessor owner recognized rather than selected, and the taproot output key is recomputed in the conformance oracle's own arithmetic, delivered through a capability so the transaction package stays free of curve code, and compared byte for byte in the vectors integration. The curve-point loop closes at the one place every owner passes. The private test construction records the central-public-fixture model as a type that refuses every other model, with a new non-claim that no range proof is produced or checked. The mutation census sharpens to five cases waiting only on the target-native run; two multi-owner authorization cases flip while the sponsor-owner case deliberately does not, its signer being adapter-borne. Found in passing: the vectors package had been failing the documentation bar on two pre-existing links, repaired, and the gate's suite and doc bar both widen to five packages; the conformance meson census is membership-checked but not order-checked, flagged; the brief's may-not-select count was a miscount against the guide's fourteen, the guide's own list implemented and asserted. Merged after diff review with the suites recomputed including the tweak-against-oracle equality; server gate green in one round |
| `T5-016` | DONE | Guide-13 Wave 10, the complete live-transfer safety evidence and the run against a real target: the whole hundred-and-eight-row section-15 matrix lands as constant registries in guide order — sixteen positive explicit, ten positive private, fourteen owner and signature, sixteen object, twenty-two value, thirteen sponsor, seventeen structural — each row resolving against the published plan to exactly one requirement, intending the whole census, or naming its typed reason, with the explicit arithmetic rows inactive under the private plan by the conditional-coverage rule rather than dropped. The canonical evidence plan derives with its seven sources accounted for, the minimality registry honestly typed as not yet built for Wave 11. Eleven owner-and-signature rows discharge first-party, each staged case driving the signing flow or the offer check twice so the refusal names the row's own class; the two sponsor report rows are answered at the report boundary by key rather than substring, since a substring search would have flagged the matrix's own row name. The validated safety report excludes all ten volatile fields from its canonical bytes with a test that proves each marker present in the diagnostic rendering and absent from the canonical one, and the renderer takes no diagnostics argument so the exclusion is structural. The zero-required-infrastructure-errors bar is honestly not achieved and the report validation refuses completeness naming every outstanding row: seventy-nine rows are typed-blocked and sixteen first-party rows undischarged, the root cause being one missing component — no first-party code computes the target's own taproot sighash and the guide forbids asserting one. The native run made that blocker observed rather than argued: against a real Elements daemon the ceremony issued its asset, re-linked the deployment against the issued identity through a new asset-parameterized link, funded both constructors, and submitted the explicit transfer, which the node refused with its own words — mandatory script verify flag failed, invalid Schnorr signature — meaning the covenant ran, every coordinator check passed over real coins, and evaluation failed only at the owner signature; the run of record is committed as a typed transcription labeled not evidence with the destroyed chain's outpoints deliberately absent, and the ceremony proved deterministic when the server rerun reproduced it field for field down to the issued asset. Both parity forms and the range-proof question are recorded unanswerable through this boundary because the funding subject has no confidential form, so no confidential predecessor can exist to spend — a protocol-revision-level gap, not a wave one. Both census flip counts are honestly zero, the worker correcting the brief's claim that two authorization cases waited on the run when they wait on the sighash review no run discharges. Found in passing: the built node was unrunnable outside its build toolbox until its libraries were staged beside it, which prompted the standing move of the native lane to the shared instance where the node is now built from our own hub at the reviewed tip; the ten-second plan derivations are memoized. Merged after diff review with the scoreboard recomputed structurally from source to exactly one hundred and eight, the canonicalization spot-checked, and the suite proven green with no node present; one gate bounce on a package-name typo in the orchestrator's own script, second gate green over five packages in the usual four minutes |
| `T5-017` | DONE | Guide-13 Wave 11, the disclosure-minimality comparison and the first-party completion of the safety matrix: the section-16 pair registry lands with all five pairs claimed against the candidate ABI's own shape table — one-to-one, split, merge, many-to-many, and sponsor — each pair growing from exactly one semantic fixture with both members built by the same function, and all ten members actually finalize with real target bytes, what is missing being only the test-only owner-signing capability. The section-16.2 acceptance conditions are scored one by one: seven of ten hold first-party for every pair — owner and class agreement, explicit reserve agreement, family closure, private amounts absent from public output, no opening in the report, no secret outside the declared model, equal lifecycle — while constructibility and acceptance carry the two observed blockers and projection equality is typed as awaiting both verdicts rather than as a third blocker, because a reader repairing the pipeline needs the dependency direction. Minimality is therefore reported unanswered, in a validated report deliberately separate from the safety report down to a qualified schema identifier on its first line, with the two documents rendered side by side in a test and told apart at the first line and the role line in both directions. The disclosure tables land verbatim from the guide with the counts recorded as transaction-shape leakage under both plans, the two explicit-plan exact-amount disclosures each carrying the typed reason that the program checks conservation over fields it can read. First-party discharge rises from eleven rows to twenty-four of twenty-seven: every staged case drove its owning validator twice across three refusal vocabularies kept unmerged, and the three resisting rows carry specific typed obstacles rather than absence — one row's declared boundary disagrees with the validator that actually owns its class, one mutation is structurally inexpressible in the leaf vocabulary, and one names a genuine gap: nothing on the live-transfer request path enforces the semantic amount ceiling, the transaction crate holding the realization vocabulary as a dev-dependency only, recorded as a design decision for the owner rather than fixed unilaterally. A dead refusal variant was found in passing and left in place, named. The evidence plan's minimality source flips from not-yet-built to built with five pairs and zero supporting, carrying counts and no verdict. Merged after diff review with the scoreboard recomputed to one hundred and eight and the report-separation test read in full; per the no-host-compute ruling the server gate is the sole suite verdict, green in one round over five packages, and the native lane rerun on the server against the wave's own tree produced a report byte-identical to the committed run of record |
| `T5-018` | DONE | Guide-13 Wave 12, the resource study: the three bound symbols land with the guide's own research candidates — two hundred ninety-four assignments, each costed by closed form and checked against the real shape and leaf sets rather than argued from the widest — and the binding constraint turns out to be the deployment's declared taptree depth of eight rather than the linker's subset oracle, asserted from both sides with two hundred fifty-five leaves building and two hundred fifty-eight refusing at the declared capacity, the leaf budget refusing the widest assignment outright. The position domain accumulates in a wider integer precisely so that exceeding the wire width is reportable, and the honest fitting statement is that the tested bundle, publishing a window the enumeration does not name, has a program for every shape of exactly eight enumerated assignments, computed over set membership. The fifteen measurement cases land as seventeen complete transactions through the real construction pipeline, each recorded across all seventeen dimensions: every case at taptree depth five, peak main stack seven, and a measured alternate-stack zero kept honest by requiring the same walk to prove itself on the main stack; the sponsor region costs five hundred twenty-nine weight, the change output three hundred ninety, one private commitment over explicit forty-four; the validation budget tracks exactly fifty per receipt input; and a merge was found to spend coordinator and member leaves at different depths, so no control total is one width times a count. The prediction-against-observation comparison ran against the real target over the same exact bytes: one dimension of seventeen is observable at this boundary — the node reads back transaction weight even for refused bytes — and predicted equals observed at nineteen hundred eleven, cross-confirmed by the study's own independently constructed sponsorless case weighing the same, while the sixteen unobserved dimensions each carry a typed reason and a staged disagreement proves the typed planner failure fires. Both Wave-11 hooks are filled and their deferral standings retired: four sponsorless pairs carry measured weights whose type name says no confidential proof is counted, the sponsor pair carries the signer absence as the reason no complete transaction exists, and the hard-target-limit failure mode is computed within declared limits rather than deferred. The validated resource report has its own schema identifier, the ten volatile fields excluded with a teeth-bearing test, and a result vocabulary holding the guide's candidate-bounds sentence verbatim with no production variant to reach for. Found in passing: Wave 10's wire protocol already carried transaction weight that the observation record dropped, repaired; a duplicate abstract-walk implementation now exists as a named consolidation candidate; the executor writes absent operation figures as zero on the wire, a substitution the guide forbids, recorded as a feature-request candidate; and the two-resource-study ambiguity in the vectors plan is Wave 13's disambiguation. Merged after diff review with the closed forms and boundary assertions recomputed by hand and the weight deltas cross-footed from the table; the wave verified entirely on the server per the no-host-compute protocol, and the merged tip's gate ran six packages' tests including the freshly changed labels crate plus the five-package documentation bar — a first attempt at extending that bar to the labels crate surfaced pre-existing unresolved intra-doc links there, recorded as documentation debt rather than fixed in a resource wave |

| `T5-019` | DONE | Guide-13 Wave 13, the Phase-5 gate and handoff: the three rows that resisted first-party discharge in the T5-017 snapshot are resolved by re-typing what each one claimed rather than by staging cases against falsified boundaries, and the first-party half of the safety matrix closes at twenty-five of twenty-five with the undischarged-fault register deleted outright rather than left empty. The wrong-constructor-schema row moves to a freshly minted constructor-derivation boundary and a static-constructor-schema mutation layer — the guide erratum is filed in the feature-request register, the row's three malformations each driven to their exact typed refusal against one accepted control, and the matrix's shared vocabulary keeps the two guides' counts apart so a member minted for one cannot be read as named by the other. The amount-outside-semantic-domain row is re-typed under the owner ruling that the ceiling is blockchain-enforced, the same class as conservation: its boundary becomes consensus rejection before script, where the reviewed target refuses a stated amount above its own bound, the transaction layer deliberately gains no request-path check, and the verdict stays typed-blocked behind the absent accepting control like every other target-negative. The mixed-operation-program row leaves the staged-refusal denominator as an operation-vocabulary closure that no layer answers because no typed input names its fault — a two-member row-boundary type carries the distinction, the standing is counted in its own census bucket that no report reads as evidence, and no general inexpressibility discharge is minted. The dead refusal variant the T5-017 record left in place, named, is now wired: an unlinked representation is refused at the head of finalization before the ten construction stages, with both mirror cases tested and the owner-specific refusal proven not swallowed. The safety report schema steps to revision two for the new census line, and the native run of record is byte-identical across three runs at three tips, verified by digest on the server. The Phase-5 card records the candidate pipeline complete, the acceptance lines that hold, the two typed blockers owned by the chartered intermediate guide, the impact and non-claim record, and the four-item handoff queue. Merged after diff review with the twenty-five pre-target boundary occurrences recounted from the row tables and the report digests compared server-side; the wave verified entirely on the server, five packages plus the native lane green in three hundred five seconds at the final tip |
| `T5-020` | DONE | Intermediate confidential-funding guide, Wave 0, elaborating custody and reproducibility: all four preflight findings of the guide's own repository reading are CONFIRMED, each by running code rather than by reading, and nothing whatever is implemented. The two proof-field refusals fire on any nonempty field with no reference to the output's value form, reproduced by decoding one byte string carrying a sixty-four-byte range proof against the hybrid form the guide fixes — explicit asset, committed value, empty surjection field — and observing the same refusal the explicit form draws, with the surjection refusal for the hybrid form kept as a standing guarantee because the repaired behaviour owes it too. The protected preimage is byte-identical to the witnessless serialization, which is exactly the empty-output-witness case the recorded target diagnosis names, reproduced on the first-party side by finalizing a private transfer through the ordinary construction path and comparing the two byte strings; the target half is cited from the existing runnable diagnosis and was not rerun. The proof-fields datum is carried by the spent outputs alone, reproduced by reading the selected owner profile's own coverage and observing a one-member carrier set, and the created outputs' dimension the repair must add is already a required dimension of the profile, so the widening asks nothing new of the target. The per-output confidential value role is a characterization rather than a defect and is confirmed as one across four measured limits: the role is told the protocol asset, one amount, the published randomness, and a position and nothing else, one call per destination in request order; a capability answering every call with one constant commitment finalizes without complaint, so no blinder sum, conservation, or cross-destination relation is checked anywhere in the private path; every output takes the null nonce; and the output type has no proof member at all, so the encoder writes two empty proof fields per output and there is nowhere for a range proof to go. Three rulings of the concept and three of the guide's charter decisions are carried forward and checked against the real types rather than restated: the accepted custody model has its typed name in the transaction package and is the only model the construction recorder admits, the reviewed target package really does carry no dependency and really does own the confidential field encoding the contract enum would sit beside, the reproducibility contract's four comparison surfaces correspond one to one with the concept's own sentence, and the terminal-audit-closure classification the no-allocation decision rests on is the Phase-5 card's own words. The dependency boundary is re-read from the three manifests and holds unchanged: the conformance package reaches the transaction and vectors packages as development dependencies only and its manifest forbids promoting either, the transaction package names the conformance package nowhere and records the absence as load-bearing, and the vectors package is the one library where both are reachable. Six divergences between the guide, the rulings, and the tree are reported to the orchestrator for recording in the guide rather than repaired here, the sharpest being that two of the three protected-bytes repair parts are not expressible at this tree at all until the output-witness census row lands first. Verified on the server per the no-host-compute ruling: the worker lane green over both owning packages with the formatter and the workspace clippy pass, and the confirmed rows reproduced by a separate run of the ignored reproductions whose observed failures are the evidence |
| `T5-021` | DONE | Intermediate confidential-funding guide, Wave 1, revising the funding wire: the confidential arm lands as a fifth untagged operation subject with its own step kind and its own capability, and the explicit funding subject is not re-shaped — a tagged union inside a variant of an untagged enum is the shape that file refuses, and the two arms stay untagged-disjoint by member set, sharing exactly the asset question and nothing else, which a test holds rather than a comment. The request types carry destinations and a binding and there is no member an amount could be written into: a validated public handle, a thirty-two byte drift digest, and four closed profiles of one member each, so a second profile is an added variant a peer must advertise rather than a silent change of meaning. The response gains the two members the guide fixes, neither defaulted, and that is the whole revision: the sponsor members were defaulted so adding them was not one, and these are not so that adding them is, with the protocol revision moved from four to five in the harness and the reviewed native adapter in the same commit — the adapter reads the confidential subject strictly, refuses every unknown tag and undeclared member before construction, advertises no confidential capability because it implements no deterministic materializer, and writes both new members on every operation answer, and a contract test carries its refusal record in the adapter's exact spelling so the two implementations are checked against each other rather than each against itself. The reproducibility contract is placed once, in the reviewed target package, as candidate ceremony vocabulary, with that package's contract note and crate documentation widened in the same change to say so; the target package still serializes nothing and gains no dependency, and the wire encodes the contract through the vocabulary's own code so the workspace holds one word per contract rather than two spellings held equal by a census. The complete typed refusal vocabulary lands closed with no catch-all, twenty-nine variants, every one of them reachable from a test and none of them a target verdict, and readback binding takes no output fact from a request echo: the identities are recomputed from decoded bytes, the block is an independent observation, and the proofs, commitments, nonces, programs, and scripts are read out of the decoding, with the request checked only for the arm, the handle, the digest, and the profiles. The three collaborators the binding needs are traits implemented outside the package, so no library edge is added in either direction, and no test reaches a target: the executor's side is scripted, the chain is a stub, and the only real arithmetic is the first-party commitment oracle, used so an admitted encoding can be told from an inadmissible one. Two precisions are reported rather than assumed: the thirty-three byte commitment and nonce travel as byte vectors because the serialization framework implements no array reader past thirty-two, which makes both encoding refusals reachable by width as well as by prefix; and the materializer refusal is typed by the non-verdict layer the executor reported, because the response carries no cause member and a first-party derivation vocabulary would be a second spelling of one the later waves own. Verified on the server per the no-host-compute ruling: the formatter, the workspace clippy pass, and the three owning packages green, with five hundred one conformance tests and two hundred sixty-two vectors tests passing and check-plans green on the plans clone. No blocker moved, no matrix row moved, and nothing here claims a transfer, a proof, or a production property |
| `T5-022` | DONE | Owner-sighash parallel work, Waves 0 and 1, recording the charter and reviewing the target's message from source: nothing is implemented, no digest is asserted, and no dimension moves to reviewed. Wave 0 is the record. The three rulings of the owner-sighash concept are carried forward as recorded and are never reopened: the profile decision, at the concept's profile-decision rule, fixes a default-only type byte, one profile for both lanes, a refused annex, and script path only; the accepted-result decision, at the concept's result-decision rule, fixes option B — protected bytes, the signing-input census, and opaque authorization bytes plus the returned hash-type byte, bound to one candidate identity — with its three constraints, that the census is observed target data and candidate structure only, that the binding is exact-byte in the pattern the offered-transaction check already sets, and that the returned hash-type byte is checked against the profile rather than merely recorded; and the evidence decision, at the concept's evidence-decision rule, fixes source review plus recomputation plus one observed acceptance, populating the reviewed capability dimension by dimension, each dimension carrying both its citation and the observation that exercised it. The profile's constants are stated here as the values later waves assert rather than choose: the hash-type byte is 0x00; the authorizing signature is 64 bytes wide, because a 65-byte signature is one carrying a trailing type byte and the target refuses a trailing zero, so the already-measured unauthorizing-signature width stays true; the spend-type byte is 0x02 for every candidate this arc produces, being the script-path extension flag shifted with a zero annex bit; and the spend path is script path only, so the tapleaf hash, the key version, and the codeseparator position are message terms for every input this work ever signs. The option-B census is named field by field: the protected bytes; the output-witness vector at its consensus length; the spent-output census per input, carrying each spent output's asset field, value field, and script; the deployment's genesis block hash; and, per signing input, the input index, the tapleaf hash, the leaf version, the codeseparator position, and the annex disposition. Its exclusions are a rule and not an omission: no opening, no blinder, no nonce input, no key, and no proof input enters the census under any lane, which is what keeps a secret-bearing interface from being selected at all. The review boundary is recorded in both directions: no wave of the confidential-funding guide reviews this profile, and no wave of this work reviews confidential funding, the transaction-wide materializer, or blinding, and the confidential-predecessor funding blocker is untouched by every wave here under every result for every row. The census's ADR-015 disposition is public test material under the existing test-material rule: every key, signature, chain, and census value this work touches is disposable regtest material authorizing nothing anywhere else, and because the census excludes openings by rule the design gate for a secret-bearing interface is never reached. The package and dependency boundary is recorded without implementation by implication: the reviewed target package owns the reviewed sighash capability and its dimension vocabulary; the tapscript package owns the selected profile, its coverage map, and its disposition; the transaction package owns the finalized form and the signing request, and is therefore the one surface a widening can happen on without forking the signing path; the vectors package owns the blockers, the matrix classification, and the evidence that moves a standing; and no dependency is approved by naming a need. Wave 1 is the source review, landing as a new technical reference beside the tapscript one, censused and indexed in the same commit: the taproot message is read term by term at the pinned Elements tip, nineteen terms with the position each is written at and the helper each is computed by, and each of the selected profile's eight required dimensions is read against the term that carries it. Seven of the eight are established from source in the only sense a reading can establish anything — the term exists, its place in the stream is fixed, and its content is the dimension's own subject. The eighth is settled the other way and it is the wave's sharpest finding: the spent-scripts term commits each spent output's script, which for a taproot output holds the tweaked output key and not the internal key, so the message construction carries no internal-key term at all; the internal key is bound to that output key by the control-block check that runs before the leaf executes, which is a witness check the message never covers, and the message is identical for any two internal-key-and-path pairs tweaking to the same output key under the same leaf, so what exists is a composition and a hardness argument rather than a commitment. The consequence is typed rather than rhetorical: the profile recomputes its disposition from the required set, so one permanently unexercisable member holds it at review-incomplete forever, and Wave 4 cannot reach an established disposition until the owner chooses between re-typing the dimension as carried by the spent outputs and keeping it required with a control-block check named as what exercises it — a reading the coverage map supports from the other side, since it assigns no protected datum to that dimension at all. Both refused dimensions are checked against the branches that would have carried them and both refusals hold, with the second understating itself: refusing the permitted-extension dimension does not merely leave the input set open, it drops seven whole-transaction terms and replaces them with the signing input's own, which is what makes the spent-outputs dimension a whole-transaction dimension in the first place. The issuance-rangeproof hazard is reproduced and refuted at once, by the recorded diagnosis's own method extended so the two witness vectors vary independently and four candidate messages are computed per signature instead of two: the input-side term is length-dependent exactly as its output-side sibling is, and the target's own signer does not fall into it, because the wallet resizes the input-witness vector to the input count before any signing begins and nothing resizes the output-witness vector until serialization, which the run confirms by the explicit row's signature verifying against the output-side-emptied candidate and against no other while the accepted confidential control verifies against the fully grown one. So the twin is confirmed as a property of the message and refuted as a defect of this signer, the recorded output-witness finding stands alone in that path, and no new upstream entry is earned because nothing here costs an adaptation the filed one does not already cost. What it does cost is a precondition on the accepted census: the input-side term is recoverable from the input count only while no input bears an issuance, the profile admits issuance-bearing candidates, and the census wave owes either a field or a typed refusal for that shape. The refused single-output branch is recorded as an observation with teeth: it bounds-checks the signing position against the output list and then indexes the output-witness vector at that position, the two lengths agree on every wire form and disagree on every signing path, and asking the node for that hash type through an ordinary wallet request ended its process on signal eleven with no shutdown record written — not a consensus defect, since no wire form reaches the read, but a wallet-reachable one, minted as an upstream friction whose entry at the owning register is owed and could not be filed by this lane. Seven message terms are named by no dimension and are dispositioned rather than left implicit: two are census inputs, three become constants a test asserts, one is absent by the annex refusal, and one is a census field whose value is constant while no leaf emits the opcode that moves it. The reviewed sighash capability is still constructed with an empty reviewed set and every dimension unreviewed, the profile still assesses review-incomplete naming all eight, and the residual stands in every vocabulary that carries it: a source review alone populates nothing, and the review document says so in its own section rather than leaving it to be inferred. Verified entirely on the server per the no-host-compute ruling, every step timed: the twin diagnosis and the recorded output-witness diagnosis each ran about a second against the server-built node at the reviewed tip, check-plans passed in the plans clone, and the five documentation and census lanes passed at the branch tip after the label register was regenerated by its meson target and its one-count diff reviewed as text before being applied |
| `T5-023` | DONE | Intermediate confidential-funding guide, Wave 2, proving one confidential predecessor: the fixture registry lands as the only lookup authority, with the freeze a type transition rather than a flag — registration consumes the mutable registry to produce the frozen one, so a late write is not a rule that could be broken but a value that no longer exists — and the handle grammar, the drift digest, and the deterministic derivation recipe land with it. The handle is a validated public identity whose one digit belongs to the grammar version, so a later grammar is a different prefix rather than a reinterpretation of this one, and eight spellings are refused by name at the clause each fails. The digest is a tagged hash over a framed transcript whose every member carries its own width, and the contract tag sits inside that transcript rather than beside it, which is what makes a semantic-only recorded-randomness digest impossible to mistake for a byte-identity one — both contracts are registered and their digests are held apart by a test rather than by intent. Derivation is domain-separated by role and by case and the search is bounded, upward from zero, without wrapping, skipping, randomness, or concurrency; the balancing blinder is solved from the input blinder sum minus the others rather than derived, and an independent recheck reads every blinder back and adds them again, so the predecessor's ordered additive inverses are an instance of the general rule rather than a special case. One deterministic funding transaction meets the fixed shape exactly and was submitted, accepted, mined, and read back raw from a real node: one explicit protocol-asset input contributing a zero value blinder, spent from a coin mined at exactly the fixture's total by a separate explicit transaction that is not the funding transaction, because a funding transaction returning protocol-asset change would carry a third protocol member and turn the balance into an argument about which region a member was put in; exactly two protocol outputs carrying the explicit asset and positive confidential values summing to the input, zero asset blinders and therefore an empty surjection field on both, the two admitted commitment parities in fixed order under the bounded counter search, a deterministic nonce per output, and a valid rangeproof per output bound to that output's commitment, the unblinded asset generator, and the program. The policy-asset change and fee are classified into an explicit non-protocol region by an exhaustive classifier with no member it may decline to place, and six typed refusals cover a member that carries the protocol asset, a commitment, or a proof outside that region and a protocol position that carries none of them. Recomputation and raw readback agree on all eight members of the funding agreement census for both outputs, in both runs, and every comparison is between two distinct origins: the commitment and its parity are the first-party bignum oracle against the chain, the program and the nonce are the registered fixture against the chain, the proof shape is the reference verifier against the chain, and the asset, the outpoint, and the witness transaction identity are the materializer's own answer against the chain, with a comparison whose two sides carry one origin refused where the census is assembled rather than reported as agreement. The two independent commitment comparisons are carried by opaque newtypes whose only constructors are first-party arithmetic and a readback decoding, so a comparison of a readback with a readback is not a rule but an unspellable expression. The validated record's only constructor is the validator, which does not check an offered record but builds one by recomputing every member in the fixed order, with a named refusal per step and the summary recomputed from the members rather than carried, and it carries all ten non-claims and all nine canonical exclusions as sets rather than as prose. The executor gained the deterministic materializer the arm needs and now advertises the capability and the advertisement together on one condition, because the harness refuses a peer that states either without the other; it holds the same public fixture catalogue and recomputes the digest from its own arithmetic, so a drifted catalogue is a refusal rather than a different materialization. Three findings are reported rather than smoothed over: the target refuses a rangeproof whose proven minimum is zero unless the program is unspendable, which is a consensus rule the first attempt met head-on and the target's own blinding path states from the other side; the readback decoder is the reference library rather than the first-party one, because at this tree the first-party decoder refuses any nonempty proof field and that repair belongs to the next wave; and the inclusion condition is discharged by the adapter's own reading of the block rather than by a second query from the harness, because the wire carries no member a block's contents could arrive in. Verified on the server per the no-host-compute ruling: the formatter, the workspace clippy pass, five hundred nineteen conformance tests and two hundred sixty-two vectors tests, and the live native lane green against the server's real node, with the ceremony run twice against two disposable chains and the two funding transactions compared byte for byte rather than the byte-identity contract being asserted from its own name. Exactly one residual moved, on an observed result rather than on a capability existing, and it moved at its owning boundary only; no matrix row moved, and neither owner-sighash blocker was touched |
| `T5-024` | DONE | Intermediate confidential-funding guide, Wave 3, materializing private transactions transaction-wide: the four serialization changes land together because three are one another's preconditions, and the encoder's bytes for every existing lane are unchanged by them — the transaction gains an output-witness vector, one entry per output, modelled as a transaction-level vector rather than an output field because that is where the target's own serialization puts it and because the hazard is the vector's length rather than any output's content; the witness predicate becomes a disjunction over both vectors, since a confidential funding transaction has null input witnesses and proofs its outputs are invalid without, and the old conjunction would have serialized it without the section carrying them; the superfluous-section condition follows and is a widening rather than a weakening, admitting every byte string it admitted before; and the range-proof refusal becomes conditional on the output's own value field in both directions, with the requiring direction new, while the surjection refusal stays unconditional because that is already the form-conditional answer for every form this workspace builds, an asymmetry recorded at the refusal rather than left to be rediscovered. The protected-bytes repair is scoped rather than universal: the private lane's preimage is the witnessless serialization followed by the output-witness vector, so an owner binds to bytes containing the range proofs the target's digest covers, and the explicit lane's preimage is unchanged byte for byte because its output-witness vector is empty under both readings and widening a settled lane's bytes to say nothing new would move a signature's preimage under every existing owner for nothing. Both Wave-0 reproductions lose their ignore attributes and stand as the repaired properties' guarantees, and the sharpest test drives the difference directly: two candidates differing in exactly one range-proof byte, same length, same commitments, same nonces, same programs, same version and lock time, have different protected preimages and identical witnessless serializations, which is the omission the repair closes stated as a measurement rather than as an argument. The transaction-wide materializer is one entry point taking an intent, a frozen fixture view, and two injected collaborators, with the preflight's five clauses before any cryptographic work and the seven stages in order after it; the cryptography is injected because the construction package may not depend on the package that is the independent oracle its output is compared against, and the two collaborators' answers travel in two different opaque types so that the comparison cannot be handed the same origin twice — a property carried by the type system rather than by a reviewer's attention. Stage four refuses a check that declares the construction's own origin, the reference bindings' origin, or the target's, because the reference bindings bind the same library the target vendors and their agreement would be conformance evidence wearing independence's name. The refusal vocabulary lands closed with no catch-all, thirty-two variants matching the guide's own table exactly, every one a construction refusal and none a target verdict; twenty-nine are reachable from a test and the remaining three are structural guards the construction cannot violate without being edited to disagree with itself — the two census guards and the freeze's round-trip check — which is reported rather than papered over. The frozen candidate has private fields, a constructor only the materializer reaches, a region census of eight, and seven spelled mutation attempts that each refuse naming their region, so the census is walked by a test rather than sampled. The third origin-tagged commitment newtype completes the independence rule where its two siblings already live, with two comparison functions each taking two different types, and the boundary widening it implies is recorded in the vectors manifest in the same change: what crosses that package's conformance edge is now two boundaries, the executor and the commitment oracle, with the reason the second is not the forbidden direction stated exactly — the forbidden direction is obtaining an expectation from the component that will produce the observation, and the oracle produces no observation, computing curve arithmetic over published constants and never touching a node. Both origins are real where both are reachable: the first-party bignum oracle and the reference bindings compute the same thirty-three bytes for every registered opening and agree with what the registry derived, and every range proof the materializer produces verifies through the reference verifier against that output's own commitment, the unblinded asset generator, and its program, while a proof checked against the other output's program does not verify — so the binding is measured rather than assumed. No target has seen any of it, nothing here is a verdict about a chain, and the proof generation and verification are both conformance evidence and are not independent evidence. The derivation recipe gains a test-only source seam, authorized and bounded: pub(crate), sitting between the preimage and its bytes, with the tagged hash as its only non-test implementation and the public entry point a one-line call selecting it, changing no public signature and no registered digest, which a test checks rather than asserts. Two derivation refusals become reachable through it, the degenerate solved balancing scalar and the exhausted bounded scalar search with its role and attempt count. The identity value commitment does not, and the reason is a property of the arithmetic rather than a gap in the seam: a source chooses bytes and does not choose what the oracle makes of them, and a seam that reached the oracle would have made the independence claim checkable by substitution — so that half is discharged at the layer that genuinely injects its cryptography, where the materializer's invalid-commitment refusal covers it. The per-output role is retired by scope and not by wish: the range-proof non-claim is dropped for the transaction-wide lane through a separate constructor, stays in force for the per-output lane it still describes, and the other three non-claims stand unchanged under both, while selecting the per-output role at the transaction-wide entry point is a typed refusal before any cryptographic work. One deliverable is not done and is reported as not done: the private branch of the live finalization does not yet take the transaction-wide path, because that call site carries no intent, no fixture view, and no collaborators, and supplying them is a signature change across every caller that the evidence lane's own restart owns. The sites that named the funding blocker as the reason a private materialization is not constructible are re-pointed in prose with every typed value and every standing left exactly where it is: a confidential predecessor is fundable and one has been mined, so the condition is scoped to this pipeline's own funding step, which still has no confidential form and still consumes no predecessor funded through the other arm. The narrower condition that actually keeps the private half blocked has no name of its own in that vocabulary, and minting one was reported rather than done. Verified on the server per the no-host-compute ruling, every step timed: the formatter, the workspace clippy pass, and the three owning packages green. No native run, because the materializer is first-party and this wave's exit needs no target run. No blocker moved, no matrix row moved, and neither owner-sighash blocker was touched |
| `T5-025` | DONE | Owner-sighash parallel work, Wave 2, the signing-input census and an independent message construction: the census lands as a typed value carrying exactly the fields the accepted option-B result names — the protected bytes, the output-witness vector at its consensus length, the spent-output census per input with each spent output's asset field, value field and script, the deployment's genesis block hash, and per signing input the index, the tapleaf hash, the leaf version, the codeseparator position and the annex disposition — and it carries the frozen candidate beside them as candidate structure, because the private lane's preimage is the witnessless serialization followed by the output-witness vector and is therefore not a decodable transaction, so the version, lock-time, input and output terms cannot be read back out of the bytes the owner binds to. There is no public constructor: the one public route takes a materialized confidential candidate, which the materializer alone produces, so no route skips finalization, and the route takes the materialized value rather than the frozen candidate inside it because the frozen candidate does not carry the spent-output census and a caller assembling one beside it would be the skipping route. The exclusion rule is enforced by shape rather than by review: there is no field an opening could be written into, and the one place a key would naturally appear is the control block, which travels as a constructor argument the taproot commitment check consumes and drops, so nothing key-shaped is retained. The message construction is written from the Wave-1 source review's own term table, every term cited at the line the review cites, and the independence is real rather than declared — a second authoring of the same nineteen-term stream, spelled out byte group by byte group at the call site, agrees with the module's construction, and the boundary of what is shared is stated in the module rather than left to be inferred: the tagged-hash primitive and the output, output-witness and field serializations are this crate's own evidenced product, and respelling them would have created a second opinion about encoding free to drift from the bytes the crate emits, so the encodings are shared and the stream is independent. The four witness treatments the twin diagnosis named are all present and all four messages are distinct, which is what lets a later run say which of the two length-dependent terms moved rather than only that something did, and the two candidate messages the recorded diagnosis compares are pinned as constants a test holds, both computed over a deterministic first-party candidate whose every field is a fixed byte string, so the pair is replayable rather than a snapshot of one run. That pinning is a divergence from the wave's entry condition as written and it is reported as one: the recorded diagnoses persist no digest anywhere in the repository, both generate fresh wallet keys on every invocation and print their digests to standard output alone, and both exercise the key path while the selected profile is script path only, so there were no recorded values to reproduce and this wave takes no target run — what is landed is the recorded method reproduced over a fixture that can be replayed, and the cross-origin comparison against a node belongs to the observation wave. The profile's constants are asserted as constants and the spend-type byte is recomputed rather than compared with itself, so a recomputation that had stopped depending on the annex would fail rather than pass. The refusal vocabulary lands closed with no catch-all, fifteen variants, every one of them a construction refusal and none a target verdict, and every one reached by a named test through a crate-private test-only parts seam that runs the same clause list as the public route rather than a relaxed one — including the refusal the source review put on this wave's desk by name, since the census's silence about the input-witness vector is a claim resting on the precondition that no input bears an issuance, and a declared issuance is now a refusal rather than an assumption. The internal-key dimension is re-typed onto the spent outputs under the ruling: the required set becomes seven message-carried dimensions, the role vocabulary grows a third member rather than one of the two being stretched, and the review's composition argument travels with the dimension at the point of use with its citations, so a reader who asks why the set is seven finds the argument and not a dimension that quietly stopped being mentioned; the coverage map assigned the internal key no protected datum, which a test measures rather than asserts, so no protected datum moves. The proof-fields coverage remap lands with no tension to report: the map becomes set-valued, which is the type change the confidential-funding guide fixed as a type change rather than a value edit, the proof fields name the created outputs' dimension alongside the spent outputs', both carriers are dimensions the profile requires, and the totality argument moves with the type and is recomputed by a published function rather than left to a lookup that could only fail one of its two ways. The guide's ignored preflight reproduction loses its ignore attribute and stands as the repaired property's guarantee, and the conformance package's committed-protected-data derivation is re-read as any rather than all, because a datum is committed as soon as one dimension carrying it is required and reading it the other way would have dropped a datum out of the set the moment the map recorded a second carrier for it. No digest is asserted as authoritative, no dimension moves to reviewed, the sighash capability is still constructed with an empty reviewed set, the profile still assesses review-incomplete, and both the profile residual and the computability blocker stand in every vocabulary that carries them; no matrix row moved and the confidential-predecessor blocker was not touched. Verified entirely on the server per the no-host-compute ruling, every step timed, with no native run because this wave observes nothing |
| `T5-026` | DONE | Owner-sighash parallel work, Wave 3, observing the selected profile on the explicit lane: one finalized explicit candidate was authorized against the Wave-2 message, submitted to a real node, and accepted into a block at height five. The census had one route and it took the proof-finalized value, so the explicit lane could not reach a message at all; the second route lands on the same only-route discipline, taking the value that lane's own finalization produces and reading every member out of it, and the two members it needed were ones construction already read and dropped, since the public view states each consumed outpoint's asset, value and program while the receipt record kept the value alone. A sponsor-bearing candidate is refused as the cardinality mismatch it is rather than repaired, because construction never reads a public view for a sponsor input and letting a caller supply the missing entries would be the skipping route the finalized form exists to prevent; that gap is reported and not closed. Acceptance and recomputation stayed two origins: the node formed its own message and ran its own verifying primitive, and then the accepted transaction was read back out of the node, the signature was taken from those bytes rather than from the value the builder still held, and it was verified against a message recomputed here from the source review's own term table with this workspace's own curve arithmetic. The read-back bytes are the submitted bytes, the signature verifies against the recomputed message, and it does not verify against the empty-vector message, so the pair the recorded diagnosis compares is distinct on the accepted witness rather than only in the abstract. Reading the accepted witness back needed a protocol change, because the submission step returned an identity and nothing else: a mined readback is now a member the submission kind owns alongside the confidential funding kind, an accepted submission that omits it is a shape defect, and the adapter reads the confirmed transaction back out of the node after the block it made rather than projecting it from what it was handed. Six negative controls were offered to the same node on the same chain before the accepted case, each moving exactly one term, and every one was refused with the node's own invalid Schnorr signature verdict: the output-witness vector emptied, the genesis hash of another chain, the non-selected type byte appended to a sixty-five byte witness, a valid signature by the other published owner, a valid signature over a candidate whose destinations are exchanged, and the deployment seed in the order a target prints a block identity rather than the order it hashes it in. Two of them were refused on this side as well and the construction refusal is recorded beside the target verdict rather than merged with it, so the deployment mismatch and the two profile answer checks are first-party refusals and never verdicts. The byte-order control is the wave's own finding and it is measured rather than read: the first run refused every case including the selected one, the deployment seed is the single term of the stream carrying a value a run printed rather than a value this crate encoded, and offering both orders to the same node settled which one the target seeds with. The spent-output triple every signature is taken over is the node's own report of the funded coins rather than the ceremony's expectation of them, with the expectation compared against it and recorded, because the message commits to the spent asset, value and program and signing over what a builder assumed would be signing a different message than the target forms. The computability blocker is cleared at its owning boundary on that observed acceptance and never on a capability existing: the positive rows stop being infrastructure-blocked and become rows a run could answer, the negative half stops being blocked on the absent control for the same reason, the three rows carrying a blocker of their own do not move, and not one row becomes answered because a standing is not evidence and no run of any row has been filed. The profile residual stands in all four vocabularies, untouched, because a digest that can be computed is not yet a settled claim about what it commits to and that residual is cleared by a review verdict rather than by a run; no dimension moved to reviewed and the sighash capability is still constructed with an empty reviewed set. The observation establishes nothing about the proof-bearing lane, in the code and in the artifact rather than by implication, because the output-witness term is recoverable from the protected bytes exactly while every entry is default-constructed and the proof-bearing lane's entries carry range proofs the preimage does not contain in any form. Verified entirely on the server per the no-host-compute ruling, every step timed, with the native run the evidence of record. |
| `T5-027` | DONE | Owner-sighash parallel work, Wave 4, the review verdict and the accepted result: the verdict is a TYPED STOP and it stops on one named dimension. The reviewed sighash capability stops being two sets of bare members and becomes two maps to grounds, because a bare member was free while both sets were empty of content and stops being free the moment a dimension moves — a reviewed set of bare members could be grown by an edit that read no source and ran nothing, and an unreviewed set of bare members cannot tell a dimension nobody reached from one that was reached and could not be exercised. A reviewed dimension now carries both halves the accepted evidence ruling requires, the source review's citation and the observation that exercised it, and an unreviewed one carries which of three different kinds of unreviewed it is. Six of the profile's seven required dimensions move to reviewed on the Wave-3 acceptance: the two output-side terms, the three whole-transaction input terms, the version and lock-time terms, the two spent-output terms, and the tapleaf term, each cited at the line the Wave-1 review cites and each exercised by the same accepted transaction. The candidate that acceptance was taken over consumed two receipts and created two destinations, which is what makes the four whole-transaction dimensions exercised rather than merely present — a single-input, single-output candidate would have left every all-versus-one distinction unobserved and the honest verdict would have been narrower. Two of the six carry a negative control besides, both on the output side, since the ceremony's six controls moved exactly three of the message's nineteen terms between them: the deployment seed twice, the output list once, and the output-witness vector once. The seventh required dimension does not move, and the reason is a property of this workspace rather than of the target. The target's message does carry the issuance dimension, in the two terms the review cites, but no candidate this arc builds bears an issuance — the census refuses a declared one by type rather than carrying a field for its input-witness proofs — so the construction that recomputes the message reads no issuance field at all and forms both terms from the input count alone. That is the line the verdict is drawn on: the acceptance and the recomputation agree over the whole stream at once, so a model wrong about any term would have produced a different digest, which makes the agreement per-dimension evidence exactly where the recomputation read the dimension's own subject out of the candidate and nowhere else. A term computed without consulting the dimension's subject cannot disagree with the target about the subject, and a model that cannot be wrong about a dimension is not checked about it by any number of agreeing digests. The stop is therefore recorded as the kind of unreviewed whose repair is a construction and not a rerun, which is the consequence the Wave-1 review put on this arc's desk by name when it said the census's silence about the input side is a claim resting on the precondition that no input bears an issuance. The disposition is recomputed and never handed in: the assessment reads the profile's required set on one side and the reviewed contract on the other, there is no argument, field or constructor anywhere that lets a caller state the answer, and it recomputes to review-incomplete naming exactly the issuance dimension. The accepted result lands as a typed value under the owner's option-B ruling, as the back half of a boundary whose out half the census already was: opaque authorization bytes plus the returned hash-type byte, bound to one candidate by exact byte comparison of the protected preimage in the pattern the finalized form already sets, with the deployment checked because nothing about a returned authorization reveals which chain it was formed for, and with the answers required to cover the census's signing inputs exactly once — a surplus answer and a missing one being different faults with different owners. The returned type byte is checked and then dropped rather than stored, because the ruling's third constraint is that a byte disagreeing with the profile is a refusal and not a variant, and a field for it would be either dead or a place for a second opinion to live. Nothing in that module parses an authorization and no accessor interprets one, so no digest crosses in either direction. The profile residual is NOT cleared, in any of the four vocabularies, and each of the four now names the dimension that stopped it instead of standing on nobody having looked; the computability blocker cleared by the Wave-3 run is untouched, the two are never simultaneous, and no matrix row, minimality pair, resource dimension or report standing moves. No new package dependency was taken to gate the accepted result on the disposition: the profile constants a returned answer is checked against already live beside the census from Wave 2, the establishment gate is the consuming guide's own handoff refusal rather than this type's, and the cross-package claim is stated once in the only crate that depends on the profile, the capability and the accepted result together. Verified entirely on the server per the no-host-compute ruling, every step timed, with no native run because this wave observes nothing and the observation it rests on already exists. |
| `T5-028` | DONE | Consensus-exclusion labeling, the register and its citations: eleven elected exclusions now mint a tag in a committed reference register and every code row that enforces one cites its tag, so the doc claim and the code refusal fail together rather than drifting apart. The register is written as a register and not a summary: each section states what is excluded, the ground with its existing citations rather than a re-derivation, where the code enforces it by crate and type and variant, and the revision surface a future revision would have to touch, written per section because the surfaces genuinely differ — some exclusions are one refusal reached by one test, and the issuance dimension is a profile decision carried in four separate vocabularies across four packages. The prefix half of the charter is answered by discovery rather than by invention, and the answer is that no new prefix is admissible and none is needed: the signature is closed under its registered families by the label calculus, the owner partition maps the whole of the planning tree to one owner by a single path rule, and a new prefix would need a signature row, an owner variant, an earlier-ordered partition rule, a harvest sink and a recorded decision — which is exactly the amendment the script-tree carrier record considered and declined, on the ground that an owner whose register is empty is a promise the corpus does not keep. The new document therefore takes the planning owner like every other reference register, and its labels are cited across owners in the ordinary bracketed form, which is the mechanism working rather than a workaround for it. Nineteen citations land across six modules: the decoder's peg-in, issuance-marker, issuance-proof and surjection refusals, the census's annex, type-byte, signature-width and issuance-bearing refusals, the materializer's unexpected-surjection and confidential-asset refusals, the profile's two dimension refusals, the target's profile-refusal ground, the internal-key policy whose single variant makes five other key sources unrepresentable rather than merely unused, and the materializer's module documentation, which is the enforcing row for the one exclusion whose enforcement is a component existing rather than a refusal firing. The issuance dimension lands as its own section under the post-verdict ruling that re-types it from required to refused, with the ground the verdict recorded — the two issuance terms hash one zero byte per input for every candidate this arc builds, so an agreeing digest exercises the input count the all-inputs dimension already carries and nothing about any issuance — and its enforcing rows today are the census and decoder refusals, which cite it alongside their own tags. The rows the follow-up sighash wave will add are named in that section's revision surface as owed rather than done, and the section says plainly that the profile still types the dimension required and the recomputed disposition still names it, which is the honest state of the tree. Nothing else moved: no refusal renamed, no variant added or removed, no dimension re-typed, no residual or standing touched, and every code edit is a doc comment. Found in passing and reported rather than fixed: the issuance-proof refusal is reached by no test anywhere in the repository, the peg-in refusal's witness-field raise site is likewise unreached while its outpoint-marker site is covered, and neither vocabulary has the exhaustiveness guard the census refusals have, which is why both gaps are invisible today — all three are recorded inside the sections whose revision surfaces would otherwise mislead a reviser into assuming coverage. The documentation bar carries one pre-existing failure inherited from the branch point, a public item linking to a private one in the finalization module, reproduced identically on the base commit and therefore not this wave's regression and not this wave's to repair. The brief's row inventory named the profile-refusal grounds in the tapscript authorization module; they are in the target-elements authorization module, and the tapscript module holds the profile-side refusal vocabulary instead, so both are cited rather than one guessed at. Verified entirely on the server per the no-host-compute ruling, every step timed: both plans validators green, the register generator's only delta the reference-kind mint count moving by one, the three owning packages green over eight hundred eighty-two tests, and the documentation bar clean for the two packages the inherited failure does not block |

Current blockers are:
