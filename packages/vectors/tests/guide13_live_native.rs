//! The live-transfer lane: one candidate run against a real node.
//!
//! # Why this is an ignored test rather than a gate
//!
//! It needs a live Elements node, so it cannot run in an ordinary lane,
//! and what it produces is a transcript rather than a verdict. The whole
//! workspace suite stays green without a node present, which is what the
//! `#[ignore]` buys: the evidence this wave commits is the *report*
//! artifacts and the tests that validate them, and none of those needs
//! the node to be re-run.
//!
//! Run it as:
//!
//! ```text
//! TRIPOD_LIVE_EXECUTOR=<adapter> \
//! TRIPOD_LIVE_NETWORK_ID=<64 hex> \
//! TRIPOD_LIVE_GENESIS_ID=<64 hex> \
//! TRIPOD_LIVE_REPORT=<path> \
//!   cargo test -p tripod-vectors --test guide13_live_native -- --ignored --nocapture
//! ```
//!
//! # Nothing here decides what the run should have found
//!
//! The assertions are about the *shape* of a run that completed: that the
//! ceremony reached the node, that every step it asked for was answered,
//! and that the transcript records an observation per step. What the node
//! decided is written down and asserted nowhere — a lane that asserted a
//! verdict would fail rather than report when the honest answer changed
//! `(´[PLAN-rule:guide12-exec:failure-layers]´)`.
//!
//! # And nothing here discharges a matrix row
//!
//! The submitted transfers carry a witness whose signature position holds
//! bytes that authorize nothing, because no first-party component
//! computes the digest the target's verifying primitive forms. Every
//! rejection is attributable to that rather than to the row a reader
//! might wish it were about, and `vectors::live_evidence` records the
//! whole matrix as blocked for exactly that reason.

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
use vectors::live_native::{LiveNativeStep, LiveTransferOperationPlanner, render_live_native_run};

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
/// Beside the report and never inside it (§13.6). Duration is a property
/// of the machine the run happened on rather than of the run's result.
fn timing_path(report: &Path) -> PathBuf {
    let mut path = report.to_path_buf();
    let mut name = path.file_name().map_or_else(
        || "live-native-run".to_owned(),
        |name| name.to_string_lossy().into_owned(),
    );
    name.push_str(".timing");
    path.set_file_name(name);
    path
}

#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_live_transfer_candidate_runs_against_a_real_target() {
    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let report = environment("TRIPOD_LIVE_REPORT")
        .map(PathBuf::from)
        .expect("TRIPOD_LIVE_REPORT names where the transcript is written");

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

    let timeout = environment("TRIPOD_LIVE_TIMEOUT_SECONDS")
        .and_then(|value| value.parse::<u64>().ok())
        .map_or(DEFAULT_EXECUTOR_TIMEOUT, Duration::from_secs);
    let configuration = ExecutorConfiguration::new(
        Path::new(&executor),
        // The operator's declaration about the program they selected. It
        // establishes nothing about it, and this lane produces a
        // transcript rather than a gate verdict.
        ExecutorTrust::ReviewedNonMock,
        timeout,
        ExecutorDiagnostics::in_directory(report.parent().unwrap_or_else(|| Path::new("."))),
    );

    let mut planner = LiveTransferOperationPlanner::new().expect("the planner builds");
    let started = Instant::now();
    let outcome = execute_operations(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let transcript = planner.transcript();
    let rendered = render_live_native_run(transcript);
    std::fs::write(&report, &rendered).expect("the transcript is written");
    std::fs::write(
        timing_path(&report),
        format!("wall_seconds {:.1}\n", wall.as_secs_f64()),
    )
    .expect("the run's wall time is written");
    // A refused run is written down beside the transcript rather than
    // printed: the file is the artifact, and a lane whose only record was
    // captured console output would leave nothing behind.
    if let Err(error) = &outcome {
        std::fs::write(
            report.with_extension("executor-refusal"),
            format!("{error}\n"),
        )
        .expect("the executor's refusal is written");
    }

    // A run that reached the node at all funded both constructors and
    // recorded an observation for every step it got an answer to. What
    // those answers were is written down and asserted nowhere.
    if outcome.is_ok() {
        // Every step is accounted for: it produced an observation, or the
        // plan recorded why its form was never submitted. A step that was
        // neither would be one the run quietly dropped.
        assert_eq!(
            transcript.observations().len() + transcript.not_submitted().len(),
            LiveNativeStep::ALL.len(),
            "a step was neither observed nor accounted for",
        );
        for step in LiveNativeStep::ALL {
            let submitted = transcript.observation(*step).is_some();
            let accounted = match step {
                LiveNativeStep::SubmitPrivateTransfer => transcript
                    .gap_for(linker::live_backend::LiveTransferRepresentationPlan::PrivateCommitted)
                    .is_some(),
                _ => false,
            };
            assert!(
                submitted || accounted,
                "{step:?} produced neither an observation nor a stated gap",
            );
        }
        // The deployment was welded to the chain before anything was
        // funded, so every program the ceremony paid to belongs to a
        // deployment of the asset the target issued.
        assert!(transcript.relinked(), "the run funded before it linked");
        // The ceremony materialized predecessors at both constructors, or
        // it did not reach the submissions at all — and either way the
        // transcript says which rather than reporting a smaller run.
        let materialized = transcript.materialized();
        assert_ne!(
            materialized.values().sum::<usize>(),
            0,
            "the ceremony created no predecessor at any constructor",
        );
        assert!(transcript.issued_asset().is_some());
    }

    // The run discharges nothing, and the rendering says so in its own
    // bytes rather than leaving a reader to infer it.
    assert!(rendered.contains("discharges_no_matrix_row true"));
}
