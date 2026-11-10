//! Attestation layout facts and lossless placement-coverage compatibility.
use crate::{
    CompileError, coverage::CarrierAssignmentAlternative, engine_coverage_join as core,
    engine_placement as placement, placement::PlacedCarrier,
};
#[cfg(test)]
use crate::{
    carrier::CarrierEligibility,
    coverage::PlanCoverageAnalysis,
    layout::{LayoutRequirement, selected_carrier_requirements},
    placement::{PlacementCandidate, RelationCaseKey},
};
use std::collections::BTreeMap;
#[cfg(test)]
use std::collections::BTreeSet;

fn placed(value: &PlacedCarrier) -> placement::Placed {
    placement::Placed {
        carrier: crate::engine_attestation::identity::attestation_carrier(&value.carrier),
        quantification: value.quantification,
    }
}
pub fn alternative(value: &CarrierAssignmentAlternative) -> core::Alternative {
    core::Alternative {
        carriers: value.carriers.iter().map(placed).collect(),
        layout: value
            .layout
            .iter()
            .map(crate::engine_attestation::identity::attestation_layout)
            .collect(),
    }
}
#[cfg(test)]
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
            Purpose::Owner(_) => continue,
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
#[cfg(test)]
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
                            carrier: crate::engine_attestation::identity::attestation_carrier(
                                &entry.carrier,
                            ),
                            quantification: entry.quantification,
                        },
                        layout: selected_carrier_requirements(value, entry)
                            .into_iter()
                            .map(|value| {
                                crate::engine_attestation::identity::attestation_layout(&value)
                            })
                            .collect(),
                    })
                    .collect(),
            )
        })
        .collect()
}
#[cfg(test)]
pub fn carrier_alternatives(
    eligibility: &[CarrierEligibility],
    placements: &[PlacementCandidate],
) -> Result<BTreeMap<RelationCaseKey, BTreeSet<CarrierAssignmentAlternative>>, CompileError> {
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
    let layouts = eligibility
        .iter()
        .flat_map(|row| {
            row.eligible
                .iter()
                .flat_map(|entry| crate::layout::selected_carrier_requirements(row, entry))
        })
        .map(|value| {
            (
                crate::engine_attestation::identity::attestation_layout(&value),
                value,
            )
        })
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
        Ok((
            keys.get(&key)
                .cloned()
                .ok_or(CompileError::MissingOwnerIdentity)?,
            alternatives
                .into_iter()
                .map(|value| {
                    Ok(CarrierAssignmentAlternative {
                        carriers: value
                            .carriers
                            .iter()
                            .map(|key| {
                                carriers
                                    .get(key)
                                    .cloned()
                                    .ok_or(CompileError::MissingOwnerIdentity)
                            })
                            .collect::<Result<_, _>>()?,
                        layout: value
                            .layout
                            .iter()
                            .map(|key| super::operation::layout_lookup(&layouts, key))
                            .collect::<Result<_, _>>()?,
                    })
                })
                .collect::<Result<_, CompileError>>()?,
        ))
    })
    .collect()
}

#[cfg(test)]
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
        &layout
            .iter()
            .map(crate::engine_attestation::identity::attestation_layout)
            .collect(),
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
