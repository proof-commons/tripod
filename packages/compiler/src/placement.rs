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
//! Classification assigns no carrier. A runtime requirement states the
//! semantic scope, the multiplicity, and the source rows an eventual
//! carrier must receive; which abstract carrier roles are eligible, and
//! what layout must route those sources there, are separate stages.
//! Nothing here names a transaction index, tapleaf, stack slot, or
//! witness position.
//!
//! # Exact placement search (Guide-5 Tranche E)
//!
//! The search consumes one proof-plan candidate's eligible carrier sets
//! and returns the *complete* feasible placement set. It weighs nothing:
//! there is no target resource model yet, so a cost objective would be
//! invented rather than measured, and "cheapest placement" is not a
//! question this stage may answer. Several feasible placements are
//! retained side by side in canonical order.
//!
//! Two properties keep that set finite and meaningful. First, retention
//! is minimal by policy: an exactly-one obligation retains one carrier,
//! an every-member obligation retains one quantified role or one typed
//! complete-family proof, an at-least-one obligation retains only
//! inclusion-minimal sets, and only a deliberate-duplication obligation
//! retains more than one carrier at once — no arbitrary superset is ever
//! generated, because "more carriers" is not automatically better and
//! supersets would grow the set exponentially without adding semantic
//! content. Second, exhaustion of an explicit state or candidate limit
//! returns a typed error and no partial result: neither infeasibility
//! nor completeness may be claimed from a truncated search, and no
//! first-seen placement is silently selected.
//!
//! The search state is a product over independent relation-case
//! requirements rather than a graph, so it is enumerated by a
//! deterministic depth-first odometer over canonically ordered option
//! lists. D007 licenses direct Petgraph use for graph-shaped state; this
//! state has no edges to walk, and wrapping a product in a graph would
//! add storage and index metadata whose only effect is to leak a
//! `NodeIndex` into a stage that must never observe one.

// The item-level allowances below are of three kinds: declared
// placement vocabulary the two pilots do not exercise, the stable
// projections (§12) that nothing inside the analysis builds, and the
// global cross-operation placement stack. The last is the important
// one — §9.6 and §20.2 forbid the canonical path from enumerating the
// product across operations, so the global enumerator is *expected* to
// have no production caller and is retained for the manual phase-exit
// comparison against the factors.

#[cfg(test)]
use crate::{relation::CompilerRelationAnalysis, sponsor_region::ordinary_lbtc_role};
use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZeroU64,
};

use architecture::{ObjectId, OperationId};
use realization::{
    ConstructibilityClass, ExternalEvidenceRequirement, Relation, RelationDeclaration, RelationId,
    RepresentationMode, TransactionSide,
};

use crate::{
    CompileError,
    carrier::{CarrierEligibility, CarrierQuantification, CarrierRole},
    case::{
        ConservedAmountVisibility, ExecutionCase, ExecutionCaseId, SponsorCase,
        conserved_amount_visibility, is_sponsor_region_family,
    },
    layout::LayoutRequirement,
    proof::ProofPlanCandidate,
    source::SourceRequirement,
    sponsor_region::OrdinaryLbtcRole,
};
#[cfg(test)]
use crate::{
    carrier::relation_case_eligibility,
    case::{case_census, execution_cases, validate_case_census},
    layout::{layout_requirements, validate_layout_census},
};

/// Where one requirement of a relation is discharged.
///
/// Compiler-static does not mean target-verified, and backend-structural
/// does not mean already emitted: both are obligations recorded against
/// a future backend, not completed evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DischargeBoundary {
    /// Scope evaluation requires this policy; no leaf carries it (`rule:compiler:relations`).
    ModelScope,
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
    #[allow(dead_code)]
    ObjectFamilyGlobal { object: ObjectId },
    /// The whole operation.
    TransactionGlobal,
}

/// When one relation's obligation exists at all.
///
/// A case-independent property of the relation. The per-case resolution
/// of it is [`RelationActivity`].
///
/// A representation condition names its object as well as its mode, for
/// the same reason [`crate::source::RequirementActivation`] does: a
/// representation choice for one object must never activate a condition
/// belonging to another.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ActivationCondition {
    Always,
    WhenSponsorPresent,
    #[allow(dead_code)]
    WhenRepresentation {
        object: ObjectId,
        mode: RepresentationMode,
    },
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
    #[allow(dead_code)]
    AtLeastOne,
    #[allow(dead_code)]
    DeliberateDuplication,
}

/// A property the compiler validates directly from typed input.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
/// change with the case, with the single stated exception of
/// [`ConservedAmountVisibility`]; only activity and the active source
/// set do.
#[derive(Clone, Debug, PartialEq, Eq)]
struct RelationDischarge {
    boundaries: BTreeSet<DischargeBoundary>,
    activation: ActivationCondition,
    runtime: Option<(SemanticScope, CarrierMultiplicity)>,
}

/// Closure retains scope-only resource policy without a leaf (`rule:compiler:relations`).
fn classify_declared_discharge(
    declaration: &RelationDeclaration,
    ordinary_lbtc: OrdinaryLbtcRole,
    amounts: ConservedAmountVisibility,
) -> RelationDischarge {
    if matches!(
        declaration.id.operation(),
        OperationId::CompactAsh | OperationId::TransferLive
    ) && matches!(
        declaration.relation,
        Relation::SponsorEnvelopeMultiplicity { .. }
    ) {
        RelationDischarge {
            boundaries: BTreeSet::from([DischargeBoundary::ModelScope]),
            activation: ActivationCondition::Always,
            runtime: None,
        }
    } else {
        classify_discharge(&declaration.relation, ordinary_lbtc, amounts)
    }
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
    ordinary_lbtc: OrdinaryLbtcRole,
) -> Result<RelationCasePlan, CompileError> {
    let operation = declaration.id.operation();
    let amounts = conserved_amount_visibility(declaration, &case.id)?;
    let discharge = classify_declared_discharge(declaration, ordinary_lbtc, amounts);
    let activity = resolve_activity(discharge.activation, &case.id);

    let mut compiler_requirements = Vec::new();
    let mut structural_requirements = Vec::new();
    let mut external_evidence = BTreeSet::new();

    match &declaration.relation {
        Relation::OperatorAuthorization
        | Relation::Constructibility {
            class: realization::ConstructibilityClass::Operator,
        } => {
            external_evidence
                .insert(ExternalEvidenceRequirement::OperatorAuthorization { operation });
        }

        Relation::Constructibility { class } => {
            compiler_requirements
                .push(CompilerStaticRequirement::ConstructibilityValidated { class: *class });
        }

        Relation::Representation { object, allowed } => {
            let selected = selected_representation(operation, *object, case)?;

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

        // Conservation over amounts the target holds as commitments is
        // the target's own confidential-transaction rules to establish
        // and nothing this compiler emits ever reads (Guide-13 §9.3,
        // §10.6). Conservation over readable amounts is arithmetic a
        // carrier performs, and raises nothing here.
        Relation::AmountConservation { asset, .. }
            if amounts == ConservedAmountVisibility::Committed =>
        {
            external_evidence.insert(ExternalEvidenceRequirement::ConfidentialValueConservation {
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
#[cfg(test)]
pub fn classify_relation_cases(
    relations: &CompilerRelationAnalysis,
    cases: &[ExecutionCase],
) -> Result<Vec<RelationCasePlan>, CompileError> {
    let mut plans = Vec::new();
    let mut roles: BTreeMap<OperationId, OrdinaryLbtcRole> = BTreeMap::new();

    for case in cases {
        let ordinary_lbtc = *roles
            .entry(case.id.operation)
            .or_insert_with(|| ordinary_lbtc_role(relations, case.id.operation));

        for node in relations.graph.node_weights() {
            if node.source.id.operation() != case.id.operation {
                continue;
            }

            plans.push(classify_relation_case(&node.source, case, ordinary_lbtc)?);
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
#[cfg(test)]
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

fn selected_representation(
    operation: OperationId,
    object: ObjectId,
    case: &ExecutionCase,
) -> Result<RepresentationMode, CompileError> {
    case.id
        .representations
        .get(&object)
        .copied()
        .ok_or(CompileError::MissingRepresentationChoice { operation, object })
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
fn classify_discharge(
    relation: &Relation,
    ordinary_lbtc: OrdinaryLbtcRole,
    amounts: ConservedAmountVisibility,
) -> RelationDischarge {
    use CarrierMultiplicity as Multiplicity;
    use DischargeBoundary as Boundary;

    let runtime = |scope, multiplicity, activation| RelationDischarge {
        boundaries: BTreeSet::from([Boundary::RuntimeCarrier]),
        activation,
        runtime: Some((scope, multiplicity)),
    };

    // Committed conservation has no carrier at all. Giving it one would
    // be the §10.6 defect exactly: a local program that appears to
    // implement CT conservation because target consensus eventually
    // accepts the transaction. The relation is still declared, still
    // active, and still carries its capability and source rows — what
    // it does not have is a script that discharges it.
    if let (Relation::AmountConservation { .. }, ConservedAmountVisibility::Committed) =
        (relation, amounts)
    {
        return RelationDischarge {
            boundaries: BTreeSet::from([Boundary::ExternalEvidence]),
            activation: ActivationCondition::Always,
            runtime: None,
        };
    }

    match relation {
        Relation::Cardinality { side, object, .. } => runtime(
            SemanticScope::FamilyGlobal {
                side: observed_to_transaction(*side),
                object: *object,
            },
            Multiplicity::ExactlyOne,
            family_activation(*object, ordinary_lbtc),
        ),

        Relation::Recognition { side, object, .. } => {
            let side = observed_to_transaction(*side);
            let activation = family_activation(*object, ordinary_lbtc);

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

        Relation::SubstrateConservation { .. }
        | Relation::OperatorAuthorization
        | Relation::Constructibility {
            class: realization::ConstructibilityClass::Operator,
        } => RelationDischarge {
            boundaries: BTreeSet::from([Boundary::ExternalEvidence]),
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
    }
}

/// Relations over the optional sponsor family exist only where that
/// family does.
const fn family_activation(
    object: ObjectId,
    ordinary_lbtc: OrdinaryLbtcRole,
) -> ActivationCondition {
    if is_sponsor_region_family(object, ordinary_lbtc) {
        ActivationCondition::WhenSponsorPresent
    } else {
        ActivationCondition::Always
    }
}

/// Resolve one relation's activation in one case.
///
/// Reachable from outside the module — the module itself is private, so
/// this stays inside the crate — because the census tests must exercise
/// activation conditions no pilot relation currently constructs: the
/// classification matrix produces only the unconditional and
/// sponsor-conditional forms today, and a representation-conditional
/// relation would otherwise be untestable.
pub fn resolve_activity(
    activation: ActivationCondition,
    case: &ExecutionCaseId,
) -> RelationActivity {
    let active = match activation {
        ActivationCondition::Always => true,
        ActivationCondition::WhenSponsorPresent => case.sponsor == SponsorCase::Present,
        // A representation-conditional relation is active only where
        // the plan selected that mode for the condition's own object.
        ActivationCondition::WhenRepresentation { object, mode } => {
            case.representations.get(&object) == Some(&mode)
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

// --- exact placement search (Guide-5 Tranche E) ---

/// One carrier selected for one relation-case obligation.
///
/// The quantification travels with the selection: a coordinator proving
/// a property of one complete authenticated family and a role executing
/// once per member are different discharges of the same obligation, and
/// a later stage must not have to re-derive which one was chosen.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlacedCarrier {
    pub carrier: CarrierRole,
    pub quantification: CarrierQuantification,
}

/// The carriers one placement assigns to one relation-case obligation.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlacementAssignment {
    pub relation: RelationId,
    pub case: ExecutionCaseId,
    /// Non-empty, canonically sorted, and free of duplicates.
    pub carriers: Vec<PlacedCarrier>,
}

impl PlacementAssignment {
    /// This assignment's census key.
    #[must_use]
    pub fn key(&self) -> RelationCaseKey {
        RelationCaseKey {
            relation: self.relation.clone(),
            case: self.case.clone(),
        }
    }
}

/// One complete feasible placement of one proof-plan candidate.
///
/// Every active runtime relation-case of the candidate appears exactly
/// once. The layout requirements are the ones *this* selection depends
/// on — a strict subset, in general, of the operation-wide census, which
/// covers every eligible carrier rather than the chosen ones.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlacementCandidate {
    /// Sorted by relation then case.
    pub assignments: Vec<PlacementAssignment>,
    /// Canonically sorted and free of duplicates.
    pub layout_requirements: Vec<LayoutRequirement>,
}

/// Explicit work limits for the exact placement search.
///
/// Configuration with an explicit constructor and no ambient default:
/// a silent limit is a silent truncation, and no configuration identity
/// is minted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlacementSearchLimits {
    /// The most search states one operation's placement search may
    /// visit before the analysis fails as too complex.
    pub maximum_states: NonZeroU64,
    /// The most complete candidate assignments it may consider.
    pub maximum_candidates: NonZeroU64,
}

impl PlacementSearchLimits {
    /// States both limits explicitly.
    #[must_use]
    pub const fn new(maximum_states: NonZeroU64, maximum_candidates: NonZeroU64) -> Self {
        Self {
            maximum_states,
            maximum_candidates,
        }
    }
}

/// Diagnostic search statistics — never semantic identity.
///
/// Deliberately absent from the stable projection: a state count is an
/// artifact of how the search walked its options, not a property of the
/// placements it found.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlacementSearchReport {
    pub states_visited: u64,
    pub complete_assignments: u64,
    pub feasible_placements: u64,
    pub retained_options: u64,
}

/// The complete canonical feasible placement set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeasiblePlacements {
    pub candidates: Vec<PlacementCandidate>,
    pub search: PlacementSearchReport,
}

/// One proof-plan candidate with its complete placement analysis.
///
/// The proof plan is carried as its complete typed value rather than by
/// vector position, search order, graph index, candidate number, or
/// hash: until an admitted plan identity exists, the typed value is the
/// only honest boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlacedProofPlanCandidate {
    pub proof_plan: ProofPlanCandidate,
    pub execution_cases: Vec<ExecutionCase>,
    pub relation_case_plans: Vec<RelationCasePlan>,
    pub feasible_placements: Vec<PlacementCandidate>,
    /// The operation-wide census over every eligible carrier.
    pub layout_requirements: Vec<LayoutRequirement>,
}

/// The stable projection of one placement.
///
/// Set- and map-shaped by construction, so no iteration order, vector
/// position, or search state can reach a comparison.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlacementCandidateProjection {
    pub assignments: BTreeMap<RelationCaseKey, BTreeSet<PlacedCarrier>>,
    pub layout_requirements: BTreeSet<LayoutRequirement>,
}

/// The stable projection of a complete feasible placement set.
///
/// Typed semantic values only: relation IDs, execution-case IDs, carrier
/// roles, relation-case assignments, and layout requirements. Graph
/// handles, search order, state counts, candidate positions, elapsed
/// time, diagnostics, and digests are all excluded, and the search
/// limits are excluded too — they can only turn a complete result into a
/// typed failure, never change which placements are feasible.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct PlacementProjection {
    pub candidates: BTreeSet<PlacementCandidateProjection>,
}

impl PlacementCandidate {
    /// This placement's stable projection.
    #[must_use]
    pub fn project(&self) -> PlacementCandidateProjection {
        PlacementCandidateProjection {
            assignments: self
                .assignments
                .iter()
                .map(|assignment| {
                    (
                        assignment.key(),
                        assignment.carriers.iter().cloned().collect(),
                    )
                })
                .collect(),
            layout_requirements: self.layout_requirements.iter().cloned().collect(),
        }
    }
}

impl FeasiblePlacements {
    /// This feasible set's stable projection.
    #[must_use]
    #[allow(dead_code)]
    pub fn project(&self) -> PlacementProjection {
        PlacementProjection {
            candidates: self
                .candidates
                .iter()
                .map(PlacementCandidate::project)
                .collect(),
        }
    }
}

/// Validate the exact placement census.
///
/// The carried relation-cases are exactly the relation-cases whose plan
/// states a runtime requirement. A compiler-static, backend-structural,
/// or externally evidenced relation-case must not appear — assigning it
/// a runtime carrier would report an unfinished obligation as target
/// execution — and no active runtime relation-case may disappear.
///
/// # Errors
///
/// [`CompileError::PlacementCensusMismatch`] when the carried census
/// differs from the required one, including when one relation-case is
/// carried twice.
#[cfg(test)]
pub fn validate_placement_census(
    plans: &[RelationCasePlan],
    eligibility: &[CarrierEligibility],
) -> Result<(), CompileError> {
    let lowered = crate::engine_attestation::placement::lower(plans, eligibility, None);
    crate::engine_placement::validate_census(&lowered.records)
        .map_err(|error| lowered.refusal(error))
}

/// Enumerate the complete feasible placement set of one proof plan.
///
/// # Errors
///
/// Any failure of [`validate_placement_census`];
/// [`CompileError::GlobalRelationHasOnlyLocalCarrier`],
/// [`CompileError::UnconditionalRelationOnOptionalCarrier`], or
/// [`CompileError::NoEligibleCarrier`] when one obligation retains no
/// admissible carrier at all; any failure of [`validate_placement`];
/// [`CompileError::PlacementSearchStateLimitExceeded`] or
/// [`CompileError::PlacementCandidateLimitExceeded`] on exhaustion, in
/// which case no partial result is returned.
#[cfg(test)]
pub fn enumerate_feasible_placements(
    plans: &[RelationCasePlan],
    eligibility: &[CarrierEligibility],
    limits: PlacementSearchLimits,
) -> Result<FeasiblePlacements, CompileError> {
    let relation_case_count = crate::instrumentation::count(plans.len());
    let span = tracing::debug_span!(target: "compiler::placement", "enumerate_feasible_placements", state_limit = limits.maximum_states.get(), candidate_limit = limits.maximum_candidates.get(), relation_case_count);
    let _entered = span.enter();
    let mut search = PlacementSearchReport::default();
    let result = enumerate_feasible_placements_core(plans, eligibility, limits, &mut search);
    match &result {
        Ok(_) => {
            tracing::debug!(target: "compiler::placement", outcome = "completed", visited_state_count = search.states_visited, complete_assignment_count = search.complete_assignments, feasible_placement_count = search.feasible_placements, retained_option_count = search.retained_options, message = "placement search completed");
        }
        Err(error) if error.tracing_failed() => {
            tracing::debug!(target: "compiler::placement", outcome = "failed", error_kind = error.tracing_kind(), visited_state_count = search.states_visited, complete_assignment_count = search.complete_assignments, feasible_placement_count = search.feasible_placements, retained_option_count = search.retained_options, message = "placement search completed");
        }
        Err(error) => {
            tracing::debug!(target: "compiler::placement", outcome = "refused", refusal_kind = error.tracing_kind(), visited_state_count = search.states_visited, complete_assignment_count = search.complete_assignments, feasible_placement_count = search.feasible_placements, retained_option_count = search.retained_options, message = "placement search completed");
        }
    }
    result.map(|candidates| FeasiblePlacements { candidates, search })
}

#[cfg(test)]
fn enumerate_feasible_placements_core(
    plans: &[RelationCasePlan],
    eligibility: &[CarrierEligibility],
    limits: PlacementSearchLimits,
    search: &mut PlacementSearchReport,
) -> Result<Vec<PlacementCandidate>, CompileError> {
    validate_placement_census(plans, eligibility)?;
    crate::engine_attestation::placement::lower(plans, eligibility, None).enumerate(limits, search)
}

/// Validate one complete placement against every hard constraint.
///
/// Restating the constraints over the assembled value catches a
/// generator that satisfied them option by option and still produced an
/// inconsistent whole, and lets a placement built elsewhere be checked
/// without trusting how it was built. The minimality policy of §10.3 is
/// restated here rather than delegated to the retained-option helper: a
/// validator that called the enumerator's own helper could only agree
/// with it, and the property being checked is precisely that the
/// enumerator's output is inside the candidate language.
///
/// # Errors
///
/// [`CompileError::PlacementCensusMismatch`] when the assignments are
/// not exactly the carried relation-cases;
/// [`CompileError::UnpermittedCarrierPlacement`] when an assigned
/// carrier is not eligible, is a non-runtime role, cannot discharge the
/// obligation's semantic scope, cannot discharge its multiplicity, or is
/// an optional sponsor carrier of an unconditional relation;
/// [`CompileError::DuplicatePlacedCarrier`] when one assignment repeats
/// a carrier role; [`CompileError::NonCanonicalCarrierAssignment`] when
/// a carrier set is not one exact retained option of its obligation;
/// [`CompileError::MissingLayoutRequirement`] when a selected carrier
/// depends on a layout requirement the placement does not state;
/// [`CompileError::UnexpectedLayoutRequirement`] when the placement
/// states one no selected carrier depends on.
pub fn validate_placement(
    plans: &[RelationCasePlan],
    eligibility: &[CarrierEligibility],
    placement: &PlacementCandidate,
) -> Result<(), CompileError> {
    let lowered = crate::engine_attestation::placement::lower(plans, eligibility, Some(placement));
    crate::engine_placement::validate_placement(
        &lowered.records,
        &crate::engine_attestation::placement::candidate(placement),
    )
    .map_err(|error| lowered.refusal(error))
}

/// Place one complete proof-plan candidate.
///
/// The per-candidate constructor of the Guide-5 §3 value: cases,
/// relation-case plans, the complete feasible placement set, and the
/// operation-wide layout census for exactly this plan. Assembling the
/// analysis across the complete feasible plan set is a later stage.
///
/// # Errors
///
/// Any failure of [`crate::case::execution_cases`],
/// [`classify_relation_cases`],
/// [`crate::carrier::relation_case_eligibility`],
/// [`crate::layout::layout_requirements`], or
/// [`enumerate_feasible_placements`].
#[cfg(test)]
pub fn place_proof_plan(
    relations: &CompilerRelationAnalysis,
    candidate: &ProofPlanCandidate,
    limits: PlacementSearchLimits,
) -> Result<PlacedProofPlanCandidate, CompileError> {
    let cases = execution_cases(relations, candidate)?;
    let plans = classify_relation_cases(relations, &cases)?;
    let eligibility = relation_case_eligibility(relations, &plans)?;
    let requirements = layout_requirements(relations, &plans, &eligibility)?;
    let placements = enumerate_feasible_placements(&plans, &eligibility, limits)?;

    Ok(PlacedProofPlanCandidate {
        proof_plan: candidate.clone(),
        execution_cases: cases,
        relation_case_plans: plans,
        feasible_placements: placements.candidates,
        layout_requirements: requirements,
    })
}

/// The complete placement analysis of one feasible proof-plan set.
///
/// One entry per offered proof-plan candidate, in the canonical order of
/// the typed plan values themselves. There is no plan index, plan
/// number, or plan digest here: the offered order is not observed, and
/// two analyses of the same plan set are equal rather than merely
/// similar.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct PlacedProofPlans {
    pub placed: Vec<PlacedProofPlanCandidate>,
}

/// The stable projection of one placed proof-plan candidate.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct PlacedProofPlanProjection {
    pub execution_cases: BTreeSet<ExecutionCaseId>,
    pub relation_case_plans: BTreeMap<RelationCaseKey, RelationCasePlan>,
    pub feasible_placements: BTreeSet<PlacementCandidateProjection>,
    pub layout_requirements: BTreeSet<LayoutRequirement>,
}

/// The stable projection of a complete placed plan set.
///
/// Keyed by the complete typed proof plan, so the key is the same
/// boundary the analysis itself uses. Search order, vector position, and
/// candidate counts cannot reach a comparison of two projections.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct PlacedAnalysisProjection {
    pub plans: BTreeMap<ProofPlanCandidate, PlacedProofPlanProjection>,
}

impl PlacedProofPlanCandidate {
    /// This placed candidate's stable projection.
    #[must_use]
    #[allow(dead_code)]
    pub fn project(&self) -> PlacedProofPlanProjection {
        PlacedProofPlanProjection {
            execution_cases: self
                .execution_cases
                .iter()
                .map(|case| case.id.clone())
                .collect(),
            relation_case_plans: self
                .relation_case_plans
                .iter()
                .map(|plan| {
                    (
                        RelationCaseKey {
                            relation: plan.relation.clone(),
                            case: plan.case.clone(),
                        },
                        plan.clone(),
                    )
                })
                .collect(),
            feasible_placements: self
                .feasible_placements
                .iter()
                .map(PlacementCandidate::project)
                .collect(),
            layout_requirements: self.layout_requirements.iter().cloned().collect(),
        }
    }
}

impl PlacedProofPlans {
    /// The semantic execution-case census across the whole plan set.
    ///
    /// A set rather than a count of candidates: two plans that fixed the
    /// same representations share their case identities, and the census
    /// counts cases.
    #[must_use]
    #[allow(dead_code)]
    pub fn execution_case_census(&self) -> BTreeSet<ExecutionCaseId> {
        self.placed
            .iter()
            .flat_map(|entry| entry.execution_cases.iter().map(|case| case.id.clone()))
            .collect()
    }

    /// This analysis's stable projection.
    #[must_use]
    #[allow(dead_code)]
    pub fn project(&self) -> PlacedAnalysisProjection {
        PlacedAnalysisProjection {
            plans: self
                .placed
                .iter()
                .map(|entry| (entry.proof_plan.clone(), entry.project()))
                .collect(),
        }
    }
}

/// Place every feasible proof-plan candidate of one analysis.
///
/// Placement is plan-specific — a candidate selecting a private
/// committed representation with a confidential conservation proof has
/// different source and layout requirements from one selecting explicit
/// values with public arithmetic — so every candidate is placed on its
/// own terms rather than once for a merged plan. The result is the
/// complete collection: no candidate is dropped, none is preferred, and
/// nothing is weighted.
///
/// # Errors
///
/// Any failure of [`place_proof_plan`] for any candidate, or of
/// [`validate_placed_proof_plans`] on the assembled collection.
#[cfg(test)]
pub fn place_feasible_proof_plans(
    relations: &CompilerRelationAnalysis,
    candidates: &[ProofPlanCandidate],
    limits: PlacementSearchLimits,
) -> Result<PlacedProofPlans, CompileError> {
    let proof_plan_count = crate::instrumentation::count(candidates.len());
    let span = tracing::debug_span!(target: "compiler::placement", "place_feasible_proof_plans", proof_plan_count);
    let _entered = span.enter();
    let mut placed = Vec::with_capacity(candidates.len());

    for candidate in candidates {
        placed.push(place_proof_plan(relations, candidate, limits)?);
    }

    placed.sort_by(|left, right| left.proof_plan.cmp(&right.proof_plan));

    let analysis = PlacedProofPlans { placed };

    validate_placed_proof_plans(relations, candidates, &analysis)?;
    Ok(analysis)
}

/// Validate one complete placed plan set.
///
/// Every stage is re-checked over the assembled value rather than
/// trusted because the assembler produced it: each candidate's case
/// census, its relation-case census, every one of its feasible
/// placements against the hard constraints, and its layout census. The
/// set-level properties are that no plan is placed twice, that the
/// placed plans are exactly the offered plans, and that the union of the
/// per-candidate cases is exactly the semantic case census the relations
/// and the offered candidates require.
///
/// The plan-set comparison is the primary one, and the case census does
/// not subsume it: representation choices reach case identity but proof
/// choices do not, so two plans differing only in a selected proof share
/// their case identities, and dropping one would leave the union intact.
/// The comparison is over complete typed plan values, which is the same
/// boundary the analysis itself is keyed by.
///
/// # Errors
///
/// [`CompileError::DuplicatePlacedProofPlan`] when one typed plan is
/// placed twice; [`CompileError::PlacedProofPlanCensusMismatch`] when
/// the placed plan set differs from the offered one;
/// [`CompileError::ExecutionCaseCensusMismatch`] when the
/// union of the placed cases differs from the required census; any
/// failure of [`crate::case::validate_case_census`],
/// [`validate_relation_case_census`],
/// [`crate::carrier::relation_case_eligibility`], [`validate_placement`],
/// or [`crate::layout::validate_layout_census`].
#[cfg(test)]
pub fn validate_placed_proof_plans(
    relations: &CompilerRelationAnalysis,
    candidates: &[ProofPlanCandidate],
    analysis: &PlacedProofPlans,
) -> Result<(), CompileError> {
    let mut placed = BTreeSet::new();

    for entry in &analysis.placed {
        if !placed.insert(entry.proof_plan.clone()) {
            return Err(CompileError::DuplicatePlacedProofPlan);
        }
    }

    let offered = candidates.iter().cloned().collect::<BTreeSet<_>>();

    if placed != offered {
        return Err(CompileError::PlacedProofPlanCensusMismatch {
            missing: offered.difference(&placed).count(),
            unexpected: placed.difference(&offered).count(),
        });
    }

    for entry in &analysis.placed {
        validate_case_census(relations, &entry.proof_plan, &entry.execution_cases)?;
        validate_relation_case_census(
            relations,
            &entry.execution_cases,
            &entry.relation_case_plans,
        )?;

        let eligibility = relation_case_eligibility(relations, &entry.relation_case_plans)?;

        for placement in &entry.feasible_placements {
            validate_placement(&entry.relation_case_plans, &eligibility, placement)?;
        }

        validate_layout_census(
            &entry.relation_case_plans,
            &eligibility,
            &entry.layout_requirements,
        )?;
    }

    let derived = analysis.execution_case_census();
    let expected = case_census(relations, candidates)?;

    if derived != expected {
        return Err(CompileError::ExecutionCaseCensusMismatch {
            missing: expected.difference(&derived).cloned().collect(),
            unexpected: derived.difference(&expected).cloned().collect(),
        });
    }

    Ok(())
}
