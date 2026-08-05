//! Relation discharge classification per execution case (Guide-5
//! Tranche B).
//!
//! Discharge is not one category. A relation has an independent
//! discharge boundary, semantic scope, activation condition, and
//! carrier multiplicity, and a single relation may produce
//! requirements at more than one boundary at once — a representation
//! relation is both a compiler-static selection and a backend
//! structural encoding obligation. Collapsing those axes into one
//! local/global/conditional/duplicated enum would make an ordinary
//! hybrid relation unrepresentable, so each axis is stored separately
//! and every relation carries four independent requirement lists.
//!
//! This stage assigns no carrier. A runtime requirement states the
//! semantic scope, the multiplicity, and the source rows an eventual
//! carrier must receive; which abstract carrier roles are eligible,
//! and what layout must route those sources there, are later stages.
//! Nothing here names a transaction index, tapleaf, stack slot, or
//! witness position.

// The analysis stages have no non-test consumer until the P2-012
// analyzed program; unit tests exercise them until then. Remove with
// the first real consumer.
#![allow(dead_code)]

use std::collections::BTreeSet;

use architecture::{ObjectId, OperationId};
use realization::{
    ConstructibilityClass, ExternalEvidenceRequirement, Relation, RelationDeclaration, RelationId,
    RepresentationMode, TransactionSide,
};

use crate::{
    CompileError,
    case::{ExecutionCase, ExecutionCaseId, SponsorCase, is_sponsor_object},
    relation::CompilerRelationAnalysis,
    source::SourceRequirement,
};

/// Where one requirement of a relation is discharged.
///
/// Compiler-static does not mean target-verified, and backend-structural
/// does not mean already emitted: both are obligations recorded against
/// a future backend, not completed evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DischargeBoundary {
    CompilerStatic,
    BackendStructural,
    RuntimeCarrier,
    ExternalEvidence,
}

/// The semantic extent one relation constrains.
///
/// Independent of the discharge boundary: a member-local obligation and
/// a transaction-global one are both runtime-discharged, and the
/// difference is what a carrier must see, not whether one exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticScope {
    /// One authenticated member of one family.
    MemberLocal {
        side: TransactionSide,
        object: ObjectId,
    },
    /// One complete family census on one side.
    FamilyGlobal {
        side: TransactionSide,
        object: ObjectId,
    },
    /// Every family on one transaction side.
    TransactionSideGlobal { side: TransactionSide },
    /// One object family across both sides.
    ObjectFamilyGlobal { object: ObjectId },
    /// The whole operation.
    TransactionGlobal,
}

/// When one relation's obligation exists at all.
///
/// A case-independent property of the relation. The per-case resolution
/// of it is [`RelationActivity`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ActivationCondition {
    Always,
    WhenSponsorPresent,
    WhenRepresentation(RepresentationMode),
}

/// Whether one relation is active in one execution case.
///
/// Vacuous is an explicit disposition, not an omission: a sponsor-family
/// member check with no sponsor region still appears in the census, with
/// no runtime requirement attached.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelationActivity {
    Active,
    Vacuous,
}

/// How many carriers an active runtime relation needs.
///
/// Independent of scope: an every-member obligation is not a weaker
/// global one, and a single coordinator cannot silently stand in for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CarrierMultiplicity {
    ExactlyOne,
    EveryMember,
    AtLeastOne,
    DeliberateDuplication,
}

/// A property the compiler validates directly from typed input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompilerStaticRequirement {
    /// The selected mode is a member of the relation's allowed set.
    RepresentationSelection {
        object: ObjectId,
        allowed: BTreeSet<RepresentationMode>,
        selected: RepresentationMode,
    },
    /// The operation's constructibility class is validated in scope.
    ConstructibilityValidated { class: ConstructibilityClass },
    /// A required lifecycle exit is declared for the object.
    LifecycleExitDeclared { object: ObjectId, exit: OperationId },
}

/// An obligation on a future emitted bundle or ABI that is not an
/// ordinary runtime predicate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackendStructuralRequirement {
    /// The selected representation must be encoded and authenticated.
    EncodeAndAuthenticateRepresentation {
        object: ObjectId,
        representation: RepresentationMode,
    },
    /// A permissionless path must contain no owner or operator secret
    /// gate — a structural property of the path, not a predicate a
    /// carrier evaluates.
    SecretFreeOperationPath { operation: OperationId },
    /// A required lifecycle exit must remain represented downstream.
    RetainRequiredLifecycleExit { object: ObjectId, exit: OperationId },
}

/// What an eventual carrier must discharge for one active relation.
///
/// Carrier eligibility and carrier roles are deliberately absent: this
/// value states the obligation, and the carrier stage consumes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePlacementRequirement {
    pub relation: RelationId,
    pub scope: SemanticScope,
    pub multiplicity: CarrierMultiplicity,
    /// The active source rows this relation's carrier must receive.
    pub sources: Vec<SourceRequirement>,
}

/// One relation's complete discharge plan in one execution case.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationCasePlan {
    pub relation: RelationId,
    pub case: ExecutionCaseId,
    pub activation: ActivationCondition,
    pub activity: RelationActivity,
    /// Every boundary this relation discharges at, in this case.
    pub boundaries: BTreeSet<DischargeBoundary>,
    pub compiler_requirements: Vec<CompilerStaticRequirement>,
    pub structural_requirements: Vec<BackendStructuralRequirement>,
    pub runtime_requirements: Vec<RuntimePlacementRequirement>,
    pub external_evidence: BTreeSet<ExternalEvidenceRequirement>,
}

/// Stable key of one relation-case census entry.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RelationCaseKey {
    pub relation: RelationId,
    pub case: ExecutionCaseId,
}

/// The case-independent classification of one relation.
///
/// Separated from the per-case plan so the axes stay independent: the
/// boundaries, scope, multiplicity, and activation of a relation do not
/// change with the case; only activity and the active source set do.
#[derive(Clone, Debug, PartialEq, Eq)]
struct RelationDischarge {
    boundaries: BTreeSet<DischargeBoundary>,
    activation: ActivationCondition,
    runtime: Option<(SemanticScope, CarrierMultiplicity)>,
}

/// Classify one relation in one execution case.
///
/// The match over relation variants is exhaustive with no wildcard arm,
/// so a new realization relation fails to compile here until its
/// discharge is stated rather than being absorbed into a default.
///
/// # Errors
///
/// [`CompileError::MissingRepresentationChoice`] when a representation
/// relation's object has no mode fixed by the case.
pub fn classify_relation_case(
    declaration: &RelationDeclaration,
    case: &ExecutionCase,
) -> Result<RelationCasePlan, CompileError> {
    let operation = declaration.id.operation();
    let discharge = classify_discharge(&declaration.relation);
    let activity = resolve_activity(discharge.activation, &case.id);

    let mut compiler_requirements = Vec::new();
    let mut structural_requirements = Vec::new();
    let mut external_evidence = BTreeSet::new();

    match &declaration.relation {
        Relation::Constructibility { class } => {
            compiler_requirements
                .push(CompilerStaticRequirement::ConstructibilityValidated { class: *class });
        }

        Relation::Representation { object, allowed } => {
            let selected = case.id.representations.get(object).copied().ok_or(
                CompileError::MissingRepresentationChoice {
                    operation,
                    object: *object,
                },
            )?;

            compiler_requirements.push(CompilerStaticRequirement::RepresentationSelection {
                object: *object,
                allowed: allowed.clone(),
                selected,
            });
            structural_requirements.push(
                BackendStructuralRequirement::EncodeAndAuthenticateRepresentation {
                    object: *object,
                    representation: selected,
                },
            );
        }

        Relation::LifecycleExit { object, exit } => {
            compiler_requirements.push(CompilerStaticRequirement::LifecycleExitDeclared {
                object: *object,
                exit: *exit,
            });
            structural_requirements.push(
                BackendStructuralRequirement::RetainRequiredLifecycleExit {
                    object: *object,
                    exit: *exit,
                },
            );
        }

        Relation::PermissionlessAuthorization => {
            structural_requirements
                .push(BackendStructuralRequirement::SecretFreeOperationPath { operation });
        }

        Relation::SubstrateConservation { asset } => {
            external_evidence.insert(ExternalEvidenceRequirement::SubstrateConservation {
                operation,
                asset: *asset,
            });
        }

        Relation::Cardinality { .. }
        | Relation::AllowedObjectFamilies { .. }
        | Relation::Recognition { .. }
        | Relation::AmountConservation { .. }
        | Relation::OwnerAuthorization { .. }
        | Relation::SponsorIsolation
        | Relation::SponsorEnvelopeMultiplicity { .. }
        | Relation::RootPolicy { .. }
        | Relation::ProjectionPolicy { .. }
        | Relation::CanonicalDeltaPolicy { .. }
        | Relation::OpenFlowPolicy { .. }
        | Relation::ExpressionPredicate { .. } => {}
    }

    let runtime_requirements = match (activity, discharge.runtime) {
        (RelationActivity::Active, Some((scope, multiplicity))) => {
            vec![RuntimePlacementRequirement {
                relation: declaration.id.clone(),
                scope,
                multiplicity,
                sources: relation_sources(&declaration.id, case),
            }]
        }
        _ => Vec::new(),
    };

    Ok(RelationCasePlan {
        relation: declaration.id.clone(),
        case: case.id.clone(),
        activation: discharge.activation,
        activity,
        boundaries: discharge.boundaries,
        compiler_requirements,
        structural_requirements,
        runtime_requirements,
        external_evidence,
    })
}

/// Classify every in-scope relation in every applicable case.
///
/// # Errors
///
/// Any failure of [`classify_relation_case`], or of
/// [`validate_relation_case_census`] on the result.
pub fn classify_relation_cases(
    relations: &CompilerRelationAnalysis,
    cases: &[ExecutionCase],
) -> Result<Vec<RelationCasePlan>, CompileError> {
    let mut plans = Vec::new();

    for case in cases {
        for node in relations.graph.node_weights() {
            if node.source.id.operation() != case.id.operation {
                continue;
            }

            plans.push(classify_relation_case(&node.source, case)?);
        }
    }

    plans.sort_by(|left, right| (&left.relation, &left.case).cmp(&(&right.relation, &right.case)));

    validate_relation_case_census(relations, cases, &plans)?;
    Ok(plans)
}

/// Validate the exact relation-case census.
///
/// Every in-scope relation crossed with every applicable execution case
/// appears exactly once. No relation may disappear because it is
/// vacuous, compiler-static, structural, or externally evidenced: those
/// are dispositions the plan states explicitly, not reasons to omit it.
///
/// # Errors
///
/// [`CompileError::DuplicateRelationCasePlan`] when one pair is planned
/// twice; [`CompileError::RelationCaseCensusMismatch`] when the planned
/// pairs differ from the required ones.
pub fn validate_relation_case_census(
    relations: &CompilerRelationAnalysis,
    cases: &[ExecutionCase],
    plans: &[RelationCasePlan],
) -> Result<(), CompileError> {
    let mut planned = BTreeSet::new();

    for plan in plans {
        let key = RelationCaseKey {
            relation: plan.relation.clone(),
            case: plan.case.clone(),
        };

        if !planned.insert(key) {
            return Err(CompileError::DuplicateRelationCasePlan {
                relation: plan.relation.clone(),
                case: plan.case.clone(),
            });
        }
    }

    let mut expected = BTreeSet::new();

    for case in cases {
        for node in relations.graph.node_weights() {
            if node.source.id.operation() != case.id.operation {
                continue;
            }

            expected.insert(RelationCaseKey {
                relation: node.source.id.clone(),
                case: case.id.clone(),
            });
        }
    }

    if planned != expected {
        return Err(CompileError::RelationCaseCensusMismatch {
            missing: expected.difference(&planned).cloned().collect(),
            unexpected: planned.difference(&expected).cloned().collect(),
        });
    }

    Ok(())
}

/// The case-independent discharge of one relation variant.
///
/// The pilots' conceptual matrix in one place: input recognition and
/// owner authorization are member-local every-member obligations;
/// output recognition is family-global, because an output object's
/// program is not assumed to execute while the output is created;
/// conservation, closure, and the policy relations are
/// transaction-global; permissionless authorization, representation,
/// lifecycle, and constructibility leave the runtime boundary
/// entirely; substrate conservation is external evidence and gets no
/// runtime carrier at all.
fn classify_discharge(relation: &Relation) -> RelationDischarge {
    use CarrierMultiplicity as Multiplicity;
    use DischargeBoundary as Boundary;

    let runtime = |scope, multiplicity, activation| RelationDischarge {
        boundaries: BTreeSet::from([Boundary::RuntimeCarrier]),
        activation,
        runtime: Some((scope, multiplicity)),
    };

    match relation {
        Relation::Cardinality { side, object, .. } => runtime(
            SemanticScope::FamilyGlobal {
                side: observed_to_transaction(*side),
                object: *object,
            },
            Multiplicity::ExactlyOne,
            family_activation(*object),
        ),

        Relation::Recognition { side, object, .. } => {
            let side = observed_to_transaction(*side);
            let activation = family_activation(*object);

            match side {
                // Every authenticated input member carries its own
                // recognition obligation.
                TransactionSide::Input => runtime(
                    SemanticScope::MemberLocal {
                        side,
                        object: *object,
                    },
                    Multiplicity::EveryMember,
                    activation,
                ),
                // An output object is not assumed to execute while it
                // is created, so its recognition is a family-global
                // obligation.
                TransactionSide::Output => runtime(
                    SemanticScope::FamilyGlobal {
                        side,
                        object: *object,
                    },
                    Multiplicity::ExactlyOne,
                    activation,
                ),
            }
        }

        Relation::AllowedObjectFamilies { side, .. } => runtime(
            SemanticScope::TransactionSideGlobal {
                side: observed_to_transaction(*side),
            },
            Multiplicity::ExactlyOne,
            ActivationCondition::Always,
        ),

        Relation::OwnerAuthorization { object } => runtime(
            SemanticScope::MemberLocal {
                side: TransactionSide::Input,
                object: *object,
            },
            Multiplicity::EveryMember,
            ActivationCondition::Always,
        ),

        // Conservation needs authenticated family totals on both sides,
        // so a per-member assignment can never discharge it. The
        // sponsor relations are active in every case too: with no
        // sponsor region the relation still establishes that none
        // exists, and only its sponsor-local source rows are
        // conditional — that is the activation of the row, not of the
        // relation.
        Relation::AmountConservation { .. }
        | Relation::SponsorIsolation
        | Relation::SponsorEnvelopeMultiplicity { .. }
        | Relation::OpenFlowPolicy { .. }
        | Relation::CanonicalDeltaPolicy { .. }
        | Relation::RootPolicy { .. }
        | Relation::ProjectionPolicy { .. }
        | Relation::ExpressionPredicate { .. } => runtime(
            SemanticScope::TransactionGlobal,
            Multiplicity::ExactlyOne,
            ActivationCondition::Always,
        ),

        Relation::PermissionlessAuthorization => RelationDischarge {
            boundaries: BTreeSet::from([Boundary::BackendStructural]),
            activation: ActivationCondition::Always,
            runtime: None,
        },

        Relation::Constructibility { .. } => RelationDischarge {
            boundaries: BTreeSet::from([Boundary::CompilerStatic]),
            activation: ActivationCondition::Always,
            runtime: None,
        },

        Relation::Representation { .. } | Relation::LifecycleExit { .. } => RelationDischarge {
            boundaries: BTreeSet::from([Boundary::CompilerStatic, Boundary::BackendStructural]),
            activation: ActivationCondition::Always,
            runtime: None,
        },

        Relation::SubstrateConservation { .. } => RelationDischarge {
            boundaries: BTreeSet::from([Boundary::ExternalEvidence]),
            activation: ActivationCondition::Always,
            runtime: None,
        },
    }
}

/// Relations over the optional sponsor family exist only where that
/// family does.
const fn family_activation(object: ObjectId) -> ActivationCondition {
    if is_sponsor_object(object) {
        ActivationCondition::WhenSponsorPresent
    } else {
        ActivationCondition::Always
    }
}

fn resolve_activity(activation: ActivationCondition, case: &ExecutionCaseId) -> RelationActivity {
    let active = match activation {
        ActivationCondition::Always => true,
        ActivationCondition::WhenSponsorPresent => case.sponsor == SponsorCase::Present,
        // A representation-conditional relation is active only under
        // the mode the plan already selected for the case.
        ActivationCondition::WhenRepresentation(mode) => {
            case.representations.values().any(|value| *value == mode)
        }
    };

    if active {
        RelationActivity::Active
    } else {
        RelationActivity::Vacuous
    }
}

/// The case's active source rows belonging to one relation.
fn relation_sources(relation: &RelationId, case: &ExecutionCase) -> Vec<SourceRequirement> {
    case.active_sources
        .iter()
        .filter(|row| row.operand.relation() == relation)
        .cloned()
        .collect()
}

const fn observed_to_transaction(side: realization::ObservedSide) -> TransactionSide {
    match side {
        realization::ObservedSide::Input => TransactionSide::Input,
        realization::ObservedSide::Output => TransactionSide::Output,
    }
}
