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

use crate::authorization::UnknownPublicKeyTypeRule;
use crate::capability::{ElementsCapability, StaticCapabilityStatus};
use crate::confidential::{
    ConfidentialCapabilityState, ConfidentialValueCapability, IssuanceField,
};
use crate::definition::TargetDefinition;
use crate::encoding::EncodingClass;
use crate::error::TargetError;
use crate::evidence::TargetEvidenceRequirementId;
use crate::opcode::{FailureCause, FailureOutcome, OpcodeId, StackContract, StackValueType};
use crate::operand::OperandContract;
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
    weld_stack_growth(definition, errors);
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
        .map(|case| case.effect().computed_types())
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

/// The signature subcontract's whole behavior, compared against every
/// signature opcode.
///
/// # One expectation, derived and compared
///
/// The subcontract states the key and signature encodings, what an
/// empty signature does, what an invalid one does, what an unknown key
/// type does, the per-check budget, and the evidence the behavior
/// rests on. Each of those also appears on the opcodes, as an operand
/// admission, a success case, or a failure effect. This derives one
/// complete expected behavior from the subcontract and compares every
/// signature opcode against it, rather than checking whichever fields
/// happen to be convenient: the earlier form never read
/// `unknown_public_key_type` at all, so a definition could say unknown
/// keys reject and succeed unverified at once.
fn weld_signature(definition: &TargetDefinition, errors: &mut Vec<TargetError>) {
    let signature = definition.authorization().signature();

    // The whole expected behavior, derived once from the subcontract,
    // and then compared against every signature opcode. Deriving it
    // once is the point: the weld's earlier form read some of the
    // subcontract's fields and left others — the unknown-public-key
    // rule above all — so a definition could say unknown keys reject in
    // the subcontract and succeed unverified in every opcode, and
    // nothing looked at both `(´[PLAN-rule:guide11-exec:signature-weld]´)`.
    let unknown_key_succeeds = matches!(
        signature.unknown_public_key_type(),
        UnknownPublicKeyTypeRule::SucceedsWithoutVerification
    );
    let mut disagrees = false;

    // The subcontract names what an empty signature does, and that
    // outcome is only reachable if the operand admits the empty item.
    // A position refusing it as a malformed operand would put the
    // documented path outside the primitive's domain, so the named
    // outcome would describe behavior nothing could produce.
    let expected_signature_operand = OperandContract::Signature {
        nonempty_encoding: signature.signature_encoding(),
        empty_allowed: true,
    };
    // The key position admits unknown nonempty forms exactly when the
    // rule says they succeed. If the rule rejects them, admitting them
    // would leave the operand describing a path the subcontract denies.
    let expected_public_key_operand = OperandContract::PublicKey {
        recognized_encoding: signature.public_key_encoding(),
        unknown_nonempty_allowed: unknown_key_succeeds,
    };

    for (opcode, verifying) in BRANCHING_SIGNATURE_OPCODES
        .iter()
        .map(|opcode| (opcode, false))
        .chain(VERIFYING_SIGNATURE_OPCODES.iter().map(|o| (o, true)))
    {
        let Some(spec) = definition.opcodes().get(opcode) else {
            disagrees = true;
            continue;
        };

        // -- Failure behavior ------------------------------------
        //
        // The branchable forms are the ones the subcontract's
        // empty-signature field describes. The verifying forms turn
        // that branchable result into an abort: a contract letting them
        // push a false would be describing a primitive that does not
        // exist.
        let expected_empty = if verifying {
            FailureOutcome::AbortEvaluation
        } else {
            signature.empty_signature()
        };
        if outcome_of(definition, *opcode, FailureCause::EmptySignature) != Some(expected_empty) {
            disagrees = true;
        }

        if outcome_of(definition, *opcode, FailureCause::InvalidSignature)
            != Some(signature.invalid_signature())
        {
            disagrees = true;
        }

        // An empty key is refused outright, and that refusal is a
        // different fact from the unknown-nonempty rule: one is a
        // rejection, the other a success without verification. A
        // contract that dropped the empty-key abort would be saying the
        // forward-compatibility path swallows emptiness too.
        if outcome_of(definition, *opcode, FailureCause::EmptyPublicKey)
            != Some(FailureOutcome::AbortEvaluation)
        {
            disagrees = true;
        }

        // Where an unknown key succeeds without verifying, an invalid
        // signature must abort rather than push a false. A branchable
        // "verification failed" would be indistinguishable on the stack
        // from the empty-signature false, on a primitive whose other
        // path already succeeds without verifying anything — so a
        // consumer reading the result could not tell which of three
        // things happened.
        if unknown_key_succeeds
            && outcome_of(definition, *opcode, FailureCause::InvalidSignature)
                != Some(FailureOutcome::AbortEvaluation)
        {
            disagrees = true;
        }

        // -- Operand admission -----------------------------------
        //
        // Exactly one position of each kind, each matching the derived
        // expectation. Counting rather than searching: a primitive with
        // two signature positions states a shape the subcontract cannot
        // describe.
        let operands = spec.stack().operands();
        let signature_positions = operands
            .iter()
            .filter(|operand| matches!(operand, OperandContract::Signature { .. }))
            .collect::<Vec<_>>();
        let key_positions = operands
            .iter()
            .filter(|operand| matches!(operand, OperandContract::PublicKey { .. }))
            .collect::<Vec<_>>();

        if signature_positions.as_slice() != [&expected_signature_operand]
            || key_positions.as_slice() != [&expected_public_key_operand]
        {
            disagrees = true;
        }

        // -- Success algebra -------------------------------------
        if signature_success_disagrees(spec, verifying, unknown_key_succeeds) {
            disagrees = true;
        }

        // -- Shared numbers and evidence --------------------------
        //
        // One number, two views: the budget a check can charge is a
        // resource cost and a signature-contract field alike.
        if spec.resources().validation_budget() != signature.budget_per_check() {
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

/// Whether one signature opcode's successful forms disagree with the
/// behavior the subcontract states.
///
/// Verification against a recognized key is the primitive's reason for
/// existing, so its form is present exactly once. The unknown-key form
/// is present exactly when the rule permits it, and absent when it does
/// not: an opcode advertising it under a rejecting rule is the
/// contradiction this weld exists to catch, in the direction the review
/// found it.
fn signature_success_disagrees(
    spec: &crate::opcode::OpcodeSpec,
    verifying: bool,
    unknown_key_succeeds: bool,
) -> bool {
    // A verifying form leaves no branchable result; a branching form
    // pushes exactly one Boolean, which is the whole of what a caller
    // branches on.
    let expected_results: Vec<StackValueType> = if verifying {
        Vec::new()
    } else {
        vec![StackValueType::Bool]
    };

    let cases = spec.stack().success().cases();
    let mut disagrees = false;

    for condition in [
        SuccessCondition::RecognizedKeyVerifiedSignature,
        SuccessCondition::UnknownKeyTypeUnverified,
    ] {
        let expected_count = usize::from(
            condition == SuccessCondition::RecognizedKeyVerifiedSignature || unknown_key_succeeds,
        );
        let matching = cases
            .iter()
            .filter(|case| case.condition() == condition)
            .collect::<Vec<_>>();

        if matching.len() != expected_count {
            disagrees = true;
            continue;
        }

        // Both admitted forms leave the same thing behind: the
        // unknown-key path succeeds without verifying, but it is still a
        // success of this primitive and pushes what this primitive's
        // successes push.
        for case in matching {
            if case.effect().computed_types() != expected_results {
                disagrees = true;
            }
        }
    }

    disagrees
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

/// Every primitive's declared stack growth must be the growth its own
/// stack contract implies.
///
/// # Why the field needed a weld at all
///
/// [`OpcodeResourceCost::maximum_stack_growth`] is the one resource
/// dimension a downstream stack scheduler reads directly, and until
/// this weld it was compared against nothing: the resource weld reads
/// the budget dimensions, and the success algebra states the depth
/// arithmetic, but no check made the two say the same thing. A row
/// transcribed from the wrong primitive would validate.
///
/// # The transient the surviving depth does not show
///
/// The obvious derivation — the greatest depth change over every
/// surviving outcome — is wrong for two of the fifty-five primitives,
/// and wrong in the direction that matters. The field is documented as
/// the greatest increase *at any point during execution*, not the
/// increase the primitive settles at, and the reviewed target reaches
/// a deeper stack mid-primitive than it leaves behind.
///
/// The target implements a verifying signature primitive as its
/// branching counterpart followed by an implicit verification: it pops
/// the operands, pushes the truth value, and only then pops that value
/// again, aborting instead if it was false. So a verifying form
/// transiently occupies exactly its branching counterpart's depth, one
/// item above where it settles, and a scheduler sizing the stack from
/// the surviving figure alone would size it one too shallow.
///
/// That is a fact about the target, recorded with its source location
/// in the tapscript reference. It is derived here from the verifying
/// primitives the signature weld already distinguishes, rather than
/// carried as a per-opcode number, because a per-opcode number is
/// another transcription for a later weld to check.
///
/// [`OpcodeResourceCost::maximum_stack_growth`]:
///     crate::opcode::OpcodeResourceCost::maximum_stack_growth
fn weld_stack_growth(definition: &TargetDefinition, errors: &mut Vec<TargetError>) {
    let mut disagrees = false;

    for (id, spec) in definition.opcodes() {
        if spec.resources().maximum_stack_growth() != derived_stack_growth(*id, spec.stack()) {
            disagrees = true;
        }
        // No reviewed primitive touches the alternate stack. A nonzero
        // row here is a claim no reviewed behavior supports.
        if spec.resources().maximum_altstack_growth() != 0 {
            disagrees = true;
        }
    }

    if disagrees {
        errors.push(TargetError::StackGrowthContractMismatch);
    }
}

/// The greatest main-stack depth one primitive's contract can reach,
/// relative to the depth it started at.
///
/// Aborting failures contribute nothing: evaluation ends, so there is
/// no surviving depth for a later primitive to stand on. The two
/// pushing failure outcomes do contribute, and they do not agree with
/// each other — consuming the operands leaves a shallower stack than
/// the successful path, retaining them leaves a deeper one — which is
/// the reason the maximum is taken over outcomes rather than read off
/// the successful form.
pub(crate) fn derived_stack_growth(id: OpcodeId, stack: &StackContract) -> i64 {
    let surviving = surviving_stack_growth(stack);
    if VERIFYING_SIGNATURE_OPCODES.contains(&id) {
        // The transient peak: the branching counterpart's depth, held
        // until the implicit verification consumes the truth value.
        surviving + 1
    } else {
        surviving
    }
}

/// The greatest depth one primitive can *leave behind*, over every
/// outcome a later primitive could stand on.
///
/// This is the whole derivation for fifty-three of the fifty-five
/// reviewed primitives, and it is kept separate from the transient term
/// so that the two verifying forms' extra item is visible as its own
/// claim rather than folded into an arithmetic no test can point at.
pub(crate) fn surviving_stack_growth(stack: &StackContract) -> i64 {
    let declared_operands = i64::try_from(stack.operands().len()).unwrap_or(i64::MAX);

    let surviving_failures =
        stack
            .failure()
            .effects()
            .iter()
            .filter_map(|effect| match effect.outcome() {
                FailureOutcome::AbortEvaluation => None,
                FailureOutcome::ConsumeOperandsPushFalse => Some(1 - declared_operands),
                FailureOutcome::RetainOperandsPushFalse => Some(1),
            });

    stack
        .success()
        .cases()
        .iter()
        .map(|case| case.effect().depth_change())
        .chain(surviving_failures)
        .max()
        .unwrap_or(0)
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
