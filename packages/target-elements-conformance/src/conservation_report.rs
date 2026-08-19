//! The confidential-transaction safety report.
//!
//! # This report is experimental, and says so in its own type
//!
//! Every other report in this package answers a settled question with a
//! settled role. This one does not: Guide 11 §8 establishes the CT
//! substrate and the consensus facts about conservation, and it does not
//! establish a candidate, an opening prototype, or a normalization
//! policy. A report that presented these rows as canonical CT evidence
//! would be claiming a completeness no wave has reached, and later waves
//! would inherit the claim rather than the findings.
//!
//! So the role is [`ConservationReportRole::Experimental`] and it is not
//! optional — the type has one variant, and a canonical role must be
//! *added* by whichever wave earns it rather than selected by a caller
//! who would like one.
//!
//! # What a row establishes, and what it cannot
//!
//! A row establishes a target fact only where the observed layer is a
//! target verdict. A fixture this adapter could not build and an
//! environment that failed around a run are recorded in full and
//! contribute nothing: [`RowOutcome::establishes_target_fact`] is what
//! separates them, and a reader counting evidence must count through it.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::conservation::{
    ClosureObligation, ConservationDefect, ConservationRowId, DeterminismLevel,
    ExpectedOutcomeLayer, RowDeferral,
};
use crate::protocol::ObservedOutcomeLayer;

/// What this report claims to be.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ConservationReportRole {
    /// Findings about the CT substrate and the conservation layer.
    ///
    /// Not a canonical evidence role. It establishes what the target does
    /// with the §8.4 matrix and nothing about a candidate, an opening
    /// prototype, or a normalization policy.
    Experimental,
}

/// How one row's expectation met its observation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RowVerdict {
    /// The target reached the expected layer.
    Agrees,
    /// The target reached a different layer.
    ///
    /// A disagreement is reported, never absorbed by rewriting the
    /// expectation `(´[PLAN-rule:guide11-exec:three-way]´)`.
    Disagrees,
    /// The run produced no target verdict at all.
    NotTargetEvidence,
    /// The row was never executed.
    Deferred,
}

/// One row's result.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RowOutcome {
    /// Which row.
    pub id: ConservationRowId,
    /// The defect the row introduced.
    pub defect: ConservationDefect,
    /// Where the row's author expected the execution to end up.
    pub expected_layer: ExpectedOutcomeLayer,
    /// Where it actually ended up, where the row ran.
    pub observed_layer: Option<ObservedOutcomeLayer>,
    /// What the target or the adapter said, verbatim.
    pub observed_detail: Option<String>,
    /// How the two met.
    pub verdict: RowVerdict,
    /// Why the row was not executed, where it was not.
    pub deferral: Option<RowDeferral>,
    /// The obligation consensus leaves undischarged, where the row
    /// carries one.
    pub closure_obligation: Option<ClosureObligation>,
    /// The transaction the run materialized, where it built one.
    ///
    /// Recorded because this materializer cannot produce the same bytes
    /// twice: without the bytes the row is not reproducible as evidence
    /// at all.
    pub materialized_transaction: Option<String>,
}

impl RowOutcome {
    /// Whether this row establishes anything about the target.
    ///
    /// A construction failure and an infrastructure failure establish
    /// nothing: the target was never asked. Counting them as evidence is
    /// the specific mistake Guide 11 §8.3 exists to prevent.
    #[must_use]
    pub fn establishes_target_fact(&self) -> bool {
        self.observed_layer
            .is_some_and(|layer| layer.is_target_verdict())
    }
}

/// What one conservation run established.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConservationReport {
    /// What this report claims to be.
    pub role: ConservationReportRole,
    /// How reproducible the run's transactions are.
    pub determinism: DeterminismLevel,
    /// Why the determinism level is what it is.
    ///
    /// Carried as text because it is a finding about the target's
    /// interfaces rather than a value this package computes, and a reader
    /// who sees only the level would have to take it on trust.
    pub determinism_finding: String,
    /// The chain the run observed.
    pub observed_genesis: String,
    /// The network identity that chain was bound to.
    pub observed_network: String,
    /// Every row, in matrix order.
    pub rows: Vec<RowOutcome>,
}

impl ConservationReport {
    /// How many rows fell into each verdict.
    #[must_use]
    pub fn tally(&self) -> BTreeMap<RowVerdict, usize> {
        let mut counts = BTreeMap::new();
        for row in &self.rows {
            *counts.entry(row.verdict).or_insert(0) += 1;
        }
        counts
    }

    /// Whether every row that produced a target verdict met its
    /// expectation.
    ///
    /// This is deliberately not a gate and deliberately not "the run
    /// passed". A run in which most rows never reached the target would
    /// satisfy it, which is why [`Self::rows_establishing_target_facts`]
    /// is reported beside it and neither is useful alone.
    #[must_use]
    pub fn every_target_verdict_agrees(&self) -> bool {
        self.rows
            .iter()
            .filter(|row| row.establishes_target_fact())
            .all(|row| row.verdict == RowVerdict::Agrees)
    }

    /// How many rows actually reached the target.
    #[must_use]
    pub fn rows_establishing_target_facts(&self) -> usize {
        self.rows
            .iter()
            .filter(|row| row.establishes_target_fact())
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(
        ordinal: u32,
        expected: ExpectedOutcomeLayer,
        observed: Option<ObservedOutcomeLayer>,
        verdict: RowVerdict,
    ) -> RowOutcome {
        RowOutcome {
            id: ConservationRowId {
                ordinal,
                name: format!("row-{ordinal}"),
            },
            defect: ConservationDefect::None,
            expected_layer: expected,
            observed_layer: observed,
            observed_detail: None,
            verdict,
            deferral: None,
            closure_obligation: None,
            materialized_transaction: None,
        }
    }

    fn report(rows: Vec<RowOutcome>) -> ConservationReport {
        ConservationReport {
            role: ConservationReportRole::Experimental,
            determinism: DeterminismLevel::FixtureInputsOnly,
            determinism_finding: "probe".to_owned(),
            observed_genesis: "00".to_owned(),
            observed_network: "11".to_owned(),
            rows,
        }
    }

    #[test]
    fn a_construction_failure_establishes_no_target_fact() {
        let outcome = row(
            1,
            ExpectedOutcomeLayer::Accepted,
            Some(ObservedOutcomeLayer::FixtureConstructionFailure),
            RowVerdict::NotTargetEvidence,
        );
        assert!(!outcome.establishes_target_fact());
    }

    #[test]
    fn an_infrastructure_failure_establishes_no_target_fact() {
        let outcome = row(
            1,
            ExpectedOutcomeLayer::Accepted,
            Some(ObservedOutcomeLayer::ExecutorInfrastructureFailure),
            RowVerdict::NotTargetEvidence,
        );
        assert!(!outcome.establishes_target_fact());
    }

    #[test]
    fn agreement_is_counted_only_over_rows_that_reached_the_target() {
        // The trap this guards: a run whose rows mostly failed to build
        // must not read as a clean run. Agreement holds over the rows that
        // reached the target, and the count of those rows is what says
        // whether that means anything.
        let document = report(vec![
            row(
                1,
                ExpectedOutcomeLayer::Accepted,
                Some(ObservedOutcomeLayer::Accepted),
                RowVerdict::Agrees,
            ),
            row(
                2,
                ExpectedOutcomeLayer::Accepted,
                Some(ObservedOutcomeLayer::FixtureConstructionFailure),
                RowVerdict::NotTargetEvidence,
            ),
        ]);
        assert!(document.every_target_verdict_agrees());
        assert_eq!(document.rows_establishing_target_facts(), 1);
    }

    #[test]
    fn a_disagreement_is_not_absorbed() {
        let document = report(vec![row(
            1,
            ExpectedOutcomeLayer::ConsensusRejectionBeforeScript,
            Some(ObservedOutcomeLayer::Accepted),
            RowVerdict::Disagrees,
        )]);
        assert!(!document.every_target_verdict_agrees());
        assert_eq!(document.tally()[&RowVerdict::Disagrees], 1);
    }

    #[test]
    fn the_role_admits_no_canonical_claim() {
        // One variant, so a caller cannot select a canonical role this
        // wave has not earned.
        let document = report(vec![]);
        assert_eq!(document.role, ConservationReportRole::Experimental);
    }
}
