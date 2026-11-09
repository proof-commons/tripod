//! Attestation layout facts and lossless placement-coverage compatibility.
use crate::{
    CompileError,
    carrier::CarrierEligibility,
    coverage::{CarrierAssignmentAlternative, PlanCoverageAnalysis},
    engine_coverage_join as core, engine_placement as placement,
    layout::{LayoutRequirement, selected_carrier_requirements},
    placement::{PlacedCarrier, PlacementCandidate, RelationCaseKey},
};
use std::collections::{BTreeMap, BTreeSet};

fn placed(value: &PlacedCarrier) -> placement::Placed {
    placement::Placed {
        carrier: placement::CarrierKey::Attestation(value.carrier.clone()),
        quantification: value.quantification,
    }
}
pub fn alternative(value: &CarrierAssignmentAlternative) -> core::Alternative {
    core::Alternative {
        carriers: value.carriers.iter().map(placed).collect(),
        layout: value
            .layout
            .iter()
            .cloned()
            .map(placement::LayoutKey)
            .collect(),
    }
}
pub fn attach_requirements(plan: &mut crate::coverage::RelationCoveragePlan) {
    use crate::{
        coverage::{
            CoveragePurpose, CoverageRequirementId, EvidenceRole, PositiveCoverageRequirement,
        },
        engine_coverage::Purpose,
    };
    let carriers = plan
        .carrier
        .iter()
        .flat_map(|value| &value.allowed_assignments)
        .flat_map(|value| &value.carriers)
        .map(|value| (placed(value), value.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut joined = core::Joined {
        plan: super::coverage::lowered(plan),
        carrier: plan
            .carrier
            .as_ref()
            .map(|value| value.allowed_assignments.iter().map(alternative).collect()),
        projections: plan.projections.keys().copied().collect(),
    };
    core::attach_requirements(&mut joined);
    for row in joined.plan.positive {
        let (purpose, role, operands) = match row.purpose {
            Purpose::AcceptedProjection => (
                CoveragePurpose::AcceptedProjection,
                crate::coverage::boundary_role(plan, row.boundary),
                plan.projections[&row.boundary].operands.clone(),
            ),
            Purpose::CarrierExecution(value) => (
                CoveragePurpose::CarrierExecution(carriers[&value].clone()),
                EvidenceRole::TargetExecution,
                Vec::new(),
            ),
            _ => continue,
        };
        plan.positive.push(PositiveCoverageRequirement {
            id: CoverageRequirementId {
                relation: plan.relation.clone(),
                case: plan.case.clone(),
                boundary: row.boundary,
                purpose,
            },
            role,
            representation: None,
            operands,
        });
    }
    plan.positive.sort();
    plan.positive.dedup();
}
fn entries(eligibility: &[CarrierEligibility]) -> BTreeMap<placement::Key, Vec<placement::Entry>> {
    eligibility
        .iter()
        .map(|value| {
            (
                super::placement::key(&RelationCaseKey {
                    relation: value.relation.clone(),
                    case: value.case.clone(),
                }),
                value
                    .eligible
                    .iter()
                    .map(|entry| placement::Entry {
                        placed: placement::Placed {
                            carrier: placement::CarrierKey::Attestation(entry.carrier.clone()),
                            quantification: entry.quantification,
                        },
                        layout: selected_carrier_requirements(value, entry)
                            .into_iter()
                            .map(placement::LayoutKey)
                            .collect(),
                    })
                    .collect(),
            )
        })
        .collect()
}
pub fn carrier_alternatives(
    eligibility: &[CarrierEligibility],
    placements: &[PlacementCandidate],
) -> BTreeMap<RelationCaseKey, BTreeSet<CarrierAssignmentAlternative>> {
    let keys = placements
        .iter()
        .flat_map(|value| &value.assignments)
        .map(|row| (super::placement::key(&row.key()), row.key()))
        .collect::<BTreeMap<_, _>>();
    let carriers = placements
        .iter()
        .flat_map(|value| &value.assignments)
        .flat_map(|row| &row.carriers)
        .map(|value| (placed(value), value.clone()))
        .collect::<BTreeMap<_, _>>();
    core::alternatives(
        &entries(eligibility),
        placements
            .iter()
            .map(super::placement::candidate)
            .flat_map(|value| value.assignments),
    )
    .into_iter()
    .map(|(key, alternatives)| {
        (
            keys[&key].clone(),
            alternatives
                .into_iter()
                .map(|value| CarrierAssignmentAlternative {
                    carriers: value
                        .carriers
                        .iter()
                        .map(|key| carriers[key].clone())
                        .collect(),
                    layout: value.layout.into_iter().map(|key| key.0).collect(),
                })
                .collect(),
        )
    })
    .collect()
}

pub fn validate(
    eligibility: &[CarrierEligibility],
    placements: &[PlacementCandidate],
    layout: &[LayoutRequirement],
    analysis: &PlanCoverageAnalysis,
) -> Result<(), CompileError> {
    let entries = entries(eligibility);
    let expected = core::alternatives(
        &entries,
        placements
            .iter()
            .map(super::placement::candidate)
            .flat_map(|value| value.assignments),
    );
    let plans = analysis
        .plans()
        .map(|plan| core::Joined {
            plan: super::coverage::lowered(plan),
            carrier: plan
                .carrier
                .as_ref()
                .map(|value| value.allowed_assignments.iter().map(alternative).collect()),
            projections: plan.projections.keys().copied().collect(),
        })
        .collect::<Vec<_>>();
    let keys = analysis
        .plans()
        .map(crate::coverage::RelationCoveragePlan::key)
        .chain(
            placements
                .iter()
                .flat_map(|value| &value.assignments)
                .map(crate::placement::PlacementAssignment::key),
        )
        .map(|key| (super::placement::key(&key), key))
        .collect::<BTreeMap<_, _>>();
    core::validate(
        &plans,
        &entries.keys().cloned().collect(),
        &layout.iter().cloned().map(placement::LayoutKey).collect(),
        &expected,
    )
    .map_err(|error| refusal(error, &keys))
}
pub fn refusal(
    error: core::Error,
    keys: &BTreeMap<placement::Key, crate::placement::RelationCaseKey>,
) -> CompileError {
    match error {
        core::Error::UnexpectedCarrier(key) => {
            let key = &keys[&key];
            CompileError::UnexpectedRuntimeCarrierCoverage {
                relation: key.relation.clone(),
                case: key.case.clone(),
            }
        }
        core::Error::MissingCarrier(key) => {
            let key = &keys[&key];
            CompileError::MissingCarrierCoverage {
                relation: key.relation.clone(),
                case: key.case.clone(),
            }
        }
        core::Error::MissingProjection(key) => {
            let key = &keys[&key];
            CompileError::MissingProjectionCoverage {
                relation: key.relation.clone(),
                case: key.case.clone(),
            }
        }
        core::Error::UnexpectedLayout(key) => {
            let key = &keys[&key];
            CompileError::UnexpectedCoverageLayoutRequirement {
                relation: key.relation.clone(),
                case: key.case.clone(),
            }
        }
        core::Error::MissingLayout(key) => {
            let key = &keys[&key];
            CompileError::MissingCoverageLayoutRequirement {
                relation: key.relation.clone(),
                case: key.case.clone(),
            }
        }
    }
}
