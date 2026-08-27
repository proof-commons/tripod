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
    ClosureObligation, ConservationDefect, ConservationRow, ConservationRowId, DeterminismLevel,
    ExpectedOutcomeLayer, RowDeferral,
};
use crate::protocol::{
    NATIVE_PROTOCOL_SCHEMA, NativeConservationResponse, ObservedOutcomeLayer, ResponseShapeDefect,
};

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
    /// expectation, which is Guide 11 §7.4's standing requirement: a
    /// disagreement stops the batch and is triaged rather than fitted.
    Disagrees,
    /// The run produced no target verdict at all.
    NotTargetEvidence,
    /// The row was never executed.
    Deferred,
}

/// Why a native conservation response cannot enter a report.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConservationIngestionDefect {
    /// The response belongs to another native protocol revision.
    UnsupportedProtocolSchema {
        /// The revision the response declared.
        offered: u32,
    },
    /// The response names a different row from the one being ingested.
    ResponseCaseMismatch {
        /// The row the report is ingesting.
        expected: ConservationRowId,
        /// The row the response names.
        observed: ConservationRowId,
    },
    /// The response contradicts revision 7's conservation semantics.
    SelfContradictoryResponse {
        /// The row the response names.
        row: ConservationRowId,
        /// The contradiction the protocol found.
        defect: ResponseShapeDefect,
    },
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
    /// Ingests one native response before adding its classified outcome.
    ///
    /// This is the production report path: the response passes schema,
    /// identity, and revision-7 shape validation before its observed layer
    /// is compared with the row's expectation. An error leaves the report
    /// unchanged.
    ///
    /// # Errors
    ///
    /// [`ConservationIngestionDefect`] if the response belongs to another
    /// revision or row, or contradicts the protocol's conservation shape.
    pub fn ingest_response(
        &mut self,
        row: &ConservationRow,
        response: NativeConservationResponse,
    ) -> Result<(), ConservationIngestionDefect> {
        let outcome = ingest_conservation_response(row, response)?;
        self.rows.push(outcome);
        Ok(())
    }

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

/// Validates and classifies one conservation response.
///
/// Shape validation precedes the evidence comparison. This function is
/// kept separate from [`ConservationReport::ingest_response`] so callers
/// assembling another typed artifact cannot bypass the same gate merely
/// because their destination is not a [`ConservationReport`].
///
/// # Errors
///
/// [`ConservationIngestionDefect`] if the response belongs to another
/// revision or row, or contradicts the protocol's conservation shape.
pub fn ingest_conservation_response(
    row: &ConservationRow,
    response: NativeConservationResponse,
) -> Result<RowOutcome, ConservationIngestionDefect> {
    if response.schema != NATIVE_PROTOCOL_SCHEMA {
        return Err(ConservationIngestionDefect::UnsupportedProtocolSchema {
            offered: response.schema,
        });
    }
    if response.case != row.id {
        return Err(ConservationIngestionDefect::ResponseCaseMismatch {
            expected: row.id.clone(),
            observed: response.case,
        });
    }
    response.validate_shape().map_err(|defect| {
        ConservationIngestionDefect::SelfContradictoryResponse {
            row: row.id.clone(),
            defect,
        }
    })?;

    let verdict = classify_response(row.expected_layer, response.observed_layer);
    Ok(RowOutcome {
        id: row.id.clone(),
        defect: row.defect,
        expected_layer: row.expected_layer,
        observed_layer: Some(response.observed_layer),
        observed_detail: response.observed_detail,
        verdict,
        deferral: row.deferral,
        closure_obligation: row.closure_obligation,
        materialized_transaction: response.transaction_bytes.as_deref().map(hexadecimal),
    })
}

/// Classifies a response whose schema, identity, and shape are validated.
const fn classify_response(
    expected: ExpectedOutcomeLayer,
    observed: ObservedOutcomeLayer,
) -> RowVerdict {
    if !observed.is_target_verdict() {
        return RowVerdict::NotTargetEvidence;
    }
    if expected_layer_agrees(expected, observed) {
        RowVerdict::Agrees
    } else {
        RowVerdict::Disagrees
    }
}

/// Whether one expected layer names the observed target verdict.
const fn expected_layer_agrees(
    expected: ExpectedOutcomeLayer,
    observed: ObservedOutcomeLayer,
) -> bool {
    match expected {
        ExpectedOutcomeLayer::Accepted => matches!(observed, ObservedOutcomeLayer::Accepted),
        ExpectedOutcomeLayer::ConsensusRejectionBeforeScript => {
            matches!(
                observed,
                ObservedOutcomeLayer::ConsensusRejectionBeforeScript
            )
        }
        ExpectedOutcomeLayer::ScriptPathRejection => {
            matches!(observed, ObservedOutcomeLayer::ScriptPathRejection)
        }
        ExpectedOutcomeLayer::RelayPolicyRejection => {
            matches!(observed, ObservedOutcomeLayer::RelayPolicyRejection)
        }
    }
}

/// Lowercase hexadecimal for transaction bytes retained in the report.
#[must_use]
fn hexadecimal(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(DIGITS[usize::from(*byte >> 4)]));
        encoded.push(char::from(DIGITS[usize::from(*byte & 0x0f)]));
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::conservation::canonical_conservation_matrix;
    use crate::protocol::{
        NATIVE_PROTOCOL_SCHEMA, NativeConservationResponse, ResponseShapeDefect,
    };

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
    fn typed_ingestion_validates_shape_before_classifying_evidence() {
        let row = canonical_conservation_matrix()
            .into_iter()
            .next()
            .expect("the matrix carries an accepted row");
        let response = NativeConservationResponse {
            schema: NATIVE_PROTOCOL_SCHEMA,
            case: row.id.clone(),
            observed_layer: ObservedOutcomeLayer::Accepted,
            observed_detail: None,
            transaction_bytes: None,
            observed_value_commitments: Vec::new(),
            observed_asset_commitments: Vec::new(),
            observed_openings: Vec::new(),
        };
        let mut document = report(Vec::new());
        assert_eq!(
            document.ingest_response(&row, response),
            Err(ConservationIngestionDefect::SelfContradictoryResponse {
                row: row.id,
                defect: ResponseShapeDefect::AcceptedConservationOmitsTransaction,
            }),
        );
        assert_eq!(document.rows, Vec::new());
    }

    #[test]
    fn typed_ingestion_classifies_only_a_validated_response() {
        let row = canonical_conservation_matrix()
            .into_iter()
            .next()
            .expect("the matrix carries an accepted row");
        let response = NativeConservationResponse {
            schema: NATIVE_PROTOCOL_SCHEMA,
            case: row.id.clone(),
            observed_layer: ObservedOutcomeLayer::Accepted,
            observed_detail: None,
            transaction_bytes: Some(vec![0x02]),
            observed_value_commitments: Vec::new(),
            observed_asset_commitments: Vec::new(),
            observed_openings: Vec::new(),
        };
        let mut document = report(Vec::new());
        document
            .ingest_response(&row, response)
            .expect("a valid accepted response enters the report");
        let ingested = document.rows.first().expect("the response entered");
        assert_eq!(ingested.verdict, RowVerdict::Agrees);
        assert!(ingested.establishes_target_fact());
        assert_eq!(ingested.materialized_transaction.as_deref(), Some("02"));
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
