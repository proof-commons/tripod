//! Shared coverage matrix assembly and exact boundary census.
use crate::{coverage::CoverageBoundary, engine_placement::Key, engine_source::Evidence};
use std::collections::BTreeSet;

/// Opaque semantic identity: the engine compares purposes but never interprets
/// an attestation purpose on a keyed path.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Purpose {
    Attestation(crate::coverage::CoveragePurpose),
    #[cfg(test)]
    Verified(Evidence),
    #[cfg(test)]
    Rejected {
        evidence: Evidence,
        defect: EvidenceDefect,
    },
    #[cfg(test)]
    ClauseAccepted,
    #[cfg(test)]
    ClauseRejected,
    AcceptedProjection,
    CarrierExecution(crate::engine_placement::Placed),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg(test)]
pub enum EvidenceDefect {
    Missing,
    Failed,
    IdentityMismatch,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Requirement {
    pub key: Key,
    pub boundary: CoverageBoundary,
    pub purpose: Purpose,
    pub inactive: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    pub key: Key,
    pub active: bool,
    pub boundaries: BTreeSet<CoverageBoundary>,
    pub positive: Vec<Requirement>,
    pub negative: Vec<Requirement>,
    pub evidence: BTreeSet<Evidence>,
}
pub struct Subject {
    pub positive: Requirement,
    pub negative: Vec<Requirement>,
}
pub struct Matrix {
    pub key: Key,
    pub active: bool,
    pub boundaries: BTreeSet<CoverageBoundary>,
    pub inactive: Vec<Requirement>,
    pub subjects: Vec<Subject>,
    pub evidence: BTreeSet<Evidence>,
}
pub fn derive(matrix: Matrix) -> Plan {
    let span = tracing::debug_span!(target: "compiler::engine_coverage", "derive", boundary_count = crate::instrumentation::count(matrix.boundaries.len()), subject_count = crate::instrumentation::count(matrix.subjects.len()));
    let _entered = span.enter();
    let (mut positive, mut negative) = if matrix.active {
        let mut positive = Vec::new();
        let mut negative = Vec::new();
        for subject in matrix.subjects {
            positive.push(subject.positive);
            negative.extend(subject.negative);
        }
        (positive, negative)
    } else {
        (matrix.inactive, Vec::new())
    };
    positive.sort();
    negative.sort();
    Plan {
        key: matrix.key,
        active: matrix.active,
        boundaries: matrix.boundaries,
        positive,
        negative,
        evidence: matrix.evidence,
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    DuplicateRequirement(Requirement),
    CensusMismatch {
        missing: Vec<Key>,
        unexpected: Vec<Key>,
    },
    MissingInactive(Key),
    UnexpectedRejection(Key),
    MissingPositive {
        key: Key,
        boundary: CoverageBoundary,
    },
    MissingNegative {
        key: Key,
        boundary: CoverageBoundary,
    },
}
pub fn validate_census(planned: &BTreeSet<Key>, plans: &[Plan]) -> Result<(), Error> {
    let span = tracing::debug_span!(target: "compiler::engine_coverage", "validate_census", planned_count = crate::instrumentation::count(planned.len()), covered_count = crate::instrumentation::count(plans.len()));
    let _entered = span.enter();
    let mut seen = BTreeSet::new();
    for plan in plans {
        for row in plan.positive.iter().chain(&plan.negative) {
            if !seen.insert(row.clone()) {
                return Err(Error::DuplicateRequirement(row.clone()));
            }
        }
    }
    let covered = plans
        .iter()
        .map(|plan| plan.key.clone())
        .collect::<BTreeSet<_>>();
    if &covered != planned {
        return Err(Error::CensusMismatch {
            missing: planned.difference(&covered).cloned().collect(),
            unexpected: covered.difference(planned).cloned().collect(),
        });
    }
    for plan in plans {
        validate_plan(plan)?;
    }
    Ok(())
}
pub fn validate_plan(plan: &Plan) -> Result<(), Error> {
    if plan.active {
        for boundary in &plan.boundaries {
            if !plan.positive.iter().any(|row| row.boundary == *boundary) {
                return Err(Error::MissingPositive {
                    key: plan.key.clone(),
                    boundary: *boundary,
                });
            }
            if !plan.negative.iter().any(|row| row.boundary == *boundary) {
                return Err(Error::MissingNegative {
                    key: plan.key.clone(),
                    boundary: *boundary,
                });
            }
        }
    } else {
        if !plan.positive.iter().any(|row| row.inactive) {
            return Err(Error::MissingInactive(plan.key.clone()));
        }
        if !plan.negative.is_empty() {
            return Err(Error::UnexpectedRejection(plan.key.clone()));
        }
    }
    Ok(())
}
