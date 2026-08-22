//! Typed execution cases derived from one proof-plan candidate
//! (Guide-5 Tranche A).
//!
//! An execution case is a finite semantic equivalence class, never a
//! sample transaction and never a vector fixture: within one case the
//! same relations are active, the same source requirements are active,
//! and the same representation is selected. The representation is
//! therefore not a case dimension the compiler crosses — it is already
//! fixed by the proof-plan candidate, and crossing it again would
//! manufacture a case that contradicts its own plan. Across the
//! complete feasible plan set every representation family is still
//! covered.
//!
//! The only dimension this stage expands is the operation's allowed
//! optional sponsor flow, read from the operation's own open-flow
//! relation. Counts, denominations, sponsor change, owner identities,
//! and object orderings are deliberately *not* case dimensions: they do
//! not change which relation is active or which source is required, and
//! expanding on them would inflate the census without adding semantic
//! content. Coverage vectors for those shapes belong to a later stage.

// One item-level allowance remains: `case_census` aggregates cases
// across a whole scope, and the factorized analysis derives cases per
// operation instead (§9.4).

use std::collections::{BTreeMap, BTreeSet};

use architecture::{ObjectId, OpenFlowKind, OperationId};
use realization::{Relation, RepresentationMode};

use crate::{
    CompileError,
    lifecycle::RepresentationChoiceId,
    proof::ProofPlanCandidate,
    relation::CompilerRelationAnalysis,
    source::{RequirementActivation, SourceRequirement},
    sponsor_region::{ORDINARY_LBTC, OrdinaryLbtcRole},
};

/// Whether the optional sponsor region exists in this case.
///
/// Two values, not a count: "one sponsor input" and "three sponsor
/// inputs with change" share every active relation and every active
/// source row, so they are the same placement case.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SponsorCase {
    Absent,
    Present,
}

/// Stable identity of one execution case.
///
/// Typed semantic dimensions only: no digest, no candidate index, no
/// search order, and no target position. Two analyses of the same
/// operation under the same plan produce equal identities.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExecutionCaseId {
    pub operation: OperationId,
    pub sponsor: SponsorCase,
    pub representations: BTreeMap<ObjectId, RepresentationMode>,
}

/// One execution case with the source rows active inside it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionCase {
    pub id: ExecutionCaseId,
    /// The candidate's source rows whose activation holds in this
    /// case, restricted to the case's own operation.
    pub active_sources: Vec<SourceRequirement>,
}

/// Whether a conservation relation's amounts are readable in its case.
///
/// The one axis on which a relation's *discharge* depends on its case.
/// Everything else the discharge classification fixes is a property of
/// the relation alone; this is not, because the representation a case
/// selected decides whether the amounts being conserved exist as
/// numbers a program can add or as commitments only the target relates.
///
/// It lives here, beside the case identity it is read from, because two
/// stages need the same answer — placement decides where the relation
/// discharges, and coverage decides which mutations that discharge
/// requires — and two derivations of one decision are two chances for
/// the plan and its coverage to describe different transactions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConservedAmountVisibility {
    /// The relation conserves nothing, or conserves amounts a program
    /// can read.
    ///
    /// The two are one answer on purpose: a relation with no amounts
    /// and a relation whose amounts are readable both leave the
    /// discharge exactly as the relation itself states it, and giving
    /// them separate answers would invite a caller to branch on a
    /// difference that changes nothing.
    Readable,
    /// The relation conserves amounts the target holds as commitments.
    Committed,
}

/// Whether one relation's conserved amounts are readable in one case.
///
/// # Why `PublicCommitted` counts as readable
///
/// The question this answers is not whether a commitment is present.
/// It is whether the analysis can hand a program the numbers to add. A
/// publicly committed amount is one an authenticated opening publishes,
/// so a program that receives the opening adds numbers and the
/// arithmetic discharge stands. A privately committed amount is one
/// nothing publishes, and there is no opening to receive. Grouping the
/// public mode with the private one because both involve commitments
/// would move a discharge the analysis can still perform to a target
/// that was never asked for it.
///
/// # Errors
///
/// [`CompileError::MixedRepresentationConservation`] when the case
/// fixed both a readable and a committed representation among the
/// families one relation conserves. Guide-13 §6.5 leaves mixed
/// representation unsupported until it is separately admitted, and the
/// two halves of such a relation have no common discharge: half of it
/// is arithmetic a carrier performs and half of it is evidence only the
/// target produces. Refusing is the disposition; silently choosing
/// either half would publish a plan for a transaction nobody planned.
pub fn conserved_amount_visibility(
    declaration: &realization::RelationDeclaration,
    case: &ExecutionCaseId,
) -> Result<ConservedAmountVisibility, CompileError> {
    let Relation::AmountConservation {
        input_objects,
        output_objects,
        ..
    } = &declaration.relation
    else {
        return Ok(ConservedAmountVisibility::Readable);
    };

    let committed = input_objects
        .iter()
        .chain(output_objects)
        .filter_map(|object| case.representations.get(object))
        .map(|mode| *mode == RepresentationMode::PrivateCommitted)
        .collect::<BTreeSet<_>>();

    match (committed.contains(&true), committed.contains(&false)) {
        (true, true) => Err(CompileError::MixedRepresentationConservation {
            relation: declaration.id.clone(),
        }),
        (true, false) => Ok(ConservedAmountVisibility::Committed),
        (false, _) => Ok(ConservedAmountVisibility::Readable),
    }
}

/// Whether one object family is the erased sponsor region *here*.
///
/// Both halves of the question are needed and neither is sufficient.
/// The family must be the one the architecture uses for open L-BTC
/// flows, and the operation must be one whose declared flows put that
/// family in the sponsor region rather than a protocol one. Testing the
/// family alone was the S2-01 defect: it made a mandatory owner-funded
/// input look like an optional sponsor region, and a formula-bound
/// payout look like an amount no relation may read.
#[must_use]
pub const fn is_sponsor_region_family(object: ObjectId, ordinary_lbtc: OrdinaryLbtcRole) -> bool {
    matches!(object, ORDINARY_LBTC) && ordinary_lbtc.is_sponsor_region()
}

/// The sponsor cases one operation admits, from its typed open-flow
/// relation — never from the operation's name.
///
/// An operation whose open-flow policy admits a fee sponsor has both
/// cases; any other operation has the sponsorless case only.
#[must_use]
pub fn sponsor_cases(
    relations: &CompilerRelationAnalysis,
    operation: OperationId,
) -> Vec<SponsorCase> {
    let sponsored = relations.graph.node_weights().any(|node| {
        node.source.id.operation() == operation
            && matches!(
                &node.source.relation,
                Relation::OpenFlowPolicy { allowed } if allowed.contains(&OpenFlowKind::FeeSponsor)
            )
    });

    if sponsored {
        vec![SponsorCase::Absent, SponsorCase::Present]
    } else {
        vec![SponsorCase::Absent]
    }
}

/// Operations carrying at least one in-scope relation, sorted.
#[must_use]
pub fn case_operations(relations: &CompilerRelationAnalysis) -> BTreeSet<OperationId> {
    relations
        .graph
        .node_weights()
        .map(|node| node.source.id.operation())
        .collect()
}

/// Derive every execution case of one feasible proof-plan candidate.
///
/// The candidate fixes the representation of every object it decided;
/// this stage adds only the sponsor dimension. The result is sorted by
/// case identity and free of duplicates by construction — the census
/// validator proves that independently rather than assuming it.
pub fn derive_execution_cases(
    relations: &CompilerRelationAnalysis,
    candidate: &ProofPlanCandidate,
) -> Vec<ExecutionCase> {
    let mut cases = Vec::new();

    for operation in case_operations(relations) {
        let representations = candidate_representations(candidate, operation);

        for sponsor in sponsor_cases(relations, operation) {
            let id = ExecutionCaseId {
                operation,
                sponsor,
                representations: representations.clone(),
            };
            let active_sources = active_sources(candidate, &id);

            cases.push(ExecutionCase { id, active_sources });
        }
    }

    cases.sort_by(|left, right| left.id.cmp(&right.id));
    cases
}

/// Validate the exact case census of one candidate.
///
/// # Errors
///
/// [`CompileError::MissingRepresentationChoice`] when the candidate
/// decided no mode for an object whose operation declares a
/// representation relation; [`CompileError::DuplicateExecutionCase`]
/// when one case identity occurs twice;
/// [`CompileError::ExecutionCaseCensusMismatch`] when the derived
/// census differs from the census the relations and the candidate
/// require.
pub fn validate_case_census(
    relations: &CompilerRelationAnalysis,
    candidate: &ProofPlanCandidate,
    cases: &[ExecutionCase],
) -> Result<(), CompileError> {
    let mut derived = BTreeSet::new();

    for case in cases {
        if !derived.insert(case.id.clone()) {
            return Err(CompileError::DuplicateExecutionCase {
                case: case.id.clone(),
            });
        }
    }

    let expected = expected_case_ids(relations, candidate)?;

    if derived != expected {
        return Err(CompileError::ExecutionCaseCensusMismatch {
            missing: expected.difference(&derived).cloned().collect(),
            unexpected: derived.difference(&expected).cloned().collect(),
        });
    }

    Ok(())
}

/// Derive and validate the cases of one candidate in one step.
///
/// # Errors
///
/// Any failure of [`validate_case_census`].
pub fn execution_cases(
    relations: &CompilerRelationAnalysis,
    candidate: &ProofPlanCandidate,
) -> Result<Vec<ExecutionCase>, CompileError> {
    let cases = derive_execution_cases(relations, candidate);

    validate_case_census(relations, candidate, &cases)?;
    Ok(cases)
}

/// The complete semantic case census across a feasible plan set.
///
/// Candidates that fixed the same representations yield the same case
/// identities; the union is a set, so the census counts semantic cases
/// rather than candidates.
///
/// # Errors
///
/// Any failure of [`validate_case_census`] for any candidate.
#[allow(dead_code)]
pub fn case_census(
    relations: &CompilerRelationAnalysis,
    candidates: &[ProofPlanCandidate],
) -> Result<BTreeSet<ExecutionCaseId>, CompileError> {
    let mut census = BTreeSet::new();

    for candidate in candidates {
        for case in execution_cases(relations, candidate)? {
            census.insert(case.id);
        }
    }

    Ok(census)
}

/// Whether one source-requirement row is active in one case.
///
/// A sponsor-local row exists only where the sponsor region exists; a
/// representation-conditional row exists only where the case fixed that
/// mode *for the row's own object*, never because some other object
/// family selected the same mode.
#[must_use]
pub fn is_active(case: &ExecutionCaseId, requirement: &SourceRequirement) -> bool {
    match requirement.activation {
        RequirementActivation::Always => true,
        RequirementActivation::WhenSponsorPresent => case.sponsor == SponsorCase::Present,
        RequirementActivation::WhenRepresentation { object, mode } => {
            case.representations.get(&object) == Some(&mode)
        }
    }
}

/// The candidate's active source rows for one case's own operation.
fn active_sources(
    candidate: &ProofPlanCandidate,
    case: &ExecutionCaseId,
) -> Vec<SourceRequirement> {
    let mut rows = candidate
        .source_requirements
        .iter()
        .filter(|requirement| requirement.operand.relation().operation() == case.operation)
        .filter(|requirement| is_active(case, requirement))
        .cloned()
        .collect::<Vec<_>>();

    rows.sort();
    rows
}

/// The representation decisions the candidate fixed for one operation.
fn candidate_representations(
    candidate: &ProofPlanCandidate,
    operation: OperationId,
) -> BTreeMap<ObjectId, RepresentationMode> {
    candidate
        .representations
        .iter()
        .filter(|(choice, _)| choice.operation == operation)
        .map(|(choice, mode)| (choice.object, *mode))
        .collect()
}

/// The census the relation set and the candidate require, recomputed
/// from the representation relations rather than from the derived
/// cases.
fn expected_case_ids(
    relations: &CompilerRelationAnalysis,
    candidate: &ProofPlanCandidate,
) -> Result<BTreeSet<ExecutionCaseId>, CompileError> {
    let mut expected = BTreeSet::new();

    for operation in case_operations(relations) {
        let mut representations = BTreeMap::new();

        for node in relations.graph.node_weights() {
            if node.source.id.operation() != operation {
                continue;
            }

            let Relation::Representation { object, .. } = &node.source.relation else {
                continue;
            };

            let choice = RepresentationChoiceId {
                operation,
                object: *object,
            };
            let mode = candidate.representations.get(&choice).ok_or(
                CompileError::MissingRepresentationChoice {
                    operation,
                    object: *object,
                },
            )?;

            representations.insert(*object, *mode);
        }

        for sponsor in sponsor_cases(relations, operation) {
            expected.insert(ExecutionCaseId {
                operation,
                sponsor,
                representations: representations.clone(),
            });
        }
    }

    Ok(expected)
}
