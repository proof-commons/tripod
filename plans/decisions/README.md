# Attestation Planning Decisions

> **Status:** ACTIVE
> **Scope:** Accepted and proposed implementation decisions for the planned
> realization/compiler toolchain
> **Authority:** Planning decisions only; normative source and implemented ADRs
> take precedence
> **Machine-consumed by the toolchain:** no

This directory records durable implementation choices that affect more than one
planned package or establish a lasting toolchain rule.

Decision records explain **why an implementation direction was chosen**. They
do not define protocol semantics, replace normative source, or act as compiler
configuration.

Package plans cite these records rather than repeating their rationale.

---

## 1. Authority and scope

Planning decisions are subordinate to the repository's existing authority
hierarchy.

When artifacts disagree, use this order:

1. Layer-0 economic specification;
2. realization conformance contract;
3. typed architecture;
4. executable model behavior;
5. implemented root ADRs;
6. accepted planning decisions;
7. package-plan API sketches;
8. research candidates and exploratory notes.

The authoritative hierarchy is defined in
[`../README.md`](../README.md).

A planning decision is defective if it conflicts with:

- the Layer-0 paper;
- the realization document;
- the typed architecture;
- an architecture or behavioural-hash gate;
- the executable model's current normative role;
- an implemented ADR.

If implementing a planning decision would change protocol denotation, work must
stop and the normative versioning process must be followed first.

---

## 2. What belongs in a decision record

Create or update a decision record when a choice:

- affects more than one future package;
- establishes dependency direction;
- defines a lasting source-of-truth rule;
- determines an assurance boundary;
- chooses among materially different implementation architectures;
- constrains future public APIs;
- defines the relationship between compiler, backend, linker, evidence, and
  release tooling;
- replaces an older accepted implementation direction;
- resolves a research note whose result affects architecture.

Examples include:

- whether generated JSON may be compiler input;
- whether a target-independent realization layer exists;
- whether multiple backends are supported;
- whether compiler trust or translation validation is the initial assurance
  model;
- whether closed asset identity may be confidential;
- whether canonical transaction layouts are a published backend ABI.

---

## 3. What does not belong in a decision record

Do not create a decision record for:

- normative protocol semantics already owned by the architecture or realization
  document;
- current task status;
- phase sequencing;
- temporary implementation details local to one module;
- unmeasured target speculation;
- unresolved research alternatives;
- copied external technical facts;
- current architecture hashes or release identities;
- ordinary refactoring with no lasting architectural consequence.

Use instead:

| Information | Home |
|---|---|
| Current work | [`../backlog.md`](../backlog.md) |
| Phase sequence | [`../roadmap.md`](../roadmap.md) |
| Package-local plan | [`../packages/`](../packages/) |
| Unresolved prototype question | [`../research/`](../research/) |
| External substrate facts | [`../reference/`](../reference/) |
| Implemented repository policy | root [`../../adr/`](../../adr/) |
| Protocol semantics | normative source outside `plans/` |

---

## 4. Decision statuses

Decision records use the following statuses.

| Status | Meaning |
|---|---|
| **PROPOSED** | Candidate implementation direction awaiting acceptance. |
| **ACCEPTED** | Approved implementation direction; implementation may remain incomplete. |
| **IMPLEMENTED** | Accepted and implemented; evidence and the governing ADR or source are linked. |
| **SUPERSEDED** | Replaced by one or more newer records. |
| **REJECTED** | Considered and deliberately not selected. |
| **PARKED** | Decision intentionally deferred because its work is not on the active path. |

A record must never silently change from one meaning to another. Update its
status and append the relevant decision metadata.

---

## 5. Acceptance process

A proposed decision becomes accepted only after review of:

1. compatibility with normative source;
2. compatibility with current architecture and behavioural identities;
3. affected package boundaries;
4. dependency direction;
5. determinism and reproducibility consequences;
6. evidence and release consequences;
7. alternatives considered;
8. unresolved prototype dependencies;
9. supersession conditions;
10. roadmap and backlog impact.

An accepted decision may still contain explicitly labeled provisional details,
but its central choice must be settled.

If executable evidence is required before the choice can be settled, the
record remains `PROPOSED` or the issue remains in a research note with status
`PROTOTYPE REQUIRED`.

---

## 6. Relationship to root ADRs

Planning decisions govern intended future work.

Root ADRs govern repository policy that has been accepted and implemented.

The lifecycle is:

```text
research note
    ↓ resolves an open question
proposed planning decision
    ↓ accepted for implementation
accepted planning decision
    ↓ implementation lands and becomes repository policy
root ADR or implemented decision reference
```

Promotion to a root ADR is appropriate when a decision:

- is implemented;
- governs current first-party code;
- applies repository-wide;
- must be followed by future packages;
- is no longer merely prospective.

After promotion, the planning record should:

- link the root ADR;
- change status to `IMPLEMENTED` or `SUPERSEDED`;
- avoid duplicating the full implemented policy;
- preserve only planning history and consequences for future work.

Examples of already implemented root policy include:

- [ADR-010](../../adr/010-command-line-output-contract.md);
- [ADR-011](../../adr/011-toolchain-and-dependency-policy.md).

---

## 7. Numbering and filenames

Decision filenames use a stable three-digit sequence:

```text
NNN-short-kebab-case-title.md
```

Examples:

```text
001-typed-rust-is-normative.md
002-target-independent-realization-layer.md
003-multiple-backends-tapscript-first.md
```

Rules:

- numbers are never reused;
- filenames remain stable after acceptance;
- renaming an accepted record requires a link-preserving migration;
- superseded records retain their number;
- rejected records may retain their number when preserving the rationale is
  useful;
- a replacement receives a new number and links the record it supersedes.

The record number identifies the implementation decision, not a protocol
version.

---

## 8. Required record format

Every decision record must use this structure:

```markdown
# DNNN: Decision title

> **Status:** PROPOSED | ACCEPTED | IMPLEMENTED | SUPERSEDED | REJECTED | PARKED
> **Scope:**
> **Decision class:** Source of truth | Package architecture | Assurance | Representation | ABI | Other
> **Applies to:**
> **Depends on:**
> **Supersedes:**
> **Superseded by:**
> **Related normative constraints:**
> **Related research:**
> **Promoted ADR:**
> **Machine-consumed by the toolchain:** no

## 1. Context
## 2. Decision
## 3. Required consequences
## 4. Prohibited consequences
## 5. Alternatives considered
## 6. Assurance and evidence consequences
## 7. Determinism and identity consequences
## 8. Implementation and migration
## 9. Risks and limitations
## 10. Supersession conditions
## 11. References
```

Fields without a value use:

```text
none
```

Do not omit fields merely because a decision is early.

---

## 9. Decision-writing rules

### 9.1 State one central decision

A record should have one main decision.

If a document contains several choices that could be superseded independently,
split it.

### 9.2 Separate decision from consequence

Write:

```text
Decision:
    The compiler consumes typed Rust values.

Consequence:
    Generated architecture JSON is publication-only.
```

Do not blur the two into broad aspirational prose.

### 9.3 Name prohibited interpretations

Every record should state what it does **not** authorize.

This is important because implementation decisions often gain accidental scope
over time.

For example, accepting value-representation latitude does not authorize
confidential closed-asset identity.

### 9.4 Link normative constraints rather than retyping them

Decision records may summarize relevant constraints, but they should link to
their authoritative source.

Do not copy:

- entire invariant definitions;
- architecture manifests;
- current hash values;
- long operation declarations;
- complete target-opcode specifications.

### 9.5 Mark API sketches as provisional

If a record includes Rust pseudocode, precede it with:

> Illustrative API; not frozen by this decision.

The decision should freeze the architecture choice, not every field name.

### 9.6 Avoid confidence percentages

Use:

- explicit status;
- named blocker;
- prototype requirement;
- acceptance criterion;
- supersession condition.

Do not use numerical confidence estimates.

### 9.7 Record evidence boundaries

A decision must distinguish:

- what its implementation can establish;
- what remains model evidence;
- what remains backend evidence;
- what remains substrate evidence;
- what remains independent deployment evidence.

### 9.8 Preserve one-way planning direction

Decision Markdown is never semantic input to:

- `realization`;
- `model`;
- `compiler`;
- target packages;
- backends;
- linker;
- transaction builder;
- vector harness;
- release validation.

Machine-consumed configuration required by an implemented decision must be
represented in typed Rust or an explicitly typed external input owned by the
relevant package—not scraped from this record.

---

## 10. Current decision index

This section is the exhaustive index of the initial decision set.

| Record | Status | Class | Decision |
|---|---|---|---|
| [`001-typed-rust-is-normative.md`](001-typed-rust-is-normative.md) | **ACCEPTED** | Source of truth | First-party semantic and release tooling consumes typed Rust, never generated publication files or model source text. |
| [`002-target-independent-realization-layer.md`](002-target-independent-realization-layer.md) | **ACCEPTED** | Package architecture | A typed target-independent realization layer sits between architecture and compiler and is shared with model conformance. |
| [`003-multiple-backends-tapscript-first.md`](003-multiple-backends-tapscript-first.md) | **ACCEPTED** | Package architecture | Design the compiler for multiple backends while implementing Elements tapscript first. |
| [`004-translation-validation-over-compiler-trust.md`](004-translation-validation-over-compiler-trust.md) | **ACCEPTED** | Assurance | Validate each released target bundle through relation-indexed evidence rather than initially claiming a verified compiler. |
| [`005-value-parametric-asset-rigid.md`](005-value-parametric-asset-rigid.md) | **ACCEPTED** | Representation | Permit supported value representations while keeping closed protocol asset identity explicit and rigid in the initial Elements backend. |
| [`006-canonical-transaction-layout-abi.md`](006-canonical-transaction-layout-abi.md) | **ACCEPTED** | ABI | Generate deterministic operation layouts and witness conventions as a typed backend transaction ABI. |

The index must be updated whenever a decision record is added, removed,
superseded, rejected, parked, implemented, or promoted.

---

## 11. Initial decision dependencies

The initial records have this dependency shape:

```text
D001 Typed Rust is normative
   │
   ├──────────────▶ D002 Target-independent realization layer
   │                    │
   │                    ├────────▶ D003 Multiple backends, tapscript first
   │                    │
   │                    ├────────▶ D004 Translation validation
   │                    │
   │                    └────────▶ D005 Value-parametric, asset-rigid
   │
   └─────────────────────────────▶ D006 Canonical transaction-layout ABI
                                        ▲
                                        │
                                 D003 and D004
```

Interpretation:

- D001 establishes the source-of-truth direction.
- D002 establishes the semantic package boundary.
- D003 constrains the backend-neutral design above target emission.
- D004 establishes the evidence model for emitted programs.
- D005 constrains representation proof planning.
- D006 establishes the concrete ABI required by loopless target enforcement and
  relation-indexed evidence.

This dependency diagram is explanatory only. The records themselves contain
their authoritative dependency fields.

---

## 12. Initial research-to-decision paths

The following research notes are expected to produce later decisions:

| Research note | Expected decision |
|---|---|
| [`../research/state-object-constructor.md`](../research/state-object-constructor.md) | Concrete object/STATE constructor continuity and successor-authentication strategy. |
| [`../research/wide-arithmetic.md`](../research/wide-arithmetic.md) | Exact wide-floor proof pattern and evidence policy for the initial tapscript backend. |
| [`../research/public-declassification.md`](../research/public-declassification.md) | Concrete confidential-to-public synchronization representation for publicly usable values. |
| [`../research/settlement-layout.md`](../research/settlement-layout.md) | Canonical settlement layout and obligation-placement strategy. |

These decisions must not be pre-written as accepted outcomes before their
prototype gates pass.

---

## 13. Supersession rules

When a newer record replaces an older one:

1. create the new record with a new number;
2. set its `Supersedes` field;
3. update the old record:
   - `Status: SUPERSEDED`;
   - `Superseded by: DNNN`;
4. update this index;
5. update package plans;
6. update the roadmap and backlog where sequencing changes;
7. update or add a root ADR if implemented repository policy changes;
8. verify that no active document still treats the old decision as current.

Do not edit the older record so thoroughly that its original rationale
disappears. Supersession should preserve the decision trail.

---

## 14. Rejection rules

A rejected proposal should be retained when its rejection prevents likely
repetition or records an important safety boundary.

A rejected record must explain:

- why it was considered;
- what evidence rejected it;
- whether a different proposal replaced it;
- what would have to change for reconsideration.

Do not retain trivial rejected alternatives that add no future value.

---

## 15. Decision review checklist

Before accepting a record, verify:

- [ ] the central decision is singular and unambiguous;
- [ ] normative source remains authoritative;
- [ ] the decision does not hide a protocol-semantic change;
- [ ] affected packages are named;
- [ ] dependency direction remains acyclic;
- [ ] generated files remain one-way derivatives;
- [ ] assurance boundaries are explicit;
- [ ] determinism consequences are explicit;
- [ ] identity/hash consequences are explicit;
- [ ] forbidden interpretations are stated;
- [ ] alternatives are recorded fairly;
- [ ] prototype dependencies are named;
- [ ] supersession conditions are concrete;
- [ ] the roadmap and backlog impact is known;
- [ ] all relative links resolve;
- [ ] no current architecture hash is unnecessarily copied;
- [ ] the record is explicitly not machine-consumed.

---

## 16. Decision-set definition of done

The initial decision set is complete when:

- [ ] records D001–D006 exist;
- [ ] each uses the required format;
- [ ] each is consistent with normative source;
- [ ] each is indexed here and in [`../README.md`](../README.md);
- [ ] package plans cite the records instead of repeating their arguments;
- [ ] research-dependent choices remain outside the accepted set;
- [ ] no deleted conversational plan contains the only copy of an accepted
      rationale;
- [ ] no decision Markdown is consumed by first-party semantic tooling;
- [ ] all links resolve;
- [ ] `git diff --check` passes.

---

## 17. One-line decision policy

> Record durable cross-package implementation choices once, state their
> consequences and forbidden interpretations explicitly, promote them to root
> ADRs when they become implemented repository policy, and never let planning
> Markdown become protocol authority or compiler input.
