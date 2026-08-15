//! Cross-subcontract welds.
//!
//! # Why the contract repeats itself at all
//!
//! Several target facts are carried in more than one typed view. An
//! empty signature's behavior appears both as a failure effect on
//! `CheckSig` and as a field of the signature primitive contract; the
//! per-check validation budget appears both as an opcode resource cost
//! and as a number in the same contract; issuance fields appear as a
//! census, as an introspection result, and as two capability rows. The
//! repetition is deliberate — each view answers a different question,
//! and collapsing them would force a consumer to reconstruct one from
//! another.
//!
//! Repetition is only defensible if something checks that the copies
//! agree. Local shape checks do not: they establish that each view is
//! individually well-formed, which is exactly the property a
//! contradictory contract still has. These welds compare the views
//! against each other, so that a definition claiming an empty
//! signature aborts in one place and pushes a false in another is
//! refused rather than validated.
//!
//! # Welds are not the reviewed comparison
//!
//! A welded contract is internally consistent, not first-party. A
//! caller can move both sides of a fact together and still produce a
//! coherent contract that is not this project's; that distinction is
//! the reviewed wrapper's job, not this module's.

use std::collections::BTreeSet;

use crate::capability::{ElementsCapability, StaticCapabilityStatus};
use crate::confidential::{
    ConfidentialCapabilityState, ConfidentialValueCapability, IssuanceField,
};
use crate::definition::TargetDefinition;
use crate::encoding::EncodingClass;
use crate::error::TargetError;
use crate::evidence::TargetEvidenceRequirementId;
use crate::opcode::{FailureCause, FailureOutcome, OpcodeId, StackValueType};
use crate::push::PushPayloadPredicate;
use crate::resource::{ResourceBound, ResourceDimension};
use crate::success::{SuccessCondition, SuccessContract};

/// The signature primitives whose empty-signature result is branchable.
const BRANCHING_SIGNATURE_OPCODES: &[OpcodeId] = &[OpcodeId::CheckSig, OpcodeId::CheckSigFromStack];

/// The signature primitives that leave no branchable result.
const VERIFYING_SIGNATURE_OPCODES: &[OpcodeId] =
    &[OpcodeId::CheckSigVerify, OpcodeId::CheckSigFromStackVerify];

/// Runs every cross-subcontract weld.
pub fn validate_welds(definition: &TargetDefinition, errors: &mut Vec<TargetError>) {
    weld_pushes(definition, errors);
    weld_signature(definition, errors);
    weld_timelock(definition, errors);
    weld_issuance(definition, errors);
    weld_confidential(definition, errors);
    weld_resources(definition, errors);
    weld_evidence(definition, errors);
}

/// The outcome a primitive declares for one failure cause.
fn outcome_of(
    definition: &TargetDefinition,
    opcode: OpcodeId,
    cause: FailureCause,
) -> Option<FailureOutcome> {
    definition
        .opcodes()
        .get(&opcode)?
        .stack()
        .failure()
        .effects()
        .iter()
        .find(|effect| effect.cause() == cause)
        .map(|effect| effect.outcome())
}

/// The encoding classes one primitive names among its operands.
fn operand_classes(definition: &TargetDefinition, opcode: OpcodeId) -> BTreeSet<EncodingClass> {
    definition
        .opcodes()
        .get(&opcode)
        .into_iter()
        .flat_map(|spec| spec.stack().operands().iter())
        .flat_map(crate::operand::OperandContract::named_encodings)
        .collect()
}

/// The results one primitive pushes under one condition.
fn results_under(
    definition: &TargetDefinition,
    opcode: OpcodeId,
    condition: SuccessCondition,
) -> Option<Vec<StackValueType>> {
    definition
        .opcodes()
        .get(&opcode)?
        .stack()
        .success()
        .cases()
        .into_iter()
        .find(|case| case.condition() == condition)
        .map(|case| case.effect().results().to_vec())
}

/// The status a capability row carries.
fn status_of(
    definition: &TargetDefinition,
    capability: ElementsCapability,
) -> Option<StaticCapabilityStatus> {
    definition
        .capabilities()
        .get(&capability)
        .map(crate::capability::CapabilityContract::status)
}

/// The push forms, the minimal-form rule, the literal bound, the
/// primitive bytes, and the evidence link must agree.
///
/// The literal bound is the sharpest of these. It is stated as a push
/// rule and again as a consensus resource bound, and the two are the
/// same target number: a contract that let them drift would admit a
/// program whose pushes the resource contract says are too wide.
fn weld_pushes(definition: &TargetDefinition, errors: &mut Vec<TargetError>) {
    let pushes = definition.pushes();
    let declared_bound = definition
        .resources()
        .consensus()
        .bounds()
        .get(&ResourceDimension::StackElementBytes)
        .copied()
        .and_then(ResourceBound::maximum);
    let mut disagrees = pushes.evidence().is_empty()
        || declared_bound != u64::try_from(pushes.maximum_payload_bytes()).ok();

    // Every step of the minimal-form rule must name a declared form
    // that can actually carry the payloads the step matches. A rule
    // naming a form that refuses them would leave those payloads with
    // no encoding at all.
    for step in pushes.minimality() {
        let Some(spec) = pushes.form(step.form()) else {
            disagrees = true;
            continue;
        };
        let widest = match step.predicate() {
            PushPayloadPredicate::ExactWidth(width) | PushPayloadPredicate::WidthAtMost(width) => {
                width
            }
            PushPayloadPredicate::SingleByteInRange { .. } => 1,
        };
        if !spec.admits_width(widest) {
            disagrees = true;
        }
    }

    // A push opcode and a primitive opcode are read from the same byte
    // position, so a byte claimed by both would decode two ways.
    let occupied = pushes.occupied_opcodes();
    for spec in definition.opcodes().values() {
        if occupied.contains(&spec.code()) {
            disagrees = true;
        }
    }

    let named = pushes.evidence().iter().chain(
        pushes
            .forms()
            .values()
            .flat_map(crate::push::PushFormSpec::evidence),
    );
    for id in named {
        if !definition.evidence_requirements().contains_key(id) {
            disagrees = true;
        }
    }

    if disagrees {
        errors.push(TargetError::PushContractMismatch);
    }
}

/// Signature opcodes, the signature primitive contract, the per-check
/// budget, the operand encodings, and the evidence link must agree.
fn weld_signature(definition: &TargetDefinition, errors: &mut Vec<TargetError>) {
    let signature = definition.authorization().signature();
    let mut disagrees = false;

    for opcode in BRANCHING_SIGNATURE_OPCODES {
        // The branchable forms are the ones the contract's
        // empty-signature field describes.
        if outcome_of(definition, *opcode, FailureCause::EmptySignature)
            != Some(signature.empty_signature())
        {
            disagrees = true;
        }
    }

    for opcode in VERIFYING_SIGNATURE_OPCODES {
        // The verifying forms turn the branchable result into an
        // abort. A contract that let them push a false would be
        // describing a primitive that does not exist.
        if outcome_of(definition, *opcode, FailureCause::EmptySignature)
            != Some(FailureOutcome::AbortEvaluation)
        {
            disagrees = true;
        }
    }

    for opcode in BRANCHING_SIGNATURE_OPCODES
        .iter()
        .chain(VERIFYING_SIGNATURE_OPCODES)
    {
        if outcome_of(definition, *opcode, FailureCause::InvalidSignature)
            != Some(signature.invalid_signature())
        {
            disagrees = true;
        }

        let Some(spec) = definition.opcodes().get(opcode) else {
            disagrees = true;
            continue;
        };

        // One number, two views: the budget a check can charge is a
        // resource cost and a signature-contract field alike.
        if spec.resources().validation_budget() != signature.budget_per_check() {
            disagrees = true;
        }

        let operands = operand_classes(definition, *opcode);
        if !operands.contains(&signature.signature_encoding())
            || !operands.contains(&signature.public_key_encoding())
        {
            disagrees = true;
        }

        if spec.evidence().is_disjoint(signature.evidence()) {
            disagrees = true;
        }
    }

    if signature.evidence().is_empty() {
        disagrees = true;
    }

    if disagrees {
        errors.push(TargetError::SignatureContractMismatch);
    }
}

/// The timelock primitive, the relative-timelock contract, the version
/// prerequisite, and the evidence link must agree.
fn weld_timelock(definition: &TargetDefinition, errors: &mut Vec<TargetError>) {
    let timelock = definition.authorization().relative_timelock();

    let mut disagrees = outcome_of(
        definition,
        OpcodeId::CheckSequenceVerify,
        FailureCause::UnsatisfiedTimelock,
    ) != Some(timelock.unsatisfied());

    // The check inspects its operand and leaves it there. Any other
    // success shape would contradict the resource row and would
    // mis-schedule every program that uses a lock.
    match definition
        .opcodes()
        .get(&OpcodeId::CheckSequenceVerify)
        .map(|spec| spec.stack().success())
    {
        Some(SuccessContract::RetainsOperands { results }) if results.is_empty() => {}
        _ => disagrees = true,
    }

    // The operand is read at the lock-time width, not the ordinary
    // script-number width. The two differ by one byte, and that byte is
    // the whole of the disable flag: an operand typed at four bytes says
    // the target refuses a value it in fact treats as "no lock at all".
    let lock_time_operand = &[crate::operand::OperandContract::Exact(
        StackValueType::Encoded(EncodingClass::LockTimeScriptNumber),
    )][..];
    if definition
        .opcodes()
        .get(&OpcodeId::CheckSequenceVerify)
        .map(|spec| spec.stack().operands())
        != Some(lock_time_operand)
    {
        disagrees = true;
    }

    // Below the stated version the lock is not merely satisfied, it is
    // not enforced at all, so a program relying on it must be able to
    // constrain the version. That is only possible if the timelock
    // capability requires version introspection.
    if timelock.minimum_transaction_version() == 0 {
        disagrees = true;
    }
    match definition
        .capabilities()
        .get(&ElementsCapability::RelativeTimelock)
    {
        Some(contract) => {
            if !contract
                .prerequisites()
                .contains(&ElementsCapability::TransactionVersionInspection)
                || !contract.opcodes().contains(&OpcodeId::CheckSequenceVerify)
                || !contract.encodings().contains(&EncodingClass::Sequence)
            {
                disagrees = true;
            }
        }
        None => disagrees = true,
    }

    if timelock.modes().is_empty() || timelock.evidence().is_empty() {
        disagrees = true;
    }

    if disagrees {
        errors.push(TargetError::TimelockContractMismatch);
    }
}

/// The issuance census, the introspection result, the null marker, the
/// outpoint flag, and the issuance capabilities must agree.
fn weld_issuance(definition: &TargetDefinition, errors: &mut Vec<TargetError>) {
    let issuance = definition.issuance();
    let mut disagrees = false;

    // A partial census would describe a different issuance than the
    // one the introspection primitive actually pushes.
    let declared: BTreeSet<IssuanceField> = issuance.fields().iter().copied().collect();
    let expected: BTreeSet<IssuanceField> = IssuanceField::ALL.iter().copied().collect();
    if declared != expected {
        disagrees = true;
    }

    let introspection = issuance.introspection();
    match results_under(definition, introspection, SuccessCondition::IssuanceAbsent) {
        Some(results) => {
            if results != vec![StackValueType::Encoded(issuance.absent_marker())] {
                disagrees = true;
            }
        }
        None => disagrees = true,
    }

    match results_under(definition, introspection, SuccessCondition::IssuancePresent) {
        Some(results) => {
            for class in [
                EncodingClass::IssuanceEntropy,
                EncodingClass::IssuanceBlindingNonce,
            ] {
                if !results.contains(&StackValueType::Encoded(class)) {
                    disagrees = true;
                }
            }
        }
        None => disagrees = true,
    }

    // The flag byte is a second way to learn the same fact, so it has
    // to be reachable if the contract claims it reports issuance.
    if issuance.outpoint_flag_reports_issuance() {
        let outpoint = results_under(
            definition,
            OpcodeId::InspectInputOutpoint,
            SuccessCondition::Always,
        )
        .unwrap_or_default();
        if !outpoint.contains(&StackValueType::Encoded(EncodingClass::OutPointFlags)) {
            disagrees = true;
        }
    }

    for capability in [
        ElementsCapability::IssuanceIntrospection,
        ElementsCapability::ReissuanceIntrospection,
    ] {
        match definition.capabilities().get(&capability) {
            Some(contract) => {
                if !contract.opcodes().contains(&introspection) {
                    disagrees = true;
                }
            }
            None => disagrees = true,
        }
    }

    if issuance.evidence().is_empty() {
        disagrees = true;
    }

    if disagrees {
        errors.push(TargetError::IssuanceContractMismatch);
    }
}

/// The status a confidential-value claim state implies.
const fn implied_status(state: ConfidentialCapabilityState) -> StaticCapabilityStatus {
    match state {
        ConfidentialCapabilityState::PrimitiveReviewed => StaticCapabilityStatus::Reviewed,
        // Relied upon outside the script language: the review reached
        // the claim but no primitive demonstrates it.
        ConfidentialCapabilityState::ExternalConsensusClaim => StaticCapabilityStatus::Incomplete,
        ConfidentialCapabilityState::Unsupported => StaticCapabilityStatus::Unsupported,
    }
}

/// Confidential-value claim states, the capability rows describing the
/// same claims, the participating encodings, and the evidence link
/// must agree.
fn weld_confidential(definition: &TargetDefinition, errors: &mut Vec<TargetError>) {
    use ConfidentialValueCapability as V;

    let confidential = definition.confidential_values();
    let mut disagrees = false;

    // Each claim is carried twice, once as a confidential-value state
    // and once as one or more capability rows. Value inspection is the
    // one claim that spans two rows, because the target inspects
    // inputs and outputs with different primitives.
    let pairs: [(V, &[ElementsCapability]); 5] = [
        (
            V::ConsensusValueConservation,
            &[ElementsCapability::ConfidentialValueConservation],
        ),
        (
            V::CommitmentEquality,
            &[ElementsCapability::CommitmentEquality],
        ),
        (
            V::ExplicitValueInspection,
            &[ElementsCapability::ExplicitValueInspection],
        ),
        (
            V::ConfidentialValueInspection,
            &[
                ElementsCapability::InputValueInspection,
                ElementsCapability::OutputValueInspection,
            ],
        ),
        (
            V::AuthenticatedOpening,
            &[ElementsCapability::AuthenticatedValueOpening],
        ),
    ];

    for (claim, capabilities) in pairs {
        let Some(state) = confidential.states().get(&claim).copied() else {
            disagrees = true;
            continue;
        };
        for capability in capabilities {
            if status_of(definition, *capability) != Some(implied_status(state)) {
                disagrees = true;
            }
        }
    }

    // Whatever the value-inspection primitives can push has to be a
    // value class the conservation claim actually covers.
    for opcode in [OpcodeId::InspectInputValue, OpcodeId::InspectOutputValue] {
        for condition in [
            SuccessCondition::ExplicitEncoding,
            SuccessCondition::ConfidentialEncoding,
        ] {
            let Some(results) = results_under(definition, opcode, condition) else {
                disagrees = true;
                continue;
            };
            for value in &results {
                if let StackValueType::EncodedPayload(class) | StackValueType::EncodingPrefix(class) =
                    value
                    && !confidential.participating_encodings().contains(class)
                {
                    disagrees = true;
                }
            }
        }
    }

    if confidential.evidence().is_empty() {
        disagrees = true;
    }

    if disagrees {
        errors.push(TargetError::ConfidentialContractMismatch);
    }
}

/// Opcode resource costs, the per-check budget, the budget offset, and
/// the consensus and policy dimensions must agree.
fn weld_resources(definition: &TargetDefinition, errors: &mut Vec<TargetError>) {
    let resources = definition.resources();
    let budget = definition.authorization().signature().budget_per_check();
    let mut disagrees = false;

    // A primitive charges nothing or it charges the one per-check
    // figure. A third number would be a budget no bound is expressed
    // in.
    let mut charging = 0_usize;
    for spec in definition.opcodes().values() {
        let cost = spec.resources().validation_budget();
        if cost == 0 {
            continue;
        }
        charging += 1;
        if cost != budget {
            disagrees = true;
        }
    }
    if charging == 0 {
        disagrees = true;
    }

    // The budget dimension has to be a dimension the contract states,
    // and the starting budget has to have an offset to start from.
    if !resources
        .consensus()
        .bounds()
        .contains_key(&crate::resource::ResourceDimension::ValidationBudget)
    {
        disagrees = true;
    }
    if resources.consensus().validation_budget_offset() == 0 {
        disagrees = true;
    }

    // A policy bound on a dimension consensus does not state has
    // nothing to narrow, so it describes a rule with no subject.
    for dimension in resources.policy().bounds().keys() {
        if !resources.consensus().bounds().contains_key(dimension) {
            disagrees = true;
        }
    }

    if disagrees {
        errors.push(TargetError::ResourceContractMismatch);
    }
}

/// Every subcontract that is not an opcode must name declared evidence.
///
/// The opcode, encoding, and capability registries are already checked
/// one entry at a time. The subcontracts were not, and a subcontract
/// whose claim nothing is ever asked to demonstrate rests on this
/// crate's assertion alone.
fn weld_evidence(definition: &TargetDefinition, errors: &mut Vec<TargetError>) {
    let authorization = definition.authorization();
    let sets: [&BTreeSet<TargetEvidenceRequirementId>; 5] = [
        authorization.signature().evidence(),
        authorization.sighash().evidence(),
        authorization.relative_timelock().evidence(),
        definition.confidential_values().evidence(),
        definition.issuance().evidence(),
    ];

    let mut disagrees = false;
    for set in sets {
        if set.is_empty() {
            disagrees = true;
        }
        for id in set {
            if !definition.evidence_requirements().contains_key(id) {
                disagrees = true;
            }
        }
    }

    if disagrees {
        errors.push(TargetError::EvidenceContractMismatch);
    }
}
