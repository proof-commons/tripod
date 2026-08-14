//! Public-API boundary test for the tapscript adapter package.
//!
//! This integration test compiles as an external consumer. At the
//! crate-boundary stage it proves that the package exists, that it is
//! consumable from outside, and that the two vocabularies it was
//! created to join are both reachable through it, and that the adapter
//! answers the whole compiler census in typed multi-state form without
//! claiming that anything has been completed.

use compiler::target::{ExternalEvidenceRole, RequiredCapability};
use tapscript::{
    AssessmentDisposition, EvidenceAssessmentDisposition, StackItem, TapscriptError,
    TapscriptInstruction, TapscriptProgram,
};
use target_elements::{
    ElementsCapability, OpcodeId, ReviewedElementsTapscriptDefinition, TargetEvidenceRequirementId,
    reviewed_elements_tapscript,
};

/// The reviewed static contract.
///
/// The whole input of every public assessment entry point. An external
/// consumer cannot substitute a contract of its own here: the reviewed
/// state has no public constructor, and a definition that differs from
/// the first-party one anywhere stays generic.
fn reviewed_target() -> ReviewedElementsTapscriptDefinition {
    reviewed_elements_tapscript().expect("the reviewed contract validates")
}

#[test]
fn an_external_consumer_can_assess_the_whole_census() {
    let assessed =
        tapscript::assess_complete_census(&reviewed_target()).expect("the census assesses");

    let projected: Vec<_> = assessed
        .capability_projection()
        .into_iter()
        .map(|projection| projection.required())
        .collect();

    assert_eq!(projected, RequiredCapability::ALL);
}

#[test]
fn no_assessment_is_a_boolean_and_none_claims_completion() {
    // The typed multi-state result is the deliverable. An external
    // consumer cannot reduce an assessment to "supported": there is no
    // such accessor, and the one disposition that would mean it —
    // a complete backend pattern — has no constructible value, because
    // its identity type is uninhabited.
    let assessed = tapscript::assess_complete_census(&reviewed_target()).expect("census");

    for projection in assessed.capability_projection() {
        assert_ne!(
            projection.disposition(),
            AssessmentDisposition::CompleteBackendPattern,
        );
    }

    // The unreviewed sighash is visible from outside rather than hidden
    // behind an optimistic pattern obligation.
    let owner = assessed
        .capability_assessment(RequiredCapability::OwnerAuthorization)
        .expect("the census covers owner authorization");
    assert_eq!(
        owner.disposition(),
        AssessmentDisposition::MissingTargetPrimitives,
    );
    assert!(
        owner
            .projection()
            .primitives()
            .contains(&ElementsCapability::OutputCommittingSighash),
    );
}

#[test]
fn the_error_root_is_public_and_is_a_standard_error() {
    fn assert_error<E: std::error::Error>(_: &E) {}

    let error =
        TapscriptError::DuplicateCapabilityAssessment(RequiredCapability::PublicConstructibility);

    assert!(error.to_string().contains("assessed twice"));
    assert_error(&error);
}

#[test]
fn both_joined_vocabularies_are_reachable_through_this_package() {
    // The dependency edge is the deliverable of this commit: an
    // external consumer of `tapscript` can name a compiler-owned
    // abstract capability and a target-owned primitive in one place.
    // Nothing here relates them — relating them is the adapter's job,
    // and inventing a relation in a test would be exactly the
    // unreviewed mapping the adapter exists to make explicit.
    assert_ne!(RequiredCapability::ALL, []);
    assert_ne!(ElementsCapability::ALL, []);
}

#[test]
fn an_external_consumer_sees_the_evidence_role_census_too() {
    // The compiler's second published census survives the adapter: a
    // consumer can ask which compiler-owned evidence role caused which
    // target evidence obligation, rather than seeing only the target
    // requirement with its cause erased.
    let assessed = tapscript::assess_complete_census(&reviewed_target()).expect("census");

    let roles: Vec<_> = assessed
        .evidence_projection()
        .into_iter()
        .map(|projection| projection.role())
        .collect();
    assert_eq!(roles, ExternalEvidenceRole::ALL);

    let substrate = assessed
        .evidence_assessment(ExternalEvidenceRole::SubstrateConservation)
        .expect("the census covers substrate conservation")
        .projection();
    assert_eq!(
        substrate.disposition(),
        EvidenceAssessmentDisposition::TargetEvidenceRequired,
        "no analysis and no program discharges it, so only the target's own rules can",
    );
    assert!(
        substrate
            .evidence()
            .contains(&TargetEvidenceRequirementId::ConfidentialValueConservation),
    );
}

#[test]
fn the_reviewed_target_binds_and_resolves_no_evidence() {
    let target = reviewed_target();
    let contract = target.definition();

    // Every requirement the contract names is still open. An adapter
    // built on this value cannot read anything here as evidence that
    // one has been discharged, because there is no such value: a
    // requirement identity carries no result and has no field for one.
    assert!(!contract.evidence_requirements().is_empty());
    assert!(!contract.capabilities().is_empty());
}

#[test]
fn an_external_consumer_builds_programs_only_from_typed_instructions() {
    // The whole safe surface: a reviewed primitive identity and a
    // checked literal. There is no public constructor taking a raw
    // opcode byte, a raw instruction, or a raw program, so untrusted
    // bytes have exactly one way in — the parser, which either produces
    // typed instructions or fails.
    let target = reviewed_target();
    let item = StackItem::script_number(&target, 3).expect("three is representable");
    let program = TapscriptProgram::new(vec![
        TapscriptInstruction::Push(item),
        TapscriptInstruction::Opcode(OpcodeId::InspectInputValue),
    ])
    .expect("two instructions are within the limit");

    let bytes = program.encode(&target);
    assert_eq!(bytes, vec![0x53, 0xc9]);
    assert_eq!(
        TapscriptProgram::decode(&target, &bytes).expect("the program parses"),
        program,
    );
}

#[test]
fn an_external_consumer_cannot_push_an_oversized_literal() {
    let target = reviewed_target();

    assert!(StackItem::new(&target, vec![0; 520]).is_ok());
    assert!(matches!(
        StackItem::new(&target, vec![0; 521]),
        Err(TapscriptError::OversizedStackItem { .. }),
    ));
}

#[test]
fn an_external_consumer_sees_a_focused_reason_for_every_refused_script() {
    let target = reviewed_target();

    assert!(matches!(
        TapscriptProgram::decode(&target, &[0xff]),
        Err(TapscriptError::UnknownOpcodeByte(0xff)),
    ));
    assert!(matches!(
        TapscriptProgram::decode(&target, &[0x02, 0xab]),
        Err(TapscriptError::TruncatedInstruction),
    ));
    assert!(matches!(
        TapscriptProgram::decode(&target, &[0x01, 0x05]),
        Err(TapscriptError::NonMinimalPush),
    ));
}
