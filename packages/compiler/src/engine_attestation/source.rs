//! Attestation-owned finite source-table lowering.
use crate::{
    CompileError,
    capability::RequiredCapability,
    engine_lifecycle::{ModeKey, ObjectKey},
    engine_source as core,
    source::{
        OperandId, OperandRole, RequiredSourceKind, RequirementActivation, SourceRequirement,
        is_sponsor_amount_operand,
    },
};
use realization::{AvailabilityClass, ProofKind, Relation, RelationDeclaration};
use std::collections::{BTreeMap, BTreeSet};
/// Exhaustive relation-to-operand derivation.
///
/// A new realization relation variant makes this match non-exhaustive
/// at compile time until its source derivation is added.
pub fn relation_operands(
    declaration: &RelationDeclaration,
) -> Result<Vec<OperandId>, CompileError> {
    let id = &declaration.id;
    let operand = |role: OperandRole| OperandId::new(id.clone(), role);

    let mut operands = match &declaration.relation {
        Relation::Cardinality {
            side,
            object,
            maximum,
            ..
        } => {
            let mut operands = vec![operand(OperandRole::ObjectFamilyCount {
                side: observed_to_transaction(*side),
                object: *object,
            })];

            if let realization::CardinalityMaximum::Bound(bound) = maximum {
                operands.push(operand(OperandRole::RuntimeBound { bound: *bound }));
            }

            operands
        }

        Relation::AllowedObjectFamilies { side, allowed } => allowed
            .iter()
            .map(|object| {
                operand(OperandRole::ObjectFamilyMembers {
                    side: observed_to_transaction(*side),
                    object: *object,
                })
            })
            .collect(),

        Relation::Recognition { side, object, .. } => {
            vec![operand(OperandRole::ObjectFamilyMembers {
                side: observed_to_transaction(*side),
                object: *object,
            })]
        }

        Relation::AmountConservation {
            input_objects,
            output_objects,
            ..
        } => family_amount_operands(id, realization::TransactionSide::Input, input_objects)
            .into_iter()
            .chain(family_amount_operands(
                id,
                realization::TransactionSide::Output,
                output_objects,
            ))
            .collect(),

        Relation::OwnerAuthorization { object } => vec![
            operand(OperandRole::ObjectFamilyOwners { object: *object }),
            operand(OperandRole::ProtocolSignerSet),
        ],

        Relation::PermissionlessAuthorization => {
            vec![operand(OperandRole::ProtocolSignerSet)]
        }

        Relation::SponsorIsolation => vec![
            operand(OperandRole::SponsorRegion),
            operand(OperandRole::SponsorSignerSet),
        ],

        Relation::SponsorEnvelopeMultiplicity { .. } | Relation::OpenFlowPolicy { .. } => {
            vec![operand(OperandRole::OpenFlowSet)]
        }

        Relation::RootPolicy { .. } => vec![operand(OperandRole::RootEffects)],

        Relation::ProjectionPolicy { .. } => vec![operand(OperandRole::ProjectionSet)],

        Relation::CanonicalDeltaPolicy { .. } => {
            vec![operand(OperandRole::CanonicalPartition)]
        }

        Relation::OperatorAuthorization
        | Relation::Constructibility {
            class: realization::ConstructibilityClass::Operator,
        } => vec![operand(OperandRole::ExternalEvidence {
            requirement: realization::ExternalEvidenceRequirement::OperatorAuthorization {
                operation: id.operation(),
            },
        })],

        Relation::Constructibility { .. } => vec![operand(OperandRole::ConstructibilityCase)],

        Relation::Representation { object, .. } => {
            vec![operand(OperandRole::Representation { object: *object })]
        }

        Relation::LifecycleExit { .. } => vec![operand(OperandRole::LifecycleGraph)],

        Relation::ExpressionPredicate { expression } => {
            vec![operand(OperandRole::ExpressionResult {
                expression: expression.clone(),
            })]
        }

        Relation::SubstrateConservation { asset } => {
            vec![operand(OperandRole::ExternalEvidence {
                requirement: realization::ExternalEvidenceRequirement::SubstrateConservation {
                    operation: id.operation(),
                    asset: *asset,
                },
            })]
        }
    };

    for operand in &operands {
        if is_sponsor_amount_operand(operand.role()) {
            return Err(CompileError::SponsorValueRead);
        }
    }

    operands.sort();
    Ok(operands)
}

fn family_amount_operands(
    id: &realization::RelationId,
    side: realization::TransactionSide,
    objects: &BTreeSet<architecture::ObjectId>,
) -> Vec<OperandId> {
    objects
        .iter()
        .map(|object| {
            OperandId::new(
                id.clone(),
                OperandRole::ObjectFamilyAmount {
                    side,
                    object: *object,
                },
            )
        })
        .collect()
}

/// Abstract capabilities one (relation, proof) pairing requires.
pub fn proof_capabilities(
    declaration: &RelationDeclaration,
    proof: ProofKind,
) -> BTreeSet<RequiredCapability> {
    use RequiredCapability as Cap;

    match proof {
        ProofKind::ManifestShape => {
            let mut capabilities = BTreeSet::from([Cap::AuthenticatedObjectRecognition]);

            match &declaration.relation {
                Relation::OperatorAuthorization
                | Relation::Constructibility {
                    class: realization::ConstructibilityClass::Operator,
                } => {
                    capabilities.insert(Cap::OperatorAuthorization);
                }
                Relation::Cardinality { .. } => {
                    capabilities.insert(Cap::AuthenticatedFamilyCardinality);
                }
                Relation::RootPolicy { .. } => {
                    capabilities.insert(Cap::AuthenticatedRootEffects);
                }
                Relation::ProjectionPolicy { .. } => {
                    capabilities.insert(Cap::AuthenticatedProjectionSet);
                }
                Relation::CanonicalDeltaPolicy { .. } => {
                    capabilities.insert(Cap::AuthenticatedCanonicalPartition);
                }
                Relation::OpenFlowPolicy { .. }
                | Relation::SponsorEnvelopeMultiplicity { .. }
                | Relation::SponsorIsolation => {
                    capabilities.insert(Cap::AuthenticatedOpenFlowPartition);
                }
                _ => {}
            }

            capabilities
        }

        ProofKind::PublicArithmetic => BTreeSet::from([
            Cap::ExactPublicAmountArithmetic,
            Cap::AuthenticatedObjectRecognition,
        ]),

        ProofKind::ConfidentialConservation => BTreeSet::from([
            Cap::ConfidentialValueConservation,
            Cap::AuthenticatedObjectRecognition,
        ]),

        ProofKind::SignerMembership => BTreeSet::from([Cap::OwnerAuthorization]),

        ProofKind::PublicConstructibility => BTreeSet::from([Cap::PublicConstructibility]),

        ProofKind::SubstrateConservation => {
            BTreeSet::from([Cap::WholeTransactionValueConservation])
        }
    }
}

/// External evidence one *selected* (relation, proof) pairing leaves
/// open.
///
/// # Why a selected alternative can still leave evidence open
///
/// Selecting a proof alternative settles which class of argument
/// discharges a relation, not that the argument completes here. The
/// confidential conservation of a protocol asset is the case where the
/// two come apart: the alternative is genuinely selected — the plan
/// commits to holding the amounts as commitments rather than in the
/// clear — and the consequence of that commitment is that no program
/// this compiler emits ever reads them, so the value equation is the
/// target's own confidential-transaction rules to establish and
/// nobody else's. A capability alone would understate it: a capability
/// says a target must be *able* to do something, where this says a
/// specific run must be *observed* doing it.
///
/// # Why the substrate pairing is absent
///
/// [`ProofKind::SubstrateConservation`] is not listed as producing its
/// requirement here. No candidate ever selects it:
/// [`classify_obligations`] classifies its relation as externally
/// evidenced before any selection is made, and produces the requirement
/// there. Producing it in both places would let the two disagree about
/// one fact.
///
/// # What keeps a pairing honest
///
/// The realization's approved-alternative census does. This function
/// answers for the pairings the realization admits; a confidential
/// alternative approved for a relation that conserves nothing would be
/// a realization defect, and the selection check that rejects an
/// unapproved proof is where it is caught.
///
/// [`classify_obligations`]: crate::proof::classify_obligations
#[must_use]
pub fn proof_external_evidence(
    declaration: &RelationDeclaration,
    proof: ProofKind,
) -> BTreeSet<realization::ExternalEvidenceRequirement> {
    match (&declaration.relation, proof) {
        (Relation::AmountConservation { asset, .. }, ProofKind::ConfidentialConservation) => {
            BTreeSet::from([
                realization::ExternalEvidenceRequirement::ConfidentialValueConservation {
                    operation: declaration.id.operation(),
                    asset: *asset,
                },
            ])
        }
        _ => BTreeSet::new(),
    }
}

/// One source-requirement row for every operand of one (relation,
/// proof) pairing.
///
/// # Errors
///
/// [`CompileError::SponsorValueRead`] on any sponsor amount operand;
/// [`CompileError::MissingSourceRequirement`] when an operand derives
/// no source row (fail closed rather than silently dropping it).
fn derive_rows(
    declaration: &RelationDeclaration,
    proof: ProofKind,
) -> Result<Vec<SourceRequirement>, CompileError> {
    let operands = relation_operands(declaration)?;
    let mut rows = Vec::with_capacity(operands.len());

    for operand in operands {
        let (source, availability, activation) = operand_source(declaration, proof, &operand);

        rows.push(SourceRequirement {
            operand,
            source,
            availability,
            activation,
        });
    }

    rows.sort();
    Ok(rows)
}

/// Source classification for one operand under one proof kind.
fn operand_source(
    declaration: &RelationDeclaration,
    proof: ProofKind,
    operand: &OperandId,
) -> (RequiredSourceKind, AvailabilityClass, RequirementActivation) {
    use RequiredSourceKind as Kind;

    // Sponsor-conditional because the family *is* the sponsor region
    // here, not because of the family itself: the scope gate refuses
    // every operation whose protocol flows claim ordinary L-BTC, and a
    // mandatory owner-funded input in such an operation would not be
    // conditional on a sponsor region existing.
    let sponsor_family = |object: architecture::ObjectId| {
        if object == crate::sponsor_region::ORDINARY_LBTC {
            RequirementActivation::WhenSponsorPresent
        } else {
            RequirementActivation::Always
        }
    };

    match operand.role() {
        OperandRole::ObjectFamilyMembers { side, object } => {
            let kind = match side {
                realization::TransactionSide::Input => Kind::AuthenticatedInputObject,
                realization::TransactionSide::Output => Kind::AuthenticatedOutputObject,
            };

            (kind, AvailabilityClass::Public, sponsor_family(*object))
        }

        OperandRole::ObjectFamilyCount { object, .. } | OperandRole::Representation { object } => (
            Kind::AuthenticatedFamilyCensus,
            AvailabilityClass::Public,
            sponsor_family(*object),
        ),

        // Sponsor amounts were rejected during operand derivation, so
        // every amount here is a protocol amount.
        OperandRole::ObjectFamilyAmount { .. } => {
            let kind = match proof {
                ProofKind::ConfidentialConservation => Kind::AuthenticatedCommitmentRelation,
                _ => Kind::AuthenticatedConsensusValue,
            };

            (
                kind,
                AvailabilityClass::Public,
                RequirementActivation::Always,
            )
        }

        OperandRole::ObjectFamilyOwners { .. } => (
            Kind::AuthenticatedFamilyCensus,
            AvailabilityClass::Public,
            RequirementActivation::Always,
        ),

        OperandRole::ProtocolSignerSet => match &declaration.relation {
            Relation::OwnerAuthorization { object } => (
                Kind::InputOwnerWitness,
                AvailabilityClass::InputOwners { object: *object },
                RequirementActivation::Always,
            ),
            _ => (
                Kind::PublicConstructionData,
                AvailabilityClass::Public,
                RequirementActivation::Always,
            ),
        },

        OperandRole::SponsorSignerSet => (
            Kind::SponsorLocalWitness,
            AvailabilityClass::SponsorLocal,
            RequirementActivation::WhenSponsorPresent,
        ),

        OperandRole::SponsorRegion => (
            Kind::AuthenticatedFamilyCensus,
            AvailabilityClass::Public,
            RequirementActivation::WhenSponsorPresent,
        ),

        OperandRole::CanonicalPartition
        | OperandRole::OpenFlowSet
        | OperandRole::RootEffects
        | OperandRole::ProjectionSet => (
            Kind::AuthenticatedTransitionCertificate,
            AvailabilityClass::Public,
            RequirementActivation::Always,
        ),

        OperandRole::RuntimeBound { .. } => (
            Kind::RuntimeArchitectureBound,
            AvailabilityClass::Public,
            RequirementActivation::Always,
        ),

        OperandRole::ConstructibilityCase | OperandRole::LifecycleGraph => (
            Kind::PublicConstructionData,
            AvailabilityClass::Public,
            RequirementActivation::Always,
        ),

        OperandRole::ExpressionResult { .. } => (
            Kind::DerivedExpression,
            AvailabilityClass::Public,
            RequirementActivation::Always,
        ),

        OperandRole::ExternalEvidence { .. } => (
            Kind::ExternalEvidence,
            AvailabilityClass::Public,
            RequirementActivation::Always,
        ),
    }
}

const fn observed_to_transaction(side: realization::ObservedSide) -> realization::TransactionSide {
    match side {
        realization::ObservedSide::Input => realization::TransactionSide::Input,
        realization::ObservedSide::Output => realization::TransactionSide::Output,
    }
}

pub fn derive_source_requirements(
    declaration: &RelationDeclaration,
    proof: ProofKind,
) -> Result<Vec<SourceRequirement>, CompileError> {
    let rows = derive_rows(declaration, proof)?;
    let identities = rows
        .iter()
        .map(|row| (operand_key(&row.operand), row.clone()))
        .collect::<BTreeMap<_, _>>();
    let records = core::SourceRecords {
        tables: vec![table(declaration, proof, &rows)],
        regions: Vec::new(),
    };
    let analyzed = core::analyze_sources(&records).map_err(|_| CompileError::InvalidRealization)?;
    let mut result = analyzed
        .into_iter()
        .flatten()
        .map(|row| identities[&row.operand].clone())
        .collect::<Vec<_>>();
    result.sort();
    Ok(result)
}

pub fn lower(owner: &super::AttestationOwner) -> Result<core::SourceRecords, CompileError> {
    let span = tracing::debug_span!(target: "compiler::engine_attestation", "lower_sources", scope_count = crate::instrumentation::count(owner.scope().operations().len()));
    let _entered = span.enter();
    let declarations = owner
        .source()
        .relations()
        .filter(|row| owner.scope().operations().contains(&row.id.operation()))
        .collect::<Vec<_>>();
    let mut tables = Vec::new();
    for declaration in &declarations {
        for alternative in &declaration.proof_alternatives {
            let rows = derive_rows(declaration, alternative.proof())?;
            tables.push(table(declaration, alternative.proof(), &rows));
        }
    }
    let regions = owner
        .scope()
        .operations()
        .iter()
        .map(|operation| {
            let flows = declarations
                .iter()
                .filter(|row| row.id.operation() == *operation)
                .filter_map(|row| match &row.relation {
                    Relation::OpenFlowPolicy { allowed } => Some(allowed.iter().copied()),
                    _ => None,
                })
                .flatten()
                .collect::<BTreeSet<_>>();
            let role = crate::sponsor_region::ordinary_lbtc_role_of(&flows);
            let family = object_key(crate::sponsor_region::ORDINARY_LBTC);
            core::RegionRecord {
                operation: super::operation_key(*operation),
                declared_families: declarations
                    .iter()
                    .filter(|row| row.id.operation() == *operation)
                    .filter_map(|row| match row.relation {
                        Relation::Cardinality { object, .. } => Some(object_key(object)),
                        _ => None,
                    })
                    .collect(),
                declared_flows: flows.iter().map(|flow| flow.code()).collect(),
                funding_family: if flows.is_empty() { None } else { Some(family) },
                erased_amount_family: role.is_sponsor_region().then_some(family),
                sponsor_conditional_family: role.is_sponsor_region().then_some(family),
                decidable: role != crate::sponsor_region::OrdinaryLbtcRole::ProtocolClaimed,
            }
        })
        .collect();
    Ok(core::SourceRecords { tables, regions })
}

pub fn table(
    declaration: &RelationDeclaration,
    proof: ProofKind,
    rows: &[SourceRequirement],
) -> core::SourceTable {
    core::SourceTable {
        relation: super::relation_key(&declaration.id),
        proof_namespace: super::ATTESTATION,
        proof_kind: proof as u16,
        expected: rows.iter().map(|row| operand_key(&row.operand)).collect(),
        rows: rows.iter().map(row_key).collect(),
    }
}
pub fn row_key(row: &SourceRequirement) -> core::SourceRow {
    core::SourceRow {
        operand: operand_key(&row.operand),
        source: row.source,
        availability: super::constructibility::availability_key(row.availability),
        activation: match row.activation {
            RequirementActivation::Always => core::Activation::Always,
            RequirementActivation::WhenSponsorPresent => core::Activation::SponsorPresent,
            RequirementActivation::WhenRepresentation { object, mode } => {
                core::Activation::Representation {
                    object: object_key(object),
                    mode: ModeKey {
                        namespace: super::ATTESTATION,
                        code: mode as u16,
                    },
                }
            }
        },
    }
}

const fn object_key(object: architecture::ObjectId) -> ObjectKey {
    ObjectKey {
        namespace: super::ATTESTATION,
        code: object.code(),
    }
}
pub fn operand_key(operand: &OperandId) -> core::OperandKey {
    use core::Payload as P;
    let (role, payload) = match operand.role() {
        OperandRole::ObjectFamilyMembers { side, object } => (
            0,
            P::Family {
                side: super::side_code(*side),
                object: object_key(*object),
            },
        ),
        OperandRole::ObjectFamilyCount { side, object } => (
            1,
            P::Family {
                side: super::side_code(*side),
                object: object_key(*object),
            },
        ),
        OperandRole::ObjectFamilyAmount { side, object } => (
            2,
            P::Family {
                side: super::side_code(*side),
                object: object_key(*object),
            },
        ),
        OperandRole::ObjectFamilyOwners { object } => (3, P::Object(object_key(*object))),
        OperandRole::ProtocolSignerSet => (4, P::None),
        OperandRole::SponsorSignerSet => (5, P::None),
        OperandRole::SponsorRegion => (6, P::None),
        OperandRole::CanonicalPartition => (7, P::None),
        OperandRole::OpenFlowSet => (8, P::None),
        OperandRole::RootEffects => (9, P::None),
        OperandRole::ProjectionSet => (10, P::None),
        OperandRole::RuntimeBound { bound } => (
            11,
            P::Bound {
                namespace: super::ATTESTATION,
                code: bound.code(),
            },
        ),
        OperandRole::Representation { object } => (12, P::Object(object_key(*object))),
        OperandRole::ConstructibilityCase => (13, P::None),
        OperandRole::LifecycleGraph => (14, P::None),
        OperandRole::ExpressionResult { expression } => (
            15,
            match expression {
                realization::ExprId::Fact(fact) => P::Fact(crate::engine_proof::FactKey {
                    namespace: super::ATTESTATION,
                    subject: super::constructibility::fact_key(fact),
                }),
                realization::ExprId::Relation { relation, role } => P::Expression {
                    relation: super::relation_key(relation),
                    role: *role as u16,
                },
            },
        ),
        OperandRole::ExternalEvidence { requirement } => (
            16,
            P::Evidence(match requirement {
                realization::ExternalEvidenceRequirement::OperatorAuthorization { operation } => {
                    core::Evidence::Operator {
                        operation: super::operation_key(*operation),
                    }
                }
                realization::ExternalEvidenceRequirement::ConfidentialValueConservation {
                    operation,
                    asset,
                } => core::Evidence::ConfidentialValue {
                    operation: super::operation_key(*operation),
                    asset: asset.code(),
                },
                realization::ExternalEvidenceRequirement::SubstrateConservation {
                    operation,
                    asset,
                } => core::Evidence::SubstrateValue {
                    operation: super::operation_key(*operation),
                    asset: asset.code(),
                },
            }),
        ),
    };
    core::OperandKey {
        relation: super::relation_key(operand.relation()),
        role,
        payload,
    }
}
