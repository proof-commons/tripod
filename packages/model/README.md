# `tripod-model`

`model` is the executable verification model of the attestation contract: a pure
`World -> Result<World, Guard>` state machine that implements the frozen
architecture manifest and is checked against it.

## Position and boundary

The crate depends on `tripod-architecture` (the typed manifest it
implements) and `tripod-realization` (for conformance projection
only). The dependency direction is one-way: architecture must never depend on
this crate.

It is normative **only after** it compiles and the generated seeded and
property suites pass; until then it is the implementation target corresponding
to the frozen manifest.

### Public-data boundary

The crate accepts and exposes transparent public semantic state. It does not
accept production private keys, credentials, signing nonces, blinding factors,
private openings, or wallet-private state. `OwnerKey` and `SignerSet` are
public *abstract authorization identities*, not key material. Cryptographic
signing and secret custody remain outside the model (ADR-015).

### Trust boundary: a transparent reference model

This is deliberately a transparent reference model, **not** a capability-safe
state machine. The public API makes exactly the following contract, and
downstream consumers must rely on exactly this — no more:

- **`World` is intentionally inspectable and corruptible.** Its state fields
  are public so auditors, fault harnesses, and corruption fixtures can
  construct, examine, and deliberately damage model states. Public mutability
  is a feature of the proof surface, not an encapsulation failure.
- **`execute` is the recommended, invariant-wrapped transition path.** It
  applies a declared operation and re-checks the global invariant on the
  result. `Transition::apply` is also public and performs the operation's own
  guard checks but not the wrapper's invariant re-check; a **valid-world
  precondition** is part of its contract. Applying a transition to a
  hand-corrupted world has no defined semantic meaning beyond what the
  individual guards catch.
- **The sealed `Transition` trait closes the operation vocabulary, not the
  state space.** External crates cannot add new transition kinds, but they can
  freely assemble or mutate worlds without going through `execute`. Rust does
  not — and this crate does not claim to — prevent that.
- **Low-level kernel transaction construction is crate-private.** `TxBuilder`
  and the staging/commit machinery are unreachable from outside, so every
  *declared-operation* path does flow through the operation constructors.
- **Evidence rule.** A world is *model-valid evidence* only when every
  *protocol* state change in it was produced by `genesis` followed by a chain
  of successful `execute` calls. Any other protocol change — direct
  construction, public field mutation, raw `apply` on corrupted state — is
  corruption evidence for fault and differential testing, and must be labeled
  as such by the harness that produced it.

  The rule governs protocol transitions, not the environment they run in. Two
  things evolve a world without the protocol acting: block age advancing, and
  externally funded open objects arriving. Both are *substrate* facts — the
  chain moves time and third parties send coins whether or not this protocol
  does anything — so neither carries a transition certificate. A harness may
  therefore advance `pace_age_blocks` or call `inject_open_object` without
  invalidating the evidence, and must still route every protocol operation
  through `execute`. Consequently no invariant may depend on how far time
  advanced, in what order external funds arrived, or on that history being
  final: a reorganization can rewrite substrate history and the model must
  survive it.

## Quickstart

Construct a world, apply a declared operation, re-check the invariant.

```rust
use model::{
    AnnounceMaturity, BranchKind, CanonicalOrder, Constants, FeeEnvelope, OPERATOR_KEY, Ratio,
    Sat, TxId, bind_execution, check_invariant, execute, execute_bound, genesis,
};

// 1. Declare the finite bounds. `Ratio` has private fields, so
//    `Ratio::new` is the only constructor and an out-of-domain ratio is
//    unconstructible rather than merely rejected at use.
let constants = Constants {
    pool_id: 7,
    zeta: Ratio::new(1, 2).unwrap(),
    mint_fee: Ratio::new(1, 2).unwrap(),
    min_maturity_lead: 10,
    max_maturity_lead: 1000,
    min_cadence_blocks: 10,
    max_cadence_blocks: 100,
    admission_batch_max: 32,
    settlement_batch_max: 32,
    relabel_batch_max: 32,
    ash_batch_max: 64,
    burn_input_max: 64,
    burn_change_max: 32,
    burn_record_max: 64,
    transfer_input_max: 64,
    transfer_output_max: 64,
    fee_sponsor_input_max: 16,
};
assert_eq!(Ratio::new(1, 0), Err(model::Guard::BadConstant));

// 2. Genesis is the only origin of a valid world. It validates the
//    constants against the architecture's cardinality minima first.
let world = genesis(
    constants,
    Sat::new(1_000_000).unwrap(),
    CanonicalOrder { height: 0, tx_index: 0 },
    TxId([0_u8; 32]),
)
.unwrap();

// 3. Build a declared operation and apply it through the normative,
//    invariant-wrapped entry point.
let announce = AnnounceMaturity {
    maturity_cycle: world.constants.min_maturity_lead,
    signers: std::iter::once(OPERATOR_KEY).collect(),
    fee_envelope: FeeEnvelope::default(),
};
let order = CanonicalOrder { height: 0, tx_index: 1 };

let next = execute(&world, &announce, order).unwrap();

// 4. Consume. The successor carries exactly one new certificate.
check_invariant(&next).unwrap();
assert_eq!(
    next.history.transitions.last().unwrap().branch,
    BranchKind::AnnounceMaturity,
);
assert_eq!(
    next.history.transitions.len(),
    world.history.transitions.len() + 1,
);

// The bound form retains predecessor / request / successor together.
let bound = execute_bound(&world, announce.clone(), order).unwrap();
assert_eq!(bound.before(), &world);
assert_eq!(bound.certificate().order, order);
assert_eq!(bound.after(), &next);

// The same binding can be re-established later by deterministic replay.
let replayed = bind_execution(&world, announce, &next).unwrap();
assert_eq!(&replayed, &bound);

let successor = bound.into_world();
check_invariant(&successor).unwrap();

// 5. Substrate movement is not a protocol transition. The chain moves time
//    whether or not this protocol acts, so no certificate is appended --
//    and no invariant may depend on how far it moved.
for blocks in [0_u64, 1, 144, 10_000, u64::from(u32::MAX)] {
    let mut moved = successor.clone();
    let before = moved.history.transitions.len();

    moved.pace_age_blocks += blocks;

    assert_eq!(moved.history.transitions.len(), before);
    check_invariant(&moved).unwrap();
}
```

## Public-API tour

The crate root re-exports everything below, so `use model::X` works throughout;
module paths are given for orientation.

### Constructing a world

- `constants::Constants` — all-public finite bounds and ratios: `pool_id`,
  `zeta`, `mint_fee`, `min_maturity_lead`, `max_maturity_lead`,
  `min_cadence_blocks`, `max_cadence_blocks`, `admission_batch_max`,
  `settlement_batch_max`, `relabel_batch_max`, `ash_batch_max`,
  `burn_input_max`, `burn_change_max`, `burn_record_max`,
  `transfer_input_max`, `transfer_output_max`, `fee_sponsor_input_max`. Method
  `validate(&self) -> Result<(), Guard>` (`Guard::BadConstant` on a bad
  bound).
- `genesis::genesis(Constants, Sat, CanonicalOrder, TxId) -> Result<World, Guard>`
  — the trusted-setup constructor and the only origin of a valid world. It
  validates the constants *and* their conformance with the architecture's
  cardinality minima before building anything: a world whose finite bounds
  cannot satisfy those minima must not exist, rather than merely fail profile
  validation later.
- `genesis::GENESIS_OWNER` and `genesis::OPERATOR_KEY` — the well-known
  abstract authorization identities (both all-zero `OwnerKey`).
- `world::World` — all fields public: `utxos: BTreeMap<OutPoint, Utxo>`,
  `roots: RootCursor`, `wallets: Wallets`, `adversary: ExternalBudget`,
  `history: History`, `constants: Constants`, `next_outpoint`,
  `next_tx_nonce`, `pace_age_blocks`. Readers include
  `utxo(OutPoint) -> Result<&Utxo, Guard>`,
  `state() -> Result<(OutPoint, PoolState), Guard>`, and
  `active_resv() -> Result<(OutPoint, &Utxo), Guard>`.
- `world::RootCursor`, `world::Wallets`, `world::ExternalBudget`.

### Scalars and exact arithmetic (`scalar`)

`Sat` and `Ratio` have **private fields**; `Sat::new` and `Ratio::new` are the
only constructors, so every rejection is a rejection everywhere. `Ratio::new`
requires a proper in-domain fraction — `Ratio::new(1, 0)`, `(0, 2)`, `(2, 2)`,
`(3, 2)`, and `(1, 1024)` all return `Err(Guard::BadConstant)` — with readers
`numerator()` and `denominator()`.

Transparent newtypes with public bodies: `OwnerKey([u8; 32])`,
`AttestationAddress([u8; 32])`, `TxId([u8; 32])`, `BlockHash([u8; 32])`.
One record, `CanonicalOrder { height: BlockHeight, tx_index: TxIndex }`. The
remaining names are plain type aliases, not distinct types — `OutPoint`,
`Cycle`, `BlockHeight` are `u64`; `TxIndex` and `SchemaVersion` are `u32` — so
the compiler will not stop you mixing them up.

Checked arithmetic, all fail-closed with a `Guard`: `checked_add_to_map`,
`checked_sum_sats`, `checked_active_backing`, `floor_mul_div`, `floor_ratio`.
Constants: `ACTIVE_BACKING_MAX`, `TWO_51`.

### Applying operations (`ops`, `transition`)

Twelve declared operation constructors, each a plain public struct implementing
the sealed `Transition` trait: `CreateRequest`, `CancelRequest`,
`AdmitDeposits`, `RunCycle` (with `CycleCaller`), `SettleDistribution`,
`TransferReceipts` (with `ReceiptDestination`), `RedeemReceipt`,
`RelabelReceipts`, `BurnReceipts`, `CompactAsh`, `ClearAsh`,
`AnnounceMaturity`. Plus `inject_open_object` — a *substrate* action, not a
protocol transition, for placing externally funded open objects.

- `trait Transition: sealed::Sealed` with
  `apply(&self, &World, CanonicalOrder) -> Result<World, Guard>`. Sealed
  through a crate-private marker module, so an external `impl Transition` fails
  to compile. `apply` is public but unwrapped; its valid-world precondition is
  part of the contract.
- `execute<T: Transition>(&World, &T, CanonicalOrder) -> Result<World, Guard>`
  — the normative entry point: `apply` followed by `check_invariant`, mapping
  an invariant failure to `Guard::InvariantFailure`.
- `execute_bound<T>(&World, T, CanonicalOrder) -> Result<ExecutedTransition<T>, Guard>`
  — `execute` plus the requirement that the successor extend the predecessor by
  exactly one certificate carrying that order.
- `bind_execution<T>(&World, T, &World) -> Result<ExecutedTransition<T>, ExecutionBindingError>`
  — re-establish the binding for a successor produced earlier, by deterministic
  replay. Replay uses only model execution; realization evaluation is never
  consulted, so the binding check cannot become circular with conformance.
  Errors: `InvalidSuccessorExtension`, `RequestBindingMismatch`.
- `ExecutedTransition<T>` — private fields, no unchecked constructor (a
  `compile_fail` doctest on the type is the standing proof). Readers
  `before()`, `request()`, `after()`, `certificate() -> &TransitionCertificate`,
  and `into_world()`. Holding one is evidence that `request` applied to
  `before` under the invariant wrapper yields `after` — the binding conformance
  projection requires, so an observation cannot mix the certificate of one
  execution with the authorization data of another.

Because every transition takes `&World` and returns a fresh `World`, an error
leaves UTXOs, wallets, adversary budget, roots, history, and cadence state
unchanged by construction.

### Checking a world (`invariant`, `manifest`, `queries`)

- `check_invariant(&World) -> Result<(), InvariantError>` — the full invariant
  over state and history.
- `clause_of(InvariantError) -> architecture::InvariantClauseId` — maps a
  failure to the architecture invariant clause it violates. `AccountingFold` is
  the fold the check is built on.
- `validate_architecture_conformance() -> Result<(), Guard>` — the model's
  declarations agree with the architecture manifest.
- `validate_bound_conformance(&Constants) -> Result<(), Guard>` and
  `validate_profile_bound_conformance(..)` — finite bounds meet the manifest
  minima (the first is what `genesis` calls).
- `bound_value(&Constants, architecture::BoundId) -> usize`,
  `declared_asset(Asset) -> Option<architecture::AssetId>`,
  `asset_of(architecture::AssetId) -> Asset`,
  `branch_operation(BranchKind) -> architecture::OperationId`,
  `operation_branch(architecture::OperationId) -> BranchKind` — the
  manifest/model translation pairs, both total and both `const`.
- `assert_no_attestation_singleton(&World) -> Result<(), Guard>`.
- Read-only queries: `floor_terms(&World) -> Result<(Sat, Sat), Guard>`,
  `redemption_payout(&World, Sat) -> Result<Sat, Guard>`,
  `cycle_issuance_query(&World) -> Result<Sat, Guard>`.

### Reading state (`recognition`, `shape`, `policy`, `signer`, `fee`)

- `classify_state_object(&World, OutPoint, &Utxo) -> StateClass` — the
  projection classifier.
- Branch-specific readers, each `Result<_, Guard>`: `read_receipt` →
  `ReceiptView`, `read_entitlement` → `EntitlementView`,
  `read_distribution_control` → `DistributionControlView`, `read_ash` → `Sat`;
  plus `validate_request_for_admission(..)` and the `RequestView` /
  `CanonicalObject` types.
- `ObjectKind`, `ShapePolicy` — operation-independent branch-shape validation.
- `branch_policy(BranchKind) -> BranchPolicy`,
  `expected_value_flow_classes(BranchKind) -> BTreeSet<ValueFlowClass>`,
  `RootUse`.
- `SignerSet` and `require_signer(&SignerSet, OwnerKey) -> Result<(), Guard>`.
- `FeeEnvelope` (implements `Default`), `FeeChange`, and
  `validate_fee_envelope(&World, &FeeEnvelope) -> Result<Sat, Guard>`.

### History and objects (`history`, `object`, `asset`, `pool`)

`History`, `TransitionCertificate`, `BranchKind`, `RootEdge`, `CanonicalDelta`,
`CanonicalDeltaFamily`, `DeltaKind`, `OpenFlowKind`, the certified-flow family
(`CertifiedCanonicalFlow`, `CertifiedCanonicalPartition`,
`CertifiedDestructionLeg`, `CertifiedIssuance`), and the derived projections
(`GenesisProjection`, `BurnProjection`, `BurnRecord`, `ClearProjection`,
`OpenFlowProjection`, `DistributionResidueProjection`).

Objects: `Utxo`, `Meta`, `Tag`, `DataOutput`. Assets: `Asset`, `ReceiptClass`,
`Maturity`. Pool: `PoolState`.

### Conformance against `realization` (`conformance`)

This module **never participates in transition acceptance**. Constructors and
the kernel execute first; only after a successful transition does it project
primitive facts for a realization check.

- `observe_compact_ash(&ExecutedTransition<CompactAsh>) -> Result<ModelConformanceObservation, ConformanceProjectionError>`
- `observe_live_transfer(&ExecutedTransition<TransferReceipts>) -> Result<..>`
  — refuses a non-`Live` receipt class with `WrongBranch`.
- `ModelConformanceObservation` — private fields, producible only by those
  adapters from a bound execution. Readers `observation() -> &realization::OperationObservation`
  and `established_evidence()`. The established set is **model-side evidence
  only** — never target, deployment, or independent evidence.
- `unresolved_model_evidence(&realization::ConformanceReport, &ModelConformanceObservation) -> BTreeSet<realization::ExternalEvidenceRequirement>`
  — the premises the model side does not discharge. The pilot harnesses require
  this to be empty; each remaining entry is a premise only target evidence can
  close.

### Indexer and ledger (`ledger`)

Exact attestation indexer, canonical serializer/decoder, reorg-aware
checkpoint, and the split event/query differential comparisons.
Types: `ReferenceIndexer`, `IndependentAttestationIndexer`,
`ModelIndexerCheckpoint`, `ValidatedChainView`, `CanonicalBlock`,
`BurnTransaction`, `ClearEntry`, `ClearId`, `AttestationContext`,
`AttestationTerm`, `AttestationQueryProvider`, `AttestationQueryResult`,
`AttestationEventId`, `AttestationEventSnapshot`, `OrderedAttestationEvent`,
`RecognizedAttestationEvent`, `IndexerDiagnosticSnapshot`, `ExactRational`.
Codec: `encode_varint` / `decode_varint`, `encode_biguint` / `decode_biguint`,
`serialize_query` / `deserialize_query`. Checks: `validate_event_index`,
`validate_query`, `compare_attestation_events`, `compare_attestation_query`,
`compare_attestation_indexers`, `expected_architecture_manifest_hash`.
Constants: `ATTESTATION_SCHEMA_VERSION`, `ATTESTATION_QUERY_DOMAIN`.

### Harness surfaces (`maintenance`, `quiescence`, `audit`, `property`)

These exist for proofs and test drivers, not for production operation.

- `maintenance` — `MaintenanceAction`, `MaintenanceMode`, `MaintenanceSponsor`,
  the `MaintenanceScheduler` trait and `DeterministicMaintenanceScheduler`,
  `StateCandidate`, `AdmissionCapacityPlan`, plus the drivers
  `apply_maintenance_action`, `drive_shared_state_to_fixpoint`,
  `drive_quiescence_with_scheduler`, `drive_sponsored_quiescence`,
  `drive_shared_state_sweepability`, `maintenance_phase_potential`,
  `admission_capacity_plan`, `capacity_admissible_request_batch`,
  `next_model_order`. **The scheduler is a test harness, not covenant state.**
  The drivers prove that valid collection transitions exist and can reach the
  stated residual bounds when a scheduler supplies inclusion and fees. They do
  not prove that a real fee market will fund every transition; the zero-fee
  scheduler is a reachability harness, not an economic claim.
- `quiescence` — `QuiescenceReport`, `QuiescenceOutcome`,
  `QuiescenceEligibility`, `QuiescenceResidual`, `ProtocolObservable`,
  `QuantityId`, with `lifecycle_report`, `classify_quiescence_eligibility`,
  `quiescence_residuals`, `residuals_match_report`, `shared_state_is_swept`,
  `protocol_observable`, `quantity_reads_residue`, and the noninterference
  assertions `assert_protocol_noninterference`,
  `assert_residue_noninterference`, `assert_residue_reader_policy`,
  `perturb_residue_projection`.
- `audit` — `receipt_accounting_audit`, `compare_receipt_accounting_audit`,
  `ReceiptAccountingAuditProjection`, `ResidueAuditEvent`, under the
  external-auditor role.
- `property` — the property-test state machine: `PropertyAction`,
  `PropertyActionSeed`, `PropertyStepResult`, `apply_property_action`,
  `materialize_action`, `property_action_name`, `drive_property_trace`,
  `drive_property_seed_trace`, `fund_property_world`, and the fixed identities
  `PROPERTY_OWNER_A/B/C`, `PROPERTY_ADDRESS_A/B`, `PROPERTY_SPONSOR`.

### Artifacts (`artifacts`)

The committed generated publications. `generated/declassification.json` is a
**provisional, derivative publication**: the planned compiler derives
declassification from the typed realization dependency graph and never ingests
that JSON.

## Error handling

Five error types, each covering one boundary. None of them is a subtype of
another, and none is convertible into another.

| Type | Produced by | Meaning |
|---|---|---|
| `Guard` | every operation, query, reader, and `execute` | a transition or read was refused |
| `InvariantError` | `check_invariant` | a global invariant clause does not hold |
| `ExecutionBindingError` | `bind_execution` | a request/successor pair does not bind |
| `ConformanceProjectionError` | the `observe_*` adapters | a transition could not be projected into realization facts |
| `DecodeError` / `EncodeError` / `DifferentialError` / `QueryValidationError` | `ledger` codec and comparison functions | serialization or differential failure |

`Guard` is the model's refusal vocabulary. Its variants group as:

- **Resolution and shape** — `NoSuch`, `WrongAsset`, `WrongShape`, `WrongPool`,
  `WrongTarget`, `WrongClass`.
- **Domain and arithmetic** — `Domain`, `BadConstant`, `Overflow`, `Underflow`,
  `CycleOverflow`. `BadConstant` is what an out-of-domain `Ratio::new` or
  `Constants::validate` returns.
- **Transaction structure** — `DuplicateInput`, `DuplicateOutputIndex`,
  `MissingOutputIndex`.
- **Authorization** — `BadSignature`, `BadAuthorization`, `MissingAuthority`.
- **Lifecycle** — `Sealed`, `NoTrap`, `ZeroProgress`.
- **Value flow** — `OverDraw`, `ValuePin`, `PartitionPin`, `RecipientPin`,
  `ClassCross`, `BadIssuance`, `BadDestruction`, `CanonicalDeltaMismatch`,
  `ActiveBackingCapExceeded`.
- **Root succession** — `RootSuccession`, `ResvWeld`, `ControlVaultWeld`, and
  `RootMultiplicity`, which is declared for the emitted-script vocabulary and
  is **never produced at model level** (duplicate root outputs are
  unrepresentable in a map-keyed world).
- **Cadence and maturity** — `CadenceTooEarly`, `CadenceOperatorOnly`,
  `MaturityAlreadyAnnounced`, `MaturityLeadTooShort`, `MaturityLeadTooLong`,
  `MaturityNotComplete`.
- **Fees and sponsorship** — `FeeMismatch`, `SponsorMismatch`,
  `OpenFlowMismatch`.
- **Ledger** — `WrongCheckpoint`, `UnsupportedSchema`, `HistoryOrder`,
  `DuplicateEvent`.
- **`InvariantFailure`** — returned by `execute` when the operation succeeded
  but the wrapper's `check_invariant` did not. Seeing this from `execute` on a
  world you built through `genesis` and `execute` is a model defect, not a
  caller error.

`InvariantError` names the failing clause family: `IdentityAuthority`,
`Domains`, `Floor`, `SealedTerminal`, `Backing`, `DistributionPayability`,
`EntitlementLifecycle`, `ReceiptAccountingPreMaturity`,
`ReceiptAccountingPostMaturity`, `MaturityCoherence`,
`ConsensusValueAuthority`, `StateSuccession`, `ResvSuccession`,
`CanonicalClosure`, `ActiveBackingCap`, `HistoryProjection`. Use `clause_of`
to reach the corresponding architecture invariant clause.

`ConformanceProjectionError` covers observation-projection failures:
`NoTransition`, `WrongBranch { expected, actual }`, the missing/unknown object
and flow family (`MissingConsumedObject`, `MissingCreatedObject`,
`UnknownFlowSource`, `UnknownFlowDestination`, `UnknownDeltaSource`,
`UnknownDeltaDestination`), history-shape violations
(`NotOneTransitionExtension`, `HistoryPrefixChanged`, `NonIncreasingOrder`,
`HistoryLengthOverflow`), object-lifecycle violations
(`ConsumedCreatedOverlap`, `ConsumedObjectSurvived`,
`CreatedObjectAlreadyExisted`), domain violations (`AmountOutOfDomain`,
`BoundOutOfDomain`, `UndeclaredCanonicalAsset`),
`SponsorProtocolRegionOverlap`, and `InvalidObservation` wrapping a
`realization::RealizationError`.

## What this package deliberately does not do

- It does not sign anything, hold key material, or accept secrets. `OwnerKey`
  and `SignerSet` are abstract authorization identities (ADR-015).
- It does not emit or execute target script. No opcode, tapleaf, control block,
  or target byte appears here.
- It does not encapsulate `World`. Public mutability is deliberate; the
  evidence rule, not the type system, is what makes a world count as evidence.
- It does not let an external crate add a transition kind — `Transition` is
  sealed — and does not expose `TxBuilder`, the kernel, or the flow/issuance
  declaration types, which are crate-private.
- It does not claim to be normative before its generated seeded and property
  suites pass.
- It does not decide realization conformance. The `observe_*` adapters run
  *after* a successful transition and never gate acceptance; asking the
  realization evaluator whether a transition should be accepted would make the
  comparison circular.
- It does not claim a real fee market funds maintenance. The schedulers and
  drivers are reachability harnesses.
- It does not ingest its own generated publications; those are derivative
  outputs.

## Relationship to neighbors

- **`tripod-architecture`** is the frozen typed manifest this crate
  implements. The dependency is one-way, and the agreement is checked in-crate
  by `validate_architecture_conformance` and `validate_bound_conformance` —
  the latter is called by `genesis`, so a world with non-conforming bounds
  cannot be constructed at all.
- **`tripod-realization`** supplies the target-independent semantic
  relations this crate is checked *against*. The `conformance` module projects
  executed transitions into `realization::OperationObservation` values; the
  realization side evaluates them and returns a `ConformanceReport`. Model
  execution never consults the realization evaluator.
- **`tripod-compiler`** does not depend on this crate and this crate
  does not depend on it. Both consume `realization` independently, which is
  what keeps model conformance and compiler analysis genuinely separate
  checks of the same semantics.
- **`tripod-artifacts`** owns the committed generated publications
  under `generated/`.
