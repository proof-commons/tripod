//! The candidate live-transfer ABI (§12.1, §12.2, §12.9).
//!
//! # Derived, never parsed
//!
//! Every value here comes from the linked live bundle, the reviewed
//! contract, or the public point arithmetic the capability supplies.
//! Nothing is read from a caller and nothing is defaulted: the layouts
//! are the linker's own family ranges, the destination programs are the
//! linked constructors' committed trees tweaked, and the sighash profile
//! is the one the link selected, carrying the disposition the link gave
//! it.
//!
//! # A candidate, held there by four facts
//!
//! [`LiveAbiStatus`] has one member and is returned by a method that
//! reads no field, so nothing can set it. There is no ABI hash here and
//! no field that would hold one (§12.9). The outstanding obligations are
//! structurally non-empty, so a reader cannot find an ABI that owes
//! nothing. And the two obligations the link handed up are answered
//! explicitly rather than dropped: [`InheritedLinkObligations`] partitions
//! them into what this derivation discharged and what it carries, and the
//! derivation refuses if the partition is not exactly the handoff's own
//! set.
//!
//! # What the induction closes here
//!
//! §11's link left `DestinationConstructorTableUndischarged`: the linker
//! established one constructor per (owner, representation) and had no
//! request to select among them. §12.3 is that request, and
//! [`DestinationConstructorTable`] is the selection — every linked
//! placement, keyed by the owner a destination entry may name, with the
//! exact program an output paying to that owner carries. An owner a
//! request names and this table does not hold has no destination, which
//! is the refusal that makes the table load-bearing.

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroUsize;

use linker::backend::WitnessComponent;
use linker::live_backend::{
    CompleteFamilyRanges, LiveFamily, LiveFamilyRange, LiveInputFamily, LiveOutputFamily,
    LiveTransferLeafRole, LiveTransferRepresentationPlan, LiveTransferShape,
    LiveTransferShapeBounds, ProtectedDatum, RecognitionResidual,
};
use linker::{
    CandidateLinkedLiveTransferBundle, LinkedArtifactStatus, LinkedConstructorPlacement,
    LiveLinkObligation, LiveLinkSymbol, LiveSymbolValue, OwnerParameter, SelectedSighashProfile,
};
use target_elements::{LeafVersion, ReviewedElementsTapscriptDefinition, TargetContractVersion};

use crate::abi::TargetTransactionVersion;
use crate::bytes::AssetId;
use crate::error::TransactionRefusal;
use crate::live_taproot::{LiveCurveCapability, LiveReceiptInstance, derive_live_receipt_instance};

/// The status vocabulary a derived live-transfer ABI is distinguished
/// by (§12.9).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiveAbiStatus {
    /// Derived and internally checked; no operation evidence, and no
    /// digest.
    Candidate,
}

/// Which of the two transaction forms one shape realizes (§12.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiveTransactionForm {
    /// A sponsor suffix funds the target fee role.
    Sponsored,
    /// No sponsor region, and no fee role.
    Sponsorless,
}

/// Which input coordinates the family (§12.1).
///
/// A rule rather than a selection: §12.3 refuses a request that names
/// the coordinator, and the ABI puts it at input zero for every shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LiveCoordinatorRule {
    index: u16,
}

impl LiveCoordinatorRule {
    /// The input position the coordinator occupies.
    #[must_use]
    pub const fn index(self) -> u16 {
        self.index
    }
}

/// The order the ABI sorts a run of inputs in (§12.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum LiveCanonicalOrdering {
    /// Ascending by transaction identifier, then by index.
    AscendingOutpoint,
}

/// One item of a live-receipt script-path witness, in ABI order.
///
/// Three members where the compact-ASH census has two, and the extra one
/// is the whole difference between spending a shared object and spending
/// somebody's receipt: a live receipt's leaf checks an owner signature,
/// so the stack carries one before the script it is checked by.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum LiveWitnessItem {
    /// The consumed receipt owner's signature over the finalized
    /// message.
    OwnerSignature,
    /// The executing leaf's exact committed program.
    LeafScript,
    /// The control block authenticating that leaf.
    ControlBlock,
}

impl LiveWitnessItem {
    /// The witness order every live receipt input uses.
    ///
    /// §12.3 refuses a request that selects it, so it is a constant of
    /// the ABI rather than a parameter of a build.
    pub const ORDER: &'static [Self] =
        &[Self::OwnerSignature, Self::LeafScript, Self::ControlBlock];
}

/// The deployment constants the live-transfer ABI reads (§11.2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveDeploymentSymbols {
    protocol_asset: AssetId,
    reserve_asset: AssetId,
    destination_program_version: u8,
    sponsor_change_program: Vec<u8>,
    sponsor_change_version: u8,
    fee_program_digest: Vec<u8>,
}

impl LiveDeploymentSymbols {
    /// The explicit protocol asset every receipt carries.
    #[must_use]
    pub const fn protocol_asset(&self) -> AssetId {
        self.protocol_asset
    }

    /// The explicit reserve asset every sponsor and fee role carries.
    #[must_use]
    pub const fn reserve_asset(&self) -> AssetId {
        self.reserve_asset
    }

    /// The witness version a destination program is read at.
    #[must_use]
    pub const fn destination_program_version(&self) -> u8 {
        self.destination_program_version
    }

    /// The sponsor-change role's admitted witness program.
    #[must_use]
    pub fn sponsor_change_program(&self) -> &[u8] {
        &self.sponsor_change_program
    }

    /// The witness version the sponsor-change program is read at.
    #[must_use]
    pub const fn sponsor_change_version(&self) -> u8 {
        self.sponsor_change_version
    }

    /// The target fee role's program digest.
    #[must_use]
    pub fn fee_program_digest(&self) -> &[u8] {
        &self.fee_program_digest
    }
}

/// One admitted shape's input and output layout (§12.1, §12.2).
///
/// The ranges are the linker's, not a second derivation: §11 already
/// placed every family for every shape, and a layout computed again here
/// could disagree with the programs that authenticate it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveShapeAbi {
    shape: LiveTransferShape,
    ranges: CompleteFamilyRanges,
    coordinator: LiveCoordinatorRule,
    receipt_input_range: (u16, u16),
    sponsor_input_range: (u16, u16),
    destination_range: (u16, u16),
    sponsor_change_position: Option<u16>,
    fee_position: Option<u16>,
    form: LiveTransactionForm,
    version: TargetTransactionVersion,
}

impl LiveShapeAbi {
    /// The shape this layout realizes.
    #[must_use]
    pub const fn shape(&self) -> LiveTransferShape {
        self.shape
    }

    /// Every family's run of positions, as the link placed them.
    #[must_use]
    pub const fn ranges(&self) -> &CompleteFamilyRanges {
        &self.ranges
    }

    /// Which input coordinates the family.
    #[must_use]
    pub const fn coordinator(&self) -> LiveCoordinatorRule {
        self.coordinator
    }

    /// The half-open run of receipt input positions.
    #[must_use]
    pub const fn receipt_input_range(&self) -> (u16, u16) {
        self.receipt_input_range
    }

    /// The half-open run of sponsor input positions, empty when
    /// sponsorless.
    #[must_use]
    pub const fn sponsor_input_range(&self) -> (u16, u16) {
        self.sponsor_input_range
    }

    /// The half-open run of destination output positions.
    #[must_use]
    pub const fn destination_range(&self) -> (u16, u16) {
        self.destination_range
    }

    /// Where the sponsor-change role sits, when the shape has one.
    #[must_use]
    pub const fn sponsor_change_position(&self) -> Option<u16> {
        self.sponsor_change_position
    }

    /// Where the target fee role sits, when the shape has one.
    #[must_use]
    pub const fn fee_position(&self) -> Option<u16> {
        self.fee_position
    }

    /// Which transaction form the shape realizes.
    #[must_use]
    pub const fn form(&self) -> LiveTransactionForm {
        self.form
    }

    /// The target transaction version the form is built at.
    #[must_use]
    pub const fn version(&self) -> TargetTransactionVersion {
        self.version
    }

    /// The leaf one receipt input at `position` executes.
    ///
    /// The coordinator leaf at input zero and the member leaf elsewhere,
    /// both parameterized the way §7.1's static leaf set is: the
    /// coordinator by the whole shape, the member by the receipt-input
    /// count alone.
    ///
    /// # Errors
    ///
    /// [`TransactionRefusal::ReceiptPositionOutsideFamily`] for a
    /// position outside this shape's receipt run.
    pub const fn receipt_leaf(
        &self,
        representation: LiveTransferRepresentationPlan,
        position: u16,
    ) -> Result<LiveTransferLeafRole, TransactionRefusal> {
        let (first, end) = self.receipt_input_range;
        if position < first || position >= end {
            return Err(TransactionRefusal::ReceiptPositionOutsideFamily { position });
        }
        if position == self.coordinator.index {
            Ok(LiveTransferLeafRole::Coordinator {
                representation,
                shape: self.shape,
            })
        } else {
            Ok(LiveTransferLeafRole::Member {
                representation,
                receipt_inputs: self.shape.receipt_inputs(),
            })
        }
    }
}

/// One destination owner's linked constructor and the receipt it builds.
///
/// The placement is the link's own record of which leaves that owner's
/// constructor committed to; the instance is what an output paying to
/// that owner actually carries. Keeping both is what makes the table an
/// answer to §11's obligation rather than a lookup table of programs: a
/// reader can see which linked artifact each program came from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveDestinationConstructor {
    placement: LinkedConstructorPlacement,
    instance: LiveReceiptInstance,
}

impl LiveDestinationConstructor {
    /// The link's placement for this owner and representation.
    #[must_use]
    pub const fn placement(&self) -> &LinkedConstructorPlacement {
        &self.placement
    }

    /// The committed tree, output key, and program.
    #[must_use]
    pub const fn instance(&self) -> &LiveReceiptInstance {
        &self.instance
    }
}

/// Every owner a destination entry may name, and what it builds (§12.3).
///
/// The end of §7.6's induction, and the discharge of the link's
/// `DestinationConstructorTableUndischarged`. The link derived one
/// constructor per (owner, representation) and had no request to select
/// among them; a request's destination owners are that selection, and
/// this is what they select from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DestinationConstructorTable {
    entries: BTreeMap<(OwnerParameter, LiveTransferRepresentationPlan), LiveDestinationConstructor>,
}

impl DestinationConstructorTable {
    /// Every entry, in canonical order.
    #[must_use]
    pub const fn entries(
        &self,
    ) -> &BTreeMap<(OwnerParameter, LiveTransferRepresentationPlan), LiveDestinationConstructor>
    {
        &self.entries
    }

    /// One owner's constructor under one representation.
    #[must_use]
    pub fn get(
        &self,
        owner: &OwnerParameter,
        representation: LiveTransferRepresentationPlan,
    ) -> Option<&LiveDestinationConstructor> {
        self.entries.get(&(owner.clone(), representation))
    }

    /// Every owner the table holds a constructor for, in canonical
    /// order.
    #[must_use]
    pub fn owners(
        &self,
        representation: LiveTransferRepresentationPlan,
    ) -> BTreeSet<OwnerParameter> {
        self.entries
            .keys()
            .filter(|(_, plan)| *plan == representation)
            .map(|(owner, _)| owner.clone())
            .collect()
    }

    /// Which owner an output program belongs to, if any.
    ///
    /// How a consumed receipt's owner is *recognized* rather than
    /// selected (§12.3): the request names an outpoint, the public view
    /// gives that outpoint's program, and exactly one owner's
    /// constructor produces it. An owner nobody's constructor produces
    /// is not a live receipt of this deployment.
    #[must_use]
    pub fn owner_of_program(
        &self,
        representation: LiveTransferRepresentationPlan,
        program: &[u8],
    ) -> Option<&OwnerParameter> {
        self.entries
            .iter()
            .find(|((_, plan), constructor)| {
                *plan == representation && constructor.instance.program() == program
            })
            .map(|((owner, _), _)| owner)
    }
}

/// One obligation the live-transfer ABI creates or inherits and does not
/// discharge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum LiveAbiObligation {
    /// No target run has executed anything this ABI describes (§1.11).
    ///
    /// The least obligation, and the one no derivation can ever
    /// discharge: a verdict belongs to a run.
    TargetExecutionEvidenceAbsent,
    /// The selected sighash profile is not established by the review
    /// (§1.7, §9.2).
    ///
    /// Inherited from the link, which carries the same one, and carried
    /// rather than restated: the profile a signing request names is the
    /// link's, and its disposition travels with it.
    ///
    /// The review verdict left it standing on one dimension. Six of the
    /// seven the profile requires are established by the source review
    /// and the observed acceptance together; the issuance dimension is
    /// not, because no candidate this arc builds bears an issuance. The
    /// accepted result in [`crate::live_accepted`] is well formed and
    /// exact-byte bound while this stands — what it is not is handable,
    /// and the guide consuming it owns that refusal.
    SelectedSighashProfileUnreviewed,
    /// The internal key's unspendability is asserted by the deployment
    /// and verified by nothing here (§7.5).
    InternalKeyUnspendabilityUnverified,
    /// Neither confidential value prefix has been exercised against the
    /// target.
    ///
    /// The private representation's field form is settled on the target
    /// and nowhere else. This ABI can state which prefix a commitment
    /// carries; it cannot say the target reads it that way.
    ConfidentialFieldFormSettledOnlyOnTheTarget,
}

/// The outstanding live-transfer ABI obligations, which are never none.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OutstandingLiveAbiObligations {
    least: LiveAbiObligation,
    rest: BTreeSet<LiveAbiObligation>,
}

impl OutstandingLiveAbiObligations {
    /// Every obligation, in canonical order.
    pub fn obligations(&self) -> impl Iterator<Item = &LiveAbiObligation> {
        std::iter::once(&self.least).chain(self.rest.iter())
    }

    /// How many obligations stand.
    #[must_use]
    pub fn count(&self) -> NonZeroUsize {
        NonZeroUsize::MIN.saturating_add(self.rest.len())
    }

    /// Whether one obligation stands.
    #[must_use]
    pub fn holds(&self, obligation: LiveAbiObligation) -> bool {
        self.least == obligation || self.rest.contains(&obligation)
    }
}

/// What this derivation did about each obligation the link handed up.
///
/// A partition rather than a list, and the derivation refuses unless the
/// two halves are disjoint and together are exactly the handoff's own
/// set. An obligation quietly dropped between two waves is the failure
/// this type exists to make impossible; an obligation quietly recorded
/// in both halves is the other one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InheritedLinkObligations {
    discharged: BTreeSet<LiveLinkObligation>,
    carried: BTreeSet<LiveLinkObligation>,
}

impl InheritedLinkObligations {
    /// The link obligations this derivation discharged.
    #[must_use]
    pub const fn discharged(&self) -> &BTreeSet<LiveLinkObligation> {
        &self.discharged
    }

    /// The link obligations this derivation carries forward.
    #[must_use]
    pub const fn carried(&self) -> &BTreeSet<LiveLinkObligation> {
        &self.carried
    }
}

/// The candidate live-transfer ABI (§12.9).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateLiveTransferAbi {
    contract: TargetContractVersion,
    leaf_version: LeafVersion,
    representations: BTreeSet<LiveTransferRepresentationPlan>,
    bounds: LiveTransferShapeBounds,
    shapes: BTreeMap<LiveTransferShape, LiveShapeAbi>,
    destinations: DestinationConstructorTable,
    coordinator: LiveCoordinatorRule,
    ordering: LiveCanonicalOrdering,
    witness_order: Vec<LiveWitnessItem>,
    witness_roles: BTreeMap<LiveTransferLeafRole, BTreeSet<WitnessComponent>>,
    profile: SelectedSighashProfile,
    protected: BTreeSet<ProtectedDatum>,
    symbols: LiveDeploymentSymbols,
    lock_time: u32,
    residuals: BTreeSet<RecognitionResidual>,
    inherited: InheritedLinkObligations,
    obligations: OutstandingLiveAbiObligations,
}

impl CandidateLiveTransferAbi {
    /// The reviewed contract revision this ABI is bound to.
    #[must_use]
    pub const fn contract(&self) -> TargetContractVersion {
        self.contract
    }

    /// The leaf version every committed leaf carries.
    #[must_use]
    pub const fn leaf_version(&self) -> LeafVersion {
        self.leaf_version
    }

    /// The representation plans this ABI carries constructors for.
    #[must_use]
    pub const fn representations(&self) -> &BTreeSet<LiveTransferRepresentationPlan> {
        &self.representations
    }

    /// The candidate shape bounds.
    #[must_use]
    pub const fn bounds(&self) -> LiveTransferShapeBounds {
        self.bounds
    }

    /// Every admitted shape's layout, in canonical order.
    #[must_use]
    pub const fn shapes(&self) -> &BTreeMap<LiveTransferShape, LiveShapeAbi> {
        &self.shapes
    }

    /// One shape's layout.
    #[must_use]
    pub fn shape(&self, shape: LiveTransferShape) -> Option<&LiveShapeAbi> {
        self.shapes.get(&shape)
    }

    /// Every owner a destination may name, and what it builds.
    #[must_use]
    pub const fn destinations(&self) -> &DestinationConstructorTable {
        &self.destinations
    }

    /// Which input coordinates the family.
    #[must_use]
    pub const fn coordinator(&self) -> LiveCoordinatorRule {
        self.coordinator
    }

    /// The order a run of inputs is sorted in.
    #[must_use]
    pub const fn ordering(&self) -> LiveCanonicalOrdering {
        self.ordering
    }

    /// The witness order every receipt input uses.
    #[must_use]
    pub fn witness_order(&self) -> &[LiveWitnessItem] {
        &self.witness_order
    }

    /// Which components each committed leaf's witness carries.
    #[must_use]
    pub const fn witness_roles(
        &self,
    ) -> &BTreeMap<LiveTransferLeafRole, BTreeSet<WitnessComponent>> {
        &self.witness_roles
    }

    /// The sighash profile the link selected, with its disposition.
    #[must_use]
    pub const fn sighash_profile(&self) -> &SelectedSighashProfile {
        &self.profile
    }

    /// The protected data every owner signature commits to (§1.7).
    #[must_use]
    pub const fn protected_data(&self) -> &BTreeSet<ProtectedDatum> {
        &self.protected
    }

    /// The deployment constants.
    #[must_use]
    pub const fn symbols(&self) -> &LiveDeploymentSymbols {
        &self.symbols
    }

    /// The locktime every candidate is built at.
    #[must_use]
    pub const fn lock_time(&self) -> u32 {
        self.lock_time
    }

    /// The recognition residuals the link recorded.
    #[must_use]
    pub const fn residuals(&self) -> &BTreeSet<RecognitionResidual> {
        &self.residuals
    }

    /// What this derivation did about each link obligation.
    #[must_use]
    pub const fn inherited_link_obligations(&self) -> &InheritedLinkObligations {
        &self.inherited
    }

    /// The obligations this ABI does not discharge.
    #[must_use]
    pub const fn outstanding_obligations(&self) -> &OutstandingLiveAbiObligations {
        &self.obligations
    }

    /// This artifact's status (§12.9).
    ///
    /// Always [`LiveAbiStatus::Candidate`], and read-only.
    #[must_use]
    pub const fn status(&self) -> LiveAbiStatus {
        LiveAbiStatus::Candidate
    }
}

/// The locktime every live-transfer candidate is built at.
///
/// Zero, which is the value that imposes no height or time constraint.
/// A live transfer is not time-locked — §7.3 makes the live class
/// structural and §10.8 refuses a time-locked predecessor at a transfer
/// leaf — so a nonzero locktime would be a constraint nobody asked for
/// and every owner would be signing.
pub const LIVE_TRANSFER_LOCK_TIME: u32 = 0;

/// Derive the candidate live-transfer ABI (§12.9).
///
/// # Errors
///
/// [`TransactionRefusal::LiveBundleIsNotACandidate`] for a bundle
/// claiming more than a candidate link;
/// [`TransactionRefusal::LiveContractRevisionMismatch`] when the bundle
/// is bound to a different reviewed revision than the target;
/// [`TransactionRefusal::MissingLiveFamilyRange`] and
/// [`TransactionRefusal::MissingLiveShapeRanges`] for a handoff short of
/// a placement the ABI must name;
/// [`TransactionRefusal::MissingLiveDeploymentSymbol`] and
/// [`TransactionRefusal::MalformedLiveDeploymentSymbol`] for a symbol
/// with no definition or a definition of the wrong kind;
/// [`TransactionRefusal::LiveCoordinatorNotAtAnchor`] when a shape's
/// receipt run does not begin at input zero;
/// [`TransactionRefusal::OwnerKeyIsNotACurvePoint`] for a committed
/// owner whose key names no point;
/// [`TransactionRefusal::InheritedObligationUnaccounted`] when the
/// disposition does not partition the handoff's own set; and any failure
/// of [`derive_live_receipt_instance`].
pub fn derive_live_transfer_abi(
    target: &ReviewedElementsTapscriptDefinition,
    bundle: &CandidateLinkedLiveTransferBundle,
    curve: &dyn LiveCurveCapability,
) -> Result<CandidateLiveTransferAbi, TransactionRefusal> {
    if bundle.status() != LinkedArtifactStatus::Prototype {
        return Err(TransactionRefusal::LiveBundleIsNotACandidate);
    }
    if bundle.contract() != target.definition().version() {
        return Err(TransactionRefusal::LiveContractRevisionMismatch);
    }

    let handoff = bundle.abi_handoff();
    let coordinator = LiveCoordinatorRule { index: 0 };

    let mut shapes = BTreeMap::new();
    for shape in handoff.shapes().shapes() {
        let ranges = handoff
            .family_ranges()
            .get(&shape)
            .ok_or(TransactionRefusal::MissingLiveShapeRanges { shape })?;
        shapes.insert(shape, shape_abi(shape, ranges, coordinator)?);
    }

    let destinations = destination_table(target, bundle, curve)?;
    let symbols = read_symbols(bundle)?;
    let profile = bundle.sighash_profile().clone();
    let protected: BTreeSet<_> = profile
        .profile()
        .coverage()
        .map(|(datum, _)| datum)
        .collect();
    let inherited = disposition(handoff.owed())?;

    Ok(CandidateLiveTransferAbi {
        contract: bundle.contract(),
        leaf_version: leaf_version(bundle)?,
        representations: bundle.representation_plans().clone(),
        bounds: handoff.bounds(),
        shapes,
        destinations,
        coordinator,
        ordering: LiveCanonicalOrdering::AscendingOutpoint,
        witness_order: LiveWitnessItem::ORDER.to_vec(),
        witness_roles: handoff.witness_roles().clone(),
        profile,
        protected,
        symbols,
        lock_time: LIVE_TRANSFER_LOCK_TIME,
        residuals: bundle.residuals().clone(),
        inherited,
        obligations: OutstandingLiveAbiObligations {
            least: LiveAbiObligation::TargetExecutionEvidenceAbsent,
            rest: BTreeSet::from([
                LiveAbiObligation::SelectedSighashProfileUnreviewed,
                LiveAbiObligation::InternalKeyUnspendabilityUnverified,
                LiveAbiObligation::ConfidentialFieldFormSettledOnlyOnTheTarget,
            ]),
        },
    })
}

/// What this derivation does about each obligation the handoff owed.
///
/// Two are discharged and none is carried, and the partition is checked
/// against the handoff rather than written down: an obligation the link
/// adds later and this function does not name fails the check instead of
/// disappearing.
fn disposition(
    owed: &BTreeSet<LiveLinkObligation>,
) -> Result<InheritedLinkObligations, TransactionRefusal> {
    // Discharged here: the destination constructor table selects among
    // the linked placements (§12.3), and the output key is the tweak of
    // the committed root the capability performed.
    let discharged: BTreeSet<_> = [
        LiveLinkObligation::DestinationConstructorTableUndischarged,
        LiveLinkObligation::TaprootOutputKeyUndischarged,
    ]
    .into_iter()
    .filter(|obligation| owed.contains(obligation))
    .collect();

    let carried: BTreeSet<_> = owed.difference(&discharged).copied().collect();

    let accounted: BTreeSet<_> = discharged.union(&carried).copied().collect();
    if &accounted != owed {
        return Err(TransactionRefusal::InheritedObligationUnaccounted);
    }

    Ok(InheritedLinkObligations {
        discharged,
        carried,
    })
}

/// One shape's layout, read off the link's family ranges.
fn shape_abi(
    shape: LiveTransferShape,
    ranges: &CompleteFamilyRanges,
    coordinator: LiveCoordinatorRule,
) -> Result<LiveShapeAbi, TransactionRefusal> {
    let run = |family: LiveFamily| -> Result<(u16, u16), TransactionRefusal> {
        ranges
            .range(family)
            .map(|range| (range.first(), range.end()))
            .ok_or(TransactionRefusal::MissingLiveFamilyRange { shape, family })
    };

    let coordinator_run = run(LiveFamily::Input(LiveInputFamily::Coordinator))?;
    if coordinator_run.0 != coordinator.index {
        return Err(TransactionRefusal::LiveCoordinatorNotAtAnchor {
            placed: coordinator_run.0,
        });
    }

    // The receipt family is the coordinator's own position followed by
    // the member run, and a one-input shape has no member run at all —
    // which is why the end is taken from whichever family actually
    // reaches furthest rather than from the member range alone.
    let member_run = ranges
        .range(LiveFamily::Input(LiveInputFamily::Member))
        .map_or(coordinator_run, |range| (range.first(), range.end()));
    let receipt_input_range = (coordinator_run.0, coordinator_run.1.max(member_run.1));

    let sponsor_input_range = ranges
        .range(LiveFamily::Input(LiveInputFamily::Sponsor))
        .map_or((receipt_input_range.1, receipt_input_range.1), |range| {
            (range.first(), range.end())
        });

    let destination_range = run(LiveFamily::Output(LiveOutputFamily::Destination))?;
    let sponsor_change_position = position(ranges, LiveOutputFamily::SponsorChange);
    let fee_position = position(ranges, LiveOutputFamily::TargetFee);

    let form = if shape.sponsored() {
        LiveTransactionForm::Sponsored
    } else {
        LiveTransactionForm::Sponsorless
    };

    // The sponsorless form pays no fee, so nothing relays it on its own
    // and it travels as a package child — the same version the
    // compact-ASH ABI reaches for the same reason.
    let version = match form {
        LiveTransactionForm::Sponsored => TargetTransactionVersion::Standard,
        LiveTransactionForm::Sponsorless => TargetTransactionVersion::TopologyRestricted,
    };

    Ok(LiveShapeAbi {
        shape,
        ranges: ranges.clone(),
        coordinator,
        receipt_input_range,
        sponsor_input_range,
        destination_range,
        sponsor_change_position,
        fee_position,
        form,
        version,
    })
}

/// Where a single-position output family sits, when it has one.
fn position(ranges: &CompleteFamilyRanges, family: LiveOutputFamily) -> Option<u16> {
    ranges
        .range(LiveFamily::Output(family))
        .filter(|range| range.count() > 0)
        .map(LiveFamilyRange::first)
}

/// Every destination owner's constructor and instance.
fn destination_table(
    target: &ReviewedElementsTapscriptDefinition,
    bundle: &CandidateLinkedLiveTransferBundle,
    curve: &dyn LiveCurveCapability,
) -> Result<DestinationConstructorTable, TransactionRefusal> {
    let mut entries = BTreeMap::new();
    for ((owner, representation), constructor) in bundle.constructors() {
        // §1.8's second conjunct, at the one place every owner of this
        // deployment passes through. An owner whose key names no point
        // has no destination and no signing request.
        if !curve.owner_key_is_a_curve_point(owner.key().bytes()) {
            return Err(TransactionRefusal::OwnerKeyIsNotACurvePoint {
                owner: owner.clone(),
            });
        }

        entries.insert(
            (owner.clone(), *representation),
            LiveDestinationConstructor {
                placement: constructor.placement(),
                instance: derive_live_receipt_instance(target, constructor, curve)?,
            },
        );
    }

    Ok(DestinationConstructorTable { entries })
}

/// The leaf version every linked constructor agrees on.
fn leaf_version(
    bundle: &CandidateLinkedLiveTransferBundle,
) -> Result<LeafVersion, TransactionRefusal> {
    let mut versions = bundle
        .constructors()
        .values()
        .map(linker::LinkedLiveConstructor::leaf_version);
    let first = versions
        .next()
        .ok_or(TransactionRefusal::LiveBundleIsNotACandidate)?;
    if versions.any(|version| version != first) {
        return Err(TransactionRefusal::LiveLeafVersionDisagreement);
    }
    Ok(first)
}

/// The deployment constants, read from the link's definition census.
fn read_symbols(
    bundle: &CandidateLinkedLiveTransferBundle,
) -> Result<LiveDeploymentSymbols, TransactionRefusal> {
    let census = bundle
        .definitions()
        .values()
        .next()
        .ok_or(TransactionRefusal::LiveBundleIsNotACandidate)?;

    let value = |symbol: LiveLinkSymbol,
                 rendered: &'static str|
     -> Result<&LiveSymbolValue, TransactionRefusal> {
        census
            .definition(&symbol)
            .map(linker::LiveSymbolDefinition::value)
            .ok_or(TransactionRefusal::MissingLiveDeploymentSymbol { symbol: rendered })
    };

    let asset = |symbol, rendered| -> Result<AssetId, TransactionRefusal> {
        match value(symbol, rendered)? {
            LiveSymbolValue::Asset(item) => AssetId::from_slice(item.bytes()),
            _ => Err(TransactionRefusal::MalformedLiveDeploymentSymbol { symbol: rendered }),
        }
    };

    let version = |symbol, rendered| -> Result<u8, TransactionRefusal> {
        match value(symbol, rendered)? {
            LiveSymbolValue::ScriptNumber(number) => u8::try_from(*number).map_err(|_| {
                TransactionRefusal::MalformedLiveDeploymentSymbol { symbol: rendered }
            }),
            _ => Err(TransactionRefusal::MalformedLiveDeploymentSymbol { symbol: rendered }),
        }
    };

    let program = |symbol, rendered| -> Result<Vec<u8>, TransactionRefusal> {
        match value(symbol, rendered)? {
            LiveSymbolValue::WitnessProgram(item) | LiveSymbolValue::ProgramDigest(item) => {
                Ok(item.bytes().to_vec())
            }
            _ => Err(TransactionRefusal::MalformedLiveDeploymentSymbol { symbol: rendered }),
        }
    };

    Ok(LiveDeploymentSymbols {
        protocol_asset: asset(LiveLinkSymbol::ProtocolAsset, "protocol asset")?,
        reserve_asset: asset(LiveLinkSymbol::ReserveAsset, "reserve asset")?,
        destination_program_version: version(
            LiveLinkSymbol::DestinationProgramVersion,
            "destination program version",
        )?,
        sponsor_change_program: program(
            LiveLinkSymbol::SponsorChangeProgram,
            "sponsor change program",
        )?,
        sponsor_change_version: version(
            LiveLinkSymbol::SponsorChangeProgramVersion,
            "sponsor change program version",
        )?,
        fee_program_digest: program(LiveLinkSymbol::TargetFeeRole, "target fee role")?,
    })
}
