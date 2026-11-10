//! Attestation placement policy lowering and compatibility projections.
#[cfg(test)]
use crate::placement::{
    PlacedCarrier, PlacementAssignment, PlacementSearchLimits, PlacementSearchReport,
};
use crate::{
    CompileError,
    carrier::{
        CarrierEligibility, CarrierQuantification, CarrierRole, EligibleCarrier,
        is_sponsor_region_carrier,
    },
    engine_placement as core,
    layout::selected_carrier_requirements,
    placement::{
        ActivationCondition, CarrierMultiplicity, PlacementCandidate, RelationCaseKey,
        RelationCasePlan, SemanticScope,
    },
    sponsor_region::GATED_ORDINARY_LBTC_ROLE,
};
use realization::TransactionSide;
use std::collections::BTreeMap;

pub struct Lowered {
    pub records: core::Records,
    keys: BTreeMap<core::Key, RelationCaseKey>,
    carriers: BTreeMap<core::CarrierKey, CarrierRole>,
    layouts: BTreeMap<core::LayoutKey, crate::layout::LayoutRequirement>,
}
pub fn lower(
    plans: &[RelationCasePlan],
    eligibility: &[CarrierEligibility],
    offered: Option<&PlacementCandidate>,
) -> Lowered {
    let span = tracing::debug_span!(target: "compiler::engine_attestation", "lower_placement", plan_count = crate::instrumentation::count(plans.len()), eligibility_count = crate::instrumentation::count(eligibility.len()));
    let _entered = span.enter();
    let mut keys = BTreeMap::new();
    let mut carriers = BTreeMap::new();
    let mut layouts = eligibility
        .iter()
        .flat_map(|row| {
            row.eligible
                .iter()
                .flat_map(|entry| selected_carrier_requirements(row, entry))
        })
        .map(|value| {
            (
                crate::engine_attestation::identity::attestation_layout(&value),
                value,
            )
        })
        .collect::<BTreeMap<_, _>>();
    for plan in plans {
        let id = RelationCaseKey {
            relation: plan.relation.clone(),
            case: plan.case.clone(),
        };
        keys.insert(key(&id), id);
    }
    for analysis in eligibility {
        let id = RelationCaseKey {
            relation: analysis.relation.clone(),
            case: analysis.case.clone(),
        };
        keys.insert(key(&id), id);
        for entry in &analysis.eligible {
            carriers.insert(
                crate::engine_attestation::identity::attestation_carrier(&entry.carrier),
                entry.carrier.clone(),
            );
        }
    }
    let rows = eligibility_records(plans, eligibility);
    if let Some(offered) = offered {
        layouts.extend(offered.layout_requirements.iter().map(|value| {
            (
                crate::engine_attestation::identity::attestation_layout(value),
                value.clone(),
            )
        }));
        for assignment in &offered.assignments {
            keys.insert(key(&assignment.key()), assignment.key());
            for placed in &assignment.carriers {
                carriers.insert(
                    crate::engine_attestation::identity::attestation_carrier(&placed.carrier),
                    placed.carrier.clone(),
                );
            }
        }
    }
    Lowered {
        records: core::Records {
            plans: plans
                .iter()
                .map(|plan| {
                    (
                        key(&RelationCaseKey {
                            relation: plan.relation.clone(),
                            case: plan.case.clone(),
                        }),
                        !plan.runtime_requirements.is_empty(),
                    )
                })
                .collect(),
            eligibility: rows,
        },
        keys,
        carriers,
        layouts,
    }
}
pub fn eligibility_records(
    plans: &[RelationCasePlan],
    eligibility: &[CarrierEligibility],
) -> Vec<core::Eligibility> {
    eligibility
        .iter()
        .map(|analysis| {
            let id = RelationCaseKey {
                relation: analysis.relation.clone(),
                case: analysis.case.clone(),
            };
            let plan = plans
                .iter()
                .find(|plan| plan.relation == analysis.relation && plan.case == analysis.case);
            let validation_entries = plan.map_or_else(Vec::new, |plan| entries(plan, analysis));
            let options = plans
                .iter()
                .rev()
                .find(|plan| plan.relation == analysis.relation && plan.case == analysis.case)
                .map_or_else(Vec::new, |plan| entries(plan, analysis));
            core::Eligibility {
                key: key(&id),
                multiplicity: analysis.multiplicity,
                entries: validation_entries,
                options,
                empty_cause: plan.map_or(core::EmptyCause::NoEligible, |plan| {
                    empty_cause(plan, analysis)
                }),
            }
        })
        .collect()
}

fn entries(plan: &RelationCasePlan, analysis: &CarrierEligibility) -> Vec<core::Entry> {
    admissible_carriers(plan, analysis)
        .into_iter()
        .map(|entry| core::Entry {
            placed: core::Placed {
                carrier: crate::engine_attestation::identity::attestation_carrier(&entry.carrier),
                quantification: entry.quantification,
            },
            layout: selected_carrier_requirements(analysis, entry)
                .into_iter()
                .map(|value| crate::engine_attestation::identity::attestation_layout(&value))
                .collect(),
        })
        .collect()
}
pub fn key(id: &RelationCaseKey) -> core::Key {
    core::Key {
        relation: super::relation_key(&id.relation),
        case: crate::engine_attestation::identity::attestation_case(&id.case),
    }
}
pub fn candidate(value: &PlacementCandidate) -> core::Candidate {
    core::Candidate {
        assignments: value
            .assignments
            .iter()
            .map(|row| core::Assignment {
                key: key(&row.key()),
                carriers: row
                    .carriers
                    .iter()
                    .map(|placed| core::Placed {
                        carrier: crate::engine_attestation::identity::attestation_carrier(
                            &placed.carrier,
                        ),
                        quantification: placed.quantification,
                    })
                    .collect(),
            })
            .collect(),
        layout: value
            .layout_requirements
            .iter()
            .map(crate::engine_attestation::identity::attestation_layout)
            .collect(),
    }
}
impl Lowered {
    #[cfg(test)]
    pub fn enumerate(
        &self,
        limits: PlacementSearchLimits,
        report: &mut PlacementSearchReport,
    ) -> Result<Vec<PlacementCandidate>, CompileError> {
        let rows =
            core::enumerate(&self.records, limits, report).map_err(|error| self.refusal(error))?;
        rows.into_iter()
            .map(|row| {
                Ok(PlacementCandidate {
                    assignments: row
                        .assignments
                        .into_iter()
                        .map(|assignment| {
                            let id = &self.keys[&assignment.key];
                            PlacementAssignment {
                                relation: id.relation.clone(),
                                case: id.case.clone(),
                                carriers: assignment
                                    .carriers
                                    .into_iter()
                                    .map(|placed| PlacedCarrier {
                                        carrier: self.carriers[&placed.carrier].clone(),
                                        quantification: placed.quantification,
                                    })
                                    .collect(),
                            }
                        })
                        .collect(),
                    layout_requirements: row
                        .layout
                        .iter()
                        .map(|key| super::operation::layout_lookup(&self.layouts, key))
                        .collect::<Result<_, _>>()?,
                })
            })
            .collect()
    }
    pub fn refusal(&self, error: core::Error) -> CompileError {
        refusal(error, &self.keys, &self.carriers, &self.layouts)
    }
}

pub fn refusal(
    error: core::Error,
    keys: &BTreeMap<core::Key, RelationCaseKey>,
    carriers: &BTreeMap<core::CarrierKey, CarrierRole>,
    layouts: &BTreeMap<core::LayoutKey, crate::layout::LayoutRequirement>,
) -> CompileError {
    let id = |key: &core::Key| keys.get(key).ok_or(CompileError::MissingOwnerIdentity);
    let carrier_value = |key: &core::CarrierKey| {
        carriers
            .get(key)
            .cloned()
            .ok_or(CompileError::MissingOwnerIdentity)
    };
    let restore = || -> Result<CompileError, CompileError> {
        Ok(match error {
            core::Error::PlacementCensusMismatch {
                missing,
                unexpected,
            } => CompileError::PlacementCensusMismatch {
                missing: missing
                    .iter()
                    .map(|key| id(key).cloned())
                    .collect::<Result<_, _>>()?,
                unexpected: unexpected
                    .iter()
                    .map(|key| id(key).cloned())
                    .collect::<Result<_, _>>()?,
            },
            core::Error::NoEligibleCarrier(key) => {
                let id = id(&key)?;
                CompileError::NoEligibleCarrier {
                    relation: id.relation.clone(),
                    case: id.case.clone(),
                }
            }
            core::Error::GlobalRelationHasOnlyLocalCarrier(key) => {
                let id = id(&key)?;
                CompileError::GlobalRelationHasOnlyLocalCarrier {
                    relation: id.relation.clone(),
                    case: id.case.clone(),
                }
            }
            core::Error::UnconditionalRelationOnOptionalCarrier(key) => {
                let id = id(&key)?;
                CompileError::UnconditionalRelationOnOptionalCarrier {
                    relation: id.relation.clone(),
                    case: id.case.clone(),
                }
            }
            core::Error::DuplicatePlacedCarrier { key, carrier } => {
                let id = id(&key)?;
                CompileError::DuplicatePlacedCarrier {
                    relation: id.relation.clone(),
                    case: id.case.clone(),
                    carrier: carrier_value(&carrier)?,
                }
            }
            core::Error::NonCanonicalCarrierAssignment(key) => {
                let id = id(&key)?;
                CompileError::NonCanonicalCarrierAssignment {
                    relation: id.relation.clone(),
                    case: id.case.clone(),
                }
            }
            core::Error::UnpermittedCarrierPlacement { key, carrier } => {
                let id = id(&key)?;
                CompileError::UnpermittedCarrierPlacement {
                    relation: id.relation.clone(),
                    case: id.case.clone(),
                    carrier: carrier_value(&carrier)?,
                }
            }
            core::Error::MissingLayoutRequirement(key) => {
                let id = id(&key)?;
                CompileError::MissingLayoutRequirement {
                    relation: id.relation.clone(),
                    case: id.case.clone(),
                }
            }
            core::Error::UnexpectedLayoutRequirement(keys) => {
                CompileError::UnexpectedLayoutRequirement {
                    unexpected: keys
                        .iter()
                        .map(|key| super::operation::layout_lookup(layouts, key))
                        .collect::<Result<_, _>>()?,
                }
            }
            core::Error::StateLimit(maximum) => {
                CompileError::PlacementSearchStateLimitExceeded { maximum }
            }
            core::Error::CandidateLimit(maximum) => {
                CompileError::PlacementCandidateLimitExceeded { maximum }
            }
        })
    };
    restore().unwrap_or_else(|error| error)
}

fn admissible_carriers<'a>(
    plan: &RelationCasePlan,
    analysis: &'a CarrierEligibility,
) -> Vec<&'a EligibleCarrier> {
    let unconditional = plan.activation == ActivationCondition::Always;

    analysis
        .eligible
        .iter()
        .filter(|entry| scope_admits_carrier(analysis.scope, &entry.carrier))
        .filter(|entry| multiplicity_admits(analysis.multiplicity, entry.quantification))
        // An optional sponsor carrier may carry its own conditional
        // relations; it may never be the carrier of a relation that
        // holds whether or not the sponsor region exists.
        .filter(|entry| {
            !(unconditional && is_sponsor_region_carrier(&entry.carrier, GATED_ORDINARY_LBTC_ROLE))
        })
        .collect()
}

/// Whether one carrier role can discharge one semantic scope.
///
/// A family-global or transaction-global obligation needs a complete
/// carrier: a per-member role sees one member, so it can never establish
/// a property of the whole family or the whole transaction. The
/// backend-structural and external-evidence roles are not runtime
/// carriers at all.
fn scope_admits_carrier(scope: SemanticScope, carrier: &CarrierRole) -> bool {
    match carrier {
        CarrierRole::OperationGlobal { .. } => true,

        CarrierRole::InputFamilyCoordinator { object: family } => match scope {
            SemanticScope::MemberLocal { side, object }
            | SemanticScope::FamilyGlobal { side, object } => {
                side == TransactionSide::Input && *family == object
            }
            SemanticScope::TransactionSideGlobal { .. }
            | SemanticScope::ObjectFamilyGlobal { .. }
            | SemanticScope::TransactionGlobal => false,
        },

        CarrierRole::EveryInputFamilyMember { object: family } => match scope {
            SemanticScope::MemberLocal { side, object } => {
                side == TransactionSide::Input && *family == object
            }
            SemanticScope::FamilyGlobal { .. }
            | SemanticScope::TransactionSideGlobal { .. }
            | SemanticScope::ObjectFamilyGlobal { .. }
            | SemanticScope::TransactionGlobal => false,
        },

        CarrierRole::BackendStructural { .. } | CarrierRole::ExternalEvidence { .. } => false,
    }
}

/// Whether one quantification can discharge one multiplicity.
///
/// The every-member hard constraint reads the typed quantification, not
/// the carrier variant: a coordinator is admissible for an every-member
/// obligation only as the complete-family proof wave-2 types it as,
/// never as one carrier standing in for every member.
const fn multiplicity_admits(
    multiplicity: CarrierMultiplicity,
    quantification: CarrierQuantification,
) -> bool {
    match multiplicity {
        CarrierMultiplicity::EveryMember => matches!(
            quantification,
            CarrierQuantification::PerMember | CarrierQuantification::CompleteFamilyProof
        ),
        CarrierMultiplicity::ExactlyOne
        | CarrierMultiplicity::AtLeastOne
        | CarrierMultiplicity::DeliberateDuplication => {
            matches!(quantification, CarrierQuantification::Single)
        }
    }
}

/// The typed reason one obligation retained no admissible carrier.
///
/// The scope defect is reported first where both apply: a global
/// relation offered only local carriers is a stronger statement about
/// the analysis than the activation mismatch it also implies.
fn empty_cause(plan: &RelationCasePlan, analysis: &CarrierEligibility) -> core::EmptyCause {
    let local = |carrier: &CarrierRole| {
        matches!(
            carrier,
            CarrierRole::EveryInputFamilyMember { .. } | CarrierRole::InputFamilyCoordinator { .. }
        )
    };

    if analysis.eligible.is_empty() {
        return core::EmptyCause::NoEligible;
    }

    if !matches!(analysis.scope, SemanticScope::MemberLocal { .. })
        && analysis.eligible.iter().all(|entry| local(&entry.carrier))
    {
        return core::EmptyCause::GlobalOnlyLocal;
    }

    if plan.activation == ActivationCondition::Always
        && analysis
            .eligible
            .iter()
            .all(|entry| is_sponsor_region_carrier(&entry.carrier, GATED_ORDINARY_LBTC_ROLE))
    {
        return core::EmptyCause::UnconditionalOptional;
    }

    core::EmptyCause::NoEligible
}

#[cfg(test)]
mod identity_tests {
    use super::*;

    fn fixture() -> (
        Lowered,
        RelationCaseKey,
        CarrierRole,
        crate::layout::LayoutRequirement,
    ) {
        let operation = architecture::OperationId::CompactAsh;
        let relation = realization::RelationId::new(
            operation,
            realization::RelationKind::Recognition,
            realization::RelationSubject::ObjectFamily {
                side: TransactionSide::Input,
                object: architecture::ObjectId::Ash,
            },
        );
        let case = crate::case::ExecutionCaseId {
            operation,
            sponsor: crate::case::SponsorCase::Absent,
            representations: BTreeMap::new(),
        };
        let id = RelationCaseKey {
            relation: relation.clone(),
            case,
        };
        let key = key(&id);
        let carrier = CarrierRole::EveryInputFamilyMember {
            object: architecture::ObjectId::Ash,
        };
        let carrier_key = crate::engine_attestation::identity::attestation_carrier(&carrier);
        let layout = crate::layout::LayoutRequirement::AuthenticateFamilyCensus {
            relation,
            side: TransactionSide::Input,
            object: architecture::ObjectId::Ash,
        };
        let layout_key = crate::engine_attestation::identity::attestation_layout(&layout);
        let mut sibling_id = id.clone();
        sibling_id.case.sponsor = crate::case::SponsorCase::Present;
        let sibling_key = crate::engine_attestation::placement::key(&sibling_id);
        let keys = BTreeMap::from([(key, id.clone()), (sibling_key, sibling_id)]);
        let sibling_carrier = CarrierRole::InputFamilyCoordinator {
            object: architecture::ObjectId::Ash,
        };
        let carriers = BTreeMap::from([
            (carrier_key, carrier.clone()),
            (
                crate::engine_attestation::identity::attestation_carrier(&sibling_carrier),
                sibling_carrier,
            ),
        ]);
        let mut sibling_layout = layout.clone();
        if let crate::layout::LayoutRequirement::AuthenticateFamilyCensus { side, .. } =
            &mut sibling_layout
        {
            *side = TransactionSide::Output;
        }
        let layouts = BTreeMap::from([
            (layout_key, layout.clone()),
            (
                crate::engine_attestation::identity::attestation_layout(&sibling_layout),
                sibling_layout,
            ),
        ]);
        (
            Lowered {
                records: core::Records {
                    plans: Vec::new(),
                    eligibility: Vec::new(),
                },
                keys,
                carriers,
                layouts,
            },
            id,
            carrier,
            layout,
        )
    }

    #[test]
    fn opaque_placement_refusals_restore_exact_owner_values() {
        let (lowered, id, carrier, layout) = fixture();
        let relation = id.relation.clone();
        let case = id.case.clone();
        let key = key(&id);
        let carrier_key = crate::engine_attestation::identity::attestation_carrier(&carrier);
        let layout_key = crate::engine_attestation::identity::attestation_layout(&layout);
        let subjects = [
            (
                core::Error::PlacementCensusMismatch {
                    missing: vec![key.clone()],
                    unexpected: vec![key.clone()],
                },
                CompileError::PlacementCensusMismatch {
                    missing: vec![id.clone()],
                    unexpected: vec![id],
                },
            ),
            (
                core::Error::NoEligibleCarrier(key.clone()),
                CompileError::NoEligibleCarrier {
                    relation: relation.clone(),
                    case: case.clone(),
                },
            ),
            (
                core::Error::GlobalRelationHasOnlyLocalCarrier(key.clone()),
                CompileError::GlobalRelationHasOnlyLocalCarrier {
                    relation: relation.clone(),
                    case: case.clone(),
                },
            ),
            (
                core::Error::UnconditionalRelationOnOptionalCarrier(key.clone()),
                CompileError::UnconditionalRelationOnOptionalCarrier {
                    relation: relation.clone(),
                    case: case.clone(),
                },
            ),
            (
                core::Error::DuplicatePlacedCarrier {
                    key: key.clone(),
                    carrier: carrier_key.clone(),
                },
                CompileError::DuplicatePlacedCarrier {
                    relation: relation.clone(),
                    case: case.clone(),
                    carrier: carrier.clone(),
                },
            ),
            (
                core::Error::NonCanonicalCarrierAssignment(key.clone()),
                CompileError::NonCanonicalCarrierAssignment {
                    relation: relation.clone(),
                    case: case.clone(),
                },
            ),
            (
                core::Error::UnpermittedCarrierPlacement {
                    key: key.clone(),
                    carrier: carrier_key,
                },
                CompileError::UnpermittedCarrierPlacement {
                    relation: relation.clone(),
                    case: case.clone(),
                    carrier,
                },
            ),
            (
                core::Error::MissingLayoutRequirement(key),
                CompileError::MissingLayoutRequirement { relation, case },
            ),
            (
                core::Error::UnexpectedLayoutRequirement(vec![layout_key]),
                CompileError::UnexpectedLayoutRequirement {
                    unexpected: vec![layout],
                },
            ),
            (
                core::Error::StateLimit(17),
                CompileError::PlacementSearchStateLimitExceeded { maximum: 17 },
            ),
            (
                core::Error::CandidateLimit(19),
                CompileError::PlacementCandidateLimitExceeded { maximum: 19 },
            ),
        ];
        for (subject, expected) in subjects {
            assert_eq!(lowered.refusal(subject), expected);
        }
        assert_sibling_restoration(&lowered);
    }

    fn assert_sibling_restoration(lowered: &Lowered) {
        // Exercise every sibling too: returning the first registered value
        // regardless of the requested key must not satisfy these controls.
        for (key, id) in &lowered.keys {
            assert_eq!(
                lowered.refusal(core::Error::NoEligibleCarrier(key.clone())),
                CompileError::NoEligibleCarrier {
                    relation: id.relation.clone(),
                    case: id.case.clone()
                }
            );
            for (carrier_key, carrier) in &lowered.carriers {
                assert_eq!(
                    lowered.refusal(core::Error::UnpermittedCarrierPlacement {
                        key: key.clone(),
                        carrier: carrier_key.clone()
                    }),
                    CompileError::UnpermittedCarrierPlacement {
                        relation: id.relation.clone(),
                        case: id.case.clone(),
                        carrier: carrier.clone()
                    }
                );
            }
        }
        for (key, layout) in &lowered.layouts {
            assert_eq!(
                lowered.refusal(core::Error::UnexpectedLayoutRequirement(vec![key.clone()])),
                CompileError::UnexpectedLayoutRequirement {
                    unexpected: vec![layout.clone()]
                }
            );
        }
    }

    #[test]
    fn opaque_placement_refusals_refuse_missing_reverse_entries() {
        let (mut lowered, id, carrier, layout) = fixture();
        let case_error = core::Error::NoEligibleCarrier(key(&id));
        let carrier_error = core::Error::UnpermittedCarrierPlacement {
            key: key(&id),
            carrier: crate::engine_attestation::identity::attestation_carrier(&carrier),
        };
        let layout_error = core::Error::UnexpectedLayoutRequirement(vec![
            crate::engine_attestation::identity::attestation_layout(&layout),
        ]);
        assert_eq!(
            lowered.refusal(case_error.clone()),
            CompileError::NoEligibleCarrier {
                relation: id.relation.clone(),
                case: id.case.clone()
            }
        );
        assert_eq!(
            lowered.refusal(carrier_error.clone()),
            CompileError::UnpermittedCarrierPlacement {
                relation: id.relation,
                case: id.case,
                carrier
            }
        );
        assert_eq!(
            lowered.refusal(layout_error.clone()),
            CompileError::UnexpectedLayoutRequirement {
                unexpected: vec![layout]
            }
        );
        let keys = std::mem::take(&mut lowered.keys);
        assert_eq!(
            lowered.refusal(case_error),
            CompileError::MissingOwnerIdentity
        );
        lowered.keys = keys;
        lowered.carriers.clear();
        assert_eq!(
            lowered.refusal(carrier_error),
            CompileError::MissingOwnerIdentity
        );
        lowered.layouts.clear();
        assert_eq!(
            lowered.refusal(layout_error),
            CompileError::MissingOwnerIdentity
        );
    }
}
