# Tripod Implementation Backlog

> **Status:** ACTIVE
> **Current gate:** Phase 2 — target-independent compiler analysis
> **Current condition:** Phase 1 remains historical tagged evidence. Compiler input binding, scoped relation/expression analysis, checked constant folding, source and constructibility analysis, disclosure analysis, representation lifecycle analysis, and exact proof-plan enumeration are implemented internally. The current static review found two proof-planning correctness defects, T6 and T7, that must close before placement, coverage, or pilot analysis can rely on the feasible-plan set. Documentation findings T8 and T9 must close before the Phase-2 documentation gate.
> **Next gate:** Phase 3 — Elements target and foundational prototypes
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

### 1.2 Status vocabulary · `tbl:backlog:status`

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

### 1.3 Priority vocabulary · `tbl:backlog:priority`

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
(`[ADR016-rule:identity:admission]`) and name:

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

### 2.1 Latest static review · `tbl:backlog:review-basis`

The current review was performed statically over the supplied subtree report:

```text
tree:
   

selected files:
    343

selected bytes:
    3,645,852

submodules reported:
    none

tracked symbolic links reported:
    none
```

The supplied report excluded:

```text
Cargo.lock
LICENSE-CODE
LICENSE-DOCS
archive/
docs/attestation/human.md
packages/document-stamps/
packages/execwrap/
packages/flatten-latex-main/
```

No Cargo, Meson, TeX, target-execution, advisory, or reproducibility command was
run as part of this review.

Therefore:

- T6–T9 are static findings until reproduced;
- this review makes no current green-build claim;
- the excluded packages received no content review in this pass;
- lockfile checksums and resolved features were not independently verified;
- advisory status was not checked;
- licence compatibility was not independently checked;
- historical gate records remain historical evidence only.

### 2.2 Earlier review basis

The repository retains two earlier reviewed trees as historical context:

```text
initial reviewed tree:
   

follow-up reviewed tree:
   
```

Those reviews and their findings are evidence about their exact trees. They are
not current-checkout execution evidence.

### 2.3 Historical Phase-1 gate · `gate:backlog:phase1`

The repository records the Phase-1 evidence tag:

```text
phase1-realization-foundation-v1
```

The tag is the immutable evidence record for its exact commit. Its existence
does not establish that the current checkout passes.

### 2.4 Historical T1–T5 remediation gate · `gate:backlog:t1-t4-remediation`

The repository records a completed remediation batch for T1–T5. Its recorded
result was:

- `scripts/ci.sh`: every available lane passed;
- `cargo-audit`: skipped because unavailable, so the run was partial-green;
- real Meson compile: passed;
- Meson tests: passed;
- document byte reproducibility: deferred;
- final clean-tree check: empty.

This is historical evidence for the remediation tree, not a current-tree gate.

### 2.5 Historical compiler-input gate · `gate:backlog:p2-004`

The repository records a completed P2-004 compiler-input batch:

- typed owner revalidation;
- explicit canonical compiler scope;
- strict analysis policy;
- immutable bound compiler input;
- focused public-API tests;
- available CI and Meson lanes passed;
- advisory lane skipped where unavailable;
- document byte reproducibility deferred because paper inputs were unchanged;
- final clean-tree check empty.

### 2.6 Historical Guide-2 gate · `gate:backlog:guide2`

The repository records a completed Guide-2 batch for:

```text
C1-005
P2-005
P2-006
```

The batch covered:

- scoped relation DAG;
- scoped expression DAG;
- predicate binding;
- checked constant folding;
- independent folding oracle;
- declaration-permutation determinism;
- dependency review for the compiler’s Petgraph edge.

The latest static review did not find a defect in these components.

### 2.7 Historical Guide-3 gate · `gate:backlog:guide3`

The repository records a completed Guide-3 batch for:

```text
C1-008
P2-007
P2-008
P2-009
```

The batch covered:

- relation-obligation classification;
- exact proof-plan enumeration;
- source requirements;
- constructibility;
- disclosure;
- representation lifecycle;
- independent exhaustive planning oracle.

The latest static review found T6 and T7 in this area. The historical gate
remains evidence that its recorded commands passed; it is not evidence that the
tested assertions were complete. P2-007 and C1-008 are therefore reopened as
blocked correctness work.

---

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

### 3.1 Implemented areas · `tbl:backlog:implemented`

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
| `compiler` proof search | Exact deterministic feasible-plan enumeration, but correctness is reopened by T6 and T7 |
| `artifacts` | Generated-publication derivation, writer/checker, realization-document weld |
| `labels` | Owner-aware Markdown/Rust label graph, census, plan checks, register rendering |
| `cli-common` | ADR-010 streams, diagnostics, checker report/stamp publication, batch publication |
| Meson | Explicit source census, stamp-backed checks, mocked document graph |
| Security policy | Public-data interfaces and external execution-environment boundary |
| Path policy | Central tracked-mode audit, lexical output roles, explicit host-filesystem non-claims |

### 3.2 Current published identities · `tbl:backlog:identities`

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

### 3.3 Not implemented · `tbl:backlog:not-implemented`

```text
compiler execution-case placement
compiler target-independent layout requirements
compiler relation-indexed coverage requirements
complete pilot analyzed-program value
public complete compiler-analysis API
compiler-plan identity
target capability adapter backed by a real target package

tripod-target-elements
tripod-tapscript
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
Compiler exact planning:            implemented but correctness reopened by T6/T7
Compiler placement/layout/coverage: absent
Complete analyzed pilots:           absent
Target/backend/linker/ABI:         absent
Independent deployment evidence:   absent
Production deployment:             absent
Current-tree complete gate:        not established by the latest review
```

Current packages are public-data tools. They do not legitimately accept private
keys, seed material, signing nonces, blinding factors, private openings,
credentials, or production authority.

---

## 4. Compact historical record · `sec:backlog:history`

### 4.1 Completed phases · `tbl:backlog:completed-phases`

| Phase | Status | Durable record |
|---|---|---|
| Phase 0 | HISTORICAL | the recorded baseline and identities on [the Phase-0 card](phases/00-baseline.md) |
| Phase 1 | HISTORICAL | the completion evidence on [the Phase-1 card](phases/01-realization.md), and the gate record in [the backlog archive](history/backlog-history.md) §2.3 |
| Phase 2 | Active | current backlog and Phase-2 card |

### 4.2 Historical finding families · `tbl:backlog:historical-findings`

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

### 4.3 Identity work · `tbl:backlog:identity-work`

| ID | Status | Result |
|---|---|---|
| `I1-001` | DONE | ADR-016 adopted |
| `I1-002` | DONE | Current identity inventory |
| `I1-003` | DONE | Future immediate-edge identity DAG |
| `I1-004` | PARKED | Typed evidence envelopes; waits for a persistent report consumer |
| `I1-005` | BLOCKED | Deployment-profile migration; waits for bundle, ABI, and evidence types |
| `I1-006` | BLOCKED | Release root; waits for the release package |

The identity freeze is lifted only under ADR-016 admission. Phase 2 still mints
no public realization or compiler digest without a real consumer.

---

## 5. Current static-review findings · `sec:backlog:findings`

### 5.1 Summary · `tbl:backlog:findings-current`

| ID | Priority | Status | Finding |
|---|---:|---|---|
| `T6` | P1 | TODO | External-evidence relations bypass abstract capability validation during proof planning. |
| `T7` | P1 | TODO | Representation-relation proof choices are not constrained by the selected representation mode. |
| `T8` | P2 | TODO | Current-phase declarations disagree and the plan checker does not detect all copies. |
| `T9` | P2 | TODO | Compiler status documentation materially understates implemented internal analysis. |

T6 and T7 block P2-007 acceptance, C1-008 acceptance, P2-010, P2-011, and
P2-012.

T8 and T9 do not change semantics, but they block a trustworthy Phase-2
documentation and planning gate.

### T6 — Validate capabilities for external-evidence obligations · `task:review:external-evidence-capability`

**Priority:** P1
**Status:** TODO
**Owners:** `compiler::proof`, `compiler::source`, capability planning
**Blocks:** C1-008, P2-007, P2-010 through P2-013
**Identity impact:** none; no compiler-plan identity exists
**Schema impact:** internal obligation/candidate representation may change
**Dependency impact:** none

#### Static basis

`classify_obligations` classifies `SubstrateConservation` as:

```text
ExternalEvidence
```

rather than:

```text
ProofRequired
```

Only proof-required obligations enter the proof-variable search. Candidate
capabilities are accumulated only from selected proof variables.

The source layer correctly maps:

```text
ProofKind::SubstrateConservation
    → WholeTransactionValueConservation
```

but that mapping is not reached by production plan enumeration for the
external-evidence relation.

The candidate therefore retains:

```text
external_evidence:
    SubstrateConservation
```

without necessarily retaining:

```text
required_capabilities:
    WholeTransactionValueConservation
```

`CapabilityView::Available` cannot reject a target view that lacks the omitted
capability.

The independent test oracle repeats the same classification shape, so equality
between production and oracle does not detect this omission.

#### Required implementation

Preserve the separation:

```text
capability support
≠
evidence completion
```

A relation may remain externally evidenced while still requiring a target
capability.

Acceptable implementation shape:

```rust
ExternalEvidence {
    requirement: ExternalEvidenceRequirement,
    required_capabilities: BTreeSet<RequiredCapability>,
}
```

or an equivalent typed representation.

Planning must:

1. retain the realization-approved proof class for the external relation;
2. derive its required abstract capabilities;
3. apply the capability view;
4. add those capabilities to each feasible candidate;
5. retain the external evidence requirement unresolved;
6. avoid converting the premise into a runtime `Passed` relation.

No external evidence digest or report identity is added in this repair.

#### Required tests

For compact ASH and live transfer:

- a capability view containing every pilot capability except
  `WholeTransactionValueConservation` produces `NoFeasibleProofPlan`;
- adding that capability permits otherwise feasible plans;
- every candidate carrying substrate-conservation evidence also carries the
  whole-transaction-conservation capability;
- external evidence remains unresolved after capability validation;
- sponsor values remain erased;
- source requirements do not invent an exact sponsor amount.

The independent oracle must derive the capability requirement independently of
the production classification helper.

#### Verification

```sh
cargo test --locked -p tripod-compiler proof
cargo test --locked -p tripod-compiler oracle
cargo test --locked -p tripod-realization
cargo test --workspace --locked
```

#### Exit

- [ ] finding reproduced or disproved;
- [ ] external-evidence capabilities are retained;
- [ ] capability absence fails closed;
- [ ] evidence completion remains a separate claim;
- [ ] both pilots have focused regressions;
- [ ] independent oracle does not repeat the production omission;
- [ ] full batch gate passes and the tree is clean.

### T7 — Constrain every representation-sensitive proof choice · `task:review:representation-proof-compatibility`

**Priority:** P1
**Status:** TODO
**Owners:** `compiler::proof`, `compiler::lifecycle`, `compiler::source`
**Blocks:** C1-008, P2-007, P2-010 through P2-013
**Identity impact:** none
**Schema impact:** internal proof-plan feasibility rules may change
**Dependency impact:** none

#### Static basis

Live transfer declares proof alternatives for both:

```text
amount-conservation relation:
    public arithmetic
    confidential conservation

representation relation:
    public arithmetic
    confidential conservation
```

The exact search creates a representation choice:

```text
explicit
private committed
```

`representation_conflict` checks proof/mode compatibility only for
`AmountConservation`.

It does not check the selected proof carried by the `Representation` relation
itself.

A candidate can therefore contain:

```text
selected mode:
    private committed

conservation proof:
    confidential conservation

representation-relation proof:
    public arithmetic
```

The conservation pairing is valid, but the representation relation’s own proof
is incompatible with the selected mode.

The reverse contradictory pairing can also survive.

Additionally, a `Representation` relation currently derives a family-census
operand rather than an exact amount operand. Selecting `PublicArithmetic` for
that relation can therefore add an exact-public-arithmetic capability without a
matching exact amount source.

#### Required implementation

Apply proof/representation compatibility to every relation whose selected proof
depends on the chosen representation.

At minimum:

```text
AmountConservation touching the selected object
Representation naming the selected object
```

must both pass:

```rust
proof_supports_representation(proof, mode)
```

Prefer a typed derivation from relation operands or relation metadata over an
ever-growing ad hoc match.

Also decide whether the representation relation should select an arithmetic
proof at all. If its semantic job is only:

```text
selected mode ∈ realization-approved modes
```

then it may be more accurate to classify it statically after mode selection,
leaving the value proof on conservation.

Whichever design is selected must preserve:

- realization-approved alternatives only;
- exact source/capability consistency;
- no silent proof invention;
- no weakening of the selected representation;
- deterministic candidate ordering.

#### Required tests

For every returned live-transfer candidate:

- private committed has no public-arithmetic proof on any
  representation-sensitive relation;
- explicit has no confidential-only proof on a relation that explicit values
  cannot discharge;
- conservation relation and representation relation are both inspected;
- every selected capability has a matching source or a documented
  source-independent role;
- invalid pairings are absent from production and independent-oracle sets;
- equal typed inputs still produce equal candidate sets.

#### Verification

```sh
cargo test --locked -p tripod-compiler lifecycle
cargo test --locked -p tripod-compiler proof
cargo test --locked -p tripod-compiler oracle
cargo test --workspace --locked
```

#### Exit

- [ ] finding reproduced or disproved;
- [ ] all representation-sensitive proof choices are constrained;
- [ ] proof capabilities and source requirements agree;
- [ ] invalid public/private pairings are absent;
- [ ] independent oracle covers both relation classes;
- [ ] full batch gate passes and the tree is clean.

### T8 — Weld every current-phase declaration · `task:review:phase-declaration-drift`

**Priority:** P2
**Status:** TODO
**Owners:** `plans`, `labels::plans`
**Blocks:** Phase-2 documentation gate
**Identity impact:** none
**Schema impact:** plan-check report may gain a focused phase diagnostic
**Dependency impact:** none

#### Static basis

The supplied tree contains contradictory current-phase declarations:

```text
plans/README.md:
    Phase 1 - typed realization foundation

plans/backlog.md:
    Phase 2 - target-independent compiler analysis

plans/roadmap.md:
    Phase 2 - target-independent compiler analysis

plans/phases/02-compiler.md:
    Active
```

The plan checker compares only:

```text
backlog current gate
↔
exactly one active numbered phase card
```

It does not validate the current-phase declarations in:

- `plans/README.md`;
- `plans/roadmap.md`.

This contradicts the planning gate’s own requirement that current phase
declarations agree.

#### Required implementation

Prefer one canonical current-phase declaration and make other planning entry
points link to it.

If repetition remains, extend the checker to compare:

```text
backlog current gate
roadmap current phase
plans README current phase
exactly one active phase card
```

The checker should emit a focused diagnostic naming:

- stale path;
- declared phase;
- expected phase.

Avoid a broad free-form prose parser. A small fixed declaration shape is
sufficient.

#### Required tests

- all four declarations agree;
- plans README stale;
- roadmap stale;
- backlog stale;
- two active cards;
- no active card;
- malformed current-phase declaration;
- phase values remain deterministic under file traversal order.

#### Verification

```sh
cargo test --locked -p tripod-labels plans
scripts/check-plans.sh
meson compile -C build lint
```

#### Exit

- [ ] `plans/README.md` names Phase 2 or ceases to duplicate the value;
- [ ] roadmap, backlog, and active card agree;
- [ ] the checker detects each stale copy;
- [ ] focused tests pass;
- [ ] documentation census and labels pass;
- [ ] the tree is clean.

### T9 — Refresh compiler status documentation · `task:review:compiler-status-drift`

**Priority:** P2
**Status:** TODO
**Owners:** `compiler` Rustdoc, compiler package contract, Phase-2 planning
**Blocks:** Phase-2 documentation gate
**Identity impact:** none
**Schema impact:** none
**Dependency impact:** none

#### Static basis

Current source implements internally:

- input binding;
- relation and expression DAGs;
- checked constant folding;
- proof-obligation classification;
- exact feasible-plan enumeration;
- source requirements;
- constructibility;
- disclosure;
- representation lifecycle.

`packages/compiler/README.md` and the Phase-2 card substantially reflect this.

The following remain stale:

```text
packages/compiler/src/lib.rs
plans/packages/compiler.md
```

They still say that input binding or major implemented analysis stages are not
implemented.

This backlog previously repeated the same stale claim in several sections.
This consolidated revision corrects the backlog copy, but the package Rustdoc
and package contract remain to be updated.

#### Required implementation

Update `packages/compiler/src/lib.rs` to distinguish:

```text
implemented internal analysis stages
```

from:

```text
not yet exposed complete analyzed program
```

Update `plans/packages/compiler.md`:

- status header;
- implemented milestones;
- current open work;
- identity statement;
- pilot and exit-gate status.

Keep these statements explicit:

- analysis structures remain crate-private;
- no complete analyzed program is exposed;
- placement, layout, coverage, and complete pilot analysis remain absent;
- no compiler-plan digest exists;
- typed comparison remains the boundary.

Do not add volatile test counts or current hash values.

#### Required tests and checks

- Rustdoc and public-API tests continue to pass;
- package plan labels remain valid;
- current phase declarations agree after T8;
- documentation checker passes;
- no package documentation claims target or deployment readiness.

#### Verification

```sh
cargo test --locked -p tripod-compiler
cargo doc --locked -p tripod-compiler --no-deps
scripts/check-plans.sh
meson compile -C build lint
```

#### Exit

- [ ] crate Rustdoc matches implemented source;
- [ ] package contract matches implemented source;
- [ ] absent placement/layout/coverage work remains explicit;
- [ ] no new identity is advertised;
- [ ] documentation checks pass;
- [ ] the tree is clean.

---

## 6. Phase-2 implementation queue · `sec:backlog:phase2`

### 6.1 Summary · `tbl:backlog:phase2`

| ID | Priority | Status | Deliverable |
|---|---:|---|---|
| `P2-001` | P1 | DONE | Immutable, canonical, ownership-validated realization boundary |
| `P2-002` | P2 | DONE | Petgraph dependency and lockfile review |
| `P2-003` | P1 | DONE | Compiler crate boundary and typed error root |
| `P2-004` | P1 | DONE | Bind architecture, realization, policy, and explicit scope |
| `P2-005` | P1 | DONE | Canonical compiler relation and expression DAGs |
| `P2-006` | P1 | DONE | Checked constant folding |
| `P2-007` | P1 | BLOCKED | Exact proof planning; acceptance reopened by T6 and T7 |
| `P2-008` | P1 | DONE | Disclosure, source, and constructibility analysis |
| `P2-009` | P1 | DONE | Representation lifecycle analysis |
| `P2-010` | P1 | BLOCKED | Execution-case placement and layout requirements |
| `P2-011` | P1 | BLOCKED | Relation-indexed coverage requirements |
| `P2-012` | P1 | BLOCKED | Compact-ASH and live-transfer analyzed pilots |
| `P2-013` | Gate | BLOCKED | Complete Phase-2 evidence and exit |

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
P2-008  source, constructibility, and disclosure analysis
P2-009  representation lifecycle
```

No complete analyzed-program public API is exposed yet. This is deliberate:
partial analyses must not be mistaken for a completed compiler result.

### P2-007 — Repair and reaccept exact proof planning · `task:phase2:proof-planning`

**Priority:** P1
**Status:** BLOCKED
**Depends on:** T6, T7, P2-005, P2-006
**Blocks:** P2-010 through P2-013

The exact search implementation exists, but its feasible set is not accepted
until T6 and T7 close.

Reacceptance requires:

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
**Status:** BLOCKED
**Depends on:** accepted P2-007, P2-008, P2-009, C1-009
**Blocks:** P2-011 and P2-012

Classify relations as:

- local;
- transaction-global;
- conditional;
- deliberately duplicated.

Model finite execution cases where applicable:

```text
sponsorless / sponsored
explicit / confidential
continuing / terminal
empty / nonempty
pre-maturity / conversion / post-maturity
```

Every active required case must have a possible semantic carrier.

Compiler core does not assign concrete tapscript input indexes, stack
positions, tapleaves, or transaction slots.

Required output includes:

- stable placement requirement IDs;
- relation provenance;
- active case set;
- eligible semantic carriers;
- required fact/source sets;
- deliberate duplication policy;
- typed failure for no possible carrier;
- complexity limits and diagnostic search report.

No target bytes or concrete target positions enter this stage.

### P2-011 — Derive relation-indexed coverage requirements · `task:phase2:coverage`

**Priority:** P1
**Status:** BLOCKED
**Depends on:** P2-010, C1-010, C1-013
**Blocks:** P2-012

Require exact equality among:

```text
realization relation scope
compiler relation scope
placement requirement scope
coverage requirement scope
```

Every relation receives:

- active accepting case;
- focused rejecting mutation;
- activation requirements;
- representation cases;
- carrier requirement;
- accepted semantic projection checks.

Conditional relations additionally receive:

- inactive valid case;
- active valid case;
- active invalid case.

A broad operation test does not substitute for relation coverage.

### P2-012 — Analyze both pilots end to end · `task:phase2:pilots`

**Priority:** P1
**Status:** BLOCKED
**Depends on:** T6, T7, P2-005 through P2-011
**Blocks:** P2-013

#### Compact ASH

Analyze:

- cardinality and recognition;
- permissionless authorization;
- ownerless `U` conservation;
- exact canonical delta;
- sponsor multiplicity and role isolation;
- substrate-conservation capability and evidence premise;
- no roots;
- transition-certificate projection only;
- public constructibility;
- explicit/public representation;
- compact and clear lifecycle;
- placement and coverage in sponsorless and sponsored cases.

#### Live transfer

Analyze:

- input/output cardinality;
- live receipt recognition and closure;
- all-owner authorization;
- exact aggregate `U` conservation alternatives;
- explicit closed `U`;
- destination-family closure;
- sponsor multiplicity and role isolation;
- substrate-conservation capability and evidence premise;
- no roots;
- transition-certificate projection only;
- explicit/private-committed alternatives;
- transfer, burn, and redemption lifecycle;
- representation-sensitive proof compatibility;
- placement and coverage in explicit/private and sponsorless/sponsored cases.

Repeated analysis from equal typed inputs must produce equal stable projections.

The complete analyzed value must not claim deployment lifecycle completeness:
the pilots retain future target and operation obligations.

### P2-013 — Phase-2 evidence and exit · `gate:backlog:phase2`

**Priority:** Gate
**Status:** BLOCKED
**Depends on:** T6–T9, P2-004 through P2-012, C1-005, C1-008,
C1-009, C1-010, C1-013

Phase 2 exits only when:

- T6–T9 are closed or formally refuted;
- compiler input binding remains complete;
- relation census exactly equals realization scope;
- every relation has proof, source, constructibility, lifecycle, placement,
  layout, target-requirement, and coverage information;
- external-evidence capabilities fail closed when unavailable;
- proof/representation compatibility is complete;
- unsupported capabilities fail without semantic weakening;
- no concrete target detail enters compiler core;
- compact ASH and live transfer analyze deterministically;
- independent small-instance oracles agree;
- current required repository gates pass;
- documentation status and current-phase declarations agree;
- the final tree is clean.

Phase completion does not itself justify a persistent compiler digest.

---

## 7. Immediate algorithm preparation · `sec:backlog:algorithms`

### 7.1 Current status · `tbl:backlog:algorithms`

| ID | Status | Deliverable |
|---|---|---|
| `C1-001` | DONE | Compiler/linker/mathematics/solver research notes |
| `C1-002` | DONE | Direct Petgraph decision |
| `C1-003` | DONE | Exact/certified mathematics decision |
| `C1-004` | DONE | Petgraph dependency review |
| `C1-005` | DONE | Canonical direct-Petgraph compiler graph prototype |
| `C1-006` | PARKED | Exact keyed linear systems until a consumer exists |
| `C1-007` | PARKED | Certified numerical analysis until a consumer exists |
| `C1-008` | BLOCKED | Exact proof-plan search; reaccept after T6/T7 |
| `C1-009` | TODO | Execution-case-aware placement |
| `C1-010` | TODO | Typed symbol resolution and SCC policy |
| `C1-011` | BLOCKED | Structured relocation; linker phase |
| `C1-012` | BLOCKED | Deterministic bounded-depth target tree; linker phase |
| `C1-013` | TODO | Independent small-instance placement and coverage oracles |
| `C1-014` | BLOCKED | Preparation review and Phase-2 handoff |

### 7.2 C1-008 — Proof-plan search reacceptance

The current search remains exact in shape but is not accepted until:

- external-evidence capability requirements are retained;
- representation-sensitive proofs are checked against selected modes;
- candidate capabilities and sources agree;
- the independent oracle derives those rules independently;
- infeasibility and complexity exhaustion remain distinct;
- no partial result is returned.

### 7.3 C1-009 — Execution-case-aware placement

Enumerate required execution cases and eligible carriers.

For relation \(r\):

\[
\operatorname{requiredCases}(r)
\subseteq
\bigcup_{c\text{ carries }r}\operatorname{executedCases}(c)
\]

A relation carried somewhere but absent from one active case remains uncovered.

Compare production search with exhaustive carrier-subset enumeration on small
instances.

Required adversarial cases:

- relation available only in sponsorless case;
- relation available only in sponsored case;
- global relation assigned only to local carrier;
- unconditional relation assigned only to optional carrier;
- representation-specific carrier missing;
- duplicated carrier with conflicting fact requirements;
- equal-cost carrier sets requiring stable-key tie-breaking;
- placement-state budget exhaustion.

### 7.4 C1-010 — Typed symbols and SCC policy

Use two-pass typed resolution:

1. complete definition census;
2. complete reference resolution.

Normalize SCC members by stable key.

SCC membership does not authorize a semantic cycle. Every accepted cycle
requires an explicit typed resolution strategy.

This work prepares linker policy but must not pull linker or target types into
compiler core.

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

---

## 8. Dependency posture · `sec:backlog:dependencies`

### 8.1 Current and deferred dependencies · `tbl:backlog:dependencies`

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
    optional domain-separated commitment admitted under ADR-016
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

### 10.2 Focused commands · `tbl:backlog:focused-tests`

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

The Phase-2 gate is **not passed**.

Current blockers are:

```text
Compiler correctness:
    T6 external-evidence capability validation
    T7 complete proof/representation compatibility
    P2-007 / C1-008 reacceptance after T6 and T7

Compiler remaining work:
    C1-009 / P2-010 execution-case placement and layout
    C1-010 typed symbol/SCC policy
    C1-013 independent placement and coverage oracles
    P2-011 relation-indexed coverage
    P2-012 complete analyzed pilots

Documentation:
    T8 current-phase declaration weld
    T9 compiler implementation-status refresh

Evidence:
    no fresh complete current-tree gate record after T6–T9
```

Until (`gate:backlog:phase2`) passes:

- Phase 1 remains historical tagged evidence;
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
1. Reproduce T6 with a capability view lacking whole-transaction conservation.
2. Repair T6 while keeping capability support and evidence completion separate.
3. Reproduce T7 by enumerating representation-relation proof/mode pairings.
4. Repair T7 and independently verify the complete feasible set.
5. Reaccept C1-008 and P2-007.
6. Close T8 by selecting or checking one canonical current-phase declaration.
7. Close T9 by updating compiler Rustdoc and package-plan status.
8. Implement C1-009 and P2-010 placement/layout requirements.
9. Implement C1-010 and P2-011 coverage requirements.
10. Implement C1-013 independent placement and coverage oracles.
11. Analyze compact ASH and live transfer end to end under P2-012.
12. Run and record the complete Phase-2 gate.
13. Begin Phase-3 target work only after Phase-2 exit.
```

No new hash, target prototype, report field, or publication may defer a current
typed-boundary or correctness repair.

---

## 13. Backlog hygiene · `sec:backlog:hygiene`

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

A new digest additionally satisfies ADR-016 admission.

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

## 14. One-line backlog · `rem:backlog:one-line`

> Repair the two reopened proof-planning correctness boundaries and the two documentation-weld defects; then complete Phase-2 placement, layout, relation-indexed coverage, and deterministic analyzed pilots without adding target detail, speculative identities, or ambiguous evidence.
