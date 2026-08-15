# Guide 10 — STATE Constructor and Exact Wide-Arithmetic Prototypes

> **Status:** Execution guide
> **Phase:** Phase 3 — Elements target and foundational prototypes
> **Entry:** Recorded Guide-9 target-native primitive gate, plus closure of the blocking review findings in §5
> **Primary research owners:** [STATE constructor](../research/state-constructor.md), [wide arithmetic](../research/wide-arithmetic.md)
> **Affected packages:** `architecture`, `realization`, `target-elements`, `tapscript`, `target-elements-conformance`
> **Does not complete:** Phase 3; public declassification remains a separate required prototype
> **Does not implement:** an attestation-contract operation, linked bundle, transaction ABI, calibrated architecture bound, deployment release, or production target

---

## Mission · `sec:guide10:mission`

Guide 10 determines whether the reviewed Elements tapscript substrate can support two load-bearing backend mechanisms without weakening their target-independent semantics:

```text
metadata-dependent constructor continuity

exact wide floor arithmetic
    q = floor(a·b/d)
```

The guide must produce executable, target-native answers to two questions.

### Constructor question

Can a target program authenticate a metadata-dependent predecessor object, derive successor metadata, and require the successor to preserve the same authenticated static program relation?

Conceptually:

```text
predecessor:
    Constructor(static code root, metadata_before)

successor:
    Constructor(the same static code root, metadata_after)
```

The program must reject:

- wrong predecessor metadata;
- wrong successor metadata;
- an independently selected successor code root;
- an alternate constructor schema;
- a wrong internal key;
- a wrong target tree;
- a wrong target output program;
- noncanonical metadata;
- a spendable metadata path;
- a stale constructor from another static bundle.

### Arithmetic question

Can a target program prove exactly:

\[
q=\left\lfloor\frac{a·b}{d}\right\rfloor
\]

under the realization amount bounds:

```text
0 ≤ a,b,q < 2^51
0 < d < 2^51
```

without allowing:

- signed target overflow;
- an unchecked arithmetic-success flag;
- malformed or noncanonical limbs;
- a wrong carry;
- an under-quotient;
- an over-quotient;
- a wrong remainder;
- a proof about values other than the enclosing relation’s authenticated operands?

Guide 10 is prototype-only. Its result is one of:

```text
accepted constructor mechanism
accepted wide-floor mechanism
explicit target rejection
sharply bounded unresolved target question
```

It is not a production backend milestone by itself.

---

# 1. Executive rulings · `sec:guide10:rulings`

## 1.1 Repair the evidence boundary before recording Guide-10 evidence · `rule:guide10:validated-native-evidence`

The second static review found that the current native report and gate do not establish exact report completeness. Guide 10 must not build new prototype claims on that boundary unchanged.

Before any Guide-10 prototype report may satisfy a gate, the implementation must provide:

```text
raw native report
    ↓
complete owner validation
    ↓
validated native report wrapper
    ↓
prototype gate
```

The validator must require exact, duplicate-sensitive equality among:

```text
fixture case census
reported case census

evidence-plan row census
reported evidence row census

required claim census
passed claim census
```

The validator must recompute rather than trust:

- case status;
- evidence disposition;
- summary counts;
- completeness status;
- required-versus-unresolved classification;
- target/binding equality;
- report schema support.

Removing a failed row, relabeling it unresolved, clearing the evidence array, duplicating a passing row, or editing the summary must fail validation.

A public gate accepts only the validated wrapper:

```rust
pub fn gate(
    report: &ValidatedNativeConformanceReport,
) -> Result<(), NativeConformanceError>;
```

Equivalent ownership-preserving factoring is acceptable.

## 1.2 Reports bind complete fixtures, not ordinal case names · `rule:guide10:fixture-binding`

A native case ordinal is local navigation inside one fixture census. It is not semantic evidence identity.

Every prototype report row must carry, directly or through an identity admitted under ADR-016, the complete fixture subject:

- exact script bytes;
- exact initial stack;
- exact transaction context;
- consensus or relay enforcement layer;
- execution domain;
- leaf version;
- reviewed or unreviewed leaf status;
- script source;
- expected verdict;
- expected final stack where statically fixed;
- expected resource observations;
- exact target definition;
- exact development binding.

Guide 10 initially embeds the complete typed fixture projection.

It must not mint a fixture-set digest merely to reduce report size.

## 1.3 Broad evidence rows require exact subclaim coverage · `rule:guide10:claim-coverage`

One passing case is not sufficient to pass a broad evidence family.

Guide 10 introduces typed evidence claims beneath broad target evidence requirement IDs. A representative vocabulary is:

```rust
pub enum NativeEvidenceClaim {
    StackManipulationSuccess,
    StackManipulationUnderflow,

    ByteEqualitySuccess,
    ByteEqualityFailure,
    VerifySuccess,
    VerifyAbort,

    TapleafHashVector,
    TapbranchFirstOrdering,
    TapbranchSecondOrdering,
    TaptweakEvenResult,
    TaptweakOddResult,

    ConstructorPredecessorBinding,
    ConstructorSuccessorBinding,
    ConstructorStaticRootContinuity,
    ConstructorMetadataMutationRejected,
    ConstructorStaticRootMutationRejected,
    ConstructorInternalKeyMutationRejected,
    ConstructorMetadataEscapeRejected,

    WideFloorExactDivision,
    WideFloorNonzeroRemainder,
    WideFloorUnderQuotientRejected,
    WideFloorOverQuotientRejected,
    WideFloorZeroDivisorRejected,
    WideFloorMalformedLimbRejected,
    WideFloorWrongCarryRejected,
    WideFloorUncheckedFlagRejected,
}
```

Exact names and factoring remain implementation-owned.

For each evidence requirement \(e\), the gate requires:

\[
\operatorname{requiredClaims}(e)\subseteq\bigcup_{\substack{c\text{ passed}\\c\text{ bears on }e}}\operatorname{claims}(c).
\]

A claim without a passed case remains unresolved.

A passing explicit-value case does not silently complete confidential-value evidence. An issuance-absent case does not complete issuance-present evidence. A rejection-only signature suite does not complete successful signature verification.

## 1.4 Generic validation and reviewed trust remain separate · `rule:guide10:reviewed-binding`

A development binding validated against target definition \(A\) must not later combine with target definition \(B\) merely because both carry the same contract-version number.

Guide 10 requires one of these equivalent boundaries:

```text
ValidatedDevelopmentBinding retains the complete validated target projection

or

ReviewedDevelopmentBinding is constructible only against
ReviewedElementsTapscriptDefinition
```

Native fixtures and reports use the exact reviewed binding.

Version equality is insufficient because several internally coherent target definitions may share one version while differing in:

- capability status;
- opcode behavior;
- failure behavior;
- encodings;
- resources;
- evidence requirements.

## 1.5 The executed chain is observed, not caller-labelled · `rule:guide10:observed-environment`

The native executor must report the development environment it actually executed:

- environment class;
- chain name;
- observed genesis identity;
- observed network identity under one typed recipe;
- relevant activation/configuration state;
- reviewed leaf-version availability.

The harness compares those observations with the validated binding before accepting a case.

Synthetic run labels must not inhabit fields called `network_id` or `genesis_id`.

## 1.6 The executor process tree is supervised as one run · `rule:guide10:executor-supervision`

The executor is caller-selected code and is not sandboxed. Guide 10 nevertheless enforces the harness’s own timeout and cleanup contract.

On timeout, the harness must terminate the complete executor process group, including an adapter-spawned node, rather than only killing the immediate child.

A timed-out run must not leave:

- an orphaned node;
- a live inherited protocol pipe;
- a temporary node data directory;
- a disposable RPC cookie;
- a listening port;
- a child process holding the report open.

Protocol records receive explicit byte limits. An executor must not force unbounded allocation by writing one unterminated line.

## 1.7 Signature abstraction closes before signature-dependent constructor integration · `rule:guide10:signature-abstraction`

The current static signature model cannot represent its own documented empty-signature and unknown-key-type behavior exactly.

Before a Guide-10 result is used by an operator-authorized STATE operation, the operand and failure model must distinguish at least:

```text
empty signature
recognized nonempty valid signature
recognized nonempty invalid signature
empty public key
recognized public key
unknown nonempty public-key type
```

A type stating “exactly 64-byte signature” cannot simultaneously represent an empty-signature failure path.

A type stating “exactly x-only public key” cannot represent the documented unknown-key-type compatibility path.

The standalone constructor prototype may remain signature-free. Any Phase-6 `announce-maturity` handoff may not.

## 1.8 Relation identity is welded before compiler-visible expansion · `rule:guide10:relation-identity`

Before Guide 10 adds a compiler-visible relation or target requirement, realization validation must prove that each `RelationId` agrees with its `Relation` body.

The weld derives:

- expected `RelationKind`;
- expected `RelationSubject` class;
- owning operation;
- any architecture-owned object, asset, root, projection, or lifecycle subject.

It then compares those values with the declared ID.

`ExpressionPredicate` receives an honest relation-kind identity before expression-bearing production work relies on it.

A relation ID and relation body must never provide two different answers to:

> What semantic relation is this?

## 1.9 Production-release validation remains unavailable under an incomplete deployment-profile schema · `rule:guide10:profile-state`

Guide 10 does not construct a deployment release.

If deployment-profile schema 2 still cannot bind the final transaction ABI/configuration used by calibration, no public wrapper may claim production-release validity from schema 2.

Acceptable boundaries include:

```text
ValidatedPreReleaseDeploymentProfile

validate_deployment_profile_structure(...)

validate_production_deployment_release(...)
    unavailable or rejecting under schema 2
```

This is a trust-state correction, not a Guide-10 deployment feature.

## 1.10 The primitive substrate expands only by reviewed need · `rule:guide10:primitive-admission`

The current instruction subset describes individual Guide-9 primitives. It does not yet necessarily contain enough ordinary stack, equality, verification, control-flow, or byte operations to express Guide-10 compound proofs.

Guide 10 begins with an explicit primitive-needs census.

Potential requirements include:

```text
stack:
    duplicate
    swap or rotate
    remove
    indexed copy where unavoidable

verification:
    equality
    equality-and-abort
    boolean verification

control:
    conditional selection where unavoidable
    exact branch-join behavior

bytes:
    concatenation or streamed chunk hashing
    exact width
    lexicographic ordering where target tree construction requires it
```

No primitive is assumed available because it is familiar from Bitcoin, another Elements execution domain, or an upstream test framework.

Every admitted primitive receives:

- exact target byte;
- execution-domain contract;
- operand contract;
- every success relation;
- every failure relation;
- encoding dependencies;
- resource cost;
- target-native positive and negative cases;
- evidence requirements;
- cross-contract welds.

If a required primitive is unavailable, the affected prototype candidate is rejected. It does not enter through a raw opcode escape.

## 1.11 Prototype code cannot silently become release code · `rule:guide10:prototype-state`

Guide-10 artifacts carry an explicit prototype state.

Suitable forms include:

```rust
pub struct ConstructorPrototypeProgram {
    // private fields
}

pub struct WideFloorPrototypeProgram {
    // private fields
}

pub enum PrototypeStatus {
    Experimental,
    AcceptedResearchResult,
}
```

The production backend API must not accept these values as:

- emitted operation programs;
- relocatable bundle members;
- linked programs;
- transaction ABI programs;
- release assets.

No generic production bundle type is created in this guide.

## 1.12 Exact host references and target programs are independent · `rule:guide10:independent-oracles`

For wide arithmetic:

```text
target pattern:
    fixed-width operations and limb relations

host oracle:
    u128 or arbitrary-precision exact arithmetic
```

For constructor hashing and tweaking:

```text
target pattern:
    target instruction sequence

host oracle:
    independently reviewed target-compatible hash/tree/tweak construction
    or published vectors plus a separate first-party reference
```

The target executor’s answer must not generate its own expected value.

## 1.13 No speculative identity is minted · `rule:guide10:no-identity`

Guide 10 introduces no:

```text
ConstructorPrototypeHash
WideArithmeticPatternHash
PrototypeFixtureSetHash
PrototypeReportHash
TargetProgramHash
```

Typed values and exact bytes are retained directly.

A future linker or release consumer may activate identities under ADR-016. Guide 10 does not reserve fields for them.

---

# 2. Entry conditions · `sec:guide10:entry`

Guide 10 begins only when:

- the Guide-9 gate record exists;
- the exact starting revision is recorded;
- the repository tree is clean;
- the second-review findings are accepted into the active backlog;
- the native report cannot pass an incomplete census;
- reports bind complete fixture subjects;
- the exact target definition is bound to the development binding;
- the executor reports the chain it actually runs;
- protocol messages are bounded;
- timeout cleanup covers the full executor process group;
- required additional target primitives have completed source review or are explicitly recorded as missing.

Before editing:

```sh
git status --porcelain=v1 --untracked-files=all
git log -1 --oneline
```

The tree must be clean.

## 2.1 Blocking second-review findings · `tbl:guide10:blockers`

| Finding | Required disposition before prototype evidence |
|---|---|
| native-report census incompleteness | validated report wrapper with exact censuses |
| broad evidence rows pass on partial coverage | typed subclaim coverage |
| reports omit exact fixture subjects | complete fixture projection |
| deployment binding retains only target version | exact reviewed target binding |
| signature operand model cannot express documented behavior | repair before signature-dependent handoff |
| network/genesis are caller declarations | observed environment binding |
| relation ID/body mismatch is not generically checked | repair before compiler-visible expansion |
| executor provenance is weaker than ADR-018 | typed provenance or explicit non-claim |
| timeout kills only immediate process | process-group supervision |
| protocol records are unbounded | typed record-size limits |
| schema-2 release-valid naming overclaims | split pre-release and production states |
| blank protocol records are accepted | strict documented framing |
| public status documentation is stale | reconcile during preflight |

The signature finding does not block a deliberately signature-free standalone constructor experiment. It blocks any claim that the constructor is ready for the operator-authorized maturity operation.

The deployment-profile limitation does not block prototype execution. It must nevertheless stop being represented by a type named production-release-valid.

---

# 3. Required reading and authority · `sec:guide10:authority`

Read repository policy first:

```text
AGENTS.md

adr/010-command-line-output-contract.md
adr/011-toolchain-and-dependency-policy.md
adr/014-meson-lint-census-and-stamps.md
adr/015-public-data-and-execution-trust.md
adr/016-semantic-identities-and-evidence-binding.md
adr/017-path-scope-and-host-filesystem-trust.md
adr/018-upstream-elements-workspace.md
```

Read accepted implementation decisions:

```text
plans/decisions/001-typed-rust-source.md
plans/decisions/003-tapscript-first.md
plans/decisions/004-translation-validation.md
plans/decisions/005-value-representation.md
plans/decisions/006-transaction-abi.md
plans/decisions/008-exact-certified-mathematics.md
```

Read package and phase contracts:

```text
plans/packages/compiler.md
plans/packages/target-elements.md
plans/packages/tapscript.md
plans/phases/03-target-foundation.md
```

Read the research owners:

```text
plans/research/state-constructor.md
plans/research/wide-arithmetic.md
plans/research/public-declassification.md
```

Read the target review and Guide-9 execution record:

```text
plans/reference/elements-tapscript.md
plans/guides/guide_nine.md
```

Read the implemented target and evidence boundaries:

```text
packages/target-elements/src/
packages/tapscript/src/
packages/target-elements-conformance/src/
scripts/elements-native-executor.py
scripts/elements-native-executor.sh
```

Relevant realization imports include:

```text
(`[RZ-sec:arithmetic:gadgets]`)
(`[RZ-rule:translation:division]`)
(`[RZ-rule:translation:bind]`)
(`[RZ-rule:translation:struct]`)
(`[RZ-rule:translation:cross-utxo]`)
(`[RZ-rule:translation:certificate-leaf]`)
(`[RZ-inv:invariant:succession]`)
(`[RZ-pin:pins:arith]`)
(`[RZ-pin:pins:ident]`)
(`[RZ-pin:pins:weld]`)
```

Planning and research prose remains non-normative and is never parsed as semantic input.

---

# 4. Scope · `sec:guide10:scope`

## 4.1 In scope

Guide 10 implements or resolves:

- native-report validation hardening required by the prototypes;
- exact fixture binding;
- exact evidence-claim coverage;
- exact reviewed target/development binding;
- observed chain identity in the native protocol;
- ADR-018 executor provenance;
- executor process-group cleanup;
- protocol message limits and strict framing;
- signature operand/failure representation required by later STATE use;
- relation-ID/body welding required before compiler-visible expansion;
- source review for the minimum additional target primitives;
- typed target contracts for admitted additional primitives;
- target-native vectors for every newly admitted primitive;
- a generic metadata-dependent constructor prototype;
- an exact host constructor oracle;
- constructor continuity, tree, tweak, and escape mutations;
- at least one exact wide-floor target pattern;
- an exact host wide-floor oracle;
- target-native wide-floor vectors;
- exact abstract stack validation for both prototypes;
- target resource measurements;
- deterministic prototype reports;
- accepted or rejected research conclusions;
- Phase-3 and backlog handoff.

## 4.2 Out of scope

Guide 10 must not implement:

- `announce-maturity`;
- any other attestation-contract operation program;
- final STATE metadata ABI;
- final constructor symbols or relocations;
- a linked bundle;
- production taptree policy;
- transaction request types;
- transaction construction;
- transaction signing;
- wide arithmetic inside redemption, settlement, or cycle;
- architecture-bound calibration;
- public declassification;
- direct confidential burn or redemption;
- production network activation;
- deployment release;
- release evidence identity;
- production wallet or signer support.

A synthetic STATE-like metadata structure may exercise field changes. It must not be published as the production STATE ABI.

---

# 5. Preflight correctness series · `sec:guide10:preflight`

## 5.1 Validated native report · `rule:guide10:report-validator`

Introduce:

```rust
pub struct ValidatedNativeConformanceReport {
    report: NativeConformanceReport,
}
```

The raw report remains a DTO. It carries no assurance until owner validation succeeds.

Validation inputs include:

```rust
pub struct NativeReportValidationInputs<'a> {
    pub target: &'a ReviewedElementsTapscriptDefinition,
    pub binding: &'a ReviewedDevelopmentBinding,
    pub fixtures: &'a PrimitiveFixtureSet,
    pub plan: &'a EvidencePlan,
    pub transcript: &'a ExecutionTranscript,
}
```

Equivalent factoring is acceptable.

The validator requires:

1. supported report schema;
2. exact target contract version and projection;
3. exact reviewed development binding;
4. exact observed executor environment;
5. exact case census;
6. no duplicate case;
7. no unexpected case;
8. exact fixture projection for each case;
9. recomputed case comparison;
10. exact evidence-row census;
11. exact plan class for every row;
12. exact typed claim coverage;
13. recomputed evidence disposition;
14. recomputed summary;
15. recomputed report completeness;
16. nonmock declaration for a native-evidence gate.

The validator rejects both directions of every mismatch.

## 5.2 Native protocol and report schema migration · `rule:guide10:schema-migration`

The protocol and report shapes change materially. Guide 10 therefore does not silently reinterpret schema 1.

Introduce:

```text
native executor protocol schema 2
native conformance report schema 2
```

Schema 2 adds, at minimum:

- executor environment observation;
- complete executor provenance;
- bounded-record contract;
- complete fixture projection in report rows;
- typed evidence claims;
- exact report-validation support.

Schema-1 Guide-9 reports remain historical evidence for their exact tree. They do not become schema-2 reports by reparsing or field synthesis.

A schema-2 gate rejects schema 1.

## 5.3 Evidence-claim registry · `rule:guide10:claim-registry`

Define one complete typed claim census.

Each claim records:

- owning evidence requirement;
- subject;
- fixture groups or cases capable of exercising it;
- whether Guide 10 requires it;
- whether it remains unresolved;
- exact reason when unresolved.

The claim registry is first-party policy, not executor output.

No wildcard arm absorbs a future claim into an existing evidence family.

## 5.4 Exact fixture projection · `rule:guide10:fixture-projection`

Define a canonical projection:

```rust
pub struct PrimitiveFixtureProjection {
    pub case: NativeCaseId,
    pub target_contract: TargetProjection,
    pub deployment: DeploymentProjection,
    pub execution_domain: WireExecutionDomain,
    pub leaf_version: u8,
    pub leaf_status: LeafVersionStatus,
    pub enforcement_layer: EnforcementLayer,
    pub script_source: FixtureScriptSource,
    pub script: Vec<u8>,
    pub initial_stack: Vec<Vec<u8>>,
    pub context: Option<PrimitiveExecutionContext>,
    pub expected: ExpectedPrimitiveOutcome,
    pub expected_resources: ExpectedResourceObservation,
    pub claims: BTreeSet<NativeEvidenceClaim>,
}
```

Exact fields may differ.

Projection construction validates local fixture structure. A malformed fixture does not acquire a report subject.

## 5.5 Reviewed deployment binding · `rule:guide10:reviewed-deployment-binding`

Introduce a reviewed binding state, or store exact target equality in the generic binding.

Recommended form:

```rust
pub struct ReviewedDevelopmentBinding {
    target: TargetProjection,
    deployment: ValidatedDevelopmentBinding,
}
```

Only:

```rust
pub fn validate_reviewed_development_binding(
    target: &ReviewedElementsTapscriptDefinition,
    binding: DevelopmentDeploymentBinding,
) -> Result<ReviewedDevelopmentBinding, TargetError>;
```

constructs it.

Native fixture construction requires the reviewed binding.

A generic binding validated against one target cannot combine with another same-version target.

## 5.6 Observed environment · `rule:guide10:environment-observation`

Extend the executor handshake or add a post-handshake environment message:

```rust
pub struct ExecutorEnvironmentObservation {
    pub environment: WireEnvironment,
    pub chain_name: String,
    pub network_id: [u8; 32],
    pub genesis_id: [u8; 32],
    pub execution_domains: BTreeSet<WireExecutionDomain>,
    pub active_leaf_versions: BTreeSet<u8>,
}
```

The identifier recipes must be documented.

For the real Elements adapter:

- genesis derives from the actual node;
- chain name derives from the actual node/configuration;
- activation derives from node-observable state or an explicitly scoped declaration;
- no synthetic caller ID is reported as observed genesis.

A mismatch blocks the run before case execution.

## 5.7 Executor provenance · `rule:guide10:executor-provenance`

Record distinct provenance roles:

```rust
pub struct ExecutorProvenance {
    pub adapter_name: String,
    pub adapter_version: String,
    pub framework_revision: Option<String>,

    pub node_name: String,
    pub node_version: String,
    pub binary_reported_revision: Option<String>,

    pub intended_executed_tip: Option<String>,
    pub upstream_base: Option<String>,
    pub included_local_topics: BTreeSet<String>,
}
```

The exact shape may differ, but it must distinguish:

```text
what the binary says it is

what checkout or integration tip the operator intended

what upstream base that tip derives from

which local fix/notes branches were included

which adapter and transaction framework constructed the run
```

Checkout `HEAD` must not be substituted for a binary-reported revision.

If the executor cannot establish ADR-018 provenance, the report says so and cannot satisfy an evidence claim requiring it.

## 5.8 Process-group timeout · `rule:guide10:process-group`

On POSIX systems:

1. spawn the executor in a new process group;
2. keep protocol pipes owned by the harness;
3. on timeout send graceful termination to the group;
4. wait a bounded cleanup interval;
5. kill the group if it remains alive;
6. reap the direct child;
7. close protocol pipes;
8. report timeout as infrastructure failure.

Tests must include a child that:

- forks a descendant;
- keeps stdout open;
- ignores graceful termination;
- would survive if only the immediate process were killed.

The test must confirm no descendant remains.

## 5.9 Protocol record limits · `rule:guide10:protocol-limits`

Define explicit limits:

```rust
pub struct ProtocolLimits {
    pub maximum_handshake_bytes: usize,
    pub maximum_environment_bytes: usize,
    pub maximum_response_bytes: usize,
    pub maximum_trailing_record_bytes: usize,
}
```

Read no more than `maximum + 1` bytes before rejecting.

Required cases:

- exactly maximum;
- maximum plus one;
- no newline;
- empty line;
- whitespace-only line;
- deeply nested JSON inside the byte bound;
- trailing JSON;
- trailing blank line.

The protocol is strict NDJSON:

```text
one nonempty JSON object per line
no blank records
no trailing records
```

## 5.10 Response shape validation · `rule:guide10:response-shape`

Validate protocol combinations:

```text
Accepted:
    observed_failure absent

Rejected:
    observed_failure present when failure-class reporting is advertised

InfrastructureError:
    no target failure class
    no target final-stack claim

FinalStackReporting advertised:
    final-stack presence follows the protocol contract

FinalAltstackReporting advertised:
    final-altstack presence follows the protocol contract
```

An accepted response carrying a failure class is malformed protocol, not a passing case.

## 5.11 Signature relation repair · `rule:guide10:signature-repair`

Represent signature operand forms and failure triggers explicitly.

A suitable conceptual model is:

```rust
pub enum OperandContract {
    Exact(StackValueType),
    OneOf(BTreeSet<StackValueType>),

    Signature {
        nonempty_encoding: EncodingClass,
        empty_allowed: bool,
    },

    PublicKey {
        recognized_encoding: EncodingClass,
        unknown_nonempty_allowed: bool,
    },
}

pub enum FailureCondition {
    StackUnderflow,
    SignatureEmpty,
    SignatureNonemptyInvalid,
    PublicKeyEmpty,
    PublicKeyMalformed,
    PublicKeyUnknownNonempty,
    ValidationBudgetExceeded,
}
```

The abstract validator must distinguish:

```text
empty signature:
    no success
    consume operands
    push false, or abort for verify form

recognized valid signature:
    success

recognized nonempty invalid signature:
    abort

unknown nonempty public-key type:
    exact reviewed compatibility behavior

empty public key:
    reject
```

Add native cases for the unknown-key behavior before signature evidence is complete.

## 5.12 Relation ID/body weld · `rule:guide10:relation-weld`

Add exhaustive validation deriving relation identity from relation body.

Representative mapping:

```text
Relation::Cardinality
    → RelationKind::Cardinality
    → object-family subject

Relation::Recognition
    → RelationKind::Recognition
    → object-family subject

Relation::AmountConservation
    → RelationKind::Conservation
    → asset subject

Relation::ExpressionPredicate
    → RelationKind::ExpressionPredicate
    → expression or operation subject

Relation::LifecycleExit
    → RelationKind::Lifecycle
    → lifecycle-exit subject
```

Every variant is covered without a wildcard.

Remove unused relation-kind members or give them exact body variants before use.

## 5.13 Deployment profile state · `rule:guide10:deployment-profile-state`

Split:

```text
profile structural validity

production deployment-release validity
```

Schema 2 may satisfy the first.

It cannot satisfy the second while calibration does not bind the final transaction ABI/configuration.

No Guide-10 report is placed into the deployment profile.

## 5.14 Documentation reconciliation · `rule:guide10:documentation-preflight`

Update stale public claims:

- root README lists `target-elements-conformance`;
- root README no longer says tapscript has no instruction core;
- target package documentation states that it owns requirements, not mutable evidence completion;
- development native evidence is attributed to the conformance package;
- production target evidence remains absent.

---

# 6. Package and file plan · `sec:guide10:files`

## 6.1 `target-elements`

Likely changes:

```text
packages/target-elements/src/opcode.rs
packages/target-elements/src/success.rs
packages/target-elements/src/authorization.rs
packages/target-elements/src/capability.rs
packages/target-elements/src/evidence.rs
packages/target-elements/src/evidence_registry.rs
packages/target-elements/src/definition.rs
packages/target-elements/src/deployment.rs
packages/target-elements/src/error.rs
packages/target-elements/src/weld.rs
packages/target-elements/src/tests/...
packages/target-elements/README.md
packages/target-elements/meson.build
```

The package remains dependency-free.

## 6.2 `tapscript`

Suggested new modules:

```text
packages/tapscript/src/constructor.rs
packages/tapscript/src/wide_arithmetic.rs
packages/tapscript/src/prototype.rs
```

Existing modules may be extended:

```text
packages/tapscript/src/instruction.rs
packages/tapscript/src/program.rs
packages/tapscript/src/stack.rs
packages/tapscript/src/error.rs
packages/tapscript/src/lib.rs
packages/tapscript/src/tests/...
packages/tapscript/README.md
packages/tapscript/meson.build
```

If separate modules would expose a misleading production API, keep them crate-private and expose one narrow prototype boundary.

## 6.3 `target-elements-conformance`

Suggested additions:

```text
packages/target-elements-conformance/src/claim.rs
packages/target-elements-conformance/src/environment.rs
packages/target-elements-conformance/src/prototype.rs
packages/target-elements-conformance/src/prototype_report.rs
packages/target-elements-conformance/src/supervisor.rs

packages/target-elements-conformance/src/census/constructor.rs
packages/target-elements-conformance/src/census/wide_arithmetic.rs
```

Existing modules requiring migration:

```text
src/protocol.rs
src/executor.rs
src/fixture.rs
src/report.rs
src/validate.rs
src/error.rs
src/bin/check-target-elements-native.rs
src/bin/mock-native-executor.rs
src/tests/...
tests/executor_protocol.rs
tests/subprocess_contract.rs
README.md
meson.build
```

The native checker may remain one command if report roles are explicit. A separate prototype checker is preferable if one command would make primitive and prototype evidence indistinguishable.

Possible command:

```text
check-target-elements-prototypes
```

It follows ADR-010 and ADR-014.

## 6.4 Scripts

Likely changes:

```text
scripts/elements-native-executor.py
scripts/elements-native-executor.sh
scripts/test-elements-native-executor.sh
scripts/test-meson-mock.sh
meson.build
meson.options
```

No credential option is added.

## 6.5 Planning

Add this guide to:

```text
plans/guides/README.md
plans/guides/meson.build
```

Update:

```text
plans/backlog.md
plans/phases/03-target-foundation.md
plans/research/state-constructor.md
plans/research/wide-arithmetic.md
plans/reference/elements-tapscript.md
plans/packages/target-elements.md
plans/packages/tapscript.md
plans/packages/README.md
```

Every new tracked file joins its nearest explicit Meson census in the same commit.

---

# 7. Target contract revision and primitive closure · `sec:guide10:target-v2`

## 7.1 Contract revision · `rule:guide10:target-version`

Adding new reviewed primitives or materially changing the operand/success/failure algebra changes the typed target contract.

Guide 10 therefore introduces:

```rust
TargetContractVersion::V2
```

when the reviewed primitive census or semantic contract changes.

V1 remains the historical Guide-9 contract. It is not silently widened.

The V2 reviewed target includes:

- the corrected signature operand/failure relation;
- every newly reviewed compound-proof primitive;
- updated capability closure;
- updated evidence requirements;
- updated native evidence plan.

If the implementation can prove a change is solely a correction to V1’s measurement without changing any consumer-visible target meaning, it may propose another migration. The default is V2 because the instruction census and successful relation are expanding.

## 7.2 Primitive-needs census · `tbl:guide10:primitive-needs`

Before writing prototype code, fill this table from complete symbolic schedules:

| Need | Constructor | Wide floor | Existing reviewed primitive | Decision |
|---|---:|---:|---|---|
| duplicate top item | | | | |
| swap two items | | | | |
| rotate short frame | | | | |
| remove item | | | | |
| equality | | | | |
| equality-and-abort | | | | |
| verify Boolean | | | | |
| conditional branch | | | | |
| byte concatenation | | | | |
| byte split/slice | | | | |
| byte lexicographic order | | | | |
| streaming hash | | | existing | |
| signed fixed-width arithmetic | | | existing | |
| signed comparison | | | existing | |
| script-number conversion | | | existing | |
| input/output program inspection | | | existing | |
| tweak verification | | | existing | |

No row remains “assumed”.

## 7.3 Symbolic stack schedule · `rule:guide10:stack-schedule`

For each leading candidate, write a machine-checked symbolic schedule before implementation.

Each step records:

```rust
pub struct ScheduledInstruction {
    pub instruction: TapscriptInstruction,
    pub success_before: AbstractStackState,
    pub success_after: BTreeSet<AbstractStackState>,
    pub nonaborting_failure_after: BTreeSet<AbstractStackState>,
    pub aborts: BTreeSet<FailureCause>,
}
```

Equivalent representation is acceptable.

The schedule must prove:

- all operands exist;
- operand forms are admitted;
- all success alternatives are handled;
- no failure state rejoins success;
- all arithmetic flags are consumed;
- proof-local values are removed;
- required result values survive;
- final truth is canonical where the standalone fixture needs acceptance;
- stack/resource bounds hold.

## 7.4 Primitive review procedure · `rule:guide10:primitive-review`

For every new primitive:

1. locate the exact upstream declaration;
2. locate its interpreter implementation;
3. locate any upstream tests;
4. record target byte;
5. record execution domain;
6. record operands in deepest-first order;
7. record every success form;
8. record every failure cause and effect;
9. record resource cost;
10. record required encodings;
11. record target evidence requirements;
12. add independent expected-byte test;
13. add production abstract-stack tests;
14. add native positive and negative cases;
15. add cross-contract welds;
16. update the human target reference.

The review location and revision remain human/test provenance, not target identity.

---

# 8. Typed prototype boundary · `sec:guide10:prototype-types`

## 8.1 Prototype program

Use a private-field type:

```rust
pub struct PrototypeProgram {
    kind: PrototypeKind,
    target: TargetProjection,
    program: TapscriptProgram,
    initial_stack: Vec<StackValueType>,
    expected: PrototypeRelation,
    resources: PrototypeResourceProjection,
}
```

Possible kinds:

```rust
pub enum PrototypeKind {
    MetadataConstructorContinuity,
    WideFloor,
}
```

A public constructor validates:

- reviewed target version;
- instruction support;
- abstract stack relation;
- resource projection;
- prototype-only status.

## 8.2 No production conversion

Do not implement:

```rust
impl From<PrototypeProgram> for RelocatableProgram
```

or any equivalent implicit promotion.

A later accepted decision may define a separately reviewed lowering from an accepted research result. Guide 10 does not.

## 8.3 Stable comparison

Prototype programs compare by:

- typed kind;
- target projection;
- exact typed instructions;
- exact bytes;
- typed witness contract;
- exact abstract stack result;
- resource projection.

No digest is required.

---

# 9. STATE constructor prototype · `sec:guide10:constructor`

## 9.1 Fixed relation · `rule:guide10:constructor-relation`

The prototype models one metadata-dependent constructor:

```text
Constructor(M, C, P, S)
```

where:

- \(M\) is canonical public metadata;
- \(C\) is one authenticated static code-root value;
- \(P\) is one fixed public internal key;
- \(S\) is one constructor schema and target recipe.

The accepted transition relation is:

```text
consume Constructor(M_before, C, P, S)

derive M_after through one synthetic public transition

create Constructor(M_after, C, P, S)
```

The same \(C\), \(P\), and \(S\) bind both sides.

A caller may supply witness data needed to reconstruct the constructor. The witness must not be free to select a different static relation.

## 9.2 Leading construction · `rule:guide10:dynamic-metadata-leaf`

The leading candidate is a dynamic metadata leaf beside a static code subtree:

```text
static operation subtree root:
    C

dynamic metadata leaf:
    L(M)

tree root:
    R(M) = branch(C, L(M))

target output program:
    Q(M) = tweak(P, R(M))
```

The exact target formulas come from the reviewed target.

Conceptually:

\[
h_{\mathrm{leaf}}(M)=H_{\mathrm{TapLeaf}}(v\parallel\operatorname{compactSize}(|s(M)|)\parallel s(M))
\]

\[
h_{\mathrm{branch}}(C,h)=H_{\mathrm{TapBranch}}(\min(C,h)\parallel\max(C,h))
\]

\[
t(M)=H_{\mathrm{TapTweak}}(P\parallel h_{\mathrm{branch}}(C,h_{\mathrm{leaf}}(M)))
\]

\[
Q(M)=P+t(M)G
\]

Every domain tag, framing rule, child order, scalar rule, parity rule, and output-program encoding must be reviewed and target-evidenced.

## 9.3 Prototype metadata schema · `rule:guide10:prototype-metadata`

Stage 1 uses synthetic fixed-width metadata:

```text
constructor domain tag
prototype schema number
object-kind test tag
counter
flags
reserved-zero field
```

The transition is:

```text
counter_after = counter_before + 1
all other fields unchanged
```

The schema is deliberately STATE-like:

- multiple independently mutable fields;
- explicit domain;
- explicit schema;
- canonical field order;
- exact widths;
- reserved field with exact zero requirement.

It is not the final STATE ABI.

A representative conceptual encoding is:

```text
domain             16 or 32 bytes
schema             4-byte unsigned little-endian
object kind        4-byte unsigned little-endian
counter            8-byte unsigned little-endian
flags              4-byte unsigned little-endian
reserved           8 zero bytes
```

Exact widths are prototype design choices and are not protocol semantics.

## 9.4 Static subtree input · `rule:guide10:static-root`

The prototype receives one static subtree root \(C\) as a public witness.

The program must prove that \(C\):

- participates in the predecessor constructor;
- participates unchanged in the successor constructor;
- is exactly one 32-byte hash;
- is not independently supplied twice;
- cannot be swapped after predecessor authentication.

The internal stack should carry one authenticated instance of \(C\) from the predecessor proof into the successor proof.

Re-reading another caller-supplied root for the successor is forbidden.

## 9.5 Predecessor authentication · `rule:guide10:predecessor`

The prototype program establishes:

1. the current input is the constructor instance being spent;
2. predecessor metadata is canonical;
3. the metadata leaf derives exactly from those bytes;
4. static root \(C\) participates in the predecessor tree;
5. fixed internal key \(P\) participates;
6. the predecessor output key/program equals the consumed input program;
7. the executing path belongs to the admitted static operation subtree;
8. the metadata leaf is not the executing path.

Input-program introspection reads the actual consensus program.

An unauthenticated caller-supplied predecessor key is insufficient.

## 9.6 Successor metadata · `rule:guide10:successor-metadata`

From authenticated predecessor metadata:

```text
counter_after = checked(counter_before + 1)
```

The prototype requires:

- no overflow;
- unchanged domain;
- unchanged schema;
- unchanged object-kind tag;
- unchanged flags;
- unchanged reserved-zero field.

Every arithmetic success flag is verified immediately.

The successor metadata bytes are then encoded canonically.

## 9.7 Successor constructor · `rule:guide10:successor-constructor`

The program derives and verifies:

1. successor metadata leaf;
2. successor dynamic/static branch root;
3. successor tweak;
4. successor output key/program;
5. exact output role or index;
6. same static root \(C\);
7. same internal key \(P\);
8. same schema \(S\).

A correct target program under semantically wrong successor metadata still rejects under the metadata relation. Constructor continuity and semantic state assignment remain separate claims.

## 9.8 TapBranch ordering · `rule:guide10:tapbranch-order`

Canonical child ordering is load-bearing.

Accepted strategies include:

- reviewed byte-lexicographic target comparison;
- a verified orientation witness;
- a target primitive verifying complete branch construction;
- another exact reviewed construction.

A verified orientation witness must establish:

```text
orientation = first
    iff C ≤ h_metadata under target byte ordering

orientation = second
    iff h_metadata < C
```

It must then feed the exact branch-hash order.

It is not sufficient to:

- hash children in caller order;
- trust a branch-order bit;
- accept either concatenation;
- validate only that some tweaked key matches;
- infer target ordering from host language ordering without a cross-check.

If canonical ordering cannot be enforced, the leading candidate is rejected.

## 9.9 Metadata leaf unspendability · `rule:guide10:metadata-unspendable`

The metadata leaf must be an actual fail-closed target script.

The prototype must review and type its unspendable construction.

An empty script is not automatically unspendable: an initial witness item may survive and satisfy final truth.

Acceptable metadata leaf candidates include:

- a reviewed unconditional-abort primitive;
- another script whose every possible initial stack aborts or ends invalid;
- a target construction that marks the leaf unspendable by consensus.

Required evidence includes:

```text
operation leaf:
    valid fixture accepts

metadata leaf:
    every tested witness shape rejects

metadata leaf with one true witness item:
    rejects

metadata leaf with arbitrary nonempty witness:
    rejects
```

The metadata leaf’s failure must be a target result, not a wallet convention.

## 9.10 Internal-key policy · `rule:guide10:internal-key`

Use a deterministic public point with no known private scalar.

The prototype must not use:

- operator key;
- release key;
- test signing key;
- generated-and-discarded private key;
- mutable deployment key.

The report records the exact derivation or published constant.

The absence of a known private scalar is a residual cryptographic assumption. The prototype does not claim to prove discrete-log impossibility.

## 9.11 Output-key parity · `rule:guide10:parity`

Target output programs carry x-only keys while tweak verification may consume compressed points.

The prototype must type and verify the bridge:

```text
x-only program payload
+
parity witness
→
compressed output key
```

or use another reviewed exact target mechanism.

Required mutations:

- wrong parity;
- wrong x coordinate;
- compressed point with wrong prefix;
- x-only payload from another output;
- malformed point;
- correct tweak under the wrong output program.

A parity bit is a witness, not an authority. It must be checked.

## 9.12 Tweak totality · `rule:guide10:tweak-totality`

The constructor defines what happens when target tweak/scalar rules reject a derived value.

Candidate policies:

| Policy | Effect |
|---|---|
| Reject rare constructor instance | simplest; constructor not total |
| Canonical metadata representation nonce | public deterministic retry |
| Canonical internal-key retry | instance-specific public key policy |
| Named negligible residual | accepted constructibility limitation |

No policy is accepted silently.

If retry is selected, it must be:

- canonical;
- publicly computable;
- bounded or total under a reviewed argument;
- part of the prototype schema;
- included in resource measurements;
- semantically erased from the prototype state value;
- unable to alter the counter transition.

## 9.13 Constructor candidate matrix · `tbl:guide10:constructor-candidates`

| Candidate | Description | Main concern |
|---|---|---|
| Dynamic metadata leaf | Static root plus unspendable metadata leaf | Branch ordering, totality, metadata path |
| Witnessed root continuity | Witness one root and verify predecessor/successor instances | Must bind the witness to both target programs |
| Separate metadata output | Fixed program plus separately welded metadata object | Changes object architecture |
| Metadata in every operation leaf | Every operation leaf commits metadata | Recursive/static-code explosion |
| Alternate target commitment field | Metadata committed outside the tree | Availability and target semantics uncertain |

Only the first two are candidates for acceptance without upstream semantic change.

The separate-metadata-output candidate may be measured but not adopted by this guide.

## 9.14 Constructor threat matrix · `tbl:guide10:constructor-threats`

| Mutation | Required result |
|---|---|
| predecessor counter changed | reject |
| predecessor domain changed | reject |
| predecessor schema changed | reject |
| predecessor object kind changed | reject |
| predecessor flags changed | reject |
| predecessor reserved field nonzero | reject |
| successor counter unchanged | reject |
| successor counter incremented by two | reject |
| successor unaffected field changed | reject |
| malformed metadata length | reject |
| alternate field order | reject |
| trailing metadata bytes | reject |
| wrong metadata leaf version | reject |
| wrong static root | reject |
| predecessor and successor use different roots | reject |
| wrong internal key | reject |
| wrong output-key parity | reject |
| wrong TapLeaf framing | reject |
| wrong TapBranch order | reject |
| wrong TapTweak input | reject |
| wrong successor output role | reject |
| operation leaf removed | reject |
| extra escape leaf introduced | reject |
| metadata leaf selected for spending | reject |
| constructor from another schema | reject |
| stale prototype constructor | reject |
| target constructor correct, state transition wrong | reject |

---

# 10. Constructor prototype stages · `sec:guide10:constructor-stages`

## 10.1 Stage C1 — Independent host constructor · `rule:guide10:constructor-host`

Implement a pure typed host reference for:

- canonical metadata encoding;
- metadata leaf script;
- TapLeaf hash;
- branch ordering;
- TapBranch hash;
- tweak hash;
- tweaked key;
- output program;
- control path.

The host reference is separate from the target program builder.

Cross-check against:

- published target vectors where available;
- upstream functional-test framework;
- another reviewed target-compatible implementation where practical.

## 10.2 Stage C2 — Generic tree fixture · `rule:guide10:constructor-fixture`

Extend the fixture language to materialize:

```text
fixed internal key
operation leaf
metadata leaf
static sibling/subtree
executing path
successor target program
```

The executor must build exactly the requested tree and verify that:

- the requested operation leaf is the one spent;
- the control path belongs to that tree;
- the successor output uses the requested target program;
- no unrequested output or tree node is introduced.

## 10.3 Stage C3 — Predecessor-only proof · `rule:guide10:constructor-predecessor-stage`

First prove only:

```text
consumed program
=
Constructor(M_before, C, P, S)
```

Required cases:

- correct predecessor;
- each metadata mutation;
- wrong static root;
- wrong internal key;
- wrong branch order;
- wrong parity;
- malformed metadata;
- metadata-leaf spend.

No successor is introduced until predecessor authentication is exact.

## 10.4 Stage C4 — Successor-only proof · `rule:guide10:constructor-successor-stage`

With a separately trusted predecessor fixture, prove:

```text
created program
=
Constructor(M_after, C, P, S)
```

Required cases:

- correct successor;
- wrong metadata;
- wrong static root;
- wrong internal key;
- wrong output role;
- wrong target program;
- wrong parity.

## 10.5 Stage C5 — Continuity composition · `rule:guide10:constructor-continuity-stage`

Compose the two proofs while retaining one authenticated static root.

The target program must not accept:

```text
predecessor Constructor(M0, C0)
successor   Constructor(M1, C1)
C0 ≠ C1
```

even when both constructors are independently well-formed.

## 10.6 Stage C6 — Synthetic state transition · `rule:guide10:constructor-transition-stage`

Add exact counter increment and unchanged-field checks.

The target program’s accepted result must establish both:

```text
constructor continuity
semantic metadata transition
```

No target result is called a STATE transition.

## 10.7 Stage C7 — Resource comparison · `rule:guide10:constructor-resource-stage`

Measure the leading candidate and at least one meaningful alternative far enough to justify selection.

If the leading candidate cannot fit a standalone target transaction, reject it before integration planning.

---

# 11. Exact wide-floor prototype · `sec:guide10:wide-floor`

## 11.1 Fixed relation · `rule:guide10:wide-floor-relation`

The prototype proves:

\[
a·b=q·d+r
\]

with:

\[
0\le r<d,\qquad 0\le a,b,q<2^{51},\qquad 0<d<2^{51}.
\]

These conditions imply:

\[
q=\left\lfloor\frac{a·b}{d}\right\rfloor.
\]

The target proof establishes the complete relation.

Host-generated \(q\) and \(r\) are witnesses, not trusted answers.

## 11.2 Exact host oracle · `rule:guide10:wide-floor-host`

The host reference computes:

```rust
product = a·b
q = product / d
r = product % d
```

Because \(a,b<2^{51}\):

\[
a·b<2^{102}<2^{128}.
\]

The reference may use `u128`.

Property tests should also compare with arbitrary-precision arithmetic so a later domain change cannot silently invalidate the oracle.

The host oracle returns:

- \(q\);
- \(r\);
- canonical operand limbs;
- canonical product limbs;
- every expected carry;
- exact intermediate maxima;
- mutation candidates.

## 11.3 Candidate A — derived limbs · `candidate:guide10:derived-limbs`

Use base:

\[
B=2^{26}.
\]

Each semantic amount decomposes:

\[
x=x_0+x_1B
\]

with:

```text
0 ≤ x0 < 2^26
0 ≤ x1 < 2^25
```

The target derives limbs from the original amount through exact target division by \(B\), rather than trusting caller-supplied limbs, if the measured pattern is feasible.

For:

\[
a=a_0+a_1B,\qquad b=b_0+b_1B,
\]

derive raw coefficients:

\[
c_0=a_0b_0
\]

\[
c_1=a_0b_1+a_1b_0
\]

\[
c_2=a_1b_1.
\]

Normalize:

```text
p0     = c0 mod B
carry0 = floor(c0 / B)

t1     = c1 + carry0
p1     = t1 mod B
carry1 = floor(t1 / B)

t2     = c2 + carry1
p2     = t2 mod B
p3     = floor(t2 / B)
```

Then:

\[
a·b=p_0+p_1B+p_2B^2+p_3B^3.
\]

Compute the same product limbs for \(q·d\), add \(r\), and compare all four normalized limbs exactly.

## 11.4 Candidate A range proof · `rule:guide10:derived-limb-bounds`

For valid semantic inputs:

```text
a0,b0,q0,d0 < 2^26
a1,b1,q1,d1 < 2^25
```

Partial products satisfy:

\[
a_0b_0<2^{52}
\]

\[
a_0b_1<2^{51}
\]

\[
a_1b_0<2^{51}
\]

\[
a_1b_1<2^{50}.
\]

First normalization:

\[
\operatorname{carry}_0<2^{26}.
\]

Middle coefficient:

\[
t_1=a_0b_1+a_1b_0+\operatorname{carry}_0<2^{52}+2^{26}<2^{53}.
\]

Therefore:

\[
\operatorname{carry}_1<2^{27}.
\]

Top coefficient:

\[
t_2=a_1b_1+\operatorname{carry}_1<2^{50}+2^{27}<2^{51}.
\]

Therefore:

\[
p_3<2^{25}.
\]

Every intermediate is below \(2^{53}\), safely inside signed 64-bit arithmetic.

The same bounds apply to \(q·d\).

## 11.5 Adding the remainder · `rule:guide10:remainder-addition`

Decompose:

\[
r=r_0+r_1B
\]

with:

```text
0 ≤ r0 < 2^26
0 ≤ r1 < 2^25
```

For product limbs \(u_0,u_1,u_2,u_3\) of \(q·d\):

```text
s0     = u0 + r0
z0     = s0 mod B
carry0 = floor(s0/B)

s1     = u1 + r1 + carry0
z1     = s1 mod B
carry1 = floor(s1/B)

s2     = u2 + carry1
z2     = s2 mod B
carry2 = floor(s2/B)

z3     = u3 + carry2
```

Bounds:

```text
s0 < 2^27
carry0 ≤ 1

s1 < 2^27
carry1 ≤ 1

s2 ≤ 2^26
carry2 ≤ 1

z3 < 2^25 + 1
```

Finally require:

```text
p0 = z0
p1 = z1
p2 = z2
p3 = z3
```

and separately:

```text
0 ≤ r < d
d > 0
q < 2^51
```

## 11.6 Candidate B — witnessed limbs and carries · `candidate:guide10:witnessed-limbs`

The caller supplies:

```text
q
r
operand limbs
product limbs
carry witnesses
```

The target:

- checks each original amount;
- checks every limb bound;
- recomposes every amount;
- checks every partial-product equation;
- checks every carry equation;
- checks exact product equality;
- checks remainder bounds.

This candidate increases witness bytes but may reduce script and stack work.

A witness component that can vary without changing acceptance is either:

- removed as irrelevant; or
- represented as a deliberate equivalence class with no semantic effect.

No underconstrained carry or high limb is accepted.

## 11.7 Candidate C — sandwich proof · `candidate:guide10:sandwich`

Compare:

\[
q·d\le a·b<(q+1)·d.
\]

The candidate removes \(r\) from the witness but requires:

- wide \(a·b\);
- wide \(q·d\);
- wide \((q+1)·d\);
- two wide comparisons;
- exact \(q+1\) handling;
- high-limb comparison.

Prototype only far enough to decide whether it is materially simpler or smaller.

A shorter mathematical statement is not automatically a simpler target proof.

## 11.8 Limb representation · `rule:guide10:limb-encoding`

Each limb is a signed fixed-width target value constrained nonnegative.

Canonical limb encoding defines:

- exact width;
- little-endian order;
- upper bound;
- witness position;
- stack type;
- source relation.

Forbidden:

- script-number limb with variable width;
- negative limb;
- redundant top zero limb where the ABI requires exact limb count;
- byte-reversed limb;
- omitted high limb;
- extra limb;
- two decompositions of one amount.

## 11.9 Arithmetic success flags · `rule:guide10:success-flags`

Every target fixed-width arithmetic operation that returns a success flag must have that flag verified immediately or under a typed schedule proving it cannot be lost, overwritten, or confused with a result.

Required shape:

```text
operation
    ↓
result + success flag
    ↓
verify success flag
    ↓
authenticated result only
```

Forbidden:

- leave the flag for an unspecified caller;
- inspect the result while ignoring the flag;
- let retained failure operands satisfy the success stack shape;
- treat a pushed false as an abort;
- infer success from a nonempty stack.

The abstract stack validator must preserve and reject every non-aborting failure path.

## 11.10 Pattern output · `rule:guide10:wide-floor-output`

The accepted standalone pattern should expose:

```text
input:
    authenticated a,b,d
    public q,r witness

success:
    authenticated q remains in one canonical 8-byte form
    no unchecked flag remains
    no proof-local item remains

failure:
    abort or an explicitly rejected non-aborting state
```

The authenticated \(q\) must be the value a later operation consumes.

A pattern that proves one quotient and lets the caller provide another is rejected.

## 11.11 Wide-floor threat matrix · `tbl:guide10:wide-floor-threats`

| Mutation | Required result |
|---|---|
| exact quotient and remainder | accept |
| exact division with `r=0` | accept |
| `r=1` | accept |
| `r=d-1` | accept |
| `q-1` | reject |
| `q+1` | reject |
| correct equality with `r=d` | reject |
| zero divisor | reject |
| negative target operand | reject |
| operand equal to `2^51` | reject |
| quotient equal to `2^51` | reject |
| malformed 7-byte operand | reject |
| malformed 9-byte operand | reject |
| byte-reversed operand | reject |
| low limb above bound | reject |
| high limb above bound | reject |
| wrong first carry | reject |
| wrong second carry | reject |
| wrong top limb | reject |
| omitted high limb | reject |
| duplicated limb | reject |
| witness items reordered | reject |
| arithmetic flag left unchecked | static or target rejection |
| proof over substituted `a` | enclosing-binding rejection |
| proof over substituted `b` | enclosing-binding rejection |
| proof over substituted `d` | enclosing-binding rejection |

Boundary values include:

```text
0
1
B-1
B
B+1
2^25-1
2^25
2^26-1
2^51-2
2^51-1
```

---

# 12. Wide-floor prototype stages · `sec:guide10:wide-floor-stages`

## 12.1 Stage A1 — Exact oracle · `rule:guide10:arithmetic-oracle-stage`

Implement the exact host relation and fixed boundary vectors.

No target instructions yet.

Required oracle properties:

```text
a·b = q·d + r
r < d
q = floor(a·b/d)
```

## 12.2 Stage A2 — Independent limb normalizer · `rule:guide10:limb-oracle-stage`

Implement a small reference normalizer separately from the production target-pattern builder.

Compare:

- direct `u128` product;
- reconstructed product from normalized limbs;
- production host helper;
- property-generated cases.

## 12.3 Stage A3 — Bound proof · `rule:guide10:bound-proof-stage`

Write every symbolic maximum into:

- source documentation;
- focused boundary tests;
- a typed bound table where practical.

Any intermediate without a bound blocks target emission.

## 12.4 Stage A4 — Decomposition pattern · `rule:guide10:decomposition-stage`

Emit and test target fragments for:

```text
x → x0,x1
```

including:

- exact recomposition;
- range checks;
- malformed original amount;
- malformed target width;
- arithmetic failure paths.

## 12.5 Stage A5 — Product pattern · `rule:guide10:product-stage`

Emit and test:

```text
(a0,a1,b0,b1) → p0,p1,p2,p3
```

with exact carry checks.

Compare against host product limbs.

## 12.6 Stage A6 — Quotient/remainder relation · `rule:guide10:quotient-stage`

Compose:

```text
P(a,b)
P(q,d)
P(q,d)+r
exact equality
r<d
d>0
```

Retain authenticated \(q\).

## 12.7 Stage A7 — Adversarial matrix · `rule:guide10:arithmetic-adversarial-stage`

Run every fixed mutation, then deterministic generated cases.

Any unexpected target acceptance becomes a permanent minimized regression.

## 12.8 Stage A8 — Candidate comparison · `rule:guide10:arithmetic-comparison-stage`

Implement Candidate B only if Candidate A is:

- too large;
- too deep;
- difficult to schedule;
- difficult to audit;
- above target limits.

Implement Candidate C only if a complete schedule suggests a real reduction.

Select no candidate by aesthetic preference alone.

---

# 13. Generic compound fixtures · `sec:guide10:fixtures`

## 13.1 Distinct fixture type · `rule:guide10:compound-fixture`

Primitive fixtures and compound-prototype fixtures remain distinct.

Conceptually:

```rust
pub struct CompoundPrototypeFixture {
    pub case: PrototypeCaseId,
    pub claim: PrototypeClaim,
    pub program: PrototypeProgram,
    pub initial_stack: Vec<StackItem>,
    pub context: PrototypeExecutionContext,
    pub expected: ExpectedPrototypeOutcome,
    pub resources: ExpectedResourceObservation,
}
```

The fixture remains target-generic.

It may name:

```text
metadata constructor continuity
wide floor relation
```

It must not name:

- STATE;
- pool;
- receipt;
- redemption;
- settlement;
- cycle;
- architecture relation ID.

## 13.2 Taptree fixture · `rule:guide10:taptree-fixture`

The constructor prototype requires a typed target tree.

A suitable generic shape is:

```rust
pub enum FixtureTapTree {
    Leaf {
        version: u8,
        script: Vec<u8>,
    },

    Branch {
        left: Box<FixtureTapTree>,
        right: Box<FixtureTapTree>,
    },
}
```

Equivalent canonical path factoring is acceptable.

The fixture identifies:

- exact internal key;
- complete tree;
- executing leaf;
- exact leaf version;
- exact script bytes;
- successor output program;
- expected control path where independently known.

The executor materializes exactly that tree.

## 13.3 Stated versus supplied fields · `rule:guide10:stated-fields`

The fixture retains the Guide-9 rule:

```text
stated field:
    exact requirement the executor must materialize

absent field:
    executor-supplied under one documented rule
```

The executor rejects a stated field it cannot materialize.

For constructor fixtures, normally state:

- internal key;
- tree structure;
- leaf versions;
- exact scripts;
- predecessor program;
- successor program;
- metadata bytes;
- successor output role.

Funding outpoints may remain executor-supplied.

## 13.4 Fixture validation · `rule:guide10:fixture-validation`

Fixture validation requires:

- supported schema;
- exact reviewed target;
- exact reviewed binding;
- context input/output consistency;
- executing input exists;
- script path matches fixture program;
- tree contains executing leaf exactly once;
- stated control path matches tree where supplied;
- successor output role exists exactly once;
- expected claim belongs to fixture kind;
- expected resource rows are coherent;
- evidence claims are permitted for that fixture kind.

---

# 14. Native protocol schema 2 · `sec:guide10:protocol`

## 14.1 Exchange · `rule:guide10:protocol-exchange`

Schema 2 is lock-step:

```text
handshake request
    → executor handshake
    → environment observation

one execution request per case
    → one execution response per case

end of stdin
    → clean executor shutdown
    → EOF
```

No unsolicited message is accepted.

## 14.2 Handshake · `rule:guide10:protocol-handshake`

The handshake includes public provenance and interface capability:

```rust
pub struct ExecutorHandshake {
    pub protocol_schema: u32,

    pub adapter_name: String,
    pub adapter_version: String,
    pub framework_revision: Option<String>,

    pub node_name: String,
    pub node_version: String,
    pub binary_reported_revision: Option<String>,

    pub intended_executed_tip: Option<String>,
    pub upstream_base: Option<String>,
    pub included_local_topics: BTreeSet<String>,

    pub supported_domains: BTreeSet<WireExecutionDomain>,
    pub supported_leaf_versions: BTreeSet<u8>,
    pub capabilities: BTreeSet<ExecutorCapability>,
}
```

The handshake remains provenance, not authenticity.

## 14.3 Environment observation · `rule:guide10:protocol-environment`

```rust
pub struct ExecutorEnvironmentObservation {
    pub environment: WireEnvironment,
    pub chain_name: String,
    pub network_id: [u8; 32],
    pub genesis_id: [u8; 32],
    pub active_domains: BTreeSet<WireExecutionDomain>,
    pub active_leaf_versions: BTreeSet<u8>,
}
```

A dishonest executor can lie. The harness records that limitation.

The value nevertheless prevents an honest adapter from labeling one chain as another.

## 14.4 Request · `rule:guide10:protocol-request`

```rust
pub struct NativeExecutionRequest {
    pub schema: u32,
    pub case: NativeCaseId,
    pub fixture: NativeFixture,
}
```

The request carries no expectation field visible to a native executor if the protocol can avoid it.

Preferred design:

```text
harness retains expected outcome
executor receives only execution subject
```

This removes the possibility of a nonmock executor accidentally consulting the expected verdict.

If compatibility requires the full fixture DTO, the adapter must continue to validate and discard the expectation before execution, and the report must record that boundary.

## 14.5 Response · `rule:guide10:protocol-response`

```rust
pub struct NativeExecutionResponse {
    pub schema: u32,
    pub case: NativeCaseId,
    pub verdict: NativeVerdict,
    pub final_stack: Option<Vec<Vec<u8>>>,
    pub final_altstack: Option<Vec<Vec<u8>>>,
    pub observed_failure: Option<ObservedFailureClass>,
    pub resources: NativeResourceObservation,
}
```

Response consistency is validated before comparison.

## 14.6 Strict framing · `rule:guide10:protocol-framing`

Each protocol record is:

```text
one nonempty JSON object
one newline
```

Rejected:

- blank line;
- whitespace-only line;
- unknown field;
- unsupported schema;
- oversized line;
- missing newline beyond the configured limit;
- duplicate response;
- unexpected response;
- reordered response;
- missing response;
- trailing response;
- trailing blank line.

## 14.7 Timeout · `rule:guide10:protocol-timeout`

Timeout is infrastructure failure.

It is never:

```text
target rejected
fixture failed
claim unresolved
```

A timeout publishes no success stamp.

---

# 15. Prototype report schema · `sec:guide10:reports`

## 15.1 Report roles · `rule:guide10:report-roles`

Keep separate:

```rust
pub enum PrototypeReportRole {
    PrimitiveConformance,
    ConstructorContinuity,
    WideFloor,
}
```

A report may contain several roles only if each has its own exact case and claim census.

## 15.2 Case result · `rule:guide10:case-result`

```rust
pub struct PrototypeCaseResult {
    pub fixture: PrototypeFixtureProjection,
    pub claims: BTreeSet<NativeEvidenceClaim>,
    pub observed: ObservedNativeOutcome,
    pub status: CaseStatus,
}
```

The status is recomputed by report validation.

## 15.3 Claim result · `rule:guide10:claim-result`

```rust
pub struct NativeEvidenceClaimResult {
    pub claim: NativeEvidenceClaim,
    pub required: bool,
    pub bearing_cases: BTreeSet<PrototypeCaseId>,
    pub disposition: EvidenceDisposition,
}
```

A required claim passes only when:

- at least one required bearing case exists;
- every required bearing case passed, under the claim policy;
- no required bearing case hit infrastructure trouble;
- the claim’s subject projection matches.

## 15.4 Completeness · `rule:guide10:report-completeness`

```rust
pub enum PrototypeReportCompleteness {
    CompleteForPrimitivePlan,
    CompleteForConstructorPrototype,
    CompleteForWideFloorPrototype,
    PartialUnresolvedClaims,
    Failed,
}
```

The summary is derived.

A caller does not set it.

## 15.5 No digest · `rule:guide10:report-no-digest`

No report digest is added.

The build report is an explicit asset compared by typed content and exact bytes.

A future release consumer must admit a typed report identity separately.

---

# 16. Native comparison rules · `sec:guide10:comparison`

## 16.1 Verdict · `rule:guide10:compare-verdict`

Always compare the target verdict.

Infrastructure failure never matches target rejection.

## 16.2 Failure class · `rule:guide10:compare-failure`

Where the executor reports a class, it must belong to the fixture’s admitted class set.

Where the executor claims class-reporting capability and omits a required class, the response is malformed or the case fails under the stated protocol policy.

A coarse target class may match several reviewed causes only when that coarsening is explicitly represented.

## 16.3 Final stack · `rule:guide10:compare-stack`

For prototype acceptance, exact final stack comparison is required when the executor reports one.

If the real node cannot report interpreter stacks, the prototype must obtain stack-shape evidence through at least one of:

- a native script whose final consensus verdict distinguishes the shape;
- an instrumented interpreter whose provenance and non-independence are recorded;
- target programs reducing the expected intermediate state to one final truth item;
- static abstract-stack evidence retained as a separate claim.

Static stack evidence and node verdict evidence remain distinct.

## 16.4 Resources · `rule:guide10:compare-resources`

Compare exact figures where the fixture fixes them:

- script bytes;
- initial stack items;
- transaction weight where materialization is exact.

Record-only figures remain absent if the executor cannot observe them:

- peak main stack;
- peak alternate stack;
- maximum element;
- validation budget.

Do not encode “unobserved” as zero.

---

# 17. Independent oracles · `sec:guide10:oracles`

## 17.1 Opcode-byte oracle · `rule:guide10:opcode-oracle`

Maintain a test-only explicit table:

```text
OpcodeId → expected target byte
```

It must include every newly admitted primitive.

Expected bytes are not read from the production registry.

## 17.2 Stack-contract oracle · `rule:guide10:stack-oracle`

Maintain an independently authored stack contract for each newly admitted primitive:

- operands;
- success alternatives;
- retained/consumed behavior;
- non-aborting failure states;
- abort causes;
- resource cost.

The oracle does not call the production target builder.

## 17.3 Constructor oracle · `rule:guide10:constructor-oracle`

The constructor oracle computes:

- metadata bytes;
- metadata leaf script;
- leaf hash;
- child order;
- branch hash;
- tweak;
- output key;
- output program;
- control path.

Compare:

```text
first-party independent host reference
upstream/library target construction
target-native spend verdict
```

where possible.

## 17.4 Wide-arithmetic oracle · `rule:guide10:wide-oracle`

The exact arithmetic oracle verifies:

\[
a·b=q·d+r
\]

and:

\[
0\le r<d.
\]

The independent limb oracle reconstructs the product from limbs and compares it with direct exact multiplication.

## 17.5 Abstract execution oracle · `rule:guide10:execution-oracle`

For bounded short programs and selected compound fragments:

1. enumerate every compatible primitive success/failure transition;
2. preserve success, non-aborting failure, and abort separately;
3. compare the complete state sets with production validation.

No first-seen state is selected.

---

# 18. Security and execution trust · `sec:guide10:security`

## 18.1 Public-data interface · `rule:guide10:public-data`

Guide-10 inputs are public test data:

- metadata bytes;
- static tree roots;
- public internal keys;
- target programs;
- quotient/remainder witnesses;
- public arithmetic values;
- disposable development transactions;
- explicit executor capability.

No interface accepts:

- production private key;
- wallet seed;
- signing nonce;
- production blinding factor;
- private opening;
- RPC credential;
- bearer token;
- cookie path;
- production endpoint.

## 18.2 Test cryptographic values · `rule:guide10:test-material`

Published vectors, public NUMS points, and disposable test-network values are public fixture material.

They are labeled test-only and never reused for production authority.

## 18.3 Executor authority · `rule:guide10:executor-authority`

Selecting the executor grants execution authority.

The harness does not authenticate or sandbox it.

Untrusted source runs in an externally established:

- secretless;
- disposable;
- appropriately isolated;
- non-release-authoritative environment.

## 18.4 Diagnostics · `rule:guide10:diagnostics`

Do not emit:

- raw child argv;
- raw child stderr;
- executor path;
- environment values;
- temporary directory;
- cookie path.

Typed safe provenance and case IDs are permitted.

## 18.5 Crash artifacts · `rule:guide10:crash-artifacts`

Repository-controlled CI must not publish:

- core dumps;
- heap captures;
- debugger memory;
- disposable node directories;
- RPC cookies;
- child crash bundles.

---

# 19. Determinism · `sec:guide10:determinism`

## 19.1 Program determinism · `rule:guide10:program-determinism`

Given equal typed prototype inputs:

- instruction sequence equal;
- script bytes equal;
- stack contract equal;
- resource projection equal.

No declaration order, map iteration, path, host, thread count, or environment value affects the result.

## 19.2 Fixture determinism · `rule:guide10:fixture-determinism`

Fixture order is canonical by typed case ID.

Duplicate IDs fail.

Permutation of declaration order does not change:

- fixture projections;
- scripts;
- expected outcomes;
- claims;
- report order.

## 19.3 Report determinism · `rule:guide10:report-determinism`

Given equal:

- reviewed target;
- reviewed binding;
- fixture set;
- executor observations;
- claim plan;
- work limits;

report bytes are equal.

Exclude:

- wall clock;
- elapsed duration;
- host;
- username;
- process ID;
- temporary path;
- raw executor path;
- environment values.

## 19.4 Search determinism · `rule:guide10:search-determinism`

If constructor totality or arithmetic layout uses finite search:

- candidates are ordered by typed key;
- limits are explicit;
- exhaustion is typed failure;
- no partial result is returned;
- equal feasible candidates use canonical tie-breaking;
- diagnostic counters saturate.

---

# 20. Resource evidence · `sec:guide10:resources`

## 20.1 Constructor measurements · `tbl:guide10:constructor-resources`

Record:

| Dimension | Value |
|---|---:|
| metadata bytes | |
| operation leaf bytes | |
| metadata leaf bytes | |
| static subtree representation bytes | |
| predecessor verification bytes | |
| successor verification bytes | |
| control-path bytes | |
| witness bytes | |
| transaction weight | |
| peak abstract main stack | |
| peak abstract alternate stack | |
| maximum element bytes | |
| hash operations | |
| tweak/curve operations | |
| validation budget | |

Measure:

- minimum metadata;
- representative metadata;
- maximum prototype metadata;
- both branch-order cases;
- both output-key parity cases;
- wrong-root rejection;
- metadata-leaf escape rejection.

## 20.2 Wide-floor measurements · `tbl:guide10:wide-resources`

For each candidate record:

| Dimension | Candidate A | Candidate B | Candidate C |
|---|---:|---:|---:|
| script bytes | | | |
| witness items | | | |
| witness bytes | | | |
| arithmetic operations | | | |
| divisions | | | |
| comparisons | | | |
| verification operations | | | |
| peak main stack | | | |
| peak alternate stack | | | |
| maximum element | | | |
| transaction weight | | | |
| consensus verdict | | | |
| relay/policy verdict | | | |

Measure:

- exact division;
- maximal carry propagation;
- maximum operands;
- quotient zero;
- largest accepted quotient;
- rejected under-quotient;
- rejected over-quotient;
- malformed witness.

## 20.3 No calibration · `rule:guide10:no-calibration`

These measurements answer:

```text
Is the prototype plausible?
Which candidate is smaller?
Which target limit is first?
Which proof is clearest?
```

They do not select:

- architecture batch bounds;
- production taptree depth;
- final operation limits;
- release calibration.

No architecture default changes from a standalone prototype measurement.

---

# 21. Error vocabulary · `sec:guide10:errors`

## 21.1 Target errors

Add only errors reached by real validation branches.

Likely additions include:

```rust
pub enum TargetError {
    RelationOperandContractMismatch,
    SignatureOperandContractMismatch,
    UnknownKeyTypeContractMismatch,

    MissingCompoundPrimitive(OpcodeId),
    CompoundPrimitiveStackMismatch(OpcodeId),
    CompoundPrimitiveResourceMismatch(OpcodeId),

    ReviewedDeploymentBindingMismatch,
    ProductionProfileSchemaIncomplete,
}
```

Exact variants follow implementation.

## 21.2 Tapscript errors

Likely additions:

```rust
pub enum TapscriptError {
    PrototypeTargetMismatch,
    UnsupportedPrototypePrimitive(OpcodeId),

    ConstructorMetadataEncoding,
    ConstructorStaticRootMismatch,
    ConstructorInternalKeyMismatch,
    ConstructorBranchOrderUnavailable,
    ConstructorTweakFailure,
    ConstructorOutputProgramMismatch,
    ConstructorMetadataPathSpendable,
    ConstructorTotalityUnresolved,

    WideFloorZeroDivisor,
    WideFloorAmountOutOfDomain,
    WideFloorLimbOutOfDomain,
    WideFloorCarryOutOfDomain,
    WideFloorIntermediateOverflow,
    WideFloorUncheckedSuccessFlag,
    WideFloorEqualityFailure,
    WideFloorRemainderOutOfRange,

    PrototypeStateLimitExceeded,
    PrototypeResourceLimitExceeded,
}
```

No operation-emission, linker, ABI, signing, or release errors enter.

## 21.3 Conformance errors

Likely additions:

```rust
pub enum NativeConformanceError {
    UnsupportedProtocolSchema { offered: u32 },
    UnsupportedReportSchema { offered: u32 },

    MissingEnvironmentObservation,
    EnvironmentBindingMismatch,
    GenesisObservationMismatch,
    ActivationObservationMismatch,

    ProtocolRecordTooLarge {
        phase: ProtocolPhase,
        maximum: usize,
    },
    BlankProtocolRecord {
        phase: ProtocolPhase,
    },

    ExecutorProcessGroupTerminationFailed,

    DuplicateEvidenceClaim(NativeEvidenceClaim),
    MissingEvidenceClaim(NativeEvidenceClaim),
    UnexpectedEvidenceClaim(NativeEvidenceClaim),

    ReportCaseCensusMismatch,
    ReportEvidenceCensusMismatch,
    ReportClaimCensusMismatch,
    ReportSummaryMismatch,

    FixtureProjectionMismatch(PrototypeCaseId),
    UnvalidatedNativeReport,

    ConstructorClaimFailed(NativeEvidenceClaim),
    WideFloorClaimFailed(NativeEvidenceClaim),
}
```

Errors contain no raw child bytes or credential-bearing data.

---

# 22. Test strategy · `sec:guide10:tests`

## 22.1 Architecture and realization preflight tests

Required:

```text
invalid architecture still cannot receive semantic identity
schema-2 deployment profile cannot acquire production-release state
relation kind/body transposition rejects
expression-predicate relation has honest identity
surplus architecture-family relation rejects
```

## 22.2 Target contract tests

Required:

```text
V1 remains historical and unchanged
V2 carries the expanded reviewed primitive census
new opcode bytes match independent table
new success alternatives are complete
new failure effects are complete
signature empty/invalid/unknown-key paths are representable
cross-contract weld mutations reject
capability status closure holds
evidence requirements resolve
```

## 22.3 Tapscript tests

Required:

```text
prototype types cannot convert into production artifacts
constructor metadata codec
constructor host oracle
constructor program determinism
predecessor binding
successor binding
same-root continuity
wrong root/key/parity/order/schema/path rejection
metadata leaf unspendability

wide-floor exact oracle
limb decomposition
limb normalization
remainder addition
intermediate bounds
q-1/q/q+1
zero divisor
wrong remainder
wrong limb/carry
unchecked flag rejection
abstract-execution oracle
resource projection
```

## 22.4 Conformance tests

Required:

```text
protocol schema 2
report schema 2
blank line rejection
oversized record rejection
unterminated oversized record rejection
duplicate/missing/unexpected/reordered response
accepted response with failure class rejected
environment mismatch
genesis mismatch
exact target/binding mismatch
process-group timeout cleanup
descendant holding stdout cannot hang harness
mock cannot satisfy gate

empty evidence report rejects
missing required row rejects
duplicate row rejects
plan-class mutation rejects
summary mutation rejects
case mutation rejects
fixture script/context/layer mutation rejects
claim omission rejects
claim duplication rejects
```

## 22.5 Native constructor matrix

Required:

```text
valid predecessor
valid successor
valid continuity
both branch orders
both key parities
every metadata-field mutation
wrong static root
split predecessor/successor roots
wrong internal key
wrong parity
wrong target program
metadata-leaf spend
alternate schema
trailing metadata
wrong output role
```

## 22.6 Native wide-floor matrix

Required:

```text
zero and one
exact division
nonzero remainder
d-1 remainder
divisor one
maximum divisor
q zero
domain maximum
base boundaries
carry boundaries
q-1
q+1
r=d
d=0
negative
malformed width
byte reversal
wrong limb
wrong carry
wrong witness order
unchecked success flag
```

---

# 23. Suggested implementation waves · `sec:guide10:waves`

## Wave 0 — Accept the review register

Deliver:

- backlog rows for every accepted second-review finding;
- ownership and dependency order;
- focused reproduction where needed;
- exact Guide-10 entry checklist.

Suggested commit:

```text
plans: accept the Guide-10 preflight register
```

## Wave 1 — Make native evidence exact

Deliver:

- protocol schema 2;
- report schema 2;
- validated report wrapper;
- exact case/evidence/claim censuses;
- complete fixture projections;
- exact report summary recomputation;
- strict NDJSON framing;
- bounded records;
- response-shape validation.

Suggested commit:

```text
target-conformance: make native evidence self-validating
```

## Wave 2 — Bind the actual target and environment

Deliver:

- reviewed development binding;
- exact target equality;
- observed environment message;
- real genesis/network comparison;
- ADR-018 provenance;
- adapter/framework/node separation.

Suggested commit:

```text
target-conformance: bind native evidence to what executed
```

## Wave 3 — Supervise the executor process tree

Deliver:

- process-group startup;
- graceful group termination;
- hard group kill;
- descendant-pipe regression;
- temporary-node cleanup verification.

Suggested commit:

```text
target-conformance: supervise complete executor runs
```

## Wave 4 — Repair signature and relation abstractions

Deliver:

- signature operand alternatives;
- conditional failure cases;
- unknown-key behavior;
- native unknown-key vectors;
- relation-ID/body weld;
- profile trust-state naming correction;
- public status documentation repair.

Suggested commit:

```text
target foundations: close the typed trust-state gaps
```

## Wave 5 — Review compound-proof primitives

Deliver:

- primitive-needs census;
- complete upstream review;
- V2 target contract;
- independent opcode-byte table;
- stack and resource contracts;
- native primitive vectors;
- weld tests.

Suggested commit:

```text
target-elements: review the compound-proof substrate
```

Stop if the required substrate is unavailable.

## Wave 6 — Add generic constructor fixtures

Deliver:

- typed taptree fixture;
- exact tree materialization;
- exact control-path handling;
- complete fixture projection;
- constructor claim vocabulary.

Suggested commit:

```text
target-conformance: materialize generic constructor trees
```

## Wave 7 — Build the constructor oracle

Deliver:

- prototype metadata codec;
- independent leaf/branch/tweak construction;
- library/published-vector cross-check;
- branch-order and parity vectors;
- totality-policy candidates.

Suggested commit:

```text
target-conformance: add the constructor reference oracle
```

## Wave 8 — Build the constructor target prototype

Deliver:

- predecessor proof;
- successor proof;
- same-root continuity;
- metadata transition;
- unspendable metadata leaf;
- full mutation matrix;
- resource measurements.

Suggested commit:

```text
tapscript: prototype metadata-dependent constructor continuity
```

## Wave 9 — Build the wide-floor oracle and bound proof

Deliver:

- exact \(q,r\) oracle;
- independent limb normalizer;
- symbolic intermediate bounds;
- fixed vectors;
- property vectors.

Suggested commit:

```text
tapscript: establish the exact wide-floor reference
```

## Wave 10 — Build and compare wide-floor target candidates

Deliver:

- derived-limb candidate;
- witnessed-limb candidate only if needed;
- sandwich candidate only if plausibly simpler;
- exact flag enforcement;
- native matrices;
- resource comparison;
- accepted candidate or target rejection.

Suggested commit:

```text
tapscript: prototype exact wide floor verification
```

## Wave 11 — Record decisions and gate

Deliver:

- STATE-constructor research result;
- wide-arithmetic research result;
- Phase-3 update;
- backlog gate;
- package documentation;
- target reference;
- identity/dependency report;
- full verification record;
- clean final tree.

Suggested commit:

```text
plans: record the Guide-10 prototype decisions
```

Commit each coherent green wave promptly.

---

# 24. Working cadence · `sec:guide10:cadence`

After each coherent Rust wave:

```sh
cargo fmt --all
git status --short
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Use focused package tests during development.

Do not run the complete real native matrices after every small edit. Run them when:

- the relevant primitive contract is coherent;
- the compound prototype compiles;
- the abstract validator passes;
- the fixed native matrix is ready.

Read `git status` after formatting.

Every new tracked source joins the nearest Meson census in the same commit.

---

# 25. Focused verification commands · `sec:guide10:focused-verification`

## 25.1 Architecture and realization

```sh
cargo test --locked -p tripod-architecture
cargo test --locked -p tripod-realization
```

Focused filters must match nonzero tests.

## 25.2 Target contract

```sh
cargo test --locked -p tripod-target-elements
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-target-elements --no-deps
```

## 25.3 Tapscript prototypes

```sh
cargo test --locked -p tripod-tapscript
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-tapscript --no-deps
```

Suggested focused filters:

```sh
cargo test --locked -p tripod-tapscript constructor
cargo test --locked -p tripod-tapscript wide_floor
cargo test --locked -p tripod-tapscript stack
cargo test --locked -p tripod-tapscript oracle
```

Verify each filter runs tests.

## 25.4 Native conformance

```sh
cargo test --locked -p tripod-target-elements-conformance
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-target-elements-conformance --no-deps
```

Suggested focused filters:

```sh
cargo test --locked -p tripod-target-elements-conformance report
cargo test --locked -p tripod-target-elements-conformance protocol
cargo test --locked -p tripod-target-elements-conformance supervisor
cargo test --locked -p tripod-target-elements-conformance constructor
cargo test --locked -p tripod-target-elements-conformance wide_floor
```

## 25.5 Adapter smoke test

With explicit node configuration:

```sh
scripts/test-elements-native-executor.sh .
```

The test skips loudly when no executor environment exists. A skip is not native evidence.

## 25.6 Documentation and census

```sh
scripts/check-plans.sh
meson compile -C build lint
git diff --check
git diff --cached --check
```

---

# 26. Dependency review · `sec:guide10:dependencies`

## 26.1 Expected dependency posture

```text
target-elements:
    no dependencies

tapscript:
    compiler
    target-elements
    no new third-party dependency expected

target-elements-conformance:
    target-elements
    tapscript
    cli-common
    existing generic workspace dependencies
    possible reviewed reference crypto dependency only if required
```

## 26.2 Reference crypto dependency

If the independent constructor oracle needs a cryptographic library, review:

- exact version;
- upstream source;
- licence;
- Rust 1.88 support;
- enabled features;
- default-feature status;
- transitive graph;
- unsafe and FFI boundary;
- deterministic behavior;
- platform requirements;
- advisory status;
- replacement path;
- whether the dependency can sign or hold secret keys;
- whether only public point/hash operations are used.

The dependency belongs to conformance/prototype code, not the dependency-free target contract.

## 26.3 Commands

```sh
cargo tree --locked -p tripod-target-elements -e features
cargo tree --locked -p tripod-tapscript -e features
cargo tree --locked -p tripod-target-elements-conformance -e features
cargo metadata --locked
cargo audit
```

If `cargo-audit` is unavailable, record:

```text
cargo-audit:
    SKIPPED
```

not passed.

---

# 27. Generated artifacts and identity impact · `sec:guide10:identity-impact`

Expected impact:

```text
Attestation version:
    unchanged

realization version:
    unchanged

architecture schema:
    unchanged

architecture semantic hash:
    unchanged

architecture behavioural hash:
    unchanged

architecture generated publications:
    unchanged

declassification publication:
    unchanged

realization identity:
    none minted

compiler identity:
    none minted

target contract:
    V2 if the reviewed primitive or relation census changes

target-definition digest:
    none minted

prototype-program digest:
    none minted

prototype-report digest:
    none minted

deployment-profile identity:
    remains dormant

release identity:
    none minted
```

If architecture or realization identities move unexpectedly, stop and investigate before regenerating publications.

No generated publication is accepted merely because a generator rewrote it.

---

# 28. Full batch gate · `gate:guide10:batch`

After all coherent waves:

```sh
scripts/ci.sh
meson compile -C build
meson test -C build --print-errorlogs
```

Run the real target-native primitive, constructor, and wide-floor matrices separately with the reviewed executor.

Also run:

```sh
scripts/check-plans.sh
meson compile -C build lint
git diff --check
git diff --cached --check
```

Run:

```sh
cargo audit
```

when installed.

Run document byte reproducibility when required by changed document inputs or the repository’s batch policy:

```sh
scripts/check-document-reproducibility.sh
```

If deferred, record it as deferred.

Finally:

```sh
git status --porcelain=v1 --untracked-files=all
```

The final tree must be clean.

---

# 29. Guide-10 exit criteria · `gate:guide10:exit`

Guide 10 is complete only when every applicable item holds.

## Evidence foundation

- [ ] native protocol schema 2 exists;
- [ ] native report schema 2 exists;
- [ ] native report has a validated wrapper;
- [ ] evidence-row census is exact and duplicate-sensitive;
- [ ] fixture-case census is exact and duplicate-sensitive;
- [ ] each report row binds the complete fixture projection;
- [ ] broad evidence requirements have typed subclaim censuses;
- [ ] deleting a failed row cannot produce a pass;
- [ ] relabeling a failed row unresolved cannot produce a pass;
- [ ] summary counts and completeness are recomputed;
- [ ] accepted responses with contradictory fields fail protocol validation;
- [ ] observed chain identity equals the reviewed binding;
- [ ] exact target definition is bound to the development binding;
- [ ] executor provenance records intended tip, upstream base, and local topics where claimed;
- [ ] timeout terminates the complete executor process tree;
- [ ] protocol record sizes are bounded;
- [ ] blank protocol records fail;
- [ ] trailing protocol records fail;
- [ ] mock executor cannot satisfy native evidence.

## Typed target foundation

- [ ] signature empty, invalid, and unknown-key paths are representable;
- [ ] unknown-key target behavior has native focused coverage;
- [ ] relation IDs are welded to relation bodies;
- [ ] schema-2 profile validity is not named production-release validity;
- [ ] every required compound primitive has source review;
- [ ] every admitted primitive has exact byte, stack, failure, encoding, resource, and evidence contracts;
- [ ] target contract revisioning is explicit;
- [ ] no raw unreviewed opcode exists;
- [ ] abstract and native primitive results agree.

## Constructor prototype

- [ ] canonical prototype metadata exists;
- [ ] predecessor metadata binds to predecessor program;
- [ ] successor metadata is derived exactly;
- [ ] one authenticated static root binds both sides;
- [ ] internal-key policy is explicit;
- [ ] TapLeaf framing is exact;
- [ ] TapBranch ordering is target-enforced;
- [ ] TapTweak construction is exact;
- [ ] x-only/compressed parity bridge is exact;
- [ ] metadata leaf is unspendable;
- [ ] key path carries no known authority;
- [ ] tweak totality policy is explicit;
- [ ] wrong metadata, root, key, schema, order, parity, and path mutations reject;
- [ ] complete constructor fixtures run through the real target;
- [ ] constructor resource measurements are recorded;
- [ ] no production STATE ABI is frozen;
- [ ] result is recorded as accepted prototype or target rejection.

## Wide-floor prototype

- [ ] exact floor equivalence is documented;
- [ ] exact host oracle exists;
- [ ] independent limb oracle exists;
- [ ] intermediate range proof is complete;
- [ ] every target arithmetic flag is checked;
- [ ] operand and witness domains are exact;
- [ ] limb and carry constraints are complete;
- [ ] zero divisor rejects;
- [ ] `q-1` rejects;
- [ ] `q+1` rejects;
- [ ] `r=d` rejects;
- [ ] malformed widths, signs, limbs, carries, and witness order reject;
- [ ] abstract and native stack outcomes agree;
- [ ] fixed and deterministic generated vectors pass;
- [ ] target resource measurements are recorded;
- [ ] no operation-level feasibility is overclaimed;
- [ ] result is recorded as accepted prototype or target rejection.

## Boundary and documentation

- [ ] prototype types cannot enter production emission or release output;
- [ ] no linked bundle or ABI exists;
- [ ] no speculative identity was minted;
- [ ] no target-specific type flowed back into realization or compiler core;
- [ ] STATE-constructor research records its result;
- [ ] wide-arithmetic research records its result;
- [ ] public declassification remains visibly open;
- [ ] package READMEs are current;
- [ ] package index statuses agree;
- [ ] Phase-3 card is current;
- [ ] target reference contains exact review and native-executor provenance;
- [ ] backlog records the exact Guide-10 gate;
- [ ] every new source is in the Meson census;
- [ ] focused package tests pass;
- [ ] Rustdoc passes with warnings denied;
- [ ] workspace formatting passes;
- [ ] workspace Clippy passes;
- [ ] debug and release tests pass through the complete gate;
- [ ] mocked Meson contract passes;
- [ ] canonical Meson compile and tests pass;
- [ ] real primitive, constructor, and wide-floor matrices pass;
- [ ] skipped or deferred lanes are reported honestly;
- [ ] final repository tree is clean.

---

# 30. Completion report template · `sec:guide10:completion-report`

```text
Guide 10 result
===============

Starting state:
    source revision:
    Guide-9 gate:
    second-review register:
    clean tree:

Evidence preflight:
    protocol schema:
    report schema:
    validated report wrapper:
    fixture projection binding:
    exact case census:
    exact evidence-row census:
    claim-level coverage:
    summary recomputation:
    response-shape validation:
    target/binding exactness:
    observed network/genesis:
    executor provenance:
    process-group cleanup:
    protocol size limit:
    protocol framing:
    mock gate:

Typed-foundation repairs:
    signature operand alternatives:
    empty-signature behavior:
    invalid-signature behavior:
    unknown-key behavior:
    relation-ID/body weld:
    deployment-profile trust-state correction:
    public documentation reconciliation:

Primitive closure:
    target-contract revision:
    primitives required:
    primitives already reviewed:
    primitives newly reviewed:
    primitives unavailable:
    opcode-byte oracle:
    stack-contract oracle:
    native primitive cases:

Constructor prototype:
    selected candidate:
    metadata schema:
    static code-root representation:
    internal key:
    leaf hash:
    branch hash:
    branch ordering:
    tweak relation:
    predecessor authentication:
    successor authentication:
    static-root continuity:
    metadata-leaf unspendability:
    key-path residual:
    parity handling:
    totality policy:
    positive cases:
    negative cases:
    abstract/native agreement:
    resources:
    decision:
        accepted / target rejected / unresolved

Wide-floor prototype:
    selected candidate:
    base:
    witness:
    exact host oracle:
    independent limb oracle:
    intermediate bounds:
    arithmetic flag enforcement:
    q-1:
    q:
    q+1:
    zero divisor:
    malformed limb/carry:
    property cases:
    abstract/native agreement:
    resources:
    decision:
        accepted / target rejected / unresolved

Native executor:
    adapter:
    adapter version:
    framework:
    node:
    binary-reported revision:
    intended executed tip:
    upstream base:
    included local topics:
    chain:
    observed network:
    observed genesis:
    activation:
    exact command:

Reports:
    primitive cases:
    primitive passed:
    primitive failed:
    primitive infrastructure errors:
    primitive unresolved claims:

    constructor cases:
    constructor passed:
    constructor failed:
    constructor infrastructure errors:
    constructor unresolved claims:

    wide-floor cases:
    wide-floor passed:
    wide-floor failed:
    wide-floor infrastructure errors:
    wide-floor unresolved claims:

    report bytes deterministic:

Identity impact:
    Attestation:
    realization:
    architecture schema:
    architecture semantic hash:
    architecture behavioural hash:
    target contract version:
    prototype identities:
        none
    report identities:
        none
    deployment identity:
        none

Dependency impact:
    target-elements:
    tapscript:
    target-elements-conformance:
    Cargo.lock:
    licence:
    MSRV:
    unsafe/FFI:
    advisories:

Verification:
    cargo fmt --all:
    cargo clippy --workspace --all-targets --locked -- -D warnings:
    cargo test --workspace --locked:
    cargo test --workspace --release --locked:
    architecture tests:
    realization tests:
    target-elements tests:
    tapscript tests:
    target-elements-conformance tests:
    Rustdoc:
    cargo tree:
    cargo metadata:
    scripts/check-plans.sh:
    meson compile -C build lint:
    scripts/ci.sh:
    meson compile -C build:
    meson test -C build --print-errorlogs:
    real primitive matrix:
    real constructor matrix:
    real wide-floor matrix:
    cargo audit:
    document reproducibility:
    git diff --check:
    final git status:

Planning handoff:
    STATE constructor research:
    wide arithmetic research:
    Phase 3:
    next guide:

Residuals:
```

---

# 31. What follows Guide 10 · `sec:guide10:next`

Guide 10 does not complete Phase 3.

If the constructor and arithmetic prototypes are accepted, the remaining foundational prototype is:

```text
Guide 11 — Public Declassification and Confidential-to-Public Lifecycle
```

That guide should decide:

- explicit-only boundary policy;
- owner-authorized normalization;
- direct authenticated opening;
- residual blinding closure;
- public committed ASH;
- fresh-process permissionless maintenance;
- sponsor-region conservation evidence;
- representation safety versus minimality.

Only after all three Phase-3 research owners have accepted results or explicit target rejections may production operation guides proceed.

The likely operation sequence remains:

```text
compact ASH
live transfer
STATE and maturity
burn and clear
redemption
requests and admission
settlement
cycle
release
```

Guide 10 supplies mechanisms to those phases. It does not implement their operations.

---

# 32. One-line guide · `rem:guide10:one-line`

> Guide 10 must prove, against an exact and self-validating native-evidence boundary, that Elements tapscript can preserve one metadata-dependent constructor across a state transition and can verify \(q=\lfloor a·b/d\rfloor\) exactly under the \(2^{51}\) amount domain—without emitting an attestation-contract operation, freezing an ABI, calibrating a bound, minting an identity, or weakening either relation when the target substrate is insufficient.
