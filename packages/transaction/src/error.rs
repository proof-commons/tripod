//! Why an ABI was not derived, or a transaction not constructed.
//!
//! Construction failure, never target rejection (§1.5). Every variant
//! below names something wrong with what this crate was asked to build
//! or asked to read, and none of them is a statement about what a
//! target node would do with a well-formed result. A refusal returns no
//! partial value (§1.11): there is no variant carrying a half-built
//! transaction, and no entry point returns one alongside a diagnostic.
//!
//! The decoder's refusals sit here too, and they are the same kind of
//! thing. Bytes this crate cannot turn into validated typed values are
//! refused rather than partially interpreted, because §1.12 admits
//! external target bytes only through an explicit parser that converts
//! them immediately into validated typed values — and a parser that
//! ignored a field it did not model would be converting them into
//! something else.

use std::collections::BTreeSet;

use linker::backend::{CompactAshShape, InputRole, LeafRole, OutputRole};
use linker::live_backend::{
    LiveFamily, LiveTransferLeafRole, LiveTransferRepresentationPlan, LiveTransferShape,
};
use linker::{OwnerParameter, StateLinkRefusal};
use realization::MaturityTransitionRefusal;
use tapscript::StateConstructorRefusal;
use target_elements::{ResourceDimension, TransactionForm};

use crate::bytes::Outpoint;
use crate::live_materialize::{ConfidentialInputRegion, ConfidentialOutputRole};
use crate::operator_signing::OperatorSigningRefusal;
use crate::state_view::MaturityViewEntry;

/// Why the transaction layer refused.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TransactionRefusal {
    // --- ABI derivation ----------------------------------------------
    /// The bundle handed in claims more than a candidate link.
    ///
    /// The transaction ABI is a candidate ABI, and §1.9 keeps the
    /// states distinct in both directions: a candidate ABI derived from
    /// something claiming to be final would blur the boundary just as
    /// surely as a final ABI derived from a candidate bundle.
    BundleIsNotACandidate,
    /// The bundle's linked constructor is bound to a different reviewed
    /// contract revision than the target handed in.
    ContractRevisionMismatch,
    /// The bundle carries no layout for a shape it admits.
    MissingLayout(CompactAshShape),
    /// The bundle carries no linked program for a leaf its tree commits
    /// to, or no control-path recipe for a leaf it linked.
    MissingLeaf(LeafRole),
    /// A shape's layout does not place a role the ABI must name.
    MissingInputRole {
        /// The shape whose layout is short.
        shape: CompactAshShape,
        /// The role no run holds.
        role: InputRole,
    },
    /// A shape's layout does not place an output role the ABI must
    /// name.
    MissingOutputRole {
        /// The shape whose layout is short.
        shape: CompactAshShape,
        /// The role no position holds.
        role: OutputRole,
    },
    /// A deployment symbol the ABI needs has no linked definition.
    MissingDeploymentSymbol {
        /// A rendering of the symbol, for the report.
        symbol: &'static str,
    },
    /// A deployment symbol resolved to a value of the wrong kind.
    MalformedDeploymentSymbol {
        /// A rendering of the symbol, for the report.
        symbol: &'static str,
    },
    /// The linked coordinator leaf requires the anchor at an index the
    /// canonical layout does not put it at.
    CoordinatorNotAtAnchor {
        /// Where the layout puts the coordinator.
        placed: u16,
    },

    // --- The pinned instance -----------------------------------------
    /// The pinned ASH program is not the reviewed witness-v1 form.
    ///
    /// A pin is the one place a caller states which deployed object
    /// this ABI builds against, so its shape is checked exactly rather
    /// than trusted: a pin of the wrong form would produce a
    /// transaction that spends nothing.
    MalformedPinnedProgram {
        /// How many bytes the offered program occupies.
        offered: usize,
    },
    /// The pinned instance names an internal key other than the one the
    /// bundle linked.
    PinnedInternalKeyMismatch,
    /// The pinned instance names a leaf version other than the one the
    /// committed tree carries.
    PinnedLeafVersionMismatch {
        /// The version the pin states.
        pinned: u8,
        /// The version the tree commits under.
        linked: u8,
    },

    // --- Request validation -------------------------------------------
    /// The request selects no ASH input.
    EmptyAshSelection,
    /// The request selects one outpoint more than once.
    DuplicateOutpoint(Outpoint),
    /// The public view states one outpoint more than once.
    ///
    /// Every second statement is refused, agreeing or contradicting.
    /// A contradictory pair has no resolution a constructor could
    /// justify — believing either one is believing the order the caller
    /// listed them in — and an agreeing pair is a census the caller got
    /// wrong about a boundary whose exactness the rest of the pipeline
    /// rests on, so the two are one refusal rather than a judgement
    /// about which duplicates are harmless.
    DuplicatePublicOutputView(Outpoint),
    /// A sponsor offer names one outpoint more than once.
    ///
    /// Refused rather than collapsed into a smaller offer, for the
    /// reason [`Self::DuplicateOutpoint`] gives about the ASH
    /// selection: a set built by insertion would answer a caller
    /// naming one coin twice with a sponsor region it did not ask for.
    DuplicateSponsorOutpoint(Outpoint),
    /// An outpoint appears in both the ASH selection and the sponsor
    /// suffix, which would put one input in two regions.
    OverlappingOutpoint(Outpoint),
    /// No shape the candidate admits takes these counts at the form the
    /// request asks for.
    UnsupportedShape {
        /// How many ASH inputs the request selects.
        ash_inputs: usize,
        /// How many sponsor inputs it selects.
        sponsor_inputs: usize,
        /// Whether it selects a sponsor change destination.
        sponsor_change: bool,
    },
    /// A selected ASH input's public view states an asset other than
    /// the closed protocol asset.
    AshInputCarriesForeignAsset(Outpoint),
    /// A selected ASH input's public view states a program other than
    /// the pinned one.
    AshInputCarriesForeignProgram(Outpoint),
    /// A selected ASH input's public view states no explicit amount.
    ///
    /// The fixed Phase-4 representation is explicit `U`, so an ASH
    /// whose amount is a commitment is not an ASH this candidate can
    /// consolidate — and refusing it is not a claim that the target
    /// would reject it.
    AshInputAmountNotExplicit(Outpoint),
    /// A selected sponsor input's public view states a confidential
    /// asset.
    ///
    /// The reviewed profile forces a sponsor asset explicit, because
    /// the coordinator authenticates it. Sponsor *value* opacity
    /// survives; sponsor asset opacity does not exist under this
    /// relation.
    SponsorInputAssetNotExplicit(Outpoint),
    /// A selected sponsor input carries an asset other than the reserve
    /// asset.
    SponsorInputCarriesForeignAsset(Outpoint),
    /// A selected sponsor input's program is not an admitted class.
    SponsorProgramClassNotAdmitted(Outpoint),
    /// The request selects a sponsor change destination but no sponsor
    /// input.
    SponsorChangeWithoutSponsor,
    /// The request asks for a sponsor suffix and no capability was
    /// supplied to fill it.
    ///
    /// A refusal rather than a quiet downgrade to the sponsorless form.
    /// The two forms have different relay verdicts and different
    /// versions, so building the other one would answer a question the
    /// caller did not ask.
    SponsorRequestedWithoutCapability,
    /// A sponsor capability was supplied and the request asks for no
    /// sponsor suffix.
    ///
    /// The converse, and refused for the converse reason: silently
    /// spending a sponsor's inputs because an adapter happened to be in
    /// scope is worse than refusing.
    SponsorCapabilityWithoutRequest,
    /// The capability filling a sponsor suffix offered no input.
    ///
    /// The third term of the same equivalence. A suffix is a region of
    /// inputs, so an offer of none names no suffix: the declared fee
    /// would have no position to occupy, no signing request would be
    /// issued, and the counts would select the sponsorless shape —
    /// which is the quiet downgrade
    /// [`Self::SponsorRequestedWithoutCapability`] exists to refuse,
    /// reached from the other side.
    EmptySponsorOffer,
    /// The consolidated successor amount overflows the target's
    /// checked range.
    SuccessorAmountOutOfRange,
    /// The declared fee is not covered by the sponsor inputs the
    /// request selects.
    ///
    /// Raised only where every selected sponsor value is explicit. A
    /// sponsor whose value stays committed is not compared with
    /// anything, and this refusal is unreachable for it — which is the
    /// erasure law holding rather than a gap in the check.
    SponsorValueDoesNotCoverFee,

    // --- Signing ------------------------------------------------------
    //
    // §15.8's "no signing request before protected outputs are final"
    // has no variant here on purpose. It is enforced structurally
    // rather than checked: a `SponsorSigningRequest` carries the exact
    // finalized bytes and its constructor is crate-internal, so there
    // is no way to build one from a template and no state in which the
    // violation could be observed. A variant for it would be a refusal
    // whose failing branch could only be reasoned about.
    /// A sponsor capability returned no signature for an input it was
    /// asked to sign.
    SponsorSignatureMissing(Outpoint),
    /// A sponsor capability returned a signature against a transaction
    /// other than the one it was asked to sign.
    SponsorSignatureBindingMismatch(Outpoint),
    /// A sponsor capability returned a witness stack the admitted
    /// program class does not take.
    SponsorWitnessShapeRefused {
        /// The input.
        outpoint: Outpoint,
        /// How many items the capability returned.
        offered: usize,
        /// How many the admitted class takes.
        expected: usize,
    },

    // --- Preflight ----------------------------------------------------
    /// The assembled transaction does not conserve one asset.
    ///
    /// Only assets whose every input and output field is explicit are
    /// checked, and the check is a construction check rather than a
    /// consensus verdict.
    ConservationFailed {
        /// A rendering of the asset, for the report.
        asset: String,
    },
    /// The assembled transaction exceeds a bound the deployment states.
    ResourceBoundExceeded {
        /// The dimension.
        dimension: ResourceDimension,
        /// The figure the transaction reaches.
        reached: u64,
        /// The bound.
        bound: u64,
    },
    /// The assembled transaction's role census does not match the
    /// layout the ABI states for its shape.
    LayoutCensusMismatch {
        /// The shape.
        shape: CompactAshShape,
        /// The roles the layout places and the assembly does not.
        missing: BTreeSet<OutputRole>,
    },

    // --- Decoding target bytes ----------------------------------------
    /// The byte string ended inside a field.
    TruncatedTargetBytes {
        /// The offset the read began at.
        at: usize,
    },
    /// The byte string held more than one transaction's bytes.
    TrailingTargetBytes {
        /// Where the transaction ended.
        at: usize,
    },
    /// A count was encoded in a wider form than it needs.
    NonMinimalCompactSize {
        /// Where the count began.
        at: usize,
        /// The value it encoded.
        value: u64,
    },
    /// A field carried a prefix byte the reviewed encoding does not
    /// define.
    UnrecognizedFieldPrefix {
        /// The offered byte.
        prefix: u8,
    },
    /// The flag byte was neither of the two the target writes.
    UnrecognizedWitnessFlag {
        /// The offered byte.
        offered: u8,
    },
    /// The witness flag stands over a witness section carrying nothing.
    ///
    /// The target sets the flag only where some witness is present, so
    /// a transaction whose every witness is empty has one encoding and
    /// it is the witnessless one. Accepting the flagged spelling as
    /// well would admit two byte strings for one typed transaction,
    /// which is what [`Self::NonMinimalCompactSize`] refuses about a
    /// count and what the round-trip law forbids about a transaction.
    SuperfluousWitnessRecord,
    /// An asset identifier was not the reviewed width.
    MalformedAssetIdentifier {
        /// How many bytes were offered.
        offered: usize,
    },
    /// An outpoint index occupies a bit the target reserves for a
    /// marker.
    OutpointIndexOutOfRange {
        /// The offered index.
        offered: u32,
    },
    /// The bytes carry an issuance, which is outside the candidate.
    ///
    /// The byte-level half of the elected exclusion
    /// `(´[PLAN-rule:exclusions:issuance-bytes]´)`, and one of the
    /// rows that enforce the refused issuance dimension
    /// `(´[PLAN-rule:exclusions:issuance-dimension]´)`.
    IssuanceInputRefused,
    /// The bytes carry a peg-in, which is outside the candidate.
    ///
    /// A peg-in is a second origin for the conserved asset, so the
    /// conservation the covenant checks would be over a quantity part
    /// of which entered from outside the model
    /// `(´[PLAN-rule:exclusions:pegin]´)`.
    PeginInputRefused,
    /// The bytes carry an issuance proof, which is outside the
    /// candidate.
    ///
    /// The witness half of `(´[PLAN-rule:exclusions:issuance-bytes]´)`,
    /// enforcing the refused dimension
    /// `(´[PLAN-rule:exclusions:issuance-dimension]´)` alongside the
    /// outpoint marker's own refusal.
    IssuanceProofRefused,
    /// The bytes carry a surjection proof, which is outside the
    /// candidate.
    ///
    /// Unconditional, and that is the form-conditional answer rather
    /// than an exception to it: the one confidential form this
    /// workspace constructs pairs a committed value with an explicit
    /// asset, which is the unblinded-generator case the target requires
    /// the surjection field to be empty for. A byte string carrying one
    /// is a blinded-asset form nothing here builds.
    ///
    /// The decoder's half of the elected asset-blinding exclusion
    /// `(´[PLAN-rule:exclusions:asset-blinding]´)`.
    SurjectionProofRefused,
    /// The bytes carry a range proof for an output whose value form
    /// forbids one.
    ///
    /// An explicit value commits to nothing, so a proof about its range
    /// proves nothing, and the target's own validation never reaches a
    /// check that would consume one.
    RangeProofRefused,
    /// The bytes omit a range proof for an output whose value form
    /// requires one.
    ///
    /// The other half of the same rule, and the half a decoder written
    /// for explicit outputs alone had no occasion to own. A committed
    /// value with no range proof is refused by the target as invalid,
    /// so a decoder that returned one would be handing back a
    /// transaction no chain accepts and calling it well formed.
    RangeProofRequired {
        /// Which output position omitted it.
        output: usize,
    },
    /// A transaction was assembled with an output-witness census that
    /// is not one entry per output.
    ///
    /// The input side's reason exactly: the encoding is positional, so
    /// a shorter or longer vector binds a proof to the wrong output and
    /// still produces well-formed bytes.
    OutputWitnessCensusMismatch {
        /// How many outputs.
        outputs: usize,
        /// How many output witnesses.
        output_witnesses: usize,
    },
    /// A transaction was assembled with no input.
    EmptyInputCensus,
    /// A transaction was assembled with no output.
    EmptyOutputCensus,
    /// A transaction was assembled with a witness census that is not
    /// one entry per input.
    WitnessCensusMismatch {
        /// How many inputs.
        inputs: usize,
        /// How many witnesses.
        witnesses: usize,
    },

    // --- Live-transfer ABI derivation --------------------------------
    /// The live bundle handed in claims more than a candidate link.
    LiveBundleIsNotACandidate,
    /// The live bundle's constructors are bound to a different reviewed
    /// contract revision than the target handed in.
    LiveContractRevisionMismatch,
    /// Two linked live constructors disagree about the leaf version.
    LiveLeafVersionDisagreement,
    /// A linked live constructor's tree commits to a leaf it linked no
    /// program for.
    MissingLiveLeaf(LiveTransferLeafRole),
    /// The handoff carries no family ranges for a shape it admits.
    MissingLiveShapeRanges {
        /// The shape whose placement is absent.
        shape: LiveTransferShape,
    },
    /// A shape's ranges do not place a family the ABI must name.
    MissingLiveFamilyRange {
        /// The shape whose placement is short.
        shape: LiveTransferShape,
        /// The family no run holds.
        family: LiveFamily,
    },
    /// A shape's receipt family does not begin at the coordinator
    /// anchor.
    LiveCoordinatorNotAtAnchor {
        /// Where the ranges put the first receipt input.
        placed: u16,
    },
    /// A live deployment symbol the ABI needs has no linked definition.
    MissingLiveDeploymentSymbol {
        /// A rendering of the symbol, for the report.
        symbol: &'static str,
    },
    /// A live deployment symbol resolved to a value of the wrong kind.
    MalformedLiveDeploymentSymbol {
        /// A rendering of the symbol, for the report.
        symbol: &'static str,
    },
    /// The linked internal key is not the reviewed x-only width.
    LiveInternalKeyMalformed {
        /// How many bytes were linked.
        offered: usize,
    },
    /// A committed owner's key is the approved encoding and names no
    /// point of the target's curve (§1.8).
    OwnerKeyIsNotACurvePoint {
        /// The owner whose key names none.
        owner: OwnerParameter,
    },
    /// The capability determines no output key for a committed tree.
    DestinationOutputKeyUndetermined,
    /// The obligation disposition does not partition the link's own set.
    ///
    /// Reached when the link owes an obligation this derivation neither
    /// discharges nor carries — which is to say, when a wave added one
    /// and this one did not notice. The live-transfer derivation checks
    /// its handoff's set this way, and the maturity-announcement
    /// derivation checks the linked bundle's own five the same way; the
    /// finding is one finding in both, which is why it has one name.
    InheritedObligationUnaccounted,

    // --- Live-transfer request ---------------------------------------
    /// A destination was asked for at a semantic value of zero.
    DestinationValueIsZero,
    /// A live transfer was requested consuming no receipt.
    EmptyReceiptSelection,
    /// One receipt outpoint is named more than once (§12.1).
    DuplicateReceiptOutpoint(Outpoint),
    /// A live transfer was requested creating no destination.
    EmptyDestinationCensus,
    /// A sponsorless request asks for the sponsor-change role (§12.5).
    ///
    /// Stated of any request rather than of one generation, because a
    /// form carrying no sponsor region has no residual for a change
    /// output to take, which is why the maturity-announcement request of
    /// §12.3 refuses the same pair.
    SponsorChangeWithoutSponsoredForm,
    /// The explicit lane's declared destination roles are neither empty
    /// nor one role per destination entry.
    DeclaredRolesDoNotCoverDestinations {
        /// How many roles the caller declared.
        declared: usize,
        /// How many destination entries the request carries.
        destinations: usize,
    },
    /// A sponsored request declares a self-paid fee entry, whose fee is
    /// the sponsor's to place from the offer instead (§12.5).
    SelfPaidFeeUnderSponsoredForm,
    /// More than one destination entry was declared the fee, where the
    /// target admits exactly one fee position.
    SelfPaidFeeDeclaredMoreThanOnce {
        /// How many entries were declared the fee.
        declared: usize,
    },
    /// A fee entry was declared against a shape that carries no fee
    /// position at all.
    SelfPaidFeeHasNoShapePosition,
    /// The declared fee entry sits at a position the selected shape does
    /// not name for the fee.
    SelfPaidFeePositionDisagreesWithShape {
        /// The position the entry's own index puts it at.
        declared: u16,
        /// The position the shape names for the fee.
        shaped: u16,
    },
    /// An explicit request offers public test randomness, which only a
    /// private construction consumes.
    PublicTestRandomnessWithoutPrivateForm,
    /// A private-committed request offers no public test randomness.
    PrivateFormWithoutPublicTestRandomness,
    /// A request names a destination owner no linked constructor
    /// commits to.
    DestinationOwnerHasNoConstructor {
        /// The owner nothing was linked for.
        owner: OwnerParameter,
    },
    /// The requested representation has no linked constructor at all.
    RepresentationNotLinked,

    // --- Live-transfer form exactness (§12.5) -------------------------
    /// A sponsored request was offered no sponsor capability.
    LiveSponsorRequestedWithoutCapability,
    /// A sponsorless request was offered a sponsor capability.
    LiveSponsorCapabilityWithoutRequest,
    /// A sponsored request's capability offers no sponsor input.
    ///
    /// The third term of the equivalence, closing the downgrade from
    /// the side an empty capability would open: an offer with no input
    /// funds no fee, and accepting it would build the sponsorless form
    /// for a request that asked for the sponsored one.
    EmptyLiveSponsorOffer,
    /// A sponsored request's capability offers no change destination
    /// while the request asks for the change role.
    SponsorChangeRequestedWithoutDestination,
    /// A sponsorless request's capability offers a change destination.
    SponsorChangeOfferedWithoutRequest,
    /// The selected shape's form is not the form the request asked for.
    LiveFormDisagreesWithShape,

    // --- Live-transfer construction ----------------------------------
    /// No admitted shape realizes the requested counts.
    UnsupportedLiveShape {
        /// How many receipts are consumed.
        receipt_inputs: usize,
        /// How many destinations are created.
        destinations: usize,
        /// How many sponsor inputs the capability offers.
        sponsor_inputs: usize,
        /// Whether the sponsor takes change.
        sponsor_change: bool,
    },
    /// One outpoint is offered as both a receipt and a sponsor input
    /// (§12.1).
    SponsorOverlapsReceiptFamily(Outpoint),
    /// A selected receipt outpoint has no public view.
    MissingPublicReceiptView(Outpoint),
    /// A selected receipt's program is no linked constructor's output.
    ReceiptInputIsNotALiveReceipt(Outpoint),
    /// A selected receipt carries an asset other than the protocol
    /// asset.
    ReceiptInputCarriesForeignAsset(Outpoint),
    /// A selected receipt's value field is not the form the requested
    /// representation reads.
    ReceiptInputValueFormRefused(Outpoint),
    /// An offered sponsor input has no public view.
    ///
    /// Distinct from [`Self::LiveSponsorInputCarriesForeignAsset`] on
    /// purpose. The compact-ASH lane folds the two together, answering
    /// "foreign asset" for a coin whose asset it never saw; that reads
    /// as a claim about the coin when it is a statement about the
    /// caller's own view. This lane already keeps the pair apart for
    /// receipts, and a sponsor input is offered by the same caller from
    /// the same view.
    MissingPublicSponsorView(Outpoint),
    /// A sponsor input carries an asset other than the reserve asset.
    LiveSponsorInputCarriesForeignAsset(Outpoint),
    /// The destinations' semantic total overflows the target's explicit
    /// width.
    DestinationTotalOutOfRange,
    /// The consumed and created explicit totals are not equal.
    ///
    /// Construction failure, never target rejection (§1.11): the
    /// coordinator's own arithmetic disagrees before any target sees the
    /// transaction.
    LiveConservationFailed {
        /// What the consumed receipts carry.
        consumed: u64,
        /// What the destinations create.
        created: u64,
    },
    /// A receipt position lies outside the shape's receipt family.
    ReceiptPositionOutsideFamily {
        /// The offered position.
        position: u16,
    },
    /// The private construction was asked for under a model this
    /// materializer does not implement (§12.8).
    ConfidentialConstructionModelNotAdmitted,
    /// A private-committed request was offered no confidential value
    /// capability.
    PrivateValueCapabilityAbsent,
    /// An explicit request was offered a confidential value capability.
    PrivateValueCapabilityWithoutPrivateForm,
    /// The confidential capability determines no commitment for one
    /// destination.
    DestinationValueCommitmentUndetermined {
        /// The output position with no field.
        position: u16,
    },
    /// The transaction-wide private finalization was handed a request
    /// whose representation is not the private one.
    ///
    /// Its own refusal rather than a reuse of the explicit lane's,
    /// because the two lanes now have two entry points and a caller
    /// standing at the wrong one should be told which one it is at
    /// (rule:guide-ctf-exec:per-output-retirement).
    PrivateFinalizationIsNotTheExplicitLane {
        /// The representation the request selected.
        representation: LiveTransferRepresentationPlan,
    },
    /// The openings offered do not cover the request's inputs, or its
    /// destinations, one for one.
    ///
    /// The input side counts the WHOLE input order and not the receipts
    /// alone. A sponsored private candidate consumes the sponsor's coin
    /// after its receipts, that coin is a spent predecessor output like
    /// any other, and it needs the same opening — so a request that
    /// offers openings for its receipts only is short here rather than
    /// discovered to be short by a materializer reading past its own
    /// slice.
    PrivateOpeningsDoNotCoverTheRequest {
        /// How many entries the openings carry.
        offered: usize,
        /// How many the request needs.
        required: usize,
    },
    /// A private input opening declares a region its position cannot be.
    ///
    /// The input order is receipts first and the sponsor suffix after,
    /// which is the canonical order the explicit lane sorts into and the
    /// order the materializer's regions are read in. The caller declares
    /// each opening's region rather than having it inferred — the
    /// materializer's own reason, that a region inferred from the asset
    /// would make the balance depend on a comparison the balance is
    /// trying to decide — and a declaration is worth having only where
    /// disagreeing with it is refused.
    ///
    /// Raised in preference to silently re-sorting, because the openings
    /// carry blinders and an opening moved to another position is an
    /// opening applied to a coin it does not open.
    PrivateOpeningRegionDisagreesWithPosition {
        /// The input position, receipts first.
        position: usize,
        /// The region the opening declared.
        stated: ConfidentialInputRegion,
        /// The region that position holds.
        expected: ConfidentialInputRegion,
    },
    /// A declared sponsor-region output states an amount the sponsor's
    /// offer does not.
    ///
    /// The private lane declares its fee and its sponsor change as
    /// destination positions, so their amounts arrive from the REQUEST.
    /// The sponsor states the same two numbers in its OFFER, which is
    /// where the explicit lane reads them from. Two sources for one
    /// number is a disagreement waiting to happen, and it is refused
    /// here rather than left to a node.
    ///
    /// What a node would say instead is worth naming, because it is why
    /// this is not merely tidy. The reserve sub-equation is the sponsor
    /// input against the fee and the change, so a declared change that
    /// is not the offered one produces a transaction that does not
    /// balance — and the node answers with its balance check, a true
    /// sentence about arithmetic that says nothing about the request
    /// having asked for two different numbers.
    PrivateSponsorRegionAmountDisagreesWithOffer {
        /// Which of the two sponsor-region roles disagreed.
        role: ConfidentialOutputRole,
        /// What the request's destination declared.
        declared: u64,
        /// What the sponsor's offer stated.
        offered: u64,
    },
    /// The transaction-wide materializer refused the private candidate.
    ///
    /// Wrapped rather than flattened, on the pattern the accepted-result
    /// vocabulary already sets: the materializer's census is far finer
    /// than anything this enum should re-spell, and a caller that wants
    /// the detail should get the materializer's own word for it.
    PrivateMaterializationRefused(Box<crate::live_materialize::MaterializationRefusal>),

    // --- Live-transfer owner signing (§12.7) --------------------------
    /// A required owner offered no response.
    OwnerResponseMissing {
        /// The input whose owner is silent.
        input: u16,
    },
    /// One input's response was offered twice.
    OwnerResponseDuplicated {
        /// The input answered twice.
        input: u16,
    },
    /// A response was offered for an input the transfer does not
    /// require one for.
    UnexpectedSigner {
        /// The input nobody asked about.
        input: u16,
    },
    /// A response names an owner other than the one the input
    /// authenticates.
    ResponseFromWrongOwner {
        /// The input whose owner was misnamed.
        input: u16,
    },
    /// A response answers a signing request for another input.
    ResponseForWrongInput {
        /// The input the response was collected against.
        input: u16,
    },
    /// A response was taken under a profile other than the selected
    /// one.
    ResponseUnderWrongSighashProfile {
        /// The input whose response used another profile.
        input: u16,
    },
    /// A response is bound to bytes other than the finalized ones.
    ResponseBoundToDifferentBytes {
        /// The input whose response commits elsewhere.
        input: u16,
    },
    /// An output was changed after the finalization boundary.
    OutputMutatedAfterSigning {
        /// The output position that moved.
        position: u16,
    },
    /// An input was added after the finalization boundary.
    InputExtendedAfterSigning {
        /// How many inputs the finalized form fixed.
        finalized: usize,
        /// How many were offered afterwards.
        offered: usize,
    },
    /// An output was removed after the finalization boundary.
    OutputOmittedAfterSigning {
        /// The output position that disappeared.
        position: u16,
    },

    // --- The public current-STATE view (§12.4) -------------------------
    /// One entry of the current-STATE view was stated more than once.
    ///
    /// The compact view's law over a census of entries rather than of
    /// outpoints, and refused for its reason: a second statement is
    /// refused whether it agrees with the first or contradicts it. A
    /// contradictory pair has no resolution this layer could justify,
    /// because believing either statement is believing the order the
    /// caller listed them in; an agreeing pair is a caller who has lost
    /// count of a boundary whose exactness everything downstream reads.
    DuplicateMaturityViewEntry(MaturityViewEntry),
    /// One entry of the current-STATE view was never stated.
    ///
    /// A view is complete or it is not a view. An entry defaulted or
    /// inferred would be a fact this layer invented on a caller's behalf
    /// and then read back as though the caller had supplied it.
    MissingMaturityViewEntry(MaturityViewEntry),
    /// The supplied metadata and nonce commit to no program at all.
    ///
    /// Separate from a program that differs, because the two are
    /// different findings: this one says the pair cannot be committed
    /// under the bundle's policy at that nonce — the fixed outer branch
    /// side unsatisfied there, or a curve refusing the tweak — and the
    /// constructor's own refusal travels rather than being flattened,
    /// because the layer that owns the commitment has the precise word
    /// for what it refused.
    MaturityViewCommitmentRefused {
        /// What the commitment refused, in the constructor's vocabulary.
        refusal: StateConstructorRefusal,
    },
    /// The supplied metadata and nonce commit to a program other than the
    /// supplied predecessor program.
    ///
    /// The one thing a first party can check without observing a chain,
    /// and so the one place three of a caller's entries can be caught
    /// disagreeing with each other. Lengths alone are reported: the
    /// recomputed program is a witness program over an output key, and a
    /// refusal is a diagnostic rather than a place to publish key
    /// material.
    MaturityViewProgramNotReconstructed {
        /// How many bytes the supplied predecessor program occupies.
        supplied: usize,
        /// How many bytes the program recomputed from the pair occupies.
        recomputed: usize,
    },

    // --- The maturity-announcement ABI (§12.6, §12.9) ------------------
    /// The reviewed form census publishes no verdict for the form the ABI
    /// must state one for.
    ///
    /// Either the census holds no entry under that form, or the entry it
    /// holds there reviews a different form. Both are the same finding:
    /// the ABI publishes the target's own verdict rather than a copy of
    /// it, so a census it cannot read a verdict out of leaves it with
    /// nothing honest to publish, and a verdict invented in place of one
    /// would be the exact claim about relay behaviour this crate is not
    /// entitled to make.
    MissingReviewedTransactionForm {
        /// The form whose reviewed verdict was wanted.
        form: TransactionForm,
    },

    // --- Maturity-announcement construction (§12.5, §12.7) -------------
    /// The request asks for the sponsored form, which has no carrier.
    ///
    /// Not a malformed request: a request that could not ask for a
    /// sponsor region could not be refused for asking, and the refusal
    /// worth having is the one that names what the form would need. The
    /// reduced announcement leaf carries no sponsor check at all, and
    /// the realization's sponsor relations hold no region of the
    /// transaction until a later refit gives them one — so a builder
    /// emitting sponsored bytes now would emit a form that no leaf and
    /// no model constrains, which is a stronger objection than any
    /// shape check could state.
    SponsoredMaturityFormHasNoCarrier,
    /// The realization refused the semantic maturity transition.
    ///
    /// The refusal travels whole because the transition owns the
    /// question: whether the predecessor was already announced or
    /// already complete, whether the announced cycle sits below the
    /// window or above it, and whether the window could be derived at
    /// all are five findings, and a construction failure that reported
    /// only that the cycle was wrong would discard the four-way
    /// distinction the layer deriving the window is the one able to
    /// make.
    MaturitySuccessorTransitionRefused {
        /// What the transition refused, in the realization's vocabulary.
        refusal: MaturityTransitionRefusal,
    },
    /// The linked bundle refused the successor constructor application.
    ///
    /// Carried whole and never flattened, because the three things this
    /// can be are three different findings: a search exhausted at the
    /// host's budget is a statement about that budget and not about what
    /// the target would accept, a nonce that reconstructs the wrong
    /// program is the view's finding elsewhere, and a later admissible
    /// nonce beyond the budget is a property of a *successful* search
    /// that the evidence states as its residual. A generic failure here
    /// would collapse all three into one word.
    ///
    /// Boxed because the link's vocabulary is large and every result
    /// this crate returns would otherwise carry its width; the refusal
    /// is carried entire, and the box says nothing about its content.
    MaturitySuccessorSearchRefused {
        /// What the application refused, in the link's vocabulary.
        refusal: Box<StateLinkRefusal>,
    },
    /// The predecessor and successor constructors do not share their
    /// fixed construction parameters.
    ///
    /// The no-migration rule, read at the transaction layer: adjacent
    /// states commit under one static root, one internal key and one
    /// leaf version, and this generation implements no migration
    /// between them. The constructor's own refusal travels because
    /// which of the three differs is the whole content of the finding.
    MaturitySuccessorContinuityRefused {
        /// What the equality refused, in the constructor's vocabulary.
        refusal: StateConstructorRefusal,
    },
    /// The view states an asset other than the deployment's own
    /// singleton.
    ///
    /// The successor output carries the asset the leaf's structural
    /// patterns pin, so the caller's statement of the spent output's
    /// asset is a claim held against that definition rather than a
    /// source to build from. Raised equally when the resolved census
    /// defines no asset at all: a definition that is not there is not
    /// the one the view states, and a linked bundle cannot arrange the
    /// absence, so a second variant for it would be a name no test
    /// could reach.
    MaturityDeploymentAssetDisagreesWithView,
    /// The view states an amount other than the deployment's declared
    /// issuance.
    ///
    /// The singleton's whole issuance is what the successor output
    /// carries, and a spend of a different amount is a spend of
    /// something other than the STATE output this ABI builds against.
    /// Raised equally when the census defines no amount, and when it
    /// defines one this layer cannot read as the target's signed
    /// fixed-width integer: neither is the amount the view states, and
    /// neither is reachable through a linked bundle, so all three are
    /// one name for the same reason the asset's is.
    MaturityDeploymentAmountDisagreesWithView,

    // --- Maturity-announcement finalization (§12.7, §13.1, §13.2) ------
    /// The operator signing boundary refused to freeze the finalized
    /// candidate.
    ///
    /// Carried whole and never flattened. A binding pinning another
    /// reviewed revision, a deployment genesis the binding does not
    /// commit to, a curve refusing the committed operator key, a census
    /// clause over the candidate and its spent outputs, and a leaf that
    /// does not hash to the selection are five findings about five
    /// different things, and the boundary that performs the freeze is
    /// the only layer with words for all of them.
    ///
    /// Boxed because the operator vocabulary is large and every result
    /// this crate returns would otherwise carry its width; the refusal
    /// is carried entire, and the box says nothing about its content.
    MaturityAnnouncementSigningRequestRefused {
        /// What the freeze refused, in the operator boundary's
        /// vocabulary.
        refusal: Box<OperatorSigningRefusal>,
    },
    /// The transaction version was changed after finalization.
    ///
    /// Its own name because §12.7 censuses the version as a protected
    /// region of its own. A report naming an output position for it
    /// would name a region that did not move, which is a worse answer
    /// than a wider one: a reader checking the refusal against the
    /// census would look at an output that is exactly as it was fixed.
    MaturityVersionChangedAfterFinalization {
        /// The version the finalized form fixed.
        finalized: u32,
        /// The version offered afterwards.
        offered: u32,
    },
    /// The lock time was changed after finalization.
    ///
    /// A region of its own for the version's reason. The field is inert
    /// in the target's own evaluation of this form — no announcement
    /// leaf reads it, and the final sequence is on every input — which
    /// is why it is fixed at the value imposing no constraint and why a
    /// change to it is a change nobody asked for rather than a
    /// different policy.
    MaturityLockTimeChangedAfterFinalization {
        /// The lock time the finalized form fixed.
        finalized: u32,
        /// The lock time offered afterwards.
        offered: u32,
    },
    /// An offered candidate's protected encoding is not the finalized
    /// one.
    ///
    /// The close over what the region-naming comparisons do not reach.
    /// Inputs are compared by outpoint, so an input's sequence can move
    /// without the census changing size or any output, version or lock
    /// time differing; this is that case, and in the sponsorless
    /// announcement form it is the only one left.
    ///
    /// The offset is where the two encodings first disagree, or the
    /// common prefix's length when one is a prefix of the other. An
    /// offset rather than the bytes: the finding is that they differ,
    /// and a caller offering a candidate already holds every byte of
    /// it.
    MaturityBytesDifferAfterFinalization {
        /// Where the offered encoding first departs from the finalized
        /// one.
        at: usize,
    },
}
