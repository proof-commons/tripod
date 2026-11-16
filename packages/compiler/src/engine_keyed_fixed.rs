//! Fixed-form keyed owner lowering into the same concrete relation-graph record domain.
//! This module supplies records; it contains no graph algorithm or analyzer.

pub mod program;

use architecture::keyed_fixed::KeyedFixedOperationId;
use realization::{
    keyed_fixed::KeyedFixedRealizationError, keyed_fixed_relation::KeyedFixedRelationId,
};

use crate::engine_records::{
    LoweredRelations, OperationKey, RelationKey, RelationOwner, RelationRecord, RelationRecords,
};

use crate::input::KeyedFixedOwner;

impl KeyedFixedOwner {
    fn shells(&self) -> Result<EmptyShells, KeyedFixedLoweringError> {
        check_clauses(self.manifest(), &self.source().project())?;
        read_shells(&self.source().draft(), self.scope())
    }
}

fn check_clauses(
    manifest: &architecture::keyed_fixed::KeyedFixedManifest,
    declarations: &[realization::keyed_fixed_relation::KeyedFixedRelationDeclaration],
) -> Result<(), KeyedFixedLoweringError> {
    use architecture::keyed_fixed::KeyedFixedClause as C;
    use realization::keyed_fixed_relation::KeyedFixedRelation as R;
    let mut expected = manifest.clauses.clone();
    expected.sort();
    let mut actual = declarations
        .iter()
        .filter_map(|row| match row.relation {
            R::BornFirst => Some(C::BornFirst),
            R::OwnQualification => Some(C::OwnQualification),
            R::SuccessorQualification => Some(C::SuccessorQualification),
            R::HandleFamily { .. } | R::SourceClassification | R::MembershipReading { .. } => None,
        })
        .collect::<Vec<_>>();
    actual.sort();
    if actual != expected {
        return Err(KeyedFixedLoweringError::ClauseCensus);
    }
    Ok(())
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
pub enum KeyedFixedLoweringError {
    #[error("fixed-form manifest clauses disagree with declaration clauses")]
    ClauseCensus,
    #[error("unsupported fixed-form keyed shell {field:?} for {operation:?}: {count} entries")]
    UnsupportedShell {
        operation: KeyedFixedOperationId,
        field: ShellField,
        count: usize,
    },
    #[error("missing fixed-form keyed source table for {0:?}")]
    MissingSourceTable(RelationKey),
    #[error("invalid fixed-form keyed runtime source payload for {0:?}")]
    RuntimeSourcePayload(RelationKey),
    #[error("fixed-form keyed proof bundle refused: {0:?}")]
    ProofBundle(crate::engine_requirement::ProofBundleError),
}

/// Evidence that every in-scope declaration field has been read and is empty.
/// Reading and validating these shells precedes every empty stage construction,
/// including proof and case records that do not retain this checked value.
#[derive(Debug)]
pub struct EmptyShells {
    scope: Vec<OperationKey>,
    predicates: crate::engine_foundation::Records,
}
impl EmptyShells {
    fn lifecycle(&self) -> crate::engine_lifecycle::LifecycleRecords {
        crate::engine_lifecycle::LifecycleRecords::empty(self.scope.clone())
    }
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
    draft: &realization::keyed_fixed::KeyedFixedRealizationDraft,
    scope: &[KeyedFixedOperationId],
) -> Result<EmptyShells, KeyedFixedLoweringError> {
    use ShellField as F;
    let span = tracing::debug_span!(target: "compiler::engine_keyed_fixed", "read_declaration_shells", scope_count = crate::instrumentation::count(scope.len()));
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
                tracing::debug!(target: "compiler::engine_keyed_fixed", outcome = "refused", ?field, field_count = crate::instrumentation::count(count), "declaration shell read");
                return Err(KeyedFixedLoweringError::UnsupportedShell {
                    operation: declaration.operation,
                    field,
                    count,
                });
            }
        }
    }
    tracing::debug!(target: "compiler::engine_keyed_fixed", outcome = "completed", field_count = 0_u64, "declaration shell read");
    Ok(EmptyShells {
        scope: scope.iter().copied().map(operation_key).collect(),
        predicates: crate::engine_foundation::Records::default(),
    })
}

/// Refusals of the fixed-form keyed input binding entry, before any analysis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyedFixedBindingError {
    InvalidRealization(KeyedFixedRealizationError),
    EmptyScope,
    DuplicateOperation(KeyedFixedOperationId),
    ForeignOperation(KeyedFixedOperationId),
}

impl KeyedFixedBindingError {
    pub(crate) const fn kind(&self) -> &'static str {
        match self {
            Self::InvalidRealization(_) => "invalid_realization",
            Self::EmptyScope => "empty_scope",
            Self::DuplicateOperation(_) => "duplicate_operation",
            Self::ForeignOperation(_) => "foreign_operation",
        }
    }
}

pub struct KeyedFixedRelations {
    records: RelationRecords,
    lifecycle: crate::engine_lifecycle::LifecycleRecords,
    #[cfg(test)]
    pub identities: KeyedFixedIdentities,
}

/// Reverse identities stay with the binding-fixed fixed-form keyed lowering, never in
/// the shared engine. Production publication and refusal restoration use
/// only this binding-fixed owner registry.
pub struct KeyedFixedIdentities {
    pub cases: std::collections::BTreeMap<crate::engine_placement::CaseKey, KeyedFixedOperationId>,
    pub carriers: std::collections::BTreeMap<
        crate::engine_placement::CarrierKey,
        (
            KeyedFixedOperationId,
            architecture::keyed_fixed::KeyedFixedObjectId,
        ),
    >,
    pub purposes: std::collections::BTreeMap<crate::engine_identity::PurposeKey, KeyedFixedPurpose>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeyedFixedPurpose {
    Verified(crate::engine_source::Evidence),
    Rejected {
        evidence: crate::engine_source::Evidence,
        defect: EvidenceDefect,
    },
    ClauseAccepted,
    ClauseRejected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvidenceDefect {
    Missing,
    Failed,
    IdentityMismatch,
}

impl KeyedFixedIdentities {
    #[cfg(test)]
    fn new(owner: &KeyedFixedOwner) -> Self {
        Self::from_source(owner, &lower_sources(owner))
    }

    fn from_source(owner: &KeyedFixedOwner, sources: &crate::engine_source::SourceRecords) -> Self {
        let cases = owner
            .scope()
            .iter()
            .map(|operation| {
                (
                    crate::engine_keyed_fixed::keyed_fixed_case(operation_key(*operation)),
                    *operation,
                )
            })
            .collect();
        let carriers = owner
            .source()
            .project()
            .iter()
            .filter(|row| owner.scope().contains(&row.id.operation))
            .map(|row| {
                let [namespace, code] = crate::engine_identity::fixed_object(row.id.object);
                (
                    crate::engine_keyed_fixed::keyed_fixed_carrier(
                        operation_key(row.id.operation),
                        crate::engine_lifecycle::ObjectKey { namespace, code },
                    ),
                    (row.id.operation, row.id.object),
                )
            })
            .collect();
        let mut purposes = vec![
            KeyedFixedPurpose::ClauseAccepted,
            KeyedFixedPurpose::ClauseRejected,
        ];
        for table in &sources.tables {
            for row in &table.rows {
                if let crate::engine_source::Payload::Evidence(evidence) = &row.operand.payload {
                    purposes.push(KeyedFixedPurpose::Verified(evidence.clone()));
                    purposes.extend(
                        [
                            EvidenceDefect::Missing,
                            EvidenceDefect::Failed,
                            EvidenceDefect::IdentityMismatch,
                        ]
                        .into_iter()
                        .map(|defect| KeyedFixedPurpose::Rejected {
                            evidence: evidence.clone(),
                            defect,
                        }),
                    );
                }
            }
        }
        Self {
            cases,
            carriers,
            purposes: purposes
                .into_iter()
                .map(|value| {
                    (
                        crate::engine_keyed_fixed::keyed_fixed_purpose(&value),
                        value,
                    )
                })
                .collect(),
        }
    }

    pub fn case(
        &self,
        key: &crate::engine_placement::CaseKey,
    ) -> Result<KeyedFixedOperationId, crate::CompileError> {
        self.cases
            .get(key)
            .copied()
            .ok_or(crate::CompileError::MissingOwnerIdentity)
    }

    pub fn carrier(
        &self,
        key: &crate::engine_placement::CarrierKey,
    ) -> Result<
        (
            KeyedFixedOperationId,
            architecture::keyed_fixed::KeyedFixedObjectId,
        ),
        crate::CompileError,
    > {
        self.carriers
            .get(key)
            .copied()
            .ok_or(crate::CompileError::MissingOwnerIdentity)
    }

    pub fn purpose(
        &self,
        key: &crate::engine_identity::PurposeKey,
    ) -> Result<KeyedFixedPurpose, crate::CompileError> {
        self.purposes
            .get(key)
            .cloned()
            .ok_or(crate::CompileError::MissingOwnerIdentity)
    }
}

impl LoweredRelations for KeyedFixedRelations {
    fn records(&self) -> &RelationRecords {
        &self.records
    }
    fn lifecycle(&self) -> &crate::engine_lifecycle::LifecycleRecords {
        &self.lifecycle
    }
}

impl RelationOwner for KeyedFixedOwner {
    type Lowered = KeyedFixedRelations;
    type Error = KeyedFixedLoweringError;
    fn source_binding(&self) -> crate::engine_program::Source {
        crate::engine_program::Source::KeyedFixed {
            manifest: Box::new(self.manifest().clone()),
            realization: self.source().draft(),
            scope: self.scope().to_vec(),
        }
    }
    #[cfg(test)]
    fn lower_requirements(
        input: &crate::input::BoundCompilerInput<Self>,
    ) -> Result<Vec<crate::engine_requirement::Table>, KeyedFixedLoweringError> {
        oracle_lower_requirements(input)
    }
    #[cfg(test)]
    fn lower_predicates(
        input: &crate::input::BoundCompilerInput<Self>,
    ) -> Result<crate::engine_foundation::Records, KeyedFixedLoweringError> {
        oracle_lower_predicates(input)
    }
    #[cfg(test)]
    fn lower_proofs(
        input: &crate::input::BoundCompilerInput<Self>,
    ) -> Result<crate::engine_proof::Records, KeyedFixedLoweringError> {
        oracle_lower_proofs(input)
    }
    #[cfg(test)]
    fn lower_cases(
        input: &crate::input::BoundCompilerInput<Self>,
    ) -> Result<crate::engine_cases::Tables, KeyedFixedLoweringError> {
        oracle_lower_cases(input)
    }
    fn lower_sources(
        &self,
    ) -> Result<crate::engine_source::SourceRecords, KeyedFixedLoweringError> {
        self.shells()?;
        Ok(lower_sources(self))
    }
    #[cfg(test)]
    fn lower_constructibility(
        &self,
    ) -> Result<crate::engine_constructibility::Records, KeyedFixedLoweringError> {
        oracle_lower_constructibility(self)
    }

    fn lower_relations(&self) -> Result<Self::Lowered, KeyedFixedLoweringError> {
        let shells = self.shells()?;
        let span = tracing::debug_span!(target: "compiler::engine_keyed_fixed", "lower_relations", scope_operation_count = crate::instrumentation::count(self.scope().len()));
        let _entered = span.enter();
        let nodes = self
            .source()
            .project()
            .into_iter()
            .map(|declaration| RelationRecord {
                id: relation_key(declaration.id),
                proof_owners: Vec::new(),
            })
            .collect::<Vec<_>>();
        tracing::debug!(target: "compiler::engine_keyed_fixed", outcome = "completed", source_node_count = crate::instrumentation::count(nodes.len()), "relation records lowered");
        Ok(KeyedFixedRelations {
            #[cfg(test)]
            identities: KeyedFixedIdentities::new(self),
            lifecycle: shells.lifecycle(),
            records: RelationRecords {
                scope: shells.scope,
                nodes,
                edges: Vec::new(),
            },
        })
    }
}

fn lower_sources(owner: &KeyedFixedOwner) -> crate::engine_source::SourceRecords {
    use crate::{engine_lifecycle::ObjectKey, engine_source as core};
    use realization::keyed_fixed_relation::KeyedFixedRelation;
    let span = tracing::debug_span!(target: "compiler::engine_keyed_fixed", "lower_sources", scope_count = crate::instrumentation::count(owner.scope().len()));
    let _entered = span.enter();
    let declarations = owner
        .source()
        .project()
        .into_iter()
        .filter(|row| owner.scope().contains(&row.id.operation))
        .collect::<Vec<_>>();
    let tables = declarations.iter().copied().map(source_table).collect();
    let regions = owner
        .scope()
        .iter()
        .map(|operation| core::RegionRecord {
            operation: operation_key(*operation),
            declared_families: declarations
                .iter()
                .filter(|row| row.id.operation == *operation)
                .filter_map(|row| match row.relation {
                    KeyedFixedRelation::HandleFamily { .. } => {
                        let [namespace, code] = crate::engine_identity::fixed_object(row.id.object);
                        Some(ObjectKey { namespace, code })
                    }
                    KeyedFixedRelation::SourceClassification
                    | KeyedFixedRelation::MembershipReading { .. }
                    | KeyedFixedRelation::BornFirst
                    | KeyedFixedRelation::OwnQualification
                    | KeyedFixedRelation::SuccessorQualification => None,
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

pub const fn operation_key(operation: KeyedFixedOperationId) -> OperationKey {
    let [namespace, code] = crate::engine_identity::fixed_operation(operation);
    OperationKey { namespace, code }
}

pub fn case_facts(
    key: &crate::engine_placement::Key,
    runtime: bool,
    coverage: &crate::engine_coverage::Plan,
    sources: &crate::engine_source::SourceRecords,
) -> Result<crate::engine_case_facts::Facts, KeyedFixedLoweringError> {
    use crate::{
        engine_case_facts as core,
        engine_source::{Activation, Payload},
    };
    let rows = &sources
        .tables
        .iter()
        .find(|table| table.relation == key.relation)
        .ok_or_else(|| KeyedFixedLoweringError::MissingSourceTable(key.relation.clone()))?
        .rows;
    let runtime = if runtime {
        let Some(Payload::Object(object)) = rows.first().map(|row| &row.operand.payload) else {
            return Err(KeyedFixedLoweringError::RuntimeSourcePayload(
                key.relation.clone(),
            ));
        };
        vec![core::Runtime {
            scope: core::Scope::Member {
                side: crate::engine_identity::side_code(realization::TransactionSide::Input),
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

pub fn relation_key(id: KeyedFixedRelationId) -> RelationKey {
    let [op_namespace, op_code, object_namespace, object_code, kind] = id.encoding();
    RelationKey {
        operation: OperationKey {
            namespace: op_namespace,
            code: op_code,
        },
        kind,
        subject: vec![object_namespace, object_code],
    }
}

fn source_table(
    row: realization::keyed_fixed_relation::KeyedFixedRelationDeclaration,
) -> crate::engine_source::SourceTable {
    use crate::{
        engine_constructibility::Availability, engine_lifecycle::ObjectKey, engine_source as core,
        source::RequiredSourceKind,
    };
    use realization::keyed_fixed_relation::{
        KeyedFixedCardinality, KeyedFixedRelation, RecordRecurrence,
    };
    let [namespace, code] = crate::engine_identity::fixed_object(row.id.object);
    let object = ObjectKey { namespace, code };
    let (payload, source) = match row.relation {
        KeyedFixedRelation::HandleFamily { side, cardinality } => {
            let (minimum, maximum) = match cardinality {
                KeyedFixedCardinality::Zero => (0, Some(0)),
                KeyedFixedCardinality::One => (1, Some(1)),
                KeyedFixedCardinality::OneOrMore => (1, None),
            };
            (
                core::Payload::Evidence(core::Evidence::HandleClassification {
                    operation: operation_key(row.id.operation),
                    object,
                    side: crate::engine_identity::side_code(side),
                    minimum,
                    maximum,
                }),
                RequiredSourceKind::ExternalEvidence,
            )
        }
        KeyedFixedRelation::SourceClassification => (
            core::Payload::Evidence(core::Evidence::SourceClassification {
                operation: operation_key(row.id.operation),
                object,
            }),
            RequiredSourceKind::ExternalEvidence,
        ),
        KeyedFixedRelation::MembershipReading { recurrence } => (
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
        KeyedFixedRelation::BornFirst
        | KeyedFixedRelation::OwnQualification
        | KeyedFixedRelation::SuccessorQualification => (
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
    let mut table = core::SourceTable {
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
    };
    qualification_sources(row.relation, &mut table);
    table
}

fn qualification_sources(
    relation: realization::keyed_fixed_relation::KeyedFixedRelation,
    table: &mut crate::engine_source::SourceTable,
) {
    use crate::{engine_source as core, source::RequiredSourceKind};
    use realization::keyed_fixed_relation::KeyedFixedRelation;
    let namespace = table.proof_namespace;
    // The authenticated input object includes its birth position, program and
    // asset/value fields and forms. Qualification compares against the symbolic
    // triple; it never sums amounts or compares an input value with an output.
    if matches!(relation, KeyedFixedRelation::SuccessorQualification) {
        let mut output = table.rows[0].clone();
        output.operand.role = 1;
        output.source = RequiredSourceKind::AuthenticatedOutputObject;
        table.expected.push(output.operand.clone());
        table.rows.push(output);
    }
    if matches!(
        relation,
        KeyedFixedRelation::OwnQualification | KeyedFixedRelation::SuccessorQualification
    ) {
        use architecture::keyed_fixed::FixedParameterId;
        for parameter in [
            FixedParameterId::Key,
            FixedParameterId::Asset,
            FixedParameterId::Value,
        ] {
            let mut constant = table.rows[0].clone();
            constant.operand.role = parameter as u16 + 1;
            constant.operand.payload = core::Payload::Bound {
                namespace,
                code: parameter as u16,
            };
            constant.source = RequiredSourceKind::PublicConstructionData;
            table.expected.push(constant.operand.clone());
            table.rows.push(constant);
        }
    }
}

pub fn lower_placements(owner: &KeyedFixedOwner) -> crate::engine_placement::Records {
    use crate::{engine_lifecycle::ObjectKey, engine_placement as core};
    use realization::keyed_fixed_relation::{KeyedFixedCardinality, KeyedFixedRelation};
    let span = tracing::debug_span!(target: "compiler::engine_keyed_fixed", "lower_placements", scope_count = crate::instrumentation::count(owner.scope().len()));
    let _entered = span.enter();
    let declarations = owner.source().project();
    let mut plans = Vec::new();
    let mut eligibility = Vec::new();
    for row in declarations
        .iter()
        .filter(|row| owner.scope().contains(&row.id.operation))
    {
        let relation = relation_key(row.id);
        let operation = operation_key(row.id.operation);
        let key = core::Key {
            relation,
            case: crate::engine_keyed_fixed::keyed_fixed_case(operation),
        };
        let runtime = match row.relation {
            KeyedFixedRelation::BornFirst
            | KeyedFixedRelation::OwnQualification
            | KeyedFixedRelation::SuccessorQualification => true,
            KeyedFixedRelation::HandleFamily { .. }
            | KeyedFixedRelation::SourceClassification
            | KeyedFixedRelation::MembershipReading { .. } => false,
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
                    KeyedFixedRelation::HandleFamily {
                        side: realization::TransactionSide::Input,
                        cardinality: KeyedFixedCardinality::One | KeyedFixedCardinality::OneOrMore
                    }
                );
                if family.id.operation != row.id.operation || !mandatory {
                    return None;
                }
                let [namespace, code] = crate::engine_identity::fixed_object(family.id.object);
                Some(core::Entry {
                    placed: core::Placed {
                        carrier: crate::engine_keyed_fixed::keyed_fixed_carrier(
                            operation,
                            ObjectKey { namespace, code },
                        ),
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
pub fn lower_coverage(owner: &KeyedFixedOwner) -> Vec<crate::engine_coverage::Plan> {
    coverage_records(&lower_sources(owner))
}

use crate::engine_identity::{CarrierKey, CaseKey, Part, PurposeKey, sequence, word};
use architecture::semantic::SemanticProfileId;
pub fn keyed_fixed_case(operation: crate::engine_records::OperationKey) -> CaseKey {
    CaseKey::new(
        SemanticProfileId::KeyedFixedForm,
        vec![word(operation.namespace), word(operation.code)],
    )
}
pub fn keyed_fixed_carrier(
    operation: crate::engine_records::OperationKey,
    object: crate::engine_lifecycle::ObjectKey,
) -> CarrierKey {
    CarrierKey::new(
        SemanticProfileId::KeyedFixedForm,
        vec![
            word(operation.namespace),
            word(operation.code),
            word(object.namespace),
            word(object.code),
        ],
    )
}

pub fn keyed_fixed_purpose(value: &crate::engine_keyed_fixed::KeyedFixedPurpose) -> PurposeKey {
    use crate::engine_keyed_fixed::KeyedFixedPurpose as P;
    let payload = match value {
        P::Verified(value) => vec![word(0u16), source_evidence(value)],
        P::Rejected { evidence, defect } => {
            vec![word(1u16), source_evidence(evidence), word(*defect as u16)]
        }
        P::ClauseAccepted => vec![word(2u16)],
        P::ClauseRejected => vec![word(3u16)],
    };
    PurposeKey::new(SemanticProfileId::KeyedFixedForm, payload)
}

fn source_evidence(value: &crate::engine_source::Evidence) -> Part {
    use crate::engine_source::Evidence as E;
    let operation = |value: &crate::engine_records::OperationKey| {
        sequence([word(value.namespace), word(value.code)])
    };
    let object = |value: &crate::engine_lifecycle::ObjectKey| {
        sequence([word(value.namespace), word(value.code)])
    };
    sequence(match value {
        E::Operator { operation: op } => vec![word(0u16), operation(op)],
        E::ConfidentialValue {
            operation: op,
            asset,
        } => vec![word(1u16), operation(op), word(*asset)],
        E::SubstrateValue {
            operation: op,
            asset,
        } => vec![word(2u16), operation(op), word(*asset)],
        E::HandleClassification {
            operation: op,
            object: obj,
            side,
            minimum,
            maximum,
        } => vec![
            word(3u16),
            operation(op),
            object(obj),
            word(*side),
            word(*minimum),
            sequence(
                maximum
                    .as_ref()
                    .map_or_else(|| vec![word(0u16)], |value| vec![word(1u16), word(*value)]),
            ),
        ],
        E::SourceClassification {
            operation: op,
            object: obj,
        } => vec![word(4u16), operation(op), object(obj)],
        E::MembershipReading {
            operation: op,
            object: obj,
            recurrence,
            inclusion,
            retrieval,
            finality_for_final_reading,
        } => vec![
            word(5u16),
            operation(op),
            object(obj),
            word(*recurrence as u16),
            word(*inclusion),
            word(*retrieval),
            word(*finality_for_final_reading),
        ],
    })
}

#[cfg(test)]
fn oracle_lower_requirements(
    input: &crate::input::BoundCompilerInput<KeyedFixedOwner>,
) -> Result<Vec<crate::engine_requirement::Table>, KeyedFixedLoweringError> {
    crate::engine_requirement::proof_tables(&oracle_lower_proofs(input)?)
        .map_err(KeyedFixedLoweringError::ProofBundle)
}
#[cfg(test)]
fn oracle_lower_predicates(
    input: &crate::input::BoundCompilerInput<KeyedFixedOwner>,
) -> Result<crate::engine_foundation::Records, KeyedFixedLoweringError> {
    let span = tracing::debug_span!(target: "compiler::engine_keyed_fixed", "lower_predicates", scope_count = crate::instrumentation::count(input.owner().scope().len()));
    let _entered = span.enter();
    Ok(input.owner().shells()?.predicates)
}
#[cfg(test)]
fn oracle_lower_proofs(
    input: &crate::input::BoundCompilerInput<KeyedFixedOwner>,
) -> Result<crate::engine_proof::Records, KeyedFixedLoweringError> {
    input.owner().shells()?;
    Ok(proof_records(input.owner().lower_sources()?))
}
#[cfg(test)]
fn oracle_lower_cases(
    input: &crate::input::BoundCompilerInput<KeyedFixedOwner>,
) -> Result<crate::engine_cases::Tables, KeyedFixedLoweringError> {
    input.owner().shells()?;
    case_records(input.owner(), &lower_sources(input.owner()))
}
#[cfg(test)]
fn oracle_lower_constructibility(
    owner: &KeyedFixedOwner,
) -> Result<crate::engine_constructibility::Records, KeyedFixedLoweringError> {
    Ok(owner.shells()?.construction())
}

fn proof_records(source: crate::engine_source::SourceRecords) -> crate::engine_proof::Records {
    use crate::{engine_proof as core, engine_source::Payload};
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
    core::Records {
        obligations,
        choices: Vec::new(),
        lifecycle: Vec::new(),
        disclosure: core::DisclosureRecords {
            valid: true,
            retained: std::collections::BTreeSet::new(),
            public_effects: std::collections::BTreeMap::new(),
        },
    }
}

fn case_records(
    owner: &KeyedFixedOwner,
    sources: &crate::engine_source::SourceRecords,
) -> Result<crate::engine_cases::Tables, KeyedFixedLoweringError> {
    use crate::engine_cases as core;
    let placement = lower_placements(owner);
    let coverage = coverage_records(sources)
        .into_iter()
        .map(|plan| (plan.key.clone(), plan))
        .collect::<std::collections::BTreeMap<_, _>>();
    let rows = placement
        .plans
        .iter()
        .map(|(key, runtime)| {
            let covered = coverage
                .get(key)
                .ok_or_else(|| KeyedFixedLoweringError::MissingSourceTable(key.relation.clone()))?;
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
                coverage: Ok(covered.clone()),
                refusal: None,
                facts: Some(case_facts(key, *runtime, covered, sources)?),
                projections: covered.boundaries.clone(),
                collateral: covered
                    .negative
                    .iter()
                    .filter(|row| row.boundary == crate::coverage::CoverageBoundary::RuntimeCarrier)
                    .cloned()
                    .collect(),
            })
        })
        .collect::<Result<Vec<_>, KeyedFixedLoweringError>>()?;
    Ok(core::Tables {
        layout_scopes: std::collections::BTreeMap::new(),
        choices: Vec::new(),
        relations: placement
            .plans
            .into_iter()
            .map(|(key, _)| key.relation)
            .collect(),
        cases: owner
            .scope()
            .iter()
            .map(|operation| {
                let operation = operation_key(*operation);
                core::Case {
                    id: crate::engine_keyed_fixed::keyed_fixed_case(operation),
                    operation,
                    representations: std::collections::BTreeMap::new(),
                }
            })
            .collect(),
        rows,
    })
}

fn coverage_records(
    source: &crate::engine_source::SourceRecords,
) -> Vec<crate::engine_coverage::Plan> {
    use crate::{coverage::CoverageBoundary as B, engine_coverage as core, engine_source::Payload};
    let span = tracing::debug_span!(target: "compiler::engine_keyed_fixed", "lower_coverage", relation_count = crate::instrumentation::count(source.tables.len()));
    let _entered = span.enter();
    source
        .tables
        .clone()
        .into_iter()
        .map(|table| {
            let key = crate::engine_placement::Key {
                case: crate::engine_keyed_fixed::keyed_fixed_case(table.relation.operation),
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
                purpose: core::Purpose::Owner(crate::engine_keyed_fixed::keyed_fixed_purpose(
                    &purpose,
                )),
                inactive: false,
            };
            let subjects = if evidence.is_empty() {
                vec![core::Subject {
                    positive: requirement(KeyedFixedPurpose::ClauseAccepted),
                    negative: vec![requirement(KeyedFixedPurpose::ClauseRejected)],
                }]
            } else {
                evidence
                    .iter()
                    .map(|evidence| core::Subject {
                        positive: requirement(KeyedFixedPurpose::Verified(evidence.clone())),
                        negative: [
                            EvidenceDefect::Missing,
                            EvidenceDefect::Failed,
                            EvidenceDefect::IdentityMismatch,
                        ]
                        .into_iter()
                        .map(|defect| {
                            requirement(KeyedFixedPurpose::Rejected {
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
