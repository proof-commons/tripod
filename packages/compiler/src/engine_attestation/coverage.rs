//! Attestation coverage templates and lossless refusal projection.
#[cfg(test)]
use crate::coverage::PlanCoverageAnalysis;
use crate::{
    CompileError,
    case::conserved_amount_visibility,
    coverage::{
        CoverageBoundary, CoveragePurpose, CoverageRequirementId, EvidenceRole,
        NegativeCoverageRequirement, PositiveCoverageRequirement, RelationCoveragePlan,
        boundary_mutations, boundary_subjects, collateral, negative_purpose, positive_purpose,
        relation_mutations, relation_representation,
    },
    engine_coverage as core,
    placement::{RelationActivity, RelationCaseKey, RelationCasePlan},
    source::relation_operands,
};
use realization::RelationDeclaration;
use std::collections::BTreeMap;
#[cfg(test)]
use std::collections::BTreeSet;
pub fn derive_relation_coverage(
    declaration: &RelationDeclaration,
    plan: &RelationCasePlan,
) -> Result<RelationCoveragePlan, CompileError> {
    let operands = relation_operands(declaration)?;
    let representation = relation_representation(&declaration.relation, &plan.case);
    let mut mutations = relation_mutations(
        &declaration.relation,
        conserved_amount_visibility(declaration, &plan.case)?,
    );
    if plan.boundaries.contains(&CoverageBoundary::ModelScope) {
        for (boundary, _) in &mut mutations {
            *boundary = CoverageBoundary::ModelScope;
        }
    }

    let key = super::placement::key(&RelationCaseKey {
        relation: plan.relation.clone(),
        case: plan.case.clone(),
    });
    let mut positives = BTreeMap::new();
    let mut negatives = BTreeMap::new();
    let mut inactive = Vec::new();
    let mut subjects = Vec::new();
    let id = |boundary, purpose| CoverageRequirementId {
        relation: plan.relation.clone(),
        case: plan.case.clone(),
        boundary,
        purpose,
    };
    for boundary in &plan.boundaries {
        let row = PositiveCoverageRequirement {
            id: id(*boundary, CoveragePurpose::InactiveAccept),
            role: EvidenceRole::InactiveCaseAcceptance,
            representation,
            operands: Vec::new(),
        };
        let identity = requirement(&row.id);
        inactive.push(identity.clone());
        positives.insert(identity, row);
        for (role, subject) in boundary_subjects(*boundary, plan) {
            let row = PositiveCoverageRequirement {
                id: id(*boundary, positive_purpose(*boundary, subject.as_ref())),
                role: role.clone(),
                representation,
                operands: operands.clone(),
            };
            let positive = requirement(&row.id);
            positives.insert(positive.clone(), row);
            let mut negative = Vec::new();
            for mutation in boundary_mutations(&mutations, *boundary) {
                let row = NegativeCoverageRequirement {
                    id: id(
                        *boundary,
                        negative_purpose(*boundary, subject.as_ref(), mutation),
                    ),
                    role: role.clone(),
                    mutation,
                    collateral: collateral(*boundary),
                };
                let identity = requirement(&row.id);
                negative.push(identity.clone());
                negatives.insert(identity, row);
            }
            subjects.push(core::Subject { positive, negative });
        }
    }
    let derived = core::derive(core::Matrix {
        key,
        active: plan.activity == RelationActivity::Active,
        boundaries: plan.boundaries.clone(),
        inactive,
        subjects,
        evidence: plan
            .external_evidence
            .iter()
            .map(super::proof::evidence_key)
            .collect(),
    });
    let mut positive = derived
        .positive
        .iter()
        .map(|key| positives[key].clone())
        .collect::<Vec<_>>();
    let mut negative = derived
        .negative
        .iter()
        .map(|key| negatives[key].clone())
        .collect::<Vec<_>>();
    positive.sort();
    negative.sort();

    Ok(RelationCoveragePlan {
        relation: plan.relation.clone(),
        case: plan.case.clone(),
        activity: plan.activity,
        boundaries: plan.boundaries.clone(),
        positive,
        negative,
        // The carrier obligation and the accepted projections are bound
        // from the plan's feasible placements, which this per-relation
        // derivation does not see.
        carrier: None,
        projections: BTreeMap::new(),
        external_evidence: plan.external_evidence.clone(),
    })
}

pub fn requirement(id: &CoverageRequirementId) -> core::Requirement {
    core::Requirement {
        key: super::placement::key(&id.key()),
        boundary: id.boundary,
        purpose: core::Purpose::Attestation(id.purpose.clone()),
        inactive: id.purpose == CoveragePurpose::InactiveAccept,
    }
}
pub fn lowered(plan: &RelationCoveragePlan) -> core::Plan {
    core::Plan {
        key: super::placement::key(&plan.key()),
        active: plan.activity == RelationActivity::Active,
        boundaries: plan.boundaries.clone(),
        positive: plan
            .positive
            .iter()
            .map(|row| requirement(&row.id))
            .collect(),
        negative: plan
            .negative
            .iter()
            .map(|row| requirement(&row.id))
            .collect(),
        evidence: plan
            .external_evidence
            .iter()
            .map(super::proof::evidence_key)
            .collect(),
    }
}
#[cfg(test)]
pub fn validate_census(
    plans: &[RelationCasePlan],
    analysis: &PlanCoverageAnalysis,
) -> Result<(), CompileError> {
    let keys = plans
        .iter()
        .map(|plan| RelationCaseKey {
            relation: plan.relation.clone(),
            case: plan.case.clone(),
        })
        .chain(analysis.plans().map(RelationCoveragePlan::key))
        .map(|key| (super::placement::key(&key), key))
        .collect::<BTreeMap<_, _>>();
    let required = plans
        .iter()
        .map(|plan| {
            super::placement::key(&RelationCaseKey {
                relation: plan.relation.clone(),
                case: plan.case.clone(),
            })
        })
        .collect::<BTreeSet<_>>();
    let offered = analysis.plans().map(lowered).collect::<Vec<_>>();
    let ids = analysis
        .plans()
        .flat_map(RelationCoveragePlan::requirement_ids)
        .map(|id| (requirement(id), id.clone()))
        .collect();
    core::validate_census(&required, &offered).map_err(|error| refusal(error, &keys, &ids))
}
pub fn refusal(
    error: core::Error,
    keys: &BTreeMap<crate::engine_placement::Key, RelationCaseKey>,
    ids: &BTreeMap<core::Requirement, CoverageRequirementId>,
) -> CompileError {
    match error {
        core::Error::DuplicateRequirement(id) => CompileError::DuplicateCoverageRequirement {
            requirement: ids[&id].clone(),
        },
        core::Error::CensusMismatch {
            missing,
            unexpected,
        } => CompileError::CoverageCensusMismatch {
            missing: missing.iter().map(|key| keys[key].clone()).collect(),
            unexpected: unexpected.iter().map(|key| keys[key].clone()).collect(),
        },
        core::Error::MissingInactive(key) => {
            let key = &keys[&key];
            CompileError::MissingInactiveCoverage {
                relation: key.relation.clone(),
                case: key.case.clone(),
            }
        }
        core::Error::UnexpectedRejection(key) => {
            let key = &keys[&key];
            CompileError::UnexpectedCoverageRejection {
                relation: key.relation.clone(),
                case: key.case.clone(),
            }
        }
        core::Error::MissingPositive { key, boundary } => {
            let key = &keys[&key];
            CompileError::MissingPositiveCoverage {
                relation: key.relation.clone(),
                case: key.case.clone(),
                boundary,
            }
        }
        core::Error::MissingNegative { key, boundary } => {
            let key = &keys[&key];
            CompileError::MissingNegativeCoverage {
                relation: key.relation.clone(),
                case: key.case.clone(),
                boundary,
            }
        }
    }
}
