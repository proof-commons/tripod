//! Lossless attestation view of shared aggregate closure.
use crate::{
    CompileError, engine_requirement as core, proof::ProofPlanCandidate,
    requirement::RelationRequirements,
};
use architecture::ObjectId;
use realization::Relation;
use std::collections::{BTreeMap, BTreeSet};

type Choice = Option<crate::engine_proof::AlternativeKey>;
type Registry = BTreeMap<
    (crate::engine_records::RelationKey, Choice),
    Result<RelationRequirements, CompileError>,
>;

pub fn tables(input: &crate::input::BoundCompilerInput) -> Result<Vec<core::Table>, CompileError> {
    let relations = crate::relation::build_relation_analysis(input)?;
    let constructibility = crate::constructibility::build_constructibility_analysis(input)?;
    let declarations = relations
        .graph
        .node_weights()
        .map(|node| (node.source.id.clone(), &node.source))
        .collect();
    Ok(lower_tables(&relations, &constructibility, &declarations)?.0)
}

pub fn derive(
    relations: &crate::relation::CompilerRelationAnalysis,
    constructibility: &crate::constructibility::CompilerConstructibilityAnalysis,
    candidate: &ProofPlanCandidate,
) -> Result<BTreeMap<realization::RelationId, RelationRequirements>, CompileError> {
    let declarations = relations
        .graph
        .node_weights()
        .map(|node| (node.source.id.clone(), &node.source))
        .collect::<BTreeMap<_, _>>();
    let identities = declarations
        .keys()
        .map(|id| (super::relation_key(id), id.clone()))
        .collect::<BTreeMap<_, _>>();
    let (tables, registry) = lower_tables(relations, constructibility, &declarations)?;
    let selected = candidate
        .proofs
        .iter()
        .map(|(id, proof)| (super::relation_key(id), proof_key(proof)))
        .collect::<BTreeMap<_, _>>();
    let derived = core::derive(&tables, &selected).map_err(|error| match error {
        core::SelectionError::Missing(key) => CompileError::MissingSelectedProof {
            relation: identities[&key].clone(),
        },
        core::SelectionError::Unexpected(key) => CompileError::UnexpectedSelectedProof {
            relation: identities[&key].clone(),
        },
        core::SelectionError::Unapproved(key) => CompileError::UnapprovedSelectedProof {
            relation: identities[&key].clone(),
        },
        core::SelectionError::Duplicate(key) => CompileError::DuplicateRelation {
            relation: identities[&key].clone(),
        },
        core::SelectionError::OwnerRefusal(error) => *error,
    })?;
    derived
        .into_keys()
        .map(|key| {
            let choice = selected.get(&key).cloned();
            let mut value = registry[&(key.clone(), choice)].clone()?;
            let declaration = declarations[&identities[&key]];
            value.representation = crate::requirement::owned_representation(declaration, candidate);
            value.lifecycle = crate::requirement::owned_lifecycle(declaration, candidate);
            Ok((value.relation.clone(), value))
        })
        .collect()
}

fn proof_key(proof: &realization::ProofAlternativeId) -> crate::engine_proof::AlternativeKey {
    crate::engine_proof::AlternativeKey {
        relation: super::relation_key(proof.relation()),
        namespace: super::ATTESTATION,
        kind: proof.proof() as u16,
    }
}

fn lower_tables(
    relations: &crate::relation::CompilerRelationAnalysis,
    constructibility: &crate::constructibility::CompilerConstructibilityAnalysis,
    declarations: &BTreeMap<realization::RelationId, &realization::RelationDeclaration>,
) -> Result<(Vec<core::Table>, Registry), CompileError> {
    use crate::{proof::RelationObligationClass as Class, requirement::ProofDisposition};
    let span = tracing::debug_span!(target: "compiler::engine_attestation", "lower_requirement_tables", relation_count = crate::instrumentation::count(declarations.len()));
    let _entered = span.enter();
    let mut tables = Vec::new();
    let mut registry = BTreeMap::new();
    for obligation in crate::proof::classify_obligations(relations)? {
        let declaration = declarations[&obligation.relation];
        let relation = super::relation_key(&obligation.relation);
        let required = matches!(obligation.class, Class::ProofRequired { .. });
        let rows = match obligation.class {
            Class::StaticallyValidated => vec![(
                None,
                Ok(RelationRequirements {
                    relation: obligation.relation,
                    proof: ProofDisposition::StaticallyValidated,
                    required_capabilities: BTreeSet::new(),
                    source_requirements: BTreeSet::new(),
                    external_evidence: BTreeSet::new(),
                    representation: None,
                    lifecycle: BTreeSet::new(),
                }),
            )],
            Class::ProofRequired { alternatives } => alternatives
                .into_iter()
                .map(|proof| {
                    let key = proof_key(&proof);
                    let value =
                        super::source::derive_source_requirements(declaration, proof.proof()).map(
                            |sources| RelationRequirements {
                                relation: declaration.id.clone(),
                                proof: ProofDisposition::Selected {
                                    proof: proof.clone(),
                                },
                                required_capabilities: super::source::proof_capabilities(
                                    declaration,
                                    proof.proof(),
                                ),
                                source_requirements: sources.into_iter().collect(),
                                external_evidence: super::source::proof_external_evidence(
                                    declaration,
                                    proof.proof(),
                                ),
                                representation: None,
                                lifecycle: BTreeSet::new(),
                            },
                        );
                    (Some(key), value)
                })
                .collect(),
            Class::ExternalEvidence {
                requirement,
                proof,
                required_capabilities,
                source_requirements,
            } => vec![(
                None,
                Ok(RelationRequirements {
                    relation: declaration.id.clone(),
                    proof: ProofDisposition::ExternalEvidence {
                        approved_proof: proof,
                        requirement: requirement.clone(),
                    },
                    required_capabilities,
                    source_requirements: source_requirements.into_iter().collect(),
                    external_evidence: BTreeSet::from([requirement]),
                    representation: None,
                    lifecycle: BTreeSet::new(),
                }),
            )],
        };
        let mut choices = BTreeMap::new();
        for (choice, value) in rows {
            let value = value.and_then(|value| checked(value, constructibility));
            choices.insert(
                choice.clone(),
                value
                    .as_ref()
                    .map(|value| {
                        bundle(
                            &value.required_capabilities,
                            value.source_requirements.iter(),
                            &value.external_evidence,
                        )
                    })
                    .map_err(Clone::clone),
            );
            registry.insert((relation.clone(), choice), value);
        }
        tables.push(core::Table {
            relation,
            required,
            choices,
        });
    }
    Ok((tables, registry))
}

fn checked(
    value: RelationRequirements,
    constructibility: &crate::constructibility::CompilerConstructibilityAnalysis,
) -> Result<RelationRequirements, CompileError> {
    crate::constructibility::validate_source_constructibility(
        constructibility,
        &value.relation,
        &value
            .source_requirements
            .iter()
            .cloned()
            .collect::<Vec<_>>(),
    )?;
    crate::requirement::validate_sponsor_erasure(&value)?;
    Ok(value)
}

pub fn validate_aggregate(
    candidate: &ProofPlanCandidate,
    requirements: &BTreeMap<realization::RelationId, RelationRequirements>,
) -> Result<(), CompileError> {
    let relations = requirements
        .keys()
        .chain(requirements.values().flat_map(|bundle| {
            bundle
                .source_requirements
                .iter()
                .map(|row| row.operand.relation())
        }))
        .map(|id| (super::relation_key(id), id))
        .collect::<BTreeMap<_, _>>();
    let sources = candidate
        .source_requirements
        .iter()
        .chain(
            requirements
                .values()
                .flat_map(|bundle| &bundle.source_requirements),
        )
        .map(|row| (super::source::row_key(row), &row.operand))
        .collect::<BTreeMap<_, _>>();
    let evidence = candidate
        .external_evidence
        .iter()
        .chain(
            requirements
                .values()
                .flat_map(|bundle| &bundle.external_evidence),
        )
        .map(|value| (super::proof::evidence_key(value), value))
        .collect::<BTreeMap<_, _>>();
    let owned = requirements
        .iter()
        .map(|(id, value)| {
            (
                super::relation_key(id),
                bundle(
                    &value.required_capabilities,
                    value.source_requirements.iter(),
                    &value.external_evidence,
                ),
            )
        })
        .collect();
    let aggregate = bundle(
        &candidate.required_capabilities,
        candidate.source_requirements.iter(),
        &candidate.external_evidence,
    );
    core::validate_aggregate(&aggregate, &owned).map_err(|error| match error {
        core::Error::Capability {
            relation,
            capability,
        } => CompileError::AnalyzedCapabilityClosureMismatch {
            relation: relation.map(|key| relations[&key].clone()),
            capability,
        },
        core::Error::Source { relation, row } => CompileError::AnalyzedSourceClosureMismatch {
            relation: relation.map(|key| relations[&key].clone()),
            operand: sources[&*row].clone(),
        },
        core::Error::Evidence {
            relation,
            evidence: value,
        } => CompileError::AnalyzedEvidenceClosureMismatch {
            relation: relation.map(|key| relations[&key].clone()),
            requirement: evidence[&value].clone(),
        },
    })
}
fn bundle<'a>(
    capabilities: &BTreeSet<crate::capability::RequiredCapability>,
    sources: impl Iterator<Item = &'a crate::source::SourceRequirement>,
    evidence: &BTreeSet<realization::ExternalEvidenceRequirement>,
) -> core::Bundle {
    core::Bundle {
        capabilities: capabilities.clone(),
        sources: sources.map(super::source::row_key).collect(),
        evidence: evidence.iter().map(super::proof::evidence_key).collect(),
    }
}

pub fn relation_objects(relation: &Relation) -> BTreeSet<ObjectId> {
    match relation {
        Relation::Cardinality { object, .. }
        | Relation::Recognition { object, .. }
        | Relation::OwnerAuthorization { object }
        | Relation::Representation { object, .. }
        | Relation::LifecycleExit { object, .. } => BTreeSet::from([*object]),

        Relation::AllowedObjectFamilies { allowed, .. } => allowed.clone(),

        Relation::AmountConservation {
            input_objects,
            output_objects,
            ..
        } => input_objects.union(output_objects).copied().collect(),

        Relation::PermissionlessAuthorization
        | Relation::SponsorIsolation
        | Relation::SponsorEnvelopeMultiplicity { .. }
        | Relation::RootPolicy { .. }
        | Relation::ProjectionPolicy { .. }
        | Relation::CanonicalDeltaPolicy { .. }
        | Relation::OpenFlowPolicy { .. }
        | Relation::Constructibility { .. }
        | Relation::ExpressionPredicate { .. }
        | Relation::OperatorAuthorization
        | Relation::SubstrateConservation { .. } => BTreeSet::new(),
    }
}

const fn object_key(object: ObjectId) -> crate::engine_lifecycle::ObjectKey {
    crate::engine_lifecycle::ObjectKey {
        namespace: super::ATTESTATION,
        code: object.code(),
    }
}
pub fn owned_representation(
    declaration: &realization::RelationDeclaration,
    candidate: &ProofPlanCandidate,
) -> Option<crate::requirement::RepresentationSelection> {
    let objects = crate::requirement::relation_objects(&declaration.relation)
        .into_iter()
        .map(|object| (object_key(object), object))
        .collect::<BTreeMap<_, _>>();
    let selected = candidate
        .representations
        .iter()
        .map(|(choice, mode)| {
            (
                super::proof::choice_key(choice),
                super::proof::mode_key(*mode),
            )
        })
        .collect();
    let (object, _) = core::single_representation(
        super::operation_key(declaration.id.operation()),
        &objects.keys().copied().collect(),
        &selected,
    )?;
    let object = objects[&object];
    let mode = candidate.representations[&crate::lifecycle::RepresentationChoiceId {
        operation: declaration.id.operation(),
        object,
    }];
    Some(crate::requirement::RepresentationSelection { object, mode })
}
pub fn owned_lifecycle(
    declaration: &realization::RelationDeclaration,
    candidate: &ProofPlanCandidate,
) -> BTreeSet<crate::lifecycle::LifecycleRequirement> {
    let association = match &declaration.relation {
        Relation::LifecycleExit { object, exit } => {
            Some((object_key(*object), super::operation_key(*exit)))
        }
        _ => None,
    };
    let rows = candidate
        .lifecycle
        .iter()
        .map(|row| (super::proof::lifecycle_key(row), row))
        .collect::<BTreeMap<_, _>>();
    core::owned_lifecycle(association, &rows.keys().cloned().collect::<Vec<_>>())
        .iter()
        .map(|key| rows[key].clone())
        .collect()
}
