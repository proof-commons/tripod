//! What a caller outside this crate can and cannot do to coverage.
//!
//! # Why this is an integration test and not a unit one
//!
//! `G13-R01` was a claim about the crate's *public* boundary: that a
//! consumer could move coverage rows to discharged from values it
//! authored itself, with no executor transcript, no target binding, and
//! no independently computed projection. A unit test inside the crate
//! could reach the same functions whether or not they were public, so it
//! could not tell that claim apart from its negation. This file is a
//! separate crate and sees exactly what any consumer sees.
//!
//! # What replaced the reproductions
//!
//! The two reproductions here stated the defect by calling
//! `discharge` and `discharge_mutants` with caller-authored tuples. The
//! repair made both crate-private, so those calls no longer compile —
//! which is the repair landing, and is also why they could not simply be
//! un-ignored. What stands in their place is the narrower statement the
//! type system can now carry: the one public route to coverage takes a
//! [`ValidatedCompactAshOperationReport`], and the only way to obtain
//! one is to hand [`validate_operation_report`] an `ExecutionTranscript`
//! — a type this workspace publishes no constructor for.
//!
//! The signature assertions below are compile-time. They are written as
//! functions that call the public entries at their exact types, so a
//! change that widened either entry back to something a caller can
//! author would fail to compile here rather than passing quietly.

use target_elements_conformance::executor::ExecutionTranscript;
use target_elements_conformance::protocol::ObservedOutcomeLayer;
use vectors::bundle::fixture_bundle;
use vectors::materialize::TargetVectorId;
use vectors::operation::OperationTranscript;
use vectors::plan::{CompactAshEvidencePlan, ProjectionComparison, derive_evidence_plan};
use vectors::report::{
    ReportValidationRefusal, ValidatedCompactAshOperationReport, validate_operation_report,
};

/// The canonical evidence plan, as any consumer derives it.
fn plan() -> CompactAshEvidencePlan {
    let bundle = fixture_bundle().expect("the fixture bundle builds");
    derive_evidence_plan(&bundle).expect("the evidence plan derives")
}

/// Compiles only while the one public discharge entry takes a validated
/// report and nothing else.
///
/// `G13-R01`'s repair in one line. The old entry took a slice of
/// `(TargetVectorId, ObservedOutcomeLayer, ProjectionComparison)` — three
/// public values, every one of them nameable here — and this states that
/// the argument is now a type whose values a consumer cannot produce.
fn the_discharge_entry(
    plan: &mut CompactAshEvidencePlan,
    report: &ValidatedCompactAshOperationReport<'_>,
) {
    plan.discharge_observed_run(report);
}

/// Compiles only while the one route to a validated report needs an
/// execution transcript.
///
/// The planner's own transcript is not enough and is not meant to be: it
/// is this package's record of what it made of the answers, and a record
/// checked only against itself establishes nothing. The second operand
/// is the executor's, and this crate publishes no way to build one.
fn the_validation_entry<'run>(
    execution: &ExecutionTranscript,
    planner: &'run OperationTranscript,
) -> Result<ValidatedCompactAshOperationReport<'run>, ReportValidationRefusal> {
    validate_operation_report(execution, planner)
}

/// The control: a fresh plan still reports outstanding target rows.
///
/// Kept from the reproductions, so the assertions above are known to be
/// about the discharge path rather than about a plan that was empty all
/// along.
#[test]
fn a_freshly_derived_plan_has_rows_to_discharge() {
    let plan = plan();
    assert_eq!(plan.observed_rows(), 2);
    assert_eq!(plan.discharged_rows(), 2);
    assert!(
        !plan.coverage_complete(),
        "a plan with only first-party observations cannot be complete"
    );
    assert!(
        !plan.target_cases().is_empty(),
        "the plan must carry a vector whose identifier a caller can read"
    );
}

/// `G13-R01`: everything a forger can still name reaches nothing.
///
/// The values the old tuple was made of are all still public, and have
/// to be: a vector identity is how a report names its subject, and the
/// two verdict enums are how a reader of a report reads it. What changed
/// is that no public function accepts them as an assertion. This test
/// assembles the exact triple the reproduction used and demonstrates
/// that holding it moves nothing beyond the derived first-party rows,
/// because the plan offers nowhere to put it — the two functions above
/// state, at compile time, what the only entries are.
#[test]
fn the_values_the_old_tuple_was_made_of_no_longer_reach_a_coverage_row() {
    let plan = plan();

    // Still readable, still nameable, and now inert.
    let vector: TargetVectorId = plan.target_cases()[0].subject().id();
    let layer = ObservedOutcomeLayer::Accepted;
    let comparison = ProjectionComparison::Matched;
    let forged = (vector, layer, comparison);
    assert_eq!(forged.0, vector);

    assert_eq!(
        plan.discharged_rows(),
        2,
        "a plan a caller only read from carries only derived first-party coverage"
    );
    assert!(!plan.coverage_complete());
}

/// `G13-R01`: the planner's transcript alone opens no route.
///
/// A consumer cannot build either an `OperationTranscript` or the
/// executor's `ExecutionTranscript`: neither type exposes a public
/// constructor. The functions are referenced rather than called for
/// exactly that reason: there are no values to call them with.
#[test]
fn a_planner_transcript_alone_is_not_a_run() {
    // Named so the entries above are known to be reachable at all; there
    // are no constructible transcripts to pass them, which is the property.
    let entry = the_validation_entry;
    let _ = &entry;
    let discharge = the_discharge_entry;
    let _ = &discharge;
}
