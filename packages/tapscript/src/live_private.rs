//! The private-committed live-transfer plan (Guide-13 §6.3, §6.4, §6.6).
//!
//! # What conservation means when nobody may read an amount
//!
//! [`crate::live_plan::explicit_conservation_fragment`] settles §6.2 by
//! reading both sides and requiring the totals equal. §6.3 admits no such
//! fragment and §10.6 forbids minting a pattern that pretends to one, so
//! the private plan establishes the *closure* around the value equation
//! and leaves the equation itself to the target: every `U` input is a live
//! receipt source, every `U` output is a live receipt destination, no
//! issuance and no destruction exist, no unclassified `U` output exists,
//! and target CT conservation holds. [`private_soundness_establishments`]
//! is those six lines as data, each mapped to the fragments or the
//! external requirement that carries it, and
//! [`PrivateConditionCarrier::ExternalTargetEvidence`] is what the last
//! line gets — never a fragment, and never a pattern identity.
//!
//! # The one check the explicit plan got for free
//!
//! The explicit coordinator settles its destinations' value *form* inside
//! the conservation fragment, because reading an amount begins by
//! establishing that the field is an amount at all. Drop the conservation
//! and that check goes with it, so the private plan owes its own:
//! [`private_destination_form_fragment`] requires every destination's
//! value field to carry the confidential form, and drops the payload
//! where it stands. It reads the field and opens nothing, which is the
//! same distinction [`crate::live_plan::opens_an_amount`] draws for the
//! per-input recognition.
//!
//! What the check settles is settled on the target, not in the walk:
//! the comparison is against the reviewed prefix and the walk holds no
//! bytes for an introspected one, which is
//! [`crate::live_pattern::RecognitionResidual::FieldFormSettledOnlyOnTheTarget`]
//! and is carried here rather than quietly improved on.
//!
//! # A confidential value has two prefixes, and one of them is not enough
//!
//! The reviewed registry gives [`EncodingClass::ConfidentialValue`] the
//! prefixes `0x08` and `0x09`, which record whether the commitment's `y`
//! is a square. Both are ordinary well-formed confidential values, so a
//! comparison against one of them refuses about half of all valid private
//! receipts — and refuses them for a reason no owner can control or even
//! observe before the fact.
//!
//! [`prefix_mask`] is what this module compares with instead: the bits
//! every declared prefix of a class agrees on, and a mask covering the
//! rest. `0x08` and `0x09` differ in one bit, so the test is
//! `prefix & 0xfe == 0x08`, which admits exactly those two bytes and no
//! third. A class with a single prefix yields a full mask, and the
//! emitted bytes are then the equality they always were.
//!
//! The derivation is exact or it refuses: a prefix set that is not
//! precisely the bytes one mask admits gets
//! [`TapscriptError::PrefixSetNotDiscriminable`] rather than a comparison
//! that would accept a byte the class never declared. Widening a form
//! test to keep a fragment buildable is the failure this refusal exists
//! to make impossible.
//!
//! # Opacity as a whitelist rather than a blacklist
//!
//! [`crate::live_plan::opens_an_amount`] names the primitives that open a
//! payload, and a census of forbidden primitives is only ever as complete
//! as the list. [`value_field_uses`] states the positive form instead:
//! every value introspection in a program is followed by a form check and
//! a `Drop`, or it is not, and a program in which every value field is
//! [`ValueFieldUse::FormCheckedThenDropped`] has no payload left for any
//! primitive to take. That is what [`opens_no_value_payload`] reports, and
//! it is the property §6.4's first two prohibitions actually need.

use std::collections::{BTreeMap, BTreeSet};

use compiler::target::ExternalEvidenceRole;
use target_elements::{EncodingClass, OpcodeId, ReviewedElementsTapscriptDefinition};

use crate::capability::census_enum;
use crate::error::TapscriptError;
use crate::instruction::{StackItem, TapscriptInstruction};
use crate::live_pattern::{LiveFragmentId, RecognitionResidual};
use crate::live_shape::LiveTransferShape;
use crate::pattern::{number, op};
use crate::program::TapscriptProgram;

// --- Prefix discrimination --------------------------------------------

/// The bits a class's declared prefixes agree on, and the mask that
/// isolates them.
///
/// Derived from the reviewed registry by [`prefix_mask`] and never
/// written down: a mask stated as a literal would be a second reading of
/// the contract, and the two would part company the moment a class
/// gained a form.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PrefixMask {
    mask: u8,
    value: u8,
}

impl PrefixMask {
    /// The bits every declared prefix of the class fixes.
    #[must_use]
    pub const fn mask(self) -> u8 {
        self.mask
    }

    /// The values those bits take.
    #[must_use]
    pub const fn value(self) -> u8 {
        self.value
    }

    /// Whether one byte is a prefix this mask admits.
    #[must_use]
    pub const fn admits(self, byte: u8) -> bool {
        byte & self.mask == self.value
    }

    /// Whether the mask fixes every bit, so the test is an equality.
    #[must_use]
    pub const fn is_exact_byte(self) -> bool {
        self.mask == u8::MAX
    }

    /// How many bytes the mask admits.
    #[must_use]
    pub const fn admitted_count(self) -> usize {
        1_usize << (!self.mask).count_ones()
    }
}

/// The mask that admits exactly one encoding class's declared prefixes.
///
/// The constant bits are the intersection of the declared prefixes and
/// the varying bits are masked out, so the test `byte & mask == value` is
/// satisfied by every declared prefix by construction. What it may not be
/// satisfied by is anything else, and that is checked rather than
/// assumed: the mask admits `2^k` bytes for `k` varying bits, so the
/// derivation is exact exactly when the class declares that many
/// prefixes.
///
/// # Errors
///
/// [`TapscriptError::MalformedEncodedItem`] when the class states no
/// prefix to compare against, and
/// [`TapscriptError::PrefixSetNotDiscriminable`] when the declared
/// prefixes are not precisely the bytes one mask admits — for which a
/// masked comparison would accept a byte the class never declared, and no
/// comparison this crate emits may do that.
pub fn prefix_mask(
    target: &ReviewedElementsTapscriptDefinition,
    class: EncodingClass,
) -> Result<PrefixMask, TapscriptError> {
    let prefixes = target
        .definition()
        .encodings()
        .get(&class)
        .map(target_elements::EncodingSpec::prefixes)
        .filter(|prefixes| !prefixes.is_empty())
        .ok_or(TapscriptError::MalformedEncodedItem { class })?;

    discriminating_mask(prefixes).ok_or(TapscriptError::PrefixSetNotDiscriminable { class })
}

/// The mask that admits exactly `prefixes`, where one exists.
///
/// The arithmetic of [`prefix_mask`], separated from the registry lookup
/// so the refusal is a value this crate can be shown refusing. Every
/// prefix-discriminated class the reviewed contract states happens to
/// discriminate, so a test that could only reach the refusal through the
/// registry could not reach it at all — and an unreachable refusal is one
/// nobody has watched work.
///
/// [`None`] for an empty set, and for a set that is not precisely the
/// bytes one mask admits: `{0x01, 0x02, 0x03}` varies in two bits, so a
/// mask covering them would admit `0x00` as well, and no comparison this
/// crate emits may accept a byte the class never declared.
#[must_use]
pub fn discriminating_mask(prefixes: &BTreeSet<u8>) -> Option<PrefixMask> {
    if prefixes.is_empty() {
        return None;
    }

    let fixed_ones = prefixes.iter().fold(u8::MAX, |bits, prefix| bits & prefix);
    let any_ones = prefixes.iter().fold(0, |bits, prefix| bits | prefix);
    let candidate = PrefixMask {
        mask: !(any_ones & !fixed_ones),
        value: fixed_ones,
    };

    (candidate.admitted_count() == prefixes.len()).then_some(candidate)
}

/// Require the field just introspected to be in one class's form.
///
/// Consumes the prefix the introspection left on top and leaves the
/// payload, which is [`crate::pattern::require_explicit`]'s contract and
/// deliberately the same one: a caller that swapped the two would find
/// the stack where it expected it. What differs is that every declared
/// prefix of the class is admitted rather than the least of them.
///
/// # Errors
///
/// Any failure of [`prefix_mask`], and
/// [`TapscriptError::OversizedStackItem`] when the reviewed literal bound
/// admits nothing at all.
pub(crate) fn require_value_form(
    target: &ReviewedElementsTapscriptDefinition,
    class: EncodingClass,
) -> Result<Vec<TapscriptInstruction>, TapscriptError> {
    let discriminator = prefix_mask(target, class)?;
    let mut instructions = Vec::new();

    if !discriminator.is_exact_byte() {
        instructions.push(TapscriptInstruction::Push(StackItem::new(
            target,
            vec![discriminator.mask()],
        )?));
        instructions.push(op(OpcodeId::BitwiseAnd));
    }
    instructions.push(TapscriptInstruction::Push(StackItem::new(
        target,
        vec![discriminator.value()],
    )?));
    instructions.push(op(OpcodeId::EqualVerify));

    Ok(instructions)
}

// --- §6.3, §10.6: the private destination value form ------------------

/// Require every destination's value to carry the confidential form.
///
/// The output-side half of the private plan's representation closure.
/// [`crate::live_pattern::local_recognition_fragment`] establishes the
/// form of the input the leaf is executing in, and every receipt input
/// runs it; this fragment establishes the same fact for every destination
/// of the transfer, which no input's recognition reaches.
///
/// # Why the explicit plan needs no such fragment
///
/// It has one, inside its arithmetic:
/// [`crate::live_plan::explicit_conservation_fragment`] establishes each
/// destination's explicit form before narrowing the payload to an
/// operand, because a payload that is not an amount is not one the
/// arithmetic may take. Removing the arithmetic removes that check with
/// it, and this fragment is what the private coordinator carries in its
/// place — the residual §6.3 would otherwise leave open.
///
/// # What it establishes, and what it does not
///
/// The form, and nothing about the amount inside it. The payload is
/// dropped where it stands, so no primitive downstream can take it, and
/// [`value_field_uses`] reports the reads as
/// [`ValueFieldUse::FormCheckedThenDropped`] over the emitted bytes
/// rather than on this paragraph's authority.
///
/// It is not a conservation fragment and must not be read as one. Which
/// amounts those commitments hide, and whether they balance, is the
/// target's own confidential-transaction rule
/// ([`ExternalEvidenceRole::ConfidentialValueConservation`]) and is
/// established by nothing in this crate.
///
/// # Errors
///
/// [`TapscriptError::ScriptNumberOutOfRange`] or
/// [`TapscriptError::OversizedStackItem`] when a literal this fragment
/// pushes is outside what the reviewed contract admits, any failure of
/// [`prefix_mask`] for the confidential value class, and
/// [`TapscriptError::InstructionLimitExceeded`] when the fragment exceeds
/// the instruction bound.
pub fn private_destination_form_fragment(
    target: &ReviewedElementsTapscriptDefinition,
    shape: LiveTransferShape,
) -> Result<TapscriptProgram, TapscriptError> {
    let (first, end) = shape.destination_range();
    let mut instructions = Vec::new();

    for position in first..end {
        instructions.push(number(target, i64::from(position))?);
        instructions.push(op(OpcodeId::InspectOutputValue));
        instructions.extend(require_value_form(
            target,
            EncodingClass::ConfidentialValue,
        )?);
        // The form was the question. Dropping the payload here is what
        // makes the opacity a property of the bytes: there is nothing
        // left for a later fragment to take for an amount.
        instructions.push(op(OpcodeId::Drop));
    }

    TapscriptProgram::new(instructions)
}

/// Which destination position absorbs a consumed blinder sum (§6.5).
///
/// The LAST destination, and the choice is a declaration rather than a
/// search: the covenant states one position, the leaf checks that
/// position and no other, and a candidate that blinded a different one
/// is refused. Nothing here infers an absorber from a value form, which
/// would decide the position from the very field the position governs.
///
/// The last position is chosen because it is the one every exit-crossing
/// shape has. A shape's destination range starts at zero and runs to its
/// receipt-output count, so "the last destination" names a position for
/// every count of one or more, while any fixed interior index would name
/// none for the smallest shapes.
///
/// `None` for a shape with no destinations at all, which has nothing to
/// absorb into.
#[must_use]
pub const fn absorber_position(shape: LiveTransferShape) -> Option<u16> {
    let (first, end) = shape.destination_range();
    if end <= first { None } else { Some(end - 1) }
}

/// Every destination's value form under an EXIT crossing (§6.5).
///
/// The positional sibling of [`private_destination_form_fragment`], and
/// the one fragment representation crossing genuinely needs. A transfer
/// that consumes commitments and creates explicit values presents a
/// blinder sum the target's tally must see equalled, and explicit
/// outputs contribute zero to it — so exactly one created position stays
/// blinded and carries the difference. This fragment is what makes that
/// position a covenant term rather than a convention the builder happens
/// to follow.
///
/// # It is a declaration and not an inference
///
/// The absorber sits at [`absorber_position`], inside the destination
/// range, carrying the protocol asset like every other destination. It
/// is NOT a fourth output family: the shape's exact output count and the
/// three-family position census leave no position outside the
/// destinations, the sponsor change and the fee, so an absorber outside
/// the destination range is unrepresentable rather than merely unsound.
/// The §10.4 closure argument is therefore untouched — this fragment
/// adds no position and moves none, it only says which of the positions
/// the closure already speaks for carries which form.
///
/// # What it establishes, and what it does not
///
/// Forms, positionally, and nothing else. Each non-absorber destination
/// is required to carry the explicit value form and the absorber the
/// confidential one; every payload is dropped where it stands, exactly
/// as the homogeneous private fragment drops its own. In particular this
/// is NOT a conservation fragment and must not be read as one: the
/// consumed values are commitments, so no first-party equality over them
/// is available to claim, and whether the amounts balance is the
/// target's own confidential-transaction rule and is established by
/// nothing in this crate.
///
/// A reader may ask why the explicit destinations are not summed, since
/// their amounts are readable. Because the other side of that equality
/// is not: an equality with one readable side is not an equality, and a
/// fragment that summed the created side alone would authenticate a
/// total against nothing.
///
/// # Errors
///
/// As [`private_destination_form_fragment`].
pub fn crossing_destination_form_fragment(
    target: &ReviewedElementsTapscriptDefinition,
    shape: LiveTransferShape,
) -> Result<TapscriptProgram, TapscriptError> {
    let (first, end) = shape.destination_range();
    // A shape's receipt-output count is nonzero, so the destination
    // range is nonempty and the declared position exists. Compared as an
    // `Option` rather than unwrapped, so the one shape that could have
    // no absorber emits no absorber instead of panicking about it.
    let absorber = absorber_position(shape);
    let mut instructions = Vec::new();

    for position in first..end {
        // Read off the declared position rather than off the value, so
        // the leaf decides the form and the candidate does not.
        let class = if absorber == Some(position) {
            EncodingClass::ConfidentialValue
        } else {
            EncodingClass::ExplicitValue
        };
        instructions.push(number(target, i64::from(position))?);
        instructions.push(op(OpcodeId::InspectOutputValue));
        instructions.extend(require_value_form(target, class)?);
        instructions.push(op(OpcodeId::Drop));
    }

    TapscriptProgram::new(instructions)
}

// --- §6.4: amount opacity over the emitted bytes ----------------------

/// What one program does with a value field it introspects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValueFieldUse {
    /// The form is compared and the payload is dropped unread.
    FormCheckedThenDropped,
    /// The payload survives the read, so some primitive receives it.
    ///
    /// Not an accusation: the explicit conservation opens amounts because
    /// §6.2 is exactly that it does. The verdict is a fact about a
    /// program, and which programs are entitled to it is §6.3's question
    /// rather than this census's.
    PayloadOpened,
}

/// How every value field one program reads is used, in program order.
///
/// A whitelist, and the argument for one is in what a blacklist can
/// promise. [`crate::live_plan::opens_an_amount`] names the primitives
/// that open a payload, so it is complete only while the list is, and a
/// reviewed primitive added later would pass it silently. The two
/// admitted shapes below are the whole of what a form check looks like,
/// and any other continuation — including one built from a primitive
/// nobody has reviewed yet — is [`ValueFieldUse::PayloadOpened`].
///
/// The two shapes are exactly what a form check derived from
/// [`prefix_mask`] emits, followed by the `Drop` that ends the read:
///
/// ```text
/// push <prefix>   equal-verify   drop
/// push <mask>     bitwise-and    push <value>   equal-verify   drop
/// ```
///
/// Both comparands are single bytes, and that is checked rather than
/// assumed: a wide literal in a comparison is not a prefix test, whatever
/// else it may be.
#[must_use]
pub fn value_field_uses(program: &TapscriptProgram) -> Vec<ValueFieldUse> {
    let instructions = program.instructions();

    instructions
        .iter()
        .enumerate()
        .filter(|(_, instruction)| {
            matches!(
                instruction,
                TapscriptInstruction::Opcode(
                    OpcodeId::InspectInputValue | OpcodeId::InspectOutputValue
                )
            )
        })
        .map(|(index, _)| {
            let tail = &instructions[index + 1..];
            if is_form_check_then_drop(tail) {
                ValueFieldUse::FormCheckedThenDropped
            } else {
                ValueFieldUse::PayloadOpened
            }
        })
        .collect()
}

/// Whether a program leaves every value payload it reads unread.
///
/// True for a program that introspects no value field at all, which is
/// the honest answer: a program that never looks cannot have opened one.
/// [`crate::live_plan::reads_a_value_field`] is what separates the two
/// cases where the difference matters.
#[must_use]
pub fn opens_no_value_payload(program: &TapscriptProgram) -> bool {
    value_field_uses(program)
        .iter()
        .all(|use_| *use_ == ValueFieldUse::FormCheckedThenDropped)
}

/// Whether the instructions after a value read are a form check and a
/// drop.
fn is_form_check_then_drop(tail: &[TapscriptInstruction]) -> bool {
    let equality = |rest: &[TapscriptInstruction]| {
        matches!(
            rest,
            [
                TapscriptInstruction::Push(comparand),
                TapscriptInstruction::Opcode(OpcodeId::EqualVerify),
                TapscriptInstruction::Opcode(OpcodeId::Drop),
                ..,
            ] if comparand.bytes().len() == 1
        )
    };

    match tail {
        [
            TapscriptInstruction::Push(mask),
            TapscriptInstruction::Opcode(OpcodeId::BitwiseAnd),
            rest @ ..,
        ] if mask.bytes().len() == 1 => equality(rest),
        _ => equality(tail),
    }
}

census_enum! {
    /// One thing §6.4 forbids the private plan.
    ///
    /// The guide's own seven lines, in its own order. A prohibition this
    /// crate can check over what it emits is checked;
    /// [`ProhibitionDisposition`] is where the others say who owns them,
    /// because a prohibition nobody names is a prohibition that has been
    /// dropped rather than met.
    pub enum PrivateAmountProhibition {
        /// Open a receipt amount in script.
        AmountOpenedInScript,
        /// Compare a receipt amount with zero as a protocol predicate.
        ZeroComparisonProtocolPredicate,
        /// Publish an exact receipt amount in ordinary diagnostics.
        ExactAmountInOrdinaryDiagnostics,
        /// Publish fixture openings in canonical reports.
        FixtureOpeningsInCanonicalReports,
        /// Order outputs by private amount or blinding material.
        OrderingByPrivateAmountOrBlinding,
        /// Require a public subtotal.
        PublicSubtotal,
        /// Add a public equality witness duplicating target CT
        /// conservation.
        DuplicatedCtEqualityWitness,
    }
}

/// How one §6.4 prohibition is met.
///
/// Three dispositions and not a Boolean, because "the emitted bytes make
/// it impossible", "no type this crate publishes can express it", and
/// "another package owns the surface it would appear on" are three
/// different strengths, and a reader deciding what this wave established
/// needs to know which one is in hand.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProhibitionDisposition {
    /// A property of the emitted private programs, executed over them.
    CheckedOverEmittedPrograms,
    /// A property of the types this crate publishes, executed over them.
    CheckedOverThisCrateSurface,
    /// Named, and owed by a surface this crate does not emit.
    ///
    /// Not a weaker prohibition. It is the same prohibition, recorded
    /// where it can be discharged rather than asserted where it cannot.
    NamedNonClaim,
}

/// How the private plan meets each of §6.4's seven prohibitions.
///
/// Total over [`PrivateAmountProhibition::ALL`] by construction, so a
/// line added to §6.4 arrives here without a disposition rather than
/// silently absent.
///
/// # Why the first two share one mechanism
///
/// A receipt amount reaches a primitive only by surviving the read that
/// fetched it, and [`opens_no_value_payload`] is that no payload does.
/// A comparison with zero is then unreachable for the same reason a
/// subtotal is: there is no operand for either to be built from, and
/// neither prohibition needs a test of its own once the payload never
/// leaves the field.
#[must_use]
#[expect(
    clippy::match_same_arms,
    reason = "one arm per §6.4 prohibition, so a line added to the guide's list has to be decided \
              here; merging arms that happen to share a disposition would delete which mechanism \
              meets which prohibition"
)]
pub fn prohibition_dispositions() -> BTreeMap<PrivateAmountProhibition, ProhibitionDisposition> {
    use PrivateAmountProhibition as Forbidden;
    use ProhibitionDisposition as How;

    PrivateAmountProhibition::ALL
        .iter()
        .map(|prohibition| {
            let disposition = match prohibition {
                // Every value field a private program reads is form
                // checked and dropped, so no payload reaches any
                // primitive — an opening, a zero comparison, and a
                // subtotal alike.
                Forbidden::AmountOpenedInScript
                | Forbidden::ZeroComparisonProtocolPredicate
                | Forbidden::PublicSubtotal => How::CheckedOverEmittedPrograms,
                // The private programs push no amount-domain literal and
                // schedule no arithmetic, so there is no exact amount in
                // the bytes; and no type this crate publishes holds one.
                Forbidden::ExactAmountInOrdinaryDiagnostics => How::CheckedOverThisCrateSurface,
                // A canonical report is the conformance and vector
                // packages' artifact. This crate emits none, so the
                // prohibition is named here and discharged there.
                Forbidden::FixtureOpeningsInCanonicalReports => How::NamedNonClaim,
                // The loops are indexed by position and the fragments are
                // functions of the shape's counts alone, which is checked
                // over two shapes that share their counts.
                Forbidden::OrderingByPrivateAmountOrBlinding => How::CheckedOverEmittedPrograms,
                // No fragment of the private coordinator compares two
                // value payloads, and no pattern identity claims the
                // equation: §10.6 admits no private-conservation pattern
                // and the census mints none.
                Forbidden::DuplicatedCtEqualityWitness => How::CheckedOverEmittedPrograms,
            };
            (*prohibition, disposition)
        })
        .collect()
}

// --- §6.3: what makes the private relation sound ----------------------

census_enum! {
    /// One line of §6.3's soundness condition.
    ///
    /// The guide's own six, in its own order. The last one is not like
    /// the others and the census is built so that it cannot be made to
    /// look like them:
    /// [`private_soundness_establishments`] gives it an external carrier
    /// and no fragment, and that is a property a test executes.
    pub enum PrivateSoundnessCondition {
        /// Every `U` input is one live receipt source.
        EveryInputIsALiveSource,
        /// Every `U` output is one live receipt destination.
        EveryOutputIsALiveDestination,
        /// No `U` issuance exists.
        NoIssuance,
        /// No `U` destruction exists.
        NoDestruction,
        /// No unclassified `U` output exists.
        NoUnclassifiedOutput,
        /// Target CT conservation holds.
        TargetConfidentialConservation,
    }
}

/// What carries one of §6.3's conditions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PrivateConditionCarrier {
    /// Instructions the private programs emit.
    EmittedFragments,
    /// Every position is held by exactly one authenticated family, and
    /// the exact counts leave none over
    /// ([`crate::live_plan::live_family_ranges`]).
    PositionCensus,
    /// The target's own consensus rules, and nothing this crate emits.
    ///
    /// §10.6 admits no backend pattern for target consensus behaviour and
    /// §6.3 admits no local program that claims one because the target
    /// eventually accepts. A condition carried this way is a condition
    /// this wave states an obligation for and does not discharge.
    ExternalTargetEvidence,
}

/// How the private plan establishes one of §6.3's conditions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrivateSoundnessEstablishment {
    condition: PrivateSoundnessCondition,
    carrier: PrivateConditionCarrier,
    fragments: BTreeSet<LiveFragmentId>,
    external: BTreeSet<ExternalEvidenceRole>,
    residuals: BTreeSet<RecognitionResidual>,
}

impl PrivateSoundnessEstablishment {
    /// The §6.3 condition.
    #[must_use]
    pub const fn condition(&self) -> PrivateSoundnessCondition {
        self.condition
    }

    /// What establishes it.
    #[must_use]
    pub const fn carrier(&self) -> PrivateConditionCarrier {
        self.carrier
    }

    /// The emitted fragments that carry it, in census order.
    pub fn fragments(&self) -> impl Iterator<Item = LiveFragmentId> + '_ {
        self.fragments.iter().copied()
    }

    /// The external target requirements that carry it, in census order.
    pub fn external_evidence(&self) -> impl Iterator<Item = ExternalEvidenceRole> + '_ {
        self.external.iter().copied()
    }

    /// What the claim still owes, in census order.
    pub fn residuals(&self) -> impl Iterator<Item = RecognitionResidual> + '_ {
        self.residuals.iter().copied()
    }
}

/// How this candidate establishes each of §6.3's six conditions.
///
/// Total over [`PrivateSoundnessCondition::ALL`] by construction: the
/// census is built by mapping that constant, so a line added to §6.3
/// arrives here without a carrier rather than silently absent.
///
/// # The one line no fragment may carry
///
/// [`PrivateSoundnessCondition::TargetConfidentialConservation`] names no
/// fragment at all, and the emptiness is the claim. §10.6 mints no
/// private-conservation pattern for target consensus behaviour, and §6.3
/// refuses a local program that claims CT conservation because consensus
/// eventually accepts — so a census that listed a fragment beside this
/// condition would be recording exactly the artifact both rules exist to
/// refuse. The census is checkable in both directions: this condition
/// names only external evidence, and every other condition names at
/// least one fragment or the position census.
///
/// # Why destruction has both
///
/// Nothing is destroyed when the value that entered left, and left
/// through the family. The second half is
/// [`LiveFragmentId::DestinationClosure`]'s and the position census's;
/// the first half is the value equation, which under this plan is the
/// target's confidential-transaction rule and nobody else's. Recording
/// only the fragments would claim the whole of §5.6 from the half of it
/// the bytes reach.
#[must_use]
pub fn private_soundness_establishments()
-> BTreeMap<PrivateSoundnessCondition, PrivateSoundnessEstablishment> {
    use ExternalEvidenceRole as External;
    use LiveFragmentId as Fragment;
    use PrivateConditionCarrier as Carrier;
    use PrivateSoundnessCondition as Condition;
    use RecognitionResidual as Residual;

    PrivateSoundnessCondition::ALL
        .iter()
        .map(|condition| {
            let (carrier, fragments, external, residuals): (
                _,
                &[Fragment],
                &[External],
                &[Residual],
            ) = match condition {
                // The asset is compared with the linked symbol, the value
                // field's form with the selected plan's, and the owner's
                // signature is verified — by every receipt input, at a
                // position the role fragments pin.
                Condition::EveryInputIsALiveSource => (
                    Carrier::EmittedFragments,
                    &[
                        Fragment::LocalRecognition,
                        Fragment::OwnerAuthorization,
                        Fragment::CoordinatorRole,
                        Fragment::MemberRole,
                        Fragment::Cardinality,
                    ],
                    &[],
                    // The profile residual stood here because this
                    // condition asserts the owner's signature, and a
                    // verified signature was not yet a signature over
                    // §1.7's protected data while the profile joining the
                    // two was unreviewed. The review verdict and the
                    // re-typing supplied that join, so it
                    // is gone from this list and from every other.
                    &[
                        Residual::FieldFormSettledOnlyOnTheTarget,
                        Residual::LinkedDestinationConstructorIdentity,
                    ],
                ),
                // Every destination carries the linked protocol asset
                // under a program at the constructor's version, and its
                // value is in the confidential form the plan selected.
                Condition::EveryOutputIsALiveDestination => (
                    Carrier::EmittedFragments,
                    &[
                        Fragment::DestinationClosure,
                        Fragment::PrivateDestinationForm,
                        Fragment::Cardinality,
                    ],
                    &[],
                    &[
                        Residual::FieldFormSettledOnlyOnTheTarget,
                        Residual::LinkedDestinationConstructorIdentity,
                    ],
                ),
                Condition::NoIssuance => (
                    Carrier::EmittedFragments,
                    &[Fragment::IssuanceAbsence],
                    &[],
                    &[],
                ),
                // See the type documentation: the family half is emitted,
                // the value half is the target's.
                Condition::NoDestruction => (
                    Carrier::ExternalTargetEvidence,
                    &[
                        Fragment::DestinationClosure,
                        Fragment::SponsorIsolation,
                        Fragment::Cardinality,
                    ],
                    &[External::ConfidentialValueConservation],
                    &[Residual::LinkedDestinationConstructorIdentity],
                ),
                // A `U` output nobody classified would need a position,
                // and every position is held by exactly one family.
                Condition::NoUnclassifiedOutput => (
                    Carrier::PositionCensus,
                    &[
                        Fragment::Cardinality,
                        Fragment::DestinationClosure,
                        Fragment::SponsorIsolation,
                    ],
                    &[],
                    &[Residual::LinkedDestinationConstructorIdentity],
                ),
                Condition::TargetConfidentialConservation => (
                    Carrier::ExternalTargetEvidence,
                    &[],
                    &[External::ConfidentialValueConservation],
                    &[],
                ),
            };

            (
                *condition,
                PrivateSoundnessEstablishment {
                    condition: *condition,
                    carrier,
                    fragments: fragments.iter().copied().collect(),
                    external: external.iter().copied().collect(),
                    residuals: residuals.iter().copied().collect(),
                },
            )
        })
        .collect()
}

// --- §6.6: the comparable projections ---------------------------------

census_enum! {
    /// One axis paired explicit and private fixtures compare (§6.6).
    ///
    /// The guide's own eleven, in its own order. The comparison is
    /// Guide-13's disclosure-minimality evidence and is not this wave's;
    /// what is this wave's is that the private side can state each axis
    /// at all, and state it from a source that does not open an amount.
    ///
    /// Target bytes, commitments, proofs, witness sizes, and resource use
    /// are absent by design rather than by omission: §6.6 admits that
    /// they differ, so an axis for one of them would be an equality
    /// nobody claimed.
    pub enum RepresentationComparisonAxis {
        /// The input semantic amount multiset.
        InputSemanticAmountMultiset,
        /// The destination owner and value multiset.
        DestinationOwnerValueMultiset,
        /// The distinct input-owner set.
        DistinctInputOwnerSet,
        /// The live class.
        LiveClass,
        /// The exact explicit `U`.
        ExactExplicitAsset,
        /// The authorization result.
        AuthorizationResult,
        /// The absence of roots.
        AbsenceOfRoots,
        /// The absence of issuance and destruction.
        AbsenceOfIssuanceAndDestruction,
        /// The lateral flow.
        LateralFlow,
        /// The sponsor relation and shape.
        SponsorRelationAndShape,
        /// The transition-certificate projection.
        TransitionCertificateProjection,
    }
}

/// Where one comparison axis is read from.
///
/// The distinction §6.6 rests on. Two axes carry semantic amounts, and
/// the only admissible source for those is the fixture that fixed them:
/// reading them off a private transaction would mean opening the very
/// commitments the plan exists not to open, and a minimality argument
/// that opened them would have disproved itself in the making.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ComparisonSource {
    /// The semantic fixture the transfer was built from.
    SemanticFixture,
    /// The validated operation plan's own projections.
    OperationPlan,
    /// The transfer shape's declared counts and roles.
    TransferShape,
    /// The programs this crate emits.
    EmittedPrograms,
}

/// One axis and the source a representation states it from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComparableProjection {
    axis: RepresentationComparisonAxis,
    source: ComparisonSource,
}

impl ComparableProjection {
    /// The §6.6 axis.
    #[must_use]
    pub const fn axis(self) -> RepresentationComparisonAxis {
        self.axis
    }

    /// Where both representations read it from.
    #[must_use]
    pub const fn source(self) -> ComparisonSource {
        self.source
    }
}

impl RepresentationComparisonAxis {
    /// Whether the axis carries semantic amounts.
    ///
    /// The two that do are the two §6.4 would forbid reading from a
    /// private transaction, which is why
    /// [`representation_comparison_axes`] gives both
    /// [`ComparisonSource::SemanticFixture`] and why that pairing is
    /// checked rather than described.
    #[must_use]
    pub const fn carries_semantic_amounts(self) -> bool {
        matches!(
            self,
            Self::InputSemanticAmountMultiset | Self::DestinationOwnerValueMultiset
        )
    }
}

/// Which source each of §6.6's eleven axes is stated from.
///
/// Total over [`RepresentationComparisonAxis::ALL`] by construction, and
/// identical for both representations — which is the point. An axis a
/// private fixture could only state by opening a commitment would be an
/// axis the comparison cannot use, and the census is where that would be
/// visible rather than discovered in Wave 11.
#[must_use]
pub fn representation_comparison_axes()
-> BTreeMap<RepresentationComparisonAxis, ComparableProjection> {
    use ComparisonSource as From;
    use RepresentationComparisonAxis as Axis;

    Axis::ALL
        .iter()
        .map(|axis| {
            let source = match axis {
                // The amounts are the fixture's own, on both sides. The
                // explicit transfer could have had them read off the
                // transaction; taking them from the fixture instead is
                // what makes one comparison procedure serve both.
                Axis::InputSemanticAmountMultiset
                | Axis::DestinationOwnerValueMultiset
                | Axis::DistinctInputOwnerSet
                | Axis::AuthorizationResult => From::SemanticFixture,
                // The plan's projections are representation-independent
                // by construction: one operation, one class closure, one
                // asset, one root policy, one flow, one certificate.
                Axis::LiveClass
                | Axis::AbsenceOfRoots
                | Axis::AbsenceOfIssuanceAndDestruction
                | Axis::LateralFlow
                | Axis::TransitionCertificateProjection => From::OperationPlan,
                // Emitted, because the exact asset is a literal both
                // coordinators push and compare, and the comparison is
                // over the same symbol in both.
                Axis::ExactExplicitAsset => From::EmittedPrograms,
                Axis::SponsorRelationAndShape => From::TransferShape,
            };
            (
                *axis,
                ComparableProjection {
                    axis: *axis,
                    source,
                },
            )
        })
        .collect()
}

// --- §1.5, §1.12, §9.3: what the private plan does not claim ----------

census_enum! {
    /// One thing this wave's private plan does not claim.
    ///
    /// Stated rather than left to a reader's caution. Each member is a
    /// conclusion somebody could otherwise draw from a private plan that
    /// assembles, walks, and holds to §10.9 — and none of them follows
    /// from any of that.
    ///
    /// Every member is stated for both representations. §1.5's prohibited
    /// implications run in both directions, and a non-claim census that
    /// named only the private plan would read as though the explicit one
    /// had been cleared of the same conclusions.
    pub enum PrivatePlanNonClaim {
        /// That any target would accept a transaction built this way.
        ///
        /// §1.11 and §9.3: an emitted program is a construction, and no
        /// abstract walk, opcode census, or commitment oracle is
        /// evidence of target acceptance. Only a real complete
        /// transaction on a real target is.
        TargetAcceptance,
        /// That the confidential proofs such a transfer needs are valid,
        /// or that this crate produces any.
        ///
        /// §9.3: CT conservation is external target evidence. The
        /// rangeproofs and value commitments a private transfer carries
        /// are built by nothing here.
        ConfidentialProofValidity,
        /// That the private plan is production ready.
        ///
        /// §1.12: every artifact this wave reaches is a candidate, the
        /// lifecycle is structurally incomplete while burn and redemption
        /// are absent, and no digest is minted.
        ProductionReadiness,
        /// That any interface here accepts production secret material.
        ///
        /// §1.10: no owner scalar, blinder, opening, nonce, or seed
        /// reaches a first-party interface, and Guide 13 introduces none.
        /// A production signer would be a separate reviewed design.
        ProductionSecretInterface,
        /// That the private plan is disclosure-minimal.
        ///
        /// §1.5's two axes: this wave is safety work, and a private
        /// transfer that is accepted does not thereby disclose less than
        /// an explicit one. The paired-fixture comparison
        /// ([`representation_comparison_axes`]) is what could establish
        /// it, and running that comparison is not this wave's.
        DisclosureMinimality,
        /// That the two representations may share one coordinator
        /// program.
        ///
        /// §11.3: the leaf sets are disjoint and stay disjoint until one
        /// complete typed proof admits a shared program. Both plans
        /// emitting is not that proof.
        SharedCoordinatorProgram,
    }
}
