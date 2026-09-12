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
    carrier::{
        CarrierEligibility, CarrierQuantification, CarrierRole, EligibleCarrier,
        is_sponsor_region_carrier, relation_case_eligibility,
    },
    case::{
        ConservedAmountVisibility, ExecutionCase, ExecutionCaseId, SponsorCase, case_census,
        conserved_amount_visibility, execution_cases, is_sponsor_region_family,
        validate_case_census,
    },
    layout::{
        LayoutRequirement, layout_requirements, selected_carrier_requirements,
        validate_layout_census,
    },
    proof::ProofPlanCandidate,
    relation::CompilerRelationAnalysis,
    search_counter::{admit_search_state, record_search_event},
    source::SourceRequirement,
    sponsor_region::{GATED_ORDINARY_LBTC_ROLE, OrdinaryLbtcRole, ordinary_lbtc_role},
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
    let discharge = classify_discharge(&declaration.relation, ordinary_lbtc, amounts);
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
    case.id.representations.get(&object).copied().ok_or(
        CompileError::MissingRepresentationChoice { operation, object },
    )
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
pub fn validate_placement_census(
    plans: &[RelationCasePlan],
    eligibility: &[CarrierEligibility],
) -> Result<(), CompileError> {
    let mut carried = BTreeSet::new();
    let mut duplicated = BTreeSet::new();

    for analysis in eligibility {
        let key = RelationCaseKey {
            relation: analysis.relation.clone(),
            case: analysis.case.clone(),
        };

        if !carried.insert(key.clone()) {
            duplicated.insert(key);
        }
    }

    let required = plans
        .iter()
        .filter(|plan| !plan.runtime_requirements.is_empty())
        .map(|plan| RelationCaseKey {
            relation: plan.relation.clone(),
            case: plan.case.clone(),
        })
        .collect::<BTreeSet<_>>();

    if carried != required || !duplicated.is_empty() {
        let mut unexpected = carried.difference(&required).cloned().collect::<Vec<_>>();
        unexpected.extend(duplicated);
        unexpected.sort();
        unexpected.dedup();

        return Err(CompileError::PlacementCensusMismatch {
            missing: required.difference(&carried).cloned().collect(),
            unexpected,
        });
    }

    Ok(())
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
pub fn enumerate_feasible_placements(
    plans: &[RelationCasePlan],
    eligibility: &[CarrierEligibility],
    limits: PlacementSearchLimits,
) -> Result<FeasiblePlacements, CompileError> {
    validate_placement_census(plans, eligibility)?;

    // The input order of the eligibility and plan slices is never
    // observed: both are indexed by their typed census key, so a
    // permuted input produces an equal result rather than a permuted
    // one.
    let planned = plans
        .iter()
        .map(|plan| {
            (
                RelationCaseKey {
                    relation: plan.relation.clone(),
                    case: plan.case.clone(),
                },
                plan,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let carried = eligibility
        .iter()
        .map(|analysis| {
            (
                RelationCaseKey {
                    relation: analysis.relation.clone(),
                    case: analysis.case.clone(),
                },
                analysis,
            )
        })
        .collect::<BTreeMap<_, _>>();

    let mut keys = Vec::with_capacity(carried.len());
    let mut options = Vec::with_capacity(carried.len());

    for (key, analysis) in &carried {
        let plan = planned
            .get(key)
            .ok_or_else(|| CompileError::PlacementCensusMismatch {
                missing: vec![key.clone()],
                unexpected: Vec::new(),
            })?;

        keys.push(key.clone());
        options.push(retained_options(plan, analysis)?);
    }

    let mut search = PlacementSearchReport {
        retained_options: options
            .iter()
            .map(|list| u64::try_from(list.len()).unwrap_or(u64::MAX))
            .sum(),
        ..PlacementSearchReport::default()
    };
    let mut candidates = Vec::new();
    let mut state = PlacementSearchState {
        keys: &keys,
        options: &options,
        plans,
        eligibility,
        limits,
        search: &mut search,
        candidates: &mut candidates,
    };

    visit_placement(&mut state, &mut Vec::new(), 0)?;

    candidates.sort();
    candidates.dedup();
    search.feasible_placements = u64::try_from(candidates.len()).unwrap_or(u64::MAX);

    Ok(FeasiblePlacements { candidates, search })
}

/// Validate one complete placement against every hard constraint.
///
/// Restating the constraints over the assembled value catches a
/// generator that satisfied them option by option and still produced an
/// inconsistent whole, and lets a placement built elsewhere be checked
/// without trusting how it was built. The minimality policy of §10.3 is
/// restated here rather than delegated to [`retained_options`]: a
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
    validate_placement_census(plans, eligibility)?;

    let assigned = placement
        .assignments
        .iter()
        .map(PlacementAssignment::key)
        .collect::<BTreeSet<_>>();
    let carried = eligibility
        .iter()
        .map(|analysis| RelationCaseKey {
            relation: analysis.relation.clone(),
            case: analysis.case.clone(),
        })
        .collect::<BTreeSet<_>>();

    if assigned != carried || assigned.len() != placement.assignments.len() {
        return Err(CompileError::PlacementCensusMismatch {
            missing: carried.difference(&assigned).cloned().collect(),
            unexpected: assigned.difference(&carried).cloned().collect(),
        });
    }

    // A repeated layout requirement is surplus for the same reason an
    // unrelated one is: the placement states an obligation twice and
    // justifies it once.
    let mut stated = BTreeSet::new();
    let mut surplus = BTreeSet::new();

    for requirement in &placement.layout_requirements {
        if !stated.insert(requirement.clone()) {
            surplus.insert(requirement.clone());
        }
    }

    let mut depended = BTreeSet::new();

    for assignment in &placement.assignments {
        depended.extend(validate_assignment(
            plans,
            eligibility,
            assignment,
            &stated,
        )?);
    }

    // Exact placement-local layout: every requirement the selected
    // carriers depend on is stated, and nothing else is. A requirement
    // of an unselected carrier or of another relation-case belongs to
    // the operation-wide census, not to this placement.
    surplus.extend(
        stated
            .into_iter()
            .filter(|requirement| !depended.contains(requirement)),
    );

    if !surplus.is_empty() {
        return Err(CompileError::UnexpectedLayoutRequirement {
            unexpected: surplus.into_iter().collect(),
        });
    }

    Ok(())
}

/// Validate one assignment and return the layout it depends on.
///
/// The obligation's own plan and eligible carrier set are looked up
/// again here rather than passed in pre-paired, so a caller cannot hand
/// this check a mismatched pair.
///
/// # Errors
///
/// The typed reason this assignment is not one exact retained option of
/// its obligation, or lacks a layout requirement one of its selected
/// carriers depends on.
fn validate_assignment(
    plans: &[RelationCasePlan],
    eligibility: &[CarrierEligibility],
    assignment: &PlacementAssignment,
    stated: &BTreeSet<LayoutRequirement>,
) -> Result<BTreeSet<LayoutRequirement>, CompileError> {
    let key = assignment.key();
    let census = || CompileError::PlacementCensusMismatch {
        missing: vec![key.clone()],
        unexpected: Vec::new(),
    };
    let analysis = eligibility
        .iter()
        .find(|analysis| analysis.relation == key.relation && analysis.case == key.case)
        .ok_or_else(census)?;
    let plan = plans
        .iter()
        .find(|plan| plan.relation == key.relation && plan.case == key.case)
        .ok_or_else(census)?;

    let admissible = admissible_carriers(plan, analysis);
    let unpermitted = |carrier: &CarrierRole| CompileError::UnpermittedCarrierPlacement {
        relation: key.relation.clone(),
        case: key.case.clone(),
        carrier: carrier.clone(),
    };

    if assignment.carriers.is_empty() {
        return Err(CompileError::NoEligibleCarrier {
            relation: key.relation.clone(),
            case: key.case.clone(),
        });
    }

    // A repeated carrier role would vanish into the stable set
    // projection, so a raw invalid assignment and a valid one would
    // become indistinguishable downstream.
    let mut selected = BTreeSet::new();

    for placed in &assignment.carriers {
        if !selected.insert(&placed.carrier) {
            return Err(CompileError::DuplicatePlacedCarrier {
                relation: key.relation.clone(),
                case: key.case.clone(),
                carrier: placed.carrier.clone(),
            });
        }
    }

    // Minimality is a property of the whole assignment, not of any one
    // carrier, and it is the same policy §10.3 enumerates: an
    // exactly-one obligation carries one carrier; an every-member
    // obligation carries exactly one quantified alternative, since a
    // per-member role and a complete-family proof each discharge it
    // alone; an at-least-one obligation carries the accepted
    // inclusion-minimal singleton; and only deliberate duplication
    // carries more than one carrier — exactly the complete admitted set
    // the policy names, never a smaller or a larger one.
    let canonical = match analysis.multiplicity {
        CarrierMultiplicity::ExactlyOne
        | CarrierMultiplicity::EveryMember
        | CarrierMultiplicity::AtLeastOne => assignment.carriers.len() == 1,
        CarrierMultiplicity::DeliberateDuplication => {
            selected
                == admissible
                    .iter()
                    .map(|entry| &entry.carrier)
                    .collect::<BTreeSet<_>>()
        }
    };

    if !canonical {
        return Err(CompileError::NonCanonicalCarrierAssignment {
            relation: key.relation.clone(),
            case: key.case.clone(),
        });
    }

    let mut depended = BTreeSet::new();

    for placed in &assignment.carriers {
        let entry = admissible
            .iter()
            .find(|entry| entry.carrier == placed.carrier)
            .ok_or_else(|| unpermitted(&placed.carrier))?;

        if entry.quantification != placed.quantification {
            return Err(unpermitted(&placed.carrier));
        }

        for requirement in selected_carrier_requirements(analysis, entry) {
            if !stated.contains(&requirement) {
                return Err(CompileError::MissingLayoutRequirement {
                    relation: key.relation.clone(),
                    case: key.case.clone(),
                });
            }

            depended.insert(requirement);
        }
    }

    Ok(depended)
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
#[allow(dead_code)]
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
#[allow(dead_code)]
pub fn place_feasible_proof_plans(
    relations: &CompilerRelationAnalysis,
    candidates: &[ProofPlanCandidate],
    limits: PlacementSearchLimits,
) -> Result<PlacedProofPlans, CompileError> {
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
#[allow(dead_code)]
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

/// One retained carrier set of one obligation, with what it depends on.
#[derive(Clone, Debug, PartialEq, Eq)]
struct PlacementOption {
    carriers: Vec<PlacedCarrier>,
    layout: BTreeSet<LayoutRequirement>,
}

struct PlacementSearchState<'a> {
    keys: &'a [RelationCaseKey],
    options: &'a [Vec<PlacementOption>],
    plans: &'a [RelationCasePlan],
    eligibility: &'a [CarrierEligibility],
    limits: PlacementSearchLimits,
    search: &'a mut PlacementSearchReport,
    candidates: &'a mut Vec<PlacementCandidate>,
}

/// Depth-first product enumeration over the retained option lists.
fn visit_placement(
    state: &mut PlacementSearchState<'_>,
    chosen: &mut Vec<usize>,
    depth: usize,
) -> Result<(), CompileError> {
    admit_search_state(
        &mut state.search.states_visited,
        state.limits.maximum_states,
    )
    .map_err(|maximum| CompileError::PlacementSearchStateLimitExceeded { maximum })?;

    if depth == state.options.len() {
        record_search_event(&mut state.search.complete_assignments);

        let candidate = assemble_placement(state.keys, state.options, chosen);

        validate_placement(state.plans, state.eligibility, &candidate)?;

        if u64::try_from(state.candidates.len()).unwrap_or(u64::MAX)
            >= state.limits.maximum_candidates.get()
        {
            return Err(CompileError::PlacementCandidateLimitExceeded {
                maximum: state.limits.maximum_candidates.get(),
            });
        }

        state.candidates.push(candidate);
        return Ok(());
    }

    let width = state.options[depth].len();

    for index in 0..width {
        chosen.push(index);

        let result = visit_placement(state, chosen, depth + 1);

        chosen.pop();
        result?;
    }

    Ok(())
}

/// Materialize one complete choice as a placement candidate.
fn assemble_placement(
    keys: &[RelationCaseKey],
    options: &[Vec<PlacementOption>],
    chosen: &[usize],
) -> PlacementCandidate {
    let mut assignments = Vec::with_capacity(keys.len());
    let mut layout = BTreeSet::new();

    for (position, key) in keys.iter().enumerate() {
        let option = &options[position][chosen[position]];

        assignments.push(PlacementAssignment {
            relation: key.relation.clone(),
            case: key.case.clone(),
            carriers: option.carriers.clone(),
        });
        layout.extend(option.layout.iter().cloned());
    }

    assignments.sort();

    PlacementCandidate {
        assignments,
        layout_requirements: layout.into_iter().collect(),
    }
}

/// The retained carrier sets of one obligation (Guide-5 §10.3).
///
/// # Errors
///
/// The typed reason no admissible carrier remains.
fn retained_options(
    plan: &RelationCasePlan,
    analysis: &CarrierEligibility,
) -> Result<Vec<PlacementOption>, CompileError> {
    let admissible = admissible_carriers(plan, analysis);

    if admissible.is_empty() {
        return Err(no_admissible_carrier(plan, analysis));
    }

    let option = |entries: Vec<&EligibleCarrier>| PlacementOption {
        carriers: {
            let mut carriers = entries
                .iter()
                .map(|entry| PlacedCarrier {
                    carrier: entry.carrier.clone(),
                    quantification: entry.quantification,
                })
                .collect::<Vec<_>>();

            carriers.sort();
            carriers.dedup();
            carriers
        },
        layout: entries
            .iter()
            .flat_map(|entry| selected_carrier_requirements(analysis, entry))
            .collect(),
    };

    Ok(match analysis.multiplicity {
        // One carrier per assignment. An every-member obligation is not
        // an exception: one quantified per-member role discharges it,
        // and so does one typed complete-family proof, so both are
        // inclusion-minimal singletons rather than a set to accumulate.
        CarrierMultiplicity::ExactlyOne
        | CarrierMultiplicity::AtLeastOne
        | CarrierMultiplicity::EveryMember => admissible
            .into_iter()
            .map(|entry| option(vec![entry]))
            .collect(),

        // Duplication is deliberate, so the retained set is the one the
        // policy names — every admissible carrier — rather than an
        // arbitrary pair chosen because it happened to be smaller.
        CarrierMultiplicity::DeliberateDuplication => vec![option(admissible)],
    })
}

/// The eligible carriers one obligation actually permits.
///
/// Eligibility answers whether a carrier could receive the relation's
/// sources; admissibility answers whether the obligation's own scope,
/// multiplicity, and activation let that carrier discharge it.
fn admissible_carriers<'a>(
    plan: &RelationCasePlan,
    analysis: &'a CarrierEligibility,
) -> Vec<&'a EligibleCarrier> {
    let unconditional = plan.activation == ActivationCondition::Always;

    analysis
        .eligible
        .iter()
        .filter(|entry| scope_admits_carrier(analysis.scope, &entry.carrier))
        .filter(|entry| multiplicity_admits(analysis.multiplicity, entry.quantification))
        // An optional sponsor carrier may carry its own conditional
        // relations; it may never be the carrier of a relation that
        // holds whether or not the sponsor region exists.
        .filter(|entry| {
            !(unconditional && is_sponsor_region_carrier(&entry.carrier, GATED_ORDINARY_LBTC_ROLE))
        })
        .collect()
}

/// Whether one carrier role can discharge one semantic scope.
///
/// A family-global or transaction-global obligation needs a complete
/// carrier: a per-member role sees one member, so it can never establish
/// a property of the whole family or the whole transaction. The
/// backend-structural and external-evidence roles are not runtime
/// carriers at all.
fn scope_admits_carrier(scope: SemanticScope, carrier: &CarrierRole) -> bool {
    match carrier {
        CarrierRole::OperationGlobal { .. } => true,

        CarrierRole::InputFamilyCoordinator { object: family } => match scope {
            SemanticScope::MemberLocal { side, object }
            | SemanticScope::FamilyGlobal { side, object } => {
                side == TransactionSide::Input && *family == object
            }
            SemanticScope::TransactionSideGlobal { .. }
            | SemanticScope::ObjectFamilyGlobal { .. }
            | SemanticScope::TransactionGlobal => false,
        },

        CarrierRole::EveryInputFamilyMember { object: family } => match scope {
            SemanticScope::MemberLocal { side, object } => {
                side == TransactionSide::Input && *family == object
            }
            SemanticScope::FamilyGlobal { .. }
            | SemanticScope::TransactionSideGlobal { .. }
            | SemanticScope::ObjectFamilyGlobal { .. }
            | SemanticScope::TransactionGlobal => false,
        },

        CarrierRole::BackendStructural { .. } | CarrierRole::ExternalEvidence { .. } => false,
    }
}

/// Whether one quantification can discharge one multiplicity.
///
/// The every-member hard constraint reads the typed quantification, not
/// the carrier variant: a coordinator is admissible for an every-member
/// obligation only as the complete-family proof wave-2 types it as,
/// never as one carrier standing in for every member.
const fn multiplicity_admits(
    multiplicity: CarrierMultiplicity,
    quantification: CarrierQuantification,
) -> bool {
    match multiplicity {
        CarrierMultiplicity::EveryMember => matches!(
            quantification,
            CarrierQuantification::PerMember | CarrierQuantification::CompleteFamilyProof
        ),
        CarrierMultiplicity::ExactlyOne
        | CarrierMultiplicity::AtLeastOne
        | CarrierMultiplicity::DeliberateDuplication => {
            matches!(quantification, CarrierQuantification::Single)
        }
    }
}

/// The typed reason one obligation retained no admissible carrier.
///
/// The scope defect is reported first where both apply: a global
/// relation offered only local carriers is a stronger statement about
/// the analysis than the activation mismatch it also implies.
fn no_admissible_carrier(plan: &RelationCasePlan, analysis: &CarrierEligibility) -> CompileError {
    let relation = analysis.relation.clone();
    let case = analysis.case.clone();
    let local = |carrier: &CarrierRole| {
        matches!(
            carrier,
            CarrierRole::EveryInputFamilyMember { .. } | CarrierRole::InputFamilyCoordinator { .. }
        )
    };

    if analysis.eligible.is_empty() {
        return CompileError::NoEligibleCarrier { relation, case };
    }

    if !matches!(analysis.scope, SemanticScope::MemberLocal { .. })
        && analysis.eligible.iter().all(|entry| local(&entry.carrier))
    {
        return CompileError::GlobalRelationHasOnlyLocalCarrier { relation, case };
    }

    if plan.activation == ActivationCondition::Always
        && analysis
            .eligible
            .iter()
            .all(|entry| is_sponsor_region_carrier(&entry.carrier, GATED_ORDINARY_LBTC_ROLE))
    {
        return CompileError::UnconditionalRelationOnOptionalCarrier { relation, case };
    }

    CompileError::NoEligibleCarrier { relation, case }
}
