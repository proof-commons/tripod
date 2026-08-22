//! The operation lane: one compact-ASH run against a real node.
//!
//! # Why this is an ignored test rather than a command
//!
//! It needs a live Elements node, so it cannot run in an ordinary lane,
//! and what it produces is a transcript rather than a gate verdict. An
//! ADR-010 command would have to state a report contract this wave has
//! not settled; an ignored test states its inputs as environment,
//! writes what happened to a file, and asserts only what a run that
//! happened at all must satisfy.
//!
//! Run it as:
//!
//! ```text
//! TRIPOD_OPERATION_EXECUTOR=<adapter> \
//! TRIPOD_OPERATION_NETWORK_ID=<64 hex> \
//! TRIPOD_OPERATION_GENESIS_ID=<64 hex> \
//! TRIPOD_OPERATION_REPORT=<path> \
//!   cargo test -p tripod-vectors --test compact_ash_operation -- --ignored --nocapture
//! ```
//!
//! # What this file still owns
//!
//! Obtaining a run, and asserting the shape of one that completed. The
//! rendering it used to carry moved into `vectors::render`, where the
//! crate's own tests can cover it without a node: a function reachable
//! only through a live Elements node is a function nothing checks
//! (`G13-R12`). What is written here is a *validated* report — the
//! executor's record and the planner's, laid beside each other and
//! agreeing — so the coverage figures in the file are attributable to
//! the run rather than to this lane's arithmetic.
//!
//! # Nothing here decides what the run should have found
//!
//! The assertions are about the *shape* of a completed run: that the
//! ceremony reached the target, that every submission was answered, that
//! the transcript records the same number of outcomes as vectors
//! submitted, that the run met exactly the money-bound divergences the
//! plan's census derived before it started, and that a §17.4 comparison
//! was performed for every acceptance. Whether the target accepted
//! anything, and what the comparisons found, are written down and not
//! asserted: a wave that asserted acceptance would fail rather than
//! report when the honest answer is a refusal
//! `(´[PLAN-rule:guide12-exec:failure-layers]´)`.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding, LeafVersion,
    reviewed_elements_tapscript, validate_reviewed_development_binding,
};
use target_elements_conformance::executor::{
    DEFAULT_EXECUTOR_TIMEOUT, ExecutorConfiguration, ExecutorDiagnostics, ExecutorTrust,
    execute_operations,
};
use target_elements_conformance::protocol::ObservedOutcomeLayer;
use vectors::operation::{CompactAshOperationPlanner, OperationTranscript};
use vectors::render::{render_refused_run, render_validated_report};
use vectors::report::validate_operation_report;

fn environment(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|value| !value.is_empty())
}

fn identifier(text: &str) -> [u8; 32] {
    let mut bytes = [0_u8; 32];
    let raw = text.as_bytes();
    let (pairs, _) = raw.as_chunks::<2>();
    for (slot, pair) in bytes.iter_mut().zip(pairs) {
        let digits = std::str::from_utf8(pair).expect("the identifier is hex");
        *slot = u8::from_str_radix(digits, 16).expect("the identifier is hex");
    }
    bytes
}

/// Where the run's wall time is written.
///
/// Beside the report and never inside it. Duration is a property of the
/// machine the run happened on rather than of the run's result, and a
/// canonical stream carrying one differs from every other by
/// construction, so two runs of the same census could never be compared
/// byte for byte (`G13-R12`).
fn timing_path(report: &Path) -> PathBuf {
    let mut path = report.to_path_buf();
    let mut name = path.file_name().map_or_else(
        || "operation-report".to_owned(),
        |name| name.to_string_lossy().into_owned(),
    );
    name.push_str(".timing");
    path.set_file_name(name);
    path
}

#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn compact_ash_runs_against_a_real_target() {
    let executor = environment("TRIPOD_OPERATION_EXECUTOR")
        .expect("TRIPOD_OPERATION_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_OPERATION_NETWORK_ID")
        .expect("TRIPOD_OPERATION_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_OPERATION_GENESIS_ID")
        .expect("TRIPOD_OPERATION_GENESIS_ID states the chain the run is bound to");
    let report = environment("TRIPOD_OPERATION_REPORT")
        .map(PathBuf::from)
        .expect("TRIPOD_OPERATION_REPORT names where the transcript is written");

    let target = reviewed_elements_tapscript().expect("the reviewed target validates");
    let binding = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            target.definition().version(),
            DeploymentEnvironment::Development,
            identifier(&network),
            identifier(&genesis),
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .expect("the development binding validates");

    let timeout = environment("TRIPOD_OPERATION_TIMEOUT_SECONDS")
        .and_then(|value| value.parse::<u64>().ok())
        .map_or(DEFAULT_EXECUTOR_TIMEOUT, Duration::from_secs);
    let configuration = ExecutorConfiguration::new(
        std::path::Path::new(&executor),
        // The trust declaration is the operator's and establishes nothing
        // about the program; this lane produces a transcript rather than
        // a gate verdict, so it declares the adapter it was pointed at.
        ExecutorTrust::ReviewedNonMock,
        timeout,
        // Beside the report, because they are the same kind of thing:
        // what one run on one host produced. Derived from the report's
        // own directory rather than asked for separately, so the
        // operator recipe above still states every destination once.
        ExecutorDiagnostics::in_directory(report.parent().unwrap_or_else(|| Path::new("."))),
    );

    let mut planner = CompactAshOperationPlanner::new().expect("the planner builds");
    let started = Instant::now();
    let outcome = execute_operations(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let transcript = planner.transcript();
    // The two records of the run, laid beside each other. A refusal here
    // is a defect and not a target verdict: it says the executor's own
    // record and the planner's do not describe the same run, so the lane
    // stops rather than writing a report about which of them to believe.
    let validated = outcome.as_ref().ok().map(|execution| {
        validate_operation_report(execution, transcript)
            .expect("the executor's record and the planner's describe the same run")
    });
    let out = validated.as_ref().map_or_else(
        || {
            let reason = outcome
                .as_ref()
                .err()
                .map_or_else(|| "unknown".to_owned(), ToString::to_string);
            render_refused_run(transcript, &reason).expect("the refused run renders")
        },
        |report| render_validated_report(report).expect("the validated run renders"),
    );
    std::fs::write(&report, &out).expect("the transcript is written");
    std::fs::write(
        timing_path(&report),
        format!("wall_seconds {:.1}\n", wall.as_secs_f64()),
    )
    .expect("the run's wall time is written");

    // A run that reached the target at all answered every step it asked
    // for. Nothing here says what the answers were.
    if let Some(report) = validated.as_ref() {
        assert_eq!(
            transcript.submissions().len(),
            planner.vectors().len(),
            "a submission went unanswered"
        );

        // And the run did what the plan's own census said it would.
        // The census is derived from the reviewed target bound before
        // anything runs; the transcript is what the target answered.
        // A disagreement means one of the two is describing a different
        // wave than the other.
        let bundle = vectors::bundle::fixture_bundle().expect("the fixture bundle builds");
        let plan = vectors::derive_evidence_plan(&bundle).expect("the evidence plan derives");
        assert_eq!(
            transcript.submissions().len(),
            plan.census().submittable_vectors() + plan.census().sponsored_vectors(),
            "the run submitted a different number of vectors than the plan admits"
        );

        // And the sponsored half actually ran. The count comes from the
        // plan's own census rather than from a number written here, so a
        // candidate whose bounds admitted more sponsored rows would be
        // held to the larger figure without this line changing.
        assert_eq!(
            transcript
                .submissions()
                .iter()
                .filter(|submission| submission.vector().sponsors() > 0)
                .count(),
            plan.census().sponsored_vectors(),
            "the run submitted a different number of sponsored vectors than the plan names"
        );
        assert_eq!(
            transcript.divergences().len(),
            plan.census().divergent_vectors(),
            "the run met a different number of money-bound divergences than the plan derived"
        );

        // The §17.4 comparison was attempted for every acceptance. What
        // it found is written down, not asserted; that it was performed
        // at all is a property of this lane and is checked, because a
        // silently skipped comparison and a matching one would otherwise
        // look the same in the report.
        let accepted = transcript
            .submissions()
            .iter()
            .filter(|submission| submission.layer() == ObservedOutcomeLayer::Accepted)
            .count();
        assert_eq!(
            report.projections().len(),
            accepted,
            "an accepted transaction went uncompared"
        );

        check_weight_comparison(transcript, accepted);
        check_negative_half(transcript, accepted);
    }
}

/// §20.5's comparison, asserted rather than merely written down.
///
/// Unlike the §17.4 projection, whose verdicts this lane records and
/// does not judge, a weight disagreement is a defect on its face: the
/// transaction layer and the target weighed the *same bytes*, so the
/// two answers cannot honestly differ. §20.5 says a mismatch fails the
/// resource report even where the transaction was accepted, and this is
/// where that happens.
///
/// An unobserved weight is not a failure. An executor that reports none
/// leaves the comparison unmade, and this lane refuses to read that as
/// agreement — but it also refuses to invent a requirement the protocol
/// does not place on an executor.
fn check_weight_comparison(transcript: &OperationTranscript, accepted: usize) {
    for submission in transcript.submissions() {
        assert_ne!(
            submission.weight_agrees(),
            Some(false),
            "the target weighed vector {:?} at {:?} where the ABI settled {}",
            submission.vector(),
            submission.observed_weight(),
            submission.predicted_weight(),
        );
    }

    // An executor advertising the observation must actually make it on
    // every acceptance. Without this, an adapter that quietly stopped
    // reporting weights would turn every comparison into an unmade one
    // and the assertion above would pass by vacuum.
    let observed = transcript
        .submissions()
        .iter()
        .filter(|submission| submission.observed_weight().is_some())
        .count();
    assert!(
        observed == 0 || observed >= accepted,
        "an executor that weighs anything must weigh every acceptance: \
         {observed} observed against {accepted} accepted",
    );
}

/// The negative half's shape, and nothing about its verdicts.
///
/// A run that accepted something must have staged every mutation —
/// applied, or refused for want of material in the accepted shape — so
/// a wave that silently dropped an arm fails here rather than reporting
/// a smaller matrix. What each mutation actually produced is written to
/// the transcript and asserted nowhere.
fn check_negative_half(transcript: &OperationTranscript, accepted: usize) {
    if accepted > 0 {
        assert_eq!(
            transcript.mutants().len(),
            vectors::mutation::NegativeMutation::ALL.len(),
            "a mutation went unstaged"
        );
    } else {
        assert!(
            transcript.mutants().is_empty(),
            "mutations were staged from a transaction no target accepted"
        );
    }

    // Every mutation that was actually submitted carries the bytes it
    // was submitted as. A row with no bytes and a target verdict would
    // claim an answer to something nobody sent.
    for mutant in transcript.mutants() {
        if mutant.layer() == ObservedOutcomeLayer::FixtureConstructionFailure {
            continue;
        }
        assert!(
            !mutant.bytes().is_empty(),
            "a submitted mutation recorded no bytes: {:?}",
            mutant.mutation()
        );
    }
}
