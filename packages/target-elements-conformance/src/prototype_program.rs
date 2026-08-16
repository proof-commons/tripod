//! The constructor prototype as an emitted target program.
//!
//! # What changes here, relative to a schedule
//!
//! The schedules in the `tapscript` package are statements about the
//! reviewed contracts: a typed instruction sequence, a typed initial
//! stack, and the complete set of states the contracts say it can
//! reach. They carry placeholder literals of the right widths, because
//! a schedule is about stack discipline and a width is all it needs.
//!
//! A prototype program is the same construction with every literal
//! resolved to the value a spend would actually carry: both tag digests
//! of each tagged hash, the leaf version byte, the compact-size length
//! of the metadata leaf script, the push form the metadata arrives in,
//! the leaf script's tail, and the published internal key. The abstract
//! validator then accepts *these bytes'* schedule exactly as it accepted
//! the hand-written one, which is the property that makes the emitted
//! program the same construction rather than a second one that happens
//! to look similar (Guide-10 `rule:guide10:stack-schedule`).
//!
//! # A research object, and it stays one
//!
//! Nothing here converts into a production type, and no such conversion
//! may be added. A prototype program is admitted by a constructor that
//! checks the reviewed contract revision, the primitive census, the
//! abstract schedule, and the resource projection — and none of those
//! checks is evidence that a node accepts the program. The status field
//! carries that distinction and the constructor never sets the accepting
//! one on its own (Guide-10 `rule:guide10:no-calibration`).
//!
//! # No identity
//!
//! A prototype program is compared by its typed content. There is no
//! digest of one and no field reserved for one, for the reason the
//! package documents generally: a hash would become an identity that
//! outlived the content it summarized.

use std::collections::{BTreeMap, BTreeSet};

use tapscript::instruction::{StackItem, TapscriptInstruction};
use tapscript::program::TapscriptProgram;
use tapscript::stack::{AbstractLimits, AbstractStackState, resource_projection, validate_program};
use target_elements::{
    EncodingClass, OpcodeId, PayloadWidth, ResourceDimension, ReviewedElementsTapscriptDefinition,
    StackValueType, TargetContractVersion,
};

use crate::constructor::curve::FIELD_ELEMENT_BYTES;
use crate::constructor::internal_key::UNSPENDABLE_INTERNAL_KEY;
use crate::constructor::metadata::METADATA_BYTES;
use crate::constructor::metadata_leaf::metadata_leaf_script;
use crate::constructor::tagged::{
    DIGEST_BYTES, TAP_BRANCH_TAG, TAP_LEAF_TAG, TAP_TWEAK_TAG, compact_size, sha256,
};

/// Which prototype construction a program is.
///
/// One variant, and a new one is a reviewed addition rather than a
/// string somebody passes in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum PrototypeKind {
    /// The continuity proof: one static subtree root carried across a
    /// predecessor constructor and its successor.
    MetadataConstructorContinuity,
}

/// What a passing execution of a prototype program would establish.
///
/// Kept apart from the kind on purpose. The kind is what the program
/// *is*; the relation is what an accepting verdict would mean, and a
/// program of one kind that established some other relation would be a
/// defect rather than a variation (Guide-10
/// `rule:guide10:relation-identity`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum PrototypeProgramRelation {
    /// One metadata object's constructor derives from its predecessor's
    /// by the stated transition.
    MetadataConstructorContinuity,
}

/// How far a prototype program has been taken.
///
/// # Why the accepting status is not reachable from the constructor
///
/// Every check the constructor makes is a statement about the reviewed
/// contracts. None of them is a node's verdict, and a constructor that
/// returned the accepting status would be reporting a target-native
/// result it never obtained. The status is therefore set by whatever
/// consumes a real run, and the constructor sets the experimental one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum PrototypeStatus {
    /// Admitted against the reviewed contracts, and nothing more.
    Experimental,
    /// Carried by a target-native run that accepted it.
    AcceptedResearchResult,
}

/// The reviewed target facts a prototype program is stated against.
///
/// A projection rather than the target itself: what a prototype program
/// must pin is the contract revision and the leaf version it was emitted
/// under, and carrying the whole definition would make two programs
/// compare unequal for reasons that have nothing to do with either.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PrototypeTargetProjection {
    contract_version: u32,
    leaf_version: u8,
}

impl PrototypeTargetProjection {
    /// The reviewed contract revision.
    #[must_use]
    pub const fn contract_version(self) -> u32 {
        self.contract_version
    }

    /// The reviewed leaf version byte.
    #[must_use]
    pub const fn leaf_version(self) -> u8 {
        self.leaf_version
    }
}

/// What one execution of a prototype program costs, by dimension.
///
/// # Why the units stay apart
///
/// Script bytes, witness bytes, stack depth, element width, and the
/// script-path validation budget bound different things and are measured
/// in different units. A single combined score would be a number no
/// target limit is expressed in, so each dimension is its own field and
/// the question "which limit binds first" is answered by comparing each
/// against its own bound.
///
/// # The weight is the witness contribution, not a transaction weight
///
/// A transaction's weight depends on its inputs, its outputs, and its
/// fee arrangement, none of which a program determines. What a program
/// determines is the weight its own witness contributes, at one unit per
/// witness byte, and that is what this field states. A consumer that
/// needs a transaction weight has to weigh a transaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PrototypeResourceProjection {
    script_bytes: u64,
    witness_bytes: u64,
    peak_main_stack: u64,
    largest_element_bytes: u64,
    hash_operations: u64,
    curve_operations: u64,
    validation_budget: u64,
    witness_weight: u64,
}

impl PrototypeResourceProjection {
    /// The emitted script's exact byte count.
    #[must_use]
    pub const fn script_bytes(self) -> u64 {
        self.script_bytes
    }

    /// The witness's serialized byte count, control block included.
    #[must_use]
    pub const fn witness_bytes(self) -> u64 {
        self.witness_bytes
    }

    /// The greatest main-stack depth any reachable state holds.
    #[must_use]
    pub const fn peak_main_stack(self) -> u64 {
        self.peak_main_stack
    }

    /// The widest single stack element the execution handles.
    #[must_use]
    pub const fn largest_element_bytes(self) -> u64 {
        self.largest_element_bytes
    }

    /// How many streaming-hash primitives the program invokes.
    #[must_use]
    pub const fn hash_operations(self) -> u64 {
        self.hash_operations
    }

    /// How many elliptic-curve checks the program invokes.
    #[must_use]
    pub const fn curve_operations(self) -> u64 {
        self.curve_operations
    }

    /// The script-path validation budget the curve checks charge.
    #[must_use]
    pub const fn validation_budget(self) -> u64 {
        self.validation_budget
    }

    /// The weight the witness contributes, at one unit per byte.
    #[must_use]
    pub const fn witness_weight(self) -> u64 {
        self.witness_weight
    }
}

/// Why a prototype program was not admitted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PrototypeProgramDefect {
    /// The reviewed contract is not the revision the prototype is
    /// stated against.
    ContractRevisionUnsupported {
        /// What the reviewed contract is.
        reviewed: u32,
    },
    /// The program names a primitive the reviewed census does not
    /// carry.
    PrimitiveUnsupported {
        /// The offending primitive.
        opcode: OpcodeId,
    },
    /// The program is longer than a typed program may be.
    ProgramNotExpressible,
    /// A literal the program pushes is wider than the target admits.
    LiteralNotExpressible,
    /// The abstract validator refused the emitted program's schedule.
    ScheduleRefused,
    /// The schedule does not end in exactly one canonically true item.
    ScheduleDoesNotEndInOneTrueItem,
    /// A state reached through a non-aborting failure survives, so a
    /// failed check could still leave a spendable stack.
    ScheduleAdmitsASurvivingFailure,
    /// A resource the program consumes exceeds a bound the reviewed
    /// contract states.
    ResourceBoundExceeded {
        /// Which bound.
        dimension: ResourceDimension,
        /// What the program needs.
        needed: u64,
        /// What the reviewed contract admits.
        maximum: u64,
    },
}

/// One constructor prototype, as bytes a target could execute.
///
/// Fields are private and there is no literal form: a prototype program
/// is what the constructor admitted, and a caller that could assemble
/// one field by field could assemble one whose schedule was never
/// validated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrototypeProgram {
    kind: PrototypeKind,
    status: PrototypeStatus,
    target: PrototypeTargetProjection,
    program: TapscriptProgram,
    initial_stack: AbstractStackState,
    relation: PrototypeProgramRelation,
    resources: PrototypeResourceProjection,
}

impl PrototypeProgram {
    /// Emits and admits the continuity prototype.
    ///
    /// Every check is a statement about the reviewed contracts: that the
    /// contract revision is the reviewed one, that every primitive the
    /// program names is in the reviewed census, that the abstract
    /// validator accepts the emitted program's schedule and finds it
    /// ending in exactly one true item with no surviving failure state,
    /// and that no resource the program consumes exceeds a bound the
    /// reviewed contract states.
    ///
    /// None of that is a node's verdict, which is why the admitted
    /// program carries the experimental status.
    ///
    /// # Errors
    ///
    /// [`PrototypeProgramDefect`] naming the first check that failed.
    pub fn continuity(
        target: &ReviewedElementsTapscriptDefinition,
    ) -> Result<Self, PrototypeProgramDefect> {
        let reviewed = target.definition().version();
        if reviewed != TargetContractVersion::V2 {
            return Err(PrototypeProgramDefect::ContractRevisionUnsupported {
                reviewed: reviewed.get(),
            });
        }

        let instructions = continuity_instructions(target)?;
        let program = TapscriptProgram::new(instructions)
            .map_err(|_| PrototypeProgramDefect::ProgramNotExpressible)?;

        for instruction in program.instructions() {
            if let TapscriptInstruction::Opcode(id) = instruction
                && !target.definition().opcodes().contains_key(id)
            {
                return Err(PrototypeProgramDefect::PrimitiveUnsupported { opcode: *id });
            }
        }

        let initial_stack = continuity_initial_stack();
        let outcome = validate_program(
            target,
            &program,
            &initial_stack,
            AbstractLimits::for_target(target),
        )
        .map_err(|_| PrototypeProgramDefect::ScheduleRefused)?;

        // The reviewed domain ends evaluation by requiring exactly one
        // true item, so a schedule leaving anything else is a program
        // no honest spend satisfies.
        let one_true_item: BTreeSet<AbstractStackState> =
            BTreeSet::from([AbstractStackState::from_main(vec![StackValueType::Bytes {
                minimum: 1,
                maximum: 1,
            }])]);
        if outcome.success() != &one_true_item {
            return Err(PrototypeProgramDefect::ScheduleDoesNotEndInOneTrueItem);
        }
        if !outcome.nonaborting_failure().is_empty() {
            return Err(PrototypeProgramDefect::ScheduleAdmitsASurvivingFailure);
        }

        let resources = project_resources(target, &program, &initial_stack);
        check_resource_bounds(target, resources)?;

        Ok(Self {
            kind: PrototypeKind::MetadataConstructorContinuity,
            status: PrototypeStatus::Experimental,
            target: PrototypeTargetProjection {
                contract_version: reviewed.get(),
                leaf_version: target.definition().leaf_version().get(),
            },
            program,
            initial_stack,
            relation: PrototypeProgramRelation::MetadataConstructorContinuity,
            resources,
        })
    }

    /// Which construction this is.
    #[must_use]
    pub const fn kind(&self) -> PrototypeKind {
        self.kind
    }

    /// How far it has been taken.
    #[must_use]
    pub const fn status(&self) -> PrototypeStatus {
        self.status
    }

    /// The reviewed target facts it was emitted under.
    #[must_use]
    pub const fn target(&self) -> PrototypeTargetProjection {
        self.target
    }

    /// The typed program.
    #[must_use]
    pub const fn program(&self) -> &TapscriptProgram {
        &self.program
    }

    /// The initial stack the schedule was validated against.
    #[must_use]
    pub const fn initial_stack(&self) -> &AbstractStackState {
        &self.initial_stack
    }

    /// What a passing execution would establish.
    #[must_use]
    pub const fn relation(&self) -> PrototypeProgramRelation {
        self.relation
    }

    /// What one execution costs.
    #[must_use]
    pub const fn resources(&self) -> PrototypeResourceProjection {
        self.resources
    }

    /// The exact target bytes.
    #[must_use]
    pub fn encode(&self, target: &ReviewedElementsTapscriptDefinition) -> Vec<u8> {
        self.program.encode(target)
    }
}

/// The tagged-hash prefix one tag contributes: its own digest, twice.
fn tag_prefix(tag: &str) -> Vec<u8> {
    let digest = sha256(tag.as_bytes());
    let mut prefix = Vec::with_capacity(2 * DIGEST_BYTES);
    prefix.extend_from_slice(&digest);
    prefix.extend_from_slice(&digest);
    prefix
}

/// The metadata leaf script's framing, either side of the metadata.
///
/// Read off the bytes the reviewed builder emits rather than restated,
/// so the program streams the preimage the leaf hash actually covers.
/// The probe metadata carries no zero byte, so the window search finds
/// the metadata and not the leaf script's own false.
fn leaf_framing(target: &ReviewedElementsTapscriptDefinition) -> Option<(Vec<u8>, Vec<u8>, usize)> {
    let probe: Vec<u8> = (0..METADATA_BYTES)
        .map(|index| u8::try_from(index % 255 + 1).unwrap_or(1))
        .collect();
    let script = metadata_leaf_script(target, &probe).ok()?;
    let at = script.windows(METADATA_BYTES).position(|w| w == probe)?;
    let head = script[..at].to_vec();
    let tail = script[at + METADATA_BYTES..].to_vec();
    Some((head, tail, script.len()))
}

/// One reviewed primitive, as an instruction.
const fn op(id: OpcodeId) -> TapscriptInstruction {
    TapscriptInstruction::Opcode(id)
}

/// A literal, as an instruction.
fn raw(
    target: &ReviewedElementsTapscriptDefinition,
    bytes: Vec<u8>,
) -> Result<TapscriptInstruction, PrototypeProgramDefect> {
    StackItem::new(target, bytes)
        .map(TapscriptInstruction::Push)
        .map_err(|_| PrototypeProgramDefect::LiteralNotExpressible)
}

/// A script-number literal, as an instruction.
fn number(
    target: &ReviewedElementsTapscriptDefinition,
    value: i64,
) -> Result<TapscriptInstruction, PrototypeProgramDefect> {
    StackItem::script_number(target, value)
        .map(TapscriptInstruction::Push)
        .map_err(|_| PrototypeProgramDefect::LiteralNotExpressible)
}

/// The constant literals the continuity program pushes.
///
/// Each is a preimage prefix the program streams into a tagged hash, or
/// the leaf script's tail, resolved to the exact bytes a spend carries
/// rather than to a placeholder of the right width.
struct ContinuityLiterals {
    /// Both tag digests of the leaf hash, the leaf version byte, the
    /// leaf script's compact-size length, and the push form the metadata
    /// arrives in.
    leaf_prefix: Vec<u8>,
    /// The leaf script's tail: the literal false and the verification
    /// that ends evaluation on it.
    leaf_tail: Vec<u8>,
    /// Both tag digests of the branch hash.
    branch_prefix: Vec<u8>,
    /// Both tag digests of the tweak hash, then the internal key.
    tweak_prefix: Vec<u8>,
}

impl ContinuityLiterals {
    /// Resolves every constant from the reviewed contract and the
    /// oracle's own tags and key.
    fn resolve(
        target: &ReviewedElementsTapscriptDefinition,
    ) -> Result<Self, PrototypeProgramDefect> {
        let (leaf_head, leaf_tail, leaf_script_bytes) =
            leaf_framing(target).ok_or(PrototypeProgramDefect::LiteralNotExpressible)?;

        let mut leaf_prefix = tag_prefix(TAP_LEAF_TAG);
        leaf_prefix.push(target.definition().leaf_version().get());
        leaf_prefix.extend_from_slice(&compact_size(leaf_script_bytes));
        leaf_prefix.extend_from_slice(&leaf_head);

        // The internal key is a published constant of the construction
        // rather than a witness (Guide-10 `rule:guide10:internal-key`).
        let mut tweak_prefix = tag_prefix(TAP_TWEAK_TAG);
        tweak_prefix.extend_from_slice(&UNSPENDABLE_INTERNAL_KEY);

        Ok(Self {
            leaf_prefix,
            leaf_tail,
            branch_prefix: tag_prefix(TAP_BRANCH_TAG),
            tweak_prefix,
        })
    }
}

/// The curve step: the internal key, then the tweak verification.
fn curve_step(
    target: &ReviewedElementsTapscriptDefinition,
) -> Result<Vec<TapscriptInstruction>, PrototypeProgramDefect> {
    Ok(vec![
        raw(target, UNSPENDABLE_INTERNAL_KEY.to_vec())?,
        op(OpcodeId::TweakVerify),
    ])
}

/// Requires an introspected program to be the one a witnessed
/// compressed key determines.
///
/// The introspection pushes a payload and a prefix. The prefix must be
/// the reviewed witness version, and the payload must be the witnessed
/// key's own coordinate, so a caller cannot offer a key the output does
/// not carry.
///
/// `lift` is how the witnessed key is brought back within reach once the
/// prefix is consumed, and it differs between the two uses because the
/// two arrive with different things beneath them: the predecessor half
/// has the key immediately below, and the output binding has it one
/// deeper.
fn bind_introspected_program(
    target: &ReviewedElementsTapscriptDefinition,
    lift: OpcodeId,
) -> Result<Vec<TapscriptInstruction>, PrototypeProgramDefect> {
    Ok(vec![
        number(target, 1)?,
        op(OpcodeId::EqualVerify),
        op(lift),
        op(OpcodeId::Duplicate),
        number(target, 1)?,
        number(target, i64::try_from(FIELD_ELEMENT_BYTES).unwrap_or(32))?,
        op(OpcodeId::Substring),
        op(OpcodeId::Rotate),
        op(OpcodeId::EqualVerify),
    ])
}

/// One constructor derivation: the metadata leaf hash, the branch hash
/// against the static root, and the tweak.
///
/// `retain_root` decides whether the static root survives the branch
/// hash. The predecessor half hashes it from a copy so the authenticated
/// instance outlives the whole second derivation; the successor half
/// consumes the one it was handed, which is what binds the two
/// constructors to the same value by construction rather than by
/// comparing two witnesses (Guide-10 `rule:guide10:static-root`).
fn derive_constructor(
    target: &ReviewedElementsTapscriptDefinition,
    literals: &ContinuityLiterals,
    retain_root: bool,
) -> Result<Vec<TapscriptInstruction>, PrototypeProgramDefect> {
    let mut out = vec![
        op(OpcodeId::Swap),
        raw(target, literals.leaf_prefix.clone())?,
        op(OpcodeId::Sha256Initialize),
        op(OpcodeId::Swap),
        op(OpcodeId::Sha256Update),
        raw(target, literals.leaf_tail.clone())?,
        op(OpcodeId::Sha256Finalize),
        raw(target, literals.branch_prefix.clone())?,
        op(OpcodeId::Sha256Initialize),
        op(OpcodeId::Swap),
        op(OpcodeId::Sha256Update),
    ];
    if retain_root {
        out.extend([
            op(OpcodeId::Rotate),
            op(OpcodeId::Duplicate),
            op(OpcodeId::Rotate),
        ]);
    }
    out.extend([op(OpcodeId::Swap), op(OpcodeId::Sha256Finalize)]);
    out.push(raw(target, literals.tweak_prefix.clone())?);
    out.extend([
        op(OpcodeId::Sha256Initialize),
        op(OpcodeId::Swap),
        op(OpcodeId::Sha256Finalize),
    ]);
    Ok(out)
}

/// The whole continuity program, with every literal resolved.
///
/// The instruction sequence is the one the schedules established, in the
/// same order: the predecessor half, its curve check, the successor
/// half, the successor output binding, its curve check, and the final
/// truth value. What differs is that every literal is the value a spend
/// carries.
fn continuity_instructions(
    target: &ReviewedElementsTapscriptDefinition,
) -> Result<Vec<TapscriptInstruction>, PrototypeProgramDefect> {
    let literals = ContinuityLiterals::resolve(target)?;

    // The predecessor half: bind the witnessed output key to the program
    // the consumed input carries, then derive that program's own tweak.
    let mut out = vec![
        op(OpcodeId::PushCurrentInputIndex),
        op(OpcodeId::InspectInputScriptPubKey),
    ];
    out.extend(bind_introspected_program(target, OpcodeId::Swap)?);
    out.extend(derive_constructor(target, &literals, true)?);
    // Place the curve step's operands, the retained root beneath them.
    out.extend([op(OpcodeId::Rotate), op(OpcodeId::Swap)]);
    out.extend(curve_step(target)?);

    // The successor half, consuming the one authenticated root.
    out.extend(derive_constructor(target, &literals, false)?);

    // The successor output binding, read at one stated role rather than
    // searched for among the outputs
    // (Guide-10 `rule:guide10:successor-constructor`).
    out.push(number(target, 0)?);
    out.push(op(OpcodeId::InspectOutputScriptPubKey));
    out.extend(bind_introspected_program(target, OpcodeId::Rotate)?);
    out.push(op(OpcodeId::Swap));
    out.extend(curve_step(target)?);

    // The final truth value a standalone spend needs.
    out.push(raw(target, vec![1])?);

    Ok(out)
}

/// The witness the continuity program consumes, deepest item first.
///
/// The order is the consumption order reversed, and it is forced rather
/// than chosen: no reviewed primitive reads below the third item, so a
/// witness is reachable only while fewer than three computed values sit
/// above it, and the one static root must outlive both constructor
/// derivations (Guide-10 `rule:guide10:static-root`).
fn continuity_initial_stack() -> AbstractStackState {
    AbstractStackState::from_main(vec![
        StackValueType::Encoded(EncodingClass::CompressedPublicKey),
        metadata_item(),
        digest_item(),
        metadata_item(),
        StackValueType::Encoded(EncodingClass::CompressedPublicKey),
    ])
}

/// One metadata object, as a witness item.
const fn metadata_item() -> StackValueType {
    StackValueType::Bytes {
        minimum: METADATA_BYTES,
        maximum: METADATA_BYTES,
    }
}

/// One 32-byte digest, as a witness item.
const fn digest_item() -> StackValueType {
    StackValueType::Bytes {
        minimum: DIGEST_BYTES,
        maximum: DIGEST_BYTES,
    }
}

/// How many nodes the prototype's control path carries.
///
/// The tree is the metadata leaf beside the static operation subtree, so
/// a spend of a leaf inside that subtree authenticates one sibling at
/// each level. The prototype's subtree is one leaf, so the path is the
/// metadata leaf alone.
const CONTROL_PATH_NODES: u64 = 1;

/// A compressed public key's width, as a witness item carries it.
const COMPRESSED_KEY_BYTES: usize = 33;

/// Measures the emitted program.
fn project_resources(
    target: &ReviewedElementsTapscriptDefinition,
    program: &TapscriptProgram,
    initial: &AbstractStackState,
) -> PrototypeResourceProjection {
    let script = program.encode(target);
    let script_bytes = u64::try_from(script.len()).unwrap_or(u64::MAX);

    let mut hash_operations = 0_u64;
    let mut curve_operations = 0_u64;
    for instruction in program.instructions() {
        if let TapscriptInstruction::Opcode(id) = instruction {
            match id {
                OpcodeId::Sha256Initialize | OpcodeId::Sha256Update | OpcodeId::Sha256Finalize => {
                    hash_operations += 1;
                }
                OpcodeId::TweakVerify | OpcodeId::EcMulScalarVerify => curve_operations += 1,
                _ => {}
            }
        }
    }

    let projected: BTreeMap<ResourceDimension, u64> = resource_projection(target, program);
    let validation_budget = projected
        .get(&ResourceDimension::ValidationBudget)
        .copied()
        .unwrap_or(0);

    // Every width the execution handles: the witness items, every
    // literal the program pushes, and the streaming-hash context the
    // program carries on the stack between the initialize and the
    // finalize.
    let mut widest = 0_u64;
    for value in initial.main() {
        widest = widest.max(settled_width(value));
    }
    for instruction in program.instructions() {
        if let TapscriptInstruction::Push(item) = instruction {
            widest = widest.max(u64::try_from(item.bytes().len()).unwrap_or(u64::MAX));
        }
    }
    widest = widest.max(sha256_context_bytes(target));

    let control_block_bytes =
        u64::try_from(1 + FIELD_ELEMENT_BYTES).unwrap_or(0) + CONTROL_PATH_NODES * 32;
    let witness_bytes = serialized_witness_bytes(initial, script_bytes, control_block_bytes);

    PrototypeResourceProjection {
        script_bytes,
        witness_bytes,
        peak_main_stack: peak_main_stack(target, program, initial),
        largest_element_bytes: widest,
        hash_operations,
        curve_operations,
        validation_budget,
        // The reviewed domain weighs a witness byte as one unit.
        witness_weight: witness_bytes,
    }
}

/// The width a settled abstract type fixes, or zero where it fixes none.
fn settled_width(value: &StackValueType) -> u64 {
    match value {
        StackValueType::Bytes { minimum, maximum } if minimum == maximum => {
            u64::try_from(*minimum).unwrap_or(0)
        }
        StackValueType::Encoded(EncodingClass::CompressedPublicKey) => {
            u64::try_from(COMPRESSED_KEY_BYTES).unwrap_or(0)
        }
        _ => 0,
    }
}

/// The streaming-hash context's width, from the reviewed encoding.
fn sha256_context_bytes(target: &ReviewedElementsTapscriptDefinition) -> u64 {
    target
        .definition()
        .encodings()
        .get(&EncodingClass::Sha256Context)
        .map_or(0, |spec| match spec.payload() {
            PayloadWidth::Exact(bytes) => u64::try_from(bytes.get()).unwrap_or(0),
            PayloadWidth::Bounded { maximum, .. } => u64::try_from(maximum.get()).unwrap_or(0),
            PayloadWidth::Absent => 0,
        })
}

/// The witness's serialized width.
///
/// Each item carries its own compact-size length, and the whole stack
/// carries a count. The script and the control block are two more items.
fn serialized_witness_bytes(
    initial: &AbstractStackState,
    script_bytes: u64,
    control_block_bytes: u64,
) -> u64 {
    let mut total = u64::try_from(compact_size(initial.main().len() + 2).len()).unwrap_or(1);
    for value in initial.main() {
        let width = settled_width(value);
        total += width
            + u64::try_from(compact_size(usize::try_from(width).unwrap_or(0)).len()).unwrap_or(1);
    }
    for width in [script_bytes, control_block_bytes] {
        total += width
            + u64::try_from(compact_size(usize::try_from(width).unwrap_or(0)).len()).unwrap_or(1);
    }
    total
}

/// The greatest main-stack depth any reachable state holds.
///
/// Measured by validating each prefix of the program and taking the
/// deepest state any of them reaches, so the number is the validator's
/// rather than a hand count of pushes and pops.
fn peak_main_stack(
    target: &ReviewedElementsTapscriptDefinition,
    program: &TapscriptProgram,
    initial: &AbstractStackState,
) -> u64 {
    let mut peak = u64::try_from(initial.main().len()).unwrap_or(0);
    for length in 1..=program.len() {
        let Ok(prefix) = TapscriptProgram::new(program.instructions()[..length].to_vec()) else {
            continue;
        };
        let Ok(outcome) =
            validate_program(target, &prefix, initial, AbstractLimits::for_target(target))
        else {
            continue;
        };
        for state in outcome
            .success()
            .iter()
            .chain(outcome.nonaborting_failure())
        {
            peak = peak.max(u64::try_from(state.main().len()).unwrap_or(0));
        }
    }
    peak
}

/// Compares each measured dimension against the bound the reviewed
/// contract states for it.
///
/// A dimension the contract leaves unbounded is not checked, and a
/// dimension the projection does not measure is not invented.
fn check_resource_bounds(
    target: &ReviewedElementsTapscriptDefinition,
    measured: PrototypeResourceProjection,
) -> Result<(), PrototypeProgramDefect> {
    let bounds = target.definition().resources().consensus().bounds();
    for (dimension, needed) in [
        (ResourceDimension::ScriptBytes, measured.script_bytes),
        (ResourceDimension::WitnessBytes, measured.witness_bytes),
        (ResourceDimension::PeakStackItems, measured.peak_main_stack),
        (
            ResourceDimension::StackElementBytes,
            measured.largest_element_bytes,
        ),
        (ResourceDimension::ControlPathDepth, CONTROL_PATH_NODES),
    ] {
        let Some(maximum) = bounds.get(&dimension).and_then(|bound| bound.maximum()) else {
            continue;
        };
        if needed > maximum {
            return Err(PrototypeProgramDefect::ResourceBoundExceeded {
                dimension,
                needed,
                maximum,
            });
        }
    }
    Ok(())
}
