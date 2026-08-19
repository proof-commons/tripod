# Static review

Overall, this is an unusually disciplined repository: ownership boundaries are explicit, validation states are often represented in types, non-claims are documented carefully, and semantic, target, and evidence concerns are generally kept separate.

I nevertheless found several issues. The first three affect the native-evidence boundary and should be treated as release-blocking for that subsystem.

## Findings

### 1. **P0 — An execution transcript can be rebound to fixtures and a deployment that were never executed**

**Affected code**

- `packages/target-elements-conformance/src/executor.rs`
  - `ExecutionTranscript`
  - `execute`
  - `execute_prototypes`
- `packages/target-elements-conformance/src/validate.rs`
  - `evaluate`
  - `validate_native_report`
- `packages/target-elements-conformance/src/prototype_validate.rs`
  - `evaluate_prototypes`
  - `validate_prototype_report`

`ExecutionTranscript` retains only:

```rust
handshake
environment
trust
responses keyed by NativeCaseId
prototype_responses keyed by PrototypeCaseId
```

It does **not** retain or bind:

- the reviewed target projection;
- the reviewed deployment binding;
- the exact primitive fixture projections sent to the child;
- the exact prototype matrix sent to the child.

The evaluation functions then accept those subjects again from the caller and correlate responses using only the local case ID. Consequently, this sequence is possible in principle:

```text
execute target/binding/fixtures A
    ↓
obtain transcript A
    ↓
evaluate transcript A against target/binding/fixtures B
    ↓
validate the resulting report using the same B inputs
    ↓
gate the report
```

For primitive fixtures, an external caller can construct a different fixture with the same `NativeCaseId`, a different program or context, and a compatible expected verdict. Prototype fixtures are even more directly mutable because their fields are public. The report will embed fixture B as “the complete fixture the executor was handed,” although the child was actually sent fixture A.

The same omission permits deployment rebinding. A transcript produced under binding A can be evaluated under binding B. The resulting report can contain:

```text
declared network/genesis: B
observed environment:     A
```

`validate_native_report` and `validate_prototype_report` recompute that same inconsistent report rather than rejecting the cross-binding. The gates do not compare the two environments.

This defeats the complete-subject report guarantee and can manufacture false target evidence.

**Recommended repair**

Make the transcript itself subject-bound. For example, retain typed copies of:

```rust
struct PrimitiveExecutionTranscript {
    target: TargetProjection,
    deployment: DeploymentProjection,
    requests: BTreeMap<NativeCaseId, PrimitiveFixtureProjection>,
    responses: BTreeMap<NativeCaseId, NativeExecutionResponse>,
    // handshake/environment/trust...
}
```

and an analogous prototype transcript.

Then:

1. compare the supplied target and binding against the transcript’s retained projections;
2. compare every supplied fixture or matrix row exactly against the request actually sent;
3. reject any declared/observed network or genesis mismatch again during evaluation;
4. ideally remove the second fixture/matrix argument from report evaluation altogether and consume the transcript’s bound subjects.

This does not require introducing a digest. Exact typed comparison is sufficient and consistent with ADR-016.

Add regressions for:

- same case ID, different script;
- same case ID, different initial stack;
- same case ID, different expected verdict;
- same prototype case ID, different construction;
- same transcript, different network/genesis binding;
- same transcript, changed target projection.

---

### 2. **P1 — Consensus resource cases are incorrectly credited as policy-resource evidence**

**Affected code**

- `packages/target-elements-conformance/src/validate.rs`
  - `requirements_of`
  - `bearing_requirements`
  - `evidence_rows`
  - `EVIDENCE_PLAN`
- `packages/target-elements-conformance/src/claim.rs`
  - `PolicyResourceBoundObserved`
- `packages/target-elements-conformance/src/census/encoding.rs`
  - `resource_boundaries`

The evidence mapping says:

```rust
NativeCaseGroup::Resource => &[
    TargetEvidenceRequirementId::ConsensusResourceLimits,
    TargetEvidenceRequirementId::PolicyResourceLimits,
]
```

But `bearing_requirements` receives only a `NativeCaseId`, so it cannot inspect the fixture’s `EnforcementLayer`. Every resource-group case is therefore credited to both consensus and policy evidence.

The current resource cases are stated at the consensus layer. Meanwhile, the claim registry explicitly records:

```rust
PolicyResourceBoundObserved => Unresolved(...)
```

Despite that, `PolicyResourceLimits` is classified as `Required` in `EVIDENCE_PLAN`, receives the passing consensus case statuses, and can be reported as `Passed`. Because the policy claim is marked unresolved rather than required, `missing_required_claims` does not stop the row from passing.

That creates an internally contradictory report:

```text
PolicyResourceLimits evidence row: Passed
PolicyResourceBoundObserved claim:  UnresolvedByDesign
```

It also means the gate can claim all required evidence rows passed without a policy-resource observation.

The source comments say the intended rule is to split resource evidence by enforcement layer, but the implementation cannot do that from a case ID alone.

**Recommended repair**

Derive evidence ownership from the complete fixture, not just its group:

```rust
fn bearing_requirements(
    fixture: &PrimitiveFixture,
) -> BTreeSet<TargetEvidenceRequirementId>
```

For resource cases:

```text
Consensus layer  → ConsensusResourceLimits only
RelayPolicy      → PolicyResourceLimits only
```

Until real policy-resource fixtures exist, classify `PolicyResourceLimits` as `UnresolvedByDesign`, or make its claim required and add actual relay-policy cases.

Also add a registry invariant such as:

```text
a Required evidence-plan row must own at least one required claim
```

unless there is a separately documented reason why claim-level coverage does not apply. That would prevent a broad required row from passing through case aggregation while its defining claim remains absent.

---

### 3. **P1 — The primitive native gate can accept a validated report whose summary is `Failed`**

**Affected code**

- `packages/target-elements-conformance/src/validate.rs`
  - `summarize`
  - `gate`
- Compare with:
  - `packages/target-elements-conformance/src/prototype_validate.rs`
    - `prototype_gate`

`summarize` correctly classifies a report as failed if any case failed or encountered infrastructure trouble:

```rust
let completeness = if required_passed != required.len()
    || required_claims_passed != required_claims.len()
    || cases_failed > 0
    || cases_infrastructure_error > 0
{
    ReportCompleteness::Failed
}
```

But `gate` never checks:

- `report.summary.completeness`;
- the individual `report.cases` statuses.

It checks only required claims and required evidence rows. Therefore a failed case confined to a deferred or non-required evidence class can coexist with:

```text
summary.completeness = Failed
gate(report) = Ok(())
```

This is externally constructible with a custom fixture set. For example, a failed case under a deferred evidence group can avoid every required row and required claim.

The prototype gate does not have this problem: after checking required claims, it iterates every case and rejects both `Failed` and `InfrastructureError`.

**Recommended repair**

Unless optional failures are a deliberately supported gate mode, apply the same rule in the primitive gate:

```rust
if report.summary.completeness == ReportCompleteness::Failed {
    return Err(...);
}

for row in &report.cases {
    match row.status {
        CaseStatus::Passed => {}
        CaseStatus::Failed => return Err(...),
        CaseStatus::InfrastructureError => return Err(...),
    }
}
```

If optional failing cases are intended to be admissible, then:

- `ReportCompleteness::Failed` should not describe that state;
- the gate needs an explicit typed scope saying which cases it certifies;
- the report should distinguish “failed outside gate scope” from a genuinely failed report.

The current combination—`Failed` plus successful gate—is too easy to misread as evidence success.

---

### 4. **P1/P2 — Relation bodies do not determine unique semantic relation identities**

**Affected code**

- `packages/realization/src/validate.rs`
  - `validate_relation_identity`
  - `subject_describes_body`
- `packages/realization/src/identity.rs`
  - `RelationId`
  - `RelationSubject`

The validator claims to derive the expected relation identity from the relation body. The kind is uniquely derived, but several subject checks admit multiple different IDs for the same body.

Examples:

#### `AllowedObjectFamilies`

Any allowed object can be used as the subject:

```rust
Relation::AllowedObjectFamilies { side, allowed } => match subject {
    Subject::ObjectFamily { side: declared, object } =>
        *declared == transaction_side(*side) && allowed.contains(object),
    _ => false,
}
```

A closure over:

```text
{ ASH, PLAIN_LBTC }
```

can therefore be identified as either the ASH relation or the PLAIN_LBTC relation without changing its semantics.

#### `RootPolicy`

The same root-policy body may be identified by:

```text
RelationSubject::Operation
```

or by any root occurring in the policy.

#### `ProjectionPolicy`

Likewise, it may use the operation subject or any governed projection.

#### `CanonicalDeltaPolicy` and `OpenFlowPolicy`

These admit:

```rust
Subject::Operation | Subject::Projection { .. }
```

without checking which projection was named. The same body can therefore be filed under `TransitionCertificate`, `BurnEvent`, `ClearEvent`, or another projection.

This breaks the “complete typed key” property: presentation of the key can move while the semantic body remains identical. Downstream compiler coverage, provenance, diagnostics, and any future identity can then be filed under an arbitrary subject.

**Recommended repair**

Make subject derivation a function, not a predicate admitting several alternatives.

For operation-wide policies, use only:

```rust
RelationSubject::Operation
```

For side-wide closure, add a subject representing the side itself, for example:

```rust
RelationSubject::TransactionSide {
    side: TransactionSide,
}
```

Alternatively, define one canonical owner family by explicit rule, but that would be less clear than representing the actual subject.

The validator should compute exactly one expected `(kind, subject)` pair and compare equality. Add mutation tests proving that changing only the relation subject fails derivation.

---

### 5. **P2 — Committed planning records use bare report digests despite the recorded “no report identity” decision**

**Affected documents**

- `plans/research/state-constructor.md`
  - records a concrete “report digest”
- `plans/research/wide-arithmetic.md`
  - records a concrete “report digest”
- `plans/backlog.md`, Guide-10 gate record
  - says report digests were recorded while report artifacts were not checked in
- `plans/registers/identities.md`, §3.5
  - records the decision not to mint a native conformance report identity
- `packages/target-elements-conformance/README.md`
  - says no report digest exists and none may be added there

The two research documents persist SHA-256 values as the only durable locators for reports that are not retained in the repository. That makes the values function as bare report identities, even though the identity register says the proposal stopped because reports are compared by typed content and exact bytes in-process.

The result has the weakest form ADR-016 warns against:

```text
bare digest
no retained report bytes
no typed report reference
no package-owned recipe record
no consumer capable of revalidation from the repository
```

The surrounding prose supplies some role and provenance context, but the digest itself cannot recover or validate the report and conflicts with the explicit stopped outcome.

**Recommended repair**

Choose one coherent state:

1. **No persistent report identity:** remove the committed report hashes and characterize the runs as historical narrative evidence only; or
2. **Persistent evidence identity:** retain the exact report asset and admit a typed report reference under ADR-016, binding:
   - report role;
   - schema;
   - target and deployment subjects;
   - fixture/matrix census;
   - executor provenance;
   - result;
   - recipe identifier and stale conditions.

Do not leave “report artifact absent, bare digest retained” as the middle state.

---

### 6. **P2 — The active backlog materially contradicts the current implementation state**

**Affected file**

- `plans/backlog.md`

Several current-state statements still describe work that later sections record as complete:

- The opening “Current condition” says the next work is the Guide-8 target foundation, although the same backlog records Guide 8, Guide 9, and Guide 10 as complete.
- §3.3 lists the STATE-constructor and wide-arithmetic prototypes as not implemented, while §2.14 and the corresponding research notes record both as accepted.
- The one-line backlog still says to settle STATE constructor, wide arithmetic, and declassification; only declassification remains open.
- The checker findings table contains two different `DI-F02` rows, one `DONE` and one `ACTIVE`.
- `DI-F03` remains `ACTIVE` even though the DI-003 narrative says the corresponding import and participation work was completed.

This matters because the backlog declares itself the authority for current task ordering. A reader following its opening state is sent backwards by several completed guides.

**Recommended repair**

Reconcile the top-level state to the current gate:

```text
Current: Phase 3
Completed: target foundation, native primitive gate,
           constructor prototype, wide-floor prototype
Open blocker: public declassification
Next gate: Phase 4 after Phase-3 exit
```

Give the duplicate DI findings distinct permanent IDs or delete the obsolete duplicate if Git history is the intended archive.

It would also be useful for `check-plans` to enforce:

- uniqueness of finding/task IDs in the backlog;
- no `ACTIVE` finding beneath a parent task marked `DONE`, unless explicitly designated residual;
- agreement between “not implemented,” current blockers, and gate records for named milestones.

---

### 7. **P2/P3 — Process-group establishment failure can leave the direct child unreaped**

**Affected code**

- `packages/target-elements-conformance/src/executor.rs`
  - `ExecutorSupervisor::adopt`
  - `execute_workload`

The child is spawned before `ExecutorSupervisor::adopt` establishes that it leads the expected process group:

```rust
let supervisor = match ExecutorSupervisor::adopt(child) {
    Ok(supervisor) => Arc::new(supervisor),
    Err(error) => {
        return Err(error);
    }
};
```

`adopt` takes ownership of `Child`. If `SupervisedGroup::establish` fails, the `Child` is dropped. Rust’s `Child` destructor does not kill or wait for the process.

The common race is likely an already-exited child, which still risks leaving a zombie until the harness exits. On an unexpected live-group-establishment failure, the direct child can continue running despite the run having been refused.

This is not a sandbox issue and does not contradict ADR-015’s non-containment claim. It is narrower: the harness should clean up the process it directly started on every startup failure path.

**Recommended repair**

Establish the group while retaining mutable ownership of the child, and on failure:

```text
kill direct child
wait/reap direct child
return ExecutorProcessGroupUnavailable
```

A scope guard around the spawned child would make every pre-supervisor return path cleanup-safe.

Add a subprocess regression that forces establishment failure or an immediate child exit and verifies no unreaped child remains.

---

## Additional observations

### Generated architecture consistency

The realization appendix and generated TOML shown in the supplied material agree on the current major identities:

- architecture schema 17;
- realization major 13;
- semantic algorithm `sha256-canonical-json-v3`;
- behavioural algorithm `sha256-canonical-json-behavioural-v3`;
- current semantic, behavioural, and attestation anchor-set values.

I did not independently regenerate or hash them.

### Strong design choices

Several parts deserve explicit credit:

- Architecture and deployment identity APIs require validated wrappers.
- The production-release/profile distinction is represented truthfully; schema 2 cannot accidentally become a production release.
- The reviewed target trust state is distinct from generic target validation.
- Target static contracts, deployment declarations, and native observations are separate types.
- Unknown fields are generally rejected at external serialization boundaries.
- The abstract stack validator preserves success, non-aborting failure, and abort outcomes separately.
- Sponsor-value opacity is represented structurally rather than by best-effort redaction.
- The repository is unusually careful not to turn Petgraph indices, paths, timestamps, or traversal order into semantic identity.
- First-party unsafe code remains denied.

## Suggested repair order

1. Bind `ExecutionTranscript` to the exact target, deployment, and request subjects.
2. Correct resource evidence attribution by enforcement layer.
3. Make the primitive gate reject reports classified as failed.
4. Make relation identity derivation unique.
5. Reconcile report-digest policy and retained evidence.
6. Repair current backlog drift and duplicate finding IDs.
7. Close the process-group adoption cleanup path.

## Review scope and verification status

This was a static review of the supplied 220-file concatenation at tree:

```text
0.3.4-dev
```

The filter excluded 352 files, notably:

- `Cargo.lock`;
- most unit and integration tests;
- most compiler implementation files;
- most executable-model implementation files;
- several build/tooling packages and scripts;
- paper sources.

I did **not** run Cargo, Meson, the native executor, document reproducibility, or advisory checks. Therefore this review makes no green-build, test-coverage, lockfile, licensing, or advisory-status claim.

# Second static review

This second pass reviewed the same supplied tree:

```text
0.3.4-dev
```

Because the tree is unchanged, the findings from the first review remain applicable unless separately disproved. This pass concentrated on a different question:

> Does the native-evidence layer prove that the executed program actually bears on the claim under which the report files it?

The answer is currently **no** for both primitive and compound-prototype reports. The exact report-census validation is strong, but it validates a caller-selected fixture census against itself. It does not establish that the selected fixtures are the canonical evidence subjects or that their claim metadata describes their scripts.

The two findings below are more fundamental than report-row deletion or transcript rebinding: even a perfectly subject-bound transcript from an honest real node can be turned into false evidence if the fixture itself is allowed to assign an unrelated claim to a trivial program.

---

## Findings

## 1. **P0 — Primitive native evidence can be manufactured by attaching claim-bearing case metadata to an unrelated script**

### Affected code

- `packages/target-elements-conformance/src/fixture.rs`
  - `PrimitiveFixture::new`
  - `PrimitiveFixture::state`
  - `NativeCaseId`
- `packages/target-elements-conformance/src/claim.rs`
  - `claims_of`
  - `primitive_claims`
  - `numeric_claims`
  - `authorization_claims`
  - `boundary_claims`
- `packages/target-elements-conformance/src/validate.rs`
  - `requirements_of`
  - `bearing_requirements`
  - `evaluate`
  - `validate_native_report`
  - `gate`
- `packages/target-elements-conformance/src/executor.rs`
  - `execute`

### Problem

`PrimitiveFixture::new` accepts independently:

```rust
case: NativeCaseId
program: &TapscriptProgram
expected: ExpectedPrimitiveOutcome
```

Nothing verifies that:

- the opcode named by `case.opcode()` occurs in `program`;
- the program exercises the semantic dimension named by `case.group()`;
- the expected outcome is a contract-derived outcome for that program;
- the fixture is a member of the repository’s canonical fixture census.

The evidence claims are then derived primarily from the case metadata and expected verdict, not from the program that was executed.

For example, `authorization_claims` effectively reasons as follows:

```text
case group is Signature
+
case opcode is CheckSig
+
expected/observed verdict is accepting
=
TransactionSignatureAccepted
```

It does not inspect whether the program actually contains `CheckSig`.

Likewise, `bearing_requirements` maps the case group and optional opcode to broad evidence requirements. It does not receive the fixture or program:

```rust
fn bearing_requirements(case: NativeCaseId) -> Vec<TargetEvidenceRequirementId>
```

### Concrete false-evidence shape

A caller can construct a typed program that merely pushes a true value:

```text
PUSH 1
```

and file it under a case such as:

```text
group:  Signature
opcode: CheckSig
expected: Accept
```

The real target will accept the trivial script. The report can then derive:

```text
PrimitiveSuccessObserved
TransactionSignatureAccepted
SignatureSemantics
OpcodeSemantics
```

even though no signature opcode executed and no transaction signature was checked.

The same pattern applies broadly:

- arithmetic claims can be attached to a script unrelated to arithmetic;
- curve claims can be attached to a simple true script;
- rejection claims can be attached to any script chosen to abort;
- timelock claims can be attached to a rejection that never evaluated a timelock;
- opcode semantics can be credited whenever `NativeCaseId` names an opcode, whether or not the script contains it.

By constructing enough such fixtures, a caller can satisfy the required claim census against an honest nonmock target executor.

### Why current report validation does not stop it

`validate_native_report` recomputes:

- the fixture projection;
- claims derived from the same fixture metadata;
- case status;
- evidence rows;
- summary.

That proves the report faithfully describes the caller-selected fixture set. It does **not** prove that the fixture set is the canonical evidence plan or that the metadata accurately describes the executed script.

The validation is internally exact but semantically circular:

```text
caller labels fixture as signature evidence
    ↓
claims_of reads caller’s label
    ↓
report records signature evidence
    ↓
validator recomputes from caller’s same label
```

### Impact

This can manufacture false target evidence without compromising the target node or the executor. The node may behave perfectly; it is simply asked an easier question than the report claims it answered.

That falls directly within the repository’s P0 definition: false evidence can be created at a trusted evidence boundary.

### Recommended repair

Introduce separate trust states for arbitrary experiments and the canonical primitive plan.

For example:

```rust
pub struct PrimitiveFixtureSet {
    // arbitrary, useful for tests and experiments
}

pub struct CanonicalPrimitiveFixtureSet {
    fixtures: PrimitiveFixtureSet,
    // no public constructor
}
```

Only:

```rust
canonical_fixture_set(&target, &binding)
```

should construct the canonical wrapper.

Then make the evidence-bearing path require it:

```rust
execute_canonical(
    target,
    binding,
    configuration,
    fixtures: &CanonicalPrimitiveFixtureSet,
) -> Result<CanonicalExecutionTranscript, _>;

evaluate_canonical(
    transcript: &CanonicalExecutionTranscript,
    plan: &EvidencePlan,
    registry: &ClaimRegistry,
) -> Result<NativeConformanceReport, _>;

gate(
    validated: &ValidatedCanonicalNativeConformanceReport,
) -> Result<(), _>;
```

Arbitrary fixture sets may retain a report path, but the result should have an explicitly non-evidence role such as:

```text
ExperimentalPrimitiveReport
AdHocExecutionReport
```

and must not reach the native gate.

Also consider deriving claim ownership from a closed canonical case registry rather than from caller-supplied group/opcode metadata. At minimum, canonical validation should compare the complete fixture projection against the independently regenerated canonical fixture set.

A check that the named opcode merely appears in the script would be useful but insufficient. A script can contain an irrelevant opcode or execute it in an irrelevant context. Evidence-bearing membership must be defined by the canonical fixture registry or another independently validated typed case declaration.

### Required regressions

- A true literal program labeled as `CheckSig` must not satisfy signature evidence.
- A true literal program labeled as `EcMulScalarVerify` must not satisfy curve evidence.
- A `Verify(false)` program labeled as arithmetic overflow must not satisfy arithmetic-failure evidence.
- An arbitrary fixture set must be reportable only as experimental.
- Only the exact regenerated canonical fixture census may produce a gate-eligible report.
- Permuting canonical fixture declaration order should remain harmless.
- Changing any canonical fixture’s script, stack, context, expected result, layer, or case metadata must remove gate eligibility.

---

## 2. **P0 — Prototype gates trust caller-authored claim sets and can certify a trivial true script as constructor or wide-floor evidence**

### Affected code

- `packages/target-elements-conformance/src/prototype.rs`
  - `CompoundPrototypeFixture`
  - `PrototypeClaim`
  - `CompoundPrototypeFixture::defect`
  - `constructor_case_matrix`
  - `wide_floor_case_matrix`
- `packages/target-elements-conformance/src/prototype_validate.rs`
  - `evaluate_prototypes`
  - `claim_rows`
  - `validate_prototype_report`
  - `prototype_gate`
- `packages/target-elements-conformance/src/executor.rs`
  - `execute_prototypes`
- `packages/target-elements-conformance/src/prototype_report.rs`

### Problem

`CompoundPrototypeFixture` exposes all of its fields publicly:

```rust
pub struct CompoundPrototypeFixture {
    pub case: PrototypeCaseId,
    pub claims: BTreeSet<PrototypeClaim>,
    pub target_contract_version: u32,
    pub script: Vec<u8>,
    pub initial_stack: Vec<Vec<u8>>,
    pub construction: PrototypeConstruction,
    pub expected: ExpectedPrototypeOutcome,
    pub expected_resources: ExpectedResourceObservation,
}
```

A caller directly chooses:

- the program bytes;
- the case identity;
- every claim the case purports to bear on;
- the expected verdict;
- the construction.

`CompoundPrototypeFixture::defect` verifies useful local coherence:

- target revision;
- claim relation ownership;
- executing leaf shape;
- leaf script equality;
- tree membership;
- predecessor program consistency;
- optional control block consistency;
- required output-role cardinality.

But it does **not** verify that:

- a constructor-continuity fixture executes `PrototypeProgram::continuity`;
- a wide-floor fixture executes `PrototypeProgram::wide_floor`;
- the fixture belongs to the canonical constructor or wide-floor matrix;
- the selected claims are the claims assigned to that canonical case;
- the successor output program is a successor constructor derived from the stated transition;
- the script has any relation to the claims beyond the caller saying it does.

### Concrete constructor forgery

A caller can create:

```text
script:
    PUSH 1

tree:
    one leaf containing PUSH 1

predecessor program:
    correctly derived from that trivial tree

successor output:
    one arbitrary nonempty program

claims:
    all nine MetadataConstructorContinuity claims

expected:
    Accepted
```

The fixture can pass `defect`:

- the tree really contains the executing leaf;
- the predecessor program really matches the tree;
- there is exactly one successor output;
- all nine claims belong to the constructor relation.

A real node accepts the true script. `evaluate_prototypes` then credits all nine claims. `prototype_gate` sees:

```text
all required claims passed
all cases passed
```

and accepts the run as constructor-continuity evidence, although the continuity prototype never executed.

### Concrete wide-floor forgery

The same construction is simpler for wide floor:

```text
script:
    PUSH 1

tree:
    bare leaf containing PUSH 1

outputs:
    none

claims:
    all eleven WideFloor claims

expected:
    Accepted
```

A wide-floor relation requires no output role, so this can be locally coherent. The node accepts it, and the report can claim that exact division, nonzero remainder handling, limb derivation, under-quotient rejection, over-quotient rejection, domain rejection, encoding rejection, witness ordering, and success-flag handling were all established.

### Why validation does not help

`validate_prototype_report` recomputes from the same caller-supplied matrix. It validates that the report agrees with that matrix, not that the matrix is the repository’s canonical matrix.

The claims themselves are not derived independently. They are copied from:

```rust
fixture.claims
```

The bearing-case map is therefore an inversion of caller assertions:

```text
fixture says “I bear on claim X”
    ↓
claim_rows credits this fixture to X
```

This is exactly the broad-evidence self-assertion that the primitive claim registry was intended to avoid.

### Impact

A real executor and a real node can produce a fully gate-accepted but semantically false constructor or wide-floor report.

The target is not being spoofed. The evidence subject is.

### Recommended repair

Create relation-specific validated matrix wrappers with no public constructors:

```rust
pub struct ValidatedConstructorMatrix {
    rows: Vec<CompoundPrototypeFixture>,
}

pub struct ValidatedWideFloorMatrix {
    rows: Vec<CompoundPrototypeFixture>,
}
```

Only the canonical matrix generators should return them:

```rust
constructor_case_matrix(&target)
    -> Result<ValidatedConstructorMatrix, ConstructorMatrixDefect>;

wide_floor_case_matrix(&target)
    -> Result<ValidatedWideFloorMatrix, WideFloorMatrixDefect>;
```

The gate-eligible executor and report paths should accept only those wrappers.

Alternatively, `validate_prototype_report` can independently regenerate the canonical matrix for the report’s relation and compare the complete matrix exactly, including:

- script;
- stack;
- construction;
- expected result;
- claims;
- expected resources;
- case order and census.

Do not let the gate accept an arbitrary `&[CompoundPrototypeFixture]`.

Claims should also be derived from a closed typed case identity or canonical registry, not supplied as an arbitrary public `BTreeSet`. If ad hoc compound fixtures remain useful, give them a separate report role that can never become complete prototype evidence.

Finally, the report currently marks every compound script as:

```rust
FixtureScriptSource::TypedProgram
```

even though `CompoundPrototypeFixture.script` is public raw bytes and `defect` does not prove the bytes came from a typed `TapscriptProgram`. That field should either be derived from an unforgeable typed program wrapper or omitted from arbitrary compound reports.

### Required regressions

- `PUSH 1` carrying all constructor claims must not be gate-eligible.
- `PUSH 1` carrying all wide-floor claims must not be gate-eligible.
- A canonical case with one added claim must fail.
- A canonical case with one removed claim must fail.
- Replacing the canonical prototype program while retaining the same case name must fail.
- Replacing the constructor successor program with an arbitrary nonempty program must fail canonical-matrix validation.
- An ad hoc coherent compound fixture may produce an experimental report, but never a gate-eligible one.
- The report must not claim `TypedProgram` provenance for caller-supplied raw bytes.

---

## 3. **P1 — Native and prototype gates do not enforce the ADR-018 execution-provenance requirements**

### Affected code

- `packages/target-elements-conformance/src/report.rs`
  - `ExecutorProvenance`
  - `ExecutorProvenance::establishes_workspace_provenance`
- `packages/target-elements-conformance/src/validate.rs`
  - `gate`
- `packages/target-elements-conformance/src/prototype_validate.rs`
  - `prototype_gate`
- `packages/target-elements-conformance/src/protocol.rs`
  - `ExecutorHandshake`
- `adr/018-upstream-elements-workspace.md`

### Problem

The report type contains the fields ADR-018 requires:

```rust
binary_reported_revision
intended_executed_tip
upstream_base
included_local_topics
```

It even provides:

```rust
pub const fn establishes_workspace_provenance(&self) -> bool
```

But neither evidence gate calls it.

The primitive gate checks:

1. executor declaration is not `Mock`;
2. required claims pass;
3. required evidence rows pass.

The prototype gate checks:

1. executor declaration is not `Mock`;
2. the matrix is nonempty;
3. required claims pass;
4. every case passes.

Neither rejects:

- absent binary revision;
- absent intended tip;
- absent upstream base;
- blank provenance strings;
- binary revision disagreeing with the intended tip;
- a topic census inconsistent with the intended merged workspace;
- a provenance claim that identifies only a checkout rather than the binary.

ADR-018 states a stronger rule:

> A gate or evidence record states the executed tip, the upstream base, and the census of local branches included. A binary whose embedded revision does not match the intended tip is refused for evidence and rebuilt.

The report schema can express that state, but the evidence gate does not enforce it.

### Additional issue

`establishes_workspace_provenance` checks only whether three options are `Some`:

```rust
self.binary_reported_revision.is_some()
    && self.intended_executed_tip.is_some()
    && self.upstream_base.is_some()
```

Therefore these values qualify:

```rust
Some("")
Some("not a revision")
Some("different revision")
```

No semantic handshake validator checks nonblank names or revision syntax.

### Impact

A report can satisfy the native evidence gate while explicitly recording that the binary’s source provenance was not established.

That weakens the report from:

```text
this exact reviewed executable produced these observations
```

to:

```text
some caller-declared nonmock executable produced these observations
```

The latter may still be useful as an experimental run, but it does not satisfy ADR-018’s gate requirements.

### Recommended repair

Add an explicit typed provenance policy to the executor configuration or report-validation inputs:

```rust
pub struct ExpectedExecutorProvenance {
    pub intended_tip: RevisionId,
    pub upstream_base: RevisionId,
    pub included_local_topics: BTreeSet<TopicName>,
}
```

Then validate:

- adapter and node names are nonblank;
- required revision fields are nonblank and syntactically valid;
- the binary-reported revision equals the expected intended tip, under one documented exact or full-prefix rule;
- the upstream base equals the expected base;
- the local-topic census equals the expected census;
- no checkout-derived fallback stands in for binary provenance.

Return a validated provenance wrapper and require it in gate-eligible reports.

A dishonest executor can still lie. The repository already documents that non-claim. This repair closes the honest mismatch and omission cases ADR-018 explicitly assigns to the gate.

### Required regressions

- Missing binary revision fails the evidence gate.
- Missing intended tip fails.
- Missing upstream base fails.
- Empty strings fail.
- Binary revision differing from intended tip fails.
- Wrong topic census fails.
- Correct full revision passes.
- An explicitly scoped experimental run may retain incomplete provenance, but must not reach an evidence gate.

---

## 4. **P1 — Meson silently declares an executor “reviewed nonmock” by default**

### Affected code

- `meson.options`
  - `target_native_executor_class`
- `meson.build`
  - native and prototype targets
- `packages/target-elements-conformance/src/bin/check-target-elements-native.rs`
- `packages/target-elements-conformance/src/bin/check-target-elements-prototypes.rs`

### Problem

The command-line checkers require an explicit executor class:

```text
--executor-class mock | reviewed-non-mock
```

But Meson defines:

```meson
option(
  'target_native_executor_class',
  type: 'combo',
  choices: ['mock', 'reviewed-non-mock'],
  value: 'reviewed-non-mock',
)
```

Therefore a caller who sets only:

```text
-Dtarget_native_executor=/path/to/program
```

has not explicitly classified the executor, yet Meson tells the checker:

```text
reviewed-non-mock
```

This is a fail-open trust default.

A wrapper around `mock-native-executor` can exploit the default:

- the primitive mock can echo every fixture’s expectation;
- the prototype mock can materialize the package oracle’s construction and echo every matrix expectation;
- the gate sees `ReviewedNonMock`, because the transcript trust comes from the Meson-supplied declaration;
- the handshake’s mock-looking adapter name is recorded but not rejected.

The repository correctly documents that it cannot authenticate an executor and that a mock falsely declared reviewed remains a mock. But the build system should not make that false declaration on the caller’s behalf.

### Recommended repair

Use a three-state option:

```text
unselected
mock
reviewed-non-mock
```

with `unselected` as the default.

If `target_native_executor` is nonempty while the class is `unselected`, fail configuration with a message requiring an explicit classification.

A less desirable but still fail-closed alternative is to default the class to `mock`, so a caller must opt into evidence status deliberately.

The network and genesis options should follow the same principle: if the native target is enabled, all required binding values should be explicitly selected and validated at configuration time rather than failing later in the command.

### Required regressions

- Executor path with unselected class fails configuration.
- Mock path with explicit `mock` runs but fails the evidence gate.
- Reviewed path with explicit `reviewed-non-mock` reaches the gate.
- No executor path defines no native target, as today.
- A mock wrapper cannot satisfy a Meson gate through option defaults.

---

## 5. **P1/P2 — Target contract V1 is advertised as supported, but validation applies the V2 census and algebra to it**

### Affected code

- `packages/target-elements/src/definition.rs`
  - `TargetContractVersion`
  - `TargetContractVersion::SUPPORTED`
  - `validate_target_definition`
  - `validate_opcodes`
  - `validate_encodings`
- `packages/target-elements/src/opcode.rs`
  - `OpcodeId::ALL`
- `packages/target-elements/src/capability.rs`
  - `ElementsCapability::ALL`
- `packages/target-elements/README.md`

### Problem

The package says:

```rust
pub const V1: Self = Self(1);
pub const V2: Self = Self(2);
pub const SUPPORTED: &'static [Self] = &[Self::V1, Self::V2];
```

The documentation says V1 remains the historical Guide-9 contract and “is not widened.”

But validation is not version-dispatched. It applies the current global censuses:

```rust
for id in OpcodeId::ALL {
    if !definition.opcodes.contains_key(id) {
        errors.push(TargetError::MissingOpcodeContract(*id));
    }
}
```

`OpcodeId::ALL` includes the seventeen V2 compound-proof primitives.

The same general problem applies to current capability and evidence censuses and to the V2-expanded operand/success algebra: they are used regardless of the offered target contract version.

Consequently:

- a real historical V1 definition lacking the V2 primitives is rejected as incomplete; or
- a definition containing the V2 census can be stamped `V1` and accepted generically.

The version therefore does not identify the contract shape it claims to identify.

### Impact

The generic validated-target boundary can produce a contract whose version field says V1 while its semantic content is V2. Conversely, the actual historical V1 contract cannot pass the current validator.

The reviewed first-party wrapper remains protected because the built-in declaration is V2 and `validate_as_reviewed_elements` uses exact typed equality. The defect is in the public generic version/validation contract.

### Recommended repair

Choose one of two honest states.

#### Option A: remove V1 support

If no current consumer needs V1:

```rust
pub const SUPPORTED: &'static [Self] = &[Self::V2];
```

Make `TargetContractVersion::supported(1)` fail.

Retain V1 only in historical documentation.

#### Option B: implement version-specific schemas

Define per-version censuses and validation:

```rust
fn required_opcodes(version: TargetContractVersion) -> &'static [OpcodeId];
fn required_capabilities(version: TargetContractVersion) -> &'static [ElementsCapability];
fn required_evidence(version: TargetContractVersion) -> &'static [TargetEvidenceRequirementId];
```

If V1’s operand and success algebra cannot be represented faithfully by current types without silent widening, it needs a versioned DTO or migration boundary rather than reuse of the V2 type under a V1 number.

### Required regressions

- A true historical V1 contract validates if V1 remains supported.
- Adding a V2-only primitive to V1 is rejected or explicitly versioned.
- Removing a V2 primitive from V2 is rejected.
- A V2 body stamped V1 is rejected.
- `TargetContractVersion::SUPPORTED` contains only revisions for which a complete accepted definition can actually be constructed.

---

## 6. **P2 — The signature cross-contract weld ignores the unknown-public-key rule and much of the success algebra**

### Affected code

- `packages/target-elements/src/authorization.rs`
  - `SignaturePrimitiveContract`
  - `UnknownPublicKeyTypeRule`
- `packages/target-elements/src/opcode.rs`
  - signature opcode contracts
- `packages/target-elements/src/operand.rs`
  - `OperandContract::Signature`
  - `OperandContract::PublicKey`
- `packages/target-elements/src/success.rs`
  - signature success conditions
- `packages/target-elements/src/weld.rs`
  - `weld_signature`

### Problem

`SignaturePrimitiveContract` independently states:

```rust
unknown_public_key_type: UnknownPublicKeyTypeRule
```

The opcode contracts independently state unknown-key behavior through:

- `OperandContract::PublicKey { unknown_nonempty_allowed }`;
- `SuccessCondition::UnknownKeyTypeUnverified`.

`weld_signature` verifies:

- empty-signature failure outcome;
- invalid-signature failure outcome;
- validation budget;
- named operand encodings;
- evidence overlap.

It never reads:

```rust
signature.unknown_public_key_type()
```

It also does not require the opcode success contract to contain exactly the expected pair:

```text
RecognizedKeyVerifiedSignature
UnknownKeyTypeUnverified
```

or require the operand to admit unknown nonempty key forms.

### Concrete contradictory definition

A caller can reconstruct a generic target definition whose signature subcontract says:

```rust
UnknownPublicKeyTypeRule::Rejected
```

while its opcode contracts still contain:

```rust
SuccessCondition::UnknownKeyTypeUnverified
```

and an operand allowing unknown nonempty keys.

Nothing in `weld_signature` detects the contradiction.

Likewise, the public-key operand can be narrowed to an exact recognized encoding while the success contract still advertises unknown-key success, provided the remaining local shape checks stay valid.

This is a direct recurrence of the cross-view contradiction that the weld layer was introduced to prevent.

### Impact

A `ValidatedTargetDefinition` can be internally contradictory about one of the target’s sharpest authorization behaviors: whether an unknown key form rejects or succeeds without verifying a signature.

The reviewed first-party wrapper remains protected by exact equality, but the generic “validated” state does not establish the coherence it claims.

### Recommended repair

Extend `weld_signature` to derive one complete expected signature behavior from `SignaturePrimitiveContract` and compare every signature opcode against it.

Check at least:

- signature operand admits empty iff the subcontract says empty is a semantic path;
- nonempty signature encoding matches;
- public-key recognized encoding matches;
- unknown nonempty key admission matches `UnknownPublicKeyTypeRule`;
- recognized-key success case exists exactly once;
- unknown-key unverified success case exists iff the rule permits it;
- branching forms push exactly one Boolean;
- verifying forms push nothing;
- empty-signature failure differs correctly between branching and verifying forms;
- invalid signature aborts only on the recognized-key path;
- empty public key remains a rejection distinct from unknown nonempty key.

### Required regressions

- Change only `unknown_public_key_type` to `Rejected`; validation must fail.
- Remove `UnknownKeyTypeUnverified`; validation must fail.
- Set `unknown_nonempty_allowed = false`; validation must fail.
- Remove empty-signature admission while retaining the empty failure; validation must fail.
- Change a branching signature success result from Boolean to empty; validation must fail.
- The reviewed contract remains valid.

---

## 7. **P3 — Constructor nonce retry retries defects that its own contract says cannot be repaired**

### Affected code

- `packages/target-elements-conformance/src/constructor/canonical.rs`
  - `construct_canonically_ordered`
- `packages/target-elements-conformance/src/constructor/totality.rs`
  - `construct_under_policy`
- `packages/target-elements-conformance/src/constructor/curve.rs`
  - `PointDecodingDefect`
- `packages/target-elements-conformance/src/constructor/tree.rs`
  - `ConstructionDefect`
  - `TweakDefect`

### Problem

Both retry implementations document that defects unaffected by the metadata nonce must fail immediately.

The documentation specifically names:

- an executing leaf absent from the tree;
- an invalid internal key that is not a curve point.

But both implementations short-circuit only tree defects.

In `construct_canonically_ordered`:

```rust
match construct(...) {
    Ok(output) => return Ok(...),
    Err(defect @ ConstructionDefect::Tree(_)) => {
        return Err(CanonicalOrderDefect::NotRepairableByRetry(defect));
    }
    Err(_) => {}
}
```

In `construct_under_policy`:

```rust
match construct(...) {
    Ok(output) => return Ok(...),
    Err(defect @ ConstructionDefect::Tree(_)) => {
        return Err(TotalityDefect::NotRepairableByRetry(defect));
    }
    Err(defect) => last = Some(defect),
}
```

An internal key failing:

```rust
TweakDefect::InternalKeyNotOnCurve(...)
```

is invariant under every metadata nonce. Nevertheless, the code retries until exhaustion and then reports `SearchExhausted` or `RetryExhausted`, contrary to the documented error classification.

### Impact

- unnecessary potentially large work;
- misleading diagnostic;
- the retry policy’s typed distinction between transient and permanent failure is not upheld;
- a caller may respond by raising the retry limit to a failure no number of retries can repair.

This does not compromise the canonical constructor currently using the fixed published internal key, but it affects the public generic constructor functions and their stated contract.

### Recommended repair

Classify repairability explicitly:

```rust
fn retryable(defect: ConstructionDefect) -> bool {
    matches!(
        defect,
        ConstructionDefect::Tweak(TweakDefect::TweakNotAScalar)
            | ConstructionDefect::Tweak(TweakDefect::TweakedKeyIsIdentity)
    )
}
```

Treat as nonretryable:

```text
every TreeDefect
InternalKeyNotOnCurve
```

Be careful not to mark every tweak failure permanent: the tweak scalar and tweaked identity can change when the metadata leaf, root, and tweak change with the nonce.

Use the same helper in both retry implementations so their policies cannot drift.

### Required regressions

- Invalid internal key fails after one attempt as nonretryable.
- Missing executing leaf fails after one attempt.
- Repeated executing leaf fails after one attempt.
- Invalid tweak scalar may retry.
- Tweaked-key identity may retry.
- Exhaustion reports exactly the configured attempt count.
- Canonical current fixtures continue to produce identical outputs.

---

# Additional hardening observations

These are lower confidence or lower priority than the findings above, but they deserve focused review during remediation.

## A. Opcode resource stack-growth fields are not generally welded to stack behavior

`OpcodeResourceCost::maximum_stack_growth` is derivable from:

- every success effect;
- every non-aborting failure effect.

The generic target validator does not appear to compare the declared resource growth against those effects, except for special-case checks such as the timelock weld.

A caller-built target definition can therefore potentially remain “validated” while claiming a stack-growth bound inconsistent with its own stack contract.

A generic weld should derive the maximum over:

```text
all success depth changes
all ConsumeOperandsPushFalse depth changes
all RetainOperandsPushFalse depth changes
```

and compare it to the resource row.

The reviewed wrapper’s exact-equality boundary protects the built-in target, but the generic validated state should still be internally coherent.

## B. Future realization issuance observations carry authority fields that the evaluator currently ignores

`ObservedIssuance` carries:

```rust
asset
authority
authority_input
amount
destinations
```

But `canonical_delta_policy_holds` validates the amount and destinations and does not appear to verify:

- the expected authority asset;
- authority-input object/family;
- uniqueness of authority use;
- non-overlap of an authority input with another role;
- correspondence with an observed root effect.

The current Phase-1 realization scope contains no issuance operation, so this is not an active pilot defect. It is a future-scope blocker: issuance-bearing realization declarations should not be added until the observation evaluator enforces the authority relation or carries it as explicit external evidence.

---

# Positive observations from the second pass

The issues above are not caused by generally weak engineering. In fact, they stand out because the surrounding design is strong.

Particularly good:

- Report validation compares complete rows and catches duplicate, missing, unexpected, and reordered subjects.
- Primitive and prototype report types are correctly separate.
- Infrastructure failure is not confused with target rejection.
- The executor protocol is bounded and strictly framed on the harness side.
- Process-group supervision is designed around the whole executor tree rather than only the immediate child.
- Target trust states distinguish generic validation from the reviewed first-party definition.
- The static target contract distinguishes primitive capability, backend pattern, and external evidence.
- Signature success and failure behavior is represented much more accurately than a Boolean support flag.
- The constructor and wide-floor prototypes carry explicit non-production status.
- Exact typed comparison is used in many places where an unnecessary digest would have weakened the design.
- The code repeatedly distinguishes target behavior, report provenance, implementation independence, and authenticity rather than collapsing them.

The central lesson of this pass is narrower:

> Exact validation of an evidence report is not sufficient if the evidence subject and its claim classification are themselves caller-authored.

The repository already applies this principle to expected-versus-observed results. It now needs to apply it one layer earlier, to the fixture-to-claim relation.

---

# Suggested remediation order

1. **Close primitive claim laundering**
   - canonical fixture-set trust state;
   - gate accepts only canonical native reports;
   - ad hoc fixtures produce experimental reports only.

2. **Close prototype claim laundering**
   - validated canonical relation matrices;
   - no public claim-set authority at the gate;
   - bind matrix rows to the exact admitted prototype programs.

3. **Combine those repairs with the first review’s transcript binding**
   - transcript retains exact target, binding, and requests actually sent;
   - report evaluation consumes those retained subjects rather than a second caller-supplied copy.

4. **Enforce executor provenance**
   - expected tip/base/topic policy;
   - binary/intended revision agreement;
   - no missing or blank provenance on evidence reports.

5. **Make Meson’s executor classification fail closed**
   - no default reviewed-nonmock declaration.

6. **Repair target-version semantics**
   - either remove V1 support or implement genuine version-dispatched validation.

7. **Complete the signature weld**
   - especially unknown-key success-without-verification.

8. **Correct constructor retry classification**
   - shared retryability predicate.

---

# Minimum adversarial acceptance tests

Before considering the evidence boundary repaired, I recommend adding these end-to-end negative tests against a real or protocol-faithful executor:

```text
primitive:
    trivial true script labeled CheckSig
        → cannot become signature evidence

    trivial true script labeled EcMulScalarVerify
        → cannot become curve evidence

    arbitrary fixture set with complete-looking metadata
        → experimental report only; native gate unavailable

prototype:
    trivial true leaf carrying every constructor claim
        → prototype gate rejects

    trivial true leaf carrying every wide-floor claim
        → prototype gate rejects

    canonical matrix with one claim moved to another row
        → canonical-matrix validation rejects

binding:
    transcript produced for fixture A, evaluated against fixture B
        → rejects

    transcript produced for binding A, evaluated against binding B
        → rejects

provenance:
    nonmock declaration with no binary revision
        → evidence gate rejects

    binary revision different from intended tip
        → evidence gate rejects

build:
    executor path selected without explicit executor class
        → Meson configuration rejects
```

---

# Review scope and verification status

This was a static review of the supplied concatenation. I did **not** run:

- Cargo formatting, Clippy, or tests;
- Meson compile or tests;
- native primitive or prototype execution;
- generated-artifact regeneration;
- document reproducibility;
- `cargo audit`;
- lockfile or resolved-feature analysis.

The supplied filter excluded 352 files, including:

- `Cargo.lock`;
- most unit and integration tests;
- most compiler implementation files;
- most executable-model implementation files;
- the label-tool implementation;
- several build scripts;
- the paper sources.

Therefore this report makes no current green-build, lockfile, license, advisory, or coverage claim. The findings above arise from public construction paths and validation flows visible in the selected source; excluded tests may exercise some adjacent behavior, but a passing test cannot make the demonstrated caller-authored claim authority safe without changing the production path.
