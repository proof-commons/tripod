# Guide 4 — Reaccept Exact Proof Planning and Restore Phase-2 Planning Consistency

## Mission

Repair the four findings introduced by the latest static review:

```text
T6  external-evidence relations bypass capability validation
T7  representation-sensitive proof choices can contradict selected modes
T8  current-phase declarations disagree and are incompletely checked
T9  compiler status documentation understates implemented analysis
```

Then reaccept:

```text
C1-008  exact proof-plan search
P2-007  exact proof-alternative planning
```

This batch must leave Phase 2 ready to begin:

```text
C1-009 / P2-010
    execution-case placement and target-independent layout requirements
```

Do **not** implement placement, concrete layout, target programs, target
capability adapters, or a complete analyzed-program public API in this batch.
The first priority is making the feasible proof-plan set correct.

---

## 1. Authority and required reading

Before editing, read these implementation owners:

```text
AGENTS.md
adr/011-toolchain-and-dependency-policy.md
adr/016-semantic-identities-and-evidence-binding.md

packages/compiler/README.md
packages/compiler/src/capability.rs
packages/compiler/src/proof.rs
packages/compiler/src/source.rs
packages/compiler/src/lifecycle.rs
packages/compiler/src/tests/proof_tests.rs
packages/compiler/src/tests/oracle_tests.rs

packages/realization/src/declarations/compact_ash.rs
packages/realization/src/declarations/transfer_live.rs
packages/realization/src/relation.rs

packages/labels/src/plans.rs

plans/README.md
plans/roadmap.md
plans/packages/compiler.md
plans/phases/02-compiler.md
plans/backlog.md
```

The relevant ownership rules are:

```text
realization:
    owns approved relation and proof semantics

compiler:
    owns capability filtering and exact plan feasibility

target/backend:
    later proves that a selected capability is implemented

evidence:
    remains unresolved until the owning evidence boundary discharges it
```

In particular:

```text
capability availability
    ≠
evidence completion
```

and:

```text
selected representation
    must be compatible with every proof whose semantics depend on it
```

---

## 2. Scope and non-goals

### In scope

- reproduce T6 and T7 with focused tests;
- repair external-evidence capability propagation;
- preserve external evidence as unresolved;
- repair representation/proof compatibility;
- strengthen the independent exhaustive oracle;
- update planning phase consistency checks;
- update stale compiler-status documentation;
- update the backlog and Phase-2 status only after verification;
- run focused, workspace, and full repository gates.

### Out of scope

- concrete target capabilities from `target-elements`;
- target opcodes, scripts, tapleaves, stacks, or transaction positions;
- placement or layout implementation;
- relation-indexed target coverage implementation;
- compiler-plan serialization;
- compiler, realization, or proof-plan digest;
- deployment evidence envelopes;
- generated compiler publications;
- public complete analyzed-program API;
- optimization objectives or canonical selection of one feasible plan.

No new dependency is expected.

---

# Part I — T6: External-evidence capability validation

## 3. Reproduce T6 before fixing it

Add a focused test to:

```text
packages/compiler/src/tests/proof_tests.rs
```

or, if the comparison belongs with the independent exhaustive implementation:

```text
packages/compiler/src/tests/oracle_tests.rs
```

Construct an available-capability set containing every current compiler
capability except:

```rust
RequiredCapability::WholeTransactionValueConservation
```

The set should include the other current variants explicitly:

```rust
BTreeSet::from([
    RequiredCapability::AuthenticatedObjectRecognition,
    RequiredCapability::AuthenticatedFamilyCardinality,
    RequiredCapability::AuthenticatedCanonicalPartition,
    RequiredCapability::AuthenticatedOpenFlowPartition,
    RequiredCapability::AuthenticatedRootEffects,
    RequiredCapability::AuthenticatedProjectionSet,
    RequiredCapability::ExactPublicAmountArithmetic,
    RequiredCapability::ConfidentialValueConservation,
    RequiredCapability::OwnerAuthorization,
    RequiredCapability::OperatorAuthorization,
    RequiredCapability::RefundAuthorization,
    RequiredCapability::PublicConstructibility,
])
```

Run the planner for at least:

```text
compact-ash
transfer-live-receipts
```

The pre-fix reproduction should show that a feasible plan is returned despite
the missing whole-transaction-conservation capability.

Record the reproduction in the eventual backlog evidence, but do not commit a
deliberately failing tree.

---

## 4. Preserve capability requirements on external evidence

### 4.1 Current defect

`Relation::SubstrateConservation` becomes:

```rust
RelationObligationClass::ExternalEvidence { requirement }
```

It does not enter the proof-variable search. Candidate capabilities and source
requirements are accumulated only from selected proof variables.

The realization declaration nevertheless approves:

```rust
ProofKind::SubstrateConservation
```

and the source layer correctly derives:

```rust
RequiredCapability::WholeTransactionValueConservation
```

That derivation is currently bypassed by production enumeration.

### 4.2 Required internal representation

Extend `RelationObligationClass::ExternalEvidence` so it retains the fixed
planning requirements of the external relation.

A suitable shape is:

```rust
ExternalEvidence {
    requirement: realization::ExternalEvidenceRequirement,
    proof: realization::ProofAlternativeId,
    required_capabilities: BTreeSet<RequiredCapability>,
    source_requirements: Vec<SourceRequirement>,
}
```

Equivalent typed factoring is acceptable.

The important properties are:

1. the relation remains externally evidenced;
2. its approved proof class is validated;
3. its capabilities are available to capability filtering;
4. its source requirement survives into the candidate;
5. it does not become a runtime `Passed` relation;
6. it does not require a sponsor amount.

### 4.3 Validate the external proof declaration

For the current `SubstrateConservation` relation, require the exact
realization-approved alternative:

```rust
ProofAlternativeId::new(
    declaration.id.clone(),
    ProofKind::SubstrateConservation,
)
```

The compiler must not silently manufacture this proof class if the realization
declaration omits it.

The current realization has exactly one approved proof alternative for this
relation. Validate that invariant rather than choosing the first element of an
arbitrary set.

If a focused compiler error is needed, add a non-exhaustive public variant such
as:

```rust
InvalidExternalEvidenceProofAlternatives {
    relation: realization::RelationId,
}
```

Use a precise name and add focused coverage. Do not overload
`MissingProofAlternative` if the actual defect is an extra or wrong
alternative.

### 4.4 Derive fixed capability and source requirements

During obligation classification, derive:

```rust
proof_capabilities(
    declaration,
    ProofKind::SubstrateConservation,
)
```

and:

```rust
derive_source_requirements(
    declaration,
    ProofKind::SubstrateConservation,
)
```

The expected fixed capability is:

```rust
RequiredCapability::WholeTransactionValueConservation
```

The expected source requirement remains typed external evidence. It must not be
rewritten as:

```text
exact sponsor input amounts
exact sponsor output amounts
public sponsor arithmetic
```

### 4.5 Apply capability filtering before search

In `enumerate_feasible_plans`:

1. collect all fixed external-evidence capabilities;
2. collect all fixed external source rows;
3. validate fixed source/constructibility consistency;
4. test the complete fixed capability set against `CapabilityView`;
5. if unavailable, fail with `NoFeasibleProofPlan`;
6. identify the substrate-conservation relation as blocked;
7. do not begin or return a partial candidate search.

A missing fixed capability is infeasibility, not complexity exhaustion.

### 4.6 Carry fixed requirements into every candidate

`complete` should begin with the fixed requirements and then extend them with
the selected proof-variable requirements.

Conceptually:

```rust
let mut required_capabilities = state.fixed_required_capabilities.clone();
let mut source_requirements = state.fixed_source_requirements.clone();

for selected proof {
    required_capabilities.extend(...);
    source_requirements.extend(...);
}
```

Sort and deduplicate source rows before constructing the candidate.

Every candidate carrying:

```rust
ExternalEvidenceRequirement::SubstrateConservation
```

must also carry:

```rust
RequiredCapability::WholeTransactionValueConservation
```

and the matching external-evidence source row.

The evidence requirement remains in:

```rust
candidate.external_evidence
```

It is not marked complete.

---

## 5. T6 focused tests

Add tests establishing all of the following:

### 5.1 Missing capability fails

For each pilot:

```text
all required capabilities except WholeTransactionValueConservation
    ⇒ NoFeasibleProofPlan
```

The blocked relation list must include the exact substrate-conservation
relation.

### 5.2 Available capability succeeds

For each pilot:

```text
same capability set
+ WholeTransactionValueConservation
    ⇒ at least one feasible plan
```

### 5.3 Candidate carries both dimensions

For every feasible candidate:

```text
external_evidence contains substrate conservation
required_capabilities contains whole-transaction value conservation
source_requirements contains typed external evidence
```

### 5.4 Evidence remains unresolved

No test should reinterpret capability availability as completed evidence.

The candidate must still carry the external evidence requirement.

### 5.5 Sponsor erasure remains structural

Assert that none of the fixed source requirements names:

```text
PLAIN_LBTC input amount
PLAIN_LBTC output amount
aggregate sponsor amount
```

---

# Part II — T7: Complete representation/proof compatibility

## 6. Reproduce T7 before fixing it

Add a focused test that enumerates live-transfer candidates and inspects:

```rust
candidate.representations
candidate.proofs
```

Locate:

```text
live-transfer representation relation
live-transfer conservation relation
```

The pre-fix reproduction should find at least one contradictory candidate such
as:

```text
selected representation:
    PrivateCommitted

conservation proof:
    ConfidentialConservation

representation relation proof:
    PublicArithmetic
```

or the corresponding explicit/confidential contradiction.

Again, record the reproduction in the eventual evidence entry but do not commit
a failing final tree.

---

## 7. Make representation relations static mode constraints

The cleanest current design is to stop treating the representation relation as
a second arithmetic-proof selection.

### 7.1 Semantic interpretation

The representation relation means:

```text
selected representation mode
    ∈
realization-approved representation modes
```

The conservation relation owns how value preservation is proved:

```text
Explicit / PublicCommitted
    → PublicArithmetic where approved

PrivateCommitted / PublicCommitted
    → ConfidentialConservation where approved
```

The representation relation should not independently select another arithmetic
proof that can contradict the conservation proof.

### 7.2 Realization declaration changes

In:

```text
packages/realization/src/declarations/compact_ash.rs
packages/realization/src/declarations/transfer_live.rs
```

change the representation relation from a proof-bearing declaration to a
relation-only declaration.

Conceptually, replace:

```rust
declaration(
    ids.representation.clone(),
    Relation::Representation { ... },
    [...proof alternatives...],
)
```

with:

```rust
relation_only(
    ids.representation.clone(),
    Relation::Representation { ... },
)
```

For compact ASH, retain the allowed modes:

```text
Explicit
PublicCommitted
```

For live transfer, retain:

```text
Explicit
PrivateCommitted
```

No mode is added or removed.

This is an internal semantic-cleanup change. It does not alter the architecture
manifest, architecture semantic hash, architecture behavioural hash, or any
generated architecture publication.

### 7.3 Compiler classification changes

In `classify_obligations`, classify these as statically validated:

```rust
Relation::LifecycleExit { .. }
Relation::Representation { .. }
```

The representation remains a compiler decision variable derived from the
relation’s allowed mode set. It simply ceases to be an independent
proof-alternative variable.

### 7.4 Keep conservation compatibility exact

`representation_conflict` must continue to enforce compatibility for
representation-sensitive `AmountConservation` relations.

For live transfer:

```text
PrivateCommitted
    + PublicArithmetic
    ⇒ reject

PrivateCommitted
    + ConfidentialConservation
    ⇒ permit

Explicit
    + PublicArithmetic
    ⇒ permit

Explicit
    + ConfidentialConservation
    ⇒ reject
```

For compact ASH:

```text
Explicit
    + PublicArithmetic
    ⇒ permit

PublicCommitted
    + PublicArithmetic
    ⇒ permit
```

Do not broaden representation support beyond what the realization declares.

### 7.5 Keep target enforcement visible

Classifying representation as static at proof-selection time must not erase its
later backend obligation.

The selected representation remains present in:

```rust
ProofPlanCandidate::representations
```

Later placement, backend, ABI, and coverage work will consume that typed choice.

Do not describe “statically validated” here as proof that a target encoded the
selected representation correctly. It means only that the compiler selected an
allowed mode and that no independent arithmetic proof variable remains on the
mode constraint.

---

## 8. T7 focused tests

### 8.1 Representation relations are static

For both pilots, assert:

```text
Relation::Representation
    → RelationObligationClass::StaticallyValidated
```

### 8.2 No representation proof variable remains

For every candidate:

```text
candidate.proofs
```

must not contain the representation relation ID.

### 8.3 Conservation proof matches selected mode

For every live-transfer candidate:

```text
PrivateCommitted
    ⇒ conservation proof is ConfidentialConservation

Explicit
    ⇒ conservation proof is PublicArithmetic
```

No invalid pairing may appear.

### 8.4 Candidate capabilities match selected proof

For every live-transfer candidate:

```text
PublicArithmetic conservation
    ⇒ ExactPublicAmountArithmetic present
    ⇒ ConfidentialValueConservation absent unless another relation requires it

ConfidentialConservation
    ⇒ ConfidentialValueConservation present
    ⇒ ExactPublicAmountArithmetic absent unless another relation requires it
```

Do not assert absence globally if another active relation legitimately requires
the capability. Compare against the candidate’s selected proof set.

### 8.5 Source requirements match selected proof

For the conservation relation:

```text
PublicArithmetic
    ⇒ AuthenticatedConsensusValue source rows

ConfidentialConservation
    ⇒ AuthenticatedCommitmentRelation source rows
```

No proof requiring exact public arithmetic may survive with only a family-census
source.

---

# Part III — Repair the independent exhaustive oracle

## 9. Oracle independence requirements

The current oracle enumerates assignments differently from production, which is
good, but it shares enough requirement derivation to reproduce T6.

Repair it so the properties under review are independently stated.

### 9.1 External evidence in the oracle

For:

```rust
Relation::SubstrateConservation { asset }
```

the oracle should independently add:

```rust
RequiredCapability::WholeTransactionValueConservation
```

and construct the matching typed external-evidence source row.

Do not call production obligation classification to learn this fact.

It may still use common stable types.

### 9.2 Representation in the oracle

Treat:

```rust
Relation::Representation
```

as a representation choice and static validity condition, not a proof variable.

The oracle’s Cartesian product should include:

```text
proof variables for proof-required relations
representation variables for allowed modes
```

but no second proof variable for the representation relation.

### 9.3 Complete-assignment validation

For every complete assignment, independently check:

1. fixed external capabilities are available;
2. selected proof capabilities are available;
3. source requirements derive successfully;
4. source/constructibility compatibility holds;
5. proof/representation compatibility holds;
6. disclosure derives and validates;
7. lifecycle requirements remain valid;
8. external evidence remains attached.

The production and oracle candidate sets must compare equal as complete typed
values.

### 9.4 New oracle regressions

Add:

- missing whole-transaction capability;
- explicit live transfer;
- private live transfer;
- invalid private/public-arithmetic pairing;
- no proof variable on the representation relation;
- combined two-pilot scope;
- random capability views including and excluding whole-transaction
  conservation.

The random capability-view test must still distinguish:

```text
production infeasible
↔
oracle feasible set empty
```

from unexpected compiler errors.

---

# Part IV — T8: Current-phase declaration consistency

## 10. Correct the current phase

The current phase is:

```text
Phase 2 — target-independent compiler analysis
```

Update `plans/README.md`, which currently identifies Phase 1.

Use one fixed declaration form. Prefer:

```text
Current: Phase 2 - target-independent compiler analysis
```

or another format that the checker can parse deterministically.

Do not add a new planning label merely for the phase value.

---

## 11. Extend the plan checker

In:

```text
packages/labels/src/plans.rs
```

extend phase consistency checking to compare:

```text
plans/backlog.md current gate
plans/roadmap.md current phase
plans/README.md current phase
exactly one active numbered phase card
```

### 11.1 Canonical comparison

Normalize each declaration to the numeric phase:

```text
2
```

The displayed title may also be compared if a fixed title registry already
exists, but do not add a second hand-maintained phase-title table merely for
this check.

The minimum required weld is numeric phase equality plus exactly one active
phase card whose filename begins with that number.

### 11.2 Focused diagnostics

A failure should identify:

- stale file;
- observed phase;
- expected phase.

Examples:

```text
phase: plans/README.md declares Phase 1 but backlog declares Phase 2
phase: plans/roadmap.md declares Phase 3 but active card is Phase 2
```

A malformed or missing current-phase declaration should fail explicitly.

### 11.3 Update test fixture

The synthetic plan fixture currently lacks a roadmap declaration and does not
exercise a root README current phase.

Update it so a valid fixture contains:

```text
plans/README.md
plans/backlog.md
plans/roadmap.md
plans/phases/README.md
plans/phases/02-pilot.md
```

or equivalent numbered fixture values.

Remember to update:

- nearest-README indexing;
- expected file counts;
- phase-card filename;
- active status;
- current-gate text.

### 11.4 Required plan-check tests

Add focused tests for:

- all declarations agree;
- stale `plans/README.md`;
- stale `plans/roadmap.md`;
- stale backlog gate;
- no active phase;
- two active phases;
- malformed current-phase declaration;
- current gate points to a missing phase card;
- traversal order does not change diagnostics.

---

# Part V — T9: Compiler status documentation

## 12. Update compiler crate-level Rustdoc

Update:

```text
packages/compiler/src/lib.rs
```

The crate-level state section must accurately say that these internal stages are
implemented:

```text
validated typed input binding
canonical scoped relation DAG
canonical scoped expression DAG
checked constant folding
proof-obligation classification
exact feasible-plan enumeration
source requirements
constructibility analysis
disclosure analysis
representation lifecycle analysis
independent exhaustive proof-search oracle
```

It must also say what remains absent:

```text
accepted proof-planning result until T6/T7 close
execution-case placement
concrete target-independent layout requirements
relation-indexed coverage requirements
complete pilot analyzed program
public complete-analysis result
compiler-plan identity
target program emission
```

Retain the strongest boundary statement:

> No public value produced by the crate today can be mistaken for a completed
> compiler analysis.

Do not expose the private analysis modules merely to make documentation easier.

---

## 13. Update the compiler package contract

Update:

```text
plans/packages/compiler.md
```

### 13.1 Status header

Replace the stale claim that only the crate boundary and error root exist.

The final status after this guide passes should say approximately:

```text
Active — input, graph, folding, source, constructibility,
disclosure, lifecycle, and exact proof-planning foundations implemented
internally; placement, layout, coverage, and complete analyzed pilots remain
open
```

### 13.2 Milestones

Mark implemented milestones accurately:

```text
crate
relations
folding
proofs
disclosure
sources
lifecycle
```

Keep these open:

```text
placement
coverage
pilots
```

Do not claim a frozen public API or stable identity.

### 13.3 Typed outputs

Clarify that the complete analyzed value is planned, while its component
analyses currently remain crate-private.

### 13.4 Open questions

Remove questions already answered by implementation where appropriate, or
rewrite them as the next actual questions:

- execution-case carrier modeling;
- placement complexity;
- target-independent layout requirements;
- coverage case generation;
- public complete-analysis boundary;
- target capability adapter ownership;
- identity activation only after a real consumer.

### 13.5 Check other stale phrases

Run:

```sh
rg -n \
  'boundary and error root only|input binding and analysis not implemented|relation DAG not implemented|constant folding not implemented|proof planning not implemented|lifecycle analysis not implemented' \
  packages plans README.md
```

Review every match. Historical records may retain historical wording when
clearly marked historical. Current-state documents must not.

---

# Part VI — Reaccept planning and update records

## 14. Update status only after tests pass

After T6–T9 are implemented and focused tests pass, update:

```text
plans/backlog.md
plans/phases/02-compiler.md
plans/packages/compiler.md
packages/compiler/README.md if any wording remains stale
packages/compiler/src/lib.rs
```

### 14.1 Backlog status changes

Change:

```text
T6  TODO → DONE
T7  TODO → DONE
T8  TODO → DONE
T9  TODO → DONE

P2-007  BLOCKED → DONE
C1-008  BLOCKED → DONE
P2-010  BLOCKED → TODO
```

Keep:

```text
P2-011  BLOCKED on P2-010
P2-012  BLOCKED on P2-010/P2-011
P2-013  BLOCKED
```

### 14.2 Current condition

The final current condition should identify the next work as:

```text
C1-009 / P2-010
execution-case placement and target-independent layout requirements
```

### 14.3 Evidence records

Each finding’s evidence entry should record:

- exact reproduction;
- exact repair;
- focused test names;
- package/workspace gate outcomes;
- whether `cargo-audit` ran or was skipped;
- whether document reproducibility ran or was deferred;
- final clean-tree result.

Do not invent command outcomes before running them.

### 14.4 Identity statement

Record:

```text
architecture identity:
    unchanged

realization public identity:
    none exists

compiler-plan identity:
    none exists

generated architecture publications:
    unchanged

public schema:
    unchanged

dependency graph:
    unchanged
```

If implementation differs from this expectation, stop and review the identity
impact before committing.

---

# Part VII — Verification

## 15. Focused development sequence

### 15.1 T6 reproduction and repair

```sh
cargo test --locked -p tripod-compiler proof
cargo test --locked -p tripod-compiler oracle
```

### 15.2 T7 realization and compiler repair

```sh
cargo test --locked -p tripod-realization
cargo test --locked -p tripod-compiler lifecycle
cargo test --locked -p tripod-compiler proof
cargo test --locked -p tripod-compiler oracle
```

### 15.3 T8 planning repair

```sh
cargo test --locked -p tripod-labels plans
scripts/check-plans.sh
```

### 15.4 T9 documentation and public boundary

```sh
cargo test --locked -p tripod-compiler
cargo doc --locked -p tripod-compiler --no-deps
scripts/check-plans.sh
```

`cargo doc` is a focused documentation build, not a substitute for tests.

---

## 16. Working Rust cadence

After the focused fixes stabilize:

```sh
cargo fmt --all
git status --short

cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Use `cargo fmt --all`, not `--check`, while working.

Read `git status` after formatting. Commit unrelated formatter repairs
separately if necessary.

---

## 17. Generated artifacts and labels

No generated architecture artifact is expected to change.

Verify:

```sh
meson compile -C build lint
```

If `build/` does not exist:

```sh
meson setup build
```

Do not create another production build directory.

If `check-generated` reports changed architecture or declassification bytes,
stop and determine why. T6/T7 should alter compiler/realization analysis only,
not architecture publications or model declassification.

The realization label register should remain unchanged unless a documentation
label was deliberately changed. Avoid label renames in this batch.

---

## 18. Full batch gate

Run once after the batch is complete:

```sh
scripts/ci.sh
meson test -C build --print-errorlogs
```

If the ordinary build graph needs compiling first:

```sh
meson compile -C build
```

Run the compile in the canonical `build/` directory only.

Record skipped lanes honestly. In particular:

```text
cargo-audit unavailable
    ⇒ skipped, not passed

document byte reproducibility not run
    ⇒ deferred, not passed
```

Because this batch should not change paper inputs, document byte
reproducibility may be reported as deferred unless release policy requires it
for the batch.

Finally run:

```sh
git diff --check
git diff --cached --check
git status --porcelain=v1 --untracked-files=all
```

The final status must be empty after commits.

---

# Part VIII — Suggested commit structure

Use small reviewable commits.

## Commit 1 — Reproduce and fix T6

Suggested scope:

```text
packages/compiler/src/proof.rs
packages/compiler/src/error.rs if needed
packages/compiler/src/tests/proof_tests.rs
```

Suggested message:

```text
compiler: retain capabilities on external evidence
```

## Commit 2 — Fix T7 representation/proof compatibility

Suggested scope:

```text
packages/realization/src/declarations/compact_ash.rs
packages/realization/src/declarations/transfer_live.rs
packages/compiler/src/proof.rs
packages/compiler/src/tests/lifecycle_tests.rs
packages/compiler/src/tests/proof_tests.rs
```

Suggested message:

```text
compiler: make representation a static mode constraint
```

## Commit 3 — Strengthen the independent planning oracle

Suggested scope:

```text
packages/compiler/src/tests/oracle_tests.rs
packages/compiler/src/tests/source_tests.rs
```

Suggested message:

```text
compiler: independently check fixed evidence requirements
```

## Commit 4 — Weld current-phase declarations

Suggested scope:

```text
plans/README.md
packages/labels/src/plans.rs
```

Suggested message:

```text
plans: weld every current phase declaration
```

No Meson census update is needed unless a new file is added. Prefer not to add
a new file.

## Commit 5 — Refresh compiler status documentation

Suggested scope:

```text
packages/compiler/src/lib.rs
plans/packages/compiler.md
plans/phases/02-compiler.md
```

Suggested message:

```text
docs: describe the implemented compiler foundation
```

## Commit 6 — Record completion

Suggested scope:

```text
plans/backlog.md
```

Suggested message:

```text
plans: reaccept exact proof planning
```

Only make this commit after the recorded commands have actually run.

---

# Part IX — Exit checklist

## T6

- [ ] Missing `WholeTransactionValueConservation` capability is reproduced.
- [ ] External-evidence obligations retain their approved proof class.
- [ ] External-evidence capabilities participate in capability filtering.
- [ ] External-evidence source requirements survive into candidates.
- [ ] Capability absence yields `NoFeasibleProofPlan`.
- [ ] External evidence remains unresolved.
- [ ] No sponsor amount is introduced.
- [ ] Independent oracle covers the fixed requirement independently.

## T7

- [ ] Contradictory representation/proof candidate is reproduced.
- [ ] Representation relations are static mode constraints.
- [ ] Representation relations no longer create arithmetic proof variables.
- [ ] Conservation proof is compatible with every selected mode.
- [ ] Candidate capabilities match selected proof semantics.
- [ ] Candidate source requirements match selected proof semantics.
- [ ] Production and independent oracle candidate sets agree.

## T8

- [ ] `plans/README.md` identifies Phase 2.
- [ ] Backlog, roadmap, README, and active phase card agree.
- [ ] Missing, stale, or malformed declarations fail.
- [ ] Exactly one phase card is active.
- [ ] Focused plan-check tests pass.

## T9

- [ ] Compiler crate-level Rustdoc reflects implemented internal stages.
- [ ] Compiler package contract reflects implemented internal stages.
- [ ] Placement, layout, coverage, and complete analyzed pilots remain open.
- [ ] No complete public analysis API is claimed.
- [ ] No compiler digest is claimed.
- [ ] Stale current-state phrases are removed.

## Reacceptance

- [ ] C1-008 is reaccepted.
- [ ] P2-007 is reaccepted.
- [ ] P2-010 becomes the next TODO.
- [ ] Architecture hashes and generated publications remain unchanged.
- [ ] No new dependency enters.
- [ ] Focused tests pass.
- [ ] Workspace format, Clippy, and tests pass.
- [ ] Documentation and lint checks pass.
- [ ] Full batch gate is recorded honestly.
- [ ] Final tree is clean.

---

# Completion report template

Use this structure when the guide is complete:

```text
Guide 4 result
==============

T6:
    reproduced:
    repair:
    focused tests:

T7:
    reproduced:
    repair:
    focused tests:

T8:
    repair:
    focused tests:

T9:
    repair:
    documentation checks:

Planning result:
    C1-008:
    P2-007:
    next task:

Identity impact:
    architecture semantic hash:
    architecture behavioural hash:
    realization identity:
    compiler identity:
    generated artifacts:

Dependency impact:
    new dependencies:
    lockfile:

Verification:
    cargo fmt --all:
    cargo clippy --workspace --all-targets --locked -- -D warnings:
    cargo test --workspace --locked:
    scripts/check-plans.sh:
    meson compile -C build lint:
    scripts/ci.sh:
    meson test -C build --print-errorlogs:
    cargo audit:
    document reproducibility:
    git diff --check:
    final git status:

Residuals:
```

---

## Next guide after this one

Once this guide passes, the next implementation guide should be:

```text
Guide 5 — Execution-Case Placement and Target-Independent Layout Requirements
```

Its scope should be exactly:

```text
C1-009
P2-010
```

It should introduce:

- typed execution-case identities;
- relation activation by case;
- eligible semantic carrier identities;
- exact relation × case coverage;
- target-independent layout requirements;
- exhaustive small-instance carrier-subset oracle;
- explicit complexity limits;
- no concrete target positions or target package dependency.

Do not begin Guide 5 until Guide 4’s proof-plan candidate set is reaccepted.
