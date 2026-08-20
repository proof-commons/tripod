//! Tests for the reviewed compact-ASH transaction forms.
//!
//! These are guard tests over a review, so they check the shape of what
//! the review states rather than recomputing target behavior — nothing
//! in this crate executes, and a test that pretended otherwise would be
//! asserting against the same declarations it reads.

use std::collections::BTreeSet;

use crate::{
    DecisionStatus, EvidenceClaimClass, ExplicitZeroValueRule, FeeRecognitionTerm, FieldForm,
    FormAdmission, FormConstraint, SponsorAuthorizationSource, SponsorInspectedField,
    SubstrateSelection, TargetEvidenceRequirementId, TargetEvidenceSubject, TransactionForm,
    ZeroFeeRepresentation, reviewed_elements_tapscript, reviewed_explicit_zero_value_rule,
    reviewed_fee_output_contract, reviewed_sponsor_input_profile, reviewed_substrate_decision,
    reviewed_transaction_forms, transaction_form_evidence,
};

#[test]
fn the_fee_role_is_recognized_by_form_and_never_by_an_amount() {
    let contract = reviewed_fee_output_contract();

    assert_eq!(
        contract.recognition(),
        &FeeRecognitionTerm::ALL
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
    );
    assert!(
        contract
            .recognition()
            .contains(&FeeRecognitionTerm::EmptyProgram)
    );
    assert!(
        contract
            .recognition()
            .contains(&FeeRecognitionTerm::ExplicitValue)
    );

    // The role is not a protocol object and cannot be blinded, so a
    // protocol relation cannot recognize it and cannot hide behind it.
    assert!(!contract.protocol_object());
    assert!(!contract.blindable());
}

#[test]
fn zero_fee_is_the_absence_of_the_output_and_not_a_zero_in_it() {
    let contract = reviewed_fee_output_contract();

    assert_eq!(contract.zero_fee(), ZeroFeeRepresentation::AbsentOutput);
    assert_eq!(
        contract.refused_zero_fee(),
        ZeroFeeRepresentation::ZeroValuedOutput
    );
    assert_ne!(contract.zero_fee(), contract.refused_zero_fee());
}

#[test]
fn the_target_constrains_neither_the_count_nor_the_position_of_fee_outputs() {
    let contract = reviewed_fee_output_contract();

    assert_eq!(contract.cardinality(), FormConstraint::Unconstrained);
    assert_eq!(contract.position(), FormConstraint::Unconstrained);

    // A construction interface does constrain both, which is weaker
    // than either enforcement and is recorded as such rather than
    // promoted to a target obligation.
    assert_eq!(
        contract.interface_cardinality(),
        FormConstraint::InterfaceConvention
    );
    assert_eq!(
        contract.interface_position(),
        FormConstraint::InterfaceConvention
    );
}

#[test]
fn a_sponsorless_form_is_admitted_by_consensus_and_conditioned_only_by_relay() {
    let forms = reviewed_transaction_forms();
    assert_eq!(
        forms.keys().copied().collect::<BTreeSet<_>>(),
        TransactionForm::ALL
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
    );

    let sponsorless = &forms[&TransactionForm::Sponsorless];
    assert_eq!(sponsorless.consensus(), FormAdmission::Admitted);
    assert_ne!(sponsorless.consensus(), FormAdmission::Refused);
    assert_eq!(sponsorless.relay(), FormAdmission::AdmittedUnderCondition);
    assert!(!sponsorless.relay_conditions().is_empty());
    assert!(!sponsorless.fee_output_present());
}

#[test]
fn both_forms_are_admitted_by_consensus_so_sponsorship_stays_optional() {
    let forms = reviewed_transaction_forms();

    for form in TransactionForm::ALL {
        assert_eq!(
            forms[form].consensus(),
            FormAdmission::Admitted,
            "{form:?} would make sponsorship mandatory"
        );
    }

    assert!(forms[&TransactionForm::Sponsored].fee_output_present());
}

#[test]
fn a_zero_valued_spendable_output_is_refused_by_consensus() {
    let rule = reviewed_explicit_zero_value_rule();

    assert!(rule.contains(&ExplicitZeroValueRule::SpendableRefused));
    assert!(rule.contains(&ExplicitZeroValueRule::UnspendableAdmitted));
    assert_eq!(rule.len(), ExplicitZeroValueRule::ALL.len());
}

#[test]
fn the_sponsor_profile_reads_the_asset_and_never_the_value() {
    let profile = reviewed_sponsor_input_profile();

    assert_eq!(
        profile.source(),
        SponsorAuthorizationSource::OwnTargetSpendingCondition
    );
    assert!(profile.inspected().contains(&SponsorInspectedField::Asset));
    assert_eq!(
        profile.inspected(),
        &SponsorInspectedField::ALL
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
    );

    // Requiring the reserve asset forces the asset field open and
    // leaves the value field closed. That asymmetry is the whole
    // sponsor-opacity result, so it is asserted rather than assumed.
    assert_eq!(profile.asset_form(), FieldForm::ExplicitOnly);
    assert_eq!(profile.value_form(), FieldForm::EitherUninspected);
}

#[test]
fn the_sponsor_profile_claims_no_arbitrary_wallet_support() {
    let profile = reviewed_sponsor_input_profile();

    assert!(!profile.arbitrary_program_support_claimed());
    assert!(!profile.admitted_programs().is_empty());
}

#[test]
fn the_substrate_decision_is_first_party_and_awaits_ratification() {
    let decision = reviewed_substrate_decision();

    assert_eq!(decision.selection(), SubstrateSelection::FirstParty);
    assert_eq!(decision.status(), DecisionStatus::AwaitingRatification);
    assert!(!decision.grounds().is_empty());
    assert!(!decision.revisit().is_empty());
}

#[test]
fn the_reviewed_candidate_is_pinned_rather_than_described() {
    let decision = reviewed_substrate_decision();
    let candidate = decision.candidate();

    assert_eq!(candidate.package(), "elements");
    assert_eq!(candidate.version(), "0.27.0");
    assert!(!candidate.version().is_empty());
    assert!(!candidate.licence().is_empty());
    assert!(!candidate.minimum_rust().is_empty());

    // The native-linked entries are the decisive finding, so their
    // presence is a test rather than a comment.
    assert!(!candidate.native_linked().is_empty());
    assert!(candidate.lockfile_entries_added());
    assert!(candidate.required_direct().contains(&"secp256k1-zkp"));
}

#[test]
fn the_first_party_scope_and_its_delegations_are_both_stated() {
    let decision = reviewed_substrate_decision();

    assert!(!decision.first_party().is_empty());
    assert!(!decision.delegated().is_empty());
}

#[test]
fn every_transaction_form_requirement_is_registered_and_can_go_stale() {
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let registry = definition.definition().evidence_requirements();

    for id in transaction_form_evidence() {
        let requirement = registry
            .get(&id)
            .unwrap_or_else(|| panic!("{id:?} is not registered"));
        assert_eq!(requirement.id(), id);
        assert!(
            !requirement.stale_on().is_empty(),
            "{id:?} would never expire"
        );
    }
}

#[test]
fn the_new_requirements_are_filed_under_the_transaction_form_subject() {
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let registry = definition.definition().evidence_requirements();

    for id in [
        TargetEvidenceRequirementId::FeeOutputForm,
        TargetEvidenceRequirementId::ExplicitZeroValueOutputRule,
        TargetEvidenceRequirementId::FeelessTransactionAdmission,
    ] {
        assert_eq!(
            registry[&id].subject(),
            TargetEvidenceSubject::TransactionForm
        );
    }

    assert_eq!(
        registry[&TargetEvidenceRequirementId::FeeOutputForm].claim(),
        EvidenceClaimClass::RoleRecognition
    );
}

#[test]
fn whole_transaction_conservation_remains_the_external_evidence_role() {
    // The conservation requirement is not minted here: the review
    // reuses the existing identity, so that the external claim about a
    // whole transaction has one owner rather than two.
    assert!(
        transaction_form_evidence()
            .contains(&TargetEvidenceRequirementId::ConfidentialValueConservation)
    );

    // And it keeps the subject it already had, so reusing it did not
    // quietly reclassify it as a transaction-form claim.
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let registry = definition.definition().evidence_requirements();
    assert_eq!(
        registry[&TargetEvidenceRequirementId::ConfidentialValueConservation].subject(),
        TargetEvidenceSubject::ConfidentialValues
    );
}
