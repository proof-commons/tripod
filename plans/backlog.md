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

### 2.15 Structural-removals gate · `gate:backlog:structural-removals`

The removals arc closed 2026-08-25 across two waves (T5-041/T5-042 at the registry, T5-045 at the node) on the rule that a structural removal is complete only when the shape runs. The single-output solved-balancing form landed with the zero-blinder degeneracy refused by name, and the strict one-to-one transfer ran and was accepted; the fee output role joined the fixture vocabulary with its empty program required rather than permitted. The completion wave then ran the private merge and it is ACCEPTED — two receipts consumed, one output created, the forced blinder shown nonzero by summing the consumed coins against the registry's solve rather than assumed, over a three-output predecessor whose blinders cancel in no pair — moving the private-merge row on an observed acceptance of its own shape, the answered private count now seven of ten. The fee-bearing shape built a real fee output and typed-stopped at a third structural layer the second removal uncovered exactly as the second was uncovered by the first: the reviewed shape vocabulary has no sponsorless fee-bearing member, so the first-party covenant demanded a receipt program where the empty fee program sits; the census row is submitted-and-refused, the removal path is filed and not taken, because widening a reviewed reading is a ruling and not an edit. The latent conformance-record arity panic is fixed with width tests, and the earlier accepted shapes' digests are proven unmoved rather than presumed. Full suite at the arc tip: 47 of 48 meson lanes green in 492.5 seconds with the one failure a named infrastructure flake — the advisory-database fetch failed on a server-side network error, and the audit step rerun at the same tip passed clean in 1.5 seconds with zero findings — both validators clean and the register byte-stable.

### 2.16 Explicit-table and fee-vocabulary gate · `gate:backlog:explicit-fee`

Two parallel waves closed 2026-08-26 under one gate, their diffs disjoint by fence and their backlog rows the only merge contact. The explicit-runs wave (T5-046) moved the explicit positive table from zero to fifteen of sixteen — thirteen acceptances by a new shape-parameterized explicit-lane ceremony, each with mined bytes read back equal and every input's signature verified against an independently recomputed message, plus the two sponsor rows on the envelope lane re-run at the tip — and opened the negative half with two attributable refusals, keeping the submission-order lesson in the register: a witness mutant offered after its control refuses txn-already-known, which is about order rather than the row's class, so mutants go first. The sixteenth row is a typed stop that corrects an earlier prediction: sponsor-change-present needs an offer that takes change, not a symbol repair. The fee-vocabulary wave (T5-047) took the sponsorless fee-bearing removal path under the owner fee-matrix ruling and the confidential sponsorless one-to-one-with-fee shape was accepted, its census row moving off SubmittedAndRefused onto the observation with the removal recorded taken; the fee axis rides the shape bounds so exactly one never-accepted digest moved and every recorded identity re-derived bit-for-bit, and the taking uncovered and fixed a fourth structural layer, the deployment's welded fee-program digest. Full suite at the merge tip: 48 of 48 meson lanes green, zero failures, 895.1 seconds of wall time — roughly double the recent norm because the shared lane cache had been deliberately cleaned after the explicit wave's phantom-failure finding, so the run was cold; the duration is explained rather than regressed. Both validators clean and the register byte-stable.

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
**passed** (§2.8). The Guide-13 batch gate is §2.9: Phase 5 is not
exited, the batch having stopped honestly on the two typed blockers the
intermediate confidential-funding guide owns.

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
| `T5-001` | DONE | Guide-13 Wave 0 preflight reproduction: all eighteen review rows dispositioned. Sixteen CONFIRMED with committed reproductions in six packages. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T5-002` | DONE | Guide-13 Wave 0 preflight repairs: all eighteen rows closed across six serial-then-parallel lanes, each merged only after diff review and a server-side integration lane. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T5-003` | DONE | Two residuals named in the Wave-0 completion record discharged by the first Codex worker lanes (gpt-5.5, read-back gated, edits reviewed and committed by the orchestrator). Full narrative archived in [backlog history](history/backlog-history.md) |
| `T5-004` | DONE | Guide-13 Wave 1, the evidence handoff: every canonical negative vector states its whole declaration as typed data. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T5-005` | DONE | The ruled first-party discharge wired into derivation: derive_evidence_plan builds the canonical permissionless-private-dependency case. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T5-006` | DONE | Guide-13 Wave 2, the validated live-transfer target plan. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T5-007` | DONE | The compact-ASH plan validator gains the aggregate-census checks the live-transfer wave found missing. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T5-008` | DONE | Guide-13 Wave 3, signature and CT target closure: the confidential-conservation evidence requirement follows the selected proof alternative. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T5-009` | DONE | Three residual discharges by parallel Codex lanes under the verify loop, merged serially after review. The operation transcript is buildable only by its planner. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T5-010` | DONE | Guide-13 Wave 4, the candidate live-receipt constructor: canonical owner metadata derived from the target's own encoding registry. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T5-011` | DONE | Guide-13 Wave 5, recognition and owner authorization: the abstract walk learned to say what section 1.8 asks. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T5-012` | DONE | Guide-13 Wave 6, the explicit live-transfer plan: the coordinator gains its global checks. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T5-013` | DONE | Guide-13 Wave 7, the private live-transfer plan. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T5-014` | DONE | Guide-13 Wave 8, the linked live-transfer candidate. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T5-015` | DONE | Guide-13 Wave 9, the candidate transaction ABI and signing flow. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T5-016` | DONE | Guide-13 Wave 10, the complete live-transfer safety evidence and the run against a real target: the whole hundred-and-eight-row section-15 matrix lands as constant registries in guide order. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T5-017` | DONE | Guide-13 Wave 11, the disclosure-minimality comparison and the first-party completion of the safety matrix. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T5-018` | DONE | Guide-13 Wave 12, the resource study: the three bound symbols land with the guide's own research candidates. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T5-019` | DONE | Guide-13 Wave 13, the Phase-5 gate and handoff. Full narrative archived in [backlog history](history/backlog-history.md) |
| `T5-020` | DONE | Intermediate confidential-funding guide, Wave 0, elaborating custody and reproducibility: all four preflight findings of the guide's own repository reading are CONFIRMED, each by running code rather than by reading, and nothing whatever is implemented. The two proof-field refusals fire on any nonempty field with no reference to the output's value form, reproduced by decoding one byte string carrying a sixty-four-byte range proof against the hybrid form the guide fixes — explicit asset, committed value, empty surjection field — and observing the same refusal the explicit form draws, with the surjection refusal for the hybrid form kept as a standing guarantee because the repaired behaviour owes it too. The protected preimage is byte-identical to the witnessless serialization, which is exactly the empty-output-witness case the recorded target diagnosis names, reproduced on the first-party side by finalizing a private transfer through the ordinary construction path and comparing the two byte strings; the target half is cited from the existing runnable diagnosis and was not rerun. The proof-fields datum is carried by the spent outputs alone, reproduced by reading the selected owner profile's own coverage and observing a one-member carrier set, and the created outputs' dimension the repair must add is already a required dimension of the profile, so the widening asks nothing new of the target. The per-output confidential value role is a characterization rather than a defect and is confirmed as one across four measured limits: the role is told the protocol asset, one amount, the published randomness, and a position and nothing else, one call per destination in request order; a capability answering every call with one constant commitment finalizes without complaint, so no blinder sum, conservation, or cross-destination relation is checked anywhere in the private path; every output takes the null nonce; and the output type has no proof member at all, so the encoder writes two empty proof fields per output and there is nowhere for a range proof to go. Three rulings of the concept and three of the guide's charter decisions are carried forward and checked against the real types rather than restated: the accepted custody model has its typed name in the transaction package and is the only model the construction recorder admits, the reviewed target package really does carry no dependency and really does own the confidential field encoding the contract enum would sit beside, the reproducibility contract's four comparison surfaces correspond one to one with the concept's own sentence, and the terminal-audit-closure classification the no-allocation decision rests on is the Phase-5 card's own words. The dependency boundary is re-read from the three manifests and holds unchanged: the conformance package reaches the transaction and vectors packages as development dependencies only and its manifest forbids promoting either, the transaction package names the conformance package nowhere and records the absence as load-bearing, and the vectors package is the one library where both are reachable. Six divergences between the guide, the rulings, and the tree are reported to the orchestrator for recording in the guide rather than repaired here, the sharpest being that two of the three protected-bytes repair parts are not expressible at this tree at all until the output-witness census row lands first. Verified on the server per the no-host-compute ruling: the worker lane green over both owning packages with the formatter and the workspace clippy pass, and the confirmed rows reproduced by a separate run of the ignored reproductions whose observed failures are the evidence |
| `T5-021` | DONE | Intermediate confidential-funding guide, Wave 1, revising the funding wire: the confidential arm lands as a fifth untagged operation subject with its own step kind and its own capability, and the explicit funding subject is not re-shaped — a tagged union inside a variant of an untagged enum is the shape that file refuses, and the two arms stay untagged-disjoint by member set, sharing exactly the asset question and nothing else, which a test holds rather than a comment. The request types carry destinations and a binding and there is no member an amount could be written into: a validated public handle, a thirty-two byte drift digest, and four closed profiles of one member each, so a second profile is an added variant a peer must advertise rather than a silent change of meaning. The response gains the two members the guide fixes, neither defaulted, and that is the whole revision: the sponsor members were defaulted so adding them was not one, and these are not so that adding them is, with the protocol revision moved from four to five in the harness and the reviewed native adapter in the same commit — the adapter reads the confidential subject strictly, refuses every unknown tag and undeclared member before construction, advertises no confidential capability because it implements no deterministic materializer, and writes both new members on every operation answer, and a contract test carries its refusal record in the adapter's exact spelling so the two implementations are checked against each other rather than each against itself. The reproducibility contract is placed once, in the reviewed target package, as candidate ceremony vocabulary, with that package's contract note and crate documentation widened in the same change to say so; the target package still serializes nothing and gains no dependency, and the wire encodes the contract through the vocabulary's own code so the workspace holds one word per contract rather than two spellings held equal by a census. The complete typed refusal vocabulary lands closed with no catch-all, twenty-nine variants, every one of them reachable from a test and none of them a target verdict, and readback binding takes no output fact from a request echo: the identities are recomputed from decoded bytes, the block is an independent observation, and the proofs, commitments, nonces, programs, and scripts are read out of the decoding, with the request checked only for the arm, the handle, the digest, and the profiles. The three collaborators the binding needs are traits implemented outside the package, so no library edge is added in either direction, and no test reaches a target: the executor's side is scripted, the chain is a stub, and the only real arithmetic is the first-party commitment oracle, used so an admitted encoding can be told from an inadmissible one. Two precisions are reported rather than assumed: the thirty-three byte commitment and nonce travel as byte vectors because the serialization framework implements no array reader past thirty-two, which makes both encoding refusals reachable by width as well as by prefix; and the materializer refusal is typed by the non-verdict layer the executor reported, because the response carries no cause member and a first-party derivation vocabulary would be a second spelling of one the later waves own. Verified on the server per the no-host-compute ruling: the formatter, the workspace clippy pass, and the three owning packages green, with five hundred one conformance tests and two hundred sixty-two vectors tests passing and check-plans green on the plans clone. No blocker moved, no matrix row moved, and nothing here claims a transfer, a proof, or a production property |
| `T5-022` | DONE | Owner-sighash parallel work, Waves 0 and 1, recording the charter and reviewing the target's message from source: nothing is implemented, no digest is asserted, and no dimension moves to reviewed. Wave 0 is the record. The three rulings of the owner-sighash concept are carried forward as recorded and are never reopened: the profile decision, at the concept's profile-decision rule, fixes a default-only type byte, one profile for both lanes, a refused annex, and script path only; the accepted-result decision, at the concept's result-decision rule, fixes option B — protected bytes, the signing-input census, and opaque authorization bytes plus the returned hash-type byte, bound to one candidate identity — with its three constraints, that the census is observed target data and candidate structure only, that the binding is exact-byte in the pattern the offered-transaction check already sets, and that the returned hash-type byte is checked against the profile rather than merely recorded; and the evidence decision, at the concept's evidence-decision rule, fixes source review plus recomputation plus one observed acceptance, populating the reviewed capability dimension by dimension, each dimension carrying both its citation and the observation that exercised it. The profile's constants are stated here as the values later waves assert rather than choose: the hash-type byte is 0x00; the authorizing signature is 64 bytes wide, because a 65-byte signature is one carrying a trailing type byte and the target refuses a trailing zero, so the already-measured unauthorizing-signature width stays true; the spend-type byte is 0x02 for every candidate this arc produces, being the script-path extension flag shifted with a zero annex bit; and the spend path is script path only, so the tapleaf hash, the key version, and the codeseparator position are message terms for every input this work ever signs. The option-B census is named field by field: the protected bytes; the output-witness vector at its consensus length; the spent-output census per input, carrying each spent output's asset field, value field, and script; the deployment's genesis block hash; and, per signing input, the input index, the tapleaf hash, the leaf version, the codeseparator position, and the annex disposition. Its exclusions are a rule and not an omission: no opening, no blinder, no nonce input, no key, and no proof input enters the census under any lane, which is what keeps a secret-bearing interface from being selected at all. The review boundary is recorded in both directions: no wave of the confidential-funding guide reviews this profile, and no wave of this work reviews confidential funding, the transaction-wide materializer, or blinding, and the confidential-predecessor funding blocker is untouched by every wave here under every result for every row. The census's ADR-015 disposition is public test material under the existing test-material rule: every key, signature, chain, and census value this work touches is disposable regtest material authorizing nothing anywhere else, and because the census excludes openings by rule the design gate for a secret-bearing interface is never reached. The package and dependency boundary is recorded without implementation by implication: the reviewed target package owns the reviewed sighash capability and its dimension vocabulary; the tapscript package owns the selected profile, its coverage map, and its disposition; the transaction package owns the finalized form and the signing request, and is therefore the one surface a widening can happen on without forking the signing path; the vectors package owns the blockers, the matrix classification, and the evidence that moves a standing; and no dependency is approved by naming a need. Wave 1 is the source review, landing as a new technical reference beside the tapscript one, censused and indexed in the same commit: the taproot message is read term by term at the pinned Elements tip, nineteen terms with the position each is written at and the helper each is computed by, and each of the selected profile's eight required dimensions is read against the term that carries it. Seven of the eight are established from source in the only sense a reading can establish anything — the term exists, its place in the stream is fixed, and its content is the dimension's own subject. The eighth is settled the other way and it is the wave's sharpest finding: the spent-scripts term commits each spent output's script, which for a taproot output holds the tweaked output key and not the internal key, so the message construction carries no internal-key term at all; the internal key is bound to that output key by the control-block check that runs before the leaf executes, which is a witness check the message never covers, and the message is identical for any two internal-key-and-path pairs tweaking to the same output key under the same leaf, so what exists is a composition and a hardness argument rather than a commitment. The consequence is typed rather than rhetorical: the profile recomputes its disposition from the required set, so one permanently unexercisable member holds it at review-incomplete forever, and Wave 4 cannot reach an established disposition until the owner chooses between re-typing the dimension as carried by the spent outputs and keeping it required with a control-block check named as what exercises it — a reading the coverage map supports from the other side, since it assigns no protected datum to that dimension at all. Both refused dimensions are checked against the branches that would have carried them and both refusals hold, with the second understating itself: refusing the permitted-extension dimension does not merely leave the input set open, it drops seven whole-transaction terms and replaces them with the signing input's own, which is what makes the spent-outputs dimension a whole-transaction dimension in the first place. The issuance-rangeproof hazard is reproduced and refuted at once, by the recorded diagnosis's own method extended so the two witness vectors vary independently and four candidate messages are computed per signature instead of two: the input-side term is length-dependent exactly as its output-side sibling is, and the target's own signer does not fall into it, because the wallet resizes the input-witness vector to the input count before any signing begins and nothing resizes the output-witness vector until serialization, which the run confirms by the explicit row's signature verifying against the output-side-emptied candidate and against no other while the accepted confidential control verifies against the fully grown one. So the twin is confirmed as a property of the message and refuted as a defect of this signer, the recorded output-witness finding stands alone in that path, and no new upstream entry is earned because nothing here costs an adaptation the filed one does not already cost. What it does cost is a precondition on the accepted census: the input-side term is recoverable from the input count only while no input bears an issuance, the profile admits issuance-bearing candidates, and the census wave owes either a field or a typed refusal for that shape. The refused single-output branch is recorded as an observation with teeth: it bounds-checks the signing position against the output list and then indexes the output-witness vector at that position, the two lengths agree on every wire form and disagree on every signing path, and asking the node for that hash type through an ordinary wallet request ended its process on signal eleven with no shutdown record written — not a consensus defect, since no wire form reaches the read, but a wallet-reachable one, minted as an upstream friction whose entry at the owning register is owed and could not be filed by this lane. Seven message terms are named by no dimension and are dispositioned rather than left implicit: two are census inputs, three become constants a test asserts, one is absent by the annex refusal, and one is a census field whose value is constant while no leaf emits the opcode that moves it. The reviewed sighash capability is still constructed with an empty reviewed set and every dimension unreviewed, the profile still assesses review-incomplete naming all eight, and the residual stands in every vocabulary that carries it: a source review alone populates nothing, and the review document says so in its own section rather than leaving it to be inferred. Verified entirely on the server per the no-host-compute ruling, every step timed: the twin diagnosis and the recorded output-witness diagnosis each ran about a second against the server-built node at the reviewed tip, check-plans passed in the plans clone, and the five documentation and census lanes passed at the branch tip after the label register was regenerated by its meson target and its one-count diff reviewed as text before being applied |
| `T5-023` | DONE | Intermediate confidential-funding guide, Wave 2, proving one confidential predecessor: the fixture registry lands as the only lookup authority, with the freeze a type transition rather than a flag — registration consumes the mutable registry to produce the frozen one, so a late write is not a rule that could be broken but a value that no longer exists — and the handle grammar, the drift digest, and the deterministic derivation recipe land with it. The handle is a validated public identity whose one digit belongs to the grammar version, so a later grammar is a different prefix rather than a reinterpretation of this one, and eight spellings are refused by name at the clause each fails. The digest is a tagged hash over a framed transcript whose every member carries its own width, and the contract tag sits inside that transcript rather than beside it, which is what makes a semantic-only recorded-randomness digest impossible to mistake for a byte-identity one — both contracts are registered and their digests are held apart by a test rather than by intent. Derivation is domain-separated by role and by case and the search is bounded, upward from zero, without wrapping, skipping, randomness, or concurrency; the balancing blinder is solved from the input blinder sum minus the others rather than derived, and an independent recheck reads every blinder back and adds them again, so the predecessor's ordered additive inverses are an instance of the general rule rather than a special case. One deterministic funding transaction meets the fixed shape exactly and was submitted, accepted, mined, and read back raw from a real node: one explicit protocol-asset input contributing a zero value blinder, spent from a coin mined at exactly the fixture's total by a separate explicit transaction that is not the funding transaction, because a funding transaction returning protocol-asset change would carry a third protocol member and turn the balance into an argument about which region a member was put in; exactly two protocol outputs carrying the explicit asset and positive confidential values summing to the input, zero asset blinders and therefore an empty surjection field on both, the two admitted commitment parities in fixed order under the bounded counter search, a deterministic nonce per output, and a valid rangeproof per output bound to that output's commitment, the unblinded asset generator, and the program. The policy-asset change and fee are classified into an explicit non-protocol region by an exhaustive classifier with no member it may decline to place, and six typed refusals cover a member that carries the protocol asset, a commitment, or a proof outside that region and a protocol position that carries none of them. Recomputation and raw readback agree on all eight members of the funding agreement census for both outputs, in both runs, and every comparison is between two distinct origins: the commitment and its parity are the first-party bignum oracle against the chain, the program and the nonce are the registered fixture against the chain, the proof shape is the reference verifier against the chain, and the asset, the outpoint, and the witness transaction identity are the materializer's own answer against the chain, with a comparison whose two sides carry one origin refused where the census is assembled rather than reported as agreement. The two independent commitment comparisons are carried by opaque newtypes whose only constructors are first-party arithmetic and a readback decoding, so a comparison of a readback with a readback is not a rule but an unspellable expression. The validated record's only constructor is the validator, which does not check an offered record but builds one by recomputing every member in the fixed order, with a named refusal per step and the summary recomputed from the members rather than carried, and it carries all ten non-claims and all nine canonical exclusions as sets rather than as prose. The executor gained the deterministic materializer the arm needs and now advertises the capability and the advertisement together on one condition, because the harness refuses a peer that states either without the other; it holds the same public fixture catalogue and recomputes the digest from its own arithmetic, so a drifted catalogue is a refusal rather than a different materialization. Three findings are reported rather than smoothed over: the target refuses a rangeproof whose proven minimum is zero unless the program is unspendable, which is a consensus rule the first attempt met head-on and the target's own blinding path states from the other side; the readback decoder is the reference library rather than the first-party one, because at this tree the first-party decoder refuses any nonempty proof field and that repair belongs to the next wave; and the inclusion condition is discharged by the adapter's own reading of the block rather than by a second query from the harness, because the wire carries no member a block's contents could arrive in. Verified on the server per the no-host-compute ruling: the formatter, the workspace clippy pass, five hundred nineteen conformance tests and two hundred sixty-two vectors tests, and the live native lane green against the server's real node, with the ceremony run twice against two disposable chains and the two funding transactions compared byte for byte rather than the byte-identity contract being asserted from its own name. Exactly one residual moved, on an observed result rather than on a capability existing, and it moved at its owning boundary only; no matrix row moved, and neither owner-sighash blocker was touched |
| `T5-024` | DONE | Intermediate confidential-funding guide, Wave 3, materializing private transactions transaction-wide: the four serialization changes land together because three are one another's preconditions, and the encoder's bytes for every existing lane are unchanged by them — the transaction gains an output-witness vector, one entry per output, modelled as a transaction-level vector rather than an output field because that is where the target's own serialization puts it and because the hazard is the vector's length rather than any output's content; the witness predicate becomes a disjunction over both vectors, since a confidential funding transaction has null input witnesses and proofs its outputs are invalid without, and the old conjunction would have serialized it without the section carrying them; the superfluous-section condition follows and is a widening rather than a weakening, admitting every byte string it admitted before; and the range-proof refusal becomes conditional on the output's own value field in both directions, with the requiring direction new, while the surjection refusal stays unconditional because that is already the form-conditional answer for every form this workspace builds, an asymmetry recorded at the refusal rather than left to be rediscovered. The protected-bytes repair is scoped rather than universal: the private lane's preimage is the witnessless serialization followed by the output-witness vector, so an owner binds to bytes containing the range proofs the target's digest covers, and the explicit lane's preimage is unchanged byte for byte because its output-witness vector is empty under both readings and widening a settled lane's bytes to say nothing new would move a signature's preimage under every existing owner for nothing. Both Wave-0 reproductions lose their ignore attributes and stand as the repaired properties' guarantees, and the sharpest test drives the difference directly: two candidates differing in exactly one range-proof byte, same length, same commitments, same nonces, same programs, same version and lock time, have different protected preimages and identical witnessless serializations, which is the omission the repair closes stated as a measurement rather than as an argument. The transaction-wide materializer is one entry point taking an intent, a frozen fixture view, and two injected collaborators, with the preflight's five clauses before any cryptographic work and the seven stages in order after it; the cryptography is injected because the construction package may not depend on the package that is the independent oracle its output is compared against, and the two collaborators' answers travel in two different opaque types so that the comparison cannot be handed the same origin twice — a property carried by the type system rather than by a reviewer's attention. Stage four refuses a check that declares the construction's own origin, the reference bindings' origin, or the target's, because the reference bindings bind the same library the target vendors and their agreement would be conformance evidence wearing independence's name. The refusal vocabulary lands closed with no catch-all, thirty-two variants matching the guide's own table exactly, every one a construction refusal and none a target verdict; twenty-nine are reachable from a test and the remaining three are structural guards the construction cannot violate without being edited to disagree with itself — the two census guards and the freeze's round-trip check — which is reported rather than papered over. The frozen candidate has private fields, a constructor only the materializer reaches, a region census of eight, and seven spelled mutation attempts that each refuse naming their region, so the census is walked by a test rather than sampled. The third origin-tagged commitment newtype completes the independence rule where its two siblings already live, with two comparison functions each taking two different types, and the boundary widening it implies is recorded in the vectors manifest in the same change: what crosses that package's conformance edge is now two boundaries, the executor and the commitment oracle, with the reason the second is not the forbidden direction stated exactly — the forbidden direction is obtaining an expectation from the component that will produce the observation, and the oracle produces no observation, computing curve arithmetic over published constants and never touching a node. Both origins are real where both are reachable: the first-party bignum oracle and the reference bindings compute the same thirty-three bytes for every registered opening and agree with what the registry derived, and every range proof the materializer produces verifies through the reference verifier against that output's own commitment, the unblinded asset generator, and its program, while a proof checked against the other output's program does not verify — so the binding is measured rather than assumed. No target has seen any of it, nothing here is a verdict about a chain, and the proof generation and verification are both conformance evidence and are not independent evidence. The derivation recipe gains a test-only source seam, authorized and bounded: pub(crate), sitting between the preimage and its bytes, with the tagged hash as its only non-test implementation and the public entry point a one-line call selecting it, changing no public signature and no registered digest, which a test checks rather than asserts. Two derivation refusals become reachable through it, the degenerate solved balancing scalar and the exhausted bounded scalar search with its role and attempt count. The identity value commitment does not, and the reason is a property of the arithmetic rather than a gap in the seam: a source chooses bytes and does not choose what the oracle makes of them, and a seam that reached the oracle would have made the independence claim checkable by substitution — so that half is discharged at the layer that genuinely injects its cryptography, where the materializer's invalid-commitment refusal covers it. The per-output role is retired by scope and not by wish: the range-proof non-claim is dropped for the transaction-wide lane through a separate constructor, stays in force for the per-output lane it still describes, and the other three non-claims stand unchanged under both, while selecting the per-output role at the transaction-wide entry point is a typed refusal before any cryptographic work. One deliverable is not done and is reported as not done: the private branch of the live finalization does not yet take the transaction-wide path, because that call site carries no intent, no fixture view, and no collaborators, and supplying them is a signature change across every caller that the evidence lane's own restart owns. The sites that named the funding blocker as the reason a private materialization is not constructible are re-pointed in prose with every typed value and every standing left exactly where it is: a confidential predecessor is fundable and one has been mined, so the condition is scoped to this pipeline's own funding step, which still has no confidential form and still consumes no predecessor funded through the other arm. The narrower condition that actually keeps the private half blocked has no name of its own in that vocabulary, and minting one was reported rather than done. Verified on the server per the no-host-compute ruling, every step timed: the formatter, the workspace clippy pass, and the three owning packages green. No native run, because the materializer is first-party and this wave's exit needs no target run. No blocker moved, no matrix row moved, and neither owner-sighash blocker was touched |
| `T5-025` | DONE | Owner-sighash parallel work, Wave 2, the signing-input census and an independent message construction: the census lands as a typed value carrying exactly the fields the accepted option-B result names — the protected bytes, the output-witness vector at its consensus length, the spent-output census per input with each spent output's asset field, value field and script, the deployment's genesis block hash, and per signing input the index, the tapleaf hash, the leaf version, the codeseparator position and the annex disposition — and it carries the frozen candidate beside them as candidate structure, because the private lane's preimage is the witnessless serialization followed by the output-witness vector and is therefore not a decodable transaction, so the version, lock-time, input and output terms cannot be read back out of the bytes the owner binds to. There is no public constructor: the one public route takes a materialized confidential candidate, which the materializer alone produces, so no route skips finalization, and the route takes the materialized value rather than the frozen candidate inside it because the frozen candidate does not carry the spent-output census and a caller assembling one beside it would be the skipping route. The exclusion rule is enforced by shape rather than by review: there is no field an opening could be written into, and the one place a key would naturally appear is the control block, which travels as a constructor argument the taproot commitment check consumes and drops, so nothing key-shaped is retained. The message construction is written from the Wave-1 source review's own term table, every term cited at the line the review cites, and the independence is real rather than declared — a second authoring of the same nineteen-term stream, spelled out byte group by byte group at the call site, agrees with the module's construction, and the boundary of what is shared is stated in the module rather than left to be inferred: the tagged-hash primitive and the output, output-witness and field serializations are this crate's own evidenced product, and respelling them would have created a second opinion about encoding free to drift from the bytes the crate emits, so the encodings are shared and the stream is independent. The four witness treatments the twin diagnosis named are all present and all four messages are distinct, which is what lets a later run say which of the two length-dependent terms moved rather than only that something did, and the two candidate messages the recorded diagnosis compares are pinned as constants a test holds, both computed over a deterministic first-party candidate whose every field is a fixed byte string, so the pair is replayable rather than a snapshot of one run. That pinning is a divergence from the wave's entry condition as written and it is reported as one: the recorded diagnoses persist no digest anywhere in the repository, both generate fresh wallet keys on every invocation and print their digests to standard output alone, and both exercise the key path while the selected profile is script path only, so there were no recorded values to reproduce and this wave takes no target run — what is landed is the recorded method reproduced over a fixture that can be replayed, and the cross-origin comparison against a node belongs to the observation wave. The profile's constants are asserted as constants and the spend-type byte is recomputed rather than compared with itself, so a recomputation that had stopped depending on the annex would fail rather than pass. The refusal vocabulary lands closed with no catch-all, fifteen variants, every one of them a construction refusal and none a target verdict, and every one reached by a named test through a crate-private test-only parts seam that runs the same clause list as the public route rather than a relaxed one — including the refusal the source review put on this wave's desk by name, since the census's silence about the input-witness vector is a claim resting on the precondition that no input bears an issuance, and a declared issuance is now a refusal rather than an assumption. The internal-key dimension is re-typed onto the spent outputs under the ruling: the required set becomes seven message-carried dimensions, the role vocabulary grows a third member rather than one of the two being stretched, and the review's composition argument travels with the dimension at the point of use with its citations, so a reader who asks why the set is seven finds the argument and not a dimension that quietly stopped being mentioned; the coverage map assigned the internal key no protected datum, which a test measures rather than asserts, so no protected datum moves. The proof-fields coverage remap lands with no tension to report: the map becomes set-valued, which is the type change the confidential-funding guide fixed as a type change rather than a value edit, the proof fields name the created outputs' dimension alongside the spent outputs', both carriers are dimensions the profile requires, and the totality argument moves with the type and is recomputed by a published function rather than left to a lookup that could only fail one of its two ways. The guide's ignored preflight reproduction loses its ignore attribute and stands as the repaired property's guarantee, and the conformance package's committed-protected-data derivation is re-read as any rather than all, because a datum is committed as soon as one dimension carrying it is required and reading it the other way would have dropped a datum out of the set the moment the map recorded a second carrier for it. No digest is asserted as authoritative, no dimension moves to reviewed, the sighash capability is still constructed with an empty reviewed set, the profile still assesses review-incomplete, and both the profile residual and the computability blocker stand in every vocabulary that carries them; no matrix row moved and the confidential-predecessor blocker was not touched. Verified entirely on the server per the no-host-compute ruling, every step timed, with no native run because this wave observes nothing |
| `T5-026` | DONE | Owner-sighash parallel work, Wave 3, observing the selected profile on the explicit lane: one finalized explicit candidate was authorized against the Wave-2 message, submitted to a real node, and accepted into a block at height five. The census had one route and it took the proof-finalized value, so the explicit lane could not reach a message at all; the second route lands on the same only-route discipline, taking the value that lane's own finalization produces and reading every member out of it, and the two members it needed were ones construction already read and dropped, since the public view states each consumed outpoint's asset, value and program while the receipt record kept the value alone. A sponsor-bearing candidate is refused as the cardinality mismatch it is rather than repaired, because construction never reads a public view for a sponsor input and letting a caller supply the missing entries would be the skipping route the finalized form exists to prevent; that gap is reported and not closed. Acceptance and recomputation stayed two origins: the node formed its own message and ran its own verifying primitive, and then the accepted transaction was read back out of the node, the signature was taken from those bytes rather than from the value the builder still held, and it was verified against a message recomputed here from the source review's own term table with this workspace's own curve arithmetic. The read-back bytes are the submitted bytes, the signature verifies against the recomputed message, and it does not verify against the empty-vector message, so the pair the recorded diagnosis compares is distinct on the accepted witness rather than only in the abstract. Reading the accepted witness back needed a protocol change, because the submission step returned an identity and nothing else: a mined readback is now a member the submission kind owns alongside the confidential funding kind, an accepted submission that omits it is a shape defect, and the adapter reads the confirmed transaction back out of the node after the block it made rather than projecting it from what it was handed. Six negative controls were offered to the same node on the same chain before the accepted case, each moving exactly one term, and every one was refused with the node's own invalid Schnorr signature verdict: the output-witness vector emptied, the genesis hash of another chain, the non-selected type byte appended to a sixty-five byte witness, a valid signature by the other published owner, a valid signature over a candidate whose destinations are exchanged, and the deployment seed in the order a target prints a block identity rather than the order it hashes it in. Two of them were refused on this side as well and the construction refusal is recorded beside the target verdict rather than merged with it, so the deployment mismatch and the two profile answer checks are first-party refusals and never verdicts. The byte-order control is the wave's own finding and it is measured rather than read: the first run refused every case including the selected one, the deployment seed is the single term of the stream carrying a value a run printed rather than a value this crate encoded, and offering both orders to the same node settled which one the target seeds with. The spent-output triple every signature is taken over is the node's own report of the funded coins rather than the ceremony's expectation of them, with the expectation compared against it and recorded, because the message commits to the spent asset, value and program and signing over what a builder assumed would be signing a different message than the target forms. The computability blocker is cleared at its owning boundary on that observed acceptance and never on a capability existing: the positive rows stop being infrastructure-blocked and become rows a run could answer, the negative half stops being blocked on the absent control for the same reason, the three rows carrying a blocker of their own do not move, and not one row becomes answered because a standing is not evidence and no run of any row has been filed. The profile residual stands in all four vocabularies, untouched, because a digest that can be computed is not yet a settled claim about what it commits to and that residual is cleared by a review verdict rather than by a run; no dimension moved to reviewed and the sighash capability is still constructed with an empty reviewed set. The observation establishes nothing about the proof-bearing lane, in the code and in the artifact rather than by implication, because the output-witness term is recoverable from the protected bytes exactly while every entry is default-constructed and the proof-bearing lane's entries carry range proofs the preimage does not contain in any form. Verified entirely on the server per the no-host-compute ruling, every step timed, with the native run the evidence of record. |
| `T5-027` | DONE | Owner-sighash parallel work, Wave 4, the review verdict and the accepted result: the verdict is a TYPED STOP and it stops on one named dimension. The reviewed sighash capability stops being two sets of bare members and becomes two maps to grounds, because a bare member was free while both sets were empty of content and stops being free the moment a dimension moves — a reviewed set of bare members could be grown by an edit that read no source and ran nothing, and an unreviewed set of bare members cannot tell a dimension nobody reached from one that was reached and could not be exercised. A reviewed dimension now carries both halves the accepted evidence ruling requires, the source review's citation and the observation that exercised it, and an unreviewed one carries which of three different kinds of unreviewed it is. Six of the profile's seven required dimensions move to reviewed on the Wave-3 acceptance: the two output-side terms, the three whole-transaction input terms, the version and lock-time terms, the two spent-output terms, and the tapleaf term, each cited at the line the Wave-1 review cites and each exercised by the same accepted transaction. The candidate that acceptance was taken over consumed two receipts and created two destinations, which is what makes the four whole-transaction dimensions exercised rather than merely present — a single-input, single-output candidate would have left every all-versus-one distinction unobserved and the honest verdict would have been narrower. Two of the six carry a negative control besides, both on the output side, since the ceremony's six controls moved exactly three of the message's nineteen terms between them: the deployment seed twice, the output list once, and the output-witness vector once. The seventh required dimension does not move, and the reason is a property of this workspace rather than of the target. The target's message does carry the issuance dimension, in the two terms the review cites, but no candidate this arc builds bears an issuance — the census refuses a declared one by type rather than carrying a field for its input-witness proofs — so the construction that recomputes the message reads no issuance field at all and forms both terms from the input count alone. That is the line the verdict is drawn on: the acceptance and the recomputation agree over the whole stream at once, so a model wrong about any term would have produced a different digest, which makes the agreement per-dimension evidence exactly where the recomputation read the dimension's own subject out of the candidate and nowhere else. A term computed without consulting the dimension's subject cannot disagree with the target about the subject, and a model that cannot be wrong about a dimension is not checked about it by any number of agreeing digests. The stop is therefore recorded as the kind of unreviewed whose repair is a construction and not a rerun, which is the consequence the Wave-1 review put on this arc's desk by name when it said the census's silence about the input side is a claim resting on the precondition that no input bears an issuance. The disposition is recomputed and never handed in: the assessment reads the profile's required set on one side and the reviewed contract on the other, there is no argument, field or constructor anywhere that lets a caller state the answer, and it recomputes to review-incomplete naming exactly the issuance dimension. The accepted result lands as a typed value under the owner's option-B ruling, as the back half of a boundary whose out half the census already was: opaque authorization bytes plus the returned hash-type byte, bound to one candidate by exact byte comparison of the protected preimage in the pattern the finalized form already sets, with the deployment checked because nothing about a returned authorization reveals which chain it was formed for, and with the answers required to cover the census's signing inputs exactly once — a surplus answer and a missing one being different faults with different owners. The returned type byte is checked and then dropped rather than stored, because the ruling's third constraint is that a byte disagreeing with the profile is a refusal and not a variant, and a field for it would be either dead or a place for a second opinion to live. Nothing in that module parses an authorization and no accessor interprets one, so no digest crosses in either direction. The profile residual is NOT cleared, in any of the four vocabularies, and each of the four now names the dimension that stopped it instead of standing on nobody having looked; the computability blocker cleared by the Wave-3 run is untouched, the two are never simultaneous, and no matrix row, minimality pair, resource dimension or report standing moves. No new package dependency was taken to gate the accepted result on the disposition: the profile constants a returned answer is checked against already live beside the census from Wave 2, the establishment gate is the consuming guide's own handoff refusal rather than this type's, and the cross-package claim is stated once in the only crate that depends on the profile, the capability and the accepted result together. Verified entirely on the server per the no-host-compute ruling, every step timed, with no native run because this wave observes nothing and the observation it rests on already exists. |
| `T5-028` | DONE | Consensus-exclusion labeling, the register and its citations: eleven elected exclusions now mint a tag in a committed reference register and every code row that enforces one cites its tag, so the doc claim and the code refusal fail together rather than drifting apart. The register is written as a register and not a summary: each section states what is excluded, the ground with its existing citations rather than a re-derivation, where the code enforces it by crate and type and variant, and the revision surface a future revision would have to touch, written per section because the surfaces genuinely differ — some exclusions are one refusal reached by one test, and the issuance dimension is a profile decision carried in four separate vocabularies across four packages. The prefix half of the charter is answered by discovery rather than by invention, and the answer is that no new prefix is admissible and none is needed: the signature is closed under its registered families by the label calculus, the owner partition maps the whole of the planning tree to one owner by a single path rule, and a new prefix would need a signature row, an owner variant, an earlier-ordered partition rule, a harvest sink and a recorded decision — which is exactly the amendment the script-tree carrier record considered and declined, on the ground that an owner whose register is empty is a promise the corpus does not keep. The new document therefore takes the planning owner like every other reference register, and its labels are cited across owners in the ordinary bracketed form, which is the mechanism working rather than a workaround for it. Nineteen citations land across six modules: the decoder's peg-in, issuance-marker, issuance-proof and surjection refusals, the census's annex, type-byte, signature-width and issuance-bearing refusals, the materializer's unexpected-surjection and confidential-asset refusals, the profile's two dimension refusals, the target's profile-refusal ground, the internal-key policy whose single variant makes five other key sources unrepresentable rather than merely unused, and the materializer's module documentation, which is the enforcing row for the one exclusion whose enforcement is a component existing rather than a refusal firing. The issuance dimension lands as its own section under the post-verdict ruling that re-types it from required to refused, with the ground the verdict recorded — the two issuance terms hash one zero byte per input for every candidate this arc builds, so an agreeing digest exercises the input count the all-inputs dimension already carries and nothing about any issuance — and its enforcing rows today are the census and decoder refusals, which cite it alongside their own tags. The rows the follow-up sighash wave will add are named in that section's revision surface as owed rather than done, and the section says plainly that the profile still types the dimension required and the recomputed disposition still names it, which is the honest state of the tree. Nothing else moved: no refusal renamed, no variant added or removed, no dimension re-typed, no residual or standing touched, and every code edit is a doc comment. Found in passing and reported rather than fixed: the issuance-proof refusal is reached by no test anywhere in the repository, the peg-in refusal's witness-field raise site is likewise unreached while its outpoint-marker site is covered, and neither vocabulary has the exhaustiveness guard the census refusals have, which is why both gaps are invisible today — all three are recorded inside the sections whose revision surfaces would otherwise mislead a reviser into assuming coverage. The documentation bar carries one pre-existing failure inherited from the branch point, a public item linking to a private one in the finalization module, reproduced identically on the base commit and therefore not this wave's regression and not this wave's to repair. The brief's row inventory named the profile-refusal grounds in the tapscript authorization module; they are in the target-elements authorization module, and the tapscript module holds the profile-side refusal vocabulary instead, so both are cited rather than one guessed at. Verified entirely on the server per the no-host-compute ruling, every step timed: both plans validators green, the register generator's only delta the reference-kind mint count moving by one, the three owning packages green over eight hundred eighty-two tests, and the documentation bar clean for the two packages the inherited failure does not block |
| `T5-029` | DONE | Owner-sighash parallel work, the issuance re-typing and the completed verdict: the disposition is ESTABLISHED and it is established by recomputation, never by declaration — the assessment reads the profile's required set on one side and the reviewed contract on the other, there is still no argument, field or constructor anywhere that lets a caller state the answer, and what moved was the two inputs rather than the function. The post-verdict ruling re-types the issuance dimension from required to refused, so it joins the single-output and input-extension dimensions as a refusal with content and not as a requirement that quietly stopped being mentioned: the profile's refusal vocabulary grows a third member naming what the other two layers do, since the census refuses an issuance-bearing signing request and the decoder refuses issuance-bearing bytes, which made the required typing the one layer promising evidence the other two refuse to admit. The required set becomes six and the capability's unestablished table records the dimension under the ground that the selected profile refuses it, keeping the review's own citation at the dimension table rather than moving it to the refusal table, because the two issuance terms are written for every message this arc forms and are not a branch the profile declines to take. The re-typing is NOT free of consequence the way the internal key's was, and the difference is landed rather than absorbed: the coverage map did assign the issuance fields to the dimension, so that datum's carrier moves to the all-inputs dimension, which is the ruling's own sentence and not a repair of it — both terms are one zero byte per input for every candidate this arc builds, so what they commit to is the input count and the all-inputs dimension already carries it, while what makes the fields absent at all is not a message term but the two refusals. No protected datum leaves the census, every carrier is still a required dimension, and the invariant that no coverage lands on a refused dimension is what says so rather than the paragraph. The Wave-4 stop test is replaced rather than flipped, and its successor asserts the repair from both sides with the count first, because an established disposition is reachable two ways and only one of them is the verdict: a required set emptied until nothing was left unreviewed would satisfy the disposition assertion and fail the literal six beside it. The re-typed dimension is asserted refused by the profile AND still unreviewed by the contract, in the profile tests and again across packages, because the ruling refused the dimension and did not establish it, and a tree that had cleared the stop by marking it reviewed would be recording evidence no run produced. The profile residual clears in all five vocabularies on that recomputation: it leaves the tapscript recognition residual sets at both pattern sites and the private soundness condition, the linker's outstanding link obligations, which fall from five to four, the transaction ABI's obligations, which fall from four to three, the vectors carried residual set, which falls from three to two, and the conformance package's per-case residual, which leaves every owner-authorization case. Each vocabulary keeps its word and gains the record of what moved it and what did not move with it, on the pattern the computability blocker's own clearing set, and each clearing is asserted absent at its site rather than dropped from a test, because a test that merely stopped mentioning a residual cannot tell a clearing from an omission. The clearing-order rule is honoured exactly: the computability blocker cleared on the Wave-3 run and this cleared afterwards on a review verdict, never simultaneously and never on the same evidence, which is the discipline the two were separated to keep and which is now recorded in both directions in the source. Nothing else moved: no row became answered, no matrix row, minimality pair, resource dimension or report standing moved, the proof-bearing lane stays unestablished, and the accepted result is exactly the value it was — well formed and exact-byte bound before and now, with the establishment gate still the consuming guide's handoff to read. Routed findings from the exclusion-labeling audit are closed: both issuance-proof raise sites in the input-witness decoder are reached, not one, since a test filling only the amount rangeproof would leave the inflation-keys rangeproof exactly as unexercised as both were; the peg-in refusal's witness-field site is reached beside its already-covered outpoint-marker site, which is why the gap was invisible, one variant raised from two places looking exercised from either; and a splice control is asserted to reproduce the original bytes and decode unmodified first, so each refusal is attributable to the field under test rather than to a corrupted splice. Both refusal vocabularies gain the exhaustiveness guard neither had, one arm per vocabulary listing every variant with no catch-all, a hundred and six and thirty-two, so the next variant added without a test is a compile error rather than an invisible gap — and the guard states what it does not do, since a match arm is not a test and a guard claiming coverage it had not measured would be the false assurance the register warned a reviser against. The exclusion register's issuance-dimension section moves from owed rows to as-landed rows, naming the two typing rows by file, type and variant beside the three subject rows, with the revision surface rewritten over what exists; it also records that the unreviewed ground the re-typing left carried by no dimension is kept deliberately, so a revision returning the dimension to required restores its one occupant rather than reintroducing a vocabulary member. The documentation bar was red at the branch point and is green over all five packages, and it carried TWO pre-existing breaks rather than the one that was known: repairing the finalization module's public-to-private link let the bar reach a package it had never got to, where the computability blocker's own documentation linked a crate-private helper — a break that was masked rather than absent, which is the failure mode a bar that stops at the first error has. The profile residual turned out to have a FIFTH carrier that no charter named, the conformance package's own case residual attached to every owner-authorization case, and it was reported as a divergence rather than cleared unilaterally and then cleared under the orchestrator's adjudication that the concept's four-vocabulary wording is a census of what was known when it was written and not a cap on how many carriers exist, since a carrier still reading unreviewed against a recomputed established disposition is the exact drift the clearing discipline exists to prevent; it converts by its own site's rules on the same verdict, and it is the site where the claim mattered most, because the failure that census is shaped against is the one an unestablished profile leaves open, a narrower profile turning a mutation case into a transaction the target accepts whose evidence is then worth nothing. Clearing it discharged no case and could not have, and the census proved that rather than the prose asserting it: this vocabulary refuses a case with nothing outstanding, so removing the residual and leaving the set empty made validation fail and name the valid case as one claiming to be runnable. What was outstanding had not become nothing. The residual had been carrying two claims, the profile gap and the fact that no run had produced a verdict, and only the first was cleared, so the second is minted as its own member and carried by name on every case — a vocabulary addition this wave did not plan and the site's own rule required before the old carrier could go, which is reported as the one place the surgical scope had to widen. The claim it names is not new work: it was always true, always required by the census's rule, and previously spelled nowhere, so section 1.11's bar is now typed rather than implied and is cleared by an observed run and by nothing else, no further review reaching it. The test that used to carry that claim through the profile residual is split into one asserting the clearing at the site and one asserting the member that now enforces the non-claim; the mutation coupling did not move; every case is still present at the same expectation; and the sponsor envelope is still the only residual any case carries beyond the run. Reported and not fixed: the Wave-1 review document still says the assessment returns review-incomplete naming all eight required dimensions and that the residual stands in every vocabulary, which was already stale before this wave and sits in a section its own opening sentence time-scopes to that document's completion. Verified entirely on the server per the no-host-compute ruling, every step timed, with no native run because this wave observes nothing and the verdict recomputes from recorded evidence: both plans validators green with the register generator leaving the tracked tree unchanged, the label lane green with zero diagnostics and no near-miss warning introduced, the documentation bar green where it was red, and the six owning packages green |
| `T5-030` | DONE | ADR-019 adoption-parameter split, per-row labels for the seven fixings: the single parameters-table entry becomes seven labeled entries, one for each adoption parameter, so an amendment cites the parameter it amends instead of citing all seven and naming its two in prose beside the citation. The seven take the rule kind in the labels area, which is the kind the amendment list already gives an adoption datum in both adopting records — the area-syntax and heading-depth entries here, the acceptee and extension-entry entries in the kind registry's record — and their names are the parameters' own: owner-prefixes, owner-partition, profile-signature, reserved-kind-set, typed-data-classes, citation-indexes, scanned-regions, none of which collides with the three signature labels the adopted body already mints. The table label RETIRES rather than surviving over a residual index of the seven, on the record's own ground: an entry of that list is an amendment and states exactly what it changes, so an index of the entries below it would change nothing in the adopted text and would be an eighth entry amending nothing. Retirement is an authorship lapsing by recorded decision, which re-points the citations it breaks in the same commit, and the corpus held exactly one citation of the retired label — the script-tree record's header — which now cites the owner-partition and scanned-region entries directly; the checker confirms no citation of the retired label survives anywhere. Both changes are recorded as labeled amendment entries, ADR-019's parameter-entries entry carrying the old-to-new mapping and the retirement ground, and the script-tree record gaining an amendments section and its own parameter-citations entry, because a directly-written record had no such list and the discipline admits no unlisted change to a record's text. Parameter content is byte-preserved and was verified so mechanically: each entry's fixing equals its row's fixing cell word for word, with the single presentation adaptation of restoring the terminal full stop a table cell drops, reported here rather than left for a reader to find. Nothing else moved: no code changed, no adoption datum changed, and the checker's typed adoption data are untouched. The register census did not move and the generator's output is byte-identical to the committed register, which is the expected result rather than an omission — the companion register carries mint counts for the fourteen recorded extension kinds alone, and rule, rem, sec and tab are registry kinds outside that set. Two stale references are reported and NOT repaired, both outside this row's scope: the records directory README still calls the amended parameters rows, and the labels crate's adoption module still calls the ADR-019 parameters an adoption-parameter table in a module comment. Verified entirely on the server per the no-host-compute ruling, every step timed, in a lane clone of its own because the shared worker clone was checked out at the parallel row's branch: the documentation tree valid in 4.4s with only the pre-existing weight warnings, and the register generator clean in 15.7s with an empty diff. ADR-019 grew from 38720 to 40961 bytes, well inside the root-record weight class |
| `T5-031` | DONE | Owner-sighash parallel work, Wave 5, the proof-bearing handoff: one owner authorization over a candidate whose two output-witness entries carry 4174 range-proof bytes each was submitted to a real node and ACCEPTED in block six, then re-verified against an independently recomputed message the read-back signature satisfies and the emptied-vector message does not. The census is built through the proof-finalized route alone, at the vector's real length; the spent-output triple signed over is the node's commitment and never the ceremony's amount. Three submitted controls, all designed from what differs here and all refused at the script-path layer over identical bytes, and two construction controls refused before any message is formed. The protected-bytes repair is exercised from the signing side: a census over a candidate whose proof moved one byte refuses the bytes the owner holds, while the witnessless serializations are identical. The funding, materialization and blinding are the confidential-funding guide's, called and not checked, disclaimed in the record and the artifact; the leaf is a bare owner check and not a receipt covenant, because the private finalization branch is that guide's restart. The run of record is committed with the observation identity the reviewed contract already names the explicit run by, and omits the outpoints. No row converted: the per-case run residual needs a run of that census's own cases and its validator refuses an empty residual set, the carried residuals are unrelated, and the not-submitted and paired-materialization standings are scoped by their own words. Two sites bear on the run without converting and are named in the wave report. Verified on the server, every step timed |
| `T5-032` | DONE | Backlog headroom restored by archiving the closed Guide-13 batch: the 2.9 gate record and the nineteen Phase-5 wave narratives moved byte-verbatim with their labels into the Guide-13 records group of backlog history, proven identical by hash rather than by eye, and each live row compressed to a verbatim prefix of itself plus one pointer into that group, so no claim was re-worded. The current-gate statement is re-pointed at 2.9 and T5-020 onward is untouched. Combined markdown 753479 of the 786432 cap, headroom 123 to 32953 bytes |
| `T5-033` | DONE | Confidential-funding execution guide, Wave 4, the external sighash handoff closed: the proof-finalized candidate's bytes enter the signing request UNCHANGED and are checked so rather than trusted, at every state — the request's bytes, the census's, and every signer input's byte binding are the freeze's own string, re-read again at the fully-authorized and the submit-ready states, and none of them is the witnessless serialization. The four handoff states are type transitions in a new transaction module and each of the four named refusals is reached by a test: the profile gate reads a recomputation on every call rather than a stored answer, and is reached through a capability built for the purpose rather than by disturbing the reviewed one, which still assesses to established beside it in the same test; an answer bound to a candidate whose single range-proof byte moved is the wrong candidate, as is a surplus answer for an input the census does not have; an unanswered signing input is a missing owner, which is the only shape a wrong owner has at a boundary where authorizations carry no key; and all eight protected regions refuse mutation after signing started across four shaped operations, plus regeneration, repair and reblinding by name. Two owners over a two-input candidate share one byte binding and one preimage, and one sponsorless private candidate reaches submit-ready through the mandatory order end to end — materialize, finalize proofs, census, sign, bind — with no submission and no evidence conversion, both of which are the next wave's. NO DIGEST was designed, computed or consumed: the module says so in its first section, and the output-witness commitment is cited to the sighash-side tests that own it rather than re-tested here. The submit-ready candidate's identity, a label over its protected bytes and not a signing message, is pinned. The refusal vocabulary carries two members beyond the guide's four, wrapping the census and accepted-result vocabularies rather than flattening them, so a duplicate answer stays distinguishable from a missing owner; this is reported as a shape divergence rather than absorbed. No blocker, residual or matrix row moved and none could, because nothing here observes a target. No native run: the target-runs table's private-candidate row is a submission run and this wave submits nothing. Verified entirely on the server per the no-host-compute ruling, every step timed: formatter and clippy clean, 252 transaction tests green and the five owning packages green, and the documentation bar green across six packages in 8.6s |
| `T5-034` | DONE | Confidential-funding execution guide, Wave 5, restarting the Guide-13 evidence: the mandatory restart order ran in order against a real node and STOPPED TYPED at step three, which is a valid closeout and the shape the guide's own closing statement anticipates. Step one accepted — one sponsorless private one-to-one receipt-covenant control, spending a mined confidential predecessor at this deployment's own private receipt constructor, taken, with the witness read back out of the node's own copy verifying against an independently recomputed message; 9136 bytes submitted, two output-witness entries of 4174 range-proof bytes each, 11.5 seconds of wall time. Step two accepted — the second run consumed the predecessor's balancing output at commitment prefix 0x09 where the first carried 0x08 and was taken, so both admitted parities are exercised in complete accepted successors rather than inferred from one candidate touching both; the two runs are checked to BE two, by different successor digests and different identities. Step three stops because a conservation claim needs a refused non-conserving case beside the accepted conserving one and that case is step four's wrong-blinder mutation, which this wave did not run; steps four through seven are recorded as not reached rather than omitted, and the sponsor cases DID NOT RUN and may not, their signer dependency being one this guide does not close. TWO ROWS MOVED of the ten positive private classes, each on an observed acceptance of its own shape: private-one-to-one and both-commitment-parity-forms, and the eight that did not move are named so a reader counts ten either way. Moving them needed a standing that did not exist — the matrix could say a row NEEDED a run and could not say a run had ANSWERED one, and recording an acceptance would have meant filing a target's verdict under a first-party refusal — so NativeRunObserved is minted carrying the identity the target computed, and the four tests that existed to make a row move impossible to do quietly all failed and were updated to the new fact rather than loosened. The enabler was the private lane's own transaction-wide finalization, which shares the explicit lane's shape selection, receipt recognition and record assembly so each input still carries the leaf its position executes, and differs only in that the outputs and the protected transaction are the materializer's. A REAL BUG was found and fixed on the way: the live-transfer lane kept a funding step's outpoints and rebuilt asset, value and program from its own expectations, so a candidate's message was formed over a spent-output triple nobody had asked the chain about — invisible while every witness authorized nothing, live the moment signatures authorized. Five sites still named the cleared OwnerSighashNotComputable as a current stopper, including the minimality deficit, which was counting a cleared blocker towards its own shortfall; all five are re-pointed and the deficit drops from two named components to one. Four divergences are reported and NOT repaired: a strict one-to-one is not constructible under the guide's own fixture and materializer rules, so step one ran under a stated reading; §11.1's four carried residuals are three in the repository; the narrower minimality blocker is NOT MINTED because an unwired call site is not a missing component and the vocabulary admits only the latter; and the NoObservedTargetVerdict tension is untouched because this wave's runs do not reach that census. Exactly one residual cleared and the two carried ones unchanged, the pre-sighash delta zero, and all three closeout invariants checked by a function rather than asserted in prose. Full record in [backlog history](history/ctf-guide-wave-5-closeout.md). Verified entirely on the server per the no-host-compute ruling, every step timed: formatter and clippy clean, 292 vectors and 256 transaction tests green, six native tests green in 20.3s, the documentation bar green in 2.4s, and near-miss label warnings 8 to 0 |
| `T5-035` | DONE | Phase-5 handoff, sponsor envelope signing, the first honest wiring step taken and the residual REPORTED STANDING: one integration test now finalizes an explicit sponsored control over really funded receipts and a really funded sponsor coin, sends its exact sponsor request to the adapter, replays the returned witness through the same sponsor capability into the same finalization, and verifies byte binding — and it ran against a real node. The four sub-steps the card names are four separate observables rather than one claim implying three. The control is sponsored and explicit, and its sponsor input is the suffix member at position two following both receipt inputs, checked against the receipt count rather than assumed. The request carried 334 bytes and the adapter echoed 334 bytes, compared byte for byte and not by length, because two serializations of one shape have the same length and are not the same bytes. The returned witness is two items of 72 and 33 bytes, a DER signature with its sighash byte and a compressed public key, and replaying it completed a control of 1480 bytes where the placeholder pass completed 1375, with every returned item present in the replayed bytes as a contiguous run. The BINDING IS EARNED AND NOT ANNOUNCED: the same witness bound to bytes with one byte moved is refused with SponsorSignatureBindingMismatch naming the sponsor outpoint, so a lane that stopped checking would fail rather than pass quietly. The residual SponsorEnvelopeSignerAbsent STAYS, judged against its own clearing rule at its own site and never against the capability existing — the rule asks for the sponsor owner's target authorization and says a returned byte stack is not that until a target has ACCEPTED a control carrying it, and this step submits nothing, so no such acceptance exists. A second test outside the node gate asserts the residual is still in the carried set, so a later change has to notice it, and the run record says in its own bytes that it submitted nothing and cleared nothing. Three stale claims are re-pointed rather than left standing: the handoff card's clause blaming the owner sighash, which is false now that the profile is established over its required set and both lanes carry observed acceptances; the residual site's identical sighash clause and its neighbouring sentence that nothing reaches the capability; and the conformance README's claim that the capability has no first-party caller. Every re-pointing leaves the clearing rule exactly as strong as it was — a target must still have accepted a control carrying the witness — and only removes what had become untrue about why. One divergence is reported and NOT repaired: the control is not submittable, because the deployment's reserve asset is a fixture constant no chain has issued while the sponsor coin the adapter funds is one of its own, so the fee output names an asset its sponsor input does not carry; that costs the round trip nothing, the signature and its binding being over the bytes either way, and it is the next thing a submission step has to solve. A second, smaller one: the shared worker lane's native flag is hard-wired to the live-transfer test, so this test ran through a one-off lane script beside it rather than through the flag. Verified entirely on the server per the no-host-compute ruling, every step timed: the worker lane green in 109.8 seconds with formatter and clippy clean and both owning packages' tests passing, and the native run green in 14.0 seconds with the round trip itself taking 6.8 |
| `T5-036` | DONE | Internal-key unspendability probe, phase A: one key-path spend attempt was offered to a real node and REFUSED, and the refusal discharges only that the attempt was observed and refused. The target said mandatory-script-verify-flag-failed (Invalid Schnorr signature), recorded verbatim and unmapped, at a layer it typed as a target verdict, with no accepted identity; 281 bytes submitted, one witness item of 64, 4.1 seconds of wall time. The candidate spends one funded explicit-constructor receipt through the EXISTING generic submission wire UNCHANGED, which carries transaction bytes and nothing else, so the attempt turned out expressible and no wire finding arose — the refusal vocabulary still carries the member where an inexpressible attempt would have landed, because a probe that could only report success would not have been reporting. The witness is the one-item shape a key path has: a signature, no leaf script, no control block, and the ceremony funds ONE receipt rather than the observation ceremony's two so that the phrase describes one witness instead of two. The program binding is recorded whole rather than as a program alone, because the probe's subject is the relation between its parts: the funded program is the witness-version-one script for an output key that is the published unspendable point tweaked by the constructor's merkle root, all four values written down, and the internal key is RECOMPUTED against the published constant rather than assumed, so a deployment that had inherited some other point would make the record say so. NOTHING IS DISCHARGED and the record says so in its own bytes rather than in a comment: six non-claims are carried as data, the offered signature is by the first published owner's key and NOT by the output key — both keys written down side by side so a reader sees they differ — and a target refusing a signature that does not verify is doing what any target would do for any key anyone does not hold. The residual discrete-log assumption on the internal key stands exactly where it stood. NO OBSERVATION IDENTITY is minted and the module states the ground: the target accepted nothing, and filing a non-acceptance in the vocabulary acceptances are cited from would be the one error a run of record exists to prevent, so the candidate's own reproducible bytes anchor it instead. The key-path message construction lands beside the reviewed script-path one in the transaction crate, sharing the twelve whole-transaction terms through an extracted prefix so the two differ in their tail and in nothing else, and it is candidate-scoped in the strict sense — no review here covers the key path and no target has been asked whether it forms this message. Two of its four tests carry that claim rather than restating it: moving a tapscript term moves the reviewed message and leaves the key-path one exactly where it was, and the output-witness treatment still moves the key-path one, so the tail is absent rather than the prefix stubbed. The isolating term is the codeseparator position rather than the leaf hash, because the census refuses a leaf hash that does not commit and every admissible leaf change moves the spent program the shared prefix already commits to; the first attempt used the leaf hash, failed on that refusal, and was replaced rather than loosened. THREE DIVERGENCES are reported and NOT repaired. The observed-layer vocabulary has no key-path member, so this key-path refusal is filed under a script-path name, the adapter classifying on the refusal text's prefix because that is all the vocabulary it has — which is the typed carrier phase B owns and is named in the run of record rather than worked around. The native lane runs its tests in parallel and a seventh concurrent node exceeds what the shared instance sustains: the unmodified base is 6 of 6 green in parallel in 20.0s, while this tip in parallel is 5 of 7 on one run and 6 of 7 on another, with a DIFFERENT casualty each time — the confidential-predecessor funding and the commitment-parity successor once, the explicit-lane owner observation the next, that one failing at ExecutorHandshakeFailed, which is an adapter that could not start a node rather than a ceremony that decided anything. A varying victim and a startup failure are contention and not an interaction, and none of the three is touched by this row. Serialized, the same tip is 7 of 7 green in 77.9s, so the verdict of record is taken there and the shared lane needs the same serialization or a lane of its own for the seventh test. The probe passed in both modes, its bytes and its refusal identical across them. And the probe ran in a lane clone of its own, the shared worker clone being checked out at a parallel row's branch. NO matrix row, blocker or residual moved and none could: this observes a target refusing, and a refusal answers no row. Verified entirely on the server per the no-host-compute ruling, every step timed: formatter and clippy clean, 260 transaction and 298 vectors tests green, 7 of 7 native green in 77.9s with the probe among them, the documentation bar green in 2.7s, and both plans validators green with the tracked tree unchanged |
| `T5-037` | DONE | Confidential-funding follow-up evidence wave, restarting the order under the orchestrator's interlock ruling that Wave 5's step-three stop was unwarranted, ran against a real node and ADVANCED the frontier from step three to step five. Steps one and two RE-RAN on fresh disposable chains and reproduced Wave 5's identities byte for byte — the one-to-one control at prefix 0x08 and the parity at prefix 0x09 — a determinism observation, not a matrix move. Step three RECORDED target CT conservation as ACCEPTED against the balance-valid control, its conserving half the accepted control and its non-conserving half one wrong-blinder mutant's balance-layer refusal; step four ran the THREE proof-negatives from that control, all submitted to the mempool boundary and all refused at ConsensusRejectionBeforeScript with the one identical string bad-txns-in-ne-out, each attributed by its MUTATED FIELD — the wrong blinder to the output's 33-byte value commitment at range 81..114, the missing and malformed range proofs to the output-witness range-proof bytes at 781..4958. THE MANDATORY DISCLOSURE is carried in both ledger entries in words: step three's non-conserving half is the SAME observed run step four records as its wrong-blinder case, one observed run and not two, so two Accepted entries cannot be double-counted. THREE ROWS MOVED of the ten positive private classes, each on an observed acceptance of its own shape: one-to-one and both-commitment-parity-forms re-affirmed, and target-ct-conservation — the row Wave 5 stopped before — moved for the first time; the seven that did not move are named so a reader counts ten either way. The order STOPS TYPED at step five, its remaining positive shapes needing multi-output and multi-input fixtures this wave does not build, typed MultiOutputShapeConstructorAbsent, a constructor absence like the predecessor one and not a target verdict; private-merge is additionally structurally unconstructible under the registry's two-output rule, reported as a divergence against §14.5's merge predicate. Steps six and seven are not reached — the sponsor step does not run and this guide does not close its signer dependency, and minimality follows it in the order. The new machine-checked closeout wave_six_closeout NAMES Wave 5's step-three mis-typing as corrected: it cited NoAcceptingControlExists against a ledger whose first entry was an accepted control, contradicting a_positive_control_exists, carried_residuals, and has_accepted_control. Exactly one residual cleared (NoConfidentialPredecessorCanBeFunded), the two carried ones unchanged, the pre-sighash delta zero, all invariants checked by the function rather than asserted. Repo repairs both landed: the CtConservation role doc records its "refused a non-conserving one" clause as a repo-authored strengthening of §10.5, and the closeout names the mis-typing. One guide-side erratum filed against §10.5 step four's undischargeable "each attributed to its own layer" — the balance check precedes the range-proof loop and VerifyAmounts discards the specific error, so the target emits one identical string for all three mutations. The construction spine is EXTRACTED and shared, so the proof-negative ceremony reproduces the exact balance-valid control rather than copying its construction. Full record in [backlog history](history/ctf-guide-wave-6-closeout.md). Verified entirely on the server per the no-host-compute ruling, every step timed: formatter and clippy clean, 302 vectors lib tests green, four native runs green serialized against a real node — steps one and two at 12.3 and 12.4 seconds, the conservation-and-proof-negatives run at 12.7 — both plans validators green, banned-token zero. |
| `T5-038` | DONE | Shared abstract-program analysis, first slice: a public program_stack_profile now sits beside validate_program in tapscript, walking every prefix of a program and returning a ProgramStackProfile carrying the deepest main and alternate stacks any reached state holds — read from the abstract executor rather than hand-counted, seeded from the initial state, and skipping any prefix the executor refuses. It is exposed once, publicly, where two packages had each measured it privately: the conformance package's peak_main_stack over prototype_program.rs and the vectors package's walk_program over live_measurements.rs, both of which validate each prefix and read the deepest stack the same way. THREE side-by-side tests DEMONSTRATE the equivalence and consume none of it. A tapscript public-API boundary test reads the profile of a two-push addition as a main peak of three and an alternate peak of zero, proving the surface is genuinely external. A conformance descendant module compares the public main peak against the private peak_main_stack over the continuity prototype, anchored to that prototype's canonical nine, and over the wide-floor prototype at five or more. A vectors descendant module compares BOTH public peaks against walk_program over every linked program the demonstration bundle carries, and they agree on each. NEITHER WRAPPER WAS DELETED, no canonical output moved, no consumer was switched to the shared API, and nothing was optimized: retirement of the wrappers and any optimization are explicitly later slices gated on this evidence. The public type carries rustdoc under the doc bar with backticked identifiers and short first paragraphs, adds no dependency, and introduces no new file, so the ADR-014 census is unchanged. NO native run: this slice observes no target, adds only first-party analysis and its equivalence tests, and moves no blocker, residual or matrix row. Verified entirely on the server per the no-host-compute ruling, every step timed: the worker lane green in 305.0s over the three owning packages with formatter, clippy and the doc bar clean; the three equivalence tests green by name — the tapscript boundary test in a 4.6s run, the two conformance comparisons in 0.67s, the vectors comparison in 13.06s; banned-token count zero across the whole diff |
| `T5-039` | DONE | Confidential-funding execution guide, Wave 7, the step-five stop CLEARED: the remaining positive private shapes are built as multi-output and multi-input fixtures and all three ran against the pinned node and were ACCEPTED, each moving its own §11.2 row on an acceptance of its own shape — a split of one receipt into three outputs with 13499 bytes and three range proofs of 4174 each, a representative many-to-many of two receipts into three outputs with 13882 bytes, and a two-receipt transfer under two DISTINCT owners into two outputs with 9519 bytes; every one carries a readback witness verified from the node's own copy against an independently recomputed message, and the three successor digests differ so the record says these are three runs rather than one reported three times. THE CONSTRUCTOR ABSENCE WAS DEEPER THAN THE MANIFEST BUILDER and was found by running rather than by reading: the fixture registry's bounded parity search compared the target's admitted commitment-prefix pair against a manifest's openings BY LENGTH, so any manifest but a two-output one matched no counter and spun through all 4096 attempts before refusing — the first lane against a three-output fixture ran for minutes and had to be stopped. Nothing in the target contract says a confidential transaction has two outputs; the cardinality was the conformance file's own assumption. The repair keeps the two-output case bit-for-bit — the admitted pair in FIXED ORDER, which is what makes the dual-parity predecessor carry one of each parity rather than one twice, held by its own test — and holds wider fixtures to the only rule the contract states about them, with the code saying plainly that the search then degenerates to a well-formedness check the first counter satisfies, so no discriminating power is claimed where none exists. SIX ROWS ARE NOW ANSWERED of the ten positive private classes, three inherited and three this wave's, and the four that did not move are named with their grounds. A DEFECT WAS REPAIRED on the way: the follow-up wave's closeout said target-ct-conservation had moved while the matrix still classified it as needing a run, so two artifacts disagreed about a row; the matrix now cites that wave's own run of record. The one-output shapes are typed rather than forced — private-merge, the strict one-to-one, and the fee-only case all hit the registry's two-output floor with OutputSetTooSmall, and the fee case is additionally inexpressible because the fixture vocabulary has no fee role and refuses an empty output program, so a reader who saw only the cardinality refusal does not conclude a relaxed floor would admit it. The merge-predicate conflict against the registry floor is filed as an erratum in the Guide-13 feature-request register, scoped to that conflict only, with neither rule relaxed. Deterministic-public-fixture-openings does NOT move: the determinism observation the row asks for is produced — the same manifest recomputes to the same digest and the same per-output blinders byte for byte — but it carries no target-computed identity, and filing a first-party determinism fact in a vocabulary whose members are acceptances would be the one error a run of record exists to prevent; whether that row's gate should be an acceptance at all is reported for the guide's owner. STEP SEVEN IS NOT REACHED and the ordering tension is REPORTED rather than ruled: the order's preamble makes each step's entry the previous step's observed result, step six has no observed result and is conditional on a dependency this guide does not close, so a strict chain reading makes step seven permanently unreachable rather than merely deferred — a stronger consequence than the guide's text states it intends — while step seven's own clause names only its pairs' acceptance; the strict reading was taken, no pairs ran and no minimality row moved, and a test holds the unreachability structural by asserting the ledger refuses the pairs after the stop with AfterStop. The new machine-checked wave_seven_closeout records step five ACCEPTED with its own disclosure in its own words — three shapes of four, the fourth unconstructible rather than unrun — and stops typed at step six on SponsorEnvelopeSignerAbsent; MultiOutputShapeConstructorAbsent is NOT added to the cleared set, because it was never a carried residual and a constructor absence leaves by the step it stopped being recorded accepted. Exactly one residual cleared and the two carried ones unchanged, the pre-sighash delta zero. Full record in [backlog history](history/ctf-guide-wave-7-closeout.md). Verified entirely on the server per the no-host-compute ruling, every step timed: the worker lane green in 308.8s with formatter and clippy clean and 308 vectors, 260 transaction and the conformance package's tests passing; the ignored native lane 11 of 11 green in 133.8s taken SERIALIZED through a one-off lane script with --test-threads=1, the shared script unedited, the three shapes at 12.9, 13.9 and 15.6 seconds; both plans validators green and banned-token zero across the whole diff |
| `T5-040` | DONE | Consensus shape-possibility register: the blinded shapes of the confidential live lane are censused with TWO verdicts per row, computed and cited separately, because one word had been carrying two facts — a shape the target's balance rule cannot admit and a shape this workspace's own registry declines to express were both called unconstructible, and the second is a convention this repository chose while the first is arithmetic nobody votes on. EIGHT SHAPES over a stated closure rule, and the rule is what makes the enumeration closed rather than merely long: the consensus verdict depends on NOTHING but whether at least one output is blinded, not on the input count and not on the output count beyond some-blinded against none, so every shape outside the small-shape window inherits the verdict of the case it falls into and adding it would restate a row. FOUR are OBSERVED-ACCEPTED, each citing a run-of-record identity rather than a transcribed literal — one-to-two, one-to-three, two-to-two, two-to-three. THREE are SOURCE-DERIVED-POSSIBLE and refused by first-party convention: the strict one-to-one and the two-to-one merge at the two-output floor with OutputSetTooSmall, and one-to-one-with-fee at the absent fee role with OutputProgramEmpty. ONE is SOURCE-DERIVED-IMPOSSIBLE: fee-only, because a fee output is mandatorily explicit at src/primitives/transaction.h:324-327 and so contributes a zero blinder, leaving no output to absorb a nonzero input blinder sum against the tally at src/confidential_validation.cpp:73-81; the two escapes are recorded AS escapes rather than solutions, since a zero-blinder input is not a blinded input and an added blinded dummy output is the fee-bearing shape under another name. THE CONSENSUS HALF IS A COMPUTED PREDICATE, not a table of remembered answers — an explicit value joins the same tally at a zero blinder at :345-349, so possibility reduces to whether some output is blinded — and a test holds every recorded verdict against it, so the register cannot state a verdict its own arithmetic denies. THE REFUSALS ARE RECOMPUTED by driving the live registry, which surfaced the register's sharpest observation: the two-output floor returns the SAME refusal for the impossible fee-only shape and the perfectly possible merge, so it guards consensus BY ACCIDENT, and a reader taking the refusal as a verdict would be wrong about one of the two — held by a test that drives both and compares. Both limitations carry their convention and a NAMED REMOVAL PATH, and the single-output form is filed WITH its degeneracy rather than plainly: a merge's forced blinder is the SUM of the consumed blinders, which for the inverse-pair dual-parity predecessor cancels to ZERO, leaving a commitment of exactly the value times the value generator that anyone recomputes from a guessed amount — the tally still balances and only confidentiality fails, silently — so the removal must require a non-canceling predecessor or refuse a solved zero blinder outright. THE PRIVATE-MERGE ERRATUM'S GROUND IS CORRECTED: it said a single-output confidential transaction has no free blinder to balance, and it needs none, the lone output's blinder being FORCED to the input blinder sum and a forced value being solved rather than impossible; the conclusion and both existing options are kept verbatim and a third is added, the only one of the three admitting the merge as the guide's own predicate states it. Both removal paths are FILED and neither implemented, as T5-041 and T5-042 in this register rather than as guide feature requests, the work being this repository's own conformance package. THE PLANS WEIGHT CAP WAS RAISED rather than carved around: the 768 KiB hard cap and 520 KiB soft target were inherited verbatim from the retired Python checker with no derivation stated, and the tree had grown into them far enough that one routine gate paragraph could breach the cap, so they go to 1 MiB and to 709632 bytes, the exact proportional 709973 rounded down to a whole 693 KiB; two budget-exclusion tests that wrote a 900 KiB fixture and meant "bigger than the cap" were found by the raise and now derive the size from the constant. NO NATIVE RUN and nothing observed: every consensus-possible-but-unrun shape is recorded source-derived and never "run", and NO matrix row, blocker or residual moved or could — a register is not evidence. Verified entirely on the server per the no-host-compute ruling, every step timed: the worker lane green in 106.1s with formatter clean in 1.8s, clippy clean in 4.6s, and 319 vectors, 196 labels and all 9 shape-census tests passing; labels-check OK in 0.6s with zero diagnostics; both plans validators green with the attestation register byte-stable; banned-token zero across the whole diff |
| `T5-041` | DONE | Removal path filed by the shape census: extend the confidential fixture registry with a SINGLE-OUTPUT FULLY-SOLVED BALANCING FORM, so the strict one-to-one and the private merge stop being refused by a construction-model convention the shape register now types as first-party rather than consensus. The manifest's one output is balancing and carries no freely chosen blinder, taking the input blinder sum directly; the bounded parity search then has nothing to search and degenerates to a well-formedness check, which the code must say plainly rather than claim a discriminating power it would not have; the two-output case stays bit-for-bit as it is. A CONDITION IS PART OF THE FILING and not a caveat on it: the forced blinder can be ZERO and a zero blinder hides nothing — a merge's single output takes the SUM of the consumed coins' blinders, which for this workspace's inverse-pair dual-parity predecessor cancels by construction, leaving an output commitment of exactly the value times the value generator, a point anyone recomputes from a guessed amount, carrying a blinded output's form and none of its hiding; the tally still balances and only confidentiality fails, silently. The work must therefore either require a non-canceling predecessor or refuse a solved zero blinder outright, and a version admitting the degenerate case is not this row done. Ground, pins and revision surface in [the shape-possibility register](reference/shape-possibility.md); the guide-side half is the refined private-merge erratum's third option in the Guide-13 feature-request register. Filed here rather than as a guide feature request because the work is this repository's own conformance package, not a request to the guide's author. DONE, and every clause of the opening statement was kept. THE FORM IS DECLARED RATHER THAN INFERRED: a new SoleBalancing output role says that a manifest's one output takes the input blinder sum directly, and the cardinality clause asks whether a short manifest declares it rather than how many outputs it has, so a lone output stating Balancing draws OutputSetTooSmall with the same count it always drew. Nothing that was refused registers now unless it names the form it wants, which makes this a NARROWING and not a relaxation. THE DECLARATION RIDES IN THE ROLE rather than in a new manifest member, and that choice is what kept every existing case byte-identical: the digest transcript already emits a role code per output unconditionally, so adding variants perturbs nothing a previous manifest hashed, while a manifest-level field would have shifted every registered digest. The identity is not asserted but RECOMPUTED — a new test registers the one-to-two run of record's own fixtures and holds their digests against the recorded predecessor and both recorded successors, values a ceremony wrote down before this vocabulary existed, so a perturbed derivation lands there before it lands on a chain. THE DEGENERACY WAS ANSWERED BY BOTH ADMITTED PATHS AT ONCE rather than by choosing between them: the registry's existing DegenerateBalancingScalar refusal is left standing and becomes load-bearing, because for a sole output the solve returns the input blinder sum unchanged and so the refusal fires exactly when the consumed coins' blinders cancel; and the shape that ran was built from a one-input transfer whose blinder sum is a single coin's blinder with nothing to cancel against. The parity search's degeneration is stated where a reader meets it rather than claimed away — for a sole output the blinder is not chosen at all, so the counter cannot move the commitment and the search spans one candidate. THE ROW CLOSES ON AN ACCEPTANCE, as it said it would: a target accepted the STRICT ONE-TO-ONE, 4773 bytes with one range proof of 4174 and a readback witness verified against an independently recomputed message, in 11.2 seconds — the smallest submission this lane has made, for the structural reason that one output means one range proof. It moves NO matrix row and the code says so plainly: the guide's §15.2 positive private table has no member for the strict one-to-one, so row_name returns an Option and None is a real answer rather than a missing entry; what moves is the census entry. THE CENSUS ROW KEEPS ITS HISTORY through a new ConstructibleAfterRemoval standing that cites the removed limitation and carries what ended it, because a row saying only that a shape works loses the fact that a wall stood there, and a wall nobody remembers is one that gets rebuilt; a test holds all four stages of the ruling's arc and holds that a limitation still standing claims no removal. THE MERGE DID NOT COME FREE, and that is this row's most important disclosure: the same removal freed one of the two shapes it was filed for. The floor no longer refuses the merge and the registry admits it; what refuses it is the ZERO blinder its only available inputs would force, because this ceremony funds one predecessor whose two output blinders are ordered additive inverses. That is a limitation of the ceremony's predecessor rather than of the registry's rules, it was UNCOVERED by the removal rather than created by it, and it is censused as its own kind with its own removal path — a chained precursor whose outputs do not cancel. THE REGISTER'S SHARPEST OBSERVATION IS RETIRED BY BEING ANSWERED: the floor used to return the same refusal for the impossible fee-only shape and the possible merge, guarding consensus by accident, and the two now draw different refusals each about its own shape, held by a test that drives both. The fee-only row is UNTOUCHED, still OutputSetTooSmall on a cardinality wall, because a lone fee output can never declare the solved form. Verified entirely on the server per the no-host-compute ruling, every step timed |
| `T5-042` | DONE | Removal path filed by the shape census: add a FEE OUTPUT ROLE to the confidential fixture vocabulary, so the one-to-one-with-fee shape stops being inexpressible. The role is explicit-valued, held OUT of the blinder solve at a zero blinder, and REQUIRED to carry an empty output program rather than merely permitted one, so the role is checked and not just excused from the nonempty-program clause; the clause then reads on the role instead of on every output alike. A second face of the same absence is in scope: the shared multi-output manifest builder assigns the balancing role BY POSITION to the last output, so a fee output — the one output that must never balance — arrives cast as the output that does, and the balancing-role uniqueness clause today counts a positional assignment. No consensus rule is involved anywhere in this row: a fee output is mandatorily explicit at the target, and this workspace simply has no vocabulary member for it, which is an absence nobody ever decided. Ground, pins and revision surface in [the shape-possibility register](reference/shape-possibility.md). Filed here rather than as a guide feature request for the same reason as the row above. DONE at the registry, and the row's own closing condition was NOT met — which is recorded as the finding it is rather than smoothed over. THE ROLE IS CHECKED AND NOT EXCUSED: a Fee member joins the fixture output role vocabulary, explicit-valued and held out of the blinder solve at a zero blinder, and it is REQUIRED to carry an empty output program rather than permitted one, because an empty scriptPubKey is the fee's whole identity at the target — an output with a program is not a fee but a payment, and a role that merely tolerated an empty program would let a manifest declare a fee the target would read as something else. The empty-program clause moved from every output alike onto the role, so a non-fee output with no program is refused exactly as before; that is the difference between a rule and an exception. THE POSITIONAL ASSIGNMENT IS RETIRED in both the manifest builder and the openings layer, the second face of the same absence: a fee output is the one output that must never balance, and arriving last it arrived cast as the output that does. Every call site now states the roles it used to be handed implicitly, which is why no fixture's digest moved. TWO CORRECTIONS CAME WITH IT and both are the same correction said twice — a fee output's opening is ABSENT rather than zero-filled, because a record of zeroes reads like an opening and an explicit output has none, and the admitted-prefix rule reads on the outputs that HAVE commitments rather than on the output count, so a two-output fixture of one blinded output beside a fee is not the dual-parity case whatever its output count says. NO EXISTING DIGEST MOVES: whether an opening block follows in the transcript is decided by the role code already emitted above it, so no presence flag was added — a flag would have shifted the bytes of every manifest registered before this role existed. THE SHAPE DID NOT RUN, and the row does not claim it did. The registry expresses a fee-bearing manifest, which registers, derives and digests; nothing between the registry and a chain has learned the role. The materializer's own output-role vocabulary has no fee member, its per-output stage would compute a commitment and a range proof for an output that must carry an explicit value and no witness at all, and the executor adapter's fixture catalogue and parity search read every output as a committed one. The projection therefore refuses FeeRoleNotProjectable BY NAME rather than mapping a fee onto the balancing role, which would have compiled and produced a candidate whose fee output was blinded — not a fee at the target, and a silently wrong transaction instead of an honest stop. THE CENSUS RECORDS THAT AS ITS OWN STANDING, ExpressibleAndUnrun, with a removal carrying NO identity, because a vocabulary that can express a shape is not a chain that has accepted one and collapsing those two is the exact failure the register exists to prevent; a test holds that such a row's evidence class stays a derivation and that it names the layer it stops at. The fee-only shape is UNTOUCHED and still consensus-impossible: the role's arrival gives it words without giving it a tally, since a lone fee output can never declare the solved form and no blinded output remains to absorb the input blinder sum. Verified entirely on the server per the no-host-compute ruling, every step timed |
| `T5-043` | DONE | Sponsor wave, runtime-threaded reserve asset and the first sponsor-signed submission, which TYPED-STOPPED at a third obstacle and did NOT clear the residual. THE RESERVE IS NOW LEARNED: it is a parameter beside the protocol asset through live_deployment_for_asset, link_live_bundle_for_asset and live_abi_for_asset, the demonstration keeps its literal at a new RESERVE_ASSET constant so every ceremony that funds no sponsor links exactly the deployment it linked before, and the sponsor-signing lane settles it from the sponsor-funding answer with the compact-ASH lane's own single-identity check — one reserve across every coin or SponsorAssetsDisagree. THE STEP ORDER MOVED and the move is the repair: the sponsor region is funded BEFORE the receipts, because the reserve is pushed as a literal by the isolation fragments and therefore committed in the taptree, so receipts funded first would be paid to the destination programs of a deployment the lane is about to stop using. THE GUARD WAS ALREADY MINTED AND NEVER RAISED: LiveSponsorInputCarriesForeignAsset existed with its own documentation and nothing had ever returned it, so the live lane read its sponsor inputs from an offer and never from the view; it now refuses a sponsor input carrying anything but the reserve, with the absent-view case kept apart as MissingPublicSponsorView because this lane already keeps that pair apart for receipts and the compact-ASH conflation answers a claim about a coin when it is a statement about the caller. EVERY LIVE FIXTURE THAT SPONSORED turns out to have named an outpoint it never showed — six construction paths across the fixtures, the pairs, the measurements and the fault discharge — and all of them now state the sponsor coin they spend. TWO OBSTACLES WERE FOUND BY RUNNING AND NOT BY READING. First, the owner signing census had no route to the sponsored form at all: from_explicit_finalized derived its spent outputs from the receipts alone, so a three-input sponsored control was refused SpentOutputCardinalityMismatch inputs 3 spent_outputs 2, and an owner who cannot be handed a message cannot authorize a spend; construction is the last place holding a view of the sponsor region, so the finalization now keeps the sponsor spent outputs it already checked and the census chains them after the receipts in input order. Second, and the stop: with the owners really signing, the target's verdict moved off the signature entirely — the same submission previously refused mandatory-script-verify-flag-failed (Invalid Schnorr signature) and now refuses mandatory-script-verify-flag-failed (Script failed an OP_EQUALVERIFY operation), which is the sponsored leaf's own fee-role check. ATTRIBUTED RATHER THAN GUESSED: the isolation fragment ends by inspecting the fee output's scriptPubKey and requiring its digest to equal the deployment's fee_program_digest symbol, construction writes the fee role with the EMPTY program because the role's identity is target-structural, and the symbol is the fixture constant no program hashes to — the same defect the reserve asset had, at the same site, unrepaired. The finding is held WITHOUT a node by a test of the deployment itself, so it is a property of the deployment rather than a verdict somebody must re-run a chain to see. THE RESIDUAL SponsorEnvelopeSignerAbsent STAYS, judged against its own clearing rule: the rule asks for the sponsor owner's target authorization and says a returned byte stack is not that until a target has ACCEPTED a control carrying it, and no acceptance exists. NO residual entered or left the carried set, so no historical closeout's machine check moved. NO matrix row moved and none could — the private-sponsor-values row asks for confidential sponsor values and this control is explicit. THE RELAY BOUNDARY REMAINS UNEXERCISED for a sponsored control: the submission path asks testmempoolaccept first, and this control never got past script evaluation, so nothing was learned about standardness or the fee. The step-seven ordering erratum is NOT filed, because the dependency it is conditional on is not closed. The removal path is filed as the row below. One prediction of the wave's own brief is falsified and recorded as such: the fee program digest was expected to stay out of scope, and a sponsored control cannot avoid it. Verified entirely on the server per the no-host-compute ruling, every step timed |
| `T5-044` | DONE | Fee-digest wave, and the FIRST OBSERVED SPONSOR-SIGNED ACCEPTANCE. The fee-role program digest is threaded beside both assets through live_deployment_for_asset, link_live_bundle_for_asset and live_abi_for_asset, on the ruling recorded at the sponsor wave's merge and by the twice-established precedent of the protocol asset and the reserve: thread, not pin. The demonstration keeps the fixture constant at a new FEE_PROGRAM_DIGEST beside RESERVE_ASSET, six demonstration callers pass it, and the sponsored ceremony passes the digest of the fee program it ACTUALLY constructs. THE DIGEST IS COMPUTED, NEVER GUESSED, and the rule was read out of the emitted program rather than assumed: the isolation fragment emits NO HASH OPCODE AT ALL. It pushes the fee output's position, runs OP_INSPECTOUTPUTSCRIPTPUBKEY, and compares what the TARGET pushed — against a version marker of -1 and then against the symbol, two OP_EQUALVERIFYs in packages/tapscript/src/live_plan.rs at the sponsored branch of live_sponsor_isolation_fragment. What the target pushes for a program that is not a witness program is modelled in packages/target-elements/src/opcode.rs as the encoding class ScriptPubKeySha256 under a negative version marker: single SHA-256 over the whole scriptPubKey, exactly 32 bytes. The fee role's whole identity is its empty program, so the value demanded is SHA-256 of nothing. NOTHING NEW WAS WRITTEN TO COMPUTE IT: the ceremony reuses the existing bundle::fee_program_digest helper, which is sha256 of the empty slice. THIS DEFECT HAD BEEN MET AND DIAGNOSED BEFORE, on the other lane, and the live lane repeated it: the compact-ASH fixtures carried an arbitrary 0xa5 pattern in the same symbol, nothing noticed while no sponsored shape was ever built, and their first sponsored rows were refused with the same words — Script failed an OP_EQUALVERIFY operation — at the same comparison that stopped this lane against 0xb5. That helper's own doc already carried the upstream citation, pushspk and GetOutputScriptPubKeysSHA256 in the target's interpreter source, and the live lane had a hardcoded copy of the answer sitting in a test asserting the mismatch; the literal is deleted and the test now reads the helper, since a written-out digest beside a helper computing the same value is the drift the helper exists to prevent. BYTE IDENTITY RE-PROVEN, and by four identities rather than one: the native lane run at the threaded tip reproduces every run-of-record identity the shape-possibility register cites — the split whose whole report is byte-identical to the pre-change baseline, the many-to-many likewise byte-identical, the two-owner shape and the one-to-two control. The last two are recorded as REPRODUCED rather than compared, and the reason is stated rather than glossed: the inherited baseline was a DEGRADED run carrying an executor-refusal artifact and two empty reports, so those two files had nothing in them to be identical to, and the honest claim is that this run filled them with the identities the plans already cite. Seven of the substantive reports compare byte-identical outright and only the timing files differ. THE SUBMISSION WAS ACCEPTED. The sponsor-signed explicit control was taken by the pinned node, 1480 bytes submitted and 1480 read back from the node's own copy and equal to the submitted bytes, mined into a block at height 6, carrying a two-item sponsor witness of 72 and 33 bytes replayed from the adapter's answer. THE RELAY BOUNDARY WAS CROSSED, and this is read off the path rather than assumed: the executor reaches an accepted layer ONLY when testmempoolaccept answered allowed, and it then confirms with generateblock, so this control was judged relayable AND consensus-valid — which none of the three previous submissions reached, every one of them having died at script evaluation and learned nothing about standardness. The node weighed 2482 for it and the fee it weighed was 250 in the reserve asset, the sponsor coin being funded to exactly the offer since the sponsor asks for no change. THE RESIDUAL SponsorEnvelopeSignerAbsent IS CLEARED, by its own defining site's rule and in that order — run first, verdict second. That rule said a returned byte stack is not the sponsor owner's target authorization until a target has accepted a control carrying it; a target has. The variant is KEPT in the vocabulary on the SighashProfileUnreviewed precedent, with an it-is-no-longer-carried section naming the observation, a what-did-not-move-with-it section, and the closing sentence that keeps the word — several lanes genuinely still have no signer wired and must be able to say so. NO HISTORICAL CLOSEOUT'S MACHINE CHECK MOVED, and the reason is mechanical rather than lucky: carried_residuals has exactly one consumer in the whole workspace, and every wave closeout hardcodes both its blockers set and the assertion against it, so the wave five, six and seven records stand untouched. The wave-seven ledger is annotated rather than amended, saying in its own doc that it records what wave seven observed and that backdating an acceptance into a wave that did not have one is the one thing a ledger of record exists to prevent. EXACTLY ONE CENSUS FIGURE MOVED and it is reported as a move: the specific-blocker map no longer answers for missing-sponsor-authorization, because leaving it would make two artifacts disagree about one row — the residual set saying the dependency is closed while the matrix still said the row is blocked on it, which is the precise defect a previous wave had to repair between a closeout and the matrix. That row is NOT thereby answered; it falls through to a native-run-required standing, since it is a NEGATIVE asking that a control missing the sponsor's authorization be refused while what was accepted is a POSITIVE sponsored control. The blocked-row census therefore falls from three to two and the completeness assertion beside it does not change at all. NO MATRIX ROW MOVED. private-sponsor-values did not and could not: it asks for confidential sponsor values and this control is explicit, and its may-enter-the-delta ground is restated in those terms rather than left resting on a residual that has cleared. Step-six restart sponsor cases and step-seven minimality pairs are a follow-up wave's and were not touched. THE STEP-7 ORDERING ERRATUM IS FILED against the confidential-funding guide's restart order: the mandatory chain plus step six's conditionality on a dependency the guide does not close makes step seven permanently unreachable rather than merely deferred, which is stronger than the text intends. It is filed WITH the fact that closing the dependency is what made the defect legible rather than what removed it — the chain held step seven hostage for three waves to a symbol bug in a repository the guide does not own, which is exactly the case a mandatory chain should not be able to reach; a revision should stop a step typed-stopped on a named external dependency from consuming the entry of the steps after it, and admit non-sponsor minimality pairs on step five's observed result. WHAT THIS DOES NOT ESTABLISH is written into the run record itself and into the defining site: the sponsor envelope's WIRE and ONE target acceptance, and NOT production multi-party sponsor signing — one fixed regtest key signed once, and a single key answering a request is not a ceremony; ADR-015 public disposable material throughout. Verified entirely on the server per the no-host-compute ruling, every step timed. Removal path as filed, now discharged: Removal path filed by the sponsor wave: the live deployment's FEE PROGRAM DIGEST symbol is a fixture constant no program hashes to, and it makes every sponsored control the demonstration deployment builds unspendable at its own fee-role check. The isolation fragment requires the fee output's scriptPubKey digest to equal the symbol; construction writes the empty program, whose digest is the specification's SHA-256 of the empty string. Two options, and they are not equivalent: pin the symbol to that digest, which is correct for every deployment whose fee role is the target's structural one but MOVES the demonstration deployment's committed taptree and therefore every live run-of-record identity and weight; or thread the digest at run time as the reserve asset now is, which keeps the demonstration byte-identical and lets a submitting lane supply the real one. The second is the shape the reserve took and the reason it was taken — a deployment is welded to its symbols, and the demonstration's identities are cited across the plans. The sponsor-change program symbol is the same class and is NOT in this row, being unreachable while no sponsored control takes change. Until this is closed no sponsor-signed control can be accepted, and SponsorEnvelopeSignerAbsent cannot clear |
| `T5-045` | DONE | Removals-completion wave: the two structural removals the shape-census wave landed at the registry are finished on the rule that a removal is complete only when the shape RUNS, and one of the two shapes ran. THE PRIVATE MERGE IS ACCEPTED -- two receipts consumed, one output created, 5156 bytes submitted, one range proof of 4174, the mined bytes equal to the submitted ones and the read-back witness verifying against an independently recomputed message, 10.9 seconds of wall time. The wall was never the merge and never the protocol: it was the ARITY of the one predecessor the ceremony funded, two output blinders funded from an explicit input being ordered additive inverses, so merging both halves forced a ZERO blinder -- a commitment of exactly the value times the value generator, which anybody recomputes from a guessed amount, hiding nothing while the tally still balances, and which the registry refuses by name and should. Cancellation is a consequence of having two outputs and not of the explicit input, so the ceremony gained a SECOND predecessor of three outputs, identical in every other respect, whose blinders cancel in no pair: any two sum to the negation of the third and no blinder is ever zero, a derived one being searched until it is nonzero and a solved one refused when it is not. ONE DIVERGENCE from the filed path is reported and is not a defect: the path named a CHAINED precursor submission and what was built is a wider FUNDING stage, because a funding response carries decoded coins with their openings while a submission response carries none, so chaining would have meant building a coin-return path the ceremony does not have; the arithmetic is the arithmetic the path named. THE FORCED BLINDER IS SHOWN NONZERO rather than argued: the ceremony sums the coins it actually consumes, compares that sum with the blinder the registry solved, and writes four facts into its own transcript -- the coin count, whether the consumed sum is zero, whether the forced blinder is zero, and whether the two agree -- never the scalar, which would be publishing an opening. ONE ROW MOVED, `private-merge`, on an observed acceptance of its own shape through the established NativeRunObserved standing; the private table's answered count goes six to seven and the census's observed set gains a CONVERSE check, so a shape that becomes observed can no longer be added to the register without being added to the list that counts it. THE FEE ROLE IS PROJECTED and the shape STOPPED TYPED one layer further on. The materializer's role vocabulary gained a Fee member, its per-output stage a fee stage emitting explicit value, explicit asset, null nonce, empty program and an empty witness entry which then asks the built output whether it IS a fee by the target's own three-conjunct predicate, the projected view's three opening scalars became optional because a fee output's opening is ABSENT and not zero, and both projections state the role instead of a catch-all that would have solved a blinder for it. FeeRoleNotProjectable is REMOVED rather than left standing as a stopper nothing can return, and its test is updated to the new fact rather than loosened. The fee output built by that chain IS a fee: the run's output-witness census reads [4174, 0], one range proof and one empty entry, which a fee mapped onto the balancing role could not have produced. The target REFUSED the candidate at mandatory-script-verify-flag-failed, Script failed an OP_EQUALVERIFY operation, 4870 bytes submitted -- and NOT for carrying a fee: Elements admits a fee output in a non-policy asset at consensus and at policy alike, IsFee constraining the form and never the asset and HasValidFee checking only that the amount is nonzero and in range, and the adapter's node runs at a zero minimum relay feerate. It failed the covenant this workspace built for it, the reviewed live-transfer shape vocabulary having no sponsorless fee-bearing member, so a two-destination sponsorless request selects a shape of TWO RECEIPT OUTPUTS and the receipt covenant demands a receipt program where the empty fee program sits. That is a THIRD layer, uncovered by the second removal exactly as the second was uncovered by the first, and NO IDENTITY IS MINTED for it: the run of record carries the digest, the bytes, the witness census and the target's verbatim verdict, and states that the accepted identity is None. The census gains the layer with a filed path -- a sponsorless shape member that pays its own fee, filed rather than taken because the vocabulary states a REVIEWED reading of the target and changing one is a ruling -- together with a sixth first-party status, SubmittedAndRefused, minted because expressible-and-unrun could not say that a node had been asked and had answered. A LATENT PANIC is fixed: the confidential funding record read the admitted prefix pair into a two-element array and indexed it by output index, so a funded record wider than two outputs panicked on the third and a one-output record silently demanded the FIRST prefix -- a rule nobody stated and one the registry's own parity search contradicts. It now reads the arity-general rule, two-output fixed order preserved in every byte and membership at every other width, on both sides of the executor wire; four tests cover widths one, two, three and five and check the record's reading against the registry's. The three-output predecessor's coins carry prefixes 0x09, 0x08, 0x09, which a fixed-order rule would have rejected and the reviewed contract does not. BYTE IDENTITY is proven and not asserted: the four shapes that ran before this wave re-register their successors under exactly the digests those runs recorded, the dual-parity predecessor and both one-to-one successors under theirs, and the six shapes' successor digests are checked pairwise distinct. NO OTHER ROW MOVED -- deterministic-openings, projection-equality, private-sponsor-values and the whole explicit table are untouched, and the fee-bearing shape is emphatically not private-sponsor-values, paying its own fee out of its own consumed coin with no sponsor and no signer dependency anywhere near it. Verified entirely on the server per the no-host-compute ruling, every step timed: formatter and clippy clean, 326 vectors tests green in 138.4s, 262 transaction tests in 19.5s, the conformance package green in 184.7s, and both native runs serialized against a real node, the merge in 10.9s and the fee-bearing candidate in 12.4s |
| `T5-046` | DONE | Explicit-runs wave: THIRTEEN OF THE SIXTEEN positive explicit rows of the live-transfer safety matrix are answered, moving the explicit table from ZERO on thirteen acceptances at the pinned node. A shape-parameterized explicit-lane ceremony was built at live_explicit_shapes, the sibling of the private lane's multi-shape ceremony and standing on the explicit spine the owner-observation lane established, varying nothing between runs but the shape: how many coins each published owner is funded, how many destinations are created and for whom, what each is worth, and the order the receipts are offered in. EVERY ONE OF THE THIRTEEN WAS ACCEPTED, and each carries three separate observations rather than one: the identity the target computed, the node's own copy of the transaction read back EQUAL to the bytes it was handed, and EVERY INPUT'S signature taken out of that copy and verified against a message this workspace recomputed for that input's own position -- per input rather than per transaction, because a multi-input candidate whose first signature verified would say nothing about its second and most of these shapes are multi-input. The one-to-one in 593 bytes is the smallest submission this lane has ever made, for the structural reason that an explicit transfer carries no range proof at all; the widest is the three-input maximum in 1421. THIRTEEN RUNS PRODUCED TEN IDENTITIES and the shortfall is a finding rather than a defect: three pairs of rows are two CLASSES of one transaction rather than two transactions, since a single transfer is an instance of several of the sixteen classes at once, and building a gratuitously different transfer so each row could cite its own hex string would be dressing one fact up as two. THE SHARPEST OF THE THREE IS THE NORMALIZATION ROW, whose shared identity is its evidence rather than a shortcut: the normalization run offered its two receipts in the REVERSE of their canonical order and the merge run offered them in it, the two built byte-identical transactions, and the node computed one identity for them -- a second identity would have been evidence that the request does not normalize. A census test holds the collision set to exactly those three pairs, so a pair that began sharing an identity because a shape stopped differing fails rather than passing quietly. The boundary-values row ran at the boundary the request type actually states, one unit and the remainder, one being the smallest value ProtocolValue admits rather than a small number somebody picked, and the node took it. Every key, blinder and opening is ADR-015 public disposable test material, named as such at each use. THE TWO SPONSOR ROWS FOLLOWED on the sponsor-envelope lane's own accepted control, re-run at this tip rather than transcribed out of a closeout, reproducing it with 1480 bytes read back equal and the relay boundary crossed before the mine; it carries a sponsor region so it is the sponsored class, and it requests no change -- read off the ceremony's own request rather than inferred from the outputs -- so it is the sponsor-change-absent class too. FIFTEEN OF SIXTEEN, and the sixteenth is a TYPED STOP with its path filed at the register: sponsor-change-present asks for a sponsored transfer that TAKES CHANGE and no ceremony here builds one. THE FILED OBSTACLE CORRECTS AN EARLIER WAVE'S PREDICTION rather than repeating it. That prediction was that the sponsor-change program symbol is a fixture pattern no program hashes to, and that such a control would die at its own change-role check the way the first sponsored controls died at the fee-role check. Reading both sites says otherwise: the FEE role's program is target-structural, so construction wrote the empty program while the symbol was arbitrary and the two disagreed, whereas the CHANGE role's program is a deployment's own choice and construction writes the change output FROM the symbol, refusing outright if a sponsor capability offers any other destination -- so the covenant compares against the value construction just wrote. What is actually missing is an OFFER that carries change: the construction path places a change output only where the sponsor's offer states a change amount, and the sponsor lane funds its coin to exactly the offer. Closing it needs a sponsor funding step that funds ABOVE the offer, and the sponsor ceremony lifted out of the integration test it currently lives in. THE NEGATIVE HALF OPENED with the two witness-content rows of the owner-and-signature table, empty-signature and malformed-signature, answered on refusals that are ATTRIBUTABLE rather than merely recorded: one run submits both mutants and then the unmutated control to one node on one chain, so the accepted form and the refused ones differ in nothing but the item that moved, measured by comparing the two submissions rather than argued from the code that built them. THE SUBMISSION ORDER WAS THE FINDING, and a run falsified the obvious one. A witness-content mutation changes the WITNESS, and the witness is not part of a transaction's identity, so a mutant carries the SAME identity as its control; with the control submitted and mined first, both mutants came back refused txn-already-known at a layer before script evaluation -- a true refusal about an identity already on the chain, attributable to the submission order and to nothing the witness offered, and counting it would have been the exact error the attributability rule exists to prevent. With the mutants offered first the target's answers MOVED, which is itself the demonstration that the earlier answers were about the order: the empty offering drew Script failed an OP_CHECKSIGVERIFY operation and the well-sized non-signature drew Invalid Schnorr signature, two DIFFERENT verdicts that make each row its own rather than one observation counted twice, and the control that followed was accepted. The wrong-order refusal is kept in the register rather than deleted, because a record of only the ordering that worked would lose the reason the ordering matters. A NEW STANDING WAS MINTED and the machinery extended symmetrically rather than in parallel: NativeRefusalObserved sits beside NativeRunObserved, carrying the ACCEPTED CONTROL'S identity -- the half a reader can check against a chain, the refusal having left no transaction to look up -- together with the target's verbatim words, and it is counted in its own census bucket rather than added to the acceptance figure, because one number would let a reader take a refusal for an acceptance. ONE PROCESS FINDING is reported against this wave's own conduct rather than against the code: the iteration clone shares its CARGO_TARGET_DIR with the lane's checkout, so building the BASE commit there to attribute a failure poisoned the shared cache and produced three phantom results in a row -- a private-lane digest mismatch, a native binary carrying fourteen tests where the source has twenty-eight, and a fee-bearing shape refused at construction as an unsupported shape. None of the three reproduced: the digest test passes three consecutive times at the tip, and the fee-bearing shape passes at BOTH the base and the tip when each is built in an ISOLATED target directory, which is the comparison that attributes it. The verdict of record was then taken from a fully cold build. It is recorded because the cheapest way to manufacture a regression is to build two commits in one cache. Verified entirely on the server per the no-host-compute ruling, every step timed |
| `T5-047` | DONE | Fee-vocabulary wave V1: the sponsorless fee-bearing CONFIDENTIAL shape. The live-transfer shape vocabulary gains a fee axis carried on the BOUNDS so the demonstration set unrolls to exactly the members it does today and every recorded fixture digest re-derives bit-for-bit, the three coupled sponsored() reads that decide where the fee sits move together, and the confidential sponsorless one-to-one-plus-fee candidate the removals-completion wave built and a real node refused at OP_EQUALVERIFY is driven to an observed acceptance at the pinned node. On that observation the census row moves off SubmittedAndRefused and the filed removal path is recorded TAKEN. THE AXIS IS LANDED AND THE DIGEST AUDIT IS DONE, recomputed rather than argued. LiveTransferShape and LiveTransferShapeBounds carry FeePresence, the bounds leave it off for the demonstration candidate, and a separate fee-bearing candidate is what the fee ceremony links against -- so exactly ONE recorded digest moved, the fee-bearing successor's, which never carried an acceptance and therefore cost no evidence to re-record. Every other recorded digest re-derives untouched and that is a RUN and not a claim: the vectors lane reports 325 passed with the single failure being the moved digest's own assertion, the four shapes that ran before the removals wave re-registering under their own recorded figures and the dual-parity predecessor and both one-to-one successors under theirs. FOUR COUPLED READS MOVED, not the three the brief named. The fourth is has_sponsor_region, which gated whether the isolation PATTERN was emitted at all, the coordinator's disclosure set, and the pattern census; it is a pure alias of sponsored() and had it kept asking the sponsor question the fragment would have carried the fee clause while the census said the leaf was empty. It keeps its meaning and loses its second job to emits_isolation_fragment. THE ACTUAL WALL WAS IN NEITHER SET: select_shape matched the request's destination count against a shape's receipt_outputs, and the private request vocabulary has no fee destination member, so the ceremony states the fee AS a destination and two destinations selected a two-receipt-output shape. The fee positions are counted out before the comparison from the OPENINGS' roles, which is where this lane already decides a destination is a fee. ONE STRUCTURAL CORRECTION the design memo had backwards: it fixed the fee at the RESERVE asset because the isolation clause required it, while the ceremony denominates the fee in the PROTOCOL asset as the only asset that closes the tally. Both cannot hold, and Elements balances per asset, so a reserve-asset fee beside no reserve-asset input dies at the tally. The clause now reads the asset off who funded the fee. The 10.4 closure argument is restated rather than weakened: what it forbids is an UNACCOUNTED protocol-asset output, and a declared fee position at an index the exact output count fixes is spoken for as completely as a destination is. THE SHAPE RAN AND THE NODE TOOK IT -- one receipt consumed, one blinded destination created, the transaction's own fee paid out of the value it consumed with no sponsor anywhere in it, 4927 bytes submitted and the mined bytes read back EQUAL to them, the witness verified against an independently recomputed message, 12.3 seconds of wall time. The fee output is a fee by MEASUREMENT and not by claim: the run's output-witness census reads one range proof of 4174 bytes and one EMPTY entry, which a fee mapped onto the balancing role could not have produced. IT TOOK TWO NODE RUNS AND THE FIRST ONE EARNED ITS KEEP. The first still returned Script failed an OP_EQUALVERIFY operation, but with construction_refusal none -- which located the wall rather than merely reporting it, the fee-bearing shape having been selected so the failure had moved one comparison further on into the fee clause itself. A FOURTH LAYER, uncovered exactly as the third was: the deployment is welded to a fee-program digest of 0xb5 bytes that no program hashes to, kept deliberately so the demonstration's committed taptree would not move, and nothing had ever executed the clause that reads it. The constant's own doc had predicted this and named the answer, and the fee-bearing deployment now supplies the digest of the fee program it constructs -- at no cost to the demonstration, whose taptree and identities are untouched, precisely because the fee-bearing vocabulary is a separate deployment. ONE ROW MOVED and it moved on an OBSERVATION: OneToOneWithFee from SubmittedAndRefused to ObservedAccepted with ConstructibleAfterRemoval, the removal carrying proven_by, which is the whole of what separates a taken path from a filed one. THE SIXTH STATUS IS NOW UNOCCUPIED and is KEPT, its two tests re-read to assert COHERENCE rather than occupancy on the same reasoning that keeps ConstructibleAfterRemoval alive -- the next shape a target refuses must be able to say so. A check is added that each removal is proven by a DISTINCT identity, so one acceptance can never be cited for work it did not do. The removal path RemovalPath::SponsorlessFeeBearingShape is recorded TAKEN under the owner fee-matrix ruling at the gate commit. TWO CORRECTIONS THE DESIGN MEMO HAD BACKWARDS, both forced by the target. It fixed the fee at the RESERVE asset because the isolation clause required it, while the ceremony denominates the fee in the PROTOCOL asset as the only asset that closes the tally; Elements balances per asset, so a reserve-asset fee beside no reserve-asset input dies at the tally, and the clause now reads its asset off who funded the fee. And the explicit conservation relation gained the fee as a TERM per ruling R-C -- a latent trap the axis itself created, since making the form representable made an explicit fee-bearing leaf emittable that demanded a positive fee be zero and would have refused every candidate offered to it. The 10.4 closure argument is restated rather than weakened: what it forbids is an UNACCOUNTED protocol-asset output, and a declared fee position at an index the exact output count fixes is spoken for as completely as a destination is. FOUR COUPLED READS MOVED, not the three the brief named; the fourth was has_sponsor_region, a pure alias of sponsored() gating whether the isolation PATTERN was emitted at all, which had it kept asking the sponsor question would have carried the fee clause while the census recorded an empty leaf. THE ACTUAL WALL WAS IN NEITHER SET: select_shape matched destination count against receipt_outputs, and the fee positions are now counted out from the OPENINGS' roles, where the lane already decides a destination is a fee. Verified entirely on the server per the no-host-compute ruling, every step timed: tapscript 32.5s, transaction 37.7s with 262 tests, vectors 127.4s with 328 tests, conformance 197.7s, both plans validators 12.1s with the register byte-stable, and the serialized native lane of record 176.8s with 14 of 14 green. |
| `T5-048` | ACTIVE | Sponsor-change wave: the SPONSORED side of the fee matrix and the confidential sponsor values, in three phases each committed before the next begins. Phase A builds the first sponsored transfer that TAKES CHANGE as an EXPLICIT control -- a sponsor funding step that funds ABOVE the offer, an offer that states a change amount, the sponsor ceremony lifted out of the integration test it lives in today into a reusable lane, and the whole driven to an observed acceptance at the pinned node with the mined bytes read back equal and the sponsor witness replayed against an independently recomputed message. On that acceptance the sponsor-change-present row of the explicit table moves and the typed stop the explicit-runs wave filed is answered. Phase B builds the confidential sponsor FUNDING form, whose value blinds while its asset stays explicit because the covenant reads the asset and the reserve-asset guard has to keep its food: the private lane admits a sponsored request, the funding-region classifier gains a member for the sponsor's own region, the executor funds a blinded sponsor coin on the pattern the confidential receipt path already uses, and the shape vocabulary gains the minimal sponsored-and-confidential member the exit-gate ruling entails. Phase C drives a sponsored CONFIDENTIAL with-change control to an observed acceptance of ITS OWN shape -- blinded sponsor value in, blinded change back, explicit fee out -- which is the only observation the private-sponsor-values row moves on, and answers the missing-sponsor-authorization negative behind it if that refusal is attributable. A blinded sponsor value is meaningless WITHOUT change and that is arithmetic rather than preference: in a without-change shape the reserve sub-equation forces the sponsor input's value equal to the explicit fee, so the commitment opens to a publicly derivable number. Any phase that cannot complete takes a TYPED STOP with its path filed and the later phases stop on the dependency rather than narrating around it. PHASE A IS DONE AND THE EXPLICIT POSITIVE TABLE IS COMPLETE AT SIXTEEN OF SIXTEEN. The sponsored control that TAKES CHANGE was accepted at the pinned node, 1636 bytes handed over and the node's own copy of the mined transaction read back EQUAL to them, weight 2872, mined at height 6 with the relay boundary crossed before the mine. The change role is read OUT OF THE NODE'S COPY rather than off the request and located by the deployment's own sponsor-change program rather than by counting -- position 2, value 1000, reserve asset -- because a position is what a shape degraded to the without-change form would still have and the program is what it would not; a with-change run that reached a node carrying no change output is a HARD STOP in the ceremony rather than an acceptance reported under a row it says nothing about. Both owner signatures were taken out of that copy and verified against messages recomputed here, per input rather than per transaction, and the sponsor adapter's exact two witness items were found in the mined bytes. THE PREDICTED OBSTACLE WAS NOT THE OBSTACLE, and running is what decided it rather than more reading. One reading said the demonstration's sponsor-change program symbol is a fixture pattern no program hashes to, so a control taking change would die at its own change-role check the way the first sponsored controls died at the fee-role check. The other said the two symbols only LOOK alike: the fee role's program is target-structural, so construction wrote the empty program while the symbol was arbitrary and the two disagreed, whereas the change role's program is a deployment's own choice and construction writes the change output FROM the symbol, refusing outright if a sponsor capability offers any other destination. The second survived. The control was accepted AT THE FIRST ATTEMPT with the symbol untouched: nothing threaded, nothing repointed, the committed taptree unmoved. What was actually missing was an OFFER that carries change, and the whole fix is a sponsor funding step that funds ABOVE the fee -- the construction places a change output only where the offer states a change amount, and an offer can only state one where the coin holds more than the fee. THE CEREMONY IS LIFTED and the lift has its own control. It lived inside an integration test, where it could answer exactly one row because a test cannot be called from another test; it is now a lane with ONE axis, whether the sponsor asks for change, holding the issuance, the receipts, the destinations, the owners and the fee equal so the difference between the two submissions is attributable to the change role. The without-change member REPRODUCED at 1480 bytes and weight 2482 after the move, which is the lift's own byte-identity proof: a lift that changed what the ceremony builds would have produced a different identity. The two identities are asserted DISTINCT, so a with-change run that silently degraded fails rather than passing quietly, and the change role's measured width is stated as the 156 bytes the node charged for it rather than bounded. The duplicated ceremony leaves the test that no longer builds one, which keeps its three deployment-symbol claims that need no node. TWO STALE GROUNDS ARE CORRECTED. The wave-7 closeout's reading that the confidential fixture vocabulary has no fee role is stale -- the role exists and REQUIRES the empty program -- and the correction is recorded at the role's own site rather than in the closeout, because a closeout is history and history is not edited. And the E5 row of the confidential-funding guide rested private-sponsor-values on SponsorEnvelopeSignerAbsent, which is cleared; its live ground is the explicit/confidential gap, plus the arithmetic that a confidential sponsor value requires change at all. THE SPONSOR-AUTHORIZATION NEGATIVE IS ANSWERED, and answered behind an acceptance rather than merely recorded. One run offered a sponsored control whose sponsor input carries NO authorization and then the unmutated control, to one node on one chain. The mutant is not a second ceremony: it is the RECORDING pass's own completion, one finalization with every owner really signing and the sponsor capability answering with an empty stack, so it differs from its control in the sponsor witness and in nothing else -- measured at the node, which weighed it at 1375 bytes against the control's 1480, short by exactly the seventy-two-byte signature and thirty-three-byte key the adapter returned. The target refused it at ScriptPathRejection with mandatory-script-verify-flag-failed (Script failed an OP_EQUALVERIFY operation) and then accepted the control. THE VERDICT READS LIKE AN EARLIER WAVE'S AND IS NOT IT, which is worth saying because the string is identical: that one was the covenant's own fee-role comparison before the fee digest was threaded. What separates them is the control accepted in the SAME run -- the fee-role check passes for this deployment, so what failed is a comparison the sponsor witness reaches, and the admitted sponsor program being the target's version-zero key hash, an EMPTY item where the key goes makes the hash of nothing meet the committed digest before any signature is judged. The mutants-first order is kept for the reason the previous wave learned it: a witness mutation leaves the identity alone, so a mined control makes its own mutant read txn-already-known at a layer before script evaluation. THE THIRD OBSERVED REFUSAL is counted in its own census bucket beside the two witness-content rows. THE recognize_sponsors ASYMMETRY IS ASSESSED AND THE ANSWER IS INTENDED, decided by running rather than by more reading. The spike flagged it as possibly unintended: a receipt's value form is gated against the representation plan and a sponsor's is not, with no prose either way. Closing it is what proved it is not a gap -- the mirror clause was written, and it immediately refused the disclosure-minimality pair registry's own SPONSOR pair, whose PRIVATE member deliberately carries an EXPLICIT sponsor value. The reason is section 1.9's: a representation plan is about the PROTOCOL region, and the sponsor region is kept outside every protocol claim -- reserve asset, outside both balance equations, and section 10.7's isolation fragment introspects no value field in it at all, a property checked on the emitted instructions. A clause there would not enforce section 6.3 but extend it over a region it was written to exclude. The clause is REVERTED, the reasoning is recorded at the site, and a test now holds the shape that proves it: private receipts carrying commitments, sponsored by a reserve coin whose value is explicit. THE THIRD READING THIS WAVE OVERTURNED BY RUNNING, and the first of the three was this wave's own. PHASE B TAKES A TYPED STOP AND PHASE C STOPS ON IT. No ceremony funds a sponsor coin whose value is blinded, and the path is FILED as a typed value carrying its site inventory rather than as prose. What the arithmetic does NOT need is recorded too, because it is the cheap half and a later wave should not re-derive it: the transaction-wide solve subtracts freely chosen Primary blinders from the input blinder sum and holds the fee out at a zero blinder, and the blinder term does not depend on which asset an output carries, so a blinded sponsor change is simply a Primary and the existing single balancing election absorbs the residue with no second election and no new role; the destination intents already carry a PER-OUTPUT asset, so a reserve-asset blinded change needs no widening there either. What it does need was read at each site rather than predicted: the private lane refuses a sponsored request at its entry and passes a sponsor count of zero to shape selection, its openings vocabulary indexes inputs against the receipts alone, its destination intents are built from the receipt destinations, and -- the structural one, which is not a widening of anything -- it has NO sponsor signing stage at all, its control assembly building one witness per receipt record with no sponsor slot and no capability call. Outside the lane the funding-region classifier needs a region member as well as both of its clauses, and the executor funds and signs its sponsor coin explicitly throughout with no blinding key. THE CLASSIFIER'S OWN ORDERING IS NOW A RUN rather than a reading of the branch order: a blinded sponsor coin trips the explicitness clause FIRST, which MASKS the proof clause, so a repair that fixed only the refusal it saw would meet the second at once. Since Phase B stops, the section 15.2 private-sponsor-values row does NOT move and no confidential sponsor claim is made anywhere; what it is no longer blocked on is its arithmetic precondition, the with-change shape, which now runs. |
| `T5-049` | ACTIVE | Sponsor-values wave: the CONFIDENTIAL sponsor funding form, taking the typed stop the sponsor-change wave filed and driving the sponsored confidential WITH-CHANGE shape to an acceptance of its own shape. The stop is a typed value carrying its own site inventory rather than prose, and this wave exists to flip it by building what it inventories: the executor funds a sponsor coin whose VALUE is a commitment on the pattern the confidential receipt path already uses, the private lane admits a sponsored request where it hard-refuses one today and passes a real sponsor count where it passes a hardcoded zero, the funding-region classifier gains a member for the sponsor's own region, the reviewed shape vocabulary gains the minimal sponsored-and-confidential member the exit-gate ruling entails, and the private lane gains the SPONSOR SIGNING STAGE it has never had -- the structural site, which is not a widening of anything. The sponsor input's ASSET stays EXPLICIT because the covenant reads it and the reserve-asset guard must keep something to read, and the fee stays explicit because consensus defines a fee by its explicitness; only VALUES blind. The arithmetic is already settled by the stop and is not re-derived: a blinded sponsor change is simply a Primary, the existing single balancing election absorbs the residue with no second election and no new role, and change and absorber outputs are DECLARED DESTINATION POSITIONS rather than a fourth family. The classifier's observed ordering is repaired alongside, because the explicitness clause MASKS the proof clause today and a refusal that cannot be attributed to its own cause is not a refusal this workspace counts. The section 15.2 private-sponsor-values row moves ONLY on an observed acceptance of the sponsored confidential with-change shape at the pinned node, and any layer that cannot pass takes a TYPED STOP with its path filed rather than being narrated around. THE CLASSIFIER LAYER IS DONE AND THE MASKING IS GONE. The funding-region classifier decided a member's region LAST, after one explicit-only rule that every non-protocol member met whatever region it actually belonged to, which is why a blinded sponsor coin tripped the explicitness clause first and masked the proof clause it also tripped. The region is now decided on the PROGRAM ALONE -- the one property none of the clauses judge, which is what keeps the decision from depending on its own outcome -- and each clause became a rule of the region that owns it. The sponsor's reserve coin has that region and four rules of its own, each separately refusable and each with a mutant: a committed value, the range proof a committed value requires, an explicit asset the isolation fragment can introspect, and the empty surjection proof an explicit asset entails. The program is passed in rather than inferred, because a sponsor's coin and the funding party's change are alike in BOTH properties the classifier could otherwise use and telling them apart by value form would decide the region from the very field the region governs. The two explicit-only refusals are now each reachable on a mutant that breaks one rule and no other, so neither stands in front of the other. Conformance lane green at 228.8 seconds. THE ABSORBER CLAIM IS RUN RATHER THAN ARGUED AGAIN, and the materializer's own reading of the spike is CORRECTED. The spike said the destination intents already carry a per-output asset so a reserve-asset change needs no widening there: half right and half not, because the TYPE carries one and both the caller and the validator wrote the protocol asset over it. The materializer folded every destination into ONE protocol asset and refused a set that was not single-asset, so a sponsored confidential transfer -- protocol asset in the receipts, reserve asset in the sponsor's change and its fee -- was INEXPRESSIBLE rather than merely unbuilt, and the refusal it died at is named. The sponsor's change is now a DECLARED DESTINATION POSITION and not a second region, carrying its own asset, and two answers moved out of inline matches onto the role itself so the semantic balance and the asset agreement read one answer rather than each deciding it. The protocol balance of a sponsored transaction is the equation it would be if the sponsor were not there, which is section 1.9 rather than a convenience. The claim itself is executed: a sponsored candidate materializes and the balancing output's blinder is the solve over BOTH freely chosen blinders, with the test asserting first that the two candidate solves are DIFFERENT values so a run that had quietly left the sponsor change out of the subtraction fails rather than passing on a coincidence. No existing path moves, because all three existing roles answer the protocol-member question the same way they always did. Transaction lane green at 13.6 seconds, 265 of 265. |

Current blockers are: the ONE carried residual of the live evidence set, `PredecessorConstructorAbsent`. `SponsorEnvelopeSignerAbsent` is CLEARED, by its own defining site's rule and on an observed result rather than on a capability existing: a sponsor-signed explicit control was accepted by a real node, read back from the node's own copy byte for byte, and mined, which is the acceptance that rule said a returned byte stack had to wait for. The sighash blockers are cleared, the confidential predecessor is funded, the signing handoff is closed, and the evidence restart has taken its first five steps against a real node — six positive private matrix rows are answered by observed acceptances of their own shapes, the three proof-negatives are observed at the mempool boundary and attributed by field, and the multi-output and multi-input fixture constructor exists and derives at any width. The restart order stopped typed at step six on the sponsor signer dependency through three waves; that dependency is now closed by observation, and the step-six restart sponsor cases and step seven's disclosure-minimality pairs are a follow-up wave's to record, neither having been run. The four positive private rows that remain unanswered are each unanswered for their own stated reason: private-merge is structurally unconstructible against the registry's two-output floor, private-sponsor-values may not move at all, deterministic-public-fixture-openings has its determinism observation recorded but no target-computed identity to move on, and projection-equality-with-paired-explicit needs both sides of a pair the order has not reached. The remaining work is the step-six and step-seven records, now that nothing structural stands in front of them; one question is still handed up for a ruling, the deterministic-fixture-openings gate, and the step-six/step-seven ordering question is answered by the erratum filed against the guide's mandatory chain. The sponsor dependency is CLOSED and its closure was a repair to this repository rather than to the target: the deployment's fee-role program digest was a fixture constant no program hashes to, threading it to the digest the target itself computes for the empty fee program construction writes produced the first observed sponsor-signed acceptance, and the record says in its own bytes that this establishes the sponsor envelope's wire and one acceptance and not production multi-party sponsor signing.
