//! Typed compact-ASH tapscript proof patterns (Guide-12 §12).
//!
//! # What a pattern is here
//!
//! A pattern is the ten things §8.4 requires before an identity may be
//! minted for it: a semantic owner, target prerequisites, a typed
//! instruction fragment, a stack contract, failure behaviour, ABI
//! assumptions, source requirements, a resource formula, positive and
//! negative vectors, and operation evidence. [`BackendPattern`] carries
//! all of them, and the census in [`operation_patterns`] is what
//! [`crate::capability::BackendPatternId`] draws its variants from —
//! the identity exists because the record does, not the other way round.
//!
//! # What the vectors here establish, and what they do not
//!
//! The vectors are *abstract* vectors: a fragment, an initial stack,
//! and what the reviewed primitive contracts say it can reach. A
//! passing positive vector is a statement about the contracts, and a
//! passing negative vector says a stated mutation loses the successful
//! form or gains an abort. Neither is a target execution result, and
//! nothing here claims a node ran anything. Relation-indexed target
//! evidence is a later layer's obligation and stays outstanding.
//!
//! # The one thing the abstract walk cannot narrow
//!
//! Every field-introspection primitive has an explicit and a
//! confidential successful form, selected by a property of the value
//! that was read. A program cannot decide that and neither can the
//! validator, so both forms stay live through a fragment. The prefix
//! comparison in the recognition fragments is what settles it *on the
//! target* — the target pushes the confidential prefix for a blinded
//! field, and the comparison then fails — but the walk holds no bytes
//! for an introspected prefix, so it cannot remove the branch. That is
//! recorded as [`AbiAssumption::ExplicitFormEstablishedByPrefixEquality`]
//! rather than smuggled into a success set, and it is the reason the
//! aggregate fragment slices its operand to a fixed width before any
//! arithmetic touches it: the slice makes the width a program fact
//! instead of a branch the walk is trusting.
//!
//! # No literal here is minted
//!
//! The closed asset, the ASH constructor's program, the reserve asset,
//! the sponsor-change program, and the fee role's program digest are
//! supplied as [`CompactAshSymbols`], not written down. They are
//! link-time facts — a later wave resolves them — and a byte constant
//! invented here would be an identity this wave has no authority to
//! mint (§1.10).

use std::collections::{BTreeMap, BTreeSet};

use compiler::operation_plan::RequiredSourceKind;
use target_elements::{
    ElementsCapability, EncodingClass, FailureCause, OpcodeId, ResourceDimension,
    ReviewedElementsTapscriptDefinition, StackValueType, TargetEvidenceRequirementId,
};

use crate::capability::{BackendPatternId, census_enum};
use crate::error::TapscriptError;
use crate::instruction::{StackItem, TapscriptInstruction};
use crate::program::TapscriptProgram;
use crate::shape::{CompactAshShape, SponsorChangePresence};
use crate::stack::{
    AbstractExecutionResult, AbstractLimits, AbstractStackState, resource_projection,
    validate_program,
};

/// The semantic or target-structural relation one pattern owns.
///
/// The pattern does not restate the relation, it names it: the semantic
/// content stays with architecture and realization, and a target
/// package that copied a cardinality bound or an amount relation into
/// its own field would be a second source of truth for something it
/// does not own (§1.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PatternOwner {
    /// §12.1: an ASH input is the linked object, in the explicit form,
    /// in the authenticated range.
    AshInputRecognition,
    /// §12.2: output 0 is the successor, explicit, carrying the exact
    /// aggregate.
    SuccessorRecognition,
    /// §12.3: the coordinator authenticates the exact shape.
    FamilyCardinality,
    /// §12.4: the exact explicit sum, with every arithmetic flag
    /// consumed.
    ExactAggregateArithmetic,
    /// §12.5 and §12.6: every admitted position accounted for, and the
    /// partition proved rather than aggregated.
    CanonicalPartition,
    /// §10.3: the coordinator is at the canonical anchor and nowhere
    /// else.
    CoordinatorRole,
    /// §12.7 and §10.3: a member lies in the shape's member range.
    MemberParticipation,
    /// §12.8: the protocol leaves carry no authorization at all.
    PermissionlessPath,
    /// §12.9: the sponsor region is exact, disjoint, and never read for
    /// an amount.
    SponsorIsolation,
}

/// One assumption a pattern makes about the candidate ABI or the target
/// that its own instructions do not establish.
///
/// Stated rather than implied. Each of these is a place where the
/// fragment is correct *given* something another layer owes, and a
/// pattern that hid one would look self-contained when it is not.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AbiAssumption {
    /// Input 0 is the coordinator and the ASH range is the prefix
    /// `0..n` (§10.1).
    CanonicalInputOrder,
    /// Output 0 is the successor, then the optional change role, then
    /// the target fee role (§10.2).
    CanonicalOutputOrder,
    /// A field is established explicit by comparing its introspected
    /// prefix with the reviewed explicit prefix.
    ///
    /// The target pushes the confidential prefix for a blinded field
    /// and the comparison then fails. The abstract walk holds no bytes
    /// for an introspected prefix, so it keeps both successful forms
    /// live and cannot make that narrowing itself.
    ExplicitFormEstablishedByPrefixEquality,
    /// The symbols the fragment pushes are resolved to the linked
    /// object's exact bytes by a later layer.
    SymbolsResolvedAtLink,
    /// Whole-transaction value conservation is the target's own
    /// consensus claim and no program establishes it (§12.9).
    ExternalWholeTransactionConservation,
}

/// The exact link-time literals a compact-ASH fragment pushes.
///
/// Supplied, never minted. Each item is checked against the width the
/// reviewed contract fixes for the field it will be compared with, so a
/// symbol of the wrong shape is refused here rather than producing a
/// fragment that could never match anything.
///
/// # No ASH constructor program
///
/// There is no field for the ASH family's own witness program or for
/// the version it is read at, and the absence is structural. That
/// program is the taproot output over the taptree these very fragments
/// live in, so no caller can supply it and no fragment pushes it; the
/// fragments read it from the input they are spending instead (see
/// `require_program_matches_this_input`). A field reserved for it
/// would be a link-time parameter nothing consumes, which §1.10
/// refuses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompactAshSymbols {
    closed_asset: StackItem,
    reserve_asset: StackItem,
    sponsor_change_program: StackItem,
    sponsor_change_version: i64,
    fee_program_digest: StackItem,
}

impl CompactAshSymbols {
    /// Assemble the symbol set, checking each item's width.
    ///
    /// # Errors
    ///
    /// [`TapscriptError::MalformedEncodedItem`] when an item is not the
    /// width its encoding class admits.
    pub fn new(
        target: &ReviewedElementsTapscriptDefinition,
        closed_asset: Vec<u8>,
        reserve_asset: Vec<u8>,
        sponsor_change_program: Vec<u8>,
        sponsor_change_version: i64,
        fee_program_digest: Vec<u8>,
    ) -> Result<Self, TapscriptError> {
        Ok(Self {
            closed_asset: StackItem::encoded(target, EncodingClass::ExplicitAsset, closed_asset)?,
            reserve_asset: StackItem::encoded(target, EncodingClass::ExplicitAsset, reserve_asset)?,
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

    /// The closed protocol asset the ASH family carries.
    #[must_use]
    pub const fn closed_asset(&self) -> &StackItem {
        &self.closed_asset
    }

    /// The reserve asset every sponsor and fee role carries.
    #[must_use]
    pub const fn reserve_asset(&self) -> &StackItem {
        &self.reserve_asset
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
    ///
    /// The reviewed target replaces a program that is not a witness
    /// program by a digest of it, under a negative version marker, so
    /// the empty program the fee role carries reaches a script as this
    /// digest and that marker — and never as an amount test.
    #[must_use]
    pub const fn fee_program_digest(&self) -> &StackItem {
        &self.fee_program_digest
    }
}

/// The stack contract of one pattern's fragment (§12).
///
/// The initial stack the fragment is scheduled from, and every
/// successful state the reviewed contracts say it can reach. Both are
/// computed by the abstract validator when the pattern is built, not
/// declared alongside it: a declared contract and a computed one are
/// two answers that can disagree, and the whole point of the walk is
/// that there is one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternStackContract {
    initial: AbstractStackState,
    success: BTreeSet<AbstractStackState>,
}

impl PatternStackContract {
    /// The stack the fragment is scheduled from.
    #[must_use]
    pub const fn initial(&self) -> &AbstractStackState {
        &self.initial
    }

    /// Every successful state the fragment can reach.
    #[must_use]
    pub const fn success(&self) -> &BTreeSet<AbstractStackState> {
        &self.success
    }
}

/// What one pattern's fragment does when it does not succeed (§12.11).
///
/// The two are separate because they are different findings: an abort
/// ends the spend, and a non-aborting failure leaves a stack the next
/// instruction has to be scheduled against. A pattern with a surviving
/// non-aborting failure state has an unconsumed flag, which §12.11
/// forbids, so the emptiness of that set is a property worth carrying
/// rather than a detail.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternFailure {
    aborts: BTreeSet<FailureCause>,
    nonaborting: BTreeSet<AbstractStackState>,
}

impl PatternFailure {
    /// Every cause the fragment can abort on.
    #[must_use]
    pub const fn aborts(&self) -> &BTreeSet<FailureCause> {
        &self.aborts
    }

    /// Every state the fragment can reach through a non-aborting
    /// failure.
    #[must_use]
    pub const fn nonaborting(&self) -> &BTreeSet<AbstractStackState> {
        &self.nonaborting
    }
}

census_enum! {
    /// One typed mutation a negative vector applies to a fragment.
    ///
    /// Each names a way a fragment could be built wrong, and each is
    /// required to cost the fragment its successful form or to add an
    /// abort. A negative vector that changed nothing observable would be
    /// coverage that proves nothing.
    pub enum PatternMutation {
        /// Drop the fragment's last verifying primitive.
        ///
        /// The mis-scheduling §12.11 names: a result the program never
        /// checked, left on the stack where a caller might read it as
        /// truth.
        DropFinalVerification,
        /// Change one byte of the fragment's last pushed literal, keeping
        /// its width.
        ///
        /// A symbol resolved to the wrong object, or a count that does not
        /// match the shape. The abstract walk cannot decide this one where
        /// the literal is compared with a target-supplied value it holds no
        /// bytes for, and that limit is a result rather than a gap: it is
        /// exactly which of a pattern's claims rest on the link step
        /// resolving a symbol correctly.
        CorruptFinalLiteral,
        /// Append a byte to the fragment's last pushed literal.
        ///
        /// A symbol of the wrong shape rather than the wrong value. Where
        /// the literal feeds an operand whose width the contract fixes,
        /// this is decidable from the abstract types alone and the mutant
        /// stops being a schedulable program at all.
        WidenFinalLiteral,
    }
}

/// What one negative vector established about a pattern.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MutationOutcome {
    /// The mutation does not apply to this fragment.
    ///
    /// Never a pass: a fragment with no verifying primitive has none to
    /// drop, and a caller must read this as "there is no such vector".
    Inapplicable,
    /// The mutant is not a schedulable program.
    ///
    /// The strongest outcome: the defect is decided by the abstract
    /// types, before any question about a target value arises.
    Refused,
    /// The mutant schedules, and reaches a different contract.
    ContractChanged,
    /// The mutant schedules and reaches exactly the pattern's contract.
    ///
    /// The abstract walk cannot separate them, because the literal the
    /// mutation changed is compared with a value the walk holds no
    /// bytes for. The pattern's claim at that point rests on the link
    /// step, and this outcome is what says so.
    AbstractlyIndistinguishable,
}

/// The exact resource cost of one pattern's fragment.
///
/// Read from the resource projection, which charges the exact encoded
/// bytes of the program including its pushes. Nothing here is an
/// estimate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternResources {
    dimensions: BTreeMap<ResourceDimension, u64>,
}

impl PatternResources {
    /// Every charged dimension, in canonical order.
    #[must_use]
    pub const fn dimensions(&self) -> &BTreeMap<ResourceDimension, u64> {
        &self.dimensions
    }

    /// The exact encoded script bytes.
    #[must_use]
    pub fn script_bytes(&self) -> u64 {
        self.dimensions
            .get(&ResourceDimension::ScriptBytes)
            .copied()
            .unwrap_or_default()
    }
}

/// One complete backend proof pattern (§8.4).
///
/// Constructed only by [`build_pattern`], which computes the stack
/// contract, the failure behaviour, and the resources from the fragment
/// itself. There is no constructor that takes them as claims.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackendPattern {
    id: BackendPatternId,
    owner: PatternOwner,
    prerequisites: BTreeSet<ElementsCapability>,
    fragment: TapscriptProgram,
    stack: PatternStackContract,
    failure: PatternFailure,
    abi: BTreeSet<AbiAssumption>,
    sources: BTreeSet<RequiredSourceKind>,
    resources: PatternResources,
    evidence: BTreeSet<TargetEvidenceRequirementId>,
}

impl BackendPattern {
    /// The pattern's minted identity.
    #[must_use]
    pub const fn id(&self) -> BackendPatternId {
        self.id
    }

    /// The relation this pattern owns.
    #[must_use]
    pub const fn owner(&self) -> PatternOwner {
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

    /// The fragment's stack contract.
    #[must_use]
    pub const fn stack(&self) -> &PatternStackContract {
        &self.stack
    }

    /// The fragment's failure behaviour.
    #[must_use]
    pub const fn failure(&self) -> &PatternFailure {
        &self.failure
    }

    /// The ABI assumptions the fragment rests on.
    #[must_use]
    pub const fn abi(&self) -> &BTreeSet<AbiAssumption> {
        &self.abi
    }

    /// The compiler fact-source kinds this pattern routes.
    #[must_use]
    pub const fn sources(&self) -> &BTreeSet<RequiredSourceKind> {
        &self.sources
    }

    /// The fragment's exact resource cost.
    #[must_use]
    pub const fn resources(&self) -> &PatternResources {
        &self.resources
    }

    /// The target evidence the pattern's correctness depends on.
    #[must_use]
    pub const fn evidence(&self) -> &BTreeSet<TargetEvidenceRequirementId> {
        &self.evidence
    }

    /// The fragment with one mutation applied, for a negative vector.
    ///
    /// Returns `None` where the mutation does not apply — a fragment
    /// with no verifying primitive has none to drop, and one with no
    /// pushed literal has none to corrupt. A caller must treat that as
    /// "this pattern has no such negative vector", never as a pass.
    #[must_use]
    pub fn mutated(
        &self,
        target: &ReviewedElementsTapscriptDefinition,
        mutation: PatternMutation,
    ) -> Option<TapscriptProgram> {
        let mut instructions = self.fragment.instructions().to_vec();

        match mutation {
            PatternMutation::DropFinalVerification => {
                let position = instructions.iter().rposition(|instruction| {
                    matches!(
                        instruction,
                        TapscriptInstruction::Opcode(OpcodeId::Verify | OpcodeId::EqualVerify)
                    )
                })?;
                instructions.remove(position);
            }
            PatternMutation::CorruptFinalLiteral | PatternMutation::WidenFinalLiteral => {
                let position = instructions
                    .iter()
                    .rposition(|instruction| match instruction {
                        TapscriptInstruction::Push(item) => !item.is_empty(),
                        TapscriptInstruction::Opcode(_) => false,
                    })?;
                let TapscriptInstruction::Push(item) = &instructions[position] else {
                    return None;
                };
                let mut bytes = item.bytes().to_vec();
                if mutation == PatternMutation::WidenFinalLiteral {
                    bytes.push(0);
                } else {
                    // The first byte, flipped: enough to change the
                    // value without changing the width, so the mutation
                    // is the wrong literal rather than a differently
                    // shaped one.
                    bytes[0] ^= 0xff;
                }
                instructions[position] =
                    TapscriptInstruction::Push(StackItem::new(target, bytes).ok()?);
            }
        }

        TapscriptProgram::new(instructions).ok()
    }

    /// What one mutation establishes about this pattern.
    ///
    /// The negative half of the pattern's vectors, computed rather than
    /// declared: the mutant is walked and its contract compared with
    /// the pattern's.
    #[must_use]
    pub fn negative_vector(
        &self,
        target: &ReviewedElementsTapscriptDefinition,
        mutation: PatternMutation,
    ) -> MutationOutcome {
        let Some(mutant) = self.mutated(target, mutation) else {
            return MutationOutcome::Inapplicable;
        };
        let limits = AbstractLimits::for_target(target);
        let Ok(result) = validate_program(target, &mutant, self.stack.initial(), limits) else {
            return MutationOutcome::Refused;
        };

        if result.success() == &self.stack.success
            && result.aborts() == &self.failure.aborts
            && result.nonaborting_failure() == &self.failure.nonaborting
        {
            MutationOutcome::AbstractlyIndistinguishable
        } else {
            MutationOutcome::ContractChanged
        }
    }
}

/// Build one pattern by walking its fragment through the validator.
///
/// # Errors
///
/// Any failure of [`validate_program`], which is a defect in the
/// fragment rather than a property of the target: the validator refuses
/// a program whose operand it cannot admit, and a pattern built from
/// one would be a pattern nobody could schedule.
#[expect(
    clippy::too_many_arguments,
    reason = "the ten items §8.4 requires before a pattern may be minted; a builder \
              taking fewer would be a pattern admitted on less"
)]
pub fn build_pattern(
    target: &ReviewedElementsTapscriptDefinition,
    id: BackendPatternId,
    owner: PatternOwner,
    fragment: TapscriptProgram,
    initial: AbstractStackState,
    prerequisites: BTreeSet<ElementsCapability>,
    abi: BTreeSet<AbiAssumption>,
    sources: BTreeSet<RequiredSourceKind>,
    evidence: BTreeSet<TargetEvidenceRequirementId>,
) -> Result<BackendPattern, TapscriptError> {
    let limits = AbstractLimits::for_target(target);
    let result: AbstractExecutionResult = validate_program(target, &fragment, &initial, limits)?;
    let resources = PatternResources {
        dimensions: resource_projection(target, &fragment),
    };

    Ok(BackendPattern {
        id,
        owner,
        prerequisites,
        stack: PatternStackContract {
            initial,
            success: result.success().clone(),
        },
        failure: PatternFailure {
            aborts: result.aborts().clone(),
            nonaborting: result.nonaborting_failure().clone(),
        },
        abi,
        sources,
        resources,
        fragment,
        evidence,
    })
}

// --- Fragment construction ------------------------------------------

/// One reviewed primitive, as an instruction.
pub(crate) const fn op(id: OpcodeId) -> TapscriptInstruction {
    TapscriptInstruction::Opcode(id)
}

/// The reviewed prefix byte that selects one encoding class.
///
/// Read from the contract rather than written down, so a fragment
/// cannot compare against a prefix the target does not use.
///
/// # Errors
///
/// [`TapscriptError::MalformedEncodedItem`] when the class states no
/// prefix, and [`TapscriptError::OversizedStackItem`] when the reviewed
/// literal bound admits nothing at all.
pub(crate) fn prefix(
    target: &ReviewedElementsTapscriptDefinition,
    class: EncodingClass,
) -> Result<StackItem, TapscriptError> {
    let byte = target
        .definition()
        .encodings()
        .get(&class)
        .and_then(|spec| spec.prefixes().iter().next().copied())
        .ok_or(TapscriptError::MalformedEncodedItem { class })?;

    StackItem::new(target, vec![byte])
}

/// A script-number literal.
///
/// # Errors
///
/// [`TapscriptError::ScriptNumberOutOfRange`] when the value needs more
/// bytes than the reviewed script-number encoding admits.
pub(crate) fn number(
    target: &ReviewedElementsTapscriptDefinition,
    value: i64,
) -> Result<TapscriptInstruction, TapscriptError> {
    Ok(TapscriptInstruction::Push(StackItem::script_number(
        target, value,
    )?))
}

/// A signed fixed-width literal.
fn wide(target: &ReviewedElementsTapscriptDefinition, value: i64) -> TapscriptInstruction {
    TapscriptInstruction::Push(StackItem::signed_le64(target, value))
}

/// The exclusive upper bound of the semantic amount domain.
///
/// `2^51`, the bound §12.1 states. Restated here as the literal a
/// fragment pushes because the fragment has to push *something*; the
/// semantic ownership of the bound stays with the realization, and this
/// is the target-side encoding of it.
const AMOUNT_DOMAIN_BOUND: i64 = 1 << 51;

/// Establish that a field just introspected is in its explicit form.
///
/// Consumes the prefix the introspection pushed on top and leaves the
/// payload. See [`AbiAssumption::ExplicitFormEstablishedByPrefixEquality`]
/// for what this does and does not settle in the abstract walk.
fn require_explicit(
    target: &ReviewedElementsTapscriptDefinition,
    class: EncodingClass,
) -> Result<Vec<TapscriptInstruction>, TapscriptError> {
    Ok(vec![
        TapscriptInstruction::Push(prefix(target, class)?),
        op(OpcodeId::EqualVerify),
    ])
}

/// Narrow an explicit amount payload to a fixed-width arithmetic
/// operand.
///
/// The reviewed explicit-value payload is eight little-endian bytes,
/// and the arithmetic primitives take an eight-byte signed operand —
/// but they are two named encoding classes, and the validator does not
/// let one satisfy the other's declared type. Slicing the eight bytes
/// out yields an unconstrained item of settled width, which the
/// arithmetic operand does admit, and it makes the width a fact the
/// program established rather than one the schedule assumed.
fn narrow_to_operand(
    target: &ReviewedElementsTapscriptDefinition,
) -> Result<Vec<TapscriptInstruction>, TapscriptError> {
    Ok(vec![
        number(target, 0)?,
        number(target, 8)?,
        op(OpcodeId::Substring),
    ])
}

/// Require the amount on top to lie in the semantic domain (§12.1).
///
/// Positive and below `2^51`, checked in both directions, with each
/// comparison's Boolean consumed immediately so no unchecked result
/// survives (§12.11).
fn require_amount_domain(
    target: &ReviewedElementsTapscriptDefinition,
) -> Vec<TapscriptInstruction> {
    vec![
        op(OpcodeId::Duplicate),
        wide(target, 0),
        op(OpcodeId::GreaterThan64),
        op(OpcodeId::Verify),
        op(OpcodeId::Duplicate),
        wide(target, AMOUNT_DOMAIN_BOUND),
        op(OpcodeId::LessThan64),
        op(OpcodeId::Verify),
    ]
}

/// Require the field at `index` on `side` to carry exactly `asset`.
fn require_asset(
    target: &ReviewedElementsTapscriptDefinition,
    inspect: OpcodeId,
    index: i64,
    asset: &StackItem,
) -> Result<Vec<TapscriptInstruction>, TapscriptError> {
    let mut instructions = vec![number(target, index)?, op(inspect)];
    instructions.extend(require_explicit(target, EncodingClass::ExplicitAsset)?);
    instructions.push(TapscriptInstruction::Push(asset.clone()));
    instructions.push(op(OpcodeId::EqualVerify));
    Ok(instructions)
}

/// Require the program at `index` on `side` to be exactly this witness
/// program at this version.
///
/// For a program the deployment settles from outside the ASH family —
/// the sponsor-change role's. The ASH constructor's own program is not
/// settleable this way and uses
/// `require_program_matches_this_input` instead.
fn require_program(
    target: &ReviewedElementsTapscriptDefinition,
    inspect: OpcodeId,
    index: i64,
    version: i64,
    program: &StackItem,
) -> Result<Vec<TapscriptInstruction>, TapscriptError> {
    Ok(vec![
        number(target, index)?,
        op(inspect),
        number(target, version)?,
        op(OpcodeId::EqualVerify),
        TapscriptInstruction::Push(program.clone()),
        op(OpcodeId::EqualVerify),
    ])
}

/// Require the program at `index` on `side` to be the program of the
/// input this leaf is spending, version and payload alike.
///
/// # Why no literal
///
/// The ASH constructor's witness program is the taproot output that
/// commits to the taptree over the very leaves that would carry it, so
/// a literal for it is a value whose bytes must appear inside the thing
/// that computes them. Searching for that fixed point is finding a hash
/// preimage, and no deployment can supply one. A leaf that pushed such
/// a literal could therefore never be part of a linkable bundle at all.
///
/// The program does not need the literal. A leaf executes inside an
/// input, and that input's own witness program is on the target's
/// introspection surface, so the leaf can read the constructor's
/// program instead of carrying it. What the comparison then establishes
/// is *sameness* rather than a named value: the compared field carries
/// whatever program this input carries. That is the property the family
/// actually rests on — every ASH object and the successor share one
/// constructor — and it is established without any layer settling what
/// that constructor's bytes are.
///
/// # The schedule
///
/// Both introspections push the payload first and the version above it,
/// so the four items are, deepest first, this input's program, this
/// input's version, the compared program, the compared version. One
/// rotation brings this input's version to the top, above the compared
/// version, and the two equalities then run in the order the items
/// stand in. Each equality is a verifying form, so neither Boolean
/// survives to be read as truth (§12.11).
fn require_program_matches_this_input(
    target: &ReviewedElementsTapscriptDefinition,
    inspect: OpcodeId,
    index: i64,
) -> Result<Vec<TapscriptInstruction>, TapscriptError> {
    Ok(vec![
        op(OpcodeId::PushCurrentInputIndex),
        op(OpcodeId::InspectInputScriptPubKey),
        number(target, index)?,
        op(inspect),
        op(OpcodeId::Rotate),
        op(OpcodeId::EqualVerify),
        op(OpcodeId::EqualVerify),
    ])
}

/// Recognize one ASH input and leave its amount on the stack (§12.1).
///
/// Asset, program, explicit value encoding, and the amount domain. The
/// amount is left because the aggregate consumes it: recomputing it
/// would introspect the same field twice and pay for it twice.
///
/// The program test is against the input this leaf is spending rather
/// than against a literal — see `require_program_matches_this_input`
/// for why the literal is unobtainable and what sameness establishes
/// instead. At `index` zero the coordinator's own input is compared
/// with itself and the test is vacuous; that is not a hole but the
/// reason the whole scheme is sound, because the coordinator leaf runs
/// at input 0 (§10.3) and so the family's constructor is exactly the
/// program every other test is measured against.
///
/// # Errors
///
/// [`TapscriptError::ScriptNumberOutOfRange`] or
/// [`TapscriptError::OversizedStackItem`] when a literal this fragment
/// pushes is outside what the reviewed contract admits, and
/// [`TapscriptError::InstructionLimitExceeded`] when the fragment
/// exceeds the instruction bound.
pub fn ash_input_recognition_fragment(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &CompactAshSymbols,
    index: u8,
) -> Result<TapscriptProgram, TapscriptError> {
    let index = i64::from(index);
    let mut instructions = require_asset(
        target,
        OpcodeId::InspectInputAsset,
        index,
        symbols.closed_asset(),
    )?;
    instructions.extend(require_program_matches_this_input(
        target,
        OpcodeId::InspectInputScriptPubKey,
        index,
    )?);
    instructions.extend([number(target, index)?, op(OpcodeId::InspectInputValue)]);
    instructions.extend(require_explicit(target, EncodingClass::ExplicitValue)?);
    instructions.extend(narrow_to_operand(target)?);
    instructions.extend(require_amount_domain(target));

    TapscriptProgram::new(instructions)
}

/// Recognize the successor at output 0 (§12.2).
///
/// Asset, program, explicit encoding, and the amount domain, leaving
/// the successor's amount for the aggregate comparison.
///
/// The program test compares output 0 with the input this leaf is
/// spending (see `require_program_matches_this_input`), which is what
/// makes the successor a member of the same family rather than an
/// object under some separately named program. This is the fragment
/// where the comparison carries its full content: nothing else in the
/// candidate ties an output back to the constructor.
///
/// # Errors
///
/// [`TapscriptError::ScriptNumberOutOfRange`] or
/// [`TapscriptError::OversizedStackItem`] when a literal this fragment
/// pushes is outside what the reviewed contract admits, and
/// [`TapscriptError::InstructionLimitExceeded`] when the fragment
/// exceeds the instruction bound.
pub fn successor_recognition_fragment(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &CompactAshSymbols,
) -> Result<TapscriptProgram, TapscriptError> {
    let mut instructions = require_asset(
        target,
        OpcodeId::InspectOutputAsset,
        0,
        symbols.closed_asset(),
    )?;
    instructions.extend(require_program_matches_this_input(
        target,
        OpcodeId::InspectOutputScriptPubKey,
        0,
    )?);
    instructions.extend([number(target, 0)?, op(OpcodeId::InspectOutputValue)]);
    instructions.extend(require_explicit(target, EncodingClass::ExplicitValue)?);
    instructions.extend(narrow_to_operand(target)?);
    instructions.extend(require_amount_domain(target));

    TapscriptProgram::new(instructions)
}

/// Authenticate the shape's exact input and output counts (§12.3).
///
/// The caller-proposed counts of §12.3 are not witnesses here at all:
/// the shape is compiled in, and the fragment compares the target's own
/// introspected counts against it.
///
/// # Errors
///
/// [`TapscriptError::ScriptNumberOutOfRange`] or
/// [`TapscriptError::OversizedStackItem`] when a literal this fragment
/// pushes is outside what the reviewed contract admits, and
/// [`TapscriptError::InstructionLimitExceeded`] when the fragment
/// exceeds the instruction bound.
pub fn cardinality_fragment(
    target: &ReviewedElementsTapscriptDefinition,
    shape: CompactAshShape,
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

/// Bind the coordinator leaf to input 0 (§10.3).
///
/// # Errors
///
/// [`TapscriptError::ScriptNumberOutOfRange`] or
/// [`TapscriptError::OversizedStackItem`] when a literal this fragment
/// pushes is outside what the reviewed contract admits, and
/// [`TapscriptError::InstructionLimitExceeded`] when the fragment
/// exceeds the instruction bound.
pub fn coordinator_role_fragment(
    target: &ReviewedElementsTapscriptDefinition,
) -> Result<TapscriptProgram, TapscriptError> {
    TapscriptProgram::new(vec![
        op(OpcodeId::PushCurrentInputIndex),
        number(target, 0)?,
        op(OpcodeId::EqualVerify),
    ])
}

/// Bind a member leaf to the shape's member range (§10.3, §12.7).
///
/// `1 ≤ index < n`, both ends checked and both Booleans consumed. A
/// member leaf at input 0 fails the lower bound; one in the sponsor
/// suffix fails the upper.
///
/// # Errors
///
/// [`TapscriptError::ScriptNumberOutOfRange`] or
/// [`TapscriptError::OversizedStackItem`] when a literal this fragment
/// pushes is outside what the reviewed contract admits, and
/// [`TapscriptError::InstructionLimitExceeded`] when the fragment
/// exceeds the instruction bound.
pub fn member_role_fragment(
    target: &ReviewedElementsTapscriptDefinition,
    shape: CompactAshShape,
) -> Result<TapscriptProgram, TapscriptError> {
    member_range_fragment(target, shape.ash_inputs())
}

/// Bind the executing leaf to `1 ≤ index < protocol_inputs`.
///
/// The whole of a member role, with the count as a figure rather than a
/// shape, so the two operations that have a member range share the
/// bytes rather than each writing them out. A live transfer's member
/// range is the same statement over its own receipt-input count
/// (Guide-13 §10.3), and two copies of a bound check are two places for
/// an off-by-one to live.
///
/// # Errors
///
/// [`TapscriptError::ScriptNumberOutOfRange`] or
/// [`TapscriptError::OversizedStackItem`] when a literal this fragment
/// pushes is outside what the reviewed contract admits, and
/// [`TapscriptError::InstructionLimitExceeded`] when the fragment
/// exceeds the instruction bound.
pub(crate) fn member_range_fragment(
    target: &ReviewedElementsTapscriptDefinition,
    protocol_inputs: u8,
) -> Result<TapscriptProgram, TapscriptError> {
    TapscriptProgram::new(vec![
        op(OpcodeId::PushCurrentInputIndex),
        op(OpcodeId::ScriptNumToLe64),
        op(OpcodeId::Duplicate),
        wide(target, 1),
        op(OpcodeId::GreaterThanOrEqual64),
        op(OpcodeId::Verify),
        wide(target, i64::from(protocol_inputs)),
        op(OpcodeId::LessThan64),
        op(OpcodeId::Verify),
    ])
}

/// Sum `n` recognized amounts and require the successor to equal the
/// total (§12.4).
///
/// Scheduled from a stack holding the successor's amount *below* the
/// `n` source amounts — the shape the recognition fragments leave when
/// the successor is recognized first. That order is what makes the
/// aggregate expressible at all: the reviewed primitives add the top
/// two items, so a running total must be built on top of anything it
/// will later be compared with, and no alternate stack is touched.
///
/// Each addition's success flag is verified immediately, so no
/// unchecked arithmetic result survives (§12.11) and an overflow ends
/// the spend rather than leaving a false to be reduced later. The
/// closing byte-equality is exact: both operands are canonical
/// eight-byte little-endian encodings, so equality as bytes is equality
/// as numbers.
///
/// # Errors
///
/// [`TapscriptError::ScriptNumberOutOfRange`] or
/// [`TapscriptError::OversizedStackItem`] when a literal this fragment
/// pushes is outside what the reviewed contract admits, and
/// [`TapscriptError::InstructionLimitExceeded`] when the fragment
/// exceeds the instruction bound.
pub fn explicit_sum_fragment(
    target: &ReviewedElementsTapscriptDefinition,
    shape: CompactAshShape,
) -> Result<TapscriptProgram, TapscriptError> {
    let _ = target;
    let mut instructions = Vec::new();

    for _ in 1..shape.ash_inputs() {
        instructions.extend([op(OpcodeId::Add64), op(OpcodeId::Verify)]);
    }

    instructions.push(op(OpcodeId::EqualVerify));
    TapscriptProgram::new(instructions)
}

/// Account for every output the shape declares (§12.5, §12.6, §12.10).
///
/// Output 0's recognition is the successor pattern's, so this fragment
/// covers the remaining roles: the optional sponsor change and the
/// target fee role. Together with the exact output count the
/// cardinality fragment pins, every admitted position is visited, which
/// is what makes "no unexpected output" and the absence of every root
/// and specialized-event output structural rather than asserted.
///
/// # Errors
///
/// [`TapscriptError::ScriptNumberOutOfRange`] or
/// [`TapscriptError::OversizedStackItem`] when a literal this fragment
/// pushes is outside what the reviewed contract admits, and
/// [`TapscriptError::InstructionLimitExceeded`] when the fragment
/// exceeds the instruction bound.
pub fn output_census_fragment(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &CompactAshSymbols,
    shape: CompactAshShape,
) -> Result<TapscriptProgram, TapscriptError> {
    let mut instructions = Vec::new();
    let mut position = 1_i64;

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
            symbols.sponsor_change_version,
            &symbols.sponsor_change_program,
        )?);
        position += 1;
    }

    if shape.sponsored() {
        // The fee role by form, never by amount: the reviewed target
        // replaces the empty program by its digest under a negative
        // version marker, and that pair is the whole discriminator.
        instructions.extend(require_asset(
            target,
            OpcodeId::InspectOutputAsset,
            position,
            symbols.reserve_asset(),
        )?);
        instructions.extend([
            number(target, position)?,
            op(OpcodeId::InspectOutputScriptPubKey),
            number(target, -1)?,
            op(OpcodeId::EqualVerify),
            TapscriptInstruction::Push(symbols.fee_program_digest().clone()),
            op(OpcodeId::EqualVerify),
        ]);
    }

    TapscriptProgram::new(instructions)
}

/// Isolate the sponsor region (§12.9).
///
/// Every sponsor input is required to carry the reserve asset, in the
/// explicit form the target's own prefix rule forces. Nothing here
/// reads a sponsor *value*: there is no value introspection in this
/// fragment at all, which is the property §1.6 asks for and the one an
/// audit of the emitted bytes can check.
///
/// The sponsorless case emits nothing, and that is the point: the
/// region's absence is established by the exact input count the
/// cardinality fragment pins, not by a test that could be omitted.
///
/// # Errors
///
/// [`TapscriptError::ScriptNumberOutOfRange`] or
/// [`TapscriptError::OversizedStackItem`] when a literal this fragment
/// pushes is outside what the reviewed contract admits, and
/// [`TapscriptError::InstructionLimitExceeded`] when the fragment
/// exceeds the instruction bound.
pub fn sponsor_isolation_fragment(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &CompactAshSymbols,
    shape: CompactAshShape,
) -> Result<TapscriptProgram, TapscriptError> {
    let (start, end) = shape.sponsor_range();
    let mut instructions = Vec::new();

    for index in start..end {
        instructions.extend(require_asset(
            target,
            OpcodeId::InspectInputAsset,
            i64::from(index),
            symbols.reserve_asset(),
        )?);
    }

    TapscriptProgram::new(instructions)
}

/// The canonical true item every program ends on (§12.11).
///
/// # Errors
///
/// [`TapscriptError::ScriptNumberOutOfRange`] or
/// [`TapscriptError::OversizedStackItem`] when a literal this fragment
/// pushes is outside what the reviewed contract admits, and
/// [`TapscriptError::InstructionLimitExceeded`] when the fragment
/// exceeds the instruction bound.
pub fn final_truth_fragment(
    target: &ReviewedElementsTapscriptDefinition,
) -> Result<TapscriptProgram, TapscriptError> {
    TapscriptProgram::new(vec![number(target, 1)?])
}

/// The whole coordinator program for one shape.
///
/// Concatenated in the order the schedules were verified in: the
/// coordinator's own anchor first, then the shape, so a transaction of
/// the wrong shape is refused before anything is introspected; then the
/// successor, so its amount sits below the running total; then every
/// ASH source and the aggregate; then the remaining outputs and the
/// sponsor region; then the canonical true item.
///
/// # Errors
///
/// [`TapscriptError::ScriptNumberOutOfRange`] or
/// [`TapscriptError::OversizedStackItem`] when a literal this fragment
/// pushes is outside what the reviewed contract admits, and
/// [`TapscriptError::InstructionLimitExceeded`] when the fragment
/// exceeds the instruction bound.
pub fn coordinator_program(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &CompactAshSymbols,
    shape: CompactAshShape,
) -> Result<TapscriptProgram, TapscriptError> {
    let mut instructions = coordinator_role_fragment(target)?.instructions().to_vec();

    instructions.extend_from_slice(cardinality_fragment(target, shape)?.instructions());
    instructions.extend_from_slice(successor_recognition_fragment(target, symbols)?.instructions());
    for index in 0..shape.ash_inputs() {
        instructions.extend_from_slice(
            ash_input_recognition_fragment(target, symbols, index)?.instructions(),
        );
    }
    instructions.extend_from_slice(explicit_sum_fragment(target, shape)?.instructions());
    instructions.extend_from_slice(output_census_fragment(target, symbols, shape)?.instructions());
    instructions
        .extend_from_slice(sponsor_isolation_fragment(target, symbols, shape)?.instructions());
    instructions.extend_from_slice(final_truth_fragment(target)?.instructions());

    TapscriptProgram::new(instructions)
}

/// The whole member program for one shape.
///
/// A member proves its range, its own object, and nothing else: §12.7
/// says member leaves do not independently re-prove the whole
/// aggregate, and this candidate does not select duplicate enforcement.
/// It ends on the canonical true item like every other program.
///
/// # Errors
///
/// [`TapscriptError::ScriptNumberOutOfRange`] or
/// [`TapscriptError::OversizedStackItem`] when a literal this fragment
/// pushes is outside what the reviewed contract admits, and
/// [`TapscriptError::InstructionLimitExceeded`] when the fragment
/// exceeds the instruction bound.
pub fn member_program(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &CompactAshSymbols,
    shape: CompactAshShape,
) -> Result<TapscriptProgram, TapscriptError> {
    let mut instructions = member_role_fragment(target, shape)?.instructions().to_vec();

    // The member's own asset, at whichever input it is spending. The
    // index is the target's own current-input index rather than a
    // compiled-in position, so one member leaf serves every member
    // position of the shape.
    //
    // There is no program test here, and its absence is a result rather
    // than an omission. A member leaf has one input to look at — its
    // own — so the only program test it could schedule compares that
    // input's program with itself. The coordinator's fragments compare
    // *other* positions against the input they run in, which is what
    // gives them content; a member has no other position in view. What
    // binds this leaf to the family is that it executes at all: a leaf
    // runs only from a taptree its input's program commits to.
    instructions.extend([
        op(OpcodeId::PushCurrentInputIndex),
        op(OpcodeId::InspectInputAsset),
    ]);
    instructions.extend(require_explicit(target, EncodingClass::ExplicitAsset)?);
    instructions.extend([
        TapscriptInstruction::Push(symbols.closed_asset().clone()),
        op(OpcodeId::EqualVerify),
    ]);
    instructions.extend_from_slice(final_truth_fragment(target)?.instructions());

    TapscriptProgram::new(instructions)
}

/// Every reviewed primitive one program uses, as capability identities.
///
/// Derived from the program rather than declared beside it, so a
/// pattern's prerequisite census cannot drift from the instructions it
/// actually schedules.
#[must_use]
pub fn fragment_prerequisites(program: &TapscriptProgram) -> BTreeSet<ElementsCapability> {
    let mut prerequisites = BTreeSet::new();

    for instruction in program.instructions() {
        let TapscriptInstruction::Opcode(id) = instruction else {
            continue;
        };
        prerequisites.extend(capability_of(*id));
    }
    prerequisites
}

/// The capability one reviewed primitive belongs to.
///
/// Exhaustive with no wildcard arm: a primitive added to the reviewed
/// census stops this crate compiling until its capability is stated,
/// which is the same mechanism the assessment mapping uses.
#[expect(
    clippy::match_same_arms,
    reason = "one arm per primitive group; merging coincident ones would delete which \
              group a primitive belongs to"
)]
const fn capability_of(id: OpcodeId) -> Option<ElementsCapability> {
    use ElementsCapability as C;
    use OpcodeId as O;

    Some(match id {
        O::Sha256Initialize | O::Sha256Update | O::Sha256Finalize => return None,

        O::InspectInputOutpoint => C::InputOutpointInspection,
        O::InspectInputAsset => C::InputAssetInspection,
        O::InspectInputValue => C::InputValueInspection,
        O::InspectInputScriptPubKey => C::InputProgramInspection,
        O::InspectInputSequence => C::InputSequenceInspection,
        O::InspectInputIssuance => C::InputIssuanceInspection,
        O::PushCurrentInputIndex => C::CurrentInputIndexInspection,
        O::InspectOutputAsset => C::OutputAssetInspection,
        O::InspectOutputValue => C::OutputValueInspection,
        O::InspectOutputNonce => C::OutputNonceInspection,
        O::InspectOutputScriptPubKey => C::OutputProgramInspection,
        O::InspectVersion => C::TransactionVersionInspection,
        O::InspectLockTime => C::TransactionLockTimeInspection,
        O::InspectNumInputs => C::InputCountInspection,
        O::InspectNumOutputs => C::OutputCountInspection,
        O::TxWeight => C::TransactionWeightInspection,

        O::Add64 | O::Sub64 | O::Mul64 | O::Div64 | O::Neg64 => C::SignedFixedWidthArithmetic,
        O::LessThan64 | O::LessThanOrEqual64 | O::GreaterThan64 | O::GreaterThanOrEqual64 => {
            C::SignedFixedWidthComparison
        }
        O::ScriptNumToLe64 | O::Le64ToScriptNum | O::Le32ToLe64 => C::ScriptNumberConversion,

        O::EcMulScalarVerify => C::EcScalarVerification,
        O::TweakVerify => C::TweakVerification,
        O::CheckSig | O::CheckSigVerify => C::SignatureVerification,
        O::CheckSigFromStack | O::CheckSigFromStackVerify => C::StackMessageSignatureVerification,
        O::CheckSequenceVerify => C::RelativeTimelock,

        O::Duplicate
        | O::DuplicateTwo
        | O::CopyOver
        | O::Swap
        | O::Rotate
        | O::RemoveSecond
        | O::Tuck
        | O::Drop
        | O::DropTwo => C::StackRearrangement,

        O::Equal | O::EqualVerify => C::ByteStringEquality,
        O::Verify => C::BooleanVerification,
        O::Concatenate => C::ByteStringConcatenation,
        O::Size => C::ByteStringWidth,
        O::Substring => C::ByteStringSlicing,
        O::BitwiseAnd | O::BitwiseXor => C::BitwiseByteLogic,

        // The primitive census is non-exhaustive, so a primitive this
        // version has not been taught has no capability rather than a
        // guessed one. A pattern using it would report an incomplete
        // prerequisite census, which is visible; a guessed capability
        // would not be.
        _ => return None,
    })
}

/// Every primitive that verifies a signature.
///
/// The census §12.8's byte-level audit runs against. Written as its own
/// list rather than derived from a capability, because the claim is
/// about the emitted instructions and an audit that went through a
/// capability mapping would be checking the mapping.
pub const AUTHORIZATION_PRIMITIVES: &[OpcodeId] = &[
    OpcodeId::CheckSig,
    OpcodeId::CheckSigVerify,
    OpcodeId::CheckSigFromStack,
    OpcodeId::CheckSigFromStackVerify,
    OpcodeId::CheckSequenceVerify,
];

/// Whether a program carries any authorization or cadence primitive
/// (§12.8).
///
/// The permissionless claim is exactly the falsity of this, checked on
/// the typed instructions a program is made of.
#[must_use]
pub fn carries_authorization(program: &TapscriptProgram) -> bool {
    program.instructions().iter().any(|instruction| {
        matches!(
            instruction,
            TapscriptInstruction::Opcode(id) if AUTHORIZATION_PRIMITIVES.contains(id)
        )
    })
}

/// The width of one explicit amount operand, in bytes.
///
/// The reviewed explicit-value payload's width, which is also the
/// arithmetic operand's. Named once so the initial stack of the
/// aggregate schedule and the slice the recognition fragment takes
/// cannot drift apart.
pub const AMOUNT_OPERAND_BYTES: usize = 8;

/// Every operation-proven pattern, for one shape.
///
/// The census [`BackendPatternId`] draws its variants from. Each record
/// is built by walking its own fragment, so nothing here is a claim
/// about a fragment that was not scheduled.
///
/// # Errors
///
/// Any failure of [`build_pattern`] or of the fragment builders.
#[expect(
    clippy::too_many_lines,
    reason = "one block per admitted pattern, each naming its own owner, assumptions, \
              sources, and evidence; splitting the census would hide which patterns \
              exist and let one be dropped without the count changing"
)]
pub fn operation_patterns(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &CompactAshSymbols,
    shape: CompactAshShape,
) -> Result<BTreeMap<BackendPatternId, BackendPattern>, TapscriptError> {
    use AbiAssumption as A;
    use RequiredSourceKind as S;
    use TargetEvidenceRequirementId as R;

    let empty = AbstractStackState::from_main(Vec::new());
    let introspection = [R::OpcodeSemantics, R::EncodingSemantics];

    let mut patterns = BTreeMap::new();

    let mut admit = |pattern: BackendPattern| {
        patterns.insert(pattern.id(), pattern);
    };

    admit(build_pattern(
        target,
        BackendPatternId::CompactAshCoordinatorRoleV1,
        PatternOwner::CoordinatorRole,
        coordinator_role_fragment(target)?,
        empty.clone(),
        fragment_prerequisites(&coordinator_role_fragment(target)?),
        BTreeSet::from([A::CanonicalInputOrder]),
        BTreeSet::from([S::AuthenticatedFamilyCensus]),
        introspection
            .into_iter()
            .chain([R::InputIntrospectionSemantics])
            .collect(),
    )?);

    admit(build_pattern(
        target,
        BackendPatternId::CompactAshMemberRoleV1,
        PatternOwner::MemberParticipation,
        member_role_fragment(target, shape)?,
        empty.clone(),
        fragment_prerequisites(&member_role_fragment(target, shape)?),
        BTreeSet::from([A::CanonicalInputOrder]),
        BTreeSet::from([S::AuthenticatedFamilyCensus]),
        introspection
            .into_iter()
            .chain([
                R::InputIntrospectionSemantics,
                R::ComparisonSemantics,
                R::ConversionSemantics,
            ])
            .collect(),
    )?);

    let recognition = ash_input_recognition_fragment(target, symbols, 0)?;
    admit(build_pattern(
        target,
        BackendPatternId::CompactAshObjectRecognitionV1,
        PatternOwner::AshInputRecognition,
        recognition.clone(),
        empty.clone(),
        fragment_prerequisites(&recognition),
        BTreeSet::from([
            A::CanonicalInputOrder,
            A::ExplicitFormEstablishedByPrefixEquality,
            A::SymbolsResolvedAtLink,
        ]),
        BTreeSet::from([S::AuthenticatedInputObject]),
        introspection
            .into_iter()
            .chain([
                R::InputIntrospectionSemantics,
                R::ComparisonSemantics,
                // The program test reads this leaf's own input and then
                // rotates the four introspected items into comparison
                // order, so the rearrangement primitives' semantics are
                // now part of what this pattern rests on.
                R::StackRearrangementSemantics,
            ])
            .collect(),
    )?);

    let cardinality = cardinality_fragment(target, shape)?;
    admit(build_pattern(
        target,
        BackendPatternId::CompactAshShapeV1,
        PatternOwner::FamilyCardinality,
        cardinality.clone(),
        empty.clone(),
        fragment_prerequisites(&cardinality),
        BTreeSet::from([A::CanonicalInputOrder, A::CanonicalOutputOrder]),
        BTreeSet::from([S::AuthenticatedFamilyCensus]),
        introspection
            .into_iter()
            .chain([R::TransactionIntrospectionSemantics])
            .collect(),
    )?);

    // The aggregate is scheduled from the stack its upstream fragments
    // leave: the successor's amount, then one amount per ASH source.
    let sum = explicit_sum_fragment(target, shape)?;
    let operands = vec![operand_type(AMOUNT_OPERAND_BYTES); usize::from(shape.ash_inputs()) + 1];
    admit(build_pattern(
        target,
        BackendPatternId::CompactAshExplicitSumV1,
        PatternOwner::ExactAggregateArithmetic,
        sum.clone(),
        AbstractStackState::from_main(operands),
        fragment_prerequisites(&sum),
        BTreeSet::from([A::ExplicitFormEstablishedByPrefixEquality]),
        BTreeSet::from([S::AuthenticatedConsensusValue]),
        introspection
            .into_iter()
            .chain([R::ArithmeticSemantics])
            .collect(),
    )?);

    let census = output_census_fragment(target, symbols, shape)?;
    admit(build_pattern(
        target,
        BackendPatternId::CompactAshCanonicalPartitionV1,
        PatternOwner::CanonicalPartition,
        census.clone(),
        empty.clone(),
        fragment_prerequisites(&census),
        BTreeSet::from([A::CanonicalOutputOrder, A::SymbolsResolvedAtLink]),
        BTreeSet::from([
            S::AuthenticatedOutputObject,
            S::AuthenticatedTransitionCertificate,
        ]),
        introspection
            .into_iter()
            .chain([
                R::OutputIntrospectionSemantics,
                R::TransactionIntrospectionSemantics,
                R::FeeOutputForm,
                R::ExplicitZeroValueOutputRule,
                R::FeelessTransactionAdmission,
            ])
            .collect(),
    )?);

    let isolation = sponsor_isolation_fragment(target, symbols, shape)?;
    admit(build_pattern(
        target,
        BackendPatternId::CompactAshSponsorIsolationV1,
        PatternOwner::SponsorIsolation,
        isolation.clone(),
        empty.clone(),
        fragment_prerequisites(&isolation),
        BTreeSet::from([
            A::CanonicalInputOrder,
            A::ExplicitFormEstablishedByPrefixEquality,
            A::ExternalWholeTransactionConservation,
        ]),
        BTreeSet::from([S::AuthenticatedFamilyCensus]),
        introspection
            .into_iter()
            .chain([
                R::InputIntrospectionSemantics,
                R::ConfidentialValueConservation,
            ])
            .collect(),
    )?);

    // The permissionless claim is about the whole emitted program
    // rather than about a fragment of it, so the coordinator program is
    // what carries it. Its content is the absence checked by
    // [`carries_authorization`] and by the abort set, which holds no
    // signature cause.
    let whole = coordinator_program(target, symbols, shape)?;
    admit(build_pattern(
        target,
        BackendPatternId::CompactAshPermissionlessPathV1,
        PatternOwner::PermissionlessPath,
        whole.clone(),
        empty,
        fragment_prerequisites(&whole),
        BTreeSet::from([A::CanonicalInputOrder, A::CanonicalOutputOrder]),
        BTreeSet::from([S::PublicConstructionData]),
        introspection.into_iter().collect(),
    )?);

    Ok(patterns)
}

/// The abstract type of an item of `width` bytes.
///
/// The shape a recognition fragment leaves its amount in, so a
/// downstream fragment can be scheduled from the stack the upstream one
/// produced rather than from a guess about it.
#[must_use]
pub const fn operand_type(width: usize) -> StackValueType {
    StackValueType::Bytes {
        minimum: width,
        maximum: width,
    }
}
