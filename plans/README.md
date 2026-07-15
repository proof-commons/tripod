# Tripod Implementation Plans

This directory contains implementation planning for the toolchain below the
existing attestation architecture and executable model.

The plans describe intended engineering work. They are **not** protocol inputs,
compiler inputs, deployment evidence, or substitutes for normative source.

The next substantial implementation milestone is the typed,
target-independent `realization` package. Before that work begins, the current
baseline-hardening gate in [`backlog.md`](backlog.md) must be completed.

---

## 1. Authority hierarchy

Planning documents never override normative source.

When two artifacts disagree, use the following authority order.

### 1.1 Layer-0 economic specification

The Layer-0 paper owns the abstract economic interface and its specified
properties:

```text
papers/attestation/main.tex
papers/attestation/sections/
```

Layer 0 is the root of the stack. It does not depend on the realization,
executable model, compiler, or deployment toolchain.

### 1.2 Realization conformance contract

The realization document owns the prose conformance contract, invariant
interpretation, translation obligations, oracle obligations, trust surface,
residuals, and release pins:

```text
docs/attestation/realization.md
```

Its attached architecture appendix is a verbatim publication of the generated
typed manifest, not an independent semantic source.

### 1.3 Typed architecture

The architecture crate owns every enumerated architecture set and relation:

```text
packages/architecture/
```

This includes assets, roots, objects, operations, quantities, witnesses,
invariant clauses, dependencies, decisions, bounds, amount limits, tags,
authorization evidence, stable identifiers, and discriminants.

Generated JSON and TOML are derivative publications of this typed Rust source.

### 1.4 Executable model

The model crate owns the current executable reference behavior:

```text
packages/model/
```

It implements the abstract transition system, operation constructors, exact
canonical and open-flow partitions, transition certificates, invariant
checking, attestation indexing, receipt-accounting audit, maintenance, and
deterministic/property test evidence.

The model is a transparent reference model. Its source files are not a language
for the future compiler to parse or scrape.

### 1.5 Implemented architecture decisions

Repository-wide decisions that have already been implemented live in:

```text
adr/
```

For example:

- [ADR-010](../adr/010-command-line-output-contract.md) governs first-party
  command-line output.
- [ADR-011](../adr/011-toolchain-and-dependency-policy.md) governs Rust,
  dependencies, locking, reproducibility, and unsafe code.

An implemented ADR takes precedence over an older planning decision.

### 1.6 Plans

Files in this directory describe proposed implementation work beneath the
artifacts above.

A plan that conflicts with normative source or an implemented ADR is defective.
If an implementation proposal requires a protocol-semantic change, the
normative artifacts and their versioning gates must change first.

---

## 2. Current phase

The repository is in:

```text
Phase 1 — typed realization foundation
```

Current work: implement the typed realization foundation per
[`packages/realization.md`](packages/realization.md) and the `R1`
tasks in [`backlog.md`](backlog.md).

The first realization declarations are:

1. `compact-ash`;
2. `transfer-live-receipts`.

They exercise opposite sides of the intended representation boundary:

- public, ownerless, permissionless aggregation;
- owner-authorized lateral movement with value-opacity latitude.

The complete phase sequence and exit gates are in
[`roadmap.md`](roadmap.md).

---

## 3. Development laws

The toolchain plans are governed by the following cross-package laws.

1. **Typed Rust is normative.** Compiler semantics come from typed Rust values,
   never from generated publication files.

2. **Generated artifacts are one-way derivatives.** JSON, TOML, Markdown,
   LaTeX, generated label indexes, and generated declassification reports may
   be published and checked, but are never trusted semantic inputs.

3. **Declassification is derived.** The future compiler derives disclosure
   requirements from the typed realization dependency graph. It does not ingest
   `declassification.json` and does not scrape model source.

4. **The model and compiler share one realization.** Model-conformance tests and
   compiler analysis consume or check against the same target-independent
   realization declaration.

5. **Architecture and deployment are separate assurance boundaries.** A final,
   valid architecture and a green model are necessary but insufficient for a
   deployment release.

6. **Translation evidence is trusted over compiler implementation.** The initial
   project does not claim a fully verified compiler. Each released bundle must
   carry relation-indexed differential and integration evidence.

7. **Safety and disclosure minimality are distinct.** Rejecting unauthorized
   transactions proves safety. Accepting equivalent lower-disclosure
   representations proves minimality. Neither claim substitutes for the other.

8. **Value representation may vary; closed protocol asset identity is initially
   rigid.** The initial Elements backend may use supported explicit or committed
   value representations, while closed protocol assets remain explicitly
   classified at protocol seams.

9. **Permissionless means publicly constructible.** A permissionless operation
   cannot require another participant's private opening, blinding factor,
   operator secret, or unavailable metadata.

10. **Canonical transaction layouts are backend ABI.** Loopless target
    enforcement lowers bounded collections through deterministic layouts,
    bounded unrolling, per-input decomposition, and coordinator obligations.

11. **Builds are deterministic.** Identical typed inputs must produce
    byte-identical generated artifacts, layouts, proof plans, scripts, linked
    bundles, witness ABIs, and reports.

12. **Plans are never machine-consumed semantics.** No compiler, linker,
    transaction builder, vector harness, or release validator may treat a file
    under `plans/` as semantic input.

The detailed dependency graph and package boundaries are in
[`toolchain-architecture.md`](toolchain-architecture.md).

---

## 4. Plan document classes

Each kind of planning information has one home.

| Class | Location | Purpose |
|---|---|---|
| Index | `README.md` | Authority, status, document census, and navigation. |
| Architecture | `toolchain-architecture.md` | Cross-package laws, dependencies, identities, and assurance boundaries. |
| Roadmap | `roadmap.md` | Phase sequence, milestones, dependencies, and exit gates. |
| Backlog | `backlog.md` | Current and immediately upcoming actionable work. |
| Package plan | `packages/*.md` | Purpose, typed inputs/outputs, API boundary, evidence, milestones, and exit criteria for one package. |
| Decision record | `decisions/*.md` | Accepted implementation direction and rationale. |
| Research note | `research/*.md` | An unresolved question with a prototype and decision procedure. |
| Reference | `reference/*.md` | External technical reference material; never project authority or compiler input. |

Substantive facts should have one authoritative planning home. Other documents
link to that home instead of repeating the argument.

---

## 5. Active document index

This section is the exhaustive index of the rewritten planning tree.

### 5.1 Cross-package plans

| Document | Status | Purpose |
|---|---|---|
| [`README.md`](README.md) | **ACTIVE** | This directory's authority, governance, and exhaustive index. |
| [`toolchain-architecture.md`](toolchain-architecture.md) | **ACTIVE** | Planned package graph, cross-cutting development laws, typed data flow, identities, and assurance boundaries. |
| [`roadmap.md`](roadmap.md) | **ACTIVE** | Ordered implementation phases and phase-exit gates. |
| [`backlog.md`](backlog.md) | **ACTIVE** | Current execution queue, beginning with baseline hardening. |

### 5.2 Package plans

| Document | Status | Planned package |
|---|---|---|
| [`packages/realization.md`](packages/realization.md) | **ACTIVE — NEXT SUBSTANTIAL WORK** | `tripod-realization` |
| [`packages/compiler.md`](packages/compiler.md) | **PLANNED** | `tripod-compiler` |
| [`packages/target-elements.md`](packages/target-elements.md) | **PLANNED** | `tripod-target-elements` |
| [`packages/tapscript.md`](packages/tapscript.md) | **PLANNED / PROTOTYPE-DEPENDENT** | `tripod-tapscript` |
| [`packages/simplicity.md`](packages/simplicity.md) | **PARKED** | `tripod-simplicity` |
| [`packages/linker.md`](packages/linker.md) | **PLANNED** | `tripod-linker` |
| [`packages/transaction.md`](packages/transaction.md) | **PLANNED** | `tripod-transaction` |
| [`packages/vectors.md`](packages/vectors.md) | **PLANNED** | `tripod-vectors` |
| [`packages/release.md`](packages/release.md) | **PLANNED** | `tripod-release` |

### 5.3 Accepted implementation decisions

| Record | Status | Decision |
|---|---|---|
| [`decisions/001-typed-rust-is-normative.md`](decisions/001-typed-rust-is-normative.md) | **ACCEPTED** | Compiler and release semantics consume typed Rust, never generated publication files. |
| [`decisions/002-target-independent-realization-layer.md`](decisions/002-target-independent-realization-layer.md) | **ACCEPTED** | A typed realization layer sits between architecture and compiler and is shared with model conformance. |
| [`decisions/003-multiple-backends-tapscript-first.md`](decisions/003-multiple-backends-tapscript-first.md) | **ACCEPTED** | Design for multiple backends; implement Elements tapscript first. |
| [`decisions/004-translation-validation-over-compiler-trust.md`](decisions/004-translation-validation-over-compiler-trust.md) | **ACCEPTED** | Validate each emitted bundle through evidence rather than claiming an initially verified compiler. |
| [`decisions/005-value-parametric-asset-rigid.md`](decisions/005-value-parametric-asset-rigid.md) | **ACCEPTED** | Value representation may vary where proven; closed protocol asset identity is initially explicit and rigid. |
| [`decisions/006-canonical-transaction-layout-abi.md`](decisions/006-canonical-transaction-layout-abi.md) | **ACCEPTED** | Canonical operation layouts and witness conventions form a typed backend ABI. |

These are implementation decisions, not protocol authority. Once implemented
as repository-wide policy, they may be promoted or restated as root ADRs.

### 5.4 Open research

The research register and its note conventions live in
[`research/README.md`](research/README.md).

| Document | Status | Blocks |
|---|---|---|
| [`research/state-object-constructor.md`](research/state-object-constructor.md) | **PROTOTYPE REQUIRED** | STATE-spending operations and metadata-dependent constructor continuity. |
| [`research/wide-arithmetic.md`](research/wide-arithmetic.md) | **PROTOTYPE REQUIRED** | Exact floor arithmetic for redemption, admission, settlement, and cycle. |
| [`research/public-declassification.md`](research/public-declassification.md) | **OPEN / PROTOTYPE REQUIRED** | Confidential-to-public synchronization and permissionless use of public openings. |
| [`research/settlement-layout.md`](research/settlement-layout.md) | **OPEN / PROTOTYPE REQUIRED** | Bounded settlement layout, obligation placement, and resource feasibility. |

An open research note must state the experiment that resolves it, the evidence
to collect, and the decision it is expected to produce.

### 5.5 Technical references

| Document | Status | Purpose |
|---|---|---|
| [`reference/elements-tapscript.md`](reference/elements-tapscript.md) | **REFERENCE — NOT A TARGET OR CONSENSUS-IMPLEMENTATION PIN** | Survey of relevant Elements tapscript capabilities and known evidence gaps; not a source pin (ADR-011). |

Reference documents are informational. Machine-consumed target facts will live
in the future `target-elements` crate as typed Rust values.

---

## 6. Status vocabulary

Documents use these statuses.

| Status | Meaning |
|---|---|
| **ACTIVE** | Current, maintained plan on the implementation path. |
| **ACCEPTED** | An implementation decision has been accepted, although implementation may remain. |
| **PROPOSED** | Candidate direction awaiting review and acceptance. |
| **PLANNED** | Intended work whose package or milestone has not started. |
| **PROTOTYPE REQUIRED** | The boundary is understood, but an executable experiment must succeed before interfaces freeze. |
| **OPEN** | Unresolved technical or architectural question. |
| **BLOCKED** | Cannot proceed until a named dependency or research result is resolved. |
| **PARKED** | Deliberately deferred and outside the current critical path. |
| **SUPERSEDED** | Replaced by a newer decision or plan and retained only when necessary to explain the decision chain. |
| **REFERENCE** | External or informational material, not a project plan or source of truth. |

Backlog tasks use:

```text
TODO
IN PROGRESS
BLOCKED
DONE
DROPPED
```

A task is `DONE` only when it links to implementation and verification
evidence.

The plans deliberately avoid numerical confidence estimates. Named blockers,
prototype requirements, and exit criteria are more useful and remain accurate
longer.

---

## 7. Conflict and supersession rules

When planning material disagrees:

1. normative source wins;
2. implemented ADRs win over planning decisions;
3. accepted decision records win over package-plan sketches;
4. package plans govern package-specific work;
5. research notes describe unresolved alternatives but do not override accepted decisions;
6. the roadmap governs sequencing;
7. the backlog governs current execution;
8. this README governs document status and ownership.

A superseded document should not remain silently active. Mark it
`SUPERSEDED`, link its replacement, or remove it. Git history preserves old
versions; active plans do not need to retain conversational transcripts.

---

## 8. Updating the plans

A planning change should follow these rules.

### 8.1 New accepted direction

If a change affects more than one package or establishes a lasting toolchain
rule:

1. add or update a decision record;
2. update `toolchain-architecture.md`;
3. update affected package plans;
4. update the roadmap or backlog if sequencing changes.

### 8.2 New unresolved question

Add a research note that identifies:

- the exact question;
- why it matters;
- existing constraints;
- candidate approaches;
- the prototype or measurement needed;
- acceptance and rejection criteria;
- blocked packages or milestones;
- the decision record expected after resolution.

### 8.3 New package

Every new package plan must state:

- purpose;
- normative typed inputs;
- forbidden inputs;
- typed outputs;
- dependency direction;
- public API boundary;
- deterministic ordering and identity;
- evidence obligations;
- generated artifacts;
- non-goals;
- milestones;
- exit criteria;
- open questions.

### 8.4 Completed work

When a backlog item is completed:

1. link the implementing source;
2. link the relevant test, report, artifact, or ADR;
3. record the verification command;
4. mark the task `DONE`;
5. remove stale “future work” language from package plans;
6. update phase status when its exit gate is fully satisfied.

---

## 9. Immediate navigation

For current work:

1. read [`backlog.md`](backlog.md);
2. close the Phase-0 baseline findings;
3. run the full baseline gate from [`roadmap.md`](roadmap.md);
4. read [`packages/realization.md`](packages/realization.md);
5. begin the two pilot realization declarations only after the gate is green.

For architecture context:

1. read [`toolchain-architecture.md`](toolchain-architecture.md);
2. read the accepted records under [`decisions/`](decisions/README.md);
3. consult research notes only for unresolved seams relevant to the current
   milestone.

---

## 10. One-line roadmap

> Preserve a clean and reproducible architecture/model/document baseline;
> derive one typed target-independent realization shared by model conformance
> and compiler analysis; lower it into deterministic relation, proof,
> disclosure, and layout plans; emit and link target programs through a
> canonical transaction ABI; and accept a deployment only through
> relation-indexed translation evidence and independent indexer/auditor
> reports—never by consuming generated publication files or merely trusting
> the compiler.
