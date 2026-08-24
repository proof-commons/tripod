//! The candidate linked live-transfer bundle (§11.6), and the link that
//! produces one.
//!
//! # A candidate, held there by four separate facts
//!
//! §1.12 keeps the candidate and final states distinct, and this type
//! stays on the candidate side without relying on anyone reading a
//! comment:
//!
//! - [`crate::LinkedArtifactStatus`] is read, never written. There is no
//!   field, argument, or setter through which a caller could claim
//!   anything but [`crate::LinkedArtifactStatus::Prototype`];
//! - the outstanding burn and redemption lifecycle is carried through
//!   from the constructor, where its outstanding count is a
//!   [`NonZeroUsize`], so a linked bundle whose lifecycle was complete
//!   has no representation;
//! - [`OutstandingLiveLinkObligations`] is a second structurally
//!   non-empty set: a linked live bundle always owes at least the taproot
//!   output key, because this wave computes no hash and no tweak;
//! - the residuals the emitted bundles carry are carried on, unchanged
//!   and unsummarized.
//!
//! # No digest, and no room for one
//!
//! §1.13 admits a digest only once a real consumer of one appears, and
//! forbids a candidate type reserving a field for a future one. There is
//! no bundle hash here, no leaf hash, no tree hash, no constructor hash,
//! and no field that would hold one. `LiveTransferBundleHash` and
//! `LiveReceiptConstructorHash` are both among the identities §1.13 mints
//! none of, and both remain unminted.
//!
//! # The §10.4 induction, and exactly where it now stands
//!
//! This is the wave the induction was waiting on, so it is worth being
//! precise about what moved and what did not.
//!
//! §10.4's step says every output of an accepted transfer that carries
//! the protocol asset is a live receipt under this family's constructor
//! for a canonical owner. What the emitted programs establish is weaker:
//! the destination's asset and the version its program is read at.
//! [`tapscript::RecognitionResidual::LinkedDestinationConstructorIdentity`]
//! is the gap, and its stated reason had three parts — a destination's
//! constructor is owner-parameterized, so §11.4's deterministic tree and
//! §12's destination table were both owed, and then a target-native run.
//!
//! The first is now discharged, and discharged *concretely*:
//! [`link_live_candidate`] produces, for each (owner, representation),
//! the exact linked leaf programs and the deterministic tree over them.
//! That committed tree is the one a destination's version check points
//! at — [`LinkedLiveConstructor`] is it, as a typed value, and
//! [`LiveInductionStep`] is where the link says so rather than leaving a
//! reader to infer it.
//!
//! The other two are not, and the honest statement is that they are two
//! rather than one:
//!
//! - the taproot output key is still uncomputed. §1.13 mints no identity
//!   before a consumer exists, so this wave settles the tree's *shape*
//!   and leaves the merkle root, the tweak, and the output key to the
//!   layer that builds a transaction against a real deployment
//!   ([`LiveLinkObligation::TaprootOutputKeyUndischarged`]);
//! - §12.3 chooses each destination's owner per request, so which
//!   linked constructor a given output position is under is the ABI's
//!   statement and not this one
//!   ([`LiveLinkObligation::DestinationConstructorTableUndischarged`]).
//!
//! So the residual is re-scoped rather than cleared, and it is re-scoped
//! on both ends: the tapscript census still carries it because a
//! *program* still cannot establish it, and this crate now says which
//! part of the reason has stopped applying. A wave that had cleared it
//! here would have claimed the destination table and the output key
//! along with the tree.

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroUsize;

use tapscript::upstream::{
    ExternalEvidenceRole, LiveTransferRepresentationPlan, ValidatedLiveTransferOperationPlan,
};
use tapscript::{
    BackendArtifactStatus, CandidateRelocatableLiveTransferBundle, CandidateTransferLifecycle,
    CompleteFamilyRanges, ConstructorAssumption, InternalKeyPolicy, KeyPathPolicy,
    LiveBundleSymbol, LiveProgramRole, LiveTransferLeafRole, LiveTransferShape,
    LiveTransferShapeBounds, LiveTransferShapeSet, RecognitionResidual, StackItem,
    WitnessComponent,
};
use target_elements::{
    LeafVersion, ResourceDimension, ReviewedElementsTapscriptDefinition, TargetContractVersion,
    TargetEvidenceRequirementId,
};

use crate::bundle::LinkedArtifactStatus;
use crate::error::LinkRefusal;
use crate::live_carrier::{LiveCarrierClosure, close_live};
use crate::live_deployment::LiveLinkDeploymentParameters;
use crate::live_relocate::{LinkedLiveLeafProgram, substitute_live};
use crate::live_resource::{LinkedLiveResourceFormula, linked_formula};
use crate::live_symbol::{
    LinkedConstructorPlacement, LiveDefinitionCensus, LiveDefinitionOrigin, LiveLinkSymbol,
    LiveSymbolValue, OwnerParameter, SelectedSighashProfile, collect_live_definitions,
    link_role_defects,
};
use crate::live_taptree::{assemble_live, live_taptree_input};
use crate::taptree::DeterministicTaptree;

/// One obligation a live link creates and does not discharge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum LiveLinkObligation {
    /// The taptree's merkle root, the internal-key tweak, and the
    /// resulting taproot output key are not computed here.
    ///
    /// §11.4 states the tree input and the exact comparison against an
    /// independent optimum, both of which are structure; the hashes and
    /// the curve arithmetic that turn a tree into an output key belong to
    /// the layer that builds a transaction against a real deployment.
    /// Computing them here would mint an identity before a consumer of
    /// one exists (§1.13).
    TaprootOutputKeyUndischarged,
    /// Which linked constructor each destination position is under is
    /// not settled here.
    ///
    /// The half of §10.4's residual this wave does not reach. The link
    /// builds one constructor per (owner, representation) and can say
    /// exactly which committed tree each is; §12.3 chooses which owner
    /// receives which destination, per request, so the map from output
    /// position to constructor is the candidate ABI's and not this
    /// bundle's.
    DestinationConstructorTableUndischarged,
    /// The resolved internal key was not verified unspendable from
    /// public data.
    ///
    /// The constructor states
    /// [`ConstructorAssumption::InternalKeyUnspendabilityVerifiableFromPublicData`]
    /// and names the linker as the layer that owes it. Discharging it
    /// needs curve arithmetic over a real key, so it is recorded rather
    /// than claimed.
    InternalKeyUnspendabilityUnverified,
    /// The owner keys were not verified to be points on the target's
    /// curve.
    ///
    /// [`tapscript::OwnerKeyResidual::CurvePointMembership`] reaching the
    /// link unchanged. This crate holds no curve arithmetic and takes no
    /// dependency that would give it any, so a refusal would be a claim
    /// rather than a check.
    OwnerKeyCurvePointMembershipUnverified,
    /// The selected sighash profile's required dimensions are not
    /// established by the review.
    ///
    /// [`tapscript::RecognitionResidual::SighashProfileUnreviewed`]
    /// reaching the link unchanged. §11.2 puts the profile at the link,
    /// which is where it now is; it does not put the review at the link,
    /// and no part of Guide 13 completes one. A consumer reading a
    /// verified signature as authorization over §1.7's protected data
    /// while this stands is reading past the obligation rather than
    /// through it.
    ///
    /// The review verdict did not clear it. Six of the seven required
    /// dimensions are established; the issuance dimension is not, because
    /// no candidate this arc builds bears an issuance and the two message
    /// terms carrying the dimension are formed from the input count
    /// alone. The recomputed disposition names that dimension and no
    /// other.
    SighashProfileUnreviewed,
}

/// The outstanding live link obligations, which are never none.
///
/// Structurally non-empty for the same reason the backend's outstanding
/// lifecycle is: the least obligation is a field of its own, so a linked
/// bundle owing nothing has no representation.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OutstandingLiveLinkObligations {
    least: LiveLinkObligation,
    rest: BTreeSet<LiveLinkObligation>,
}

impl OutstandingLiveLinkObligations {
    /// Every outstanding obligation, in canonical order.
    pub fn obligations(&self) -> impl Iterator<Item = &LiveLinkObligation> {
        std::iter::once(&self.least).chain(self.rest.iter())
    }

    /// How many are outstanding, which is never zero.
    ///
    /// Saturation is unreachable: `rest` is a set of the remaining
    /// [`LiveLinkObligation`] variants while `least` is held separately,
    /// so the count is bounded by the enum's finite variant set.
    #[must_use]
    pub fn count(&self) -> NonZeroUsize {
        NonZeroUsize::MIN.saturating_add(self.rest.len())
    }

    /// Whether one obligation is outstanding.
    #[must_use]
    pub fn holds(&self, obligation: LiveLinkObligation) -> bool {
        self.least == obligation || self.rest.contains(&obligation)
    }
}

/// One owner's linked live-receipt constructor under one representation.
///
/// §11.6's *linked constructors*, and the concrete answer to the first
/// third of §10.4's residual: this is the constructor a destination
/// carrying the protocol asset under this owner is under, stated as a
/// typed value because §1.13 mints no identity to state it as.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkedLiveConstructor {
    owner: OwnerParameter,
    representation: LiveTransferRepresentationPlan,
    contract: TargetContractVersion,
    leaf_version: LeafVersion,
    internal_key: StackItem,
    internal_key_policy: InternalKeyPolicy,
    key_path: KeyPathPolicy,
    assumptions: BTreeSet<ConstructorAssumption>,
    programs: BTreeMap<LiveTransferLeafRole, LinkedLiveLeafProgram>,
    taptree: DeterministicTaptree<LiveTransferLeafRole>,
    witness: BTreeMap<LiveTransferLeafRole, BTreeSet<WitnessComponent>>,
    ranges: BTreeMap<LiveTransferShape, CompleteFamilyRanges>,
    substituted: BTreeMap<LiveTransferLeafRole, BTreeSet<LiveBundleSymbol>>,
}

impl LinkedLiveConstructor {
    /// The committed owner this constructor is parameterized by.
    #[must_use]
    pub const fn owner(&self) -> &OwnerParameter {
        &self.owner
    }

    /// The representation whose leaves it holds.
    #[must_use]
    pub const fn representation(&self) -> LiveTransferRepresentationPlan {
        self.representation
    }

    /// The reviewed contract revision it is bound to.
    #[must_use]
    pub const fn contract(&self) -> TargetContractVersion {
        self.contract
    }

    /// The leaf version every committed leaf carries.
    #[must_use]
    pub const fn leaf_version(&self) -> LeafVersion {
        self.leaf_version
    }

    /// The resolved unspendable internal key.
    #[must_use]
    pub const fn internal_key(&self) -> &StackItem {
        &self.internal_key
    }

    /// The internal-key policy, which admits one value.
    #[must_use]
    pub const fn internal_key_policy(&self) -> InternalKeyPolicy {
        self.internal_key_policy
    }

    /// The key-path policy, which admits one value.
    #[must_use]
    pub const fn key_path(&self) -> KeyPathPolicy {
        self.key_path
    }

    /// Every assumption the constructor rests on.
    #[must_use]
    pub const fn assumptions(&self) -> &BTreeSet<ConstructorAssumption> {
        &self.assumptions
    }

    /// Every linked program, in canonical order (§11.6).
    #[must_use]
    pub const fn programs(&self) -> &BTreeMap<LiveTransferLeafRole, LinkedLiveLeafProgram> {
        &self.programs
    }

    /// One linked program by leaf.
    #[must_use]
    pub fn program(&self, leaf: LiveTransferLeafRole) -> Option<&LinkedLiveLeafProgram> {
        self.programs.get(&leaf)
    }

    /// The committed taptree and its control recipes (§11.4).
    #[must_use]
    pub const fn taptree(&self) -> &DeterministicTaptree<LiveTransferLeafRole> {
        &self.taptree
    }

    /// The exact control-path depth the committed tree settles.
    #[must_use]
    pub const fn control_path_depth(&self) -> u32 {
        self.taptree.depth()
    }

    /// Every leaf's witness handoff.
    #[must_use]
    pub const fn witness_roles(
        &self,
    ) -> &BTreeMap<LiveTransferLeafRole, BTreeSet<WitnessComponent>> {
        &self.witness
    }

    /// Every shape's family ranges — §11.6's concrete placements.
    #[must_use]
    pub const fn family_ranges(&self) -> &BTreeMap<LiveTransferShape, CompleteFamilyRanges> {
        &self.ranges
    }

    /// Every symbol substituted into each leaf, in canonical order.
    #[must_use]
    pub const fn substituted(&self) -> &BTreeMap<LiveTransferLeafRole, BTreeSet<LiveBundleSymbol>> {
        &self.substituted
    }

    /// This constructor's placement, as the symbol census records it.
    #[must_use]
    pub fn placement(&self) -> LinkedConstructorPlacement {
        LinkedConstructorPlacement::new(
            self.owner.clone(),
            self.representation,
            self.taptree.recipes().keys().copied().collect(),
        )
    }
}

/// What §10.4's induction step now rests on, and what it still owes.
///
/// Stated as a value rather than left to prose because a reader deciding
/// how far to trust a destination's recognition needs to know which third
/// of the residual moved. Every field is derived from the link that
/// produced it; none is a claim a caller could set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveInductionStep {
    established:
        BTreeMap<(OwnerParameter, LiveTransferRepresentationPlan), LinkedConstructorPlacement>,
    outstanding: BTreeSet<LiveLinkObligation>,
    residual: RecognitionResidual,
}

impl LiveInductionStep {
    /// Every (owner, representation) whose constructor this link settled.
    ///
    /// The link-time end of §10.4: for each of these, the committed tree
    /// over that owner's leaves *is* the tree a destination's version
    /// check points at, and this is the typed value of it.
    #[must_use]
    pub const fn established(
        &self,
    ) -> &BTreeMap<(OwnerParameter, LiveTransferRepresentationPlan), LinkedConstructorPlacement>
    {
        &self.established
    }

    /// What still stands between the step and §10.4 holding whole.
    ///
    /// Never empty. Two obligations remain by construction — the taproot
    /// output key and the destination table — and a step reporting none
    /// would be claiming both.
    #[must_use]
    pub const fn outstanding(&self) -> &BTreeSet<LiveLinkObligation> {
        &self.outstanding
    }

    /// The residual the emitted programs still carry.
    ///
    /// Unchanged, and deliberately so: a *program* still cannot establish
    /// a destination's constructor bytes, whatever this link establishes
    /// about which constructor a given owner has. Re-scoping the reason
    /// is honest; clearing the residual would have been the link claiming
    /// something the leaves do.
    #[must_use]
    pub const fn residual(&self) -> RecognitionResidual {
        self.residual
    }
}

/// What the candidate ABI of §12 receives from this link, and what it
/// still owes.
///
/// §11.6's *ABI handoff*. Not the ABI: §12 is a later wave and §1.12
/// keeps `CandidateLiveTransferAbi` a thing Guide 13 constructs later
/// rather than a field this bundle reserves. What is here is the material
/// §12 consumes — the admitted shapes with their exact bounds, each
/// shape's family ranges, and each leaf's witness components — together
/// with the one thing §12 must supply and this link cannot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveAbiHandoff {
    shapes: LiveTransferShapeSet,
    bounds: LiveTransferShapeBounds,
    ranges: BTreeMap<LiveTransferShape, CompleteFamilyRanges>,
    witness: BTreeMap<LiveTransferLeafRole, BTreeSet<WitnessComponent>>,
    owed: BTreeSet<LiveLinkObligation>,
}

impl LiveAbiHandoff {
    /// The admitted shape set the ABI builds transactions over.
    #[must_use]
    pub const fn shapes(&self) -> &LiveTransferShapeSet {
        &self.shapes
    }

    /// The exact candidate shape bounds (§11.6).
    #[must_use]
    pub const fn bounds(&self) -> LiveTransferShapeBounds {
        self.bounds
    }

    /// Every shape's family ranges, in canonical order.
    #[must_use]
    pub const fn family_ranges(&self) -> &BTreeMap<LiveTransferShape, CompleteFamilyRanges> {
        &self.ranges
    }

    /// Every leaf's witness components.
    #[must_use]
    pub const fn witness_roles(
        &self,
    ) -> &BTreeMap<LiveTransferLeafRole, BTreeSet<WitnessComponent>> {
        &self.witness
    }

    /// What the ABI owes that this link does not supply.
    #[must_use]
    pub const fn owed(&self) -> &BTreeSet<LiveLinkObligation> {
        &self.owed
    }
}

/// The candidate linked live-transfer bundle (§11.6).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateLinkedLiveTransferBundle {
    plan: ValidatedLiveTransferOperationPlan,
    contract: TargetContractVersion,
    shapes: LiveTransferShapeSet,
    plans: BTreeSet<LiveTransferRepresentationPlan>,
    constructors: BTreeMap<(OwnerParameter, LiveTransferRepresentationPlan), LinkedLiveConstructor>,
    definitions: BTreeMap<(OwnerParameter, LiveTransferRepresentationPlan), LiveDefinitionCensus>,
    sighash_profile: SelectedSighashProfile,
    closure: LiveCarrierClosure,
    handoff: LiveAbiHandoff,
    formulas: BTreeMap<
        (LiveTransferRepresentationPlan, LiveProgramRole),
        BTreeMap<ResourceDimension, LinkedLiveResourceFormula>,
    >,
    evidence: BTreeSet<TargetEvidenceRequirementId>,
    external: BTreeSet<ExternalEvidenceRole>,
    residuals: BTreeSet<RecognitionResidual>,
    lifecycle: CandidateTransferLifecycle,
    induction: LiveInductionStep,
    obligations: OutstandingLiveLinkObligations,
}

impl CandidateLinkedLiveTransferBundle {
    /// The validated live-transfer plan the linked programs serve.
    ///
    /// §11.6's *exact live-transfer scope*: the operation, its class,
    /// owner, value, sponsor, root, certificate, and lifecycle closures,
    /// carried whole rather than summarized.
    #[must_use]
    pub const fn plan(&self) -> &ValidatedLiveTransferOperationPlan {
        &self.plan
    }

    /// The reviewed contract revision — §11.6's exact target projection.
    #[must_use]
    pub const fn contract(&self) -> TargetContractVersion {
        self.contract
    }

    /// The candidate shape set and its exact bounds (§11.6).
    #[must_use]
    pub const fn shapes(&self) -> &LiveTransferShapeSet {
        &self.shapes
    }

    /// Every representation plan this bundle carries programs for.
    ///
    /// §11.6's *explicit/private plan status*. A plan absent here is one
    /// this candidate does not link, and nothing about the plans present
    /// says anything about it.
    #[must_use]
    pub const fn representation_plans(&self) -> &BTreeSet<LiveTransferRepresentationPlan> {
        &self.plans
    }

    /// Every linked constructor, keyed by owner and representation.
    #[must_use]
    pub const fn constructors(
        &self,
    ) -> &BTreeMap<(OwnerParameter, LiveTransferRepresentationPlan), LinkedLiveConstructor> {
        &self.constructors
    }

    /// One linked constructor.
    #[must_use]
    pub fn constructor(
        &self,
        owner: &OwnerParameter,
        representation: LiveTransferRepresentationPlan,
    ) -> Option<&LinkedLiveConstructor> {
        self.constructors.get(&(owner.clone(), representation))
    }

    /// Each linked constructor's definition census (§11.2).
    ///
    /// Keyed by the same (owner, representation) pair the constructors
    /// are, because a census is one emitted bundle's and a bundle is one
    /// owner's: two owners under one representation have two censuses
    /// with two different owner-key symbols in them.
    #[must_use]
    pub const fn definitions(
        &self,
    ) -> &BTreeMap<(OwnerParameter, LiveTransferRepresentationPlan), LiveDefinitionCensus> {
        &self.definitions
    }

    /// The selected sighash profile and what the review establishes.
    #[must_use]
    pub const fn sighash_profile(&self) -> &SelectedSighashProfile {
        &self.sighash_profile
    }

    /// The per-plan carrier closure (§11.5).
    #[must_use]
    pub const fn carrier_closure(&self) -> &LiveCarrierClosure {
        &self.closure
    }

    /// What the candidate ABI of §12 receives (§11.6).
    #[must_use]
    pub const fn abi_handoff(&self) -> &LiveAbiHandoff {
        &self.handoff
    }

    /// The refitted exact resource formulas (§11.6).
    #[must_use]
    pub const fn formulas(
        &self,
    ) -> &BTreeMap<
        (LiveTransferRepresentationPlan, LiveProgramRole),
        BTreeMap<ResourceDimension, LinkedLiveResourceFormula>,
    > {
        &self.formulas
    }

    /// Every target evidence requirement still unresolved (§11.6).
    #[must_use]
    pub const fn unresolved_target_evidence(&self) -> &BTreeSet<TargetEvidenceRequirementId> {
        &self.evidence
    }

    /// Every external-evidence role the plans leave open (§11.6).
    ///
    /// The private plan's [`ExternalEvidenceRole::ConfidentialValueConservation`]
    /// among them, carried as external because §11.5 forbids reassigning
    /// it to a local target program and §10.6 forbids minting a backend
    /// pattern for it.
    #[must_use]
    pub const fn unresolved_external_evidence(&self) -> &BTreeSet<ExternalEvidenceRole> {
        &self.external
    }

    /// Every recognition residual the linked programs carry.
    #[must_use]
    pub const fn residuals(&self) -> &BTreeSet<RecognitionResidual> {
        &self.residuals
    }

    /// The outstanding burn and redemption lifecycle (§11.6).
    ///
    /// Structurally incomplete: its outstanding count is a
    /// [`NonZeroUsize`], so a linked bundle whose lifecycle was complete
    /// has no representation.
    #[must_use]
    pub const fn outstanding_lifecycle(&self) -> &CandidateTransferLifecycle {
        &self.lifecycle
    }

    /// Where §10.4's induction step now stands.
    #[must_use]
    pub const fn induction_step(&self) -> &LiveInductionStep {
        &self.induction
    }

    /// The obligations this link created and did not discharge.
    #[must_use]
    pub const fn outstanding_obligations(&self) -> &OutstandingLiveLinkObligations {
        &self.obligations
    }

    /// This artifact's status (§1.12).
    ///
    /// Always [`LinkedArtifactStatus::Prototype`], and read-only. The
    /// evidence a promotion rests on is a later wave's, does not exist,
    /// and has no field reserved for it here.
    #[must_use]
    pub const fn status(&self) -> LinkedArtifactStatus {
        LinkedArtifactStatus::Prototype
    }

    /// The exact total linked program bytes of every committed leaf.
    ///
    /// # Panics
    ///
    /// Panics only if the construction-time bound below stops holding,
    /// which no linked bundle can arrange.
    #[must_use]
    pub fn total_script_bytes(&self) -> u64 {
        // A returned linked bundle has passed the tree leaf budget, so
        // each constructor holds at most `TREE_LEAF_BUDGET` programs;
        // each is built through `TapscriptProgram::new`, which caps it at
        // ten thousand instructions, and under the reviewed Elements push
        // contract one instruction encodes to at most one opcode byte,
        // four width bytes, and 520 payload bytes. The checked `u128` sum
        // keeps that bound load-bearing if those limits ever move.
        let total = self
            .constructors
            .values()
            .flat_map(|constructor| constructor.programs.values())
            .filter_map(|program| program.charged(ResourceDimension::ScriptBytes))
            .try_fold(0_u128, |total, bytes| total.checked_add(u128::from(bytes)))
            .expect("linked script-byte total obeys the construction-time bound");
        u64::try_from(total).expect("linked script-byte total obeys the construction-time bound")
    }
}

/// Link the candidate live-transfer bundles of both representations
/// (§11).
///
/// One relocatable bundle per (owner, representation) is offered and each
/// is linked on its own terms; the carrier closure of §11.5 is then run
/// *per plan* against each plan's own committed tree, which is what makes
/// [`LinkRefusal::PlanStarvedOfCarrier`] reachable.
///
/// The bundles must agree about the plan they were emitted for. That is
/// the same rule [`crate::bundle`] keeps by construction — its linker
/// reads the plan out of the bundle — held here by comparison instead,
/// because there is more than one bundle and two of them describing
/// different plans would be a link over two different operations.
///
/// # Errors
///
/// [`LinkRefusal::NoLiveBundleOffered`] for an empty offering,
/// [`LinkRefusal::LiveBundleIsNotACandidate`] for a bundle already
/// claiming more than a prototype,
/// [`LinkRefusal::LiveBundlePlansDisagree`] when two bundles were emitted
/// for different validated plans,
/// [`LinkRefusal::DuplicateLinkedConstructor`] when two bundles claim one
/// (owner, representation), [`LinkRefusal::LiveSymbolRoleUnfilled`] when
/// the census leaves a §11.2 role unfilled, and every refusal the symbol,
/// relocation, taptree, and carrier stages raise.
pub fn link_live_candidate(
    target: &ReviewedElementsTapscriptDefinition,
    bundles: &[CandidateRelocatableLiveTransferBundle],
    deployment: &LiveLinkDeploymentParameters,
) -> Result<CandidateLinkedLiveTransferBundle, LinkRefusal> {
    let first = check_offering(bundles)?;

    let mut constructors = BTreeMap::new();
    let mut definitions: BTreeMap<
        (OwnerParameter, LiveTransferRepresentationPlan),
        LiveDefinitionCensus,
    > = BTreeMap::new();
    let mut emitted_leaves: BTreeMap<LiveTransferRepresentationPlan, BTreeSet<_>> = BTreeMap::new();
    let mut committed_leaves: BTreeMap<LiveTransferRepresentationPlan, BTreeSet<_>> =
        BTreeMap::new();
    let mut evidence = BTreeSet::new();
    let mut residuals = BTreeSet::new();
    let mut plans = BTreeSet::new();
    let mut ranges: BTreeMap<LiveTransferShape, CompleteFamilyRanges> = BTreeMap::new();
    let mut witness: BTreeMap<LiveTransferLeafRole, BTreeSet<WitnessComponent>> = BTreeMap::new();

    for bundle in bundles {
        let representation = bundle.representation();
        let owner = OwnerParameter::new(bundle.constructor().owner().clone());
        let (census, constructor) = link_one_bundle(target, bundle, deployment)?;

        plans.insert(representation);
        emitted_leaves
            .entry(representation)
            .or_default()
            .extend(bundle.leaves().keys().copied());
        committed_leaves
            .entry(representation)
            .or_default()
            .extend(constructor.taptree.recipes().keys().copied());
        evidence.extend(bundle.target_evidence().iter().copied());
        residuals.extend(bundle.residuals().iter().copied());
        ranges.extend(
            bundle
                .family_ranges()
                .iter()
                .map(|(shape, census)| (*shape, census.clone())),
        );
        witness.extend(
            bundle
                .leaves()
                .iter()
                .map(|(leaf, program)| (*leaf, program.witness().clone())),
        );

        // Keyed by the pair, because a census is a bundle's and a bundle
        // is one owner's: two owners under one representation are two
        // censuses, and a map keyed by representation alone would have
        // let the second silently replace the first.
        if definitions
            .insert((owner.clone(), representation), census)
            .is_some()
            || constructors
                .insert((owner, representation), constructor)
                .is_some()
        {
            return Err(LinkRefusal::DuplicateLinkedConstructor { representation });
        }
    }

    // §11.2's role census is a property of the link rather than of any
    // one bundle: each emitted bundle carries one representation, so no
    // single census can fill both plans' program roles. Checked once,
    // here, over the union and against the plans this link actually
    // carries.
    let unfilled = link_role_defects(
        definitions
            .iter()
            .map(|((_, representation), census)| (*representation, census)),
    );
    if !unfilled.is_empty() {
        return Err(LinkRefusal::LiveSymbolRoleUnfilled {
            roles: unfilled.into_iter().collect(),
        });
    }

    let closure = close_live(first.plan(), &emitted_leaves, &committed_leaves)?;
    let external: BTreeSet<ExternalEvidenceRole> = closure
        .plans()
        .values()
        .flat_map(|plan| plan.external_evidence().iter().copied())
        .collect();

    let formulas = refit(&constructors);
    let obligations = obligations();
    let induction = induction_step(&constructors);

    let shapes = first.shapes().clone();
    let bounds = shapes.bounds();

    Ok(CandidateLinkedLiveTransferBundle {
        plan: first.plan().clone(),
        contract: first.contract(),
        shapes: shapes.clone(),
        plans,
        constructors,
        definitions,
        sighash_profile: deployment.sighash_profile().clone(),
        closure,
        handoff: LiveAbiHandoff {
            shapes,
            bounds,
            ranges,
            witness,
            owed: BTreeSet::from([
                LiveLinkObligation::TaprootOutputKeyUndischarged,
                LiveLinkObligation::DestinationConstructorTableUndischarged,
            ]),
        },
        formulas,
        evidence,
        external,
        residuals,
        lifecycle: first.lifecycle().clone(),
        induction,
        obligations,
    })
}

/// The one bundle every other offered bundle must agree with.
///
/// # Errors
///
/// [`LinkRefusal::NoLiveBundleOffered`] for an empty offering,
/// [`LinkRefusal::LiveBundleIsNotACandidate`] for a bundle already
/// claiming more than a prototype, and
/// [`LinkRefusal::LiveBundlePlansDisagree`] when two bundles were emitted
/// for different validated plans.
fn check_offering(
    bundles: &[CandidateRelocatableLiveTransferBundle],
) -> Result<&CandidateRelocatableLiveTransferBundle, LinkRefusal> {
    let first = bundles.first().ok_or(LinkRefusal::NoLiveBundleOffered)?;
    for bundle in bundles {
        if bundle.status() != BackendArtifactStatus::Prototype {
            return Err(LinkRefusal::LiveBundleIsNotACandidate);
        }
        if bundle.plan() != first.plan() {
            return Err(LinkRefusal::LiveBundlePlansDisagree);
        }
    }
    Ok(first)
}

/// Where §10.4's induction step stands after one link.
///
/// The outstanding pair is stated rather than computed, and that is the
/// honest shape: neither the taproot output key nor §12.3's destination
/// table depends on what was linked, so a link that reported one of them
/// discharged would be reporting something no input could have changed.
fn induction_step(
    constructors: &BTreeMap<
        (OwnerParameter, LiveTransferRepresentationPlan),
        LinkedLiveConstructor,
    >,
) -> LiveInductionStep {
    LiveInductionStep {
        established: constructors
            .iter()
            .map(|(key, constructor)| (key.clone(), constructor.placement()))
            .collect(),
        outstanding: BTreeSet::from([
            LiveLinkObligation::TaprootOutputKeyUndischarged,
            LiveLinkObligation::DestinationConstructorTableUndischarged,
        ]),
        residual: RecognitionResidual::LinkedDestinationConstructorIdentity,
    }
}

/// Link one relocatable bundle into one constructor and its census.
///
/// The per-bundle half of [`link_live_candidate`], split out because the
/// stages it runs — the definition census, substitution, the tree, and
/// the link's own constructor symbol — are one bundle's business, while
/// the carrier closure and the role census are the whole link's.
///
/// # Errors
///
/// Every refusal the symbol, relocation, and taptree stages raise.
fn link_one_bundle(
    target: &ReviewedElementsTapscriptDefinition,
    bundle: &CandidateRelocatableLiveTransferBundle,
    deployment: &LiveLinkDeploymentParameters,
) -> Result<(LiveDefinitionCensus, LinkedLiveConstructor), LinkRefusal> {
    let representation = bundle.representation();
    let owner = OwnerParameter::new(bundle.constructor().owner().clone());

    let mut census = collect_live_definitions(target, bundle, deployment)?;
    let programs = substitute_live(target, bundle, &census, deployment.resolved())?;

    // §11.4's tree input takes the *declaration sequence* rather than a
    // set, so a duplicate is refused rather than absorbed. The programs
    // map is already keyed, so the declarations are its keys — one
    // census used twice, rather than a second list of leaves that could
    // disagree with the first.
    let input = live_taptree_input(
        programs.keys().copied(),
        bundle.leaf_version(),
        deployment.maximum_control_path_depth(),
    )?;
    let taptree = assemble_live(&input)?;

    let constructor = LinkedLiveConstructor {
        owner: owner.clone(),
        representation,
        contract: bundle.contract(),
        leaf_version: bundle.leaf_version(),
        internal_key: deployment.internal_key().clone(),
        internal_key_policy: bundle.internal_key(),
        key_path: bundle.key_path(),
        assumptions: bundle.assumptions().clone(),
        witness: bundle
            .leaves()
            .iter()
            .map(|(leaf, program)| (*leaf, program.witness().clone()))
            .collect(),
        ranges: bundle.family_ranges().clone(),
        substituted: programs
            .iter()
            .map(|(leaf, program)| (*leaf, program.substituted().clone()))
            .collect(),
        programs,
        taptree,
    };

    // The link's own half of the census: the constructor symbol, which is
    // owner-parameterized and settled by nothing before this point.
    census.define(
        LiveLinkSymbol::LiveReceiptConstructor {
            owner,
            representation,
        },
        LiveSymbolValue::LinkedConstructor(Box::new(constructor.placement())),
        LiveDefinitionOrigin::Link,
    )?;

    Ok((census, constructor))
}

/// The obligations every live link leaves outstanding.
///
/// A constant set rather than a computed one, and that is the honest
/// shape: none of the five depends on what was linked. Two are §10.4's
/// remaining halves, one is the internal key's unspendability, one is the
/// owner keys' curve membership, and one is the sighash review — and no
/// deployment parameter, bundle, or owner discharges any of them.
fn obligations() -> OutstandingLiveLinkObligations {
    OutstandingLiveLinkObligations {
        least: LiveLinkObligation::TaprootOutputKeyUndischarged,
        rest: BTreeSet::from([
            LiveLinkObligation::DestinationConstructorTableUndischarged,
            LiveLinkObligation::InternalKeyUnspendabilityUnverified,
            LiveLinkObligation::OwnerKeyCurvePointMembershipUnverified,
            LiveLinkObligation::SighashProfileUnreviewed,
        ]),
    }
}

/// Refit every representation and role's resource behaviour over the
/// linked programs.
fn refit(
    constructors: &BTreeMap<
        (OwnerParameter, LiveTransferRepresentationPlan),
        LinkedLiveConstructor,
    >,
) -> BTreeMap<
    (LiveTransferRepresentationPlan, LiveProgramRole),
    BTreeMap<ResourceDimension, LinkedLiveResourceFormula>,
> {
    let mut formulas: BTreeMap<_, BTreeMap<_, _>> = BTreeMap::new();

    for ((_, representation), constructor) in constructors {
        let mut dimensions = BTreeSet::new();
        for program in constructor.programs.values() {
            dimensions.extend(program.dimensions().keys().copied());
        }

        for role in [LiveProgramRole::Coordinator, LiveProgramRole::Member] {
            for dimension in &dimensions {
                let mut measurements = BTreeMap::new();
                for (leaf, program) in &constructor.programs {
                    if leaf.program_role() != role {
                        continue;
                    }
                    let Some(measure) = program.charged(*dimension) else {
                        continue;
                    };
                    // A coordinator leaf serves exactly one shape; a
                    // member leaf serves every shape of its receipt-input
                    // count, so every one of those shapes measures the
                    // same program. Both are read off the leaf's own
                    // shape census rather than re-derived here.
                    for shape in shapes_of(constructor, *leaf) {
                        measurements.insert(shape, measure);
                    }
                }
                if measurements.is_empty() {
                    continue;
                }
                formulas.entry((*representation, role)).or_default().insert(
                    *dimension,
                    linked_formula(*representation, role, *dimension, measurements),
                );
            }
        }
    }

    formulas
}

/// Every shape one committed leaf serves.
///
/// Read from the family-range census the bundle emitted, which is keyed
/// by shape: a coordinator leaf names its shape directly, and a member
/// leaf serves every shape whose receipt-input count it bounds.
fn shapes_of(
    constructor: &LinkedLiveConstructor,
    leaf: LiveTransferLeafRole,
) -> Vec<LiveTransferShape> {
    match leaf {
        LiveTransferLeafRole::Coordinator { shape, .. } => vec![shape],
        LiveTransferLeafRole::Member { receipt_inputs, .. } => constructor
            .ranges
            .keys()
            .filter(|shape| shape.receipt_inputs() == receipt_inputs)
            .copied()
            .collect(),
    }
}
