# Tripod Implementation Backlog

> **Status:** ACTIVE
> **Current gate:** Phase 3 — Elements target and foundational prototypes
> **Current condition:** Phases 1 and 2 are complete. The target-independent compiler constructs one deterministic, validated scoped analyzed program for the complete pilot scope, factorized per operation, with relation-owned requirements, a corruption-resistant assembly validator, and independent assembly oracles; the Phase-2 exit gate record is in §2.11, including passed document reproducibility. Phase 3 then delivered the Guide-8 target foundation, the Guide-9 native primitive gate, the Guide-10 constructor and wide-floor prototypes, and the Guide-11 declassification result, each with its own gate record. Public declassification is no longer an open blocker: an initial policy is selected — an explicit boundary reached by owner-authorized normalization, with the direct and public-committed paths deferred against three named target blockers — and the gate record is in §2.4. Phase-3 exit now turns on that recorded result together with the standing contract and infrastructure conditions in (`gate:phase3:exit`), which are checked at merge and are not claimed here. The next work is the first complete compiler-to-target operation, compact ASH, which consumes the declassification policy rather than reopening it.
> **Next gate:** Phase 4 — compact-ash end-to-end pipeline
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
(`req:identity:admission-record`) and name:

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
backend proof patterns
direct authenticated opening and public-committed representation
public opening capsule
production target-native evidence

tripod-linker
tripod-transaction
tripod-vectors
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
Backend patterns/linker/ABI:       absent
Independent deployment evidence:   absent
Production deployment:             absent
Phase-2 exit gate:                 passed and recorded, section 2.11
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
| Phase 3 | Active | current backlog and Phase-3 card |

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
(`req:identity:admission-record`). Phase 2 still mints
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

### 5.8 Dependency decision and commitment oracle · `tab:backlog:findings-ct-oracle`

The Guide-11 §6 dependency decision and the §7 independent commitment
oracle. Filed by content for the reason §5.7 gives: the preflight arc
spends the wave numbers on a different sequence.

**The dependency decision is to add nothing.** The guide permits a
generic Elements or secp256k1-zkp dependency in
`target-elements-conformance` for generator derivation, commitment
construction, and independent vectors. It was reviewed against the
ADR-011 dependency policy and declined.

| Candidate | Licence | Disposition |
|---|---|---|
| `secp256k1` / `secp256k1-zkp` (Rust FFI bindings) | MIT | **Refused on independence.** Both wrap the same in-tree C library the node vendors for confidential values. The oracle exists to be a second opinion; binding it to the implementation it checks would make agreement a tautology rather than evidence (§6.3). |
| A pure-Rust curve crate (`k256`, `crypto-bigint`) | Apache-2.0/MIT | **Not taken.** It would preserve independence, but it buys nothing the workspace lacks: the whole computation is one curve map, two point additions, and two scalar multiplications over published constants. Against that it adds a cryptographic dependency, a transitive graph, an advisory surface, and a lock-file delta. |
| First-party arithmetic over the workspace bignum | — | **Taken.** `num-bigint` and `sha2` are existing workspace dependencies with existing first-party consumers, so nothing entered the lock file, and the exact-arithmetic stance of (`dec:math:exact-certified`) is served directly. |

Because no library is shared with the materializer, the §6.3
independence claim is unqualified and its fallback — record the shared
dependency and narrow the claim — does not arise. `target-elements`
remains standard-library-only (§6.1); the oracle lives entirely in the
conformance crate.

**The oracle's expected values rest on three derivations, not one.**
Every pinned byte string was computed by the crate's oracle and, before
being pinned, by a separate implementation written from the same source
citations in a different language. The two agree byte for byte. The
third and strongest leg is external: the vendored library carries its
own published fixed vectors, and all of them reproduce.

| ID | Priority | Status | Finding |
|---|---:|---|---|
| `G11-O01` | P1 | DONE | Upstream low-level confidential vectors **do** exist, contradicting the §5 review's premise. That review read the Python functional framework, which indeed carries no Pedersen or generator helper, and concluded the oracle could not be cross-checked against published vectors. The vendored C library's own unit tests carry three sets: thirty-four curve-map outputs stated to match an independent SAGE program, thirty-two generator derivations for the asset identifiers that are thirty-one zero bytes followed by `i`, and one point under both prefix conventions. All sixty-eight reproduce exactly. The oracle's two pinned assets are drawn from the published table, so its generators rest on upstream bytes. |
| `G11-O02` | P1 | DONE | The upstream fixture named `two_g` does not encode `2G`. It carries prefix `0x0b` and is a parse-and-serialize round trip; parsing takes the point whose y is a quadratic residue and negates it when the prefix is odd, so those bytes name the *negation* of the doubled base point. `2G`'s own y is a square, so `2G` encodes as `0x0a` and `0x08`. Pinned explicitly, because reading the fixture's name as a claim would put a sign error into every later commitment. |
| `G11-O03` | P2 | DONE | A one-bit mutation of a commitment behaves in two entirely different ways depending on where it lands. Flipping a bit of the x coordinate leaves the curve and is refused; flipping the parity bit yields a well-formed encoding of the negated point. Nothing in the encoding binds the prefix to the commitment that was intended, which is the encoding-level face of the reviewed blocker that a witness-supplied parity byte is bound to nothing. |

The three-way comparison of §7.4 is landed as an interface, not as a
result. The oracle's expectation is the subject; the construction-library
and target-introspection legs are optional and arrive from later native
runs. A leg that was never supplied is reported as absent rather than as
agreement, so a comparison that did not happen cannot read as one that
passed.

| ID | Priority | Status | Item |
|---|---:|---|---|
| `G11-O04` | OPTIONAL | DONE | The oracle derives only the unblinded asset generator. The reviewed recipe also has a blinded form, which prepends a scalar multiple of the base point and is used to check that a reissuance input carries its blinded token. No present consumer needs it, and a wave that reaches reissuance should add it with its own published-vector check rather than by analogy. **Closed by Wave 7, and not by reissuance.** The §8.4 conservation matrix needed it for an ordinary reason the row did not anticipate: every balanced confidential output has a *blinded* asset, so the unblinded generator predicts nothing for exactly the rows the §7.4 comparison matters most for. The check is not by analogy and not a published vector either — it is three commitments observed on a real node, which the oracle reproduces exactly (`G11-W7-04`). |

### 5.9 Seventh-review preflight findings · `tab:backlog:findings-sr7`

The seventh static review reported two passes over the tree recorded in
§2.1. Its rows are the Guide-12 preflight identifiers `G12-R01` through
`G12-R16`, and the Guide-12 preflight table owns the required
dispositions. Guide-12 Wave 0 adjudicated every row against the working
tree rather than against the review's snapshot: each row below carries a
disposition, a DONE-or-OPEN state, and the artifact or file location
that establishes it.

Register state after Wave 0: **ADJUDICATED, ONE ROW CLOSED**. Fifteen
rows were CONFIRMED and open; `G12-R08` is RECLASSIFIED and DONE,
because a prose row's reproduction is the textual comparison and its
repair is the same act. Every other row was open, because Wave 0
reproduces and does not repair: the reproductions assert the current
defective behaviour and name the row they belong to, so the wave that
fixes each one flips its own witness.

Wave 1a is closing the identity and evidence-boundary rows. A row
reaching DONE below carries the flipped witness that now asserts the
guarantee rather than the defect. The blocking rule
(`gate:guide12-exec:preflight`) stands over every row still open.

| ID | Priority | Disposition | Evidence and trust boundary |
|---|---:|---|---|
| `G12-R01` | P1 | CONFIRMED, DONE — Wave 1a | The three fields were public and `RevisionId::new` admits a seven-digit prefix, so a struct literal reached the gate with an abbreviated expectation that `ExpectedExecutorProvenance::new` would refuse, and the whole match then compared prefix against prefix. Repaired by making the width a type: `FullRevisionId` has one validating constructor, the expectation's members are private behind read-only accessors, `RevisionId::matches_full` takes the full-width type so a prefix cannot be an expectation, and the gate re-asserts the width before comparing. `RevisionId::full` is replaced by `FullRevisionId::new`, and the validated provenance carries the full identifiers rather than the reported text. Boundary: executor provenance, ADR-018 tip attestation. Witness `an_abbreviated_expectation_is_refused_before_it_can_be_expected`, with the public surface checked by `an_expectation_is_reachable_only_through_its_validating_constructor`. The struct-literal route is now a compile error rather than a runtime refusal. |
| `G12-R02` | P1 | CONFIRMED, DONE — Wave 1a | `anchor_set_hash` took any string iterator and framed by newline join, so one name holding a newline and two names had one preimage. Repaired by validation before identity: `AnchorName` enforces the three-part label grammar and refuses the separator by name, `ValidatedAnchorSet` is all-or-nothing over its members, and the hash accepts only that set. The recipe is untouched — same domain prefix, same newline join over the same sorted distinct names — which the published-recipe test still checks against a digest computed outside this crate. Boundary: Layer-0 anchor-set pin, the manifest value release validation refuses to leave unset. Witness `no_anchor_name_can_hold_the_separator_the_framing_joins_with`. The label checker now builds the validated set and reports a diagnostic rather than hashing a census it could not parse; all 136 harvested Layer-0 anchor names satisfy the grammar, so the pinned value is unchanged. The recipe test's vector gained an area segment on its one two-part name, which was never a well-formed anchor name; the same independent computation over the old vector still returns the digest the test carried before. |
| `G12-R03` | P1 | CONFIRMED, DONE — Wave 1b | The four `emit-*` binaries in `target-elements-conformance` were absent from every `cli_common` user: no shared panic hook, no stdout TTY refusal, `std::env::args().nth(1)` argv handling, and plain-text stderr diagnostics that interpolate the caller's path. Repaired by splitting each command in two rather than by decorating it. The documents move to `src/emit.rs` — `conservation_matrix_document`, `normalization_matrix_document`, `normalization_report_document`, and `lifecycle_report_document` — and the binaries keep only what cannot be tested any other way: `clap` argument parsing behind `BaseArgs`, `install_json_panic_hook` before parsing so an early panic still fails closed, and `run_stdout_json_command`, which supplies the terminal refusal, the ADR-010 exit classes, and the write. Two things stopped being crashes on the way: the normalization matrix asserted its claim conserves and now returns a refusal a caller can observe, and the lifecycle report's gates moved into the library beside the document they gate. The run-record path became `--run-record PATH`; no in-tree caller passes it positionally, the emitters being invoked by hand under the Guide-11 evidence procedure, and the output is now the compact one-JSON-result line ADR-010 specifies rather than a pretty-printed block, which the Python readers consume unchanged through `json.load`. Boundary: shipped-binary subprocess contract, ADR-010. The source-read disposition is discharged: `src/tests/emit_tests.rs` exercises all four documents without spawning anything — the conservation requests carry the case, schema, and subject and nothing else, every executed row is asked and no deferred row is, exactly one normalization row is the unmutated claim, and the two report builders refuse a record that is not a census, a record carrying no public handoff, and a malformed check. The contract half was also confirmed by running the built binaries: a clean single-line JSON result with empty stderr and status 0, `--help` as a control-plane record, an unknown argument as a usage refusal with status 2, and a missing run record as status 1 with a JSON stderr diagnostic that names the failure without interpolating the path — `G12-R04`'s property holding through this migration. `src/emit.rs` is the seam Wave 2's subprocess harness consumes: it checks the contract, this checks the content. |
| `G12-R04` | P1 | CONFIRMED, DONE — Wave 1b | The executor raised `AdapterError` carrying `one_line(completed.stderr)` from the `elements-cli` child, and that note reaches first-party protocol records as `observed_detail` and `detail`. The module docstring's claim that the harness never reads child stderr described the harness, not this path: this side never read the executor's stderr, but the executor read its own child's into a field this side does read, so the same class of bytes arrived by the back door. Repaired by making the boundary transitive rather than by filtering what crosses it. The RPC note now states only the method the harness itself named and the client's exit status, both fixed and typed, and says that the reason is omitted; the reason is logged on the adapter's own stderr, which the harness nulls, so the diagnostic is honest about where it went rather than silently shorter. The same treatment is applied to the framework's taproot-construction exception, whose message is uncontrolled third-party text that can carry an operator's filesystem path: the record now names the exception's type, which is bounded and typed, and the message goes to the nulled stderr. The adapter's module docstring states the second direction of the contract, and `protocol.rs` records that the closure is transitive and why it previously was not. Boundary: target-adapter to first-party diagnostic and report layer. Python lane; the source-read disposition is discharged by the two call sites being the only readers of a child's stderr in the file, which `grep` over `.stderr` now shows is a single `log` call. The runtime half remains blocked on a live node, and no unit witness can stand in for it: what a real `elements-cli` writes on a real failure is exactly what cannot be observed here. |
| `G12-R05` | P1/P2 | CONFIRMED, DONE — Wave 1a | Report ingestion indexed responses into a `BTreeMap` keyed by mutation name, so a second answer for one row silently replaced the first, and the report loop read the matrix rather than the census, so a response naming a row the matrix does not carry was dropped without a word. Repaired by moving the ingestion into `normalization_report`, the module that owns the report, where `ingest_normalization_responses` refuses rather than repairs: a duplicate, an unexpected row, and an unanswered row are each a named defect, and the census is compared with `canonical_mutation_matrix` in both directions. Boundary: normalization safety report. The source-read disposition is discharged — the library seam is what makes the row testable, and the witness is `a_normalization_run_answers_the_matrix_exactly_once_each`. The binary keeps its own argument and output handling; the ADR-010 migration of the shipped emitters remains `G12-R03`. |
| `G12-R06` | P1/P2 | CONFIRMED, DONE — Wave 1a | `matrix_is_complete` consulted `self.passes.first()` alone, so a later pass could omit rows; it also asked only whether a row was present, so a duplicated row satisfied it. The emitting binary gated on `boundary_holds` and `matrix_is_complete` and never called `passes_agree`, and `passes_ran_in_distinct_processes` was satisfied by one pass. Repaired by making the census exact for every pass alike: each pass must carry every canonical row minus the globally unbuilt ones, once each and nothing else; `passes_ran_in_distinct_processes` now requires distinct attempt ordinals as well as distinct pids; and `cache_independence_established` states the whole claim in one predicate, which the emitting binary gates on. Boundary: fresh-process lifecycle evidence, the cache-independence claim. Witness `matrix_completeness_is_decided_by_every_pass`, whose required regression is pass one complete and pass two empty, with `two_passes_under_one_ordinal_are_one_pass_recorded_twice` beside it. |
| `G12-R07` | P2 | CONFIRMED, DONE — Wave 1b | The consensus judgement took the substring to the first `)` after the wrapper, and two mapped messages carry a `)` of their own, so each truncated to a string the class table does not hold and the rejection lost its class. The two are `Signature must be zero for failed CHECK(MULTI)SIG operation`, which truncated after `CHECK(MULTI`, and `OP_CHECKMULTISIG(VERIFY) is not available in tapscript`, which truncated after `OP_CHECKMULTISIG(VERIFY`. Nothing failed loudly, which is the sharp edge of the row: a lost class reads as a target that refused for an unclassified reason rather than as a parser that mislaid one. Repaired by closing the wrapper's own parenthesis through depth counting, which is what parsing the exact outer wrapper means when the payload may itself contain parentheses, and by expressing the relay judgement's already-correct whole-wrapper reading through the same `script_error_in` helper, because two spellings of one rule are two things that can disagree. An absent or unbalanced wrapper returns nothing rather than a guess, so a caller still distinguishes a reason that is not a script verdict from a script verdict naming nothing. Verified by recomputation over the whole class table rather than over the two known offenders: all 34 mapped messages were extracted in both the consensus lane, embedded in the surrounding client stderr, and the relay lane, as the bare reject reason, with no misses, and both negative shapes return nothing. Entangled with `G12-R04` and resolved together: the consensus script error is stated only on the `elements-cli` child's stderr, so removing that stderr from the note would have silently disabled every consensus classification. `AdapterError` therefore carries the client's bytes as `client_detail`, separate from the note that reaches records, and the judgement classifies them into the typed vocabulary rather than propagating them — an unmapped message becomes no class, never a record's text by another route. Boundary: target failure classification. Python lane; source-read disposition discharged by exhaustive recomputation. |
| `G12-R08` | P1/P2 | RECLASSIFIED, DONE | The count disagreement the review saw was already gone from the backlog and the code: the matrix states nine rows and the Wave-11 record states 18 of 18 over two passes. Two statements had not caught up — the declassification research file reported 16 of 16 and stopped its finding list at `G11-W11-05`, and the Phase-3 card's status blockquote broke off mid-clause after the exit-gate sentence. Both are reconciled here, which is the whole of the row: a prose row's reproduction is the textual comparison, so finding it and fixing it are one act. Boundary: active planning-document agreement. Reclassified from a census disagreement to two stale statements. |
| `G12-R09` | P0/P1 | CONFIRMED, DONE — Wave 1b | Both sides declared protocol revision 3 while implementing two different schemas. The executor wrote `observed_openings` on every conservation response and `NativeConservationResponse` carried `deny_unknown_fields` without that field, so the Rust type could not read the Python's own answers; and the lifecycle exchange had no Rust protocol type at all, so the one workload whose evidence is a public record was read out of an untyped value tree. Repaired by unifying rather than by splitting the schemas: revision 4 states the union both sides were already implementing, and the two constants move together, because a bump that reached one side alone would reproduce the fault it exists to end. `ConservationOpening` is now a declared member of the conservation response, and `LifecycleStepRole`, `LifecycleCaseId`, `LifecycleSubject`, `NativeLifecycleRequest`, `LifecycleCheck`, `LifecycleSpend`, and `NativeLifecycleResponse` give the lifecycle step the typed records every other workload had. The openings join the infrastructure refusal, which is the adjudication the row required: an opening is read back out of a transaction the node created and confirmed, so one beside a layer saying the execution never happened is a value with no possible provenance, and the section 7.4 three-way comparison would be resting on it. `NativeLifecycleResponse::validate_shape` states the same rule over the handoff, checks, spend, and witness figures, exempting only the detail, because a failure is entitled to a reason. A check's two sides are now carried as text and the agreement is decided over the values before either is spelled, so a member that was sometimes a number and sometimes a flag is gone; `emit-lifecycle-report` parses the checks into `LifecycleCheck` and refuses a malformed one instead of defaulting its name to the empty string and its agreement to false. Boundary: harness-executor protocol. Witnesses `a_conservation_response_round_trips_through_its_own_type`, `an_infrastructure_conservation_response_carries_no_openings`, `a_verify_lifecycle_answer_is_read_by_its_protocol_type`, `an_infrastructure_lifecycle_answer_carries_no_observation`, and `a_lifecycle_request_is_read_by_its_protocol_type`, the round trips being asserted against the adapter's literal wire shape rather than against this crate's own serialization agreeing with itself. Phase-4 blocker, now cleared. |
| `G12-R10` | P1 | CONFIRMED, DONE — Wave 1a | The nonce field was typed `PointParityConvention::QuadraticResidue` beside a comment saying no parity claim is made, while its committed prefixes are the compressed-point pair. Repaired by removing the claim as a separate statement rather than by correcting it: `ConfidentialFieldEncoding` no longer takes a convention, and `parity` is read off the committed prefix pair, so a field naming a convention its bytes do not use is not a value anyone can write. The nonce therefore reports `CompressedOddness`, which is what the standard compressed pair records, and the reviewed table now exhibits the disagreement the opening blockers rest on instead of flattening it — the alternative of an opaque no-claim state was rejected because it would have deleted the table's own instance of `EncodingDomainMismatch`. Boundary: reviewed target facts, the opening-feasibility reasoning that rests on the two conventions differing. Witness `the_nonce_field_claims_a_parity_convention_its_prefixes_do_not_use`, which pins the nonce's pair against the encoding registry's compressed-public-key pair and then asserts both commitment fields disagree with the nonce. The accessor doc that called the low prefix a square y, and the neighbouring test comment that said the same of all three fields, are corrected with it. |
| `G12-R11` | P1 | CONFIRMED, DONE — Wave 2 | `resource_projection` skipped every non-opcode instruction outright, so no push opcode, width prefix, or payload byte reached the `ScriptBytes` total, and opcode script bytes came from the per-primitive cost table rather than from an encoded length. The undercount ran in the one direction that matters: a program pushing a key and a digest looked admissible against a target bound it exceeds. Repaired by deriving the dimension rather than by adding the missing terms — script bytes are now `TapscriptProgram::encoded_length`, so the projection cannot omit a form the encoder knows about. `encode` and `encoded_length` share `encode_instruction`, the single place an instruction's layout is decided, so the two cannot drift; the length is measured one instruction at a time into a reused buffer, so nothing assembles the whole program to size it. The other two dimensions stay a tally over the primitives, which is what the contracts price, and all three are now stated whatever the program holds, so a program that pushes and calls nothing reports zero rather than an absent dimension. Boundary: resource prediction against target bounds. Witness `every_pushed_payload_reaches_the_projected_script_bytes`, renamed with the flip: it recomputes 99 bytes by hand for the two-literal fixture rather than asking the encoder, then checks the projection against the serialized length over eleven widths chosen either side of every push-form boundary — 0, 1, 32, 64, 75, 76, 77, 254, 255, 256, and 520. `a_resource_projection_states_each_unit_separately` moves from 2 to 100 for the same reason. The conformance package had already declined to use the dimension, measuring `program.encode(target).len()` itself in `project_resources`; that reading was the correct one and is now the projection's too. Phase-4 blocker, now cleared. |
| `G12-R12` | P1 | CONFIRMED, OPEN — Wave 1 | A pushed literal enters the abstract state as a width range, and its value survives only where it is a minimal script number. A single zero byte is not one, so the walk carries no constant, the verifying primitive branches both ways, and the success set gains a state the target cannot reach. Boundary: abstract program validation, any liveness conclusion drawn from the success set. Witness `a_pushed_zero_byte_leaves_a_success_state_the_target_cannot_reach`. Phase-4 blocker, as the guide states. |
| `G12-R13` | P2 | CONFIRMED, DONE — Wave 2 | `decode` ran its parse loop to the end of the input and applied the instruction limit only in the constructor it finally called, so the work and the allocation were unbounded by the limit. Made visible by a script whose bytes pass the limit long before an invalid byte arrives: the parser reported the invalid byte rather than the limit, which it could only have reached by continuing. Repaired by moving the bound into the loop rather than by shortening the input: the head of each iteration refuses once the sequence already holds `MAXIMUM_PROGRAM_INSTRUCTIONS` and bytes remain, so no further byte is examined, no further instruction is allocated, and the answer is the limit whatever those bytes were. The constructor keeps its own check, which is still the reachable one for a caller assembling typed instructions directly. Boundary: untrusted-script parsing. Witness `decoding_stops_at_the_work_bound_before_it_reads_further`, renamed with the flip because the assertion is now the guarantee; `a_script_of_too_many_instructions_is_refused` continues to hold the at-limit script, so the bound admits exactly ten thousand instructions and refuses the ten thousand and first. |
| `G12-R14` | P1/P2 | CONFIRMED, DONE — Wave 1a | The infrastructure arm of the shape rules checked the failure class and the two stacks and returned, leaving the resource observation unread, so a response saying the run never happened could still carry interpreter figures. Repaired in `validate_observation_shape`, which the execution and prototype paths share rather than restate, so one edit closes both: the arm now also refuses `observes_interpreter`. The refusal could not be left to the advertised-observation rule further down, because the arm returns before reaching it — an executor advertising resource observation would have kept its figures. Where the line falls is `observes_interpreter`'s to say and is not redrawn: the script's size and the initial stack's depth are the fixture's own, restated by every executor, and stay legal, which is the decided disposition of the request-derived fields. The same gap is closed for `observed_witness_sizes` in `NativeNormalizationResponse::validate_shape` — they are read out of the transaction the response may not claim to have built and are the evidence the authorization profile rests on, so refusing the profile while admitting its support was the same contradiction; `claimed_outputs` stay legal, being resolved from the claim before any mutation. Boundary: evidence-layer separation. Witness `an_infrastructure_response_may_still_carry_interpreter_figures`, extended to the prototype path and to the fixture figures that remain legal, with `a_normalization_run_that_did_not_happen_reports_no_witness_sizes` beside it, which reaches the rule through `ingest_normalization_responses` as a run does. Phase-4 blocker cleared. |
| `G12-R15` | P1/P2 | CONFIRMED, DONE — Wave 1b | The runners read records with an unbounded `readline`, spawned without a new session so the node group was not theirs to kill, carried no total deadline over the row loop, and cleaned up only on the two handshake refusals — a failure inside the loop left the child unkilled and unreaped. Repaired by stating the four guarantees once, in `scripts/executor_supervision.py`, and putting all three runners on it, so they can no longer drift from each other or from the reviewed supervisor: `read_bounded` gives `readline` a size and refuses a record that reaches the bound without ending, `SupervisedExecutor` spawns with `start_new_session` so the executor and the node it starts are one group this runner can signal, `Deadline` bounds the whole run rather than any single row, and the context manager kills the group and reaps the child on every exit path including an exception from any row. The lifecycle lane's `AdapterProcess` keeps only what is particular to it — the three arguments that distinguish one process from another, and the capability its handshake must advertise — and its two reading passes share one deadline, because two passes each just under a per-process bound are unbounded together. Two things the Rust supervisor's constants could not simply be copied into: the cleanup grace is 180 seconds rather than that supervisor's five, because these executors stop a real node and delete the datadir they own, and killing that group after five seconds would leave the datadir behind and record the creator as killed rather than exited — which is exactly the exit status the lifecycle destruction record states as evidence the process boundary held; and that exit status is now captured before the child is released, since the boundary check reads it after the block that owned the process. Boundary: harness process supervision. Verified behaviourally rather than by reading: a probe confirmed the bounded read refuses a 200 000-unit flood carrying no newline, the spawned child leads its own group, a group kill reaches a grandchild the executor started, an exception raised inside the block leaves no live child, the deadline refuses once overrun, and the exit status survives cleanup. |
| `G12-R16` | P2 | CONFIRMED, DONE — Wave 1a | The evidence-registry module stated that every requirement is unresolved and that none has been evidenced against any node, while the readiness statement records development target-native evidence and the crate carries a native evidence path. Repaired by stating the boundary the module was reaching for instead of the status it was not entitled to report: an entry is immutable, so it carries no status by design rather than by omission, and whether evidence exists is the conformance package's question — development native evidence recorded there, production evidence absent. The wording now matches `evidence.rs`, which already carried the reconciled statement. No typed value gains a status, and the registry's own contents are untouched. Boundary: documentation only. This closes the half of `SR5-13` that its DONE record claimed prematurely — the two READMEs and `evidence.rs` had been reconciled and this module had not; the history record is annotated to say so. |

### Wave 7 — CT fixtures and the conservation matrix · `task:guide11:conservation`

**Priority:** P1
**Status:** DONE

The §8 fixture language, a generic confidential-transaction
materializer, and the §8.4 matrix run against a real node. The report
role is **experimental** and the type has one variant: §8 establishes the
CT substrate and the consensus facts, and nothing about a candidate, an
opening prototype, or a normalization policy. A canonical role belongs to
whichever wave earns it.

**The matrix's discriminating power is the layer, not the class.** This
target answers every CT conservation failure with one consensus code, so
§8.3's six layers are what a row can actually distinguish.

| ID | Priority | Status | Finding |
|---|---:|---|---|
| `G11-W7-01` | P1 | DONE | **Byte-level determinism is not achievable through this materializer, and §8.2 is met one level down.** Confidential value on this target can only be produced by the node: the upstream Python framework carries no Pedersen commitment, range proof, or surjection proof. `BlindTransaction` draws every output blinding factor from `GetStrongRandBytes` and generates a fresh ephemeral nonce key, and no RPC on the path takes a seed — `rawblindrawtransaction` included. Established by reading the source and by blinding one identical raw transaction twice and comparing. So the fixture's own inputs are fully determined and reproducible, the transaction bytes are recorded per run, and the level is typed as `FixtureInputsOnly` so a later materializer that can seed its blinding reports a different value rather than quietly improving. |
| `G11-W7-02` | P1 | DONE | **Ten of eleven executed rows agree with expectations written before the target was asked.** Rows 1, 2, 4 accept; 6, 7, 8, 9, 10, 11 are consensus rejections before script; 12 accepts. The one row that reached no verdict is `G11-W7-03`. |
| `G11-W7-03` | P1 | DONE | **Several confidential inputs to a single explicit output is not constructible**, which is the exact evidence §10.2 said it required and refused to take from algebra. The node answers "Add another output to blind in order to complete the blinding": residual blinding has nowhere to go without a blinded output to absorb it. Recorded as a fixture-construction failure, which establishes no target verdict — the target was never asked — and is a finding about the candidate rather than a defect of the row. |
| `G11-W7-04` | P1 | DONE | **The §7.4 third leg lands.** The oracle predicts, and the target produces, the same commitment bytes for three observed openings. Both the asset identifier and the blinding factors are printed reversed from the order the arithmetic reads them; the convention was settled by trying all four combinations against a real observation, and only one reproduces the target's commitment. The construction-library leg stays absent and is reported as absent: the node built these transactions, no third-party library did. |
| `G11-W7-05` | P1 | DONE | **Consensus does not police a hidden confidential output.** §8.4 states the row as "closure reject *where claimed*", and the conditional is load-bearing: a hidden output makes the commitments balance, so conservation is satisfied and consensus has nothing to refuse. The row expects acceptance and names the obligation — any candidate claiming disclosure-completeness has to police it itself. Writing it as a consensus rejection would have failed the run against an expectation the target never owed. |
| `G11-W7-06` | P1 | DONE | **Block validation cannot be read for the layer.** Amount checks run in the same check queue as script checks, so a malformed range proof, a broken surjection proof, and a one-unit imbalance all reach the block layer as `mandatory-script-verify-flag-failed (unknown error)`. Classifying on that string attributes a conservation failure to an opening script that never ran, which is the exact misattribution §8.3 exists to prevent and which the first revision of the adapter's `judge` committed. The mempool names these precisely, so the mempool reason decides the layer and the block only separates unrelayable from consensus-invalid. |
| `G11-W7-07` | P2 | DONE | **Four adapter faults produced false target facts before they were caught**, all in the first run and all recorded here because each is a way a harness manufactures evidence. `getnewaddress` returns a blinded address by default, so every row that said it was explicit was confidential end to end and nothing in the result said so. The wallet's coin selection spent a row's own freshly created coin while funding the next, and `bad-txns-inputs-missingorspent` was recorded as a conservation verdict. A partially signed transaction was judged as if it were the row. And `MalformedRangeProof` serializes as `malformed_range_proof` while the adapter compared against `malformed_rangeproof`, so the branch never fired, the transaction ran undamaged, and *the target accepting a valid transaction was recorded as the target accepting a malformed range proof*. An unknown defect is now refused by name rather than falling through every branch. |

| ID | Priority | Status | Item |
|---|---:|---|---|
| `G11-W7-08` | P2 | OPEN | The conservation lane runs through `run-conservation-matrix.py`, which records and does not gate. The typed report exists and is tested; what is not yet built is the Rust executor driver, the gate, the published report asset, and the Meson target that would make this lane refusable in CI the way the primitive and prototype lanes are. Until then a conservation run is evidence a reader consults, not a gate a build enforces. |
| `G11-W7-09` | OPTIONAL | OPEN | Both public-committed rows of §8.4 remain deferred against `G11-C03`'s blockers. One of the two is executed in its explicit form (`G11-W7-03`); the public-committed form of both awaits a selected candidate. |

### Waves 8+10 — Candidate dispositions and normalization · `task:guide11:dispositions`

**Priority:** P1
**Status:** PARTIAL

The typed dispositions the evidence already dictated, and the typed
disclosure reasons the one live candidate needs. The normalization
prototype, its threat matrix, and its safety report are **not started**;
the wave was wound down after the disposition half landed.

**A deferral and a rejection are not the same decision, and §11 is both.**
Recording only one of them would have stated something the review never
established, in whichever direction it was collapsed.

| ID | Priority | Status | Finding |
|---|---:|---|---|
| `G11-W8-01` | P1 | DONE | **The direct authenticated opening is deferred as a class and refused as a shape, and the register carries both.** §11.4 rejects any candidate that loses parity, and two of the three reviewed blockers are parity facts, so the §11.3 proof outline as instantiated on the reviewed primitives is refused outright under that criterion — as is an opening checked off-script and asserted on-script, under §21's "only a host library verifies the opening". The class is deferred rather than rejected because a pattern proving the normalization or negation §11.4 asks for was not found, which is not the same as shown impossible. Cites `G11-C03`, `G11-C01`, `G11-O02`, `G11-O03`. |
| `G11-W8-02` | P1 | DONE | **The public-committed representation defers on the same three blockers.** A public committed output exists to publish an amount and opening a later relation can verify; on this target that verification has no on-script form, so an output publishing one would be publishing unauthenticated metadata, which §21 refuses by name. Conservation row 3 already carried the deferral as a typed row rather than as an absence. |
| `G11-W8-03` | P1 | DONE | **The capsule is not-applicable-while-deferred, which is a third state rather than a deferral of its own.** A capsule's contents are an opening and the fields binding it to one output; with no public committed representation there is no opening to carry and no output to bind it to, so §12.4–§12.6 have no subject. Nothing about the capsule was examined and found wanting. The normalization path supplies no subject either: its public output is explicit, and an explicit amount is already recoverable public chain data. |
| `G11-W8-04` | P1 | DONE | **Dispositions belong in conformance, not in `target-elements`.** Wave 5 put `OpeningFeasibility` in the target package correctly — whether the reviewed language *can* carry an opening is a target fact. A disposition is this project's decision taken in the light of target facts, and §6.2 puts candidate work in conformance. The blockers a disposition names are Wave 5's own typed values rather than a restatement, reached through a vocabulary spelling table because the target crate is standard-library-only and derives no serialization. |
| `G11-W8-05` | P1 | DONE | **§14.4's preferred reuse of realization's disclosure types is not available, for two independent reasons.** `realization::DisclosureReason` does express the required distinction. But conformance refuses the realization dependency by decision rather than oversight — the manifest says naming it would import a publication boundary this harness has no asset for — and two of its variants carry an `OperationId` and a `RelationId` that a conformance run does not have. Filling them would mean inventing identities, which in an evidence record is worse than a duplicated enum. The vocabulary is therefore §14.4's own, stated in conformance, with the correspondence documented. |
| `G11-W8-06` | P1 | DONE | **Every normalization disclosure is deployment policy and none is boundary arithmetic.** The relation does not need the amount in the clear: the §8.4 matrix conserved value over commitments without it. Each non-semantic disclosure states what a deployment that did not want the fact public would do instead, so that a policy choice cannot harden into an apparent necessity across waves. §9.3's rule that an explicit-only result is not disclosure-minimal is what this makes checkable. |

| ID | Priority | Status | Item |
|---|---:|---|---|
| `G11-W10-01` | P1 | DONE | **The normalization prototype is built, and the disposition it was recorded under is now true.** The claim is private → explicit + private change, the constructible variant. The owner's coin and every destination are taproot, so the wallet signs a key-path spend: both inputs carried a single 64-byte witness item on the run, which is a Schnorr signature with no trailing sighash byte — the default all-outputs non-anyone-can-pay profile §10.3 requires, per the reviewed output-committing signature profile table. The profile is read out of the witness by the adapter rather than asserted, so a narrower signature is a construction failure instead of silent evidence. The unmutated claim was accepted by a real node at the declared tip, with closure and preservation both holding. |
| `G11-W10-02` | P1 | DONE | **The closure check is implemented as exact multiset equality in both directions, and all nine §10.4 rows agree with expectations written and committed before the run.** Three rows are consensus-valid transactions the report layer alone refuses — amount changed with the change compensating, owner changed, and a hidden private output — and the target accepted all three, which is the wave's substantive finding rather than a gap. The hidden-output row is what justifies the shape of the check: the owner is paid exactly right so preservation holds, the value comes out of the blinded change so no amount is observable, and the only evidence is an output the claim never named. A subset test would have passed it and a count test would have passed a swap, which is why neither is used. The layer is recorded as report-layer and never as consensus, so `G11-W7-06`'s misattribution is not repeated. A relay-policy refusal now has its own refusal layer as well: no row expects one, so a post-signing row refused for its fee would surface as a disagreement rather than pass as a signature refusal. |
| `G11-W10-03` | P1 | DONE | **The typed §14 safety report is built.** Role is `Experimental`, as `ConservationReportRole` is: it establishes what the target does with the §10.4 matrix and nothing about a candidate being selected, which is §24's question. It carries `declassification::normalization_declassifications` — three disclosures, every one `DeploymentPolicy` and none claiming semantic necessity — the observed authorization profile, the genesis and network the run was bound to, the declared tip and the revision the node binary reported about itself, and every row's expected against observed layer. The judgement lives in the crate that owns the claim: the runner records responses verbatim and `emit-normalization-report` rebuilds the expectations from source, so a run cannot supply the answer it is checked against. |

### Wave 11 — Fresh-process lifecycle · `task:guide11:lifecycle`

**Priority:** P1
**Status:** DONE

The §13 proof that public evidence survives its creator. The boundary is
an operating-system boundary rather than a reset of state inside one
process, and the reading process is held to public chain data alone.

| ID | Priority | Status | Item |
|---|---:|---|---|
| `G11-W11-01` | P1 | DONE | §13 boundary is an OS boundary: A publishes and exits, each B is a fresh process and node, three distinct pids; wallet destroyed, chain kept. |
| `G11-W11-02` | P1 | DONE | The public record is a typed 12-field schema of chain data, with `deny_unknown_fields` and a field-NAME ban. Both tested. |
| `G11-W11-03` | P1 | DONE | 18/18 rows agree over two passes against `lifecycle::canonical_lifecycle_matrix`, never the run record. All nine canonical rows are now sent; the stale row joined them when `G11-W11-06` closed. |
| `G11-W11-04` | P1 | DONE | B locates by block locator, parses independently, rebuilds the script by bech32m; it cannot spend the owned object and spends its own funds. |
| `G11-W11-05` | P1 | DONE | Cache independence: fresh process and node per pass, both identical. The object needed locking against A's coin selection (`G11-W7-07`). |
| `G11-W11-06` | P2 | DONE | Diagnosed and repaired. The taproot digest commits the output-witness vector at its actual length, so an unblinded spend signs one digest and consensus checks another; paying to a single confidential address did not blind it, because the node declines to balance a lone blinded output against an explicit input and says so only if asked. The spend now carries two confidential outputs and refuses a silent non-blinding. The row builds and both passes answer `refused_output_spent`. |

The `G11-W11-06` diagnosis, because the finding outlives the row. The
target's taproot digest commits the transaction's output-witness vector,
and it hashes that vector at whatever length the vector has rather than
at one entry per output. A transaction carrying no witness deserializes
with that vector empty, and serializing a transaction that has any
witness grows it to one entry per output — so a wallet asked to sign an
unblinded transaction signs a digest those same bytes can never produce
once they are on the wire. The wallet reports the signing complete and
the target refuses an invalid Schnorr signature.

Proven rather than argued: the signature the wallet produced verifies
against the digest computed with the vector empty and fails against the
digest computed with it grown, while a genuinely blinded control spend
verifies the other way round and is accepted. The check is kept runnable
at `scripts/diagnose-taproot-output-witness-digest.py`.

The repair is ours and is in the adapter, so the row is closed here. The
upstream half is a defect worth filing against the target and is drafted
for the register another lane owns: the signing path builds its constant
transaction view before growing the output-witness vector, so it computes
a digest over a shape the wire form cannot have, and reports the result
complete. It is not a consensus question — every transaction that reaches
consensus with a taproot spend already carries the grown vector — so the
correction belongs on the signer's side.

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
| mocked Meson graph | `scripts/test-meson-mock.sh .` |

Focused filters supplement but never replace complete package and workspace
runs.

### 10.3 Full gate

After a completed batch:

```sh
scripts/ci.sh
meson compile -C build
meson test -C build --print-errorlogs
```

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

The Phase-2 gate is **passed**; the record is §2.11. The current gate is
Phase 3 — Elements target and foundational prototypes.

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

Current blockers are:

```text
Phase-3 work:
    the exit gate itself, checked at merge against gate:phase3:exit
```

Standing rules that survive the phase exit:

- Phases 1 and 2 remain evidence about their recorded trees;
- no compiler public complete-analysis API is frozen;
- no public realization/compiler digest is minted without a real consumer;
- target-specific fields remain forbidden in realization and compiler core;
- no stable linker or transaction ABI exists;
- no floating value or local graph handle enters semantic identity;
- no draft bound becomes deployment calibration;
- no raw report digest is treated as evidence identity without typed role and
  subject binding;
- no self-consistent model checkpoint is described as independent
  target-chain evidence;
- no architecture, model, realization, hash, build, or test success is
  described as deployment readiness;
- no current checkout is described as green without a fresh complete execution
  record.

---

## 12. Execution order · `sec:backlog:order`

Execute in this order unless reproduction changes dependencies:

```text
1. Check the Phase-3 exit gate; all three foundational questions are
   answered, with the constructor and wide-floor prototypes accepted in
   gate record 2.14 and the declassification policy in 2.4.
2. Keep target types out of compiler core; the target packages consume
   the analyzed boundary, never the reverse.
3. Mint no target hash and claim no production activation without a real
   consumer and reviewed evidence.
```

No new hash, target prototype, report field, or publication may defer a current
typed-boundary or correctness repair.

---

## 13. Adopted-draft integration queue · `sec:backlog:drafts`

Four externally authored normative drafts were accepted 2026-08-16, audited (28 findings), corrected upstream by the author, and re-adopted and
archived verbatim under [plans/drafts/](drafts/README.md): the label
calculus, the environment-kind registry, the identity-adjudication
procedure, and the interchange conventions. Integration is chartered as
its own batch; the drafts bind nothing until the integrating changes
land. The user has ruled that a superseded ADR is deleted, not kept
marked superseded: retired records belong to Git history, not the
active tree.

### 13.1 Integration tasks · `tab:backlog:draft-tasks`

| ID | Status | Task |
|---|---|---|
| `DI-001` | ACTIVE | Gap census: map each draft clause onto the present ADR-012/ADR-013 text, the ADR-016 identity rules, and the implemented labels package; classify every clause as already-implemented, divergent, or new; record the checker-engineering findings register. |
| `DI-002` | DONE | Adopt the corrected label calculus and kind registry as ADRs; delete the ADR text they retire; migrate the eight conflicting kind tokens to the registry forms corpus-wide per the user's ruling, with every citation updated in the same commit; record the hyphenated-area amendment and the local-extension register. |
| `DI-002b` | DONE | Replace the second-edition adopted drafts with the author's third-edition texts and repair every citation that dangled as a result. The third edition was verified against the audit register before the swap: all 26 findings fixed, the 7 defects among them included, and a mechanical re-audit clean — every per-document citation resolves, every mint is unique, and the kind registry's headline counts of 333 names, 349 rows, 208 kinds, 3 declared hybrids and 4 device classes derive exactly from its tables. Fourteen citations across three plan files were retargeted in the same commit as the swap, per the calculus's same-commit rule. Both adopting records were then refreshed to the editions they adopt: ADR-019 restates all seven adoption parameters and records that the checker implements the authorship warrant species only, ADR-020 names this repository as the registry's acceptee and recasts its extension register as the recorded extension set with located first-hand evidence, and a second kind-migration round settled two further tokens. |
| `DI-003` | DONE | Re-engineer the labels checker to the calculus: a single participation scanner shared by every check (mints, citations, links, hygiene, inline-code discipline), owner signatures with registered prefixes, imported and synthetic citations, anchor harvests, and the kind registry as the checker's kind vocabulary. W1 landed the participation scanner and the DI-F02 repair, W2 the seven adoption parameters as typed data with the kind vocabulary and warrant totality, and W3 the near-miss warnings, the companion attestation register, and the gate reconciliation across both records. |
| `DI-004` | DONE | Adopt the identity-adjudication procedure against ADR-016: classify every existing digest through the benefit criterion with admission records, and MIGRATE the two grandfathered recipes to domain-separated forms per the user's ruling — the architecture semantic and anchor-set hashes change under a recorded recipe migration, superseding the ADR-016 grandfather clause. Both halves delivered: the migration, and a census of the whole tree read from the owning code, recorded as six admission records and eleven stop records in the identities register. |
| `DI-005` | DONE | Interchange conventions: promoted wholesale per the user's ruling of 2026-08-19 and adopted by [ADR-022](../adr/022-interchange-conventions.md) as the standing wire-format discipline for externally consumed documents. The edition adopted is the fourth, refreshed 2026-08-18: ceiling-based acceptance with downward-closed holding, open-companion tolerant validation, never-assigned stamps rejected as checkably false claims; the fourth-edition audit's two defects and one editorial finding all fixed on resupply, delta purely additive, nothing dangling. ADR-022 carries adoption data only — the boundary against ADR-010, the restated executor stop, the labeling and identity alignment records, and the no-implementation standing — and restates no clause of the discipline. Nothing is built: normative now, unimplemented by design. |
| `DI-006` | DONE | Three-part labels: the paper's 16 two-segment labels take the area `attestation`, a division's home being the document itself; `abs` takes the registry's `abst` per the user's ruling. 16 mints and 59 sites moved in one commit; the anchor-set pin moved……, retired value reproduced; the semantic hash followed, the behavioural did not. Layer-0 kinds enforced. W-B holds the arity in one rule over every entry point: a label-intended occurrence that is not three-part fails as `malformed_label_shape`, and the realization's 20 two-segment divisions are frozen by name, not exempted as a surface. |
| `DI-008` | DONE | Promote the identity-adjudication draft: delete ADR-016 and mint [ADR-021](../adr/021-identity-adjudication.md) as the adopting record, carrying only adoption data — the single local recipe convention, the current-identities table, the concrete identity chain, the recorded separation migration verbatim, the ADR-011 amendment linkage, and three recorded divergences. Every generic mechanism, class, flow, edge, duty, evidence and rejected-alternative section is deleted rather than restated: the draft holds them and is cited at the `PLAN` prefix. Ten imported citations to the retired owner were retargeted in the same commit, seven of them to the draft's own mints and three to ADR-021's local records. |

The user's ruling of 2026-08-19 promoted the identity-adjudication
draft: ADR-016 is deleted, the draft takes priority as the discipline,
and a single local-environment convention is added to the adopting
record for what deletion would otherwise lose. ADR-021 is that record.
The convention is the domain-separated construction ADR-016 prescribed,
kept as recipe data rather than as a competing prescription — the draft
reduces every construction question to its property table and leaves
the scheme to the recipe record, so the construction is adoption data
and not an amendment. This settles the first open ruling of the gap
census, which asked exactly whether the prescription moves into recipe
records or stays in the record.

Deletion took the overlap with it. The mechanisms table, the admission
and rejection lists, the identity classes, the three flow diagrams, the
immediate-edge, migration, verification and evidence rules, the seven
rejected alternatives and the generic gate items are the draft's, and
ADR-021 restates none of them. Three divergences are recorded rather
than reconciled: the adjudication walk is not a standing checked
requirement on new proposals, the evidence and release surfaces do not
exist, and the well-founded-graph rule is vacuous with one link built.

The same ruling promoted the interchange conventions, and DI-005 closes
on ADR-022. No separate promotion identifier was minted: unlike the
identity draft, the conventions never had a repository record to retire,
so DI-005 was always the promotion itself and DI-009 stays unclaimed.
ADR-022 restates no clause and mints no overlap — the two languages, the
satisfaction judgment, the base theory, acceptance and the open
companion, the assignable fragment, the registry signature, the four
invariants, the Law, the six meta-theorems and the caveats stay the
draft's, cited at the `PLAN` prefix from outside `plans/` and in the
local parenthesized form within it. What the record adds is the boundary
against ADR-010, which owns first-party command-line streams while
ADR-022 owns externally consumed documents; the executor protocol's stop
from section 3.10 of the identities register, restated for the wire
format rather than for a digest, with the register's revisit condition
unchanged; the two alignment records the draft's preamble leaves to its
adopting corpus, on documentation labels and on identity recipes; and
the standing that adoption is normative now and builds nothing until a
real externally consumed document exists. That first consumer class is
named but not chartered.

The user's integration rulings, 2026-08-16: kind-token conflicts migrate
to the registry forms; the two grandfathered identity recipes migrate
now rather than persisting as a recorded divergence; the area grammar is
adopted with a recorded amendment admitting hyphens; per-package owner
prefixes are registered as checker data in DI-003. DI-002 landed
ADR-019 and ADR-020, deleted the retired records, migrated six of the
eight conflicts (task and res adjudicated as genuine local extensions),
and re-pinned the three identities the label rename moved: the Layer-0
anchor-set hash, the architecture semantic hash, and the
deployment-profile identity, each with the retired pin reproduced
before the new one was taken. The Layer-0 LaTeX surface keeps three
unadjudicated tokens (motto, a true collision; invest; abs), recorded
in ADR-020.

DI-002b's swap carried three consequences beyond the citation repairs.
The calculus's third edition adds a derivation authority the checker has
no notion of — a profile signature, a reserved-kind set, two warrant
rules, warrant totality and inventory discipline — which is now the
largest unmet item of its implementation gate and belongs to DI-003. The
gap census's postcondition checklist was reframed as a gate checklist,
both the calculus and the registry having moved to Gates. And this
record's own label-calculus gate was renamed from the implementation
name to the adoption name: the third edition mints the implementation
text for its own gate, and although the owner-keyed registries keep two
owners minting one text from being a duplicate, one text naming two
gates is a reader hazard the record yields on.

A second kind-migration round followed the swap. The Layer-0
verification appendix minted three walkthrough anchors under the token
ver, which the adopted registry assigns to Version and Revision;
verification is verif. The three mints in
`papers/attestation/sections/A1_verification.tex` moved to the verif
token, and the generated Layer-0 register followed in the same commit,
six sites in two files and no other occurrence anywhere in the tree.

The identity consequence was measured rather than assumed. The Layer-0
anchor-set recipe was reproduced from the pre-change tree over the
realization contract's thirty-eight cited anchors and returned the
pinned value exactly, which is what validates the reproduction; none of the three
renamed labels is in that set, because the contract cites the
walkthrough section but never these three sub-anchors. The same
recomputation after the rename returns the same value, so nothing was
re-pinned: the ceremony ran and its answer was that the anchor set did
not move. No welded witness semantic tag carries a walkthrough anchor
either, so the architecture semantic hash and the deployment-profile
identity are likewise untouched.

DI-002b closes with the refresh of the two adopting records. ADR-019
now fixes all seven adoption parameters the third edition asks for: the
owner signature, the owner partition by tree location, an empty profile
signature, an empty reserved-kind set, the designated typed-data
classes, the two citation-index designations, and the scanned-region
recognition for Markdown, Rust, and LaTeX. The profile signature and the
reserved-kind set are empty together on purpose — a reserved kind no
profile governs admits neither warrant rule, so a nonempty reserved set
under an empty profile signature would reserve kinds nobody could use —
and the record states plainly that the checker implements the authorship
warrant species alone. ADR-020 names this repository as the registry's
acceptee, recasts its register as the recorded extension set in the
edition's own terms with a located occurrence behind each of the
thirteen entries, and adjudicates the six keeps by genre: task against
the registry's exercise and code-asset readings of Task, res against
Result, trap against Pitfall, obl against Requirement, err against
Erratum, and pin against Version. Four tokens left the set because the
third edition carries them as rows of its own — pkg, q, req, and test —
with no mint moved. Postcondition and Requirement remained registry
rows, so the round-one decisions on post and req stand as taken.

Four residual items outlive the close, none of them DI-002b's to
discharge. The derivation-warrant machinery — a profile signature, a
census, standard places, inventory discipline, and the half of warrant
totality that governs reserved kinds — is unbuilt, as are the near-miss
warnings the calculus asks for, the general form of synthetic-citation
totality, and the registry's companion attestation register with its
homonym view; all of these are DI-003, and ADR-019 and ADR-020 record
them as unmet in their own terms. The identity census with admission and
stop records is DI-004. The interchange conventions stay dormant until a
consumer exists, which is DI-005. And the Layer-0 LaTeX surface keeps
three unadjudicated tokens under ADR-020 — motto, a true collision;
invest; and abs — which enter scope with that surface and not before.

DI-004 opened with the recipe migration the user ruled on, folded into
one recorded migration covering both identities. The retired values were
reproduced before anything moved, and reproduced independently of the
repository's own Rust: the architecture semantic recipe recomputed from
the committed manifest body under sorted-key compact JSON returned the pinned value, and
the Layer-0 recipe recomputed over the thirty-eight anchors harvested
straight from the realization document returned the pinned value. Both
matched the pins exactly, which is what licensed the migration to
proceed.

Each recipe then took a domain-separation prefix folding in its own
identifier, the idiom the behavioural and deployment-profile hashes
already use. The semantic hash moved to `sha256-canonical-json-v3`; the
anchor set moved to `sha256-anchor-set-v2`. Both
new values were predicted from the recipe before the code was run and
then confirmed by the generator, so the implementation was checked
against an independent computation rather than trusted. No projection or
encoding changed: the meaning identified is the same and only the
measurement moved.

The welds moved atomically with the values — the typed pin, both
generated manifests, the realization document's masthead and attached
appendix, the identities register, and the synthetic release-profile
identity, which moved because the deployment profile binds the
architecture semantic hash as a hashed input while its own recipe stayed
at version 1. The behavioural hash did not move and no version was
bumped: its recipe and body are untouched, the versioning gate keys on
it alone, and ADR-016 holds that a recipe migration does not by itself
imply a semantic version change. The grandfather clause is superseded by
the migration record in the same ADR.

DI-004 then closed with the census. It was taken from the code outward
rather than from the ADR's table — every digest, identity and hash the
tree computes was found by sweeping packages and scripts and then
reading the owning modules — and each was put to the benefit criterion:
which named consumer decides what, and in which assurance class. The
answer is six admitted identity classes and eleven recorded stops, all
in `plans/registers/identities.md`, which now opens with a census table
of one line per entry and carries admission records and stop records as
its two register bodies. Nothing new was admitted and no unclassified
digest was found, so the migration remains DI-004's only change to what
the tree computes.

The reading paid for itself twice. Eight of the eleven refusals were
practice, or enforced in code, without ever having been written down as
decided outcomes: the native conformance reports that deliberately carry
no digest and no field reserved for one, the compiler's analyzed
program whose refusal is held by a compile-time probe and an exhaustive
destructuring rather than by convention, the Phase-1 realization that
publishes an architecture binding instead of an identity, the target's
own tagged hashes and sighashes, which are the modelled protocol's
values and not identities of this corpus, the chain identifiers received
as validated typed fields, the executor's wire frames, the CI lane
records, and field-level digests in general. And the profile's raw hash
fields were undercounted: the register named the report and artifact
rows, while the schema also carries one evidence hash per substrate
dependency and the calibration's own evidence and script-bundle hashes.
All four families are recorded together, with the one binding profile
validation actually enforces — the calibration's bundle hash must equal
the released emitted-script-bundle row — separated from the presence
checks that decide nothing.

Two admission records gained precision the ADR's table does not carry.
The architecture semantic hash is the one admitted identity that crosses
a process boundary today, because the ledger's schema-13 attestation
context carries it, and each of its consumers decides by rejection: the
indexer refuses a context whose manifest hash is not the build's
expected typed hash on every construction path, profile validation
refuses a profile bound to another architecture, and the weld fails the
build on a disagreeing masthead. The document and instance identities
are locator-grade provenance, and their 128-bit truncation is admissible
only on that condition — a release-bound provenance identity could not
carry it — which the records now state rather than leave implied.

The cross-check against ADR-016 found no row the census contradicts: all
nine rows of its current-identities table hold against the code, and the
ADR was not amended. One imprecision is recorded rather than corrected —
the table calls the two publication identities UUIDs, which the
producing functions do too, though neither value sets a version or
variant field — because the register already carries that non-claim and
the ADR remains the owner of its own wording. The gap census was
repaired where the migration and the census falsified it: domain
separation and recipe identifiers are implemented rather than divergent,
the no-identity clause is implemented rather than divergent, the
implementation gate's evidence names the three items DI-004 discharged,
and the second open ruling is settled.

What outlives DI-004 belongs elsewhere. The deployment-profile hash and
the profile's raw hash fields stay pre-admission with their revisit
conditions bound to the release surface, not to this batch; the benefit
walk as a standing requirement on new proposals, rather than as the
one-time census just taken, is a change to ADR-016's admission rule that
no task yet owns; and the executor protocol's stop records the reasoning
DI-005 restated in ADR-022 when the interchange conventions were adopted
as the discipline for externally consumed documents.

DI-003 opened with W1, the participation scanner. The survey found the
judgment re-derived in six places: the Markdown scanner's own fence
loop, a second fence loop in the plan-tree link check, and the
double-backtick skip restated inline at each of the four span consumers
in `packages/labels/src/repository.rs`. All six now consume one module,
`packages/labels/src/participation.rs`, whose module documentation
carries the survey as a table — decision site, the logic it used to
carry, where it lives now — so a seventh recognizer has somewhere to be
refused. The Rust comment-and-literal segmentation and the LaTeX comment
strip moved into it as well, making it the single home of the
scanned-region recognition ADR-019 fixes for all three concrete
syntaxes.

One genuine disagreement was found and resolved rather than preserved.
The Realization harvest read its span list from the fence-aware scanner
but computed the boundaries of the generated upward-citation index by
walking the raw source, so a section heading displayed inside a fenced
block could open that index region — and, because the region stayed open
across the fence close, swallow the real body citation below it. The
anchor set derives from body citations, so the effect was a body
citation lost and the genuine index reported stale. The region walk now
reads participating lines only. The resolution is what the calculus
requires, since a displayed heading is not authored text and cannot
partition a document; it is a deliberate behavior change, recorded here
and reproduced by a test that fails with the fix reverted and passes
with it in place. No occurrence of the pattern exists in the tree today,
so no diagnostic moved.

Behavior preservation was measured, not reviewed. A capture script,
`scripts/capture-label-goldens.sh`, runs all four labels binaries over
the whole tree with the argv the CI lanes use, writing each stream and
exit status; the before and after captures are byte-identical once the
tracing wall-clock field is normalized, which includes both generated
registers reproduced whole at 17376 and 35384 bytes. The committed
registers under `plans/labels/` are unchanged. The label census is
identical across the refactor at 136 Layer-0, 329 realization, 129 ADR,
307 model, 1206 planning and 6 documentation mints with 211 imported
citations.

Two classes of site were surveyed and deliberately left alone, both
recorded in the scanner's module documentation. The forbidden-text audit
is participation-blind by design — a banned token is banned inside a
fence and inside a string literal too — so it keeps its whole-tree grep.
And several plan-tree hygiene checks read raw text: the scaffolding,
placeholder, confidence, deleted-path and machine-input markers, and the
backlog's task-status agreement walk. Their subjects are not label
occurrences, and rewiring them would change what they report, so they
are recorded as participation-blind rather than quietly converted.

W2 made the adoption parameters typed data. All seven now sit in
`packages/labels/src/adoption.rs` for comparison against the ADR-019
table, and the signature drives prefix resolution rather than restating
it: an imported token splits at its first hyphen, exact because a prefix
carries none, and becomes an owner only through the signature. The
package owners are registered there, settling the DI-003 ruling, their
prefixes derived by rule from the directory name and checked against the
census, so a new package is a reviewed registration, never a silent
owner.

The kind vocabulary is now the registry's: the 208 distinct kinds of the
draft's Convention tables and the 13 recorded extensions, committed
rather than parsed at check time so an edition swap shows every moved
token in review, and checked against their documents in each direction
so the tables cannot go stale unseen. The census found 2113 mints over
64 distinct kinds, every one governed except `motto` and `abs`, single
occurrences on the Layer-0 LaTeX surface that ADR-020 already records as
unadjudicated. That record puts the surface outside its scope, so those
two report rather than fail, and enforcement over the owners it does
govern lands with the tree already conformant.

Warrant totality is enforced ahead of its subject. The reserved set and
the profile signature are empty, so its reserved and inventory arms are
vacuous, and both are implemented and fixture-tested all the same, so
the first registered profile finds the enforcement live. Place detection
is unbuilt and reports no place, which fails closed. Two-pass staging is
structural: the adoption data load before any source is read, and
warrant totality derives from completed registries before any citation
resolves.

One defect surfaced in passing: diagnostics were emitted only when the
check failed, so no passing run could ever show a warning. Warnings now
emit on a passing check, which is what makes the two Layer-0 tokens
visible.

W3 closed the batch with the near-miss warnings, in four families and
warnings only: an interior differing from a label in case, in spacing,
or in bracketing, and a label-shaped backtick span in scanned comment
text where the acute carries the label syntax. Each family repairs the
span before testing it and every repair changes the text, so a form the
grammar accepts can never reach a warning; displayed spans stay silent
and no existing error was demoted. The live tree raises 198, all of them
the comment family — one finding rather than 198, since Rust comments
cite labels in backticks throughout.

The companion attestation register the acceptee owes is generated at
plans/labels/attestation.md, 6176 bytes, by its own target: separate
from the upstream generator because its derivation is corpus-wide where
theirs is scoped, its evidence rows carrying a mint census that answers
to every owner. Three views of one base under the recorded ordering —
the 13 first-hand extension rows with spelling, locator, sense, and
census; the status map, 349 base rows by reference and the three the
edition daggers; and Hom(C_A), 32 pairs over 15 names. The derived base
relation is welded to the registry's published headline counts, so a
parse that drifts fails rather than re-deriving quietly. Task rows in
Hom only because this corpus extends it, which is the register's point.

Both records were then walked bullet by bullet. The calculus's
implementation gate has one unmet bullet left, inventory discipline,
waiting on the decision that registers the first profile rather than on
code; two ADR-019 claims had gone stale the other way, package prefixes
and the profile note, and both now read as built. The registry's
adoption gate lost its recorded unmet item with the register and gained
one this walk found: head validation is unimplemented, the checker
validating a kind token but never the pair a head declares. The 146
environment heads under adr/ and plans/ were read by hand and all carry
catalogued pairs, so nothing rests on the gap, and ADR-020 names it.

Three residuals outlive DI-003, none its own to discharge: the
derivation-warrant machinery waits on a first inventory profile, Π and K
empty by decision; Layer-0 kind scope waits on the motto, invest, and
abs adjudication, entering with that surface; and the interchange
conventions stay dormant until a consumer exists, which is DI-005.

One of those three tokens is now settled. On the user's ruling this
corpus keeps its opening motto apart from the registry's slogan genre,
so Motto and `motto` enter X_A as its fourteenth entry, on first-hand
evidence at the title section. ADR-020 records the deviation, the
checker's kind and pair tables carry it, and the regenerated companion
register puts Hom(C_A) at 34 pairs over 16 names. Layer-0 stays
reported: `invest` and `abs` are open.

The acute code syntax now classifies as the calculus fixes it. The
harvester had paired acutes blindly, one comment line at a time, so
every acute opened: a lone one used as an apostrophe failed the file
and a pair swallowed the prose between into a bogus label. Pairing is
now settled over the logical comment region, an acute opens only where
label-shaped text follows, and one that opens nothing stays text, while
an opener whose region ends first still fails hard. No rule anywhere
banned the acute in Rust sources — the forbidden-text audit knows one
token and it is not this — so nothing needed repealing. Both syntaxes
are readable in comments during the migration: the acute participates,
the backtick spelling only warns, and the sweep of the 198 warned sites
follows. Goldens over the tree are byte-identical.

That sweep is done: all 198 warned spans are gone. 196 became live
imported citations, each carrying the registered prefix of the owner
that mints the label — PLAN for the Guide-10 rules, R13, PA, and
numbered ADRs elsewhere — and imported citations rose by exactly 196,
from 211 to 407. Conformance held 132, tapscript 23, architecture 12,
target-elements 11. Nothing was minted to make a citation resolve: no
crate mints a label, so every reference was owed elsewhere. Two sites
quote a label's spelling rather than cite it, in the guard-listing
weld that parses the document for it; both are double-backtick spans,
shown rather than meant. Groups that straddled a line were brought
onto one, a group being read within its own line. The generated
registers regenerate unchanged, so no anchor set moved.

W3 closed the batch with head validation: each head's name is now held
to the kind its label declares, over the decision records and the
authored planning tree. The recognizer finds exactly the 146 heads the
hand reading counted — 4 authored here, 142 in the archive it does not
judge — and all validate by an exact pair, so no head or label moved.
Reduction implements the sub- prefix and the catalogued modifiers only;
the other devices are unimplemented and untriggered, so a numbered or
lettered head fails until they are. The three waves: acute
classification, the 196-site sweep, head validation.

### 13.2 Checker findings so far · `tab:backlog:draft-findings`

A finding identifier is a permanent key. It names one finding for good,
so a resolved row stays here with its status rather than being deleted
into git history, and a resolved identifier is never reissued.

Two rows once shared `DI-F02`. The one still numbered so is the standing
finding the DI-001 gap census restates under that identifier; the other
was a narrower defect found while repairing it, which took the same
number by accident and now carries the next free one, `DI-F04`. Renaming
the standing finding instead would have moved an identifier the census
already cites (G11-R12).

| ID | Status | Finding |
|---|---|---|
| `DI-F01` | DONE | The plans-tree link scanner read bracketed patterns inside fenced blocks as Markdown links, so a CDDL regex in an archived draft failed as a broken link. Fenced interiors are now blanked before link scanning; the systematic single-scanner repair landed as DI-003 W1, which folded this blanking into the shared scanner. |
| `DI-F04` | DONE | The Realization harvest computed the boundaries of the generated upward-citation index from the raw source while reading its spans from the fence-aware scanner. A section heading displayed inside a fenced block therefore opened the index region, which stayed open across the fence close and swallowed the body citation below it, losing an anchor and reporting the genuine index stale. The region walk now reads participating lines only. Found and resolved by DI-003 W1; no occurrence existed in the tree, so no diagnostic moved. |
| `DI-F02` | DONE | Participation is enforced inconsistently across checks: the label scanner honors fences, the link scanner did not, and the scaffolding, placeholder, and confidence hygiene checks still scan fenced material. One participation model must feed every check. Closed by DI-003 W1, which landed exactly that single shared scanner. |
| `DI-F05` | OPTIONAL | Wave 4 observed that the DI-001 gap census and this table describe `DI-F01` through `DI-F03` in different terms, so a reader reconciling them must infer which wording is authoritative. The identifiers and statuses agree; the drift is semantic, in what each row is said to be about. Optional cleanup: state one wording as the finding and let the other cite it. No diagnostic and no gate depends on the difference. |
| `DI-F03` | DONE | The calculus's owner signatures, imported-citation prefixes, synthetic citations, anchor harvests, and acute-delimiter hard failure are only partially realized in the present checker; the gap census of DI-001 owns the exact delta. Closed by DI-003: W1 and W2 landed the owner signatures with registered prefixes, the imported and synthetic citations, the anchor harvests, and the kind registry as the checker's kind vocabulary; the acute-delimiter hard failure is enforced in `participation.rs` and asserted by `an_unclosed_opening_acute_fails_at_its_delimiter`. |

### 13.3 Toolchain engineering · `tab:backlog:toolchain-tasks`

| ID | Status | Task |
|---|---|---|
| `CI-001` | DONE | Rewrite the shell CI driver in Python with a typed lane tracker: every lane declared with status and skip reason, every lane and the whole run wall-timed, and a timing report emitted as the success output and on failure alike. Delivered as scripts/ci.py with an eleven-lane registry and the ci.sh shim; the first dataset shows the two test lanes at eighty-four percent of a twenty-minute gate. |
| `CI-002` | ACTIVE | Move test execution to the meson layer per the user's ruling: no workspace-level cargo test in the gate; each package's test groups run as individual meson-driven lanes, and per-test timing uses the nightly libtest JSON output, which the user has admitted as not affecting what the tests prove. The timing report gains per-package and per-test figures; attribution replaces the aggregate block. Contention on the shared cargo target directory is measured and the chosen serialization or partitioning recorded honestly. |
| `CI-003` | DONE | User ruling: the generated registers are archive-budget, by role not directory; labels/README.md stays prose. Combined 783755 to 724577 bytes; archive 1168518 to 1228130. |
| `CI-004` | DONE | Second half of the same ruling: closed backlog sections move to archived history. New archive-class directory plans/history/ holds the twelve completed gate records (§2.3–2.14) and the four remediated review registers (§5.2–5.5), moved verbatim with their labels; the backlog keeps a numbered stub at each origin. plans/README.md now separates superseded prose, which is deleted, from a closed record, which moves. Combined 786243 to 744305 bytes, headroom 189 to 42127. |
| `CI-005` | TODO | The workspace test lane costs too much for what most changes need (user, 2026-08-19). Two halves. First, honor the cadence the backlog already states: §10.1 exempts documentation-only changes from the Rust lane, yet the working ritual has been running the full workspace suite before every commit regardless of content — the promotion batch itself proved the corpus oracle for documentation edits is the label/plans/forbidden-text checker run plus register regeneration, with the workspace suite contributing nothing to those edits. The per-change cadence becomes content-scoped: documentation-only tranches run the checker oracle alone, Rust tranches run the focused package lanes of §10.2 plus clippy, and the full workspace lane runs once per batch before merge rather than per commit. Second, measure where the workspace wall time actually goes with the per-test timing CI-002 already charters, and split or serialize the worst suites so the full lane itself gets cheaper. Third, the user's deeper ruling: wall time is a first-class concept — every lane, test, and gate reports its duration or that silence is itself a defect to report; each recurring action carries an expected-duration budget beside its definition, and exceeding budget is a finding. |
| `CI-006` | TODO | Wave-1a worker finding, 2026-08-20: every workspace manifest names its lint groups in kebab case — nine names of the pattern `lints.rust.deprecated-safe` through `unsafe-code` per manifest — and cargo warns for each that the name will not work in a future edition, roughly ninety warnings on every build. A mechanical snake-case sweep over the workspace root and the package manifests, verified by one clippy lane running warning-free of that class. Alongside it, one recorded decision: `PointParityConvention::ImpliedEvenY` documents the x-only convention but became formally unconstructible when `G12-R10` made the convention derive from the prefix pair, so either a future encoding earns it or the variant is removed. |
| `CI-007` | TODO | Wave-1b worker findings, 2026-08-20, two related gaps in the executor test surface. First, `scripts/test-elements-native-executor.sh` is dead: it declares schema 2 and sends a request member the adapter refuses, so it has been unrunnable since revision 3 landed, hidden behind its live-node skip. The Wave-1b lane deliberately did not blind-rewrite it against a node it could not run; the rewrite to revision 4 belongs to whichever wave next holds a live node. Second, no node-free Python test lane exists: `ci.py` declares eleven lanes and none runs Python, so pure string logic such as the parenthesis-safe script-error parser is verified only in a lane worker's recomputation and never re-checked by CI. Creating the lane changes the `ci.py` registry, which is the orchestrator's to decide; the lane should carry the exhaustive mapped-message check. |

## 14. Backlog hygiene · `sec:backlog:hygiene`

### 13.1 Adding work · `rule:backlog:add`

A new task states:

- phase or lane;
- priority and status;
- dependencies;
- concrete output;
- assurance class;
- affected files and packages;
- focused exit test;
- verification command;
- identity and schema impact;
- dependency impact.

A new digest additionally satisfies
(`req:identity:admission-record`).

### 13.2 Splitting work · `rule:backlog:split`

Split a task when it:

- crosses semantic, compiler, linker, target, transaction, evidence, or release
  ownership;
- mixes exact correctness with numerical diagnostics;
- mixes dependency adoption with algorithm acceptance;
- combines independent security consequences;
- contains one part that can complete while another remains research-blocked.

### 13.3 Dropping or parking work · `rule:backlog:drop`

A dropped or parked task records:

- why it is unnecessary or premature;
- supporting evidence;
- activation condition;
- replacement, if any;
- identity and release consequences.

“No present consumer” is sufficient reason to park an identity, report digest,
dependency, or publication.

### 13.4 Retention · `rule:backlog:retention`

After a phase or remediation series:

- move durable evidence into the owning phase card, ADR, decision, research
  result, release record, or annotated tag;
- retain permanent task IDs in compact tables;
- delete implementation diaries from the active backlog;
- rely on Git history rather than creating a second archive under `plans/`;
- keep only current and immediately preparatory work detailed here.

---

## 15. One-line backlog · `rem:backlog:one-line`

> Check the Phase-3 exit gate now that all three foundational questions are answered — the STATE-constructor and wide-floor prototypes accepted in §2.14, the declassification policy selected in §2.4 — then begin the first complete compiler-to-target operation without reopening the policy, minting speculative identities, or claiming production activation.
