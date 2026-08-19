# Guide 11 — Public Declassification and Confidential-to-Public Lifecycle

> **Status:** Execution guide; not yet executed
> **Phase:** Phase 3 — Elements target and foundational prototypes
> **Entry:** Guide-10 constructor and wide-floor prototype decisions recorded; all Guide-11 preflight findings below reproduced and closed
> **Primary research owner:** [`plans/research/public-declassification.md`](../research/public-declassification.md)
> **Affected packages:** `target-elements`, `tapscript`, `target-elements-conformance`
> **May affect after acceptance:** `realization`, `compiler`, `transaction`, `vectors`
> **Supersedes as execution direction:** `guide_eleven_concept.md`; the concept remains an archived design input unless repository policy chooses replacement rather than coexistence
> **Does not implement:** an attestation-contract operation, linked bundle, transaction ABI, calibrated bound, production signer, wallet, deployment, or release
> **Required result:** one accepted initial declassification policy, or an explicit typed target rejection/deferral
> **Review basis:** static reviews of tree `0.3.4-dev`; review findings are hypotheses until reproduced on the working tree
> **Trust boundary:** repository code and selected executors are executable; run untrusted contributions only in an externally established secretless environment under ADR-015

---

## Mission · `sec:guide11-exec:mission`

Guide 11 determines how a semantically private committed value may cross into a representation that is publicly authenticated and usable by an unrelated future constructor.

The decisive lifecycle is:

```text
PrivateCommitted value
    ↓
owner-authorized boundary or normalization
    ↓
PublicCommitted or Explicit value
    ↓
destruction of creator-local private state
    ↓
fresh unrelated process reconstructs a valid future spend
```

The guide must answer three independent questions:

1. **Disclosure** — is the semantic amount public?
2. **Authentication** — is that amount cryptographically bound to the exact target value commitment and asset generator?
3. **Availability** — can an unrelated future constructor obtain every public fact and witness needed to verify and use it?

These implications are forbidden:

```text
owner knows amount
    ⇏ amount is publicly available

metadata states amount
    ⇏ amount is authenticated

transaction balances
    ⇏ this output's stated amount opens this commitment

opening verified once
    ⇏ opening remains durably available

host library accepts opening
    ⇏ emitted target program enforces opening

one representation is safe
    ⇏ every representation is safe
```

Guide 11 must select one honest initial policy:

```text
direct authenticated public opening

owner-authorized normalization

explicit-only public boundary

or

target rejection / typed deferral
```

A mixed result is valid. For example:

```text
private lateral transfer:
    supported

normalization to public committed:
    supported

direct private boundary:
    deferred

explicit boundary:
    supported initial policy
```

Success means establishing the matrix honestly, not maximizing the number of supported cells.

---

## One-line thesis · `rem:guide11-exec:thesis`

> A confidential value becomes publicly usable only when its amount is exactly authenticated against the target commitment and explicit asset, its commitment algebra closes, its evidence is canonically bound to the intended object, and a fresh unrelated process can recover and verify it from public chain data alone.

---

# 1. Governing rulings · `sec:guide11-exec:rulings`

## 1.1 Safety, disclosure, constructibility, and evidence are separate

The guide keeps four axes distinct:

| Axis | Question |
|---|---|
| safety | Can an invalid amount, asset, commitment, recipient, authorization, or transition be accepted? |
| disclosure | Which facts become public, and under which typed reason? |
| constructibility | Can the authorized or permissionless constructor obtain all required inputs? |
| evidence | Did an independently stated target case actually exercise the claim under which its report files it? |

Passing one axis does not imply another.

In particular:

```text
target accepted transaction
≠
public amount authenticated

public amount authenticated
≠
future constructor can obtain opening

future constructor can obtain opening
≠
representation is disclosure-minimal

report validates against caller-supplied fixtures
≠
fixtures are canonical evidence subjects
```

## 1.2 The three value modes remain distinct

Guide 11 uses the realization-owned modes:

```text
PrivateCommitted
    amount not publicly available
    target commitment remains consensus-enforced
    opening remains owner-private

PublicCommitted
    amount and authenticated opening are public
    target commitment remains consensus-enforced
    future target relations can verify the opening

Explicit
    amount directly encoded
    no value-blinding term remains on that output
```

All three may denote the same semantic amount. They differ in disclosure, construction, witness requirements, commitment algebra, target cost, and future lifecycle.

`PublicCommitted` never means merely:

```text
metadata contains amount
```

It means:

```text
public amount
+
public proof/opening
+
exact binding to the actual target commitment
+
exact asset/generator binding
+
durable public availability
```

## 1.3 Closed protocol asset identity remains explicit

Value representation may vary. Closed protocol asset identity may not.

A future protocol consumer must classify explicitly:

```text
U
ENT
DIST_CTL
PID
PACE
ENT_AUTH
DIST_AUTH
```

Guide-11 fixtures remain target-generic and do not encode those protocol identities. They test the lower target relation:

```text
the exact explicit fixture asset or linked generator
is bound to the value commitment being opened
```

A confidential or unclassified asset commitment cannot satisfy a future closed-asset relation.

## 1.4 Whole-transaction conservation and local opening are distinct

Elements consensus may establish:

```text
the complete transaction balances
```

A target program may establish:

```text
public amount v and opening r bind exact commitment C
under exact asset generator H_A
```

Neither substitutes for the other.

The conceptual relation is:

\[C = rG + vH_A\]

The exact Elements convention may differ in sign, generator derivation, encoding, parity treatment, or argument order. Source review must establish the actual target relation before implementation.

Guide 11 requires separate typed claims for:

```text
whole-transaction confidential-value conservation

local commitment equality

authenticated public opening

residual blinding closure
```

## 1.5 Low-level primitives do not imply a complete proof

The reviewed target carries low-level hashing and curve operations. Their availability does not imply that commitment equality or authenticated opening exists as a complete pattern.

A public-opening candidate is accepted only after it establishes:

1. exact amount domain;
2. exact asset/generator binding;
3. exact commitment relation;
4. scalar and point validity;
5. parity handling;
6. canonical witness encoding;
7. every success flag;
8. durable public availability;
9. complete target-native positive and negative evidence;
10. complete-transaction resource feasibility.

## 1.6 Capability ownership must be explicit

Each accepted or rejected claim receives one disposition:

```text
PrimitiveReviewed

BackendPatternPrototypeAccepted

ExternalConsensusEvidenceRequired

Unsupported

DeferredWithNamedBlocker
```

Recommended ownership:

```text
target-elements
    primitive, encoding, consensus, and resource facts

tapscript
    complete backend proof-pattern prototypes

target-elements-conformance
    exact target-native execution and report mechanics

future transaction
    production confidential transaction construction and secret boundary
```

An accepted backend pattern does not silently turn into a primitive target capability. An experimental prototype does not automatically inhabit a production `BackendPatternId`.

## 1.7 No production secret enters Guide 11

Guide-11 openings, blinders, nonces, and test signing values are deterministic public test data.

They must:

- authorize nothing of value;
- be generated only for disposable development chains;
- be labeled test-only;
- never derive from production material;
- be reproducible from explicit fixture inputs;
- be destroyed with the disposable environment;
- never enter a production-secret interface.

No command accepts:

```text
production private key
wallet seed
production blinding factor
production opening
RPC password
cookie path
bearer token
production wallet
production endpoint
```

A future production transaction package requires its own secret-bearing design under ADR-015. Guide 11 does not establish it.

## 1.8 No speculative identity is introduced

Guide 11 mints no:

```text
OpeningPatternHash
CapsuleHash
NormalizationPatternHash
DeclassificationFixtureSetHash
NativeReportHash
DeclassificationReportHash
```

Typed equality and exact bytes are sufficient for this phase.

If a later release consumes a report across a process or distribution boundary, report identity is admitted then under ADR-016 with typed role and subject binding.

---

# 2. Entry conditions · `sec:guide11-exec:entry`

Guide 11 core work begins only when all of the following hold:

- the Guide-10 constructor decision is recorded;
- the Guide-10 wide-floor decision is recorded;
- the target contract is in its reviewed trust state;
- the development deployment binding is exact;
- the native executor reports its actual chain and genesis;
- executor protocol records are bounded;
- executor timeout supervision covers the directly spawned process tree;
- canonical fixture and prototype subjects cannot be replaced by caller-authored alternatives at an evidence gate;
- transcripts bind the exact target, deployment, and requests actually executed;
- evidence reports cannot pass while their summary is failed;
- executable provenance required by ADR-018 is validated;
- the starting tree is clean;
- the exact starting revision is recorded.

Before editing:

```sh
git status --porcelain=v1 --untracked-files=all
git log -1 --oneline
```

If the tree is not clean, stop and classify every change before continuing.

---

# 3. Guide-11 preflight review register · `tab:guide11-exec:preflight`

The two static reviews of tree `0.3.4-dev…` identified the following findings. Each finding must be reproduced against the working tree, then either fixed, disproved with a typed argument, or reclassified with a narrower assurance claim.

| ID | Priority | Finding | Required disposition |
|---|---:|---|---|
| `G11-R01` | P0 | A transcript can be rebound to another fixture census, prototype matrix, target, or deployment binding. | Bind transcript to exact subjects and requests. |
| `G11-R02` | P0 | Primitive claims can be manufactured by attaching claim-bearing case metadata to an unrelated script. | Gate only a canonical validated primitive plan. |
| `G11-R03` | P0 | Prototype claims are caller-authored and can certify a trivial true script as constructor or wide-floor evidence. | Gate only relation-specific canonical matrices. |
| `G11-R04` | P1 | Consensus resource cases are credited to policy-resource evidence because evidence ownership is derived from case ID without enforcement layer. | Derive evidence from complete fixture; correct plan class. |
| `G11-R05` | P1 | Primitive native gate can accept a report whose summary is `Failed`. | Gate every canonical case and reject failed completeness. |
| `G11-R06` | P1 | ADR-018 execution provenance is recorded but not enforced by evidence gates. | Validate executable provenance before gate eligibility. |
| `G11-R07` | P1 | Meson defaults an executor to `reviewed-non-mock`. | Require explicit caller selection; fail closed. |
| `G11-R08` | P1/P2 | Relation bodies can admit several semantic relation subjects. | Derive exactly one canonical subject from each body. |
| `G11-R09` | P1/P2 | Target V1 is advertised as supported while current validation applies the V2 census and algebra. | Remove V1 support or implement genuine version dispatch. |
| `G11-R10` | P1/P2 | Signature weld omits unknown-key behavior and much of the success algebra. | Weld the complete signature relation. |
| `G11-R11` | P2 | Bare prototype report digests persist despite the recorded no-report-identity decision. | Remove them or admit typed retained report references. |
| `G11-R12` | P2 | Current backlog state contradicts recorded Guide-8 through Guide-10 completion and duplicates finding IDs. | Reconcile current state and enforce unique IDs. |
| `G11-R13` | P2/P3 | Process-group establishment failure can leave the direct child unreaped. | Kill and reap on every pre-supervisor failure. |
| `G11-R14` | P3 | Constructor retry retries internal-key defects no metadata nonce can repair. | Share a typed retryability predicate. |
| `G11-H01` | Hardening | Opcode resource stack-growth rows are not generically welded to success and non-aborting failure effects. | Derive and compare exact maximum stack growth. |
| `G11-H02` | Future blocker | Issuance observations carry authority fields the realization evaluator does not yet enforce. | Do not add issuance realization scope until enforced or externally evidenced. |

No public-declassification prototype begins while `G11-R01` through `G11-R10` remain open.

---

# 4. Preflight evidence-boundary redesign · `sec:guide11-exec:evidence-boundary`

## 4.1 Separate arbitrary experiments from canonical evidence

The current fixture constructors are useful for tests and experiments. They must not, by themselves, create gate-eligible evidence subjects.

Introduce distinct trust states:

```rust
pub struct PrimitiveFixtureSet {
    // Arbitrary, caller-built, useful for experiments.
}

pub struct CanonicalPrimitiveFixtureSet {
    fixtures: PrimitiveFixtureSet,
    // No public unchecked constructor.
}

pub struct ExperimentalExecutionTranscript {
    // Ad hoc execution only.
}

pub struct CanonicalPrimitiveTranscript {
    // Bound to exact target, binding, and canonical requests.
}

pub struct ValidatedCanonicalNativeReport {
    // The only primitive report accepted by the native gate.
}
```

Only:

```rust
canonical_fixture_set(&target, &binding)
```

may construct `CanonicalPrimitiveFixtureSet`.

Ad hoc fixtures may still execute, but their result has an experimental role and no path to the native evidence gate.

## 4.2 Canonical prototype matrices are typed states

Replace arbitrary gate input of:

```rust
&[CompoundPrototypeFixture]
```

with relation-specific wrappers:

```rust
pub struct ConstructorPrototypeMatrix {
    rows: Vec<CompoundPrototypeFixture>,
}

pub struct WideFloorPrototypeMatrix {
    rows: Vec<CompoundPrototypeFixture>,
}
```

Their fields are private. Their only constructors are the canonical matrix builders:

```rust
constructor_case_matrix(&target)
    -> Result<ConstructorPrototypeMatrix, ConstructorMatrixDefect>

wide_floor_case_matrix(&target)
    -> Result<WideFloorPrototypeMatrix, WideFloorMatrixDefect>
```

A future Guide-11 matrix receives its own wrapper and claim census. It is not represented as an arbitrary vector carrying an arbitrary claim set.

Ad hoc compound fixtures may remain available for experiments, but they produce an experimental report that cannot reach `prototype_gate`.

## 4.3 Claims are canonical policy, not fixture assertions

A gate-eligible claim set must derive from canonical case ownership.

It must not be accepted merely because a public field says:

```rust
claims: BTreeSet<PrototypeClaim>
```

For canonical matrices, derive claims from one closed registry keyed by canonical case identity, or retain them in a private matrix row whose exact projection is compared against a freshly regenerated canonical matrix.

Required invariant:

```text
canonical case subject
↔
exact program and context
↔
exact expected outcome
↔
exact claim set
```

Changing any member invalidates canonical eligibility.

Checking only that a named opcode appears in a script is insufficient. A script can contain an irrelevant opcode or execute it in a context unrelated to the claimed property.

## 4.4 Transcripts bind exactly what was sent · `rule:guide11-exec:transcript-binding`

A canonical transcript retains:

```rust
pub struct CanonicalPrimitiveTranscript {
    target: target_elements::TargetProjection,
    deployment: target_elements::DeploymentProjection,
    requests: BTreeMap<NativeCaseId, PrimitiveExecutionSubjectProjection>,
    responses: BTreeMap<NativeCaseId, NativeExecutionResponse>,
    handshake: ExecutorHandshake,
    environment: ExecutorEnvironmentObservation,
    trust: ExecutorTrust,
}
```

The prototype counterpart retains its exact matrix projections.

The report evaluator must consume the transcript’s retained subjects. It must not accept a second, independently supplied fixture set or matrix as the claimed execution subject.

At minimum, report validation rejects:

- same case ID with different script;
- same case ID with different initial stack;
- different transaction context;
- different expected outcome;
- different enforcement layer;
- different claim set;
- different target projection;
- different deployment projection;
- different network or genesis;
- different prototype construction.

No digest is needed. Exact typed comparison is the correct mechanism.

## 4.5 Protocol revision 3 removes expectations from executor requests · `rule:guide11-exec:request-subject`

The executor should receive the execution subject, not the answer.

Define a protocol revision in which the request carries:

```text
case identity
script
initial stack
transaction context
construction, where applicable
target/deployment execution facts
```

and does not carry:

```text
expected verdict
expected failure class
expected final stack
expected resources
claim set
evidence plan class
```

The harness retains expectations privately and performs the comparison after receiving the executor’s answer.

Recommended new report value:

```text
RequestExpectationBoundary::ExecutorReceivesSubjectOnly
```

Revision-2 reports remain historical. They are not silently parsed as revision 3.

The test mock may still have explicit test behavior, but it must not obtain the expected answer by reading it from the request. A mock remains unable to satisfy an evidence gate regardless.

## 4.6 Environment binding is checked twice · `rule:guide11-exec:environment-twice`

The harness compares the executor’s observed environment with the reviewed deployment binding:

1. before sending any case;
2. again when constructing or validating a gate-eligible report.

The second check protects against transcript rebinding.

Require exact equality for:

```text
deployment environment
network identity
genesis identity
active execution domain
required leaf version
```

## 4.7 Resource evidence uses complete fixture semantics

Evidence ownership must derive from the full fixture, including enforcement layer.

For resource cases:

```text
Consensus fixture
    → ConsensusResourceLimits

RelayPolicy fixture
    → PolicyResourceLimits
```

Do not credit consensus cases to policy evidence.

Until an actual policy-resource matrix exists, classify:

```text
PolicyResourceLimits:
    UnresolvedByDesign
```

A nonminimal-push relay case is evidence about push standardness. It is not automatically evidence for the deployment’s transaction-weight policy limit.

Add an invariant:

```text
a Required evidence row owns at least one required typed claim
```

unless an explicit typed exception explains why claim-level decomposition does not apply.

## 4.8 Gates reject failed reports and failed cases

A canonical primitive gate checks:

```text
executor is eligible
provenance is valid
summary completeness is not Failed
every required claim passed
every required evidence row passed
every canonical case status is Passed
```

A canonical case whose expected result is rejection still has `CaseStatus::Passed` when the target rejected it as expected.

No gate returns success for a report whose own summary says `Failed`.

If an experimental report intentionally allows failed optional cases, it uses a separate non-evidence gate or no gate.

## 4.9 Executable provenance is typed and validated

Introduce expected provenance as explicit configuration:

```rust
pub struct ExpectedExecutorProvenance {
    pub intended_tip: RevisionId,
    pub upstream_base: RevisionId,
    pub included_local_topics: BTreeSet<TopicName>,
}
```

Validate before evidence eligibility:

- adapter name is nonblank;
- adapter version is nonblank;
- node name is nonblank;
- node version is nonblank;
- binary-reported revision is present;
- intended tip is present;
- upstream base is present;
- revision fields have the admitted syntax;
- binary revision matches the intended tip under one explicit rule;
- upstream base matches the expected base;
- local topic census matches exactly.

If the binary embeds a Git prefix rather than a full object ID, define the accepted rule explicitly, for example:

```text
reported prefix has an accepted minimum width
and is an exact prefix of the expected full intended tip
```

Do not treat arbitrary text equality or a checkout’s `HEAD` as binary provenance.

A dishonest executor can still lie. The report remains provenance, not authenticity. This repair closes omission and honest mismatch, which ADR-018 assigns to the evidence gate.

## 4.10 Meson makes no trust declaration by default

Change the executor-class option to:

```text
unselected
mock
reviewed-non-mock
```

with `unselected` as the default.

Rules:

```text
executor path empty
    → native targets are not defined

executor path nonempty + class unselected
    → Meson configuration fails

executor path nonempty + class mock
    → targets may run, evidence gates refuse

executor path nonempty + class reviewed-non-mock
    → evidence eligibility remains subject to report and provenance checks
```

Require explicit network, genesis, intended tip, upstream base, and topic census whenever a reviewed nonmock target is configured.

## 4.11 Startup cleanup owns every spawned child

Before a supervisor is fully established, guard the direct child with cleanup-on-error behavior.

If process-group establishment fails:

1. terminate the direct child;
2. wait for and reap it;
3. return `ExecutorProcessGroupUnavailable`.

No startup failure path may drop a live or exited unreaped `Child`.

## 4.12 Relation identity is one-to-one with its body · `rule:guide11-exec:relation-subject`

`validate_relation_identity` must derive exactly one expected kind and subject from each body.

Recommended subject additions include:

```rust
RelationSubject::TransactionSide {
    side: TransactionSide,
}
```

Canonical examples:

```text
AllowedObjectFamilies
    → TransactionSide { side }

RootPolicy
    → Operation

ProjectionPolicy
    → Operation

CanonicalDeltaPolicy
    → Operation

OpenFlowPolicy
    → Operation
```

A body must not validate under several subjects.

This changes internal relation keys, but no public realization or compiler digest exists. Record that consequence explicitly and update all compiler projections, tests, and diagnostics together.

## 4.13 Target-contract versions describe real schemas · `rule:guide11-exec:target-version-honesty`

Choose one honest state.

Recommended for the current tree:

```text
TargetContractVersion::V2
    supported

TargetContractVersion::V1
    historical constant or documentation only
    not accepted by TargetContractVersion::supported
```

If V1 remains supported, implement genuine version-specific opcode, capability, evidence, operand, and success-algebra validation. A V2 body stamped V1 must never validate.

## 4.14 Signature weld covers the whole behavior · `rule:guide11-exec:signature-weld`

Extend the signature weld to check:

- nonempty signature encoding;
- empty signature admission;
- recognized public-key encoding;
- unknown nonempty key admission;
- `UnknownPublicKeyTypeRule`;
- recognized-key success case;
- unknown-key unverified success case;
- branching-form Boolean result;
- verifying-form empty result;
- empty-signature failure behavior;
- invalid-signature abort behavior;
- empty-public-key rejection;
- validation budget;
- evidence requirements.

A subcontract saying unknown keys reject must not coexist with opcodes saying unknown keys succeed without verification.

## 4.15 Constructor retry distinguishes transient and permanent defects

Use one shared predicate:

```rust
fn retryable(defect: ConstructionDefect) -> bool {
    matches!(
        defect,
        ConstructionDefect::Tweak(TweakDefect::TweakNotAScalar)
            | ConstructionDefect::Tweak(TweakDefect::TweakedKeyIsIdentity)
    )
}
```

Nonretryable:

```text
every TreeDefect
InternalKeyNotOnCurve
```

Retryable:

```text
tweak not a scalar
tweaked key is identity
```

Use the same classifier in canonical ordering and totality policy.

## 4.16 Resource rows are welded to stack behavior

For every opcode, derive the greatest surviving main-stack growth over:

- every success case;
- `ConsumeOperandsPushFalse`;
- `RetainOperandsPushFalse`.

Abort-only outcomes have no surviving state but may still require a separately documented transient-resource rule if the target accounts for transient stack growth.

Compare the derived maximum with `OpcodeResourceCost::maximum_stack_growth`.

Altstack growth remains zero until a reviewed primitive changes it.

## 4.17 Report-digest policy is made coherent

Current policy says native reports have no admitted persistent identity.

Therefore the default Guide-11 repair is:

- remove bare report digests from maintained research claims where exact report assets are not retained;
- preserve run facts as historical narrative evidence;
- do not add replacement hashes.

If maintainers instead choose persistent report identity, the report artifact must be retained and admitted under ADR-016 with:

```text
typed role
report schema
exact target/deployment subjects
exact canonical fixture or matrix subject
producer and configuration
result status
recipe identifier
stale conditions
payload or payload digest
```

“Artifact absent, bare digest retained” is not an accepted middle state.

---

## Preflight gate · `gate:guide11-exec:preflight`

Core declassification work begins only when:

- arbitrary primitive fixtures cannot reach the native evidence gate;
- arbitrary prototype matrices cannot reach prototype gates;
- canonical wrappers have no public unchecked constructor;
- transcripts retain exact target, deployment, and request subjects;
- executor requests no longer disclose expected outcomes;
- environment rebinding rejects;
- resource evidence is attributed by enforcement layer;
- primitive gate rejects failed completeness and every failed canonical case;
- required provenance is validated;
- Meson requires explicit executor classification;
- V1 is either genuinely supported or no longer advertised;
- signature behavior is fully welded;
- relation identity has one canonical subject per body;
- startup failure reaps the direct child;
- constructor retry reports permanent defects immediately;
- stack resource rows are welded;
- bare unretained report digests are removed or formally admitted;
- backlog state and finding IDs are reconciled;
- focused tests pass;
- the complete working Rust lane passes;
- the tree is clean.

---

# 5. Target review for confidential values · `sec:guide11-exec:target-review`

## 5.1 Review source under ADR-018

Perform target review in the ADR-018 Elements workspace:

```text
master
    pristine upstream mirror

fix/*
    one upstreamable fix each

notes
    local review register and tooling only

merged
    derived integration branch used for builds and native runs
```

Record:

- intended merged tip;
- upstream base;
- included local topic census;
- node binary’s embedded revision;
- framework revision;
- exact source locations;
- exact tests consulted.

Do not import the upstream revision into target semantic identity.

## 5.2 Questions the review must answer

Review exact target behavior for:

1. explicit value encoding;
2. confidential value encoding;
3. explicit asset encoding;
4. confidential asset encoding;
5. value commitment point encoding;
6. asset generator derivation;
7. spent-input commitment introspection;
8. output commitment introspection;
9. exact byte equality of canonical commitments;
10. scalar multiplication relation;
11. point-addition or tweak relation;
12. compressed and x-only parity handling;
13. scalar canonicality and group-order rejection;
14. transaction confidential-value conservation;
15. full-consumption blinding balance;
16. rangeproof requirements;
17. surjection-proof requirements;
18. issuance interaction;
19. selected output-committing sighash behavior;
20. whether a public opening is verifiable without hidden node state.

Every accepted fact enters typed Rust. Human source provenance remains in `plans/reference/elements-tapscript.md`.

## 5.3 Do not presume the conceptual formula

The conceptual relation is:

\[C = rG + vH_A\]

Source review must establish:

- whether the target uses this sign convention;
- how \(H_A\) is derived;
- whether \(C\) is compressed, x-only, or another encoding;
- whether the opening scalar is interpreted big-endian or little-endian;
- whether zero or out-of-range scalars are admitted;
- whether parity is retained or normalized;
- whether the target relation verifies a general point addition or only a narrower tweak relation.

A pattern built by analogy with taproot tweak verification is rejected unless its exact relation is proved equivalent.

## 5.4 Explicit linked generators are permitted only under exact binding

For a closed asset whose identity remains explicit, a future backend may link a generator constant derived from that exact asset identity.

The prototype must then establish:

```text
explicit asset identity checked
+
linked generator corresponds to that asset under the reviewed recipe
+
opening proof uses that exact generator
```

The target may not need to recompute the generator on-chain if the linked bundle binds both values and the relation is separately translation-validated. Guide 11 must state which layer owns that binding.

## 5.5 Selected sighash profile is a prerequisite for accepted normalization

An owner-authorized normalization result is not accepted until a selected output-committing signature profile is reviewed and target-native acceptance exists.

Required signature commitment includes:

- normalized output;
- public amount;
- capsule bytes or authenticated capsule commitment;
- output value commitment;
- output program;
- protected change;
- every other protected output.

A transaction-signature rejection suite alone is insufficient. At least one valid signature over the executor-materialized transaction must pass.

---

# 6. Dependency decision · `sec:guide11-exec:dependencies`

## 6.1 Keep `target-elements` dependency-free

The static target contract remains standard-library-only.

No curve, transaction, serialization, or node library enters `target-elements`.

## 6.2 Candidate dependency belongs in conformance

A generic Elements or secp256k1-zkp dependency may enter `target-elements-conformance` only when required for:

- generator derivation;
- Pedersen commitment construction;
- rangeproof construction;
- surjection-proof construction;
- confidential transaction serialization;
- independent public vectors.

Before adoption, record:

- crate and exact version;
- upstream repository and tag;
- licence;
- Rust 1.88 compatibility;
- enabled features;
- transitive graph;
- build scripts;
- dependency-internal unsafe/FFI;
- native-library requirements;
- deterministic construction behavior;
- platform support;
- advisory status;
- lockfile impact;
- why the current upstream Python framework is insufficient.

## 6.3 Independent oracle and materializer must not be one opinion

Preferred separation:

```text
target transaction materialization:
    upstream Elements framework and node

expected commitment oracle:
    independent Rust/public-vector implementation

target execution:
    external reviewed Elements node
```

If a shared cryptographic library is unavoidable, record the shared dependency and narrow the independence claim accordingly.

---

# 7. Independent commitment oracle · `sec:guide11-exec:oracle`

## 7.1 Purpose

The host oracle computes expected generators, commitments, and opening relations without using:

- the tapscript opening pattern;
- the native executor’s answer;
- a report row;
- target observations as expected values.

Suggested public test type:

```rust
pub struct PublicCommitmentVector {
    pub asset_id: [u8; 32],
    pub amount: u64,
    pub blinding_factor: [u8; 32],
    pub expected_generator: Vec<u8>,
    pub expected_commitment: Vec<u8>,
}
```

The internal implementation may use typed point and scalar values.

## 7.2 Domain checks

The semantic amount satisfies:

\[0 \le v < 2^{51}\]

The oracle also validates:

- asset identifier width;
- scalar width;
- scalar group domain;
- canonical point encoding;
- generator validity;
- commitment validity;
- parity;
- no trailing bytes.

## 7.3 Required vectors

At minimum:

- amount zero;
- amount one;
- amount \(2^{51}-1\);
- zero blinding factor;
- nonzero blinding factor;
- both output-point parities;
- generator for at least two distinct assets;
- wrong asset with same amount and blinder;
- scalar zero;
- scalar at group order;
- scalar above group order;
- malformed compressed point;
- malformed x-only point;
- one-bit commitment mutation;
- byte-reversed scalar;
- wrong-width scalar.

## 7.4 Three-way comparison

Where practical, require:

```text
independent host oracle commitment bytes
=
transaction-construction library commitment bytes
=
commitment bytes observed by target introspection
```

A disagreement stops the batch. Expected values are not rewritten to match target observations without triage.

---

# 8. Confidential transaction fixture language · `sec:guide11-exec:ct-fixtures`

## 8.1 Test-only representation

Add explicit test-only fixture types rather than smuggling confidential construction through raw byte arrays alone.

Illustrative shape:

```rust
pub enum TestValueRepresentation {
    Explicit {
        amount: u64,
    },
    Confidential {
        amount: u64,
        asset_blinding_factor: [u8; 32],
        value_blinding_factor: [u8; 32],
        nonce_seed: [u8; 32],
        rangeproof_seed: [u8; 32],
    },
}
```

A corresponding asset representation states whether the asset is explicit or confidential.

Every field is public disposable fixture data.

## 8.2 Deterministic test randomness

All randomness is explicit:

```text
asset blinding factor
value blinding factor
nonce seed
rangeproof seed
surjection-proof seed
test signing scalar where required
```

Given equal explicit inputs, the materialized transaction bytes must be equal.

Production randomness is out of scope.

## 8.3 Execution outcomes distinguish layers

Extend the execution result vocabulary as needed so a report distinguishes:

```text
fixture construction failure

executor infrastructure failure

consensus transaction rejection before script

script-path rejection

relay-policy rejection

accepted transaction
```

A malformed rangeproof rejected before script execution is CT consensus evidence, not opening-script evidence.

A fixture-construction error is not a target rejection.

## 8.4 Required conservation matrix

| Case | Expected layer and result |
|---|---|
| explicit input → explicit output, balanced | accept |
| confidential input → confidential output, balanced | accept |
| confidential input → public committed output, balanced | accept if candidate supports |
| confidential input → explicit output + private change | accept if blinders close |
| several confidential inputs → public committed output | accept if blinders close |
| one-unit semantic imbalance | consensus reject |
| correct amounts, wrong blinding balance | consensus reject |
| malformed rangeproof | consensus reject |
| malformed surjection proof | consensus reject |
| wrong explicit asset/generator | reject at owning relation |
| copied output commitment from another asset | reject |
| hidden confidential output | closure reject where claimed |

Whole-transaction conservation and local opening remain separate report claims.

---

# 9. Candidate A — Explicit-only public boundary · `candidate:guide11-exec:explicit-only`

## 9.1 Policy

```text
PrivateCommitted
    lateral transfer only

PublicCommitted
    unsupported or normalization-only

Explicit
    accepted at public amount-dependent boundary
```

## 9.2 Strengths

- smallest public-boundary target program;
- no public-opening proof;
- no capsule;
- direct public amount availability;
- simplest fresh-process lifecycle;
- easiest auditability.

## 9.3 Limitations

- private values have no direct amount-dependent boundary;
- enabled private objects require normalization or explicit lifecycle incompleteness;
- normalization costs an extra transaction;
- disclosure occurs no later than normalization;
- explicit-only cannot be described as disclosure-minimal.

## 9.4 Acceptance

Explicit-only may be selected only if one of these holds:

1. private committed representation is not enabled for objects requiring a public boundary;
2. owner-authorized normalization exists;
3. lifecycle incompleteness is typed and blocks release claims.

A package must not claim:

```text
private representation supported
```

while omitting all required private exits.

---

# 10. Candidate B — Owner-authorized normalization · `candidate:guide11-exec:normalization`

## 10.1 Policy

```text
PrivateCommitted
    ↓ owner-authorized normalization
PublicCommitted or Explicit
```

Normalization changes representation only.

It preserves:

```text
semantic amount
explicit asset identity
owner
object family
class
protocol role
non-representation public projection
```

## 10.2 Variants

### Private → PublicCommitted

The output retains commitment algebra and publishes an authenticated amount and opening.

Advantages:

- residual blinding may remain on the output;
- future target relations can verify the opening;
- unrelated constructors can use public evidence.

Costs:

- opening pattern;
- public capsule;
- additional proof and witness bytes;
- public disclosure.

### Private → Explicit + private change

The public output is explicit. A private change output carries residual blinding where necessary.

Advantages:

- future public use is simple.

Risks:

- complete output closure must prevent hidden private value;
- full consumption may have no change output to carry residual blinding;
- private change remains owner-dependent;
- normalization and ordinary transfer semantics must not be conflated.

### Several private inputs → Explicit

This may allow blinders to cancel, but it is accepted only after exact target transaction evidence. It is not assumed from algebra alone.

## 10.3 Authorization

Normalization is owner-authorized.

Every consumed owner signs the finalized output set.

No operator key. No permissionless trigger.

The prototype preserves the owner rather than importing destination-changing transfer policy.

## 10.4 Threat matrix

| Mutation | Result |
|---|---|
| amount changed | reject |
| owner changed | reject |
| asset changed | reject |
| confidential closed asset substituted | reject |
| opening copied from another output | reject |
| output commitment mismatches amount/opening | reject |
| wrong blinding balance | consensus reject |
| hidden private output absorbs value | closure reject |
| extra output appears after signing | signature reject |
| representation changes without owner authorization | reject |
| public evidence exists only in creator memory | fresh-process failure |

---

# 11. Candidate C — Direct authenticated public opening · `candidate:guide11-exec:direct-opening`

## 11.1 Policy

```text
PrivateCommitted input
    ↓ owner-authorized amount-dependent transition
public result or PublicCommitted successor
```

The target authenticates the input amount during the same transition.

## 11.2 Required relation

A candidate must establish:

- exact input commitment;
- exact explicit asset or linked generator;
- exact public amount;
- exact opening relation;
- amount domain;
- semantic boundary equation;
- complete target value conservation;
- residual blinding closure;
- owner authorization;
- durable public evidence where a successor needs it.

## 11.3 Candidate proof outline

Subject to source review, one possible shape is:

1. authenticate the explicit asset;
2. obtain the exact linked asset generator;
3. validate \(v < 2^{51}\);
4. bind the amount’s scalar encoding to \(v\);
5. establish the amount component;
6. establish the blinding component;
7. establish their sum equals the actual value commitment;
8. check every primitive success flag;
9. leave authenticated \(v\) available to the caller.

This is not an accepted construction. It is a question the source review and prototype answer.

## 11.4 Parity is load-bearing

The proof must test both point parities.

An x-only primitive that implicitly selects even \(y\) cannot be used to verify a general compressed commitment unless the pattern proves the normalization or negation that makes the relations equivalent.

A candidate that loses parity is rejected.

## 11.5 Direct boundary versus durable successor

Two cases remain separate:

```text
opening needed only during current boundary
    public amount may appear in current event/result

PublicCommitted successor needed later
    opening evidence must remain durably available and instance-bound
```

The first does not imply the second.

---

# 12. Public opening capsule · `sec:guide11-exec:capsule`

## 12.1 Required contents

A candidate capsule contains only fields not already authenticated elsewhere.

Possible fields:

```text
domain separator
capsule schema
target contract version
prototype constructor schema
output role or ordinal
explicit asset or generator identity
public amount
opening scalar or proof data
exact value commitment
exact output program
metadata commitment
```

Each omitted field must have an explicit authenticated source.

## 12.2 Candidate locations

| Location | Strength | Main concern |
|---|---|---|
| constructor metadata | strong object binding | constructor cost and coupling |
| nonspendable data output | durable and indexable | exact output-instance binding |
| creating transaction witness | already public chain data | retrieval and pruning assumptions |
| separate ordinary output | straightforward publication | spendability and role ambiguity |
| external index only | easy cache | rejected as sole authority |

An index may cache capsules. It cannot be the only authoritative source.

## 12.3 No circular transaction identity

A capsule must not require a final transaction ID if that transaction ID commits to the capsule and creates an unresolved self-reference.

Prefer noncircular binding:

```text
output role
output ordinal
exact commitment
exact explicit asset/generator
exact output program
constructor schema
capsule schema
```

## 12.4 Canonical encoding

The capsule fixes:

- field order;
- field widths;
- byte order;
- scalar encoding;
- point encoding;
- domain string;
- schema number;
- absent-field rules;
- duplicate rejection;
- trailing-byte rejection;
- unknown-schema rejection.

No tolerant or alternate encoding is accepted in the prototype.

## 12.5 Swap resistance

Swapping any of the following must reject:

- output role or ordinal;
- commitment;
- asset/generator;
- output program;
- constructor schema;
- capsule schema;
- amount;
- opening.

If identical outputs intentionally admit interchangeable capsules, state that explicitly and prove it does not weaken object identity.

## 12.6 Signature commitment

Owner authorization commits:

- exact capsule bytes or authenticated capsule commitment;
- exact public amount;
- value commitment;
- output program;
- output role;
- recipient;
- protected change;
- complete protected output set.

A capsule appended or changed after signing is rejected.

---

# 13. Fresh-process lifecycle proof · `sec:guide11-exec:fresh-process`

## 13.1 Purpose

“Public” means:

```text
recoverable from canonical public chain data
by a party that did not participate in creation
```

It does not mean:

```text
still present in the creator’s process
```

## 13.2 Process A — construction

Process A receives:

- private test opening;
- deterministic test blinders and proof seeds;
- reviewed target;
- public prototype configuration.

It constructs and confirms the private-to-public transaction.

It publishes only the selected canonical chain data and capsule location.

## 13.3 Destruction boundary

Before Process B starts, destroy:

- creator-local private opening state;
- unpublished blinding values;
- construction plans;
- unpublished side files;
- creator process memory;
- temporary directories not part of the public fixture.

The test oracle may retain the expected semantic result outside Process B. Process B must not receive it through its construction interface.

## 13.4 Process B — unrelated future construction

Process B receives only:

- public chain transaction and witness data;
- outpoint;
- reviewed target;
- public prototype schema;
- public capsule;
- its own sponsor-local funds where needed.

It must:

1. locate public evidence;
2. parse it canonically;
3. bind it to the intended output;
4. verify the commitment opening;
5. recover the public semantic amount;
6. construct a future generic spend;
7. execute the spend;
8. use no owner or operator secret.

## 13.5 Required failures

- missing capsule;
- private-file-only capsule;
- copied capsule;
- stale capsule;
- wrong output;
- wrong amount;
- wrong opening;
- wrong asset;
- wrong constructor;
- malformed capsule;
- owner-private dependency;
- wrong chain context;
- hidden cache dependency.

The fresh-process test must use an actual process boundary, not two functions in one process sharing memory.

---

# 14. Safety and minimality reports · `sec:guide11-exec:reports`

## 14.1 Separate report roles

Use distinct validated report types or explicit disjoint roles for:

```text
ConfidentialTransactionSafety

AuthenticatedOpeningSafety

CapsuleBindingSafety

FreshProcessLifecycle

RepresentationMinimality
```

A report of one role cannot satisfy another.

No report carries a digest in this phase.

## 14.2 Safety report

Required mutation classes:

- wrong amount;
- wrong asset;
- wrong generator;
- wrong commitment;
- wrong blinding factor;
- malformed scalar;
- malformed point;
- malformed rangeproof;
- malformed surjection proof;
- copied capsule;
- hidden value output;
- wrong owner;
- missing authorization;
- output mutation after authorization;
- incomplete public data;
- invalid CT balance;
- confidential closed asset.

## 14.3 Minimality report

Where supported, compare semantically equivalent:

```text
Explicit

PublicCommitted

PrivateCommitted → normalization

PrivateCommitted → direct boundary
```

Compare:

- target verdict;
- semantic amount;
- explicit asset;
- owner;
- object role;
- public transition projection;
- lifecycle result;
- disclosures;
- target resources.

A green explicit path proves nothing about PublicCommitted support.

A green PublicCommitted path proves nothing about direct private support.

## 14.4 Typed disclosure reasons

Every newly public fact carries a typed reason:

```rust
pub enum DeclassificationReason {
    PublicState,
    PublicEvent,
    PermissionlessConstructibility,
    BoundaryArithmetic,
    TargetSafety,
    DeploymentPolicy,
}
```

Prefer existing realization-owned disclosure types where they already express the required distinction.

Keep separate:

```text
semantic disclosure

target-safety disclosure

deployment-policy disclosure
```

An explicit-only policy may add deployment-policy disclosure without claiming the semantic relation intrinsically requires explicit encoding.

---

# 15. Complete Guide-11 vector matrix · `sec:guide11-exec:vectors`

## 15.1 Opening relation

- amount zero;
- amount one;
- amount \(2^{51}-1\);
- zero blinding;
- representative nonzero blinding;
- both point parities;
- wrong amount by one;
- wrong blinding factor;
- wrong asset generator;
- wrong commitment;
- copied opening;
- malformed compressed point;
- malformed x-only point;
- scalar zero where forbidden;
- scalar at group order;
- scalar above group order;
- wrong scalar width;
- wrong byte order;
- alternate commitment encoding;
- trailing bytes;
- unchecked arithmetic/curve success flag.

## 15.2 Commitment equality

- canonical commitment against itself;
- one-bit difference;
- same amount, different blinding;
- same blinding, different amount;
- same values, different asset generators;
- noncanonical encoding of the apparent same point;
- malformed point;
- commitment copied from another output.

## 15.3 Capsule

- canonical capsule;
- wrong domain;
- wrong schema;
- wrong target version;
- wrong constructor schema;
- wrong output ordinal;
- wrong output role;
- wrong output program;
- wrong asset/generator;
- wrong commitment;
- wrong amount;
- wrong opening;
- missing field;
- duplicate field;
- reordered canonical field;
- trailing bytes;
- copied capsule;
- post-signing mutation;
- private-state-only capsule.

## 15.4 Confidential transaction balance

- explicit balanced;
- confidential balanced;
- mixed balanced;
- full private consumption;
- partial consumption with private change;
- several confidential inputs;
- one-unit imbalance;
- correct semantic values with wrong blinder sum;
- malformed rangeproof;
- malformed surjection proof;
- wrong nonce;
- hidden extra output;
- asset mismatch;
- copied commitment.

## 15.5 Normalization

- private → PublicCommitted;
- private → Explicit where algebra permits;
- private → Explicit + private change;
- several private inputs;
- full consumption;
- partial consumption;
- owner preserved;
- owner mutation;
- amount mutation;
- asset mutation;
- output commitment mutation;
- opening omitted;
- opening copied;
- hidden private output;
- output mutation after signing.

## 15.6 Direct boundary

Use one generic target amount-dependent transition, not an attestation-contract operation.

Required:

- correct private opening and public result;
- wrong public result;
- wrong input commitment;
- opening belongs to another input;
- public result shortened while another output grows;
- full-consumption blinding closure;
- omitted private residual;
- extra hidden output;
- missing public evidence;
- stale public evidence;
- output mutation after authorization.

## 15.7 Fresh process

- public evidence recovered from chain bytes;
- future generic spend accepted;
- creator opening deleted;
- creator process absent;
- missing evidence rejected;
- copied evidence rejected;
- wrong constructor rejected;
- wrong chain context rejected;
- cache rebuilt from chain data;
- no hidden environment value;
- no owner-private witness.

---

# 16. Resource evidence · `sec:guide11-exec:resources`

Measure complete target transactions, not isolated fragments only.

## 16.1 Opening verifier

Record:

- program bytes;
- witness bytes;
- amount bytes;
- opening bytes;
- point/generator bytes;
- initial and peak stack;
- altstack;
- largest item;
- hash operations;
- curve operations;
- validation budget;
- transaction weight;
- consensus verdict;
- policy verdict.

## 16.2 Capsule

Record:

- capsule bytes;
- constructor growth where metadata-bound;
- data-output growth where separate;
- witness growth where history-bound;
- parsing and verification program bytes;
- retrieval diagnostics, explicitly noncanonical.

## 16.3 Normalization

Measure:

- full private consumption;
- private change;
- several private inputs;
- PublicCommitted output;
- Explicit output where permitted;
- owner authorization;
- proof bytes;
- complete transaction weight;
- policy result.

## 16.4 Direct boundary

Measure:

- one private input;
- multiple private inputs;
- public result;
- no private change;
- private change;
- maximum semantic amount;
- maximum capsule;
- complete signature and proof witness.

## 16.5 Decision table

| Path | Transactions | Public amount timing | Additional proof bytes | Fresh-process future use | Decision |
|---|---:|---|---:|---:|---|
| already explicit | 0 extra | already public | low | yes | |
| normalize to explicit | +1 | normalization | measured | yes | |
| normalize to public committed | +1 | normalization | measured | yes | |
| direct public committed | 0 extra | boundary | measured | only if capsule passes | |

No architecture batch bound is calibrated from these figures.

---

# 17. Implementation waves · `sec:guide11-exec:waves`

Each wave must end formatted, focused-test green, documented, and committed before the next begins.

## Wave 0 — Reproduce and classify the review register

**Deliverables**

- one focused reproduction per `G11-R01` through `G11-R14`;
- explicit confirmation, refutation, or reclassification;
- no broad refactor yet;
- updated backlog finding table with unique IDs.

**Suggested commit**

```text
plans: record the Guide-11 preflight findings
```

## Wave 1 — Canonical evidence subjects

**Files likely affected**

```text
packages/target-elements-conformance/src/fixture.rs
packages/target-elements-conformance/src/prototype.rs
packages/target-elements-conformance/src/executor.rs
packages/target-elements-conformance/src/validate.rs
packages/target-elements-conformance/src/prototype_validate.rs
packages/target-elements-conformance/src/report.rs
packages/target-elements-conformance/src/prototype_report.rs
```

**Deliverables**

- canonical primitive fixture wrapper;
- canonical constructor and wide-floor matrix wrappers;
- experimental versus evidence report states;
- claim sets no longer caller-authoritative;
- exact matrix regeneration and comparison;
- trivial-true-script laundering regressions.

**Suggested commit**

```text
target-conformance: bind evidence claims to canonical subjects
```

## Wave 2 — Subject-bound transcript and protocol revision 3

**Deliverables**

- transcript retains exact target, deployment, and sent requests;
- executor request omits expected results and claims;
- protocol schema bump;
- report schema bump where required;
- direct fixture and deployment rebinding regressions;
- mock updated without reading request expectations.

**Suggested commit**

```text
target-conformance: bind transcripts to the executed subjects
```

## Wave 3 — Gate and provenance closure

**Deliverables**

- environment comparison repeated at report validation;
- primitive gate rejects failed summary and every failed canonical case;
- resource evidence attribution corrected;
- policy resource row made honestly unresolved or supplied real policy evidence;
- explicit expected executor provenance;
- ADR-018 validation;
- Meson executor class defaults to unselected;
- pre-supervisor child cleanup.

**Suggested commit**

```text
target-conformance: close native gate provenance and status
```

## Wave 4 — Target and realization internal consistency

**Deliverables**

- one canonical subject per realization relation body;
- target V1 support corrected;
- complete signature weld;
- generic opcode resource stack-growth weld;
- constructor retryability predicate;
- no published architecture, behavioural, or anchor-set identity movement unless independently justified;
- report-digest and backlog documentation reconciliation.

**Suggested commits**

```text
realization: canonicalize relation identity subjects
target-elements: complete target contract welds
target-conformance: classify constructor retry failures
plans: reconcile current evidence and identity records
```

## Wave 5 — Confidential-value target review

**Deliverables**

- exact source review;
- commitment encoding;
- asset-generator relation;
- input/output commitment availability;
- CT consensus behavior;
- signature profile required by normalization;
- capability and evidence dispositions;
- reference provenance update.

**Suggested commit**

```text
target-elements: review the confidential value boundary
```

## Wave 6 — Dependency and independent oracle

**Deliverables**

- dependency review, if needed;
- exact generator vectors;
- exact commitment vectors;
- scalar and point boundary vectors;
- host/library equality;
- no target observation used as expected value.

**Suggested commit**

```text
target-conformance: add the independent commitment oracle
```

## Wave 7 — Generic confidential transactions

**Deliverables**

- deterministic test-only blinders and proof seeds;
- confidential inputs and outputs;
- rangeproof and surjection-proof materialization;
- balance failures;
- exact failure-layer classification;
- complete CT safety report.

**Suggested commit**

```text
target-conformance: materialize generic confidential transactions
```

## Wave 8 — Opening and equality prototypes

**Deliverables**

- readable target pattern candidates;
- exact stack schedule;
- immediate success-flag checks;
- amount-domain checks;
- asset/generator binding;
- parity vectors;
- native target report;
- resource measurements;
- explicit accepted, rejected, or deferred disposition.

**Suggested commit**

```text
tapscript: prototype authenticated public openings
```

## Wave 9 — Capsule candidates

**Deliverables**

- at least two viable capsule locations compared where target support permits;
- canonical capsule schema;
- noncircular binding;
- swap and mutation vectors;
- signature commitment;
- resource comparison.

**Suggested commit**

```text
tapscript: prototype durable public opening capsules
```

## Wave 10 — Normalization and direct-boundary comparison

**Deliverables**

- owner-authorized normalization;
- output-committing transaction signature;
- full and partial private consumption;
- direct private-to-public candidate;
- residual blinding closure;
- safety report;
- minimality report.

**Suggested commit**

```text
target-conformance: compare confidential-to-public paths
```

## Wave 11 — Fresh-process lifecycle

**Deliverables**

- Process A creates and confirms output;
- all creator-private state destroyed;
- Process B starts independently;
- public evidence reconstructed from chain;
- generic future spend accepted;
- missing/copied/stale evidence failures.

**Suggested commit**

```text
target-conformance: prove public lifecycle availability
```

## Wave 12 — Decision and Phase-3 handoff

**Deliverables**

- final representation policy matrix;
- public-declassification research result;
- D005 update if required;
- target/tapscript/conformance package documentation;
- Phase-3 card;
- backlog;
- identity and dependency impact;
- complete repository gate;
- clean tree.

**Suggested commit**

```text
plans: record the Guide-11 declassification decision
```

---

# 18. Focused verification · `sec:guide11-exec:verification`

## 18.1 Working cadence

For Rust changes:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Do not run the full release/Meson surface after every small edit.

## 18.2 Evidence-boundary tests

```sh
cargo test --locked -p tripod-target-elements-conformance
```

Required focused filters include:

```text
canonical fixture trust state
canonical prototype matrix trust state
primitive claim laundering
prototype claim laundering
transcript fixture rebinding
transcript target rebinding
transcript deployment rebinding
environment mismatch
failed-summary gate
failed optional case
resource enforcement-layer attribution
executor provenance
process-group adoption cleanup
protocol revision 3
request expectation exclusion
```

## 18.3 Static target

```sh
cargo test --locked -p tripod-target-elements
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-target-elements --no-deps
```

Focused areas:

```text
V1/V2 support
signature complete weld
unknown-key success path
stack/resource growth weld
commitment encodings
asset-generator facts
CT capabilities
sighash profile
evidence requirements
reviewed-target trust state
```

## 18.4 Tapscript

```sh
cargo test --locked -p tripod-tapscript
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-tapscript --no-deps
```

Focused areas:

```text
commitment equality
authenticated opening
amount domain
asset/generator binding
point parity
success flags
capsule parsing and binding
normalization schedule
direct-boundary schedule
abstract execution oracle
resource projection
prototype/release separation
```

## 18.5 Realization and compiler

Run whenever relation identity or representation/lifecycle declarations change:

```sh
cargo test --locked -p tripod-realization
cargo test --locked -p tripod-compiler
```

Required focused assertions:

```text
one relation body → one canonical relation ID
subject-only mutation rejects
scope projection remains deterministic
no public identity minted
representation and lifecycle censuses remain exact
```

## 18.6 Documentation and census

```sh
scripts/check-plans.sh
meson compile -C build lint
git diff --check
git diff --cached --check
```

Every new source or documentation file enters the nearest `meson.build` census in the same commit.

---

# 19. Full batch gate · `gate:guide11-exec:batch`

After all coherent waves:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
scripts/ci.sh
meson compile -C build
meson test -C build --print-errorlogs
```

Run real native matrices separately:

```text
canonical primitive matrix
constructor matrix
wide-floor matrix
confidential transaction matrix
commitment/generator matrix
opening/equality matrix
capsule matrix
normalization matrix
direct-boundary matrix
fresh-process lifecycle matrix
```

For each real matrix record:

- exact target contract;
- exact development binding;
- executor declaration;
- binary-reported revision;
- intended tip;
- upstream base;
- local topic census;
- observed network;
- observed genesis;
- case total;
- required claim total;
- failures;
- infrastructure errors;
- deterministic report-byte comparison.

Run dependency checks if the graph changed:

```sh
cargo tree --locked -e features
cargo metadata --locked
cargo audit
```

A missing `cargo-audit` is recorded as skipped, never passed.

Run document reproducibility when document inputs changed or batch policy requires it:

```sh
scripts/check-document-reproducibility.sh
```

A deferred run is recorded as deferred.

Final check:

```sh
git status --porcelain=v1 --untracked-files=all
```

It must be empty.

---

# 20. Acceptance criteria · `sec:guide11-exec:acceptance`

## 20.1 Direct authenticated opening

Accept only when:

- exact target commitment/value relation passes;
- exact explicit asset or linked generator is bound;
- amount domain is exact;
- malformed scalar and point encodings reject;
- wrong amount rejects;
- wrong blinding factor rejects;
- wrong generator rejects;
- copied commitment/opening rejects;
- both point parities are covered;
- every target success flag is checked;
- whole-transaction conservation passes separately;
- full-consumption residual blinding closes;
- owner authorization commits all protected outputs and public data;
- capsule is canonical and instance-bound where needed;
- fresh Process B constructs a valid future spend;
- complete target-native transactions pass;
- abstract and native stack behavior agree;
- predicted and observed resources agree.

## 20.2 Normalization

Accept only when:

- owner authorization is real and output-committing;
- amount is preserved;
- owner is preserved;
- explicit asset is preserved;
- object role is preserved;
- public representation is exactly authenticated;
- hidden private output escape is impossible;
- residual blinding closes;
- future public lifecycle succeeds from fresh-process data;
- explicit and normalized semantic projections agree;
- complete target-native transactions pass.

## 20.3 Explicit-only policy

Accept as the initial policy only when:

- direct and normalization candidates are rejected or deferred with typed reasons;
- private representations carry explicit lifecycle incompleteness;
- no package claims direct private boundary support;
- clients can determine before construction that explicit representation is required;
- the policy is typed as target/backend/deployment policy rather than semantic necessity;
- explicit-path safety evidence passes;
- the result is not described as disclosure-minimal.

## 20.4 Evidence acceptance

Every accepted result requires:

- canonical evidence subject;
- exact subject-bound transcript;
- executor receives no expected answer;
- exact case census;
- exact claim census;
- exact evidence-row census;
- validated report wrapper;
- observed chain identity;
- complete required executor provenance;
- zero failed canonical cases;
- zero required infrastructure errors;
- deterministic report bytes;
- no broad row passing while a defining required claim is absent.

---

# 21. Rejection criteria · `sec:guide11-exec:rejection`

Reject a representation candidate if:

- a false opening passes;
- commitment bytes can be swapped across assets;
- a capsule can be copied to another output;
- public amount is unauthenticated metadata;
- full consumption cannot close blinding;
- a future constructor needs owner-private state;
- confidential closed asset identity escapes;
- target stack behavior differs from the reviewed contract;
- an arithmetic or curve success flag is unchecked;
- tested and proposed production bytes differ;
- expected values are generated by the target or executor under test;
- only a host library verifies the opening;
- the complete transaction exceeds a hard target limit.

Reject the Guide-11 gate if:

- arbitrary fixtures or matrices can reach evidence gates;
- claims remain caller-authoritative;
- transcripts can be rebound;
- a report with `Failed` completeness can pass;
- a mock can satisfy a gate through a build-system default;
- required executable provenance is absent;
- broad evidence rows pass from the wrong enforcement layer;
- target V1/V2 semantics remain ambiguous;
- unsupported target capabilities are relabeled complete without a full pattern;
- network and genesis are caller labels rather than observed and rechecked facts;
- public evidence exists only in creator memory;
- normalization success is reported as direct support;
- direct support is reported as a production operation;
- a speculative digest is minted.

---

# 22. Identity and schema impact · `sec:guide11-exec:identity-impact`

Expected impact:

```text
Attestation version:
    unchanged

realization version:
    unchanged

realization letter:
    unchanged unless normative realization text changes

architecture schema:
    unchanged

architecture semantic hash:
    unchanged

architecture behavioural hash:
    unchanged

anchor-set hash:
    unchanged

generated architecture publications:
    unchanged

realization identity:
    none minted

compiler identity:
    none minted

target digest:
    none minted

opening-pattern digest:
    none minted

capsule digest:
    none minted

native report digest:
    none minted

deployment-profile identity:
    remains dormant and not production-release-valid
```

Likely schema changes:

```text
native executor protocol:
    revision 3, because expectations and claims leave requests

primitive native report:
    new revision if the expectation boundary or subject-binding fields change

prototype report:
    new revision if canonical matrix identity and request boundary change

target contract:
    revision bump only if reviewed target semantics or the algebra changes;
    research prototype code alone does not move it
```

Internal realization relation IDs may change when subjects become canonical. No public realization or compiler identity exists, but the change must still be recorded because diagnostics, test fixtures, and internal projections move.

Old protocol and report revisions remain historical values. They are not silently widened.

---

# 23. Dependency impact · `sec:guide11-exec:dependency-impact`

Expected:

```text
target-elements:
    no dependency

tapscript:
    no new dependency expected

target-elements-conformance:
    possible reviewed Elements/secp256k1-zkp dependency
    if the current framework cannot provide an adequate materializer
    and an independent oracle remains available

Cargo.lock:
    changes only with explicit dependency review
```

The dependency review records:

```text
source
version
features
licence
MSRV
transitive graph
unsafe/FFI
native build requirements
determinism
parallelism
advisories
lockfile diff
replacement path
```

---

# 24. Final policy matrix · `tab:guide11-exec:result`

The Guide-11 result fills every cell with one of:

```text
supported directly
supported through normalization
explicit only
unsupported
deferred with named blocker
not applicable
```

| Object/use | Explicit | PublicCommitted | Private direct | Normalize first |
|---|---|---|---|---|
| lateral value transfer | | | | not applicable |
| public ownerless maintenance object | | | forbidden | not applicable |
| owner-authorized amount-dependent boundary | | | | |
| permissionless future maintenance | | | forbidden | not applicable |
| formula-bound payout boundary | | | | |
| full private input consumption | | | | |
| partial private consumption with private change | | | | |

Operation-specific names remain owned by later phases. Guide 11 chooses representation policy, not an operation ABI.

---

# 25. Exit checklist · `gate:guide11-exec:exit`

## Evidence foundation

- [ ] arbitrary primitive fixtures cannot reach the native evidence gate;
- [ ] arbitrary prototype matrices cannot reach prototype gates;
- [ ] primitive claims are canonical, not caller-authored;
- [ ] prototype claims are canonical, not caller-authored;
- [ ] transcripts retain exact target, deployment, and sent request subjects;
- [ ] executor requests contain no expected outcome or claim set;
- [ ] target/deployment rebinding rejects;
- [ ] report environment is rechecked against binding;
- [ ] resource evidence is attributed by enforcement layer;
- [ ] no failed report or canonical case passes a gate;
- [ ] executor classification is explicit;
- [ ] executable provenance satisfies ADR-018;
- [ ] startup failures kill and reap the direct child;
- [ ] protocol records remain bounded and strict.

## Target contract

- [ ] V1/V2 support is honest and version-dispatched or V1 is no longer advertised;
- [ ] relation bodies have one canonical relation identity;
- [ ] signature unknown-key behavior is fully welded;
- [ ] signature empty/invalid paths are fully welded;
- [ ] opcode stack resource rows are derived from stack contracts;
- [ ] commitment format is reviewed;
- [ ] asset-generator relation is reviewed;
- [ ] input/output commitment availability is reviewed;
- [ ] CT conservation remains separate evidence;
- [ ] commitment equality disposition is explicit;
- [ ] authenticated opening disposition is explicit;
- [ ] target primitive and backend pattern ownership remain distinct.

## Independent oracle

- [ ] generator oracle is independent of the target pattern;
- [ ] commitment oracle is independent of the target pattern;
- [ ] published or independently computed vectors exist;
- [ ] amount and scalar boundaries exist;
- [ ] both parities are covered;
- [ ] host/library/native comparison passes;
- [ ] target observations never generate expected values.

## Confidential transactions

- [ ] explicit balanced transaction passes;
- [ ] confidential balanced transaction passes;
- [ ] mixed transaction passes where claimed;
- [ ] one-unit imbalance rejects at consensus;
- [ ] wrong blinding balance rejects;
- [ ] malformed rangeproof rejects;
- [ ] malformed surjection proof rejects;
- [ ] test randomness is explicit and reproducible;
- [ ] construction failure remains distinct from target rejection.

## Opening pattern

- [ ] public amount binds exact commitment;
- [ ] exact asset/generator binds the relation;
- [ ] amount domain is enforced;
- [ ] malformed scalar and point encodings reject;
- [ ] wrong amount rejects;
- [ ] wrong blinder rejects;
- [ ] wrong generator rejects;
- [ ] copied opening rejects;
- [ ] every target success flag is checked;
- [ ] authenticated amount remains available to the caller;
- [ ] abstract and native stack behavior agree.

## Capsule

- [ ] canonical schema exists;
- [ ] no circular transaction-ID binding exists;
- [ ] capsule binds the exact output role;
- [ ] capsule swap rejects;
- [ ] malformed and unknown schemas reject;
- [ ] duplicate and trailing fields reject;
- [ ] owner authorization commits public evidence;
- [ ] capsule is available from canonical public data;
- [ ] Process B reconstructs the future spend without owner-private state.

## Normalization and direct paths

- [ ] normalization preserves amount;
- [ ] normalization preserves owner;
- [ ] normalization preserves explicit asset;
- [ ] normalization preserves object role;
- [ ] normalization is owner-authorized;
- [ ] full-consumption blinding closes;
- [ ] partial-consumption private change closes;
- [ ] hidden confidential output escape rejects;
- [ ] direct support is accepted only if complete;
- [ ] normalization is not reported as direct support;
- [ ] explicit-only remains an honest possible result;
- [ ] unsupported private lifecycles remain explicit.

## Reports and planning

- [ ] safety and minimality reports remain separate;
- [ ] every newly public fact has a typed reason;
- [ ] exact fixture, matrix, claim, and evidence censuses pass;
- [ ] reports reproduce byte-for-byte;
- [ ] no bare unretained report digest remains;
- [ ] public-declassification research records the result;
- [ ] D005 is updated only if required;
- [ ] Phase-3 card records the final matrix;
- [ ] backlog current state is accurate;
- [ ] finding IDs are unique;
- [ ] no operation, ABI, bundle, calibration, deployment, or release is overclaimed;
- [ ] no speculative identity is minted;
- [ ] all full gates pass;
- [ ] final tree is clean.

---

# 26. Completion report template · `sec:guide11-exec:completion-report`

```text
Guide 11 result
===============

Starting state:
    source revision:
    working tree:
    Guide-10 constructor decision:
    Guide-10 wide-floor decision:
    public-declassification research revision:

Preflight review:
    G11-R01 transcript binding:
    G11-R02 primitive claim authority:
    G11-R03 prototype claim authority:
    G11-R04 resource attribution:
    G11-R05 failed-report gate:
    G11-R06 executor provenance:
    G11-R07 Meson executor default:
    G11-R08 relation identity:
    G11-R09 target V1/V2:
    G11-R10 signature weld:
    G11-R11 report digests:
    G11-R12 backlog:
    G11-R13 startup cleanup:
    G11-R14 constructor retry:
    G11-H01 stack resource weld:
    G11-H02 issuance future blocker:

Evidence boundary:
    canonical primitive wrapper:
    canonical constructor matrix:
    canonical wide-floor matrix:
    ad hoc report role:
    transcript subject binding:
    request expectation removed:
    protocol schema:
    primitive report schema:
    prototype report schema:
    exact case census:
    exact claim census:
    exact evidence census:
    environment recheck:
    failed-case gate:
    deterministic bytes:

Executor provenance:
    adapter:
    adapter version:
    framework revision:
    node:
    node version:
    binary-reported revision:
    intended tip:
    upstream base:
    local topics:
    observed network:
    observed genesis:
    classification explicitly selected:

Target review:
    commitment encoding:
    asset-generator relation:
    input commitment introspection:
    output commitment introspection:
    commitment equality:
    authenticated opening:
    whole-transaction conservation:
    selected sighash profile:
    target-contract version impact:

Dependency review:
    package:
    version:
    features:
    licence:
    MSRV:
    transitive graph:
    unsafe/FFI:
    native build:
    determinism:
    advisories:
    Cargo.lock:

Independent oracle:
    implementation:
    public vectors:
    generator vectors:
    commitment vectors:
    scalar and parity vectors:
    host/library agreement:
    host/native agreement:

Confidential transaction executor:
    explicit transaction:
    confidential transaction:
    mixed transaction:
    full consumption:
    private change:
    deterministic randomness:
    rangeproof:
    surjection proof:
    one-unit imbalance:
    wrong blinder sum:
    failure-layer classification:

Opening prototype:
    candidate:
    amount domain:
    asset/generator binding:
    scalar relation:
    point relation:
    parity:
    success flags:
    positive vectors:
    negative vectors:
    target-native result:
    resources:
    disposition:

Capsule:
    selected location:
    schema:
    domain separator:
    output binding:
    noncircular construction:
    signature commitment:
    swap mutations:
    fresh-process availability:
    resources:

Normalization:
    private → PublicCommitted:
    private → Explicit:
    private → Explicit + private change:
    full consumption:
    partial consumption:
    owner authorization:
    semantic equality:
    target-native result:
    resources:
    disposition:

Direct boundary:
    private opening:
    public result:
    residual blinding:
    owner authorization:
    durable public evidence:
    fresh-process future use:
    target-native result:
    disposition:

Final representation policy:
    lateral transfer:
    public maintenance:
    owner-authorized public boundary:
    permissionless future use:
    formula-bound payout:
    direct private support:
    normalization:
    explicit-only fallback:

Reports:
    CT safety cases:
    opening safety cases:
    capsule safety cases:
    lifecycle cases:
    minimality cases:
    failures:
    infrastructure errors:
    unresolved claims:
    deterministic bytes:

Identity impact:
    Attestation:
    realization version:
    architecture schema:
    architecture semantic hash:
    architecture behavioural hash:
    anchor-set hash:
    target contract version:
    report identities:
        none
    opening/capsule identities:
        none
    deployment-profile identity:
        dormant

Verification:
    cargo fmt --all:
    cargo clippy --workspace --all-targets --locked -- -D warnings:
    cargo test --workspace --locked:
    target-elements:
    tapscript:
    target-elements-conformance:
    realization:
    compiler:
    Rustdoc:
    scripts/check-plans.sh:
    meson compile -C build lint:
    scripts/ci.sh:
    meson compile -C build:
    meson test -C build --print-errorlogs:
    real primitive matrix:
    real constructor matrix:
    real wide-floor matrix:
    real CT matrix:
    real opening matrix:
    real capsule matrix:
    real normalization matrix:
    fresh-process lifecycle:
    cargo audit:
    document reproducibility:
    git diff --check:
    final git status:

Planning handoff:
    public-declassification research:
    D005:
    Phase 3:
    backlog:
    next guide:

Residuals:
```

---

# 27. Handoff after Guide 11 · `sec:guide11-exec:handoff`

Guide 11 completes the final Phase-3 foundational prototype question only if:

- STATE constructor continuity remains accepted or explicitly rejected;
- exact wide-floor arithmetic remains accepted or explicitly rejected;
- public declassification selects an initial policy or records explicit target rejection;
- every preflight evidence-boundary finding is closed;
- Phase-3 exit gate passes.

The next guide may then begin the first complete compiler-to-target operation:

```text
Guide 12 — End-to-End Compact ASH
```

Guide 12 consumes rather than reopens:

- reviewed target contract;
- typed instruction core;
- canonical native-evidence boundary;
- constructor result where applicable;
- wide-floor result where applicable;
- public-declassification policy;
- sponsor-value opacity;
- exact compiler target requirements.

Compact ASH remains the first operation because it requires neither STATE continuity nor wide-floor arithmetic, but it does require the final public-value and sponsor-opacity policy.

Guide 12 may introduce, for one operation only:

- compiler target-plan consumption;
- backend proof patterns;
- concrete relation placement;
- target layout;
- relocatable artifacts;
- linker candidate;
- transaction ABI;
- complete target transactions;
- relation-indexed positive and negative evidence.

Guide 11 introduces none of those.

---

## Closing statement · `rem:guide11-exec:closing`

> The public side of a confidential-value boundary is valid only when the exact amount is authenticated, the asset identity remains rigid, the commitment algebra closes, public evidence binds the intended object without circularity, and a fresh unrelated process can recover and verify that evidence from canonical chain data. If the target cannot establish all five, the correct result is normalization, explicit-only use, typed deferral, or explicit rejection—not an optimistic representation claim.
