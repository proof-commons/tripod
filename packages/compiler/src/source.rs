//! Typed relation operands and authenticatable source requirements
//! (Guide-3 Tranche C).
//!
//! A relation cannot be planned safely from its ID and proof kind
//! alone: the planner must know which facts each proof consumes and
//! how those facts are authenticated. This module derives, for every
//! relation and approved proof alternative, the exact operand census
//! and one source-requirement row per operand — typed, canonically
//! sorted, and free of any sponsor amount: sponsor erasure means the
//! value is absent, not merely secret, so an amount operand over the
//! sponsor family is a structural compile error.

// The analysis stages have no non-test consumer until the P2-012
// analyzed program; unit tests exercise them until then. Remove with
// the first real consumer.
#![allow(dead_code)]

use std::collections::BTreeSet;

use realization::{
    AvailabilityClass, ProofKind, Relation, RelationDeclaration, RepresentationMode,
};

use crate::{CompileError, capability::RequiredCapability};

/// Stable identity of one relation operand.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OperandId {
    relation: realization::RelationId,
    role: OperandRole,
}

impl OperandId {
    #[must_use]
    pub const fn new(relation: realization::RelationId, role: OperandRole) -> Self {
        Self { relation, role }
    }

    #[must_use]
    pub const fn relation(&self) -> &realization::RelationId {
        &self.relation
    }

    #[must_use]
    pub const fn role(&self) -> &OperandRole {
        &self.role
    }
}

/// Compiler-owned semantic operand role. Never a target position.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OperandRole {
    ObjectFamilyMembers {
        side: realization::TransactionSide,
        object: architecture::ObjectId,
    },
    ObjectFamilyCount {
        side: realization::TransactionSide,
        object: architecture::ObjectId,
    },
    ObjectFamilyAmount {
        side: realization::TransactionSide,
        object: architecture::ObjectId,
    },
    ObjectFamilyOwners {
        object: architecture::ObjectId,
    },

    ProtocolSignerSet,
    SponsorSignerSet,
    SponsorRegion,

    CanonicalPartition,
    OpenFlowSet,
    RootEffects,
    ProjectionSet,

    RuntimeBound {
        bound: architecture::BoundId,
    },

    Representation {
        object: architecture::ObjectId,
    },

    ConstructibilityCase,
    LifecycleGraph,

    ExpressionResult {
        expression: realization::ExprId,
    },

    ExternalEvidence {
        requirement: realization::ExternalEvidenceRequirement,
    },
}

/// How one operand's source must be authenticated. The source proof
/// obligation, not proof completion.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RequiredSourceKind {
    AuthenticatedInputObject,
    AuthenticatedOutputObject,
    AuthenticatedFamilyCensus,
    AuthenticatedConsensusValue,
    AuthenticatedCommitmentRelation,
    AuthenticatedTransitionCertificate,

    RuntimeArchitectureBound,
    DerivedExpression,

    InputOwnerWitness,
    OperatorWitness,
    RefundKeyWitness,
    SponsorLocalWitness,

    PublicConstructionData,
    ExternalEvidence,
}

/// When one source requirement is active.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RequirementActivation {
    Always,
    WhenSponsorPresent,
    WhenRepresentation(RepresentationMode),
}

/// One canonical source-requirement row.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceRequirement {
    pub operand: OperandId,
    pub source: RequiredSourceKind,
    pub availability: AvailabilityClass,
    pub activation: RequirementActivation,
}

/// True for the erased sponsor family's amount: structurally absent
/// from the compiler, never an allowable private fact.
pub const fn is_sponsor_amount_operand(role: &OperandRole) -> bool {
    matches!(
        role,
        OperandRole::ObjectFamilyAmount {
            object: architecture::ObjectId::PlainLbtc,
            ..
        }
    )
}

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
        } => {
            let inputs = input_objects.iter().map(|object| {
                operand(OperandRole::ObjectFamilyAmount {
                    side: realization::TransactionSide::Input,
                    object: *object,
                })
            });
            let outputs = output_objects.iter().map(|object| {
                operand(OperandRole::ObjectFamilyAmount {
                    side: realization::TransactionSide::Output,
                    object: *object,
                })
            });

            inputs.chain(outputs).collect()
        }

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

/// One source-requirement row for every operand of one (relation,
/// proof) pairing.
///
/// # Errors
///
/// [`CompileError::SponsorValueRead`] on any sponsor amount operand;
/// [`CompileError::MissingSourceRequirement`] when an operand derives
/// no source row (fail closed rather than silently dropping it).
pub fn derive_source_requirements(
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

    let sponsor_family = |object: architecture::ObjectId| {
        if object == architecture::ObjectId::PlainLbtc {
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
