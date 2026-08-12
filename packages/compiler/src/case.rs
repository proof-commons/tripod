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

/// True for the erased optional sponsor family.
///
/// The sponsor family is the one optional family the pilots admit; the
/// same object is what [`crate::source`] treats as sponsor-activated,
/// so both stages agree on one typed test rather than two.
#[must_use]
pub const fn is_sponsor_object(object: ObjectId) -> bool {
    matches!(object, ObjectId::PlainLbtc)
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
