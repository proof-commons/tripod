//! Attestation case facts lowered without discarding typed obligations.
use crate::{
    engine_case_facts as core,
    engine_lifecycle::ObjectKey,
    engine_source::Activation,
    placement::{
        ActivationCondition, BackendStructuralRequirement, CompilerStaticRequirement,
        RelationCasePlan, SemanticScope,
    },
};
use std::collections::BTreeSet;

const fn object(value: architecture::ObjectId) -> ObjectKey {
    ObjectKey {
        namespace: super::ATTESTATION,
        code: value.code(),
    }
}
pub fn lower(
    plan: &RelationCasePlan,
    sources: &BTreeSet<crate::source::SourceRequirement>,
    layout: &[crate::layout::LayoutRequirement],
) -> core::Facts {
    let span = tracing::debug_span!(target: "compiler::engine_attestation", "lower_case_facts", source_count = crate::instrumentation::count(sources.len()), layout_count = crate::instrumentation::count(layout.len()));
    let _entered = span.enter();
    core::Facts {
        key: super::placement::key(&crate::placement::RelationCaseKey {
            relation: plan.relation.clone(),
            case: plan.case.clone(),
        }),
        case_operation: super::operation_key(plan.case.operation),
        activation: activation(plan.activation),
        active: plan.activity == crate::placement::RelationActivity::Active,
        boundaries: plan.boundaries.clone(),
        sources: if plan.activity == crate::placement::RelationActivity::Active {
            sources.iter().map(super::source::row_key).collect()
        } else {
            BTreeSet::new()
        },
        compiler: plan.compiler_requirements.iter().map(compiler).collect(),
        structural: plan
            .structural_requirements
            .iter()
            .map(structural)
            .collect(),
        runtime: plan
            .runtime_requirements
            .iter()
            .map(|value| core::Runtime {
                scope: scope(value.scope),
                multiplicity: value.multiplicity,
                sources: value.sources.iter().map(super::source::row_key).collect(),
            })
            .collect(),
        evidence: plan
            .external_evidence
            .iter()
            .map(super::proof::evidence_key)
            .collect(),
        layout: layout
            .iter()
            .cloned()
            .map(crate::engine_placement::LayoutKey)
            .collect(),
    }
}
pub fn compiler(value: &CompilerStaticRequirement) -> core::CompilerRequirement {
    match value {
        CompilerStaticRequirement::RepresentationSelection {
            object: family,
            allowed,
            selected,
        } => core::CompilerRequirement::Representation {
            object: object(*family),
            allowed: allowed
                .iter()
                .copied()
                .map(super::proof::mode_key)
                .collect(),
            selected: super::proof::mode_key(*selected),
        },
        CompilerStaticRequirement::ConstructibilityValidated { class } => {
            core::CompilerRequirement::Constructibility(constructibility(*class))
        }
        CompilerStaticRequirement::LifecycleExitDeclared {
            object: family,
            exit,
        } => core::CompilerRequirement::Lifecycle {
            object: object(*family),
            exit: super::operation_key(*exit),
        },
    }
}
const fn constructibility(value: realization::ConstructibilityClass) -> core::Constructibility {
    match value {
        realization::ConstructibilityClass::PublicPermissionless => {
            core::Constructibility::PublicPermissionless
        }
        realization::ConstructibilityClass::Operator => core::Constructibility::Operator,
        realization::ConstructibilityClass::OwnersOf { object: family } => {
            core::Constructibility::OwnersOf(object(family))
        }
    }
}
pub const fn structural(value: &BackendStructuralRequirement) -> core::StructuralRequirement {
    match value {
        BackendStructuralRequirement::EncodeAndAuthenticateRepresentation {
            object: family,
            representation,
        } => core::StructuralRequirement::Representation {
            object: object(*family),
            mode: super::proof::mode_key(*representation),
        },
        BackendStructuralRequirement::SecretFreeOperationPath { operation } => {
            core::StructuralRequirement::SecretFree(super::operation_key(*operation))
        }
        BackendStructuralRequirement::RetainRequiredLifecycleExit {
            object: family,
            exit,
        } => core::StructuralRequirement::Lifecycle {
            object: object(*family),
            exit: super::operation_key(*exit),
        },
    }
}
const fn scope(value: SemanticScope) -> core::Scope {
    match value {
        SemanticScope::MemberLocal {
            side,
            object: family,
        } => core::Scope::Member {
            side: super::side_code(side),
            object: object(family),
        },
        SemanticScope::FamilyGlobal {
            side,
            object: family,
        } => core::Scope::Family {
            side: super::side_code(side),
            object: object(family),
        },
        SemanticScope::TransactionSideGlobal { side } => core::Scope::Side(super::side_code(side)),
        SemanticScope::ObjectFamilyGlobal { object: family } => core::Scope::Object(object(family)),
        SemanticScope::TransactionGlobal => core::Scope::Transaction,
    }
}
const fn activation(value: ActivationCondition) -> Activation {
    match value {
        ActivationCondition::Always => Activation::Always,
        ActivationCondition::WhenSponsorPresent => Activation::SponsorPresent,
        ActivationCondition::WhenRepresentation {
            object: family,
            mode,
        } => Activation::Representation {
            object: object(family),
            mode: super::proof::mode_key(mode),
        },
    }
}

pub fn validate(
    operation: architecture::OperationId,
    census: &[crate::layout::LayoutRequirement],
    bundle: &crate::analyzed_operation::RelationCaseRequirements,
) -> Result<(), crate::CompileError> {
    let plan = RelationCasePlan {
        relation: bundle.key.relation.clone(),
        case: bundle.key.case.clone(),
        activation: bundle.activation,
        activity: bundle.activity,
        boundaries: bundle.boundaries.clone(),
        compiler_requirements: bundle.compiler_requirements.iter().cloned().collect(),
        structural_requirements: bundle.structural_requirements.iter().cloned().collect(),
        runtime_requirements: Vec::new(),
        external_evidence: bundle.external_evidence.clone(),
    };
    let mut facts = lower(
        &plan,
        &bundle.active_sources,
        &bundle
            .layout_requirements
            .iter()
            .cloned()
            .collect::<Vec<_>>(),
    );
    // An offered bundle is evidence to validate, not a declaration to lower:
    // retain even a forbidden source on an inactive case so it is refused.
    facts.sources = bundle
        .active_sources
        .iter()
        .map(super::source::row_key)
        .collect();
    let coverage = crate::coverage::RelationCoveragePlan {
        relation: bundle.key.relation.clone(),
        case: bundle.key.case.clone(),
        activity: bundle.coverage.activity,
        boundaries: bundle.coverage.boundaries.clone(),
        positive: bundle.coverage.positive.iter().cloned().collect(),
        negative: bundle.coverage.negative.iter().cloned().collect(),
        carrier: None,
        projections: bundle.coverage.projections.clone(),
        external_evidence: bundle.coverage.external_evidence.clone(),
    };
    let joined = crate::engine_coverage_join::Joined {
        plan: super::coverage::lowered(&coverage),
        carrier: Some(
            bundle
                .coverage
                .carrier
                .iter()
                .map(super::coverage_join::alternative)
                .collect(),
        ),
        projections: bundle.coverage.projections.keys().copied().collect(),
    };
    let lowered = core::Bundle {
        facts,
        coverage: joined,
        carriers: bundle
            .carrier_assignments
            .iter()
            .map(super::coverage_join::alternative)
            .collect(),
    };
    core::validate(
        super::operation_key(operation),
        &census
            .iter()
            .cloned()
            .map(crate::engine_placement::LayoutKey)
            .collect(),
        &lowered,
    )
    .map_err(
        |defect| crate::CompileError::InvalidRelationCaseRequirements {
            relation: bundle.key.relation.clone(),
            case: bundle.key.case.clone(),
            defect,
        },
    )
}
