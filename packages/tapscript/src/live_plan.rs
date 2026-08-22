//! The explicit live-transfer plan (Guide-13 §10.4 – §10.8, §6.2).
//!
//! # What the coordinator gains here
//!
//! [`crate::live_pattern`] built the per-input half of §10: each receipt
//! input recognizes the object it is spending and verifies its owner's
//! signature. That is local by construction and says nothing about the
//! transaction as a whole, so the eleven transaction-global checks §10.3
//! gives the coordinator were declared outstanding rather than emitted.
//!
//! This module emits four of the five patterns they were waiting on —
//! the destination closure of §10.4, the exact explicit conservation of
//! §10.5, the sponsor isolation of §10.7, and the absence relations of
//! §10.8 — and states the family ranges of §12.1 and §12.2 as a census
//! whose completeness is checkable rather than asserted. §10.6's private
//! conservation is not here and is not pretended to be: it is Wave 7's,
//! and [`crate::live_pattern::OutstandingGlobalPattern`] still names it.
//!
//! # Every position, exactly once
//!
//! The absence relations do not have a fragment of their own for the
//! object families §10.8 forbids, and that is the design rather than a
//! gap. A root, a reserve, a burn record, or a projected event would have
//! to occupy a position of the transaction, and the coordinator pins the
//! exact input and output counts and then classifies every position: the
//! receipt range by the role fragments and the local pair, the sponsor
//! suffix and the sponsor output roles by §10.7, and the destination
//! range by §10.4. [`live_family_ranges`] is that classification as data,
//! and [`family_range_defects`] is what makes "no gap and no overlap" a
//! statement a test executes over the shape rather than a sentence.
//!
//! Issuance is the one absence no position census reaches, because an
//! issuance is a field of an input rather than an object at a position.
//! [`issuance_absence_fragment`] is therefore emitted, over every input
//! of the transaction and not only the receipts.
//!
//! # What §10.4 establishes, and the one thing it cannot
//!
//! A live-receipt constructor is owner-parameterized (§7.6), and a
//! transfer's destination owners are chosen per request (§12.3). So a
//! coordinator leaf fixed at construction cannot carry a literal for a
//! destination's program: the value does not exist when the leaf is
//! built. What the leaf can compare is the destination's *asset*, which
//! is the linked protocol asset, and the *version* its program is read
//! at, which is the one every live-receipt constructor takes.
//!
//! The remaining half — that the exact program bytes are the taproot
//! output over that destination owner's leaf set — is
//! [`crate::live_pattern::RecognitionResidual::LinkedDestinationConstructorIdentity`],
//! and it is the same residual the induction of §10.4 reduces Wave 5's
//! cross-input question to. See
//! [`crate::live_pattern::recognition_establishments`], where the
//! induction is stated.
//!
//! # No amount leaves the explicit coordinator
//!
//! [`explicit_conservation_fragment`] is the only fragment in this crate
//! that reads a receipt amount, and §1.9 keeps every sponsor amount out
//! of even that one: [`live_sponsor_isolation_fragment`] schedules no
//! value introspection at all, which is a property of the emitted bytes
//! and is checked as one.

use std::collections::BTreeSet;

use target_elements::{EncodingClass, OpcodeId, ReviewedElementsTapscriptDefinition};

use crate::bundle::FieldSide;
use crate::capability::census_enum;
use crate::error::TapscriptError;
use crate::instruction::{StackItem, TapscriptInstruction};
use crate::live_pattern::{LiveFragmentId, LiveTransferSymbols};
use crate::live_shape::LiveTransferShape;
use crate::pattern::{
    narrow_to_operand, number, op, require_amount_domain, require_asset, require_explicit,
    require_program,
};
use crate::program::TapscriptProgram;
use crate::shape::SponsorChangePresence;

// --- Family ranges (§10.3, §12.1, §12.2) ------------------------------

census_enum! {
    /// The role one run of input positions holds (§12.1).
    pub enum LiveInputFamily {
        /// Input 0, which is the coordinator and also a receipt.
        Coordinator,
        /// A nonzero receipt position.
        Member,
        /// A member of the isolated sponsor suffix.
        Sponsor,
    }
}

census_enum! {
    /// The role one run of output positions holds (§12.2).
    pub enum LiveOutputFamily {
        /// A created live receipt.
        Destination,
        /// The optional sponsor-change role.
        SponsorChange,
        /// The target's own fee role.
        TargetFee,
    }
}

/// Which family occupies one run of positions.
///
/// One enum would have merged the two sides, and the two sides are not
/// interchangeable: an input family and an output family with the same
/// name would still be different relations, and a census that compared
/// them would be comparing a spend with a creation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiveFamily {
    /// An input-side run.
    Input(LiveInputFamily),
    /// An output-side run.
    Output(LiveOutputFamily),
}

impl LiveFamily {
    /// Which side of the transaction the run lies on.
    #[must_use]
    pub const fn side(self) -> FieldSide {
        match self {
            Self::Input(_) => FieldSide::Input,
            Self::Output(_) => FieldSide::Output,
        }
    }
}

/// One contiguous run of positions holding one family (§12.1, §12.2).
///
/// Half-open, and never empty: a range covering nothing is not a narrow
/// family but an absent one, and [`live_family_ranges`] omits the role
/// instead of stating a run of zero positions.
///
/// The fragments are carried beside the bounds because that is what
/// makes the census evidence rather than a diagram. A run whose
/// positions no emitted fragment authenticates is a run the coordinator
/// has merely described, and [`FamilyRangeDefect::RangeAuthenticatedByNothing`]
/// is what says so.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LiveFamilyRange {
    family: LiveFamily,
    first: u16,
    end: u16,
    authenticated_by: BTreeSet<LiveFragmentId>,
}

impl LiveFamilyRange {
    /// State one run of positions and what authenticates it.
    ///
    /// Unchecked, deliberately: this type is the *claim*, and
    /// [`family_range_defects`] is the check. Splitting them is what lets
    /// a consumer state a census of its own and have the same validator
    /// run over it — and what lets the validator's own refusals be
    /// executed rather than described.
    #[must_use]
    pub const fn new(
        family: LiveFamily,
        first: u16,
        end: u16,
        authenticated_by: BTreeSet<LiveFragmentId>,
    ) -> Self {
        Self {
            family,
            first,
            end,
            authenticated_by,
        }
    }

    /// The family these positions hold.
    #[must_use]
    pub const fn family(&self) -> LiveFamily {
        self.family
    }

    /// The first position of the run.
    #[must_use]
    pub const fn first(&self) -> u16 {
        self.first
    }

    /// One past the last position of the run.
    #[must_use]
    pub const fn end(&self) -> u16 {
        self.end
    }

    /// How many positions the run covers, which is never zero.
    #[must_use]
    pub const fn count(&self) -> u16 {
        self.end.saturating_sub(self.first)
    }

    /// Whether the run holds this exact position.
    #[must_use]
    pub const fn holds(&self, position: u16) -> bool {
        self.first <= position && position < self.end
    }

    /// The emitted fragments that authenticate the run, in census order.
    pub fn authenticated_by(&self) -> impl Iterator<Item = LiveFragmentId> + '_ {
        self.authenticated_by.iter().copied()
    }
}

/// Every position of one shape's transaction, classified (§12.1, §12.2).
///
/// Derived from the shape rather than declared beside it, and then
/// required to account for every position exactly once — which is the
/// whole content of §10.3's *complete protocol ranges* and, together with
/// the exact counts, of §10.8's absence of every other object family.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CompleteFamilyRanges {
    shape: LiveTransferShape,
    inputs: Vec<LiveFamilyRange>,
    outputs: Vec<LiveFamilyRange>,
}

impl CompleteFamilyRanges {
    /// State one shape's classification, in position order.
    ///
    /// Unchecked for the reason [`LiveFamilyRange::new`] gives: the
    /// completeness this type is named for is
    /// [`family_range_defects`]'s answer about a value, never a property
    /// a constructor conferred by accepting one.
    #[must_use]
    pub const fn new(
        shape: LiveTransferShape,
        inputs: Vec<LiveFamilyRange>,
        outputs: Vec<LiveFamilyRange>,
    ) -> Self {
        Self {
            shape,
            inputs,
            outputs,
        }
    }

    /// The shape these ranges belong to.
    #[must_use]
    pub const fn shape(&self) -> LiveTransferShape {
        self.shape
    }

    /// Every input run, in position order.
    #[must_use]
    pub fn inputs(&self) -> &[LiveFamilyRange] {
        &self.inputs
    }

    /// Every output run, in position order.
    #[must_use]
    pub fn outputs(&self) -> &[LiveFamilyRange] {
        &self.outputs
    }

    /// Every run on either side, inputs first.
    pub fn ranges(&self) -> impl Iterator<Item = &LiveFamilyRange> {
        self.inputs.iter().chain(&self.outputs)
    }

    /// The run holding one family, if the shape has one.
    #[must_use]
    pub fn range(&self, family: LiveFamily) -> Option<&LiveFamilyRange> {
        self.ranges().find(|range| range.family == family)
    }
}

/// Why one family-range census does not cover the transaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FamilyRangeDefect {
    /// Two runs claim one position.
    PositionClaimedTwice {
        /// Which side.
        side: FieldSide,
        /// The position.
        position: u16,
    },
    /// No run claims one position.
    ///
    /// The defect the census exists to make impossible: a position no
    /// family holds is a position any object family could occupy, and
    /// §10.8's absence relations rest on there being none.
    PositionUnaccounted {
        /// Which side.
        side: FieldSide,
        /// The position.
        position: u16,
    },
    /// A run covers no position.
    RangeIsEmpty {
        /// The family whose run is empty.
        family: LiveFamily,
    },
    /// A run's positions are authenticated by no emitted fragment.
    RangeAuthenticatedByNothing {
        /// The family whose run nothing establishes.
        family: LiveFamily,
    },
}

/// Classify every position of one shape's transaction (§12.1, §12.2).
///
/// The receipt family occupies the input prefix with the coordinator at
/// input 0, the sponsor suffix follows, the destinations occupy the
/// output prefix, and the optional sponsor-change and fee roles follow.
/// A role the shape does not carry contributes no run at all rather than
/// an empty one.
///
/// # Why the one-to-one shape has no member run
///
/// Its only receipt is the coordinator's, so a member run would be the
/// half-open range `1..1`. That is not a family with no members; it is a
/// family with no positions, and stating it would make the census claim
/// a run [`family_range_defects`] would then have to refuse.
#[must_use]
pub fn live_family_ranges(shape: LiveTransferShape) -> CompleteFamilyRanges {
    use LiveFragmentId as Fragment;
    use LiveInputFamily as In;
    use LiveOutputFamily as Out;

    let (receipts_first, receipts_end) = shape.receipt_input_range();
    let (sponsor_first, sponsor_end) = shape.sponsor_range();
    let (destinations_first, destinations_end) = shape.destination_range();

    let mut inputs = vec![
        // Input 0 is authenticated as the coordinator by the anchor
        // fragment, and as a receipt by the local pair every receipt
        // input runs — the coordinator included (§10.3).
        range(
            LiveFamily::Input(In::Coordinator),
            receipts_first,
            receipts_first + 1,
            &[
                Fragment::CoordinatorRole,
                Fragment::Cardinality,
                Fragment::LocalRecognition,
                Fragment::OwnerAuthorization,
            ],
        ),
    ];
    if receipts_end > receipts_first + 1 {
        inputs.push(range(
            LiveFamily::Input(In::Member),
            receipts_first + 1,
            receipts_end,
            &[
                Fragment::MemberRole,
                Fragment::Cardinality,
                Fragment::LocalRecognition,
                Fragment::OwnerAuthorization,
            ],
        ));
    }
    if sponsor_end > sponsor_first {
        inputs.push(range(
            LiveFamily::Input(In::Sponsor),
            sponsor_first,
            sponsor_end,
            &[Fragment::SponsorIsolation, Fragment::Cardinality],
        ));
    }

    let mut outputs = vec![range(
        LiveFamily::Output(Out::Destination),
        destinations_first,
        destinations_end,
        &[Fragment::DestinationClosure, Fragment::Cardinality],
    )];
    let mut next = destinations_end;
    if shape.sponsor_change() == SponsorChangePresence::Present {
        outputs.push(range(
            LiveFamily::Output(Out::SponsorChange),
            next,
            next + 1,
            &[Fragment::SponsorIsolation, Fragment::Cardinality],
        ));
        next += 1;
    }
    if shape.sponsored() {
        outputs.push(range(
            LiveFamily::Output(Out::TargetFee),
            next,
            next + 1,
            &[Fragment::SponsorIsolation, Fragment::Cardinality],
        ));
    }

    CompleteFamilyRanges::new(shape, inputs, outputs)
}

/// One run, with the fragments that authenticate it.
fn range(
    family: LiveFamily,
    first: u16,
    end: u16,
    authenticated_by: &[LiveFragmentId],
) -> LiveFamilyRange {
    LiveFamilyRange::new(
        family,
        first,
        end,
        authenticated_by.iter().copied().collect(),
    )
}

/// Every way one family-range census fails to cover the transaction.
///
/// Empty for a census that holds. A function rather than an assertion
/// inside the builder, for the reason
/// [`crate::live_constructor::mutation_census_defects`] is one: a
/// consumer relying on the ranges is entitled to check them over a value
/// it holds, and a census checked only where it was built would be a
/// census a later edit could stop checking.
#[must_use]
pub fn family_range_defects(ranges: &CompleteFamilyRanges) -> Vec<FamilyRangeDefect> {
    let mut defects = Vec::new();

    for range in ranges.ranges() {
        if range.count() == 0 {
            defects.push(FamilyRangeDefect::RangeIsEmpty {
                family: range.family,
            });
        }
        if range.authenticated_by.is_empty() {
            defects.push(FamilyRangeDefect::RangeAuthenticatedByNothing {
                family: range.family,
            });
        }
    }

    defects.extend(cover(
        FieldSide::Input,
        ranges.shape.inputs(),
        &ranges.inputs,
    ));
    defects.extend(cover(
        FieldSide::Output,
        ranges.shape.outputs(),
        &ranges.outputs,
    ));

    defects
}

/// Require every position below `total` to be claimed exactly once.
fn cover(side: FieldSide, total: u16, ranges: &[LiveFamilyRange]) -> Vec<FamilyRangeDefect> {
    (0..total)
        .filter_map(
            |position| match ranges.iter().filter(|range| range.holds(position)).count() {
                1 => None,
                0 => Some(FamilyRangeDefect::PositionUnaccounted { side, position }),
                _ => Some(FamilyRangeDefect::PositionClaimedTwice { side, position }),
            },
        )
        .collect()
}

// --- §10.4: output constructor closure --------------------------------

/// Close the destination range (§10.4).
///
/// Every destination carries the exact linked protocol asset in its
/// explicit form, and its program is read at the version every
/// live-receipt constructor takes. The program payload is dropped rather
/// than compared: see the module documentation for why no leaf can hold a
/// literal for it, and
/// [`crate::live_pattern::RecognitionResidual::LinkedDestinationConstructorIdentity`]
/// for what that leaves owed.
///
/// The other half of §10.4 — that no output carrying the protocol asset
/// lies outside this range — is not here either, and is not owed: it is
/// [`live_sponsor_isolation_fragment`]'s, which requires the reserve
/// asset at every output position this range does not hold, and the
/// exact output count, which leaves no third kind of position.
///
/// # Errors
///
/// [`TapscriptError::ScriptNumberOutOfRange`] or
/// [`TapscriptError::OversizedStackItem`] when a literal this fragment
/// pushes is outside what the reviewed contract admits,
/// [`TapscriptError::MalformedEncodedItem`] when an encoding class states
/// no prefix to compare against, and
/// [`TapscriptError::InstructionLimitExceeded`] when the fragment exceeds
/// the instruction bound.
pub fn destination_closure_fragment(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &LiveTransferSymbols,
    shape: LiveTransferShape,
) -> Result<TapscriptProgram, TapscriptError> {
    let (first, end) = shape.destination_range();
    let mut instructions = Vec::new();

    for position in first..end {
        let position = i64::from(position);
        instructions.extend(require_asset(
            target,
            OpcodeId::InspectOutputAsset,
            position,
            symbols.protocol_asset(),
        )?);
        instructions.extend([
            number(target, position)?,
            op(OpcodeId::InspectOutputScriptPubKey),
            number(target, symbols.destination_program_version())?,
            op(OpcodeId::EqualVerify),
            // The payload is the owner-parameterized part, and this leaf
            // has no value to compare it with. Dropping it is what keeps
            // the residual a residual rather than an unconsumed operand
            // a later fragment could take for something it is not.
            op(OpcodeId::Drop),
        ]);
    }

    TapscriptProgram::new(instructions)
}

// --- §10.5: exact explicit aggregate conservation ---------------------

/// Sum both sides exactly and require them equal (§10.5, §6.2, §5.3).
///
/// The two sums are computed independently: every destination's amount is
/// read and folded to one total, then every receipt input's amount is read
/// and folded to a second, and the two are compared. Nothing is carried
/// between them and neither is derived from the other, which is what
/// §6.2's *independently* asks for.
///
/// # Why this is value-parametric (§1.3)
///
/// The fragment is a function of the shape's two counts and of nothing
/// else. No amount appears in it, so a split is `(1, m)`, a merge is
/// `(n, 1)`, and a redistribution is `(n, m)` — the same instructions
/// over different counts, and the exact semantic value is preserved
/// across every shape the set admits (§5.3) because the equality is over
/// the totals rather than over any denomination.
///
/// The two literals that do appear are the semantic domain's own bounds
/// (§5.1's `x > 0`), pushed by the domain schedule this crate shares with
/// compact ASH. They are the domain, never an amount.
///
/// # Every flag is consumed where it is produced
///
/// Each addition is followed immediately by the verifying primitive, so
/// an overflow ends the spend at the addition that overflowed rather than
/// leaving a false for a later instruction to reduce. The closing
/// equality is a verifying form for the same reason, and
/// [`crate::live_pattern::final_stack_defects`] is where a program that
/// left one behind would be reported.
///
/// The soundness of the closing comparison is Wave 5's: a false equality
/// followed by the verifying form has no abstract success path, because
/// the walk carries exact computed Boolean facts through composed
/// instructions. That is established in [`crate::stack`] and is cited
/// here rather than restated.
///
/// # Errors
///
/// [`TapscriptError::ScriptNumberOutOfRange`] or
/// [`TapscriptError::OversizedStackItem`] when a literal this fragment
/// pushes is outside what the reviewed contract admits,
/// [`TapscriptError::MalformedEncodedItem`] when an encoding class states
/// no prefix to compare against, and
/// [`TapscriptError::InstructionLimitExceeded`] when the fragment exceeds
/// the instruction bound.
pub fn explicit_conservation_fragment(
    target: &ReviewedElementsTapscriptDefinition,
    shape: LiveTransferShape,
) -> Result<TapscriptProgram, TapscriptError> {
    let mut instructions = Vec::new();

    let (destinations_first, destinations_end) = shape.destination_range();
    for position in destinations_first..destinations_end {
        instructions.extend(explicit_amount(
            target,
            OpcodeId::InspectOutputValue,
            i64::from(position),
        )?);
    }
    instructions.extend(fold(shape.receipt_outputs()));

    let (receipts_first, receipts_end) = shape.receipt_input_range();
    for position in receipts_first..receipts_end {
        instructions.extend(explicit_amount(
            target,
            OpcodeId::InspectInputValue,
            i64::from(position),
        )?);
    }
    instructions.extend(fold(shape.receipt_inputs()));

    // Both operands are canonical eight-byte little-endian encodings, so
    // equality as bytes is equality as numbers.
    instructions.push(op(OpcodeId::EqualVerify));

    TapscriptProgram::new(instructions)
}

/// Read one explicit amount and leave it as an arithmetic operand.
///
/// The form is established before the payload is touched, the payload is
/// narrowed to the operand width the arithmetic primitives declare, and
/// the semantic domain is checked in both directions with each Boolean
/// consumed immediately.
fn explicit_amount(
    target: &ReviewedElementsTapscriptDefinition,
    inspect: OpcodeId,
    position: i64,
) -> Result<Vec<TapscriptInstruction>, TapscriptError> {
    let mut instructions = vec![number(target, position)?, op(inspect)];
    instructions.extend(require_explicit(target, EncodingClass::ExplicitValue)?);
    instructions.extend(narrow_to_operand(target)?);
    instructions.extend(require_amount_domain(target));
    Ok(instructions)
}

/// Fold `addends` operands on top of the stack into one total.
///
/// `addends - 1` additions, each with its success flag verified where it
/// is produced. A single addend needs no addition at all and is already
/// its own total, which is the one-to-one transfer and the one-destination
/// merge.
fn fold(addends: u8) -> Vec<TapscriptInstruction> {
    (1..addends)
        .flat_map(|_| [op(OpcodeId::Add64), op(OpcodeId::Verify)])
        .collect()
}

// --- §10.7: sponsor isolation -----------------------------------------

/// Isolate the sponsor region (§10.7, §1.9, §5.8).
///
/// Every sponsor input carries the reserve asset, the sponsor-change role
/// carries the reserve asset under the linked change program at its
/// linked version, and the fee role carries the reserve asset under the
/// digest the reviewed target substitutes for a program that is not a
/// witness program. Together with the exact counts, that is the exact
/// suffix, the change role's presence or absence, the fee role, the
/// disjointness of the two regions, and the absence of a second envelope.
///
/// # No sponsor amount is an operand
///
/// There is no value introspection in this fragment. Not one that is
/// discarded, and not one whose result is compared with zero: §1.9 keeps
/// individual sponsor amounts out of every protocol claim, and the
/// strongest form of that is for the emitted bytes to contain no
/// primitive that could read one. The property is checked on the
/// instructions rather than argued.
///
/// # Why a sponsorless shape emits nothing
///
/// Its sponsor region is empty, so there is no position to test. The
/// region's absence is established by the exact input and output counts,
/// which leave no position for a sponsor role to occupy — a test that
/// could be omitted would be weaker than a count that cannot.
///
/// # Errors
///
/// [`TapscriptError::ScriptNumberOutOfRange`] or
/// [`TapscriptError::OversizedStackItem`] when a literal this fragment
/// pushes is outside what the reviewed contract admits,
/// [`TapscriptError::MalformedEncodedItem`] when an encoding class states
/// no prefix to compare against, and
/// [`TapscriptError::InstructionLimitExceeded`] when the fragment exceeds
/// the instruction bound.
pub fn live_sponsor_isolation_fragment(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &LiveTransferSymbols,
    shape: LiveTransferShape,
) -> Result<TapscriptProgram, TapscriptError> {
    let (sponsor_first, sponsor_end) = shape.sponsor_range();
    let mut instructions = Vec::new();

    for position in sponsor_first..sponsor_end {
        instructions.extend(require_asset(
            target,
            OpcodeId::InspectInputAsset,
            i64::from(position),
            symbols.reserve_asset(),
        )?);
    }

    let mut position = i64::from(shape.receipt_outputs());
    if shape.sponsor_change() == SponsorChangePresence::Present {
        instructions.extend(require_asset(
            target,
            OpcodeId::InspectOutputAsset,
            position,
            symbols.reserve_asset(),
        )?);
        instructions.extend(require_program(
            target,
            OpcodeId::InspectOutputScriptPubKey,
            position,
            symbols.sponsor_change_version(),
            symbols.sponsor_change_program(),
        )?);
        position += 1;
    }

    if shape.sponsored() {
        instructions.extend(require_asset(
            target,
            OpcodeId::InspectOutputAsset,
            position,
            symbols.reserve_asset(),
        )?);
        // The fee role by form, never by amount: the reviewed target
        // replaces a program that is not a witness program by a digest of
        // it under a negative version marker, and that pair is the whole
        // discriminator.
        instructions.extend([
            number(target, position)?,
            op(OpcodeId::InspectOutputScriptPubKey),
            number(target, FEE_ROLE_VERSION_MARKER)?,
            op(OpcodeId::EqualVerify),
            TapscriptInstruction::Push(symbols.fee_program_digest().clone()),
            op(OpcodeId::EqualVerify),
        ]);
    }

    TapscriptProgram::new(instructions)
}

/// The version marker the reviewed target reads a non-witness program at.
///
/// Negative, and therefore not a witness version any output could carry
/// by accident. Named once so the fee role's test and any reader of it
/// agree on the figure.
const FEE_ROLE_VERSION_MARKER: i64 = -1;

// --- §10.8: absence relations -----------------------------------------

/// Refuse an issuance on any input (§10.8).
///
/// The reviewed introspection has two forms: an input carrying no
/// issuance pushes one empty marker, and an input carrying one pushes six
/// items with a thirty-two byte blinding nonce on top. Comparing the top
/// item with the empty marker therefore separates them, on the target and
/// in the abstract walk alike — the walk decides it from the widths,
/// which is the narrowing [`crate::stack`] documents.
///
/// Over every input, not only the receipts: an issuance carried by a
/// sponsor input is as much an issuance in this transaction as one
/// carried by a receipt, and §10.8 forbids the relation rather than a
/// region of it.
///
/// # What this fragment is not asked to establish
///
/// The object families §10.8 forbids — the roots, the reserve, the
/// projected events, and the burn record — occupy positions rather than
/// fields, and their absence is [`live_family_ranges`]'s: every position
/// of the transaction is held by exactly one family the coordinator
/// authenticates, and the exact counts leave no position over. Emitting a
/// test for each forbidden family would be enumerating what a complete
/// census already excludes, and it would go out of date the moment the
/// architecture named one more.
///
/// # Errors
///
/// [`TapscriptError::ScriptNumberOutOfRange`] or
/// [`TapscriptError::OversizedStackItem`] when a literal this fragment
/// pushes is outside what the reviewed contract admits, and
/// [`TapscriptError::InstructionLimitExceeded`] when the fragment exceeds
/// the instruction bound.
pub fn issuance_absence_fragment(
    target: &ReviewedElementsTapscriptDefinition,
    shape: LiveTransferShape,
) -> Result<TapscriptProgram, TapscriptError> {
    let mut instructions = Vec::new();

    for position in 0..shape.inputs() {
        instructions.extend([
            number(target, i64::from(position))?,
            op(OpcodeId::InspectInputIssuance),
            // The absent form's own marker, which the reviewed contract
            // gives no prefix and no payload. Written as the empty item
            // rather than read from the encoding registry because the
            // registry states no prefix byte to read.
            TapscriptInstruction::Push(StackItem::empty()),
            op(OpcodeId::EqualVerify),
        ]);
    }

    TapscriptProgram::new(instructions)
}

// --- Shape predicates -------------------------------------------------

/// Whether one shape has a sponsor region at all.
///
/// The sponsor-isolation fragment of a shape without one is empty, and
/// [`crate::live_pattern::patterns_for`] uses this to omit the pattern
/// record rather than mint an identity over zero instructions — the same
/// answer [`crate::live_pattern::has_member_position`] gives for the
/// member leaf of a one-to-one transfer.
#[must_use]
pub const fn has_sponsor_region(shape: LiveTransferShape) -> bool {
    shape.sponsored() || matches!(shape.sponsor_change(), SponsorChangePresence::Present)
}

/// Whether a program introspects a value field at all.
///
/// The stronger of the two amount claims, and the one §1.9 asks of the
/// sponsor region: not "the amount is read and discarded" but "the field
/// is never looked at". [`live_sponsor_isolation_fragment`] satisfies it,
/// and a fragment that did not could not be said to keep a sponsor value
/// out of every protocol claim.
#[must_use]
pub fn reads_a_value_field(program: &TapscriptProgram) -> bool {
    program.instructions().iter().any(|instruction| {
        matches!(
            instruction,
            TapscriptInstruction::Opcode(
                OpcodeId::InspectInputValue | OpcodeId::InspectOutputValue
            )
        )
    })
}

/// Whether a program opens an amount out of a value field.
///
/// The weaker claim, and the one §10.6 and §10.1 ask: a program may read
/// a value field to establish which *form* it is in — that is the whole
/// of [`crate::live_pattern::local_recognition_fragment`], which then
/// drops the payload — without ever reading the number inside it.
/// Opening the payload takes the slice this crate narrows an operand
/// with, or one of the arithmetic primitives, and nothing else in the
/// live-transfer fragments takes either.
///
/// Deliberately not the ordering or conversion primitives: a member leaf
/// bounds its own *index* with them, and an index is a position rather
/// than an amount. A predicate that counted those would report every
/// member program as reading a value and would say nothing.
#[must_use]
pub fn opens_an_amount(program: &TapscriptProgram) -> bool {
    program.instructions().iter().any(|instruction| {
        matches!(
            instruction,
            TapscriptInstruction::Opcode(
                OpcodeId::Substring
                    | OpcodeId::Add64
                    | OpcodeId::Sub64
                    | OpcodeId::Mul64
                    | OpcodeId::Div64
                    | OpcodeId::Neg64
            )
        )
    })
}
