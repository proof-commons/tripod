//! Guide-13 preflight reproductions that need an external caller.
//!
//! # Why this is an integration test and not a unit one
//!
//! `G13-R01` is a claim about the crate's *public* boundary: that a
//! caller outside this crate can move coverage rows to discharged from
//! values it authored itself, with no executor transcript, no target
//! binding, and no independently computed projection. A unit test inside
//! the crate could reach the same functions whether or not they were
//! public, so it could not tell that claim apart from its negation. This
//! file is a separate crate and sees exactly what any consumer sees.
//!
//! # What the assertions say
//!
//! Each test states the property the row's repair must establish, so it
//! fails while the defect stands. Confirmed rows carry `#[ignore]` with
//! the row id, keeping the ordinary lane green; replay them with
//! `-- --ignored`.
//!
//! Note for the repair wave: the recommended repair makes the tuple-level
//! discharge helpers crate-private, at which point this file stops
//! compiling by design. That is the repair landing, not a regression —
//! the test is replaced by the narrower public-API surface rather than
//! un-ignored.

use target_elements_conformance::protocol::ObservedOutcomeLayer;
use vectors::bundle::fixture_bundle;
use vectors::mutation::NegativeMutation;
use vectors::plan::{CompactAshEvidencePlan, ProjectionComparison, derive_evidence_plan};

/// The canonical evidence plan, as any consumer derives it.
fn plan() -> CompactAshEvidencePlan {
    let bundle = fixture_bundle().expect("the fixture bundle builds");
    derive_evidence_plan(&bundle).expect("the evidence plan derives")
}

/// `G13-R01`: caller-authored outcomes do not discharge coverage.
///
/// The plan's own constructor is checked, but its evidence state is then
/// mutated from a bare tuple of public enum values. Nothing in that tuple
/// is provenance-bearing: an external caller reads a real vector
/// identifier off the plan, states the two answers that together mean
/// "discharged", and the plan's coverage counters move.
///
/// The counters are what a downstream report or gate reads, so this is
/// the difference between coverage attributed to a supervised target run
/// and coverage attributed to an enum literal.
#[test]
#[ignore = "G13-R01: confirmed, repair pending"]
fn positive_coverage_is_not_dischargeable_from_a_caller_authored_tuple() {
    let mut plan = plan();
    assert_eq!(
        plan.discharged_rows(),
        0,
        "a freshly derived plan must start with nothing discharged"
    );

    // Everything below is available to any consumer of this crate.
    let vector = plan.target_cases()[0].subject().id();
    plan.discharge(&[(
        vector,
        ObservedOutcomeLayer::Accepted,
        ProjectionComparison::Matched,
    )]);

    assert_eq!(
        plan.discharged_rows(),
        0,
        "coverage was discharged from values the caller authored, with no transcript, target binding or recomputed projection behind them"
    );
}

/// `G13-R01`: the negative half is forgeable the same way.
///
/// A linked mutation and a stated refusal layer build an
/// `ObservedRefusal`, which the discharge predicate accepts
/// unconditionally, so a refusal nobody observed counts as negative
/// coverage.
#[test]
#[ignore = "G13-R01: confirmed, repair pending"]
fn negative_coverage_is_not_dischargeable_from_a_caller_authored_tuple() {
    let mut plan = plan();
    let before = plan.discharged_rows();
    let vector = plan.target_cases()[0].subject().id();

    plan.discharge_mutants(&[(
        NegativeMutation::SplitSuccessorInTwo,
        vector,
        ObservedOutcomeLayer::ScriptPathRejection,
    )]);

    assert_eq!(
        plan.discharged_rows(),
        before,
        "a refusal the caller stated rather than observed was counted as negative coverage"
    );
}

/// The control: a fresh plan really does report outstanding rows.
///
/// Unignored, so the two ignored tests are known to be about the
/// discharge path rather than about a plan that was empty all along.
#[test]
fn a_freshly_derived_plan_has_rows_to_discharge() {
    let plan = plan();
    assert_eq!(plan.observed_rows(), 0);
    assert_eq!(plan.discharged_rows(), 0);
    assert!(
        !plan.coverage_complete(),
        "a plan with nothing observed cannot be complete"
    );
    assert!(
        !plan.target_cases().is_empty(),
        "the plan must carry a vector whose identifier a caller can read"
    );
}
