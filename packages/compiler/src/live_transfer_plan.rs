//! The public live-transfer target-operation plan (Guide-13 §5, §6, §8).
//!
//! [`crate::operation_plan`] plans one operation whose Phase-4 policy
//! fixed a single representation, and everything it publishes is a
//! single census. A live transfer is the first operation Phase 5 plans
//! under *two* admitted representations at once — §8.2 keeps the
//! explicit plan and the private-committed plan side by side — so the
//! censuses that depend on the representation are published once per
//! representation, and the facts §5 fixes for the operation itself are
//! published once for the operation.
//!
//! # Where the split falls, and why
//!
//! The line is not a matter of taste. §6.6 states exactly which facts a
//! paired explicit and private fixture must agree on — live class,
//! exact explicit `U`, authorization, absence of roots, absence of
//! issuance and destruction, lateral flow, the sponsor relation and
//! shape, and the transition-certificate projection — and every one of
//! those is a relation the realization declares for the operation
//! regardless of which representation a plan selected. They are
//! published once, as the operation's own projections, and the
//! validator proves the relation census carrying them really is equal
//! across the representations rather than assuming it.
//!
//! What §6.6 leaves free is the other side of the same sentence: target
//! bytes, commitments, proofs, and witness sizes may differ. The proof
//! alternative selected for value conservation is exactly that
//! difference, and it propagates into the capability census, the layout
//! requirements, and the coverage rows, all of which are therefore
//! per-representation. §19.4 fixes the shape of the difference —
//! explicit arithmetic active and CT conservation inactive under the
//! explicit plan, the reverse under the private plan — so the validator
//! requires the divergence to be exactly the value-conservation
//! relation and exactly that pair of capabilities. A second diverging
//! relation is a defect, and so is a conservation relation that does
//! not diverge.
//!
//! # The contract is checked, not restated
//!
//! §5 fixes concrete values: the protocol object is `RECEIPT_L`, the
//! closed asset is explicit `U`, both cardinality minima are one, every
//! architecture root is forbidden, the transition certificate is the
//! one required projection, the only open flow is the fee sponsor, and
//! at most one sponsor envelope exists. Each of those is also a typed
//! realization declaration, so the projections below are derived from
//! the declarations and then required to say what §5 says. A projection
//! that merely repeated the guide would agree with it forever and would
//! never notice a realization that had drifted; a projection that only
//! copied the realization would publish the drift as though it were the
//! contract. Deriving and then checking is the only arrangement in
//! which the two can disagree, and [`LiveTransferClause`] names which
//! sentence they disagreed about.
//!
//! # What the plan does not carry
//!
//! The prohibitions of [`crate::operation_plan`] hold here unchanged
//! and one is sharper. No digest and no field reserved for one (§1.13).
//! No graph index, search count, target opcode, target position, script
//! byte, filesystem path, or environment value (§8.1). No individual
//! sponsor amount anywhere, in any published field, under any
//! representation (§1.9): the sponsor projection carries membership,
//! cardinality, envelope multiplicity, and the isolation relation, and
//! there is no amount-shaped field for one to occupy. And no key,
//! blinder, opening, or nonce (§1.10) — the private-committed plan is a
//! statement about which proof alternative a target must supply, never
//! an interface that handles the material the proof is built from.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{
    AssetId, DeltaKind, ObjectId, OpenFlowKind, OperationId, ProjectionId, ProjectionRule, RootId,
    RootUse,
};
use realization::{
    CardinalityMaximum, ConstructibilityClass, Count, ExpectedCanonicalDelta, ObservedSide,
    Relation, RelationDeclaration, RelationId, RepresentationMode,
};

pub use crate::{
    capability::RequiredCapability,
    coverage::CoverageRequirementId,
    layout::LayoutRequirement,
    lifecycle::LifecycleRequirement,
    operation_plan::{
        AbstractCarrierRequirement, TargetCoverageObligation, TargetCoverageRequirement,
        TargetExecutionCase, TargetLifecycleStatus, TargetOperationSource, TargetRelationCase,
        TargetRelationRequirement,
    },
    placement::PlacementSearchLimits,
    sponsor_region::OrdinaryLbtcRole,
    target::ExternalEvidenceRole,
};

use crate::{
    CompileError,
    analyzed::{AnalyzedProofPlan, ScopedAnalyzedProgram, analyze_scoped_program},
    analyzed_operation::AnalyzedOperation,
    capability::census_enum,
    case::ExecutionCaseId,
    input::BoundCompilerInput,
    layout::names_sponsor_amount,
    lifecycle::RepresentationChoiceId,
    operation_plan::{
        operation_relations, project_carriers, project_cases, project_coverage, project_lifecycle,
        project_relations,
    },
    sponsor_region::{ORDINARY_LBTC, ordinary_lbtc_role_of},
    target::canonical_census,
};

/// The one operation this boundary plans (§8.2).
const PLANNED: OperationId = OperationId::TransferLive;

/// The one protocol object class a live transfer consumes and creates
/// (§5.1, §5.4).
const RECEIPT_L: ObjectId = ObjectId::ReceiptLive;

/// The closed protocol asset, explicit on both sides (§5.1, §8.2).
const CLOSED_ASSET: AssetId = AssetId::U;

/// The substrate asset the optional sponsor envelope moves (§5.8).
const SPONSOR_ASSET: AssetId = AssetId::Lbtc;

/// The one semantic projection a live transfer must produce (§5.7).
const REQUIRED_PROJECTION: ProjectionId = ProjectionId::TransitionCertificate;

/// The lifecycle exit a live transfer itself implements (§8.2).
const IMPLEMENTED_EXIT: OperationId = OperationId::TransferLive;

/// The lifecycle exits a live-transfer candidate leaves outstanding
/// (§1.12, §8.2).
const OUTSTANDING_EXITS: [OperationId; 2] = [OperationId::Redeem, OperationId::Burn];

census_enum! {
    /// One representation plan Phase 5 admits for a live transfer.
    ///
    /// §6.1 states this vocabulary and states it exhaustively: these two
    /// modes and no others. `PublicCommitted` is absent by ruling rather
    /// than by omission, and [`DeferredRepresentation`] is where it is
    /// accounted for, so a reader who finds only two members here can
    /// find out why without leaving the type.
    pub enum LiveTransferRepresentationPlan {
        /// Explicit receipt values and checked aggregate arithmetic
        /// (§6.2).
        Explicit,
        /// Confidential receipt values and target CT conservation
        /// (§6.3).
        PrivateCommitted,
    }
}

impl LiveTransferRepresentationPlan {
    /// The realization mode this plan selects.
    #[must_use]
    pub const fn mode(self) -> RepresentationMode {
        match self {
            Self::Explicit => RepresentationMode::Explicit,
            Self::PrivateCommitted => RepresentationMode::PrivateCommitted,
        }
    }

    /// The plan one realization mode names, where Guide 13 admits it.
    ///
    /// Exhaustive with no wildcard arm: a realization mode added later
    /// stops this crate compiling until Guide 13 says whether it is an
    /// admitted plan or a recorded deferral, which is the only mechanism
    /// that keeps the two censuses a partition rather than two lists.
    #[must_use]
    pub const fn of(mode: RepresentationMode) -> Option<Self> {
        match mode {
            RepresentationMode::Explicit => Some(Self::Explicit),
            RepresentationMode::PrivateCommitted => Some(Self::PrivateCommitted),
            RepresentationMode::PublicCommitted => None,
        }
    }

    /// The capability this plan's value-conservation proof requires
    /// (§19.4).
    #[must_use]
    pub const fn conservation_capability(self) -> RequiredCapability {
        match self {
            Self::Explicit => RequiredCapability::ExactPublicAmountArithmetic,
            Self::PrivateCommitted => RequiredCapability::ConfidentialValueConservation,
        }
    }
}

census_enum! {
    /// One representation Guide 13 names and this plan does not admit.
    ///
    /// A deferral is data, not silence. Both members are facts the
    /// analysis can still check, so each is re-derived rather than
    /// asserted, and a realization that made one of them admissible
    /// fails the plan instead of quietly widening the ABI.
    pub enum DeferredRepresentation {
        /// The public-committed mode, deferred while Guide 11 leaves
        /// authenticated public opening open (§6.1).
        PublicCommittedMode,
        /// Inputs and outputs combining both admitted plans in one
        /// transfer (§6.5).
        MixedComposition,
    }
}

census_enum! {
    /// Why one deferred representation is not admitted here.
    ///
    /// The ground is what the analysis can still see, not the guide
    /// sentence that decided it: a ground stops holding when the
    /// declarations change, and that is what makes the deferral
    /// checkable.
    pub enum RepresentationDeferralGround {
        /// The operation's representation relation approves no such
        /// mode.
        UnapprovedMode,
        /// The representation decision is one variable per object
        /// family, so no per-reference mode exists to mix.
        NoPerReferenceVariable,
    }
}

impl DeferredRepresentation {
    /// The ground on which this representation stays deferred.
    #[must_use]
    pub const fn ground(self) -> RepresentationDeferralGround {
        match self {
            Self::PublicCommittedMode => RepresentationDeferralGround::UnapprovedMode,
            Self::MixedComposition => RepresentationDeferralGround::NoPerReferenceVariable,
        }
    }
}

census_enum! {
    /// One sentence of the §5 live-transfer contract.
    ///
    /// The clause a derived projection and the guide disagreed about.
    /// Every member names one fixed value, so a failure points at the
    /// declaration to read rather than at the whole operation.
    pub enum LiveTransferClause {
        /// Both protocol sides recognize `RECEIPT_L` carrying `U`
        /// (§5.1, §5.4).
        ProtocolObject,
        /// Each side admits exactly the protocol and sponsor families
        /// (§5.4).
        ClassClosure,
        /// At least one protocol input, bounded by an architecture bound
        /// (§5.1).
        InputCardinality,
        /// At least one protocol output, bounded by an architecture
        /// bound (§5.1).
        OutputCardinality,
        /// Every consumed receipt requires its owner's authorization
        /// (§5.5, §8.2).
        OwnerAuthorization,
        /// The operation is constructible by the owners of `RECEIPT_L`
        /// (§5.5).
        Constructibility,
        /// Aggregate `U` is conserved from receipts to receipts (§5.1).
        ValueConservation,
        /// The one canonical `U` flow is lateral, with no issuance and
        /// no destruction (§5.6).
        CanonicalFlow,
        /// Every architecture root is forbidden (§5.7).
        RootPolicy,
        /// The transition certificate is required and every specialized
        /// projection is forbidden (§5.7).
        CertificateProjection,
        /// The sponsor region recognizes ordinary L-BTC (§5.8).
        SponsorObject,
        /// The fee sponsor is the only open flow (§5.8).
        SponsorFlow,
        /// Sponsor inputs are optional and bounded (§5.8).
        SponsorInputs,
        /// Sponsor change is optional and at most one output (§5.8).
        SponsorChange,
        /// At most one sponsor envelope exists (§1.9, §5.8).
        SponsorEnvelope,
        /// The sponsor region is disjoint from the protocol region
        /// (§1.9).
        SponsorIsolation,
        /// Whole-transaction substrate conservation stays external
        /// evidence (§1.9).
        SubstrateConservation,
        /// The approved representation set is exactly the admitted plans
        /// (§6.1).
        RepresentationApproval,
        /// Transfer is implemented and burn and redeem stay outstanding
        /// (§1.12, §8.2).
        LifecycleExits,
    }
}

/// The admitted and deferred representations of one live transfer
/// (§6.1, §6.5).
///
/// Both halves are censuses complete by construction: the admitted set
/// is keyed by [`LiveTransferRepresentationPlan::ALL`] and the deferred
/// set by [`DeferredRepresentation::ALL`], so neither can lose a member
/// silently. What the policy adds beyond the two constants is the
/// realization's own approved set, which is what the admitted plans are
/// checked against — a policy that reported its own choices as the
/// approved set could never disagree with the analysis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveTransferRepresentationPolicy {
    approved: BTreeSet<RepresentationMode>,
    admitted: BTreeSet<LiveTransferRepresentationPlan>,
    deferred: BTreeMap<DeferredRepresentation, RepresentationDeferralGround>,
    relation: RelationId,
}

impl LiveTransferRepresentationPolicy {
    /// Every representation mode the realization approves for
    /// `RECEIPT_L`.
    pub fn approved(&self) -> impl Iterator<Item = RepresentationMode> + '_ {
        self.approved.iter().copied()
    }

    /// Every representation plan this operation admits, in census order.
    pub fn admitted(&self) -> impl Iterator<Item = LiveTransferRepresentationPlan> + '_ {
        self.admitted.iter().copied()
    }

    /// Every representation Guide 13 names and defers, with its ground.
    pub fn deferred(
        &self,
    ) -> impl Iterator<Item = (DeferredRepresentation, RepresentationDeferralGround)> + '_ {
        self.deferred
            .iter()
            .map(|(representation, ground)| (*representation, *ground))
    }

    /// The ground one named representation stays deferred on.
    #[must_use]
    pub fn deferral(
        &self,
        representation: DeferredRepresentation,
    ) -> Option<RepresentationDeferralGround> {
        self.deferred.get(&representation).copied()
    }

    /// The realization relation that decides the representation.
    #[must_use]
    pub const fn relation(&self) -> &RelationId {
        &self.relation
    }
}

/// One declared object-family cardinality (§5.1, §5.8).
///
/// The maximum is retained in the architecture-owned form the relation
/// declares. §5.1 calls the transfer bounds candidate assignments for
/// architecture-owned bounds, so resolving one to an integer here would
/// publish a candidate assignment as a settled bound — the resource
/// study is what assigns them, and this plan states the bound's
/// identity and nothing more.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LiveTransferCardinality {
    minimum: Count,
    maximum: CardinalityMaximum,
}

impl LiveTransferCardinality {
    /// The declared minimum count.
    #[must_use]
    pub const fn minimum(self) -> Count {
        self.minimum
    }

    /// The declared maximum, exact or architecture-bound.
    #[must_use]
    pub const fn maximum(self) -> CardinalityMaximum {
        self.maximum
    }

    /// Whether the family may be absent entirely.
    #[must_use]
    pub const fn is_optional(self) -> bool {
        self.minimum.is_zero()
    }
}

/// The live-class closure of one transfer (§5.4).
///
/// The forbidden census is derived by subtraction from
/// [`ObjectId::ALL`] rather than transcribed from §5.4's list. The list
/// there is illustrative and the subtraction is complete: an object
/// family added to the architecture is forbidden here the moment it
/// exists, with no edit and no chance of an omission reading as an
/// admission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveTransferClassProjection {
    protocol: ObjectId,
    sponsor: ObjectId,
    admitted: BTreeSet<ObjectId>,
    forbidden: BTreeSet<ObjectId>,
    input_closure: RelationId,
    output_closure: RelationId,
    input_recognition: RelationId,
    output_recognition: RelationId,
}

impl LiveTransferClassProjection {
    /// The one protocol object class, on both sides.
    #[must_use]
    pub const fn protocol(&self) -> ObjectId {
        self.protocol
    }

    /// The object family the optional sponsor envelope uses.
    #[must_use]
    pub const fn sponsor(&self) -> ObjectId {
        self.sponsor
    }

    /// Every object family either side admits, in census order.
    pub fn admitted(&self) -> impl Iterator<Item = ObjectId> + '_ {
        self.admitted.iter().copied()
    }

    /// Every object family forbidden as a transfer input or output.
    pub fn forbidden(&self) -> impl Iterator<Item = ObjectId> + '_ {
        self.forbidden.iter().copied()
    }

    /// Whether one object family is forbidden here.
    #[must_use]
    pub fn forbids(&self, object: ObjectId) -> bool {
        self.forbidden.contains(&object)
    }

    /// The closure relation of one transaction side.
    #[must_use]
    pub const fn closure(&self, side: ObservedSide) -> &RelationId {
        match side {
            ObservedSide::Input => &self.input_closure,
            ObservedSide::Output => &self.output_closure,
        }
    }

    /// The protocol recognition relation of one transaction side.
    #[must_use]
    pub const fn recognition(&self, side: ObservedSide) -> &RelationId {
        match side {
            ObservedSide::Input => &self.input_recognition,
            ObservedSide::Output => &self.output_recognition,
        }
    }
}

/// The owner-authorization closure of one transfer (§5.5, §8.2).
///
/// §8.2 fixes the authorization as *every* input owner, and the
/// quantifier is not a separate field because no declaration carries
/// one. What carries it is the agreement of two relations over one
/// object family: an owner-authorization relation naming `RECEIPT_L`
/// and a constructibility relation whose class is the owners of
/// `RECEIPT_L`. Either alone admits a weaker reading, so the derivation
/// requires both and requires them to name the same family.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveTransferOwnerProjection {
    object: ObjectId,
    class: ConstructibilityClass,
    authorization: RelationId,
    constructibility: RelationId,
}

impl LiveTransferOwnerProjection {
    /// The object family whose owners must authorize.
    #[must_use]
    pub const fn object(&self) -> ObjectId {
        self.object
    }

    /// The constructibility class of the operation.
    #[must_use]
    pub const fn class(&self) -> ConstructibilityClass {
        self.class
    }

    /// The owner-authorization relation.
    #[must_use]
    pub const fn authorization(&self) -> &RelationId {
        &self.authorization
    }

    /// The constructibility relation.
    #[must_use]
    pub const fn constructibility(&self) -> &RelationId {
        &self.constructibility
    }
}

/// The value and partition contract of one transfer (§5.1, §5.3, §5.6).
///
/// No amount appears and none can: the compiler plans relations over
/// amounts and never holds one. What is published is the conservation
/// relation's own shape — the closed asset, the object families on each
/// side, the declared cardinalities, and the canonical flow — from
/// which issuance, destruction, and laterality are derived rather than
/// asserted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveTransferValueProjection {
    asset: AssetId,
    input_objects: BTreeSet<ObjectId>,
    output_objects: BTreeSet<ObjectId>,
    input_cardinality: LiveTransferCardinality,
    output_cardinality: LiveTransferCardinality,
    flow: BTreeSet<ExpectedCanonicalDelta>,
    conservation: RelationId,
    canonical_flow: RelationId,
    input_cardinality_relation: RelationId,
    output_cardinality_relation: RelationId,
}

impl LiveTransferValueProjection {
    /// The exact closed asset the transfer conserves.
    #[must_use]
    pub const fn asset(&self) -> AssetId {
        self.asset
    }

    /// The object families the conservation relation sums on one side.
    pub fn objects(&self, side: ObservedSide) -> impl Iterator<Item = ObjectId> + '_ {
        match side {
            ObservedSide::Input => self.input_objects.iter().copied(),
            ObservedSide::Output => self.output_objects.iter().copied(),
        }
    }

    /// The declared protocol cardinality of one side.
    #[must_use]
    pub const fn cardinality(&self, side: ObservedSide) -> LiveTransferCardinality {
        match side {
            ObservedSide::Input => self.input_cardinality,
            ObservedSide::Output => self.output_cardinality,
        }
    }

    /// The cardinality relation of one side.
    #[must_use]
    pub const fn cardinality_relation(&self, side: ObservedSide) -> &RelationId {
        match side {
            ObservedSide::Input => &self.input_cardinality_relation,
            ObservedSide::Output => &self.output_cardinality_relation,
        }
    }

    /// Every canonical delta the operation expects.
    pub fn flow(&self) -> impl Iterator<Item = &ExpectedCanonicalDelta> {
        self.flow.iter()
    }

    /// The value-conservation relation.
    #[must_use]
    pub const fn conservation(&self) -> &RelationId {
        &self.conservation
    }

    /// The canonical-delta policy relation.
    #[must_use]
    pub const fn canonical_flow(&self) -> &RelationId {
        &self.canonical_flow
    }

    /// Whether any expected delta issues the closed asset.
    ///
    /// Derived from the delta census, never asserted: a stored `false`
    /// could disagree with the flow beside it.
    #[must_use]
    pub fn issues(&self) -> bool {
        self.flow
            .iter()
            .any(|delta| delta.kind == DeltaKind::Issuance)
    }

    /// Whether any expected delta destroys the closed asset.
    #[must_use]
    pub fn destroys(&self) -> bool {
        self.flow
            .iter()
            .any(|delta| delta.kind == DeltaKind::Destruction || delta.destruction_tag.is_some())
    }
}

/// The sponsor contract of one transfer (§1.9, §5.8).
///
/// Membership, region, cardinality, envelope multiplicity, isolation,
/// and the external substrate-conservation relation. There is no amount
/// field, no aggregate, and no positivity flag, because §1.9 forbids a
/// compiler plan from requiring an individual sponsor amount to be any
/// of those things — and a field that existed only to hold `None` would
/// be the reservation the ruling refuses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveTransferSponsorProjection {
    object: ObjectId,
    region: OrdinaryLbtcRole,
    open_flows: BTreeSet<OpenFlowKind>,
    envelope_maximum: Count,
    input_cardinality: LiveTransferCardinality,
    change_cardinality: LiveTransferCardinality,
    isolation: RelationId,
    multiplicity: RelationId,
    open_flow_policy: RelationId,
    substrate_conservation: RelationId,
}

impl LiveTransferSponsorProjection {
    /// The object family the sponsor envelope uses.
    #[must_use]
    pub const fn object(&self) -> ObjectId {
        self.object
    }

    /// What ordinary L-BTC means inside this operation.
    #[must_use]
    pub const fn region(&self) -> OrdinaryLbtcRole {
        self.region
    }

    /// Every open flow the operation declares.
    pub fn open_flows(&self) -> impl Iterator<Item = OpenFlowKind> + '_ {
        self.open_flows.iter().copied()
    }

    /// The greatest number of sponsor envelopes admitted.
    #[must_use]
    pub const fn envelope_maximum(&self) -> Count {
        self.envelope_maximum
    }

    /// The declared sponsor-input cardinality.
    #[must_use]
    pub const fn input_cardinality(&self) -> LiveTransferCardinality {
        self.input_cardinality
    }

    /// The declared sponsor-change cardinality.
    #[must_use]
    pub const fn change_cardinality(&self) -> LiveTransferCardinality {
        self.change_cardinality
    }

    /// Whether the sponsor envelope is optional in this operation.
    #[must_use]
    pub const fn is_optional(&self) -> bool {
        self.input_cardinality.is_optional()
    }

    /// The sponsor-isolation relation.
    #[must_use]
    pub const fn isolation(&self) -> &RelationId {
        &self.isolation
    }

    /// The sponsor-envelope multiplicity relation.
    #[must_use]
    pub const fn multiplicity(&self) -> &RelationId {
        &self.multiplicity
    }

    /// The open-flow policy relation.
    #[must_use]
    pub const fn open_flow_policy(&self) -> &RelationId {
        &self.open_flow_policy
    }

    /// The whole-transaction substrate-conservation relation.
    #[must_use]
    pub const fn substrate_conservation(&self) -> &RelationId {
        &self.substrate_conservation
    }
}

/// The architecture-root contract of one transfer (§5.7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveTransferRootProjection {
    expected: BTreeMap<RootId, RootUse>,
    relation: RelationId,
}

impl LiveTransferRootProjection {
    /// Every architecture root with the use this operation makes of it.
    pub fn expected(&self) -> impl Iterator<Item = (RootId, RootUse)> + '_ {
        self.expected.iter().map(|(root, use_)| (*root, *use_))
    }

    /// The use this operation makes of one root.
    #[must_use]
    pub fn root_use(&self, root: RootId) -> Option<RootUse> {
        self.expected.get(&root).copied()
    }

    /// The root-policy relation.
    #[must_use]
    pub const fn relation(&self) -> &RelationId {
        &self.relation
    }

    /// Whether every architecture root is forbidden.
    ///
    /// Derived over the complete [`RootId::ALL`] census, so a root the
    /// policy failed to mention makes this false rather than absent.
    #[must_use]
    pub fn forbids_every_root(&self) -> bool {
        RootId::ALL
            .iter()
            .all(|root| self.expected.get(root) == Some(&RootUse::Forbidden))
    }
}

/// The semantic-projection contract of one transfer (§5.7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveTransferCertificateProjection {
    expected: BTreeMap<ProjectionId, ProjectionRule>,
    relation: RelationId,
}

impl LiveTransferCertificateProjection {
    /// Every semantic projection with the rule this operation applies.
    pub fn expected(&self) -> impl Iterator<Item = (ProjectionId, ProjectionRule)> + '_ {
        self.expected.iter().map(|(id, rule)| (*id, *rule))
    }

    /// The rule this operation applies to one projection.
    #[must_use]
    pub fn rule(&self, projection: ProjectionId) -> Option<ProjectionRule> {
        self.expected.get(&projection).copied()
    }

    /// Every projection this operation must produce.
    pub fn required(&self) -> impl Iterator<Item = ProjectionId> + '_ {
        self.rules(ProjectionRule::Required)
    }

    /// Every projection this operation must not produce.
    pub fn forbidden(&self) -> impl Iterator<Item = ProjectionId> + '_ {
        self.rules(ProjectionRule::Forbidden)
    }

    /// The projection-policy relation.
    #[must_use]
    pub const fn relation(&self) -> &RelationId {
        &self.relation
    }

    /// Every projection carrying one rule, in census order.
    fn rules(&self, wanted: ProjectionRule) -> impl Iterator<Item = ProjectionId> + '_ {
        self.expected
            .iter()
            .filter(move |(_, rule)| **rule == wanted)
            .map(|(projection, _)| *projection)
    }
}

/// The lifecycle-exit closure of one transfer (§1.12, §8.2).
///
/// The exits, not the per-representation rows. Each representation's
/// own [`TargetLifecycleStatus`] carries the rows, which differ because
/// a lifecycle requirement names the mode it belongs to; the exits
/// themselves do not differ, and the validator proves that rather than
/// publishing whichever representation it read first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveTransferLifecycleClosure {
    implemented: BTreeSet<OperationId>,
    outstanding: BTreeSet<OperationId>,
}

impl LiveTransferLifecycleClosure {
    /// Every required exit this compiler scope implements.
    pub fn implemented(&self) -> impl Iterator<Item = OperationId> + '_ {
        self.implemented.iter().copied()
    }

    /// Every required exit declared outside the compiler scope.
    pub fn outstanding(&self) -> impl Iterator<Item = OperationId> + '_ {
        self.outstanding.iter().copied()
    }

    /// Whether every required exit is implemented in scope.
    ///
    /// False for the Guide-13 candidate, whose burn and redeem exits are
    /// outstanding (§1.12).
    #[must_use]
    pub fn release_complete(&self) -> bool {
        self.outstanding.is_empty()
    }
}

/// Everything one representation plan requires of a backend (§8.1).
///
/// The per-representation half of the plan. Every census here is the
/// analyzed census of the plans that selected this representation, and
/// the validator compares each one against every such plan rather than
/// against the one the assembler happened to read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveTransferRepresentationProjection {
    plan: LiveTransferRepresentationPlan,
    cases: BTreeMap<ExecutionCaseId, TargetExecutionCase>,
    relations: BTreeMap<RelationId, TargetRelationRequirement>,
    carriers: BTreeSet<AbstractCarrierRequirement>,
    layout: BTreeSet<LayoutRequirement>,
    coverage: BTreeMap<CoverageRequirementId, TargetCoverageRequirement>,
    capabilities: BTreeSet<RequiredCapability>,
    external_evidence: BTreeSet<ExternalEvidenceRole>,
    lifecycle: TargetLifecycleStatus,
}

impl LiveTransferRepresentationProjection {
    /// The representation plan this projection belongs to.
    #[must_use]
    pub const fn plan(&self) -> LiveTransferRepresentationPlan {
        self.plan
    }

    /// Every execution case, in canonical order.
    pub fn cases(&self) -> impl Iterator<Item = &TargetExecutionCase> {
        self.cases.values()
    }

    /// One execution case by identity.
    #[must_use]
    pub fn case(&self, id: &ExecutionCaseId) -> Option<&TargetExecutionCase> {
        self.cases.get(id)
    }

    /// The exact relation census, in canonical order.
    pub fn relations(&self) -> impl Iterator<Item = &TargetRelationRequirement> {
        self.relations.values()
    }

    /// One relation's requirements by identity.
    #[must_use]
    pub fn relation(&self, relation: &RelationId) -> Option<&TargetRelationRequirement> {
        self.relations.get(relation)
    }

    /// The exact abstract carrier census, in canonical order.
    pub fn carriers(&self) -> impl Iterator<Item = &AbstractCarrierRequirement> {
        self.carriers.iter()
    }

    /// The exact layout census, in canonical order.
    pub fn layout(&self) -> impl Iterator<Item = &LayoutRequirement> {
        self.layout.iter()
    }

    /// The exact relation-indexed coverage census, in canonical order.
    pub fn coverage(&self) -> impl Iterator<Item = &TargetCoverageRequirement> {
        self.coverage.values()
    }

    /// One coverage requirement by identity.
    #[must_use]
    pub fn coverage_requirement(
        &self,
        id: &CoverageRequirementId,
    ) -> Option<&TargetCoverageRequirement> {
        self.coverage.get(id)
    }

    /// Every abstract capability this representation requires.
    pub fn capabilities(&self) -> impl Iterator<Item = RequiredCapability> + '_ {
        self.capabilities.iter().copied()
    }

    /// Every external-evidence role this representation leaves open.
    pub fn external_evidence(&self) -> impl Iterator<Item = ExternalEvidenceRole> + '_ {
        self.external_evidence.iter().copied()
    }

    /// The representation's own lifecycle rows.
    #[must_use]
    pub const fn lifecycle(&self) -> &TargetLifecycleStatus {
        &self.lifecycle
    }
}

/// The validated public plan for one live transfer (§1.12, §8.1).
///
/// Every field is private and there is no public constructor, no
/// `Default`, and no builder: the sole route to a value of this type is
/// [`plan_live_transfer_target_operation`], which runs the complete
/// analysis, its own independent validator, the Phase-5 representation
/// filter, and then the complete re-derivation validation of the
/// assembled plan. A plan assembled from arbitrary fields would be a
/// request rather than an analysis, and nothing downstream could tell
/// the two apart once they shared a type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedLiveTransferOperationPlan {
    operation: OperationId,
    source: TargetOperationSource,
    representation: LiveTransferRepresentationPolicy,
    class: LiveTransferClassProjection,
    owner: LiveTransferOwnerProjection,
    value: LiveTransferValueProjection,
    sponsor: LiveTransferSponsorProjection,
    roots: LiveTransferRootProjection,
    certificate: LiveTransferCertificateProjection,
    lifecycle: LiveTransferLifecycleClosure,
    representations: BTreeMap<LiveTransferRepresentationPlan, LiveTransferRepresentationProjection>,
}

impl ValidatedLiveTransferOperationPlan {
    /// The operation this plan is about — always
    /// `transfer-live-receipts`.
    #[must_use]
    pub const fn operation(&self) -> OperationId {
        self.operation
    }

    /// The complete typed source the plan was derived from.
    #[must_use]
    pub const fn source(&self) -> &TargetOperationSource {
        &self.source
    }

    /// The admitted and deferred representations.
    #[must_use]
    pub const fn representation(&self) -> &LiveTransferRepresentationPolicy {
        &self.representation
    }

    /// The live-class closure.
    #[must_use]
    pub const fn class(&self) -> &LiveTransferClassProjection {
        &self.class
    }

    /// The owner-authorization closure.
    #[must_use]
    pub const fn owner(&self) -> &LiveTransferOwnerProjection {
        &self.owner
    }

    /// The value and partition contract.
    #[must_use]
    pub const fn value(&self) -> &LiveTransferValueProjection {
        &self.value
    }

    /// The sponsor contract.
    #[must_use]
    pub const fn sponsor(&self) -> &LiveTransferSponsorProjection {
        &self.sponsor
    }

    /// The architecture-root contract.
    #[must_use]
    pub const fn roots(&self) -> &LiveTransferRootProjection {
        &self.roots
    }

    /// The semantic-projection contract.
    #[must_use]
    pub const fn certificate(&self) -> &LiveTransferCertificateProjection {
        &self.certificate
    }

    /// The lifecycle-exit closure.
    #[must_use]
    pub const fn lifecycle(&self) -> &LiveTransferLifecycleClosure {
        &self.lifecycle
    }

    /// Every admitted representation's projection, in census order.
    pub fn representations(&self) -> impl Iterator<Item = &LiveTransferRepresentationProjection> {
        self.representations.values()
    }

    /// One admitted representation's projection.
    #[must_use]
    pub fn projection(
        &self,
        plan: LiveTransferRepresentationPlan,
    ) -> Option<&LiveTransferRepresentationProjection> {
        self.representations.get(&plan)
    }
}

/// Plan the live-transfer target operation from one bound input.
///
/// The complete analysis runs first, including its own corruption-
/// resistant assembly validator; the §5 contract projections are
/// derived from the realization declarations the analysis retained and
/// checked against the values §5 fixes; the §8.2 filter groups the
/// retained feasible plans by the representation they selected for
/// `RECEIPT_L`; each group is projected onto the operation and required
/// to agree exactly; and the assembled plan is validated by an
/// independent re-derivation before it is returned. A caller receives a
/// complete validated plan or a typed failure. There is no partial
/// result and no route to the analyzed program itself.
///
/// # Errors
///
/// Any failure of the scoped analysis or its validator;
/// [`CompileError::TargetOperationOutOfScope`] when the bound scope does
/// not analyze `transfer-live-receipts`;
/// [`CompileError::LiveTransferContractDefect`] when a derived
/// projection and §5 disagree;
/// [`CompileError::LiveTransferDeferralDefect`] when a recorded deferral
/// is no longer derivable;
/// [`CompileError::MissingLiveTransferRepresentation`] when the analysis
/// retained no plan selecting an admitted representation;
/// [`CompileError::AmbiguousTargetOperationPlan`] when the plans of one
/// representation disagree about the operation;
/// [`CompileError::DuplicateTargetCoverageRequirement`] when one
/// coverage identity is claimed twice; and any defect the plan validator
/// raises.
pub fn plan_live_transfer_target_operation(
    input: &BoundCompilerInput,
    placement_limits: PlacementSearchLimits,
) -> Result<ValidatedLiveTransferOperationPlan, CompileError> {
    let analyzed = analyze_scoped_program(input, placement_limits)?;
    let plan = derive_live_transfer_plan(&analyzed)?;

    validate_live_transfer_plan(&analyzed, &plan)?;

    Ok(plan)
}

/// Derive the plan from a complete analyzed program.
///
/// Crate-private and reachable only below
/// [`plan_live_transfer_target_operation`], so the validated-analysis
/// precondition is a property of the call graph rather than a comment.
///
/// # Errors
///
/// The variants listed by [`plan_live_transfer_target_operation`].
pub(crate) fn derive_live_transfer_plan(
    analyzed: &ScopedAnalyzedProgram,
) -> Result<ValidatedLiveTransferOperationPlan, CompileError> {
    if !analyzed
        .source
        .compilation_scope
        .operations()
        .contains(&PLANNED)
    {
        return Err(CompileError::TargetOperationOutOfScope { operation: PLANNED });
    }

    let mut representations = BTreeMap::new();

    for plan in LiveTransferRepresentationPlan::ALL.iter().copied() {
        representations.insert(plan, derive_representation(analyzed, plan)?);
    }

    Ok(ValidatedLiveTransferOperationPlan {
        operation: PLANNED,
        source: TargetOperationSource::of(analyzed),
        representation: derive_representation_policy(analyzed)?,
        class: derive_class(analyzed)?,
        owner: derive_owner(analyzed)?,
        value: derive_value(analyzed)?,
        sponsor: derive_sponsor(analyzed)?,
        roots: derive_roots(analyzed)?,
        certificate: derive_certificate(analyzed)?,
        lifecycle: derive_lifecycle_closure(&representations)?,
        representations,
    })
}

// --- §5 contract projections ---

/// A typed contract defect naming the clause that failed.
const fn defect(clause: LiveTransferClause) -> CompileError {
    CompileError::LiveTransferContractDefect { clause }
}

/// The live-transfer relation declarations the analysis retained.
///
/// Read from the realization projection rather than from the compiler's
/// own restatement of it, for the reason the compact-ASH policy reads
/// its approved modes there: the declarations are the authored source,
/// and a derivation that read a restatement could never disagree with
/// the analysis.
fn declarations(analyzed: &ScopedAnalyzedProgram) -> impl Iterator<Item = &RelationDeclaration> {
    analyzed
        .source
        .realization
        .relations
        .nodes
        .iter()
        .filter(|declaration| declaration.id.operation() == PLANNED)
}

/// The one live-transfer declaration whose body the reader accepts.
///
/// Exactly one, in both directions. A missing declaration is a contract
/// the realization does not state; a second one is two authored answers
/// to a question §5 answers once, and picking either would make the
/// plan depend on declaration order.
///
/// # Errors
///
/// [`CompileError::LiveTransferContractDefect`] naming the clause.
fn sole<T>(
    analyzed: &ScopedAnalyzedProgram,
    clause: LiveTransferClause,
    read: impl Fn(&Relation) -> Option<T>,
) -> Result<(RelationId, T), CompileError> {
    let mut found = declarations(analyzed).filter_map(|declaration| {
        read(&declaration.relation).map(|value| (declaration.id.clone(), value))
    });

    let Some(first) = found.next() else {
        return Err(defect(clause));
    };

    if found.next().is_some() {
        return Err(defect(clause));
    }

    Ok(first)
}

/// The declared cardinality of one object family on one side.
///
/// # Errors
///
/// [`CompileError::LiveTransferContractDefect`] naming the clause.
fn cardinality(
    analyzed: &ScopedAnalyzedProgram,
    clause: LiveTransferClause,
    wanted_side: ObservedSide,
    wanted_object: ObjectId,
) -> Result<(RelationId, LiveTransferCardinality), CompileError> {
    sole(analyzed, clause, |relation| match relation {
        Relation::Cardinality {
            side,
            object,
            minimum,
            maximum,
        } if *side == wanted_side && *object == wanted_object => Some(LiveTransferCardinality {
            minimum: *minimum,
            maximum: *maximum,
        }),
        _ => None,
    })
}

/// The protocol recognition relation of one side (§5.1, §5.4).
///
/// # Errors
///
/// [`CompileError::LiveTransferContractDefect`] naming the clause.
fn recognition(
    analyzed: &ScopedAnalyzedProgram,
    clause: LiveTransferClause,
    wanted_side: ObservedSide,
    wanted_asset: AssetId,
) -> Result<(RelationId, ObjectId), CompileError> {
    sole(analyzed, clause, |relation| match relation {
        Relation::Recognition {
            side,
            object,
            asset,
        } if *side == wanted_side && *asset == wanted_asset => Some(*object),
        _ => None,
    })
}

/// The object families one side admits (§5.4).
///
/// # Errors
///
/// [`CompileError::LiveTransferContractDefect`] naming the clause.
fn closure(
    analyzed: &ScopedAnalyzedProgram,
    wanted_side: ObservedSide,
) -> Result<(RelationId, BTreeSet<ObjectId>), CompileError> {
    sole(
        analyzed,
        LiveTransferClause::ClassClosure,
        |relation| match relation {
            Relation::AllowedObjectFamilies { side, allowed } if *side == wanted_side => {
                Some(allowed.clone())
            }
            _ => None,
        },
    )
}

/// Derive the live-class closure (§5.4).
///
/// # Errors
///
/// [`CompileError::LiveTransferContractDefect`] naming the clause that
/// failed.
fn derive_class(
    analyzed: &ScopedAnalyzedProgram,
) -> Result<LiveTransferClassProjection, CompileError> {
    let clause = LiveTransferClause::ProtocolObject;
    let (input_recognition, input_object) =
        recognition(analyzed, clause, ObservedSide::Input, CLOSED_ASSET)?;
    let (output_recognition, output_object) =
        recognition(analyzed, clause, ObservedSide::Output, CLOSED_ASSET)?;

    if input_object != RECEIPT_L || output_object != RECEIPT_L {
        return Err(defect(clause));
    }

    // The sponsor recognition relation is read for its object family
    // alone. The class projection publishes the protocol relations and
    // the sponsor projection owns the sponsor ones, so binding the
    // identity here would put one relation in two published places.
    let (_, sponsor_object) = recognition(
        analyzed,
        LiveTransferClause::SponsorObject,
        ObservedSide::Input,
        SPONSOR_ASSET,
    )?;

    if sponsor_object != ORDINARY_LBTC {
        return Err(defect(LiveTransferClause::SponsorObject));
    }

    let (input_closure, input_families) = closure(analyzed, ObservedSide::Input)?;
    let (output_closure, output_families) = closure(analyzed, ObservedSide::Output)?;
    let admitted = BTreeSet::from([RECEIPT_L, sponsor_object]);

    if input_families != admitted || output_families != admitted {
        return Err(defect(LiveTransferClause::ClassClosure));
    }

    Ok(LiveTransferClassProjection {
        protocol: RECEIPT_L,
        sponsor: sponsor_object,
        forbidden: ObjectId::ALL
            .iter()
            .copied()
            .filter(|object| !admitted.contains(object))
            .collect(),
        admitted,
        input_closure,
        output_closure,
        input_recognition,
        output_recognition,
    })
}

/// Derive the owner-authorization closure (§5.5, §8.2).
///
/// # Errors
///
/// [`CompileError::LiveTransferContractDefect`] naming the clause that
/// failed.
fn derive_owner(
    analyzed: &ScopedAnalyzedProgram,
) -> Result<LiveTransferOwnerProjection, CompileError> {
    let (authorization, object) = sole(
        analyzed,
        LiveTransferClause::OwnerAuthorization,
        |relation| match relation {
            Relation::OwnerAuthorization { object } => Some(*object),
            _ => None,
        },
    )?;

    if object != RECEIPT_L {
        return Err(defect(LiveTransferClause::OwnerAuthorization));
    }

    let (constructibility, class) = sole(
        analyzed,
        LiveTransferClause::Constructibility,
        |relation| match relation {
            Relation::Constructibility { class } => Some(*class),
            _ => None,
        },
    )?;

    if class != (ConstructibilityClass::OwnersOf { object: RECEIPT_L }) {
        return Err(defect(LiveTransferClause::Constructibility));
    }

    Ok(LiveTransferOwnerProjection {
        object,
        class,
        authorization,
        constructibility,
    })
}

/// Derive the value and partition contract (§5.1, §5.3, §5.6).
///
/// # Errors
///
/// [`CompileError::LiveTransferContractDefect`] naming the clause that
/// failed.
fn derive_value(
    analyzed: &ScopedAnalyzedProgram,
) -> Result<LiveTransferValueProjection, CompileError> {
    let (conservation, (asset, input_objects, output_objects)) = sole(
        analyzed,
        LiveTransferClause::ValueConservation,
        |relation| match relation {
            Relation::AmountConservation {
                asset,
                input_objects,
                output_objects,
            } if *asset == CLOSED_ASSET => {
                Some((*asset, input_objects.clone(), output_objects.clone()))
            }
            _ => None,
        },
    )?;

    let protocol = BTreeSet::from([RECEIPT_L]);

    if input_objects != protocol || output_objects != protocol {
        return Err(defect(LiveTransferClause::ValueConservation));
    }

    let (canonical_flow, flow) =
        sole(
            analyzed,
            LiveTransferClause::CanonicalFlow,
            |relation| match relation {
                Relation::CanonicalDeltaPolicy { expected } => Some(expected.clone()),
                _ => None,
            },
        )?;
    let expected_flow = BTreeSet::from([ExpectedCanonicalDelta {
        asset: CLOSED_ASSET,
        kind: DeltaKind::Lateral,
        destruction_tag: None,
    }]);

    if flow != expected_flow {
        return Err(defect(LiveTransferClause::CanonicalFlow));
    }

    let (input_cardinality_relation, input_cardinality) = cardinality(
        analyzed,
        LiveTransferClause::InputCardinality,
        ObservedSide::Input,
        RECEIPT_L,
    )?;
    let (output_cardinality_relation, output_cardinality) = cardinality(
        analyzed,
        LiveTransferClause::OutputCardinality,
        ObservedSide::Output,
        RECEIPT_L,
    )?;

    // §5.1: at least one receipt on each side. The maxima stay in the
    // architecture-owned form the declaration carries.
    if input_cardinality.minimum != Count::ONE {
        return Err(defect(LiveTransferClause::InputCardinality));
    }

    if output_cardinality.minimum != Count::ONE {
        return Err(defect(LiveTransferClause::OutputCardinality));
    }

    Ok(LiveTransferValueProjection {
        asset,
        input_objects,
        output_objects,
        input_cardinality,
        output_cardinality,
        flow,
        conservation,
        canonical_flow,
        input_cardinality_relation,
        output_cardinality_relation,
    })
}

/// Derive the sponsor contract (§1.9, §5.8).
///
/// # Errors
///
/// [`CompileError::LiveTransferContractDefect`] naming the clause that
/// failed.
fn derive_sponsor(
    analyzed: &ScopedAnalyzedProgram,
) -> Result<LiveTransferSponsorProjection, CompileError> {
    let (open_flow_policy, open_flows) = sole(
        analyzed,
        LiveTransferClause::SponsorFlow,
        |relation| match relation {
            Relation::OpenFlowPolicy { allowed } => Some(allowed.clone()),
            _ => None,
        },
    )?;

    // §5.8: the fee sponsor is the *only* open flow, so ordinary L-BTC
    // is exactly the sponsor region and no protocol flow claims it.
    if open_flows != BTreeSet::from([OpenFlowKind::FeeSponsor]) {
        return Err(defect(LiveTransferClause::SponsorFlow));
    }

    let region = ordinary_lbtc_role_of(&open_flows);

    if region != OrdinaryLbtcRole::SponsorRegion {
        return Err(defect(LiveTransferClause::SponsorFlow));
    }

    let (isolation, ()) =
        sole(
            analyzed,
            LiveTransferClause::SponsorIsolation,
            |relation| match relation {
                Relation::SponsorIsolation => Some(()),
                _ => None,
            },
        )?;
    let (multiplicity, envelope_maximum) = sole(
        analyzed,
        LiveTransferClause::SponsorEnvelope,
        |relation| match relation {
            Relation::SponsorEnvelopeMultiplicity { maximum } => Some(*maximum),
            _ => None,
        },
    )?;

    if envelope_maximum != Count::ONE {
        return Err(defect(LiveTransferClause::SponsorEnvelope));
    }

    let (substrate_conservation, substrate_asset) = sole(
        analyzed,
        LiveTransferClause::SubstrateConservation,
        |relation| match relation {
            Relation::SubstrateConservation { asset } => Some(*asset),
            _ => None,
        },
    )?;

    if substrate_asset != SPONSOR_ASSET {
        return Err(defect(LiveTransferClause::SubstrateConservation));
    }

    let (_, input_cardinality) = cardinality(
        analyzed,
        LiveTransferClause::SponsorInputs,
        ObservedSide::Input,
        ORDINARY_LBTC,
    )?;
    let (_, change_cardinality) = cardinality(
        analyzed,
        LiveTransferClause::SponsorChange,
        ObservedSide::Output,
        ORDINARY_LBTC,
    )?;

    // §5.8: the envelope is optional, and its change is one output at
    // most. Both are the *presence* of the region, never an amount.
    if !input_cardinality.is_optional() {
        return Err(defect(LiveTransferClause::SponsorInputs));
    }

    if !change_cardinality.is_optional()
        || change_cardinality.maximum != CardinalityMaximum::Exact(Count::ONE)
    {
        return Err(defect(LiveTransferClause::SponsorChange));
    }

    Ok(LiveTransferSponsorProjection {
        object: ORDINARY_LBTC,
        region,
        open_flows,
        envelope_maximum,
        input_cardinality,
        change_cardinality,
        isolation,
        multiplicity,
        open_flow_policy,
        substrate_conservation,
    })
}

/// Derive the architecture-root contract (§5.7).
///
/// # Errors
///
/// [`CompileError::LiveTransferContractDefect`] naming the clause that
/// failed.
fn derive_roots(
    analyzed: &ScopedAnalyzedProgram,
) -> Result<LiveTransferRootProjection, CompileError> {
    let (relation, expected) =
        sole(
            analyzed,
            LiveTransferClause::RootPolicy,
            |relation| match relation {
                Relation::RootPolicy { expected } => Some(expected.clone()),
                _ => None,
            },
        )?;
    let projection = LiveTransferRootProjection { expected, relation };

    if !projection.forbids_every_root() {
        return Err(defect(LiveTransferClause::RootPolicy));
    }

    Ok(projection)
}

/// Derive the semantic-projection contract (§5.7).
///
/// # Errors
///
/// [`CompileError::LiveTransferContractDefect`] naming the clause that
/// failed.
fn derive_certificate(
    analyzed: &ScopedAnalyzedProgram,
) -> Result<LiveTransferCertificateProjection, CompileError> {
    let (relation, expected) = sole(
        analyzed,
        LiveTransferClause::CertificateProjection,
        |relation| match relation {
            Relation::ProjectionPolicy { expected } => Some(expected.clone()),
            _ => None,
        },
    )?;

    // §5.7 over the complete projection census: the transition
    // certificate is required and every specialized projection is
    // forbidden. A projection the policy omitted fails here rather than
    // defaulting to permitted.
    let contract = ProjectionId::ALL.iter().all(|projection| {
        let rule = if *projection == REQUIRED_PROJECTION {
            ProjectionRule::Required
        } else {
            ProjectionRule::Forbidden
        };

        expected.get(projection) == Some(&rule)
    });

    if !contract {
        return Err(defect(LiveTransferClause::CertificateProjection));
    }

    Ok(LiveTransferCertificateProjection { expected, relation })
}

/// Derive the admitted and deferred representations (§6.1, §6.5).
///
/// # Errors
///
/// [`CompileError::LiveTransferContractDefect`] when the approved set is
/// not the admitted one; [`CompileError::LiveTransferDeferralDefect`]
/// when a recorded deferral no longer holds.
fn derive_representation_policy(
    analyzed: &ScopedAnalyzedProgram,
) -> Result<LiveTransferRepresentationPolicy, CompileError> {
    let (relation, approved) = sole(
        analyzed,
        LiveTransferClause::RepresentationApproval,
        |relation| match relation {
            Relation::Representation { object, allowed } if *object == RECEIPT_L => {
                Some(allowed.clone())
            }
            _ => None,
        },
    )?;

    let admitted = LiveTransferRepresentationPlan::ALL
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();

    // §6.1: the admitted plans are exactly the realization's approved
    // modes. An approved mode with no admitted plan would be a silent
    // narrowing; an admitted plan the realization does not approve would
    // be a plan for a representation nothing declared.
    if approved != admitted.iter().map(|plan| plan.mode()).collect() {
        return Err(defect(LiveTransferClause::RepresentationApproval));
    }

    let mut deferred = BTreeMap::new();

    for representation in DeferredRepresentation::ALL.iter().copied() {
        let ground = representation.ground();
        let holds = match ground {
            // The mode the vocabulary names and this operation does not
            // approve. A realization that approved it would need a
            // Guide-13 decision, not a deferral that quietly stopped
            // being true.
            RepresentationDeferralGround::UnapprovedMode => {
                !approved.contains(&RepresentationMode::PublicCommitted)
            }
            // One representation variable for the object family, so no
            // per-reference mode exists for a transfer to mix. `sole`
            // above already refused a second variable; this states the
            // consequence the deferral rests on.
            RepresentationDeferralGround::NoPerReferenceVariable => {
                declarations(analyzed)
                    .filter(|declaration| {
                        matches!(
                            declaration.relation,
                            Relation::Representation { object, .. } if object == RECEIPT_L
                        )
                    })
                    .count()
                    == 1
            }
        };

        if !holds {
            return Err(CompileError::LiveTransferDeferralDefect { representation });
        }

        deferred.insert(representation, ground);
    }

    Ok(LiveTransferRepresentationPolicy {
        approved,
        admitted,
        deferred,
        relation,
    })
}

/// Derive the lifecycle-exit closure from the representations (§8.2).
///
/// # Errors
///
/// [`CompileError::TargetPlanLifecycleMismatch`] when the
/// representations disagree about the exits;
/// [`CompileError::LiveTransferContractDefect`] when the exits are not
/// the ones §8.2 fixes.
fn derive_lifecycle_closure(
    representations: &BTreeMap<
        LiveTransferRepresentationPlan,
        LiveTransferRepresentationProjection,
    >,
) -> Result<LiveTransferLifecycleClosure, CompileError> {
    let mut closures = representations
        .values()
        .map(|projection| exit_closure(&projection.lifecycle));

    let Some(closure) = closures.next() else {
        return Err(CompileError::TargetPlanLifecycleMismatch);
    };

    if closures.any(|other| other != closure) {
        return Err(CompileError::TargetPlanLifecycleMismatch);
    }

    // §8.2: transfer implemented, burn and redeem outstanding.
    if closure.implemented != BTreeSet::from([IMPLEMENTED_EXIT])
        || closure.outstanding != OUTSTANDING_EXITS.into_iter().collect()
    {
        return Err(defect(LiveTransferClause::LifecycleExits));
    }

    Ok(closure)
}

/// The exits one representation's lifecycle rows name.
fn exit_closure(status: &TargetLifecycleStatus) -> LiveTransferLifecycleClosure {
    LiveTransferLifecycleClosure {
        implemented: status.implemented().collect(),
        outstanding: status
            .outstanding()
            .map(|requirement| requirement.exit)
            .collect(),
    }
}

// --- per-representation projections ---

/// The analyzed factors of every plan selecting one representation.
fn factors(
    analyzed: &ScopedAnalyzedProgram,
    representation: LiveTransferRepresentationPlan,
) -> Vec<(&AnalyzedProofPlan, &AnalyzedOperation)> {
    analyzed
        .proof_plans
        .values()
        .filter(|plan| selects(plan, representation))
        .filter_map(|plan| plan.operations.get(&PLANNED).map(|factor| (plan, factor)))
        .collect()
}

/// Whether one analyzed plan selects a given live-receipt
/// representation.
///
/// The choice variable is read by its typed identity — the planned
/// operation and the live-receipt family — never by scanning for a mode
/// some other object family happened to select.
fn selects(plan: &AnalyzedProofPlan, representation: LiveTransferRepresentationPlan) -> bool {
    plan.proof_plan
        .representations
        .get(&RepresentationChoiceId {
            operation: PLANNED,
            object: RECEIPT_L,
        })
        .is_some_and(|mode| *mode == representation.mode())
}

/// Project one representation from every plan that selected it.
///
/// The §8.2 filter is stated over the whole scope's plans, and the plans
/// of a multi-operation scope also differ in choices this operation does
/// not make. Projecting each survivor onto the operation and requiring
/// exact agreement is stronger than a tie-break: it proves the
/// projection is a property of the operation and its representation
/// rather than of which alternative happened to be chosen.
///
/// # Errors
///
/// [`CompileError::MissingLiveTransferRepresentation`],
/// [`CompileError::AmbiguousTargetOperationPlan`], or any defect raised
/// while projecting one factor.
fn derive_representation(
    analyzed: &ScopedAnalyzedProgram,
    representation: LiveTransferRepresentationPlan,
) -> Result<LiveTransferRepresentationProjection, CompileError> {
    let mut derived = Vec::new();

    for (plan, factor) in factors(analyzed, representation) {
        let candidate = project_factor(representation, plan, factor)?;

        if !derived.contains(&candidate) {
            derived.push(candidate);
        }
    }

    let mut derived = derived.into_iter();
    let projection = derived
        .next()
        .ok_or(CompileError::MissingLiveTransferRepresentation {
            operation: PLANNED,
            representation,
        })?;

    if derived.next().is_some() {
        return Err(CompileError::AmbiguousTargetOperationPlan { operation: PLANNED });
    }

    Ok(projection)
}

/// Project one analyzed factor onto one representation's censuses.
///
/// # Errors
///
/// [`CompileError::TargetPlanRelationCensusMismatch`],
/// [`CompileError::DuplicateTargetCoverageRequirement`], or a census
/// defect from [`canonical_census`].
fn project_factor(
    representation: LiveTransferRepresentationPlan,
    plan: &AnalyzedProofPlan,
    factor: &AnalyzedOperation,
) -> Result<LiveTransferRepresentationProjection, CompileError> {
    let relations = operation_relations(factor);
    let relation_requirements = project_relations(plan, factor, &relations)?;

    Ok(LiveTransferRepresentationProjection {
        plan: representation,
        cases: project_cases(factor),
        carriers: project_carriers(factor),
        layout: factor.layout_requirements.clone(),
        coverage: project_coverage(plan, factor)?,
        capabilities: capability_census(&relation_requirements)?,
        external_evidence: evidence_census(&relation_requirements)?,
        lifecycle: project_lifecycle(plan, &relations),
        relations: relation_requirements,
    })
}

/// The capability census one relation set owns, in canonical order.
///
/// # Errors
///
/// [`CompileError::NoncanonicalCapabilityCensus`].
fn capability_census(
    relations: &BTreeMap<RelationId, TargetRelationRequirement>,
) -> Result<BTreeSet<RequiredCapability>, CompileError> {
    let present = relations
        .values()
        .flat_map(|requirement| requirement.required_capabilities.iter().copied())
        .collect::<BTreeSet<_>>();

    canonical_census(&present, RequiredCapability::ALL, |capability| {
        CompileError::NoncanonicalCapabilityCensus { capability }
    })
    .map(|census| census.into_iter().collect())
}

/// The evidence-role census one relation set owns, in canonical order.
///
/// # Errors
///
/// [`CompileError::NoncanonicalEvidenceRoleCensus`].
fn evidence_census(
    relations: &BTreeMap<RelationId, TargetRelationRequirement>,
) -> Result<BTreeSet<ExternalEvidenceRole>, CompileError> {
    let present = relations
        .values()
        .flat_map(|requirement| requirement.external_evidence.iter())
        .map(ExternalEvidenceRole::of)
        .collect::<BTreeSet<_>>();

    canonical_census(&present, ExternalEvidenceRole::ALL, |role| {
        CompileError::NoncanonicalEvidenceRoleCensus { role }
    })
    .map(|census| census.into_iter().collect())
}

// --- the independent assembly validator (§8.3) ---

/// Validate one assembled plan against the analyzed program (§8.3).
///
/// The plan is not trusted because its assembler produced it. Every
/// census it claims is re-derived here from the analyzed program
/// directly and compared for exact equality in both directions; every
/// §5 projection is re-derived from the realization declarations and
/// compared; the cross-representation agreement §6.6 requires and the
/// divergence §19.4 requires are both checked; and no published field
/// anywhere may name an erased sponsor amount.
///
/// Re-derivation reads the analyzed program rather than the plan's own
/// construction, which is what makes this a check and not a restatement.
///
/// # Errors
///
/// [`CompileError::TargetOperationOutOfScope`],
/// [`CompileError::TargetPlanSourceMismatch`],
/// [`CompileError::LiveTransferContractDefect`],
/// [`CompileError::LiveTransferDeferralDefect`],
/// [`CompileError::LiveTransferRepresentationDivergence`],
/// [`CompileError::MissingLiveTransferRepresentation`], the
/// `TargetPlan*` census-mismatch variants, and
/// [`CompileError::SponsorValueRead`] when a published field names an
/// erased sponsor amount.
pub(crate) fn validate_live_transfer_plan(
    analyzed: &ScopedAnalyzedProgram,
    plan: &ValidatedLiveTransferOperationPlan,
) -> Result<(), CompileError> {
    if plan.operation != PLANNED {
        return Err(CompileError::TargetOperationOutOfScope {
            operation: plan.operation,
        });
    }

    if !plan.source.is_analyzed_source(analyzed) {
        return Err(CompileError::TargetPlanSourceMismatch);
    }

    // §5: every contract projection is the one the declarations state.
    if plan.representation != derive_representation_policy(analyzed)? {
        return Err(defect(LiveTransferClause::RepresentationApproval));
    }

    if plan.class != derive_class(analyzed)? {
        return Err(defect(LiveTransferClause::ClassClosure));
    }

    if plan.owner != derive_owner(analyzed)? {
        return Err(defect(LiveTransferClause::OwnerAuthorization));
    }

    if plan.value != derive_value(analyzed)? {
        return Err(defect(LiveTransferClause::ValueConservation));
    }

    if plan.sponsor != derive_sponsor(analyzed)? {
        return Err(defect(LiveTransferClause::SponsorIsolation));
    }

    if plan.roots != derive_roots(analyzed)? {
        return Err(defect(LiveTransferClause::RootPolicy));
    }

    if plan.certificate != derive_certificate(analyzed)? {
        return Err(defect(LiveTransferClause::CertificateProjection));
    }

    // §8.2: exactly the admitted representations, each agreeing with
    // every analyzed factor that selected it.
    let admitted = LiveTransferRepresentationPlan::ALL
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();

    if plan
        .representations
        .keys()
        .copied()
        .collect::<BTreeSet<_>>()
        != admitted
    {
        return Err(defect(LiveTransferClause::RepresentationApproval));
    }

    for (representation, projection) in &plan.representations {
        if projection.plan != *representation {
            return Err(defect(LiveTransferClause::RepresentationApproval));
        }

        let factors = factors(analyzed, *representation);

        if factors.is_empty() {
            return Err(CompileError::MissingLiveTransferRepresentation {
                operation: PLANNED,
                representation: *representation,
            });
        }

        for (candidate, factor) in factors {
            validate_against_factor(projection, candidate, factor)?;
        }
    }

    if plan.lifecycle != derive_lifecycle_closure(&plan.representations)? {
        return Err(CompileError::TargetPlanLifecycleMismatch);
    }

    validate_representation_equivalence(plan)?;
    validate_sponsor_erasure(plan)
}

/// Compare one representation's projection against one analyzed factor.
///
/// Both directions of every equality are defects: a missing member is a
/// completeness defect and an unexpected one is an unowned requirement.
///
/// # Errors
///
/// The census-mismatch variants listed by
/// [`validate_live_transfer_plan`].
fn validate_against_factor(
    projection: &LiveTransferRepresentationProjection,
    candidate: &AnalyzedProofPlan,
    factor: &AnalyzedOperation,
) -> Result<(), CompileError> {
    let relations = operation_relations(factor);
    let published = projection
        .relations
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();

    if let Some(relation) = relations
        .difference(&published)
        .chain(published.difference(&relations))
        .next()
    {
        return Err(CompileError::TargetPlanRelationCensusMismatch {
            relation: relation.clone(),
        });
    }

    if projection.cases.keys().cloned().collect::<BTreeSet<_>>() != factor.execution_cases
        || project_cases(factor) != projection.cases
    {
        return Err(CompileError::TargetPlanCaseCensusMismatch);
    }

    if project_carriers(factor) != projection.carriers {
        return Err(CompileError::TargetPlanCarrierCensusMismatch);
    }

    if factor.layout_requirements != projection.layout {
        return Err(CompileError::TargetPlanLayoutCensusMismatch);
    }

    let requirements = project_relations(candidate, factor, &relations)?;

    if requirements != projection.relations {
        return Err(CompileError::TargetPlanRelationRequirementMismatch);
    }

    if project_coverage(candidate, factor)? != projection.coverage {
        return Err(CompileError::TargetPlanCoverageCensusMismatch);
    }

    // §8.3: an omitted capability and an omitted evidence role are both
    // rejections. The aggregates are re-derived from the relations that
    // own them rather than trusted.
    if capability_census(&requirements)? != projection.capabilities {
        return Err(CompileError::TargetPlanCapabilityCensusMismatch);
    }

    if evidence_census(&requirements)? != projection.external_evidence {
        return Err(CompileError::TargetPlanEvidenceCensusMismatch);
    }

    if project_lifecycle(candidate, &relations) != projection.lifecycle {
        return Err(CompileError::TargetPlanLifecycleMismatch);
    }

    Ok(())
}

/// Check what §6.6 requires equal and what §19.4 requires different.
///
/// The explicit projection is the reference and every other admitted
/// representation is compared against it. Equal: the relation census,
/// because §6.6's whole agreement list is carried by relations and
/// §1.2 lets no relation disappear. Different: exactly the
/// value-conservation relation, in exactly the direction §19.4 fixes.
///
/// # Errors
///
/// [`CompileError::LiveTransferRepresentationDivergence`] when a
/// relation other than value conservation diverges;
/// [`CompileError::LiveTransferContractDefect`] when value conservation
/// does not diverge, or diverges the wrong way.
pub(crate) fn validate_representation_equivalence(
    plan: &ValidatedLiveTransferOperationPlan,
) -> Result<(), CompileError> {
    let conservation = plan.value.conservation.clone();
    let reference = plan
        .representations
        .get(&LiveTransferRepresentationPlan::Explicit)
        .ok_or(CompileError::MissingLiveTransferRepresentation {
            operation: PLANNED,
            representation: LiveTransferRepresentationPlan::Explicit,
        })?;

    for (representation, projection) in &plan.representations {
        // §19.4: each representation's conservation proof requires its
        // own capability and not the other's.
        let capability = representation.conservation_capability();
        let owned = projection
            .relations
            .get(&conservation)
            .ok_or(defect(LiveTransferClause::ValueConservation))?;

        if !owned.required_capabilities.contains(&capability)
            || LiveTransferRepresentationPlan::ALL
                .iter()
                .filter(|other| *other != representation)
                .any(|other| {
                    owned
                        .required_capabilities
                        .contains(&other.conservation_capability())
                })
        {
            return Err(defect(LiveTransferClause::ValueConservation));
        }

        if *representation == LiveTransferRepresentationPlan::Explicit {
            continue;
        }

        // §6.6, §1.2: the relation censuses are equal.
        let published = projection.relations.keys().collect::<BTreeSet<_>>();
        let expected = reference.relations.keys().collect::<BTreeSet<_>>();

        if let Some(relation) = expected.symmetric_difference(&published).next() {
            return Err(CompileError::TargetPlanRelationCensusMismatch {
                relation: (*relation).clone(),
            });
        }

        // §6.6: exactly one relation may differ, and §19.4 names it.
        for (relation, requirement) in &projection.relations {
            let other = reference.relations.get(relation).ok_or_else(|| {
                CompileError::TargetPlanRelationCensusMismatch {
                    relation: relation.clone(),
                }
            })?;
            let diverges = requirement.proof != other.proof
                || requirement.required_capabilities != other.required_capabilities;

            if diverges != (*relation == conservation) {
                return Err(CompileError::LiveTransferRepresentationDivergence {
                    relation: relation.clone(),
                });
            }
        }
    }

    Ok(())
}

/// Refuse any published field naming an erased sponsor amount (§1.9).
///
/// Every published layout requirement, every relation-case layout
/// requirement, and every carrier alternative's layout, under every
/// admitted representation. The traversal is structural rather than
/// textual: an amount operand over the erased sponsor family is matched
/// by name wherever it is published from.
///
/// # Errors
///
/// [`CompileError::SponsorValueRead`].
fn validate_sponsor_erasure(plan: &ValidatedLiveTransferOperationPlan) -> Result<(), CompileError> {
    let mut published = plan.representations.values().flat_map(|projection| {
        projection
            .layout
            .iter()
            .chain(projection.relations.values().flat_map(|relation| {
                relation
                    .cases
                    .values()
                    .flat_map(|case| case.layout_requirements.iter())
            }))
            .chain(
                projection
                    .carriers
                    .iter()
                    .flat_map(|carrier| carrier.alternatives.iter())
                    .flat_map(|alternative| alternative.layout.iter()),
            )
    });

    if published.any(names_sponsor_amount) {
        return Err(CompileError::SponsorValueRead);
    }

    Ok(())
}

/// Test-only corruption handles.
///
/// The published fields are private and there is no public route to
/// them, which is the property §8.3 requires. The corruption oracles
/// nevertheless have to damage a validated plan the way a defect in the
/// join could, so the handles exist for tests alone and are compiled out
/// of every other build.
#[cfg(test)]
impl ValidatedLiveTransferOperationPlan {
    pub(crate) const fn source_mut(&mut self) -> &mut TargetOperationSource {
        &mut self.source
    }

    pub(crate) const fn representation_mut(&mut self) -> &mut LiveTransferRepresentationPolicy {
        &mut self.representation
    }

    pub(crate) const fn class_mut(&mut self) -> &mut LiveTransferClassProjection {
        &mut self.class
    }

    pub(crate) const fn owner_mut(&mut self) -> &mut LiveTransferOwnerProjection {
        &mut self.owner
    }

    pub(crate) const fn value_mut(&mut self) -> &mut LiveTransferValueProjection {
        &mut self.value
    }

    pub(crate) const fn sponsor_mut(&mut self) -> &mut LiveTransferSponsorProjection {
        &mut self.sponsor
    }

    pub(crate) const fn roots_mut(&mut self) -> &mut LiveTransferRootProjection {
        &mut self.roots
    }

    pub(crate) const fn certificate_mut(&mut self) -> &mut LiveTransferCertificateProjection {
        &mut self.certificate
    }

    pub(crate) const fn lifecycle_mut(&mut self) -> &mut LiveTransferLifecycleClosure {
        &mut self.lifecycle
    }

    pub(crate) const fn representations_mut(
        &mut self,
    ) -> &mut BTreeMap<LiveTransferRepresentationPlan, LiveTransferRepresentationProjection> {
        &mut self.representations
    }
}

#[cfg(test)]
impl LiveTransferRepresentationProjection {
    pub(crate) const fn relations_mut(
        &mut self,
    ) -> &mut BTreeMap<RelationId, TargetRelationRequirement> {
        &mut self.relations
    }

    pub(crate) const fn cases_mut(
        &mut self,
    ) -> &mut BTreeMap<ExecutionCaseId, TargetExecutionCase> {
        &mut self.cases
    }

    pub(crate) const fn carriers_mut(&mut self) -> &mut BTreeSet<AbstractCarrierRequirement> {
        &mut self.carriers
    }

    pub(crate) const fn layout_mut(&mut self) -> &mut BTreeSet<LayoutRequirement> {
        &mut self.layout
    }

    pub(crate) const fn coverage_mut(
        &mut self,
    ) -> &mut BTreeMap<CoverageRequirementId, TargetCoverageRequirement> {
        &mut self.coverage
    }

    pub(crate) const fn capabilities_mut(&mut self) -> &mut BTreeSet<RequiredCapability> {
        &mut self.capabilities
    }

    pub(crate) const fn external_evidence_mut(&mut self) -> &mut BTreeSet<ExternalEvidenceRole> {
        &mut self.external_evidence
    }
}

#[cfg(test)]
impl LiveTransferRepresentationPolicy {
    pub(crate) const fn deferred_mut(
        &mut self,
    ) -> &mut BTreeMap<DeferredRepresentation, RepresentationDeferralGround> {
        &mut self.deferred
    }
}

#[cfg(test)]
impl LiveTransferClassProjection {
    pub(crate) const fn forbidden_mut(&mut self) -> &mut BTreeSet<ObjectId> {
        &mut self.forbidden
    }
}

#[cfg(test)]
impl LiveTransferOwnerProjection {
    pub(crate) const fn constructibility_mut(&mut self) -> &mut RelationId {
        &mut self.constructibility
    }
}

#[cfg(test)]
impl LiveTransferValueProjection {
    pub(crate) const fn flow_mut(&mut self) -> &mut BTreeSet<ExpectedCanonicalDelta> {
        &mut self.flow
    }
}

#[cfg(test)]
impl LiveTransferSponsorProjection {
    pub(crate) const fn open_flows_mut(&mut self) -> &mut BTreeSet<OpenFlowKind> {
        &mut self.open_flows
    }
}

#[cfg(test)]
impl LiveTransferRootProjection {
    pub(crate) const fn expected_mut(&mut self) -> &mut BTreeMap<RootId, RootUse> {
        &mut self.expected
    }
}

#[cfg(test)]
impl LiveTransferCertificateProjection {
    pub(crate) const fn expected_mut(&mut self) -> &mut BTreeMap<ProjectionId, ProjectionRule> {
        &mut self.expected
    }
}

#[cfg(test)]
impl LiveTransferLifecycleClosure {
    pub(crate) fn clear_outstanding(&mut self) {
        self.outstanding.clear();
    }
}
