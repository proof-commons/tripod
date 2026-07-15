# Attestation Toolchain Research Notes

> **Status:** ACTIVE
> **Scope:** Unresolved implementation questions that require source review,
> executable prototypes, measurement, or formal analysis before an interface or
> deployment choice can be accepted
> **Authority:** Research planning only; normative source, implemented ADRs, and
> accepted planning decisions take precedence
> **Machine-consumed by the toolchain:** no

This directory contains unresolved technical questions for the planned the attestation realization/compiler toolchain.

A research note records:

- what is not yet known;
- why the answer matters;
- which normative and accepted constraints already apply;
- which candidate approaches are under consideration;
- which prototype or measurement resolves the question;
- what evidence accepts or rejects each candidate;
- which packages and roadmap phases are blocked;
- which accepted decision should be created or updated after resolution.

A research note does **not** define production semantics, freeze a public API,
approve a target proof pattern, or authorize release.

---

## 1. Place in the authority hierarchy

Research notes are subordinate to:

1. Layer-0 economic specification;
2. realization conformance contract;
3. typed architecture;
4. executable model behavior;
5. implemented root ADRs;
6. accepted planning decisions;
7. active package plans and roadmap constraints.

The full hierarchy is defined in
[`../README.md`](../README.md).

If a candidate research design conflicts with normative source, the candidate
is invalid unless the project deliberately begins the normative versioning
process.

Research does not silently reinterpret the architecture or realization to make
a target implementation easier.

---

## 2. Purpose of a research note

Use a research note when:

- the package boundary is understood but one internal design is unresolved;
- an exact target behavior must be source-verified;
- several target proof methods may implement the same semantic relation;
- resource feasibility must be measured before a bound or ABI freezes;
- a concrete transaction-construction problem may invalidate an otherwise
  plausible script relation;
- an accepted decision deliberately leaves one implementation seam open;
- a prototype result may affect several packages;
- evidence stronger than ordinary unit tests may be needed;
- the answer determines whether work is implementation-only or requires
  normative revision.

Initial examples are:

- STATE/object constructor continuity;
- exact wide arithmetic;
- confidential-to-public value synchronization;
- bounded settlement layout and obligation placement.

---

## 3. What does not belong here

Do not use research notes for:

- current task status;
- ordinary implementation bugs;
- normative protocol semantics already settled;
- package responsibility descriptions;
- accepted cross-package implementation choices;
- generic educational material;
- copied upstream documentation;
- unscoped brainstorming;
- confidence percentages;
- product feature requests with no technical question;
- final deployment evidence reports;
- current release identities or hashes.

Use instead:

| Information | Home |
|---|---|
| Current implementation task | [`../backlog.md`](../backlog.md) |
| Phase sequencing and gates | [`../roadmap.md`](../roadmap.md) |
| Cross-package architecture | [`../toolchain-architecture.md`](../toolchain-architecture.md) |
| Package responsibilities | [`../packages/`](../packages/) |
| Accepted implementation choice | [`../decisions/`](../decisions/) |
| External technical survey | [`../reference/`](../reference/) |
| Implemented repository policy | [`../../adr/`](../../adr/) |
| Protocol semantics | normative source outside `plans/` |

---

## 4. Research statuses

Research notes use the following statuses.

| Status | Meaning |
|---|---|
| **OPEN** | The question is unresolved and no complete prototype is yet required or available. |
| **PROTOTYPE REQUIRED** | The package boundary is understood, but an executable experiment or measurement must succeed before an interface freezes. |
| **IN PROGRESS** | The defined prototype or source review is actively being executed. |
| **MEASUREMENT REQUIRED** | The construction is plausible, but a target/resource decision depends on measured results. |
| **RESOLVED** | The question has produced an accepted result, decision, and implementation handoff. |
| **REJECTED** | The investigated candidate or question was rejected; the reason is retained because it prevents repeated work. |
| **PARKED** | Work is intentionally deferred and does not block the active critical path. |
| **SUPERSEDED** | Another research note or decision replaces the question. |

A note may use a combined status when the distinction is useful, for example:

```text
OPEN / PROTOTYPE REQUIRED
```

Prefer one primary status once the work begins.

---

## 5. Research does not freeze production interfaces

A prototype may contain:

- temporary Rust APIs;
- target scripts;
- test-only metadata layouts;
- synthetic constructors;
- provisional witness formats;
- hard-coded fixture constants;
- limited operation scope;
- inefficient stack schedules;
- experimental report formats.

None of those becomes production ABI merely because the prototype succeeds.

After a successful prototype:

1. record the measured result;
2. state what the prototype did and did not establish;
3. create or update an accepted decision;
4. update affected package plans;
5. define the production typed API separately;
6. migrate or discard prototype code;
7. add permanent regression evidence;
8. update roadmap and backlog status.

Prototype code must be visibly experimental until that process completes.

---

## 6. Required research-note format

Every research note must use this structure:

```markdown
# Research Question: Title

> **Status:** OPEN | PROTOTYPE REQUIRED | IN PROGRESS |
> MEASUREMENT REQUIRED | RESOLVED | REJECTED | PARKED | SUPERSEDED
> **Blocks:**
> **Affected packages:**
> **Depends on decisions:**
> **Related normative constraints:**
> **Expected decision output:**
> **Machine-consumed by the toolchain:** no

## 1. Question
## 2. Why the answer matters
## 3. Existing constraints
## 4. Definitions and terminology
## 5. Required properties
## 6. Candidate approaches
## 7. Threat and failure model
## 8. Prototype design
## 9. Test and vector plan
## 10. Measurement plan
## 11. Acceptance criteria
## 12. Rejection criteria
## 13. Result
## 14. Decision and implementation handoff
## 15. Residual risks
## 16. References
```

Before work begins, `Result` and `Decision and implementation handoff` may say:

```text
Pending.
```

Do not omit them. Their presence makes the expected research output explicit.

---

## 7. Research-question quality rules

### 7.1 Ask one decisive question

A research note should have one primary question.

Bad:

```text
How should the compiler, linker, transaction builder, and release process work?
```

Better:

```text
Can an Elements tapscript program authenticate a metadata-dependent STATE
predecessor and reconstruct its successor under the same static code subtree
within target resource limits?
```

If candidates can be resolved independently, split the note.

### 7.2 State blockers precisely

Name the blocked:

- package;
- API;
- operation;
- roadmap phase;
- publication identity;
- deployment claim.

Avoid vague statements such as:

```text
blocks the compiler
```

when the issue actually blocks only STATE-spending backend operations.

### 7.3 Distinguish feasibility from preference

A prototype may ask:

```text
Does this construction fit and enforce the relation?
```

A later implementation decision may ask:

```text
Which feasible construction should the project select?
```

Do not treat one feasible result as automatically preferred if several
candidates remain.

### 7.4 State falsifiable criteria

A useful research note defines how a candidate can fail.

Examples:

- accepts successor under a different static code subtree;
- requires an owner secret on a permissionless path;
- cannot route residual blinding;
- exceeds one target resource limit;
- has ambiguous canonical encoding;
- cannot support exact target-native execution;
- produces non-deterministic bundle identity;
- changes abstract semantic capabilities.

### 7.5 Avoid “probably” as a result

Replace:

```text
This should probably fit.
```

with:

```text
The candidate is accepted only if the measured complete transaction stays
within the declared limits under the exact pinned target and every required
negative vector rejects.
```

---

## 8. Prototype discipline

### 8.1 Exact target binding

Target-dependent prototypes must identify:

- exact target definition;
- upstream source revision;
- network flavor;
- activation state;
- target program identity;
- test runner/tool version;
- relevant policy mode.

A prototype against an unspecified or mutable target cannot settle a release
design.

### 8.2 Typed fixture inputs

Use typed fixtures derived from:

- architecture IDs;
- realization relations;
- compiler proof requirements;
- exact target types;
- linked candidate identities where applicable.

Do not use planning strings as semantic input.

### 8.3 Isolation from production APIs

Prototype modules or crates should be:

- test-only;
- feature-gated as experimental;
- private;
- outside release package paths;
- clearly named as prototypes.

A release package must reject prototype-only target programs or report
schemas.

### 8.4 No hidden source-tree mutation

Prototype tests and measurements must not rewrite tracked files.

Explicit generators may write only to caller-selected output directories.

### 8.5 Reproducibility

A prototype result should be reproducible from:

- exact source revision;
- exact target identity;
- explicit test fixtures;
- explicit random seeds;
- explicit configuration;
- documented command.

Reports must be deterministic where the target behavior is deterministic.

### 8.6 Secret handling

Use test-only keys and blinding material.

Do not use:

- production keys;
- real wallet data;
- production RPC credentials;
- secret-bearing URLs in logs.

Canonical reports must remain secret-free.

---

## 9. Evidence classes in research

A research note must state which evidence classes its prototype produces.

Possible classes include:

### 9.1 Substrate-review evidence

Shows typed target facts match the reviewed upstream source (review
provenance recorded; not a consensus audit, ADR-011).

### 9.2 Pattern-level evidence

Shows one target program pattern satisfies its local typed contract.

### 9.3 Target-native execution evidence

Shows the exact target accepts/rejects concrete transactions or scripts.

### 9.4 Resource evidence

Shows script/transaction/witness/resource measurements under the exact target.

### 9.5 Semantic differential evidence

Compares target result with realization/model expectation.

### 9.6 Constructibility evidence

Shows the authorized actor can obtain all required witnesses.

### 9.7 Lifecycle evidence

Shows objects created under one representation retain all required exits.

### 9.8 Formal evidence

Examples:

- bounded SMT equivalence;
- proof-assistant theorem;
- machine-checked target expression equivalence.

Formal evidence must state its exact scope.

One evidence class does not silently satisfy another.

---

## 10. Measurement rules

### 10.1 Measure the right unit

Target resource decisions must measure:

```text
complete valid transaction
```

when target limits bind complete transactions.

An isolated leaf/program measurement is useful but insufficient for deployment
calibration.

### 10.2 Record predicted and observed values

Where backend/linker formulas exist, report both:

- predicted resources;
- observed resources.

A mismatch is a research result, not a value to overwrite silently.

### 10.3 Several worst cases may exist

Measure separately for dimensions such as:

- transaction weight;
- witness bytes;
- stack depth;
- stack item count;
- maximum element size;
- crypto budget;
- target opcode/project cost;
- standardness;
- package relay behavior.

Do not call one fixture universally worst-case without proof.

### 10.4 Candidate identities

Every measurement binds:

- target;
- backend configuration;
- candidate bundle/program;
- transaction ABI;
- bound assignment;
- representation;
- fixture;
- tool version.

Measurements for changed bytes are stale.

### 10.5 No draft-default conclusion

Architecture draft defaults are starting candidates, not deployment results.

A research measurement must not convert one draft default into a release
constant without the calibration and release gates.

---

## 11. Acceptance and rejection

### 11.1 Acceptance requires all mandatory criteria

A candidate is accepted only when every criterion marked mandatory passes.

Do not average safety and resource results.

For example:

```text
safe but too large
```

is not accepted for the selected target deployment.

It may remain a semantic proof candidate for another backend.

### 11.2 Conditional acceptance

A result may be conditionally accepted when the condition is explicit and
represented in the implementation plan.

Examples:

- accepted only for explicit values;
- accepted only below a measured bound;
- accepted only for owner-authorized normalization;
- accepted only on regtest, with production activation evidence pending.

A conditional result must not be described as unconditional support.

### 11.3 Rejected candidates remain documented

Retain enough information to prevent repeated investigation:

- candidate summary;
- failed criterion;
- measured evidence;
- whether reconsideration is possible;
- what would have to change.

### 11.4 Inconclusive results

If infrastructure failure or missing target support prevents a result, mark the
research:

```text
inconclusive
```

within the result section and retain the primary status as open or blocked.

Do not treat infrastructure failure as target rejection.

---

## 12. Research-to-decision handoff

A resolved research note should produce one of:

1. a new accepted decision record;
2. an update to an existing decision;
3. a package-plan revision;
4. a roadmap revision;
5. a rejected candidate retained in the note;
6. a normative change proposal if the implementation cannot satisfy existing
   semantics.

The handoff must identify:

- selected candidate;
- exact scope;
- production API consequences;
- identity/schema consequences;
- required permanent tests;
- required generated artifacts;
- required deployment evidence;
- residual risks;
- superseded candidates.

The research note becomes `RESOLVED` only after the handoff documents are
updated or explicitly queued.

---

## 13. Initial research index

This section is the exhaustive index of the initial research set.

| Document | Status | Primary question | Blocks |
|---|---|---|---|
| [`state-object-constructor.md`](state-object-constructor.md) | **PROTOTYPE REQUIRED** | Can the initial Elements backend authenticate metadata-dependent object/STATE predecessors and reconstruct successors under the intended static code relation without a target escape? | STATE constructor ABI; `announce-maturity`; every later STATE-spending operation |
| [`wide-arithmetic.md`](wide-arithmetic.md) | **PROTOTYPE REQUIRED / MEASUREMENT REQUIRED** | Which exact target proof and stack schedule establishes `floor_mul_div` under target arithmetic and resource limits? | redemption, admission, settlement, cycle |
| [`public-declassification.md`](public-declassification.md) | **OPEN / PROTOTYPE REQUIRED** | How does a confidential semantic input become publicly and permissionlessly usable while preserving commitment balance and exact value binding? | confidential burn-to-ASH, direct confidential redemption, other private-to-public paths |
| [`settlement-layout.md`](settlement-layout.md) | **OPEN / PROTOTYPE REQUIRED** | Which bounded transaction layout and obligation placement enforces settlement's per-entitlement arithmetic, owner routing, control/vault relation, and terminal behavior within target limits? | final settlement ABI, settlement calibration, cycle-to-settlement deployment path |

No other research file should be added without updating this index and
[`../README.md`](../README.md).

---

## 14. Dependency relationships among initial research notes

The initial questions are related but not identical.

```text
state-object-constructor
    ├── blocks STATE-spending backend operations
    └── informs public metadata commitment

wide-arithmetic
    ├── blocks redemption
    ├── blocks admission/cycle issuance
    └── blocks settlement floor calculations

public-declassification
    ├── depends on target commitment semantics
    ├── may use constructor/public metadata mechanisms
    └── affects representation lifecycle

settlement-layout
    ├── depends on wide-arithmetic feasibility
    ├── depends on transaction ABI concepts
    ├── may depend on public value availability
    └── does not define STATE constructor semantics directly
```

Recommended execution order:

1. typed target compatibility contract and foundational tapscript builder;
2. STATE constructor and wide arithmetic in parallel;
3. public declassification after exact CT/constructor capabilities are
   available;
4. settlement batch-size-2 prototype after wide arithmetic and transaction ABI
   foundations exist.

This order is advisory. The roadmap remains authoritative.

---

## 15. Relationship to package plans

### 15.1 `realization`

Research may refine supported semantic proof alternatives and lifecycle
requirements.

It must not introduce target opcodes into `RealizationSpec`.

### 15.2 `compiler`

Research may add or refine:

- abstract target capabilities;
- proof alternatives;
- fact-source requirements;
- placement requirements;
- layout requirements;
- coverage cases.

It must preserve relation identities unless the semantic relation itself was
wrong.

### 15.3 `target-elements`

Research consumes exact target facts.

When a new complete proof pattern is accepted, the target package may advertise:

```text
source-supported primitive
+
approved proof pattern available
```

The static target definition still does not record mutable test status.

### 15.4 `tapscript`

Prototype target programs may become production patterns only after acceptance
and permanent evidence.

### 15.5 `linker`

Constructor research affects:

- symbol categories;
- reference strategies;
- SCC interpretation;
- relocation types;
- linked constructor recipes.

### 15.6 `transaction`

Research affects:

- metadata encoding;
- public openings;
- witness ABI;
- proof generation;
- canonical layouts;
- worst-case fixtures.

### 15.7 `vectors`

Every accepted result requires permanent positive/negative vectors and report
provenance.

### 15.8 `release`

Accepted research results must identify required:

- target evidence;
- artifact identity;
- calibration evidence;
- report schema;
- residual disclosure.

---

## 16. Preventing accidental production adoption

Prototype code must not enter production merely through import convenience.

Required safeguards may include:

- private modules;
- `#[cfg(test)]`;
- an `experimental` Cargo feature disabled in release;
- separate prototype binary/package;
- type-level `Prototype` status;
- release validation rejecting prototype artifacts;
- no stable publication hash;
- explicit source comments and package documentation.

A prototype feature must not be enabled in the final release build unless the
research note is resolved and the package plan/decision has been updated.

---

## 17. Research artifact policy

### 17.1 Temporary artifacts

Prototype outputs may include:

- scripts;
- target transactions;
- measurement reports;
- stack traces;
- SMT inputs;
- solver outputs;
- disassembly;
- source-conformance reports.

Store them in caller-selected temporary/output directories during active work.

### 17.2 Committed artifacts

Commit a research artifact only when it is needed for:

- reproducibility;
- review;
- permanent regression;
- accepted decision evidence.

A committed artifact needs:

- typed or documented source;
- exact target/tool version;
- canonical path;
- deterministic content where possible;
- no secrets;
- generator/checker or a clear immutable-source rule;
- provenance.

### 17.3 Promotion to release evidence

Research artifacts are not automatically release evidence.

Promotion requires:

- accepted production design;
- exact final target/program identity;
- permanent report schema;
- final bundle/ABI binding;
- release-policy recognition.

### 17.4 Large artifacts

Do not introduce LFS or large binary archives without a separate repository
policy decision.

---

## 18. Research command behavior

Prototype binaries should follow repository CLI policy when first-party and
retained.

Under ADR-010:

- stdout result data is JSON;
- diagnostics are JSON on stderr;
- files use explicit output paths;
- no plain text;
- no raw secret-bearing command line logging;
- result commands refuse terminal stdout;
- exit codes use 0/1/2 classes;
- panic payloads remain hidden without debug.

Throwaway local scripts that are never committed do not define repository
policy. Any committed script must be reviewed.

---

## 19. Reproducibility checklist

A research result intended to settle a decision should record:

- [ ] repository source revision;
- [ ] exact target identity;
- [ ] target binary/source revision;
- [ ] compiler/backend prototype revision;
- [ ] configuration identity;
- [ ] typed fixture/vector identity;
- [ ] explicit random seed;
- [ ] test-only key identities where relevant;
- [ ] command or test entry point;
- [ ] predicted measurements;
- [ ] observed measurements;
- [ ] report hash;
- [ ] no secret material;
- [ ] clean-tree result.

If one item is intentionally unavailable, state why.

---

## 20. Security review checklist

Before accepting a target construction, review:

- [ ] predecessor authenticity;
- [ ] successor authenticity;
- [ ] static code/program continuity;
- [ ] metadata canonicalization;
- [ ] object/asset closure;
- [ ] consensus-value binding;
- [ ] authorization;
- [ ] permissionless constructibility;
- [ ] lifecycle exits;
- [ ] public disclosure and opening authenticity;
- [ ] witness availability;
- [ ] malformed target inputs;
- [ ] mixed-program combinations;
- [ ] resource limits;
- [ ] target policy;
- [ ] target-native execution;
- [ ] relation-indexed negative vectors;
- [ ] deterministic identity;
- [ ] deployment evidence requirement.

A construction that passes only the happy path is not resolved research.

---

## 21. Normative-change trigger

Research must stop and request normative review when a result suggests the
initial target cannot implement the current abstract system without changing:

- operation acceptance;
- formula;
- recipient;
- authorization;
- public observable;
- permissionless capability;
- object recognition;
- lifecycle exit;
- invariant;
- Layer-0 interface.

The implementation team must not classify a denotation change as a backend
optimization.

Use the realization versioning law to determine whether the change is:

- implementation-latitude refinement;
- letter revision;
- major revision.

No plan file can make that determination unilaterally.

---

## 22. Research review process

### 22.1 Opening a note

Before implementation:

- write the primary question;
- list blockers;
- list existing constraints;
- define candidates;
- define acceptance and rejection criteria;
- identify exact target/prototype dependencies.

### 22.2 Midpoint review

For substantial prototypes, review:

- whether the question remains correctly scoped;
- whether new candidates appeared;
- whether target/source assumptions changed;
- whether measurement units are correct;
- whether production adoption is still prevented.

### 22.3 Final review

Before marking `RESOLVED`, verify:

- prototype is reproducible;
- all mandatory vectors pass;
- failed candidates are recorded;
- resource evidence is complete;
- exact scope is stated;
- package/API consequences are clear;
- decision record exists or is updated;
- permanent tests are queued or implemented;
- release evidence requirements are identified.

### 22.4 Reopening

A resolved note may be reopened if:

- target revision changes relevant semantics;
- permanent regression fails;
- resource calibration invalidates the result;
- another backend reveals a target-specific assumption;
- a security review finds an omitted attack;
- implementation differs materially from the prototype.

Record the reopening reason and status transition.

---

## 23. Initial research-set definition of done

The initial research planning set is complete when:

- [ ] this index exists;
- [ ] all four initial research notes use the required structure;
- [ ] every note names exact blockers;
- [ ] every note distinguishes semantic requirements from target candidates;
- [ ] every note defines positive and negative vectors;
- [ ] every note defines resource/measurement requirements;
- [ ] every note defines acceptance and rejection criteria;
- [ ] every note defines the expected decision output;
- [ ] no candidate is prematurely labeled accepted;
- [ ] package plans link the relevant notes;
- [ ] roadmap dependencies match the notes;
- [ ] no prototype result is required as a semantic input before it exists;
- [ ] all links resolve;
- [ ] `git diff --check` passes.

---

## 24. One-line research policy

> Isolate each unresolved implementation seam in one falsifiable,
> provenance-recorded, prototype-driven note; state the semantic constraints,
> target candidates, threat model, vectors, measurements, acceptance and
> rejection criteria, and decision handoff before coding; and never allow
> experimental bytes, APIs, or reports to become production policy merely
> because a prototype succeeds.
