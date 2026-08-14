//! Architecture-manifest validation.
//!
//! Draft validation checks internal consistency of the typed
//! declaration. Architecture-release validation additionally requires
//! the attestation anchor-set pin and a final publication status; it does
//! not claim deployability, which is the deployment profile's concern.

use std::collections::BTreeSet;
use std::fmt;

use crate::ids::{
    AllocatorId, AmountLimitId, AssetClass, AssetId, AssetRole, BoundId, DataOutputKind,
    DeallocatorId, DecisionId, DecisionStatus, DeltaKind, DependencyId, InputAuthorization,
    InvariantClauseId, LifecycleClass, ObjectId, OpenFlowKind, OperationId, PermissionClass,
    ProjectionId, ProjectionRule, PublicationStatus, QuantityId, QuantityKind, ReaderId, RootId,
    RootRole, RootUse, TagId, ValueFlowClass, WitnessId,
};
use crate::spec::{Architecture, MaxCount, OperationSpec, owner_bearing};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ManifestError {
    DuplicateId {
        category: &'static str,
        id: String,
    },
    MissingId {
        category: &'static str,
        id: String,
    },
    InvalidAsset {
        asset: AssetId,
        reason: &'static str,
    },
    InvalidRoot {
        root: RootId,
        reason: &'static str,
    },
    InvalidObject {
        object: ObjectId,
        reason: &'static str,
    },
    InvalidOperation {
        operation: OperationId,
        reason: &'static str,
    },
    InvalidQuantity {
        quantity: QuantityId,
        reason: &'static str,
    },
    InvalidBound {
        bound: BoundId,
        reason: &'static str,
    },
    InvalidDecision {
        decision: DecisionId,
        reason: &'static str,
    },
    InvalidDocument {
        reason: &'static str,
    },
    UnpinnedSpecification,
    DraftPublication,
}

impl fmt::Display for ManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateId { category, id } => {
                write!(formatter, "duplicate {category} id: {id}")
            }
            Self::MissingId { category, id } => {
                write!(formatter, "missing {category} id: {id}")
            }
            Self::InvalidAsset { asset, reason } => {
                write!(formatter, "invalid asset {asset}: {reason}")
            }
            Self::InvalidRoot { root, reason } => {
                write!(formatter, "invalid root {root}: {reason}")
            }
            Self::InvalidObject { object, reason } => {
                write!(formatter, "invalid object {object}: {reason}")
            }
            Self::InvalidOperation { operation, reason } => {
                write!(formatter, "invalid operation {operation}: {reason}")
            }
            Self::InvalidQuantity { quantity, reason } => {
                write!(formatter, "invalid quantity {quantity}: {reason}")
            }
            Self::InvalidBound { bound, reason } => {
                write!(formatter, "invalid bound {bound}: {reason}")
            }
            Self::InvalidDecision { decision, reason } => {
                write!(formatter, "invalid decision {decision}: {reason}")
            }
            Self::InvalidDocument { reason } => {
                write!(formatter, "invalid document metadata: {reason}")
            }
            Self::UnpinnedSpecification => {
                formatter.write_str("specification anchor-set hash is not pinned")
            }
            Self::DraftPublication => formatter.write_str("architecture status is still draft"),
        }
    }
}

impl std::error::Error for ManifestError {}

/// An architecture that has passed draft validation.
///
/// The wrapper is the type-level record of ADR-016's identity rule —
/// a complete typed object is validated first, then projected
/// canonically, and only then does it bear an identity — applied to
/// the architecture exactly as `ValidatedDeploymentProfile` applies it
/// to a deployment profile (R2-N03). Because the only constructor is
/// [`validate_draft`], holding one is proof that draft validation
/// accepted the architecture, so no caller can mint an architecture
/// semantic identity for an architecture with duplicate declarations,
/// a missing root, an issuance/authority mismatch, or any other draft
/// defect.
///
/// Draft validity is the right precondition for identity: publication
/// status and the attestation anchor-set pin are envelope metadata excluded from the
/// hashed body, so a draft and its later final revision share one
/// semantic hash. [`ValidatedReleaseArchitecture`] adds the release
/// obligations for consumers that need them.
#[derive(Clone, Copy, Debug)]
pub struct ValidatedDraftArchitecture<'a> {
    architecture: &'a Architecture,
}

impl<'a> ValidatedDraftArchitecture<'a> {
    /// The architecture draft validation accepted.
    pub const fn architecture(&self) -> &'a Architecture {
        self.architecture
    }
}

/// An architecture that has passed release validation.
///
/// Strictly stronger than [`ValidatedDraftArchitecture`]: release
/// validation runs draft validation and additionally requires a pinned
/// attestation anchor set and a final publication status. The only
/// constructor is [`validate_architecture_release`].
#[derive(Clone, Copy, Debug)]
pub struct ValidatedReleaseArchitecture<'a> {
    architecture: &'a Architecture,
}

impl<'a> ValidatedReleaseArchitecture<'a> {
    /// The architecture release validation accepted.
    pub const fn architecture(&self) -> &'a Architecture {
        self.architecture
    }

    /// The draft-validated view, since release validation subsumes it.
    ///
    /// This is the value the identity functions accept: a release is
    /// draft-valid by construction, and both states share one semantic
    /// hash recipe.
    pub const fn draft(&self) -> ValidatedDraftArchitecture<'a> {
        ValidatedDraftArchitecture {
            architecture: self.architecture,
        }
    }
}

/// Validate an architecture draft and carry the verdict in the type.
///
/// This is the sole entry to architecture identity: the returned
/// wrapper is the only value the canonical projection and hash
/// functions accept.
pub fn validate_draft(
    architecture: &Architecture,
) -> Result<ValidatedDraftArchitecture<'_>, Vec<ManifestError>> {
    let mut errors = Vec::new();

    check_unique_ids(
        "asset",
        architecture.assets.iter().map(|asset| asset.id.as_str()),
        &mut errors,
    );

    check_unique_ids(
        "root",
        architecture.roots.iter().map(|root| root.id.as_str()),
        &mut errors,
    );

    check_unique_ids(
        "object",
        architecture.objects.iter().map(|object| object.id.as_str()),
        &mut errors,
    );

    check_unique_ids(
        "operation",
        architecture
            .operations
            .iter()
            .map(|operation| operation.id.as_str()),
        &mut errors,
    );

    check_unique_ids(
        "quantity",
        architecture
            .quantities
            .iter()
            .map(|quantity| quantity.id.as_str()),
        &mut errors,
    );

    check_unique_ids(
        "witness",
        architecture
            .witnesses
            .iter()
            .map(|witness| witness.id.as_str()),
        &mut errors,
    );

    check_unique_ids(
        "clause",
        architecture.clauses.iter().map(|clause| clause.as_str()),
        &mut errors,
    );

    check_unique_ids(
        "dependency",
        architecture
            .dependencies
            .iter()
            .map(|dependency| dependency.id.as_str()),
        &mut errors,
    );

    check_unique_ids(
        "decision",
        architecture
            .decisions
            .iter()
            .map(|decision| decision.id.as_str()),
        &mut errors,
    );

    check_unique_ids(
        "bound",
        architecture.bounds.iter().map(|bound| bound.id.as_str()),
        &mut errors,
    );

    check_unique_ids(
        "tag",
        architecture.tags.iter().map(|tag| tag.id.as_str()),
        &mut errors,
    );

    check_unique_ids(
        "amount limit",
        architecture
            .amount_limits
            .iter()
            .map(|limit| limit.id.as_str()),
        &mut errors,
    );

    validate_document(architecture, &mut errors);
    validate_set_like_uniqueness(architecture, &mut errors);
    validate_expected_id_coverage(architecture, &mut errors);
    validate_assets(architecture, &mut errors);
    validate_roots(architecture, &mut errors);
    validate_objects(architecture, &mut errors);
    validate_bounds(architecture, &mut errors);
    validate_object_lifecycles(architecture, &mut errors);
    validate_operations(architecture, &mut errors);
    validate_projection_policy(architecture, &mut errors);
    validate_required_operation_minima(architecture, &mut errors);
    validate_quantities(architecture, &mut errors);
    validate_decisions(architecture, &mut errors);
    validate_tags(architecture, &mut errors);
    validate_amount_limits(architecture, &mut errors);

    if errors.is_empty() {
        Ok(ValidatedDraftArchitecture { architecture })
    } else {
        Err(errors)
    }
}

/// Architecture-release validation: the typed manifest is internally
/// consistent, the attestation anchor set is pinned, and the publication
/// status is final.
///
/// This validates the *architecture* only. It makes no deployability
/// claim: calibrated bounds, verified substrate dependencies, and
/// implementation artifact hashes are the deployment profile's
/// obligations, checked by `validate_deployment_release`.
pub fn validate_architecture_release(
    architecture: &Architecture,
) -> Result<ValidatedReleaseArchitecture<'_>, Vec<ManifestError>> {
    let mut errors = match validate_draft(architecture) {
        Ok(_) => Vec::new(),
        Err(errors) => errors,
    };

    // An all-zero digest is the unset placeholder, never a release
    // pin: treat it exactly like an absent anchor set.
    match architecture.document.specification.anchor_set_hash {
        Some(hash) if hash != [0_u8; 32] => {}
        _ => errors.push(ManifestError::UnpinnedSpecification),
    }

    if architecture.document.status != PublicationStatus::Final {
        errors.push(ManifestError::DraftPublication);
    }

    if errors.is_empty() {
        Ok(ValidatedReleaseArchitecture { architecture })
    } else {
        Err(errors)
    }
}

/// Document-metadata validation: the schema version is the one this
/// crate implements, and every version/identity string is nonblank.
/// These fields are envelope metadata outside both hashes, so nothing
/// downstream catches a blank or mismatched value unless it is checked
/// here.
fn validate_document(architecture: &Architecture, errors: &mut Vec<ManifestError>) {
    let document = &architecture.document;

    if document.architecture_schema_version != crate::spec::ARCHITECTURE_SCHEMA_VERSION {
        errors.push(ManifestError::InvalidDocument {
            reason: "architecture schema version is not the supported schema",
        });
    }

    if document.realization_version.trim().is_empty() {
        errors.push(ManifestError::InvalidDocument {
            reason: "realization version is blank",
        });
    } else if !crate::spec::realization_version_well_formed(document.realization_version) {
        errors.push(ManifestError::InvalidDocument {
            reason: "realization version is not a tracked major.minor.0 binding",
        });
    }

    if document.specification.version.trim().is_empty() {
        errors.push(ManifestError::InvalidDocument {
            reason: "specification version is blank",
        });
    }

    if document.target_network.trim().is_empty() {
        errors.push(ManifestError::InvalidDocument {
            reason: "target network is blank",
        });
    }
}

/// True when the iterator yields any element twice.
fn has_duplicates<T: Ord>(items: impl IntoIterator<Item = T>) -> bool {
    let mut seen = BTreeSet::new();
    items.into_iter().any(|item| !seen.insert(item))
}

/// Set-like declarations must be declared without duplicates.
///
/// The export layer sorts **and deduplicates** these arrays, so a
/// duplicated declaration would survive in the normative typed source
/// while being invisible to both manifest hashes. Rejecting it here
/// keeps declaration exactness: every set-like array is a set in the
/// source, not merely in the publication.
fn validate_set_like_uniqueness(architecture: &Architecture, errors: &mut Vec<ManifestError>) {
    for asset in architecture.assets {
        if has_duplicates(asset.destruction_operations.iter().copied()) {
            errors.push(ManifestError::InvalidAsset {
                asset: asset.id,
                reason: "duplicate destruction-operation declaration",
            });
        }
    }

    for object in architecture.objects {
        if has_duplicates(object.allocators.iter().copied()) {
            errors.push(ManifestError::InvalidObject {
                object: object.id,
                reason: "duplicate allocator declaration",
            });
        }
        if has_duplicates(object.mutators.iter().copied()) {
            errors.push(ManifestError::InvalidObject {
                object: object.id,
                reason: "duplicate mutator declaration",
            });
        }
        if has_duplicates(object.deallocators.iter().copied()) {
            errors.push(ManifestError::InvalidObject {
                object: object.id,
                reason: "duplicate deallocator declaration",
            });
        }
        if has_duplicates(object.witnesses.iter().copied()) {
            errors.push(ManifestError::InvalidObject {
                object: object.id,
                reason: "duplicate witness declaration",
            });
        }
    }

    for operation in architecture.operations {
        let operation_sets: [(&'static str, bool); 6] = [
            (
                "duplicate open-flow declaration",
                has_duplicates(operation.open_flows.iter().copied()),
            ),
            (
                "duplicate read declaration",
                has_duplicates(operation.reads.iter().copied()),
            ),
            (
                "duplicate write declaration",
                has_duplicates(operation.writes.iter().copied()),
            ),
            (
                "duplicate witness declaration",
                has_duplicates(operation.witnesses.iter().copied()),
            ),
            (
                "duplicate value-flow declaration",
                has_duplicates(operation.value_flows.iter().copied()),
            ),
            (
                "duplicate bound declaration",
                has_duplicates(operation.bounds.iter().copied()),
            ),
        ];

        for (reason, duplicated) in operation_sets {
            if duplicated {
                errors.push(ManifestError::InvalidOperation {
                    operation: operation.id,
                    reason,
                });
            }
        }

        if has_duplicates(operation.issuances.iter().map(|issuance| issuance.asset)) {
            errors.push(ManifestError::InvalidOperation {
                operation: operation.id,
                reason: "duplicate issuance declaration for one asset",
            });
        }

        if has_duplicates(operation.canonical_deltas.iter().map(|delta| {
            (
                delta.asset,
                delta.kind,
                delta.condition,
                delta.destruction_tag,
            )
        })) {
            errors.push(ManifestError::InvalidOperation {
                operation: operation.id,
                reason: "duplicate canonical-delta declaration",
            });
        }

        if has_duplicates(
            operation
                .data_outputs
                .iter()
                .map(|output| (output.kind, output.tag, output.asset, output.condition)),
        ) {
            errors.push(ManifestError::InvalidOperation {
                operation: operation.id,
                reason: "duplicate data-output declaration",
            });
        }
    }

    for quantity in architecture.quantities {
        if has_duplicates(quantity.reads.iter().copied()) {
            errors.push(ManifestError::InvalidQuantity {
                quantity: quantity.id,
                reason: "duplicate read declaration",
            });
        }
        if has_duplicates(quantity.writers.iter().copied()) {
            errors.push(ManifestError::InvalidQuantity {
                quantity: quantity.id,
                reason: "duplicate writer declaration",
            });
        }
        if has_duplicates(quantity.readers.iter().copied()) {
            errors.push(ManifestError::InvalidQuantity {
                quantity: quantity.id,
                reason: "duplicate reader declaration",
            });
        }
    }
}

fn check_unique_ids<'item>(
    category: &'static str,
    ids: impl IntoIterator<Item = &'item str>,
    errors: &mut Vec<ManifestError>,
) {
    let mut seen = BTreeSet::new();

    for id in ids {
        if !seen.insert(id) {
            errors.push(ManifestError::DuplicateId {
                category,
                id: id.to_owned(),
            });
        }
    }
}

/// Requires every expected stable identifier to be declared exactly
/// once in its category.
fn require_all_ids<T: Copy + Ord + fmt::Display>(
    category: &'static str,
    expected: &[T],
    actual: impl Iterator<Item = T>,
    errors: &mut Vec<ManifestError>,
) {
    let declared = actual.collect::<BTreeSet<_>>();

    for id in expected {
        if !declared.contains(id) {
            errors.push(ManifestError::MissingId {
                category,
                id: id.to_string(),
            });
        }
    }
}

fn validate_expected_id_coverage(architecture: &Architecture, errors: &mut Vec<ManifestError>) {
    require_all_ids(
        "asset",
        AssetId::ALL,
        architecture.assets.iter().map(|asset| asset.id),
        errors,
    );

    require_all_ids(
        "root",
        RootId::ALL,
        architecture.roots.iter().map(|root| root.id),
        errors,
    );

    require_all_ids(
        "object",
        ObjectId::ALL,
        architecture.objects.iter().map(|object| object.id),
        errors,
    );

    require_all_ids(
        "operation",
        OperationId::ALL,
        architecture.operations.iter().map(|operation| operation.id),
        errors,
    );

    require_all_ids(
        "quantity",
        QuantityId::ALL,
        architecture.quantities.iter().map(|quantity| quantity.id),
        errors,
    );

    require_all_ids(
        "witness",
        WitnessId::ALL,
        architecture.witnesses.iter().map(|witness| witness.id),
        errors,
    );

    require_all_ids(
        "clause",
        InvariantClauseId::ALL,
        architecture.clauses.iter().copied(),
        errors,
    );

    require_all_ids(
        "dependency",
        DependencyId::ALL,
        architecture
            .dependencies
            .iter()
            .map(|dependency| dependency.id),
        errors,
    );

    require_all_ids(
        "decision",
        DecisionId::ALL,
        architecture.decisions.iter().map(|decision| decision.id),
        errors,
    );

    require_all_ids(
        "bound",
        BoundId::ALL,
        architecture.bounds.iter().map(|bound| bound.id),
        errors,
    );

    require_all_ids(
        "amount limit",
        AmountLimitId::ALL,
        architecture.amount_limits.iter().map(|limit| limit.id),
        errors,
    );

    require_all_ids(
        "tag",
        TagId::ALL,
        architecture.tags.iter().map(|tag| tag.id),
        errors,
    );
}

fn validate_assets(architecture: &Architecture, errors: &mut Vec<ManifestError>) {
    for asset in architecture.assets {
        if asset.class == AssetClass::Open
            && (asset.authority.is_some() || asset.issue_operation.is_some() || asset.reissuable)
        {
            errors.push(ManifestError::InvalidAsset {
                asset: asset.id,
                reason: "open asset may not have covenant issuance authority",
            });
        }

        if asset.reissuable {
            let Some(authority) = asset.authority else {
                errors.push(ManifestError::InvalidAsset {
                    asset: asset.id,
                    reason: "reissuable asset lacks authority",
                });
                continue;
            };

            let Some(issue_operation) = asset.issue_operation else {
                errors.push(ManifestError::InvalidAsset {
                    asset: asset.id,
                    reason: "reissuable asset lacks issue operation",
                });
                continue;
            };

            let Some(authority_spec) = architecture.asset(authority) else {
                errors.push(ManifestError::MissingId {
                    category: "asset authority",
                    id: authority.as_str().to_owned(),
                });
                continue;
            };

            if authority_spec.role != AssetRole::IssuanceAuthority
                || authority_spec.fixed_amount != Some(1)
                || authority_spec.reissuable
            {
                errors.push(ManifestError::InvalidAsset {
                    asset: asset.id,
                    reason: "issuance authority is not a fixed non-reissuable amount-one asset",
                });
            }

            let Some(operation) = architecture.operation(issue_operation) else {
                errors.push(ManifestError::MissingId {
                    category: "issue operation",
                    id: issue_operation.as_str().to_owned(),
                });
                continue;
            };

            if !operation
                .issuances
                .iter()
                .any(|issuance| issuance.asset == asset.id && issuance.authority == authority)
            {
                errors.push(ManifestError::InvalidAsset {
                    asset: asset.id,
                    reason: "issue operation does not declare matching issuance",
                });
            }
        }

        if asset.role == AssetRole::IssuanceAuthority
            && (asset.fixed_amount != Some(1) || asset.reissuable)
        {
            errors.push(ManifestError::InvalidAsset {
                asset: asset.id,
                reason: "authority must be fixed amount one and non-reissuable",
            });
        }

        for operation in asset.destruction_operations {
            if architecture.operation(*operation).is_none() {
                errors.push(ManifestError::MissingId {
                    category: "destruction operation",
                    id: operation.as_str().to_owned(),
                });
            }
        }

        // The declared destruction lifecycle must equal the set of
        // operations declaring a destruction delta for the asset.
        let declared = asset
            .destruction_operations
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();

        if declared.len() != asset.destruction_operations.len() {
            errors.push(ManifestError::InvalidAsset {
                asset: asset.id,
                reason: "duplicate destruction operation declaration",
            });
        }

        if declared != derived_destruction_operations(architecture, asset.id) {
            errors.push(ManifestError::InvalidAsset {
                asset: asset.id,
                reason: "destruction operations disagree with declared destruction deltas",
            });
        }
    }

    for expected in AssetId::ALL {
        if architecture.asset(*expected).is_none() {
            errors.push(ManifestError::MissingId {
                category: "asset",
                id: expected.as_str().to_owned(),
            });
        }
    }
}

/// Operations declaring a destruction delta for the asset.
fn derived_destruction_operations(
    architecture: &Architecture,
    asset: AssetId,
) -> BTreeSet<OperationId> {
    architecture
        .operations
        .iter()
        .filter(|operation| {
            operation
                .canonical_deltas
                .iter()
                .any(|delta| delta.asset == asset && delta.kind == DeltaKind::Destruction)
        })
        .map(|operation| operation.id)
        .collect()
}

/// The protocol object realizing each constant-cardinality root.
fn root_object(root: RootId) -> ObjectId {
    match root {
        RootId::State => ObjectId::State,
        RootId::Resv => ObjectId::Resv,
        RootId::Pace => ObjectId::Pace,
        RootId::EntAuth => ObjectId::EntitlementAuthority,
        RootId::DistAuth => ObjectId::DistributionAuthority,
    }
}

/// The constant-cardinality root realized by a protocol object, if
/// any.
fn object_root(object: ObjectId) -> Option<RootId> {
    RootId::ALL
        .iter()
        .copied()
        .find(|root| root_object(*root) == object)
}

/// The declared root role each semantic root identity must carry.
const fn root_expected_role(root: RootId) -> RootRole {
    match root {
        RootId::State => RootRole::StateIdentity,
        RootId::Resv => RootRole::ActiveReserve,
        RootId::Pace => RootRole::CadenceAndIssuance,
        RootId::EntAuth => RootRole::EntitlementAuthority,
        RootId::DistAuth => RootRole::DistributionAuthority,
    }
}

fn validate_roots(architecture: &Architecture, errors: &mut Vec<ManifestError>) {
    for root in architecture.roots {
        let Some(asset) = architecture.asset(root.asset) else {
            errors.push(ManifestError::MissingId {
                category: "root asset",
                id: root.asset.as_str().to_owned(),
            });
            continue;
        };

        if root.role == RootRole::ActiveReserve {
            if asset.class != AssetClass::Open || asset.role != AssetRole::ReserveAsset {
                errors.push(ManifestError::InvalidRoot {
                    root: root.id,
                    reason: "active reserve must use the open reserve asset",
                });
            }

            if root.fixed_amount.is_some() {
                errors.push(ManifestError::InvalidRoot {
                    root: root.id,
                    reason: "active reserve root must not declare a fixed amount",
                });
            }
        } else {
            if asset.class != AssetClass::Closed {
                errors.push(ManifestError::InvalidRoot {
                    root: root.id,
                    reason: "non-reserve roots must use closed assets",
                });
            }

            if root.fixed_amount != Some(1) {
                errors.push(ManifestError::InvalidRoot {
                    root: root.id,
                    reason: "closed root must have amount one",
                });
            }
        }

        // Root/object closure: the root's role, asset, and realizing
        // object must agree as one declaration, not merely each pass
        // their own generic checks.
        if root.role != root_expected_role(root.id) {
            errors.push(ManifestError::InvalidRoot {
                root: root.id,
                reason: "root role does not match its semantic root identity",
            });
        }

        match architecture.object(root_object(root.id)) {
            None => errors.push(ManifestError::InvalidRoot {
                root: root.id,
                reason: "root has no realizing protocol object",
            }),
            Some(object) => {
                if object.asset != root.asset {
                    errors.push(ManifestError::InvalidRoot {
                        root: root.id,
                        reason: "root asset differs from its realizing object's asset",
                    });
                }

                if object.lifecycle != LifecycleClass::ConstantRoot {
                    errors.push(ManifestError::InvalidRoot {
                        root: root.id,
                        reason: "realizing object must have constant-root lifecycle",
                    });
                }
            }
        }
    }

    // Closure in the other direction: every constant-root object must
    // be realized by a declared root.
    for object in architecture.objects {
        if object.lifecycle == LifecycleClass::ConstantRoot {
            let realized =
                object_root(object.id).is_some_and(|root| architecture.root(root).is_some());

            if !realized {
                errors.push(ManifestError::InvalidObject {
                    object: object.id,
                    reason: "constant-root object has no corresponding root declaration",
                });
            }
        }
    }

    for expected in RootId::ALL {
        if architecture.root(*expected).is_none() {
            errors.push(ManifestError::MissingId {
                category: "root",
                id: expected.as_str().to_owned(),
            });
        }
    }
}

/// A bound that does not require deployment calibration is a fixed
/// protocol value: its authoritative maximum must be declared in the
/// manifest, not left to whatever constant the runtime happens to use.
fn validate_bounds(architecture: &Architecture, errors: &mut Vec<ManifestError>) {
    for bound in architecture.bounds {
        if bound.requires_deployment_calibration {
            continue;
        }

        match bound.default_value {
            None => errors.push(ManifestError::InvalidBound {
                bound: bound.id,
                reason: "fixed bound must declare its value",
            }),
            Some(0) => errors.push(ManifestError::InvalidBound {
                bound: bound.id,
                reason: "fixed bound must be positive",
            }),
            Some(_) => {}
        }
    }
}

fn validate_objects(architecture: &Architecture, errors: &mut Vec<ManifestError>) {
    for object in architecture.objects {
        if architecture.asset(object.asset).is_none() {
            errors.push(ManifestError::MissingId {
                category: "object asset",
                id: object.asset.as_str().to_owned(),
            });
        }

        for allocator in object.allocators {
            if let AllocatorId::Operation(operation) = allocator
                && architecture.operation(*operation).is_none()
            {
                errors.push(ManifestError::MissingId {
                    category: "object allocator",
                    id: operation.as_str().to_owned(),
                });
            }
        }

        for mutator in object.mutators {
            if architecture.operation(*mutator).is_none() {
                errors.push(ManifestError::MissingId {
                    category: "object mutator",
                    id: mutator.as_str().to_owned(),
                });
            }
        }

        for deallocator in object.deallocators {
            if let DeallocatorId::Operation(operation) = deallocator
                && architecture.operation(*operation).is_none()
            {
                errors.push(ManifestError::MissingId {
                    category: "object deallocator",
                    id: operation.as_str().to_owned(),
                });
            }
        }

        for witness in object.witnesses {
            if !architecture
                .witnesses
                .iter()
                .any(|spec| spec.id == *witness)
            {
                errors.push(ManifestError::MissingId {
                    category: "object witness",
                    id: witness.as_str().to_owned(),
                });
            }
        }
    }

    for expected in ObjectId::ALL {
        if architecture.object(*expected).is_none() {
            errors.push(ManifestError::MissingId {
                category: "object",
                id: expected.as_str().to_owned(),
            });
        }
    }
}

/// Bidirectional object-lifecycle consistency between object
/// allocator/mutator/deallocator declarations and operation
/// input/output declarations.
fn validate_object_lifecycles(architecture: &Architecture, errors: &mut Vec<ManifestError>) {
    for object in architecture.objects {
        for allocator in object.allocators {
            if let AllocatorId::Operation(operation) = allocator
                && let Some(spec) = architecture.operation(*operation)
                && !spec.outputs.iter().any(|output| {
                    output.object == object.id && max_is_positive(architecture, output.maximum)
                })
            {
                errors.push(ManifestError::InvalidObject {
                    object: object.id,
                    reason: "allocator operation does not produce the object",
                });
            }
        }

        for mutator in object.mutators {
            if let Some(spec) = architecture.operation(*mutator)
                && !spec.inputs.iter().any(|input| input.object == object.id)
            {
                errors.push(ManifestError::InvalidObject {
                    object: object.id,
                    reason: "mutator operation does not consume the object",
                });
            }
        }

        for deallocator in object.deallocators {
            if let DeallocatorId::Operation(operation) = deallocator
                && let Some(spec) = architecture.operation(*operation)
                && !spec.inputs.iter().any(|input| input.object == object.id)
            {
                errors.push(ManifestError::InvalidObject {
                    object: object.id,
                    reason: "deallocator operation does not consume the object",
                });
            }
        }

        // External-wallet objects are consumed and produced by
        // ordinary wallet activity, so the reverse census does not
        // apply to them.
        if object.lifecycle == LifecycleClass::ExternalWallet {
            continue;
        }

        for operation in architecture.operations {
            let consumes = operation
                .inputs
                .iter()
                .any(|input| input.object == object.id);

            if consumes
                && !object.mutators.contains(&operation.id)
                && !object
                    .deallocators
                    .contains(&DeallocatorId::Operation(operation.id))
            {
                errors.push(ManifestError::InvalidObject {
                    object: object.id,
                    reason: "operation consumes the object without mutator or deallocator role",
                });
            }

            let produces = operation
                .outputs
                .iter()
                .any(|output| output.object == object.id);

            if produces
                && !object
                    .allocators
                    .contains(&AllocatorId::Operation(operation.id))
                && !object.mutators.contains(&operation.id)
            {
                errors.push(ManifestError::InvalidObject {
                    object: object.id,
                    reason: "operation produces the object without allocator or mutator role",
                });
            }
        }
    }
}

fn max_is_positive(architecture: &Architecture, maximum: MaxCount) -> bool {
    match maximum {
        MaxCount::Exact(value) => value > 0,
        MaxCount::Bound(bound) => architecture
            .bound(bound)
            .is_none_or(|spec| spec.default_value != Some(0)),
    }
}

fn validate_operations(architecture: &Architecture, errors: &mut Vec<ManifestError>) {
    for operation in architecture.operations {
        let mut roots = BTreeSet::new();

        for root in operation.roots {
            if architecture.root(root.root).is_none() {
                errors.push(ManifestError::MissingId {
                    category: "operation root",
                    id: root.root.as_str().to_owned(),
                });
            }

            if !roots.insert(root.root) {
                errors.push(ManifestError::InvalidOperation {
                    operation: operation.id,
                    reason: "duplicate root-use declaration",
                });
            }
        }

        for issuance in operation.issuances {
            let Some(asset) = architecture.asset(issuance.asset) else {
                errors.push(ManifestError::MissingId {
                    category: "issued asset",
                    id: issuance.asset.as_str().to_owned(),
                });
                continue;
            };

            if asset.authority != Some(issuance.authority)
                || asset.issue_operation != Some(operation.id)
            {
                errors.push(ManifestError::InvalidOperation {
                    operation: operation.id,
                    reason: "issuance disagrees with asset declaration",
                });
            }

            let authority_root_present = operation.roots.iter().any(|root| {
                architecture.root(root.root).is_some_and(|root_spec| {
                    root_spec.asset == issuance.authority && root.use_kind == RootUse::Succession
                })
            });

            if !authority_root_present {
                errors.push(ManifestError::InvalidOperation {
                    operation: operation.id,
                    reason: "issuance operation does not consume and recreate its authority",
                });
            }
        }

        for cardinality in operation
            .inputs
            .iter()
            .map(|input| (input.object, input.minimum, input.maximum))
            .chain(
                operation
                    .outputs
                    .iter()
                    .map(|output| (output.object, output.minimum, output.maximum)),
            )
        {
            let (object, minimum, maximum) = cardinality;

            if architecture.object(object).is_none() {
                errors.push(ManifestError::MissingId {
                    category: "operation object",
                    id: object.as_str().to_owned(),
                });
            }

            validate_cardinality(architecture, operation.id, minimum, maximum, errors);
        }

        check_duplicate_objects(
            operation.id,
            "duplicate input object declaration",
            operation.inputs.iter().map(|input| input.object),
            errors,
        );

        check_duplicate_objects(
            operation.id,
            "duplicate output object declaration",
            operation.outputs.iter().map(|output| output.object),
            errors,
        );

        validate_operation_root_objects(operation, errors);
        validate_operation_input_authorizations(operation, errors);
        validate_operation_permission_coherence(operation, errors);
        validate_operation_deltas(architecture, operation, errors);
        validate_operation_data_outputs(architecture, operation, errors);
        validate_operation_witness_completeness(operation, errors);
        validate_operation_bound_coverage(operation, errors);

        for witness in operation.witnesses {
            if !architecture
                .witnesses
                .iter()
                .any(|spec| spec.id == *witness)
            {
                errors.push(ManifestError::MissingId {
                    category: "operation witness",
                    id: witness.as_str().to_owned(),
                });
            }
        }

        for quantity in operation.reads.iter().chain(operation.writes) {
            if architecture.quantity(*quantity).is_none() {
                errors.push(ManifestError::MissingId {
                    category: "operation quantity",
                    id: quantity.as_str().to_owned(),
                });
            }
        }

        for bound in operation.bounds {
            if architecture.bound(*bound).is_none() {
                errors.push(ManifestError::MissingId {
                    category: "operation bound",
                    id: bound.as_str().to_owned(),
                });
            }
        }

        // Declared quantity reads must be mirrored by reader
        // declarations on the quantity side.
        for quantity in operation.reads {
            if let Some(spec) = architecture.quantity(*quantity)
                && !spec.allows(ReaderId::Operation(operation.id))
            {
                errors.push(ManifestError::InvalidOperation {
                    operation: operation.id,
                    reason: "operation reads a quantity that does not list it as a reader",
                });
            }
        }

        // Declared quantity writes must be mirrored by writer
        // declarations on the quantity side.
        for quantity in operation.writes {
            if let Some(spec) = architecture.quantity(*quantity)
                && !spec.writers.contains(&operation.id)
            {
                errors.push(ManifestError::InvalidOperation {
                    operation: operation.id,
                    reason: "operation writes a quantity that does not list it as a writer",
                });
            }
        }
    }

    for expected in OperationId::ALL {
        if architecture.operation(*expected).is_none() {
            errors.push(ManifestError::MissingId {
                category: "operation",
                id: expected.as_str().to_owned(),
            });
        }
    }
}

/// Pins the intended projection policy exactly, so the event-type
/// side of the raw event-projection differential is machine-checked:
///
/// - every operation requires a transition certificate;
/// - only burn requires a burn projection;
/// - only clear requires a clear projection;
/// - only settlement permits a distribution-residue projection;
/// - no operation declares a projection twice.
///
/// The complementary direction — every projection required by a
/// data-output family is declared — is checked by
/// `validate_operation_data_outputs`.
fn validate_projection_policy(architecture: &Architecture, errors: &mut Vec<ManifestError>) {
    for operation in architecture.operations {
        let mut seen = BTreeSet::new();

        for projection in operation.projections {
            if !seen.insert(projection.projection) {
                errors.push(ManifestError::InvalidOperation {
                    operation: operation.id,
                    reason: "duplicate projection declaration",
                });
            }
        }

        if operation.projection_rule(ProjectionId::TransitionCertificate)
            != ProjectionRule::Required
        {
            errors.push(ManifestError::InvalidOperation {
                operation: operation.id,
                reason: "every operation requires a transition-certificate projection",
            });
        }

        let expected_burn = if operation.id == OperationId::Burn {
            ProjectionRule::Required
        } else {
            ProjectionRule::Forbidden
        };

        if operation.projection_rule(ProjectionId::BurnEvent) != expected_burn {
            errors.push(ManifestError::InvalidOperation {
                operation: operation.id,
                reason: "burn-event projection violates the projection policy",
            });
        }

        let expected_clear = if operation.id == OperationId::Clear {
            ProjectionRule::Required
        } else {
            ProjectionRule::Forbidden
        };

        if operation.projection_rule(ProjectionId::ClearEvent) != expected_clear {
            errors.push(ManifestError::InvalidOperation {
                operation: operation.id,
                reason: "clear-event projection violates the projection policy",
            });
        }

        let expected_residue = if operation.id == OperationId::SettleDistribution {
            ProjectionRule::Optional
        } else {
            ProjectionRule::Forbidden
        };

        if operation.projection_rule(ProjectionId::DistributionResidue) != expected_residue {
            errors.push(ManifestError::InvalidOperation {
                operation: operation.id,
                reason: "distribution-residue projection violates the projection policy",
            });
        }
    }
}

fn validate_cardinality(
    architecture: &Architecture,
    operation: OperationId,
    minimum: u16,
    maximum: MaxCount,
    errors: &mut Vec<ManifestError>,
) {
    match maximum {
        MaxCount::Exact(value) => {
            if minimum > value {
                errors.push(ManifestError::InvalidOperation {
                    operation,
                    reason: "cardinality minimum exceeds exact maximum",
                });
            }
        }

        MaxCount::Bound(bound) => match architecture.bound(bound) {
            None => {
                errors.push(ManifestError::MissingId {
                    category: "operation bound",
                    id: bound.as_str().to_owned(),
                });
            }

            Some(spec) => {
                if let Some(default) = spec.default_value
                    && u64::from(minimum) > default
                {
                    errors.push(ManifestError::InvalidOperation {
                        operation,
                        reason: "cardinality minimum exceeds the bound's draft default",
                    });
                }
            }
        },
    }
}

fn check_duplicate_objects(
    operation: OperationId,
    reason: &'static str,
    objects: impl Iterator<Item = ObjectId>,
    errors: &mut Vec<ManifestError>,
) {
    let mut seen = BTreeSet::new();

    for object in objects {
        if !seen.insert(object) {
            errors.push(ManifestError::InvalidOperation { operation, reason });
        }
    }
}

/// Root-use declarations and operation input/output declarations must
/// agree in both directions.
fn validate_operation_root_objects(operation: &OperationSpec, errors: &mut Vec<ManifestError>) {
    for root in RootId::ALL {
        let object = root_object(*root);

        let input = operation.inputs.iter().find(|input| input.object == object);

        let output = operation
            .outputs
            .iter()
            .find(|output| output.object == object);

        match operation.root_use(*root) {
            RootUse::Forbidden => {
                if input.is_some() || output.is_some() {
                    errors.push(ManifestError::InvalidOperation {
                        operation: operation.id,
                        reason: "forbidden root appears in operation inputs or outputs",
                    });
                }
            }

            RootUse::Succession => {
                let input_exact = input
                    .is_some_and(|input| input.minimum == 1 && input.maximum == MaxCount::Exact(1));

                let output_exact = output.is_some_and(|output| {
                    output.minimum == 1 && output.maximum == MaxCount::Exact(1)
                });

                if !input_exact || !output_exact {
                    errors.push(ManifestError::InvalidOperation {
                        operation: operation.id,
                        reason: "succession root must have exactly one input and one output",
                    });
                }
            }

            RootUse::SuccessionOrTermination => {
                let input_exact = input
                    .is_some_and(|input| input.minimum == 1 && input.maximum == MaxCount::Exact(1));

                let output_optional = output.is_some_and(|output| {
                    output.minimum == 0 && output.maximum == MaxCount::Exact(1)
                });

                if !input_exact || !output_optional {
                    errors.push(ManifestError::InvalidOperation {
                        operation: operation.id,
                        reason:
                            "succession-or-termination root must have one input and 0..1 outputs",
                    });
                }
            }
        }
    }
}

/// Per-input authorization legality.
///
/// Every root-object input is a covenant companion: it carries no
/// independent owner signature and may not be declared permissionless
/// or signer-backed. No non-root object may claim companion status,
/// and a companion may only name a root the operation is permitted to
/// use.
fn validate_operation_input_authorizations(
    operation: &OperationSpec,
    errors: &mut Vec<ManifestError>,
) {
    for input in operation.inputs {
        if let Some(root) = object_root(input.object) {
            if input.authorization != InputAuthorization::CovenantCompanion {
                errors.push(ManifestError::InvalidOperation {
                    operation: operation.id,
                    reason: "root input must use covenant-companion authorization",
                });
            }

            if operation.root_use(root) == RootUse::Forbidden {
                errors.push(ManifestError::InvalidOperation {
                    operation: operation.id,
                    reason: "companion input names a root the operation forbids",
                });
            }

            continue;
        }

        let legal = match input.authorization {
            InputAuthorization::CovenantCompanion => false,
            InputAuthorization::SponsorOwner => input.object == ObjectId::PlainLbtc,
            InputAuthorization::RefundKey => input.object == ObjectId::DepositRequest,
            InputAuthorization::InputOwner => owner_bearing(input.object),
            InputAuthorization::Permissionless => true,
        };

        if !legal {
            errors.push(ManifestError::InvalidOperation {
                operation: operation.id,
                reason: "input authorization is not legal for the object",
            });
        }
    }
}

/// Coherence between the operation's primary permission class and its
/// declared input authorizations. These are declaration-drift guards,
/// not full authorization proofs.
fn validate_operation_permission_coherence(
    operation: &OperationSpec,
    errors: &mut Vec<ManifestError>,
) {
    let input_owner_inputs = operation
        .inputs
        .iter()
        .filter(|input| input.authorization == InputAuthorization::InputOwner)
        .count();

    let refund_key_inputs = operation
        .inputs
        .iter()
        .filter(|input| input.authorization == InputAuthorization::RefundKey)
        .count();

    match operation.authorization {
        PermissionClass::ClientAuthorized | PermissionClass::ReceiptOwners => {
            if input_owner_inputs == 0 {
                errors.push(ManifestError::InvalidOperation {
                    operation: operation.id,
                    reason: "owner-authorized operation requires an input-owner input",
                });
            }
        }

        PermissionClass::RefundKey => {
            if refund_key_inputs != 1 {
                errors.push(ManifestError::InvalidOperation {
                    operation: operation.id,
                    reason: "refund-key operation requires exactly one refund-key input",
                });
            }
        }

        PermissionClass::CadenceBand => {
            if operation.id != OperationId::Cycle {
                errors.push(ManifestError::InvalidOperation {
                    operation: operation.id,
                    reason: "cadence-band authorization is reserved for the cycle operation",
                });
            }

            if operation.root_use(RootId::Pace) == RootUse::Forbidden {
                errors.push(ManifestError::InvalidOperation {
                    operation: operation.id,
                    reason: "cadence-band operation must use the PACE root",
                });
            }
        }

        PermissionClass::Permissionless | PermissionClass::Operator => {
            // Neither class requires an operation-level owner or
            // refund signature; sponsor signatures cover only the
            // sponsor's own funds.
            if input_owner_inputs > 0 || refund_key_inputs > 0 {
                errors.push(ManifestError::InvalidOperation {
                    operation: operation.id,
                    reason: "operation declares owner or refund inputs its permission \
                             class does not require",
                });
            }
        }
    }
}

/// Asset-specific canonical-delta shape and correspondence rules.
fn validate_operation_deltas(
    architecture: &Architecture,
    operation: &OperationSpec,
    errors: &mut Vec<ManifestError>,
) {
    for delta in operation.canonical_deltas {
        if !matches!(delta.asset, AssetId::U | AssetId::Ent | AssetId::DistCtl) {
            errors.push(ManifestError::InvalidOperation {
                operation: operation.id,
                reason: "canonical delta asset must be a closed value or capability asset",
            });
        }

        match delta.kind {
            DeltaKind::Issuance => {
                if delta.destruction_tag.is_some() {
                    errors.push(ManifestError::InvalidOperation {
                        operation: operation.id,
                        reason: "issuance delta may not carry a destruction tag",
                    });
                }

                let matching = operation
                    .issuances
                    .iter()
                    .filter(|issuance| {
                        issuance.asset == delta.asset
                            && issuance.condition.as_str() == delta.condition.as_str()
                    })
                    .count();

                if matching != 1 {
                    errors.push(ManifestError::InvalidOperation {
                        operation: operation.id,
                        reason: "issuance delta lacks exactly one matching issuance declaration",
                    });
                }
            }

            DeltaKind::Destruction => {
                let Some(tag) = delta.destruction_tag else {
                    errors.push(ManifestError::InvalidOperation {
                        operation: operation.id,
                        reason: "destruction delta lacks a destruction tag",
                    });
                    continue;
                };

                if !architecture.tags.iter().any(|spec| spec.id == tag) {
                    errors.push(ManifestError::MissingId {
                        category: "delta tag",
                        id: tag.as_str().to_owned(),
                    });
                }

                let asset_declares = architecture
                    .asset(delta.asset)
                    .is_some_and(|spec| spec.destruction_operations.contains(&operation.id));

                if !asset_declares {
                    errors.push(ManifestError::InvalidOperation {
                        operation: operation.id,
                        reason: "asset does not declare the operation as destructive",
                    });
                }

                let matching_outputs = operation
                    .data_outputs
                    .iter()
                    .filter(|output| {
                        output.kind == DataOutputKind::Destruction
                            && output.tag == tag
                            && output.asset == Some(delta.asset)
                            && output.condition == delta.condition
                    })
                    .count();

                if matching_outputs != 1 {
                    errors.push(ManifestError::InvalidOperation {
                        operation: operation.id,
                        reason: "destruction delta lacks exactly one matching data output",
                    });
                }
            }

            DeltaKind::Lateral | DeltaKind::OwnerlessLateral => {
                if delta.destruction_tag.is_some() {
                    errors.push(ManifestError::InvalidOperation {
                        operation: operation.id,
                        reason: "lateral delta may not carry a destruction tag",
                    });
                }
            }
        }
    }

    // Every issuance declaration must have exactly one matching
    // issuance delta.
    for issuance in operation.issuances {
        let matching = operation
            .canonical_deltas
            .iter()
            .filter(|delta| {
                delta.kind == DeltaKind::Issuance
                    && delta.asset == issuance.asset
                    && delta.condition.as_str() == issuance.condition.as_str()
            })
            .count();

        if matching != 1 {
            errors.push(ManifestError::InvalidOperation {
                operation: operation.id,
                reason: "issuance declaration lacks exactly one matching canonical delta",
            });
        }
    }
}

/// Declared data-output families.
fn validate_operation_data_outputs(
    architecture: &Architecture,
    operation: &OperationSpec,
    errors: &mut Vec<ManifestError>,
) {
    for output in operation.data_outputs {
        if !architecture.tags.iter().any(|spec| spec.id == output.tag) {
            errors.push(ManifestError::MissingId {
                category: "data-output tag",
                id: output.tag.as_str().to_owned(),
            });
        }

        validate_cardinality(
            architecture,
            operation.id,
            output.minimum,
            output.maximum,
            errors,
        );

        match output.kind {
            DataOutputKind::BurnRecord => {
                if output.tag != TagId::Burn
                    || output.asset.is_some()
                    || output.maximum != MaxCount::Bound(BoundId::BurnRecordMax)
                {
                    errors.push(ManifestError::InvalidOperation {
                        operation: operation.id,
                        reason: "burn-record data output must be tag-burn, assetless, and \
                                 bounded by BURN_RECORD_MAX",
                    });
                }

                if operation.projection_rule(ProjectionId::BurnEvent) == ProjectionRule::Forbidden {
                    errors.push(ManifestError::InvalidOperation {
                        operation: operation.id,
                        reason: "burn-record family requires a burn-event projection",
                    });
                }
            }

            DataOutputKind::Destruction => {
                let Some(asset) = output.asset else {
                    errors.push(ManifestError::InvalidOperation {
                        operation: operation.id,
                        reason: "destruction data output must name an asset",
                    });
                    continue;
                };

                let matching = operation
                    .canonical_deltas
                    .iter()
                    .filter(|delta| {
                        delta.kind == DeltaKind::Destruction
                            && delta.asset == asset
                            && delta.destruction_tag == Some(output.tag)
                            && delta.condition == output.condition
                    })
                    .count();

                if matching != 1 {
                    errors.push(ManifestError::InvalidOperation {
                        operation: operation.id,
                        reason: "destruction data output lacks a matching destruction delta",
                    });
                }

                if output.tag == TagId::Recon
                    && operation.projection_rule(ProjectionId::ClearEvent)
                        == ProjectionRule::Forbidden
                {
                    errors.push(ManifestError::InvalidOperation {
                        operation: operation.id,
                        reason: "recon destruction requires a clear-event projection",
                    });
                }

                if output.tag == TagId::DistributionResidue
                    && operation.projection_rule(ProjectionId::DistributionResidue)
                        == ProjectionRule::Forbidden
                {
                    errors.push(ManifestError::InvalidOperation {
                        operation: operation.id,
                        reason: "residue destruction requires a residue projection",
                    });
                }
            }
        }
    }
}

/// Witness and value-flow classes implied by roots, deltas, receipts,
/// and open sponsor flows.
fn validate_operation_witness_completeness(
    operation: &OperationSpec,
    errors: &mut Vec<ManifestError>,
) {
    let has = |witness: WitnessId| operation.witnesses.contains(&witness);

    if !operation.canonical_deltas.is_empty() && !has(WitnessId::CanonicalDelta) {
        errors.push(ManifestError::InvalidOperation {
            operation: operation.id,
            reason: "canonical deltas require the canonical-delta witness",
        });
    }

    if operation.root_use(RootId::State) != RootUse::Forbidden && !has(WitnessId::StateSuccession) {
        errors.push(ManifestError::InvalidOperation {
            operation: operation.id,
            reason: "STATE root use requires the state-succession witness",
        });
    }

    if operation.root_use(RootId::Resv) != RootUse::Forbidden && !has(WitnessId::ResvSuccession) {
        errors.push(ManifestError::InvalidOperation {
            operation: operation.id,
            reason: "RESV root use requires the resv-succession witness",
        });
    }

    let receipt_object =
        |object: ObjectId| matches!(object, ObjectId::ReceiptLive | ObjectId::ReceiptTimeLocked);

    let receipt_inputs = operation
        .inputs
        .iter()
        .any(|input| receipt_object(input.object));

    let receipt_outputs = operation
        .outputs
        .iter()
        .any(|output| receipt_object(output.object));

    if receipt_inputs && receipt_outputs && !has(WitnessId::ReceiptClassClosure) {
        errors.push(ManifestError::InvalidOperation {
            operation: operation.id,
            reason: "receipt input/output closure requires the receipt-class-closure witness",
        });
    }

    if !has(WitnessId::ValueFlowClosure) {
        errors.push(ManifestError::InvalidOperation {
            operation: operation.id,
            reason: "every operation requires the value-flow-closure witness",
        });
    }

    if operation.open_flows.contains(&OpenFlowKind::FeeSponsor)
        && !operation
            .value_flows
            .contains(&ValueFlowClass::SponsorEnvelope)
    {
        errors.push(ManifestError::InvalidOperation {
            operation: operation.id,
            reason: "an open fee-sponsor input requires the sponsor-envelope value flow",
        });
    }
}

/// The declared finite-bound set must equal the set of bounds
/// referenced by input, output, and data-output maxima.
fn validate_operation_bound_coverage(operation: &OperationSpec, errors: &mut Vec<ManifestError>) {
    let mut referenced = BTreeSet::new();

    let maxima = operation
        .inputs
        .iter()
        .map(|input| input.maximum)
        .chain(operation.outputs.iter().map(|output| output.maximum))
        .chain(operation.data_outputs.iter().map(|output| output.maximum));

    for maximum in maxima {
        if let MaxCount::Bound(bound) = maximum {
            referenced.insert(bound);
        }
    }

    let declared = operation.bounds.iter().copied().collect::<BTreeSet<_>>();

    if referenced != declared {
        errors.push(ManifestError::InvalidOperation {
            operation: operation.id,
            reason: "declared bounds disagree with referenced cardinality bounds",
        });
    }
}

/// Required v13 progress minima.
///
/// The executable shape policy follows whatever the manifest says, so
/// drift in a required minimum could otherwise silently relax a branch
/// contract. These facts pin the intended v13 declarations: each
/// operation must consume at least one progress object and each
/// destination family must retain its declared floor.
fn validate_required_operation_minima(
    architecture: &Architecture,
    errors: &mut Vec<ManifestError>,
) {
    let required_input_minima: &[(OperationId, ObjectId, u16)] = &[
        (OperationId::CancelRequest, ObjectId::DepositRequest, 1),
        (OperationId::AdmitDeposits, ObjectId::DepositRequest, 1),
        (
            OperationId::SettleDistribution,
            ObjectId::DepositEntitlement,
            1,
        ),
        (OperationId::TransferLive, ObjectId::ReceiptLive, 1),
        (
            OperationId::TransferTimeLocked,
            ObjectId::ReceiptTimeLocked,
            1,
        ),
        (OperationId::Redeem, ObjectId::ReceiptLive, 1),
        (OperationId::ReceiptRelabel, ObjectId::ReceiptTimeLocked, 1),
        (OperationId::Burn, ObjectId::ReceiptLive, 1),
        (OperationId::CompactAsh, ObjectId::Ash, 2),
        (OperationId::Clear, ObjectId::Ash, 1),
    ];

    let required_output_minima: &[(OperationId, ObjectId, u16)] = &[
        (OperationId::CreateRequest, ObjectId::DepositRequest, 1),
        (OperationId::CancelRequest, ObjectId::PlainLbtc, 1),
        (OperationId::AdmitDeposits, ObjectId::DepositEntitlement, 1),
        (OperationId::TransferLive, ObjectId::ReceiptLive, 1),
        (
            OperationId::TransferTimeLocked,
            ObjectId::ReceiptTimeLocked,
            1,
        ),
        (OperationId::Redeem, ObjectId::PlainLbtc, 1),
        (OperationId::ReceiptRelabel, ObjectId::ReceiptLive, 1),
        (OperationId::Burn, ObjectId::Ash, 1),
        (OperationId::CompactAsh, ObjectId::Ash, 1),
    ];

    for (operation_id, object, minimum) in required_input_minima {
        let Some(operation) = architecture.operation(*operation_id) else {
            continue;
        };

        let declared = operation
            .inputs
            .iter()
            .find(|input| input.object == *object)
            .map(|input| input.minimum);

        if declared != Some(*minimum) {
            errors.push(ManifestError::InvalidOperation {
                operation: *operation_id,
                reason: "required v13 input minimum is not declared",
            });
        }
    }

    for (operation_id, object, minimum) in required_output_minima {
        let Some(operation) = architecture.operation(*operation_id) else {
            continue;
        };

        let declared = operation
            .outputs
            .iter()
            .find(|output| output.object == *object)
            .map(|output| output.minimum);

        if declared != Some(*minimum) {
            errors.push(ManifestError::InvalidOperation {
                operation: *operation_id,
                reason: "required v13 output minimum is not declared",
            });
        }
    }
}

fn validate_quantities(architecture: &Architecture, errors: &mut Vec<ManifestError>) {
    for quantity in architecture.quantities {
        for reader in quantity.readers {
            if let ReaderId::Operation(operation) = reader {
                match architecture.operation(*operation) {
                    None => {
                        errors.push(ManifestError::MissingId {
                            category: "quantity operation reader",
                            id: operation.as_str().to_owned(),
                        });
                    }

                    // Every declared operation reader must correspond
                    // to an operation-side read declaration.
                    Some(spec) => {
                        if !spec.reads.contains(&quantity.id) {
                            errors.push(ManifestError::InvalidQuantity {
                                quantity: quantity.id,
                                reason: "operation reader does not declare the read",
                            });
                        }
                    }
                }
            }
        }

        for writer in quantity.writers {
            match architecture.operation(*writer) {
                None => {
                    errors.push(ManifestError::MissingId {
                        category: "quantity operation writer",
                        id: writer.as_str().to_owned(),
                    });
                }

                Some(spec) => {
                    if !spec.writes.contains(&quantity.id) {
                        errors.push(ManifestError::InvalidQuantity {
                            quantity: quantity.id,
                            reason: "operation writer does not declare the write",
                        });
                    }
                }
            }
        }

        // Audit-only quantities reject covenant readers and writers.
        if quantity.kind == QuantityKind::AuditOnly {
            let operation_reader = quantity
                .readers
                .iter()
                .any(|reader| matches!(reader, ReaderId::Operation(_)));

            if operation_reader || !quantity.writers.is_empty() {
                errors.push(ManifestError::InvalidQuantity {
                    quantity: quantity.id,
                    reason: "audit-only quantity may not have operation readers or writers",
                });
            }
        }
    }

    for historical in [
        QuantityId::HistoricalLiveResidue,
        QuantityId::HistoricalTimeLockedResidue,
    ] {
        let Some(quantity) = architecture.quantity(historical) else {
            errors.push(ManifestError::MissingId {
                category: "quantity",
                id: historical.as_str().to_owned(),
            });
            continue;
        };

        let allowed = BTreeSet::from([ReaderId::InvariantChecker, ReaderId::ExternalAuditor]);

        let actual = quantity.readers.iter().copied().collect::<BTreeSet<_>>();

        if actual != allowed {
            errors.push(ManifestError::InvalidQuantity {
                quantity: historical,
                reason: "historical residue must be audit-only",
            });
        }

        for operation in OperationId::ALL {
            if quantity.allows(ReaderId::Operation(*operation)) {
                errors.push(ManifestError::InvalidQuantity {
                    quantity: historical,
                    reason: "covenant operation may not read historical residue",
                });
            }
        }

        if quantity.allows(ReaderId::AttestationIndexer)
            || quantity.allows(ReaderId::ConsumerFormula)
        {
            errors.push(ManifestError::InvalidQuantity {
                quantity: historical,
                reason: "historical residue may not feed attestation or consumer formulas",
            });
        }
    }
}

fn validate_decisions(architecture: &Architecture, errors: &mut Vec<ManifestError>) {
    let required_closed = [
        DecisionId::NoOnchainAttestationAccumulator,
        DecisionId::FullChainAttestationVerification,
        DecisionId::HistoricalResidueAuditOnly,
        DecisionId::OneEntitlementPerRequest,
        DecisionId::SettlementPermissionless,
        DecisionId::AtomicMaturityCycle,
        DecisionId::ActiveBackingCap,
    ];

    for decision in required_closed {
        match architecture
            .decisions
            .iter()
            .find(|spec| spec.id == decision)
        {
            Some(spec) if spec.status == DecisionStatus::Closed => {}

            Some(_) => errors.push(ManifestError::InvalidDecision {
                decision,
                reason: "required v13 decision is not closed",
            }),

            None => errors.push(ManifestError::MissingId {
                category: "decision",
                id: decision.as_str().to_owned(),
            }),
        }
    }

    // The no-accumulator decision is structurally enforced: an
    // attestation-root role is not representable, so the declared root
    // set must be exactly the expected five roots.
    if architecture.roots.len() != RootId::ALL.len() {
        errors.push(ManifestError::InvalidDecision {
            decision: DecisionId::NoOnchainAttestationAccumulator,
            reason: "declared root set is not exactly the expected roots",
        });
    }
}

fn validate_tags(architecture: &Architecture, errors: &mut Vec<ManifestError>) {
    let attestation_tags = architecture
        .tags
        .iter()
        .filter(|tag| tag.participates_in_attestation)
        .map(|tag| tag.id)
        .collect::<Vec<_>>();

    if attestation_tags != vec![TagId::Burn] {
        errors.push(ManifestError::InvalidDecision {
            decision: DecisionId::NoOnchainAttestationAccumulator,
            reason: "only tag-burn may participate in attestation",
        });
    }
}

fn validate_amount_limits(architecture: &Architecture, errors: &mut Vec<ManifestError>) {
    let Some(limit) = architecture.amount_limit(AmountLimitId::ActiveBackingMax) else {
        errors.push(ManifestError::MissingId {
            category: "amount limit",
            id: AmountLimitId::ActiveBackingMax.as_str().to_owned(),
        });

        return;
    };

    if limit.asset != AssetId::Lbtc {
        errors.push(ManifestError::InvalidAsset {
            asset: limit.asset,
            reason: "active backing cap must be denominated in L-BTC",
        });
    }

    if limit.value == 0 || limit.value >= (1_u64 << 51) {
        errors.push(ManifestError::InvalidAsset {
            asset: AssetId::Lbtc,
            reason: "active backing cap must be positive and below 2^51",
        });
    }
}
