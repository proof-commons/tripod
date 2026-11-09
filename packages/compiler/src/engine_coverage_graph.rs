//! Shared definition census, dependency resolution and collateral traversal.
use crate::{
    coverage_graph::{CoverageEdge, CoverageNodeClass},
    engine_coverage::Requirement,
    engine_placement::Key as CaseKey,
    engine_records::OperationKey,
    engine_source::Evidence,
};
use petgraph::{
    Direction,
    algo::kosaraju_scc,
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
// Neutral symbols are consumed by shared coverage assembly.
pub enum Key {
    #[cfg(test)]
    Attestation(crate::coverage_graph::CoverageNodeId),
    Relation(CaseKey),
    Requirement(Requirement),
    Evidence(Evidence),
    Carrier {
        key: CaseKey,
        placed: crate::engine_placement::Placed,
    },
    Layout(crate::engine_placement::LayoutKey),
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Node {
    pub id: Key,
    pub operation: OperationKey,
    pub class: CoverageNodeClass,
    pub relation_case: Option<CaseKey>,
    pub active: Option<bool>,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dependency {
    pub source: Key,
    pub target: Key,
    pub edge: CoverageEdge,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cycle {
    pub members: Vec<Key>,
    pub internal_edges: Vec<Dependency>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Projection {
    pub nodes: Vec<Node>,
    pub edges: Vec<Dependency>,
}

/// Finite owner-produced symbols; assembly never interprets their identities.
#[derive(Clone, Debug)]
pub struct PlanSymbols {
    pub relation: Node,
    pub requirements: Vec<Node>,
    pub carriers: Vec<(Node, Vec<Node>)>,
    pub layouts: Vec<Node>,
    pub projections: Vec<Key>,
    pub evidence: Vec<Node>,
}

pub fn definition_census(plans: &[PlanSymbols]) -> Result<Vec<Node>, Box<Key>> {
    let span = tracing::debug_span!(target: "compiler::engine_coverage_graph", "definition_census", plan_count = crate::instrumentation::count(plans.len()));
    let _entered = span.enter();
    let mut nodes = BTreeMap::new();
    for plan in plans {
        for node in std::iter::once(&plan.relation).chain(&plan.requirements) {
            if nodes.insert(node.id.clone(), node.clone()).is_some() {
                return Err(Box::new(node.id.clone()));
            }
        }
        for node in plan
            .carriers
            .iter()
            .flat_map(|(carrier, layout)| std::iter::once(carrier).chain(layout))
            .chain(&plan.layouts)
            .chain(&plan.evidence)
        {
            if nodes.get(&node.id).is_some_and(|existing| existing != node) {
                return Err(Box::new(node.id.clone()));
            }
            nodes.insert(node.id.clone(), node.clone());
        }
    }
    Ok(nodes.into_values().collect())
}

pub fn derive_dependencies(plans: &[PlanSymbols], prerequisites: &[Dependency]) -> Vec<Dependency> {
    let span = tracing::debug_span!(target: "compiler::engine_coverage_graph", "derive_dependencies", plan_count = crate::instrumentation::count(plans.len()), prerequisite_count = crate::instrumentation::count(prerequisites.len()));
    let _entered = span.enter();
    let mut dependencies = prerequisites.iter().cloned().collect::<BTreeSet<_>>();
    for plan in plans {
        for (carrier, layout) in &plan.carriers {
            dependencies.insert(Dependency {
                source: carrier.id.clone(),
                target: plan.relation.id.clone(),
                edge: CoverageEdge::CarrierEnablesRelation,
            });
            for requirement in layout {
                dependencies.insert(Dependency {
                    source: requirement.id.clone(),
                    target: carrier.id.clone(),
                    edge: CoverageEdge::LayoutEnablesCarrier,
                });
            }
        }
        for projection in &plan.projections {
            dependencies.insert(Dependency {
                source: projection.clone(),
                target: plan.relation.id.clone(),
                edge: CoverageEdge::ProjectionEnablesRelation,
            });
        }
        for evidence in &plan.evidence {
            dependencies.insert(Dependency {
                source: evidence.id.clone(),
                target: plan.relation.id.clone(),
                edge: CoverageEdge::EvidenceEnablesRelation,
            });
        }
    }
    dependencies.into_iter().collect()
}

/// Join declared relation prerequisites to the same execution case. A complete
/// namespace-qualified operation identity is the scope boundary.
pub fn prerequisites(
    cases: &BTreeMap<OperationKey, Vec<crate::engine_placement::CaseKey>>,
    symbols: &BTreeMap<CaseKey, Key>,
    edges: &[(
        crate::engine_records::RelationKey,
        crate::engine_records::RelationKey,
    )],
) -> Result<Vec<Dependency>, Box<(CaseKey, CaseKey)>> {
    let span = tracing::debug_span!(target: "compiler::engine_coverage_graph", "prerequisites", edge_count = crate::instrumentation::count(edges.len()));
    let _entered = span.enter();
    let mut result = BTreeSet::new();
    for (source, target) in edges {
        if source.operation != target.operation {
            if let Some(source_case) = cases.get(&source.operation).and_then(|rows| rows.first())
                && let Some(target_case) =
                    cases.get(&target.operation).and_then(|rows| rows.first())
            {
                return Err(Box::new((
                    CaseKey {
                        relation: source.clone(),
                        case: source_case.clone(),
                    },
                    CaseKey {
                        relation: target.clone(),
                        case: target_case.clone(),
                    },
                )));
            }
            continue;
        }
        for case in cases.get(&source.operation).into_iter().flatten() {
            let source = CaseKey {
                relation: source.clone(),
                case: case.clone(),
            };
            let target = CaseKey {
                relation: target.clone(),
                case: case.clone(),
            };
            if let Some(source) = symbols.get(&source)
                && let Some(target) = symbols.get(&target)
            {
                result.insert(Dependency {
                    source: source.clone(),
                    target: target.clone(),
                    edge: CoverageEdge::RelationPrerequisite,
                });
            }
        }
    }
    Ok(result.into_iter().collect())
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    UnknownSymbol(Box<Key>),
    DuplicateSymbol(Box<Key>),
    EndpointClass(Box<Dependency>),
    CrossOperation(Box<Dependency>),
    DuplicateDependency(Box<Dependency>),
    Cycle(Vec<Cycle>),
}
pub fn check_definition(existing: Option<&Node>, offered: &Node, once: bool) -> Result<(), Error> {
    if existing.is_some_and(|existing| once || existing != offered) {
        return Err(Error::DuplicateSymbol(Box::new(offered.id.clone())));
    }
    Ok(())
}
#[derive(Debug)]
pub struct Graph {
    dag: DiGraph<Node, CoverageEdge, u32>,
    node_by_id: BTreeMap<Key, NodeIndex<u32>>,
    declared: BTreeSet<Dependency>,
}
impl Graph {
    /// Owner records choose which negative requirements claim dependency
    /// closure; the traversal and edge assembly are shared.
    pub fn collateral_dependencies(&self, requirements: &[(CaseKey, Key)]) -> Vec<Dependency> {
        let span = tracing::debug_span!(target: "compiler::engine_coverage_graph", "collateral_dependencies", requirement_count = crate::instrumentation::count(requirements.len()));
        let _entered = span.enter();
        let symbols = self
            .dag
            .node_weights()
            .filter_map(|node| node.relation_case.as_ref().map(|case| (case, &node.id)))
            .collect::<BTreeMap<_, _>>();
        let mut result = BTreeSet::new();
        for (case, requirement) in requirements {
            for dependent in self.dependency_collateral(case) {
                if let Some(target) = symbols.get(&dependent) {
                    result.insert(Dependency {
                        source: requirement.clone(),
                        target: (*target).clone(),
                        edge: CoverageEdge::DependencyCollateral,
                    });
                }
            }
        }
        result.into_iter().collect()
    }
    fn resolve(&self, id: &Key) -> Result<NodeIndex<u32>, Error> {
        self.node_by_id
            .get(id)
            .copied()
            .ok_or_else(|| Error::UnknownSymbol(Box::new(id.clone())))
    }
    pub fn add_dependencies(&mut self, dependencies: &[Dependency]) -> Result<(), Error> {
        for dependency in dependencies {
            let source = self.resolve(&dependency.source)?;
            let target = self.resolve(&dependency.target)?;
            let (expected_source, expected_target) = dependency.edge.endpoint_classes();
            if self.dag[source].class != expected_source
                || self.dag[target].class != expected_target
            {
                return Err(Error::EndpointClass(Box::new(dependency.clone())));
            }
            if self.dag[source].operation != self.dag[target].operation {
                return Err(Error::CrossOperation(Box::new(dependency.clone())));
            }
            if !self.declared.insert(dependency.clone()) {
                return Err(Error::DuplicateDependency(Box::new(dependency.clone())));
            }
            self.dag.add_edge(source, target, dependency.edge);
        }
        Ok(())
    }
    pub fn dependency_collateral(&self, key: &CaseKey) -> BTreeSet<CaseKey> {
        let Some(start) = self
            .dag
            .node_indices()
            .find(|node| self.dag[*node].relation_case.as_ref() == Some(key))
        else {
            return BTreeSet::new();
        };
        let mut visited = BTreeSet::new();
        let mut stack = vec![start];
        let mut collateral = BTreeSet::new();
        while let Some(node) = stack.pop() {
            for edge in self.dag.edges_directed(node, Direction::Outgoing) {
                if *edge.weight() != CoverageEdge::RelationPrerequisite {
                    continue;
                }
                let next = edge.target();
                if !visited.insert(next) {
                    continue;
                }
                stack.push(next);
                let weight = &self.dag[next];
                if weight.active != Some(true) {
                    continue;
                }
                if let Some(dependent) = &weight.relation_case
                    && dependent != key
                {
                    collateral.insert(dependent.clone());
                }
            }
        }
        collateral
    }
    pub fn validate_acyclic(&self) -> Result<(), Error> {
        let components = self.cyclic_components();
        if components.is_empty() {
            Ok(())
        } else {
            Err(Error::Cycle(components))
        }
    }
    pub fn cyclic_components(&self) -> Vec<Cycle> {
        let mut components = kosaraju_scc(&self.dag)
            .into_iter()
            .filter(|component| {
                component.len() > 1
                    || component
                        .first()
                        .is_some_and(|node| self.dag.find_edge(*node, *node).is_some())
            })
            .map(|component| {
                let members_by_index = component.iter().copied().collect::<BTreeSet<_>>();
                let mut members = component
                    .iter()
                    .map(|node| self.dag[*node].id.clone())
                    .collect::<Vec<_>>();
                members.sort();

                let mut internal_edges = self
                    .dag
                    .edge_references()
                    .filter(|edge| {
                        members_by_index.contains(&edge.source())
                            && members_by_index.contains(&edge.target())
                    })
                    .map(|edge| Dependency {
                        source: self.dag[edge.source()].id.clone(),
                        target: self.dag[edge.target()].id.clone(),
                        edge: *edge.weight(),
                    })
                    .collect::<Vec<_>>();
                internal_edges.sort();

                Cycle {
                    members,
                    internal_edges,
                }
            })
            .collect::<Vec<_>>();

        components.sort();
        components
    }

    /// This graph's stable projection.
    #[must_use]
    pub fn project(&self) -> Projection {
        let mut nodes = self.dag.node_weights().cloned().collect::<Vec<_>>();
        nodes.sort();

        let mut edges = self
            .dag
            .edge_references()
            .map(|edge| Dependency {
                source: self.dag[edge.source()].id.clone(),
                target: self.dag[edge.target()].id.clone(),
                edge: *edge.weight(),
            })
            .collect::<Vec<_>>();
        edges.sort();

        Projection { nodes, edges }
    }
}

pub fn build(nodes: &[Node], dependencies: &[Dependency]) -> Result<Graph, Error> {
    let span = tracing::debug_span!(target: "compiler::engine_coverage_graph", "build", node_count = crate::instrumentation::count(nodes.len()), edge_count = crate::instrumentation::count(dependencies.len()));
    let _entered = span.enter();
    let mut census = BTreeMap::new();
    for node in nodes {
        check_definition(census.get(&node.id), node, false)?;
        census.insert(node.id.clone(), node.clone());
    }
    let mut graph = DiGraph::with_capacity(census.len(), dependencies.len());
    let mut node_by_id = BTreeMap::new();
    for node in census.into_values() {
        let key = node.id.clone();
        let index = graph.add_node(node);
        node_by_id.insert(key, index);
    }
    let mut resolved = Graph {
        dag: graph,
        node_by_id,
        declared: BTreeSet::new(),
    };
    resolved.add_dependencies(dependencies)?;
    Ok(resolved)
}
