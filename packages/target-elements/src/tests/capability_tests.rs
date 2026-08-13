//! Capability, authorization, confidential-value, issuance, resource,
//! and evidence cross-checks.

use std::collections::{BTreeMap, BTreeSet};

use crate::authorization::{SighashDimension, TimelockMode, UnknownPublicKeyTypeRule};
use crate::capability::{
    CapabilityContract, ElementsCapability, StaticCapabilityStatus, prerequisite_cycle_residual,
};
use crate::confidential::{
    ConfidentialCapabilityState, ConfidentialValueCapability, IssuanceField,
};
use crate::definition::reviewed_elements_tapscript;
use crate::encoding::EncodingClass;
use crate::evidence::TargetEvidenceRequirementId;
use crate::evidence_registry::RequiredEvidenceEnvironment;
use crate::opcode::{FailureOutcome, OpcodeId, VALIDATION_BUDGET_PER_CHECK};
use crate::resource::{ResourceBound, ResourceDimension};

/// Fetches one capability contract.
fn contract(capability: ElementsCapability) -> CapabilityContract {
    reviewed_elements_tapscript()
        .expect("the reviewed contract validates")
        .definition()
        .capabilities()
        .get(&capability)
        .expect("every capability key has a contract")
        .clone()
}

#[test]
fn the_capability_census_is_complete_and_duplicate_free() {
    let declared: BTreeSet<ElementsCapability> = ElementsCapability::ALL.iter().copied().collect();
    assert_eq!(declared.len(), ElementsCapability::ALL.len());

    let registered: BTreeSet<ElementsCapability> = reviewed_elements_tapscript()
        .expect("the reviewed contract validates")
        .definition()
        .capabilities()
        .keys()
        .copied()
        .collect();
    assert_eq!(registered, declared);
}

#[test]
fn the_statuses_are_the_independently_expected_ones() {
    // Written out here rather than read back from the registry. The
    // three capabilities that are not `Reviewed` are the whole point
    // of having three states, so a silent promotion of any of them
    // must fail this test.
    let not_reviewed: BTreeMap<ElementsCapability, StaticCapabilityStatus> = [
        // The signature primitives were reviewed; the sighash
        // construction was not, so these cannot claim more.
        (
            ElementsCapability::OutputCommittingSighash,
            StaticCapabilityStatus::Incomplete,
        ),
        (
            ElementsCapability::InputCommitmentControl,
            StaticCapabilityStatus::Incomplete,
        ),
        // An external consensus claim: no primitive demonstrates it.
        (
            ElementsCapability::ConfidentialValueConservation,
            StaticCapabilityStatus::Incomplete,
        ),
        (
            ElementsCapability::CommitmentEquality,
            StaticCapabilityStatus::Unsupported,
        ),
        // Never complete. Curve and hash primitives exist; that is not
        // an opening proof.
        (
            ElementsCapability::AuthenticatedValueOpening,
            StaticCapabilityStatus::Unsupported,
        ),
    ]
    .into_iter()
    .collect();

    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    for (capability, contract) in definition.definition().capabilities() {
        let expected = not_reviewed
            .get(capability)
            .copied()
            .unwrap_or(StaticCapabilityStatus::Reviewed);
        assert_eq!(contract.status(), expected, "{capability:?}");
    }
}

#[test]
fn an_authenticated_opening_is_never_marked_complete() {
    // Guarded separately from the table above because it is the single
    // inference this contract most needs to refuse: the parts being
    // present is not the proof being available.
    let opening = contract(ElementsCapability::AuthenticatedValueOpening);
    assert_eq!(opening.status(), StaticCapabilityStatus::Unsupported);
    assert!(
        opening.opcodes().is_empty(),
        "no primitive establishes an opening"
    );
    assert!(
        opening
            .prerequisites()
            .contains(&ElementsCapability::EcScalarVerification),
        "the prerequisites are present, and are not sufficient"
    );
}

#[test]
fn every_capability_maps_to_evidence() {
    // Without exception. There is no capability here whose claim is
    // purely type-level: even that the execution domain exists is
    // something a node must be asked to show.
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    for (capability, contract) in definition.definition().capabilities() {
        assert!(
            !contract.evidence().is_empty(),
            "{capability:?} names no evidence"
        );
    }
}

#[test]
fn the_prerequisite_relation_is_acyclic() {
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    assert_eq!(
        prerequisite_cycle_residual(definition.definition().capabilities()),
        Vec::new()
    );
}

#[test]
fn an_introduced_cycle_is_detected() {
    // An independent check of the detector: with an edge added back
    // from the leaf version to something that requires it, nothing can
    // be ordered.
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let mut capabilities = definition.definition().capabilities().clone();

    let leaf = capabilities[&ElementsCapability::RequiredLeafVersion].clone();
    capabilities.insert(
        ElementsCapability::RequiredLeafVersion,
        CapabilityContract::new(
            leaf.capability(),
            [ElementsCapability::SignatureVerification],
            leaf.opcodes().iter().copied(),
            leaf.encodings().iter().copied(),
            leaf.evidence().iter().copied(),
            leaf.status(),
        ),
    );

    let residual = prerequisite_cycle_residual(&capabilities);
    assert!(
        residual.contains(&ElementsCapability::RequiredLeafVersion)
            && residual.contains(&ElementsCapability::TapscriptExecution)
            && residual.contains(&ElementsCapability::SignatureVerification),
        "the residual names every capability the cycle traps: {residual:?}"
    );
}

#[test]
fn a_self_prerequisite_is_detected() {
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let mut capabilities = definition.definition().capabilities().clone();

    let victim = capabilities[&ElementsCapability::StreamingSha256].clone();
    capabilities.insert(
        ElementsCapability::StreamingSha256,
        CapabilityContract::new(
            victim.capability(),
            [ElementsCapability::StreamingSha256],
            victim.opcodes().iter().copied(),
            victim.encodings().iter().copied(),
            victim.evidence().iter().copied(),
            victim.status(),
        ),
    );

    // The residual is every capability the cycle traps, not only the
    // capabilities on it. The opening capability requires streaming
    // hashing, so it can never be established either, and a report
    // naming only the self-edge would understate the damage.
    assert_eq!(
        prerequisite_cycle_residual(&capabilities),
        vec![
            ElementsCapability::StreamingSha256,
            ElementsCapability::AuthenticatedValueOpening,
        ]
    );
}

#[test]
fn no_sighash_dimension_is_claimed_as_reviewed() {
    // The review reached the signature primitives but not the sighash
    // construction. Every dimension is therefore classified as
    // unreviewed, and absence from the reviewed set means "this
    // package has not established it", not "the target lacks it".
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let sighash = definition.definition().authorization().sighash();

    assert!(sighash.reviewed().is_empty());
    assert_eq!(
        sighash.unreviewed().len(),
        SighashDimension::ALL.len(),
        "every dimension is classified, and all of them the same way"
    );
    assert_eq!(sighash.contradictory(), None);
    assert_eq!(sighash.unclassified(), None);
}

#[test]
fn the_signature_primitive_records_its_two_distinct_failure_modes() {
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let signature = definition.definition().authorization().signature();

    assert_eq!(
        signature.empty_signature(),
        FailureOutcome::ConsumeOperandsPushFalse
    );
    assert_eq!(
        signature.invalid_signature(),
        FailureOutcome::AbortEvaluation
    );
    assert_eq!(signature.budget_per_check(), VALIDATION_BUDGET_PER_CHECK);
    assert_eq!(
        signature.public_key_encoding(),
        EncodingClass::XOnlyPublicKey
    );
}

#[test]
fn an_unknown_public_key_type_turns_verification_into_a_no_op() {
    // A sharp edge worth failing loudly over. A non-empty signature
    // paired with an unrecognized key encoding reports success without
    // any verification having happened, so a backend must constrain
    // the key encoding itself rather than trust the check's result.
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    assert_eq!(
        definition
            .definition()
            .authorization()
            .signature()
            .unknown_public_key_type(),
        UnknownPublicKeyTypeRule::SucceedsWithoutVerification
    );
}

#[test]
fn the_relative_timelock_fields_are_the_reviewed_bits() {
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let timelock = definition.definition().authorization().relative_timelock();

    assert_eq!(timelock.sequence_disable_flag(), 0x8000_0000);
    assert_eq!(timelock.sequence_mode_flag(), 0x0040_0000);
    assert_eq!(timelock.sequence_value_mask(), 0x0000_ffff);
    assert_eq!(timelock.time_interval_shift(), 9);
    // Below this version the lock is not enforced at all, so a program
    // relying on it must constrain the version too.
    assert_eq!(timelock.minimum_transaction_version(), 2);
    assert_eq!(timelock.unsatisfied(), FailureOutcome::AbortEvaluation);
    assert!(!timelock.has_overlapping_fields());
    assert_eq!(
        timelock.modes(),
        &BTreeSet::from([TimelockMode::BlockHeight, TimelockMode::TimeInterval])
    );
}

#[test]
fn no_cadence_band_appears_in_the_timelock_contract() {
    // The protocol's operator window and its permissionless tail are
    // built from these primitives downstream. Nothing here names a
    // minimum age, a maximum age, or a role, and the accessor list is
    // the whole surface.
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let timelock = definition.definition().authorization().relative_timelock();
    assert_eq!(timelock.sequence_value_mask().count_ones(), 16);
}

#[test]
fn the_confidential_value_claims_are_classified_exactly_once_each() {
    use ConfidentialCapabilityState as S;
    use ConfidentialValueCapability as V;

    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let contract = definition.definition().confidential_values();

    let expected: BTreeMap<V, S> = [
        (V::ConsensusValueConservation, S::ExternalConsensusClaim),
        (V::CommitmentEquality, S::Unsupported),
        (V::ExplicitValueInspection, S::PrimitiveReviewed),
        (V::ConfidentialValueInspection, S::PrimitiveReviewed),
        (V::AuthenticatedOpening, S::Unsupported),
    ]
    .into_iter()
    .collect();

    assert_eq!(contract.states(), &expected);
    assert_eq!(contract.unclassified(), None);
}

#[test]
fn value_conservation_is_an_external_claim_rather_than_a_primitive() {
    // No script instruction demonstrates it, so a deployment must
    // evidence it and no relation may be marked discharged on the
    // strength of the contract alone.
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    assert_eq!(
        definition
            .definition()
            .confidential_values()
            .states()
            .get(&ConfidentialValueCapability::ConsensusValueConservation),
        Some(&ConfidentialCapabilityState::ExternalConsensusClaim)
    );
    assert!(
        contract(ElementsCapability::ConfidentialValueConservation)
            .opcodes()
            .is_empty()
    );
}

#[test]
fn the_issuance_contract_states_target_facts_only() {
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let issuance = definition.definition().issuance();

    assert_eq!(
        issuance.fields(),
        &IssuanceField::ALL.iter().copied().collect::<BTreeSet<_>>()
    );
    assert_eq!(issuance.introspection(), OpcodeId::InspectInputIssuance);
    assert_eq!(issuance.absent_marker(), EncodingClass::NullValue);
    assert!(issuance.outpoint_flag_reports_issuance());
}

#[test]
fn consensus_and_policy_limits_stay_apart() {
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let resources = definition.definition().resources();

    assert_eq!(
        resources
            .consensus()
            .bounds()
            .get(&ResourceDimension::TransactionWeight),
        Some(&ResourceBound::Maximum(4_000_000))
    );
    // A deployment may be stricter than consensus, and here it is.
    assert_eq!(
        resources
            .policy()
            .bounds()
            .get(&ResourceDimension::TransactionWeight),
        Some(&ResourceBound::Maximum(400_000))
    );
    assert_eq!(resources.policy_looser_than_consensus(), None);
    assert_eq!(resources.missing_consensus_dimension(), None);
    assert_eq!(resources.zero_bound(), None);
}

#[test]
fn the_unenforced_dimensions_are_stated_as_unbounded_rather_than_omitted() {
    // The reviewed execution domain enforces neither a script size nor
    // an operation budget, unlike the domains it replaced. Saying so
    // is a reviewed fact; leaving the entries out would have read as
    // an oversight.
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let consensus = definition.definition().resources().consensus();

    for dimension in [
        ResourceDimension::ScriptBytes,
        ResourceDimension::OperationCost,
    ] {
        assert_eq!(
            consensus.bounds().get(&dimension),
            Some(&ResourceBound::Unbounded),
            "{dimension:?}"
        );
    }
    assert_eq!(consensus.witness_scale_factor(), 4);
    assert_eq!(consensus.validation_budget_offset(), 50);
}

#[test]
fn no_protocol_batch_bound_is_derived_from_a_target_limit() {
    // The resource contract exposes target figures and nothing else.
    // Every bound below is a target constant; none is a count of
    // protocol inputs, outputs, or batch members, and no arithmetic
    // here turns one into the other.
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let consensus = definition.definition().resources().consensus();

    assert_eq!(
        consensus
            .bounds()
            .get(&ResourceDimension::StackElementBytes),
        Some(&ResourceBound::Maximum(520))
    );
    assert_eq!(
        consensus.bounds().get(&ResourceDimension::PeakStackItems),
        Some(&ResourceBound::Maximum(1_000))
    );
    assert_eq!(
        consensus.bounds().get(&ResourceDimension::ControlPathDepth),
        Some(&ResourceBound::Maximum(128))
    );
}

#[test]
fn the_evidence_registry_is_complete_and_every_requirement_can_go_stale() {
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let registry = definition.definition().evidence_requirements();

    let declared: BTreeSet<TargetEvidenceRequirementId> =
        TargetEvidenceRequirementId::ALL.iter().copied().collect();
    assert_eq!(declared.len(), TargetEvidenceRequirementId::ALL.len());
    assert_eq!(registry.keys().copied().collect::<BTreeSet<_>>(), declared);

    for (id, requirement) in registry {
        assert_eq!(requirement.id(), *id);
        assert!(
            !requirement.stale_on().is_empty(),
            "{id:?} would never expire"
        );
    }
}

#[test]
fn policy_evidence_is_deployment_scoped() {
    // A deployment's own bounds cannot be evidenced on "any network",
    // because they are not a property of the target at all.
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    assert_eq!(
        definition
            .definition()
            .evidence_requirements()
            .get(&TargetEvidenceRequirementId::PolicyResourceLimits)
            .expect("declared")
            .environment(),
        RequiredEvidenceEnvironment::DevelopmentNetwork
    );
}

#[test]
fn no_evidence_requirement_carries_a_result() {
    // A compile-time boundary expressed as a readable one: the
    // accessor surface is id, subject, claim, environment, and stale
    // conditions. There is no pass field, no report, no timestamp, no
    // endpoint, and no credential to read.
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let requirement = definition
        .definition()
        .evidence_requirements()
        .get(&TargetEvidenceRequirementId::TapscriptExecutionDomain)
        .expect("declared");

    let _ = requirement.id();
    let _ = requirement.subject();
    let _ = requirement.claim();
    let _ = requirement.environment();
    let _ = requirement.stale_on();
}
