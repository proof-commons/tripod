//! Abstract carrier roles and carrier eligibility (Guide-5 Tranche C).
//!
//! A carrier is a semantic enforcement role, never a transaction
//! position: nothing here names a graph index, an input or output
//! ordinal, a tapleaf, a stack or witness slot, a target program, or a
//! transaction slot, and no later stage may add one without leaving the
//! compiler boundary.
//!
//! Two properties do the real work. First, a global coordinator is
//! never conjured: its anchors are derived from the input families the
//! typed cardinality declarations guarantee are present, so an
//! operation whose every input family is optional has no coordinator
//! rather than a magical one, and the optional sponsor family can never
//! become the sole carrier of an unconditional relation. Second,
//! eligibility is decided by availability, not by preference: a carrier
//! is eligible for a relation exactly when every active source row of
//! that relation is intrinsically available there or can be routed
//! there by an explicit layout requirement. Owner authorization is
//! therefore not merely *preferred* per member — an owner witness is
//! tied to its authorized input family, so no coordinator is eligible
//! for it at all, and no coordinator placement can silently replace
//! all-owner authorization.
//!
//! Input recognition is the one member-local obligation a coordinator
//! may discharge, because its sources are public and a layout can route
//! the complete authenticated family census. That alternative is typed
//! as [`CarrierQuantification::CompleteFamilyProof`] rather than left
//! implicit, so a later stage cannot mistake it for an ordinary single
//! carrier standing in for every member.

// The item-level allowances below are declared carrier vocabulary the
// two pilots do not exercise, and query helpers over eligibility that
// the analysis does not need to ask. Deleting them would narrow the
// typed model to the pilots rather than to what the realization
// declares.

use std::collections::BTreeSet;

use architecture::{ObjectId, OperationId};
use realization::{
    AvailabilityClass, ExternalEvidenceRequirement, ObservedSide, Relation, RelationId,
    TransactionSide,
};

use crate::{
    CompileError,
    case::{ExecutionCaseId, SponsorCase, is_sponsor_region_family},
    placement::{
        ActivationCondition, CarrierMultiplicity, RelationCasePlan, RuntimePlacementRequirement,
        SemanticScope,
    },
    relation::CompilerRelationAnalysis,
    source::{OperandId, OperandRole, RequiredSourceKind, SourceRequirement},
    sponsor_region::{GATED_ORDINARY_LBTC_ROLE, OrdinaryLbtcRole},
};

/// An abstract semantic enforcement role.
///
/// Target-independent by construction: every variant is identified by
/// typed semantic values only.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CarrierRole {
    /// Every authenticated member of one bounded input family carries
    /// the obligation. Quantified, never a runtime member enumeration.
    EveryInputFamilyMember { object: ObjectId },

    /// One member of one input family coordinates for that family.
    InputFamilyCoordinator { object: ObjectId },

    /// The operation's global coordinator, anchored in an input family
    /// the typed declarations guarantee is present.
    OperationGlobal {
        operation: OperationId,
        anchor: ObjectId,
    },

    /// The emitted bundle or ABI, for obligations that are structural
    /// rather than predicates any carrier evaluates.
    BackendStructural { operation: OperationId },

    /// The external-evidence boundary. Not a runtime carrier: an
    /// external relation reaching a runtime carrier would be a defect,
    /// not a discharge.
    ExternalEvidence {
        requirement: ExternalEvidenceRequirement,
    },
}

/// How one eligible carrier discharges its relation's multiplicity.
///
/// A single coordinator proving a property of a complete authenticated
/// family is a different obligation from one carrier standing in for
/// every member, so the two are typed apart rather than conflated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CarrierQuantification {
    /// The obligation executes once per authenticated member.
    PerMember,
    /// One carrier proves the obligation for the complete authenticated
    /// family at once, and depends on the family census layout
    /// requirements that make that completeness real.
    CompleteFamilyProof,
    /// One carrier discharges one obligation that is not quantified
    /// over family members.
    Single,
}

/// Where one active source row comes from at one carrier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CarrierAvailability {
    /// Available at the carrier with no layout obligation.
    Intrinsic,
    /// Available only if an explicit layout requirement routes it.
    LayoutProvided,
    /// Discharged inside the isolated optional sponsor region and
    /// reaching this carrier only as region-level evidence — never as
    /// an exact sponsor value.
    SponsorRegionConfined,
}

/// One active source row and how it reaches one carrier.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceRouting {
    pub source: SourceRequirement,
    pub availability: CarrierAvailability,
}

/// One carrier that can discharge one relation in one case.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EligibleCarrier {
    pub carrier: CarrierRole,
    pub quantification: CarrierQuantification,
    /// One routing per active source row, canonically sorted.
    pub routings: Vec<SourceRouting>,
}

impl EligibleCarrier {
    /// The source rows this carrier can only receive through layout.
    #[must_use]
    #[allow(dead_code)]
    pub fn layout_provided(&self) -> Vec<&SourceRouting> {
        self.routings
            .iter()
            .filter(|routing| routing.availability == CarrierAvailability::LayoutProvided)
            .collect()
    }

    /// Whether any source row of this carrier is confined to the
    /// optional sponsor region.
    #[must_use]
    #[allow(dead_code)]
    pub fn confines_sponsor_region(&self) -> bool {
        self.routings
            .iter()
            .any(|routing| routing.availability == CarrierAvailability::SponsorRegionConfined)
    }
}

/// Why one candidate carrier cannot discharge one relation.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CarrierIneligibility {
    /// The carrier is anchored in an optional family while the relation
    /// holds unconditionally.
    OptionalCarrierForUnconditionalRelation,
    /// One active source row is available neither intrinsically nor
    /// through any layout requirement.
    SourceUnavailable { operand: OperandId },
}

/// One rejected candidate carrier and its typed reason.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IneligibleCarrier {
    pub carrier: CarrierRole,
    pub reason: CarrierIneligibility,
}

/// The complete eligible carrier set of one runtime requirement.
///
/// This value is the Guide-5 Tranche C deliverable: for one relation in
/// one case it states which abstract roles could enforce it and what
/// each one depends on. It selects nothing — the exact placement search
/// consumes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CarrierEligibility {
    pub relation: RelationId,
    pub case: ExecutionCaseId,
    pub scope: SemanticScope,
    pub multiplicity: CarrierMultiplicity,
    /// Non-empty and canonically sorted.
    pub eligible: Vec<EligibleCarrier>,
    /// Rejected candidates, canonically sorted. Retained so a later
    /// stage reports why a carrier is absent rather than guessing.
    pub ineligible: Vec<IneligibleCarrier>,
}

/// The input families of one operation, with mandatory presence.
///
/// Presence is read from the typed cardinality minimum, never from an
/// operation or object name.
#[must_use]
pub fn input_families(
    relations: &CompilerRelationAnalysis,
    operation: OperationId,
) -> BTreeSet<ObjectId> {
    declared_input_families(relations, operation, false)
}

/// The input families the typed declarations guarantee are present.
#[must_use]
pub fn mandatory_input_families(
    relations: &CompilerRelationAnalysis,
    operation: OperationId,
) -> BTreeSet<ObjectId> {
    declared_input_families(relations, operation, true)
}

/// The families that may anchor the operation's global coordinator.
///
/// Exactly the mandatory input families: an optional family cannot
/// anchor a coordinator, because a case in which it is absent would
/// leave the operation's unconditional relations with no carrier at
/// all.
#[must_use]
pub fn coordinator_anchors(
    relations: &CompilerRelationAnalysis,
    operation: OperationId,
) -> BTreeSet<ObjectId> {
    mandatory_input_families(relations, operation)
}

/// The operation's global coordinator roles, one per anchor.
#[must_use]
pub fn coordinator_carriers(
    relations: &CompilerRelationAnalysis,
    operation: OperationId,
) -> BTreeSet<CarrierRole> {
    coordinator_anchors(relations, operation)
        .into_iter()
        .map(|anchor| CarrierRole::OperationGlobal { operation, anchor })
        .collect()
}

/// The carrier role a backend-structural obligation belongs to.
#[must_use]
#[allow(dead_code)]
pub const fn structural_carrier(operation: OperationId) -> CarrierRole {
    CarrierRole::BackendStructural { operation }
}

/// The carrier role one external-evidence requirement belongs to.
#[must_use]
#[allow(dead_code)]
pub const fn external_carrier(requirement: ExternalEvidenceRequirement) -> CarrierRole {
    CarrierRole::ExternalEvidence { requirement }
}

/// Whether one carrier lies inside the optional sponsor region.
///
/// The operation's ordinary-L-BTC role decides this, not the object
/// family alone: the same family carries protocol references in
/// operations whose declared flows claim it, and a protocol-region
/// carrier is not optional.
#[must_use]
pub const fn is_sponsor_region_carrier(
    carrier: &CarrierRole,
    ordinary_lbtc: OrdinaryLbtcRole,
) -> bool {
    match carrier {
        CarrierRole::EveryInputFamilyMember { object }
        | CarrierRole::InputFamilyCoordinator { object } => {
            is_sponsor_region_family(*object, ordinary_lbtc)
        }
        CarrierRole::OperationGlobal { .. }
        | CarrierRole::BackendStructural { .. }
        | CarrierRole::ExternalEvidence { .. } => false,
    }
}

/// The candidate carrier roles of one runtime requirement.
///
/// Candidacy is structural — it answers which roles the requirement's
/// semantic scope admits at all. Availability then decides which of
/// them are eligible.
///
/// Output families are deliberately absent: the compiler does not
/// assume an output object's program executes while that output is
/// created, so output recognition and closure are coordinator
/// obligations rather than output-carried ones.
#[must_use]
pub fn candidate_carriers(
    relations: &CompilerRelationAnalysis,
    requirement: &RuntimePlacementRequirement,
) -> BTreeSet<CarrierRole> {
    let operation = requirement.relation.operation();
    let families = input_families(relations, operation);
    let mut candidates = coordinator_carriers(relations, operation);

    match requirement.scope {
        SemanticScope::MemberLocal { side, object } => {
            if side == TransactionSide::Input && families.contains(&object) {
                candidates.insert(CarrierRole::EveryInputFamilyMember { object });
                candidates.insert(CarrierRole::InputFamilyCoordinator { object });
            }
        }

        SemanticScope::FamilyGlobal { side, object } => {
            if side == TransactionSide::Input && families.contains(&object) {
                candidates.insert(CarrierRole::InputFamilyCoordinator { object });
            }
        }

        SemanticScope::TransactionSideGlobal { .. }
        | SemanticScope::ObjectFamilyGlobal { .. }
        | SemanticScope::TransactionGlobal => {}
    }

    candidates
}

/// The eligible carrier set of one active runtime requirement.
///
/// # Errors
///
/// [`CompileError::MissingCanonicalCoordinator`] when the operation has
/// no mandatory input family to anchor a coordinator the requirement's
/// scope needs; [`CompileError::UnconditionalRelationOnOptionalCarrier`]
/// when every candidate of an unconditionally active relation is
/// anchored in an optional family;
/// [`CompileError::NoEligibleCarrier`] when candidates exist but none
/// can receive every active source row.
pub fn carrier_eligibility(
    relations: &CompilerRelationAnalysis,
    plan: &RelationCasePlan,
    requirement: &RuntimePlacementRequirement,
) -> Result<CarrierEligibility, CompileError> {
    let operation = requirement.relation.operation();
    let candidates = candidate_carriers(relations, requirement);

    if candidates.is_empty() {
        return Err(CompileError::MissingCanonicalCoordinator { operation });
    }

    let mandatory = mandatory_input_families(relations, operation);
    let unconditional = plan.activation == ActivationCondition::Always;
    let mut eligible = Vec::new();
    let mut ineligible = Vec::new();

    for carrier in candidates {
        match classify_carrier(&mandatory, unconditional, plan, requirement, &carrier) {
            Ok(entry) => eligible.push(entry),
            Err(reason) => ineligible.push(IneligibleCarrier { carrier, reason }),
        }
    }

    eligible.sort();
    ineligible.sort();

    if eligible.is_empty() {
        let optional_only = ineligible.iter().all(|entry| {
            entry.reason == CarrierIneligibility::OptionalCarrierForUnconditionalRelation
        });

        return Err(if optional_only {
            CompileError::UnconditionalRelationOnOptionalCarrier {
                relation: requirement.relation.clone(),
                case: plan.case.clone(),
            }
        } else {
            CompileError::NoEligibleCarrier {
                relation: requirement.relation.clone(),
                case: plan.case.clone(),
            }
        });
    }

    Ok(CarrierEligibility {
        relation: requirement.relation.clone(),
        case: plan.case.clone(),
        scope: requirement.scope,
        multiplicity: requirement.multiplicity,
        eligible,
        ineligible,
    })
}

/// The eligible carrier sets of every active runtime requirement.
///
/// Relations discharged statically, structurally, or by external
/// evidence contribute nothing here — deliberately: they carry no
/// runtime requirement, so assigning them a runtime carrier would
/// misreport an obligation as target execution.
///
/// # Errors
///
/// Any failure of [`carrier_eligibility`].
pub fn relation_case_eligibility(
    relations: &CompilerRelationAnalysis,
    plans: &[RelationCasePlan],
) -> Result<Vec<CarrierEligibility>, CompileError> {
    let mut analyses = Vec::new();

    for plan in plans {
        for requirement in &plan.runtime_requirements {
            analyses.push(carrier_eligibility(relations, plan, requirement)?);
        }
    }

    analyses
        .sort_by(|left, right| (&left.relation, &left.case).cmp(&(&right.relation, &right.case)));
    Ok(analyses)
}

/// Classify one candidate carrier of one requirement.
fn classify_carrier(
    mandatory: &BTreeSet<ObjectId>,
    unconditional: bool,
    plan: &RelationCasePlan,
    requirement: &RuntimePlacementRequirement,
    carrier: &CarrierRole,
) -> Result<EligibleCarrier, CarrierIneligibility> {
    if unconditional && is_optional_carrier(mandatory, carrier) {
        return Err(CarrierIneligibility::OptionalCarrierForUnconditionalRelation);
    }

    let mut routings = Vec::with_capacity(requirement.sources.len());

    for source in &requirement.sources {
        let availability =
            carrier_availability(carrier, plan.case.sponsor, source).ok_or_else(|| {
                CarrierIneligibility::SourceUnavailable {
                    operand: source.operand.clone(),
                }
            })?;

        routings.push(SourceRouting {
            source: source.clone(),
            availability,
        });
    }

    routings.sort();

    Ok(EligibleCarrier {
        carrier: carrier.clone(),
        quantification: quantification(requirement, carrier),
        routings,
    })
}

/// How one carrier discharges one requirement's multiplicity.
const fn quantification(
    requirement: &RuntimePlacementRequirement,
    carrier: &CarrierRole,
) -> CarrierQuantification {
    match requirement.multiplicity {
        CarrierMultiplicity::EveryMember => match carrier {
            CarrierRole::EveryInputFamilyMember { .. } => CarrierQuantification::PerMember,
            // A single carrier may stand for an every-member obligation
            // only as a proof about the complete authenticated family,
            // and that completeness is a layout obligation rather than
            // an assumption.
            _ => CarrierQuantification::CompleteFamilyProof,
        },

        CarrierMultiplicity::ExactlyOne
        | CarrierMultiplicity::AtLeastOne
        | CarrierMultiplicity::DeliberateDuplication => CarrierQuantification::Single,
    }
}

/// Whether one carrier is anchored in a family that may be absent.
fn is_optional_carrier(mandatory: &BTreeSet<ObjectId>, carrier: &CarrierRole) -> bool {
    match carrier {
        CarrierRole::EveryInputFamilyMember { object }
        | CarrierRole::InputFamilyCoordinator { object } => !mandatory.contains(object),
        // Coordinator anchors are mandatory families by construction.
        CarrierRole::OperationGlobal { .. }
        | CarrierRole::BackendStructural { .. }
        | CarrierRole::ExternalEvidence { .. } => false,
    }
}

/// Where one source row comes from at one carrier, if at all.
///
/// The availability class is load-bearing, not advisory: an owner
/// witness stays tied to its authorized input family, a sponsor-local
/// witness stays inside the sponsor region, and the operator and
/// refund-key classes are unavailable in the current pilots.
fn carrier_availability(
    carrier: &CarrierRole,
    sponsor: SponsorCase,
    source: &SourceRequirement,
) -> Option<CarrierAvailability> {
    match source.availability {
        AvailabilityClass::Public => public_availability(carrier, source),

        // No layout may route an owner witness to a coordinator: doing
        // so would replace all-owner authorization with one member's
        // signature. Only an explicitly modeled all-owner carrier could
        // be eligible, and none is modeled.
        AvailabilityClass::InputOwners { object } => match carrier {
            CarrierRole::EveryInputFamilyMember { object: family } if *family == object => {
                Some(CarrierAvailability::Intrinsic)
            }
            _ => None,
        },

        AvailabilityClass::SponsorLocal => {
            if sponsor != SponsorCase::Present {
                return None;
            }

            Some(
                if is_sponsor_region_carrier(carrier, GATED_ORDINARY_LBTC_ROLE) {
                    CarrierAvailability::Intrinsic
                } else {
                    // The protocol coordinator learns that the region is
                    // authorized, never the sponsor's exact value.
                    CarrierAvailability::SponsorRegionConfined
                },
            )
        }

        AvailabilityClass::Operator
        | AvailabilityClass::RefundKey { .. }
        | AvailabilityClass::ClientOwners { .. } => None,
    }
}

/// Availability of one public source row at one carrier.
fn public_availability(
    carrier: &CarrierRole,
    source: &SourceRequirement,
) -> Option<CarrierAvailability> {
    match source.source {
        // Public architecture and construction facts are available
        // wherever the operation executes.
        RequiredSourceKind::RuntimeArchitectureBound
        | RequiredSourceKind::PublicConstructionData
        | RequiredSourceKind::DerivedExpression => Some(CarrierAvailability::Intrinsic),

        // A member has its own object facts; anyone else needs them
        // routed.
        RequiredSourceKind::AuthenticatedInputObject => {
            Some(if own_input_object(carrier, source.operand.role()) {
                CarrierAvailability::Intrinsic
            } else {
                CarrierAvailability::LayoutProvided
            })
        }

        RequiredSourceKind::AuthenticatedOutputObject
        | RequiredSourceKind::AuthenticatedFamilyCensus
        | RequiredSourceKind::AuthenticatedConsensusValue
        | RequiredSourceKind::AuthenticatedCommitmentRelation
        | RequiredSourceKind::AuthenticatedTransitionCertificate => {
            Some(CarrierAvailability::LayoutProvided)
        }

        // A public witness kind is a classification defect rather than
        // a fact any carrier may read, and external evidence is
        // available at the external boundary only.
        RequiredSourceKind::InputOwnerWitness
        | RequiredSourceKind::OperatorWitness
        | RequiredSourceKind::RefundKeyWitness
        | RequiredSourceKind::SponsorLocalWitness
        | RequiredSourceKind::ExternalEvidence => None,
    }
}

/// Whether one carrier is a member of the family the operand names.
fn own_input_object(carrier: &CarrierRole, role: &OperandRole) -> bool {
    matches!(
        (carrier, role),
        (
            CarrierRole::EveryInputFamilyMember { object: family },
            OperandRole::ObjectFamilyMembers {
                side: TransactionSide::Input,
                object,
            },
        ) if family == object
    )
}

/// The operation's declared input families, optionally restricted to
/// the ones a typed cardinality minimum guarantees are present.
fn declared_input_families(
    relations: &CompilerRelationAnalysis,
    operation: OperationId,
    mandatory_only: bool,
) -> BTreeSet<ObjectId> {
    relations
        .graph
        .node_weights()
        .filter(|node| node.source.id.operation() == operation)
        .filter_map(|node| match &node.source.relation {
            Relation::Cardinality {
                side: ObservedSide::Input,
                object,
                minimum,
                ..
            } if !mandatory_only || minimum.get() >= 1 => Some(*object),
            _ => None,
        })
        .collect()
}
