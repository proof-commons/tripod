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
//! The owner supplies typed symbols to the shared dependency resolver.
//! Endpoint classes and edge roles remain checked before traversal; the
//! compatibility registry preserves the attestation projection identities.
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

#[cfg(test)]
use crate::coverage::{CollateralPolicy, CoveragePurpose, PlanCoverageAnalysis};
#[cfg(test)]
use realization::RelationDependencyProjection;
use std::collections::BTreeMap;
#[cfg(test)]
use std::collections::BTreeSet;

use crate::engine_coverage_graph as core;
use architecture::OperationId;
use realization::ExternalEvidenceRequirement;

use crate::{
    CompileError,
    coverage::CoverageRequirementId,
    layout::LayoutRequirement,
    placement::{PlacedCarrier, RelationActivity, RelationCaseKey},
};
#[cfg(test)]
use crate::{
    coverage::bind_dependency_collateral,
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
#[cfg(test)]
pub struct CoverageDefinitionCensus {
    nodes: BTreeMap<CoverageNodeId, CoverageNode>,
}

#[cfg(test)]
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
        let existing = self.nodes.get(&node.id).map(lower_node);
        core::check_definition(existing.as_ref(), &lower_node(&node), false)
            .map_err(|_| duplicate_symbol(&node.id))?;
        self.nodes.insert(node.id.clone(), node);
        Ok(())
    }

    /// Define one symbol that must occur exactly once.
    ///
    /// # Errors
    ///
    /// [`CompileError::DuplicateCoverageRequirement`] on any repeat.
    #[cfg(test)]
    pub fn define_once(&mut self, node: CoverageNode) -> Result<(), CompileError> {
        let existing = self.nodes.get(&node.id).map(lower_node);
        core::check_definition(existing.as_ref(), &lower_node(&node), true)
            .map_err(|_| duplicate_symbol(&node.id))?;
        self.nodes.insert(node.id.clone(), node);
        Ok(())
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
#[cfg(test)]
pub fn coverage_definition_census(
    analysis: &PlanCoverageAnalysis,
) -> Result<CoverageDefinitionCensus, CompileError> {
    let (plans, identities) = lower_assembly(analysis);
    let nodes =
        core::definition_census(&plans).map_err(|key| duplicate_symbol(&identities[&*key].id))?;
    let mut census = CoverageDefinitionCensus::new();
    for node in nodes {
        census.define(identities[&node.id].clone())?;
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
#[cfg(test)]
pub fn derive_coverage_dependencies(
    analysis: &PlanCoverageAnalysis,
    census: &CoverageDefinitionCensus,
    relation_edges: &[RelationDependencyProjection],
) -> Result<Vec<CoverageDependency>, CompileError> {
    let (plans, mut identities) = lower_assembly(analysis);
    for node in census.nodes() {
        identities.insert(lower_node(node).id, node.clone());
    }
    let cases = analysis
        .operations
        .iter()
        .map(|(operation, analysis)| {
            (
                crate::engine_attestation::operation_key(*operation),
                analysis
                    .cases
                    .iter()
                    .cloned()
                    .map(crate::engine_placement::CaseKey::Attestation)
                    .collect(),
            )
        })
        .collect();
    let symbols = census
        .nodes()
        .filter_map(|node| {
            let lowered = lower_node(node);
            lowered.relation_case.map(|case| (case, lowered.id))
        })
        .collect();
    let relations = relation_edges
        .iter()
        .flat_map(|edge| [&edge.source, &edge.target])
        .map(|id| (crate::engine_attestation::relation_key(id), id))
        .collect::<BTreeMap<_, _>>();
    let case_ids = analysis
        .operations
        .values()
        .flat_map(|operation| &operation.cases)
        .map(|case| {
            (
                crate::engine_placement::CaseKey::Attestation(case.clone()),
                case,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let edges = relation_edges
        .iter()
        .map(|edge| {
            (
                crate::engine_attestation::relation_key(&edge.source),
                crate::engine_attestation::relation_key(&edge.target),
            )
        })
        .collect::<Vec<_>>();
    let lowered = core::prerequisites(&cases, &symbols, &edges).map_err(|pair| {
        let restore = |key: &crate::engine_placement::Key| {
            CoverageNodeId::RelationCase(RelationCaseKey {
                relation: relations[&key.relation].clone(),
                case: case_ids[&key.case].clone(),
            })
        };
        CompileError::CrossOperationCoverageDependency {
            prerequisite: Box::new(restore(&pair.0)),
            dependent: Box::new(restore(&pair.1)),
        }
    })?;
    Ok(core::derive_dependencies(&plans, &lowered)
        .into_iter()
        .map(|row| CoverageDependency {
            source: identities[&row.source].id.clone(),
            target: identities[&row.target].id.clone(),
            edge: row.edge,
        })
        .collect())
}

/// Owner identity lowering for the common two-pass assembly.
#[cfg(test)]
fn lower_assembly(
    analysis: &PlanCoverageAnalysis,
) -> (Vec<core::PlanSymbols>, BTreeMap<core::Key, CoverageNode>) {
    let mut identities = BTreeMap::new();
    let mut register = |id: CoverageNodeId, activity| {
        let node = CoverageNode { id, activity };
        let lowered = lower_node(&node);
        identities.insert(lowered.id.clone(), node);
        lowered
    };
    let mut plans = Vec::new();
    for plan in analysis.plans() {
        let key = plan.key();
        let relation = register(
            CoverageNodeId::RelationCase(key.clone()),
            Some(plan.activity),
        );
        let requirements = plan
            .positive
            .iter()
            .map(|row| &row.id)
            .chain(plan.negative.iter().map(|row| &row.id))
            .map(|id| register(CoverageNodeId::Requirement(id.clone()), None))
            .collect();
        let mut carriers = Vec::new();
        let mut layouts = Vec::new();
        for alternative in plan
            .carrier
            .iter()
            .flat_map(|carrier| &carrier.allowed_assignments)
        {
            let layout = alternative
                .layout
                .iter()
                .map(|row| register(CoverageNodeId::Layout(row.clone()), None))
                .collect::<Vec<_>>();
            layouts.extend(layout.iter().cloned());
            for carrier in &alternative.carriers {
                let node = register(
                    CoverageNodeId::Carrier {
                        relation_case: key.clone(),
                        carrier: carrier.clone(),
                    },
                    None,
                );
                carriers.push((node, layout.clone()));
            }
            // Preserve definitions even for an offered empty carrier alternative.
            // These detached layout definitions remain subject to coverage validation.
        }
        let projections = plan
            .projections
            .keys()
            .map(|boundary| {
                register(
                    CoverageNodeId::Requirement(CoverageRequirementId {
                        relation: key.relation.clone(),
                        case: key.case.clone(),
                        boundary: *boundary,
                        purpose: CoveragePurpose::AcceptedProjection,
                    }),
                    None,
                )
                .id
            })
            .collect();
        let evidence = plan
            .external_evidence
            .iter()
            .map(|value| register(CoverageNodeId::ExternalEvidence(value.clone()), None))
            .collect();
        plans.push(core::PlanSymbols {
            relation,
            requirements,
            carriers,
            layouts,
            projections,
            evidence,
        });
    }
    (plans, identities)
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
#[cfg(test)]
pub struct CoverageDependencyGraph {
    core: core::Graph,
    nodes: BTreeMap<core::Key, CoverageNode>,
    identities: BTreeMap<core::Key, CoverageNodeId>,
    cases: BTreeMap<crate::engine_placement::Key, RelationCaseKey>,
}
#[cfg(test)]
fn lower_node(node: &CoverageNode) -> core::Node {
    core::Node {
        id: core::Key::Attestation(node.id.clone()),
        operation: crate::engine_attestation::operation_key(node.id.operation()),
        class: node.id.class(),
        relation_case: match &node.id {
            CoverageNodeId::RelationCase(key) => {
                Some(crate::engine_attestation::placement::key(key))
            }
            _ => None,
        },
        active: node
            .activity
            .map(|activity| activity == RelationActivity::Active),
    }
}
#[cfg(test)]
fn lower_dependency(row: &CoverageDependency) -> core::Dependency {
    core::Dependency {
        source: core::Key::Attestation(row.source.clone()),
        target: core::Key::Attestation(row.target.clone()),
        edge: row.edge,
    }
}
#[cfg(test)]
impl CoverageDependencyGraph {
    fn add_dependencies(
        &mut self,
        dependencies: &[CoverageDependency],
    ) -> Result<(), CompileError> {
        for row in dependencies {
            self.identities.insert(
                core::Key::Attestation(row.source.clone()),
                row.source.clone(),
            );
            self.identities.insert(
                core::Key::Attestation(row.target.clone()),
                row.target.clone(),
            );
        }
        self.core
            .add_dependencies(
                &dependencies
                    .iter()
                    .map(lower_dependency)
                    .collect::<Vec<_>>(),
            )
            .map_err(|error| self.refusal(error))
    }
    pub fn dependency_collateral(&self, key: &RelationCaseKey) -> BTreeSet<RelationCaseKey> {
        self.core
            .dependency_collateral(&crate::engine_attestation::placement::key(key))
            .iter()
            .map(|key| self.cases[key].clone())
            .collect()
    }
    pub fn attach_dependency_collateral(
        &mut self,
        analysis: &PlanCoverageAnalysis,
    ) -> Result<(), CompileError> {
        let requirements = analysis
            .plans()
            .flat_map(|plan| {
                plan.negative
                    .iter()
                    .filter(|requirement| {
                        requirement.collateral.policy
                            == CollateralPolicy::RequireIntendedAndDependencyClosure
                    })
                    .map(|requirement| {
                        (
                            crate::engine_attestation::placement::key(&plan.key()),
                            core::Key::Attestation(CoverageNodeId::Requirement(
                                requirement.id.clone(),
                            )),
                        )
                    })
            })
            .collect::<Vec<_>>();
        let dependencies = self
            .core
            .collateral_dependencies(&requirements)
            .iter()
            .map(|row| self.dependency(row))
            .collect::<Vec<_>>();
        self.add_dependencies(&dependencies)
    }

    pub fn validate_acyclic(&self) -> Result<(), CompileError> {
        self.core
            .validate_acyclic()
            .map_err(|error| self.refusal(error))
    }
    #[cfg(test)]
    pub fn cyclic_components(&self) -> Vec<CoverageCycleComponent> {
        self.core
            .cyclic_components()
            .iter()
            .map(|row| self.cycle(row))
            .collect()
    }
    fn cycle(&self, row: &core::Cycle) -> CoverageCycleComponent {
        CoverageCycleComponent {
            members: row
                .members
                .iter()
                .map(|key| self.identities[key].clone())
                .collect(),
            internal_edges: row
                .internal_edges
                .iter()
                .map(|row| self.dependency(row))
                .collect(),
        }
    }
    fn dependency(&self, row: &core::Dependency) -> CoverageDependency {
        CoverageDependency {
            source: self.identities[&row.source].clone(),
            target: self.identities[&row.target].clone(),
            edge: row.edge,
        }
    }
    pub fn project(&self) -> CoverageGraphProjection {
        let result = self.core.project();
        CoverageGraphProjection {
            nodes: result
                .nodes
                .iter()
                .map(|node| self.nodes[&node.id].clone())
                .collect(),
            edges: result
                .edges
                .iter()
                .map(|row| self.dependency(row))
                .collect(),
        }
    }
    fn refusal(&self, error: core::Error) -> CompileError {
        refusal_from_identities(error, &self.identities)
    }
}

pub fn refusal_from_identities(
    error: core::Error,
    identities: &BTreeMap<core::Key, CoverageNodeId>,
) -> CompileError {
    match error {
        core::Error::UnknownSymbol(key) => CompileError::UnknownCoverageSymbol {
            symbol: identities[key.as_ref()].clone(),
        },
        core::Error::DuplicateSymbol(key) => duplicate_symbol(&identities[key.as_ref()]),
        core::Error::EndpointClass(row) => CompileError::CoverageDependencyEndpointClass {
            prerequisite: Box::new(identities[&row.source].clone()),
            dependent: Box::new(identities[&row.target].clone()),
            edge: row.edge,
        },
        core::Error::CrossOperation(row) => CompileError::CrossOperationCoverageDependency {
            prerequisite: Box::new(identities[&row.source].clone()),
            dependent: Box::new(identities[&row.target].clone()),
        },
        core::Error::DuplicateDependency(row) => CompileError::DuplicateCoverageDependency {
            prerequisite: Box::new(identities[&row.source].clone()),
            dependent: Box::new(identities[&row.target].clone()),
            edge: row.edge,
        },
        core::Error::Cycle(rows) => CompileError::CoverageDependencyCycle {
            components: rows
                .iter()
                .map(|row| CoverageCycleComponent {
                    members: row
                        .members
                        .iter()
                        .map(|key| identities[key].clone())
                        .collect(),
                    internal_edges: row
                        .internal_edges
                        .iter()
                        .map(|edge| CoverageDependency {
                            source: identities[&edge.source].clone(),
                            target: identities[&edge.target].clone(),
                            edge: edge.edge,
                        })
                        .collect(),
                })
                .collect(),
        },
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
#[cfg(test)]
pub fn build_coverage_graph(
    census: &CoverageDefinitionCensus,
    dependencies: &[CoverageDependency],
) -> Result<CoverageDependencyGraph, CompileError> {
    let span = tracing::debug_span!(target: "compiler::engine_coverage_graph", "lower_attestation_graph", definition_count = crate::instrumentation::count(census.len()));
    let _entered = span.enter();
    let nodes = census
        .nodes()
        .map(|node| (core::Key::Attestation(node.id.clone()), node.clone()))
        .collect::<BTreeMap<_, _>>();
    let identities = nodes
        .iter()
        .map(|(key, node)| (key.clone(), node.id.clone()))
        .collect();
    let cases = census
        .nodes()
        .filter_map(|node| match &node.id {
            CoverageNodeId::RelationCase(key) => {
                Some((crate::engine_attestation::placement::key(key), key.clone()))
            }
            _ => None,
        })
        .collect();
    let core = core::build(&census.nodes().map(lower_node).collect::<Vec<_>>(), &[])
        .map_err(|_| CompileError::InvalidRealization)?;
    let mut resolved = CoverageDependencyGraph {
        core,
        nodes,
        identities,
        cases,
    };
    resolved.add_dependencies(dependencies)?;
    Ok(resolved)
}

/// The realization relation dependency edges of one compiler relation
/// analysis.
#[cfg(test)]
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
#[cfg(test)]
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
#[cfg(test)]
pub fn resolve_coverage_dependencies(
    analysis: &mut PlanCoverageAnalysis,
    relations: &CompilerRelationAnalysis,
) -> Result<CoverageDependencyGraph, CompileError> {
    let graph = analyze_coverage_dependencies(analysis, relations)?;

    bind_dependency_collateral(analysis, &|key| graph.dependency_collateral(key));
    Ok(graph)
}
