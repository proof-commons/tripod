//! Placement-to-coverage joins over finite owner records.
use crate::{
    coverage::CoverageBoundary,
    engine_coverage::Plan,
    engine_placement::{Assignment, Entry, Key, LayoutKey, Placed},
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Alternative {
    pub carriers: BTreeSet<Placed>,
    pub layout: BTreeSet<LayoutKey>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Joined {
    pub plan: Plan,
    pub carrier: Option<BTreeSet<Alternative>>,
    pub projections: BTreeSet<CoverageBoundary>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    UnexpectedCarrier(Key),
    MissingCarrier(Key),
    MissingProjection(Key),
    UnexpectedLayout(Key),
    MissingLayout(Key),
}

/// Attach the common accepted-projection and carrier-execution requirements.
pub fn attach_requirements(joined: &mut Joined) {
    use crate::engine_coverage::{Purpose, Requirement};
    let span = tracing::debug_span!(target: "compiler::engine_coverage_join", "attach_requirements", projection_count = crate::instrumentation::count(joined.projections.len()));
    let _entered = span.enter();
    for boundary in &joined.projections {
        joined.plan.positive.push(Requirement {
            key: joined.plan.key.clone(),
            boundary: *boundary,
            purpose: Purpose::AcceptedProjection,
            inactive: false,
        });
    }
    for placed in joined
        .carrier
        .iter()
        .flatten()
        .flat_map(|alternative| &alternative.carriers)
    {
        joined.plan.positive.push(Requirement {
            key: joined.plan.key.clone(),
            boundary: CoverageBoundary::RuntimeCarrier,
            purpose: Purpose::CarrierExecution(placed.clone()),
            inactive: false,
        });
    }
    joined.plan.positive.sort();
    joined.plan.positive.dedup();
}

/// Translate the concrete joined interface into graph symbols without an owner.
#[allow(dead_code)]
pub fn symbols(joined: &Joined) -> crate::engine_coverage_graph::PlanSymbols {
    use crate::{
        coverage_graph::CoverageNodeClass as Class,
        engine_coverage::{Purpose, Requirement},
        engine_coverage_graph::{Key as Symbol, Node, PlanSymbols},
    };
    let plan = &joined.plan;
    let operation = plan.key.relation.operation;
    let node = |id, class| Node {
        id,
        operation,
        class,
        relation_case: None,
        active: None,
    };
    let relation = Node {
        relation_case: Some(plan.key.clone()),
        active: Some(plan.active),
        ..node(Symbol::Relation(plan.key.clone()), Class::RelationCase)
    };
    let requirements = plan
        .positive
        .iter()
        .chain(&plan.negative)
        .map(|row| node(Symbol::Requirement(row.clone()), Class::Requirement))
        .collect();
    let mut carriers = Vec::new();
    let mut layouts = Vec::new();
    for alternative in joined.carrier.iter().flatten() {
        let layout = alternative
            .layout
            .iter()
            .map(|row| node(Symbol::Layout(row.clone()), Class::Layout))
            .collect::<Vec<_>>();
        layouts.extend(layout.iter().cloned());
        for placed in &alternative.carriers {
            carriers.push((
                node(
                    Symbol::Carrier {
                        key: plan.key.clone(),
                        placed: placed.clone(),
                    },
                    Class::Carrier,
                ),
                layout.clone(),
            ));
        }
    }
    let projections = joined
        .projections
        .iter()
        .map(|boundary| {
            Symbol::Requirement(Requirement {
                key: plan.key.clone(),
                boundary: *boundary,
                purpose: Purpose::AcceptedProjection,
                inactive: false,
            })
        })
        .collect();
    let evidence = plan
        .evidence
        .iter()
        .map(|value| node(Symbol::Evidence(value.clone()), Class::ExternalEvidence))
        .collect();
    PlanSymbols {
        relation,
        requirements,
        carriers,
        layouts,
        projections,
        evidence,
    }
}

pub fn alternatives(
    entries: &BTreeMap<Key, Vec<Entry>>,
    assignments: impl Iterator<Item = Assignment>,
) -> BTreeMap<Key, BTreeSet<Alternative>> {
    let span = tracing::debug_span!(target: "compiler::engine_coverage_join", "alternatives", eligibility_count = crate::instrumentation::count(entries.len()));
    let _entered = span.enter();
    let mut result: BTreeMap<Key, BTreeSet<Alternative>> = BTreeMap::new();
    let mut seen = BTreeSet::new();
    for assignment in assignments {
        if !seen.insert(assignment.clone()) {
            continue;
        }
        let Some(eligible) = entries.get(&assignment.key) else {
            continue;
        };
        let layout = assignment
            .carriers
            .iter()
            .filter_map(|carrier| eligible.iter().find(|entry| &entry.placed == carrier))
            .flat_map(|entry| entry.layout.iter().cloned())
            .collect();
        result
            .entry(assignment.key)
            .or_default()
            .insert(Alternative {
                carriers: assignment.carriers.into_iter().collect(),
                layout,
            });
    }
    result
}

pub fn projection_boundaries(
    plan: &Plan,
    available: &BTreeSet<CoverageBoundary>,
) -> BTreeSet<CoverageBoundary> {
    if plan.active {
        plan.boundaries.intersection(available).copied().collect()
    } else {
        BTreeSet::new()
    }
}

pub fn validate(
    plans: &[Joined],
    carried: &BTreeSet<Key>,
    stated: &BTreeSet<LayoutKey>,
    expected: &BTreeMap<Key, BTreeSet<Alternative>>,
) -> Result<(), Error> {
    let span = tracing::debug_span!(target: "compiler::engine_coverage_join", "validate", plan_count = crate::instrumentation::count(plans.len()), layout_count = crate::instrumentation::count(stated.len()));
    let _entered = span.enter();
    for joined in plans {
        let plan = &joined.plan;
        let runtime = plan.active
            && plan.boundaries.contains(&CoverageBoundary::RuntimeCarrier)
            && carried.contains(&plan.key);
        if joined.carrier.is_some() && !runtime {
            return Err(Error::UnexpectedCarrier(plan.key.clone()));
        }
        if joined.carrier.is_none() && runtime {
            return Err(Error::MissingCarrier(plan.key.clone()));
        }
        if runtime
            && !joined
                .projections
                .contains(&CoverageBoundary::RuntimeCarrier)
        {
            return Err(Error::MissingProjection(plan.key.clone()));
        }
        if joined
            .carrier
            .iter()
            .flatten()
            .any(|alternative| !alternative.layout.is_subset(stated))
        {
            return Err(Error::UnexpectedLayout(plan.key.clone()));
        }
    }
    let index = plans
        .iter()
        .map(|joined| (&joined.plan.key, joined))
        .collect::<BTreeMap<_, _>>();
    for (key, alternatives) in expected {
        let covered = index
            .get(key)
            .and_then(|plan| plan.carrier.as_ref())
            .ok_or_else(|| Error::MissingCarrier(key.clone()))?;
        if !alternatives.is_subset(covered) {
            return Err(Error::MissingLayout(key.clone()));
        }
    }
    Ok(())
}
