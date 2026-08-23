# `tripod-architecture`

`architecture` is the typed normative architecture manifest of the attestation
realization: the finite identifier vocabulary, the static declaration built
from it, validation, canonical semantic hashing, and the Serde export model
behind the two published artifacts `architecture.json` and `architecture.toml`.

## Position and boundary

The crate is the root of the first-party dependency graph and depends on no
other first-party package. It is a leaf in the other direction too: nothing in
it opens a file, reads an environment variable, or touches the network. Its
entire input is the [`ARCHITECTURE`](src/spec.rs) constant compiled into it,
plus whatever typed `Architecture` value a caller passes in; everything it
produces is a deterministic function of that value.

It deliberately contains no UTXO state, branch execution, script
implementation, indexer database, or property-test state machine. Those live in
`tripod-model`, which depends on this crate and is checked against
it. The dependency direction is one-way: this crate must never depend on the
executable model.

Two assurance claims are kept apart here, and neither is a deployment claim.
`validate_architecture_release` establishes that the typed manifest is
internally consistent, anchor-set pinned, and final. `validate_deployment_profile`
establishes that one deployment-evidence profile is structurally complete and
bound to that manifest. Production-release validity is a third question, and
under the current profile schema (2) `validate_production_deployment_release`
refuses every profile, because schema 2 has nowhere to bind the transaction ABI
and configuration its calibrations were measured under. No deployment release
is constructed by this crate, and none can be until a schema binds the ABI.

The deployment-profile identity is likewise dormant: the hash function exists
and is defined only over a validated profile, but no consumer, publication, or
generated artifact carries a deployment-profile hash today.

## Quickstart

Validation precedes identity (`[ADR021-rule:identity:admission-order]`), so
the workflow is fixed: start from
the typed manifest, validate it, and pass the *validated wrapper* — never a raw
`Architecture` — to every identity, projection, and publication function.

```rust
use architecture::{
    ARCHITECTURE, OperationId, PublishedArchitecture, behavioural_hash_hex,
    semantic_hash_hex, validate_architecture_release,
};

// 1. Construct: the shipped manifest is a compile-time constant. Any
//    `Architecture` value works; this one is the normative declaration.
let architecture = &ARCHITECTURE;

// 2. Validate. Release validation subsumes draft validation and adds the
//    attestation anchor-set pin and a final publication status. On failure it
//    returns *every* defect it found, not just the first.
let release = validate_architecture_release(architecture)
    .expect("the shipped manifest is release-valid");

// 3. Consume. Identity functions accept the draft-validated view, which a
//    release yields for free.
let semantic = semantic_hash_hex(&release.draft()).unwrap();
let behavioural = behavioural_hash_hex(&release.draft()).unwrap();
assert_eq!(semantic.len(), 64);
assert_ne!(semantic, behavioural);

// Typed lookups go through the manifest, keyed by architecture-owned ids.
let compact_ash = release
    .architecture()
    .operation(OperationId::CompactAsh)
    .expect("declared operation");
assert_eq!(compact_ash.id.as_str(), "compact-ash");

// The publication envelope re-derives both hashes and can re-check itself.
let published = PublishedArchitecture::from_architecture(&release.draft()).unwrap();
assert_eq!(published.semantic_hash, semantic);
assert_eq!(published.behavioural_hash, behavioural);
published.validate_release_envelope().unwrap();

// The two artifact renderings are presentation encodings, never hash inputs.
let json = published.to_artifact_json().unwrap();
let toml = published.to_artifact_toml().unwrap();
assert!(!json.is_empty() && !toml.is_empty());
```

## Public-API tour

Every item below is re-exported at the crate root, so `use architecture::X`
works for all of them; the module paths are given for orientation.

### `ids` — the stable identifier vocabulary

Glob-re-exported (`pub use ids::*`). These are the finite registries the whole
first-party graph keys on, and downstream packages are required to cite them
rather than mint local restatements.

Subject ids: `AssetId`, `ReceiptClassId`, `RootId`, `OperationId`, `ObjectId`,
`InvariantClauseId`, `WitnessId`, `QuantityId`, `DataId`, `DependencyId`,
`DecisionId`, `AmountLimitId`, `BoundId`, `ProjectionId`, `TagId`.

Classification enums: `ValueFlowClass`, `OpenFlowKind`, `DeltaKind`,
`InputAuthorization`, `DeltaCondition`, `DataOutputKind`,
`AuthorizationEvidenceKind`, `RootUse`, `OperationKind`, `PermissionClass`,
`AssetClass`, `AssetRole`, `RootRole`, `QuantityKind`, `DecisionStatus`,
`PublicationStatus`, `ProjectionRule`, `LimitKind`, `IssuanceCondition`,
`LifecycleClass`, `AccountingDomain`, plus the hand-written `AllocatorId`,
`DeallocatorId`, `ReaderId`, and the `AuthorizationEvidence` record.

Each id enum is generated by one macro and therefore carries the same surface:

- `Self::ALL: &'static [Self]` — the complete census, in declaration order.
- `fn code(self) -> u16` — the explicit discriminant. It is part of the
  canonical semantic encoding: variants are never reordered or renumbered after
  publication, only added under a new schema version.
- `fn as_str(self) -> &'static str` — the canonical textual name used in the
  exports; `Display` writes the same string.
- Derived `Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash`, so ids
  are usable as `BTreeMap`/`BTreeSet` keys and sort canonically.

There is no `FromStr` and no parser: text never becomes an id inside this
crate.

### `spec` — the declaration

- `Architecture` — the manifest struct. Its fields are public `&'static [..]`
  slices: `document`, `assets`, `roots`, `objects`, `operations`, `quantities`,
  `witnesses`, `clauses`, `dependencies`, `decisions`, `bounds`,
  `amount_limits`, `tags`.
- Lookup helpers, each a linear scan returning `Option<&T>` and `None` for an
  id the manifest does not declare:
  `asset(AssetId)`, `root(RootId)`, `object(ObjectId)`, `operation(OperationId)`,
  `quantity(QuantityId)`, `bound(BoundId)`, `amount_limit(AmountLimitId)`.
- `ARCHITECTURE: Architecture` — the normative constant. The per-category
  constants it is assembled from (`DOCUMENT`, `ASSETS`, `ROOTS`, `OPERATIONS`,
  `QUANTITIES`, `WITNESSES`, `DEPENDENCIES`, `DECISIONS`, `BOUNDS`, `TAGS`,
  `OBJECTS`, `AMOUNT_LIMITS`, `CLAUSES`) are public too, for consumers that
  want one registry without the whole manifest.
- `ARCHITECTURE_SCHEMA_VERSION: u32` — the schema the constant declares.
- Record types: `DocumentSpec`, `SpecificationBinding`, `AssetSpec`, `RootSpec`,
  `RootUseSpec`, `IssuanceSpec`, `MaxCount`, `InputSpec`, `OutputSpec`,
  `CanonicalDeltaSpec`, `DataOutputSpec`, `ProjectionSpec`, `ObjectSpec`,
  `OperationSpec`, `QuantitySpec`, `WitnessSpec`, `DependencySpec`,
  `DecisionSpec`, `BoundSpec`, `AmountLimitSpec`, `TagSpec`.

This module is static data, not executable policy. It computes nothing.

### `validate` — verdicts carried in the type

- `validate_draft(&Architecture) -> Result<ValidatedDraftArchitecture<'_>, Vec<ManifestError>>`
  — internal consistency: unique ids, resolvable references, coherent assets,
  roots, objects, operations, quantities, bounds, decisions, and document
  metadata. Accumulates *all* errors.
- `validate_architecture_release(&Architecture) -> Result<ValidatedReleaseArchitecture<'_>, Vec<ManifestError>>`
  — runs draft validation and additionally requires a pinned attestation anchor set
  (an all-zero digest counts as unset) and `PublicationStatus::Final`. It makes
  no deployability claim.
- `ValidatedDraftArchitecture<'a>` — `Copy`; only constructor is
  `validate_draft`; `architecture() -> &'a Architecture`. Holding one is proof
  that draft validation accepted the value, which is why every identity
  function takes it.
- `ValidatedReleaseArchitecture<'a>` — strictly stronger; `architecture()` and
  `draft() -> ValidatedDraftArchitecture<'a>`.

### `canonical` — projection and identity

- `SEMANTIC_HASH_ALGORITHM: &str` (`"sha256-canonical-json-v3"`),
  `BEHAVIOURAL_HASH_ALGORITHM: &str`
  (`"sha256-canonical-json-behavioural-v3"`),
  `ANCHOR_SET_HASH_ALGORITHM: &str` (`"sha256-anchor-set-v2"`), and the
  retired-identifier registers `RETIRED_SEMANTIC_HASH_ALGORITHMS`,
  `RETIRED_BEHAVIOURAL_HASH_ALGORITHMS`, and
  `RETIRED_ANCHOR_SET_HASH_ALGORITHMS: &[&str]`. Every recipe is
  domain-separated; the anchor-set identifier is a code-side register, not a
  published manifest field.
- `canonical_json_bytes(&ValidatedDraftArchitecture) -> Result<Vec<u8>, serde_json::Error>`
  — the canonical encoding: sorted object keys, set-like arrays sorted during
  export conversion, insensitive to comments, formatting, and declaration
  order. TOML and pretty-printed JSON are presentation encodings and are never
  hash inputs.
- `semantic_hash` / `semantic_hash_hex` — SHA-256 over the whole canonical
  export body (not the envelope, which contains the hash), under a
  domain-separation prefix. `canonical_json_bytes` returns the projection
  without that prefix: the separator belongs to the digest, not the encoding.
- `behavioural_hash` / `behavioural_hash_hex` — SHA-256 over the behavioural
  arrays only (assets, roots, objects, operations, quantities, witnesses,
  clauses, bounds, amount limits, tags), under a domain-separation prefix;
  dependencies, decisions, and envelope metadata are excluded. The versioning
  gate keys on this hash: if it moves while `realization_version` is unchanged,
  the build fails.
- `anchor_set_hash(impl IntoIterator<Item = &str>) -> [u8; 32]` — the attestation anchor-set
  pin recipe: SHA-256 over the domain prefix followed by
  `join("\n", sorted distinct anchor names)`. Sorting and deduplication happen
  inside, so any occurrence order with repeats is accepted.
- `hex(&[u8]) -> String` — lowercase hex helper.

The unchecked projections over an unvalidated `Architecture` are deliberately
crate-private: rehashing is not revalidation, so a manifest with duplicate
declarations or a missing root has no semantic identity through any public
path.

### `export` — DTOs and the publication envelope

- `ArchitectureExport` — the owned Serde mirror of the manifest body, with
  `from_architecture(&Architecture) -> Self`. Sub-DTOs: `SpecificationExport`,
  `AssetExport`, `RootExport`, `RootUseExport`, `AllocatorExport`,
  `DeallocatorExport`, `ObjectAuthorizationExport`, `ObjectExport`,
  `IssuanceExport`, `MaximumExport`, `InputExport`, `OutputExport`,
  `CanonicalDeltaExport`, `DataOutputExport`, `ProjectionExport`,
  `OperationExport`, `ReaderExport`, `QuantityExport`, `WitnessExport`,
  `ClauseExport`, `DependencyExport`, `DecisionExport`, `BoundExport`,
  `AmountLimitExport`, `TagExport`, `AuthorizationEvidenceExport`,
  `OperationAuthorizationEvidenceExport`.
- `PublishedArchitecture` — the envelope: `publication_status`,
  `architecture_schema_version`, `realization_version`, both hash algorithm
  names, both hashes, and the `architecture` body. Schema 17 lifted the version
  fields out of the hashed body precisely so a schema or letter bump cannot
  move either hash.
  - `from_architecture(&ValidatedDraftArchitecture) -> Result<Self, serde_json::Error>`
  - `verify_body_hash()` / `verify_behavioural_hash() -> Result<bool, serde_json::Error>`
  - `verify_hashes() -> Result<(), EnvelopeError>`
  - `validate_envelope() -> Result<(), EnvelopeError>` — use this when
    ingesting a published artifact; it checks algorithm names, schema version,
    status, version syntax, and both hashes.
  - `validate_release_envelope() -> Result<(), EnvelopeError>` — envelope
    validation plus the release obligations.
  - `validate_against_expected(&Self) -> Result<(), EnvelopeError>`
  - `to_artifact_json() -> Result<String, serde_json::Error>` and
    `to_artifact_toml() -> Result<String, toml::ser::Error>` — the exact
    presentation bytes the generator, the stale-artifact checker, and the
    conformance tests compare against.

### `deployment` — the separate evidence boundary

- `DEPLOYMENT_PROFILE_SCHEMA_VERSION: u32` (currently 2) and
  `DEPLOYMENT_HASH_ALGORITHM: &str`
  (`"sha256-canonical-json-deployment-v1"`), domain-separated from the manifest
  algorithm: the architecture recipe is never reused for profile bytes.
- `DeploymentProfile` — calibrated bounds, verified dependencies, artifact
  hashes, and network/genesis identity bound to one architecture semantic hash.
  Components: `BoundCalibration`, `DependencyEvidence`, `ArtifactHashes`,
  `TestEvidence`, `ScriptLimits`, `VerificationStatus` (with
  `as_str(self) -> &'static str`).
- `manifest_minimum_for_bound(&Architecture, BoundId) -> u64` — the floor a
  calibration must clear.
- `validate_deployment_profile_structure(&Architecture, &DeploymentProfile) -> Result<(), Vec<DeploymentError>>`
  — the structural check, accumulating every defect.
- `validate_deployment_profile(&Architecture, &DeploymentProfile) -> Result<ValidatedPreReleaseDeploymentProfile<'_>, Vec<DeploymentError>>`
  — the same check, with the verdict carried in the type. The wrapper exposes
  `architecture()` and `profile()` and is the sole entry to profile identity.
- `validate_production_deployment_release(&Architecture, &DeploymentProfile) -> Result<(), Vec<DeploymentError>>`
  — **always returns `Err`** under schema 2. It runs structural validation
  first so a caller learns both the profile's own defects and the schema's
  limit, then appends
  `DeploymentError::ProductionReleaseUnsupported`. It exists rather than being
  omitted so the release question has a truthful answer instead of the
  structural answer under a name that sounds like this one.
- `deployment_profile_hash` / `deployment_profile_hash_hex(&ValidatedPreReleaseDeploymentProfile) -> Result<.., serde_json::Error>`
  — defined only over a validated profile. The identity is dormant; nothing
  published carries it.

## Error handling

Four independent error types, each covering one boundary.

| Type | Produced by | Shape |
|---|---|---|
| `ManifestError` | `validate_draft`, `validate_architecture_release` | `Vec<ManifestError>` — all defects, not the first |
| `DeploymentError` | the three deployment validators | `Vec<DeploymentError>` — all defects |
| `EnvelopeError` | `PublishedArchitecture::verify_hashes`, `validate_envelope`, `validate_release_envelope`, `validate_against_expected` | single value |
| `serde_json::Error` / `toml::ser::Error` | every projection, hashing, and rendering function | single value |

`ManifestError` and `DeploymentError` implement `Display` and
`std::error::Error`. Which variants to expect where:

- **Draft validation** yields `DuplicateId`, `MissingId`, `InvalidAsset`,
  `InvalidRoot`, `InvalidObject`, `InvalidOperation`, `InvalidQuantity`,
  `InvalidBound`, `InvalidDecision`, `InvalidDocument`. The `category`/`reason`
  fields are `&'static str` classifications, not free text.
- **Release validation** adds exactly two: `UnpinnedSpecification` (absent or all-zero
  anchor-set hash) and `DraftPublication` (status is not `Final`).
- **Structural profile validation** yields the schema, binding, and
  zero-value variants (`UnsupportedSchemaVersion`, `ArchitectureNotReleasable`,
  `ArchitectureHashMismatch`, `ProfileNotFinal`, `ZeroNetworkId`,
  `ZeroGenesisId`, `ZeroScriptLimit`), the per-`BoundId` calibration family
  (`MissingBoundCalibration` through `MeasurementExceedsScriptLimit`), the
  per-`DependencyId` evidence family, and `MissingArtifactHash` /
  `MissingTestReportHash`.
- **Production-release validation** always includes
  `ProductionReleaseUnsupported`, possibly alongside structural variants. Treat
  its `Err` as the expected outcome, not a bug in the profile.
- **Envelope validation** yields `UnsupportedHashAlgorithm`, `HashMismatch`,
  `UnsupportedBehaviouralHashAlgorithm`, `BehaviouralHashMismatch`,
  `UnsupportedSchemaVersion`, `UnrecognizedPublicationStatus`,
  `MissingRealizationVersion`, `MalformedRealizationVersion`, `NotFinal`,
  `UnexpectedPublication`, and `Serialization`. `serde_json::Error` converts
  into it via `From`.

The `serde_json::Error` returned by the hashing functions is a genuine
serialization failure, not a validation verdict; it cannot arise from an
architecture this crate declares.

## What this package deliberately does not do

- It does not execute anything: no UTXO state, no branch execution, no script
  implementation, no indexer, no property-test state machine.
- It does not perform I/O of any kind — no file, environment, or network
  access, in the crate or in its tests' public paths.
- It does not parse: no `FromStr`, no text-to-id mapping, no ingestion of the
  generated `architecture.json` / `architecture.toml`. Those are outputs.
- It does not mint an identity for an unvalidated value. The unchecked
  projections are crate-private.
- It does not claim deployability from architecture release, and it constructs
  no deployment release at all under the current profile schema.
- It does not know about targets: no opcode, script fragment, tapleaf,
  transaction position, or network value beyond the declared
  `target_network` string.

## Relationship to neighbors

`architecture` has no first-party dependencies. Downstream:

- **`tripod-realization`** consumes a validated `Architecture` and
  the id vocabulary, and adds the target-independent semantic layer
  (expressions, relations, constructibility, lifecycle) that the finite
  registries do not carry. It re-cites architecture ids rather than minting its
  own.
- **`tripod-model`** is the executable verification model. It depends
  on this crate and is checked against it (`validate_architecture_conformance`,
  `validate_bound_conformance` live on the model side). The direction is
  one-way.
- **`tripod-compiler`** cites architecture-owned ids in its error
  vocabulary — for example `CompileError::IncompleteRealizationScope { operation: OperationId }` — precisely so a compiler-local operation identity
  never duplicates an upstream one.
- **`tripod-artifacts`** owns the generated `architecture.json` and
  `architecture.toml` publications, rendering them through
  `PublishedArchitecture`.
