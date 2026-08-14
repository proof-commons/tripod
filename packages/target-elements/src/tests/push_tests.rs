//! The reviewed literal-push rules, against independent expectations.
//!
//! Every expected byte, span, and minimal form below is written out by
//! hand. None of it is read back out of the push contract, because a
//! test that asked the registry what the registry says would pass for
//! any registry at all.

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroUsize;

use crate::definition::{
    TargetDefinition, TargetDefinitionParts, reviewed_elements_tapscript,
    validate_target_definition,
};
use crate::encoding::ByteOrder;
use crate::error::TargetError;
use crate::evidence::TargetEvidenceRequirementId;
use crate::push::{
    PushContract, PushDefect, PushEnforcement, PushForm, PushFormParts, PushFormSpec,
    PushMinimalityStep, PushOpcodeMapping, PushPayloadPredicate,
};
use crate::resource::{
    ConsensusResourceLimits, PolicyResourceLimits, ResourceBound, ResourceContract,
    ResourceDimension,
};

/// The reviewed push contract.
fn pushes() -> PushContract {
    reviewed_elements_tapscript()
        .expect("the reviewed contract validates")
        .definition()
        .pushes()
        .clone()
}

/// The opcode span each form is independently expected to occupy.
///
/// Transcribed from the reviewed upstream reading, not from the
/// registry: the empty push, the direct pushes whose byte is their own
/// width, the three extended widths, the negative-one literal, and the
/// small positive values.
const EXPECTED_SPANS: &[(PushForm, u8, u8)] = &[
    (PushForm::Empty, 0x00, 0x00),
    (PushForm::Direct, 0x01, 0x4b),
    (PushForm::ExtendedOneByteWidth, 0x4c, 0x4c),
    (PushForm::ExtendedTwoByteWidth, 0x4d, 0x4d),
    (PushForm::ExtendedFourByteWidth, 0x4e, 0x4e),
    (PushForm::NegativeOne, 0x4f, 0x4f),
    (PushForm::SmallNumber, 0x51, 0x60),
];

/// A payload of `width` bytes that no numeric form ever carries.
fn filler(width: usize) -> Vec<u8> {
    vec![0xab; width]
}

#[test]
fn every_reviewed_form_has_a_contract() {
    let contract = pushes();

    for form in PushForm::ALL {
        assert!(
            contract.form(*form).is_some(),
            "form {form:?} has no contract"
        );
    }
    assert_eq!(contract.forms().len(), PushForm::ALL.len());
}

#[test]
fn each_form_occupies_the_independently_expected_opcode_span() {
    let contract = pushes();

    for (form, first, last) in EXPECTED_SPANS {
        let spec = contract.form(*form).expect("the form is declared");
        assert_eq!(spec.first_opcode(), *first, "{form:?} first opcode");
        assert_eq!(spec.last_opcode(), *last, "{form:?} last opcode");
    }
    assert_eq!(
        EXPECTED_SPANS.len(),
        PushForm::ALL.len(),
        "the expectation covers every form",
    );
}

#[test]
fn the_reserved_byte_between_the_literals_belongs_to_no_push_form() {
    // The byte between the negative-one literal and the first small
    // number is not a push at all. A contract that swept it into a span
    // would decode a non-push opcode as a literal.
    assert!(pushes().form_for_opcode(0x50).is_none());
}

#[test]
fn no_push_opcode_is_also_a_primitive_byte() {
    let reviewed = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let occupied = reviewed.definition().pushes().occupied_opcodes();

    for spec in reviewed.definition().opcodes().values() {
        assert!(
            !occupied.contains(&spec.code()),
            "primitive {:?} claims push byte {:#04x}",
            spec.id(),
            spec.code(),
        );
    }
}

#[test]
fn the_minimal_form_of_each_boundary_payload_is_the_expected_one() {
    let contract = pushes();
    let cases: Vec<(Vec<u8>, PushForm)> = vec![
        (Vec::new(), PushForm::Empty),
        (vec![0x01], PushForm::SmallNumber),
        (vec![0x10], PushForm::SmallNumber),
        (vec![0x81], PushForm::NegativeOne),
        // Just outside each small-number boundary, so the direct push
        // takes over rather than the numeric literal.
        (vec![0x00], PushForm::Direct),
        (vec![0x11], PushForm::Direct),
        (vec![0x80], PushForm::Direct),
        (vec![0x82], PushForm::Direct),
        (filler(1), PushForm::Direct),
        (filler(75), PushForm::Direct),
        (filler(76), PushForm::ExtendedOneByteWidth),
        (filler(255), PushForm::ExtendedOneByteWidth),
        (filler(256), PushForm::ExtendedTwoByteWidth),
        (filler(520), PushForm::ExtendedTwoByteWidth),
    ];

    for (payload, expected) in cases {
        assert_eq!(
            contract.minimal_form(&payload),
            Ok(expected),
            "payload of {} bytes",
            payload.len(),
        );
    }
}

#[test]
fn a_payload_one_byte_past_the_limit_has_no_form() {
    let contract = pushes();

    assert_eq!(contract.maximum_payload_bytes(), 520);
    assert_eq!(
        contract.minimal_form(&filler(521)),
        Err(PushDefect::Oversized),
    );
}

#[test]
fn the_widest_extended_form_is_never_the_minimal_one() {
    // It is declared because the target decodes it, and it is
    // unreachable as a minimal form because no payload the target
    // accepts is wide enough to need it. Both facts matter: a
    // serializer must never emit it, and a parser must still recognize
    // it in order to refuse it.
    let contract = pushes();

    assert!(
        contract
            .minimality()
            .iter()
            .all(|step| step.form() != PushForm::ExtendedFourByteWidth),
    );
    assert!(contract.form(PushForm::ExtendedFourByteWidth).is_some());
}

#[test]
fn the_forms_that_carry_their_payload_in_the_opcode_round_trip() {
    let contract = pushes();

    for (opcode, expected) in [
        (0x00_u8, Vec::new()),
        (0x4f, vec![0x81_u8]),
        (0x51, vec![0x01]),
        (0x55, vec![0x05]),
        (0x60, vec![0x10]),
    ] {
        let spec = contract
            .form_for_opcode(opcode)
            .expect("a push form occupies the byte");
        assert_eq!(spec.payload_for(opcode), Some(expected.clone()));
        assert_eq!(spec.opcode_for(&expected), Some(opcode));
    }
}

#[test]
fn a_direct_push_states_its_width_in_its_own_opcode() {
    let contract = pushes();
    let spec = contract
        .form(PushForm::Direct)
        .expect("the direct form is declared");

    for width in [1_usize, 2, 40, 75] {
        let payload = filler(width);
        let opcode = spec.opcode_for(&payload).expect("the width is admissible");
        assert_eq!(usize::from(opcode), width);
        assert_eq!(spec.width_in_opcode(opcode), Some(width));
        assert_eq!(spec.payload_for(opcode), None, "the payload follows");
    }

    assert_eq!(spec.opcode_for(&filler(76)), None);
    assert_eq!(spec.width_prefix_bytes(), 0);
}

#[test]
fn each_extended_form_states_the_width_of_its_own_prefix() {
    let contract = pushes();

    for (form, bytes) in [
        (PushForm::ExtendedOneByteWidth, 1_usize),
        (PushForm::ExtendedTwoByteWidth, 2),
        (PushForm::ExtendedFourByteWidth, 4),
    ] {
        let spec = contract.form(form).expect("the form is declared");
        assert_eq!(spec.width_prefix_bytes(), bytes, "{form:?}");
        assert_eq!(
            *spec.mapping(),
            PushOpcodeMapping::WidthPrefix {
                bytes: NonZeroUsize::new(bytes).expect("a declared prefix is nonzero"),
                byte_order: ByteOrder::LittleEndian,
            },
            "{form:?} reads its width least significant byte first",
        );
    }
}

#[test]
fn the_size_bound_is_consensus_and_minimality_is_relay_policy() {
    // The distinction the whole enforcement census exists for: a
    // program that overruns the literal bound is invalid, and one that
    // pushes nonminimally is merely unrelayable.
    let contract = pushes();

    assert_eq!(
        contract.enforcement().get(&PushDefect::Oversized),
        Some(&PushEnforcement::Consensus),
    );
    assert_eq!(
        contract.enforcement().get(&PushDefect::Truncated),
        Some(&PushEnforcement::Consensus),
    );
    assert_eq!(
        contract.enforcement().get(&PushDefect::NonMinimal),
        Some(&PushEnforcement::RelayPolicy),
    );
    assert_eq!(contract.enforcement().len(), PushDefect::ALL.len());
}

#[test]
fn every_form_names_evidence_the_registry_declares() {
    let reviewed = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let definition = reviewed.definition();
    let contract = definition.pushes();

    assert!(!contract.evidence().is_empty());
    for id in contract
        .evidence()
        .iter()
        .chain(contract.forms().values().flat_map(PushFormSpec::evidence))
    {
        assert!(
            definition.evidence_requirements().contains_key(id),
            "{id:?} is not declared",
        );
    }
    assert!(
        contract
            .evidence()
            .contains(&TargetEvidenceRequirementId::PushEncodingSemantics),
    );
}

// ---------------------------------------------------------------
// Mutations. Each drives one diagnostic to failure.
// ---------------------------------------------------------------

/// The parts of the reviewed contract, ready for one mutation.
fn parts() -> TargetDefinitionParts {
    let reviewed = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let source = reviewed.definition();
    TargetDefinitionParts {
        version: source.version(),
        execution_domain: source.execution_domain(),
        leaf_version: source.leaf_version(),
        opcodes: source.opcodes().clone(),
        encodings: source.encodings().clone(),
        pushes: source.pushes().clone(),
        authorization: source.authorization().clone(),
        confidential_values: source.confidential_values().clone(),
        issuance: source.issuance().clone(),
        resources: source.resources().clone(),
        capabilities: source.capabilities().clone(),
        evidence_requirements: source.evidence_requirements().clone(),
    }
}

/// The diagnostics a mutated push contract produces.
fn reject(pushes: PushContract) -> Vec<TargetError> {
    validate_target_definition(TargetDefinition::new(TargetDefinitionParts {
        pushes,
        ..parts()
    }))
    .expect_err("the mutation is a defect")
}

/// The reviewed contract's parts, one form at a time.
fn form_parts(spec: &PushFormSpec) -> PushFormParts {
    PushFormParts {
        form: spec.form(),
        first_opcode: spec.first_opcode(),
        last_opcode: spec.last_opcode(),
        mapping: spec.mapping().clone(),
        minimum_payload_bytes: spec.minimum_payload_bytes(),
        maximum_payload_bytes: spec.maximum_payload_bytes(),
        evidence: spec.evidence().clone(),
    }
}

/// Rebuilds the reviewed push contract with mutated forms.
fn with_forms(forms: Vec<PushFormSpec>) -> PushContract {
    let source = pushes();
    PushContract::new(
        forms,
        source.maximum_payload_bytes(),
        source.minimality().to_vec(),
        source
            .enforcement()
            .iter()
            .map(|(defect, rule)| (*defect, *rule)),
        source.evidence().iter().copied(),
    )
}

/// The reviewed forms, with one of them replaced.
fn forms_with(form: PushForm, replacement: Option<PushFormSpec>) -> Vec<PushFormSpec> {
    let source = pushes();
    let mut forms: Vec<PushFormSpec> = source
        .forms()
        .values()
        .filter(|spec| spec.form() != form)
        .cloned()
        .collect();
    forms.extend(replacement);
    forms
}

#[test]
fn a_missing_form_is_reported() {
    let errors = reject(with_forms(forms_with(PushForm::Direct, None)));

    assert!(errors.contains(&TargetError::MissingPushForm(PushForm::Direct)));
}

#[test]
fn a_form_admitting_no_encoding_is_reported() {
    let source = pushes();
    let spec = source
        .form(PushForm::Direct)
        .expect("the direct form is declared");
    let mut mutated = form_parts(spec);
    mutated.minimum_payload_bytes = 80;

    let errors = reject(with_forms(forms_with(
        PushForm::Direct,
        Some(PushFormSpec::new(mutated)),
    )));

    assert!(errors.contains(&TargetError::InvalidPushForm(PushForm::Direct)));
}

#[test]
fn two_forms_claiming_one_byte_are_reported() {
    let source = pushes();
    let spec = source
        .form(PushForm::NegativeOne)
        .expect("the literal form is declared");
    let mut mutated = form_parts(spec);
    // Reach down into the direct span, which another form already owns.
    mutated.first_opcode = 0x4b;

    let errors = reject(with_forms(forms_with(
        PushForm::NegativeOne,
        Some(PushFormSpec::new(mutated)),
    )));

    assert!(errors.contains(&TargetError::DuplicatePushOpcode(0x4b)));
}

#[test]
fn a_form_naming_no_evidence_is_reported() {
    let source = pushes();
    let spec = source
        .form(PushForm::Empty)
        .expect("the empty form is declared");
    let mut mutated = form_parts(spec);
    mutated.evidence = BTreeSet::new();

    let errors = reject(with_forms(forms_with(
        PushForm::Empty,
        Some(PushFormSpec::new(mutated)),
    )));

    assert!(errors.contains(&TargetError::MissingPushFormEvidence(PushForm::Empty)));
}

#[test]
fn a_defect_with_no_stated_enforcement_is_reported() {
    let source = pushes();
    let mutated = PushContract::new(
        source.forms().values().cloned(),
        source.maximum_payload_bytes(),
        source.minimality().to_vec(),
        [(PushDefect::Truncated, PushEnforcement::Consensus)],
        source.evidence().iter().copied(),
    );

    let errors = reject(mutated);

    assert!(errors.contains(&TargetError::MissingPushEnforcement(PushDefect::Oversized)));
    assert!(errors.contains(&TargetError::MissingPushEnforcement(PushDefect::NonMinimal)));
}

#[test]
fn a_literal_bound_that_drifts_from_the_resource_bound_is_welded_shut() {
    let source = pushes();
    let mutated = PushContract::new(
        source.forms().values().cloned(),
        source.maximum_payload_bytes() - 1,
        source.minimality().to_vec(),
        source
            .enforcement()
            .iter()
            .map(|(defect, rule)| (*defect, *rule)),
        source.evidence().iter().copied(),
    );

    assert!(reject(mutated).contains(&TargetError::PushContractMismatch));
}

#[test]
fn a_rule_naming_a_form_that_refuses_its_payloads_is_welded_shut() {
    let source = pushes();
    let mutated = PushContract::new(
        source.forms().values().cloned(),
        source.maximum_payload_bytes(),
        // The empty form carries nothing wider than nothing, so naming
        // it for a seventy-five byte payload leaves that payload with
        // no encoding at all.
        vec![PushMinimalityStep::new(
            PushPayloadPredicate::WidthAtMost(75),
            PushForm::Empty,
        )],
        source
            .enforcement()
            .iter()
            .map(|(defect, rule)| (*defect, *rule)),
        source.evidence().iter().copied(),
    );

    assert!(reject(mutated).contains(&TargetError::PushContractMismatch));
}

#[test]
fn a_push_byte_that_a_primitive_also_claims_is_welded_shut() {
    // The weld runs over the contract as a whole, so the collision is
    // built by moving the resource bound's twin: here the push span is
    // widened until it swallows a reviewed primitive's byte.
    let source = pushes();
    let spec = source
        .form(PushForm::SmallNumber)
        .expect("the small number form is declared");
    let mut mutated = form_parts(spec);
    mutated.last_opcode = 0xb2;

    assert!(
        reject(with_forms(forms_with(
            PushForm::SmallNumber,
            Some(PushFormSpec::new(mutated)),
        )))
        .contains(&TargetError::PushContractMismatch),
    );
}

#[test]
fn a_resource_bound_that_drifts_from_the_literal_bound_is_welded_shut() {
    // The same weld from the other side: moving the consensus element
    // bound while the push contract keeps the reviewed one.
    let source = parts();
    let consensus = source.resources.consensus();
    let mut bounds: BTreeMap<ResourceDimension, ResourceBound> = consensus.bounds().clone();
    bounds.insert(
        ResourceDimension::StackElementBytes,
        ResourceBound::Maximum(519),
    );
    let resources = ResourceContract::new(
        ConsensusResourceLimits::new(
            bounds,
            consensus.witness_scale_factor(),
            consensus.validation_budget_offset(),
        ),
        PolicyResourceLimits::new(
            source
                .resources
                .policy()
                .bounds()
                .iter()
                .map(|(dimension, bound)| (*dimension, *bound)),
        ),
    );

    let errors = validate_target_definition(TargetDefinition::new(TargetDefinitionParts {
        resources,
        ..source
    }))
    .expect_err("the drifted bound is a defect");

    assert!(errors.contains(&TargetError::PushContractMismatch));
}
