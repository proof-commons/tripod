//! Evidence-plan, comparison, and gate tests.

use target_elements::{
    CapabilityContract, ElementsCapability, StaticCapabilityStatus, TargetEvidenceRequirementId,
};

use crate::fixture::NativeCaseGroup;
use crate::report::{EvidencePlanClass, NATIVE_REPORT_SCHEMA};
use crate::validate::guide_nine_evidence_plan;

use super::support::reviewed_target;

#[test]
fn the_plan_partitions_the_evidence_census_exactly() {
    let plan = guide_nine_evidence_plan().expect("the plan is a partition");
    for id in TargetEvidenceRequirementId::ALL {
        assert!(
            plan.class(*id).is_some(),
            "every requirement needs a plan class",
        );
    }
    assert_eq!(plan.iter().count(), TargetEvidenceRequirementId::ALL.len());
}

#[test]
fn the_required_plan_is_the_guide_nine_one() {
    let plan = guide_nine_evidence_plan().expect("the plan is a partition");
    assert_eq!(
        plan.class(TargetEvidenceRequirementId::PushEncodingSemantics),
        Some(EvidencePlanClass::Required),
        "the push-encoding row is required, not inherited from opcode semantics",
    );
    assert_eq!(
        plan.class(TargetEvidenceRequirementId::SighashSemantics),
        Some(EvidencePlanClass::UnresolvedByDesign),
    );
    assert_eq!(
        plan.class(TargetEvidenceRequirementId::CommitmentEquality),
        Some(EvidencePlanClass::UnsupportedByStaticContract),
    );
    assert_eq!(
        plan.class(TargetEvidenceRequirementId::ConfidentialValueConservation),
        Some(EvidencePlanClass::DeferredToTransactionEvidence),
    );
}

#[test]
fn the_unsupported_classification_agrees_with_the_reviewed_contract() {
    // The plan calls commitment equality unsupported. That is only
    // honest while the contract says so too, which is what this checks:
    // a contract that later gains a reviewed mechanism must force the
    // plan to be revisited rather than leaving a stale classification.
    let target = reviewed_target();
    let status = target
        .definition()
        .capabilities()
        .get(&ElementsCapability::CommitmentEquality)
        .map(CapabilityContract::status);
    assert_eq!(
        status,
        Some(StaticCapabilityStatus::Unsupported),
        "the plan's unsupported classification must match the contract",
    );
}

#[test]
fn every_fixture_group_bears_on_a_requirement() {
    // A group that bears on nothing would let a case run, pass, and
    // establish no evidence at all.
    let plan = guide_nine_evidence_plan().expect("the plan is a partition");
    for group in NativeCaseGroup::ALL {
        let requirements = crate::validate::requirements_for_tests(*group);
        assert!(
            !requirements.is_empty(),
            "the {} group bears on no requirement",
            group.wire_name(),
        );
        for requirement in requirements {
            assert!(
                plan.class(*requirement).is_some(),
                "a group bears on a requirement the plan does not classify",
            );
        }
    }
}

#[test]
fn the_report_schema_is_stated() {
    assert_eq!(NATIVE_REPORT_SCHEMA, 1);
}
