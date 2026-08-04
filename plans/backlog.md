# Tripod Implementation Backlog

> **Status:** ACTIVE
> **Current gate:** Phase 2 — target-independent compiler analysis
> **Current condition:** Phase 1 is historical tagged evidence. The compiler crate exists only as a package boundary and typed error root; input binding and compiler analysis are not implemented. A static review of tree identified four open findings that require reproduction and repair. The next correctness work is T1–T4; the next compiler task remains P2-004.
> **Next gate:** Phase 3 — Elements target and foundational prototypes
> **Authority:** Current execution queue only. The specification, the realization document, typed architecture, implemented ADRs, accepted decisions, package contracts, phase cards, and accepted research results take precedence.

This file contains only:

- the current repository and readiness state;
- compact historical milestones and closed task families;
- the current static-review findings;
- the active Phase-2 compiler queue;
- immediate algorithm preparation;
- verification and clean-tree requirements.

Detailed historical implementation narratives belong to Git history, annotated
tags, phase cards, ADRs, and accepted decisions—not to the active backlog.

---

## 1. Backlog contract · `sec:backlog:contract`

### 1.1 Authority · `rule:backlog:authority`

This backlog sequences work. It is not protocol, realization, compiler, target,
ABI, evidence, deployment, or release input.

When this file disagrees with an owning artifact:

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
| **TODO** | Ready when named dependencies are complete. |
| **IN PROGRESS** | Actively being implemented, reviewed, or verified. |
| **BLOCKED** | A named dependency prevents safe progress. |
| **PARKED** | Deliberately inactive until a concrete consumer exists. |
| **DONE** | Implementation, focused evidence, required gates, documentation, and clean-tree evidence are recorded. |
| **DROPPED** | Deliberately not implemented; rationale and replacement are recorded. |
| **SUPERSEDED** | Replaced by a named task, ADR, decision, or package contract. |
| **HISTORICAL** | Immutable evidence about an earlier revision; not a claim about the current checkout. |

Code resembling the intended result is not sufficient for `DONE`.

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
| **P2** | Required correctness, determinism, publication, or identity work before phase exit. |
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

### 2.1 Static review basis · `tbl:backlog:review-basis`

This rewrite incorporates a static review of the supplied repository tree:

```text
reviewed tree:
   

selected files:
    342

selected-file bytes:
    3,682,109

submodules:
    none

tracked symlinks observed in supplied tree:
    none
```

The supplied content excluded:

```text
Cargo.lock
LICENSE-CODE
LICENSE-DOCS
archive/
```

No Cargo, Meson, TeX, advisory, target-execution, or reproducibility command was
run as part of that review. Therefore:

- T1–T4 are static findings until reproduced;
- this review makes no current green-build claim;
- dependency checksums and resolved features were not independently verified;
- advisory status was not checked;
- licence compatibility was not independently checked;
- historical gate records remain historical evidence only.

### 2.2 Historical gate evidence · `gate:backlog:phase1`

The repository records:

```text

Phase-1 realization foundation:
    phase1-realization-foundation-v1
```

Those tags are immutable evidence for their exact commits. They do not establish
that the current checkout passes.

The latest complete gate narrative retained by the prior backlog was for an
earlier tree. It must not be reported as evidence for
`e383bfd6b30f6eb3295b94806c650cd1f8d92410`.

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
| Layer 0 | Published specification, version `0.6.0` |
| Realization document | Realization with final architecture appendix |
| `architecture` | Typed architecture, validation, semantic/behavioural hashes, deployment-profile scaffolding |
| `model` | Executable state machine, invariants, property/corruption suites, indexer and accounting projections |
| `realization` | Target-independent typed semantics for compact ASH and live receipt transfer |
| `compiler` | Package boundary and typed error root only |
| `artifacts` | Generated-publication derivation, writer/checker, realization-document weld |
| `labels` | Owner-aware Markdown/Rust label graph, census, plan checks, register rendering |
| `cli-common` | ADR-010 streams, diagnostics, checker report/stamp publication |
| `document-stamps` | Git-derived deterministic paper metadata |
| `execwrap` | Child process byte routing and build-local mock TeX helper |
| `flatten-latex-main` | Deterministic allowlist-based LaTeX flattening |
| Meson | Explicit source census, stamp-backed check targets, mocked document graph |
| Security policy | Public-data interfaces and external execution-environment boundary |
| Path policy | Central tracked-mode audit, lexical generic output roles, explicit host-filesystem non-claims |

### 3.2 Current published identities · `tbl:backlog:identities`

The current values below are transcribed from the typed/generated architecture
publication in the reviewed tree:

| Identity | Current value |
|---|---|
| Layer-0 version | `0.6.0` |
| Realization version | tracked compiler-line binding |
| Architecture schema | `17` |
| Architecture semantic algorithm | `sha256-canonical-json-v2` |
| Architecture semantic hash | |
| Architecture behavioural algorithm | `sha256-canonical-json-behavioural-v3` |
| Architecture behavioural hash | |
| Layer-0 anchor-set hash | |
| Attestation wire schema | `13` |
| Deployment-profile schema | `2` |
| Cargo workspace version | `0.1.0` |
| Meson project version | `0.0.1` |

The authoritative homes are the typed architecture and its checked generated
publications. This table is informational and must be updated or removed if it
drifts again.

Architecture finality does not imply:

- complete realization scope;
- compiler completeness;
- target support;
- linked bundle or ABI existence;
- deployment evidence;
- production readiness.

### 3.3 Not implemented · `tbl:backlog:not-implemented`

```text
compiler input binding
compiler relation analysis
compiler proof planning
compiler disclosure/source analysis
compiler lifecycle/placement/coverage analysis

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
Compiler package:                  boundary and error root only
Compiler analysis:                 absent
Target/backend/linker/ABI:         absent
Independent deployment evidence:   absent
Production deployment:             absent
Current-tree complete gate:        not established by this review
```

Current packages are public-data tools. They do not legitimately accept private
keys, seed material, signing nonces, blinding factors, private openings,
credentials, or production authority.

---

## 4. Compact historical record · `sec:backlog:history`

Permanent task IDs remain recorded here without retaining their implementation
diaries.

### 4.1 Completed phases · `tbl:backlog:completed-phases`

| Phase | Status | Durable record |
|---|---|---|
| Phase 0 | HISTORICAL | the recorded baseline and identities on [the Phase-0 card](phases/00-baseline.md) |
| Phase 1 | HISTORICAL | the completion evidence on [the Phase-1 card](phases/01-realization.md), and the gate record in [the backlog archive](history/backlog-history.md) §2.3 |
| Phase 2 | Active | current backlog and phase card |

### 4.2 Historical finding families · `tbl:backlog:historical-findings`

| Family | Status | Scope |
|---|---|---|
| `F1` | HISTORICAL | Phase-1 remediation |
| `F2` | HISTORICAL | Post-Phase-1 boundary remediation |
| `F3-001`–`F3-010` | HISTORICAL | Meson, publication, projection, ownership, and stamp remediation |
| `F4-001`–`F4-005` | DONE or DROPPED | Predicate/lifecycle/labels findings; F4-003 reproduced false and dropped |
| `A17-001`–`A17-005` | DONE | ADR-017 implementation |
| `R1`–`R6` | DONE | Path, ADR status, dependency, CI, recipe, and compiler-status review |
| `S1`–`S7` | DONE | Semantic-boundary review and status weld |

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
| `T1` | P1 | DONE | Model-to-realization conformance observations are not bound to the request that produced the successor. |
| `T2` | P1 | DONE | Sponsor isolation can pass without typed substrate-conservation evidence. |
| `T3` | P2 | TODO | Multi-output generators publish one final path at a time and can leave mixed generations after failure. |
| `T4` | P2 | DONE | The global model invariant does not re-check architecture-derived runtime-bound minima. |
| `T5` | P2 | DONE | The backlog’s current architecture semantic hash was stale. |

All T1–T4 entries are static-review findings. Reproduction is the first step;
their descriptions are not execution evidence.

### T1 — Bind conformance observations to executed requests · `task:review:conformance-binding`

**Priority:** P1
**Status:** DONE
**Owners:** `model`, `realization` conformance boundary
**Blocks:** trusted model/realization evidence for P2-012 and Phase-2 exit
**Identity impact:** none unless a persistent conformance report is later added
**Schema impact:** possible typed execution-result API change
**Dependency impact:** none

#### Static basis

The pilot adapters receive independent values:

```rust
before
request
after
```

They validate that `after` extends `before` by one certificate of the expected
branch. They do not validate that `request` is the request that produced
`after`.

For live transfer, signer evidence is taken from the supplied request while
consumed/created objects and canonical flows are taken from the certificate in
`after`. The adapter does not compare:

- request receipt inputs with consumed live-receipt inputs;
- requested destinations with created owner/value outputs;
- request sponsor inputs with sponsor-flow sources;
- sponsor change and fee with the observed sponsor flow;
- the supplied signer sets with the request actually executed.

Compact ASH has the same binding issue for ASH inputs and sponsor evidence.

A caller can therefore construct a hybrid conformance observation from one
executed transition and another request of the same branch.

This does not appear to weaken model transition acceptance, because model
execution happens first. It can, however, produce false conformance evidence or
mask a harness-wiring defect.

#### Required implementation

Preferred design:

```text
execute request
    ↓
typed execution result binding
    predecessor
    request
    successor
    certificate
    ↓
conformance projection
```

The conformance adapter should consume one bound execution result rather than
three independently supplied values.

A smaller acceptable first repair is deterministic request replay:

1. execute the supplied request against `before` at the certificate order;
2. require the resulting world to equal `after`;
3. fail with a focused `RequestBindingMismatch` before projection.

The replay is a binding check only. Realization conformance must still evaluate
independently from model transition acceptance.

#### Required tests

For compact ASH and live transfer:

- matching request and successor succeed;
- different input list fails;
- different output destinations fail;
- different protocol signer set fails;
- different sponsor input list fails;
- different sponsor signer set fails;
- different sponsor fee/change fails;
- successor from another same-branch request fails;
- focused error is deterministic under declaration ordering.

#### Verification

```sh
cargo test --locked -p tripod-model realization_conformance
cargo test --locked -p tripod-realization
cargo test --workspace --locked
```

#### Exit

- [x] finding reproduced or disproved;
- [x] request/successor binding is structural or explicitly replay-validated;
- [x] both pilot adapters have focused mismatch tests;
- [x] model execution remains independent of realization evaluation;
- [ ] complete required gates pass and the tree is clean (recorded once at
      the batch remediation gate).

#### Evidence

Reproduced 2026-08-04: a temporary test executed one live transfer and
projected the observation with a different same-branch request; the hybrid
observation was accepted and reported conformant.

Repair: the model now owns an opaque bound execution. The struct
ExecutedTransition holds private predecessor, request, and successor;
construction is only through execute_bound (invariant-wrapped execution that
retains the binding) or bind_execution (deterministic replay: the successor
must extend the predecessor by exactly one certificate, the request is
re-executed at that certificate's order, and the replayed world must equal the
supplied successor completely, else RequestBindingMismatch). Both conformance
adapters now consume the bound execution, so request-side signer and sponsor
evidence necessarily comes from the request that produced the successor.
Replay uses model execution only; realization evaluation is never consulted.

Focused evidence: mismatch tests for both pilots (different inputs, outputs,
signer set, and successor-from-another-request all fail with
RequestBindingMismatch; the exact executed request binds and projects), a
public-API test exercising execute_bound, bind_execution, the accessors, and
into_world from outside the crate, and a compile-fail doctest showing external
assembly from independent parts is unconstructible. Model crate tests
(296 unit, 7 public API, 3 doctests) and workspace clippy -D warnings are
green. The full repository gate for this batch is recorded once at the
remediation gate, per the verification cadence.

### T2 — Make substrate conservation explicit at the sponsor-erased boundary · `task:review:sponsor-balance-evidence`

**Priority:** P1
**Status:** DONE
**Owners:** `realization`, model conformance adapter
**Blocks:** compiler proof/source planning for sponsor isolation
**Policy:** v13d sponsor erasure and D005
**Identity impact:** none until persistent evidence envelopes exist
**Schema impact:** likely observation or relation-status change
**Dependency impact:** none

#### Static basis

The sponsor-erased observation correctly omits individual sponsor amounts:

```rust
ObservedValue::SponsorOpaque
```

The sponsor-isolation evaluator checks:

- exact sponsor member references;
- declared sponsor family and L-BTC asset;
- source/destination uniqueness;
- input-owner authorization;
- at-most-one envelope.

It does not check sponsor balance, and `OperationObservation` carries no typed
verdict that the enclosing model or target transaction passed substrate
conservation.

The realization document’s sponsor-erasure projection says the retained
projection includes a balance verdict. The current Rust observation retains the
role structure but not that verdict.

A caller-authored observation can therefore receive `Passed` for sponsor
isolation even though no typed premise states that target-wide conservation
held. The design decision that conservation belongs to Elements is valid; the
missing part is evidence that the premise was actually established.

#### Required design

Do not reintroduce individual sponsor amounts.

Choose one typed boundary before implementation. Acceptable forms include:

```rust
enum SubstrateConservationEvidence {
    ModelKernelAccepted {
        transition: ModelTransitionIdentity,
    },
    TargetExecutionAccepted {
        target: TargetIdentity,
        transaction: TransactionIdentity,
        report: EvidenceReference,
    },
}
```

or a validated observation typestate:

```text
OperationObservation
    → structural validation
    → bind accepted model/target execution
    → ValidatedOperationObservation
    → realization evaluation
```

A second acceptable design is to split the relation:

```text
sponsor role isolation:
    runtime Passed/Failed

sponsor conservation:
    externally required / assumption-backed / target-evidenced
```

Whichever design is selected must make these claims distinct:

1. sponsor role structure is valid;
2. sponsor owners authorized their inputs;
3. substrate conservation accepted the complete transaction.

A standalone caller-authored observation must not receive an unconditional
runtime pass for claim 3.

#### Required tests

- valid role structure without substrate evidence does not obtain an
  unconditional conservation pass;
- accepted model execution discharges the model-side premise;
- future target evidence must bind exact target and transaction identities;
- different sponsor denominations with one sponsor-erased projection produce
  equal protocol-semantic verdicts;
- unsigned sponsor input still fails;
- duplicate sponsor reference still fails;
- unclaimed sponsor member still fails;
- foreign family in the sponsor region still fails;
- second sponsor envelope still fails.

#### Verification

```sh
cargo test --locked -p tripod-realization
cargo test --locked -p tripod-model realization_conformance
cargo test --workspace --locked
```

#### Exit

- [x] finding reproduced or disproved;
- [x] the substrate-conservation premise is typed;
- [x] no individual sponsor amount crosses the realization boundary;
- [x] role isolation and conservation evidence remain separate;
- [x] compiler-facing proof alternatives cannot infer an exact sponsor-value read;
- [ ] complete required gates pass and the tree is clean (recorded once at
      the batch remediation gate).

#### Evidence

Reproduced 2026-08-04: a caller-authored observation with one fee-sponsor
open flow of zero sources, zero destinations, and a nonzero fee passed
sponsor role isolation with no typed indication that whole-transaction
conservation had been established anywhere.

Repair (split relation and status, the reviewed option B): both pilots now
declare a substrate-conservation relation for L-BTC with the new proof
family SubstrateConservation (deliberately neither PublicArithmetic, whose
operands are erased, nor ConfidentialConservation, which is one target
mechanism). The evaluator maps it to the new status EvidenceRequired
carrying a typed ExternalEvidenceRequirement; it never evaluates to Passed
at this boundary. Sponsor role isolation remains a separate runtime
relation and now blocks the conservation requirement when it fails. Report
helpers required_external_evidence, has_semantic_failure, and
is_evidence_complete make the distinction queryable; is_conformant is
documented as not implying evidence completion.

Model side: the conformance adapters return ModelConformanceObservation,
producible only from a T1 bound execution whose kernel run validated
open-asset conservation on the exact observed transition; its established
set discharges only the model-side copy of the premise for the exact
operation, checked by unresolved_model_evidence. It is model evidence
only, never target or deployment evidence. Sponsor values remain
structurally erased; no new amount field or digest was introduced.

The former sponsor-imbalance fixture was renamed to the role-structure
failure it actually exercises (an unowned source), and typed-premise tests
replaced the value claim: the review's empty-flow shape now demonstrably
retains EvidenceRequired, for both pilots. Realization (154), model (299
unit, 7 public API), and workspace clippy -D warnings are green; the full
repository gate for this batch is recorded once at the remediation gate.

### T3 — Batch-stage multi-output generated publications · `task:review:batch-publication`

**Priority:** P2
**Status:** TODO
**Owners:** `artifacts`, `labels`, shared publication infrastructure if justified
**Policy:** (`[ADR017-rule:path:publication]`)
**Blocks:** Phase-2 clean publication gate
**Identity impact:** none
**Schema impact:** none
**Dependency impact:** none

#### Static basis

`generate-all` derives all expected artifact bytes in memory, then atomically
publishes each destination in sequence.

`generate_registers` similarly publishes the two register files one at a time.

Each individual file rename is atomic, but the publication set is not staged
before the first final path changes. If staging or publishing a later member
fails:

- earlier members may contain new bytes;
- later members may remain old;
- the command exits failure with a mixed-generation set.

Both writers also rewrite unchanged outputs rather than preserving them
compare-if-changed.

The checkers should detect the mixed state on the next run, so this is not
silent semantic acceptance. It remains an honest-tool publication defect and
can leave a failed explicit generation command with a dirty tracked tree.

#### Required implementation

Provide a batch-publication path that:

1. receives every output role and destination explicitly;
2. validates lexical role uniqueness before staging;
3. computes all bytes before mutation;
4. compares every destination with expected bytes;
5. stages every changed member in its destination directory;
6. flushes all staged files;
7. publishes final renames only after all staging succeeds;
8. preserves unchanged destination mtimes;
9. documents that separate final renames are not one filesystem transaction;
10. repairs any prior partial state on the next successful invocation.

A shared helper is justified only if both generators use the same stable
contract. Do not introduce a general publication crate for planning
convenience alone.

#### Required tests

For artifacts and label registers:

- output 1 unchanged when staging output 2 fails;
- all outputs unchanged when staging the final member fails;
- unchanged rerun preserves mtimes;
- prior mixed-generation state is repaired;
- aliased role paths fail before staging;
- failed generation leaves no staged files;
- concurrent successful invocations produce complete individual files;
- checker remains non-writing.

#### Verification

```sh
cargo test --locked -p tripod-artifacts
cargo test --locked -p tripod-labels
scripts/test-meson-mock.sh .
meson test -C build --print-errorlogs
```

#### Exit

- [ ] finding reproduced or disproved;
- [ ] all changed members stage before any final publication;
- [ ] unchanged members are compare-if-changed;
- [ ] focused failure and repair tests pass;
- [ ] mocked Meson generation/repair behavior passes;
- [ ] complete required gates pass and the tree is clean.

### T4 — Weld runtime-bound conformance into the global invariant · `task:review:invariant-bound-conformance`

**Priority:** P2
**Status:** DONE
**Owner:** `model`
**Blocks:** complete model-validity claim used by Phase 2
**Identity impact:** none
**Schema impact:** possibly one focused invariant-error variant
**Dependency impact:** none

#### Static basis

`genesis` performs:

```rust
constants.validate()?;
validate_bound_conformance(&constants)?;
```

The global `check_invariant` performs only `constants.validate()`.

`Constants::validate` requires finite bounds to be nonzero, but it does not
enforce architecture-derived cardinality minima. For example:

```text
ash_batch_max = 1
```

is nonzero but makes `compact-ash`, whose minimum is two ASH inputs,
unconstructible. `validate_bound_conformance` rejects it; `check_invariant`
does not.

Normal transitions do not mutate constants, so a world produced by valid
genesis and valid operations should preserve the stronger property. However,
`World` is intentionally publicly mutable for audit and corruption fixtures.
The global invariant is expected to detect such corruption, and currently has a
weaker notion of valid constants than genesis.

#### Required implementation

Make `check_invariant` enforce the same architecture-derived runtime-bound
minimum relation as genesis.

Preferred form:

```rust
validate_bound_conformance(&world.constants)
    .map_err(|_| InvariantError::...)
```

A dedicated `BoundConformance` reason is acceptable if it maps to the existing
domains invariant clause. Reusing `Domains` is also acceptable if the focused
diagnostic remains clear.

Do not duplicate cardinality minima in model constants. The typed architecture
remains the owner.

#### Required tests

- mutate each bound to one below its architecture-derived minimum;
- require the invariant to fail;
- set each bound to the exact minimum;
- require the invariant to pass where the rest of the fixture is valid;
- preserve the existing genesis rejection tests;
- preserve profile/runtime bound-conformance tests.

#### Verification

```sh
cargo test --locked -p tripod-model bound_conformance
cargo test --locked -p tripod-architecture
cargo test --workspace --locked
```

#### Exit

- [x] finding reproduced or disproved;
- [x] genesis and `check_invariant` use one bound-conformance authority;
- [x] every architecture bound has minimum-minus-one and exact-minimum coverage;
- [x] no duplicate model-owned minimum table is introduced;
- [ ] complete required gates pass and the tree is clean (recorded once at
      the batch remediation gate).

#### Evidence

Reproduced 2026-08-04: two focused tests failed against the unrepaired
invariant — a world mutated to the named regression ash_batch_max = 1
passed Constants::validate and check_invariant while
validate_bound_conformance rejected it, and the per-bound sweep showed the
same gap for every bound with a nonzero derived minimum.

Repair: check_invariant now calls the shared authority
validate_bound_conformance immediately after Constants::validate, mapped
to the existing Domains clause (no new welded failure vocabulary, per the
review's recommendation). Minima remain derived through
architecture::manifest_minimum_for_bound; no model-owned minimum table
exists. The reproduction tests are the permanent regressions:
minimum-minus-one fails and the exact minimum passes for every declared
bound, and ash_batch_max = 1 is the named case. Model crate suites
(301 unit, 7 public API, 3 doctests) are green; genesis rejection and
deployment-calibration tests are unchanged. Full gate at the batch
remediation gate.

### T5 — Correct stale current identity in planning · `task:review:backlog-identity-drift`

**Priority:** P2
**Status:** DONE
**Owner:** this backlog
**Identity impact:** none; planning text only
**Schema impact:** none
**Dependency impact:** none

#### Resolution

The previous backlog reported an obsolete architecture semantic hash.

This rewrite updates the informational current-state table to the value carried
consistently by:

- the typed architecture derivation;
- `packages/model/generated/architecture.json`;
- `packages/model/generated/architecture.toml`;
- the Realization masthead and appendix weld.

The generated architecture publication remains authoritative. This planning
table must be updated or removed if it drifts again; planning prose must not
become a competing identity source.

---

## 6. Phase-2 implementation queue · `sec:backlog:phase2`

### 6.1 Summary · `tbl:backlog:phase2`

| ID | Priority | Status | Deliverable |
|---|---:|---|---|
| `P2-001` | P1 | DONE | Immutable, canonical, ownership-validated realization boundary |
| `P2-002` | P2 | DONE | Petgraph dependency and lockfile review |
| `P2-003` | P1 | DONE | Compiler crate boundary and typed error root |
| `P2-004` | P1 | TODO | Bind architecture, realization, policy, and explicit scope |
| `P2-005` | P1 | BLOCKED | Canonical compiler relation DAG |
| `P2-006` | P1 | BLOCKED | Checked constant folding |
| `P2-007` | P1 | BLOCKED | Exact proof-alternative planning |
| `P2-008` | P1 | BLOCKED | Disclosure, source, and constructibility analysis |
| `P2-009` | P1 | BLOCKED | Representation lifecycle analysis |
| `P2-010` | P1 | BLOCKED | Execution-case placement and layout requirements |
| `P2-011` | P1 | BLOCKED | Relation-indexed coverage requirements |
| `P2-012` | P1 | BLOCKED | Compact-ASH and live-transfer analyzed pilots |
| `P2-013` | Gate | BLOCKED | Complete Phase-2 evidence and exit |

T1 and T2 may be repaired in parallel with P2-004, but must close before
P2-012 can establish trusted pilot evidence. T3 and T4 must close before
P2-013.

### P2-004 — Bind compiler input and scope · `task:phase2:bind-input`

**Priority:** P1
**Status:** TODO
**Depends on:** P2-001 through P2-003
**Blocks:** P2-005 through P2-012
**Owners:** `compiler`, `realization`
**Identity impact:** no public compiler digest
**Dependency impact:** no new dependency expected

#### Deliverable

Define one immutable compiler input boundary containing:

- architecture binding inherited from the validated realization;
- canonical realization projection;
- explicit operation scope;
- typed analysis policy;
- optional abstract target capabilities;
- no concrete target package, target bytes, deployment profile, or report.

The compiler must revalidate owner-controlled input rather than trust that a
caller previously did so.

#### Required behavior

Reject:

- unsupported realization schema;
- invalid realization;
- architecture-binding mismatch;
- operation absent from realization scope;
- duplicate scope member;
- policy inconsistent with the declared scope;
- target-specific or generated-publication input.

The existing `CompileError` input-boundary variants should be used or refined
rather than replaced by string errors.

#### API constraints

- consume typed values only;
- no filesystem or environment reads;
- no generated JSON/TOML/Markdown input;
- no model source or callback input;
- no target opcode, stack position, tapleaf, transaction slot, or bytecode;
- no speculative identity field.

Typed comparison remains the boundary until a real cross-process, cached, or
published consumer activates a compiler-plan identity under ADR-016.

#### Required tests

- valid Phase-1 scope;
- duplicate scope;
- missing operation;
- architecture mismatch;
- invalid or unsupported realization;
- operation-scope permutation;
- no generated-file parser or path API in the public surface;
- external public-API integration test.

#### Verification

```sh
cargo test --locked -p tripod-compiler
cargo test --locked -p tripod-realization
cargo clippy --workspace --all-targets --locked -- -D warnings
```

#### Exit

- [ ] typed input boundary exists;
- [ ] every input-boundary failure has focused coverage;
- [ ] scope and ownership are explicit;
- [ ] no target or filesystem detail enters compiler core;
- [ ] no compiler digest is minted;
- [ ] required gates pass and the tree is clean.

### P2-005 — Build the canonical compiler relation DAG · `task:phase2:relation-dag`

**Priority:** P1
**Status:** BLOCKED
**Depends on:** P2-004 and C1-005
**Blocks:** P2-006 through P2-012

Use a compiler-owned direct Petgraph graph with:

```text
typed stable keys
typed node and edge weights
stable-key → NodeIndex metadata
canonical stable-key projection
```

Requirements:

- compiler relation census equals realization scope;
- every source relation retains operation ownership and provenance;
- duplicate IDs reject;
- unknown endpoints reject;
- unsupported cycles reject with canonical SCC diagnostics;
- Petgraph indices remain local handles;
- insertion permutations produce equal typed projections;
- standard topology, SCC, and reachability use Petgraph.

### P2-006 — Implement checked constant folding · `task:phase2:constant-folding`

**Priority:** P1
**Status:** BLOCKED
**Depends on:** P2-005
**Blocks:** P2-007 and P2-012

Initial legal folds:

- typed literals;
- boolean identities;
- exact count and amount operations;
- statically known activation;
- explicitly set-like canonical ordering;
- structural sharing retaining complete provenance.

Do not:

- reassociate checked arithmetic;
- move or combine floor operations;
- change overflow or underflow behavior;
- reorder named failure conditions;
- remove relation ownership;
- change disclosure or witness requirements.

Compare every folded result with a non-folded evaluator.

### P2-007 — Implement exact proof planning · `task:phase2:proof-planning`

**Priority:** P1
**Status:** BLOCKED
**Depends on:** P2-005, P2-006, C1-008
**Blocks:** P2-008 through P2-012

For each relation:

1. enumerate realization-approved alternatives;
2. reject missing capabilities;
3. reject unauthenticated sources;
4. reject unavailable witnesses;
5. reject permissionless owner/operator secrets;
6. reject representation failures;
7. reject lifecycle failures;
8. reject disclosure failures;
9. retain the exact feasible set or Pareto frontier;
10. select canonically only under explicit policy.

Pilot planning uses deterministic exact enumeration or branch-and-bound.

Complexity exhaustion returns a typed error and never:

- drops a relation;
- weakens authorization;
- silently increases disclosure;
- removes a lifecycle exit;
- selects the best partial result;
- claims optimality.

### P2-008 — Derive disclosure, sources, and constructibility · `task:phase2:constructibility`

**Priority:** P1
**Status:** BLOCKED
**Depends on:** P2-007 and closure of T2
**Blocks:** P2-010 through P2-012

Every relation operand records an authenticatable source class.

Permissionless cases require public facts or constructor-local sponsor
capabilities only.

Disclosure reasons remain separate:

```text
semantic public state or event
permissionless constructibility
target safety
deployment policy
```

Sponsor-value opacity remains structural:

- individual sponsor amounts are not protocol facts;
- sponsor role isolation and substrate conservation evidence are distinct;
- compiler analysis must not infer an exact sponsor-value read.

### P2-009 — Analyze representation lifecycle · `task:phase2:lifecycle`

**Priority:** P1
**Status:** BLOCKED
**Depends on:** P2-007
**Blocks:** P2-010 through P2-012

For every supported representation, record paths to required exits.

At minimum:

```text
live receipt:
    transfer
    burn
    redeem

ASH:
    compact
    clear
```

A pilot may be semantically valid in current scope while lifecycle-incomplete
for deployment. That distinction remains typed and explicit.

### P2-010 — Derive execution-case placement and layout requirements · `task:phase2:placement`

**Priority:** P1
**Status:** BLOCKED
**Depends on:** P2-008, P2-009, C1-009
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

### P2-011 — Derive relation-indexed coverage requirements · `task:phase2:coverage`

**Priority:** P1
**Status:** BLOCKED
**Depends on:** P2-010
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

### P2-012 — Analyze both pilots end to end · `task:phase2:pilots`

**Priority:** P1
**Status:** BLOCKED
**Depends on:** T1, T2, P2-005 through P2-011
**Blocks:** P2-013

#### Compact ASH

Analyze:

- cardinality and recognition;
- permissionless authorization;
- ownerless `U` conservation;
- exact canonical delta;
- sponsor multiplicity and role isolation;
- substrate conservation premise;
- no roots;
- transition-certificate projection only;
- public constructibility;
- explicit/public representation;
- compact and clear lifecycle.

#### Live transfer

Analyze:

- input/output cardinality;
- live receipt recognition and closure;
- all-owner authorization;
- exact aggregate `U` conservation alternatives;
- explicit closed `U`;
- destination-family closure;
- sponsor multiplicity and role isolation;
- substrate conservation premise;
- no roots;
- transition-certificate projection only;
- explicit/private-committed alternatives;
- transfer, burn, and redemption lifecycle.

Repeated analysis from equal typed inputs must produce equal stable projections.

### P2-013 — Phase-2 evidence and exit · `gate:backlog:phase2`

**Priority:** Gate
**Status:** BLOCKED
**Depends on:** T1–T4, P2-004 through P2-012, C1-005, C1-008, C1-009,
C1-010, C1-013

Phase 2 exits only when:

- current review findings are closed or formally refuted;
- compiler input binding is complete;
- relation census exactly equals realization scope;
- every relation has proof, source, constructibility, lifecycle, placement,
  layout, target-requirement, and coverage information;
- unsupported capabilities fail without semantic weakening;
- no concrete target detail enters compiler core;
- compact ASH and live transfer analyze deterministically;
- independent small-instance oracles agree;
- current required repository gates pass;
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
| `C1-005` | TODO | Canonical direct-Petgraph compiler graph prototype |
| `C1-006` | PARKED | Exact keyed linear systems until a consumer exists |
| `C1-007` | PARKED | Certified numerical analysis until a consumer exists |
| `C1-008` | TODO | Exact proof-plan search |
| `C1-009` | TODO | Execution-case-aware placement |
| `C1-010` | TODO | Typed symbol resolution and SCC policy |
| `C1-011` | BLOCKED | Structured relocation; linker phase |
| `C1-012` | BLOCKED | Deterministic bounded-depth target tree; linker phase |
| `C1-013` | TODO | Independent small-instance oracles |
| `C1-014` | BLOCKED | Preparation review and Phase-2 handoff |

### 7.2 C1-005 — Canonical Petgraph construction

Implement:

- typed nodes and edges;
- stable semantic keys;
- canonical insertion;
- stable-key/local-index metadata;
- Petgraph topology, SCC, and reachability;
- canonical stable-key projection;
- deterministic diagnostics.

Test:

- node and edge insertion permutations;
- duplicate keys and edges;
- unknown endpoints;
- self-loops;
- disconnected graphs;
- deep chains;
- SCCs;
- attempted publication of local graph indices.

### 7.3 C1-008 — Exact proof-plan search

Implement an exact pilot planner and compare every generated small case with
exhaustive enumeration.

Required adversarial cases:

- cheapest local alternatives form an invalid global plan;
- sharing changes the optimum;
- lifecycle-safe plan differs from the cheapest immediate plan;
- equal-cost plans require stable-key tie-breaking;
- no feasible plan;
- complexity budget exhausted.

### 7.4 C1-009 — Execution-case-aware placement

Enumerate required execution cases and eligible carriers.

A relation carried somewhere but absent from one active case remains uncovered.

Compare production search with exhaustive carrier-subset enumeration.

### 7.5 C1-010 — Typed symbols and SCC policy

Use two-pass typed resolution:

1. complete definition census;
2. complete reference resolution.

Normalize SCC members by stable key.

SCC membership does not authorize a semantic cycle. Every accepted cycle
requires an explicit typed resolution strategy.

### 7.6 C1-013 — Independent algorithm oracles

| Production analysis | Independent oracle |
|---|---|
| canonical topology | valid-order enumeration plus least-key rule |
| SCC | mutual-reachability equivalence |
| expression interning | non-interned evaluator |
| dependency closure | repeated complete scan |
| proof selection | exhaustive candidate enumeration |
| placement | exhaustive carrier subsets |
| relation census | direct set equality |
| constant folding | non-folded evaluator |
| stable projection | declaration-order permutation |

---

## 8. Dependency posture · `sec:backlog:dependencies`

### 8.1 Current and deferred dependencies · `tbl:backlog:dependencies`

| Dependency | Status | Role |
|---|---|---|
| `petgraph = 0.8.3` | Adopted and reviewed; `serde-1` only | Graph storage and standard algorithms |
| `num-bigint` | Existing | Exact arbitrary-size integers |
| `num-integer` | Existing | Exact integer helpers |
| `num-traits` | Existing | Numeric traits |
| `num-rational` | Deferred | Future exact-rational consumer |
| `faer` | Deferred | Future certified numerical diagnostics |
| `fixedbitset` | Deferred | Dense local coverage sets if measured |
| Elements libraries | Phase-3 review | Target transaction and consensus types |
| SAT/LP/MILP solver | Deferred | Larger exact planning only after measured need |
| `salsa` | Deferred | Incremental compiler queries |
| `egg` | Deferred | Equality saturation |
| `rayon` | Removed and deferred | Requires measured need and schedule-independent results |

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

A report from one class never silently satisfies another.

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
| architecture/profile | `cargo test --locked -p tripod-architecture` |
| realization | `cargo test --locked -p tripod-realization` |
| model conformance | `cargo test --locked -p tripod-model realization_conformance` |
| model bounds | `cargo test --locked -p tripod-model bound_conformance` |
| artifacts | `cargo test --locked -p tripod-artifacts` |
| labels/plans | `cargo test --locked -p tripod-labels` |
| checker report/stamps | `cargo test --locked -p cli-common` |
| document stamps | `cargo test --locked -p tripod-document-stamps` |
| flattener | `cargo test --locked -p flatten-latex-main` |
| process wrapper | `cargo test --locked -p execwrap` |
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

Use `meson setup build` only if `build/` does not exist.

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

Untrusted contributions run only in an externally established, credential-free
isolated environment under ADR-015. A clean-tree result is a correctness check,
not malicious-code containment.

---

## 11. Current gate · `gate:backlog:current`

The Phase-2 gate is **not passed**.

Current blockers are:

```text
Static review:
    T1 request/successor conformance binding
    T2 typed sponsor substrate-conservation premise
    T3 batch-staged multi-output publication
    T4 invariant/runtime-bound conformance

Compiler:
    P2-004 input binding not implemented
    relation DAG not implemented
    constant folding not implemented
    proof planning not implemented
    disclosure/source/constructibility analysis not implemented
    lifecycle analysis not implemented
    placement/layout analysis not implemented
    coverage analysis not implemented
    pilot analysis not implemented

Evidence:
    no current-tree complete gate record for the reviewed tree
```

Until (`gate:backlog:phase2`) passes:

- Phase 1 remains historical tagged evidence;
- no compiler public analysis API is frozen;
- no public realization/compiler digest is minted without a real consumer;
- target-specific fields remain forbidden in realization and compiler core;
- no stable linker or transaction ABI exists;
- no floating value or local graph handle enters semantic identity;
- no draft bound becomes deployment calibration;
- no raw report digest is treated as evidence identity without typed role and
  subject binding;
- no self-consistent model checkpoint is described as independent target-chain
  evidence;
- no architecture, model, realization, hash, build, or test success is
  described as deployment readiness;
- no current checkout is described as green without a fresh complete execution
  record.

---

## 12. Execution order · `sec:backlog:order`

Execute in this order unless reproduction changes dependencies:

```text
1. Reproduce T1 and T2.
2. Decide and implement the T2 typed substrate-evidence boundary.
3. Bind conformance observations to executed requests under T1.
4. Reproduce and close T4.
5. Reproduce and close T3.
6. Implement P2-004 compiler input binding.
7. Implement C1-005 and P2-005 relation DAG construction.
8. Implement P2-006 checked constant folding.
9. Implement C1-008 and P2-007 exact proof planning.
10. Implement P2-008 and P2-009 source, disclosure, constructibility,
    and lifecycle analysis.
11. Implement C1-009/C1-010 and P2-010/P2-011 placement, layout,
    SCC, and coverage requirements.
12. Analyze compact ASH and live transfer end to end.
13. Run and record the complete Phase-2 gate.
14. Begin Phase-3 target work only after Phase-2 exit.
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

> Repair the four newly reviewed evidence, invariant, and publication boundaries; then build the Phase-2 compiler as an exact, deterministic, target-independent analysis with complete relation preservation, explicit constructibility and lifecycle, no speculative identities, and no ambiguous evidence.
