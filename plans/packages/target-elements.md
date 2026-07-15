# Elements Target Package Plan

> **Status:** PLANNED
> **Planned source directory:** `packages/target-elements`
> **Planned Cargo package:** `tripod-target-elements`
> **Planned Rust library name:** `target_elements`
> **Implementation phase:** Phase 3 — exact Elements target and foundational
> backend prototypes
> **Depends on decisions:**
> [D001](../decisions/001-typed-rust-is-normative.md),
> [D003](../decisions/003-multiple-backends-tapscript-first.md),
> [D004](../decisions/004-translation-validation-over-compiler-trust.md),
> [D005](../decisions/005-value-parametric-asset-rigid.md),
> [D006](../decisions/006-canonical-transaction-layout-abi.md)
> **Depends on planning reference:**
> [`../reference/elements-tapscript.md`](../reference/elements-tapscript.md)
> for human review only
> **Open research dependencies:** exact state-constructor, arithmetic,
> public-declassification, and settlement prototypes consume this package but do
> not define its protocol-neutral target boundary
> **Authority:** Typed backend compatibility contract and capability model for
> one Elements target; no authority over attestation-contract protocol semantics
> **Scope boundary (ADR-011):** no Elements consensus-implementation source
> revision is part of the protocol identity or a release pin; where this plan
> says "pin", it means the typed compatibility contract (opcodes, stack
> contracts, encodings, execution domain, resource interfaces) plus recorded
> integration-test provenance — not a consensus source audit
> **Machine-consumed planning document:** no

---

## 1. Purpose

The `target-elements` package will provide the exact typed target definition
used by:

- target-capability matching;
- Elements tapscript backend emission;
- target transaction construction;
- target resource analysis;
- target-native vector execution;
- deployment dependency evidence;
- release identity binding.

It turns externally verified Elements substrate facts into typed Rust values.

The package owns:

- the typed compatibility-contract identity (target schema/version);
- reviewed upstream provenance, recorded as review metadata (ADR-011);
- target network flavor;
- tapscript activation assumptions;
- tapscript leaf version;
- opcode assignments used by the project;
- opcode semantics relied upon by the backend;
- explicit/confidential asset and value encodings;
- transaction introspection result shapes;
- byte-order conventions;
- sighash capabilities and restrictions;
- relative-timelock semantics;
- confidential transaction capability claims;
- transaction and script resource limits;
- policy constraints relevant to construction and relay;
- target capability identity;
- source provenance for every advertised capability;
- target-specific deployment evidence requirements.

It does not own:

- attestation-contract assets, objects, operations, formulas, or invariants;
- proof-plan selection;
- operation transaction layouts;
- tapscript instruction selection;
- object-constructor design;
- deployment calibration;
- release evidence status.

The planned direction is:

```text
source-reviewed Elements revision
        +
typed target declarations
        ↓
validated ElementsTargetDefinition
        +
typed deployment/network instance
        ↓
ElementsTarget
   ├─────────────────────▶ compiler capability matching
   ├─────────────────────▶ tapscript backend
   ├─────────────────────▶ transaction construction
   ├─────────────────────▶ vector execution
   └─────────────────────▶ release evidence binding
```

The Markdown target survey remains a review aid. The compiler and backend
consume typed values from this package, not the Markdown.

---

## 2. Why this package is required

The compiler and backend need exact substrate facts.

Statements such as:

```text
output values can be introspected
relative timelocks are available
value commitments can be compared
one opcode consumes three stack items
one crypto opcode costs a target budget amount
```

are not protocol semantics. They are target claims.

If those claims remain distributed across:

- planning Markdown;
- backend comments;
- copied opcode constants;
- integration-test shell scripts;
- developer memory;
- deployment profiles;

then target behavior has no single first-party typed source.

That creates several risks:

- opcode assignments drift from the target revision;
- target activation is assumed on a network where it is absent;
- backend stack contracts disagree with interpreter behavior;
- explicit/confidential prefixes are interpreted incorrectly;
- byte order differs between compiler and transaction builder;
- resource limits are estimated from the wrong target version;
- a capability is advertised before its proof pattern is implemented;
- deployment evidence tests a different node revision from the emitted bundle;
- one target bundle is relabeled for another network.

The target package closes this gap by defining one exact typed target
capability and identity model.

---

## 3. Target definition versus deployment instance

The package must distinguish target software semantics from one deployment
instance.

### 3.1 Target definition

A target definition describes the reviewed execution and policy model of
one substrate.

Conceptually:

```rust
pub struct ElementsTargetDefinition {
    pub schema_version: TargetSchemaVersion,
    pub upstream: UpstreamSourceIdentity,
    pub network_flavor: ElementsNetworkFlavor,
    pub tapscript: TapscriptDefinition,
    pub opcodes: OpcodeRegistry,
    pub encodings: ElementsEncodingRules,
    pub sighash: SighashCapabilities,
    pub timelocks: TimelockCapabilities,
    pub confidential_transactions: ConfidentialTransactionCapabilities,
    pub consensus_limits: ConsensusLimits,
    pub policy_limits: PolicyLimits,
    pub evidence_requirements: TargetEvidenceRequirementSet,
}
```

> Illustrative API; exact fields and names are not frozen until the Phase-3
> target gate passes.

Examples of target-definition differences include:

- relied-upon substrate semantics;
- tapscript activation set;
- leaf version;
- opcode semantics;
- policy limits;
- enabled deployment features.

### 3.2 Deployment instance

A deployment instance binds the target definition to one network and genesis.

Conceptually:

```rust
pub struct ElementsDeploymentInstance {
    pub target_definition: TargetDefinitionIdentity,
    pub network_id: [u8; 32],
    pub genesis_id: [u8; 32],
    pub chain_name: String,
    pub activation_evidence: ActivationEvidenceBinding,
}
```

> Illustrative API; not frozen.

The exact network and genesis IDs are deployment inputs, not hardcoded protocol
architecture facts.

### 3.3 Combined target

Backend and release code may consume a validated combined value:

```rust
pub struct ElementsTarget {
    pub definition: ElementsTargetDefinition,
    pub deployment: ElementsDeploymentInstance,
}
```

The target definition and deployment instance must have separate identities so
that:

- the same software semantics may be tested on regtest and deployed elsewhere;
- one network instance cannot silently inherit evidence from another;
- substrate-semantics changes are distinguishable from network changes.

---

## 4. Initial target profiles

The first implementation should define a small number of explicit target
profiles.

### 4.1 Development profile

Expected role:

```text
Elements regtest target
```

Purpose:

- target-native script execution;
- backend integration tests;
- transaction construction tests;
- resource measurements;
- deterministic test-chain setup;
- foundational prototypes.

The exact flavor may be:

```text
elementsregtest
```

or:

```text
liquidregtest
```

after source review determines which profile most faithfully matches the
intended production semantics.

Do not treat the two names as interchangeable without comparing their target
configuration.

### 4.2 Production candidate profile

Expected role:

```text
Liquid production-compatible target definition
```

Purpose:

- final target capability identity;
- release compilation;
- deployment evidence;
- resource and policy validation;
- network/genesis binding.

The production candidate remains undefined until the network identity,
activation state, supported test node, and evidence environment are
selected. Per ADR-011, no consensus-implementation source revision enters
that identity.

### 4.3 Profile relationship

The regtest target must not be assumed production-equivalent merely because
the same opcodes execute.

The release evidence must state which claims are:

- identical by source definition;
- configuration-dependent;
- activation-dependent;
- network-policy-dependent;
- functionary/deployment-dependent.

Any equivalence claim between development and production targets requires
typed provenance and tests.

---

## 5. Normative typed inputs

The package is authored from source-reviewed target facts.

### 5.1 Upstream review provenance

Per ADR-011, no upstream source revision is target or protocol identity.
The typed contract is authored from a review of the deployed Liquid
substrate; the material consulted during that review is recorded as
review provenance so the review is reproducible:

```text
repository URL
commit ID or release tag reviewed
source-tree digest, if adopted
```

A mutable branch name such as:

```text
master
main
latest
```

is not sufficient as review provenance, because the review could not be
re-run against it.

### 5.2 Source paths

Every advertised capability should carry provenance to one or more upstream
source locations at the pinned revision.

Examples include:

```text
opcode definitions
interpreter semantics
OP_SUCCESS carve-out
network activation parameters
sighash implementation
transaction serialization
policy limits
functional tests
```

Source paths are provenance, not semantic identity by themselves.

### 5.3 Typed selected target declaration

The package contains Rust constants or pure constructors representing the
reviewed target.

For example:

```rust
pub const INITIAL_ELEMENTS_REGTEST: ElementsTargetDefinition = ...;
```

> Illustrative declaration; no exact target constant is frozen by this plan.

### 5.4 Deployment values

Network/genesis identities and activation evidence enter through typed values.

They must not be read implicitly from:

- a running node selected by environment variable;
- a default RPC endpoint;
- a chain-name string without genesis verification;
- planning Markdown.

### 5.5 No live target probing in semantic APIs

The target-definition library should not query a running node to decide what
capabilities exist.

Live target probing belongs in evidence/integration tooling.

The typed target is selected explicitly, then integration tests verify the
target environment matches it.

---

## 6. Forbidden inputs and behaviors

The package must not consume:

- `plans/reference/elements-tapscript.md` programmatically;
- the old copied opcode Markdown;
- architecture JSON/TOML;
- declassification JSON;
- realization Markdown as parsed input;
- model source;
- compiler output as the source of target semantics;
- backend script bytes as the source of capability claims;
- environment variables;
- current RPC node state in pure target-definition APIs;
- current time;
- filesystem enumeration order.

The package must not:

- define attestation-contract operation policy;
- decide proof alternatives;
- decide disclosure;
- decide transaction layout;
- emit tapscript;
- construct transactions;
- calibrate attestation-contract bounds;
- mark deployment evidence verified merely because a capability is declared;
- advertise a capability whose concrete semantics have not passed the
  substrate review;
- silently substitute unreviewed substrate assumptions.

---

## 7. Dependency direction

The package should be independent of protocol implementation packages where
possible.

Likely dependencies:

```text
small target identity/encoding support
serde, if canonical target publication is required
sha2, for target identity
thiserror
possibly an exact Elements library version for typed transaction constants
```

It must not depend on:

```text
realization
model
tapscript
simplicity
linker
transaction
vectors
release
artifacts
```

The compiler may define the abstract target-capability vocabulary. To avoid a
dependency cycle, one of these approaches should be chosen:

### Option A — shared abstract capability package

Move only the generic capability trait/types into a small target-neutral crate.

### Option B — target adapter in backend/compiler integration

Keep `target-elements` independent, then adapt its typed capabilities to the
compiler interface in a downstream package.

### Option C — compiler-owned trait implemented externally

If Rust coherence and package direction permit, `target-elements` implements a
compiler-owned trait by depending on compiler.

This is less attractive because the target package should ideally remain
usable by transaction and evidence tooling without pulling in compiler
analysis.

Initial preference:

> Keep the target definition independent and provide a narrow adapter below the
> compiler boundary. Do not create a shared crate until concrete type pressure
> justifies it.

The exact Cargo graph should be resolved before implementation.

---

## 8. Public API boundary

### 8.1 Target-definition access

The package should expose typed, immutable target definitions.

Conceptually:

```rust
pub fn elements_regtest_definition() -> &'static ElementsTargetDefinition;
```

```rust
pub fn production_candidate_definition() -> &'static ElementsTargetDefinition;
```

> Illustrative APIs; not frozen.

Avoid APIs returning mutable global target declarations.

### 8.2 Deployment-instance validation

A typed constructor should validate:

- target-definition identity;
- network ID;
- genesis ID;
- expected chain flavor;
- activation evidence binding;
- nonzero IDs;
- supported schema.

Conceptually:

```rust
pub fn bind_deployment(
    definition: &ElementsTargetDefinition,
    deployment: ElementsDeploymentParameters,
) -> Result<ElementsTarget, TargetError>;
```

### 8.3 Capability inspection

Consumers need typed queries such as:

```rust
pub fn capability(
    target: &ElementsTargetDefinition,
    capability: ElementsCapability,
) -> CapabilityDeclaration;
```

The declaration should distinguish:

- supported by source semantics;
- unsupported;
- conditionally supported;
- available only in tapscript;
- available only under one activation;
- available but lacking an approved attestation-contract proof pattern;
- target dependency evidence required.

### 8.4 Opcode inspection

The tapscript backend needs exact opcode declarations and stack contracts.

Conceptually:

```rust
pub struct OpcodeDefinition {
    pub code: u8,
    pub name: OpcodeName,
    pub execution_domain: ExecutionDomain,
    pub stack_contract: StackContract,
    pub failure_modes: BTreeSet<TargetFailureMode>,
    pub resource_cost: ResourceCost,
    pub provenance: SourceProvenance,
}
```

> Illustrative API; not frozen.

The target package should not expose only raw opcode bytes without semantic
metadata.

### 8.5 Resource-limit access

Consumers need typed consensus and policy limits.

Conceptually:

```rust
pub struct ConsensusLimits {
    pub max_stack_and_altstack_elements: usize,
    pub max_stack_element_bytes: usize,
    pub tapscript_crypto_budget: CryptoBudgetRule,
    pub transaction_weight_limit: u64,
}
```

```rust
pub struct PolicyLimits {
    pub max_initial_push_bytes: Option<usize>,
    pub standardness: StandardnessRules,
}
```

> Illustrative fields; exact limits must come from the pinned target.

Do not copy assumed values into this plan and later treat them as typed facts.

### 8.6 No generic string capability API

Avoid:

```rust
target.supports("op_tweakverify")
```

Prefer stable typed capability IDs.

Strings may be used for canonical publication and diagnostics.

---

## 9. Target identity

### 9.1 Domain-separated identity

The target definition requires a canonical domain-separated identity.

It should bind:

- target schema version;
- upstream repository identity;
- exact commit/release;
- target network flavor;
- tapscript activation rules;
- leaf version;
- opcode registry used by the project;
- relied-upon opcode semantics;
- encoding rules;
- sighash rules;
- timelock rules;
- CT capabilities;
- consensus limits;
- policy limits;
- evidence-requirement registry.

The deployment-instance identity separately binds:

- target-definition identity;
- network ID;
- genesis ID;
- activation evidence;
- chain-specific parameters relevant to behavior.

### 9.2 Identity exclusions

The target identity should not include:

- local source checkout path;
- node RPC URL;
- host operating system;
- test execution time;
- process ID;
- local build directory;
- credentials;
- comments or planning prose;
- source formatting where semantics are unchanged, unless a full source commit
  identity is already the binding.

### 9.3 Typed contract as primary identity

The target's canonical identity binds the typed compatibility contract —
opcode discriminants, stack contracts, encodings, execution domain,
resource interfaces — selected by this package. The upstream commit
consulted during review is recorded beside it as review provenance, not
folded into the identity (ADR-011): a compatible substrate does not
change identity merely because a node implementation revision moved.

If the typed interpretation changes while the source commit remains fixed,
the target identity must change.

### 9.4 Algorithm migration

Before publishing a stable target hash, document:

- algorithm identifier;
- canonical projection;
- domain separator;
- ordering;
- included/excluded fields;
- error behavior;
- migration policy;
- mutation tests.

No target identity algorithm should be silently redefined after publication.

---

## 10. Capability model

### 10.1 Capability declaration levels

A capability must not be represented as only `bool`.

Use a richer status model, conceptually:

```rust
pub enum CapabilityStatus {
    Unsupported,
    SourceSupported,
    ConditionallySupported {
        conditions: Vec<CapabilityCondition>,
    },
    ProofPatternAvailable {
        pattern: ProofPatternId,
    },
}
```

> Illustrative vocabulary; not frozen.

Important distinction:

```text
source-supported target primitive
```

does not necessarily mean:

```text
approved complete attestation-contract proof pattern exists
```

For example, low-level EC operations may exist without a complete authenticated
value-opening construction.

### 10.2 Capability categories

The initial target model should cover at least:

#### Transaction inspection

- current input index;
- input outpoint;
- input asset;
- input value;
- input scriptPubKey/program;
- input sequence;
- input issuance;
- output asset;
- output value;
- output nonce;
- output scriptPubKey/program;
- transaction version;
- locktime;
- input count;
- output count;
- transaction weight.

#### Arithmetic

- fixed-width signed 64-bit addition;
- subtraction;
- multiplication;
- division/remainder;
- negation;
- comparisons;
- conversion between script numbers and fixed-width values;
- explicit overflow signaling/failure behavior;
- exact operand widths.

#### Hashing and bytes

- SHA-256;
- streaming SHA-256 where present;
- byte concatenation;
- bitwise operations where relied upon;
- stack element size constraints.

#### Signatures and crypto

- tapscript signature semantics;
- signature-from-stack semantics if selected;
- output commitment through selected sighash;
- scalar multiplication verification;
- tweak verification;
- target crypto budget.

#### Timelocks

- relative timelock semantics;
- sequence/version preconditions;
- locktime interpretation;
- target activation constraints.

#### Confidential transactions

- explicit value encoding;
- confidential value encoding;
- explicit asset encoding;
- confidential asset encoding;
- CT value conservation;
- commitment equality mechanisms available to script/backend;
- public opening support if fully implemented;
- rangeproof construction/validation assumptions;
- issuance/reissuance behavior.

#### Script execution

- tapscript-only opcode availability;
- OP_SUCCESS carve-out;
- leaf version;
- script and stack limits;
- initial stack policy;
- control-block behavior;
- standardness constraints.

### 10.3 Conditional capability examples

A capability may require conditions such as:

```text
tapscript script-path execution
specific leaf version
target activation complete
transaction version supports relative locktime
input value uses explicit prefix
output asset uses explicit prefix
nonempty signature
crypto budget available
```

Conditions must be typed and source-provenanced.

### 10.4 Capability requirements versus evidence requirements

The target definition may state:

```text
capability source-supported
deployment evidence required
```

The deployment profile later states whether evidence is verified.

Do not store mutable verification status in the static target definition.

---

## 11. Opcode semantics model

### 11.1 Exact stack contracts

For every opcode used by the backend, define:

- operand count;
- operand order;
- operand encoding;
- result count;
- result order;
- success signaling;
- overflow/failure behavior;
- malformed operand behavior;
- resource cost;
- execution domain;
- source provenance.

### 11.2 No prose-only semantics

Backend stack scheduling must consume typed stack contracts or target-specific
typed builder functions.

It must not rely only on comments copied from upstream documentation.

### 11.3 Arithmetic success behavior

Where an arithmetic opcode leaves operands intact on overflow and pushes a
failure flag, the typed contract must represent that exact behavior.

Do not normalize all arithmetic failures into one assumed stack effect.

This is essential for stack-effect validation.

### 11.4 Multi-result behavior

Division/remainder and introspection tuple outputs require exact ordering.

For example, if one opcode pushes:

```text
remainder
quotient
success
```

or another target-defined order, the typed contract must match the pinned
interpreter exactly.

Do not infer result order from intuitive arithmetic notation.

### 11.5 Index failure

Indexed introspection operations must represent:

- valid index domain;
- negative index behavior;
- out-of-range behavior;
- target error class;
- whether failure aborts script or pushes a value.

### 11.6 Script-domain restriction

Each project-used opcode must identify where it is valid:

```text
legacy script
segwit v0
tapscript
specific leaf version
```

The backend must reject emission into an unsupported execution domain.

### 11.7 Crypto budget

Each crypto opcode used must identify its target cost under the exact budget
rule.

The target package describes the rule. Backend resource analysis computes
consumption per concrete script/witness.

---

## 12. Encoding rules

### 12.1 Asset and value prefixes

The target package must encode exact prefix classes for:

- explicit asset;
- confidential asset variants;
- explicit value;
- confidential value variants;
- null value conventions where applicable;
- output nonce forms.

Use typed enums rather than raw magic bytes in backend code.

Conceptually:

```rust
pub enum AssetEncodingClass {
    Explicit,
    ConfidentialEven,
    ConfidentialOdd,
}
```

```rust
pub enum ValueEncodingClass {
    Explicit,
    ConfidentialEven,
    ConfidentialOdd,
    NullExplicitZero,
}
```

> Illustrative vocabulary; exact classes depend on pinned source.

### 12.2 Byte order

The package must explicitly define byte order for every inspected field relied
upon by the backend:

- explicit value;
- asset ID;
- transaction ID;
- outpoint index;
- sequence;
- transaction version;
- locktime;
- issuance entropy;
- issuance blinding nonce;
- commitment bytes;
- scalars;
- hashes.

Avoid one global “Elements uses little endian” rule. Different fields may use
different conventions.

### 12.3 Script-number conversion

Define:

- minimal script-number rules;
- fixed-width conversion behavior;
- signed versus unsigned interpretation;
- range limits;
- malformed encoding failure.

### 12.4 Canonical serialization

The target package may expose target encoding helpers, but it must not become a
general transaction builder.

Helpers should be:

- typed;
- deterministic;
- exact to the pinned source semantics;
- unit tested with canonical vectors.

### 12.5 Unknown encodings

Unknown or future prefix/encoding classes must fail closed unless the target
schema explicitly models forward-compatible handling.

The backend must not treat an unknown commitment prefix as an inert explicit
value.

---

## 13. Sighash model

### 13.1 Purpose

Owner, refund-key, operator, and sponsor authorization in the abstract model
assumes commitment to the complete required economic output set.

The target package must describe which concrete sighash modes and script
semantics can support that relation.

### 13.2 Typed sighash capabilities

The target model should distinguish dimensions such as:

- commits all outputs;
- permits input-set extension;
- commits current input;
- commits all inputs;
- commits issuance data;
- commits annex where applicable;
- script-path semantics;
- transaction version/locktime commitment;
- target-specific message construction.

Do not collapse sighash support into one name string.

### 13.3 Initial target policy

The initial backend is expected to require output-committing signatures for
owner/operator/sponsor paths.

Whether an input-extension mode such as an ANYONECANPAY-like policy is allowed
is a separate policy decision and must not weaken output commitment.

The exact selected flags are not frozen by this plan.

### 13.4 Deployment evidence

The release must include target tests demonstrating:

- selected signature validates the intended transaction;
- changing a protected output invalidates the signature;
- changing recipient invalidates the signature;
- appending/modifying burn records behaves according to the intended output
  commitment;
- permitted input extension, if enabled, has the documented behavior;
- invalid/malformed signatures fail;
- empty-signature behavior matches the target.

### 13.5 No private keys in target definitions

The target package defines signature semantics, not deployment keys.

Keys enter through typed deployment and transaction inputs.

---

## 14. Timelock model

### 14.1 Relative cadence only

The target package provides relative-timelock semantics used by the cycle
cadence band.

It does not decide that maturity uses a timelock. The realization explicitly
uses committed cycle arithmetic for maturity.

### 14.2 Required semantics

The target model must specify:

- sequence interpretation;
- transaction-version precondition;
- relative block versus time mode;
- disable/type flags;
- minimum-age comparison;
- failure behavior;
- activation context.

### 14.3 Fail-closed target behavior

If a transaction fails target timelock prerequisites, it must not gain access to
a weaker cycle branch.

The backend maps the three semantic cadence regimes to target programs. The
target package supplies the exact substrate behavior.

### 14.4 Deployment evidence

The target evidence must test:

- below minimum;
- at minimum;
- inside operator-only band;
- one below maximum;
- at maximum;
- delayed permissionless branch;
- invalid transaction version;
- invalid sequence form;
- wrong clock type.

---

## 15. Confidential transaction model

### 15.1 Separation of asset and value axes

The target package must model:

- asset encoding;
- value encoding;
- nonce;
- rangeproof/surjection-proof role;
- CT conservation;
- issuance/reissuance.

Do not represent one generic `Confidential` bit.

### 15.2 Initial protocol policy handoff

Under D005:

- closed protocol asset identity is explicit;
- values may use supported explicit/private/public-committed modes;
- sponsor asset is explicit L-BTC in the initial profile;
- sponsor value may remain confidential;
- public workflow values require explicit or authenticated public opening;
- confidential asset identity is not an approved closed-asset proof.

The target package reports capabilities. The compiler/backend policy enforces
the attestation-contract choice.

### 15.3 CT conservation capability

The target declaration must state the exact conservation relation the backend
may rely upon.

Deployment evidence must demonstrate:

- valid confidential transfer;
- invalid value imbalance rejection;
- mixed explicit/confidential behavior;
- issuance interaction;
- fee handling;
- correct asset grouping;
- relevant rangeproof behavior.

### 15.4 Commitment equality

If target programs compare commitment bytes or otherwise prove equality, define:

- commitment encoding;
- equality operation;
- asset identity preconditions;
- blinding implications;
- source provenance;
- target tests.

### 15.5 Authenticated opening

Do not advertise a complete value-opening capability until the project has:

- one exact proof construction;
- canonical opening format;
- script/backend implementation;
- malformed-opening vectors;
- range/domain checks;
- public/secret availability model;
- resource formula;
- target-native tests.

Low-level crypto opcodes alone do not establish this capability.

### 15.6 Transaction-construction support

The target package may describe target-level proof requirements but should not
own wallet blinding algorithms.

The `transaction` package owns concrete construction and rangeproof generation
using target definitions.

---

## 16. Issuance and reissuance

### 16.1 Required target facts

The target package must model:

- asset issuance fields;
- asset entropy;
- blinding nonce;
- issued amount encoding;
- inflation/reissuance token amount;
- asset ID derivation;
- reissuance-token semantics;
- issuance introspection;
- outpoint issuance flags;
- null issuance representation.

### 16.2 Protocol handoff

attestation uses native issuance/reissuance for:

```text
U
ENT
DIST_CTL
```

with separate authority assets/roots.

The exact mapping between architecture authorities and Elements issuance or
reissuance-token mechanisms belongs to compiler/backend/genesis design, not
this target package.

The target package supplies the substrate semantics needed to evaluate that
mapping.

### 16.3 Evidence

Target evidence must test:

- correct issuance inspection;
- wrong asset/authority rejection by emitted pattern;
- issued amount visibility/commitment behavior;
- destination exhaustion vectors;
- reissuance-token semantics where used;
- null issuance behavior;
- issuance outpoint flags.

---

## 17. Script and transaction resource model

### 17.1 Consensus and policy separation

Maintain separate typed structures for:

- consensus validity limits;
- standardness/policy limits;
- deployment-selected stricter limits.

A transaction may be consensus-valid but nonstandard under relay policy.

The release profile should state which class is required for deployment.

### 17.2 Required resource dimensions

At minimum model:

- transaction weight;
- witness bytes;
- script bytes;
- control data;
- initial stack element count;
- stack plus altstack count;
- stack element size;
- executed opcode/resource cost where target defines one;
- crypto operation count;
- per-input crypto budget;
- target policy acceptance;
- package-relay constraints relevant to CPFP.

### 17.3 Crypto budget

Represent the exact target formula for:

- initial budget;
- dependence on serialized input witness bytes;
- cost per crypto opcode;
- empty-signature behavior where applicable;
- failure condition.

Backend resource analysis applies the formula to concrete ABI witnesses.

### 17.4 Initial push policy

The target model must source-verify:

- which witness elements are subject to any initial-push policy limit;
- whether tapleaf script and control data are exempt or treated separately;
- exact policy failure behavior.

Do not perpetuate an uncertain copied statement into typed target facts.

### 17.5 Script size and opcode limits

The target definition must distinguish:

- legacy limits;
- segwit v0 limits;
- tapscript limits;
- implicit block/transaction weight bounds;
- policy constraints.

### 17.6 Package relay

Package relay is not only a script capability.

The target package should declare the target/deployment claim required by the
maturity-cycle CPFP anchor, while deployment evidence verifies actual node
policy and package selection behavior.

### 17.7 Calibration handoff

The target package provides limits and measurement rules.

The linker/transaction/calibration runner constructs and measures complete
attestation-contract transactions.

The target package does not choose protocol bounds.

---

## 18. Source provenance

### 18.1 Provenance type

Every relied-upon target declaration should carry typed provenance.

Conceptually:

```rust
pub struct SourceProvenance {
    pub upstream: UpstreamSourceIdentity,
    pub path: &'static str,
    pub symbol_or_section: &'static str,
    pub claim_kind: SourceClaimKind,
}
```

> Illustrative API; not frozen.

### 18.2 Provenance is review metadata

Source paths help reviewers reproduce the target declaration.

They should not be the sole target identity because paths can move while a
source commit remains exact.

### 18.3 Functional-test provenance

Where upstream has a relevant functional test, record it separately from
interpreter source.

The project must distinguish:

- source code says what should happen;
- upstream tests cover some behavior;
- first-party deployment tests cover the exact project claim.

### 18.4 Known gaps

The typed target may include known evidence gaps, such as:

- capability source-supported but upstream test coverage incomplete;
- source behavior reviewed but first-party integration test pending;
- policy behavior environment-dependent;
- production activation evidence pending.

Known gaps are not support status. They are inputs to the deployment evidence
requirements.

---

## 19. Evidence requirement registry

### 19.1 Purpose

The target definition should produce one typed set of target claims that
deployment release must verify.

Conceptually:

```rust
pub struct TargetEvidenceRequirement {
    pub id: TargetEvidenceRequirementId,
    pub capability: ElementsCapability,
    pub claim: TargetClaim,
    pub required_tool_class: EvidenceToolClass,
    pub source_provenance: Vec<SourceProvenance>,
}
```

> Illustrative API; not frozen.

### 19.2 Initial requirement classes

Expected classes include:

- native asset conservation;
- issuance/reissuance semantics;
- issuance introspection;
- explicit asset/value introspection;
- confidential value conservation;
- commitment equality;
- authenticated opening, if used;
- sighash output commitment;
- relative timelock behavior;
- script-path/leaf-version enforcement;
- opcode stack behavior;
- crypto budget;
- constructor/tweak verification;
- unspendable-output exclusion;
- package relay;
- transaction policy/standardness;
- L-BTC settlement dependency where target/deployment relevant.

### 19.3 Static requirement versus runtime status

The target definition states:

```text
this evidence is required
```

The deployment profile states:

```text
this evidence is present and verified
```

Do not mutate target constants to reflect one CI run.

### 19.4 Optional capabilities

An optional capability, such as authenticated opening before the first backend
uses it, may have:

```text
source-supported or research candidate
verification not required by current deployment
```

Once selected by a proof plan, its evidence becomes required for that
deployment.

---

## 20. Validation

### 20.1 Definition validation

Validate:

- supported nonzero target schema;
- typed contract schema/version identity;
- nonblank review-provenance references;
- unique opcode codes;
- unique opcode names;
- no opcode conflict;
- execution domain declared;
- stack contracts complete;
- failure modes complete for relied-upon opcodes;
- resource costs declared;
- encoding classes unique;
- capability conditions resolve;
- evidence requirements resolve;
- no protocol operation IDs embedded as target semantics.

### 20.2 Deployment-instance validation

Validate:

- target-definition identity matches;
- network and genesis IDs nonzero;
- chain flavor supported;
- activation evidence present where required;
- no production target uses a development-only assumption;
- target instance compatible with backend configuration.

### 20.3 Capability closure

Validate that advertised high-level capabilities have the required lower-level
primitives.

For example, a claimed proof pattern may require:

```text
input value inspection
output value inspection
fixed-width arithmetic
canonical explicit prefix
```

If one dependency is absent, the high-level capability must not be advertised
as complete.

### 20.4 Provenance closure

Every relied-upon capability must have source provenance.

A capability without provenance may exist as:

```text
research candidate
```

but cannot enter a release target definition.

### 20.5 Evidence closure

Every release-required capability must map to at least one evidence
requirement.

### 20.6 No protocol leakage

Validation or review should reject target declarations that contain:

- attestation-contract operation names as target semantics;
- protocol bounds;
- protocol recipient rules;
- protocol asset IDs;
- architecture hashes;
- compiler proof decisions.

A target may have generic names such as:

```text
value introspection
relative timelock
```

The compiler maps protocol requirements onto them.

---

## 21. Error model

Errors should be typed and deterministic.

Candidate classes include:

```rust
pub enum TargetError {
    UnsupportedTargetSchema,
    MissingUpstreamIdentity,
    InvalidUpstreamRevision,
    DuplicateOpcodeCode(u8),
    DuplicateOpcodeName(OpcodeName),
    MissingOpcodeSemantics(OpcodeName),
    MissingStackContract(OpcodeName),
    MissingResourceCost(OpcodeName),
    InvalidExecutionDomain(OpcodeName),

    UnknownEncodingPrefix(u8),
    ConflictingEncodingClass,
    MissingByteOrder(FieldKind),

    MissingCapabilityProvenance(ElementsCapability),
    IncompleteCapability(ElementsCapability),
    ConflictingCapabilityDeclaration(ElementsCapability),

    MissingEvidenceRequirement(ElementsCapability),
    InvalidEvidenceRequirement(TargetEvidenceRequirementId),

    DeploymentDefinitionMismatch,
    ZeroNetworkId,
    ZeroGenesisId,
    UnsupportedNetworkFlavor,
    MissingActivationEvidence,
    IncompatibleActivationState,

    ProtocolPolicyLeak,
}
```

> Illustrative vocabulary; not frozen.

Errors must not include:

- RPC credentials;
- raw private URLs;
- private keys;
- environment secrets.

---

## 22. Testing strategy

### 22.1 Pure unit tests

Test:

- opcode registry completeness;
- no duplicate codes/names;
- typed stack contracts;
- arithmetic success/failure shapes;
- introspection result order;
- encoding prefix classification;
- byte-order helpers;
- capability dependency closure;
- provenance closure;
- target identity stability;
- deployment binding validation;
- evidence requirement census;
- deterministic canonical rendering.

### 22.2 Source-conformance tests

Where feasible, add first-party tests that compare typed target constants to
the pinned upstream source.

Possible mechanisms:

1. vendor a small canonical source-derived fixture;
2. run a source-check script against a caller-supplied exact checkout;
3. compile an integration helper against the pinned Elements library;
4. compare expected opcode/activation constants through node RPC or binary
   behavior.

The normal Rust unit suite must not require network access or silently clone
upstream source.

Source-conformance checks may be a separate explicit CI lane.

### 22.3 Target-native functional tests

Run against the exact pinned Elements regtest binary.

Cover every capability actually used by the backend.

Tests should include:

- correct opcode behavior;
- malformed operands;
- index boundaries;
- explicit/confidential forms;
- arithmetic overflow;
- crypto failures;
- timelock boundaries;
- sighash mutations;
- policy acceptance;
- transaction resource measurement.

### 22.4 Negative activation tests

Where practical, verify:

- opcode unavailable outside tapscript;
- wrong leaf version fails;
- unsupported execution context fails;
- inactive deployment does not accidentally pass.

### 22.5 Identity mismatch tests

Reject:

- report for wrong source commit;
- regtest evidence attached to production target without accepted equivalence;
- network/genesis mismatch;
- backend plan bound to a different target definition;
- changed capability registry under old identity.

### 22.6 Public API tests

An external integration test should prove a downstream backend can:

- obtain a target definition;
- inspect typed capabilities;
- inspect opcode stack contracts;
- inspect encoding rules;
- inspect resource limits;
- inspect evidence requirements;
- bind a deployment instance;
- do so without protocol/model internals.

### 22.7 Mutation tests

Mutate target declarations and require validation failure:

- duplicate opcode;
- missing source provenance;
- wrong execution domain;
- conflicting prefix;
- missing evidence requirement;
- unsupported high-level capability with missing primitive;
- zero network/genesis;
- invalid activation binding;
- protocol-specific field leakage.

---

## 23. Integration-test environment

### 23.1 Exact binary identity

Target-native reports must record:

- Elements binary version;
- source revision;
- build configuration relevant to semantics;
- network flavor;
- genesis ID;
- activation state;
- test tool version.

### 23.2 Hermetic chain setup

The regtest harness should:

- create a temporary data directory;
- use caller-controlled ports or isolated allocation;
- use deterministic chain setup where possible;
- avoid ambient user node configuration;
- avoid production credentials;
- clean up on success/failure;
- capture machine-readable logs under ADR-010-compatible wrappers.

### 23.3 No default RPC secrets in reports

RPC credentials and cookie contents must never enter canonical reports.

Reports may record redacted endpoint identity or process role, not credentials.

### 23.4 Separate test lanes

Potential lanes:

```text
target unit tests
source-conformance checks
target-native functional tests
backend integration tests
production-equivalence/activation evidence
```

Not every local edit must run every node test, but release must run every
required lane.

---

## 24. Determinism

### 24.1 Target definitions are pure constants

For one target definition, target declarations are deterministic typed values.

No live node response alters the target definition.

### 24.2 Canonical ordering

Use stable ordering for:

- opcodes by numeric code;
- capabilities by typed ID;
- evidence requirements by typed ID;
- provenance by source path and symbol;
- encoding classes by prefix;
- failure modes by typed ID.

### 24.3 Deterministic identity tests

Require:

- repeated target-definition identity equality;
- source declaration order changes do not alter canonical identity where order
  is not semantic;
- changed opcode semantics change target identity;
- changed relied-upon substrate facts change target identity;
- changed deployment network changes deployment-instance identity;
- changed evidence status does not mutate static target definition.

### 24.4 No timestamps

Target publications and canonical evidence-requirement manifests contain no
ambient timestamp.

A release record may contain an externally supplied release date outside the
target identity.

---

## 25. Generated artifacts

### 25.1 Phase-3 default

The package may initially provide typed values only.

A canonical target publication becomes useful when:

- backend reports need an external target identity document;
- independent implementers need capability details;
- release needs a reviewable target manifest.

### 25.2 Candidate publication

Potential artifact:

```text
target-elements.json
```

It may include:

- target schema;
- upstream source identity;
- network flavor;
- opcode registry subset;
- encoding rules;
- capability declarations;
- resource limits;
- evidence requirements;
- target hash.

The name is illustrative.

### 25.3 Artifact law

If committed, it must have:

- one typed source;
- one generator;
- one non-writing checker;
- canonical ordering;
- unknown-field rejection;
- deterministic bytes;
- identity verification;
- no reverse semantic dependency.

The backend consumes typed target values, not the JSON.

### 25.4 Source license/provenance publication

If any upstream source text, tables, or fixtures are copied, retain:

- exact attribution;
- upstream license;
- source revision;
- path;
- modification note.

Prefer concise typed summaries and links over large source copies.

---

## 26. Dependency and unsafe-code policy

The package inherits ADR-011.

Requirements:

- Rust edition 2024;
- workspace MSRV;
- workspace lints;
- `unsafe_code = "deny"`;
- `--locked`;
- permissive dependencies;
- deterministic output;
- no network access in library code;
- no build-time source download;
- no hidden node probing.

If a later FFI test tool requires unsafe code, place it in a separate narrowly
scoped crate or module with explicit safety documentation. Do not weaken the
target-definition library.

Potential dependencies must be selected deliberately:

- a Rust Elements library may provide typed constants and transaction types;
- its exact version and relation to the pinned node source must be documented;
- the dependency must not silently become the target authority when node source
  differs.

---

## 27. Non-goals

The package does not:

- define attestation-contract protocol semantics;
- derive realization relations;
- choose proof alternatives;
- emit tapscript;
- schedule stack operations;
- construct taptrees;
- resolve protocol object constructors;
- build transactions;
- generate rangeproofs;
- calibrate protocol bounds;
- execute model operations;
- run an indexer;
- assemble a deployment profile;
- replace the exact target-native test environment;
- prove target source correct;
- prove cryptographic assumptions;
- guarantee production activation from a regtest result;
- support every Elements opcode;
- model irrelevant upstream features.

The target package should initially model only the target surface the project
uses or is actively evaluating.

---

## 28. Phase-3 milestones

### E1.1 — Substrate review

Deliver:

- upstream repository reviewed;
- reviewed revision recorded as review provenance (ADR-011);
- source provenance map;
- network flavor selection;
- license review.

### E1.2 — Crate skeleton

Deliver:

- workspace package;
- documentation;
- typed identity/error skeleton;
- no protocol dependency.

### E1.3 — Opcode registry

Deliver typed definitions for project-relevant:

- introspection;
- arithmetic;
- conversion;
- hashing/byte operations;
- signatures/crypto.

### E1.4 — Encoding rules

Deliver:

- asset/value/nonce classes;
- byte-order rules;
- script-number/fixed-width conversion;
- canonical helpers.

### E1.5 — Sighash and timelocks

Deliver typed target semantics and conditions.

### E1.6 — CT and issuance capabilities

Deliver:

- CT conservation;
- commitment-equality capability;
- issuance/reissuance semantics;
- optional opening status.

### E1.7 — Resource model

Deliver consensus/policy limits and crypto budget.

### E1.8 — Evidence registry

Deliver typed deployment requirement census and provenance.

### E1.9 — Development deployment binding

Deliver validated regtest target instance.

### E1.10 — Target-native test harness

Deliver hermetic node tests for every capability used by the first backend
prototype.

### E1.11 — Target identity

Deliver canonical target-definition and deployment-instance identities with
mutation tests.

### E1.12 — Phase gate

Run complete source-conformance and target-native verification.

---

## 29. Phase-3 target exit criteria

The target package is ready for production backend use only when:

- [ ] `packages/target-elements` is a workspace member;
- [ ] the reviewed upstream repository and revision are recorded as review
      provenance — not protocol or release identity (ADR-011);
- [ ] upstream license/provenance is recorded;
- [ ] target schema is explicit;
- [ ] development network flavor is selected explicitly;
- [ ] production candidate profile is distinct from development profile;
- [ ] every used opcode has typed code, execution domain, stack contract,
      failure modes, resource cost, and provenance;
- [ ] opcode codes/names are unique;
- [ ] OP_SUCCESS carve-out and tapscript activation assumptions carry
      review provenance;
- [ ] asset/value/nonce encoding classes are typed;
- [ ] byte-order rules are explicit and tested;
- [ ] sighash capabilities are typed;
- [ ] output commitment behavior is target-tested;
- [ ] relative-timelock semantics are typed and target-tested;
- [ ] CT conservation capability is reviewed and target-tested;
- [ ] commitment equality is defined only to the extent actually supported;
- [ ] authenticated opening is not advertised without a complete proof pattern;
- [ ] issuance/reissuance semantics are typed and tested;
- [ ] consensus and policy resource limits are separated;
- [ ] crypto budget is modeled exactly;
- [ ] initial-push policy uncertainty is resolved from source;
- [ ] evidence requirements are complete for selected capabilities;
- [ ] target definition validates;
- [ ] deployment-instance binding validates;
- [ ] target and deployment identities are deterministic;
- [ ] a hermetic regtest harness verifies every capability used by the first
      backend;
- [ ] no protocol operation policy appears in the target crate;
- [ ] no Markdown is consumed as target input;
- [ ] no network access occurs in pure library APIs;
- [ ] debug/release workspace tests pass;
- [ ] Clippy with `-D warnings` passes;
- [ ] the checkout remains clean.

---

## 30. Open questions

### 30.1 Reviewed upstream revision

The upstream revision consulted by the substrate review must be selected
explicitly (it is review provenance, not target identity; ADR-011).

Criteria include:

- required opcode implementation;
- activation behavior;
- security fixes;
- compatibility with intended production deployment;
- reproducible build availability;
- functional test support.

### 30.2 Development network flavor

Choose between available regtest profiles based on exact configuration
equivalence to the intended production capabilities.

### 30.3 Rust Elements library

Decide whether the package should depend on a Rust Elements library for:

- transaction types;
- consensus encoding;
- asset/value constants;
- sighash types.

The node source remains the target behavior pin. The Rust library version must
be compatible and separately pinned.

### 30.4 Capability trait ownership

Resolve the compiler/target adapter without introducing package cycles.

### 30.5 Policy versus consensus target identity

Decide whether:

- one target identity binds both;
- consensus and policy receive separate identities;
- one deployment target embeds both sub-identities.

Separate sub-identities may better support policy changes that leave consensus
unchanged.

### 30.6 Initial-push limit scope

Resolve exactly which witness/script/control elements are subject to target
standardness limits.

### 30.7 Sighash mode

The target package describes supported modes. The compiler/backend deployment
policy must later choose the exact profile.

### 30.8 Public opening capability

Remain unsupported/incomplete until
[`../research/public-declassification.md`](../research/public-declassification.md)
produces a complete target pattern.

### 30.9 Resource cost model

Determine whether target resource accounting beyond weight and crypto budget
needs an additional project-defined opcode-cost metric for deployment
calibration.

If project-defined, keep it separate from consensus/policy target identity.

### 30.10 Production activation evidence

Define how release proves that the selected production target capabilities are
active on the intended network and not merely present in source.

---

## 31. Risks

### 31.1 Source drift

The typed target may lag the pinned or deployed source.

Mitigation:

- exact revision;
- source-conformance tests;
- target-native tests;
- target identity;
- release mismatch checks.

### 31.2 Partial semantic transcription

An opcode code may be correct while its stack/failure semantics are wrong.

Mitigation:

- typed stack contracts;
- malformed vectors;
- upstream and first-party tests;
- backend stack-effect validation.

### 31.3 Regtest/production mismatch

A capability may be always active on regtest but differently deployed or
configured in production.

Mitigation:

- separate target/deployment identities;
- production activation evidence;
- explicit equivalence claims;
- no regtest-only release evidence reuse.

### 31.4 Policy instability

Relay/mining policy can change independently of consensus.

Mitigation:

- consensus/policy separation;
- exact node/version pin;
- deployment tests;
- package-relay evidence;
- target-specific release identity.

### 31.5 Over-advertised capability

Low-level primitives may be mistaken for a complete proof method.

Mitigation:

- source-supported versus proof-pattern-available status;
- capability dependency closure;
- compiler/backend proof-plan tests;
- evidence requirements.

### 31.6 Library/node divergence

A Rust Elements dependency may encode behavior differently from the pinned
node.

Mitigation:

- node source remains target pin;
- cross-library vectors;
- canonical transaction bytes;
- exact version documentation.

### 31.7 Target package accumulating protocol policy

Convenience mappings may introduce operation-specific behavior.

Mitigation:

- no architecture/model dependency;
- generic capability names;
- review/lints;
- backend adapter owns protocol mapping.

### 31.8 Secret leakage in integration tooling

RPC URLs, cookies, keys, or raw commands may enter logs.

Mitigation:

- ADR-010 wrappers;
- no raw argv logging;
- redaction;
- temporary credentials;
- canonical reports exclude secrets.

### 31.9 Identity freeze before completeness

A target hash may be published before all relied-upon semantics are included.

Mitigation:

- internal schema first;
- complete Phase-3 gate;
- explicit identity projection decision;
- mutation tests;
- migration policy.

---

## 32. Definition of done

The Elements target package plan is fulfilled for the first backend when the
repository contains one exact, reviewed, deterministic typed Elements
target definition and one validated development deployment instance; every
target primitive used by backend prototypes has exact stack, encoding,
failure, activation, and resource semantics with review provenance; compiler
capability matching can consume those facts without parsing Markdown; a
hermetic target-native test harness verifies the selected claims; and release
tooling can bind target, network, genesis, capability, and evidence identities
without conflating source support with completed deployment verification.

---

## 33. One-line package contract

> `target-elements` turns the reviewed Liquid tapscript substrate and one
> deployment flavor into a deterministic typed registry of tapscript execution,
> introspection, encoding, sighash, timelock, confidential-transaction,
> issuance, consensus-limit, policy-limit, and evidence-requirement facts used
> by compiler adapters, the tapscript backend, transaction construction,
> vectors, and release—without defining attestation-contract semantics or reading
> planning Markdown.
