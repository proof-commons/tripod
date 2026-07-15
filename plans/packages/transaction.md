# Transaction and Witness ABI Package Plan

> **Status:** PLANNED
> **Planned source directory:** `packages/transaction`
> **Planned Cargo package:** `tripod-transaction`
> **Planned Rust library name:** `transaction`
> **Implementation phase:** Phase 4 onward — first canonical `compact-ash`
> transaction and witness ABI
> **Depends on packages:** `tripod-linker`,
> `tripod-target-elements`, and target-specific linked artifact types;
> it may depend on `tripod-realization` for typed semantic expression
> evaluation and provenance
> **Depends on decisions:**
> [D001](../decisions/001-typed-rust-is-normative.md),
> [D002](../decisions/002-target-independent-realization-layer.md),
> [D003](../decisions/003-multiple-backends-tapscript-first.md),
> [D004](../decisions/004-translation-validation-over-compiler-trust.md),
> [D005](../decisions/005-value-parametric-asset-rigid.md),
> [D006](../decisions/006-canonical-transaction-layout-abi.md)
> **Open research dependencies:**
> [`../research/state-object-constructor.md`](../research/state-object-constructor.md),
> [`../research/public-declassification.md`](../research/public-declassification.md),
> and [`../research/settlement-layout.md`](../research/settlement-layout.md)
> determine later constructor, representation, and operation ABI details
> **Authority:** Concrete target transaction and witness construction beneath
> the linked bundle and typed realization
> **Machine-consumed planning document:** no

---

## 1. Purpose

The `transaction` package will provide the first-class typed transaction and
witness ABI for linked attestation-contract backend bundles.

It consumes:

- one exact typed linked bundle;
- the linked operation layout;
- the linked witness requirements;
- typed operation requests;
- typed public chain/input views;
- typed owner/operator/sponsor witness material supplied by authorized callers;
- explicit construction randomness where confidential transactions require it;
- the exact typed Elements target.

It produces:

- validated typed operation-construction plans;
- canonical unsigned Elements transactions;
- target-specific blinding and proof plans;
- signing requests;
- finalized target transactions and witnesses;
- concrete constructor instances;
- concrete metadata encodings;
- operation-specific construction reports;
- deterministic canonical transaction vectors;
- valid worst-case transactions for resource calibration;
- one canonical transaction/witness ABI derived from the linked bundle.

The package owns:

- final transaction ABI derivation from linked artifacts;
- typed operation request APIs;
- canonical input/output-family ordering;
- family range materialization;
- metadata encoding and constructor instantiation;
- target program/leaf selection;
- control-path materialization;
- target transaction assembly;
- target signing-message construction;
- witness-stack materialization;
- owner/operator/sponsor signing roles;
- confidential value/blinding construction where supported;
- public-opening materialization where supported;
- canonical data-output construction;
- client-policy request construction;
- worst-case valid transaction generation;
- transaction/ABI identity;
- transaction-construction evidence.

It does not own:

- architecture semantics;
- realization relations;
- compiler proof planning;
- backend target-program emission;
- linker constructor/program resolution;
- target opcode semantics;
- private key custody;
- node wallet behavior;
- network submission;
- mining/relay selection;
- deployment calibration search;
- final release validation;
- independent event/query/accounting implementations.

The planned direction is:

```text
LinkedBundle
        +
exact ElementsTarget
        +
typed operation request
        +
typed public input/chain view
        +
authorized caller witness material
        ↓
validate ABI and operation request
        ↓
derive canonical semantic construction plan
        ↓
instantiate constructors and metadata
        ↓
assemble canonical unsigned transaction
        ↓
apply selected value representation and proofs
        ↓
produce signing requests
        ↓
collect caller-supplied signatures
        ↓
assemble final witnesses/control paths
        ↓
final target transaction
        ├── target execution
        ├── relation-indexed vectors
        ├── calibration measurement
        └── deployment use
```

The first supported covenant operation is:

```text
compact-ash
```

The second is:

```text
transfer-live-receipts
```

---

## 2. Why this package is first-class

The concrete transaction shape is part of target implementation correctness.

The linked target programs may require:

- specific input-family ordering;
- specific output-family ordering;
- fixed or ranged family slots;
- one canonical coordinator;
- operation-specific transaction version and sequence;
- metadata in one exact canonical encoding;
- arithmetic witnesses in one exact order;
- signatures over a finalized output set;
- public openings or commitment proofs;
- one selected target leaf/program;
- one exact control path;
- sponsor inputs and change in a separate region;
- data outputs with canonical ordinals.

These requirements must be shared by:

- production clients and wallets;
- the vector harness;
- the calibration runner;
- genesis/deployment tooling;
- independent implementers;
- release publication.

If each consumer independently reconstructs transaction conventions from:

- backend source;
- target script bytes;
- test fixtures;
- planning prose;
- operation-specific hand coding;

then the project gains several competing ABIs.

The transaction package is therefore not a test helper. It is the typed
construction boundary between one linked target bundle and every authorized
transaction producer.

---

## 3. Assurance boundary

The transaction package establishes:

- the operation request is compatible with the linked ABI;
- selected inputs and outputs occupy the canonical family layout;
- metadata encodings are canonical;
- constructor instances derive from the exact linked constructor recipes;
- target program/leaf and control data correspond to the exact linked bundle;
- witness items are encoded and ordered according to the ABI;
- signatures are requested only after the protected transaction projection is
  finalized;
- supplied signatures occupy the correct witness roles;
- confidential/public representations follow the selected linked plan;
- permissionless operations require no unavailable private witness;
- concrete target transaction bytes are deterministically derived from all
  explicit construction inputs;
- worst-case fixtures are valid ABI-conforming transactions;
- construction reports bind exact bundle, target, ABI, and operation
  identities.

It does not establish by itself:

- the linked target programs implement the realization correctly;
- the target node enforces the declared semantics;
- the selected inputs are economically available or unspent at broadcast time;
- a signature was produced by an honest owner rather than merely validating;
- the transaction will relay, confirm, or win fee competition;
- calibration search selected optimal or final bounds;
- independent indexers derive correct public events;
- deployment release evidence is complete.

The vector and release boundaries remain distinct.

---

## 4. Dependency direction

### 4.1 Required dependencies

The package is expected to depend on:

```text
linker
target-elements
```

It may depend on:

```text
realization
architecture
```

for:

- typed operation/object IDs;
- semantic expression evaluation;
- semantic provenance;
- target-independent amount/count domains.

It may depend directly on:

```text
tapscript
```

only if linked tapscript types are not fully exposed through `linker`.

Preferred direction:

```text
transaction
    → linker
    → tapscript
```

This keeps the transaction package consuming final linked bundle types rather
than unlinked backend internals.

### 4.2 Prohibited dependencies

The package must not depend on:

```text
model
compiler, unless a narrow typed construction-plan interface requires it
vectors
release
artifacts
simplicity
```

The vector package depends on transaction:

```text
vectors → transaction
```

The release/calibration orchestrator depends on transaction and linker:

```text
release or calibration runner
    → transaction
    → linker
```

The linker must not depend on transaction, preventing a cycle.

### 4.3 Future backend-specific modules

The initial implementation is Elements-specific.

A likely module structure is:

```text
transaction::elements
```

A future Simplicity backend may add:

```text
transaction::simplicity
```

Do not force both targets into one lowest-common-denominator concrete
transaction type.

---

## 5. Normative typed inputs

### 5.1 Final or candidate linked bundle

Primary backend/deployment input:

```rust
&linker::CandidateLinkedBundle
```

for:

- vectors;
- calibration;
- prototype transactions.

Final deployment construction consumes:

```rust
&linker::LinkedBundle
```

The package must preserve the distinction.

A candidate bundle cannot be used by an API labeled production/final without an
explicit override restricted to tests or calibration.

### 5.2 Exact typed target

The package consumes:

```rust
&target_elements::ElementsTarget
```

It uses target facts for:

- transaction serialization;
- asset/value encoding;
- sighash construction;
- script-path witness encoding;
- timelock fields;
- confidential transaction behavior;
- issuance fields where applicable;
- consensus and policy validation;
- resource calculations.

It must reject a linked bundle for a different target.

### 5.3 Typed linked ABI input

The linker/backend handoff must provide:

- operation layout;
- family positions/ranges;
- constructor recipes;
- target program/leaf IDs;
- control-path recipes;
- witness requirements;
- representation requirements;
- calibrated or candidate bounds;
- transaction-level constraints;
- relation provenance.

The transaction package derives a final canonical ABI from those linked values.

It does not infer layout by disassembling target programs.

### 5.4 Typed operation request

Each operation receives a typed request.

Examples are discussed below.

The request identifies semantic choices the caller is authorized to make,
such as:

- input outpoints;
- destination owners;
- burn records;
- requested sponsor inputs;
- operator/owner role;
- explicit representation preference where policy allows alternatives.

The request must not let the caller choose:

- a formula-bound payout;
- a state assignment;
- an issuance amount;
- an unauthorized recipient;
- an operation family not present in the linked bundle;
- an arbitrary target program;
- a layout index;
- a target witness order;
- a closed-asset type.

Those values derive from the linked ABI and semantic construction rules.

### 5.5 Public chain/input view

Construction requires a typed view of selected inputs and public state.

Conceptually:

```rust
pub struct ConstructionView {
    pub inputs: BTreeMap<OutPoint, ResolvedInput>,
    pub roots: ResolvedRootSet,
    pub public_openings: PublicOpeningSet,
    pub checkpoint: ConstructionCheckpoint,
}
```

> Illustrative API; not frozen.

The package library should accept this typed view rather than performing node
RPC internally.

A separate client adapter may obtain it from a node/index.

The view must identify:

- exact outpoint;
- target asset/value representation;
- target program/scriptPubKey;
- authenticated metadata or public opening;
- confirmation/age facts where required;
- target checkpoint context.

### 5.6 Authorized secret material

Secret inputs are supplied explicitly by authorized caller components.

Examples:

- owner signing capability;
- operator signing capability;
- sponsor signing capability;
- owner-known value opening;
- owner-held blinding data;
- target proof-generation secret.

The core transaction plan should prefer signing requests/capability interfaces
over receiving raw private keys.

Private keys must not be stored in canonical transaction plans or reports.

### 5.7 Randomness source

Confidential transaction construction may require cryptographic randomness.

Randomness must be an explicit construction dependency.

Conceptually:

```rust
pub trait ConstructionRandomness {
    fn fill_bytes(
        &mut self,
        purpose: RandomnessPurpose,
        output: &mut [u8],
    ) -> Result<(), RandomnessError>;
}
```

> Illustrative API; not frozen.

Production adapters use a cryptographically secure source.

Canonical tests use fixed, clearly non-production deterministic fixture
randomness.

Given identical typed inputs **including the supplied randomness**, construction
must be deterministic.

### 5.8 Construction policy

Typed policy may select among linked supported choices, such as:

- explicit versus confidential sponsor value;
- explicit versus confidential live transfer;
- fee target;
- change policy;
- operation-specific client convenience policy.

Policy must not select unsupported representations or alter semantic formulas.

---

## 6. Forbidden inputs and behavior

The package must not consume:

- architecture JSON/TOML;
- declassification JSON;
- realization Markdown;
- plans;
- model source or model tests;
- target opcode Markdown;
- backend disassembly as ABI;
- environment variables in pure construction APIs;
- current time;
- ambient wallet/node configuration;
- live RPC responses inside the library;
- untyped deployment-profile files;
- raw private keys in canonical reports.

The package must not:

- redefine operation semantics;
- calculate formulas from a handwritten parallel table;
- infer family layout from target bytes;
- silently reorder semantically significant inputs/outputs;
- choose arbitrary recipients;
- change calibrated bounds;
- substitute draft bound defaults;
- add an owner/operator signature to a permissionless operation;
- omit a required public opening;
- accept confidential closed asset identity under the initial profile;
- use production randomness from deterministic test seeds;
- log secrets, raw signing material, blinding factors, or credentials;
- broadcast transactions;
- mark a candidate transaction as confirmed;
- write final release artifacts from pure library APIs.

---

## 7. Typed outputs

The package should separate several construction stages.

### 7.1 Canonical transaction ABI

The final ABI is derived from the linked bundle.

Conceptually:

```rust
pub struct TransactionAbi {
    pub schema_version: TransactionAbiSchemaVersion,

    pub architecture: ArchitectureBinding,
    pub realization: RealizationBinding,
    pub target: TargetBinding,
    pub bundle: LinkedBundleBinding,

    pub operations: BTreeMap<
        architecture::OperationId,
        OperationTransactionAbi,
    >,

    pub metadata_schemas: MetadataSchemaRegistry,
    pub constructor_schemas: ConstructorSchemaRegistry,
    pub witness_schemas: WitnessSchemaRegistry,

    pub identity: TransactionAbiIdentity,
}
```

> Illustrative API; not frozen.

### 7.2 Construction plan

A construction plan is a typed, non-secret plan for one operation.

Conceptually:

```rust
pub struct ConstructionPlan {
    pub operation: architecture::OperationId,
    pub target: TargetBinding,
    pub bundle: BundleBinding,
    pub abi: TransactionAbiBinding,

    pub selected_inputs: Vec<PlannedInput>,
    pub planned_outputs: Vec<PlannedOutput>,
    pub data_outputs: Vec<PlannedDataOutput>,

    pub state_effects: Vec<PlannedStateEffect>,
    pub representation: RepresentationConstructionPlan,
    pub signing_roles: Vec<SigningRole>,
    pub witness_roles: Vec<WitnessRole>,
    pub relation_bindings: Vec<RelationConstructionBinding>,
}
```

> Illustrative API; not frozen.

The plan must not contain production private keys or secret blinding values.

### 7.3 Unsigned transaction

The package produces a typed unsigned target transaction whose:

- input order;
- output order;
- metadata;
- target programs;
- public values;
- fee fields;
- sequence/version;
- operation shape

are finalized as far as required before signing.

For confidential transactions, “unsigned” may still need a staged distinction
between:

- unblinded transaction template;
- blinded/proved transaction;
- signing-ready transaction.

### 7.4 Signing requests

A signing request identifies:

- signer role;
- target input;
- owner/operator/sponsor public key identity;
- exact sighash mode;
- message digest or target signing data;
- protected transaction identity;
- relation provenance;
- expected signature encoding.

It contains no private key.

### 7.5 Final target transaction

A final transaction includes:

- target transaction bytes/value;
- per-input witnesses;
- selected target programs;
- control paths;
- public proof/opening data;
- signatures;
- construction identity;
- relation and ABI bindings.

The final transaction result should include a construction report but must not
include secret material.

### 7.6 Worst-case transaction fixture

A worst-case fixture is a valid construction result for calibration.

It identifies:

- operation;
- semantic activation case;
- family counts;
- representation modes;
- witness size assumptions;
- sponsor usage;
- target/bundle/ABI identities;
- why it is worst-case for each measured resource dimension.

A single fixture may not be worst-case for every dimension. The calibration
runner may require several fixtures per operation.

---

## 8. Public API boundary

### 8.1 ABI derivation

Conceptually:

```rust
pub fn derive_abi(
    bundle: &linker::LinkedBundle,
    target: &target_elements::ElementsTarget,
) -> Result<TransactionAbi, TransactionError>;
```

> Illustrative API; not frozen.

For candidate bundles used in calibration:

```rust
pub fn derive_candidate_abi(
    bundle: &linker::CandidateLinkedBundle,
    target: &target_elements::ElementsTarget,
) -> Result<CandidateTransactionAbi, TransactionError>;
```

Candidate and final ABIs must be distinct or explicitly status-tagged.

### 8.2 Planning API

Conceptually:

```rust
pub fn plan_operation<R: OperationRequest>(
    abi: &TransactionAbi,
    view: &ConstructionView,
    request: &R,
    policy: &ConstructionPolicy,
) -> Result<ConstructionPlan, TransactionError>;
```

> Illustrative API; not frozen.

An enum-based request API may be preferable initially:

```rust
pub enum OperationRequest {
    CompactAsh(CompactAshRequest),
    TransferLive(TransferLiveRequest),
    // added in roadmap order
}
```

The exact API should optimize clarity and exhaustiveness rather than generic
framework design.

### 8.3 Build API

Construction should be staged.

Conceptually:

```rust
pub fn build_unsigned(
    abi: &TransactionAbi,
    plan: &ConstructionPlan,
) -> Result<UnsignedTransaction, TransactionError>;
```

```rust
pub fn apply_representation(
    abi: &TransactionAbi,
    transaction: UnsignedTransaction,
    witness_material: &RepresentationWitnessMaterial,
    randomness: &mut impl ConstructionRandomness,
) -> Result<SigningReadyTransaction, TransactionError>;
```

```rust
pub fn signing_requests(
    transaction: &SigningReadyTransaction,
) -> Result<Vec<SigningRequest>, TransactionError>;
```

```rust
pub fn finalize(
    transaction: SigningReadyTransaction,
    signatures: &SignatureSet,
) -> Result<FinalizedTransaction, TransactionError>;
```

> Illustrative staged APIs; not frozen.

The ordering must follow exact Elements signature and confidential transaction
semantics. In particular, all signature-committed outputs must be finalized
before signing requests are generated.

### 8.4 Verification/preflight API

The package may provide local preflight validation:

```rust
pub fn validate_construction(
    abi: &TransactionAbi,
    transaction: &FinalizedTransaction,
) -> Result<ConstructionReport, TransactionError>;
```

This checks ABI consistency and local target encoding.

It does not replace target-native script/consensus execution.

### 8.5 Worst-case generation API

Conceptually:

```rust
pub fn worst_case_transactions(
    abi: &CandidateTransactionAbi,
    operation: architecture::OperationId,
    policy: &WorstCasePolicy,
    fixtures: &WorstCaseFixtureInputs,
) -> Result<Vec<WorstCaseTransaction>, TransactionError>;
```

> Illustrative API; not frozen.

The generator must produce **valid** transactions for the candidate bundle.

### 8.6 No broadcast API in the core package

Node submission belongs in a client/deployment adapter.

The transaction library remains deterministic and network-independent given
its typed inputs.

---

## 9. Proposed module structure

A likely initial structure is:

```text
transaction/src/
├── lib.rs
├── error.rs
├── identity.rs
├── abi.rs
├── request.rs
├── plan.rs
├── view.rs
├── input.rs
├── output.rs
├── metadata.rs
├── constructor.rs
├── signing.rs
├── randomness.rs
├── representation.rs
├── resources.rs
├── worst_case.rs
│
└── elements/
    ├── mod.rs
    ├── transaction.rs
    ├── serialization.rs
    ├── script_path.rs
    ├── control.rs
    ├── sighash.rs
    ├── witness.rs
    ├── confidential.rs
    ├── issuance.rs
    └── policy.rs
```

> Illustrative module structure; create modules only as implementation requires.

Operation-specific request/construction helpers may live under:

```text
transaction/src/operations/
```

once more than the two pilot operations exist.

---

## 10. ABI derivation

### 10.1 Inputs

ABI derivation consumes linked:

- operation layouts;
- relation carriers;
- constructor recipes;
- target programs;
- witness requirements;
- calibrated bounds;
- representation support;
- transaction-level constraints;
- target identity;
- bundle identity.

### 10.2 ABI validation

Require:

- every in-scope operation has one layout;
- every layout references linked constructors/programs;
- every target program has one witness schema;
- every semantic family has concrete target placement;
- every required relation has a carrier;
- every calibrated maximum is final for a final ABI;
- no unresolved mandatory relocation remains;
- target and bundle identities match;
- candidate/final status matches the requested ABI type.

### 10.3 ABI operation record

Conceptually:

```rust
pub struct OperationTransactionAbi {
    pub operation: architecture::OperationId,
    pub enforcement_class: OperationEnforcementClass,

    pub inputs: Vec<InputFamilyAbi>,
    pub outputs: Vec<OutputFamilyAbi>,
    pub data_outputs: Vec<DataOutputFamilyAbi>,

    pub coordinator: Option<CoordinatorAbi>,
    pub sponsor: SponsorAbi,
    pub target_constraints: Vec<TargetTransactionConstraint>,

    pub programs: Vec<TargetProgramAbi>,
    pub relation_carriers: Vec<RelationCarrierAbi>,
}
```

> Illustrative API; not frozen.

### 10.4 Enforcement classes

The ABI should distinguish at least:

```text
client-policy construction
covenant-enforced transition
trusted setup/deployment construction
```

`create-request` is initially client-policy construction.

A malformed open request can exist but remains inadmissible until a covenant
operation validates it.

### 10.5 ABI canonical order

The ABI preserves backend/linker-selected family order.

Within a family, ordering policy must be explicit:

- canonical outpoint order;
- caller-specified semantic order;
- owner/key order;
- record ordinal order;
- target-defined positional correspondence.

The transaction package must not reorder an order-sensitive family.

---

## 11. Typed operation requests

### 11.1 General rule

A request contains only choices the caller is semantically authorized to make.

The builder derives:

- formulas;
- state updates;
- fixed recipients;
- constructor identities;
- output family positions;
- target programs;
- witness order.

### 11.2 `compact-ash` request

Conceptually:

```rust
pub struct CompactAshRequest {
    pub ash_inputs: Vec<OutPoint>,
    pub sponsor: Option<SponsorRequest>,
}
```

The caller does not provide:

- output ASH value;
- output constructor;
- output index;
- relation witness order;
- operation leaf ID.

The builder derives the ASH output value from authenticated inputs.

### 11.3 `transfer-live-receipts` request

Conceptually:

```rust
pub struct TransferLiveRequest {
    pub receipt_inputs: Vec<OutPoint>,
    pub destinations: Vec<ReceiptDestinationRequest>,
    pub representation: ValueRepresentationRequest,
    pub sponsor: Option<SponsorRequest>,
}
```

```rust
pub struct ReceiptDestinationRequest {
    pub owner: OwnerKey,
    pub value: SemanticAmountRequest,
}
```

> Illustrative API; not frozen.

The caller may choose destination owners and denominations subject to:

- every input owner authorizing;
- exact aggregate conservation;
- live class closure;
- target representation support;
- calibrated family bounds.

The caller does not choose a hidden asset identity or undeclared output class.

### 11.4 Later requests

Later requests should preserve semantic authorization:

- burn caller chooses record destinations and allowed live change;
- redeemer chooses no payout amount or recipient beyond the consumed owner's
  semantic role;
- settler chooses an eligible batch but not payout recipients/amounts;
- cycle caller chooses caller role but not issuance arithmetic;
- announcer chooses a maturity cycle inside the valid range;
- sponsor chooses its own inputs, fee contribution, and permitted change.

### 11.5 Request validation

Request validation must occur before secret witness generation where possible.

Reject:

- duplicate input;
- too few/many family members;
- missing public input data;
- unsupported representation;
- destination count above bound;
- value mismatch;
- wrong operation scope;
- sponsor overlap;
- unauthorized semantic choice;
- unavailable lifecycle path.

---

## 12. Construction view

### 12.1 Resolved input

A resolved input should contain typed public and authorized-private data.

Conceptually:

```rust
pub struct ResolvedInput {
    pub outpoint: OutPoint,
    pub target_output: TargetOutputView,

    pub constructor: Option<ResolvedConstructorInstance>,
    pub semantic_object: ResolvedSemanticObject,

    pub public_opening: Option<PublicValueOpening>,
    pub confirmation: ConfirmationView,
}
```

> Illustrative API; not frozen.

Owner-private opening/blinding data should be supplied separately from the
public view unless an authorized owner adapter combines them.

### 12.2 Root view

STATE and other roots need:

- canonical outpoint;
- constructor identity;
- authenticated metadata;
- confirmation/age where applicable;
- target instance;
- semantic state projection.

The view is caller-supplied typed data. Target programs remain the ultimate
enforcement boundary.

### 12.3 Checkpoint binding

Construction should bind to one chain/checkpoint context sufficient to detect
stale input assumptions before signing.

A transaction may still become stale after construction due to contention.
That is a liveness issue, not a construction proof failure.

### 12.4 No implicit database access

The library does not query:

- UTXO database;
- indexer database;
- node RPC;
- wallet store.

Adapters gather data and construct the typed view.

---

## 13. Canonical input ordering

### 13.1 Family order

Top-level family ordering comes from the linked ABI.

For example:

```text
compact-ash:
    ASH family
    sponsor family

transfer-live:
    live receipt family
    sponsor family
```

### 13.2 Within-family order

Each family ABI must state one rule.

Possible rules:

1. caller order is semantic and preserved;
2. lexicographic outpoint order;
3. owner then outpoint;
4. target-specific fixed correspondence;
5. control/root first, then bounded queue order.

### 13.3 Default rule

For a set-like family without semantic order, prefer canonical outpoint order.

This improves:

- deterministic transaction bytes;
- vector stability;
- duplicate detection;
- signing coordination.

Do not sort a family when operation semantics use positional correspondence,
such as an accepted relabel layout.

### 13.4 Duplicate detection

Reject duplicate outpoints before transaction assembly.

Also reject:

- one input assigned to two families;
- one input assigned as both protocol and sponsor;
- one root repeated;
- one semantic source referenced twice in a relation plan.

---

## 14. Canonical output ordering

### 14.1 ABI-owned order

Output family order is fixed by the linked ABI.

The request supplies semantic destination data within permitted families, not
arbitrary target indexes.

### 14.2 Within-family order

A family may use:

- caller order;
- owner/class canonical order;
- positional correspondence to inputs;
- record ordinal order;
- stable semantic-key order.

The ABI declares the rule.

### 14.3 Optional outputs

Optional families have one canonical presence rule.

Examples:

- sponsor change;
- ASH residual;
- RESV successor;
- distribution vault;
- CPFP anchor;
- terminal residue output.

The builder does not emit zero-value ordinary outputs to stand in for absence
unless the target ABI explicitly requires that representation.

### 14.4 Formula-derived outputs

The builder derives:

- STATE successor fields;
- formula-bound payout;
- reserve successor;
- issued value;
- distribution counters;
- ASH aggregate/residual;
- entitlement value;
- chain-fee partition.

The request must not override them.

### 14.5 Closed-asset closure

Every closed-asset-capable output must instantiate one declared linked
constructor or declared destruction output.

No generic “extra output” path may carry:

```text
U
ENT
DIST_CTL
PID
PACE
ENT_AUTH
DIST_AUTH
```

under the initial representation policy.

---

## 15. Metadata encoding and constructor instantiation

### 15.1 Typed metadata

Each object constructor receives typed metadata.

Examples:

```text
STATE:
    Ω, Y_L, Y_T, Q, cycle, maturity

RECEIPT:
    owner, class

ENTITLEMENT:
    owner, target cycle

CONTROL:
    cycle, principal, allocations, remainders

VAULT:
    cycle

REQUEST:
    pool ID, refund key, receipt owner, principal
```

Metadata schemas come from linked constructor recipes and typed realization
provenance.

### 15.2 Canonical encoding

For each schema define:

- field order;
- field type;
- byte order;
- fixed/variable width;
- domain bounds;
- schema version;
- object/domain separator;
- canonical zero;
- sentinel/tag encoding;
- invalid encoding behavior.

### 15.3 No independent face value

Metadata must not contain an unauthenticated scalar that overrides the
consensus value.

When metadata partitions a semantic value, the construction plan must enforce
the exact relation through linked target programs.

### 15.4 Constructor instantiation

The transaction package combines:

```text
linked static constructor recipe
+
typed metadata
+
target deployment constants
→ concrete target output program/scriptPubKey
```

The implementation must:

- validate metadata domain;
- use exact linked static roots/programs;
- use exact internal key;
- compute target commitments deterministically;
- expose target program identity for vectors/reports;
- reject alternate code subtree substitution.

### 15.5 Control-path construction

For tapscript:

- select the linked operation leaf;
- compute or obtain the concrete taptree path;
- derive parity/control fields;
- serialize control data canonically;
- verify it corresponds to the concrete input constructor before finalization.

Do not accept caller-supplied arbitrary control blocks without validation.

### 15.6 Open-output constructors

Ordinary L-BTC payout/change outputs may use typed target output constructors
rather than protocol object constructors.

Their recipient program and representation must still match the ABI and
semantic plan.

---

## 16. Transaction construction stages

### 16.1 Stage A — semantic planning

Inputs:

- ABI;
- construction view;
- operation request;
- policy.

Outputs:

- selected semantic inputs;
- derived state/formulas;
- planned outputs;
- representation requirements;
- signing roles;
- proof roles.

No target signatures are produced.

### 16.2 Stage B — constructor materialization

Materialize:

- protocol output programs;
- ordinary recipient outputs;
- data outputs;
- transaction version;
- locktime;
- sequences;
- operation-specific fields.

### 16.3 Stage C — unblinded transaction template

Assemble canonical inputs and outputs with semantic values and representation
requirements before target blinding/proof generation.

This stage is useful for:

- validation;
- expected relation projection;
- deterministic test planning;
- representation choice.

### 16.4 Stage D — confidential/public representation

Where selected:

- choose blinding factors from explicit randomness input;
- construct value commitments;
- construct nonce fields;
- generate rangeproofs/surjection proofs;
- preserve explicit closed-asset identity;
- produce public commitment/opening data where required;
- route residual blinding according to the selected proof plan.

This stage must complete before signatures whose sighash commits outputs.

### 16.5 Stage E — signing requests

Compute target signing messages after all committed transaction fields are
final.

Produce one request per required signer/input.

### 16.6 Stage F — signature collection

Accept typed signatures from external signer adapters.

Validate:

- role;
- input;
- public key;
- encoding;
- sighash mode;
- transaction identity;
- signature validity where local verification is supported.

Do not accept one signature for a different transaction revision.

### 16.7 Stage G — final witness assembly

Assemble:

- semantic witnesses;
- arithmetic witnesses;
- public openings;
- signatures;
- selected leaf/program;
- control data;
- target-specific script-path structure.

### 16.8 Stage H — preflight and finalization

Validate:

- ABI;
- target serialization;
- local target constraints;
- expected relation carrier reachability;
- resource estimates;
- transaction identity;
- no unresolved witness role.

Return a finalized target transaction and construction report.

Target-native execution remains downstream evidence.

---

## 17. Signing model

### 17.1 No private key ownership

The transaction package does not own a production keystore.

It produces signing requests and accepts signatures.

Adapters may integrate with:

- hardware wallets;
- software wallets;
- remote signers;
- test keys.

### 17.2 Signing request identity

Bind:

- target;
- bundle;
- ABI;
- operation;
- transaction template;
- input;
- public key;
- sighash mode;
- message;
- role.

### 17.3 Output commitment

The selected signature profile must commit all required economic outputs.

Construction must not generate signing requests until those outputs are fixed.

### 17.4 Multi-owner operations

For multi-owner transfer or burn:

- every required owner gets a signing request;
- all requests commit the same finalized output set;
- finalization requires every required signature;
- missing owner rejects;
- duplicate/unknown signature rejects;
- signatures cannot be mixed across transaction templates.

### 17.5 Sponsor signatures

Sponsor signatures authorize only sponsor-owned inputs but may commit the full
output set.

The package must preserve the selected sponsor/input-extension policy exactly.

### 17.6 Operator signatures

Operator signatures appear only on operations or cadence branches requiring
them.

Permissionless target programs must not request one.

### 17.7 Secret-safe diagnostics

Signing errors identify:

- role;
- input;
- public key identity;
- error class.

They must not print:

- private key;
- signing nonce;
- hardware-wallet secret;
- raw credential URL;
- hidden blinding factor.

---

## 18. Confidential transaction construction

### 18.1 Scope

The initial confidential construction scope is expected to include:

- live receipt transfer values;
- sponsor input/change values.

Later support may include other operations after research decisions.

### 18.2 Closed asset identity

Protocol closed assets remain explicit under D005.

Confidential value construction must not create a confidential asset output for
a closed protocol asset.

### 18.3 Input unblinding data

Owner-authorized private inputs may require:

- semantic amount;
- value blinding factor;
- rangeproof-related data;
- asset information;
- owner authorization.

Such material is secret and caller-supplied.

It must not enter canonical reports.

### 18.4 Permissionless inputs

A permissionless operation cannot require owner-private unblinding data.

Its input values must be:

- explicit;
- publicly opened;
- or accompanied by a public proof capsule accepted by the linked target plan.

### 18.5 Blinding balance

Construction must satisfy the exact target confidential transaction balance.

If a confidential input is fully consumed while an output becomes public, the
selected public-declassification plan must route residual blinding correctly.

Do not attempt an explicit output when target balance requires a public
commitment unless the accepted plan proves it valid.

### 18.6 Public committed values

A public-committed output must include:

- value commitment;
- publicly available amount/opening;
- target-authenticated opening proof;
- canonical opening encoding;
- future constructor availability.

Support remains blocked until the public-declassification research decision is
accepted.

### 18.7 Proof generation

Rangeproof and surjection-proof generation belongs to the transaction package
or a narrowly owned target-construction dependency.

The target package defines semantics and formats; transaction constructs the
proofs.

### 18.8 Production randomness

Use a cryptographically secure injected source.

Never reuse deterministic test randomness in production adapters.

### 18.9 Confidentiality evidence

For each claimed private representation:

- build valid confidential transaction;
- execute on target;
- compare semantic projection;
- mutate commitment/balance/proof and require rejection;
- verify no secret appears in canonical report;
- verify closed asset remains explicitly classified.

---

## 19. Public declassification

### 19.1 Research dependency

Concrete support is governed by:

```text
research/public-declassification.md
```

The transaction package must not invent its own opening scheme.

### 19.2 Required construction properties

Any accepted declassification construction must:

- bind public amount to exact consensus commitment/value;
- preserve semantic value;
- preserve explicit asset identity;
- route residual blinding;
- expose opening to future permissionless constructors;
- use canonical encoding;
- reject malformed opening;
- preserve target resource limits;
- bind the selected target proof plan.

### 19.3 Burn-to-ASH

For burn:

- source receipt/change may remain private where supported;
- fresh ASH aggregate is public/openable;
- burn records are public;
- public ASH must remain compactable and clearable without owner secret.

### 19.4 Redemption

For redemption:

- receipt amount `x` must become available for floor arithmetic;
- payout `p` is public/formula-bound;
- value destruction/state decrement must be exact;
- owner-authorized normalization may be used if declared.

---

## 20. Data-output construction

### 20.1 Burn records

Burn records require:

- canonical tag/domain;
- contiguous `record_index` from zero;
- address;
- positive amount;
- canonical output order;
- output-commitment signatures;
- public encoding.

The transaction package must derive record indexes from list order or validate
caller-provided records before encoding.

Prefer not to let callers supply arbitrary ordinals.

### 20.2 Destruction outputs

Destruction outputs require:

- declared tag;
- declared asset;
- exact amount;
- unspendable target encoding;
- operation/activation match;
- canonical placement.

### 20.3 Unspendability

The package constructs the target unspendable output according to linked/target
policy.

Target-native tests and deployment evidence establish that such outputs do not
remain spendable current state.

### 20.4 No generic OP_RETURN sidecar

The request API must not expose arbitrary unclassified data outputs in covenant
operations.

Client-policy operations may support external data only through a separate,
explicit policy not confused with protocol event outputs.

---

## 21. Client-policy request creation

### 21.1 Classification

`create-request` is a client-policy transaction over ordinary L-BTC inputs.

It has no covenant input enforcing validity at creation.

### 21.2 Canonical constructor

The transaction package may provide a canonical request builder that:

- validates principal and gross value;
- commits pool ID;
- commits refund key;
- commits receipt owner;
- constructs request output;
- constructs optional owner change;
- computes explicit chain fee;
- creates owner signing requests;
- follows the selected request representation policy.

### 21.3 Security boundary

A malformed request-shaped output can still be created by another client.

That does not violate protocol state. Admission later validates it.

The builder's checks are client-policy safety and interoperability, not
consensus proof that all request-shaped outputs are valid.

### 21.4 Cancellation compatibility

The request constructor must create metadata and representation compatible with
at least one supported cancellation path and one supported admission path.

A representation that can be created but cannot later cancel or admit is not a
supported request profile.

---

## 22. Permissionless construction

### 22.1 General theorem obligation

For every permissionless operation, construction must succeed from:

```text
public chain/input view
+
public linked bundle and ABI
+
public openings/proof capsules
+
constructor's own sponsor material
```

No other participant secret is permitted.

### 22.2 Initial `compact-ash`

Inputs required:

- public ASH outpoints;
- public/openable ASH values;
- public constructor metadata;
- optional sponsor-local funds and signatures;
- public bundle/ABI.

No owner/operator secret.

### 22.3 Later operations

#### Admission

Requires public request principal/budget facts.

#### Delayed cycle

Requires public STATE/RESV/PACE and no operator signature.

#### Settlement

Requires public entitlement/control/vault facts and fixed recipients.

#### Relabel

Requires publicly constructible value-preserving successor proof.

#### Clear

Requires public/openable ASH aggregate and public STATE.

### 22.4 Test harness discipline

Permissionless construction tests must not access hidden fixture fields that a
real arbitrary constructor would lack.

Use separate public-view and private-fixture types where helpful.

---

## 23. Worst-case transaction generation

### 23.1 Purpose

Generate valid transactions that maximize one or more target resource
dimensions for calibration.

### 23.2 Resource dimensions

Fixtures may target:

- transaction weight;
- witness bytes;
- control-path bytes;
- initial stack count;
- peak stack/altstack;
- maximum stack element;
- crypto budget;
- executed opcode/project cost;
- confidential proof size;
- target standardness.

### 23.3 Several maxima may be required

The transaction maximizing weight may differ from the transaction maximizing:

- crypto operations;
- stack depth;
- witness item count;
- one script branch.

Return a deterministic set of worst-case candidates with stated objective.

### 23.4 Validity requirement

Every calibration fixture must:

- conform to ABI;
- satisfy semantic preconditions;
- contain valid signatures/proofs or accepted deterministic test equivalents;
- execute successfully on the candidate target;
- exercise the intended maximum branch;
- bind exact candidate bundle and ABI identities.

An invalid oversized transaction is not calibration evidence.

### 23.5 Shared-bound coverage

For a bound used by several operations, generate worst-case transactions for
every affected operation family.

The calibration runner selects a bound only after all pass.

### 23.6 Candidate/final distinction

Worst-case transactions for a candidate bundle are not reused after:

- bound change;
- program change;
- layout change;
- ABI change;
- target change;
- representation change.

Final calibration regenerates all fixtures from the final candidate.

---

## 24. Resource accounting

### 24.1 Predicted resources

Use linked resource formulas plus concrete transaction serialization to
predict:

- transaction weight;
- witness bytes;
- control data;
- per-input script resources;
- crypto budget;
- policy status.

### 24.2 Observed resources

The vector/target harness measures actual resources.

### 24.3 Comparison

Construction reports should bind predicted values.

Calibration reports compare predicted and observed values.

A mismatch beyond the accepted exact/tolerance policy invalidates the formula
or fixture.

### 24.4 No self-declared success

The transaction package cannot mark a candidate bound feasible merely because
local prediction passes.

Target-native measurement is required.

---

## 25. Construction identity

### 25.1 Construction-plan identity

A non-secret plan identity should bind:

- target;
- bundle;
- ABI;
- operation;
- public input outpoints;
- public semantic request;
- planned output families;
- representation selection;
- transaction-level constraints;
- signing roles.

It must exclude:

- private keys;
- signatures;
- secret blinding factors;
- production randomness;
- unpublished openings.

### 25.2 Unsigned/signing-ready identity

The exact signing-ready target transaction must have an identity used by all
signing requests.

Any output, representation, fee, or sequence mutation changes it.

### 25.3 Final transaction identity

The final target transaction uses the target's canonical transaction IDs and
may additionally have a construction-report identity.

Do not conflate txid and wtxid where target semantics distinguish them.

### 25.4 ABI identity

The ABI identity binds:

- ABI schema;
- target;
- linked bundle;
- calibrated bounds;
- layouts;
- constructors;
- witness schemas;
- metadata schemas;
- representation support;
- transaction constraints.

### 25.5 Determinism and randomness

With identical explicit inputs, including explicit randomness and signatures,
construction produces identical bytes.

Different valid production randomness may produce different transaction bytes
while representing the same semantic operation.

Those transactions have different concrete identities.

---

## 26. Canonical ABI publication

### 26.1 Candidate artifact

Potential artifact:

```text
transaction-abi.json
```

The name is illustrative.

### 26.2 Required contents

The publication should contain enough public information for independent
construction:

- schema;
- target identity;
- linked-bundle identity;
- operation scope;
- family layouts;
- constructor schemas;
- metadata encodings;
- witness item schemas;
- representation modes;
- calibrated maxima;
- transaction constraints;
- target program/leaf references;
- control-path recipes or derivation rules;
- ABI hash.

It must not contain secrets.

### 26.3 One-way publication

First-party code consumes the typed ABI.

The JSON is derivative under D001.

### 26.4 Generator/checker

The ABI artifact requires:

- one typed source;
- one explicit generator;
- one non-writing checker;
- canonical ordering;
- unknown-field rejection;
- deterministic rendering;
- bundle identity verification;
- stale-artifact CI.

### 26.5 Independent implementation

An external wallet may consume the publication after validating:

- schema;
- target;
- bundle;
- ABI hash.

That does not make publication the first-party semantic source.

---

## 27. Error model

Errors should be typed, deterministic, stage-aware, and secret-safe.

Candidate classes include:

```rust
pub enum TransactionError {
    UnsupportedAbiSchema,
    CandidateAbiUsedAsFinal,
    TargetIdentityMismatch,
    BundleIdentityMismatch,
    AbiIdentityMismatch,
    BoundAssignmentMismatch,

    UnsupportedOperation(architecture::OperationId),
    MissingOperationAbi(architecture::OperationId),
    WrongEnforcementClass(architecture::OperationId),

    MissingInput(OutPoint),
    DuplicateInput(OutPoint),
    InputAssignedTwice(OutPoint),
    WrongInputObject(OutPoint),
    WrongInputAsset(OutPoint),
    WrongInputRepresentation(OutPoint),
    StaleInput(OutPoint),

    TooFewInputs(InputFamilyId),
    TooManyInputs(InputFamilyId),
    TooFewOutputs(OutputFamilyId),
    TooManyOutputs(OutputFamilyId),

    MissingPublicOpening(OutPoint),
    InvalidPublicOpening(OutPoint),
    PrivateWitnessRequiredForPermissionlessOperation,
    MissingOwnerWitness(OwnerKey),
    MissingOperatorWitness,
    MissingSponsorWitness(OutPoint),

    UnsupportedRepresentation,
    ConfidentialClosedAssetForbidden,
    ConfidentialProofConstructionFailed,
    BlindingBalanceFailure,
    RangeproofConstructionFailed,
    SurjectionProofConstructionFailed,
    RandomnessFailure,

    InvalidSemanticRequest,
    UnauthorizedRecipientChoice,
    ValueConservationFailure,
    FormulaEvaluationFailure,
    StateAssignmentFailure,
    LifecyclePathUnavailable,

    ConstructorNotFound(architecture::ObjectId),
    ConstructorIdentityMismatch(architecture::ObjectId),
    MetadataEncodingFailure(architecture::ObjectId),
    MetadataDomainFailure(architecture::ObjectId),
    ControlPathMismatch(ProgramId),

    InvalidFamilyOrder,
    InvalidFamilyRange,
    SponsorRegionOverlap,
    DataOutputOrderFailure,
    NonCanonicalRecordOrdinal,

    TransactionVersionMismatch,
    LocktimeMismatch,
    SequenceMismatch,
    FeeMismatch,
    TargetSerializationFailure,

    SigningRequestFailure,
    SignatureRoleMismatch,
    SignatureTransactionMismatch,
    MissingSignature(SigningRoleId),
    DuplicateSignature(SigningRoleId),
    InvalidSignature(SigningRoleId),

    WitnessEncodingFailure(WitnessItemId),
    MissingWitness(WitnessItemId),
    UnexpectedWitness(WitnessItemId),
    WitnessOrderMismatch,
    WitnessSizeExceeded,

    ResourcePredictionFailure,
    PredictedTargetLimitExceeded,

    ConstructionIdentityMismatch,
    NonDeterministicConstruction,
}
```

> Illustrative vocabulary; not frozen.

Error displays must not include:

- raw private keys;
- raw secret openings;
- blinding factors;
- signing nonces;
- RPC credentials;
- credential-bearing URLs.

---

## 28. Validation

### 28.1 ABI validation

As described above.

### 28.2 Request validation

Require:

- operation in ABI scope;
- family counts within bounds;
- no duplicate or overlapping input;
- selected representation supported;
- semantic choices authorized;
- required public inputs present;
- lifecycle path valid;
- sponsor request valid.

### 28.3 Construction-view validation

Require:

- every requested input resolved;
- target output matches target instance;
- constructor identity compatible;
- metadata/public opening valid;
- checkpoint binding present;
- root/cadence facts available where required.

### 28.4 Plan validation

Require:

- every semantic relation mapped to construction fields/witness roles;
- all formula-derived outputs computed;
- all fixed recipients derived;
- all output families complete;
- no undeclared closed-asset output;
- signing roles complete;
- representation plan complete;
- target constraints complete.

### 28.5 Final transaction validation

Require:

- canonical serialization;
- exact family layout;
- exact metadata;
- exact linked target programs;
- correct leaf/control selection;
- complete witnesses;
- complete signatures;
- no unexpected witness item;
- resource prediction available;
- target/bundle/ABI identity consistency.

### 28.6 No local acceptance overclaim

A locally valid construction is labeled:

```text
ABI-valid and locally preflighted
```

not:

```text
consensus accepted
confirmed
deployment valid
```

Target execution and release evidence remain downstream.

---

## 29. Initial `compact-ash` implementation

### 29.1 Request

```rust
pub struct CompactAshRequest {
    pub ash_inputs: Vec<OutPoint>,
    pub sponsor: Option<SponsorRequest>,
}
```

> Illustrative API.

### 29.2 Public view requirements

For each ASH input:

- outpoint;
- explicit `U` identity;
- public/openable value;
- concrete linked ASH constructor/program;
- target program instance;
- confirmation/checkpoint context.

### 29.3 Derived output

The package derives:

```text
ash_output_value = sum(ash_input_values)
```

and instantiates exactly one linked ASH constructor.

### 29.4 Input order

Use ABI family order and canonical within-family rule.

Expected initial within-family rule:

```text
ascending canonical outpoint
```

unless backend ABI selects another explicit rule.

### 29.5 Coordinator

The first ASH input after canonical ordering is expected to use the coordinator
program.

Other ASH inputs use the local participation program.

### 29.6 Sponsor

Sponsor inputs/change occupy the ABI sponsor region.

No sponsor value enters ASH conservation.

### 29.7 Witness

Expected witness roles may include:

- family counts;
- constructor/static root data;
- target script path/control data;
- sponsor signatures;
- target arithmetic witnesses if the linked program requires them.

No owner/operator signature is permitted for the ASH protocol inputs.

### 29.8 Required tests

- minimum valid batch;
- larger valid batch;
- duplicate input;
- wrong asset;
- wrong constructor;
- missing public value/opening;
- wrong output value;
- wrong coordinator;
- sponsor overlap;
- wrong witness order;
- malformed control path;
- candidate/final ABI mismatch;
- deterministic bytes;
- target-native acceptance/rejection downstream.

---

## 30. Initial live-transfer implementation

### 30.1 Request

Conceptually:

```rust
pub struct TransferLiveRequest {
    pub receipt_inputs: Vec<OutPoint>,
    pub destinations: Vec<ReceiptDestinationRequest>,
    pub representation: ValueRepresentationRequest,
    pub sponsor: Option<SponsorRequest>,
}
```

### 30.2 Public/owner-private input split

Public input view contains:

- outpoint;
- explicit `U` identity;
- live receipt constructor;
- owner public key;
- target program;
- public metadata;
- checkpoint.

For confidential value transfer, authorized owner material additionally
contains:

- semantic input amount;
- blinding data;
- proof-generation data as required.

### 30.3 Output construction

The request supplies destination owners and semantic amounts.

The builder:

- validates positive outputs;
- validates exact aggregate amount;
- preserves live class;
- instantiates linked receipt constructors;
- selects explicit or confidential value representation;
- isolates sponsor flow.

### 30.4 Signing coordination

All blinded/proved outputs are finalized before owner signing requests are
produced.

Every owner signs the same finalized economic output set.

### 30.5 Explicit and confidential vectors

Build semantically equivalent:

- explicit transfer;
- confidential-value transfer.

Require equal abstract/public semantic projection and target acceptance.

### 30.6 Closed asset safety

Every receipt output uses explicit `U` asset identity.

Reject confidential asset output even when the value commitment is otherwise
balanced.

---

## 31. Testing strategy

### 31.1 Unit tests

Cover:

- ABI derivation;
- ABI identity;
- family ordering;
- request validation;
- duplicate detection;
- metadata encoding;
- constructor instantiation;
- transaction staging;
- signing request generation;
- witness encoding;
- target serialization;
- deterministic construction;
- secret-safe diagnostics;
- candidate/final status.

### 31.2 Constructor fixtures

For every linked constructor in scope:

- known metadata → expected program/scriptPubKey;
- wrong schema;
- wrong field order;
- out-of-domain field;
- wrong static root;
- wrong internal key;
- deterministic output.

### 31.3 Signing tests

- correct signature;
- wrong signer;
- missing signer;
- duplicate signer;
- signature over stale transaction;
- protected output mutation;
- burn-record mutation under output commitment;
- sponsor policy behavior;
- malformed signature.

Use test-only keys clearly separated from production.

### 31.4 Confidential transaction tests

- valid explicit transfer;
- valid confidential transfer;
- value imbalance;
- invalid rangeproof;
- wrong blinding balance;
- confidential closed asset;
- malformed public opening;
- changed output after proof/signing;
- deterministic fixture randomness;
- no secret in report/debug output.

### 31.5 Permissionless tests

Construct `compact-ash` using only a public view and optional sponsor-local
data.

Later repeat for every permissionless operation.

### 31.6 ABI mutation tests

- wrong family count;
- shifted family range;
- swapped output families;
- wrong coordinator;
- wrong program ID;
- wrong metadata schema;
- wrong witness order;
- wrong calibrated bound;
- ABI for another bundle;
- unsupported schema.

### 31.7 Worst-case fixture tests

- fixture is valid;
- fixture reaches intended maximum branch;
- family count equals candidate maximum;
- target-native execution succeeds;
- predicted and measured resources agree;
- changed candidate identity invalidates fixture reuse.

### 31.8 Public API tests

An external integration test should prove a client/vector/calibration caller
can:

- derive ABI;
- inspect operation schemas;
- plan a request;
- build a transaction;
- obtain signing requests;
- finalize with test signatures;
- inspect construction report;
- generate worst-case transactions;
- do so without linker mutation or model internals.

### 31.9 Target-native execution

The vector package executes finalized transactions against the exact pinned
Elements target.

The transaction package may have local serialization/preflight tests but must
not claim target acceptance by itself.

---

## 32. Determinism and reproducibility

### 32.1 Deterministic semantic planning

Given identical:

- bundle;
- ABI;
- public view;
- request;
- construction policy;

the construction plan is identical.

### 32.2 Deterministic explicit transactions

For explicit-value operations with identical signatures and inputs, final bytes
are identical.

### 32.3 Deterministic confidential fixtures

Canonical test fixtures use explicit fixed randomness and signatures.

They are byte-identical and labeled test-only.

### 32.4 Production confidentiality

Production adapters use secure randomness.

Different randomness may produce different bytes and proofs while preserving
semantic operation.

This is not nondeterministic compiler output; randomness is an explicit
construction input.

### 32.5 Canonical reports

Reports exclude:

- secret values;
- ambient time;
- temporary paths;
- hostnames;
- process IDs;
- random generator internal state.

They include public identities and deterministic construction facts.

---

## 33. Generated artifacts

### 33.1 ABI publication

As described above.

### 33.2 Canonical transaction vectors

Potential artifact:

```text
canonical-transactions.json
```

or a directory of canonical binary/vector files.

It should bind:

- ABI;
- bundle;
- target;
- operation;
- fixture;
- semantic expected result;
- concrete transaction bytes;
- witness bytes;
- expected target verdict.

### 33.3 Sensitive fixture policy

Committed vectors must use:

- test-only keys;
- fixed non-production seeds;
- no production credentials;
- no real private wallet data.

### 33.4 Generator/checker

Every committed transaction artifact requires:

- typed source;
- explicit generator;
- non-writing checker;
- deterministic bytes;
- schema/version;
- identity verification;
- no reverse semantic dependency.

---

## 34. Dependency and unsafe-code policy

The package inherits ADR-011.

Requirements:

- Rust edition 2024;
- workspace MSRV;
- workspace lints;
- `unsafe_code = "deny"`;
- Cargo `--locked`;
- permissive dependencies;
- deterministic explicit-input behavior;
- no hidden network access;
- no private-key logging;
- no build-time node probing.

A cryptographic transaction library may be required.

Its exact version, license, unsafe boundary, and compatibility with the pinned
Elements target must be reviewed.

If a dependency uses unsafe internally, the first-party package can remain
unsafe-free, but dependency trust must be considered under normal dependency
policy.

---

## 35. Performance expectations

Transaction construction must be practical for:

- wallets;
- vectors;
- calibration;
- release tooling.

Priorities:

1. correctness;
2. secret safety;
3. deterministic structure;
4. target compatibility;
5. clear diagnostics;
6. performance.

Potentially expensive work includes:

- rangeproof generation;
- multi-owner signing;
- worst-case transaction generation;
- constructor hashing;
- repeated calibration candidates.

Benchmark only after concrete operations exist.

Caching may be used when keyed by exact:

- target;
- bundle;
- ABI;
- public input;
- representation;
- randomness/signature state as applicable.

Never cache secret material in canonical artifacts.

---

## 36. Non-goals

The transaction package does not:

- define protocol operations;
- redefine formulas;
- emit target programs;
- link constructors;
- select calibrated bounds;
- own a node wallet;
- store private keys;
- broadcast transactions;
- guarantee confirmation;
- choose fee-market winners;
- run independent indexers;
- validate deployment release;
- parse planning documents;
- consume generated publications as first-party semantics;
- support arbitrary transaction layouts outside the linked ABI;
- provide confidential closed-asset identity under the initial profile;
- turn client-policy checks into consensus guarantees.

---

## 37. Implementation milestones

### TX1 — Package and ABI foundation

Deliver:

- workspace package;
- ABI schema;
- identity/error types;
- linked-bundle dependency;
- target dependency;
- candidate/final distinction.

### TX2 — Constructor and metadata instantiation

Deliver:

- typed metadata schemas;
- canonical encoders;
- linked constructor recipes;
- target program/control derivation.

### TX3 — Transaction staging

Deliver:

- input/output ordering;
- version/locktime/sequence;
- unsigned template;
- operation plan.

### TX4 — Witness and signing model

Deliver:

- witness schema;
- signing requests;
- signature collection;
- final witness assembly;
- secret-safe diagnostics.

### TX5 — `compact-ash`

Deliver:

- request;
- public view;
- construction plan;
- final test transaction;
- worst-case fixtures;
- downstream target execution handoff.

### TX6 — Live transfer explicit path

Deliver explicit-value transfer.

### TX7 — Live transfer confidential path

Deliver:

- blinding;
- proofs;
- multi-owner signing;
- minimality vectors.

### TX8 — STATE operation support

Deliver after constructor research acceptance.

### TX9 — Public declassification support

Deliver after research acceptance.

### TX10 — Later operation requests

Add in roadmap order.

### TX11 — Final ABI publication

Deliver once the linked bundle is release-scoped and calibrated.

---

## 38. Phase-4 transaction exit criteria

The transaction package is ready to support end-to-end `compact-ash` evidence
when:

- [ ] `packages/transaction` is a workspace member;
- [ ] package metadata follows workspace policy;
- [ ] dependency direction is acyclic;
- [ ] candidate and final bundle/ABI states are distinct;
- [ ] candidate ABI derives from the exact linked candidate bundle;
- [ ] target and bundle identities are checked;
- [ ] `compact-ash` layout is complete;
- [ ] linked ASH constructor can be instantiated deterministically;
- [ ] ASH metadata/value domains are validated;
- [ ] canonical ASH input ordering is implemented;
- [ ] duplicate inputs reject;
- [ ] one canonical coordinator is selected;
- [ ] exactly one ASH output is derived;
- [ ] output value derives from authenticated public ASH values;
- [ ] sponsor region is isolated;
- [ ] no owner/operator secret is required;
- [ ] target leaf/program and control data match the linked bundle;
- [ ] witness order matches the linked schema;
- [ ] construction reports contain no secrets;
- [ ] final transaction bytes are deterministic from explicit inputs;
- [ ] worst-case candidate transactions are valid and identity-bound;
- [ ] public API tests pass;
- [ ] debug and release tests pass;
- [ ] Clippy with `-D warnings` passes;
- [ ] the checkout remains clean.

Target-native acceptance and complete relation evidence are Phase-4 vector
gates, not transaction-package claims.

---

## 39. Final ABI exit criteria

A final transaction ABI is release-eligible only when:

- [ ] final linked bundle identity verifies;
- [ ] final calibrated bounds are bound;
- [ ] every approved operation has one complete ABI;
- [ ] every family layout is complete and canonical;
- [ ] every constructor and metadata schema resolves;
- [ ] every target program/leaf reference resolves;
- [ ] every witness role has canonical encoding and availability;
- [ ] every permissionless operation has a public construction path;
- [ ] every supported representation has a complete lifecycle path;
- [ ] every formula-derived output is builder-derived rather than caller-chosen;
- [ ] signature requests protect the required output set;
- [ ] confidential proof construction is target-tested where claimed;
- [ ] no confidential closed-asset output is permitted;
- [ ] canonical vectors exist;
- [ ] worst-case transactions exist for every calibrated bound/operation;
- [ ] ABI publication is deterministic;
- [ ] generator/checker paths are separate;
- [ ] ABI identity is bound by the final release profile;
- [ ] no production secret appears in artifacts.

---

## 40. Open questions

### 40.1 Elements Rust library

Select one exact library/version for:

- transaction types;
- serialization;
- sighash;
- CT proof construction;
- target script/control data.

Compatibility with the pinned node target must be demonstrated.

### 40.2 ABI type ownership

Should linked layout/witness schemas be:

- linker-owned and finalized by transaction;
- transaction-owned types constructed by linker;
- target-specific associated types?

Avoid a dependency cycle.

### 40.3 Semantic expression evaluation

Should transaction derive formulas by:

- using the realization evaluator;
- consuming compiler/linker construction recipes;
- using generated typed operation builders?

Requirement:

> no parallel handwritten formula table.

Initial preference:

- linked ABI references typed realization expressions/construction recipes;
- transaction uses the realization evaluator or validated compiled recipe.

### 40.4 Input ordering

Decide per family whether canonical outpoint sorting or caller order is used.

### 40.5 Signing-capability interface

Choose a safe abstraction supporting:

- local keys;
- hardware wallets;
- remote signers;
- multi-party signing;
- test signers.

### 40.6 Confidential proof construction

Blocked on exact Rust library and public-declassification research.

### 40.7 Transaction preflight

Decide whether to embed/use a local target interpreter in addition to
target-native execution.

### 40.8 Fee selection

The package needs a typed fee input but should not own fee-market policy.

Define the boundary between:

- caller-selected fee target;
- ABI validity;
- sponsor envelope;
- calibration fixtures.

### 40.9 Genesis/trusted setup

Decide whether a later deployment module or release package owns genesis and
issuance transaction construction.

### 40.10 Canonical ABI artifact format

Define before first committed ABI publication, not before typed ABI use.

---

## 41. Risks

### 41.1 Transaction builder becomes a second semantic implementation

Handwritten operation logic may drift from realization.

Mitigation:

- ABI-driven construction;
- typed expression references;
- no parallel formulas;
- vectors compare target result;
- operation request restricts caller choices.

### 41.2 Secret leakage

Signing/blinding data may enter errors or reports.

Mitigation:

- secret wrapper types;
- custom redacted `Debug`;
- no generic serialization for secret types;
- ADR-010 diagnostics;
- explicit report projections;
- secret-leak tests.

### 41.3 Signing after mutable outputs

If outputs change after signing, authorization assumptions fail.

Mitigation:

- staged construction;
- signing-ready identity;
- immutable finalized outputs;
- signature transaction binding;
- mutation tests.

### 41.4 Confidential construction complexity

Rangeproof/blinding APIs may introduce target/library divergence.

Mitigation:

- exact dependency pin;
- target-native vectors;
- narrow initial scope;
- public explicit fallback where policy permits;
- no unsupported minimality claim.

### 41.5 Permissionless operation accidentally needs private data

Fixture code may hide the dependency.

Mitigation:

- separate public view;
- witness availability;
- permissionless construction tests;
- no owner-private fixture access.

### 41.6 ABI freezes too early

Pilot ABI may not fit settlement/cycle.

Mitigation:

- schema version;
- pilot/nonfinal status;
- incremental operation support;
- no final publication before broader scope;
- settlement prototype.

### 41.7 Candidate/final confusion

Calibration transactions may be used as deployment transactions.

Mitigation:

- distinct types/identities;
- release validation;
- explicit nonfinal reports;
- no production API for candidate bundles.

### 41.8 Worst-case generator misses a larger branch

Calibration becomes unsound.

Mitigation:

- per-resource objective fixtures;
- relation/branch census;
- target execution;
- manual review;
- shared-bound operation coverage;
- final remeasurement.

### 41.9 Local preflight overclaim

A constructed transaction may still fail target consensus or policy.

Mitigation:

- precise status names;
- target-native execution;
- no “valid” label before target result;
- deployment evidence.

### 41.10 Multi-party interaction burden

Multi-owner burns/transfers require all parties to sign one finalized output
set.

Mitigation:

- explicit signing protocol;
- signing-ready identity;
- deterministic transaction plan;
- client documentation;
- no partial-signature semantic shortcut.

---

## 42. Definition of done

The transaction package plan is fulfilled for the initial deployment when the
repository can derive one deterministic typed ABI from the exact final linked
bundle; accept typed semantic operation requests and public/authorized-private
input views; instantiate every required constructor and metadata schema;
assemble canonical Elements transactions in staged representation and signing
order; materialize exact witnesses and control paths; support permissionless
construction without hidden secrets; support claimed explicit/confidential
value modes without confidential closed-asset identity; generate valid
worst-case calibration transactions; publish canonical secret-free ABI and
transaction vectors; and hand finalized target transactions to vector and
release tooling without redefining semantics, parsing generated publications,
owning private-key custody, querying a node implicitly, or claiming consensus
acceptance before target execution.

---

## 43. One-line package contract

> `transaction` consumes one exact linked bundle, target, typed operation
> request, public chain view, authorized caller witness material, and explicit
> randomness; derives the canonical bundle-bound transaction/witness ABI;
> instantiates constructors and metadata; assembles target transactions in the
> required representation and signing order; generates signing requests,
> witnesses, canonical vectors, and valid worst-case calibration fixtures; and
> does so without redefining protocol formulas, holding production keys,
> depending on hidden private data for permissionless paths, or treating local
> construction as target or deployment acceptance.
