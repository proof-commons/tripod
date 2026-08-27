//! The canonical rendering of one validated operation run.
//!
//! # Why the rendering lives here and not in the lane
//!
//! It used to live inside the node-gated integration test, where the
//! only way to exercise it was to have a live Elements node. A function
//! nothing can run without a node is a function nothing checks: the
//! report's field set, its escaping, and its determinism were all
//! unverified, and a lane that produced subtly different bytes on two
//! equal runs would have said so nowhere.
//!
//! So the rendering is a function of this crate, over a
//! [`ValidatedCompactAshOperationReport`], and the lane is reduced to
//! obtaining one and writing what this returns. The crate's own tests
//! then cover the seams the node was never needed for.
//!
//! # What is in the canonical bytes, and what is deliberately not
//!
//! Every figure comes off the validated report or is recomputed from it
//! here — the coverage counts are taken by discharging a freshly derived
//! plan, not accepted from a caller. Two equal validated reports
//! therefore render byte-identical output, which is what makes the file
//! comparable across runs at all.
//!
//! Wall-clock duration is the one thing kept out. It is a property of
//! the machine the run happened on rather than of the run's result, and
//! putting it in the canonical stream made every report differ from
//! every other report by construction. The lane records it beside the
//! report instead.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use target_elements_conformance::executor::ExecutorTrust;
use target_elements_conformance::protocol::ObservedOutcomeLayer;

use crate::bundle::fixture_bundle;
use crate::error::VectorError;
use crate::materialize::TargetVectorId;
use crate::observed_boundary::matches_boundary;
use crate::operation::OperationTranscript;
use crate::plan::{ProjectionComparison, derive_evidence_plan};
use crate::report::{
    OPERATION_REPORT_SCHEMA, ProjectionVerdict, ValidatedCompactAshOperationReport,
};

/// Render one validated run as the canonical report object.
///
/// # Errors
///
/// [`VectorError`] where the evidence plan the coverage counts are taken
/// from cannot be derived. That is a defect in this workspace rather
/// than a fact about the run: the plan is derived from fixtures that
/// were already built once before the run started.
pub fn render_validated_report(
    report: &ValidatedCompactAshOperationReport<'_>,
) -> Result<String, VectorError> {
    let transcript = report.transcript();
    let projections = report.projections();

    let mut out = String::new();
    out.push_str("{\n");
    let _ = writeln!(out, "  \"schema\": {OPERATION_REPORT_SCHEMA},");
    let _ = writeln!(out, "  \"run\": {},", quote("completed"));
    out.push_str(&render_executor(report));
    out.push_str(&render_ceremony(transcript));
    let _ = writeln!(
        out,
        "  \"projections_compared\": {}, \"projections_matched\": {},",
        projections.len(),
        projections
            .values()
            .filter(|verdict| verdict.comparison() == ProjectionComparison::Matched)
            .count()
    );
    let _ = writeln!(
        out,
        "  \"weights_compared\": {}, \"weights_matched\": {},",
        report.weights_compared(),
        report.weights_matched(),
    );
    out.push_str(&render_coverage(report)?);
    out.push_str(&render_divergences(transcript));
    out.push_str(&render_submissions(transcript, projections));
    out.push_str(&render_mutations(transcript)?);
    out.push_str("}\n");
    Ok(out)
}

/// Render a run that never became a report.
///
/// A refused run still happened and is still worth writing down; what it
/// cannot carry is a coverage count, because the work those counts are
/// about was not completed. The object therefore states the refusal and
/// the ceremony facts and stops — a reader can see there are no coverage
/// fields, which is a different statement from coverage of zero.
///
/// The reason is the caller's text because the refusal is the executor
/// boundary's, in that package's vocabulary rather than this one's. It
/// is escaped like any other string and appears on no other path.
///
/// # Errors
///
/// [`VectorError`] where a staged mutation names no §18 class, which is
/// a defect in this workspace's own matrix.
pub fn render_refused_run(
    transcript: &OperationTranscript,
    reason: &str,
) -> Result<String, VectorError> {
    let mut out = String::new();
    out.push_str("{\n");
    let _ = writeln!(out, "  \"schema\": {OPERATION_REPORT_SCHEMA},");
    let _ = writeln!(out, "  \"run\": {},", quote(&format!("refused: {reason}")));
    out.push_str(&render_ceremony(transcript));
    out.push_str(&render_divergences(transcript));
    out.push_str(&render_submissions(transcript, &BTreeMap::new()));
    out.push_str(&render_mutations(transcript)?);
    out.push_str("}\n");
    Ok(out)
}

/// What ran, as the report records it.
///
/// The trust state is a declaration rather than a finding, and is
/// written under a name that says so. What the executor said about
/// itself is written under a name that says that too, and the ADR-018
/// comparison is written separately from both: a run that made no
/// comparison writes null there rather than borrowing the reported tip,
/// so a reader can never mistake a claim for a verified one.
fn render_executor(report: &ValidatedCompactAshOperationReport<'_>) -> String {
    let executor = report.executor();
    let mut out = String::new();
    let _ = writeln!(
        out,
        "  \"declared_executor_trust\": {}, \"steps_answered\": {},",
        quote(match report.trust() {
            ExecutorTrust::ReviewedNonMock => "reviewed-nonmock",
            _ => "mock",
        }),
        report.steps_answered(),
    );
    let _ = writeln!(
        out,
        "  \"executor_reported\": {{\"adapter_name\": {}, \"adapter_version\": {}, \"node_name\": {}, \"node_version\": {}, \"intended_executed_tip\": {}}},",
        quote(executor.adapter_name()),
        quote(executor.adapter_version()),
        quote(executor.node_name()),
        quote(executor.node_version()),
        executor
            .intended_executed_tip()
            .map_or_else(|| "null".to_owned(), quote),
    );
    let _ = writeln!(
        out,
        "  \"executor_provenance_verified\": {},",
        executor.expected_provenance().map_or_else(
            || "null".to_owned(),
            |validated| quote(validated.intended_tip().as_revision().as_str()),
        ),
    );
    out
}

/// What the ceremony established before anything was submitted.
fn render_ceremony(transcript: &OperationTranscript) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "  \"plan_refusal\": {},",
        transcript.refusal().map_or_else(
            || "null".to_owned(),
            |refusal| quote(&format!("{refusal:?}"))
        )
    );
    let _ = writeln!(
        out,
        "  \"issued_asset\": {},",
        transcript
            .issued_asset()
            .map_or_else(|| "null".to_owned(), |asset| quote(&hex(&asset)))
    );
    let _ = writeln!(
        out,
        "  \"constructor_program\": {},",
        transcript
            .constructor_program()
            .map_or_else(|| "null".to_owned(), |program| quote(&hex(program)))
    );
    let _ = writeln!(
        out,
        "  \"reserve_asset\": {},",
        transcript
            .reserve_asset()
            .map_or_else(|| "null".to_owned(), |asset| quote(&hex(&asset)))
    );
    let _ = writeln!(out, "  \"funded_vectors\": {},", transcript.funded().len());
    let _ = writeln!(
        out,
        "  \"sponsored_submissions\": {},",
        transcript
            .submissions()
            .iter()
            .filter(|submission| submission.vector().sponsors() > 0)
            .count()
    );
    out
}

/// The §19 coverage counts, recomputed from a freshly derived plan.
///
/// The plan is derived here and discharged here rather than passed in,
/// so the figures cannot be a caller's arithmetic wearing the plan's
/// name. It is a pure function of this workspace's fixtures and of the
/// validated report, which is what keeps two equal reports rendering
/// equal bytes.
fn render_coverage(report: &ValidatedCompactAshOperationReport<'_>) -> Result<String, VectorError> {
    let bundle = fixture_bundle().map_err(VectorError::from)?;
    let mut plan = derive_evidence_plan(&bundle)?;
    plan.discharge_observed_run(report);
    let mut out = String::new();
    let _ = writeln!(
        out,
        "  \"coverage_requirements\": {}, \"coverage_observed\": {}, \"coverage_discharged\": {},",
        plan.census().coverage_requirements(),
        plan.observed_rows(),
        plan.discharged_rows(),
    );
    let _ = writeln!(
        out,
        "  \"coverage_discharged_positive\": {}, \"coverage_discharged_negative\": {},",
        plan.discharged_positive_rows(),
        plan.discharged_negative_rows(),
    );
    Ok(out)
}

/// Every positive submission, with both weights beside its verdict.
fn render_submissions(
    transcript: &OperationTranscript,
    projections: &BTreeMap<TargetVectorId, ProjectionVerdict>,
) -> String {
    let mut out = String::from("  \"submissions\": [\n");
    for (index, submission) in transcript.submissions().iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        let _ = write!(
            out,
            "    {{\"ordinal\": {}, \"ash_inputs\": {}, \"sponsors\": {}, \"layer\": {}, \"txid\": {}, \"projection\": {}, \"detail\": {}, \"bytes\": {}, \"witness_bytes\": {}, \"virtual_size\": {}, \"predicted_weight\": {}, \"observed_weight\": {}, \"weight_agrees\": {}}}",
            submission.vector().fixture().ordinal(),
            submission.vector().ash_inputs(),
            submission.vector().sponsors(),
            quote(&submission.layer().to_string()),
            submission
                .accepted_txid()
                .map_or_else(|| "null".to_owned(), quote),
            projections
                .get(&submission.vector())
                .map_or_else(|| quote("not performed"), |verdict| quote(verdict.detail())),
            submission.detail().map_or_else(|| "null".to_owned(), quote),
            submission.bytes().len(),
            submission.witness_bytes(),
            submission.virtual_size(),
            submission.predicted_weight(),
            submission
                .observed_weight()
                .map_or_else(|| "null".to_owned(), |weight| weight.to_string()),
            submission
                .weight_agrees()
                .map_or_else(|| "null".to_owned(), |agrees| agrees.to_string()),
        );
    }
    out.push_str("\n  ],\n");
    out
}

/// The rows this target's own money bound forbids.
///
/// What the target said when it was asked for one anyway, written apart
/// from the submissions and never among them: nothing was built for
/// these, so nothing was judged, and a reader must not be able to count
/// one as a result.
fn render_divergences(transcript: &OperationTranscript) -> String {
    let mut out = String::from("  \"target_amount_divergences\": [\n");
    for (index, divergence) in transcript.divergences().iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        let _ = write!(
            out,
            "    {{\"ordinal\": {}, \"fixture\": {}, \"stated\": {}, \"bound\": {}, \"excess\": {}, \"layer\": {}, \"target_verdict\": {}, \"detail\": {}}}",
            divergence.vector().fixture().ordinal(),
            quote(divergence.vector().fixture().name()),
            divergence.beyond().stated(),
            divergence.beyond().bound(),
            divergence.beyond().excess(),
            quote(&divergence.layer().to_string()),
            // §1.5, stated rather than left to be inferred from the
            // layer's name: an adapter that could not perform a step has
            // not told anyone what the target thinks of it.
            divergence.layer().is_target_verdict(),
            divergence.detail().map_or_else(|| "null".to_owned(), quote),
        );
    }
    out.push_str("\n  ],\n");
    out
}

/// The negative half of the report.
///
/// Each row states the mutation, the §1.5 boundary the §18 class it
/// stages expects, the layer the target actually answered at, and
/// whether those two agree — as separate fields, because a row recording
/// only the agreement would hide which way a disagreement went.
///
/// Whether the mutation could even reach its expected boundary is a
/// further fact: an arm that unbalances the closed asset is answered by
/// consensus before any script runs, so a script-path expectation is
/// unreachable for it whatever the covenant would have said.
///
/// # Errors
///
/// [`VectorError`] where an arm names no §18 class. That is a defect in
/// this workspace's own matrix, and it is propagated rather than
/// rendered as an absent boundary: an absence here would read as a
/// mutation the guide does not determine, which is a different fact.
fn render_mutations(transcript: &OperationTranscript) -> Result<String, VectorError> {
    // The control first, because every row below depends on it. A
    // mutation refused while its control was also refused is not
    // evidence about the mutation: both could have failed for the same
    // unrelated reason, and the reader has to be able to see that
    // before reading a single mutation row.
    let subject = transcript.mutation_subject();
    let control = subject.and_then(|id| {
        transcript
            .submissions()
            .iter()
            .find(|submission| submission.vector() == id)
    });
    // Whether the subject's coins were ever demonstrably spendable. The
    // control's acceptance shows it; so does an accepted mutation, which
    // spent them itself. Either settles the question a refusal on its
    // own could not.
    let control_accepted =
        control.is_some_and(|submission| submission.layer() == ObservedOutcomeLayer::Accepted);
    let any_mutation_accepted = transcript
        .mutants()
        .iter()
        .any(|mutant| mutant.layer() == ObservedOutcomeLayer::Accepted);
    let coins_were_spendable = control_accepted || any_mutation_accepted;

    let mut out = String::new();
    let _ = writeln!(
        out,
        "  \"mutation_control\": {{\"subject_ordinal\": {}, \"layer\": {}, \"control_accepted\": {control_accepted}, \"coins_were_spendable\": {coins_were_spendable}}},",
        subject.map_or_else(
            || "null".to_owned(),
            |id| id.fixture().ordinal().to_string()
        ),
        control.map_or_else(
            || "null".to_owned(),
            |submission| quote(&submission.layer().to_string())
        ),
    );
    out.push_str("  \"mutations\": [\n");
    // Attributability is per row and order-dependent: once a mutation is
    // accepted it spends the subject's coins, so every later row is
    // refused for that rather than for what it changed. A global flag
    // would throw away the rows submitted before that happened, which
    // are the ones that actually established something.
    let mut spent = false;
    for (index, mutant) in transcript.mutants().iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        let observed = mutant.layer();
        let submitted = !mutant.bytes().is_empty();
        let attributable = submitted && !spent && coins_were_spendable;
        let expected = mutant.mutation().expected_boundary()?;
        let _ = write!(
            out,
            "    {{\"mutation\": {}, \"class\": {}, \"origin_ordinal\": {}, \"expected_boundary\": {}, \"observed_layer\": {}, \"boundary_matched\": {}, \"target_verdict\": {}, \"attributable\": {attributable}, \"submitted\": {submitted}, \"preserves_value_balance\": {}, \"txid\": {}, \"detail\": {}, \"bytes\": {}}}",
            quote(&format!("{:?}", mutant.mutation())),
            quote(mutant.mutation().class_name()),
            mutant.origin().fixture().ordinal(),
            quote(&format!("{expected:?}")),
            quote(&observed.to_string()),
            matches_boundary(expected, observed),
            observed.is_target_verdict(),
            mutant.mutation().preserves_value_balance(),
            mutant
                .accepted_txid()
                .map_or_else(|| "null".to_owned(), quote),
            mutant.detail().map_or_else(|| "null".to_owned(), quote),
            mutant.bytes().len(),
        );
        if observed == ObservedOutcomeLayer::Accepted {
            spent = true;
        }
    }
    out.push_str("\n  ]\n");
    Ok(out)
}

/// One string, escaped for the report object.
fn quote(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other if (other as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", other as u32);
            }
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

/// Bytes, in the order they are held.
fn hex(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(text, "{byte:02x}");
    }
    text
}

#[cfg(test)]
mod tests {
    use super::{hex, quote, render_refused_run, render_validated_report};
    use crate::report::OPERATION_REPORT_SCHEMA;
    use crate::report::tests::{staged_report, staged_transcript};

    /// The canonical bytes carry the revision that produced them.
    ///
    /// A reader must never have to infer which field set a file is in:
    /// a report whose shape changed, read by a reader that assumed the
    /// old one, would be read wrong rather than refused.
    #[test]
    fn every_rendered_object_states_its_schema() {
        let transcript = staged_transcript();
        let report = staged_report(&transcript);
        let canonical = render_validated_report(&report).expect("the staged run renders");
        assert!(canonical.starts_with("{\n"));
        assert!(canonical.contains(&format!("\"schema\": {OPERATION_REPORT_SCHEMA},")));

        let refused = render_refused_run(&transcript, "the executor died").expect("it renders");
        assert!(refused.contains(&format!("\"schema\": {OPERATION_REPORT_SCHEMA},")));
    }

    /// `G13-R12`: equal validated inputs render equal bytes.
    ///
    /// The property the wall clock used to make impossible. Every figure
    /// in the canonical stream is a function of the report and of this
    /// workspace's own fixtures, so rendering the same run twice — and
    /// rendering two separately validated reports of it — produces one
    /// byte string.
    #[test]
    fn one_run_renders_to_one_byte_string() {
        let transcript = staged_transcript();
        let first = render_validated_report(&staged_report(&transcript)).expect("it renders");
        let second = render_validated_report(&staged_report(&transcript)).expect("it renders");
        assert_eq!(first, second);
    }

    /// `G13-R12`: no wall clock reaches the canonical bytes.
    ///
    /// Checked against the rendering rather than trusted to the field
    /// list, because a later field would pass a field-list review and
    /// fail this.
    #[test]
    fn the_canonical_bytes_carry_no_duration() {
        let transcript = staged_transcript();
        let canonical = render_validated_report(&staged_report(&transcript)).expect("it renders");
        for forbidden in ["wall", "seconds", "elapsed", "duration", "millis"] {
            assert!(
                !canonical.contains(forbidden),
                "the canonical report rendered {forbidden}"
            );
        }
    }

    /// A refused run states no coverage rather than coverage of zero.
    ///
    /// Zero is a count, and the honest statement about a run that did
    /// not complete is that there was nothing to count.
    #[test]
    fn a_refused_run_carries_no_coverage_fields() {
        let transcript = staged_transcript();
        let refused = render_refused_run(&transcript, "the executor died").expect("it renders");
        assert!(!refused.contains("coverage_"));
        assert!(refused.contains("\"run\": \"refused: the executor died\""));

        let canonical = render_validated_report(&staged_report(&transcript)).expect("it renders");
        assert!(canonical.contains("\"coverage_requirements\":"));
        assert!(canonical.contains("\"run\": \"completed\""));
    }

    /// The validated report's own figures reach the bytes.
    ///
    /// Including the declaration the executor was run under, which a
    /// reader needs in order to know what the file is evidence of at
    /// all: a mock declared reviewed is still a mock.
    #[test]
    fn the_rendered_object_states_what_ran_and_what_was_compared() {
        let transcript = staged_transcript();
        let report = staged_report(&transcript);
        let canonical = render_validated_report(&report).expect("it renders");
        assert!(canonical.contains("\"declared_executor_trust\": \"mock\""));
        assert!(canonical.contains("\"adapter_name\": \"staged\""));
        assert!(canonical.contains(&format!(
            "\"projections_compared\": {}, \"projections_matched\": 0,",
            report.projections().len()
        )));
        assert!(canonical.contains("\"weights_compared\": 0, \"weights_matched\": 0,"));
    }

    /// An unaccepted submission renders its comparison as unperformed.
    ///
    /// §1.4's second verdict does not exist for a transaction that never
    /// reached the first, and the report must not spell that absence as
    /// a disagreement.
    #[test]
    fn a_submission_with_no_comparison_says_so() {
        let transcript = staged_transcript();
        let canonical = render_validated_report(&staged_report(&transcript)).expect("it renders");
        assert!(canonical.contains("\"projection\": \"unreadable:"));
    }

    /// A string carrying report syntax cannot break out of its field.
    #[test]
    fn a_quoted_string_escapes_what_would_end_it() {
        assert_eq!(quote("plain"), "\"plain\"");
        assert_eq!(quote("a\"b"), "\"a\\\"b\"");
        assert_eq!(quote("a\\b"), "\"a\\\\b\"");
        assert_eq!(quote("a\nb"), "\"a\\nb\"");
        assert_eq!(quote("a\u{1}b"), "\"a\\u0001b\"");
    }

    /// Bytes render in the order they are held, two digits each.
    #[test]
    fn bytes_render_in_the_order_they_are_held() {
        assert_eq!(hex(&[0x00, 0x0f, 0xff]), "000fff");
        assert_eq!(hex(&[]), "");
    }
}
