//! Symbolic obligations for a later validated maturity-announcement plan.
//!
//! These projections state required work; constructing one proves no transition,
//! constructor, target acceptance, history, or recovery. Semantic execution stays
//! in `realization::announce_maturity`. Target material has no field here.

use architecture::{OperationId, RootId};
use realization::{
    AnnouncementLeadBound, FactId, RelationId, RelationKind, RelationSubject, StateField,
    TransactionSide,
};

use crate::capability::census_enum;

const OPERATION: OperationId = OperationId::AnnounceMaturity;

census_enum! {
    /// Public information needed to recover the successor, named by role.
    pub enum AnnouncementRecoveryInputRole {
        /// Consumed STATE semantic metadata.
        PredecessorMetadata,
        /// Public request for the maturity cycle.
        RequestedCycle,
        /// Semantic successor obtained through the realization's typed transition.
        DerivedSuccessorMetadata,
        /// Representation nonce needed by successor reconstruction.
        SuccessorNonce,
        /// Schema needed to decode and encode semantic metadata.
        MetadataSchema,
        /// Static constructor recipe or exact linked static-root reference.
        StaticConstructorRecipeOrReference,
        /// Position of the actual STATE successor, assigned below the compiler.
        SuccessorOutputPosition,
        /// Target leaf-version policy, without its concrete version.
        TargetLeafVersion,
        /// Target internal-key policy, without key material.
        TargetInternalKeyPolicy,
    }
}

census_enum! {
    /// Ordered duties of an unrelated public recovery process.
    pub enum AnnouncementRecoveryStep {
        /// Locate the accepted announcement transaction.
        LocateAcceptedTransaction,
        /// Verify exact transaction bytes and deployment binding.
        VerifyBytesAndDeploymentBinding,
        /// Decode the public script-path witness.
        DecodePublicWitness,
        /// Invoke the typed transition to obtain successor semantic metadata.
        DeriveSuccessorMetadata,
        /// Reconstruct canonical successor constructor bytes from public inputs.
        ReconstructSuccessorConstructor,
        /// Compare the reconstructed program with the actual STATE output.
        CompareActualStateOutput,
    }
}

census_enum! {
    /// Effect of the typed transition on a semantic field.
    pub enum AnnouncementFieldEffect {
        /// Copy the corresponding predecessor field unchanged.
        PreservePredecessor,
        /// Set maturity to announced at the requested cycle.
        AnnounceRequestedCycle,
    }
}

census_enum! {
    /// Required predecessor maturity, without a cycle value.
    pub enum AnnouncementPredecessorMaturity {
        /// Already-announced and complete predecessors are refused.
        Unannounced,
    }
}

/// One entry in the exhaustive semantic field map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnnouncementFieldRequirement {
    /// Realization-owned semantic field key.
    pub field: StateField,
    /// Public predecessor fact read for this field.
    pub predecessor: FactId,
    /// Symbolic effect; the realization executes it.
    pub effect: AnnouncementFieldEffect,
}

impl AnnouncementFieldRequirement {
    /// Map every realization field without an extensible fallback.
    #[must_use]
    pub const fn for_field(field: StateField) -> Self {
        let effect = match field {
            StateField::Omega
            | StateField::YL
            | StateField::YT
            | StateField::Q
            | StateField::Cycle => AnnouncementFieldEffect::PreservePredecessor,
            StateField::Maturity => AnnouncementFieldEffect::AnnounceRequestedCycle,
        };
        Self {
            field,
            predecessor: FactId::StateField {
                operation: OPERATION,
                side: TransactionSide::Input,
                field,
            },
            effect,
        }
    }
}

/// Semantic inputs and effects required by the announcement transition.
///
/// Lead bounds are fact keys only. Their values and policy ownership are not
/// supplied by compiler input. The realization validates bounds and uses checked
/// arithmetic to enforce the inclusive window; this projection performs no law.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnnouncementMetadataRequirement {
    /// Exactly the six semantic fields, in realization declaration order.
    pub fields: [AnnouncementFieldRequirement; 6],
    /// Required status of the predecessor maturity field.
    pub predecessor_maturity: AnnouncementPredecessorMaturity,
    /// Public requested announcement cycle key.
    pub requested_cycle: FactId,
    /// Symbolic minimum lead key, with no value or owner binding.
    pub minimum_lead: FactId,
    /// Symbolic maximum lead key, with no value or owner binding.
    pub maximum_lead: FactId,
}

impl AnnouncementMetadataRequirement {
    /// Complete symbolic metadata requirement for maturity announcement.
    pub const REQUIRED: Self = Self {
        fields: [
            AnnouncementFieldRequirement::for_field(StateField::Omega),
            AnnouncementFieldRequirement::for_field(StateField::YL),
            AnnouncementFieldRequirement::for_field(StateField::YT),
            AnnouncementFieldRequirement::for_field(StateField::Q),
            AnnouncementFieldRequirement::for_field(StateField::Cycle),
            AnnouncementFieldRequirement::for_field(StateField::Maturity),
        ],
        predecessor_maturity: AnnouncementPredecessorMaturity::Unannounced,
        requested_cycle: FactId::RequestedAnnouncementCycle {
            operation: OPERATION,
        },
        minimum_lead: FactId::AnnouncementLead {
            operation: OPERATION,
            bound: AnnouncementLeadBound::Minimum,
        },
        maximum_lead: FactId::AnnouncementLead {
            operation: OPERATION,
            bound: AnnouncementLeadBound::Maximum,
        },
    };

    /// Exact public fact census: six fields on each side, request, and two leads.
    #[must_use]
    pub fn public_facts() -> [FactId; 15] {
        let requirement = Self::REQUIRED;
        let [omega, y_l, y_t, q, cycle, maturity] = &requirement.fields;
        [
            omega.predecessor.clone(),
            y_l.predecessor.clone(),
            y_t.predecessor.clone(),
            q.predecessor.clone(),
            cycle.predecessor.clone(),
            maturity.predecessor.clone(),
            FactId::StateField {
                operation: OPERATION,
                side: TransactionSide::Output,
                field: StateField::Omega,
            },
            FactId::StateField {
                operation: OPERATION,
                side: TransactionSide::Output,
                field: StateField::YL,
            },
            FactId::StateField {
                operation: OPERATION,
                side: TransactionSide::Output,
                field: StateField::YT,
            },
            FactId::StateField {
                operation: OPERATION,
                side: TransactionSide::Output,
                field: StateField::Q,
            },
            FactId::StateField {
                operation: OPERATION,
                side: TransactionSide::Output,
                field: StateField::Cycle,
            },
            FactId::StateField {
                operation: OPERATION,
                side: TransactionSide::Output,
                field: StateField::Maturity,
            },
            requirement.requested_cycle,
            requirement.minimum_lead,
            requirement.maximum_lead,
        ]
    }
}

census_enum! {
    /// STATE endpoint role, without an outpoint or transaction position.
    pub enum AnnouncementStateRole {
        /// The consumed STATE family member.
        InputState,
        /// The created STATE family member.
        OutputState,
    }
}

/// STATE edge and the declaration policies that constrain it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateSuccessionRequirement {
    /// Architecture-owned root family.
    pub root: RootId,
    /// Consumed STATE role.
    pub predecessor: AnnouncementStateRole,
    /// Created STATE role.
    pub successor: AnnouncementStateRole,
    /// Declaration relation requiring STATE succession and other-root absence.
    pub root_relation: RelationId,
    /// Declaration relation requiring the transition certificate.
    pub certificate_relation: RelationId,
}

impl StateSuccessionRequirement {
    /// Announcement succession; endpoint linkage remains a history obligation.
    pub const REQUIRED: Self = Self {
        root: RootId::State,
        predecessor: AnnouncementStateRole::InputState,
        successor: AnnouncementStateRole::OutputState,
        root_relation: policy_relation(RelationKind::RootPolicy),
        certificate_relation: policy_relation(RelationKind::ProjectionPolicy),
    };
}

const fn policy_relation(kind: RelationKind) -> RelationId {
    RelationId::new(OPERATION, kind, RelationSubject::Operation)
}

census_enum! {
    /// Constructor endpoint work, each using its own metadata and nonce.
    pub enum AnnouncementConstructorRole {
        /// Authenticate the consumed program against the predecessor constructor.
        AuthenticatePredecessor,
        /// Reconstruct the created program from the derived successor metadata.
        ReconstructSuccessor,
    }
}

census_enum! {
    /// Static code continuity independent of metadata continuity.
    pub enum AnnouncementStaticContinuity {
        /// Both constructors use the same exact linked static subtree; no migration.
        SameLinkedStaticSubtree,
    }
}

census_enum! {
    /// Target policy continuity independent of static code continuity.
    pub enum AnnouncementPolicyContinuity {
        /// Both constructors use the same leaf-version and internal-key policy.
        SameLeafVersionAndInternalKeyPolicy,
    }
}

/// Bidirectional constructor authentication and reconstruction duties.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstructorContinuityRequirement {
    /// Authenticate consumed constructor using predecessor metadata and nonce.
    pub predecessor: AnnouncementConstructorRole,
    /// Reconstruct successor constructor using derived metadata and successor nonce.
    pub successor: AnnouncementConstructorRole,
    /// Exact static-subtree continuity, not inferred from metadata agreement.
    pub static_continuity: AnnouncementStaticContinuity,
    /// Shared target policy, not merely equality of internal keys.
    pub policy_continuity: AnnouncementPolicyContinuity,
}

impl ConstructorContinuityRequirement {
    /// Both endpoints and their independent continuity duties.
    pub const REQUIRED: Self = Self {
        predecessor: AnnouncementConstructorRole::AuthenticatePredecessor,
        successor: AnnouncementConstructorRole::ReconstructSuccessor,
        static_continuity: AnnouncementStaticContinuity::SameLinkedStaticSubtree,
        policy_continuity: AnnouncementPolicyContinuity::SameLeafVersionAndInternalKeyPolicy,
    };
}

census_enum! {
    /// History checks beyond the declaration's root-effect and projection censuses.
    pub enum AnnouncementRootHistoryCheck {
        /// Exactly one STATE edge with unique predecessor and successor, neither missing.
        ExactlyOneStateEdge,
        /// Endpoints match the transaction and its maturity-announcement certificate.
        EndpointAndCertificateAgreement,
        /// The predecessor equals the current cursor before the edge.
        CurrentPredecessor,
        /// Validate every edge before advancing; a restored final cursor proves nothing.
        EveryIntermediateEdgeValid,
        /// Stale predecessor histories cannot establish succession.
        RejectStaleHistory,
        /// STATE termination cannot establish announcement succession.
        RejectTerminatedHistory,
        /// Other root families cannot participate in announcement history.
        RejectForeignRootHistory,
        /// Bind chain checkpoint, transaction, endpoints, target, candidate and ABI;
        /// reorg removal or change stales the observation.
        CheckpointAndReorgBinding,
        /// Synthetic origins do not establish native canonicality.
        NoSyntheticOriginCanonicalityClaim,
    }
}

/// Outstanding root-history report duties, never a validated history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RootHistoryRequirement {
    /// Declaration-bound STATE succession requirement.
    pub succession: StateSuccessionRequirement,
    /// Complete required history check census.
    pub checks: &'static [AnnouncementRootHistoryCheck],
}

impl RootHistoryRequirement {
    /// Full announcement history requirement.
    pub const REQUIRED: Self = Self {
        succession: StateSuccessionRequirement::REQUIRED,
        checks: AnnouncementRootHistoryCheck::ALL,
    };
}

census_enum! {
    /// Canonical public source available to an unrelated recovery process.
    pub enum AnnouncementPublicationRole {
        /// Accepted transaction's public script-path witness and output set.
        AcceptedTransactionWitnessAndOutputs,
    }
}

/// Public reconstruction inputs and ordered duties, with no creator-private state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicRecoveryRequirement {
    /// Accepted transaction publication role.
    pub publication: AnnouncementPublicationRole,
    /// Complete public recovery input roles.
    pub inputs: &'static [AnnouncementRecoveryInputRole],
    /// Ordered reconstruction and comparison duties.
    pub steps: &'static [AnnouncementRecoveryStep],
}

impl PublicRecoveryRequirement {
    /// Full public recovery requirement, without fixing a target output position.
    pub const REQUIRED: Self = Self {
        publication: AnnouncementPublicationRole::AcceptedTransactionWitnessAndOutputs,
        inputs: AnnouncementRecoveryInputRole::ALL,
        steps: AnnouncementRecoveryStep::ALL,
    };
}

census_enum! {
    /// Disjoint duty groups; assignment does not discharge a requirement.
    pub enum AnnouncementRequirementBoundary {
        /// Announcement leaf and coordinator runtime duties.
        Runtime,
        /// Metadata commitment, static continuity, and target constructor policy.
        LinkedConstructor,
        /// Acceptance semantics supplied as target evidence.
        TargetEvidence,
        /// Freshness and edge-sequence checks supplied by a history report.
        RootHistoryReport,
        /// Unrelated-process reconstruction supplied by a recovery report.
        PublicRecoveryReport,
    }
}

census_enum! {
    /// Duties classified by their enforcement or reporting boundary.
    pub enum AnnouncementDuty {
        /// Runtime authenticates the predecessor metadata.
        PredecessorMetadataAuthentication,
        /// Runtime requires unannounced predecessor maturity.
        PredecessorMaturity,
        /// Runtime enforces the checked inclusive lead window.
        AnnouncementLeadWindow,
        /// Runtime derives successor semantic metadata.
        SuccessorMetadataDerivation,
        /// Runtime reconstructs the successor constructor.
        SuccessorConstructorReconstruction,
        /// Runtime checks operator authorization over finalized bytes.
        OperatorAuthorization,
        /// Coordinator enforces exact counts and STATE input/output closure.
        TransactionStructure,
        /// Coordinator enforces root and specialized-event absence.
        RootAndProjectionClosure,
        /// Coordinator enforces the sponsor boundary.
        SponsorIsolation,
        /// Linked constructor commits the metadata leaf.
        MetadataLeafCommitment,
        /// Linked constructor preserves the static subtree.
        StaticSubtreeContinuity,
        /// Linked constructor enforces leaf-version and internal-key policy.
        ConstructorTargetPolicy,
        /// Target evidence establishes selected signature semantics.
        SelectedSignatureSemantics,
        /// Target evidence establishes whole-transaction conservation.
        WholeTransactionConservation,
        /// Target evidence establishes taproot commitment and control-path validity.
        TaprootCommitmentAndControlPath,
        /// History report establishes current predecessor freshness.
        CurrentRootFreshness,
        /// History report checks the complete bound root-edge sequence.
        RootHistoryEdgeSequence,
        /// Recovery report establishes unrelated-process public reconstruction.
        PublicReconstruction,
    }
}

impl AnnouncementDuty {
    /// Assign each duty exactly once without implying emitted or accepted evidence.
    #[must_use]
    pub const fn boundary(self) -> AnnouncementRequirementBoundary {
        match self {
            Self::PredecessorMetadataAuthentication
            | Self::PredecessorMaturity
            | Self::AnnouncementLeadWindow
            | Self::SuccessorMetadataDerivation
            | Self::SuccessorConstructorReconstruction
            | Self::OperatorAuthorization
            | Self::TransactionStructure
            | Self::RootAndProjectionClosure
            | Self::SponsorIsolation => AnnouncementRequirementBoundary::Runtime,
            Self::MetadataLeafCommitment
            | Self::StaticSubtreeContinuity
            | Self::ConstructorTargetPolicy => AnnouncementRequirementBoundary::LinkedConstructor,
            Self::SelectedSignatureSemantics
            | Self::WholeTransactionConservation
            | Self::TaprootCommitmentAndControlPath => {
                AnnouncementRequirementBoundary::TargetEvidence
            }
            Self::CurrentRootFreshness | Self::RootHistoryEdgeSequence => {
                AnnouncementRequirementBoundary::RootHistoryReport
            }
            Self::PublicReconstruction => AnnouncementRequirementBoundary::PublicRecoveryReport,
        }
    }
}
