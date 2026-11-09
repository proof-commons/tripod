//! Keyed owner lowering into the same concrete relation-graph record domain.
//! This module supplies records; it contains no graph algorithm or analyzer.

use architecture::{keyed::KeyedManifest, semantic::SemanticOperationId};
use realization::{
    keyed::{KeyedRealizationError, KeyedScopedRealization},
    relation::{KeyedRelationId, KeyedRelationKind},
};

use crate::engine_records::{
    LoweredRelations, OperationKey, RelationKey, RelationOwner, RelationRecord, RelationRecords,
};

/// The statically selected keyed owner retained by the input binder.
#[derive(Clone, Debug)]
pub struct KeyedOwner {
    manifest: KeyedManifest,
    source: KeyedScopedRealization,
    scope: Vec<SemanticOperationId>,
}

impl KeyedOwner {
    fn shells(&self) -> Result<EmptyShells, KeyedLoweringError> {
        read_shells(&self.source.draft(), &self.scope)
    }
    pub(crate) const fn new(
        manifest: KeyedManifest,
        source: KeyedScopedRealization,
        scope: Vec<SemanticOperationId>,
    ) -> Self {
        Self {
            manifest,
            source,
            scope,
        }
    }

    /// The declaration owner and paper binding retained at binding.
    #[must_use]
    pub const fn manifest(&self) -> &KeyedManifest {
        &self.manifest
    }

    /// The validated declaration source, with no mutable accessor.
    #[must_use]
    pub const fn source(&self) -> &KeyedScopedRealization {
        &self.source
    }

    /// The explicit compiler scope, distinct from the realization scope.
    #[must_use]
    pub fn scope(&self) -> &[SemanticOperationId] {
        &self.scope
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellField {
    Expressions,
    RelationDependencies,
    ConstructibilityNodes,
    ConstructibilityEdges,
    LifecycleNodes,
    LifecycleEdges,
    DisclosureNodes,
    DisclosureEdges,
    DisclosureSeeds,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum KeyedLoweringError {
    #[error("unsupported keyed shell {field:?} for {operation:?}: {count} entries")]
    UnsupportedShell {
        operation: SemanticOperationId,
        field: ShellField,
        count: usize,
    },
    #[cfg(test)]
    #[error("missing keyed source table for {0:?}")]
    MissingSourceTable(RelationKey),
    #[cfg(test)]
    #[error("invalid keyed runtime source payload for {0:?}")]
    RuntimeSourcePayload(RelationKey),
    #[cfg(test)]
    #[error("keyed proof bundle refused: {0:?}")]
    ProofBundle(crate::engine_requirement::ProofBundleError),
}

/// Evidence that every in-scope declaration field has been read and is empty.
/// Reading and validating these shells precedes every empty stage construction,
/// including proof and case records that do not retain this checked value.
#[derive(Debug)]
pub struct EmptyShells {
    scope: Vec<OperationKey>,
    #[cfg(test)]
    predicates: crate::engine_foundation::Records,
}
impl EmptyShells {
    fn lifecycle(&self) -> crate::engine_lifecycle::LifecycleRecords {
        crate::engine_lifecycle::LifecycleRecords::empty(self.scope.clone())
    }
    #[cfg(test)]
    fn construction(self) -> crate::engine_constructibility::Records {
        crate::engine_constructibility::Records {
            scope: self.scope,
            nodes: Vec::new(),
            edges: Vec::new(),
            operations: Vec::new(),
        }
    }
}

pub fn read_shells(
    draft: &realization::keyed::KeyedRealizationDraft,
    scope: &[SemanticOperationId],
) -> Result<EmptyShells, KeyedLoweringError> {
    use ShellField as F;
    let span = tracing::debug_span!(target: "compiler::engine_keyed", "read_declaration_shells", scope_count = crate::instrumentation::count(scope.len()));
    let _entered = span.enter();
    for declaration in draft
        .operations
        .iter()
        .filter(|declaration| scope.contains(&declaration.operation))
    {
        for (field, count) in [
            (F::Expressions, declaration.expressions.len()),
            (
                F::RelationDependencies,
                declaration.relation_dependencies.len(),
            ),
            (
                F::ConstructibilityNodes,
                declaration.constructibility_nodes.len(),
            ),
            (
                F::ConstructibilityEdges,
                declaration.constructibility_edges.len(),
            ),
            (F::LifecycleNodes, declaration.lifecycle_nodes.len()),
            (F::LifecycleEdges, declaration.lifecycle_edges.len()),
            (F::DisclosureNodes, declaration.disclosure_nodes.len()),
            (F::DisclosureEdges, declaration.disclosure_edges.len()),
            (F::DisclosureSeeds, declaration.disclosure_seeds.len()),
        ] {
            if count != 0 {
                tracing::debug!(target: "compiler::engine_keyed", outcome = "refused", ?field, field_count = crate::instrumentation::count(count), "declaration shell read");
                return Err(KeyedLoweringError::UnsupportedShell {
                    operation: declaration.operation,
                    field,
                    count,
                });
            }
        }
    }
    tracing::debug!(target: "compiler::engine_keyed", outcome = "completed", field_count = 0_u64, "declaration shell read");
    Ok(EmptyShells {
        scope: scope.iter().copied().map(operation_key).collect(),
        #[cfg(test)]
        predicates: crate::engine_foundation::Records::default(),
    })
}

/// Refusals of the keyed input binding entry, before any analysis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyedBindingError {
    InvalidRealization(KeyedRealizationError),
    EmptyScope,
    DuplicateOperation(SemanticOperationId),
    ForeignOperation(SemanticOperationId),
}

impl KeyedBindingError {
    pub(crate) const fn kind(&self) -> &'static str {
        match self {
            Self::InvalidRealization(_) => "invalid_realization",
            Self::EmptyScope => "empty_scope",
            Self::DuplicateOperation(_) => "duplicate_operation",
            Self::ForeignOperation(_) => "foreign_operation",
        }
    }
}

pub struct KeyedRelations {
    records: RelationRecords,
    lifecycle: crate::engine_lifecycle::LifecycleRecords,
}

impl LoweredRelations for KeyedRelations {
    fn records(&self) -> &RelationRecords {
        &self.records
    }
    fn lifecycle(&self) -> &crate::engine_lifecycle::LifecycleRecords {
        &self.lifecycle
    }
}

impl RelationOwner for KeyedOwner {
    type Lowered = KeyedRelations;
    type Error = KeyedLoweringError;
    fn source_binding(&self) -> crate::engine_program::Source {
        crate::engine_program::Source::Keyed {
            manifest: Box::new(self.manifest.clone()),
            realization: self.source.draft(),
            scope: self.scope.clone(),
        }
    }
    #[cfg(test)]
    fn lower_requirements(
        input: &crate::input::BoundCompilerInput<Self>,
    ) -> Result<Vec<crate::engine_requirement::Table>, KeyedLoweringError> {
        crate::engine_requirement::proof_tables(&Self::lower_proofs(input)?)
            .map_err(KeyedLoweringError::ProofBundle)
    }
    #[cfg(test)]
    fn lower_predicates(
        input: &crate::input::BoundCompilerInput<Self>,
    ) -> Result<crate::engine_foundation::Records, KeyedLoweringError> {
        let span = tracing::debug_span!(target: "compiler::engine_keyed", "lower_predicates", scope_count = crate::instrumentation::count(input.owner().scope.len()));
        let _entered = span.enter();
        Ok(input.owner().shells()?.predicates)
    }
    #[cfg(test)]
    fn lower_proofs(
        input: &crate::input::BoundCompilerInput<Self>,
    ) -> Result<crate::engine_proof::Records, KeyedLoweringError> {
        use crate::{engine_proof as core, engine_source::Payload};
        let source = input.owner().lower_sources()?;
        // Row validity belongs to the shared source stage. Lowering only
        // records the owner's source declaration and checked empty shells.
        input.owner().shells()?;
        let obligations = source
            .tables
            .into_iter()
            .map(|table| {
                let evidence = table
                    .rows
                    .iter()
                    .filter_map(|row| match &row.operand.payload {
                        Payload::Evidence(value) => Some(value.clone()),
                        _ => None,
                    })
                    .collect::<std::collections::BTreeSet<_>>();
                let external = !evidence.is_empty();
                let relation = table.relation;
                let alternative = core::Alternative {
                    id: core::AlternativeKey {
                        relation: relation.clone(),
                        namespace: table.proof_namespace,
                        kind: table.proof_kind,
                    },
                    capabilities: std::collections::BTreeSet::new(),
                    sources: Ok(table.rows),
                    constructible: true,
                    construction: Vec::new(),
                    evidence,
                    incompatible: std::collections::BTreeSet::new(),
                };
                (
                    relation,
                    if external {
                        core::Obligation::Evidence(alternative)
                    } else {
                        core::Obligation::Required(vec![alternative])
                    },
                )
            })
            .collect();
        Ok(core::Records {
            obligations,
            choices: Vec::new(),
            lifecycle: Vec::new(),
            disclosure: core::DisclosureRecords {
                valid: true,
                retained: std::collections::BTreeSet::new(),
                public_effects: std::collections::BTreeMap::new(),
            },
        })
    }
    #[cfg(test)]
    fn lower_cases(
        input: &crate::input::BoundCompilerInput<Self>,
    ) -> Result<crate::engine_cases::Tables, KeyedLoweringError> {
        use crate::{engine_cases as core, engine_placement::CaseKey};
        let owner = input.owner();
        owner.shells()?;
        let placement = lower_placements(owner);
        let sources = lower_sources(owner);
        let coverage = lower_coverage(owner)
            .into_iter()
            .map(|plan| (plan.key.clone(), plan))
            .collect::<std::collections::BTreeMap<_, _>>();
        let rows = placement
            .plans
            .iter()
            .map(|(key, runtime)| {
                Ok(core::Row {
                    key: key.clone(),
                    proof: None,
                    runtime: *runtime,
                    eligibility: placement
                        .eligibility
                        .iter()
                        .filter(|row| row.key == *key)
                        .cloned()
                        .collect(),
                    coverage: Ok(coverage[key].clone()),
                    refusal: None,
                    facts: Some(case_facts(key, *runtime, &coverage[key], &sources)?),
                    projections: coverage[key].boundaries.clone(),
                    collateral: coverage[key]
                        .negative
                        .iter()
                        .filter(|row| {
                            row.boundary == crate::coverage::CoverageBoundary::RuntimeCarrier
                        })
                        .cloned()
                        .collect(),
                })
            })
            .collect::<Result<Vec<_>, KeyedLoweringError>>()?;
        Ok(core::Tables {
            layout_scopes: std::collections::BTreeMap::new(),
            choices: Vec::new(),
            relations: placement
                .plans
                .into_iter()
                .map(|(key, _)| key.relation)
                .collect(),
            cases: owner
                .scope
                .iter()
                .map(|operation| {
                    let operation = operation_key(*operation);
                    core::Case {
                        id: CaseKey::Keyed(operation),
                        operation,
                        representations: std::collections::BTreeMap::new(),
                    }
                })
                .collect(),
            rows,
        })
    }
    fn lower_sources(&self) -> Result<crate::engine_source::SourceRecords, KeyedLoweringError> {
        self.shells()?;
        Ok(lower_sources(self))
    }
    #[cfg(test)]
    fn lower_constructibility(
        &self,
    ) -> Result<crate::engine_constructibility::Records, KeyedLoweringError> {
        Ok(self.shells()?.construction())
    }

    fn lower_relations(&self) -> Result<Self::Lowered, KeyedLoweringError> {
        let shells = self.shells()?;
        let span = tracing::debug_span!(target: "compiler::engine_keyed", "lower_relations", scope_operation_count = crate::instrumentation::count(self.scope.len()));
        let _entered = span.enter();
        let nodes = self
            .source
            .project()
            .into_iter()
            .map(|declaration| RelationRecord {
                id: relation_key(declaration.id),
                proof_owners: Vec::new(),
            })
            .collect::<Vec<_>>();
        tracing::debug!(target: "compiler::engine_keyed", outcome = "completed", source_node_count = crate::instrumentation::count(nodes.len()), "relation records lowered");
        Ok(KeyedRelations {
            lifecycle: shells.lifecycle(),
            records: RelationRecords {
                scope: shells.scope,
                nodes,
                edges: Vec::new(),
            },
        })
    }
}

fn lower_sources(owner: &KeyedOwner) -> crate::engine_source::SourceRecords {
    use crate::{engine_lifecycle::ObjectKey, engine_source as core};
    use realization::relation::KeyedRelation;
    let span = tracing::debug_span!(target: "compiler::engine_keyed", "lower_sources", scope_count = crate::instrumentation::count(owner.scope.len()));
    let _entered = span.enter();
    let declarations = owner
        .source
        .project()
        .into_iter()
        .filter(|row| owner.scope.contains(&row.id.operation))
        .collect::<Vec<_>>();
    let tables = declarations.iter().map(source_table).collect();
    let regions = owner
        .scope
        .iter()
        .map(|operation| core::RegionRecord {
            operation: operation_key(*operation),
            declared_families: declarations
                .iter()
                .filter(|row| row.id.operation == *operation)
                .filter_map(|row| match row.relation {
                    KeyedRelation::HandleFamily { .. } => {
                        let [namespace, code] = row.id.object.encoding();
                        Some(ObjectKey { namespace, code })
                    }
                    _ => None,
                })
                .collect(),
            declared_flows: std::collections::BTreeSet::new(),
            funding_family: None,
            erased_amount_family: None,
            sponsor_conditional_family: None,
            decidable: true,
        })
        .collect();
    core::SourceRecords { tables, regions }
}

pub const fn operation_key(operation: SemanticOperationId) -> OperationKey {
    let [namespace, code] = operation.encoding();
    OperationKey { namespace, code }
}

#[cfg(test)]
pub fn case_facts(
    key: &crate::engine_placement::Key,
    runtime: bool,
    coverage: &crate::engine_coverage::Plan,
    sources: &crate::engine_source::SourceRecords,
) -> Result<crate::engine_case_facts::Facts, KeyedLoweringError> {
    use crate::{
        engine_case_facts as core,
        engine_source::{Activation, Payload},
    };
    let rows = &sources
        .tables
        .iter()
        .find(|table| table.relation == key.relation)
        .ok_or_else(|| KeyedLoweringError::MissingSourceTable(key.relation.clone()))?
        .rows;
    let runtime = if runtime {
        let Some(Payload::Object(object)) = rows.first().map(|row| &row.operand.payload) else {
            return Err(KeyedLoweringError::RuntimeSourcePayload(
                key.relation.clone(),
            ));
        };
        vec![core::Runtime {
            scope: core::Scope::Member {
                side: 0,
                object: *object,
            },
            multiplicity: crate::placement::CarrierMultiplicity::EveryMember,
            sources: rows.clone(),
        }]
    } else {
        Vec::new()
    };
    Ok(core::Facts {
        key: key.clone(),
        case_operation: key.relation.operation,
        activation: Activation::Always,
        active: coverage.active,
        boundaries: coverage.boundaries.clone(),
        sources: rows.iter().cloned().collect(),
        compiler: std::collections::BTreeSet::new(),
        structural: std::collections::BTreeSet::new(),
        runtime,
        evidence: coverage.evidence.clone(),
        layout: std::collections::BTreeSet::new(),
    })
}

pub fn relation_key(id: KeyedRelationId) -> RelationKey {
    let kind = match id.kind {
        KeyedRelationKind::InputFamily => 0,
        KeyedRelationKind::OutputFamily => 1,
        KeyedRelationKind::BornFirst => 2,
        KeyedRelationKind::ContinueFirst => 3,
        KeyedRelationKind::SourceClassification => 4,
        KeyedRelationKind::MembershipReading => 5,
    };
    RelationKey {
        operation: operation_key(id.operation),
        kind,
        subject: id.object.encoding().to_vec(),
    }
}

fn source_table(
    row: &realization::relation::KeyedRelationDeclaration,
) -> crate::engine_source::SourceTable {
    use crate::{
        engine_constructibility::Availability, engine_lifecycle::ObjectKey, engine_source as core,
        source::RequiredSourceKind,
    };
    use realization::relation::{KeyedCardinality, KeyedRelation, RecordRecurrence};
    let [namespace, code] = row.id.object.encoding();
    let object = ObjectKey { namespace, code };
    let (payload, source) = match row.relation {
        KeyedRelation::HandleFamily { side, cardinality } => {
            let (minimum, maximum) = match cardinality {
                KeyedCardinality::Zero => (0, Some(0)),
                KeyedCardinality::One => (1, Some(1)),
                KeyedCardinality::OneOrMore => (1, None),
            };
            (
                core::Payload::Evidence(core::Evidence::HandleClassification {
                    operation: operation_key(row.id.operation),
                    object,
                    side: side as u16,
                    minimum,
                    maximum,
                }),
                RequiredSourceKind::ExternalEvidence,
            )
        }
        KeyedRelation::SourceClassification => (
            core::Payload::Evidence(core::Evidence::SourceClassification {
                operation: operation_key(row.id.operation),
                object,
            }),
            RequiredSourceKind::ExternalEvidence,
        ),
        KeyedRelation::MembershipReading { recurrence } => (
            core::Payload::Evidence(core::Evidence::MembershipReading {
                operation: operation_key(row.id.operation),
                object,
                recurrence: match recurrence {
                    RecordRecurrence::SourceSingleton => core::Recurrence::Singleton,
                    RecordRecurrence::ConsumedUnion => core::Recurrence::ConsumedUnion,
                },
                inclusion: true,
                retrieval: true,
                finality_for_final_reading: true,
            }),
            RequiredSourceKind::ExternalEvidence,
        ),
        KeyedRelation::BornFirst | KeyedRelation::ContinueFirst => (
            core::Payload::Object(object),
            RequiredSourceKind::AuthenticatedInputObject,
        ),
    };
    let relation = relation_key(row.id);
    let operand = core::OperandKey {
        relation: relation.clone(),
        role: 0,
        payload,
    };
    core::SourceTable {
        relation,
        proof_namespace: namespace,
        proof_kind: 0,
        expected: vec![operand.clone()],
        rows: vec![core::SourceRow {
            operand,
            source,
            availability: Availability {
                namespace,
                kind: 0,
                object: 0,
                sponsor_local: false,
            },
            activation: core::Activation::Always,
        }],
    }
}

#[cfg(test)]
pub fn lower_placements(owner: &KeyedOwner) -> crate::engine_placement::Records {
    use crate::{engine_lifecycle::ObjectKey, engine_placement as core};
    use realization::relation::{KeyedCardinality, KeyedRelation};
    let span = tracing::debug_span!(target: "compiler::engine_keyed", "lower_placements", scope_count = crate::instrumentation::count(owner.scope.len()));
    let _entered = span.enter();
    let declarations = owner.source.project();
    let mut plans = Vec::new();
    let mut eligibility = Vec::new();
    for row in declarations
        .iter()
        .filter(|row| owner.scope.contains(&row.id.operation))
    {
        let relation = relation_key(row.id);
        let operation = operation_key(row.id.operation);
        let key = core::Key {
            relation,
            case: core::CaseKey::Keyed(operation),
        };
        let runtime = match row.relation {
            KeyedRelation::BornFirst | KeyedRelation::ContinueFirst => true,
            KeyedRelation::HandleFamily { .. }
            | KeyedRelation::SourceClassification
            | KeyedRelation::MembershipReading { .. } => false,
        };
        plans.push((key.clone(), runtime));
        if !runtime {
            continue;
        }
        let entries = declarations
            .iter()
            .filter_map(|family| {
                let mandatory = matches!(
                    family.relation,
                    KeyedRelation::HandleFamily {
                        side: realization::TransactionSide::Input,
                        cardinality: KeyedCardinality::One | KeyedCardinality::OneOrMore
                    }
                );
                if family.id.operation != row.id.operation || !mandatory {
                    return None;
                }
                let [namespace, code] = family.id.object.encoding();
                Some(core::Entry {
                    placed: core::Placed {
                        carrier: core::CarrierKey::Member {
                            operation,
                            object: ObjectKey { namespace, code },
                        },
                        quantification: crate::carrier::CarrierQuantification::PerMember,
                    },
                    layout: std::collections::BTreeSet::new(),
                })
            })
            .collect::<Vec<_>>();
        eligibility.push(core::Eligibility {
            key,
            multiplicity: crate::placement::CarrierMultiplicity::EveryMember,
            options: entries.clone(),
            entries,
            empty_cause: core::EmptyCause::NoEligible,
        });
    }
    core::Records { plans, eligibility }
}

#[cfg(test)]
pub fn lower_coverage(owner: &KeyedOwner) -> Vec<crate::engine_coverage::Plan> {
    use crate::{coverage::CoverageBoundary as B, engine_coverage as core, engine_source::Payload};
    let source = lower_sources(owner);
    let span = tracing::debug_span!(target: "compiler::engine_keyed", "lower_coverage", relation_count = crate::instrumentation::count(source.tables.len()));
    let _entered = span.enter();
    source
        .tables
        .into_iter()
        .map(|table| {
            let key = crate::engine_placement::Key {
                case: crate::engine_placement::CaseKey::Keyed(table.relation.operation),
                relation: table.relation,
            };
            let evidence = table
                .rows
                .iter()
                .filter_map(|row| match &row.operand.payload {
                    Payload::Evidence(evidence) => Some(evidence.clone()),
                    _ => None,
                })
                .collect::<std::collections::BTreeSet<_>>();
            let boundary = if evidence.is_empty() {
                B::RuntimeCarrier
            } else {
                B::ExternalEvidence
            };
            let requirement = |purpose| core::Requirement {
                key: key.clone(),
                boundary,
                purpose,
                inactive: false,
            };
            let subjects = if evidence.is_empty() {
                vec![core::Subject {
                    positive: requirement(core::Purpose::ClauseAccepted),
                    negative: vec![requirement(core::Purpose::ClauseRejected)],
                }]
            } else {
                evidence
                    .iter()
                    .map(|evidence| core::Subject {
                        positive: requirement(core::Purpose::Verified(evidence.clone())),
                        negative: [
                            core::EvidenceDefect::Missing,
                            core::EvidenceDefect::Failed,
                            core::EvidenceDefect::IdentityMismatch,
                        ]
                        .into_iter()
                        .map(|defect| {
                            requirement(core::Purpose::Rejected {
                                evidence: evidence.clone(),
                                defect,
                            })
                        })
                        .collect(),
                    })
                    .collect()
            };
            core::derive(core::Matrix {
                key,
                active: true,
                boundaries: std::collections::BTreeSet::from([boundary]),
                inactive: Vec::new(),
                subjects,
                evidence,
            })
        })
        .collect()
}
