# tripod-target-elements-conformance

The secretless target-native conformance harness for the reviewed
Elements tapscript primitives.

This README is the crate documentation: it is included verbatim as the rendered
landing page, and its example runs as a doctest. It is meant to be enough to use
the public API correctly on its own.

The library crate is named `target_elements_conformance`; the Cargo package is
`tripod-target-elements-conformance`.

## Purpose

Secretless execution of generic target primitive fixtures through an
external Elements executor, and the typed report of what that executor
observed.

## Inputs

- the reviewed static target contract;
- the development deployment binding;
- public generic fixtures — script bytes, initial stack, transaction
  context, and expected outcome;
- an explicit external executor capability (a program path the caller
  selects);
- explicit report and stamp destinations.

## Outputs

- a typed native-conformance report;
- a typed prototype report, carrying per-case results, per-claim coverage,
  and an explicit completeness token;
- optionally, either report as an explicit build asset plus its success
  stamp (ADR-010, ADR-014).

## Not claimed

- executor authenticity — the handshake is provenance, not identity, and
  an executor may misdescribe itself;
- implementation independence;
- production activation;
- backend correctness;
- release evidence identity — no report digest is minted, and none may
  be added here.

Nothing in this package is hashed: there is no report digest, no
fixture-set digest, no executor digest, and no field reserved for one. A
report is compared by its typed content and its exact bytes, and no
persistent report identity is admitted until a cross-process release
consumer exists (Guide-9 section 1.8,
(`[ADR021-case:identity:no-identity]`)).

## The executor has authority, and the harness does not check it

An executor is caller-selected code. Selecting one **grants it execution
authority** (ADR-015). The harness does not authenticate it, does not sandbox
it, and does not prove it independent of anything.

Its handshake is *provenance*, not identity. A dishonest executor can
misdescribe its adapter, its node version, its revision, and its capabilities,
and the report never claims otherwise — it records what the executor said, as a
statement the executor made.

**A mock is never evidence.** A mock executor is admitted for protocol and
failure-path tests, and it can never satisfy the target-native gate. The rule is
enforced in exactly two places:

```text
validate::gate(&validated, expected)                     -> Err(MockExecutorCannotSatisfyNativeGate)
prototype_validate::prototype_gate(&validated, expected) -> Err(MockExecutorCannotSatisfyNativeGate)
```

In both, the check is the **first** thing evaluated, before every other
condition. Nothing else in the crate enforces it: `execute` will run a mock,
`evaluate` will report on it, and `validate_native_report` will validate that
report. Only the gate refuses. A caller that drives the pipeline and skips the
gate has produced a report, not evidence.

The declaration flows from one place: the `ExecutorTrust` argument to
`ExecutorConfiguration::new`, which becomes `ExecutionTranscript::trust()` and
then `ExecutorProvenance.declaration` in the report. The gate requires an
explicit nonmock selection, recorded as ordinary provenance.

## Secrets

None. No interface accepts an RPC username, password, bearer token,
cookie path, private key, signing nonce, production blinding factor,
private opening, wallet path, or production endpoint. An executor that
must authenticate to a node owns that boundary outside this process
(ADR-015).

Disposable test-network material — regtest keys and a cookie confined to
a throwaway data directory — is public fixture data under
(`[ADR015-rule:security:test-material]`), and it remains the *executor's*
material: the first-party interface neither accepts it nor reads it.

## Dependency neighbors

- **`target-elements`** owns every reviewed fact a fixture is stated against:
  the execution domain, the leaf version, the primitive contracts, the
  encodings, and the evidence requirements. This package owns **no target
  semantics whatever**.
- **`tapscript`** produces the exact script bytes a fixture carries. Scripts are
  not assembled here; a fixture takes a typed `TapscriptProgram`, except where a
  case is deliberately malformed and says so via `FixtureScriptSource`.
- **`elements`** and **`secp256k1-zkp`** are the reference implementation,
  adopted by the user's 2026-08-20 ruling and used only by the cross-check
  tests. They bind the same C library the node vendors, so agreement with them
  is conformance-to-the-target's-own-implementation evidence and never
  independence evidence; the first-party oracles below keep the independence
  claim unqualified. The raw sys FFI crate is never a direct dependency.
- **`vectors`** and **`transaction`** are **test-only** neighbors, carrying the
  fixtures and the first-party computations the cross-checks are stated
  against. Neither may be promoted to a regular dependency, because the
  library graph must stay clear of them so that the executor cannot reach an
  expectation.
- Nothing depends on this package in turn. It is the end of the chain.

## The workflow

```text
reviewed_elements_tapscript()                 the reviewed static contract
validate_reviewed_development_binding(..)     the development binding
canonical_fixture_set(&target, &binding)   -> CanonicalPrimitiveFixtureSet
ExecutorConfiguration::new(path, trust, ..)   the caller selects the executor
                                              and where its diagnostics go
executor::execute_canonical(..)      -> ExecutionTranscript
validate::evaluate(..)               -> NativeConformanceReport
validate::validate_native_report(..) -> ValidatedNativeConformanceReport
validate::gate(&validated, Some(&expected_provenance))
                                     -> Ok(()) only for a nonmock executor whose
                                        reported provenance is the expected one
```

Hand-built fixtures have their own path, which ends before the gate:

```text
PrimitiveFixtureSet::new(..)              any fixtures a caller assembles
validate::evaluate_experimental(..)    -> ExperimentalPrimitiveReport
```

Four types have no public constructor, deliberately.

- `ExecutionTranscript` can only be obtained by actually running an executor,
  so a caller cannot fabricate one — and it retains the subjects that run
  sent, so it cannot be re-pointed at a different census or binding either.
- `ValidatedNativeConformanceReport` can only be obtained from
  `validate_native_report`, which recomputes every field rather than trusting
  the report it was handed.
- `CanonicalPrimitiveFixtureSet` can only be obtained from
  `canonical_fixture_set`, and `ConstructorPrototypeMatrix` and
  `WideFloorPrototypeMatrix` only from their own canonical generators. The
  evidence path takes these and nothing else, so the subject of a gated report
  is the repository's evidence plan rather than a census the caller chose. The
  evaluator regenerates the census and compares complete projections as well,
  so the guarantee does not rest on the type alone.

An arbitrary census still executes and still produces a faithful report. What
it cannot become is evidence: `ExperimentalPrimitiveReport` and
`ExperimentalPrototypeReport` have no validator, no gate accepts them, and the
role recorded in the serialized document says so for a reader who has only the
bytes.

## Quickstart: assemble fixtures, drive the mock executor, validate the report

```rust,no_run
// `no_run`: this is compiled, so the API usage below is checked, but it
// is not executed — `execute` spawns a child process and needs a real
// executor program on disk. The crate's own integration tests in
// `tests/executor_protocol.rs` run exactly this chain against the
// `mock-native-executor` binary.
use std::time::Duration;

use tapscript::{StackItem, TapscriptInstruction, TapscriptProgram};
use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding, LeafVersion,
    OpcodeId, TargetContractVersion, reviewed_elements_tapscript,
    validate_reviewed_development_binding,
};
use target_elements_conformance::claim::claim_registry;
use target_elements_conformance::executor::{
    ExecutorConfiguration, ExecutorDiagnostics, ExecutorTrust, execute, execute_canonical,
};
use target_elements_conformance::fixture::{
    ExpectedPrimitiveOutcome, NativeCaseGroup, NativeCaseId, PrimitiveFixture, PrimitiveFixtureSet,
    canonical_fixture_set,
};
use target_elements_conformance::protocol::{
    MOCK_EXECUTOR_GENESIS_ID, MOCK_EXECUTOR_NETWORK_ID,
};
use target_elements_conformance::report::PrototypeReportRole;
use target_elements_conformance::validate::{
    NativeReportValidationInputs, evaluate, evaluate_experimental, guide_nine_evidence_plan,
    validate_native_report,
};
use target_elements_conformance::NativeConformanceError;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target = reviewed_elements_tapscript().expect("the reviewed contract validates");

    // The binding must name the identifiers the executor will report.
    // A run whose executor observed another chain is refused before any
    // case executes.
    let binding = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            TargetContractVersion::V2,
            DeploymentEnvironment::Development,
            MOCK_EXECUTOR_NETWORK_ID,
            MOCK_EXECUTOR_GENESIS_ID,
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )?;

    // One fixture: the script bytes come from a typed tapscript program,
    // never from bytes assembled here.
    let program = TapscriptProgram::new(vec![TapscriptInstruction::Opcode(OpcodeId::Add64)])?;
    let stack = [
        StackItem::signed_le64(&target, 2),
        StackItem::signed_le64(&target, 3),
    ];
    let fixture = PrimitiveFixture::new(
        &target,
        &binding,
        NativeCaseId::new(NativeCaseGroup::Arithmetic, Some(OpcodeId::Add64), 0),
        &program,
        &stack,
        None,
        ExpectedPrimitiveOutcome::accept(Some(vec![
            StackItem::signed_le64(&target, 5).bytes().to_vec(),
        ])),
    )?;
    let assembled = PrimitiveFixtureSet::new([fixture])?;

    // The caller selects the executor, and says what it is. Choosing
    // `Mock` here is what the gate will later refuse.
    let configuration = ExecutorConfiguration::new(
        std::path::Path::new("./mock-executor-wrapper.sh"),
        ExecutorTrust::Mock,
        Duration::from_secs(30),
        // Where that executor writes its own diagnostics. The harness
        // names both files and keeps them after the run.
        ExecutorDiagnostics::in_directory(std::path::Path::new("./run-diagnostics")),
    );

    let plan = guide_nine_evidence_plan()?;
    let registry = claim_registry()?;

    // An assembled census runs, and reports, as an experiment. There is
    // no validator for the result and no gate accepts it: the subject was
    // a caller's choice, and the recorded role says so.
    let ad_hoc = execute(&target, &binding, &configuration, &assembled)?;
    let experiment =
        evaluate_experimental(&target, &binding, &assembled, &ad_hoc, &plan, &registry)?;
    assert_eq!(
        experiment.report().role,
        PrototypeReportRole::ExperimentalPrimitive,
    );

    // The evidence path takes the canonical census and nothing else.
    let fixtures = canonical_fixture_set(&target, &binding)?;
    let transcript = execute_canonical(&target, &binding, &configuration, &fixtures)?;
    let report = evaluate(&target, &binding, &fixtures, &transcript, &plan, &registry)?;

    // Validation recomputes every field rather than trusting the report.
    let validated = validate_native_report(
        report,
        NativeReportValidationInputs {
            target: &target,
            binding: &binding,
            fixtures: &fixtures,
            plan: &plan,
            registry: &registry,
            transcript: &transcript,
        },
    )?;

    // The mock produced a complete, validated report — and still
    // establishes nothing. The gate refuses it first, before any other
    // condition, including the provenance comparison a reviewed run
    // would face next.
    assert!(matches!(
        target_elements_conformance::validate::gate(&validated, None),
        Err(NativeConformanceError::MockExecutorCannotSatisfyNativeGate),
    ));
    Ok(())
}
```

The executor is passed **two arguments and no others** by `execute`:
`--output` and `--elements-output`, the two files it writes its diagnostics
into. Nothing a caller states travels through that interface, which is why no
credential can. The mock's `--behavior <BEHAVIOR>` selector is therefore still
a wrapper script's to supply, and that is what `./mock-executor-wrapper.sh`
stands for above.

The two files are the run's artifacts and are kept: one holds the executor's
own typed facts, the other quarantines raw text out of whatever the executor
ran, and the separation is a property of which file a byte was written to
rather than of how each diagnostic was phrased. The executor's stderr is still
routed to the null device, because a caller-selected executor may write
anywhere; the reviewed adapter writes nothing there at all.

## Public-API tour

Twenty-five modules, twenty-four public. `census` is private; the only things it
exports outward are `ConstructorMatrixDefect`, `WideFloorMatrixDefect`,
`bearing_cases`, and `wide_floor_bearing_cases`, all re-exported through
`prototype`, plus the census itself through `fixture::canonical_fixture_set`.
Those three generators are also the only constructors of the canonical trust
states the evidence path accepts.
`NativeConformanceError` is the only crate-root re-export; everything else is
reached by module path.

### `fixture` — the generic fixture language

```text
canonical_fixture_set(target: &ReviewedElementsTapscriptDefinition,
                      binding: &ReviewedDevelopmentBinding)
    -> Result<PrimitiveFixtureSet, NativeConformanceError>

PrimitiveFixture::new(target, binding, case: NativeCaseId, program: &TapscriptProgram,
                      initial_stack: &[StackItem], context: Option<PrimitiveExecutionContext>,
                      expected: ExpectedPrimitiveOutcome)
    -> Result<Self, NativeConformanceError>
PrimitiveFixture::state(target, binding, statement: FixtureStatement<'_>)
    -> Result<Self, NativeConformanceError>
PrimitiveFixture::projection(&self) -> PrimitiveFixtureProjection

PrimitiveFixtureSet::new(fixtures: impl IntoIterator<Item = PrimitiveFixture>)
    -> Result<Self, NativeConformanceError>
PrimitiveFixtureSet::iter / len / is_empty / contains
```

`new` is the ordinary constructor; `state` takes a `FixtureStatement` parts
struct and is the one to use when a case needs a deliberately malformed script,
an unreviewed leaf version, or a relay-layer verdict. `PrimitiveFixture` has
entirely private fields, so a fixture is what a constructor accepted.

`NativeCaseId` is the complete typed key: a `NativeCaseGroup` (21 variants,
censused in `NativeCaseGroup::ALL`), an optional `OpcodeId`, and an ordinal. It
displays as `group/opcode/ordinal`.

`ExpectedPrimitiveOutcome` is what the reviewed contract *requires*:

```text
ExpectedPrimitiveOutcome::accept(static_final_stack: Option<Vec<Vec<u8>>>) -> Self
ExpectedPrimitiveOutcome::reject(classes: impl IntoIterator<Item = ObservedFailureClass>,
                                 static_final_stack: Option<Vec<Vec<u8>>>) -> Self
```

The final stacks are *static statements*, compared only when an executor happens
to report one. `reject` takes a **set** of admissible failure classes because a
node reports one reason for several reviewed causes.

Supporting vocabulary: `EnforcementLayer` (`Consensus` or `RelayPolicy`),
`LeafVersionStatus`, `FixtureScriptSource` (`TypedProgram` or
`DeliberatelyMalformed`), `ResourceExpectation` (`Exact` or `RecordedOnly`), and
the transaction-context types `PrimitiveExecutionContext`, `FixtureInput`,
`FixtureOutput`, `FixtureIssuance`, and `FixtureScriptPath`.

### `protocol` — the secretless wire

```text
NATIVE_PROTOCOL_SCHEMA: u32 = 4
MOCK_EXECUTOR_NETWORK_ID: [u8; 32] = [0x11; 32]
MOCK_EXECUTOR_GENESIS_ID: [u8; 32] = [0x22; 32]

validate_response_shape(response: &NativeExecutionResponse,
                        capabilities: &BTreeSet<ExecutorCapability>)
    -> Result<(), ResponseShapeDefect>
```

`ExecutorHandshake` carries fourteen fields of what the executor *says* about
itself, including the optional revision fields that a mock leaves as `None`
rather than filling with something plausible. `runs_prototype_fixtures()`,
`materializes_trees()`, `runs_conservation_rows()`,
`runs_normalization_rows()`, and `runs_operation_step()` read the capability
set.

`ExecutorCapability` has 11 variants; `ObservedFailureClass` has **34**,
enumerated in `src/protocol.rs`. Representative: `StackUnderflow`,
`InvalidOperandWidth`, `DivisionByZero`, `EvaluatedFalse`,
`NonSingletonFinalStack`.

`ResponseShapeDefect` has 9 variants and exists so that an executor cannot
report more than it advertised or less than it promised — for example
`StackWithoutAdvertisedReporting` and `AdvertisedStackOmitted` are both defects.

Revision 4 carries a typed record pair for each of the four workloads
Guide-12 §16.3 names: `NativeConservationRequest`/`Response`,
`NativeNormalizationRequest`/`Response`, `NativeLifecycleRequest`/`Response`,
and `NativeOperationRequest`/`Response`. Every one of them refuses unknown
fields and every response has a `validate_shape` that refuses an observation
beside a layer saying the run never happened.

The operation records are the §16.2 target-generic boundary on the wire. An
`OperationCaseId` is an `OperationStepKind` (`Fund` or `Submit`) and the
caller's own name for the step; an `OperationSubject` is a
`TargetFundingSubject` (issue an asset, pay `outputs` outputs of
`amount_per_output` to `output_program`) or a `TargetSubmissionSubject`
(`transaction_bytes`), untagged and told apart by disjoint members. A
`NativeOperationResponse` answers with an `ObservedOutcomeLayer`, the
`FundedOutput` rows a funding step created, and the `accepted_txid` a
submission earned. Nothing in that vocabulary names an operation's meaning,
which is what lets this package supervise a compact-ASH run without owning
one.

`ProtocolLimits` bounds each phase of what this side READS;
`ProtocolLimits::DEFAULT` is the standard set, and `for_phase` reads the bound
that applies. What the executor reads is bounded by
`MAXIMUM_HANDSHAKE_REQUEST_BYTES` and `MAXIMUM_REQUEST_BYTES`, which are
constants of the contract rather than configuration: the executor is a
separate program this interface passes no configuration to, so a per-run
request bound would be a bound only one side knew. `maximum_request_bytes`
reads the one that applies, and `write_message` refuses to build a record past
it.

### `executor` — the driver

```text
DEFAULT_EXECUTOR_TIMEOUT: Duration = 300s
DEFAULT_EXECUTOR_CLEANUP_GRACE: Duration = 5s

ExecutorDiagnostics::in_directory(directory: &Path) -> Self
ExecutorDiagnostics::new(typed: &Path, child: &Path) -> Self

ExecutorConfiguration::new(program: &Path, trust: ExecutorTrust, timeout: Duration,
                           diagnostics: ExecutorDiagnostics) -> Self
ExecutorConfiguration::with_limits(self, limits: ProtocolLimits) -> Self
ExecutorConfiguration::with_cleanup_grace(self, cleanup_grace: Duration) -> Self
ExecutorConfiguration::with_expected_provenance(self, expected: ExpectedExecutorProvenance) -> Self

execute(target, binding, configuration: &ExecutorConfiguration, fixtures: &PrimitiveFixtureSet)
    -> Result<ExecutionTranscript, NativeConformanceError>
execute_prototypes(target, binding, configuration, fixtures: &[CompoundPrototypeFixture])
    -> Result<ExecutionTranscript, NativeConformanceError>
execute_operations(target, binding, configuration, planner: &mut dyn TargetOperationPlanner)
    -> Result<ExecutionTranscript, NativeConformanceError>
```

`execute_operations` is the Guide-12 §16.2 boundary. Unlike the other two it
takes a *plan* rather than a set of cases: a `TargetOperationPlanner` is asked
for its `next_step` between steps, is handed the previous step's answer, and
returns `Ok(None)` when it is done or `Err(PlanRefused)` when it cannot state
one. That is what makes an operation expressible at all — a transaction cannot
be built until the outputs it spends exist, those outputs are created by an
earlier step of the same run, and each run gets its own disposable node, so a
fixed list of steps could not carry the dependency and two runs would fund one
chain and submit to another.

The capability gate is asked per step rather than per run, so an executor that
submits transactions but holds no funds is refused the funding steps and
answers the submissions. `PlanRefused` carries no reason: the plan knows why in
its own vocabulary, and this package holds none for it.

The timeout is explicit and typed; there is no untimed run. The child is spawned
in its own process group so that a stalled executor and its descendants can be
stopped together, with `cleanup_grace` before escalation.

`ExecutorTrust` is `Mock` or `ReviewedNonMock`, and it is the caller's
declaration rather than an observation — nothing here can tell the difference.

`ExecutionTranscript` reads back `handshake()`, `environment()`, `trust()`,
`target()`, `deployment()`, `requests()`, `responses()`,
`prototype_requests()`, `prototype_responses()`, `operation_requests()`, and
`operation_responses()`. It has no public constructor.

The first four of those readers are what makes a transcript *bound*. It
retains the reviewed contract projection and the deployment projection the run
was requested under, and the exact subject sent for every case, so a report
cannot be built against a contract, a binding, or a census the run never
touched. Evaluation compares them by exact typed equality — no digest — and
refuses on the first disagreement with `TranscriptTargetRebinding`,
`TranscriptDeploymentRebinding`, `TranscriptSubjectMismatch`,
`MissingCaseRequest`, or `UnrequestedCaseResponse`.

### `claim` — the claim census

```text
claim_registry() -> Result<ClaimRegistry, NativeConformanceError>
claims_of(fixture: &PrimitiveFixture) -> BTreeSet<NativeEvidenceClaim>
```

`NativeEvidenceClaim` has **40** variants, enumerated in `src/claim.rs`. Each
sits beneath one broad `TargetEvidenceRequirementId` and is either `Required` or
`Unresolved` with a stated reason — an unresolved claim carries why it is
unresolved rather than being omitted.

`ClaimRegistry` offers `record`, `iter`, `owned_by`, `required_claims`, `len`,
and `is_empty`.

### `provenance` — what the operator meant to run

```text
MINIMUM_REVISION_PREFIX_WIDTH: usize = 7
FULL_REVISION_WIDTH: usize = 40

RevisionId::new(text: &str)  -> Result<Self, ProvenanceSyntaxDefect>   an admitted prefix
RevisionId::full(text: &str) -> Result<Self, ProvenanceSyntaxDefect>   a full identifier
RevisionId::matches_full(&self, full: &RevisionId) -> bool
TopicName::new(text: &str)   -> Result<Self, ProvenanceSyntaxDefect>

ExpectedExecutorProvenance::new(intended_tip: &str, upstream_base: &str,
                                included_local_topics: impl IntoIterator<Item = &str>)
    -> Result<Self, ProvenanceSyntaxDefect>

validate_executor_provenance(reported: &ExecutorProvenance,
                             expected: &ExpectedExecutorProvenance)
    -> Result<ValidatedExecutorProvenance, NativeConformanceError>

expected_provenance_from_arguments(reviewed_non_mock: bool, intended_tip: Option<&str>,
                                   upstream_base: Option<&str>, local_topics: &[String])
    -> Result<Option<ExpectedExecutorProvenance>, ProvenanceArgumentDefect>
```

ADR-018 assigns the gate one comparison: the executable that ran must be the one
the operator meant to run. The report's provenance fields are what the executor
*said*, so the comparison needs a second operand this repository supplies — an
`ExpectedExecutorProvenance`, which both gates require and refuse to proceed
without.

**The matching rule, stated once.** A binary's reported revision matches the
expected tip when it is at least `MINIMUM_REVISION_PREFIX_WIDTH` lowercase hex
digits and is an *exact prefix* of the expected full identifier; equality is the
width-40 case of that same rule. The prefix form is what the reviewed adapter
actually reads — a node binary embeds an abbreviation in its version line, not a
full object identifier — so a full-equality rule would refuse every honest run,
and an arbitrary-text-equality rule would accept a run naming a different object.
The operator's own declarations (`intended_executed_tip`, `upstream_base`) are
compared for full equality instead, and the topic census for set equality in both
directions: a tip that folded in an unexpected branch is not the reviewed tip.

No value here is ever derived from a working tree. A checkout's `HEAD` identifies
intended source; the binary's embedded revision identifies the program.

### `validate` — plan, evaluate, validate, gate

```text
guide_nine_evidence_plan() -> Result<EvidencePlan, NativeConformanceError>
evaluate(target, binding, fixtures, transcript, plan, registry)
    -> Result<NativeConformanceReport, NativeConformanceError>
validate_native_report(report: NativeConformanceReport,
                       inputs: NativeReportValidationInputs<'_>)
    -> Result<ValidatedNativeConformanceReport, NativeConformanceError>
gate(validated: &ValidatedNativeConformanceReport,
     expected_provenance: Option<&ExpectedExecutorProvenance>)
    -> Result<(), NativeConformanceError>
check_required_rows_own_required_claims(plan: &EvidencePlan, registry: &ClaimRegistry)
    -> Result<(), NativeConformanceError>
```

`NativeReportValidationInputs` is a `Copy` parts struct with six public
reference fields: `target`, `binding`, `fixtures`, `plan`, `registry`, and
`transcript`.

`EvidencePlan` partitions the target evidence census into four classes —
`Required`, `UnresolvedByDesign`, `UnsupportedByStaticContract`, and
`DeferredToTransactionEvidence` — so a requirement this lane cannot settle is
classified rather than silently passed.

### `report` — the typed report

`NATIVE_REPORT_SCHEMA` is 2. `NativeConformanceReport` carries the schema, the
role, the target contract version, the intended `ActivationRecord`, the
`ObservedEnvironment` the executor reported, the `ExecutorProvenance`, and three
row vectors: `cases`, `evidence`, and `claims`, plus a `NativeReportSummary`.

`CaseStatus` is `Passed`, `Failed`, or `InfrastructureError` — there is
deliberately **no** `Skipped`, because a case that did not run must not read as
one that passed.

`ReportCompleteness` is `CompleteForPrimitivePlan`, `PartialUnresolvedClaims`,
or `Failed`. It is the explicit completeness token: a report never implies
completeness by the absence of failures.

### The prototype lane

`prototype` carries `PrototypeRelation` (`MetadataConstructorContinuity` and
`WideFloorRelation`), `PrototypeCaseId`, `PrototypeClaim` (**20** variants
censused in `PrototypeClaim::ALL`), and `CompoundPrototypeFixture` with its
`defect` and `is_coherent` predicates. The two matrix builders are:

```text
constructor_case_matrix(target) -> Result<Vec<CompoundPrototypeFixture>, ConstructorMatrixDefect>
wide_floor_case_matrix(target)  -> Result<Vec<CompoundPrototypeFixture>, WideFloorMatrixDefect>
```

`prototype_program` builds and measures the typed programs —
`PrototypeProgram::continuity` and `PrototypeProgram::wide_floor`, each
returning a `PrototypeProgramDefect` on refusal — and publishes the schema and
field-offset constants the oracle and the program agree on.

`prototype_validate` mirrors the primitive lane exactly:
`evaluate_prototypes`, `validate_prototype_report`, and `prototype_gate`, with
`PrototypeReportValidationInputs` and the constructorless
`ValidatedPrototypeReport`.

The two lanes are separate types answering separate questions, and one report
never carries both: a primitive census establishes what one reviewed primitive
did, and a prototype matrix establishes whether a multi-step construction held
together across a whole target output.

### `constructor` and `wide_floor` — the oracles

`constructor` re-exports the taproot and metadata surface: `tagged_hash`,
`leaf_hash`, `branch_hash`, `tweak`, `tweaked_key`, `control_block`,
`output_program`, `construct`, `construct_under_policy`, the
`FixtureTapTree` type, `PrototypeMetadata`, `metadata_leaf_program`, and
`UNSPENDABLE_INTERNAL_KEY`, with a typed defect for each stage. It is public
point arithmetic only — no secret scalar appears anywhere in it.

`wide_floor` re-exports the domain constants, the `WideFloorInstance` oracle and
its `WideFloorWitness`, the staged normalizer, and the candidate comparison.

### `reference` — vectors minted against the reference implementation

`FIXTURE_MERKLE_ROOT` and `FIXTURE_REFERENCE_OUTPUT_KEY`, with the output key's
parity bit. The root is the first-party hashing of the fixture bundle's
committed tree, confirmed by folding every leaf path through the reference
crate's own branch hashing; the output key is the BIP-341-style tweak of the
fixture internal key by that root, computed through `secp256k1-zkp` because the
first-party construction side deliberately refuses that arithmetic.

Pinning the key does **not** discharge `PinnedOutputKeyUnverifiedAgainstTree`.
That needs the funding ceremony against a real node — an output actually created
at the program and a spend the node accepted. The pin buys early detection of
tree-construction drift, and nothing more.

The cross-checks themselves live in `src/tests/reference_oracle_tests.rs`,
because they need the test-only fixture neighbors. Every one of them states its
claim class in a line: reference-implementation conformance, not independent
evidence.

### The declassification lane

Six modules answering the confidential-to-public representation question.
They are research modules: their reports carry the `Experimental` role, and
they establish what the target does with a matrix rather than that a
candidate was selected. Selection is the plan tree's, and it is recorded in
`plans/research/public-declassification.md`.

`commitment_oracle` is the independent reference, in four parts: `curve`
(field and point arithmetic, both parity conventions kept distinct), `generator`
(the two-generation asset-generator recipe over tagged digests), `commitment`
(the value commitment, with the blinder on the base point and the amount on the
asset generator), and `vector` (the pinned upstream low-level vectors). It
computes commitments as arithmetic and mints no identity of its own; it exists
so that the target's bytes can be predicted by something that is not the target.

`conservation` carries the confidential-transaction fixture language,
`ConservationRow`, `ConservationDefect`, `RowDeferral`, `ClosureObligation`, and
`canonical_conservation_matrix()` — twelve rows, eleven of them executable, one
deferred as a typed row rather than dropped. `test_scalar` derives fixture
blinding factors deterministically from a domain-separated label; it is
disposable-chain test material and authorizes nothing. `conservation_report` is
the typed report, with `ConservationReportRole` carrying its single
`Experimental` variant.

`normalization` carries the owner-authorized transition: `NormalizationClaim`,
`OutputRole`, `PreservedProperty`, `AuthorizationProfile`, `RefusalLayer`, the
`closure_finding` and `preservation_finding` predicates, and
`canonical_mutation_matrix()` — nine mutations plus the unmutated row. Closure
is exact multiset equality in both directions, because a subset test passes a
hidden output and a count test passes a swap. `normalization_report` is its
safety report.

`lifecycle` carries the fresh-process boundary: `PublicHandoff` (a twelve-field
schema with `deny_unknown_fields` and a ban on field *names* that denote
owner-private material), `LifecycleRow`, `LifecycleOutcome`, and
`canonical_lifecycle_matrix()`. `lifecycle_report` is its report.

`disposition` is where the candidates stand — `Candidate`, `DispositionState`
(`Prototyped`, `DeferredWithNamedBlocker`, `NotApplicableWhileDeferred`),
`RejectionCriterion`, `RejectedShape`, and `candidate_dispositions()`. A
disposition is this project's decision taken in the light of target facts, which
is why it lives here and not in `target-elements`.

`declassification` carries `DeclassificationReason` — six variants, with
`is_semantic_necessity` separating the four that assert the relation needs the
fact public from the two that do not — and `normalization_declassifications()`.
Every entry there is `DeploymentPolicy` and every one states the alternative a
deployment that wanted the fact private would have to take, so that a policy
choice cannot harden into an apparent necessity across waves.

### `vocabulary` — wire spellings

Six functions pairing each reviewed identity with its explicit wire name, in
both directions:

```text
opcode_name / opcode_from_name                          (55 entries)
capability_name / capability_from_name                  (45 entries)
evidence_requirement_name / evidence_requirement_from_name  (24 entries)
```

The spellings are explicit rather than derived, so a rename upstream cannot
silently change a wire format.

## Error handling

`NativeConformanceError` is the crate's single error root, has **88**
`#[non_exhaustive]` variants declared in `src/error.rs`, and derives
`thiserror::Error`. Every variant is a branch that runs — there is no catch-all
— and none carries child-process detail that could leak an executor's
environment.

By stage:

| Stage | Representative variants |
|---|---|
| Fixture assembly | `TargetContractMismatch`, `DuplicateFixtureCase`, `FixtureNotExpressible`, `MalformedFixtureContext` |
| Executor startup | `ExecutorStartupFailed`, `ExecutorProcessGroupUnavailable`, `ExecutorHandshakeFailed`, `ExecutorProtocolMismatch`, `UnsupportedProtocolSchema` |
| Executor run | `ExecutorTimeout`, `ExecutorExited`, `MalformedResponse`, `ProtocolRecordTooLarge`, `BlankProtocolRecord`, `MalformedResponseShape` |
| Environment agreement | `EnvironmentBindingMismatch`, `GenesisObservationMismatch`, `ActivationObservationMismatch`, `MissingEnvironmentObservation` |
| Response census | `Duplicate/Missing/UnexpectedCaseResponse`, `ResponseOrderViolation`, `TrailingProtocolData` |
| Evaluation | `TargetContractMismatch`, `DevelopmentBindingMismatch`, `MissingCaseResponse` |
| Report validation | `UnsupportedReportSchema`, `ReportCaseCensusMismatch`, `FixtureProjectionMismatch`, `ReportCaseOutcomeMismatch`, `ReportSummaryMismatch`, `ReportProvenanceMismatch` |
| Gate | `MockExecutorCannotSatisfyNativeGate` first, then `RequiredClaimMissing`, `RequiredClaimFailed`, `RequiredEvidenceMissing`, `RequiredEvidenceFailed`, `RequiredEvidenceInfrastructureError` |

The environment checks fire **before any case runs**: a binding naming one chain
and an executor reporting another is refused up front rather than producing rows
about the wrong network.

## Binaries

Seven, auto-discovered from `src/bin/`.

| Binary | Purpose |
|---|---|
| `check-target-elements-native` | The primitive gate: canonical census, executor, evaluate, validate, gate, publish (one JSON result, ADR-010) |
| `check-target-elements-prototypes` | The prototype gate: one relation's matrix through the same contract |
| `mock-native-executor` | The protocol and failure-path mock, selected by `--behavior <BEHAVIOR>`; echoes each fixture's own expectation back |
| `emit-conservation-matrix` | The canonical conservation matrix as JSON: `rows` for a reader and the report, `requests` for the executor |
| `emit-normalization-matrix` | The canonical normalization claim and mutation matrix, split the same way |
| `emit-normalization-report` | The normalization safety report, rebuilt from a run record |
| `emit-lifecycle-report` | The fresh-process lifecycle report, rebuilt from a run record |

The four `emit-` commands exist so that the matrices and the claims have
exactly one home. A runner that restated a claim in another language would be a
second implementation of it, free to drift — and a drifted claim agrees with
whatever it is compared against. So the runners are transport: they record the
adapter's responses verbatim, and the expectations are rebuilt here from the
crate's own source.

The checking binaries take `--executor <PROGRAM>` and
`--executor-class mock | reviewed-non-mock`, plus the network and genesis
identifiers. A declared mock run exits as a runtime failure and writes no
result.

## State

Implemented: the package boundary, the wire vocabulary, the generic
fixture language, the secretless executor protocol, the external
executor driver with its explicit typed timeout, the typed
native-conformance report, the Guide-9 evidence plan, the comparison,
the gate, and the ADR-010 checker command with its non-default Meson
lane.

Also implemented: the canonical primitive census. It covers every
reviewed primitive, and every required evidence row has cases bearing on
it.

Also implemented: the two Guide-10 prototype programs and their case
matrices — the STATE constructor and the exact wide floor — together with
the prototype report, its claim registry and completeness tokens, and the
runner command that answers a matrix from a real executor under the same
ADR-010 contract as the primitive lane. Both matrices agreed with the
executor on every row. Those programs are prototypes and are held to the
prototype status: no operation emits them, and nothing here converts one
into release output.

Also implemented: the Guide-11 declassification lane — the independent
commitment oracle, the confidential-transaction fixture language and its
conservation matrix, the owner-authorized normalization claim with its
mutation matrix and safety report, the fresh-process lifecycle with its
public handoff schema, and the candidate dispositions with their typed
disclosure reasons. Eleven conservation rows, nine normalization mutations,
and eight lifecycle rows over two passes ran against a real node and agreed
with expectations committed before the runs. Those reports carry the
`Experimental` role: they establish what the target does, and the policy
selected in the light of them lives in the plan tree, not here.

The conservation, normalization, and lifecycle lanes record and do not
gate. Their typed reports exist and are tested; the executor drivers that
would make them refusable in CI the way the primitive and prototype lanes
are, are not built.

What a case can establish is bounded by what a validating node reports.
It answers whether a spend was valid and, coarsely, why not; it exposes
no interpreter stack, and it reports one reason for several reviewed
causes. Expectations are shaped accordingly: a verdict, the failure
classes the contract admits, and the exact stacks as static statements
that are compared only when an executor reports one.

The reviewed domain requires evaluation to finish with exactly one true
item, and the reviewed primitive census has no equality, drop, or verify
primitive to reduce a deeper stack with. Several primitives therefore
have no reachable accepting case, and their cases establish the number
of items the primitive pushed instead.

Deliberately not covered, and recorded as residuals rather than filled
in with cases that cannot run: a signature over a transaction sighash,
blinded assets, amounts, and nonces, issuing inputs, an absent
introspection context, any execution domain other than the reviewed one,
and a relative timelock at the top of the sequence mask counted in
blocks — which would need an input sixty-five thousand confirmations
deep, so the same boundary counted in intervals is stated in its place.

## Consensus and relay are asked differently

Every case states which layer its verdict belongs to, and the two are
different questions. A relay rule can only be observed on a script that
is otherwise valid: a script that fails at consensus as well reports the
consensus reason, and the relay rule is never reached. That is why the
cases establishing minimal script-number encoding — which is a relay
rule and not the target's own — are the ones whose scripts would
otherwise be accepted, while the cases whose operand merely happens to
be nonminimal state what consensus does with it.

## The native lane is not part of ordinary CI

The Meson target `target-elements-native-check` is non-default and is
defined only when `-Dtarget_native_executor=` names an executor. With no
executor configured the target does not exist, nothing runs it, and
ordinary CI claims no target-native evidence — a skipped native lane
leaves the Guide-9 evidence incomplete even when every other lane is
green.

## What this package deliberately does not do

- **It owns no target semantics.** Every reviewed fact belongs to
  `target-elements`, and the script bytes come from `tapscript`. Nothing here
  decides what a primitive means.
- **It authenticates nothing.** No executor is verified, sandboxed, or shown
  independent. The handshake is provenance.
- **It mints no identity.** Nothing is hashed: no report digest, no fixture-set
  digest, no executor digest, and no field reserved for one. A report is
  compared by its typed content and its exact bytes, and no persistent report
  identity is admitted until a cross-process release consumer exists (Guide-9
  §1.8, (`[ADR021-case:identity:no-identity]`)).
- **It accepts no secret.** See *Secrets* above; the list is exhaustive and
  closed.
- **It never reports a skipped case as a passing one.** `CaseStatus` has no
  `Skipped` variant, and completeness is an explicit token rather than an
  inference from the absence of failures.
- **It converts no prototype into release output.** The two prototype programs
  are held to prototype status; no operation emits them.
- **It does not run in ordinary CI.** The native lane exists only when an
  executor is configured, and its absence is recorded as incomplete evidence
  rather than as a pass.
