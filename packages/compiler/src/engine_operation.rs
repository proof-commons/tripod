//! Operation-local assembly over concrete owner-produced records.
use crate::{
    engine_case_facts::{self, Bundle},
    engine_cases::Selected,
    engine_coverage, engine_coverage_graph as graph, engine_coverage_join as join,
    engine_placement::{self as placement, CaseKey, Key, LayoutKey},
    engine_records::{OperationKey, RelationKey},
    placement::{PlacementSearchLimits, PlacementSearchReport},
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Operation {
    pub id: OperationKey,
    pub cases: BTreeSet<CaseKey>,
    pub bundles: BTreeMap<Key, Bundle>,
    pub placements: Vec<placement::Candidate>,
    pub layout: BTreeSet<LayoutKey>,
    pub coverage: graph::Projection,
    pub search: PlacementSearchReport,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Placement(Box<placement::Error>),
    Coverage(Box<engine_coverage::Error>),
    Join(join::Error),
    Graph(graph::Error),
    Prerequisite(Box<(Key, Key)>),
    CaseFacts {
        key: Key,
        defect: crate::analyzed_operation::RelationCaseDefect,
    },
    CaseCensus {
        missing: Vec<Key>,
        unexpected: Vec<Key>,
    },
}

/// Each operation gets its own search. No product over operations is formed.
#[allow(dead_code)]
pub fn analyze(
    selected: &Selected,
    prerequisites: &[(RelationKey, RelationKey)],
    limits: PlacementSearchLimits,
) -> Result<BTreeMap<OperationKey, Operation>, Error> {
    let span = tracing::debug_span!(target: "compiler::engine_operation", "analyze", plan_count = crate::instrumentation::count(selected.placement.plans.len()));
    let _entered = span.enter();
    let operations = selected
        .placement
        .plans
        .iter()
        .map(|(key, _)| key.relation.operation)
        .collect::<BTreeSet<_>>();
    let cases = operations
        .iter()
        .map(|operation| {
            (
                *operation,
                selected
                    .placement
                    .plans
                    .iter()
                    .filter(|(key, _)| key.relation.operation == *operation)
                    .map(|(key, _)| key.case.clone())
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect(),
            )
        })
        .collect();
    // Diagnose a dependency between two covered operations before factorizing.
    graph::prerequisites(&cases, &BTreeMap::new(), prerequisites).map_err(Error::Prerequisite)?;
    operations
        .into_iter()
        .map(|operation| {
            analyze_one(selected, prerequisites, operation, limits)
                .map(|result| (operation, result))
        })
        .collect()
}

fn analyze_one(
    selected: &Selected,
    prerequisites: &[(RelationKey, RelationKey)],
    operation: OperationKey,
    limits: PlacementSearchLimits,
) -> Result<Operation, Error> {
    let records = placement::Records {
        plans: selected
            .placement
            .plans
            .iter()
            .filter(|(key, _)| key.relation.operation == operation)
            .cloned()
            .collect(),
        eligibility: selected
            .placement
            .eligibility
            .iter()
            .filter(|row| row.key.relation.operation == operation)
            .cloned()
            .collect(),
    };
    let mut search = PlacementSearchReport::default();
    let placements = placement::enumerate(&records, limits, &mut search)
        .map_err(|error| Error::Placement(Box::new(error)))?;
    let plans = selected
        .coverage
        .iter()
        .filter(|plan| plan.key.relation.operation == operation)
        .cloned()
        .collect::<Vec<_>>();
    let required = records.plans.iter().map(|(key, _)| key.clone()).collect();
    engine_coverage::validate_census(&required, &plans)
        .map_err(|error| Error::Coverage(Box::new(error)))?;
    let entries = records
        .eligibility
        .iter()
        .map(|row| (row.key.clone(), row.entries.clone()))
        .collect::<BTreeMap<_, _>>();
    let alternatives = join::alternatives(
        &entries,
        placements
            .iter()
            .flat_map(|row| row.assignments.iter().cloned()),
    );
    let facts = selected
        .facts
        .iter()
        .filter(|facts| facts.key.relation.operation == operation)
        .collect::<Vec<_>>();
    let layout = facts
        .iter()
        .flat_map(|facts| facts.layout.iter().cloned())
        .collect::<BTreeSet<_>>();
    let joined = plans
        .into_iter()
        .map(|plan| {
            let mut joined = join::Joined {
                carrier: alternatives.get(&plan.key).cloned(),
                projections: join::projection_boundaries(
                    &plan,
                    selected
                        .projections
                        .get(&plan.key)
                        .unwrap_or(&BTreeSet::new()),
                ),
                plan,
            };
            join::attach_requirements(&mut joined);
            joined
        })
        .collect::<Vec<_>>();
    join::validate(
        &joined,
        &entries.keys().cloned().collect(),
        &layout,
        &alternatives,
    )
    .map_err(Error::Join)?;
    let bundles = bundles(operation, &layout, &joined, &facts, &selected.layout_scopes)?;
    let cases = records
        .plans
        .iter()
        .map(|(key, _)| key.case.clone())
        .collect::<BTreeSet<_>>();
    let coverage = assemble_graph(
        &joined,
        &cases,
        prerequisites,
        &selected.collateral,
        operation,
    )?;
    Ok(Operation {
        id: operation,
        cases,
        bundles,
        placements,
        layout,
        coverage,
        search,
    })
}

fn bundles(
    operation: OperationKey,
    layout: &BTreeSet<LayoutKey>,
    joined: &[join::Joined],
    facts: &[&engine_case_facts::Facts],
    layout_scopes: &BTreeMap<LayoutKey, crate::engine_cases::LayoutScope>,
) -> Result<BTreeMap<Key, Bundle>, Error> {
    let required = joined
        .iter()
        .map(|row| row.plan.key.clone())
        .collect::<BTreeSet<_>>();
    let offered = facts
        .iter()
        .map(|row| row.key.clone())
        .collect::<BTreeSet<_>>();
    if required != offered || offered.len() != facts.len() {
        return Err(Error::CaseCensus {
            missing: required.difference(&offered).cloned().collect(),
            unexpected: offered.difference(&required).cloned().collect(),
        });
    }
    let by_key = joined
        .iter()
        .map(|row| (&row.plan.key, row))
        .collect::<BTreeMap<_, _>>();
    facts
        .iter()
        .map(|facts| {
            let coverage = by_key[&facts.key];
            let mut bundle = Bundle {
                facts: (*facts).clone(),
                carriers: coverage.carrier.clone().unwrap_or_default(),
                coverage: coverage.clone(),
            };
            // Case-independent layout obligations remain visible in vacuous
            // cases too. Owners supply the association before any selection.
            bundle.facts.layout.extend(
                layout
                    .iter()
                    .filter(|key| {
                        layout_scopes.get(*key).is_some_and(|scope| {
                            scope.relation.as_ref() == Some(&facts.key.relation)
                                && scope
                                    .case
                                    .as_ref()
                                    .is_none_or(|case| case == &facts.key.case)
                        })
                    })
                    .cloned(),
            );
            engine_case_facts::validate(operation, layout, &bundle).map_err(|defect| {
                Error::CaseFacts {
                    key: facts.key.clone(),
                    defect,
                }
            })?;
            Ok((facts.key.clone(), bundle))
        })
        .collect()
}

fn assemble_graph(
    joined: &[join::Joined],
    cases: &BTreeSet<CaseKey>,
    prerequisites: &[(RelationKey, RelationKey)],
    collateral: &BTreeSet<engine_coverage::Requirement>,
    operation: OperationKey,
) -> Result<graph::Projection, Error> {
    let symbols = joined.iter().map(join::symbols).collect::<Vec<_>>();
    let nodes = graph::definition_census(&symbols)
        .map_err(|key| Error::Graph(graph::Error::DuplicateSymbol(key)))?;
    let relation_symbols = nodes
        .iter()
        .filter_map(|node| {
            node.relation_case
                .as_ref()
                .map(|key| (key.clone(), node.id.clone()))
        })
        .collect();
    let dependencies = graph::prerequisites(
        &BTreeMap::from([(operation, cases.iter().cloned().collect())]),
        &relation_symbols,
        prerequisites,
    )
    .map_err(Error::Prerequisite)?;
    let edges = graph::derive_dependencies(&symbols, &dependencies);
    let mut graph = graph::build(&nodes, &edges).map_err(Error::Graph)?;
    graph.validate_acyclic().map_err(Error::Graph)?;
    let requirements = collateral
        .iter()
        .filter(|row| row.key.relation.operation == operation)
        .map(|row| (row.key.clone(), graph::Key::Requirement(row.clone())))
        .collect::<Vec<_>>();
    let edges = graph.collateral_dependencies(&requirements);
    graph.add_dependencies(&edges).map_err(Error::Graph)?;
    graph.validate_acyclic().map_err(Error::Graph)?;
    Ok(graph.project())
}
