//! Attestation placement policy lowering and compatibility projections.
use crate::{
    CompileError,
    carrier::{
        CarrierEligibility, CarrierQuantification, CarrierRole, EligibleCarrier,
        is_sponsor_region_carrier,
    },
    engine_placement as core,
    layout::selected_carrier_requirements,
    placement::{
        ActivationCondition, CarrierMultiplicity, PlacedCarrier, PlacementAssignment,
        PlacementCandidate, PlacementSearchLimits, PlacementSearchReport, RelationCaseKey,
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
    for plan in plans {
        let id = RelationCaseKey {
            relation: plan.relation.clone(),
            case: plan.case.clone(),
        };
        keys.insert(key(&id), id);
    }
    let rows = eligibility
        .iter()
        .map(|analysis| {
            let id = RelationCaseKey {
                relation: analysis.relation.clone(),
                case: analysis.case.clone(),
            };
            keys.insert(key(&id), id.clone());
            for entry in &analysis.eligible {
                carriers.insert(
                    core::CarrierKey::Attestation(entry.carrier.clone()),
                    entry.carrier.clone(),
                );
            }
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
        .collect();
    if let Some(offered) = offered {
        for assignment in &offered.assignments {
            keys.insert(key(&assignment.key()), assignment.key());
            for placed in &assignment.carriers {
                carriers.insert(
                    core::CarrierKey::Attestation(placed.carrier.clone()),
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
    }
}
fn entries(plan: &RelationCasePlan, analysis: &CarrierEligibility) -> Vec<core::Entry> {
    admissible_carriers(plan, analysis)
        .into_iter()
        .map(|entry| core::Entry {
            placed: core::Placed {
                carrier: core::CarrierKey::Attestation(entry.carrier.clone()),
                quantification: entry.quantification,
            },
            layout: selected_carrier_requirements(analysis, entry)
                .into_iter()
                .map(core::LayoutKey)
                .collect(),
        })
        .collect()
}
pub fn key(id: &RelationCaseKey) -> core::Key {
    core::Key {
        relation: super::relation_key(&id.relation),
        case: core::CaseKey::Attestation(id.case.clone()),
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
                        carrier: core::CarrierKey::Attestation(placed.carrier.clone()),
                        quantification: placed.quantification,
                    })
                    .collect(),
            })
            .collect(),
        layout: value
            .layout_requirements
            .iter()
            .cloned()
            .map(core::LayoutKey)
            .collect(),
    }
}
impl Lowered {
    pub fn enumerate(
        &self,
        limits: PlacementSearchLimits,
        report: &mut PlacementSearchReport,
    ) -> Result<Vec<PlacementCandidate>, CompileError> {
        let rows =
            core::enumerate(&self.records, limits, report).map_err(|error| self.refusal(error))?;
        Ok(rows
            .into_iter()
            .map(|row| PlacementCandidate {
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
                layout_requirements: row.layout.into_iter().map(|key| key.0).collect(),
            })
            .collect())
    }
    pub fn refusal(&self, error: core::Error) -> CompileError {
        refusal(error, &self.keys, &self.carriers)
    }
}

pub fn refusal(
    error: core::Error,
    keys: &BTreeMap<core::Key, RelationCaseKey>,
    carriers: &BTreeMap<core::CarrierKey, CarrierRole>,
) -> CompileError {
    match error {
        core::Error::PlacementCensusMismatch {
            missing,
            unexpected,
        } => CompileError::PlacementCensusMismatch {
            missing: missing.iter().map(|key| keys[key].clone()).collect(),
            unexpected: unexpected.iter().map(|key| keys[key].clone()).collect(),
        },
        core::Error::NoEligibleCarrier(key) => {
            let id = &keys[&key];
            CompileError::NoEligibleCarrier {
                relation: id.relation.clone(),
                case: id.case.clone(),
            }
        }
        core::Error::GlobalRelationHasOnlyLocalCarrier(key) => {
            let id = &keys[&key];
            CompileError::GlobalRelationHasOnlyLocalCarrier {
                relation: id.relation.clone(),
                case: id.case.clone(),
            }
        }
        core::Error::UnconditionalRelationOnOptionalCarrier(key) => {
            let id = &keys[&key];
            CompileError::UnconditionalRelationOnOptionalCarrier {
                relation: id.relation.clone(),
                case: id.case.clone(),
            }
        }
        core::Error::DuplicatePlacedCarrier { key, carrier } => {
            let id = &keys[&key];
            CompileError::DuplicatePlacedCarrier {
                relation: id.relation.clone(),
                case: id.case.clone(),
                carrier: carriers[&carrier].clone(),
            }
        }
        core::Error::NonCanonicalCarrierAssignment(key) => {
            let id = &keys[&key];
            CompileError::NonCanonicalCarrierAssignment {
                relation: id.relation.clone(),
                case: id.case.clone(),
            }
        }
        core::Error::UnpermittedCarrierPlacement { key, carrier } => {
            let id = &keys[&key];
            CompileError::UnpermittedCarrierPlacement {
                relation: id.relation.clone(),
                case: id.case.clone(),
                carrier: carriers[&carrier].clone(),
            }
        }
        core::Error::MissingLayoutRequirement(key) => {
            let id = &keys[&key];
            CompileError::MissingLayoutRequirement {
                relation: id.relation.clone(),
                case: id.case.clone(),
            }
        }
        core::Error::UnexpectedLayoutRequirement(keys) => {
            CompileError::UnexpectedLayoutRequirement {
                unexpected: keys.into_iter().map(|key| key.0).collect(),
            }
        }
        core::Error::StateLimit(maximum) => {
            CompileError::PlacementSearchStateLimitExceeded { maximum }
        }
        core::Error::CandidateLimit(maximum) => {
            CompileError::PlacementCandidateLimitExceeded { maximum }
        }
    }
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
