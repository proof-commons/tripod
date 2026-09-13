//! Symbolic obligations for a later validated maturity-announcement plan.
//!
//! These projections state required work; constructing one proves no transition,
//! constructor, target acceptance, history, or recovery. Semantic execution stays
//! in `realization::announce_maturity`. Target material has no field here.

use architecture::{OperationId, RootId};
use realization::{
    AnnouncementLeadBound, FactId, RelationId, RelationKind, RelationSubject, StateField,
    StateLawParameter, TransactionSide,
};

use crate::capability::census_enum;

const OPERATION: OperationId = OperationId::AnnounceMaturity;

census_enum! {
    /// Ancillary public information needed to reconstruct the successor.
    pub enum AnnouncementRecoveryInputRole {
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
    /// Symbolic STATE field laws. The subject's input is implicit; operands
    /// are ordered additional inputs, never deployment admissibility bounds.
    pub enum StateFieldLawKind {
        /// Copy the input field unchanged. Model: `packages/model/src/ops/maturity.rs`
        /// and `packages/model/src/ops/relabel.rs` preserve the other fields.
        Copy,
        /// Checked addition of one amount fact to the input amount.
        /// Model: `packages/model/src/ops/admission.rs` adds admitted value to Q;
        /// `packages/model/src/ops/cycle.rs` adds predecessor Q to backing.
        CheckedAmountAdd,
        /// Checked subtraction of one amount fact from the input amount.
        /// Model: `packages/model/src/ops/redeem.rs` subtracts the live receipt amount.
        CheckedAmountSubtract,
        /// Set the amount to zero. Model: `packages/model/src/ops/cycle.rs` clears Q
        /// and `packages/model/src/ops/redeem.rs` leaves a zero-amount tombstone.
        ZeroAmount,
        /// Increment the input cycle with checked arithmetic and no operands.
        /// Model: `PoolState::next_cycle` in `packages/model/src/pool.rs`.
        NextCycle,
        /// Set maturity to announced at the single requested-cycle fact.
        /// Model: `packages/model/src/ops/maturity.rs`.
        AnnounceRequestedCycle,
        /// Derive issuance from Q and total supply over backing; use all issuance
        /// as live when next cycle reaches maturity or maturity is complete,
        /// otherwise its floored Zeta share; add it
        /// to input live supply and add time-locked supply at maturity.
        /// Operands: predecessor backing, time-locked supply, Q, cycle, maturity,
        /// then published Zeta. Model: `packages/model/src/ops/cycle.rs`.
        CycleLiveSupply,
        /// Derive issuance from Q and total supply over backing, then add issuance
        /// minus its normal-phase or floored Zeta live share to time-locked supply, except
        /// set the output to zero at the maturity cycle. Operands: predecessor
        /// backing, live supply, Q, cycle, maturity, then published Zeta.
        /// Model: `packages/model/src/ops/cycle.rs`.
        CycleTimeLockedSupply,
        /// Complete an announced maturity exactly when predecessor cycle plus one
        /// reaches its announced cycle; otherwise copy input maturity.
        /// Operand: predecessor cycle. Model: `packages/model/src/ops/cycle.rs`.
        CycleMaturity,
        /// Subtract the floored payout (live input amount times input backing over
        /// total supply) from input backing, reaching zero on sealing redemption.
        /// Operands: predecessor live supply, time-locked supply, live input amount.
        /// Model: `packages/model/src/ops/redeem.rs`.
        RedemptionBacking,
        /// Subtract the minimum of ASH input amount, input live supply, and total
        /// supply minus one from input live supply. Operands: predecessor time-locked
        /// supply, input ASH amount. Model: `packages/model/src/ops/ash.rs`.
        ClearLiveSupply,
    }
}

impl StateFieldLawKind {
    /// Number of ordered additional operands in this law's signature.
    #[must_use]
    pub const fn operand_count(self) -> usize {
        match self {
            Self::Copy | Self::ZeroAmount | Self::NextCycle => 0,
            Self::CheckedAmountAdd
            | Self::CheckedAmountSubtract
            | Self::AnnounceRequestedCycle
            | Self::CycleMaturity => 1,
            Self::CycleLiveSupply | Self::CycleTimeLockedSupply => 6,
            Self::RedemptionBacking => 3,
            Self::ClearLiveSupply => 2,
        }
    }
}

/// An ordered symbolic input to a STATE field law.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StateLawOperand {
    /// A primitive semantic fact with its complete key.
    Fact(FactId),
    /// A published parameter whose value is owned by the model's constants.
    PublishedParameter(StateLawParameter),
}

/// A law with an ordered operand list matching its kind's signature.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateFieldLaw {
    pub kind: StateFieldLawKind,
    pub operands: Vec<StateLawOperand>,
}

/// One field law whose subject is the output field and whose input is explicit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateFieldRequirement {
    pub field: StateField,
    pub input: FactId,
    pub output: FactId,
    pub law: StateFieldLaw,
}

census_enum! {
    /// Structural defects in a symbolic field requirement; no transition is run.
    #[derive(thiserror::Error)]
    pub enum StateFieldRequirementError {
        /// Input does not name the declared field on the input side.
        #[error("input must name the declared STATE field on the input side")]
        InvalidInput,
        /// Output does not name the same operation and field on the output side.
        #[error("output must name the same operation and STATE field on the output side")]
        InvalidOutput,
        /// Operand count differs from the law kind's signature.
        #[error("operand count does not match the STATE field law signature")]
        OperandCount,
    }
}

impl StateFieldRequirement {
    /// Validate subject keys and signature arity, without executing the law.
    ///
    /// # Errors
    /// Returns a structural error when the subject sides, operation or field
    /// disagree, or the operand count differs from the law's signature.
    pub fn validate(&self) -> Result<(), StateFieldRequirementError> {
        let FactId::StateField {
            operation,
            side: TransactionSide::Input,
            field,
        } = &self.input
        else {
            return Err(StateFieldRequirementError::InvalidInput);
        };
        if *field != self.field {
            return Err(StateFieldRequirementError::InvalidInput);
        }
        if !matches!(&self.output,
            FactId::StateField { operation: output_operation, side: TransactionSide::Output, field: output_field }
                if output_operation == operation && *output_field == self.field)
        {
            return Err(StateFieldRequirementError::InvalidOutput);
        }
        if self.law.operands.len() != self.law.kind.operand_count() {
            return Err(StateFieldRequirementError::OperandCount);
        }
        Ok(())
    }

    /// State the announcement law for one field, without executing it.
    #[must_use]
    pub fn for_field(field: StateField) -> Self {
        let law = match field {
            StateField::Omega
            | StateField::YL
            | StateField::YT
            | StateField::Q
            | StateField::Cycle => StateFieldLaw {
                kind: StateFieldLawKind::Copy,
                operands: Vec::new(),
            },
            StateField::Maturity => StateFieldLaw {
                kind: StateFieldLawKind::AnnounceRequestedCycle,
                operands: vec![StateLawOperand::Fact(requested_cycle())],
            },
        };
        Self {
            field,
            input: state_fact(TransactionSide::Input, field),
            output: state_fact(TransactionSide::Output, field),
            law,
        }
    }
}

census_enum! {
    /// Required predecessor maturity, without a cycle value.
    pub enum AnnouncementPredecessorMaturity {
        /// Already-announced and complete predecessors are refused.
        Unannounced,
    }
}

const fn state_fact(side: TransactionSide, field: StateField) -> FactId {
    FactId::StateField {
        operation: OPERATION,
        side,
        field,
    }
}

const fn state_fields(side: TransactionSide) -> [FactId; 6] {
    [
        state_fact(side, StateField::Omega),
        state_fact(side, StateField::YL),
        state_fact(side, StateField::YT),
        state_fact(side, StateField::Q),
        state_fact(side, StateField::Cycle),
        state_fact(side, StateField::Maturity),
    ]
}

const fn requested_cycle() -> FactId {
    FactId::RequestedAnnouncementCycle {
        operation: OPERATION,
    }
}

/// The six field laws and their distinct admissibility conditions.
///
/// Predecessor maturity must be unannounced. Checked cycle arithmetic must
/// establish `input_cycle + minimum_lead <= requested_cycle <= input_cycle +
/// maximum_lead`, inclusively. The lead selectors resolve to architecture bounds;
/// their values constrain acceptance and are not field-law operands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnnouncementMetadataRequirement {
    pub fields: [StateFieldRequirement; 6],
    pub predecessor_maturity: AnnouncementPredecessorMaturity,
    /// Input cycle used to derive both checked window endpoints.
    pub input_cycle: FactId,
    pub requested_cycle: FactId,
    /// Minimum selector resolves through `AnnouncementLeadBound::bound_id`.
    pub minimum_lead: FactId,
    /// Maximum selector resolves through `AnnouncementLeadBound::bound_id`.
    pub maximum_lead: FactId,
}

impl AnnouncementMetadataRequirement {
    /// Complete symbolic requirement; constructing it validates no plan.
    #[must_use]
    pub fn required() -> Self {
        Self {
            fields: [
                StateField::Omega,
                StateField::YL,
                StateField::YT,
                StateField::Q,
                StateField::Cycle,
                StateField::Maturity,
            ]
            .map(StateFieldRequirement::for_field),
            predecessor_maturity: AnnouncementPredecessorMaturity::Unannounced,
            input_cycle: state_fact(TransactionSide::Input, StateField::Cycle),
            requested_cycle: requested_cycle(),
            minimum_lead: FactId::AnnouncementLead {
                operation: OPERATION,
                bound: AnnouncementLeadBound::Minimum,
            },
            maximum_lead: FactId::AnnouncementLead {
                operation: OPERATION,
                bound: AnnouncementLeadBound::Maximum,
            },
        }
    }

    /// Fifteen public keys in declaration order: input side, output side, request, leads.
    #[must_use]
    pub const fn public_facts() -> [FactId; 15] {
        let [i_omega, i_live, i_locked, i_q, i_cycle, i_maturity] =
            state_fields(TransactionSide::Input);
        let [o_omega, o_live, o_locked, o_q, o_cycle, o_maturity] =
            state_fields(TransactionSide::Output);
        [
            i_omega,
            i_live,
            i_locked,
            i_q,
            i_cycle,
            i_maturity,
            o_omega,
            o_live,
            o_locked,
            o_q,
            o_cycle,
            o_maturity,
            requested_cycle(),
            FactId::AnnouncementLead {
                operation: OPERATION,
                bound: AnnouncementLeadBound::Minimum,
            },
            FactId::AnnouncementLead {
                operation: OPERATION,
                bound: AnnouncementLeadBound::Maximum,
            },
        ]
    }
}

/// STATE edge and the declaration policies that constrain it.
/// An exit names only the sides it has; announcement requires both complete sides.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateSuccessionRequirement {
    pub root: RootId,
    pub input_fields: Option<[FactId; 6]>,
    pub output_fields: Option<[FactId; 6]>,
    pub root_relation: RelationId,
    pub certificate_relation: RelationId,
}

impl StateSuccessionRequirement {
    /// Announcement succession; endpoint linkage remains a history obligation.
    pub const REQUIRED: Self = Self {
        root: RootId::State,
        input_fields: Some(state_fields(TransactionSide::Input)),
        output_fields: Some(state_fields(TransactionSide::Output)),
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
    /// Six input STATE facts and the public requested-cycle fact.
    pub source_facts: [FactId; 7],
    /// Six output STATE facts recovered through the typed transition.
    pub result_facts: [FactId; 6],
    /// Six ancillary public recovery input roles.
    pub inputs: &'static [AnnouncementRecoveryInputRole],
    /// Ordered reconstruction and comparison duties.
    pub steps: &'static [AnnouncementRecoveryStep],
}

impl PublicRecoveryRequirement {
    /// Full public recovery requirement, without fixing a target output position.
    pub const REQUIRED: Self = Self {
        publication: AnnouncementPublicationRole::AcceptedTransactionWitnessAndOutputs,
        source_facts: [
            state_fact(TransactionSide::Input, StateField::Omega),
            state_fact(TransactionSide::Input, StateField::YL),
            state_fact(TransactionSide::Input, StateField::YT),
            state_fact(TransactionSide::Input, StateField::Q),
            state_fact(TransactionSide::Input, StateField::Cycle),
            state_fact(TransactionSide::Input, StateField::Maturity),
            requested_cycle(),
        ],
        result_facts: state_fields(TransactionSide::Output),
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
