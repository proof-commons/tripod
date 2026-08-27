//! Live-transfer recognition and owner authorization (Guide-13 §10).
//!
//! # What this module emits, and what it refuses to pretend
//!
//! §10.1 and §10.2 are per-input and local, and this module builds them
//! that way: a recognition fragment that reads the input the leaf is
//! executing in, and an authorization fragment that pushes that input's
//! committed owner key and verifies one signature against it. §10.3's
//! placement follows — the coordinator at input 0, members at the
//! nonzero receipt positions, and both performing the local pair like
//! every other receipt input.
//!
//! The transaction-global checks §10.3 also lists are §10.4 through
//! §10.8's patterns, and none of them exists yet. They are named as
//! outstanding slots ([`coordinator_placements`]) rather than invented
//! here or left out, because a coordinator with a check quietly missing
//! and a coordinator with a check declared outstanding look identical
//! from the emitted bytes and are not the same artifact at all.
//!
//! # Why recognition cannot borrow the compact-ASH shortcut
//!
//! [`crate::pattern`] establishes a compact-ASH input's constructor by
//! comparing that input's program with the program of the input the leaf
//! is spending: one family, one constructor, so sameness is the whole
//! claim and no literal is needed for it. A live-receipt constructor is
//! *owner-parameterized* (§7.6), so two receipt inputs held by two
//! owners carry two different programs, and the comparison has nothing
//! to say.
//!
//! Nor can the coordinator push a literal for another input's program
//! instead. Owner A's coordinator leaf would have to contain owner B's
//! program, which is the taproot output over a tree containing owner B's
//! coordinator leaf, which would have to contain owner A's — a mutual
//! fixed point no deployment can supply. And a static constructor does
//! not know which owners a future transfer will pair it with, so there
//! is no moment at which the literal could be chosen.
//!
//! That is why §10.1 says recognition is local and admits no cross-input
//! shortcut, and why
//! [`RecognitionCarrier::LeafCommitment`] exists: for the input a leaf
//! runs in, the constructor is established by the leaf executing at all;
//! for every other receipt input it is established by that input's own
//! leaf, and this wave proves nothing about whether one ran. The
//! induction that closes it is §10.4's output closure, which is Wave 6's.
//!
//! # The owner key is a literal, and that is the §1.8 argument
//!
//! §1.8 requires the approved owner-key encoding to be authenticated
//! *before* the signature result is relied on, because the reviewed
//! primitive succeeds without verifying for an unrecognized nonempty
//! key. The strongest available form of that is not a runtime width
//! test: it is for the key never to be a witness item at all. The
//! constructor commits one owner (§7.2), the fragment pushes exactly
//! those bytes, and [`crate::stack::validate_program`] then reports that
//! the forward-compatibility form is unreachable —
//! [`crate::stack::SignatureSuccessForm`] is where that answer lives,
//! and [`owner_key_mutation_outcome`] is where the same walk shows the
//! form reopening the moment the literal stops being the approved
//! encoding at its exact width.
//!
//! # What the profile now answers, and what it still does not
//!
//! The profile these signatures are taken under is
//! [`crate::authorization::selected_owner_profile`], and its review is
//! complete: the assessment recomputes to
//! [`OwnerProfileDisposition::Established`] over a required set every
//! member of which the reviewed contract establishes. So no pattern here
//! carries [`RecognitionResidual::SighashProfileUnreviewed`] any longer.
//!
//! What that residual said is what has stopped being true, and no more.
//! It said a signature verifying is not yet a signature over what §1.7
//! protects, because the profile joining the two was unreviewed; the
//! profile is now reviewed, and the join is the review verdict's. It
//! never said anything about a target having run any pattern in this
//! module, and nothing here says that now either — the patterns are
//! recognitions, and a recognition is not a run.

use std::collections::{BTreeMap, BTreeSet};

use compiler::live_transfer_plan::{LiveTransferComposition, LiveTransferRepresentationPlan};
use compiler::operation_plan::RequiredSourceKind;
use compiler::target::ExternalEvidenceRole;
use target_elements::{
    ElementsCapability, EncodingClass, EncodingDomain, FailureCause, OpcodeId, PayloadWidth,
    ResourceBound, ResourceDimension, ReviewedElementsTapscriptDefinition, StackValueType,
    TargetEvidenceRequirementId,
};

use crate::authorization::{
    OwnerKeyNegative, OwnerKeyObligation, OwnerProfileDisposition, selected_owner_profile,
};
use crate::capability::census_enum;
use crate::error::TapscriptError;
use crate::instruction::{StackItem, TapscriptInstruction};
use crate::live_constructor::{LiveProgramRole, OwnerKey, StaticLiveReceiptConstructor};
use crate::live_plan::{
    destination_closure_fragment, emits_isolation_fragment, explicit_conservation_fragment,
    has_sponsor_region, issuance_absence_fragment, live_sponsor_isolation_fragment,
};
use crate::live_private::{
    crossing_destination_form_fragment, private_destination_form_fragment, require_value_form,
};
use crate::live_shape::LiveTransferShape;
use crate::pattern::{
    coordinator_role_fragment, final_truth_fragment, fragment_prerequisites, member_range_fragment,
    number, op, prefix,
};
use crate::program::TapscriptProgram;
use crate::stack::{
    AbstractExecutionResult, AbstractLimits, AbstractStackState, SignatureSuccessForm,
    resource_projection, validate_program,
};

// --- Link-time symbols ------------------------------------------------

/// The exact link-time literals a live-transfer fragment pushes.
///
/// Supplied, never minted, and every member has a consumer: §1.10 admits
/// no reserved field, so a symbol is here exactly while some emitted
/// fragment pushes it. The protocol asset reaches §10.1's recognition and
/// §10.4's destination closure; the destination program version reaches
/// §10.4; and the reserve asset, the sponsor-change program and its
/// version, and the fee role's digest reach §10.7's sponsor isolation.
///
/// The owner key is not among them. It arrives on the
/// [`StaticLiveReceiptConstructor`] instead, where §7.2 already checked
/// it, and the shape counts arrive on the shape.
///
/// The live-receipt constructor's own program has no field here for the
/// reason [`crate::pattern::CompactAshSymbols`] gives and one more: it
/// is owner-parameterized, so there is not one program for a symbol to
/// name. Nor is a destination's, for the sharper reason that a
/// transfer's destination owners are chosen per request and no value
/// exists when the leaf is built — which is why the version its program
/// is read at is a symbol and its payload is not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveTransferSymbols {
    protocol_asset: StackItem,
    reserve_asset: StackItem,
    destination_program_version: i64,
    sponsor_change_program: StackItem,
    sponsor_change_version: i64,
    fee_program_digest: StackItem,
}

impl LiveTransferSymbols {
    /// Assemble the symbol set, checking each item's width.
    ///
    /// # Errors
    ///
    /// [`TapscriptError::MalformedEncodedItem`] when an item is not the
    /// width its encoding class admits.
    pub fn new(
        target: &ReviewedElementsTapscriptDefinition,
        protocol_asset: Vec<u8>,
        reserve_asset: Vec<u8>,
        destination_program_version: i64,
        sponsor_change_program: Vec<u8>,
        sponsor_change_version: i64,
        fee_program_digest: Vec<u8>,
    ) -> Result<Self, TapscriptError> {
        Ok(Self {
            protocol_asset: StackItem::encoded(
                target,
                EncodingClass::ExplicitAsset,
                protocol_asset,
            )?,
            reserve_asset: StackItem::encoded(target, EncodingClass::ExplicitAsset, reserve_asset)?,
            destination_program_version,
            sponsor_change_program: StackItem::encoded(
                target,
                EncodingClass::WitnessProgram,
                sponsor_change_program,
            )?,
            sponsor_change_version,
            fee_program_digest: StackItem::encoded(
                target,
                EncodingClass::ScriptPubKeySha256,
                fee_program_digest,
            )?,
        })
    }

    /// The exact explicit protocol asset every receipt carries.
    #[must_use]
    pub const fn protocol_asset(&self) -> &StackItem {
        &self.protocol_asset
    }

    /// The exact explicit reserve asset every sponsor and fee role
    /// carries.
    ///
    /// Distinct from the protocol asset by the compiler's own plan, which
    /// refuses a projection whose protocol and sponsor families coincide.
    /// That is what makes requiring it at every non-destination output
    /// the same statement as §10.4's "no output carrying the protocol
    /// asset lies outside the destination range".
    #[must_use]
    pub const fn reserve_asset(&self) -> &StackItem {
        &self.reserve_asset
    }

    /// The version a live-receipt constructor's output program is read
    /// at.
    #[must_use]
    pub const fn destination_program_version(&self) -> i64 {
        self.destination_program_version
    }

    /// The sponsor-change role's witness program.
    #[must_use]
    pub const fn sponsor_change_program(&self) -> &StackItem {
        &self.sponsor_change_program
    }

    /// The version the sponsor-change witness program is read at.
    #[must_use]
    pub const fn sponsor_change_version(&self) -> i64 {
        self.sponsor_change_version
    }

    /// The fee role's program digest.
    #[must_use]
    pub const fn fee_program_digest(&self) -> &StackItem {
        &self.fee_program_digest
    }
}

// --- What recognition establishes, and how (§10.1) --------------------

census_enum! {
    /// One fact §10.1 has every receipt input prove.
    ///
    /// The guide's own six lines, in its own order, so that the two the
    /// emitted bytes do not establish are members with a carrier rather
    /// than omissions a reader would have to notice.
    pub enum RecognizedFact {
        /// The input's asset is the exact linked protocol asset.
        ProtocolAsset,
        /// The input is held under the exact linked live-receipt
        /// constructor.
        LiveReceiptConstructor,
        /// The owner metadata is the canonical encoding.
        CanonicalOwnerMetadata,
        /// The object's class is live.
        LiveClass,
        /// The value representation is the selected plan's.
        SelectedRepresentation,
        /// The executing input lies in the receipt range.
        InputInTheReceiptRange,
    }
}

census_enum! {
    /// What a recognition claim still owes.
    ///
    /// A residual is not a caveat. Each names something a later wave or
    /// a target-native run has to show, and a fact carrying one is a
    /// fact this wave established less completely than §10.1 states it.
    pub enum RecognitionResidual {
        /// The exact program bytes of a destination's constructor.
        ///
        /// What is left of Wave 5's cross-input question after §10.4's
        /// induction, which
        /// [`recognition_establishments`] states in full: the output
        /// closure makes every output carrying the protocol asset a live
        /// receipt under a linked constructor, so every receipt input of
        /// a later transfer is one too, and the coordinator no longer
        /// needs a claim about an input it cannot see.
        ///
        /// The induction's step rests on §10.4 holding whole, and it does
        /// not yet. A destination's constructor is owner-parameterized
        /// (§7.6) and its owners are chosen per request (§12.3), so no
        /// leaf fixed at construction can carry a literal for it. The
        /// closure compares the asset and the version the program is read
        /// at.
        ///
        /// # What the linker settled, and what is still owed
        ///
        /// The reason had three parts and now has two. §11.4's
        /// deterministic tree exists: for each (owner, representation)
        /// the linker produces the exact linked leaf programs and the
        /// committed tree over them, and that tree is the one a
        /// destination's version check points at. Two things still stand
        /// between that and §10.4 holding whole — the taproot output key,
        /// which §1.13 leaves uncomputed until a consumer of one exists,
        /// and §12's destination table, which says which owner receives
        /// which output position and is chosen per request.
        ///
        /// The residual is therefore re-scoped rather than cleared, and
        /// it stays here for a reason no link changes: what a *program*
        /// can establish about a destination it cannot see is unaffected
        /// by what a linker establishes about which constructor a given
        /// owner has. The linker states its own half as
        /// `linker::LiveInductionStep`; this is still the leaves'.
        LinkedDestinationConstructorIdentity,
        /// The explicit or confidential form of an introspected field.
        ///
        /// The prefix comparison settles it on the target, which pushes
        /// the form's own prefix; the abstract walk holds no bytes for
        /// an introspected prefix and keeps both forms live.
        FieldFormSettledOnlyOnTheTarget,
        /// Curve-point membership of the owner key.
        ///
        /// Undischarged in this crate, which holds no curve arithmetic
        /// and takes no dependency that would give it any. Discharged by
        /// the conformance package's first-party point oracle, and by
        /// the target's own key decoding.
        OwnerKeyCurvePointMembership,
        /// The selected sighash profile is not established by the
        /// review.
        ///
        /// Carried wherever a pattern asserts that a signature commits
        /// to the finalized transaction. The signature verifying is a
        /// different and weaker statement than the signature covering
        /// §1.7's protected data, and only the second is authorization.
        ///
        /// # It is no longer carried, and the verdict that moved it
        ///
        /// The review verdict is in and the post-verdict re-typing
        /// with it, and between them the residual is cleared. Six of the
        /// selected profile's seven required dimensions were established
        /// by the source review's citation and the observed acceptance
        /// that exercised it. The seventh was
        /// [`target_elements::SighashDimension::Issuance`], and the
        /// reason it stopped there is a property of this workspace rather
        /// than of the target: the target's message does carry the
        /// dimension, in the two terms the review cites, but no candidate
        /// this arc builds bears an issuance, so both terms are formed
        /// from the input count alone and no number of agreeing digests
        /// checks a model that never read the subject.
        ///
        /// The owner took the second of the two repairs that verdict
        /// named — not the candidate bearing an issuance, but the
        /// re-typing of the dimension the way the internal key's was —
        /// on the ground that the census and the decoder both refuse the
        /// subject. So the required set is six, every member of it is
        /// established, and [`OwnerSighashProfile::assess`] recomputes
        /// the disposition to
        /// [`OwnerProfileDisposition::Established`]. No pattern carries
        /// this residual now.
        ///
        /// # What did not move with it
        ///
        /// Nothing about a target run. The disposition says what a
        /// signature under this profile commits to; it says nothing about
        /// any pattern in this module having been executed by anything,
        /// and the obligations that carry that claim are untouched.
        ///
        /// The word stays in this vocabulary because it is still the
        /// right name for the condition, and a pattern taken under a
        /// profile whose review had lapsed must be able to say so.
        ///
        /// [`OwnerSighashProfile::assess`]: crate::authorization::OwnerSighashProfile::assess
        SighashProfileUnreviewed,
    }
}

/// What establishes one of §10.1's facts.
///
/// Three carriers and not a Boolean, because "the bytes prove it",
/// "the leaf's own commitment proves it", and "no value can express the
/// alternative" are three different strengths of claim, and a reader
/// deciding how far to trust a recognition needs to know which one is
/// in hand.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RecognitionCarrier {
    /// Instructions the recognition or role fragment emits.
    EmittedInstructions,
    /// The leaf runs only from a taptree its input's program commits
    /// to, which settles the fact for that input and for no other.
    LeafCommitment,
    /// The constructor's own type list admits no other value.
    ///
    /// The strongest of the three where it applies: a live-transfer
    /// leaf is not a leaf of any other operation's constructor, and the
    /// selected representation's leaves are a disjoint set, so the
    /// alternative is unrepresentable rather than refused.
    ConstructorTyping,
}

/// How one recognized fact is established, and what it still owes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecognitionEstablishment {
    fact: RecognizedFact,
    carrier: RecognitionCarrier,
    residuals: BTreeSet<RecognitionResidual>,
}

impl RecognitionEstablishment {
    /// The §10.1 fact.
    #[must_use]
    pub const fn fact(&self) -> RecognizedFact {
        self.fact
    }

    /// What establishes it.
    #[must_use]
    pub const fn carrier(&self) -> RecognitionCarrier {
        self.carrier
    }

    /// What the claim still owes, in census order.
    pub fn residuals(&self) -> impl Iterator<Item = RecognitionResidual> + '_ {
        self.residuals.iter().copied()
    }
}

/// How this candidate establishes each of §10.1's six facts.
///
/// Total over [`RecognizedFact::ALL`] by construction: the census is
/// built by mapping that constant, so a fact added to §10.1 arrives here
/// without a carrier rather than silently absent.
///
/// # The induction §10.4 closes, stated
///
/// [`RecognizedFact::LiveReceiptConstructor`] is the fact a local
/// recognition cannot establish for an input other than its own, and the
/// argument that closes it is not a fragment. It is an induction over the
/// chain of transfers, and it runs like this.
///
/// *Step.* [`crate::live_plan::destination_closure_fragment`] requires
/// every output of the destination range to carry the exact linked
/// protocol asset, and [`crate::live_plan::live_sponsor_isolation_fragment`]
/// requires the reserve asset at every output position outside it. The
/// exact output count leaves no third kind of position. So every output
/// of an accepted transfer that carries the protocol asset is a
/// destination, under a program read at the version a live-receipt
/// constructor takes. A receipt input of a later transfer is an output of
/// an earlier one, so it is such a destination — and coordinator and
/// member alike may rely on that without introspecting it.
///
/// *Base.* An object carrying the protocol asset that was created by
/// something other than a transfer is outside this induction and outside
/// Guide 13: issuance is a different operation, and §10.8 forbids one
/// here. The base case is therefore an obligation on whatever creates the
/// first live receipt, not a gap in this argument.
///
/// *What the step still owes.* The step establishes the destination's
/// asset and program version, not the exact program bytes, so it
/// currently supports a weaker conclusion than §10.4 states: every such
/// input is a taproot output carrying the protocol asset, rather than one
/// under this family's constructor for a canonical owner. That gap is
/// [`RecognitionResidual::LinkedDestinationConstructorIdentity`], and it
/// is the *same* residual on both ends of the induction — which is the
/// point of stating the argument rather than carrying two.
#[must_use]
#[expect(
    clippy::match_same_arms,
    reason = "one arm per §10.1 fact, so a line added to the guide's list has to be decided \
              here; merging coincident ones would delete which fact is which"
)]
pub fn recognition_establishments() -> BTreeMap<RecognizedFact, RecognitionEstablishment> {
    use RecognitionCarrier as Carrier;
    use RecognitionResidual as Residual;

    RecognizedFact::ALL
        .iter()
        .map(|fact| {
            let (carrier, residuals): (_, &[Residual]) = match fact {
                // The asset test is a symbol comparison the fragment
                // emits. Its form is settled by the prefix comparison,
                // which the target decides and the walk cannot.
                RecognizedFact::ProtocolAsset => (
                    Carrier::EmittedInstructions,
                    &[Residual::FieldFormSettledOnlyOnTheTarget],
                ),
                RecognizedFact::LiveReceiptConstructor => (
                    Carrier::LeafCommitment,
                    &[Residual::LinkedDestinationConstructorIdentity],
                ),
                // The owner key is a literal of the approved encoding at
                // its exact width, which is what closes §1.8's
                // forward-compatibility path. What the bytes cannot say
                // is that they are a point on the curve.
                RecognizedFact::CanonicalOwnerMetadata => (
                    Carrier::EmittedInstructions,
                    &[Residual::OwnerKeyCurvePointMembership],
                ),
                // No live-transfer leaf exists in any other operation's
                // constructor, so a leaf that runs at all runs inside a
                // live one. There is no class field to compare against.
                RecognizedFact::LiveClass => (Carrier::ConstructorTyping, &[]),
                // Emitted as the value field's form, and the leaf sets
                // of the two representations are disjoint besides.
                RecognizedFact::SelectedRepresentation => (
                    Carrier::EmittedInstructions,
                    &[Residual::FieldFormSettledOnlyOnTheTarget],
                ),
                RecognizedFact::InputInTheReceiptRange => (Carrier::EmittedInstructions, &[]),
            };

            (
                *fact,
                RecognitionEstablishment {
                    fact: *fact,
                    carrier,
                    residuals: residuals.iter().copied().collect(),
                },
            )
        })
        .collect()
}

// --- Fragments --------------------------------------------------------

/// The encoding class one representation plan's value field carries.
///
/// Read off the plan rather than passed in, so a fragment cannot be
/// built for the explicit plan and then compare against a confidential
/// prefix.
const fn value_encoding(representation: LiveTransferRepresentationPlan) -> EncodingClass {
    match representation {
        LiveTransferRepresentationPlan::Explicit => EncodingClass::ExplicitValue,
        LiveTransferRepresentationPlan::PrivateCommitted => EncodingClass::ConfidentialValue,
    }
}

/// Recognize the input this leaf is executing in (§10.1).
///
/// The asset is compared with the linked symbol in the explicit form,
/// and the value field's *form* is compared with the selected plan's.
/// The value payload is then dropped: recognition establishes which
/// representation an input is in and reads no amount out of it, which is
/// what lets the same fragment serve the private plan, where §10.6
/// admits no amount inspection at all.
///
/// The position is the target's own current-input index rather than a
/// compiled-in one, so a single fragment serves every receipt position —
/// and, more to the point, so the fragment speaks about the input whose
/// leaf commitment it is running under and about no other. See the
/// module documentation for why no cross-input form of this exists.
///
/// # Every declared form of the class, not the least of them
///
/// The value comparison admits every prefix the reviewed registry
/// declares for the class — [`crate::live_private::prefix_mask`] is what
/// derives them and is where the argument lives. For the explicit plan
/// that is one byte and the emitted instructions are the equality they
/// always were. For the private plan it is two — a confidential value's
/// prefix records whether its commitment's `y` is a square — and a
/// comparison against the smaller of them would refuse about half of all
/// well-formed private receipts, for a reason their owner can neither
/// control nor see coming.
///
/// # Errors
///
/// [`TapscriptError::ScriptNumberOutOfRange`] or
/// [`TapscriptError::OversizedStackItem`] when a literal this fragment
/// pushes is outside what the reviewed contract admits,
/// [`TapscriptError::MalformedEncodedItem`] when an encoding class
/// states no prefix to compare against,
/// [`TapscriptError::PrefixSetNotDiscriminable`] when a class's declared
/// prefixes are not the bytes one mask admits, and
/// [`TapscriptError::InstructionLimitExceeded`] when the fragment
/// exceeds the instruction bound.
pub fn local_recognition_fragment(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &LiveTransferSymbols,
    representation: LiveTransferRepresentationPlan,
) -> Result<TapscriptProgram, TapscriptError> {
    let mut instructions = vec![
        op(OpcodeId::PushCurrentInputIndex),
        op(OpcodeId::InspectInputAsset),
        TapscriptInstruction::Push(prefix(target, EncodingClass::ExplicitAsset)?),
        op(OpcodeId::EqualVerify),
        TapscriptInstruction::Push(symbols.protocol_asset().clone()),
        op(OpcodeId::EqualVerify),
        op(OpcodeId::PushCurrentInputIndex),
        op(OpcodeId::InspectInputValue),
    ];
    instructions.extend(require_value_form(target, value_encoding(representation))?);
    // The form was the question; the amount is not this fragment's
    // business and is not left where a later fragment could take it
    // for one.
    instructions.push(op(OpcodeId::Drop));

    TapscriptProgram::new(instructions)
}

/// Authorize the input this leaf is executing in (§10.2, §1.8).
///
/// Two instructions, and the argument is in which two. The owner key is
/// pushed as a literal of the approved encoding at its exact width, so
/// the encoding is not something a witness could get wrong; the
/// verifying form of the signature check is used, so no Boolean survives
/// for a caller to read as truth; and the signature is the one witness
/// item the leaf consumes.
///
/// # What the schedule assumes about the witness, and why so little
///
/// [`owner_authorization_precondition`] types the signature position as
/// an unconstrained item up to the target's literal bound, not as a
/// well-formed signature. Typing it as a signature would assume exactly
/// what §10.2 requires the program to reject: the empty offering, which
/// the verifying form aborts on, would have been ruled out by the
/// schedule instead of by the target.
///
/// # Errors
///
/// [`TapscriptError::MalformedEncodedItem`] when the owner's bytes are
/// not the width their encoding class admits — unreachable for an
/// [`OwnerKey`], which §7.2 already admitted at that width — and
/// [`TapscriptError::InstructionLimitExceeded`] when the fragment
/// exceeds the instruction bound.
pub fn owner_authorization_fragment(
    target: &ReviewedElementsTapscriptDefinition,
    owner: &OwnerKey,
) -> Result<TapscriptProgram, TapscriptError> {
    TapscriptProgram::new(vec![
        TapscriptInstruction::Push(StackItem::encoded(
            target,
            owner.encoding(),
            owner.bytes().to_vec(),
        )?),
        op(OpcodeId::CheckSigVerify),
    ])
}

/// The stack an owner-authorization fragment is scheduled from.
///
/// One item: whatever the spender put in the signature position. Its
/// type is the widest thing the target admits as a stack element, which
/// is the honest description of a witness item and the reason the
/// fragment's abort set carries the empty-signature cause.
#[must_use]
pub fn owner_authorization_precondition(
    target: &ReviewedElementsTapscriptDefinition,
) -> AbstractStackState {
    AbstractStackState::from_main(vec![StackValueType::Bytes {
        minimum: 0,
        maximum: target.definition().pushes().maximum_payload_bytes(),
    }])
}

/// Authenticate the shape's exact input and output counts (§10.3).
///
/// The coordinator's, and only the coordinator's: a member leaf serves
/// every shape of one receipt-input count (§7.1), so it has no single
/// pair of counts to compare against and does not pretend to.
///
/// # Errors
///
/// [`TapscriptError::ScriptNumberOutOfRange`] or
/// [`TapscriptError::OversizedStackItem`] when a literal this fragment
/// pushes is outside what the reviewed contract admits, and
/// [`TapscriptError::InstructionLimitExceeded`] when the fragment
/// exceeds the instruction bound.
pub fn live_cardinality_fragment(
    target: &ReviewedElementsTapscriptDefinition,
    shape: LiveTransferShape,
) -> Result<TapscriptProgram, TapscriptError> {
    TapscriptProgram::new(vec![
        op(OpcodeId::InspectNumInputs),
        number(target, i64::from(shape.inputs()))?,
        op(OpcodeId::EqualVerify),
        op(OpcodeId::InspectNumOutputs),
        number(target, i64::from(shape.outputs()))?,
        op(OpcodeId::EqualVerify),
    ])
}

/// Bind a member leaf to the shape's nonzero receipt positions (§10.3).
///
/// `1 ≤ index < receipt_inputs`, both ends checked and both Booleans
/// consumed. The bytes are
/// [`crate::pattern::member_role_fragment`]'s, over this operation's own
/// count: the statement is the same one and two copies of a bound check
/// are two places for an off-by-one to live.
///
/// # Errors
///
/// [`TapscriptError::ScriptNumberOutOfRange`] or
/// [`TapscriptError::OversizedStackItem`] when a literal this fragment
/// pushes is outside what the reviewed contract admits, and
/// [`TapscriptError::InstructionLimitExceeded`] when the fragment
/// exceeds the instruction bound.
pub fn live_member_role_fragment(
    target: &ReviewedElementsTapscriptDefinition,
    receipt_inputs: u8,
) -> Result<TapscriptProgram, TapscriptError> {
    member_range_fragment(target, receipt_inputs)
}

// --- Composed programs (§10.3, §10.9) ---------------------------------

/// Why a live-transfer program was not emitted.
///
/// Construction failure, never target rejection (§1.11): each variant
/// names something wrong with what the emitter was asked to build.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LiveProgramRefusal {
    /// The constructor emits no program for this shape.
    ShapeNotAdmitted {
        /// The shape asked for.
        shape: LiveTransferShape,
    },
    /// The shape has no nonzero receipt position, so it has no member
    /// program.
    ///
    /// The one-to-one transfer. §7.1 gives it no member leaf, and an
    /// emitter that produced one anyway would produce a program no spend
    /// could reach.
    ShapeHasNoMemberPosition {
        /// The shape asked for.
        shape: LiveTransferShape,
    },
    /// A fragment the program is built from could not be assembled.
    FragmentRefused {
        /// What the fragment builder reported.
        cause: TapscriptError,
    },
    /// A recipe named a component record that was not built.
    RecipePatternMissing {
        /// The missing component record.
        pattern: LiveTransferPatternId,
    },
    /// A coordinator recipe selected no value-obligation fragment.
    RecipeValueComponentMissing,
    /// The component union did not carry `FinalTruth`'s base requirements.
    FinalTruthDependenciesMissing,
    /// A concrete coordinator placement did not match its recipe.
    CoordinatorPlacementInvalid {
        /// What the placement validator reported.
        cause: PlacementDefect,
    },
}

impl From<TapscriptError> for LiveProgramRefusal {
    fn from(cause: TapscriptError) -> Self {
        Self::FragmentRefused { cause }
    }
}

/// One fragment and the component-pattern record that describes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LiveProgramComponent {
    fragment: LiveFragmentId,
    pattern: Option<LiveTransferPatternId>,
}

impl LiveProgramComponent {
    const fn new(fragment: LiveFragmentId, pattern: Option<LiveTransferPatternId>) -> Self {
        Self { fragment, pattern }
    }
}

/// The single composition recipe for one emitted live program.
#[derive(Clone, Debug, PartialEq, Eq)]
struct LiveProgramRecipe {
    role: LiveProgramRole,
    components: Vec<LiveProgramComponent>,
}

/// Whether one recipe carries a stated component-pattern record.
fn recipe_selects_pattern(recipe: &LiveProgramRecipe, pattern: LiveTransferPatternId) -> bool {
    recipe
        .components
        .iter()
        .any(|component| component.pattern == Some(pattern))
}

/// The one value-obligation fragment a coordinator recipe selected.
fn recipe_value_fragment(recipe: &LiveProgramRecipe) -> Result<LiveFragmentId, LiveProgramRefusal> {
    recipe
        .components
        .iter()
        .map(|component| component.fragment)
        .find(|fragment| {
            matches!(
                fragment,
                LiveFragmentId::ExplicitConservation
                    | LiveFragmentId::PrivateDestinationForm
                    | LiveFragmentId::CrossingDestinationForm
            )
        })
        .ok_or(LiveProgramRefusal::RecipeValueComponentMissing)
}

/// Select every fragment one concrete live program walks, in byte order.
fn live_program_recipe(
    constructor: &StaticLiveReceiptConstructor,
    role: LiveProgramRole,
    shape: LiveTransferShape,
) -> Result<LiveProgramRecipe, LiveProgramRefusal> {
    use LiveFragmentId as Fragment;
    use LiveTransferPatternId as Pattern;

    if !constructor.shapes().admits(shape) {
        return Err(LiveProgramRefusal::ShapeNotAdmitted { shape });
    }

    let components = match role {
        LiveProgramRole::Coordinator => {
            let value = match constructor.composition() {
                LiveTransferComposition::HomogeneousExplicit => LiveProgramComponent::new(
                    Fragment::ExplicitConservation,
                    Some(Pattern::LiveExplicitConservationV1),
                ),
                LiveTransferComposition::HomogeneousPrivate
                | LiveTransferComposition::EntryBlinding => LiveProgramComponent::new(
                    Fragment::PrivateDestinationForm,
                    Some(Pattern::LivePrivateDestinationFormV1),
                ),
                LiveTransferComposition::ExitUnblinding => LiveProgramComponent::new(
                    Fragment::CrossingDestinationForm,
                    Some(Pattern::LiveCrossingDestinationFormV1),
                ),
            };
            vec![
                LiveProgramComponent::new(
                    Fragment::CoordinatorRole,
                    Some(Pattern::LiveCoordinatorRoleV1),
                ),
                LiveProgramComponent::new(Fragment::Cardinality, Some(Pattern::LiveShapeV1)),
                LiveProgramComponent::new(
                    Fragment::LocalRecognition,
                    Some(Pattern::LiveInputRecognitionV1),
                ),
                LiveProgramComponent::new(
                    Fragment::OwnerAuthorization,
                    Some(Pattern::LiveOwnerAuthorizationV1),
                ),
                LiveProgramComponent::new(
                    Fragment::DestinationClosure,
                    Some(Pattern::LiveDestinationClosureV1),
                ),
                LiveProgramComponent::new(
                    Fragment::SponsorIsolation,
                    emits_isolation_fragment(shape).then_some(Pattern::LiveSponsorIsolationV1),
                ),
                LiveProgramComponent::new(
                    Fragment::IssuanceAbsence,
                    Some(Pattern::LiveIssuanceAbsenceV1),
                ),
                value,
                LiveProgramComponent::new(Fragment::FinalTruth, None),
            ]
        }
        LiveProgramRole::Member => {
            if !has_member_position(shape) {
                return Err(LiveProgramRefusal::ShapeHasNoMemberPosition { shape });
            }
            vec![
                LiveProgramComponent::new(Fragment::MemberRole, Some(Pattern::LiveMemberRoleV1)),
                LiveProgramComponent::new(
                    Fragment::LocalRecognition,
                    Some(Pattern::LiveInputRecognitionV1),
                ),
                LiveProgramComponent::new(
                    Fragment::OwnerAuthorization,
                    Some(Pattern::LiveOwnerAuthorizationV1),
                ),
                LiveProgramComponent::new(Fragment::FinalTruth, None),
            ]
        }
    };

    Ok(LiveProgramRecipe { role, components })
}

/// Build one fragment selected by a live-program recipe.
fn build_live_fragment(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &LiveTransferSymbols,
    constructor: &StaticLiveReceiptConstructor,
    shape: LiveTransferShape,
    fragment: LiveFragmentId,
) -> Result<TapscriptProgram, LiveProgramRefusal> {
    Ok(match fragment {
        LiveFragmentId::CoordinatorRole => coordinator_role_fragment(target)?,
        LiveFragmentId::MemberRole => live_member_role_fragment(target, shape.receipt_inputs())?,
        LiveFragmentId::Cardinality => live_cardinality_fragment(target, shape)?,
        LiveFragmentId::LocalRecognition => {
            local_recognition_fragment(target, symbols, constructor.representation())?
        }
        LiveFragmentId::OwnerAuthorization => {
            owner_authorization_fragment(target, constructor.owner())?
        }
        LiveFragmentId::DestinationClosure => destination_closure_fragment(target, symbols, shape)?,
        LiveFragmentId::ExplicitConservation => explicit_conservation_fragment(target, shape)?,
        LiveFragmentId::PrivateDestinationForm => private_destination_form_fragment(target, shape)?,
        LiveFragmentId::CrossingDestinationForm => {
            crossing_destination_form_fragment(target, shape)?
        }
        LiveFragmentId::SponsorIsolation => {
            live_sponsor_isolation_fragment(target, symbols, shape)?
        }
        LiveFragmentId::IssuanceAbsence => issuance_absence_fragment(target, shape)?,
        LiveFragmentId::FinalTruth => final_truth_fragment(target)?,
    })
}

/// Compose bytes and their fragment trace by walking one recipe once.
fn compose_live_program(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &LiveTransferSymbols,
    constructor: &StaticLiveReceiptConstructor,
    shape: LiveTransferShape,
    recipe: &LiveProgramRecipe,
) -> Result<(TapscriptProgram, Vec<LiveFragmentId>), LiveProgramRefusal> {
    let expected_anchor = match recipe.role {
        LiveProgramRole::Coordinator => LiveFragmentId::CoordinatorRole,
        LiveProgramRole::Member => LiveFragmentId::MemberRole,
    };
    debug_assert_eq!(
        recipe
            .components
            .first()
            .map(|component| component.fragment),
        Some(expected_anchor),
    );
    let mut instructions = Vec::new();
    let mut trace = Vec::with_capacity(recipe.components.len());

    for component in &recipe.components {
        let fragment =
            build_live_fragment(target, symbols, constructor, shape, component.fragment)?;
        instructions.extend_from_slice(fragment.instructions());
        trace.push(component.fragment);
    }

    Ok((TapscriptProgram::new(instructions)?, trace))
}

/// The whole coordinator program for one admitted shape (§10.3).
///
/// Concatenated in the order the schedules were verified in: the
/// coordinator's own anchor first, so a leaf spent at the wrong position
/// stops before introspecting anything; then the exact counts, so a
/// transaction of the wrong shape stops before any range is relied on;
/// then the local pair §10.3 requires of every receipt input, the
/// coordinator included; then the transaction-global checks of §10.4,
/// §10.7 and §10.8, which every representation owes; then the selected
/// representation's own value obligation; then the canonical true item.
///
/// # The one slot the four compositions fill differently
///
/// The destination closure, the sponsor isolation and the issuance
/// absence read assets, programs and issuance fields, and none of those
/// changes with how a value is carried. The value obligation does:
///
/// - the explicit coordinator carries §10.5's conservation, which sums
///   both sides and requires them equal, and which establishes each
///   destination's explicit form on the way to reading it;
/// - private and entry-blinding coordinators carry §6.3's confidential
///   destination form check, which
///   establishes the same form fact for the confidential representation
///   and stops there;
/// - the exit-unblinding coordinator carries §6.5's positional crossing
///   form check.
///
/// §10.6 admits no private amount inspection, so the equation over those
/// commitments is the target's own rule
/// ([`ExternalEvidenceRole::ConfidentialValueConservation`]) and no
/// fragment here claims it.
///
/// Neither coordinator carries both, and neither carries neither: a
/// coordinator with no value obligation at all would be reading value
/// fields it had never established were value fields.
///
/// # Errors
///
/// [`LiveProgramRefusal::ShapeNotAdmitted`] for a shape this constructor
/// emits no program for, and
/// [`LiveProgramRefusal::FragmentRefused`] when a fragment cannot be
/// assembled against the reviewed contract.
pub fn live_coordinator_program(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &LiveTransferSymbols,
    constructor: &StaticLiveReceiptConstructor,
    shape: LiveTransferShape,
) -> Result<TapscriptProgram, LiveProgramRefusal> {
    let recipe = live_program_recipe(constructor, LiveProgramRole::Coordinator, shape)?;
    compose_live_program(target, symbols, constructor, shape, &recipe).map(|(program, _)| program)
}

/// The whole member program for one receipt-input count (§10.3).
///
/// The range, the local pair, and the canonical true item. No counts:
/// this leaf serves every shape of its receipt-input count, and the
/// exact input and output counts are the coordinator's to authenticate
/// at input 0.
///
/// # Errors
///
/// [`LiveProgramRefusal::ShapeHasNoMemberPosition`] for a count with no
/// nonzero receipt position, and
/// [`LiveProgramRefusal::FragmentRefused`] when a fragment cannot be
/// assembled against the reviewed contract.
pub fn live_member_program(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &LiveTransferSymbols,
    constructor: &StaticLiveReceiptConstructor,
    receipt_inputs: u8,
) -> Result<TapscriptProgram, LiveProgramRefusal> {
    let Some(shape) = constructor
        .shapes()
        .shapes()
        .find(|shape| shape.receipt_inputs() == receipt_inputs)
    else {
        return Err(LiveProgramRefusal::ShapeNotAdmitted {
            shape: smallest_shape(constructor),
        });
    };
    let recipe = live_program_recipe(constructor, LiveProgramRole::Member, shape)?;
    compose_live_program(target, symbols, constructor, shape, &recipe).map(|(program, _)| program)
}

/// The fragment trace recorded by the same walk that composes a program.
#[cfg(test)]
pub(crate) fn live_program_fragment_trace(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &LiveTransferSymbols,
    constructor: &StaticLiveReceiptConstructor,
    role: LiveProgramRole,
    shape: LiveTransferShape,
) -> Result<Vec<LiveFragmentId>, LiveProgramRefusal> {
    let recipe = live_program_recipe(constructor, role, shape)?;
    compose_live_program(target, symbols, constructor, shape, &recipe).map(|(_, trace)| trace)
}

/// The narrowest shape a constructor admits.
///
/// Only ever used to name a shape in a refusal, where the caller asked
/// for a receipt-input count no admitted shape carries and so supplied
/// no shape of its own to name.
///
/// # Panics
///
/// Never for a constructed [`StaticLiveReceiptConstructor`]: its shape
/// set refuses to be empty, so the iterator always yields.
fn smallest_shape(constructor: &StaticLiveReceiptConstructor) -> LiveTransferShape {
    constructor
        .shapes()
        .shapes()
        .next()
        .expect("a constructed shape set holds at least one shape")
}

/// The stack a composed live-transfer program is scheduled from.
///
/// The owner signature, and nothing else: every other operand a live
/// program reads is either introspected from the transaction or pushed
/// by the program itself.
#[must_use]
pub fn live_program_precondition(
    target: &ReviewedElementsTapscriptDefinition,
) -> AbstractStackState {
    owner_authorization_precondition(target)
}

// --- Coordinator placement (§10.3) ------------------------------------

census_enum! {
    /// One fragment this candidate emits.
    pub enum LiveFragmentId {
        /// The coordinator's anchor at input 0.
        CoordinatorRole,
        /// A member's nonzero receipt position.
        MemberRole,
        /// The exact input and output counts.
        Cardinality,
        /// The local predecessor recognition of §10.1.
        LocalRecognition,
        /// The per-input owner authorization of §10.2.
        OwnerAuthorization,
        /// The destination closure of §10.4.
        DestinationClosure,
        /// The exact explicit aggregate conservation of §10.5.
        ///
        /// Emitted by the explicit coordinator and by no other program:
        /// §10.6 admits no amount inspection at all, so a private
        /// coordinator carrying these bytes would be a private plan that
        /// opened an amount.
        ExplicitConservation,
        /// The private destination value form of §6.3 and §10.6.
        ///
        /// The homogeneous-private and entry-blinding coordinators', and
        /// only theirs. The explicit composition settles the same fact inside
        /// [`LiveFragmentId::ExplicitConservation`], which establishes a
        /// destination's explicit form before narrowing its payload — so
        /// one of those coordinators carrying both would be testing one
        /// field twice, and one carrying neither would be reading a value
        /// field it had never established was a value field.
        ///
        /// Not a conservation fragment. It compares forms and drops every
        /// payload, and the value equation stays
        /// [`ExternalEvidenceRole::ConfidentialValueConservation`]'s.
        PrivateDestinationForm,
        /// The positional destination value form of an exit crossing.
        ///
        /// Every created value is explicit save the declared absorber,
        /// which remains confidential. Like the homogeneous private form,
        /// this establishes form and not conservation; the equation across
        /// the private consumed side remains external target evidence.
        CrossingDestinationForm,
        /// The sponsor isolation of §10.7.
        SponsorIsolation,
        /// The issuance absence of §10.8.
        IssuanceAbsence,
        /// The canonical true item of §10.9.
        FinalTruth,
    }
}

census_enum! {
    /// One thing a coordinator slot owes and this candidate does not
    /// emit.
    ///
    /// Named rather than described, so a slot points at what has to exist
    /// and a later wave can see exactly which slots its work fills. Its
    /// one member is outside this wave by design and is not a fragment
    /// somebody forgot: it is a value no in-script comparison can reach.
    ///
    /// # Why §10.6's conservation is no longer among them
    ///
    /// It was, and it was the wrong shape of entry. An outstanding
    /// pattern is a §10 pattern a later wave builds, and §10.6 says
    /// plainly that no private-conservation backend pattern ID is minted
    /// for target consensus behaviour — so the slot was waiting for an
    /// artifact nobody may ever produce. The private plan's value
    /// equation is external target evidence, and
    /// [`GlobalCheckPlacement::external_evidence`] is where it is
    /// recorded now: a named requirement on the target rather than a
    /// promise about this crate's future.
    pub enum OutstandingGlobalPattern {
        /// The exact program bytes of a destination's constructor.
        ///
        /// Not a §10 pattern at all, and that is why it is stated: no
        /// leaf can hold a literal for an owner-parameterized program
        /// whose owner the request chooses, so the remaining half of
        /// §10.4 is owed outside this crate rather than by a fragment it
        /// has not written yet.
        ///
        /// §11.4's deterministic taptree is built and is no longer among
        /// the things owed; §12's destination table and the taproot
        /// output key are. The slot stays outstanding because the slot
        /// is about what the coordinator's emitted bytes reach, and no
        /// linked tree changes that. The matching residual is
        /// [`RecognitionResidual::LinkedDestinationConstructorIdentity`],
        /// which carries the same re-scoping.
        DestinationConstructorIdentity,
    }
}

census_enum! {
    /// One transaction-global check §10.3 gives the coordinator.
    ///
    /// The guide's own eleven, in its own order.
    pub enum CoordinatorGlobalCheck {
        /// Exact input and output counts.
        ExactInputAndOutputCounts,
        /// Complete protocol ranges.
        CompleteProtocolRanges,
        /// Live-class output closure.
        LiveClassOutputClosure,
        /// Explicit protocol-asset closure.
        ExplicitAssetClosure,
        /// Representation-specific conservation.
        RepresentationSpecificConservation,
        /// Sponsor isolation.
        SponsorIsolation,
        /// Roots absent.
        RootsAbsent,
        /// Issuance absent.
        IssuanceAbsent,
        /// Destruction absent.
        DestructionAbsent,
        /// Specialized events absent.
        SpecializedEventsAbsent,
        /// Exact role order.
        ExactRoleOrder,
    }
}

/// How completely this wave answers one global check.
///
/// Derived from the placement's three sets rather than declared beside
/// them, so a slot cannot report itself established while naming an
/// outstanding pattern or an external requirement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GlobalCheckStatus {
    /// Emitted instructions establish the check whole.
    Established,
    /// Emitted instructions establish everything a program can, and a
    /// named external target requirement carries the rest.
    ///
    /// Weaker than [`Self::Established`] and stronger than
    /// [`Self::Partial`], and the difference matters in both directions.
    /// It is not established, because something outside these bytes has
    /// to hold; and it is not partial, because nothing unbuilt is owed —
    /// the remainder is a target rule that either holds or does not, and
    /// no later wave of this crate will change which.
    EstablishedWithExternalEvidence,
    /// Emitted instructions establish part of it, and a §10 pattern that
    /// does not exist owes the rest.
    Partial,
    /// This wave emits nothing for it.
    Outstanding,
}

/// What this wave does about one of §10.3's global checks.
///
/// Three sets rather than one verdict, because a check can be part a
/// coordinator's, part a later pattern's, and part the target's own, and
/// a single verdict would have to round that either up into a claim or
/// down into an omission.
///
/// The third set is what §10.6 needs and the first two could not express.
/// A check the emitted bytes cannot finish is not thereby a check some
/// future fragment finishes: the private plan's value equation is the
/// target's confidential-transaction rule, no program may claim it, and
/// recording it as an outstanding pattern would have promised an artifact
/// §10.6 forbids anyone to build.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlobalCheckPlacement {
    check: CoordinatorGlobalCheck,
    emitted: BTreeSet<LiveFragmentId>,
    outstanding: BTreeSet<OutstandingGlobalPattern>,
    external: BTreeSet<ExternalEvidenceRole>,
}

impl GlobalCheckPlacement {
    /// State one check's slot: what is emitted, owed, and external.
    ///
    /// Unchecked, for the reason [`crate::live_plan::LiveFamilyRange::new`]
    /// is: this type is the *claim* and
    /// [`validate_coordinator_placements`] is the check. Splitting them
    /// is what lets a consumer state a census of its own and have the
    /// same validator run over it — and what lets the validator's own
    /// refusals be executed rather than described.
    #[must_use]
    pub const fn new(
        check: CoordinatorGlobalCheck,
        emitted: BTreeSet<LiveFragmentId>,
        outstanding: BTreeSet<OutstandingGlobalPattern>,
        external: BTreeSet<ExternalEvidenceRole>,
    ) -> Self {
        Self {
            check,
            emitted,
            outstanding,
            external,
        }
    }

    /// The §10.3 check.
    #[must_use]
    pub const fn check(&self) -> CoordinatorGlobalCheck {
        self.check
    }

    /// The fragments this wave emits towards it.
    pub fn emitted(&self) -> impl Iterator<Item = LiveFragmentId> + '_ {
        self.emitted.iter().copied()
    }

    /// The §10 patterns that owe the rest of it.
    pub fn outstanding(&self) -> impl Iterator<Item = OutstandingGlobalPattern> + '_ {
        self.outstanding.iter().copied()
    }

    /// The external target requirements that carry the rest of it.
    pub fn external_evidence(&self) -> impl Iterator<Item = ExternalEvidenceRole> + '_ {
        self.external.iter().copied()
    }

    /// How completely this wave answers the check.
    #[must_use]
    pub fn status(&self) -> GlobalCheckStatus {
        if self.emitted.is_empty() {
            GlobalCheckStatus::Outstanding
        } else if self.outstanding.is_empty() {
            if self.external.is_empty() {
                GlobalCheckStatus::Established
            } else {
                GlobalCheckStatus::EstablishedWithExternalEvidence
            }
        } else {
            GlobalCheckStatus::Partial
        }
    }
}

/// The coordinator's slot census (§10.3).
///
/// Every one of §10.3's eleven checks for one concrete constructor and
/// shape, with what that coordinator's recipe emits, what owes the
/// remainder, and what the target itself owes. The value-related slots
/// name exactly the selected explicit, private, or crossing fragment;
/// alternative composition branches are absent and fail validation if
/// inserted.
///
/// The destination-constructor identity remains
/// [`OutstandingGlobalPattern`]'s single member because no in-script
/// comparison can reach those exact bytes. Confidential-value
/// conservation is external only when the selected composition consumes
/// or creates confidential values; homogeneous explicit carries no such
/// external claim.
///
/// # Why three of the absences remain partial
///
/// A root, a reserve, a projected event, or a burn record would have to
/// occupy a position, and every position is held by exactly one family
/// the coordinator authenticates
/// ([`crate::live_plan::live_family_ranges`]). At a sponsor or fee
/// position that is conclusive: the asset is the reserve asset and the
/// program is pinned exactly. At a destination position it is conclusive
/// only up to the destination constructor's identity — an object of some
/// other family carrying the protocol asset under a taproot program would
/// pass the bytes this candidate emits. So the two families whose members
/// could wear that shape are Partial, and issuance, which is a field
/// rather than a position and is tested on every input, is not.
///
/// # Errors
///
/// [`LiveProgramRefusal::ShapeNotAdmitted`] when `shape` is outside the
/// constructor, or [`LiveProgramRefusal::CoordinatorPlacementInvalid`]
/// if any claimed fragment is absent from the concrete recipe.
pub fn coordinator_placements(
    constructor: &StaticLiveReceiptConstructor,
    shape: LiveTransferShape,
) -> Result<BTreeMap<CoordinatorGlobalCheck, GlobalCheckPlacement>, LiveProgramRefusal> {
    use ExternalEvidenceRole as External;
    use LiveFragmentId as Fragment;

    let recipe = live_program_recipe(constructor, LiveProgramRole::Coordinator, shape)?;
    let value_fragment = recipe_value_fragment(&recipe)?;
    let emitted = recipe
        .components
        .iter()
        .map(|component| component.fragment)
        .collect::<BTreeSet<_>>();
    let confidential_external = if value_fragment == Fragment::ExplicitConservation {
        BTreeSet::new()
    } else {
        BTreeSet::from([External::ConfidentialValueConservation])
    };

    let placements = CoordinatorGlobalCheck::ALL
        .iter()
        .map(|check| {
            (
                *check,
                coordinator_placement(*check, value_fragment, &confidential_external),
            )
        })
        .collect::<BTreeMap<_, _>>();
    validate_coordinator_placements(&placements, &emitted)
        .map_err(|cause| LiveProgramRefusal::CoordinatorPlacementInvalid { cause })?;
    Ok(placements)
}

/// Build one concrete placement from the recipe-selected value branch.
fn coordinator_placement(
    check: CoordinatorGlobalCheck,
    value_fragment: LiveFragmentId,
    confidential_external: &BTreeSet<ExternalEvidenceRole>,
) -> GlobalCheckPlacement {
    use CoordinatorGlobalCheck as Check;
    use LiveFragmentId as Fragment;
    use OutstandingGlobalPattern as Pattern;

    let (emitted, outstanding, external) = match check {
        Check::ExactInputAndOutputCounts => (
            BTreeSet::from([Fragment::Cardinality]),
            BTreeSet::new(),
            BTreeSet::new(),
        ),
        Check::CompleteProtocolRanges => (
            BTreeSet::from([
                Fragment::Cardinality,
                Fragment::CoordinatorRole,
                Fragment::DestinationClosure,
                Fragment::SponsorIsolation,
            ]),
            BTreeSet::new(),
            BTreeSet::new(),
        ),
        Check::LiveClassOutputClosure => (
            BTreeSet::from([
                Fragment::DestinationClosure,
                Fragment::SponsorIsolation,
                Fragment::Cardinality,
            ]),
            BTreeSet::from([Pattern::DestinationConstructorIdentity]),
            BTreeSet::new(),
        ),
        Check::ExplicitAssetClosure => (
            BTreeSet::from([
                Fragment::LocalRecognition,
                Fragment::DestinationClosure,
                Fragment::SponsorIsolation,
                Fragment::Cardinality,
            ]),
            BTreeSet::new(),
            BTreeSet::new(),
        ),
        Check::RepresentationSpecificConservation => (
            BTreeSet::from([
                value_fragment,
                Fragment::LocalRecognition,
                Fragment::Cardinality,
            ]),
            BTreeSet::new(),
            confidential_external.clone(),
        ),
        Check::SponsorIsolation => (
            BTreeSet::from([Fragment::SponsorIsolation, Fragment::Cardinality]),
            BTreeSet::new(),
            BTreeSet::new(),
        ),
        Check::RootsAbsent | Check::SpecializedEventsAbsent => (
            BTreeSet::from([
                Fragment::Cardinality,
                Fragment::DestinationClosure,
                Fragment::SponsorIsolation,
            ]),
            BTreeSet::from([Pattern::DestinationConstructorIdentity]),
            BTreeSet::new(),
        ),
        Check::IssuanceAbsent => (
            BTreeSet::from([Fragment::IssuanceAbsence]),
            BTreeSet::new(),
            BTreeSet::new(),
        ),
        Check::DestructionAbsent => (
            BTreeSet::from([
                value_fragment,
                Fragment::DestinationClosure,
                Fragment::SponsorIsolation,
                Fragment::Cardinality,
            ]),
            BTreeSet::new(),
            confidential_external.clone(),
        ),
        Check::ExactRoleOrder => (
            BTreeSet::from([
                Fragment::CoordinatorRole,
                Fragment::Cardinality,
                Fragment::DestinationClosure,
                Fragment::SponsorIsolation,
            ]),
            BTreeSet::new(),
            BTreeSet::new(),
        ),
    };

    GlobalCheckPlacement {
        check,
        emitted,
        outstanding,
        external,
    }
}

/// Why a coordinator slot census does not stand up.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlacementDefect {
    /// The census names a check twice, or omits one.
    CensusMismatch {
        /// Named by [`CoordinatorGlobalCheck::ALL`] and absent.
        missing: BTreeSet<CoordinatorGlobalCheck>,
        /// Present and not named by the census constant.
        unexpected: BTreeSet<CoordinatorGlobalCheck>,
    },
    /// A check has no emitted instructions, no outstanding pattern, and
    /// no external requirement.
    ///
    /// The defect the census exists to make impossible: a check nobody
    /// emits, nobody owes, and no target rule carries is a check that has
    /// been dropped.
    CheckBelongsToNobody {
        /// The abandoned check.
        check: CoordinatorGlobalCheck,
    },
    /// A check names an external requirement and nothing else.
    ///
    /// §6.3's own rule, as a defect. A coordinator slot answered
    /// *entirely* by the target's consensus behaviour is a slot the
    /// coordinator does not close at all, and recording it as though the
    /// bytes reached it would be the local program claiming a target rule
    /// because the target eventually accepts. Every check naming external
    /// evidence must also emit the closure the evidence applies to.
    ExternalEvidenceWithoutClosure {
        /// The check whose slot is external and nothing else.
        check: CoordinatorGlobalCheck,
    },
    /// A slot claims a fragment the coordinator program does not carry.
    FragmentNotEmitted {
        /// The check whose slot claims it.
        check: CoordinatorGlobalCheck,
        /// The fragment claimed.
        fragment: LiveFragmentId,
    },
    /// No slot names an outstanding pattern this wave does not build.
    ///
    /// A census in which every check is established is a census that has
    /// stopped tracking §10.4 through §10.8 — which are not built.
    NothingOutstanding,
}

/// Validate one coordinator slot census against the fragments a
/// coordinator actually emits.
///
/// `emitted` is the fragment census of a real coordinator program, so
/// the check is against emitted bytes rather than against a second list.
///
/// # Errors
///
/// [`PlacementDefect::CensusMismatch`] when the map is not exactly
/// [`CoordinatorGlobalCheck::ALL`];
/// [`PlacementDefect::CheckBelongsToNobody`] for a check with no owner on
/// any of the three sides;
/// [`PlacementDefect::ExternalEvidenceWithoutClosure`] for a check the
/// target is asked to carry whole;
/// [`PlacementDefect::FragmentNotEmitted`] for a slot claiming a fragment
/// no coordinator carries; and
/// [`PlacementDefect::NothingOutstanding`] for a census that claims
/// §10.4 through §10.8 are already answered.
pub fn validate_coordinator_placements(
    placements: &BTreeMap<CoordinatorGlobalCheck, GlobalCheckPlacement>,
    emitted: &BTreeSet<LiveFragmentId>,
) -> Result<(), PlacementDefect> {
    let named = CoordinatorGlobalCheck::ALL
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let present = placements.keys().copied().collect::<BTreeSet<_>>();
    if named != present {
        return Err(PlacementDefect::CensusMismatch {
            missing: named.difference(&present).copied().collect(),
            unexpected: present.difference(&named).copied().collect(),
        });
    }

    for placement in placements.values() {
        if placement.status() == GlobalCheckStatus::Outstanding
            && placement.outstanding.is_empty()
            && placement.external.is_empty()
        {
            return Err(PlacementDefect::CheckBelongsToNobody {
                check: placement.check,
            });
        }
        if !placement.external.is_empty() && placement.emitted.is_empty() {
            return Err(PlacementDefect::ExternalEvidenceWithoutClosure {
                check: placement.check,
            });
        }
        if let Some(fragment) = placement
            .emitted()
            .find(|fragment| !emitted.contains(fragment))
        {
            return Err(PlacementDefect::FragmentNotEmitted {
                check: placement.check,
                fragment,
            });
        }
    }

    if placements
        .values()
        .all(|placement| placement.outstanding.is_empty())
    {
        return Err(PlacementDefect::NothingOutstanding);
    }

    Ok(())
}

/// The fragments one concrete program carries.
///
/// An exact projection of the recipe the emitter walks. Fragment
/// boundaries do not survive concatenation, so deriving the census from
/// that recipe is what keeps the metadata and the instruction stream on
/// one dispatch.
///
/// # Errors
///
/// The constructor must admit `shape`, and a member role requires a
/// nonzero receipt position.
pub fn emitted_fragments(
    constructor: &StaticLiveReceiptConstructor,
    role: LiveProgramRole,
    shape: LiveTransferShape,
) -> Result<BTreeSet<LiveFragmentId>, LiveProgramRefusal> {
    Ok(live_program_recipe(constructor, role, shape)?
        .components
        .into_iter()
        .map(|component| component.fragment)
        .collect())
}

// --- The two levels of §1.6 -------------------------------------------

/// Why an owner assignment is not a live-transfer receipt assignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OwnerAssignmentRejection {
    /// No receipt input was named.
    ///
    /// §5.1 bounds the receipt-input count below by one, so an empty
    /// assignment is not a narrow transfer but an absent one.
    NoReceiptInput,
}

/// Which owner authorizes each receipt input (§1.6).
///
/// The value is a sequence and not a set, and that is the whole point:
/// §1.6 keeps distinct semantic owners, concrete receipt inputs, and
/// concrete owner signatures apart, and a set would have collapsed the
/// first two the moment one owner held two receipts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReceiptOwnerAssignment {
    owners: Vec<OwnerKey>,
}

impl ReceiptOwnerAssignment {
    /// The assignment naming one owner per receipt input, in input
    /// order.
    ///
    /// # Errors
    ///
    /// [`OwnerAssignmentRejection::NoReceiptInput`] for an empty
    /// offering.
    pub fn new(owners: Vec<OwnerKey>) -> Result<Self, OwnerAssignmentRejection> {
        if owners.is_empty() {
            return Err(OwnerAssignmentRejection::NoReceiptInput);
        }
        Ok(Self { owners })
    }

    /// The distinct semantic owners, in canonical order.
    ///
    /// The set §1.6's semantic authorization is stated over. Three
    /// receipts held by one owner contribute one member here.
    #[must_use]
    pub fn distinct_semantic_owners(&self) -> BTreeSet<OwnerKey> {
        self.owners.iter().cloned().collect()
    }

    /// How many concrete receipt inputs the transfer consumes.
    #[must_use]
    pub const fn concrete_receipt_inputs(&self) -> usize {
        self.owners.len()
    }

    /// How many concrete owner signatures the spend needs.
    ///
    /// One per receipt input, never one per distinct owner. The target's
    /// message is input-specific, and a leaf verifies the signature
    /// offered in its own input's witness, so an owner holding three
    /// receipts signs three times. §1.6 states this as the difference
    /// between semantic authorization and target realization, and this
    /// accessor is the target-realization half.
    #[must_use]
    pub const fn concrete_owner_signatures(&self) -> usize {
        self.owners.len()
    }

    /// The owner authorizing one receipt position.
    #[must_use]
    pub fn owner_at(&self, input: usize) -> Option<&OwnerKey> {
        self.owners.get(input)
    }
}

// --- §1.8 negatives, as fragment mutations ----------------------------

census_enum! {
    /// One mutation of the owner-key literal a live fragment pushes.
    ///
    /// Four, where §1.8 names six negatives. The other two are witness
    /// substitutions rather than fragment mutations — the leaf is
    /// unchanged and a different signature is offered — and
    /// [`negative_disposition`] is where each of §1.8's six is routed to
    /// the one that shows it.
    pub enum OwnerKeyMutation {
        /// The literal is replaced by the empty item.
        EmptyKey,
        /// The literal is replaced by a nonempty encoding the target
        /// does not recognize.
        UnknownNonemptyKeyType,
        /// The literal keeps the approved width and is not a well-formed
        /// member of the class.
        MalformedApprovedKey,
        /// The literal keeps the approved width and is another owner's
        /// key.
        ApprovedKeyOfAnotherOwner,
    }
}

census_enum! {
    /// Which first-party oracle refuses what the abstract walk cannot.
    ///
    /// Two, and both live in the conformance package because both need
    /// arithmetic this crate holds none of and takes no dependency for.
    pub enum OwnerKeyOracle {
        /// Curve arithmetic: the bytes are not the x coordinate of a
        /// point.
        CurvePointMembership,
        /// Signature verification against the key the leaf requires.
        SignatureVerification,
    }
}

/// What the constructor's owner-metadata gate does about a mutation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OwnerKeyMutationGate {
    /// §7.2's gate refuses it, so it is reachable only by surgery on the
    /// emitted fragment.
    RefusedByOwnerMetadata,
    /// §7.2's gate admits it: the offering is the approved encoding at
    /// the approved width, and what is wrong with it is not an encoding
    /// question.
    AdmittedByOwnerMetadata,
}

/// What the abstract walk establishes about one mutated fragment.
///
/// Three outcomes, and the third is not a pass. It says the walk cannot
/// separate the mutant from the pattern and names the oracle that can,
/// which is the honest answer for a mutation whose defect is a fact
/// about a curve point rather than about a width.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OwnerKeyMutationOutcome {
    /// The mutant has no successful path at all.
    ///
    /// The strongest outcome: every path through the mutated fragment
    /// aborts, decided by the operand types before any question about a
    /// signature arises.
    NoSuccessfulPath,
    /// The mutant succeeds, and its success verified nothing.
    ///
    /// §1.8's forward-compatibility path, reopened. The finding is that
    /// the mutant *does* have a successful form, which is exactly why
    /// the unmutated literal has to be the approved encoding.
    ReachesUnverifiedSuccess,
    /// The walk reaches the pattern's own contract, and an oracle
    /// decides it.
    AbstractlyIndistinguishable {
        /// The oracle that refuses the mutant.
        oracle: OwnerKeyOracle,
    },
}

/// Where one of §1.8's six negatives is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NegativeDisposition {
    /// A mutation of the owner-key literal, walked by this crate.
    FragmentMutation {
        /// The mutation that shows it.
        mutation: OwnerKeyMutation,
    },
    /// A substitution in the witness, decided by an oracle.
    ///
    /// The fragment is unmutated: what changes is the signature offered,
    /// and no property of the emitted bytes could tell a signature over
    /// one message from a signature over another.
    WitnessSubstitution {
        /// The oracle that refuses it.
        oracle: OwnerKeyOracle,
    },
}

/// Where §1.8's negative for one owner-key case is shown.
///
/// Exhaustive with no wildcard arm: a negative added to
/// [`OwnerKeyNegative`] stops this crate compiling until somebody says
/// which executable case shows it.
#[must_use]
pub const fn negative_disposition(negative: OwnerKeyNegative) -> NegativeDisposition {
    match negative {
        OwnerKeyNegative::EmptyKey => NegativeDisposition::FragmentMutation {
            mutation: OwnerKeyMutation::EmptyKey,
        },
        OwnerKeyNegative::UnknownNonemptyKeyType => NegativeDisposition::FragmentMutation {
            mutation: OwnerKeyMutation::UnknownNonemptyKeyType,
        },
        OwnerKeyNegative::MalformedApprovedKey => NegativeDisposition::FragmentMutation {
            mutation: OwnerKeyMutation::MalformedApprovedKey,
        },
        OwnerKeyNegative::ApprovedKeyOfAnotherOwner => NegativeDisposition::FragmentMutation {
            mutation: OwnerKeyMutation::ApprovedKeyOfAnotherOwner,
        },
        OwnerKeyNegative::ValidSignatureAgainstAnotherKey
        | OwnerKeyNegative::ValidSignatureOverAnotherTransaction => {
            NegativeDisposition::WitnessSubstitution {
                oracle: OwnerKeyOracle::SignatureVerification,
            }
        }
    }
}

/// What §7.2's gate does about one mutation.
#[must_use]
pub const fn mutation_gate(mutation: OwnerKeyMutation) -> OwnerKeyMutationGate {
    match mutation {
        // An empty offering is the absent form, and an unrecognized
        // nonempty one is either not a key encoding at all or the
        // target's other one. §7.2 names both.
        OwnerKeyMutation::EmptyKey | OwnerKeyMutation::UnknownNonemptyKeyType => {
            OwnerKeyMutationGate::RefusedByOwnerMetadata
        }
        // Both are the approved encoding at the approved width. The gate
        // has nothing to object to, and saying so is the point: this is
        // exactly where §7.2's authority ends and an oracle's begins.
        OwnerKeyMutation::MalformedApprovedKey | OwnerKeyMutation::ApprovedKeyOfAnotherOwner => {
            OwnerKeyMutationGate::AdmittedByOwnerMetadata
        }
    }
}

/// The owner-authorization fragment with one mutation applied.
///
/// # Errors
///
/// [`TapscriptError::OversizedStackItem`] when a mutated literal exceeds
/// the reviewed literal bound, and
/// [`TapscriptError::InstructionLimitExceeded`] when the fragment
/// exceeds the instruction bound.
pub fn mutated_owner_authorization_fragment(
    target: &ReviewedElementsTapscriptDefinition,
    owner: &OwnerKey,
    mutation: OwnerKeyMutation,
) -> Result<TapscriptProgram, TapscriptError> {
    let key = match mutation {
        OwnerKeyMutation::EmptyKey => StackItem::empty(),
        // The width of the target's *other* public-key encoding, whole
        // field and prefix included, read from the registry rather than
        // written down: what makes this mutation the unknown-key case is
        // that the target does not recognize the width, and a transcribed
        // figure could stop being one without anything noticing.
        OwnerKeyMutation::UnknownNonemptyKeyType => StackItem::new(
            target,
            vec![0x02; alternate_key_field_width(target, owner.encoding())],
        )?,
        // Same width, different bytes. The walk cannot separate these
        // two from each other or from the pattern, and neither claims it
        // can: what tells them apart is which oracle refuses them.
        OwnerKeyMutation::MalformedApprovedKey | OwnerKeyMutation::ApprovedKeyOfAnotherOwner => {
            let mut bytes = owner.bytes().to_vec();
            if let Some(first) = bytes.first_mut() {
                *first ^= 0xff;
            }
            StackItem::new(target, bytes)?
        }
    };

    TapscriptProgram::new(vec![
        TapscriptInstruction::Push(key),
        op(OpcodeId::CheckSigVerify),
    ])
}

/// The whole-field width of the target's other public-key encoding.
///
/// Falls back to one byte more than the approved encoding's own whole
/// field where the target names no other key class, which is still a
/// width the target does not recognize and so is still the mutation's
/// subject.
fn alternate_key_field_width(
    target: &ReviewedElementsTapscriptDefinition,
    approved: EncodingClass,
) -> usize {
    let field = |class: EncodingClass| {
        target.definition().encodings().get(&class).map(|spec| {
            let payload = match spec.payload() {
                PayloadWidth::Absent => 0,
                PayloadWidth::Exact(exact) => exact.get(),
                PayloadWidth::Bounded { maximum, .. } => maximum.get(),
            };
            payload + usize::from(!spec.prefixes().is_empty())
        })
    };

    let approved_field = field(approved).unwrap_or_default();
    EncodingClass::ALL
        .iter()
        .copied()
        .filter(|class| *class != approved && class.v1_shape().domain() == EncodingDomain::Key)
        .filter_map(field)
        .find(|width| *width != approved_field)
        .unwrap_or(approved_field + 1)
}

/// What the abstract walk establishes about one mutated fragment.
///
/// # Errors
///
/// Any failure of [`validate_program`] or of the mutated fragment's
/// assembly, which is a defect in this crate rather than a property of
/// the target.
pub fn owner_key_mutation_outcome(
    target: &ReviewedElementsTapscriptDefinition,
    owner: &OwnerKey,
    mutation: OwnerKeyMutation,
) -> Result<OwnerKeyMutationOutcome, TapscriptError> {
    let mutant = mutated_owner_authorization_fragment(target, owner, mutation)?;
    let initial = owner_authorization_precondition(target);
    let result = validate_program(
        target,
        &mutant,
        &initial,
        AbstractLimits::for_target(target),
    )?;

    if result.always_aborts() {
        return Ok(OwnerKeyMutationOutcome::NoSuccessfulPath);
    }
    if result.reaches_unverified_signature_success() {
        return Ok(OwnerKeyMutationOutcome::ReachesUnverifiedSuccess);
    }
    Ok(OwnerKeyMutationOutcome::AbstractlyIndistinguishable {
        oracle: match mutation {
            OwnerKeyMutation::MalformedApprovedKey => OwnerKeyOracle::CurvePointMembership,
            OwnerKeyMutation::EmptyKey
            | OwnerKeyMutation::UnknownNonemptyKeyType
            | OwnerKeyMutation::ApprovedKeyOfAnotherOwner => OwnerKeyOracle::SignatureVerification,
        },
    })
}

// --- §10.9: the final stack -------------------------------------------

/// Why a composed program does not satisfy §10.9.
///
/// Each variant is one of §10.9's five sentences, and the fifth is split
/// by which limit it is about, because a program over the validation
/// budget and a program over the script size are two different problems
/// with two different repairs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FinalStackDefect {
    /// A successful state is not exactly one item.
    SuccessIsNotOneItem {
        /// The state that is not.
        state: AbstractStackState,
    },
    /// The program does not end on the canonical true item.
    DoesNotEndOnTheCanonicalTrueItem,
    /// A state reached through a non-aborting failure survives.
    ///
    /// Every such state carries the canonical true item on top, because
    /// the program's last instruction pushes one, so a surviving
    /// non-aborting failure is precisely a failure state satisfying
    /// final truth.
    NonAbortingFailureSurvives {
        /// The surviving state.
        state: AbstractStackState,
    },
    /// A reachable state leaves something on the alternate stack.
    AlternateStackResidue {
        /// The state with the residue.
        state: AbstractStackState,
    },
    /// A signature primitive can succeed without verifying (§1.8).
    UnverifiedSignatureSuccessPath {
        /// Where in this program, as ephemeral context.
        instruction: usize,
    },
    /// The program charges more of one dimension than the target admits.
    ResourceBoundExceeded {
        /// The dimension.
        dimension: ResourceDimension,
        /// What the program charges.
        charged: u64,
        /// What the target admits.
        maximum: u64,
    },
}

/// Every way one composed program fails §10.9, in canonical order.
///
/// Empty for a program that holds.
///
/// # What is checked elsewhere, and why nothing is repeated here
///
/// Two of §10.9's limits are enforced before this function is reached
/// and would be tautologies in it. The stack depth is the walk's own
/// bound — [`AbstractLimits::for_target`] takes it from the target's
/// consensus limits, and a program exceeding it returns
/// [`TapscriptError::StackLimitExceeded`] instead of a result — and the
/// element width is [`StackItem`]'s, which refuses an oversized literal
/// at construction. A program that reaches this function is inside both.
///
/// # Errors
///
/// Any failure of [`validate_program`], which is a defect in the program
/// rather than a §10.9 finding: a program the walk cannot schedule has
/// no final stack to be wrong about.
pub fn final_stack_defects(
    target: &ReviewedElementsTapscriptDefinition,
    program: &TapscriptProgram,
    initial: &AbstractStackState,
) -> Result<Vec<FinalStackDefect>, TapscriptError> {
    let result = validate_program(target, program, initial, AbstractLimits::for_target(target))?;
    let mut defects = Vec::new();

    if !ends_on_the_canonical_true_item(target, program) {
        defects.push(FinalStackDefect::DoesNotEndOnTheCanonicalTrueItem);
    }
    for state in result.success() {
        if state.main().len() != 1 {
            defects.push(FinalStackDefect::SuccessIsNotOneItem {
                state: state.clone(),
            });
        }
    }
    for state in result.nonaborting_failure() {
        defects.push(FinalStackDefect::NonAbortingFailureSurvives {
            state: state.clone(),
        });
    }
    for state in result.success().iter().chain(result.nonaborting_failure()) {
        if !state.alternate().is_empty() {
            defects.push(FinalStackDefect::AlternateStackResidue {
                state: state.clone(),
            });
        }
    }
    for (instruction, forms) in result.signature_forms() {
        if forms.contains(&SignatureSuccessForm::UnknownKeyTypeUnverified) {
            defects.push(FinalStackDefect::UnverifiedSignatureSuccessPath {
                instruction: *instruction,
            });
        }
    }
    defects.extend(resource_defects(target, program));

    Ok(defects)
}

/// Whether the program's last instruction is the canonical true item.
///
/// A byte-level question rather than a question about the abstract
/// state: the state carries the item's width and not its value, and the
/// canonical true item is a specific literal §10.9 names.
fn ends_on_the_canonical_true_item(
    target: &ReviewedElementsTapscriptDefinition,
    program: &TapscriptProgram,
) -> bool {
    let Ok(canonical) = final_truth_fragment(target) else {
        return false;
    };
    program
        .instructions()
        .last()
        .is_some_and(|last| Some(last) == canonical.instructions().last())
}

/// Every dimension this program charges beyond the target's own bound.
fn resource_defects(
    target: &ReviewedElementsTapscriptDefinition,
    program: &TapscriptProgram,
) -> Vec<FinalStackDefect> {
    let bounds = target.definition().resources().consensus().bounds();

    resource_projection(target, program)
        .into_iter()
        .filter_map(|(dimension, charged)| {
            let maximum = bounds
                .get(&dimension)
                .copied()
                .and_then(ResourceBound::maximum)?;
            (charged > maximum).then_some(FinalStackDefect::ResourceBoundExceeded {
                dimension,
                charged,
                maximum,
            })
        })
        .collect()
}

// --- Pattern records (§10) --------------------------------------------

census_enum! {
    /// One live-transfer pattern this wave mints.
    ///
    /// A census of its own rather than variants of
    /// [`crate::capability::BackendPatternId`], for the reason
    /// [`crate::live_constructor::LiveTransferLeafRole`] is its own type:
    /// the compact-ASH census is mapped to compact-ASH capabilities, and
    /// a live-transfer identity inside it would be reachable from an
    /// assessment that has nothing to do with this operation.
    pub enum LiveTransferPatternId {
        /// §10.1: the executing input is the linked asset, in the
        /// selected representation's form.
        LiveInputRecognitionV1,
        /// §10.2, §1.8: the committed owner's key, and one signature
        /// verified against it.
        LiveOwnerAuthorizationV1,
        /// §10.3: the coordinator leaf spends input 0 and nowhere else.
        LiveCoordinatorRoleV1,
        /// §10.3: a member leaf lies in the nonzero receipt range.
        LiveMemberRoleV1,
        /// §10.3: the target's own counts are exactly the shape's.
        LiveShapeV1,
        /// §10.4: every destination carries the linked protocol asset
        /// under a program read at the constructor's version.
        LiveDestinationClosureV1,
        /// §10.5: the two exact sums, with every flag consumed.
        LiveExplicitConservationV1,
        /// §6.3, §10.6: every destination's value is in the confidential
        /// form, and no payload survives the read.
        ///
        /// Named for what it establishes. It is not a private
        /// conservation pattern and §10.6 mints none: this identity
        /// carries the representation closure the private plan owes, and
        /// the value equation over those commitments belongs to the
        /// target's own rule.
        LivePrivateDestinationFormV1,
        /// §6.5: every destination's value is in the EXPLICIT form save
        /// the one declared absorber position, which is confidential.
        ///
        /// The positional sibling of
        /// [`Self::LivePrivateDestinationFormV1`], and named for what it
        /// establishes for the same reason. It is not a conservation
        /// pattern either, and it is emphatically not a weaker private
        /// form: it establishes MORE about the destinations than the
        /// private one does, because it says which position carries
        /// which form rather than one answer for the whole range.
        LiveCrossingDestinationFormV1,
        /// §10.7: the sponsor region is exact, disjoint, and never read
        /// for an amount.
        LiveSponsorIsolationV1,
        /// §10.8: no input carries an issuance.
        LiveIssuanceAbsenceV1,
        /// §10.3, §10.9: the whole coordinator program.
        LiveCoordinatorProgramV1,
        /// §10.3, §10.9: the whole member program.
        LiveMemberProgramV1,
    }
}

/// The semantic or structural relation one live pattern owns.
///
/// The pattern names the relation rather than restating it: the semantic
/// content stays with the architecture and the compiler's plan, and a
/// target package holding a second copy of it would be a second source
/// of truth for something it does not own (§1.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LivePatternOwner {
    /// §10.1: the executing input is the linked object in the selected
    /// representation.
    LocalPredecessorRecognition,
    /// §10.2: the committed owner authorized this spend.
    PerInputOwnerAuthorization,
    /// §10.3: the coordinator is at the canonical anchor and nowhere
    /// else.
    CoordinatorRole,
    /// §10.3: a member lies in the nonzero receipt range.
    MemberParticipation,
    /// §10.3: the transaction has exactly the shape's counts.
    TransactionShape,
    /// §10.4: every protocol output is a destination of this family.
    OutputConstructorClosure,
    /// §10.5: the exact explicit aggregate, with every flag consumed.
    ExactAggregateArithmetic,
    /// §6.3: every destination carries the selected confidential form,
    /// and no amount is opened to establish it.
    ///
    /// Deliberately not an arithmetic relation and deliberately not named
    /// like one. The owner of the private plan's value *equation* is the
    /// target, and no member of this enum may be read as standing in for
    /// it.
    PrivateDestinationValueForm,
    /// §10.7: the sponsor region is exact, disjoint, and opaque.
    SponsorIsolation,
    /// §10.8: no relation §10.8 forbids is admitted.
    AbsenceRelations,
    /// §10.3, §10.9: one whole per-input program, ending on one truth.
    ComposedInputProgram,
}

/// What a pattern needs from the spender's witness.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiveWitnessRole {
    /// The fragment consumes no witness item.
    NoWitnessItem,
    /// One owner signature over the finalized transaction, under the
    /// selected profile.
    ///
    /// What the target verifies is a signature, and what §1.7 requires
    /// is a signature over the protected data; only a reviewed profile
    /// joins the two, which is why this role was carried with
    /// [`RecognitionResidual::SighashProfileUnreviewed`] until the review
    /// verdict and the re-typing supplied the join.
    OwnerSignature,
}

/// What a builder must supply for a pattern to be emitted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiveConstructibility {
    /// Every literal is a link-time symbol or a shape constant.
    LinkTimeOnly,
    /// A link-time symbol set, and the constructor's committed owner.
    ///
    /// Not a caller-supplied key: §7.6 gives the request no parameter
    /// through which one could arrive, and the owner reaches the
    /// fragment only by way of a constructor that already admitted it.
    CommittedOwnerRequired,
}

census_enum! {
    /// What one pattern's emitted bytes publish (§16.3).
    ///
    /// A disclosure census over the *program*, which is public whatever
    /// the transfer discloses: a leaf is revealed by the control block
    /// of any spend that uses it, so everything a leaf pushes is
    /// eventually public and saying which things those are is the only
    /// honest form of a minimality claim about a program.
    pub enum LiveDisclosure {
        /// The exact protocol asset.
        ProtocolAsset,
        /// The committed owner's public key.
        OwnerPublicKey,
        /// Which representation plan the leaf is built for.
        ValueRepresentationForm,
        /// The receipt-input count the range bound uses.
        ReceiptInputCount,
        /// The destination count the output-side loops use.
        ReceiptOutputCount,
        /// The exact input and output counts of one shape.
        TransactionCounts,
        /// The version a destination's program is read at.
        DestinationProgramVersion,
        /// The exact reserve asset the sponsor roles carry.
        ReserveAsset,
        /// The sponsor-change role's program and the version it is read
        /// at.
        SponsorChangeRole,
        /// The digest standing in for the target fee role's program.
        TargetFeeRole,
        /// The semantic amount domain the receipt values are held to.
        ///
        /// The bounds, never an amount: what a reader of the emitted
        /// bytes learns is that every receipt value is positive and below
        /// the domain's ceiling, which is a fact about the protocol and
        /// not about this transfer.
        SemanticAmountDomain,
    }
}

/// One complete live-transfer proof pattern (§10).
///
/// Constructed only by [`build_live_pattern`], which computes the stack
/// contract, the failure behaviour, the signature forms, and the
/// resources by walking the fragment. There is no constructor that takes
/// them as claims.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveTransferPattern {
    id: LiveTransferPatternId,
    owner: LivePatternOwner,
    prerequisites: BTreeSet<ElementsCapability>,
    fragment: TapscriptProgram,
    precondition: AbstractStackState,
    success: BTreeSet<AbstractStackState>,
    nonaborting: BTreeSet<AbstractStackState>,
    aborts: BTreeSet<FailureCause>,
    signature_forms: BTreeMap<usize, BTreeSet<SignatureSuccessForm>>,
    sources: BTreeSet<RequiredSourceKind>,
    witness: LiveWitnessRole,
    constructibility: LiveConstructibility,
    disclosure: BTreeSet<LiveDisclosure>,
    residuals: BTreeSet<RecognitionResidual>,
    resources: BTreeMap<ResourceDimension, u64>,
    evidence: BTreeSet<TargetEvidenceRequirementId>,
}

impl LiveTransferPattern {
    /// The pattern's identity.
    #[must_use]
    pub const fn id(&self) -> LiveTransferPatternId {
        self.id
    }

    /// The relation it owns.
    #[must_use]
    pub const fn owner(&self) -> LivePatternOwner {
        self.owner
    }

    /// The reviewed target primitives the fragment is built from.
    #[must_use]
    pub const fn prerequisites(&self) -> &BTreeSet<ElementsCapability> {
        &self.prerequisites
    }

    /// The typed instruction fragment.
    #[must_use]
    pub const fn fragment(&self) -> &TapscriptProgram {
        &self.fragment
    }

    /// The stack the fragment is scheduled from.
    #[must_use]
    pub const fn precondition(&self) -> &AbstractStackState {
        &self.precondition
    }

    /// Every successful state the fragment can reach.
    #[must_use]
    pub const fn success(&self) -> &BTreeSet<AbstractStackState> {
        &self.success
    }

    /// Every state reached through a non-aborting failure.
    #[must_use]
    pub const fn nonaborting_failure(&self) -> &BTreeSet<AbstractStackState> {
        &self.nonaborting
    }

    /// Every cause the fragment can abort on.
    #[must_use]
    pub const fn aborts(&self) -> &BTreeSet<FailureCause> {
        &self.aborts
    }

    /// The signature forms each signature primitive can reach.
    #[must_use]
    pub const fn signature_forms(&self) -> &BTreeMap<usize, BTreeSet<SignatureSuccessForm>> {
        &self.signature_forms
    }

    /// The compiler fact-source kinds the pattern routes.
    #[must_use]
    pub const fn sources(&self) -> &BTreeSet<RequiredSourceKind> {
        &self.sources
    }

    /// What the spender's witness must carry.
    #[must_use]
    pub const fn witness(&self) -> LiveWitnessRole {
        self.witness
    }

    /// What a builder must supply.
    #[must_use]
    pub const fn constructibility(&self) -> LiveConstructibility {
        self.constructibility
    }

    /// What the emitted bytes publish.
    #[must_use]
    pub const fn disclosure(&self) -> &BTreeSet<LiveDisclosure> {
        &self.disclosure
    }

    /// What the pattern's claims still owe.
    #[must_use]
    pub const fn residuals(&self) -> &BTreeSet<RecognitionResidual> {
        &self.residuals
    }

    /// The fragment's exact resource cost, by dimension.
    #[must_use]
    pub const fn resources(&self) -> &BTreeMap<ResourceDimension, u64> {
        &self.resources
    }

    /// The target evidence the pattern's correctness depends on.
    #[must_use]
    pub const fn evidence(&self) -> &BTreeSet<TargetEvidenceRequirementId> {
        &self.evidence
    }

    /// Whether this pattern can succeed without verifying a signature.
    ///
    /// False for every pattern in the census, and checked rather than
    /// asserted: it is [`Self::signature_forms`] read for §1.8's
    /// question, over the pattern in hand.
    #[must_use]
    pub fn reaches_unverified_success(&self) -> bool {
        self.signature_forms
            .values()
            .any(|forms| forms.contains(&SignatureSuccessForm::UnknownKeyTypeUnverified))
    }
}

/// Build one live pattern by walking its fragment.
///
/// # Errors
///
/// Any failure of [`validate_program`], which is a defect in the
/// fragment rather than a property of the target: a pattern built from a
/// fragment nobody can schedule would be a pattern nobody can emit.
#[expect(
    clippy::too_many_arguments,
    reason = "the items §10's preamble requires before a pattern may be stated; a builder \
              taking fewer would be a pattern admitted on less"
)]
pub fn build_live_pattern(
    target: &ReviewedElementsTapscriptDefinition,
    id: LiveTransferPatternId,
    owner: LivePatternOwner,
    fragment: TapscriptProgram,
    precondition: AbstractStackState,
    witness: LiveWitnessRole,
    constructibility: LiveConstructibility,
    disclosure: BTreeSet<LiveDisclosure>,
    sources: BTreeSet<RequiredSourceKind>,
    residuals: BTreeSet<RecognitionResidual>,
    evidence: BTreeSet<TargetEvidenceRequirementId>,
) -> Result<LiveTransferPattern, TapscriptError> {
    let result: AbstractExecutionResult = validate_program(
        target,
        &fragment,
        &precondition,
        AbstractLimits::for_target(target),
    )?;

    Ok(LiveTransferPattern {
        id,
        owner,
        prerequisites: fragment_prerequisites(&fragment),
        precondition,
        success: result.success().clone(),
        nonaborting: result.nonaborting_failure().clone(),
        aborts: result.aborts().clone(),
        signature_forms: result.signature_forms().clone(),
        sources,
        witness,
        constructibility,
        disclosure,
        residuals,
        resources: resource_projection(target, &fragment),
        evidence,
        fragment,
    })
}

/// Exact dependency unions for one composed program.
#[derive(Clone, Debug, PartialEq, Eq)]
struct LivePatternDependencies {
    sources: BTreeSet<RequiredSourceKind>,
    evidence: BTreeSet<TargetEvidenceRequirementId>,
}

/// Every authored component-record projection of one recipe.
#[derive(Clone, Debug, PartialEq, Eq)]
struct LiveComposedMetadata {
    dependencies: LivePatternDependencies,
    disclosure: BTreeSet<LiveDisclosure>,
    residuals: BTreeSet<RecognitionResidual>,
}

/// Union exactly the dependency records selected by one recipe.
fn composed_dependencies(
    recipe: &LiveProgramRecipe,
    patterns: &BTreeMap<LiveTransferPatternId, LiveTransferPattern>,
) -> Result<LivePatternDependencies, LiveProgramRefusal> {
    let mut sources = BTreeSet::new();
    let mut evidence = BTreeSet::new();

    for component in &recipe.components {
        let Some(id) = component.pattern else {
            continue;
        };
        let pattern = patterns
            .get(&id)
            .ok_or(LiveProgramRefusal::RecipePatternMissing { pattern: id })?;
        sources.extend(pattern.sources().iter().copied());
        evidence.extend(pattern.evidence().iter().copied());
    }

    let final_truth_is_covered = [
        TargetEvidenceRequirementId::OpcodeSemantics,
        TargetEvidenceRequirementId::EncodingSemantics,
    ]
    .into_iter()
    .all(|requirement| evidence.contains(&requirement));
    if recipe
        .components
        .iter()
        .any(|component| component.fragment == LiveFragmentId::FinalTruth)
        && !final_truth_is_covered
    {
        return Err(LiveProgramRefusal::FinalTruthDependenciesMissing);
    }

    Ok(LivePatternDependencies { sources, evidence })
}

/// Project disclosure and residuals from the same component records.
fn composed_metadata(
    recipe: &LiveProgramRecipe,
    patterns: &BTreeMap<LiveTransferPatternId, LiveTransferPattern>,
) -> Result<LiveComposedMetadata, LiveProgramRefusal> {
    let dependencies = composed_dependencies(recipe, patterns)?;
    let mut disclosure = BTreeSet::new();
    let mut residuals = BTreeSet::new();

    for component in &recipe.components {
        let Some(id) = component.pattern else {
            continue;
        };
        let pattern = patterns
            .get(&id)
            .ok_or(LiveProgramRefusal::RecipePatternMissing { pattern: id })?;
        disclosure.extend(pattern.disclosure().iter().copied());
        residuals.extend(pattern.residuals().iter().copied());
    }

    Ok(LiveComposedMetadata {
        dependencies,
        disclosure,
        residuals,
    })
}

/// Every live-transfer pattern, for one constructor and one shape.
///
/// Exactly [`patterns_for`]'s identities, each built by walking its own
/// fragment, so nothing here is a claim about a fragment that was not
/// scheduled and no identity is filled in with a program that is not
/// its own.
///
/// # Errors
///
/// [`LiveProgramRefusal::ShapeNotAdmitted`] for a shape the constructor
/// emits no program for, and
/// [`LiveProgramRefusal::FragmentRefused`] for any failure of a fragment
/// builder or of the walk.
#[expect(
    clippy::too_many_lines,
    reason = "one block per admitted pattern, each naming its own owner, witness role, \
              disclosure, sources, residuals, and evidence; splitting the census would hide \
              which patterns exist and let one be dropped without the count changing"
)]
pub fn live_transfer_patterns(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &LiveTransferSymbols,
    constructor: &StaticLiveReceiptConstructor,
    shape: LiveTransferShape,
) -> Result<BTreeMap<LiveTransferPatternId, LiveTransferPattern>, LiveProgramRefusal> {
    use LiveConstructibility as Build;
    use LiveDisclosure as Disclose;
    use LivePatternOwner as Owns;
    use LiveTransferPatternId as Id;
    use LiveWitnessRole as Witness;
    use RecognitionResidual as Residual;
    use RequiredSourceKind as Source;
    use TargetEvidenceRequirementId as Evidence;

    if !constructor.shapes().admits(shape) {
        return Err(LiveProgramRefusal::ShapeNotAdmitted { shape });
    }
    let coordinator_recipe = live_program_recipe(constructor, LiveProgramRole::Coordinator, shape)?;
    let member_recipe = if has_member_position(shape) {
        Some(live_program_recipe(
            constructor,
            LiveProgramRole::Member,
            shape,
        )?)
    } else {
        None
    };

    let empty = AbstractStackState::from_main(Vec::new());
    let witnessed = live_program_precondition(target);
    let introspection = [Evidence::OpcodeSemantics, Evidence::EncodingSemantics];
    let mut patterns = BTreeMap::new();

    let recognition = local_recognition_fragment(target, symbols, constructor.representation())?;
    patterns.insert(
        Id::LiveInputRecognitionV1,
        build_live_pattern(
            target,
            Id::LiveInputRecognitionV1,
            Owns::LocalPredecessorRecognition,
            recognition,
            empty.clone(),
            Witness::NoWitnessItem,
            Build::LinkTimeOnly,
            BTreeSet::from([Disclose::ProtocolAsset, Disclose::ValueRepresentationForm]),
            BTreeSet::from([Source::AuthenticatedInputObject]),
            BTreeSet::from([
                Residual::FieldFormSettledOnlyOnTheTarget,
                Residual::LinkedDestinationConstructorIdentity,
            ]),
            introspection
                .into_iter()
                .chain([
                    Evidence::InputIntrospectionSemantics,
                    Evidence::ComparisonSemantics,
                ])
                .collect(),
        )?,
    );

    let authorization = owner_authorization_fragment(target, constructor.owner())?;
    patterns.insert(
        Id::LiveOwnerAuthorizationV1,
        build_live_pattern(
            target,
            Id::LiveOwnerAuthorizationV1,
            Owns::PerInputOwnerAuthorization,
            authorization,
            witnessed.clone(),
            Witness::OwnerSignature,
            Build::CommittedOwnerRequired,
            BTreeSet::from([Disclose::OwnerPublicKey]),
            BTreeSet::from([Source::InputOwnerWitness]),
            BTreeSet::from([Residual::OwnerKeyCurvePointMembership]),
            introspection
                .into_iter()
                .chain([Evidence::SignatureSemantics, Evidence::SighashSemantics])
                .collect(),
        )?,
    );

    let anchor = coordinator_role_fragment(target)?;
    patterns.insert(
        Id::LiveCoordinatorRoleV1,
        build_live_pattern(
            target,
            Id::LiveCoordinatorRoleV1,
            Owns::CoordinatorRole,
            anchor,
            empty.clone(),
            Witness::NoWitnessItem,
            Build::LinkTimeOnly,
            BTreeSet::new(),
            BTreeSet::from([Source::AuthenticatedFamilyCensus]),
            BTreeSet::new(),
            introspection
                .into_iter()
                .chain([Evidence::InputIntrospectionSemantics])
                .collect(),
        )?,
    );

    if has_member_position(shape) {
        let range = live_member_role_fragment(target, shape.receipt_inputs())?;
        patterns.insert(
            Id::LiveMemberRoleV1,
            build_live_pattern(
                target,
                Id::LiveMemberRoleV1,
                Owns::MemberParticipation,
                range,
                empty.clone(),
                Witness::NoWitnessItem,
                Build::LinkTimeOnly,
                BTreeSet::from([Disclose::ReceiptInputCount]),
                BTreeSet::from([Source::AuthenticatedFamilyCensus]),
                BTreeSet::new(),
                introspection
                    .into_iter()
                    .chain([
                        Evidence::InputIntrospectionSemantics,
                        Evidence::ComparisonSemantics,
                        Evidence::ConversionSemantics,
                    ])
                    .collect(),
            )?,
        );
    }

    let counts = live_cardinality_fragment(target, shape)?;
    patterns.insert(
        Id::LiveShapeV1,
        build_live_pattern(
            target,
            Id::LiveShapeV1,
            Owns::TransactionShape,
            counts,
            empty.clone(),
            Witness::NoWitnessItem,
            Build::LinkTimeOnly,
            BTreeSet::from([Disclose::TransactionCounts]),
            BTreeSet::from([Source::AuthenticatedFamilyCensus]),
            BTreeSet::new(),
            introspection
                .into_iter()
                .chain([Evidence::TransactionIntrospectionSemantics])
                .collect(),
        )?,
    );

    let closure = destination_closure_fragment(target, symbols, shape)?;
    patterns.insert(
        Id::LiveDestinationClosureV1,
        build_live_pattern(
            target,
            Id::LiveDestinationClosureV1,
            Owns::OutputConstructorClosure,
            closure,
            empty.clone(),
            Witness::NoWitnessItem,
            Build::LinkTimeOnly,
            BTreeSet::from([
                Disclose::ProtocolAsset,
                Disclose::ReceiptOutputCount,
                Disclose::DestinationProgramVersion,
            ]),
            BTreeSet::from([Source::AuthenticatedOutputObject]),
            BTreeSet::from([
                Residual::FieldFormSettledOnlyOnTheTarget,
                Residual::LinkedDestinationConstructorIdentity,
            ]),
            introspection
                .into_iter()
                .chain([
                    Evidence::OutputIntrospectionSemantics,
                    Evidence::ComparisonSemantics,
                ])
                .collect(),
        )?,
    );

    match recipe_value_fragment(&coordinator_recipe)? {
        LiveFragmentId::ExplicitConservation => {
            let conservation = explicit_conservation_fragment(target, shape)?;
            patterns.insert(
                Id::LiveExplicitConservationV1,
                build_live_pattern(
                    target,
                    Id::LiveExplicitConservationV1,
                    Owns::ExactAggregateArithmetic,
                    conservation,
                    empty.clone(),
                    Witness::NoWitnessItem,
                    Build::LinkTimeOnly,
                    BTreeSet::from([
                        Disclose::ValueRepresentationForm,
                        Disclose::ReceiptInputCount,
                        Disclose::ReceiptOutputCount,
                        Disclose::SemanticAmountDomain,
                    ]),
                    BTreeSet::from([Source::AuthenticatedConsensusValue]),
                    BTreeSet::from([Residual::FieldFormSettledOnlyOnTheTarget]),
                    introspection
                        .into_iter()
                        .chain([
                            Evidence::InputIntrospectionSemantics,
                            Evidence::OutputIntrospectionSemantics,
                            Evidence::ArithmeticSemantics,
                            Evidence::ComparisonSemantics,
                        ])
                        .collect(),
                )?,
            );
        }
        LiveFragmentId::PrivateDestinationForm => {
            let form = private_destination_form_fragment(target, shape)?;
            patterns.insert(
                Id::LivePrivateDestinationFormV1,
                build_live_pattern(
                    target,
                    Id::LivePrivateDestinationFormV1,
                    Owns::PrivateDestinationValueForm,
                    form,
                    empty.clone(),
                    Witness::NoWitnessItem,
                    Build::LinkTimeOnly,
                    // No `SemanticAmountDomain`: the private plan holds
                    // no receipt amount to a bound, because it reads
                    // none. The domain is the target proof policy's
                    // (§6.4), and a disclosure census claiming it here
                    // would be publishing a check nobody emitted.
                    BTreeSet::from([
                        Disclose::ValueRepresentationForm,
                        Disclose::ReceiptOutputCount,
                    ]),
                    BTreeSet::from([Source::AuthenticatedOutputObject]),
                    BTreeSet::from([Residual::FieldFormSettledOnlyOnTheTarget]),
                    // The confidential-value conservation requirement is
                    // cited as evidence this pattern's *context* rests
                    // on, not as something the fragment establishes:
                    // §6.3's relation is sound only where that target
                    // rule holds, and a pattern that named no such
                    // dependency would read as though it did not need it.
                    introspection
                        .into_iter()
                        .chain([
                            Evidence::OutputIntrospectionSemantics,
                            Evidence::ComparisonSemantics,
                            Evidence::ConfidentialValueConservation,
                        ])
                        .collect(),
                )?,
            );
        }
        LiveFragmentId::CrossingDestinationForm => {
            let form = crossing_destination_form_fragment(target, shape)?;
            patterns.insert(
                Id::LiveCrossingDestinationFormV1,
                build_live_pattern(
                    target,
                    Id::LiveCrossingDestinationFormV1,
                    Owns::PrivateDestinationValueForm,
                    form,
                    empty.clone(),
                    Witness::NoWitnessItem,
                    Build::LinkTimeOnly,
                    // The same census the private form declares, and for
                    // the same reasons. No `SemanticAmountDomain`: this
                    // fragment reads no amount either. Its explicit
                    // destinations' amounts ARE readable, and it still
                    // does not read them -- every payload is dropped
                    // where it stands -- so a domain claimed here would
                    // be publishing a check nobody emitted.
                    BTreeSet::from([
                        Disclose::ValueRepresentationForm,
                        Disclose::ReceiptOutputCount,
                    ]),
                    BTreeSet::from([Source::AuthenticatedOutputObject]),
                    BTreeSet::from([Residual::FieldFormSettledOnlyOnTheTarget]),
                    // The confidential-value conservation requirement is
                    // cited for the private form's reason and one more:
                    // an exit crossing can read its created total and
                    // not its consumed one, so the equality between them
                    // is the target's rule and there is no first-party
                    // arithmetic that could stand in for it.
                    introspection
                        .into_iter()
                        .chain([
                            Evidence::OutputIntrospectionSemantics,
                            Evidence::ComparisonSemantics,
                            Evidence::ConfidentialValueConservation,
                        ])
                        .collect(),
                )?,
            );
        }
        _ => return Err(LiveProgramRefusal::RecipeValueComponentMissing),
    }

    if recipe_selects_pattern(&coordinator_recipe, Id::LiveSponsorIsolationV1) {
        let isolation = live_sponsor_isolation_fragment(target, symbols, shape)?;
        // Every clause this fragment can emit reads the reserve asset,
        // and the fee clause additionally reads the destination count to
        // find the fee's position, so those three are disclosed by any
        // shape that reaches the fragment at all. The change role is the
        // one clause a sponsorless fee-bearing shape never emits, so it
        // is disclosed on the sponsor question rather than on this one.
        let mut isolation_disclosure = BTreeSet::from([
            Disclose::ReserveAsset,
            Disclose::TargetFeeRole,
            Disclose::ReceiptOutputCount,
        ]);
        if has_sponsor_region(shape) {
            isolation_disclosure.insert(Disclose::SponsorChangeRole);
        }
        patterns.insert(
            Id::LiveSponsorIsolationV1,
            build_live_pattern(
                target,
                Id::LiveSponsorIsolationV1,
                Owns::SponsorIsolation,
                isolation,
                empty.clone(),
                Witness::NoWitnessItem,
                Build::LinkTimeOnly,
                isolation_disclosure,
                BTreeSet::from([Source::AuthenticatedFamilyCensus]),
                BTreeSet::from([Residual::FieldFormSettledOnlyOnTheTarget]),
                introspection
                    .into_iter()
                    .chain([
                        Evidence::InputIntrospectionSemantics,
                        Evidence::OutputIntrospectionSemantics,
                        Evidence::ComparisonSemantics,
                        Evidence::FeeOutputForm,
                        Evidence::ConfidentialValueConservation,
                    ])
                    .collect(),
            )?,
        );
    }

    let absence = issuance_absence_fragment(target, shape)?;
    patterns.insert(
        Id::LiveIssuanceAbsenceV1,
        build_live_pattern(
            target,
            Id::LiveIssuanceAbsenceV1,
            Owns::AbsenceRelations,
            absence,
            empty,
            Witness::NoWitnessItem,
            Build::LinkTimeOnly,
            BTreeSet::from([Disclose::TransactionCounts]),
            BTreeSet::from([Source::AuthenticatedInputObject]),
            BTreeSet::new(),
            introspection
                .into_iter()
                .chain([
                    Evidence::InputIntrospectionSemantics,
                    Evidence::IssuanceIntrospection,
                    Evidence::ComparisonSemantics,
                ])
                .collect(),
        )?,
    );

    // The composed programs carry §10.9's whole-program claim. Every
    // adjacent metadata set is projected from the same recipe that
    // composes the bytes; no shared composed literal survives.
    let coordinator_program =
        compose_live_program(target, symbols, constructor, shape, &coordinator_recipe)?.0;
    let coordinator_metadata = composed_metadata(&coordinator_recipe, &patterns)?;
    let mut composed = vec![(
        Id::LiveCoordinatorProgramV1,
        coordinator_program,
        coordinator_metadata,
    )];
    if let Some(member_recipe) = member_recipe {
        let member_program =
            compose_live_program(target, symbols, constructor, shape, &member_recipe)?.0;
        let member_metadata = composed_metadata(&member_recipe, &patterns)?;
        composed.push((Id::LiveMemberProgramV1, member_program, member_metadata));
    }

    for (id, program, metadata) in composed {
        patterns.insert(
            id,
            build_live_pattern(
                target,
                id,
                Owns::ComposedInputProgram,
                program,
                witnessed.clone(),
                Witness::OwnerSignature,
                Build::CommittedOwnerRequired,
                metadata.disclosure,
                metadata.dependencies.sources,
                metadata.residuals,
                metadata.dependencies.evidence,
            )?,
        );
    }

    Ok(patterns)
}

/// Whether one shape has a receipt position a member leaf can occupy.
///
/// The one-to-one transfer does not: input 0 is the coordinator and
/// there is nothing after it (§7.1, §10.3).
#[must_use]
pub const fn has_member_position(shape: LiveTransferShape) -> bool {
    shape.receipt_inputs() > 1
}

/// Which pattern identities one shape and one representation call for.
///
/// Three identities are conditional, and each is omitted rather than
/// minted over a fragment that would say nothing:
///
/// - the two member records, for a shape with no nonzero receipt
///   position. A member record built for it would be either a range check
///   no spend can satisfy — `1 ≤ i < 1` — or the coordinator's own program
///   wearing the member identity, and a reader inspecting the member
///   pattern would be shown the wrong program;
/// - the sponsor-isolation record, for a shape with no sponsor region.
///   Its fragment is empty, because the region's absence is established
///   by the exact counts; a record over zero instructions would carry an
///   empty prerequisite census and an empty resource formula and would
///   read as a pattern that had been checked;
/// - the explicit-conservation record, for the private-committed plan,
///   which admits no amount inspection at all (§10.6); and the private
///   destination-form record, for the explicit plan, which settles the
///   same form fact inside its arithmetic and would otherwise test one
///   field twice.
///
/// The last two are a partition rather than two independent conditions,
/// and that is a property a census test executes: every shape's selection
/// carries exactly one value obligation, whichever representation is
/// chosen.
#[must_use]
pub fn patterns_for(
    shape: LiveTransferShape,
    representation: LiveTransferRepresentationPlan,
) -> BTreeSet<LiveTransferPatternId> {
    patterns_for_composition(shape, LiveTransferComposition::homogeneous(representation))
}

/// Which patterns one shape selects under a stated COMPOSITION.
///
/// The general form, of which [`patterns_for`] is the homogeneous case.
/// The value obligation is a partition over compositions and not over
/// plans, which is what keeps a crossing coordinator from selecting two
/// value obligations or none.
#[must_use]
pub fn patterns_for_composition(
    shape: LiveTransferShape,
    composition: LiveTransferComposition,
) -> BTreeSet<LiveTransferPatternId> {
    LiveTransferPatternId::ALL
        .iter()
        .copied()
        .filter(|id| match id {
            LiveTransferPatternId::LiveMemberRoleV1
            | LiveTransferPatternId::LiveMemberProgramV1 => has_member_position(shape),
            LiveTransferPatternId::LiveSponsorIsolationV1 => emits_isolation_fragment(shape),
            // The three value obligations are a PARTITION over
            // compositions: exactly one holds for any composition, which
            // is what a census test executes rather than what this
            // comment asserts.
            LiveTransferPatternId::LiveExplicitConservationV1 => {
                composition == LiveTransferComposition::HomogeneousExplicit
            }
            LiveTransferPatternId::LivePrivateDestinationFormV1 => matches!(
                composition,
                LiveTransferComposition::HomogeneousPrivate
                    | LiveTransferComposition::EntryBlinding
            ),
            LiveTransferPatternId::LiveCrossingDestinationFormV1 => {
                composition == LiveTransferComposition::ExitUnblinding
            }
            _ => true,
        })
        .collect()
}

/// What the reviewed target obliges of a live owner-authorization
/// pattern.
///
/// Read off the constructor's retained closure rather than assumed, so a
/// target that refused unrecognized keys would move the answer without
/// an edit here.
#[must_use]
pub const fn owner_key_obligation(
    constructor: &StaticLiveReceiptConstructor,
) -> OwnerKeyObligation {
    constructor.owner_encoding().obligation()
}

/// What the review establishes about the profile these signatures are
/// taken under (§9.2).
///
/// # Why the profile is not a field of the constructor
///
/// It could have been. It is not, because it is not a property of one
/// constructor: [`selected_owner_profile`] is the candidate's single
/// selection, §11.2 lists it among the *link-time symbols*, and §11.6
/// has the linked bundle retain it. A copy on every constructor would be
/// one more place for the selection to be changed in, and every
/// constructor derived before a change would go on reporting the old
/// one.
///
/// Nothing in this wave needs it there either. The profile decides which
/// message the target builds, and no byte an authorization fragment
/// pushes depends on that message — the fragment pushes a key and
/// verifies whatever the witness offers against it. The profile enters
/// at the moment a signing request is built, which is §12's, and is
/// answered here as a disposition rather than carried as a field.
///
/// # What it answers today
///
/// [`OwnerProfileDisposition::Established`], and no pattern asserting a
/// signature carries [`RecognitionResidual::SighashProfileUnreviewed`]
/// any longer, because there is nothing left for it to name. It answered
/// [`OwnerProfileDisposition::ReviewIncomplete`] until the review
/// populated the reviewed contract and the owner re-typed the one
/// dimension the review could not exercise, and it moved without this
/// function changing — which is the whole reason the disposition is
/// answered here rather than carried as a field.
#[must_use]
pub fn live_owner_profile_disposition(
    target: &ReviewedElementsTapscriptDefinition,
) -> OwnerProfileDisposition {
    selected_owner_profile().assess(target.definition().authorization().sighash())
}
