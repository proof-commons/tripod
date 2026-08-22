//! Typed coverage dependencies and SCC policy (Guide-6 Tranche E).
//!
//! The coverage census of the previous stage states obligations; this
//! stage states how they depend on one another. Every dependency is a
//! typed edge between two stable coverage symbols, resolved in two
//! passes: a definition census is completed first, and only then is any
//! reference resolved against it. Nothing here resolves a symbol by
//! display string, by insertion order, or by Petgraph index — a graph
//! handle is a local artifact of how the graph was built, and a
//! dependency keyed on one would change meaning when the same analysis
//! is built in a different order.
//!
//! The graph is a direct Petgraph `DiGraph` inside a domain struct with
//! typed lookup maps, not a generic graph wrapper: the node and edge
//! vocabularies are the semantics, and a reusable container would let a
//! future caller insert an edge this vocabulary cannot express.
//!
//! # Orientation
//!
//! Direction is prerequisite → dependent for every edge: the source is
//! what must be in place, the target is what it enables. One graph
//! documenting one orientation while emitting several is not a graph
//! anything generic can read — a topological order, an ancestor or
//! descendant query, or a "what must exist before this relation is
//! discharged?" traversal would each mean something different depending
//! on which edge it happened to cross.
//!
//! | Edge | Source | Target | Class |
//! |---|---|---|---|
//! | relation prerequisite | relation-case | relation-case | prerequisite |
//! | carrier enables relation | carrier | relation-case | ownership |
//! | layout enables carrier | layout | carrier | ownership |
//! | projection enables relation | requirement | relation-case | ownership |
//! | evidence enables relation | external evidence | relation-case | ownership |
//! | dependency collateral | requirement | relation-case | collateral |
//!
//! The classes are the second half of the settlement. A shared
//! orientation makes the graph traversable; it does not make every edge
//! mean the same thing, and the closure below depends on the difference.
//! Each variant's admissible endpoint classes are declared with it and
//! checked when the edge is inserted, so a reversed dependency is a
//! typed rejection rather than a different but equally plausible graph.
//!
//! Dependency collateral is the strict active-descendant closure of the
//! intended relation, taken within one operation and one execution case.
//! A mutation of a relation really does block the relations that depend
//! on it, so a focused negative requirement must claim them; a dependent
//! that is inactive in the case is not claimed, because a relation that
//! never activates cannot be observed to become blocked.
//!
//! Coverage cycles are forbidden. Finding a strongly connected component
//! never authorizes one: an accepted cycle would need a typed resolution
//! strategy that does not exist, so the SCC analysis exists to reject,
//! and there is no generic allowed-cycle escape to reach for.

// One item-level allowance remains, on an emptiness query the
// dependency resolution does not need to ask.

use std::collections::{BTreeMap, BTreeSet};

use architecture::OperationId;
use petgraph::{
    Direction,
    algo::kosaraju_scc,
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
};
use realization::{ExternalEvidenceRequirement, RelationDependencyProjection};

use crate::{
    CompileError,
    coverage::{
        CollateralPolicy, CoveragePurpose, CoverageRequirementId, PlanCoverageAnalysis,
        bind_dependency_collateral,
    },
    layout::LayoutRequirement,
    placement::{PlacedCarrier, RelationActivity, RelationCaseKey},
    relation::{AnalysisNodeId, CompilerRelationAnalysis, CompilerRelationEdge},
};

/// Stable identity of one coverage symbol.
///
/// Complete typed values throughout: each variant already carries every
/// component that distinguishes it, so no symbol name, index, or digest
/// is needed to tell two symbols apart. The requirement identity
/// includes its boundary, so a hybrid relation's compiler-static and
/// backend-structural obligations are two symbols rather than one.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CoverageNodeId {
    /// One relation in one execution case.
    RelationCase(RelationCaseKey),
    /// One coverage requirement of one relation-case.
    Requirement(CoverageRequirementId),
    /// One carrier selected for one relation-case obligation.
    Carrier {
        relation_case: RelationCaseKey,
        carrier: PlacedCarrier,
    },
    /// One target-independent layout obligation.
    Layout(LayoutRequirement),
    /// One typed external-evidence obligation.
    ///
    /// Shared rather than per relation-case: two relations requiring the
    /// same report require the same report, and duplicating the symbol
    /// would state two obligations one report answers.
    ExternalEvidence(ExternalEvidenceRequirement),
}

impl CoverageNodeId {
    /// The operation this symbol belongs to.
    ///
    /// Every symbol has one: coverage is stored and validated per
    /// operation, and a symbol with no operation could not be checked
    /// against the cross-operation rule.
    #[must_use]
    pub const fn operation(&self) -> OperationId {
        match self {
            Self::RelationCase(key) => key.case.operation,
            Self::Requirement(id) => id.case.operation,
            Self::Carrier { relation_case, .. } => relation_case.case.operation,
            Self::Layout(requirement) => layout_operation(requirement),
            Self::ExternalEvidence(requirement) => requirement.operation(),
        }
    }

    /// Which kind of symbol this is.
    #[must_use]
    pub const fn class(&self) -> CoverageNodeClass {
        match self {
            Self::RelationCase(_) => CoverageNodeClass::RelationCase,
            Self::Requirement(_) => CoverageNodeClass::Requirement,
            Self::Carrier { .. } => CoverageNodeClass::Carrier,
            Self::Layout(_) => CoverageNodeClass::Layout,
            Self::ExternalEvidence(_) => CoverageNodeClass::ExternalEvidence,
        }
    }

    /// The relation-case this symbol belongs to, where it names one.
    #[must_use]
    pub const fn relation_case(&self) -> Option<&RelationCaseKey> {
        match self {
            Self::RelationCase(key)
            | Self::Carrier {
                relation_case: key, ..
            } => Some(key),
            Self::Requirement(_) | Self::Layout(_) | Self::ExternalEvidence(_) => None,
        }
    }
}

/// The operation one layout requirement belongs to.
const fn layout_operation(requirement: &LayoutRequirement) -> OperationId {
    match requirement {
        LayoutRequirement::CanonicalCoordinator { operation, .. } => *operation,
        LayoutRequirement::MakeSourceAvailable { case, .. }
        | LayoutRequirement::IsolateSponsorRegion { case, .. }
        | LayoutRequirement::SecretFreeOperationPath { case, .. } => case.operation,
        LayoutRequirement::AuthenticateFamilyCensus { relation, .. }
        | LayoutRequirement::CompleteAndDisjointFamilies { relation, .. }
        | LayoutRequirement::EnforceRepresentation { relation, .. } => relation.operation(),
    }
}

/// Why one coverage symbol depends on another.
///
/// Direction is prerequisite → dependent for every variant: the source
/// is what must be in place, the target is what it enables. The module
/// documentation carries the full table and the reason the orientation
/// is uniform rather than per-variant.
///
/// The variant names state the direction rather than leaving it to the
/// field names. A variant called `RequiresCarrier` reads naturally in
/// both directions, which is how the previous vocabulary managed to
/// document one orientation while emitting another.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CoverageEdge {
    /// The source relation-case must hold before the target one can.
    RelationPrerequisite,
    /// The source carrier discharges the target relation-case.
    CarrierEnablesRelation,
    /// The source layout obligation makes the target carrier usable.
    LayoutEnablesCarrier,
    /// The source accepted-projection comparison discharges the target
    /// relation-case.
    ProjectionEnablesRelation,
    /// The source external report discharges the target relation-case.
    EvidenceEnablesRelation,
    /// The source negative requirement claims the target relation-case
    /// becomes blocked.
    ///
    /// A claim rather than a prerequisite, and the one variant whose
    /// reading is not "the target needs the source". It shares the
    /// orientation because it is derived from the prerequisite closure
    /// and points the same way that closure travels; see
    /// [`CoverageEdgeClass`].
    DependencyCollateral,
}

/// What one typed coverage edge asserts.
///
/// Two edges may share an orientation and still mean different things.
/// Keeping the vocabulary separate is what lets the traversals ask for
/// the edges whose meaning they actually depend on, rather than
/// filtering on a variant list that has to be revisited whenever the
/// vocabulary grows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CoverageEdgeClass {
    /// One relation-case must hold before another can.
    ///
    /// The only class carrying dependency-closure semantics: a relation
    /// blocks a relation, and nothing else blocks a relation.
    Prerequisite,
    /// One symbol is a requirement discharging another.
    Ownership,
    /// One negative requirement claims another symbol becomes blocked.
    Collateral,
}

/// Which kind of symbol one coverage node is.
///
/// Extracted from the identity so an edge's endpoints can be checked
/// against the classes the edge is defined over, without the check
/// having to match on every identity variant itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CoverageNodeClass {
    RelationCase,
    Requirement,
    Carrier,
    Layout,
    ExternalEvidence,
}

impl CoverageEdge {
    /// What this edge asserts.
    #[must_use]
    pub const fn class(self) -> CoverageEdgeClass {
        match self {
            Self::RelationPrerequisite => CoverageEdgeClass::Prerequisite,
            Self::CarrierEnablesRelation
            | Self::LayoutEnablesCarrier
            | Self::ProjectionEnablesRelation
            | Self::EvidenceEnablesRelation => CoverageEdgeClass::Ownership,
            Self::DependencyCollateral => CoverageEdgeClass::Collateral,
        }
    }

    /// The node classes this edge is defined over, source then target.
    ///
    /// An edge vocabulary that fixes its endpoint classes is what makes
    /// a reversed dependency a typed rejection rather than a silently
    /// different graph: every variant except
    /// [`Self::RelationPrerequisite`] joins two *different* classes, so
    /// swapping its endpoints produces a pair no variant admits.
    ///
    /// A reversed relation prerequisite joins the same two classes and
    /// so cannot be caught here. It is caught where it must be — by
    /// comparison against the independently re-derived census, which is
    /// the only check that can know which of two relation-cases came
    /// first.
    #[must_use]
    pub const fn endpoint_classes(self) -> (CoverageNodeClass, CoverageNodeClass) {
        match self {
            Self::RelationPrerequisite => (
                CoverageNodeClass::RelationCase,
                CoverageNodeClass::RelationCase,
            ),
            Self::CarrierEnablesRelation => {
                (CoverageNodeClass::Carrier, CoverageNodeClass::RelationCase)
            }
            Self::LayoutEnablesCarrier => (CoverageNodeClass::Layout, CoverageNodeClass::Carrier),
            Self::EvidenceEnablesRelation => (
                CoverageNodeClass::ExternalEvidence,
                CoverageNodeClass::RelationCase,
            ),
            // Same endpoint classes, different assertions: one
            // requirement discharges its relation-case, the other claims
            // a relation-case becomes blocked. [`Self::class`] is what
            // tells those apart; the endpoint rule is not asked to.
            Self::ProjectionEnablesRelation | Self::DependencyCollateral => (
                CoverageNodeClass::Requirement,
                CoverageNodeClass::RelationCase,
            ),
        }
    }
}

/// One coverage symbol as stored in the graph.
///
/// The activity travels with the relation-case node so the collateral
/// closure filters on the graph itself: re-reading the coverage analysis
/// mid-traversal would make the closure depend on two sources that can
/// disagree.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CoverageNode {
    pub id: CoverageNodeId,
    /// Present exactly for a relation-case symbol.
    pub activity: Option<RelationActivity>,
}

/// One typed coverage dependency.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CoverageDependency {
    pub source: CoverageNodeId,
    pub target: CoverageNodeId,
    pub edge: CoverageEdge,
}

/// One canonically ordered cyclic strongly connected component.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CoverageCycleComponent {
    /// Sorted by stable coverage symbol.
    pub members: Vec<CoverageNodeId>,
    /// Sorted by typed source, target, and edge role.
    pub internal_edges: Vec<CoverageDependency>,
}

/// The complete typed definition census (pass 1).
///
/// Built before any reference is resolved. A census assembled while
/// references were being resolved would accept a forward reference in
/// one order and reject it in another, which is exactly the
/// order-dependence the two passes exist to remove.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CoverageDefinitionCensus {
    nodes: BTreeMap<CoverageNodeId, CoverageNode>,
}

impl CoverageDefinitionCensus {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Define one symbol, tolerating an exact repeat.
    ///
    /// A carrier, layout, or external-evidence symbol is genuinely
    /// shared: two assignment alternatives may select the same carrier,
    /// and two relations may require the same report. Sharing is the
    /// meaning, so an identical redefinition is idempotent.
    ///
    /// # Errors
    ///
    /// [`CompileError::DuplicateCoverageRequirement`] when the same
    /// symbol is defined with a different weight.
    pub fn define(&mut self, node: CoverageNode) -> Result<(), CompileError> {
        match self.nodes.get(&node.id) {
            Some(existing) if existing != &node => Err(duplicate_symbol(&node.id)),
            Some(_) => Ok(()),
            None => {
                self.nodes.insert(node.id.clone(), node);
                Ok(())
            }
        }
    }

    /// Define one symbol that must occur exactly once.
    ///
    /// # Errors
    ///
    /// [`CompileError::DuplicateCoverageRequirement`] on any repeat.
    pub fn define_once(&mut self, node: CoverageNode) -> Result<(), CompileError> {
        if self.nodes.contains_key(&node.id) {
            return Err(duplicate_symbol(&node.id));
        }

        self.nodes.insert(node.id.clone(), node);
        Ok(())
    }

    #[must_use]
    pub fn contains(&self, id: &CoverageNodeId) -> bool {
        self.nodes.contains_key(id)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    #[must_use]
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Every defined symbol, in stable order.
    pub fn nodes(&self) -> impl Iterator<Item = &CoverageNode> {
        self.nodes.values()
    }
}

/// The duplicate-symbol failure of one coverage symbol.
///
/// A requirement symbol is named exactly; the remaining symbol kinds
/// have no requirement identity to report, so the relation-case they
/// belong to is reported through the census mismatch instead.
fn duplicate_symbol(id: &CoverageNodeId) -> CompileError {
    match id {
        CoverageNodeId::Requirement(requirement) => CompileError::DuplicateCoverageRequirement {
            requirement: requirement.clone(),
        },
        _ => CompileError::CoverageCensusMismatch {
            missing: Vec::new(),
            unexpected: id.relation_case().cloned().into_iter().collect(),
        },
    }
}

/// Collect the complete typed definition census of one coverage
/// analysis (pass 1).
///
/// # Errors
///
/// [`CompileError::DuplicateCoverageRequirement`] when one requirement
/// identity is defined twice; [`CompileError::CoverageCensusMismatch`]
/// when any other stable symbol is.
pub fn coverage_definition_census(
    analysis: &PlanCoverageAnalysis,
) -> Result<CoverageDefinitionCensus, CompileError> {
    let mut census = CoverageDefinitionCensus::new();

    for plan in analysis.plans() {
        let key = plan.key();

        census.define_once(CoverageNode {
            id: CoverageNodeId::RelationCase(key.clone()),
            activity: Some(plan.activity),
        })?;

        for id in plan
            .positive
            .iter()
            .map(|requirement| &requirement.id)
            .chain(plan.negative.iter().map(|requirement| &requirement.id))
        {
            census.define_once(CoverageNode {
                id: CoverageNodeId::Requirement(id.clone()),
                activity: None,
            })?;
        }

        for alternative in plan
            .carrier
            .iter()
            .flat_map(|carrier| carrier.allowed_assignments.iter())
        {
            for carrier in &alternative.carriers {
                census.define(CoverageNode {
                    id: CoverageNodeId::Carrier {
                        relation_case: key.clone(),
                        carrier: carrier.clone(),
                    },
                    activity: None,
                })?;
            }

            for requirement in &alternative.layout {
                census.define(CoverageNode {
                    id: CoverageNodeId::Layout(requirement.clone()),
                    activity: None,
                })?;
            }
        }

        for requirement in &plan.external_evidence {
            census.define(CoverageNode {
                id: CoverageNodeId::ExternalEvidence(requirement.clone()),
                activity: None,
            })?;
        }
    }

    Ok(census)
}

/// Derive every typed coverage dependency (pass 2).
///
/// Relation prerequisites come from the realization relation dependency
/// edges, restricted twice: to one operation, because a coverage
/// analysis is stored per operation and a dependency across two of them
/// would be a claim neither analysis owns; and to one execution case,
/// because "this relation blocks that one" is a statement about a single
/// world, and a prerequisite holding in the sponsored case says nothing
/// about the sponsorless one. Both endpoints are resolved against the
/// completed census, so a relation outside coverage scope contributes no
/// edge rather than a dangling one.
///
/// # Errors
///
/// [`CompileError::CrossOperationCoverageDependency`] when a relation
/// edge crosses two operations that are both covered.
pub fn derive_coverage_dependencies(
    analysis: &PlanCoverageAnalysis,
    census: &CoverageDefinitionCensus,
    relation_edges: &[RelationDependencyProjection],
) -> Result<Vec<CoverageDependency>, CompileError> {
    let mut dependencies = BTreeSet::new();

    for dependency in relation_edges {
        let prerequisite = dependency.source.operation();
        let dependent = dependency.target.operation();

        if prerequisite != dependent {
            if let Some(failure) = cross_operation_failure(analysis, dependency) {
                return Err(failure);
            }

            continue;
        }

        let Some(operation) = analysis.operations.get(&prerequisite) else {
            continue;
        };

        for case in &operation.cases {
            let source = CoverageNodeId::RelationCase(RelationCaseKey {
                relation: dependency.source.clone(),
                case: case.clone(),
            });
            let target = CoverageNodeId::RelationCase(RelationCaseKey {
                relation: dependency.target.clone(),
                case: case.clone(),
            });

            if !census.contains(&source) || !census.contains(&target) {
                continue;
            }

            dependencies.insert(CoverageDependency {
                source,
                target,
                edge: CoverageEdge::RelationPrerequisite,
            });
        }
    }

    for plan in analysis.plans() {
        let key = plan.key();
        let relation_case = CoverageNodeId::RelationCase(key.clone());

        for alternative in plan
            .carrier
            .iter()
            .flat_map(|carrier| carrier.allowed_assignments.iter())
        {
            for carrier in &alternative.carriers {
                let node = CoverageNodeId::Carrier {
                    relation_case: key.clone(),
                    carrier: carrier.clone(),
                };

                dependencies.insert(CoverageDependency {
                    source: node.clone(),
                    target: relation_case.clone(),
                    edge: CoverageEdge::CarrierEnablesRelation,
                });

                // The layout obligations travel with the alternative
                // that selected this carrier, never with the union over
                // every eligible one.
                for requirement in &alternative.layout {
                    dependencies.insert(CoverageDependency {
                        source: CoverageNodeId::Layout(requirement.clone()),
                        target: node.clone(),
                        edge: CoverageEdge::LayoutEnablesCarrier,
                    });
                }
            }
        }

        for boundary in plan.projections.keys() {
            dependencies.insert(CoverageDependency {
                source: CoverageNodeId::Requirement(CoverageRequirementId {
                    relation: key.relation.clone(),
                    case: key.case.clone(),
                    boundary: *boundary,
                    purpose: CoveragePurpose::AcceptedProjection,
                }),
                target: relation_case.clone(),
                edge: CoverageEdge::ProjectionEnablesRelation,
            });
        }

        for requirement in &plan.external_evidence {
            dependencies.insert(CoverageDependency {
                source: CoverageNodeId::ExternalEvidence(requirement.clone()),
                target: relation_case.clone(),
                edge: CoverageEdge::EvidenceEnablesRelation,
            });
        }
    }

    Ok(dependencies.into_iter().collect())
}

/// The cross-operation failure of one relation edge, if both of its
/// operations are covered.
///
/// One representative case per side: no case pairing across two
/// operations is meaningful, and the pair exists to name the rejected
/// edge rather than to describe a world.
fn cross_operation_failure(
    analysis: &PlanCoverageAnalysis,
    dependency: &RelationDependencyProjection,
) -> Option<CompileError> {
    let source = analysis.operations.get(&dependency.source.operation())?;
    let target = analysis.operations.get(&dependency.target.operation())?;

    Some(CompileError::CrossOperationCoverageDependency {
        prerequisite: Box::new(CoverageNodeId::RelationCase(RelationCaseKey {
            relation: dependency.source.clone(),
            case: source.cases.first()?.clone(),
        })),
        dependent: Box::new(CoverageNodeId::RelationCase(RelationCaseKey {
            relation: dependency.target.clone(),
            case: target.cases.first()?.clone(),
        })),
    })
}

/// The stable projection of one coverage dependency graph.
///
/// Sorted stable symbols and sorted typed edges. Petgraph node and edge
/// indices, insertion order, and the order the SCC algorithm happened to
/// return components in are all absent: each is an artifact of how the
/// graph was built or walked, not a property of the dependencies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoverageGraphProjection {
    pub nodes: Vec<CoverageNode>,
    pub edges: Vec<CoverageDependency>,
}

/// The typed coverage dependency graph.
///
/// A direct Petgraph graph with typed lookup maps. The indices and the
/// declared-edge set never leave this structure.
#[derive(Debug)]
pub struct CoverageDependencyGraph {
    graph: DiGraph<CoverageNode, CoverageEdge, u32>,
    node_by_id: BTreeMap<CoverageNodeId, NodeIndex<u32>>,
    declared: BTreeSet<CoverageDependency>,
}

impl CoverageDependencyGraph {
    /// Resolve one reference against the definition census.
    fn resolve(&self, id: &CoverageNodeId) -> Result<NodeIndex<u32>, CompileError> {
        self.node_by_id
            .get(id)
            .copied()
            .ok_or_else(|| CompileError::UnknownCoverageSymbol { symbol: id.clone() })
    }

    /// Resolve and insert typed dependencies.
    ///
    /// # Errors
    ///
    /// [`CompileError::UnknownCoverageSymbol`] when an endpoint is
    /// absent from the census;
    /// [`CompileError::CrossOperationCoverageDependency`] when the
    /// endpoints belong to two operations;
    /// [`CompileError::DuplicateCoverageDependency`] when the same
    /// typed dependency is declared twice;
    /// [`CompileError::CoverageDependencyEndpointClass`] when an edge
    /// joins node classes its variant is not defined over — which is
    /// what a reversed dependency looks like.
    fn add_dependencies(
        &mut self,
        dependencies: &[CoverageDependency],
    ) -> Result<(), CompileError> {
        for dependency in dependencies {
            let source = self.resolve(&dependency.source)?;
            let target = self.resolve(&dependency.target)?;

            let (expected_source, expected_target) = dependency.edge.endpoint_classes();

            if dependency.source.class() != expected_source
                || dependency.target.class() != expected_target
            {
                return Err(CompileError::CoverageDependencyEndpointClass {
                    prerequisite: Box::new(dependency.source.clone()),
                    dependent: Box::new(dependency.target.clone()),
                    edge: dependency.edge,
                });
            }

            if dependency.source.operation() != dependency.target.operation() {
                return Err(CompileError::CrossOperationCoverageDependency {
                    prerequisite: Box::new(dependency.source.clone()),
                    dependent: Box::new(dependency.target.clone()),
                });
            }

            if !self.declared.insert(dependency.clone()) {
                return Err(CompileError::DuplicateCoverageDependency {
                    prerequisite: Box::new(dependency.source.clone()),
                    dependent: Box::new(dependency.target.clone()),
                    edge: dependency.edge,
                });
            }

            self.graph.add_edge(source, target, dependency.edge);
        }

        Ok(())
    }

    /// The strict active-descendant closure of one relation-case.
    ///
    /// Traversal follows relation prerequisites only: a carrier, layout,
    /// projection, or evidence edge is a requirement of the relation-
    /// case, not a relation that becomes blocked when it fails.
    /// Traversal passes *through* an inactive dependent but never claims
    /// it, because a relation that never activates in the case cannot be
    /// observed to block.
    #[must_use]
    pub fn dependency_collateral(&self, key: &RelationCaseKey) -> BTreeSet<RelationCaseKey> {
        let Some(start) = self
            .node_by_id
            .get(&CoverageNodeId::RelationCase(key.clone()))
        else {
            return BTreeSet::new();
        };

        let mut visited = BTreeSet::new();
        let mut stack = vec![*start];
        let mut collateral = BTreeSet::new();

        while let Some(node) = stack.pop() {
            for edge in self.graph.edges_directed(node, Direction::Outgoing) {
                if *edge.weight() != CoverageEdge::RelationPrerequisite {
                    continue;
                }

                let next = edge.target();

                if !visited.insert(next) {
                    continue;
                }

                stack.push(next);

                let weight = &self.graph[next];

                if weight.activity != Some(RelationActivity::Active) {
                    continue;
                }

                if let CoverageNodeId::RelationCase(dependent) = &weight.id
                    && dependent != key
                {
                    collateral.insert(dependent.clone());
                }
            }
        }

        collateral
    }

    /// Declare the dependency-collateral edges of every negative
    /// requirement that must claim a closure.
    ///
    /// # Errors
    ///
    /// Any failure of the shared resolution documented on
    /// [`build_coverage_graph`].
    pub fn attach_dependency_collateral(
        &mut self,
        analysis: &PlanCoverageAnalysis,
    ) -> Result<(), CompileError> {
        let mut dependencies = BTreeSet::new();

        for plan in analysis.plans() {
            let collateral = self.dependency_collateral(&plan.key());

            for requirement in &plan.negative {
                if requirement.collateral.policy
                    != CollateralPolicy::RequireIntendedAndDependencyClosure
                {
                    continue;
                }

                for dependent in &collateral {
                    dependencies.insert(CoverageDependency {
                        source: CoverageNodeId::Requirement(requirement.id.clone()),
                        target: CoverageNodeId::RelationCase(dependent.clone()),
                        edge: CoverageEdge::DependencyCollateral,
                    });
                }
            }
        }

        let dependencies = dependencies.into_iter().collect::<Vec<_>>();
        self.add_dependencies(&dependencies)
    }

    /// Reject every coverage dependency cycle.
    ///
    /// Self-loops and multi-node components alike: the current coverage
    /// cycle strategy is forbidden, and no component is exempted for
    /// being small, expected, or benign.
    ///
    /// # Errors
    ///
    /// [`CompileError::CoverageDependencyCycle`] on any cyclic
    /// component, with canonically normalized members and internal
    /// edges.
    pub fn validate_acyclic(&self) -> Result<(), CompileError> {
        let components = self.cyclic_components();

        if components.is_empty() {
            return Ok(());
        }

        Err(CompileError::CoverageDependencyCycle { components })
    }

    /// Every cyclic strongly connected component, canonically ordered.
    #[must_use]
    pub fn cyclic_components(&self) -> Vec<CoverageCycleComponent> {
        let mut components = kosaraju_scc(&self.graph)
            .into_iter()
            .filter(|component| {
                component.len() > 1
                    || component
                        .first()
                        .is_some_and(|node| self.graph.find_edge(*node, *node).is_some())
            })
            .map(|component| {
                let members_by_index = component.iter().copied().collect::<BTreeSet<_>>();
                let mut members = component
                    .iter()
                    .map(|node| self.graph[*node].id.clone())
                    .collect::<Vec<_>>();
                members.sort();

                let mut internal_edges = self
                    .graph
                    .edge_references()
                    .filter(|edge| {
                        members_by_index.contains(&edge.source())
                            && members_by_index.contains(&edge.target())
                    })
                    .map(|edge| CoverageDependency {
                        source: self.graph[edge.source()].id.clone(),
                        target: self.graph[edge.target()].id.clone(),
                        edge: *edge.weight(),
                    })
                    .collect::<Vec<_>>();
                internal_edges.sort();

                CoverageCycleComponent {
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
    pub fn project(&self) -> CoverageGraphProjection {
        let mut nodes = self.graph.node_weights().cloned().collect::<Vec<_>>();
        nodes.sort();

        let mut edges = self
            .graph
            .edge_references()
            .map(|edge| CoverageDependency {
                source: self.graph[edge.source()].id.clone(),
                target: self.graph[edge.target()].id.clone(),
                edge: *edge.weight(),
            })
            .collect::<Vec<_>>();
        edges.sort();

        CoverageGraphProjection { nodes, edges }
    }
}

/// Build the typed coverage dependency graph from a completed census.
///
/// Separated from [`analyze_coverage_dependencies`] so tests can
/// exercise every rejection path with synthetic symbols and edges a
/// derived coverage analysis could never produce.
///
/// # Errors
///
/// [`CompileError::UnknownCoverageSymbol`] when a dependency names a
/// symbol the census does not define;
/// [`CompileError::CrossOperationCoverageDependency`] when a dependency
/// crosses two operations; [`CompileError::DuplicateCoverageDependency`]
/// when the same typed dependency is declared twice.
pub fn build_coverage_graph(
    census: &CoverageDefinitionCensus,
    dependencies: &[CoverageDependency],
) -> Result<CoverageDependencyGraph, CompileError> {
    let mut graph =
        DiGraph::<CoverageNode, CoverageEdge, u32>::with_capacity(census.len(), dependencies.len());
    let mut node_by_id = BTreeMap::new();

    // Canonical insertion: symbols in stable census order.
    for node in census.nodes() {
        let index = graph.add_node(node.clone());
        node_by_id.insert(node.id.clone(), index);
    }

    let mut resolved = CoverageDependencyGraph {
        graph,
        node_by_id,
        declared: BTreeSet::new(),
    };

    resolved.add_dependencies(dependencies)?;
    Ok(resolved)
}

/// The realization relation dependency edges of one compiler relation
/// analysis.
fn source_relation_edges(
    relations: &CompilerRelationAnalysis,
) -> Vec<RelationDependencyProjection> {
    relations
        .project()
        .edges
        .into_iter()
        .map(|edge| {
            let AnalysisNodeId::SourceRelation(source) = edge.source;
            let AnalysisNodeId::SourceRelation(target) = edge.target;
            let CompilerRelationEdge::SourceDependency(kind) = edge.edge;

            RelationDependencyProjection {
                source,
                target,
                edge: kind,
            }
        })
        .collect()
}

/// Resolve one coverage analysis into its typed dependency graph.
///
/// The two passes in order: the complete definition census, then every
/// reference resolved against it. The graph is checked acyclic before
/// the collateral edges are declared and again afterwards, so a cycle
/// introduced by a collateral claim cannot hide behind the earlier
/// check.
///
/// # Errors
///
/// Any failure of [`coverage_definition_census`],
/// [`derive_coverage_dependencies`], [`build_coverage_graph`], or
/// [`CoverageDependencyGraph::validate_acyclic`].
pub fn analyze_coverage_dependencies(
    analysis: &PlanCoverageAnalysis,
    relations: &CompilerRelationAnalysis,
) -> Result<CoverageDependencyGraph, CompileError> {
    let census = coverage_definition_census(analysis)?;
    let dependencies =
        derive_coverage_dependencies(analysis, &census, &source_relation_edges(relations))?;
    let mut graph = build_coverage_graph(&census, &dependencies)?;

    graph.validate_acyclic()?;
    graph.attach_dependency_collateral(analysis)?;
    graph.validate_acyclic()?;

    Ok(graph)
}

/// Resolve the typed dependencies and bind the collateral they imply.
///
/// The typed hook coverage left open: every runtime negative
/// requirement receives the graph's own descendant closure, so the
/// closure a later report is measured against and the closure the graph
/// states are the same value rather than two derivations that can
/// disagree.
///
/// # Errors
///
/// Any failure of [`analyze_coverage_dependencies`].
pub fn resolve_coverage_dependencies(
    analysis: &mut PlanCoverageAnalysis,
    relations: &CompilerRelationAnalysis,
) -> Result<CoverageDependencyGraph, CompileError> {
    let graph = analyze_coverage_dependencies(analysis, relations)?;

    bind_dependency_collateral(analysis, &|key| graph.dependency_collateral(key));
    Ok(graph)
}
