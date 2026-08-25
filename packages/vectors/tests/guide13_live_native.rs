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

/// Every line one validated record contributes to the transcript.
///
/// Split out because the census is eight members over two outputs and a
/// function that both ran a ceremony and rendered it would be two
/// functions sharing a name.
fn census_lines(
    record: &target_elements_conformance::confidential_record::ConfidentialFundingRecord,
    attempt: u8,
) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!("run {attempt} handle {}", record.fixture_handle()));
    lines.push(format!(
        "run {attempt} contract {}",
        record.summary().contract().code()
    ));
    lines.push(format!(
        "run {attempt} parities {:?}",
        record.summary().observed_parities()
    ));
    lines.push(format!(
        "run {attempt} protocol_outputs {} non_protocol_members {}",
        record.summary().protocol_outputs(),
        record.summary().non_protocol_members()
    ));
    for census in record.agreement() {
        for entry in census.fields() {
            lines.push(format!(
                "run {attempt} output {} field {} {} {} vs {}",
                census.output(),
                entry.field(),
                if entry.agrees() { "agree" } else { "disagree" },
                entry.expectation(),
                entry.observation(),
            ));
        }
    }
    for (index, check) in record.independent_commitments().iter().enumerate() {
        lines.push(format!(
            "run {attempt} output {index} independent_commitment {} checker {}",
            if check.agrees() { "agree" } else { "disagree" },
            check.checker(),
        ));
    }
    lines.push(format!(
        "run {attempt} witness_transaction_id {}",
        record.readback().witness_transaction_id
    ));

    lines
}

/// What one confidential ceremony produced: its report lines and the
/// mined bytes a second run is compared against.
struct ConfidentialAttempt {
    lines: Vec<String>,
    mined: Vec<u8>,
}

/// Runs one confidential predecessor ceremony end to end and validates it.
///
/// Split out of the test because the byte-identity comparison is a claim
/// about two runs: the test runs this twice and compares, and this
/// function knows nothing about the comparison it will be part of.
fn fund_one_confidential_predecessor(
    target: &target_elements::ReviewedElementsTapscriptDefinition,
    binding: &target_elements::ReviewedDevelopmentBinding,
    configuration: &ExecutorConfiguration,
    attempt: u8,
) -> ConfidentialAttempt {
    use target_elements_conformance::confidential_funding::{
        ConfidentialFundingOracles, ConfidentialReadbackDecoder as _,
    };
    use target_elements_conformance::confidential_oracles::{
        ReferenceRangeproofVerifier, ReferenceReadbackDecoder,
    };
    use target_elements_conformance::confidential_record::{
        ConfidentialFundingEvidence, FieldAgreement, FundingAgreementField,
        validate_confidential_funding_record,
    };
    use vectors::confidential_predecessor::{
        AdapterReportedInclusion, ConfidentialPredecessorPlan,
    };

    let mut lines = Vec::new();
    let mut plan = ConfidentialPredecessorPlan::new();
    let started = Instant::now();
    let outcome = execute_operations(target, binding, configuration, &mut plan);
    lines.push(format!(
        "run {attempt} wall_seconds {:.1}",
        started.elapsed().as_secs_f64()
    ));

    // A stopped result is a valid outcome and is written as one. It is
    // an infrastructure or construction fact and never a target verdict.
    let transcript = match outcome {
        Ok(transcript) => transcript,
        Err(error) => {
            lines.push(format!("run {attempt} executor_refused {error}"));
            if let Some(refusal) = plan.refusal() {
                lines.push(format!("run {attempt} plan_refused {refusal:?}"));
            }
            panic!("the confidential predecessor ceremony did not reach the target: {error}");
        }
    };

    let case = ConfidentialPredecessorPlan::funding_case();
    let response = transcript
        .operation_responses()
        .get(&case)
        .expect("the confidential step was answered");
    let subject = plan.subject().expect("the confidential subject was sent");
    let registry = plan.registry().expect("the fixture was registered");
    let fixture = registry
        .resolve(
            &subject.binding.fixture_handle,
            &subject.binding.fixture_digest,
        )
        .expect("the registry resolves its own case");

    let readback = response
        .mined_readback
        .as_ref()
        .expect("the answer carries a mined readback");
    let reader = ReferenceReadbackDecoder::new();
    let mined = reader
        .decode(&readback.raw_transaction)
        .expect("the mined bytes decode");
    let inclusion = AdapterReportedInclusion::recomputed_from(mined.transaction_id.clone());
    let verifier = ReferenceRangeproofVerifier::new();
    let oracles = ConfidentialFundingOracles {
        decoder: &reader,
        inclusion: &inclusion,
        rangeproofs: &verifier,
    };
    let evidence = ConfidentialFundingEvidence {
        handshake: transcript.handshake(),
        environment: transcript.environment(),
        request: subject,
        response,
        fixture,
        decoded: &mined,
        // Never stated here. A byte comparison is the other contract's
        // own claim, and this function has seen one run.
        materialized_bytes_compared: false,
    };
    let validated = validate_confidential_funding_record(&evidence, &oracles)
        .expect("the confidential funding record validates");
    let record = validated.record();

    lines.extend(census_lines(record, attempt));

    assert_eq!(record.agreement().len(), 2);
    for census in record.agreement() {
        assert_eq!(census.fields().len(), FundingAgreementField::ALL.len());
        assert!(census.fields().iter().all(FieldAgreement::agrees));
    }
    assert_eq!(record.summary().observed_parities(), &[0x08, 0x09]);
    assert_eq!(record.non_claims().len(), 10);

    ConfidentialAttempt {
        lines,
        mined: readback.raw_transaction.clone(),
    }
}

/// One confidential predecessor, funded, mined, and read back.
///
/// # What this run establishes, and the six things it reports
///
/// Confidential funding capability and schema negotiation; deterministic
/// materialization under the selected custody profile; the exact hybrid
/// representation of both mined outputs; both accepted commitment
/// parities; valid proof-bearing predecessor outputs and target
/// readback; and a stable opening reference for a later transaction-wide
/// materializer. It reports no transfer, no authorization, no CT
/// conservation, no acceptance of any safety row, no minimality, no
/// production privacy, and no matrix discharge.
///
/// # Why it runs the ceremony twice
///
/// The selected reproducibility contract is byte identity, and byte
/// identity is a claim about two runs rather than about one. A single
/// run that reported deterministic bytes would be reporting a property
/// it never observed, so the ceremony runs twice against two disposable
/// chains and the funding transactions are compared byte for byte. A
/// difference is written down as a difference; it is never a downgrade
/// to the other contract.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn one_confidential_predecessor_is_funded_mined_and_read_back() {
    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let base = environment("TRIPOD_LIVE_REPORT")
        .map(PathBuf::from)
        .expect("TRIPOD_LIVE_REPORT names where the transcript is written");
    let report = base.with_extension("confidential-predecessor");

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
        ExecutorTrust::ReviewedNonMock,
        timeout,
        ExecutorDiagnostics::in_directory(report.parent().unwrap_or_else(|| Path::new("."))),
    );

    let started = Instant::now();
    let first = fund_one_confidential_predecessor(&target, &binding, &configuration, 0);
    let second = fund_one_confidential_predecessor(&target, &binding, &configuration, 1);

    let identical = first.mined == second.mined;
    let mut lines = first.lines;
    lines.extend(second.lines);
    lines.push(format!("byte_identity_satisfied {identical}"));
    lines.push(format!(
        "total_wall_seconds {:.1}",
        started.elapsed().as_secs_f64()
    ));
    lines.push("discharges_no_matrix_row true".to_owned());
    lines.push("clears_owner_sighash_blockers false".to_owned());
    std::fs::write(&report, lines.join("\n") + "\n").expect("the transcript is written");

    assert!(
        identical,
        "the run selected byte identity and did not achieve it",
    );
}

/// One owner authorization, produced against the Wave-2 message and
/// observed on the explicit lane.
///
/// # What this run is for
///
/// The accepted evidence ruling names three things a dimension needs: a
/// source citation, an independent recomputation, and one observed
/// acceptance. The first two are landed. This is the third, and it is
/// the only thing in this file that submits a candidate carrying a
/// signature rather than bytes that authorize nothing.
///
/// # What it asserts, and what it merely records
///
/// It asserts the *shape* of a completed ceremony: that every case was
/// submitted and answered, and that an acceptance carried a readback to
/// check. What each case's layer was is written into the artifact and
/// asserted nowhere — a lane that asserted a verdict would fail rather
/// than report when the honest answer changed.
///
/// The one thing it does assert about content is the two-origin
/// agreement, and only where an acceptance was observed: a run that
/// accepted a candidate and then could not verify its own witness
/// against its own recomputed message has found something, and it must
/// say so by failing rather than by writing a false line.
///
/// # It clears nothing by running
///
/// A blocker moves on an observed result and never on a capability
/// existing. This test existing establishes nothing at all; what
/// establishes anything is the artifact one run of it produced.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn one_owner_authorization_is_observed_on_the_explicit_lane() {
    use vectors::live_owner_observation::{
        OwnerObservationCase, OwnerObservationPlanner, render_owner_observation,
    };

    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let base = environment("TRIPOD_LIVE_REPORT")
        .map(PathBuf::from)
        .expect("TRIPOD_LIVE_REPORT names where the transcript is written");
    let report = base.with_extension("owner-observation");

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
        ExecutorTrust::ReviewedNonMock,
        timeout,
        ExecutorDiagnostics::in_directory(report.parent().unwrap_or_else(|| Path::new("."))),
    );

    // The genesis block hash the message hasher is seeded with, taken
    // from the run's own deployment binding rather than from anything a
    // candidate carries: no candidate determines it, and two identical
    // candidates on two chains have different messages.
    let mut planner =
        OwnerObservationPlanner::new(identifier(&genesis)).expect("the ceremony builds");
    let started = Instant::now();
    let outcome = execute_operations(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    let rendered = render_owner_observation(record);
    std::fs::write(&report, &rendered).expect("the transcript is written");
    std::fs::write(
        timing_path(&report),
        format!("wall_seconds {:.1}\n", wall.as_secs_f64()),
    )
    .expect("the run's wall time is written");
    if let Err(error) = &outcome {
        std::fs::write(
            report.with_extension("executor-refusal"),
            format!("{error}\n"),
        )
        .expect("the executor's refusal is written");
    }

    outcome.expect("the ceremony reached the target");

    // Every case was submitted and answered. A case that was neither is
    // one the ceremony quietly dropped, and a run that dropped a control
    // would be reporting a narrower comparison than it claims.
    assert_eq!(
        record.observations().len(),
        OwnerObservationCase::ALL.len(),
        "a case was not submitted",
    );
    assert!(record.relinked(), "the ceremony funded before it linked");
    assert_eq!(record.coins().len(), 2);

    // The signed-over spent-output triple is the node's own report of
    // the coins, and the ceremony's expectation agreed with it. A
    // disagreement is a finding about the funding boundary rather than
    // about the message, and it must not pass silently.
    assert!(
        record
            .coins()
            .iter()
            .all(vectors::live_owner_observation::ObservedFundedCoin::matches_expectation),
        "the node reported a coin the ceremony did not ask for",
    );

    // Every case's message was computed, and the empty-vector control's
    // message differs from the selected profile's. Two candidates that
    // coincided would make any verdict about which one a signature
    // verifies against a coincidence.
    let messages = record.candidate_messages();
    assert_eq!(messages.len(), OwnerObservationCase::ALL.len());
    assert_ne!(
        messages.get(&OwnerObservationCase::SelectedProfile),
        messages.get(&OwnerObservationCase::EmptyOutputWitnessVector),
        "the two candidate messages coincided",
    );

    // The two origins, where an acceptance was observed. This is the
    // only content assertion in the file, and it is here because a run
    // that accepted a candidate and then failed to verify its own
    // witness against its own recomputed message has found something.
    if let Some(check) = record.reverification() {
        assert!(
            check.readback_matches_submission(),
            "the bytes the node reported are not the bytes it was handed",
        );
        assert!(
            check.verified().is_ok(),
            "the accepted witness does not verify against the recomputed message: {:?}",
            check.verified(),
        );
        assert!(
            !check.verifies_against_empty_vector_message(),
            "the accepted witness verifies against both candidate messages",
        );
    }

    // The run says in its own bytes what it did not establish.
    assert!(rendered.contains("establishes_the_proof_bearing_lane false"));
    assert!(rendered.contains("clears_sighash_profile_unreviewed false"));
    assert!(rendered.contains("discharges_no_matrix_row true"));
}

/// One owner authorization observed on the PROOF-BEARING lane.
///
/// # What this run is for
///
/// The explicit-lane observation was forbidden to say anything about a
/// candidate whose outputs carry range proofs, because the term that
/// makes an explicit candidate authorizable from a preimage — an
/// output-witness vector recoverable from the output count — has no
/// counterpart there. This asks the same question where the vector
/// carries real proofs.
///
/// # What belongs to the other guide
///
/// The confidential predecessor is funded by that guide's own ceremony
/// machinery and adapter, and the candidate is frozen by that guide's
/// transaction-wide materializer. Neither is evidenced here, and the
/// record says so in its own bytes rather than leaving it to this
/// comment.
///
/// # What it asserts, and what it merely records
///
/// The shape of a completed ceremony: every case submitted and
/// answered, the vector at its real proof-bearing length, both
/// construction controls refused before any message was formed, and the
/// two-origin agreement where an acceptance was observed. What each
/// case's layer WAS is written into the artifact and asserted nowhere.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn one_owner_authorization_is_observed_on_the_proof_bearing_lane() {
    use vectors::live_proof_bearing_observation::{
        ProofBearingObservationPlanner, render_proof_bearing_observation,
    };

    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let base = environment("TRIPOD_LIVE_REPORT")
        .map(PathBuf::from)
        .expect("TRIPOD_LIVE_REPORT names where the transcript is written");
    let report = base.with_extension("proof-bearing-observation");

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
        ExecutorTrust::ReviewedNonMock,
        timeout,
        ExecutorDiagnostics::in_directory(report.parent().unwrap_or_else(|| Path::new("."))),
    );

    let mut planner =
        ProofBearingObservationPlanner::new(identifier(&genesis)).expect("the ceremony builds");
    let started = Instant::now();
    let outcome = execute_operations(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    let rendered = render_proof_bearing_observation(record);
    std::fs::write(&report, &rendered).expect("the transcript is written");
    std::fs::write(
        timing_path(&report),
        format!("wall_seconds {:.1}\n", wall.as_secs_f64()),
    )
    .expect("the run's wall time is written");
    if let Err(error) = &outcome {
        std::fs::write(
            report.with_extension("executor-refusal"),
            format!("{error}\n"),
        )
        .expect("the executor's refusal is written");
    }

    outcome.expect("the ceremony reached the target");

    check_proof_bearing_record(record, &rendered);
}

/// Everything the completed proof-bearing ceremony owes its reader.
///
/// Split from the test body because the run's setup and the run's
/// checks are two different readings, and a body that outgrew a hundred
/// lines is one nobody reviews as a whole. Nothing moved into here
/// decides what the target should have found: every assertion is about
/// the SHAPE of a completed ceremony, and the one content assertion is
/// the two-origin agreement.
fn check_proof_bearing_record(
    record: &vectors::live_proof_bearing_observation::ProofBearingObservationRecord,
    rendered: &str,
) {
    use vectors::live_proof_bearing_observation::{
        ProofBearingCase, ProofBearingConstructionControl,
    };

    // Every case was submitted and answered, and both construction
    // controls fired. A control that quietly did not run is a narrower
    // comparison than the report claims.
    assert_eq!(
        record.observations().len(),
        ProofBearingCase::ALL.len(),
        "a case was not submitted",
    );
    assert_eq!(
        record.construction_refusals().len(),
        ProofBearingConstructionControl::ALL.len(),
        "a construction control did not refuse",
    );

    // The predecessor is confidential and the node's report of it is
    // what the ceremony asked for. A divergence is a finding about the
    // funding boundary rather than about the message.
    assert_eq!(record.coins().len(), 2);
    assert!(
        record.coins().iter().all(
            vectors::live_proof_bearing_observation::ObservedConfidentialCoin::matches_expectation
        ),
        "the node reported a confidential coin the ceremony did not ask for",
    );

    // The deliverable's own figure: the census was built at the vector's
    // REAL proof-bearing length, and every entry carries a proof rather
    // than the two zero bytes the explicit lane's entries carry.
    assert_eq!(record.output_witness_vector_length(), Some(2));
    assert_eq!(record.output_witness_proof_bytes().len(), 2);
    assert!(
        record
            .output_witness_proof_bytes()
            .iter()
            .all(|bytes| *bytes > 2),
        "an output-witness entry carried no range proof: {:?}",
        record.output_witness_proof_bytes(),
    );

    // Every case's message was computed, and the two witness-vector
    // controls' messages differ from the selected profile's and from
    // each other. Candidates that coincided would make any verdict about
    // which one a signature verifies against a coincidence.
    let messages = record.candidate_messages();
    assert_eq!(messages.len(), ProofBearingCase::ALL.len());
    let selected = messages.get(&ProofBearingCase::SelectedProfile);
    assert_ne!(
        selected,
        messages.get(&ProofBearingCase::ProofBearingVectorEmptied),
    );
    assert_ne!(
        selected,
        messages.get(&ProofBearingCase::PreimageOnlySigner)
    );
    assert_ne!(
        messages.get(&ProofBearingCase::ProofBearingVectorEmptied),
        messages.get(&ProofBearingCase::PreimageOnlySigner),
    );
    assert_ne!(
        selected,
        messages.get(&ProofBearingCase::AnotherProofBearingCandidate),
    );

    // The two origins, where an acceptance was observed.
    if let Some(check) = record.reverification() {
        assert!(
            check.readback_matches_submission(),
            "the bytes the node reported are not the bytes it was handed",
        );
        assert!(
            check.verified().is_ok(),
            "the accepted witness does not verify against the recomputed message: {:?}",
            check.verified(),
        );
        assert!(
            !check.verifies_against_emptied_vector_message(),
            "the accepted witness verifies against both candidate messages",
        );
    }

    // The run says in its own bytes what it did not establish.
    assert!(rendered.contains("evidences_the_other_guides_funding false"));
    assert!(rendered.contains("evidences_the_other_guides_materialization false"));
    assert!(rendered.contains("evidences_the_other_guides_blinding false"));
    assert!(rendered.contains("evidences_the_receipt_covenant false"));
    assert!(rendered.contains("discharges_no_matrix_row true"));
}

/// The restart order's first step, against a real node.
///
/// # What this run is for
///
/// One accepted sponsorless private one-to-one control, which is the
/// entry condition for every later step of the mandatory restart order
/// (task:guide-ctf-exec:restart-order). Nothing else runs here: no
/// negative case, no mutation, no parity pair, because the order forbids
/// them until this one accepts.
///
/// # What it asserts, and what it merely records
///
/// It asserts the shape of a completed ceremony — that the run reached
/// the node, that the confidential funding step created the predecessor
/// the ceremony asked for, and that a candidate was built and submitted.
/// What the node decided is written into the artifact and asserted
/// nowhere: a lane that asserted an acceptance would fail rather than
/// report on the day the honest answer changed.
///
/// The one content assertion is the two-origin agreement, and only where
/// an acceptance was observed. A run that accepted a candidate and could
/// not verify the witness it read back against its own recomputed
/// message has found something, and must say so by failing.
///
/// # It moves nothing by running
///
/// A row moves on an observed acceptance, and the observation is the
/// artifact this produces rather than the existence of this test.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn one_private_one_to_one_control_is_submitted_to_a_real_target() {
    use vectors::live_private_restart::ConsumedReceipt;

    run_one_private_control(ConsumedReceipt::Primary, "private-restart-control");
}

/// The restart order's second step: the other predecessor commitment
/// parity, in a complete accepted successor.
///
/// # What this run is for, and why it is a second run
///
/// Step two asks for both predecessor commitment parities exercised in
/// complete accepted successors. The two parities are carried by the
/// predecessor's two outputs, so the honest way to exercise both is to
/// consume each of them in its own complete successor rather than to
/// assert that a candidate touching both must have covered them.
///
/// Its entry condition is step one's observed acceptance, which is why
/// it is a separate test and not a loop: a lane that ran both and
/// reported one number could not say which of them the order was
/// entitled to.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_other_commitment_parity_is_exercised_in_a_complete_successor() {
    use vectors::live_private_restart::ConsumedReceipt;

    run_one_private_control(ConsumedReceipt::Balancing, "private-restart-parity");
}

/// One private control, consuming one named predecessor output.
fn run_one_private_control(
    consumed: vectors::live_private_restart::ConsumedReceipt,
    extension: &str,
) {
    use vectors::live_private_restart::{PrivateRestartPlanner, render_private_restart};

    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let base = environment("TRIPOD_LIVE_REPORT")
        .map(PathBuf::from)
        .expect("TRIPOD_LIVE_REPORT names where the transcript is written");
    let report = base.with_extension(extension);

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
        ExecutorTrust::ReviewedNonMock,
        timeout,
        ExecutorDiagnostics::in_directory(report.parent().unwrap_or_else(|| Path::new("."))),
    );

    let mut planner = PrivateRestartPlanner::spending(identifier(&genesis), consumed)
        .expect("the restart ceremony builds");
    let started = Instant::now();
    let outcome = execute_operations(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    let rendered = render_private_restart(record);
    std::fs::write(&report, &rendered).expect("the transcript is written");
    std::fs::write(
        timing_path(&report),
        format!("wall_seconds {:.1}\n", wall.as_secs_f64()),
    )
    .expect("the run's wall time is written");
    if let Err(error) = &outcome {
        std::fs::write(
            report.with_extension("executor-refusal"),
            format!("{error}\n"),
        )
        .expect("the executor's refusal is written");
    }

    // A construction refusal is a valid outcome and is written down as
    // one. It is never a target verdict, so it is reported and the test
    // stops here rather than pretending the node said anything.
    if let Some(refusal) = record.refusal() {
        panic!("the restart ceremony refused before the node: {refusal:?}");
    }
    outcome.expect("the ceremony reached the target");

    // The predecessor is confidential and is the one the ceremony asked
    // for. A divergence is a finding about the funding boundary.
    assert_eq!(record.coins().len(), 2);
    assert!(
        record
            .coins()
            .iter()
            .all(vectors::live_private_restart::RestartConfidentialCoin::matches_expectation),
        "the node reported a confidential coin the ceremony did not ask for",
    );

    // One receipt consumed, and its outputs carry real proofs.
    assert_eq!(record.receipt_leaves(), 1);
    assert_eq!(record.output_witness_proof_bytes().len(), 2);
    assert!(
        record
            .output_witness_proof_bytes()
            .iter()
            .all(|bytes| *bytes > 2),
        "an output-witness entry carried no range proof: {:?}",
        record.output_witness_proof_bytes(),
    );

    // The candidate reached the node.
    assert!(record.submitted_bytes() > 0);
    assert!(record.observed_layer().is_some(), "no layer was observed");

    // The two origins, where an acceptance was observed.
    if let Some(check) = record.reverification() {
        assert!(
            check.readback_matches_submission(),
            "the bytes the node reported are not the bytes it was handed",
        );
        assert!(
            check.verified(),
            "the accepted witness does not verify against the recomputed message",
        );
    }

    // The parity this run exercised is the node's answer about the coin
    // it consumed, and it is one of the two the target admits.
    let prefix = record
        .consumed_commitment_prefix()
        .expect("a consumed confidential coin carries a commitment prefix");
    assert!(
        prefix == 0x08 || prefix == 0x09,
        "the node reported a commitment prefix outside the admitted pair: {prefix:#04x}",
    );

    // The run says in its own bytes what it did not establish.
    assert!(rendered.contains("evidences_no_negative_case true"));
    assert!(rendered.contains("moves_the_sponsor_row false"));
}

/// The restart order's third and fourth steps: target CT conservation
/// recorded against a balance-valid control, and the three proof-negatives
/// run from that control.
///
/// # What this run is for
///
/// Step three records the target's own commitment-balance rule accepting a
/// conserving private transaction, and step four mutates one field of that
/// same control, one case at a time, and observes the target refuse each.
/// The two are one ceremony because they share a control: the control step
/// three records the conservation of is the control step four mutates.
///
/// # What it asserts, and what it merely records
///
/// What the target DECIDED — whether it accepted the control or refused the
/// mutants — is written into the artifact and asserted nowhere, so a lane
/// that asserted an acceptance would fail rather than report on the day the
/// honest answer changed. Two things ARE asserted, both first-party
/// construction facts rather than target verdicts: where the control was
/// accepted, the two-origin agreement holds; and where a balance-valid
/// control exists, each proof-negative is confined to its declared field,
/// which is what makes its refusal attributable to that field and to
/// nothing else.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn conservation_is_recorded_against_a_control_the_proof_negatives_mutate() {
    use vectors::live_conservation_negatives::{
        ConservationNegativePlanner, render_conservation_negatives,
    };

    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let report = environment("TRIPOD_LIVE_REPORT")
        .map(PathBuf::from)
        .expect("TRIPOD_LIVE_REPORT names where the transcript is written")
        .with_extension("conservation-negatives");

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
        ExecutorTrust::ReviewedNonMock,
        timeout,
        ExecutorDiagnostics::in_directory(report.parent().unwrap_or_else(|| Path::new("."))),
    );

    let mut planner =
        ConservationNegativePlanner::new(identifier(&genesis)).expect("the ceremony builds");
    let started = Instant::now();
    let outcome = execute_operations(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    let rendered = render_conservation_negatives(record);
    std::fs::write(&report, &rendered).expect("the transcript is written");
    std::fs::write(
        timing_path(&report),
        format!("wall_seconds {:.1}\n", wall.as_secs_f64()),
    )
    .expect("the run's wall time is written");
    if let Err(error) = &outcome {
        std::fs::write(
            report.with_extension("executor-refusal"),
            format!("{error}\n"),
        )
        .expect("the executor's refusal is written");
    }

    // A construction refusal is a valid outcome and is written down as one.
    // It is never a target verdict, so it is reported and the test stops
    // here rather than pretending the node said anything.
    if let Some(refusal) = record.refusal() {
        panic!("the conservation ceremony refused before the node: {refusal:?}");
    }
    outcome.expect("the ceremony reached the target");

    // Every one of the three proof-negatives was submitted and answered.
    assert_eq!(record.mutants().len(), 3);
    for mutant in record.mutants() {
        assert!(
            mutant.observed_layer().is_some(),
            "the {} mutant was not answered",
            mutant.case().name(),
        );
    }

    // The control was submitted and answered.
    assert!(
        record.control_observed_layer().is_some(),
        "the control was not answered",
    );

    // The two origins, where an acceptance was observed.
    if let Some(check) = record.reverification() {
        assert!(
            check.readback_matches_submission(),
            "the bytes the node reported are not the bytes it was handed",
        );
        assert!(
            check.verified(),
            "the accepted witness does not verify against the recomputed message",
        );
    }

    // Where the control was accepted, its commitment balance was checked by
    // the target, so a balance-valid control exists and every mutant must
    // attribute to its own declared field. This is a first-party property
    // of the construction — the mutant changed one field and nothing else —
    // and not a claim about the target's verdict.
    if let Some(control) = record.balance_valid_control() {
        for mutant in record.mutants() {
            mutant.attribute(&control).unwrap_or_else(|refusal| {
                panic!(
                    "the {} mutant is not confined to its declared field: {refusal:?}",
                    mutant.case().name(),
                )
            });
        }
    }

    // The run says in its own bytes what it did not establish.
    assert!(rendered.contains("moves_the_sponsor_row false"));
}

/// One KEY-PATH spend attempt against a funded explicit constructor.
///
/// # What this run is for
///
/// Every constructor here is spent by its script path, under an internal
/// key that is a published nothing-up-my-sleeve point. Nobody had ever
/// offered this target a one-item witness at one of these programs, so
/// nothing was known about what layer such a candidate lands at, what
/// the node says about it, or whether the generic submission wire
/// carries it at all. This asks.
///
/// # What a refusal discharges
///
/// That the attempt was observed and refused. Nothing else. The
/// signature offered is by a published test key that is not the output
/// key, so a target refusing it is refusing a signature that does not
/// verify — which says nothing whatever about who knows the internal
/// key's discrete logarithm. The residual assumption stands regardless,
/// and the artifact says so in its own bytes.
///
/// # What it asserts, and what it merely records
///
/// The shape of a completed attempt: the deployment relinked before it
/// funded, the node's own fields agreed with the ceremony's expectation,
/// the constructor's internal key is the published point, the witness is
/// the one-item shape, and the signing key is not the output key. What
/// the target DECIDED is written into the artifact and asserted nowhere.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn one_key_path_spend_attempt_is_offered_to_a_real_target() {
    use vectors::live_keypath_probe::{KeyPathProbePlanner, ProbeProvenance, render_keypath_probe};

    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let base = environment("TRIPOD_LIVE_REPORT")
        .map(PathBuf::from)
        .expect("TRIPOD_LIVE_REPORT names where the transcript is written");
    let report = base.with_extension("keypath-probe");

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
        ExecutorTrust::ReviewedNonMock,
        timeout,
        ExecutorDiagnostics::in_directory(report.parent().unwrap_or_else(|| Path::new("."))),
    );

    // The target's provenance, as the run's own environment reports it.
    // The declared source tip is the operator's declaration about the
    // binary the adapter was pointed at; this probe records it and
    // verifies nothing about it, which is what its name says.
    let provenance = ProbeProvenance {
        network_id: network,
        genesis_id: genesis.clone(),
        target_version: format!("{:?}", target.definition().version()),
        declared_source_tip: environment("ELEMENTS_NATIVE_EXECUTOR_INTENDED_TIP"),
        executor_trust: format!("{:?}", ExecutorTrust::ReviewedNonMock),
    };

    let mut planner =
        KeyPathProbePlanner::new(identifier(&genesis), provenance).expect("the ceremony builds");
    let started = Instant::now();
    let outcome = execute_operations(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    let rendered = render_keypath_probe(record);
    std::fs::write(&report, &rendered).expect("the transcript is written");
    std::fs::write(
        timing_path(&report),
        format!("wall_seconds {:.1}\n", wall.as_secs_f64()),
    )
    .expect("the run's wall time is written");
    if let Err(error) = &outcome {
        std::fs::write(
            report.with_extension("executor-refusal"),
            format!("{error}\n"),
        )
        .expect("the executor's refusal is written");
    }

    outcome.expect("the ceremony reached the target");

    // The deployment was welded to the chain before anything was funded,
    // so the program the attempt spends belongs to a deployment of the
    // asset the target issued.
    assert!(record.relinked(), "the ceremony funded before it linked");
    assert_eq!(record.coins().len(), 1, "the probe funds exactly one coin");
    assert!(
        record
            .coins()
            .iter()
            .all(vectors::live_owner_observation::ObservedFundedCoin::matches_expectation),
        "the node reported a coin the ceremony did not ask for",
    );

    // The constructor under probe is the one this run is about. A
    // deployment whose internal key was not the published point would
    // make the whole record a report on a different question.
    let program = record
        .binding()
        .expect("the ceremony carries the constructor it funded");
    assert!(
        program.internal_key_is_the_published_point(),
        "the funded constructor did not inherit the published internal key",
    );

    // The attempt was built and offered, and it is the shape the run
    // claims: one witness item, and a signing key that is not the output
    // key the program carries. The second is the assertion that keeps
    // the refusal honest — an attempt signed by the output key would be
    // a different experiment entirely.
    let attempt = record
        .attempt()
        .expect("the ceremony built the key-path attempt");
    assert!(
        attempt.is_the_one_item_shape(),
        "the witness is not the one-item key-path shape",
    );
    assert_ne!(
        program.output_key().as_slice(),
        attempt.signing_public_key().as_slice(),
        "the attempt was signed by the output key, which is not this probe",
    );
    // The submitted bytes carry a whole transaction and not only the
    // witness item, which is the cheapest check that the attempt was
    // assembled rather than merely signed.
    assert!(
        attempt.submitted_bytes().len() > attempt.witness_stack()[0].len(),
        "the submitted bytes are no larger than the witness item",
    );

    // The target answered. What it answered is recorded and asserted
    // nowhere: a probe that panicked on an unexpected layer would hide
    // the one outcome most worth reading.
    assert!(
        record.observation().is_some(),
        "the attempt was not answered",
    );

    // The run says in its own bytes what it did not establish.
    assert!(rendered.contains("residual_internal_key_unspendability_stands true"));
    assert!(rendered.contains("discharges_no_matrix_row true"));
}

/// The restart order's fifth step, the split shape: one receipt in, three
/// outputs, against a real node.
///
/// # What this run is for
///
/// Step five asks for the remaining positive private shapes, each where it
/// accepts. This is the split: one confidential receipt consumed and split
/// into two recipients and a balancing change output, the smallest of the
/// remaining shapes and the one that needs only a three-output successor
/// and one input. Its acceptance moves the `private-split` matrix row, and
/// the move is the observed identity this run records, not the fact of the
/// test existing.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_split_shape_is_submitted_to_a_real_target() {
    use vectors::live_multi_shapes::PrivateShape;

    run_one_multi_shape(PrivateShape::Split, "multi-split");
}

/// The restart order's fifth step, the many-to-many shape: two receipts in,
/// three outputs, against a real node.
///
/// # Why this representative case
///
/// The matrix names a representative many-to-many, not a proof over every
/// cardinality. The case chosen is the smallest whose input and output
/// counts both exceed the one-to-one control's: two receipts consumed and
/// three outputs created, so that "many to many" describes both halves and
/// is not a one-to-many or many-to-one in disguise.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_many_to_many_shape_is_submitted_to_a_real_target() {
    use vectors::live_multi_shapes::PrivateShape;

    run_one_multi_shape(PrivateShape::ManyToMany, "multi-many-to-many");
}

/// The restart order's fifth step, the several-distinct-owners shape: two
/// receipts under two distinct owners in, two outputs, against a real node.
///
/// # What distinguishes it from the many-to-many run
///
/// Its subject is the input owners rather than the cardinality: the two
/// consumed receipts are owned by two distinct published owners, and each
/// input carries the leaf its own position executes. The predecessor pays
/// its two outputs to the two owners' private receipt constructors, so
/// consuming both is a transfer whose inputs have several distinct owners.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_several_distinct_owners_shape_is_submitted_to_a_real_target() {
    use vectors::live_multi_shapes::PrivateShape;

    run_one_multi_shape(PrivateShape::SeveralDistinctOwners, "multi-several-owners");
}

/// The strict one-to-one: ONE receipt in and ONE output out, against a
/// real node.
///
/// # The shape the registry used to refuse
///
/// This is not a step of the restart order and it moves no matrix row.
/// It is a row of the CONSENSUS shape census, which enumerates what the
/// target's balance rule admits rather than what the guide's own class
/// table names, and it sat there recorded source-derived-possible and
/// refused: the fixture registry's two-output floor turned it away before
/// looking at its one output. The census typed that floor as a
/// first-party convention rather than a protocol rule, and this run is
/// what a structural removal is worth — the shape's lone output declares
/// the fully-solved balancing form, takes the consumed coin's own value
/// blinder, and is offered to a node.
///
/// # Why THIS one-output shape and not the merge
///
/// It consumes one receipt, so its input blinder sum is a single coin's
/// blinder with nothing to cancel against. A merge of this ceremony's
/// predecessor consumes an inverse pair whose blinders sum to zero, and a
/// forced zero blinder hides nothing — the registry refuses it, so no
/// merge is submitted from here and none is claimed.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_strict_one_to_one_shape_is_submitted_to_a_real_target() {
    use vectors::live_multi_shapes::PrivateShape;

    run_one_multi_shape(PrivateShape::StrictOneToOne, "multi-strict-one-to-one");
}

/// One multi-output or multi-input private shape, against the node.
///
/// The same shape-only discipline the one-to-one control ran under: what
/// the node decided is written into the artifact and asserted nowhere, so
/// a lane that asserted an acceptance would fail rather than report on the
/// day the honest answer changed. What is asserted is first-party
/// construction facts — the shape's own input and output counts, that the
/// candidate reached the node, and, where an acceptance was observed, the
/// two-origin agreement.
fn run_one_multi_shape(shape: vectors::live_multi_shapes::PrivateShape, extension: &str) {
    use vectors::live_multi_shapes::{MultiShapePlanner, render_multi_shape};

    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let base = environment("TRIPOD_LIVE_REPORT")
        .map(PathBuf::from)
        .expect("TRIPOD_LIVE_REPORT names where the transcript is written");
    let report = base.with_extension(extension);

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
        ExecutorTrust::ReviewedNonMock,
        timeout,
        ExecutorDiagnostics::in_directory(report.parent().unwrap_or_else(|| Path::new("."))),
    );

    let mut planner = MultiShapePlanner::for_shape(shape, identifier(&genesis))
        .expect("the shape ceremony builds");
    let started = Instant::now();
    let outcome = execute_operations(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    let rendered = render_multi_shape(record);
    std::fs::write(&report, &rendered).expect("the transcript is written");
    std::fs::write(
        timing_path(&report),
        format!("wall_seconds {:.1}\n", wall.as_secs_f64()),
    )
    .expect("the run's wall time is written");
    if let Err(error) = &outcome {
        std::fs::write(
            report.with_extension("executor-refusal"),
            format!("{error}\n"),
        )
        .expect("the executor's refusal is written");
    }

    // A construction refusal is a valid outcome and is written down as one.
    // It is never a target verdict, so it is reported and the test stops
    // here rather than pretending the node said anything.
    if let Some(refusal) = record.refusal() {
        panic!("the shape ceremony refused before the node: {refusal:?}");
    }
    outcome.expect("the ceremony reached the target");

    // The predecessor is the confidential one the ceremony asked for.
    assert_eq!(record.coins().len(), 2);
    assert!(
        record
            .coins()
            .iter()
            .all(vectors::live_private_restart::RestartConfidentialCoin::matches_expectation),
        "the node reported a confidential coin the ceremony did not ask for",
    );

    // The shape's own cardinalities, read off the record rather than the
    // shape's name: the receipts it consumed and the outputs it created.
    assert_eq!(
        record.receipt_leaves(),
        record.input_count(),
        "the consumed receipt count is the shape's input count",
    );
    assert_eq!(
        record.output_witness_proof_bytes().len(),
        record.output_count(),
        "one output-witness entry per created output",
    );
    assert!(
        record
            .output_witness_proof_bytes()
            .iter()
            .all(|bytes| *bytes > 2),
        "an output-witness entry carried no range proof: {:?}",
        record.output_witness_proof_bytes(),
    );

    // The candidate reached the node.
    assert!(record.submitted_bytes() > 0);
    assert!(record.observed_layer().is_some(), "no layer was observed");

    // The two origins, where an acceptance was observed.
    if let Some(check) = record.reverification() {
        assert!(
            check.readback_matches_submission(),
            "the bytes the node reported are not the bytes it was handed",
        );
        assert!(
            check.verified(),
            "the accepted witness does not verify against the recomputed message",
        );
    }

    // The run says in its own bytes what it did not establish.
    assert!(rendered.contains("evidences_no_negative_case true"));
    assert!(rendered.contains("moves_the_sponsor_row false"));
}
