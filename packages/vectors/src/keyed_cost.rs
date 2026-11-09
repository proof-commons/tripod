//! Measurement (d): static predictions, serialized carriers and admitted node data.
//!
//! A transaction weight is never a clause cost. Sponsor signatures, output scripts
//! and input counts differ across archives. Native stack peaks and validation
//! budget remain absent; abstract projections occupy separate fields. The birth
//! archive predates wire JSON and is read only through its admitted journal bridge.
//! No new observation or measurement admission is made by constructing this packet.

use serde::{Deserialize, Serialize};
use tapscript::{
    AbstractLimits, AbstractStackState, TapscriptProgram, program_stack_profile,
    resource_projection, validate_program,
};
use target_elements::{
    ResourceDimension, ReviewedElementsTapscriptDefinition, reviewed_elements_tapscript,
};
use target_elements_conformance::executor::OperationStep;
use target_elements_conformance::protocol::{
    NativeOperationResponse, NativeResourceObservation, OperationSubject,
};
use transaction::bytes::{InputWitness, TargetTransaction, compact_size};

use crate::keyed_birth_native::BirthNativeRefusal;

/// Packet construction refuses a missing admission, incompatible carrier or report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CostRefusal {
    /// An underlying run of record did not admit.
    Admission,
    /// Typed construction or byte decoding failed.
    Construction,
    /// A resource report changed or asserted an unavailable dimension.
    ResourceReport,
}

/// Static program facts; none are native execution measurements.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaticCost {
    /// Exact program being projected, including standalone final truth.
    pub script: Vec<u8>,
    /// Typed instruction count, including pushes.
    pub instructions: usize,
    /// The typed engine's script length projection.
    pub script_bytes: u64,
    /// Sum of reviewed primitive operation charges.
    pub operation_cost: u64,
    /// Static validation charge, not a node budget observation.
    pub validation_budget: u64,
    /// Abstract peak main depth from every prefix.
    pub peak_main: u64,
    /// Abstract peak alternate depth from every prefix.
    pub peak_alternate: u64,
}

/// Project one exact program through the typed engine from empty stacks.
/// # Errors
/// Refuses a typed program without a successful abstract path.
pub fn static_cost(
    target: &ReviewedElementsTapscriptDefinition,
    program: &TapscriptProgram,
) -> Result<StaticCost, CostRefusal> {
    let _span = tracing::debug_span!("keyed_static_cost").entered();
    let initial = AbstractStackState::from_main(vec![]);
    let limits = AbstractLimits::for_target(target);
    let result = validate_program(target, program, &initial, limits)
        .map_err(|_| CostRefusal::Construction)?;
    if result.success().is_empty() {
        return Err(CostRefusal::Construction);
    }
    let profile = program_stack_profile(target, program, &initial, limits);
    let projection = resource_projection(target, program);
    Ok(StaticCost {
        script: program.encode(target),
        instructions: program.len(),
        script_bytes: projection[&ResourceDimension::ScriptBytes],
        operation_cost: projection[&ResourceDimension::OperationCost],
        validation_budget: projection[&ResourceDimension::ValidationBudget],
        peak_main: profile.peak_main(),
        peak_alternate: profile.peak_alternate(),
    })
}

fn standalone(
    target: &ReviewedElementsTapscriptDefinition,
    fragment: &TapscriptProgram,
) -> Result<TapscriptProgram, CostRefusal> {
    let mut instructions = fragment.instructions().to_vec();
    instructions.extend_from_slice(
        tapscript::pattern::final_truth_fragment(target)
            .map_err(|_| CostRefusal::Construction)?
            .instructions(),
    );
    TapscriptProgram::new(instructions).map_err(|_| CostRefusal::Construction)
}

/// Serialized witness accounting for one input, with Elements' three empty fields.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessCost {
    /// Exact item lengths; script and control are included, sponsor is separate.
    pub item_bytes: Vec<usize>,
    /// Count prefix plus every item's length prefix and payload.
    pub script_witness_bytes: usize,
    /// Full input witness: script witness plus empty issuance proofs and peg-in stack.
    pub input_witness_bytes: usize,
}

/// Count the exact script-witness serialization, without treating payload as framing.
#[must_use]
pub fn witness_cost(witness: &InputWitness) -> WitnessCost {
    let item_bytes: Vec<_> = witness.stack().iter().map(Vec::len).collect();
    let script_witness_bytes = compact_size(item_bytes.len() as u64).len()
        + item_bytes
            .iter()
            .map(|n| compact_size(*n as u64).len() + n)
            .sum::<usize>();
    WitnessCost {
        item_bytes,
        script_witness_bytes,
        input_witness_bytes: script_witness_bytes + 3,
    }
}

/// One exact submitted carrier and its independent node-read transaction weight.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CarrierCost {
    /// Archive provenance, not protocol identity.
    pub archive: String,
    /// Frozen request's case identifier.
    pub step: String,
    /// Exact submitted bytes, needed to reproduce every serialized figure.
    pub transaction: Vec<u8>,
    /// Input witnesses kept positional, including the sponsor.
    pub witnesses: Vec<WitnessCost>,
    /// Bytes without the witness section, including the Elements flag byte.
    pub stripped_bytes: usize,
    /// Complete witness serialization, including output proofs.
    pub witness_bytes: usize,
    /// Four times stripped bytes plus witness bytes; standard weight, not discount vsize.
    pub serialized_weight: u64,
    /// Node-reported weight for these exact bytes; null stays absent.
    pub node_weight: Option<u64>,
    /// Equal only where the node supplied the same dimension; mismatch is retained.
    pub weight_matches: Option<bool>,
    /// Raw report, including explicitly null interpreter dimensions.
    pub reported: NativeResourceObservation,
    /// Exact mined readback was present, independently checked by archive admission.
    pub mined: bool,
}

/// Freeze serialized figures and retain a weight mismatch rather than hiding it.
/// # Errors
/// Refuses anything other than a decodable submission.
pub fn carrier_cost(
    archive: &str,
    step: &OperationStep,
    response: &NativeOperationResponse,
) -> Result<CarrierCost, CostRefusal> {
    let _span =
        tracing::debug_span!("keyed_carrier_cost", archive, step = %step.case().step).entered();
    let OperationSubject::Submission(request) = step.subject() else {
        return Err(CostRefusal::Construction);
    };
    let tx = TargetTransaction::decode(&request.transaction_bytes)
        .map_err(|_| CostRefusal::Construction)?;
    let stripped_bytes = tx.encode_without_witness().len();
    let witness_bytes = request.transaction_bytes.len() - stripped_bytes;
    let serialized_weight = (4 * stripped_bytes + witness_bytes) as u64;
    let node_weight = response.resources.transaction_weight;
    Ok(CarrierCost {
        archive: archive.into(),
        step: step.case().step.clone(),
        transaction: request.transaction_bytes.clone(),
        witnesses: tx.witnesses().iter().map(witness_cost).collect(),
        stripped_bytes,
        witness_bytes,
        serialized_weight,
        node_weight,
        weight_matches: node_weight.map(|weight| weight == serialized_weight),
        reported: response.resources,
        mined: response.mined_readback.is_some(),
    })
}

/// Validate a restated report against its immutable admitted source, not against a prediction.
/// # Errors
/// Refuses changed dimensions, including an invented native peak or budget.
pub fn validate_resource_report(
    source: &CarrierCost,
    report: NativeResourceObservation,
) -> Result<(), CostRefusal> {
    let _span = tracing::debug_span!("validate_keyed_resource_report").entered();
    let matches = source.reported == report;
    tracing::debug!(matches, "resource report comparison");
    if matches {
        Ok(())
    } else {
        Err(CostRefusal::ResourceReport)
    }
}

/// Exact per-input script/control data versus fixed public constructor data.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstructorCost {
    /// Key push (33 bytes) and drop (one byte), repeated in every script-path input.
    pub key_commitment_script_bytes: usize,
    /// Full leaf repeated per input, not a one-time transaction overhead.
    pub repeated_leaf_bytes: usize,
    /// Control repeated per input.
    pub repeated_control_bytes: usize,
    /// Merkle sibling count in the control recipe.
    pub control_depth: usize,
    /// Fixed public descriptor, retained in full rather than assigned a wire weight.
    pub descriptor: crate::keyed_constructor::KeyedConstructorDescriptor,
}

/// Exact linked single-leaf carrier, including witness framing and control depth.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptPathCost {
    /// Program identity within this comparison packet.
    pub name: String,
    /// Exact leaf bytes, not a size-only placeholder.
    pub script: Vec<u8>,
    /// Reconstructed output scriptPubKey.
    pub output_program: Vec<u8>,
    /// Exact authenticated single-leaf control block.
    pub control: Vec<u8>,
    /// Number of sibling hashes, not the control's byte length.
    pub control_depth: usize,
    /// Per-input script/control serialization from an empty initial stack.
    pub witness: WitnessCost,
}

/// Packet of known dimensions; isolated continuation has no admitted native carrier yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedCostPacket {
    /// Complete admitted target/deployment/executable/policy bindings, in archive order.
    pub bindings: Vec<crate::keyed_birth::Wave1TargetPacket>,
    /// Isolated birth plus final truth.
    pub birth: StaticCost,
    /// Isolated continuation plus final truth.
    pub continuation: StaticCost,
    /// Birth plus continuation plus truth, matching the key-free archive.
    pub keyfree_composed: StaticCost,
    /// Complete keyed leaf, matching the admitted descriptor.
    pub keyed: StaticCost,
    /// Public fixed data and repeated script-path quantities.
    pub constructor: ConstructorCost,
    /// Locally linked bytes for all four static programs.
    pub script_paths: Vec<ScriptPathCost>,
    /// Every subject submission in the three admitted archives, including negatives.
    pub carriers: Vec<CarrierCost>,
    /// Explicitly unavailable observations; zero is never substituted.
    pub absent: Vec<String>,
}

/// Build a comparison packet from all three admitted archives without executing a node.
/// # Errors
/// Refuses failed archive admission, typed construction or mismatched exact leaf bytes.
pub fn cost_packet() -> Result<KeyedCostPacket, CostRefusal> {
    let _span = tracing::info_span!("keyed_cost_packet").entered();
    let target = reviewed_elements_tapscript().map_err(|_| CostRefusal::Construction)?;
    let birth =
        crate::keyed_birth_corpus::admit_run_of_record().map_err(|_| CostRefusal::Admission)?;
    let continuation = crate::keyed_continuation_corpus::admit_run_of_record()
        .map_err(|_| CostRefusal::Admission)?;
    let keyed = crate::keyed_corpus::admit_run_of_record().map_err(|_| CostRefusal::Admission)?;
    let descriptor = keyed.planner().recipes().descriptors()[0].clone();
    let birth_program = standalone(
        &target,
        &tapscript::born_first_fragment(&target).map_err(|_| CostRefusal::Construction)?,
    )?;
    let continuation_program = standalone(
        &target,
        &tapscript::keyed_pattern::continue_first_fragment(&target)
            .map_err(|_| CostRefusal::Construction)?,
    )?;
    let mut instructions = tapscript::born_first_fragment(&target)
        .map_err(|_| CostRefusal::Construction)?
        .instructions()
        .to_vec();
    instructions.extend_from_slice(continuation_program.instructions());
    let keyfree = TapscriptProgram::new(instructions).map_err(|_| CostRefusal::Construction)?;
    let keyed_program = tapscript::keyed_constructor::keyed_leaf_program(
        &target,
        tapscript::keyed_constructor::KeyedSemanticKey::new(descriptor.semantic_key),
    )
    .map_err(|_| CostRefusal::Construction)?;
    if keyed_program.encode(&target) != descriptor.leaf_bytes {
        return Err(CostRefusal::Construction);
    }
    let mut carriers = vec![];
    for row in birth.planner().records() {
        carriers.push(carrier_cost(
            "keyed-birth-run-of-record",
            &row.submission.0,
            &row.submission.1,
        )?);
    }
    for row in continuation.planner().records() {
        carriers.push(carrier_cost(
            "keyed-continuation-run-of-record",
            &row.submission.0,
            &row.submission.1,
        )?);
    }
    for row in keyed.planner().records() {
        carriers.push(carrier_cost(
            "keyed-run-of-record",
            &row.submission.0,
            &row.submission.1,
        )?);
    }
    let script_paths = [
        ("birth", &birth_program),
        ("continuation", &continuation_program),
        ("keyfree-composed", &keyfree),
        ("keyed", &keyed_program),
    ]
    .into_iter()
    .map(|(name, program)| script_path_cost(&target, name, program))
    .collect::<Result<Vec<_>, CostRefusal>>()?;
    let packet = KeyedCostPacket {
        bindings: vec![
            birth.manifest().packet().clone(),
            continuation.manifest().packet().clone(),
            keyed.packet().clone(),
        ],
        birth: static_cost(&target, &birth_program)?,
        continuation: static_cost(&target, &continuation_program)?,
        keyfree_composed: static_cost(&target, &keyfree)?,
        keyed: static_cost(&target, &keyed_program)?,
        constructor: ConstructorCost {
            key_commitment_script_bytes: 34,
            repeated_leaf_bytes: descriptor.leaf_bytes.len(),
            repeated_control_bytes: descriptor.control.bytes.len(),
            control_depth: (descriptor.control.bytes.len() - 33) / 32,
            descriptor,
        },
        script_paths,
        carriers,
        absent: [
            "isolated continuation node-read transaction weight (staged)",
            "native main-stack peak",
            "native alternate-stack peak",
            "native maximum element bytes",
            "native validation budget",
            "native opcode timing",
        ]
        .map(str::to_owned)
        .to_vec(),
    };
    tracing::info!(
        carriers = packet.carriers.len(),
        "cost packet built without new observations"
    );
    Ok(packet)
}

fn script_path_cost(
    target: &ReviewedElementsTapscriptDefinition,
    name: &str,
    program: &TapscriptProgram,
) -> Result<ScriptPathCost, CostRefusal> {
    use target_elements_conformance::constructor::{
        UNSPENDABLE_INTERNAL_KEY,
        tree::{FixtureTapTree, construct},
    };
    let script = program.encode(target);
    let tree = FixtureTapTree::Leaf {
        version: target.definition().leaf_version().get(),
        script: script.clone(),
    };
    let linked = construct(&UNSPENDABLE_INTERNAL_KEY, &tree, &tree)
        .map_err(|_| CostRefusal::Construction)?;
    let control = linked.control_block().to_vec();
    let witness = witness_cost(&InputWitness::new(vec![script.clone(), control.clone()]));
    Ok(ScriptPathCost {
        name: name.into(),
        script,
        output_program: linked.output_program().to_vec(),
        control_depth: (control.len() - 33) / 32,
        control,
        witness,
    })
}

impl From<BirthNativeRefusal> for CostRefusal {
    fn from(_: BirthNativeRefusal) -> Self {
        Self::Construction
    }
}
