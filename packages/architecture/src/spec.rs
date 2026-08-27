//! Declarative v13 architecture specification.
//!
//! This module is deliberately a static data declaration, not
//! executable policy. The executable model, compiler configuration,
//! documentation generators, and exported JSON/TOML artifacts all
//! derive from the [`ARCHITECTURE`] constant.

use crate::ids::{
    AccountingDomain, AllocatorId, AmountLimitId, AssetClass, AssetId, AssetRole, BoundId, DataId,
    DataOutputKind, DeallocatorId, DecisionId, DecisionStatus, DeltaCondition, DeltaKind,
    DependencyId, InputAuthorization, InvariantClauseId, IssuanceCondition, LifecycleClass,
    ObjectId, OpenFlowKind, OperationId, OperationKind, PermissionClass, ProjectionId,
    ProjectionRule, PublicationStatus, QuantityId, QuantityKind, ReaderId, RootId, RootRole,
    RootUse, TagId, ValueFlowClass, WitnessId,
};

/// The architecture-export schema this crate implements. Envelope
/// metadata (never a hash input); artifact ingestion rejects any other
/// value.
pub const ARCHITECTURE_SCHEMA_VERSION: u32 = 17;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpecificationBinding {
    pub version: &'static str,
    pub anchor_set_hash: Option<[u8; 32]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DocumentSpec {
    /// The tracked binding to the compiler line.
    ///
    /// `major.minor` copied from the workspace version, patch
    /// always zero, the prerelease marker carried verbatim. Patch-blind
    /// and content-blind — the behavioural hash alone witnesses
    /// denotation stability. The binding gate derives the expected
    /// value from the workspace version at compile time.
    pub realization_version: &'static str,
    pub architecture_schema_version: u32,
    pub status: PublicationStatus,
    pub specification: SpecificationBinding,
    pub target_network: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssetSpec {
    pub id: AssetId,
    pub class: AssetClass,
    pub role: AssetRole,
    pub fixed_amount: Option<u64>,
    pub reissuable: bool,
    pub authority: Option<AssetId>,
    pub issue_operation: Option<OperationId>,
    pub destruction_operations: &'static [OperationId],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RootSpec {
    pub id: RootId,
    pub asset: AssetId,
    pub role: RootRole,
    pub fixed_amount: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RootUseSpec {
    pub root: RootId,
    pub use_kind: RootUse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IssuanceSpec {
    pub asset: AssetId,
    pub authority: AssetId,
    pub condition: IssuanceCondition,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaxCount {
    Exact(u16),
    Bound(BoundId),
}

/// One operation input family: the consumed object, its cardinality,
/// and the authorization required to consume it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputSpec {
    pub object: ObjectId,
    pub minimum: u16,
    pub maximum: MaxCount,
    pub authorization: InputAuthorization,
}

/// One operation output family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OutputSpec {
    pub object: ObjectId,
    pub minimum: u16,
    pub maximum: MaxCount,
}

/// One asset-specific canonical delta of an operation.
///
/// Destructive deltas name the domain-separated tag authenticating the
/// destruction; conditional deltas name the branch condition under
/// which the delta is present.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CanonicalDeltaSpec {
    pub asset: AssetId,
    pub kind: DeltaKind,
    pub condition: DeltaCondition,
    pub destruction_tag: Option<TagId>,
}

/// One permitted data-output family of an operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataOutputSpec {
    pub kind: DataOutputKind,
    pub tag: TagId,
    pub asset: Option<AssetId>,

    pub minimum: u16,
    pub maximum: MaxCount,

    pub condition: DeltaCondition,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectionSpec {
    pub projection: ProjectionId,
    pub rule: ProjectionRule,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectSpec {
    pub id: ObjectId,
    pub asset: AssetId,

    pub lifecycle: LifecycleClass,
    pub accounting_domain: AccountingDomain,

    /// Some object types have more than one allocation provenance
    /// (for example live receipts arise from genesis, cycle issuance,
    /// settlement, transfer, and relabel), so allocation is a set.
    pub allocators: &'static [AllocatorId],
    pub mutators: &'static [OperationId],
    pub deallocators: &'static [DeallocatorId],

    pub witnesses: &'static [WitnessId],

    /// Whether the object's actual asset value is authoritative.
    pub consensus_value_authoritative: bool,
}

/// Whether an object schema commits an owner whose consent can
/// authorize consumption of the object.
///
/// An `owner` field is not sufficient: the deposit entitlement commits
/// an owner, but that owner is a *routing destination* for settlement
/// outputs, never a consent gate — entitlement consumption is
/// permissionless. Owner-bearing membership is about consent, not
/// about carrying an owner key.
pub fn owner_bearing(object: ObjectId) -> bool {
    matches!(
        object,
        ObjectId::ReceiptLive
            | ObjectId::ReceiptTimeLocked
            | ObjectId::DepositRequest
            | ObjectId::PlainLbtc
    )
}

/// Whether an asset is a canonical value asset of the exact canonical
/// partition.
///
/// Every consumed input and created output of a canonical value asset
/// belongs to exactly one declared issuance or flow. Singleton identity
/// and authority assets are governed by root succession and closed-asset
/// conservation instead, and open assets by the exact open-flow
/// partition.
pub fn canonical_value_asset(asset: AssetId) -> bool {
    matches!(asset, AssetId::U | AssetId::Ent | AssetId::DistCtl)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OperationSpec {
    pub id: OperationId,
    pub kind: OperationKind,
    pub authorization: PermissionClass,

    pub roots: &'static [RootUseSpec],
    pub issuances: &'static [IssuanceSpec],

    pub inputs: &'static [InputSpec],
    pub outputs: &'static [OutputSpec],

    pub canonical_deltas: &'static [CanonicalDeltaSpec],
    pub data_outputs: &'static [DataOutputSpec],
    pub open_flows: &'static [OpenFlowKind],

    pub reads: &'static [QuantityId],
    pub writes: &'static [QuantityId],

    pub witnesses: &'static [WitnessId],
    pub value_flows: &'static [ValueFlowClass],
    pub bounds: &'static [BoundId],
    pub projections: &'static [ProjectionSpec],
}

impl OperationSpec {
    pub fn root_use(&self, root: RootId) -> RootUse {
        self.roots
            .iter()
            .find(|entry| entry.root == root)
            .map_or(RootUse::Forbidden, |entry| entry.use_kind)
    }

    pub fn projection_rule(&self, projection: ProjectionId) -> ProjectionRule {
        self.projections
            .iter()
            .find(|entry| entry.projection == projection)
            .map_or(ProjectionRule::Forbidden, |entry| entry.rule)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuantitySpec {
    pub id: QuantityId,
    pub kind: QuantityKind,
    pub reads: &'static [DataId],
    pub readers: &'static [ReaderId],
    pub writers: &'static [OperationId],
}

impl QuantitySpec {
    pub fn allows(&self, reader: ReaderId) -> bool {
        self.readers.contains(&reader)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WitnessSpec {
    pub id: WitnessId,
    pub semantic_tag: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DependencySpec {
    pub id: DependencyId,

    /// Whether a deployment must present verification evidence for
    /// this dependency. This expresses a requirement, not a completed
    /// status; completion lives in the deployment profile's
    /// [`crate::deployment::DependencyEvidence`].
    pub verification_required: bool,

    pub rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecisionSpec {
    pub id: DecisionId,
    pub status: DecisionStatus,
    pub rationale: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoundSpec {
    pub id: BoundId,
    pub default_value: Option<u64>,
    pub requires_deployment_calibration: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmountLimitSpec {
    pub id: AmountLimitId,
    pub asset: AssetId,
    pub value: u64,
    pub unit: &'static str,
    pub rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TagSpec {
    pub id: TagId,
    pub participates_in_attestation: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Architecture {
    pub document: DocumentSpec,
    pub assets: &'static [AssetSpec],
    pub roots: &'static [RootSpec],
    pub objects: &'static [ObjectSpec],
    pub operations: &'static [OperationSpec],
    pub quantities: &'static [QuantitySpec],
    pub witnesses: &'static [WitnessSpec],
    pub clauses: &'static [InvariantClauseId],
    pub dependencies: &'static [DependencySpec],
    pub decisions: &'static [DecisionSpec],
    pub bounds: &'static [BoundSpec],
    pub amount_limits: &'static [AmountLimitSpec],
    pub tags: &'static [TagSpec],
}

impl Architecture {
    pub fn asset(&self, id: AssetId) -> Option<&AssetSpec> {
        self.assets.iter().find(|asset| asset.id == id)
    }

    pub fn root(&self, id: RootId) -> Option<&RootSpec> {
        self.roots.iter().find(|root| root.id == id)
    }

    pub fn object(&self, id: ObjectId) -> Option<&ObjectSpec> {
        self.objects.iter().find(|object| object.id == id)
    }

    pub fn operation(&self, id: OperationId) -> Option<&OperationSpec> {
        self.operations.iter().find(|operation| operation.id == id)
    }

    pub fn quantity(&self, id: QuantityId) -> Option<&QuantitySpec> {
        self.quantities.iter().find(|quantity| quantity.id == id)
    }

    pub fn bound(&self, id: BoundId) -> Option<&BoundSpec> {
        self.bounds.iter().find(|bound| bound.id == id)
    }

    pub fn amount_limit(&self, id: AmountLimitId) -> Option<&AmountLimitSpec> {
        self.amount_limits.iter().find(|limit| limit.id == id)
    }
}

/// Whether a value is a well-formed tracked realization version.
///
/// The form is `<major>.<minor>.0`, optionally `-<prerelease>` —
/// decimal major and minor, patch exactly zero (the binding is
/// patch-blind, so the field can never carry one), and a nonempty
/// lowercase-alphanumeric prerelease marker with interior `.` or `-`.
#[must_use]
pub fn realization_version_well_formed(value: &str) -> bool {
    let (core, prerelease) = match value.split_once('-') {
        Some((core, prerelease)) => (core, Some(prerelease)),
        None => (value, None),
    };

    let mut components = core.split('.');
    let well_formed_core = matches!(
        (
            components.next(),
            components.next(),
            components.next(),
            components.next(),
        ),
        (Some(major), Some(minor), Some("0"), None)
            if !major.is_empty()
                && !minor.is_empty()
                && major.bytes().all(|byte| byte.is_ascii_digit())
                && minor.bytes().all(|byte| byte.is_ascii_digit())
    );

    well_formed_core
        && prerelease.is_none_or(|marker| {
            !marker.is_empty()
                && marker.bytes().all(|byte| {
                    byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || byte == b'.'
                        || byte == b'-'
                })
        })
}

/// The tracked realization version a workspace version implies.
///
/// `major.minor` copied, patch zeroed, the prerelease marker carried
/// verbatim. The binding gate compares [`DOCUMENT`]'s recorded value
/// against this derivation over `CARGO_PKG_VERSION`, so a compiler
/// minor bump fails the build until the recorded binding is bumped
/// with it — every version movement is a deliberate, recorded event
/// even when no content changed.
#[must_use]
pub fn tracked_realization_version(workspace_version: &str) -> String {
    let (core, prerelease) = match workspace_version.split_once('-') {
        Some((core, prerelease)) => (core, Some(prerelease)),
        None => (workspace_version, None),
    };

    let mut components = core.split('.');
    let major = components.next().unwrap_or("0");
    let minor = components.next().unwrap_or("0");

    match prerelease {
        Some(marker) => format!("{major}.{minor}.0-{marker}"),
        None => format!("{major}.{minor}.0"),
    }
}

// -------------------------------------------------------------------------
// Document
// -------------------------------------------------------------------------

pub const DOCUMENT: DocumentSpec = DocumentSpec {
    // The realization follows the compiler line's tracked binding. The
    // field is outside both hashes, and the versioning gate derives the
    // expected value from the workspace version.
    realization_version: "0.6.0-dev",
    // Schema 17: lifts the version fields into the publication
    // envelope, adds the behavioural hash, and splits the
    // explicit-values dependency into its four proof-method components.
    // Behavioural arrays are unchanged from schema 16.
    architecture_schema_version: ARCHITECTURE_SCHEMA_VERSION,
    status: PublicationStatus::Final,
    specification: SpecificationBinding {
        version: "1.0.0",
        // sha256 over the domain prefix and the newline-joined sorted
        // distinct specification anchor names harvested from the
        // document's §17 index (A- prefix stripped): the pin ceremony
        // of (´[RZ-rem:overview:anchor-pin]´) under algorithm
        // `sha256-anchor-set-v2`. Re-pinned by the specification's
        // v1.0.0 release, which renamed the paper's own-division label
        // area to `attestation`: two of the thirty-eight anchors are
        // renamed (`open:attestation:leverage-timing` and
        // `open:attestation:operator-disambiguation` carry the new area), so
        // this is a rename of label values and the measured set moved
        // with them. The recipe is unchanged.
        anchor_set_hash: Some([
            0x8a, 0x7c, 0xe7, 0x65, 0xd3, 0x3c, 0x08, 0xee, 0x9e, 0x51, 0x32, 0x05, 0x3a, 0x83,
            0x08, 0xed, 0xec, 0x46, 0xd9, 0xe1, 0x92, 0x12, 0x8e, 0xb0, 0x76, 0xe2, 0x57, 0xea,
            0xbb, 0xc1, 0x6e, 0x2f,
        ]),
    },
    target_network: "liquid",
};

// -------------------------------------------------------------------------
// Assets
// -------------------------------------------------------------------------

pub const ASSETS: &[AssetSpec] = &[
    AssetSpec {
        id: AssetId::Lbtc,
        class: AssetClass::Open,
        role: AssetRole::ReserveAsset,
        fixed_amount: None,
        reissuable: false,
        authority: None,
        issue_operation: None,
        destruction_operations: &[],
    },
    AssetSpec {
        id: AssetId::U,
        class: AssetClass::Closed,
        role: AssetRole::MonetaryReceipt,
        fixed_amount: None,
        reissuable: true,
        authority: Some(AssetId::Pace),
        issue_operation: Some(OperationId::Cycle),
        // ASH movement is lateral, not destruction; terminal
        // destruction of U occurs at clear, redemption, or the
        // residue leg of a terminal distribution settlement.
        destruction_operations: &[
            OperationId::Redeem,
            OperationId::Clear,
            OperationId::SettleDistribution,
        ],
    },
    AssetSpec {
        id: AssetId::Ent,
        class: AssetClass::Closed,
        role: AssetRole::DepositEntitlement,
        fixed_amount: None,
        reissuable: true,
        authority: Some(AssetId::EntAuth),
        issue_operation: Some(OperationId::AdmitDeposits),
        destruction_operations: &[OperationId::SettleDistribution],
    },
    AssetSpec {
        id: AssetId::DistCtl,
        class: AssetClass::Closed,
        role: AssetRole::DistributionControl,
        fixed_amount: None,
        reissuable: true,
        authority: Some(AssetId::DistAuth),
        issue_operation: Some(OperationId::Cycle),
        destruction_operations: &[OperationId::SettleDistribution],
    },
    AssetSpec {
        id: AssetId::Pid,
        class: AssetClass::Closed,
        role: AssetRole::Identity,
        fixed_amount: Some(1),
        reissuable: false,
        authority: None,
        issue_operation: None,
        destruction_operations: &[],
    },
    AssetSpec {
        id: AssetId::Pace,
        class: AssetClass::Closed,
        role: AssetRole::IssuanceAuthority,
        fixed_amount: Some(1),
        reissuable: false,
        authority: None,
        issue_operation: None,
        destruction_operations: &[],
    },
    AssetSpec {
        id: AssetId::EntAuth,
        class: AssetClass::Closed,
        role: AssetRole::IssuanceAuthority,
        fixed_amount: Some(1),
        reissuable: false,
        authority: None,
        issue_operation: None,
        destruction_operations: &[],
    },
    AssetSpec {
        id: AssetId::DistAuth,
        class: AssetClass::Closed,
        role: AssetRole::IssuanceAuthority,
        fixed_amount: Some(1),
        reissuable: false,
        authority: None,
        issue_operation: None,
        destruction_operations: &[],
    },
];

// -------------------------------------------------------------------------
// Roots
// -------------------------------------------------------------------------

pub const ROOTS: &[RootSpec] = &[
    RootSpec {
        id: RootId::State,
        asset: AssetId::Pid,
        role: RootRole::StateIdentity,
        fixed_amount: Some(1),
    },
    RootSpec {
        id: RootId::Resv,
        asset: AssetId::Lbtc,
        role: RootRole::ActiveReserve,
        fixed_amount: None,
    },
    RootSpec {
        id: RootId::Pace,
        asset: AssetId::Pace,
        role: RootRole::CadenceAndIssuance,
        fixed_amount: Some(1),
    },
    RootSpec {
        id: RootId::EntAuth,
        asset: AssetId::EntAuth,
        role: RootRole::EntitlementAuthority,
        fixed_amount: Some(1),
    },
    RootSpec {
        id: RootId::DistAuth,
        asset: AssetId::DistAuth,
        role: RootRole::DistributionAuthority,
        fixed_amount: Some(1),
    },
];

// -------------------------------------------------------------------------
// Shared operation fragments
// -------------------------------------------------------------------------

const ROOTS_NONE: &[RootUseSpec] = &[];

const ROOTS_ADMISSION: &[RootUseSpec] = &[
    RootUseSpec {
        root: RootId::State,
        use_kind: RootUse::Succession,
    },
    RootUseSpec {
        root: RootId::Resv,
        use_kind: RootUse::Succession,
    },
    RootUseSpec {
        root: RootId::EntAuth,
        use_kind: RootUse::Succession,
    },
];

const ROOTS_CYCLE: &[RootUseSpec] = &[
    RootUseSpec {
        root: RootId::State,
        use_kind: RootUse::Succession,
    },
    RootUseSpec {
        root: RootId::Resv,
        use_kind: RootUse::Succession,
    },
    RootUseSpec {
        root: RootId::Pace,
        use_kind: RootUse::Succession,
    },
    RootUseSpec {
        root: RootId::DistAuth,
        use_kind: RootUse::Succession,
    },
];

const ROOTS_REDEEM: &[RootUseSpec] = &[
    RootUseSpec {
        root: RootId::State,
        use_kind: RootUse::Succession,
    },
    RootUseSpec {
        root: RootId::Resv,
        use_kind: RootUse::SuccessionOrTermination,
    },
];

const ROOTS_STATE_ONLY: &[RootUseSpec] = &[RootUseSpec {
    root: RootId::State,
    use_kind: RootUse::Succession,
}];

const NO_ISSUANCE: &[IssuanceSpec] = &[];

const ISSUANCE_ADMISSION: &[IssuanceSpec] = &[IssuanceSpec {
    asset: AssetId::Ent,
    authority: AssetId::EntAuth,
    condition: IssuanceCondition::PositiveAdmittedPrincipal,
}];

const ISSUANCE_CYCLE: &[IssuanceSpec] = &[
    IssuanceSpec {
        asset: AssetId::U,
        authority: AssetId::Pace,
        condition: IssuanceCondition::PositiveCycleIssuance,
    },
    IssuanceSpec {
        asset: AssetId::DistCtl,
        authority: AssetId::DistAuth,
        condition: IssuanceCondition::PositiveCyclePrincipal,
    },
];

const NO_DELTAS: &[CanonicalDeltaSpec] = &[];

const NO_DATA_OUTPUTS: &[DataOutputSpec] = &[];

// Every accepted modeled transaction induces one transition
// certificate; only selected operations induce specialized event
// projections.

const PROJECTION_TRANSITION: &[ProjectionSpec] = &[ProjectionSpec {
    projection: ProjectionId::TransitionCertificate,
    rule: ProjectionRule::Required,
}];

const PROJECTION_BURN: &[ProjectionSpec] = &[
    ProjectionSpec {
        projection: ProjectionId::TransitionCertificate,
        rule: ProjectionRule::Required,
    },
    ProjectionSpec {
        projection: ProjectionId::BurnEvent,
        rule: ProjectionRule::Required,
    },
];

const PROJECTION_CLEAR: &[ProjectionSpec] = &[
    ProjectionSpec {
        projection: ProjectionId::TransitionCertificate,
        rule: ProjectionRule::Required,
    },
    ProjectionSpec {
        projection: ProjectionId::ClearEvent,
        rule: ProjectionRule::Required,
    },
];

const PROJECTION_SETTLEMENT: &[ProjectionSpec] = &[
    ProjectionSpec {
        projection: ProjectionId::TransitionCertificate,
        rule: ProjectionRule::Required,
    },
    ProjectionSpec {
        projection: ProjectionId::DistributionResidue,
        rule: ProjectionRule::Optional,
    },
];

// -------------------------------------------------------------------------
// Operations
// -------------------------------------------------------------------------

pub const OPERATIONS: &[OperationSpec] = &[
    OperationSpec {
        id: OperationId::CreateRequest,
        kind: OperationKind::ClientProtocol,
        authorization: PermissionClass::ClientAuthorized,
        roots: ROOTS_NONE,
        issuances: NO_ISSUANCE,
        inputs: &[InputSpec {
            object: ObjectId::PlainLbtc,
            minimum: 1,
            maximum: MaxCount::Bound(BoundId::FeeSponsorInputMax),
            authorization: InputAuthorization::InputOwner,
        }],
        outputs: &[
            OutputSpec {
                object: ObjectId::DepositRequest,
                minimum: 1,
                maximum: MaxCount::Exact(1),
            },
            OutputSpec {
                object: ObjectId::PlainLbtc,
                minimum: 0,
                maximum: MaxCount::Exact(1),
            },
        ],
        canonical_deltas: NO_DELTAS,
        data_outputs: NO_DATA_OUTPUTS,
        open_flows: &[OpenFlowKind::RequestCreation],
        reads: &[],
        writes: &[],
        witnesses: &[WitnessId::RequestClosure, WitnessId::ValueFlowClosure],
        value_flows: &[
            ValueFlowClass::OwnerConsented,
            ValueFlowClass::ImmutableDestination,
        ],
        bounds: &[BoundId::FeeSponsorInputMax],
        projections: PROJECTION_TRANSITION,
    },
    OperationSpec {
        id: OperationId::CancelRequest,
        kind: OperationKind::CovenantBranch,
        authorization: PermissionClass::RefundKey,
        roots: ROOTS_NONE,
        issuances: NO_ISSUANCE,
        inputs: &[
            InputSpec {
                object: ObjectId::DepositRequest,
                minimum: 1,
                maximum: MaxCount::Exact(1),
                authorization: InputAuthorization::RefundKey,
            },
            InputSpec {
                object: ObjectId::PlainLbtc,
                minimum: 0,
                maximum: MaxCount::Bound(BoundId::FeeSponsorInputMax),
                authorization: InputAuthorization::SponsorOwner,
            },
        ],
        outputs: &[OutputSpec {
            object: ObjectId::PlainLbtc,
            minimum: 1,
            maximum: MaxCount::Exact(2),
        }],
        canonical_deltas: NO_DELTAS,
        data_outputs: NO_DATA_OUTPUTS,
        open_flows: &[OpenFlowKind::RequestRefund, OpenFlowKind::FeeSponsor],
        reads: &[],
        writes: &[],
        witnesses: &[WitnessId::RequestClosure, WitnessId::ValueFlowClosure],
        value_flows: &[
            ValueFlowClass::OwnerConsented,
            ValueFlowClass::ImmutableDestination,
            ValueFlowClass::SponsorEnvelope,
        ],
        bounds: &[BoundId::FeeSponsorInputMax],
        projections: PROJECTION_TRANSITION,
    },
    OperationSpec {
        id: OperationId::AdmitDeposits,
        kind: OperationKind::CovenantBranch,
        authorization: PermissionClass::Permissionless,
        roots: ROOTS_ADMISSION,
        issuances: ISSUANCE_ADMISSION,
        inputs: &[
            InputSpec {
                object: ObjectId::State,
                minimum: 1,
                maximum: MaxCount::Exact(1),
                authorization: InputAuthorization::CovenantCompanion,
            },
            InputSpec {
                object: ObjectId::Resv,
                minimum: 1,
                maximum: MaxCount::Exact(1),
                authorization: InputAuthorization::CovenantCompanion,
            },
            InputSpec {
                object: ObjectId::EntitlementAuthority,
                minimum: 1,
                maximum: MaxCount::Exact(1),
                authorization: InputAuthorization::CovenantCompanion,
            },
            InputSpec {
                object: ObjectId::DepositRequest,
                minimum: 1,
                maximum: MaxCount::Bound(BoundId::AdmissionBatchMax),
                authorization: InputAuthorization::Permissionless,
            },
        ],
        outputs: &[
            OutputSpec {
                object: ObjectId::State,
                minimum: 1,
                maximum: MaxCount::Exact(1),
            },
            OutputSpec {
                object: ObjectId::Resv,
                minimum: 1,
                maximum: MaxCount::Exact(1),
            },
            OutputSpec {
                object: ObjectId::EntitlementAuthority,
                minimum: 1,
                maximum: MaxCount::Exact(1),
            },
            OutputSpec {
                object: ObjectId::DepositEntitlement,
                minimum: 1,
                maximum: MaxCount::Bound(BoundId::AdmissionBatchMax),
            },
            OutputSpec {
                object: ObjectId::PlainLbtc,
                minimum: 0,
                maximum: MaxCount::Exact(1),
            },
        ],
        canonical_deltas: &[CanonicalDeltaSpec {
            asset: AssetId::Ent,
            kind: DeltaKind::Issuance,
            condition: DeltaCondition::PositiveAdmittedPrincipal,
            destruction_tag: None,
        }],
        data_outputs: NO_DATA_OUTPUTS,
        open_flows: &[OpenFlowKind::DepositAdmission],
        reads: &[],
        writes: &[],
        witnesses: &[
            WitnessId::CanonicalDelta,
            WitnessId::StateSuccession,
            WitnessId::ResvSuccession,
            WitnessId::EntitlementClosure,
            WitnessId::ValueFlowClosure,
        ],
        value_flows: &[
            ValueFlowClass::ImmutableDestination,
            ValueFlowClass::PreauthorizedServiceBudget,
        ],
        bounds: &[BoundId::AdmissionBatchMax],
        projections: PROJECTION_TRANSITION,
    },
    OperationSpec {
        id: OperationId::Cycle,
        kind: OperationKind::CovenantBranch,
        authorization: PermissionClass::CadenceBand,
        roots: ROOTS_CYCLE,
        issuances: ISSUANCE_CYCLE,
        inputs: &[
            InputSpec {
                object: ObjectId::State,
                minimum: 1,
                maximum: MaxCount::Exact(1),
                authorization: InputAuthorization::CovenantCompanion,
            },
            InputSpec {
                object: ObjectId::Resv,
                minimum: 1,
                maximum: MaxCount::Exact(1),
                authorization: InputAuthorization::CovenantCompanion,
            },
            InputSpec {
                object: ObjectId::Pace,
                minimum: 1,
                maximum: MaxCount::Exact(1),
                authorization: InputAuthorization::CovenantCompanion,
            },
            InputSpec {
                object: ObjectId::DistributionAuthority,
                minimum: 1,
                maximum: MaxCount::Exact(1),
                authorization: InputAuthorization::CovenantCompanion,
            },
            InputSpec {
                object: ObjectId::PlainLbtc,
                minimum: 0,
                maximum: MaxCount::Bound(BoundId::FeeSponsorInputMax),
                authorization: InputAuthorization::SponsorOwner,
            },
        ],
        outputs: &[
            OutputSpec {
                object: ObjectId::State,
                minimum: 1,
                maximum: MaxCount::Exact(1),
            },
            OutputSpec {
                object: ObjectId::Resv,
                minimum: 1,
                maximum: MaxCount::Exact(1),
            },
            OutputSpec {
                object: ObjectId::Pace,
                minimum: 1,
                maximum: MaxCount::Exact(1),
            },
            OutputSpec {
                object: ObjectId::DistributionAuthority,
                minimum: 1,
                maximum: MaxCount::Exact(1),
            },
            OutputSpec {
                object: ObjectId::ReceiptLive,
                minimum: 0,
                // The operator fee is one receipt per class: pre-maturity the live
                // share is a single RECEIPT_L; the conversion cycle's entire fee is a
                // single live receipt. Two live receipts are unreachable.
                maximum: MaxCount::Exact(1),
            },
            OutputSpec {
                object: ObjectId::ReceiptTimeLocked,
                minimum: 0,
                maximum: MaxCount::Exact(1),
            },
            OutputSpec {
                object: ObjectId::DistributionControl,
                minimum: 0,
                maximum: MaxCount::Exact(1),
            },
            OutputSpec {
                object: ObjectId::DistributionVault,
                minimum: 0,
                maximum: MaxCount::Exact(1),
            },
            OutputSpec {
                object: ObjectId::CpfpAnchor,
                minimum: 0,
                maximum: MaxCount::Exact(1),
            },
            OutputSpec {
                object: ObjectId::PlainLbtc,
                minimum: 0,
                maximum: MaxCount::Exact(1),
            },
        ],
        canonical_deltas: &[
            CanonicalDeltaSpec {
                asset: AssetId::U,
                kind: DeltaKind::Issuance,
                condition: DeltaCondition::PositiveCycleIssuance,
                destruction_tag: None,
            },
            CanonicalDeltaSpec {
                asset: AssetId::DistCtl,
                kind: DeltaKind::Issuance,
                condition: DeltaCondition::PositiveCyclePrincipal,
                destruction_tag: None,
            },
        ],
        data_outputs: NO_DATA_OUTPUTS,
        open_flows: &[OpenFlowKind::ReserveCarry, OpenFlowKind::FeeSponsor],
        reads: &[QuantityId::CycleIssuance, QuantityId::Floor],
        writes: &[],
        witnesses: &[
            WitnessId::CanonicalDelta,
            WitnessId::StateSuccession,
            WitnessId::ResvSuccession,
            WitnessId::DistributionClosure,
            WitnessId::FloorNondecrease,
            WitnessId::NativeFeeAuction,
            WitnessId::ValueFlowClosure,
        ],
        value_flows: &[
            ValueFlowClass::ImmutableDestination,
            ValueFlowClass::SponsorEnvelope,
        ],
        bounds: &[BoundId::FeeSponsorInputMax],
        projections: PROJECTION_TRANSITION,
    },
    OperationSpec {
        id: OperationId::SettleDistribution,
        kind: OperationKind::CovenantBranch,
        authorization: PermissionClass::Permissionless,
        roots: ROOTS_NONE,
        issuances: NO_ISSUANCE,
        inputs: &[
            InputSpec {
                object: ObjectId::DistributionControl,
                minimum: 1,
                maximum: MaxCount::Exact(1),
                authorization: InputAuthorization::Permissionless,
            },
            InputSpec {
                object: ObjectId::DistributionVault,
                minimum: 0,
                maximum: MaxCount::Exact(1),
                authorization: InputAuthorization::Permissionless,
            },
            InputSpec {
                object: ObjectId::DepositEntitlement,
                minimum: 1,
                maximum: MaxCount::Bound(BoundId::SettlementBatchMax),
                authorization: InputAuthorization::Permissionless,
            },
            InputSpec {
                object: ObjectId::PlainLbtc,
                minimum: 0,
                maximum: MaxCount::Bound(BoundId::FeeSponsorInputMax),
                authorization: InputAuthorization::SponsorOwner,
            },
        ],
        outputs: &[
            OutputSpec {
                object: ObjectId::DistributionControl,
                minimum: 0,
                maximum: MaxCount::Exact(1),
            },
            OutputSpec {
                object: ObjectId::DistributionVault,
                minimum: 0,
                maximum: MaxCount::Exact(1),
            },
            OutputSpec {
                object: ObjectId::ReceiptLive,
                minimum: 0,
                maximum: MaxCount::Bound(BoundId::SettlementBatchMax),
            },
            OutputSpec {
                object: ObjectId::ReceiptTimeLocked,
                minimum: 0,
                maximum: MaxCount::Bound(BoundId::SettlementBatchMax),
            },
            OutputSpec {
                object: ObjectId::PlainLbtc,
                minimum: 0,
                maximum: MaxCount::Exact(1),
            },
        ],
        canonical_deltas: &[
            CanonicalDeltaSpec {
                asset: AssetId::Ent,
                kind: DeltaKind::Destruction,
                condition: DeltaCondition::Always,
                destruction_tag: Some(TagId::Entitlement),
            },
            CanonicalDeltaSpec {
                asset: AssetId::DistCtl,
                kind: DeltaKind::Lateral,
                condition: DeltaCondition::DistributionContinues,
                destruction_tag: None,
            },
            CanonicalDeltaSpec {
                asset: AssetId::DistCtl,
                kind: DeltaKind::Destruction,
                condition: DeltaCondition::DistributionTerminates,
                destruction_tag: Some(TagId::DistributionControlClose),
            },
            CanonicalDeltaSpec {
                asset: AssetId::U,
                kind: DeltaKind::Lateral,
                condition: DeltaCondition::PositiveSettlementUOutput,
                destruction_tag: None,
            },
            CanonicalDeltaSpec {
                asset: AssetId::U,
                kind: DeltaKind::Destruction,
                condition: DeltaCondition::PositiveDistributionResidue,
                destruction_tag: Some(TagId::DistributionResidue),
            },
        ],
        data_outputs: &[
            DataOutputSpec {
                kind: DataOutputKind::Destruction,
                tag: TagId::Entitlement,
                asset: Some(AssetId::Ent),
                minimum: 1,
                maximum: MaxCount::Exact(1),
                condition: DeltaCondition::Always,
            },
            DataOutputSpec {
                kind: DataOutputKind::Destruction,
                tag: TagId::DistributionControlClose,
                asset: Some(AssetId::DistCtl),
                minimum: 1,
                maximum: MaxCount::Exact(1),
                condition: DeltaCondition::DistributionTerminates,
            },
            DataOutputSpec {
                kind: DataOutputKind::Destruction,
                tag: TagId::DistributionResidue,
                asset: Some(AssetId::U),
                minimum: 1,
                maximum: MaxCount::Exact(1),
                condition: DeltaCondition::PositiveDistributionResidue,
            },
        ],
        open_flows: &[OpenFlowKind::FeeSponsor],
        reads: &[],
        writes: &[],
        witnesses: &[
            WitnessId::CanonicalDelta,
            WitnessId::EntitlementClosure,
            WitnessId::DistributionClosure,
            WitnessId::ReceiptOwnerRouting,
            WitnessId::ValueFlowClosure,
            WitnessId::UtxoLifecycle,
        ],
        value_flows: &[
            ValueFlowClass::ImmutableDestination,
            ValueFlowClass::OwnerlessTerminalSink,
            ValueFlowClass::SponsorEnvelope,
        ],
        bounds: &[BoundId::SettlementBatchMax, BoundId::FeeSponsorInputMax],
        projections: PROJECTION_SETTLEMENT,
    },
    OperationSpec {
        id: OperationId::TransferLive,
        kind: OperationKind::CovenantBranch,
        authorization: PermissionClass::ReceiptOwners,
        roots: ROOTS_NONE,
        issuances: NO_ISSUANCE,
        inputs: &[
            InputSpec {
                object: ObjectId::ReceiptLive,
                minimum: 1,
                maximum: MaxCount::Bound(BoundId::TransferInputMax),
                authorization: InputAuthorization::InputOwner,
            },
            InputSpec {
                object: ObjectId::PlainLbtc,
                minimum: 0,
                maximum: MaxCount::Bound(BoundId::FeeSponsorInputMax),
                authorization: InputAuthorization::SponsorOwner,
            },
        ],
        outputs: &[
            OutputSpec {
                object: ObjectId::ReceiptLive,
                minimum: 1,
                maximum: MaxCount::Bound(BoundId::TransferOutputMax),
            },
            OutputSpec {
                object: ObjectId::PlainLbtc,
                minimum: 0,
                maximum: MaxCount::Exact(1),
            },
        ],
        canonical_deltas: &[CanonicalDeltaSpec {
            asset: AssetId::U,
            kind: DeltaKind::Lateral,
            condition: DeltaCondition::Always,
            destruction_tag: None,
        }],
        data_outputs: NO_DATA_OUTPUTS,
        open_flows: &[OpenFlowKind::FeeSponsor],
        reads: &[],
        writes: &[],
        witnesses: &[
            WitnessId::CanonicalDelta,
            WitnessId::ReceiptOwnerRouting,
            WitnessId::ReceiptClassClosure,
            WitnessId::ValueFlowClosure,
        ],
        value_flows: &[
            ValueFlowClass::OwnerConsented,
            ValueFlowClass::SponsorEnvelope,
        ],
        bounds: &[
            BoundId::TransferInputMax,
            BoundId::TransferOutputMax,
            BoundId::FeeSponsorInputMax,
        ],
        projections: PROJECTION_TRANSITION,
    },
    OperationSpec {
        id: OperationId::TransferTimeLocked,
        kind: OperationKind::CovenantBranch,
        authorization: PermissionClass::ReceiptOwners,
        roots: ROOTS_NONE,
        issuances: NO_ISSUANCE,
        inputs: &[
            InputSpec {
                object: ObjectId::ReceiptTimeLocked,
                minimum: 1,
                maximum: MaxCount::Bound(BoundId::TransferInputMax),
                authorization: InputAuthorization::InputOwner,
            },
            InputSpec {
                object: ObjectId::PlainLbtc,
                minimum: 0,
                maximum: MaxCount::Bound(BoundId::FeeSponsorInputMax),
                authorization: InputAuthorization::SponsorOwner,
            },
        ],
        outputs: &[
            OutputSpec {
                object: ObjectId::ReceiptTimeLocked,
                minimum: 1,
                maximum: MaxCount::Bound(BoundId::TransferOutputMax),
            },
            OutputSpec {
                object: ObjectId::PlainLbtc,
                minimum: 0,
                maximum: MaxCount::Exact(1),
            },
        ],
        canonical_deltas: &[CanonicalDeltaSpec {
            asset: AssetId::U,
            kind: DeltaKind::Lateral,
            condition: DeltaCondition::Always,
            destruction_tag: None,
        }],
        data_outputs: NO_DATA_OUTPUTS,
        open_flows: &[OpenFlowKind::FeeSponsor],
        reads: &[],
        writes: &[],
        witnesses: &[
            WitnessId::CanonicalDelta,
            WitnessId::ReceiptOwnerRouting,
            WitnessId::ReceiptClassClosure,
            WitnessId::ValueFlowClosure,
        ],
        value_flows: &[
            ValueFlowClass::OwnerConsented,
            ValueFlowClass::SponsorEnvelope,
        ],
        bounds: &[
            BoundId::TransferInputMax,
            BoundId::TransferOutputMax,
            BoundId::FeeSponsorInputMax,
        ],
        projections: PROJECTION_TRANSITION,
    },
    OperationSpec {
        id: OperationId::Redeem,
        kind: OperationKind::CovenantBranch,
        authorization: PermissionClass::ReceiptOwners,
        roots: ROOTS_REDEEM,
        issuances: NO_ISSUANCE,
        inputs: &[
            InputSpec {
                object: ObjectId::State,
                minimum: 1,
                maximum: MaxCount::Exact(1),
                authorization: InputAuthorization::CovenantCompanion,
            },
            InputSpec {
                object: ObjectId::Resv,
                minimum: 1,
                maximum: MaxCount::Exact(1),
                authorization: InputAuthorization::CovenantCompanion,
            },
            InputSpec {
                object: ObjectId::ReceiptLive,
                minimum: 1,
                maximum: MaxCount::Exact(1),
                authorization: InputAuthorization::InputOwner,
            },
            InputSpec {
                object: ObjectId::PlainLbtc,
                minimum: 0,
                maximum: MaxCount::Bound(BoundId::FeeSponsorInputMax),
                authorization: InputAuthorization::SponsorOwner,
            },
        ],
        outputs: &[
            OutputSpec {
                object: ObjectId::State,
                minimum: 1,
                maximum: MaxCount::Exact(1),
            },
            OutputSpec {
                object: ObjectId::Resv,
                minimum: 0,
                maximum: MaxCount::Exact(1),
            },
            OutputSpec {
                object: ObjectId::PlainLbtc,
                minimum: 1,
                maximum: MaxCount::Exact(2),
            },
        ],
        canonical_deltas: &[CanonicalDeltaSpec {
            asset: AssetId::U,
            kind: DeltaKind::Destruction,
            condition: DeltaCondition::Always,
            destruction_tag: Some(TagId::Redeem),
        }],
        data_outputs: &[DataOutputSpec {
            kind: DataOutputKind::Destruction,
            tag: TagId::Redeem,
            asset: Some(AssetId::U),
            minimum: 1,
            maximum: MaxCount::Exact(1),
            condition: DeltaCondition::Always,
        }],
        open_flows: &[OpenFlowKind::Redemption, OpenFlowKind::FeeSponsor],
        reads: &[QuantityId::RedemptionPayout, QuantityId::Floor],
        writes: &[],
        witnesses: &[
            WitnessId::CanonicalDelta,
            WitnessId::StateSuccession,
            WitnessId::ResvSuccession,
            WitnessId::ReceiptOwnerRouting,
            WitnessId::ValueFlowClosure,
            WitnessId::FloorNondecrease,
        ],
        value_flows: &[
            ValueFlowClass::OwnerConsented,
            ValueFlowClass::FormulaBoundPayout,
            ValueFlowClass::OwnerlessTerminalSink,
            ValueFlowClass::SponsorEnvelope,
        ],
        bounds: &[BoundId::FeeSponsorInputMax],
        projections: PROJECTION_TRANSITION,
    },
    OperationSpec {
        id: OperationId::ReceiptRelabel,
        kind: OperationKind::CovenantBranch,
        authorization: PermissionClass::Permissionless,
        roots: ROOTS_STATE_ONLY,
        issuances: NO_ISSUANCE,
        inputs: &[
            InputSpec {
                object: ObjectId::State,
                minimum: 1,
                maximum: MaxCount::Exact(1),
                authorization: InputAuthorization::CovenantCompanion,
            },
            InputSpec {
                object: ObjectId::ReceiptTimeLocked,
                minimum: 1,
                maximum: MaxCount::Bound(BoundId::RelabelBatchMax),
                authorization: InputAuthorization::Permissionless,
            },
            InputSpec {
                object: ObjectId::PlainLbtc,
                minimum: 0,
                maximum: MaxCount::Bound(BoundId::FeeSponsorInputMax),
                authorization: InputAuthorization::SponsorOwner,
            },
        ],
        outputs: &[
            OutputSpec {
                object: ObjectId::State,
                minimum: 1,
                maximum: MaxCount::Exact(1),
            },
            OutputSpec {
                object: ObjectId::ReceiptLive,
                minimum: 1,
                maximum: MaxCount::Bound(BoundId::RelabelBatchMax),
            },
            OutputSpec {
                object: ObjectId::PlainLbtc,
                minimum: 0,
                maximum: MaxCount::Exact(1),
            },
        ],
        canonical_deltas: &[CanonicalDeltaSpec {
            asset: AssetId::U,
            kind: DeltaKind::Lateral,
            condition: DeltaCondition::Always,
            destruction_tag: None,
        }],
        data_outputs: NO_DATA_OUTPUTS,
        open_flows: &[OpenFlowKind::FeeSponsor],
        reads: &[],
        writes: &[],
        witnesses: &[
            WitnessId::CanonicalDelta,
            WitnessId::StateSuccession,
            WitnessId::ReceiptOwnerRouting,
            WitnessId::ReceiptClassClosure,
            WitnessId::ValueFlowClosure,
            WitnessId::UtxoLifecycle,
        ],
        value_flows: &[
            ValueFlowClass::ImmutableDestination,
            ValueFlowClass::SponsorEnvelope,
        ],
        bounds: &[BoundId::RelabelBatchMax, BoundId::FeeSponsorInputMax],
        projections: PROJECTION_TRANSITION,
    },
    OperationSpec {
        id: OperationId::Burn,
        kind: OperationKind::CovenantBranch,
        authorization: PermissionClass::ReceiptOwners,
        roots: ROOTS_NONE,
        issuances: NO_ISSUANCE,
        inputs: &[
            InputSpec {
                object: ObjectId::ReceiptLive,
                minimum: 1,
                maximum: MaxCount::Bound(BoundId::BurnInputMax),
                authorization: InputAuthorization::InputOwner,
            },
            InputSpec {
                object: ObjectId::PlainLbtc,
                minimum: 0,
                maximum: MaxCount::Bound(BoundId::FeeSponsorInputMax),
                authorization: InputAuthorization::SponsorOwner,
            },
        ],
        outputs: &[
            OutputSpec {
                object: ObjectId::Ash,
                minimum: 1,
                maximum: MaxCount::Exact(1),
            },
            OutputSpec {
                object: ObjectId::ReceiptLive,
                minimum: 0,
                maximum: MaxCount::Bound(BoundId::BurnChangeMax),
            },
            OutputSpec {
                object: ObjectId::PlainLbtc,
                minimum: 0,
                maximum: MaxCount::Exact(1),
            },
        ],
        canonical_deltas: &[CanonicalDeltaSpec {
            asset: AssetId::U,
            kind: DeltaKind::Lateral,
            condition: DeltaCondition::Always,
            destruction_tag: None,
        }],
        data_outputs: &[DataOutputSpec {
            kind: DataOutputKind::BurnRecord,
            tag: TagId::Burn,
            asset: None,
            minimum: 0,
            maximum: MaxCount::Bound(BoundId::BurnRecordMax),
            condition: DeltaCondition::Always,
        }],
        open_flows: &[OpenFlowKind::FeeSponsor],
        reads: &[],
        // Burn produces a `BurnEvent` projection; the indexer derives
        // the attestation map. The covenant branch does not write the
        // interface map.
        writes: &[],
        witnesses: &[
            WitnessId::CanonicalDelta,
            WitnessId::AshLineage,
            WitnessId::ReceiptOwnerRouting,
            WitnessId::ReceiptClassClosure,
            WitnessId::AttestationAuthentication,
            WitnessId::ValueFlowClosure,
        ],
        value_flows: &[
            ValueFlowClass::OwnerConsented,
            ValueFlowClass::OwnerlessBoundSink,
            ValueFlowClass::SponsorEnvelope,
        ],
        bounds: &[
            BoundId::BurnInputMax,
            BoundId::BurnChangeMax,
            BoundId::BurnRecordMax,
            BoundId::FeeSponsorInputMax,
        ],
        projections: PROJECTION_BURN,
    },
    OperationSpec {
        id: OperationId::CompactAsh,
        kind: OperationKind::CovenantBranch,
        authorization: PermissionClass::Permissionless,
        roots: ROOTS_NONE,
        issuances: NO_ISSUANCE,
        inputs: &[
            InputSpec {
                object: ObjectId::Ash,
                minimum: 2,
                maximum: MaxCount::Bound(BoundId::AshBatchMax),
                authorization: InputAuthorization::Permissionless,
            },
            InputSpec {
                object: ObjectId::PlainLbtc,
                minimum: 0,
                maximum: MaxCount::Bound(BoundId::FeeSponsorInputMax),
                authorization: InputAuthorization::SponsorOwner,
            },
        ],
        outputs: &[
            OutputSpec {
                object: ObjectId::Ash,
                minimum: 1,
                maximum: MaxCount::Exact(1),
            },
            OutputSpec {
                object: ObjectId::PlainLbtc,
                minimum: 0,
                maximum: MaxCount::Exact(1),
            },
        ],
        canonical_deltas: &[CanonicalDeltaSpec {
            asset: AssetId::U,
            kind: DeltaKind::OwnerlessLateral,
            condition: DeltaCondition::Always,
            destruction_tag: None,
        }],
        data_outputs: NO_DATA_OUTPUTS,
        open_flows: &[OpenFlowKind::FeeSponsor],
        reads: &[],
        writes: &[],
        witnesses: &[
            WitnessId::CanonicalDelta,
            WitnessId::AshLineage,
            WitnessId::ValueFlowClosure,
            WitnessId::UtxoLifecycle,
        ],
        value_flows: &[
            ValueFlowClass::OwnerlessBoundMovement,
            ValueFlowClass::SponsorEnvelope,
        ],
        bounds: &[BoundId::AshBatchMax, BoundId::FeeSponsorInputMax],
        projections: PROJECTION_TRANSITION,
    },
    OperationSpec {
        id: OperationId::Clear,
        kind: OperationKind::CovenantBranch,
        authorization: PermissionClass::Permissionless,
        roots: ROOTS_STATE_ONLY,
        issuances: NO_ISSUANCE,
        inputs: &[
            InputSpec {
                object: ObjectId::State,
                minimum: 1,
                maximum: MaxCount::Exact(1),
                authorization: InputAuthorization::CovenantCompanion,
            },
            InputSpec {
                object: ObjectId::Ash,
                minimum: 1,
                maximum: MaxCount::Bound(BoundId::AshBatchMax),
                authorization: InputAuthorization::Permissionless,
            },
            InputSpec {
                object: ObjectId::PlainLbtc,
                minimum: 0,
                maximum: MaxCount::Bound(BoundId::FeeSponsorInputMax),
                authorization: InputAuthorization::SponsorOwner,
            },
        ],
        outputs: &[
            OutputSpec {
                object: ObjectId::State,
                minimum: 1,
                maximum: MaxCount::Exact(1),
            },
            OutputSpec {
                object: ObjectId::Ash,
                minimum: 0,
                maximum: MaxCount::Exact(1),
            },
            OutputSpec {
                object: ObjectId::PlainLbtc,
                minimum: 0,
                maximum: MaxCount::Exact(1),
            },
        ],
        canonical_deltas: &[
            CanonicalDeltaSpec {
                asset: AssetId::U,
                kind: DeltaKind::Destruction,
                condition: DeltaCondition::Always,
                destruction_tag: Some(TagId::Recon),
            },
            CanonicalDeltaSpec {
                asset: AssetId::U,
                kind: DeltaKind::OwnerlessLateral,
                condition: DeltaCondition::PositiveAshResidual,
                destruction_tag: None,
            },
        ],
        data_outputs: &[DataOutputSpec {
            kind: DataOutputKind::Destruction,
            tag: TagId::Recon,
            asset: Some(AssetId::U),
            minimum: 1,
            maximum: MaxCount::Exact(1),
            condition: DeltaCondition::Always,
        }],
        open_flows: &[OpenFlowKind::FeeSponsor],
        // Clear reads committed state fields and computes its own
        // deterministic decrement; it does not consume a precomputed
        // `Floor` quantity. Its derived `ClearEvent` later supplies
        // (omega, y) to the attestation indexer.
        reads: &[],
        writes: &[],
        witnesses: &[
            WitnessId::CanonicalDelta,
            WitnessId::AshLineage,
            WitnessId::StateSuccession,
            WitnessId::FloorNondecrease,
            WitnessId::ValueFlowClosure,
            WitnessId::NativeFeeAuction,
        ],
        value_flows: &[
            ValueFlowClass::OwnerlessTerminalSink,
            ValueFlowClass::SponsorEnvelope,
        ],
        bounds: &[BoundId::AshBatchMax, BoundId::FeeSponsorInputMax],
        projections: PROJECTION_CLEAR,
    },
    OperationSpec {
        id: OperationId::AnnounceMaturity,
        kind: OperationKind::CovenantBranch,
        authorization: PermissionClass::Operator,
        roots: ROOTS_STATE_ONLY,
        issuances: NO_ISSUANCE,
        inputs: &[
            InputSpec {
                object: ObjectId::State,
                minimum: 1,
                maximum: MaxCount::Exact(1),
                authorization: InputAuthorization::CovenantCompanion,
            },
            InputSpec {
                object: ObjectId::PlainLbtc,
                minimum: 0,
                maximum: MaxCount::Bound(BoundId::FeeSponsorInputMax),
                authorization: InputAuthorization::SponsorOwner,
            },
        ],
        outputs: &[
            OutputSpec {
                object: ObjectId::State,
                minimum: 1,
                maximum: MaxCount::Exact(1),
            },
            OutputSpec {
                object: ObjectId::PlainLbtc,
                minimum: 0,
                maximum: MaxCount::Exact(1),
            },
        ],
        canonical_deltas: NO_DELTAS,
        data_outputs: NO_DATA_OUTPUTS,
        open_flows: &[OpenFlowKind::FeeSponsor],
        reads: &[],
        writes: &[],
        witnesses: &[
            WitnessId::StateSuccession,
            WitnessId::NativeFeeAuction,
            WitnessId::ValueFlowClosure,
        ],
        value_flows: &[
            ValueFlowClass::OwnerConsented,
            ValueFlowClass::SponsorEnvelope,
        ],
        bounds: &[BoundId::FeeSponsorInputMax],
        projections: PROJECTION_TRANSITION,
    },
];

// -------------------------------------------------------------------------
// Quantities
// -------------------------------------------------------------------------

pub const QUANTITIES: &[QuantitySpec] = &[
    QuantitySpec {
        id: QuantityId::Floor,
        kind: QuantityKind::Monetary,
        reads: &[
            DataId::StateOmega,
            DataId::StateYLive,
            DataId::StateYTimeLocked,
        ],
        readers: &[
            ReaderId::Operation(OperationId::Cycle),
            ReaderId::Operation(OperationId::Redeem),
            ReaderId::ExternalAuditor,
        ],
        writers: &[],
    },
    QuantitySpec {
        id: QuantityId::RedemptionPayout,
        kind: QuantityKind::Monetary,
        reads: &[
            DataId::StateOmega,
            DataId::StateYLive,
            DataId::StateYTimeLocked,
        ],
        readers: &[ReaderId::Operation(OperationId::Redeem)],
        writers: &[],
    },
    QuantitySpec {
        id: QuantityId::CycleIssuance,
        kind: QuantityKind::Monetary,
        reads: &[
            DataId::StateQ,
            DataId::StateYLive,
            DataId::StateYTimeLocked,
            DataId::StateOmega,
        ],
        readers: &[ReaderId::Operation(OperationId::Cycle)],
        writers: &[],
    },
    QuantitySpec {
        id: QuantityId::AttestationDelta,
        kind: QuantityKind::Derived,
        reads: &[DataId::ClearOmega, DataId::ClearY, DataId::BurnRecordAmount],
        readers: &[ReaderId::AttestationIndexer],
        writers: &[],
    },
    QuantitySpec {
        id: QuantityId::AttestationMap,
        kind: QuantityKind::Interface,
        reads: &[DataId::AttestationTerms],
        readers: &[
            ReaderId::AttestationIndexer,
            ReaderId::ExternalAuditor,
            ReaderId::ConsumerFormula,
        ],
        writers: &[],
    },
    QuantitySpec {
        id: QuantityId::HistoricalLiveResidue,
        kind: QuantityKind::AuditOnly,
        reads: &[DataId::HistoricalLiveResidue],
        readers: &[ReaderId::InvariantChecker, ReaderId::ExternalAuditor],
        writers: &[],
    },
    QuantitySpec {
        id: QuantityId::HistoricalTimeLockedResidue,
        kind: QuantityKind::AuditOnly,
        reads: &[DataId::HistoricalTimeLockedResidue],
        readers: &[ReaderId::InvariantChecker, ReaderId::ExternalAuditor],
        writers: &[],
    },
    QuantitySpec {
        id: QuantityId::ReceiptAccountingAudit,
        kind: QuantityKind::AuditOnly,
        reads: &[
            DataId::StateYLive,
            DataId::StateYTimeLocked,
            DataId::HistoricalLiveResidue,
            DataId::HistoricalTimeLockedResidue,
        ],
        readers: &[ReaderId::InvariantChecker, ReaderId::ExternalAuditor],
        writers: &[],
    },
];

// -------------------------------------------------------------------------
// Witnesses
// -------------------------------------------------------------------------

// Each `semantic_tag` is the realization document's frozen label for
// the fact the witness discharges — the register-1 → document weld,
// CI-checked by the doc-conformance test. Many-to-one is expected and
// correct: witnesses and failure reasons are finer than clauses.
//
// Witnesses may cite any frozen document label, not only goal labels:
// `attestation-authentication` cites the ledger's dual-anchor home,
// and `native-fee-auction` cites the residual whose model content is
// exactly "every valid contender is safe".
//
// Clauses with no witness row (𝗜₂ 𝗜₃ 𝗜₄ 𝗜₅ 𝗜₉) are checked directly
// by state predicates rather than transition witnesses — their absence
// here is structural, not an omission.
pub const WITNESSES: &[WitnessSpec] = &[
    WitnessSpec {
        id: WitnessId::CanonicalDelta,
        semantic_tag: "lem:invariant:delta",
    },
    WitnessSpec {
        id: WitnessId::AshLineage,
        semantic_tag: "inv:invariant:accounting",
    },
    WitnessSpec {
        id: WitnessId::StateSuccession,
        semantic_tag: "inv:invariant:succession",
    },
    WitnessSpec {
        id: WitnessId::ResvSuccession,
        semantic_tag: "inv:invariant:succession",
    },
    WitnessSpec {
        id: WitnessId::ReceiptOwnerRouting,
        semantic_tag: "lem:invariant:flow",
    },
    WitnessSpec {
        id: WitnessId::ValueFlowClosure,
        semantic_tag: "lem:invariant:flow",
    },
    WitnessSpec {
        id: WitnessId::RequestClosure,
        semantic_tag: "lem:invariant:flow",
    },
    WitnessSpec {
        id: WitnessId::EntitlementClosure,
        semantic_tag: "inv:invariant:escrow-receipts",
    },
    WitnessSpec {
        id: WitnessId::DistributionClosure,
        semantic_tag: "inv:invariant:no-starve",
    },
    WitnessSpec {
        id: WitnessId::ReceiptClassClosure,
        semantic_tag: "inv:invariant:accounting",
    },
    WitnessSpec {
        id: WitnessId::AttestationAuthentication,
        semantic_tag: "sec:ledger:authentication",
    },
    WitnessSpec {
        id: WitnessId::NativeFeeAuction,
        semantic_tag: "res:trust:op",
    },
    WitnessSpec {
        id: WitnessId::UtxoLifecycle,
        semantic_tag: "inv:invariant:succession",
    },
    WitnessSpec {
        id: WitnessId::FloorNondecrease,
        semantic_tag: "lem:invariant:rate",
    },
];

// -------------------------------------------------------------------------
// Dependencies
// -------------------------------------------------------------------------

pub const DEPENDENCIES: &[DependencySpec] = &[
    DependencySpec {
        id: DependencyId::NativeAssetConservation,
        verification_required: true,
        rationale: "Closed-asset scarcity and no-forgery depend on Elements native-asset conservation.",
    },
    DependencySpec {
        id: DependencyId::IssuanceIntrospection,
        verification_required: true,
        rationale: "Receipt, entitlement, and control issuance are pinned through issuance introspection.",
    },
    DependencySpec {
        id: DependencyId::ExplicitValueIntrospection,
        verification_required: true,
        rationale: "Explicit-arithmetic and public-delta seams read explicit consensus values.",
    },
    DependencySpec {
        id: DependencyId::SighashProfile,
        verification_required: true,
        rationale: "Owner signatures must commit all outputs; ANYONECANPAY remains a separate input-set policy.",
    },
    DependencySpec {
        id: DependencyId::PackageRelay,
        verification_required: true,
        rationale: "The maturity-cycle CPFP anchor depends on package selection.",
    },
    DependencySpec {
        id: DependencyId::UnspendableUtxoExclusion,
        verification_required: true,
        rationale: "Tagged destruction outputs must not remain in the current UTXO set.",
    },
    DependencySpec {
        id: DependencyId::WeldEnforcement,
        verification_required: true,
        rationale: "Emitted script must enforce STATE/RESV and control/vault welds represented by the model.",
    },
    DependencySpec {
        id: DependencyId::LbtcSettlement,
        verification_required: true,
        rationale: "L-BTC settlement is provided by the Liquid functionary consortium.",
    },
    DependencySpec {
        id: DependencyId::ScriptEmissionFidelity,
        verification_required: true,
        rationale: "The compiler must emit script matching the typed transition model.",
    },
    DependencySpec {
        id: DependencyId::ConfidentialValueConservation,
        verification_required: true,
        rationale: "Blind lateral movements discharge conservation through consensus CT balancing.",
    },
    DependencySpec {
        id: DependencyId::ValueCommitmentEquality,
        verification_required: true,
        rationale: "Amount-blind relabel discharges per-object preservation by commitment equality.",
    },
    DependencySpec {
        id: DependencyId::ValueCommitmentOpening,
        verification_required: false,
        rationale: "Reveal-at-spend paths, if implemented, authenticate openings on-chain.",
    },
];

// -------------------------------------------------------------------------
// Closed decisions
// -------------------------------------------------------------------------

pub const DECISIONS: &[DecisionSpec] = &[
    DecisionSpec {
        id: DecisionId::NoOnchainAttestationAccumulator,
        status: DecisionStatus::Closed,
        rationale: &[
            "Full-chain verifiers recompute attestation from public history.",
            "A root would not make the unbounded fold self-verifying.",
            "A shared root would serialize the burn hot path.",
        ],
    },
    DecisionSpec {
        id: DecisionId::FullChainAttestationVerification,
        status: DecisionStatus::Closed,
        rationale: &[
            "Verifier parties maintain canonical history.",
            "Ordinary users may trust their Layer-1 system.",
        ],
    },
    DecisionSpec {
        id: DecisionId::HistoricalResidueAuditOnly,
        status: DecisionStatus::Closed,
        rationale: &[
            "Historical residue explains conservative recorded supply.",
            "No monetary, interface, covenant, or consumer formula reads residue.",
        ],
    },
    DecisionSpec {
        id: DecisionId::OneEntitlementPerRequest,
        status: DecisionStatus::Closed,
        rationale: &[
            "Integer floor allocation is not additive across merged requests.",
            "Admission must not possess payout-affecting aggregation discretion.",
        ],
    },
    DecisionSpec {
        id: DecisionId::SettlementPermissionless,
        status: DecisionStatus::Closed,
        rationale: &[
            "Lost owner keys must not retain shared distribution state.",
            "Settlement outputs are owner-preserving and formula-bound.",
        ],
    },
    DecisionSpec {
        id: DecisionId::AtomicMaturityCycle,
        status: DecisionStatus::Closed,
        rationale: &[
            "No announced-but-unconverted limbo state is permitted.",
            "Maturity conversion occurs in the cycle reaching the maturity cycle index.",
        ],
    },
    DecisionSpec {
        id: DecisionId::ActiveBackingCap,
        status: DecisionStatus::Closed,
        rationale: &[
            "The pool never holds more active L-BTC backing plus admitted escrow than the maximum L-BTC supply.",
            "The cap is enforced at genesis, admission, cycle, and invariant validation.",
            "The cap does not limit cumulative historical deposit volume.",
        ],
    },
];

// -------------------------------------------------------------------------
// Finite bounds
// -------------------------------------------------------------------------

pub const BOUNDS: &[BoundSpec] = &[
    BoundSpec {
        id: BoundId::AdmissionBatchMax,
        default_value: Some(32),
        requires_deployment_calibration: true,
    },
    BoundSpec {
        id: BoundId::SettlementBatchMax,
        default_value: Some(32),
        requires_deployment_calibration: true,
    },
    BoundSpec {
        id: BoundId::RelabelBatchMax,
        default_value: Some(32),
        requires_deployment_calibration: true,
    },
    BoundSpec {
        id: BoundId::AshBatchMax,
        default_value: Some(64),
        requires_deployment_calibration: true,
    },
    BoundSpec {
        id: BoundId::BurnInputMax,
        default_value: Some(64),
        requires_deployment_calibration: true,
    },
    BoundSpec {
        id: BoundId::BurnChangeMax,
        default_value: Some(32),
        requires_deployment_calibration: true,
    },
    BoundSpec {
        id: BoundId::BurnRecordMax,
        default_value: Some(64),
        requires_deployment_calibration: true,
    },
    BoundSpec {
        id: BoundId::TransferInputMax,
        default_value: Some(64),
        requires_deployment_calibration: true,
    },
    BoundSpec {
        id: BoundId::TransferOutputMax,
        default_value: Some(64),
        requires_deployment_calibration: true,
    },
    BoundSpec {
        id: BoundId::FeeSponsorInputMax,
        default_value: Some(16),
        requires_deployment_calibration: true,
    },
];

// -------------------------------------------------------------------------
// Tags
// -------------------------------------------------------------------------

pub const TAGS: &[TagSpec] = &[
    TagSpec {
        id: TagId::Burn,
        participates_in_attestation: true,
    },
    TagSpec {
        id: TagId::Recon,
        participates_in_attestation: false,
    },
    TagSpec {
        id: TagId::Redeem,
        participates_in_attestation: false,
    },
    TagSpec {
        id: TagId::Entitlement,
        participates_in_attestation: false,
    },
    TagSpec {
        id: TagId::DistributionControlClose,
        participates_in_attestation: false,
    },
    TagSpec {
        id: TagId::DistributionResidue,
        participates_in_attestation: false,
    },
];

// -------------------------------------------------------------------------
// Objects
// -------------------------------------------------------------------------

pub const OBJECTS: &[ObjectSpec] = &[
    ObjectSpec {
        id: ObjectId::State,
        asset: AssetId::Pid,
        lifecycle: LifecycleClass::ConstantRoot,
        accounting_domain: AccountingDomain::None,
        allocators: &[AllocatorId::Genesis],
        mutators: &[
            OperationId::AdmitDeposits,
            OperationId::Cycle,
            OperationId::Redeem,
            OperationId::ReceiptRelabel,
            OperationId::Clear,
            OperationId::AnnounceMaturity,
        ],
        deallocators: &[DeallocatorId::None],
        witnesses: &[WitnessId::StateSuccession],
        consensus_value_authoritative: true,
    },
    ObjectSpec {
        id: ObjectId::Resv,
        asset: AssetId::Lbtc,
        lifecycle: LifecycleClass::ConstantRoot,
        accounting_domain: AccountingDomain::Backing,
        allocators: &[AllocatorId::Genesis],
        mutators: &[
            OperationId::AdmitDeposits,
            OperationId::Cycle,
            OperationId::Redeem,
        ],
        deallocators: &[DeallocatorId::Operation(OperationId::Redeem)],
        witnesses: &[WitnessId::ResvSuccession],
        consensus_value_authoritative: true,
    },
    ObjectSpec {
        id: ObjectId::Pace,
        asset: AssetId::Pace,
        lifecycle: LifecycleClass::ConstantRoot,
        accounting_domain: AccountingDomain::None,
        allocators: &[AllocatorId::Genesis],
        mutators: &[OperationId::Cycle],
        deallocators: &[DeallocatorId::None],
        witnesses: &[WitnessId::CanonicalDelta, WitnessId::NativeFeeAuction],
        consensus_value_authoritative: true,
    },
    ObjectSpec {
        id: ObjectId::EntitlementAuthority,
        asset: AssetId::EntAuth,
        lifecycle: LifecycleClass::ConstantRoot,
        accounting_domain: AccountingDomain::None,
        allocators: &[AllocatorId::Genesis],
        mutators: &[OperationId::AdmitDeposits],
        deallocators: &[DeallocatorId::None],
        witnesses: &[WitnessId::CanonicalDelta],
        consensus_value_authoritative: true,
    },
    ObjectSpec {
        id: ObjectId::DistributionAuthority,
        asset: AssetId::DistAuth,
        lifecycle: LifecycleClass::ConstantRoot,
        accounting_domain: AccountingDomain::None,
        allocators: &[AllocatorId::Genesis],
        mutators: &[OperationId::Cycle],
        deallocators: &[DeallocatorId::None],
        witnesses: &[WitnessId::CanonicalDelta],
        consensus_value_authoritative: true,
    },
    ObjectSpec {
        id: ObjectId::ReceiptLive,
        asset: AssetId::U,
        lifecycle: LifecycleClass::UserPosition,
        accounting_domain: AccountingDomain::Receipt,
        allocators: &[
            AllocatorId::Genesis,
            AllocatorId::Operation(OperationId::Cycle),
            AllocatorId::Operation(OperationId::SettleDistribution),
            AllocatorId::Operation(OperationId::TransferLive),
            AllocatorId::Operation(OperationId::ReceiptRelabel),
        ],
        mutators: &[
            OperationId::TransferLive,
            OperationId::Redeem,
            OperationId::Burn,
        ],
        deallocators: &[
            DeallocatorId::Operation(OperationId::Redeem),
            DeallocatorId::Operation(OperationId::Burn),
        ],
        witnesses: &[
            WitnessId::CanonicalDelta,
            WitnessId::ReceiptOwnerRouting,
            WitnessId::ReceiptClassClosure,
        ],
        consensus_value_authoritative: true,
    },
    ObjectSpec {
        id: ObjectId::ReceiptTimeLocked,
        asset: AssetId::U,
        lifecycle: LifecycleClass::UserPosition,
        accounting_domain: AccountingDomain::Receipt,
        allocators: &[
            AllocatorId::Genesis,
            AllocatorId::Operation(OperationId::Cycle),
            AllocatorId::Operation(OperationId::SettleDistribution),
            AllocatorId::Operation(OperationId::TransferTimeLocked),
        ],
        mutators: &[OperationId::TransferTimeLocked, OperationId::ReceiptRelabel],
        deallocators: &[DeallocatorId::Operation(OperationId::ReceiptRelabel)],
        witnesses: &[
            WitnessId::CanonicalDelta,
            WitnessId::ReceiptOwnerRouting,
            WitnessId::ReceiptClassClosure,
        ],
        consensus_value_authoritative: true,
    },
    ObjectSpec {
        id: ObjectId::DepositRequest,
        asset: AssetId::Lbtc,
        lifecycle: LifecycleClass::OpenOffer,
        accounting_domain: AccountingDomain::None,
        allocators: &[
            AllocatorId::External,
            AllocatorId::Operation(OperationId::CreateRequest),
        ],
        mutators: &[],
        deallocators: &[
            DeallocatorId::Operation(OperationId::CancelRequest),
            DeallocatorId::Operation(OperationId::AdmitDeposits),
        ],
        witnesses: &[WitnessId::RequestClosure, WitnessId::ValueFlowClosure],
        consensus_value_authoritative: true,
    },
    ObjectSpec {
        id: ObjectId::DepositEntitlement,
        asset: AssetId::Ent,
        lifecycle: LifecycleClass::CycleQueue,
        accounting_domain: AccountingDomain::Entitlement,
        allocators: &[AllocatorId::Operation(OperationId::AdmitDeposits)],
        mutators: &[],
        deallocators: &[DeallocatorId::Operation(OperationId::SettleDistribution)],
        witnesses: &[
            WitnessId::EntitlementClosure,
            WitnessId::DistributionClosure,
        ],
        consensus_value_authoritative: true,
    },
    ObjectSpec {
        id: ObjectId::DistributionControl,
        asset: AssetId::DistCtl,
        lifecycle: LifecycleClass::CycleQueue,
        accounting_domain: AccountingDomain::Entitlement,
        allocators: &[AllocatorId::Operation(OperationId::Cycle)],
        mutators: &[OperationId::SettleDistribution],
        deallocators: &[DeallocatorId::Operation(OperationId::SettleDistribution)],
        witnesses: &[WitnessId::DistributionClosure, WitnessId::UtxoLifecycle],
        consensus_value_authoritative: true,
    },
    ObjectSpec {
        id: ObjectId::DistributionVault,
        asset: AssetId::U,
        lifecycle: LifecycleClass::CycleQueue,
        accounting_domain: AccountingDomain::Receipt,
        allocators: &[AllocatorId::Operation(OperationId::Cycle)],
        mutators: &[OperationId::SettleDistribution],
        deallocators: &[DeallocatorId::Operation(OperationId::SettleDistribution)],
        witnesses: &[WitnessId::CanonicalDelta, WitnessId::DistributionClosure],
        consensus_value_authoritative: true,
    },
    ObjectSpec {
        id: ObjectId::Ash,
        asset: AssetId::U,
        lifecycle: LifecycleClass::Backlog,
        accounting_domain: AccountingDomain::Receipt,
        allocators: &[
            AllocatorId::Operation(OperationId::Burn),
            AllocatorId::Operation(OperationId::CompactAsh),
            AllocatorId::Operation(OperationId::Clear),
        ],
        mutators: &[OperationId::CompactAsh, OperationId::Clear],
        deallocators: &[DeallocatorId::Operation(OperationId::Clear)],
        witnesses: &[
            WitnessId::CanonicalDelta,
            WitnessId::AshLineage,
            WitnessId::UtxoLifecycle,
        ],
        consensus_value_authoritative: true,
    },
    ObjectSpec {
        id: ObjectId::PlainLbtc,
        asset: AssetId::Lbtc,
        lifecycle: LifecycleClass::ExternalWallet,
        accounting_domain: AccountingDomain::Fee,
        allocators: &[AllocatorId::External],
        mutators: &[],
        deallocators: &[DeallocatorId::ExternalSpend],
        witnesses: &[WitnessId::ValueFlowClosure],
        consensus_value_authoritative: true,
    },
    ObjectSpec {
        id: ObjectId::CpfpAnchor,
        asset: AssetId::Lbtc,
        lifecycle: LifecycleClass::BoundedRetained,
        accounting_domain: AccountingDomain::None,
        allocators: &[AllocatorId::Operation(OperationId::Cycle)],
        mutators: &[],
        deallocators: &[DeallocatorId::ExternalSpend],
        witnesses: &[WitnessId::NativeFeeAuction],
        consensus_value_authoritative: true,
    },
];

// -------------------------------------------------------------------------
// Amount limits
// -------------------------------------------------------------------------

pub const AMOUNT_LIMITS: &[AmountLimitSpec] = &[AmountLimitSpec {
    id: AmountLimitId::ActiveBackingMax,
    asset: AssetId::Lbtc,
    value: 2_100_000_000_000_000,
    unit: "lbtc-atomic-unit",
    rationale: "Maximum simultaneous settled reserve plus admitted deposit escrow.",
}];

// -------------------------------------------------------------------------
// Invariant clauses
// -------------------------------------------------------------------------

/// The realization document's invariant-clause registry.
///
/// Exporting it makes the document's clause table generated rather
/// than re-typed; the executable model's `clause_of` mapping welds
/// failure reasons onto these clauses.
pub const CLAUSES: &[InvariantClauseId] = InvariantClauseId::ALL;

// -------------------------------------------------------------------------
// Complete architecture
// -------------------------------------------------------------------------

pub const ARCHITECTURE: Architecture = Architecture {
    document: DOCUMENT,
    assets: ASSETS,
    roots: ROOTS,
    objects: OBJECTS,
    operations: OPERATIONS,
    quantities: QUANTITIES,
    witnesses: WITNESSES,
    clauses: CLAUSES,
    dependencies: DEPENDENCIES,
    decisions: DECISIONS,
    bounds: BOUNDS,
    amount_limits: AMOUNT_LIMITS,
    tags: TAGS,
};
