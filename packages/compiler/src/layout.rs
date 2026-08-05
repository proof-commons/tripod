//! Target-independent layout requirements (Guide-5 Tranche D).
//!
//! Placement and layout are different questions. Placement asks which
//! abstract carrier role is responsible for a relation; a layout
//! requirement asks what authenticated structure and fact routing must
//! exist for that carrier to see its operands at all. A relation having
//! a carrier does not prove the carrier can read the facts it needs,
//! and a layout requirement existing here does not prove any backend
//! implemented it — these are obligations recorded against a future
//! backend, never completed evidence.
//!
//! Every requirement cites the relation that owns the semantics rather
//! than restating it. A family census requirement names the cardinality
//! relation and the family; it does not copy the minimum, the maximum,
//! or the bound, because duplicating a semantic value into a
//! compiler-owned field creates a second source of truth that can drift
//! from the first.
//!
//! Nothing here assigns a concrete range, index, or position. The
//! requirements say "authenticated bounded family", "complete family
//! membership", "canonical coordinator", and "disjoint protocol and
//! sponsor regions"; they never say which inputs, which output, which
//! witness item, which tapleaf, or which stack slot. Those belong below
//! the compiler boundary.
//!
//! Sponsor erasure survives the stage intact. Sponsor requirements may
//! name family membership, owner authorization, exact reference
//! membership, region disjointness, and envelope multiplicity; no
//! requirement may name a sponsor amount, sponsor positivity, or a
//! public sponsor sum, and the census validator rejects one that does.
//! Whole-transaction conservation stays external evidence and receives
//! no layout requirement at all.

// The analysis stages have no non-test consumer until the P2-012
// analyzed program; unit tests exercise them until then. Remove with
// the first real consumer.
#![allow(dead_code)]

use std::collections::BTreeSet;

use architecture::{ObjectId, OperationId};
use realization::{Relation, RelationId, RepresentationMode, TransactionSide};

use crate::{
    CompileError,
    carrier::{
        CarrierAvailability, CarrierEligibility, CarrierQuantification, CarrierRole,
        EligibleCarrier,
    },
    case::ExecutionCaseId,
    placement::{BackendStructuralRequirement, RelationCasePlan, SemanticScope},
    relation::CompilerRelationAnalysis,
    source::{SourceRequirement, is_sponsor_amount_operand},
};

/// One target-independent obligation on a future backend.
///
/// The relation named by each variant remains the semantic owner; the
/// requirement states only what the backend must expose.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LayoutRequirement {
    /// The bounded family census on one side must be authenticated.
    AuthenticateFamilyCensus {
        relation: RelationId,
        side: TransactionSide,
        object: ObjectId,
    },

    /// Family membership on one side must be complete, and the
    /// protocol and sponsor regions must be disjoint.
    CompleteAndDisjointFamilies {
        relation: RelationId,
        side: TransactionSide,
    },

    /// The operation must have a canonical coordinator at this anchor.
    CanonicalCoordinator {
        operation: OperationId,
        anchor: CarrierRole,
    },

    /// One source row must be routed to one carrier.
    MakeSourceAvailable {
        relation: RelationId,
        case: ExecutionCaseId,
        carrier: CarrierRole,
        source: SourceRequirement,
    },

    /// The optional sponsor region must be isolated: claimed exactly
    /// once, disjoint from the protocol region, owner-authorized, and
    /// — in the sponsorless case — provably absent.
    IsolateSponsorRegion {
        relation: RelationId,
        case: ExecutionCaseId,
    },

    /// The selected representation must be encoded and authenticated.
    EnforceRepresentation {
        relation: RelationId,
        object: ObjectId,
        representation: RepresentationMode,
    },

    /// The operation path must contain no owner or operator secret
    /// gate.
    SecretFreeOperationPath {
        relation: RelationId,
        case: ExecutionCaseId,
    },
}

impl LayoutRequirement {
    /// The relation that owns this requirement's semantics, if one
    /// does.
    ///
    /// A canonical-coordinator requirement is owned by its operation
    /// rather than by any single relation, so it has none.
    #[must_use]
    pub const fn relation(&self) -> Option<&RelationId> {
        match self {
            Self::AuthenticateFamilyCensus { relation, .. }
            | Self::CompleteAndDisjointFamilies { relation, .. }
            | Self::MakeSourceAvailable { relation, .. }
            | Self::IsolateSponsorRegion { relation, .. }
            | Self::EnforceRepresentation { relation, .. }
            | Self::SecretFreeOperationPath { relation, .. } => Some(relation),
            Self::CanonicalCoordinator { .. } => None,
        }
    }
}

/// Derive every layout requirement of one operation's analysis.
///
/// The input is the relation-case plans and the eligible carrier sets:
/// a layout requirement exists because some eligible carrier depends on
/// it, or because a relation's own scope demands authenticated
/// structure. The result is canonically ordered and free of duplicates.
///
/// # Errors
///
/// [`CompileError::SponsorValueRead`] when a derived requirement would
/// name an erased sponsor amount.
pub fn derive_layout_requirements(
    relations: &CompilerRelationAnalysis,
    plans: &[RelationCasePlan],
    eligibility: &[CarrierEligibility],
) -> Result<Vec<LayoutRequirement>, CompileError> {
    let mut requirements = BTreeSet::new();

    for analysis in eligibility {
        for entry in &analysis.eligible {
            carrier_requirements(analysis, entry, &mut requirements);
        }

        scope_requirements(relations, analysis, &mut requirements);
    }

    for plan in plans {
        structural_requirements(plan, &mut requirements);
    }

    let requirements = requirements.into_iter().collect::<Vec<_>>();

    for requirement in &requirements {
        if names_sponsor_amount(requirement) {
            return Err(CompileError::SponsorValueRead);
        }
    }

    Ok(requirements)
}

/// Validate that every dependency of the analysis has a requirement.
///
/// Completeness is checked against the eligible carriers and the plans
/// rather than against the derivation that produced the requirements,
/// so a derivation that silently dropped an obligation is caught here
/// instead of agreeing with itself.
///
/// # Errors
///
/// [`CompileError::MissingLayoutRequirement`] when a carrier dependency
/// or a structural obligation has no requirement;
/// [`CompileError::SponsorValueRead`] when a requirement names an
/// erased sponsor amount.
pub fn validate_layout_census(
    plans: &[RelationCasePlan],
    eligibility: &[CarrierEligibility],
    requirements: &[LayoutRequirement],
) -> Result<(), CompileError> {
    let present = requirements.iter().collect::<BTreeSet<_>>();

    for requirement in requirements {
        if names_sponsor_amount(requirement) {
            return Err(CompileError::SponsorValueRead);
        }
    }

    for analysis in eligibility {
        let mut required = BTreeSet::new();

        for entry in &analysis.eligible {
            carrier_requirements(analysis, entry, &mut required);
        }

        for requirement in &required {
            if !present.contains(requirement) {
                return Err(CompileError::MissingLayoutRequirement {
                    relation: analysis.relation.clone(),
                    case: analysis.case.clone(),
                });
            }
        }
    }

    for plan in plans {
        let mut required = BTreeSet::new();
        structural_requirements(plan, &mut required);

        for requirement in &required {
            if !present.contains(requirement) {
                return Err(CompileError::MissingLayoutRequirement {
                    relation: plan.relation.clone(),
                    case: plan.case.clone(),
                });
            }
        }
    }

    Ok(())
}

/// Derive and validate one operation's layout requirements in one step.
///
/// # Errors
///
/// Any failure of [`derive_layout_requirements`] or
/// [`validate_layout_census`].
pub fn layout_requirements(
    relations: &CompilerRelationAnalysis,
    plans: &[RelationCasePlan],
    eligibility: &[CarrierEligibility],
) -> Result<Vec<LayoutRequirement>, CompileError> {
    let requirements = derive_layout_requirements(relations, plans, eligibility)?;

    validate_layout_census(plans, eligibility, &requirements)?;
    Ok(requirements)
}

/// True for a requirement that would name an erased sponsor amount.
///
/// Membership, authorization, disjointness, and multiplicity over the
/// sponsor family remain expressible; the individual amount does not.
#[must_use]
pub const fn names_sponsor_amount(requirement: &LayoutRequirement) -> bool {
    match requirement {
        LayoutRequirement::MakeSourceAvailable { source, .. } => {
            is_sponsor_amount_operand(source.operand.role())
        }
        LayoutRequirement::AuthenticateFamilyCensus { .. }
        | LayoutRequirement::CompleteAndDisjointFamilies { .. }
        | LayoutRequirement::CanonicalCoordinator { .. }
        | LayoutRequirement::IsolateSponsorRegion { .. }
        | LayoutRequirement::EnforceRepresentation { .. }
        | LayoutRequirement::SecretFreeOperationPath { .. } => false,
    }
}

/// The layout requirements one *selected* carrier depends on.
///
/// The exact placement search records these against the placement that
/// selected the carrier, while the operation-wide census above is their
/// union over every *eligible* carrier. Both read the same derivation,
/// so a placement can never depend on a requirement the census omits.
#[must_use]
pub fn selected_carrier_requirements(
    analysis: &CarrierEligibility,
    entry: &EligibleCarrier,
) -> BTreeSet<LayoutRequirement> {
    let mut requirements = BTreeSet::new();

    carrier_requirements(analysis, entry, &mut requirements);
    requirements
}

/// What one eligible carrier depends on.
fn carrier_requirements(
    analysis: &CarrierEligibility,
    entry: &EligibleCarrier,
    requirements: &mut BTreeSet<LayoutRequirement>,
) {
    let relation = analysis.relation.clone();
    let operation = relation.operation();

    if is_coordinator(&entry.carrier) {
        requirements.insert(LayoutRequirement::CanonicalCoordinator {
            operation,
            anchor: entry.carrier.clone(),
        });
    }

    // A single carrier standing for an every-member obligation is
    // admissible only as a proof about the complete authenticated
    // family, so the completeness it relies on is stated rather than
    // assumed.
    if entry.quantification == CarrierQuantification::CompleteFamilyProof {
        if let SemanticScope::MemberLocal { side, object } = analysis.scope {
            requirements.insert(LayoutRequirement::AuthenticateFamilyCensus {
                relation: relation.clone(),
                side,
                object,
            });
            requirements.insert(LayoutRequirement::CompleteAndDisjointFamilies {
                relation: relation.clone(),
                side,
            });
        }
    }

    for routing in &entry.routings {
        match routing.availability {
            CarrierAvailability::Intrinsic => {}

            CarrierAvailability::LayoutProvided => {
                requirements.insert(LayoutRequirement::MakeSourceAvailable {
                    relation: relation.clone(),
                    case: analysis.case.clone(),
                    carrier: entry.carrier.clone(),
                    source: routing.source.clone(),
                });
            }

            // The fact stays inside the sponsor region; what the
            // carrier needs is that the region is isolated, not the
            // value.
            CarrierAvailability::SponsorRegionConfined => {
                requirements.insert(LayoutRequirement::IsolateSponsorRegion {
                    relation: relation.clone(),
                    case: analysis.case.clone(),
                });
            }
        }
    }
}

/// What one relation's semantic scope demands of the structure.
fn scope_requirements(
    relations: &CompilerRelationAnalysis,
    analysis: &CarrierEligibility,
    requirements: &mut BTreeSet<LayoutRequirement>,
) {
    let relation = analysis.relation.clone();

    match analysis.scope {
        SemanticScope::MemberLocal { .. } => {}

        SemanticScope::FamilyGlobal { side, object } => {
            requirements.insert(LayoutRequirement::AuthenticateFamilyCensus {
                relation,
                side,
                object,
            });
        }

        SemanticScope::TransactionSideGlobal { side } => {
            requirements.insert(LayoutRequirement::CompleteAndDisjointFamilies { relation, side });
        }

        SemanticScope::ObjectFamilyGlobal { object } => {
            for side in [TransactionSide::Input, TransactionSide::Output] {
                requirements.insert(LayoutRequirement::AuthenticateFamilyCensus {
                    relation: relation.clone(),
                    side,
                    object,
                });
            }
        }

        SemanticScope::TransactionGlobal => {
            global_relation_requirements(relations, analysis, requirements);
        }
    }
}

/// What one transaction-global relation demands, by relation variant.
fn global_relation_requirements(
    relations: &CompilerRelationAnalysis,
    analysis: &CarrierEligibility,
    requirements: &mut BTreeSet<LayoutRequirement>,
) {
    let relation = analysis.relation.clone();
    let Some(declaration) = declared_relation(relations, &analysis.relation) else {
        return;
    };

    match declaration {
        // Conservation is a claim about complete family totals on both
        // sides, so a per-member fact can never establish it.
        Relation::AmountConservation {
            input_objects,
            output_objects,
            ..
        } => {
            let families = input_objects
                .iter()
                .map(|object| (TransactionSide::Input, *object))
                .chain(
                    output_objects
                        .iter()
                        .map(|object| (TransactionSide::Output, *object)),
                );

            for (side, object) in families {
                requirements.insert(LayoutRequirement::AuthenticateFamilyCensus {
                    relation: relation.clone(),
                    side,
                    object,
                });
            }

            for side in [TransactionSide::Input, TransactionSide::Output] {
                requirements.insert(LayoutRequirement::CompleteAndDisjointFamilies {
                    relation: relation.clone(),
                    side,
                });
            }
        }

        // Both sponsor cases carry the isolation requirement: with no
        // sponsor region the global relation still has to establish
        // that none exists.
        Relation::SponsorIsolation | Relation::SponsorEnvelopeMultiplicity { .. } => {
            requirements.insert(LayoutRequirement::IsolateSponsorRegion {
                relation: relation.clone(),
                case: analysis.case.clone(),
            });

            for side in [TransactionSide::Input, TransactionSide::Output] {
                requirements.insert(LayoutRequirement::CompleteAndDisjointFamilies {
                    relation: relation.clone(),
                    side,
                });
            }
        }

        Relation::Cardinality { .. }
        | Relation::AllowedObjectFamilies { .. }
        | Relation::Recognition { .. }
        | Relation::OwnerAuthorization { .. }
        | Relation::PermissionlessAuthorization
        | Relation::RootPolicy { .. }
        | Relation::ProjectionPolicy { .. }
        | Relation::CanonicalDeltaPolicy { .. }
        | Relation::OpenFlowPolicy { .. }
        | Relation::Constructibility { .. }
        | Relation::Representation { .. }
        | Relation::LifecycleExit { .. }
        | Relation::ExpressionPredicate { .. }
        | Relation::SubstrateConservation { .. } => {}
    }
}

/// The layout side of one relation's backend-structural obligations.
///
/// Compiler-static requirements produce none: a property the compiler
/// validated from typed input is not a structure the backend must
/// expose. External evidence produces none either — a runtime or layout
/// obligation standing in for it would misreport unfinished evidence as
/// discharged.
fn structural_requirements(
    plan: &RelationCasePlan,
    requirements: &mut BTreeSet<LayoutRequirement>,
) {
    for requirement in &plan.structural_requirements {
        match requirement {
            BackendStructuralRequirement::EncodeAndAuthenticateRepresentation {
                object,
                representation,
            } => {
                requirements.insert(LayoutRequirement::EnforceRepresentation {
                    relation: plan.relation.clone(),
                    object: *object,
                    representation: *representation,
                });
            }

            BackendStructuralRequirement::SecretFreeOperationPath { .. } => {
                requirements.insert(LayoutRequirement::SecretFreeOperationPath {
                    relation: plan.relation.clone(),
                    case: plan.case.clone(),
                });
            }

            // The lifecycle exit is retained in the emitted bundle's
            // own census rather than in transaction structure, so it
            // states no layout obligation here.
            BackendStructuralRequirement::RetainRequiredLifecycleExit { .. } => {}
        }
    }
}

/// Whether one carrier coordinates rather than executing per member.
const fn is_coordinator(carrier: &CarrierRole) -> bool {
    matches!(
        carrier,
        CarrierRole::OperationGlobal { .. } | CarrierRole::InputFamilyCoordinator { .. }
    )
}

/// The declared relation behind one relation ID.
fn declared_relation<'a>(
    relations: &'a CompilerRelationAnalysis,
    relation: &RelationId,
) -> Option<&'a Relation> {
    relations
        .graph
        .node_weights()
        .find(|node| &node.source.id == relation)
        .map(|node| &node.source.relation)
}
