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
        // Absent, because this funding creates no sponsor coin. A
        // program stated here would name a member the transaction does
        // not carry, and the classifier would place nothing differently
        // for it.
        sponsor_reserve_program: None,
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
/// It asserts the shape of a completed ceremony and binds all seven
/// answers to T5-026's run of record: each case name, each target layer,
/// whether an accepted identity was present, and the selected case's
/// accepted and reverified identity. A changed answer is still written
/// into the artifact first, and then the lane fails closed pending owner
/// review rather than silently advancing the historical record.
///
/// The two-origin agreement is unconditional for the selected case. A
/// run that accepted a candidate and then could not verify its read-back
/// witness against the independently recomputed message has found
/// something, and it must say so by failing rather than by writing a
/// false line.
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
    assert_owner_observation_matches_run_of_record(record);

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

    // The run says in its own bytes what it did not establish.
    assert!(rendered.contains("establishes_the_proof_bearing_lane false"));
    assert!(rendered.contains("clears_sighash_profile_unreviewed false"));
    assert!(rendered.contains("discharges_no_matrix_row true"));
}

/// Bind every T5-026 answer after the transcript has been written.
fn assert_owner_observation_matches_run_of_record(
    record: &vectors::live_owner_observation::OwnerObservationRecord,
) {
    use vectors::live_owner_observation::run_of_record::{
        self, ExpectedAcceptance, ExpectedCaseOutcome,
    };

    assert_eq!(
        record.observations().len(),
        run_of_record::EXPECTED_CASE_OUTCOMES.len(),
        "the ceremony did not answer the full run-of-record census",
    );

    let mut expected_reverification_identity = None;
    for (observation, expected) in record
        .observations()
        .iter()
        .zip(run_of_record::EXPECTED_CASE_OUTCOMES)
    {
        match expected {
            ExpectedCaseOutcome::Recorded {
                case,
                name,
                layer,
                acceptance,
            } => {
                assert_eq!(observation.case(), case, "the recorded case order drifted");
                assert_eq!(
                    observation.case().name(),
                    name,
                    "the recorded case name drifted",
                );
                assert_eq!(
                    observation.layer(),
                    layer,
                    "{name} reached a different target layer",
                );
                match acceptance {
                    ExpectedAcceptance::Refused => assert_eq!(
                        observation.accepted_txid(),
                        None,
                        "{name} was refused but carried an accepted identity",
                    ),
                    ExpectedAcceptance::Accepted {
                        identity,
                        reverification_identity,
                    } => {
                        assert_eq!(
                            observation.accepted_txid(),
                            Some(identity),
                            "{name} was accepted at a different identity",
                        );
                        assert!(
                            expected_reverification_identity
                                .replace(reverification_identity)
                                .is_none(),
                            "the run of record names more than one reverification",
                        );
                    }
                }
            }
            ExpectedCaseOutcome::NotRecorded { case, name } => {
                assert_eq!(
                    observation.case(),
                    case,
                    "the unrecorded case order drifted"
                );
                assert_eq!(
                    observation.case().name(),
                    name,
                    "the unrecorded case name drifted",
                );
            }
        }
    }

    // Checked UNCONDITIONALLY. The selected case is asserted Accepted
    // above, so an absent second-origin record is itself a binding failure.
    let expected_reverification_identity = expected_reverification_identity
        .expect("the selected case names its recorded reverification identity");
    let check = record
        .reverification()
        .expect("the selected acceptance carries its two-origin readback check");
    assert_eq!(
        check.accepted_txid(),
        expected_reverification_identity,
        "the reverification named a different accepted transaction",
    );
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
        PROOF_BEARING_RUN_OF_RECORD_SCHEMA_VERSION, ProofBearingObservationPlanner,
        ProofBearingRunOfRecord, ProofBearingRunOfRecordV2, construction_run_of_record_v2,
        render_proof_bearing_observation,
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

    assert_construction_refusals_match_the_run_of_record(record);
    match construction_run_of_record_v2() {
        ProofBearingRunOfRecordV2::Pending => {
            let projection = ProofBearingRunOfRecord::try_from(record)
                .expect("the completed V2 ceremony projects before constants are minted");
            assert_eq!(
                projection.schema_version(),
                PROOF_BEARING_RUN_OF_RECORD_SCHEMA_VERSION
            );
            assert!(rendered.contains("run_of_record_v2 pending"));
            assert!(rendered.contains("run_of_record_projection ready"));
        }
        ProofBearingRunOfRecordV2::Recorded(expected) => {
            assert_proof_bearing_record_matches_run_of_record(record, expected);
        }
    }
    check_proof_bearing_record(record, &rendered);
}

/// Bind every live construction refusal to the V2 run of record where
/// the constants exist.
///
/// The live-only relations do not wait for those constants: every
/// refusal must project to the stable vocabulary, the controls must be
/// exactly the closed control census in order, and every full refusal
/// must name the first consumed coin.
///
/// # Panics
///
/// If the completed live record omits a coin or control, carries an
/// unrecognized refusal, names the wrong consumed coin, or differs from
/// a recorded V2 refusal vector.
fn assert_construction_refusals_match_the_run_of_record(
    record: &vectors::live_proof_bearing_observation::ProofBearingObservationRecord,
) {
    use transaction::live_materialize::MaterializationRefusal;
    use vectors::live_proof_bearing_observation::{
        ProofBearingConstructionControl, ProofBearingRunOfRecordV2,
        RecordedProofBearingConstructionRefusal, construction_run_of_record_v2,
    };

    let projected = record
        .construction_refusals()
        .iter()
        .map(RecordedProofBearingConstructionRefusal::try_from)
        .collect::<Result<Vec<_>, _>>()
        .expect("every live construction refusal has a stable archival reason");
    let controls: Vec<_> = projected
        .iter()
        .map(RecordedProofBearingConstructionRefusal::control)
        .collect();
    assert_eq!(
        controls.as_slice(),
        ProofBearingConstructionControl::ALL,
        "the live construction-control census drifted",
    );

    let first_outpoint = record
        .coins()
        .first()
        .expect("the completed ceremony consumed a predecessor coin")
        .outpoint();
    for refusal in record.construction_refusals() {
        let MaterializationRefusal::PredecessorOpeningMismatch { outpoint } = refusal.refusal()
        else {
            panic!("a construction control drew an unrecognized live refusal");
        };
        assert_eq!(
            *outpoint, first_outpoint,
            "a construction refusal names a coin other than the first consumed coin",
        );
    }

    if let ProofBearingRunOfRecordV2::Recorded(expected) = construction_run_of_record_v2() {
        let expected = expected
            .construction_refusals()
            .captured()
            .expect("the V2 record captures construction refusals");
        assert_eq!(
            projected.as_slice(),
            expected,
            "the live construction refusals drifted from the complete V2 vector",
        );
    }
}

/// Bind the live coins, reverification, and candidate messages to the
/// exact V2 archival record.
///
/// # Panics
///
/// If the live record is incomplete or any projected V2 field differs
/// from the recorded value.
fn assert_proof_bearing_record_matches_run_of_record(
    actual: &vectors::live_proof_bearing_observation::ProofBearingObservationRecord,
    expected: &vectors::live_proof_bearing_observation::ProofBearingRunOfRecord,
) {
    use vectors::live_proof_bearing_observation::ProofBearingRunOfRecord;

    let projected = ProofBearingRunOfRecord::try_from(actual)
        .expect("the completed live ceremony projects to the archival schema");
    assert_eq!(projected.schema_version(), expected.schema_version());
    assert_eq!(projected.issued_asset(), expected.issued_asset());
    assert_eq!(
        projected.predecessor_digest(),
        expected.predecessor_digest()
    );
    assert_eq!(
        projected.coins(),
        expected.coins(),
        "a node-reported coin field drifted from V2",
    );
    assert_eq!(
        projected.output_witness_vector_length(),
        expected.output_witness_vector_length()
    );
    assert_eq!(
        projected.output_witness_proof_bytes(),
        expected.output_witness_proof_bytes()
    );
    assert_eq!(
        projected.spent_value_prefixes(),
        expected.spent_value_prefixes()
    );
    assert_eq!(projected.observations(), expected.observations());
    assert_eq!(
        projected.reverification(),
        expected.reverification(),
        "the exact reverification outcome drifted from V2",
    );
    assert_eq!(
        projected.candidate_messages(),
        expected.candidate_messages(),
        "a candidate message drifted from V2",
    );
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

    // Every one of the four proof-negatives was submitted and answered:
    // the three range/blinder cases plus the private-ct-imbalance mutant
    // at the change output.
    assert_eq!(record.mutants().len(), 4);
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

/// One owner-signing script-path negative, signed over its own mutated
/// bytes and refused at the leaf's own clause.
///
/// # What this run is for
///
/// The `vault-control-entitlement-or-bare-u-output` row declares a
/// script-path refusal, and a mutant with a stale signature would die at
/// the signature gate before the leaf ran. This ceremony re-signs the
/// mutant over its own mutated bytes through the negative-evidence census
/// route, so it passes the signature gate and reaches the coordinator
/// leaf's `InspectOutputScriptPubKey` version clause — which refuses the
/// bare-u output. The unmutated control is accepted afterwards on the same
/// chain, which is what makes the mutant's refusal attributable.
///
/// # What it asserts, and why the standing changed
///
/// This run's verdicts are RECORDED as run-of-record constants, and a
/// record nothing checks can drift silently — the standing an adversarial
/// review found this lane resting on. So every verdict this ceremony's
/// constants carry is now ASSERTED against them: each mutant's layer and
/// verbatim detail, each declared separator, the control's ACCEPTANCE and
/// its accepted identity, and the two-origin readback unconditionally. The
/// lane fails if the control were rejected, if a mutant answered at the
/// wrong layer, or if a detail, a range, a shape or an identity moved.
///
/// The reason this used to be left unasserted — that a lane which asserts a
/// verdict fails rather than reports on the day the honest answer changes —
/// is answered by the order of operations rather than by silence: the
/// transcript and the wall time are written to disk BEFORE the first
/// assertion runs, so the artifact carries what the node actually said
/// either way, and a drift is reported AND failed rather than passed over.
/// The first-party construction facts stay asserted beside them: the
/// mutation is confined to its declared field, which is what makes the
/// refusal attributable to that field and to no other.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn one_bare_u_output_mutant_is_refused_before_the_control_is_accepted() {
    use vectors::live_owner_signing_negatives::{
        OwnerSigningNegativePlanner, render_owner_signing_negatives,
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
        .with_extension("owner-signing-negatives");

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
        OwnerSigningNegativePlanner::new(identifier(&genesis)).expect("the ceremony builds");
    let started = Instant::now();
    let outcome = execute_operations(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    let rendered = render_owner_signing_negatives(record);
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
        panic!("the owner-signing negative ceremony refused before the node: {refusal:?}");
    }
    outcome.expect("the ceremony reached the target");

    assert!(record.relinked(), "the ceremony funded before it linked");
    assert_eq!(record.coins().len(), 2);
    assert!(
        record
            .coins()
            .iter()
            .all(vectors::live_owner_observation::ObservedFundedCoin::matches_expectation),
        "the node reported a coin the ceremony did not ask for",
    );

    // The mutant and the control were each submitted and answered, and each
    // answered WHERE the run of record says, in the words it recorded.
    assert_mutant_and_control_match_the_record(record);

    // The two candidates were signed over different messages: a mutant whose
    // message coincided with the control's would be signed over the same
    // bytes and the comparison would be vacuous.
    let mutant = record.mutant().expect("the mutant was built and submitted");
    let control = record
        .control()
        .expect("the control was built and submitted");
    assert_ne!(
        mutant.message(),
        control.message(),
        "the mutant and the control were signed over one message",
    );
    assert!(rendered.contains("messages_differ true"));

    // The run says in its own bytes what it did not establish.
    assert!(rendered.contains("each_row_by_its_own_mutant true"));

    // The seven consensus-conservation mutants were each built, submitted
    // and refused at consensus in the recorded words, and each declared the
    // (range, shape) separator the run of record carries for its row, so no
    // two rows rest on one observation.
    assert_consensus_mutants_separate(record);

    // The two leaf-arrangement mutants — one per collision pair — were each
    // built, submitted and refused at the script path in their recorded
    // words, and each declared the revealed-leaf arrangement the run of
    // record carries, so no two rows rest on one observation.
    assert_leaf_arrangements_drive(record);
}

/// The bare-u mutant and the control were each answered WHERE the run of
/// record says, in the words it recorded, at the identity it names.
///
/// Split out so the test body stays under the line bound.
///
/// WHAT THIS REPLACED, and why. These facts were checked as "answered at
/// SOME layer", with the control's readback checked only where a
/// reverification happened to be present. That left the lane green on
/// exactly the drifts the recorded constants exist to make checkable: a
/// REJECTED control (which makes every refusal in the run unattributable),
/// a mutant answered at the wrong layer, or a detail or an accepted
/// identity that moved. The transcript is written to disk BEFORE any
/// assertion in this test runs, so binding these costs no report on the day
/// an answer changes — the artifact carries what the node said either way,
/// and the lane now FAILS instead of passing over a record nothing reads.
fn assert_mutant_and_control_match_the_record(
    record: &vectors::live_owner_signing_negatives::OwnerSigningNegativeRecord,
) {
    use target_elements_conformance::protocol::ObservedOutcomeLayer;
    use vectors::live_owner_signing_negatives::run_of_record;

    let mutant = record.mutant().expect("the mutant was built and submitted");
    // BOUND to the run of record's own typed layer rather than to a
    // literal written here. The constant is what the live evidence
    // classifier compares against a row's declared boundary, so a run
    // whose layer moved must fail here — where the transcript is already
    // on disk — rather than silently disagreeing with the classifier.
    assert_eq!(
        mutant.observed_layer(),
        Some(run_of_record::MUTANT_OBSERVED_LAYER),
        "the bare-u mutant was not refused at the script path",
    );
    assert_eq!(
        mutant.observed_detail(),
        Some(run_of_record::MUTANT_REJECT_DETAIL),
        "the bare-u mutant drew words the run of record does not carry",
    );
    assert_eq!(
        mutant.declared_field_range(),
        run_of_record::DECLARED_FIELD_RANGE,
        "the mutation did not stay in the range the run of record declares",
    );
    assert_eq!(
        mutant.submitted_bytes(),
        run_of_record::MUTANT_SUBMITTED_BYTES,
        "the mutant handed the node a different number of bytes",
    );

    let control = record
        .control()
        .expect("the control was built and submitted");
    assert_eq!(
        control.observed_layer(),
        Some(ObservedOutcomeLayer::Accepted),
        "the control was not ACCEPTED, so no refusal in this run is attributable",
    );
    assert_eq!(
        control.accepted_txid(),
        Some(run_of_record::CONTROL_ACCEPTED_TXID),
        "the control was accepted at an identity the run of record does not carry",
    );

    // Checked UNCONDITIONALLY. The control is asserted ACCEPTED just above,
    // so an ABSENT two-origin check is itself the failure the conditional
    // form used to skip.
    let check = control
        .reverification()
        .expect("an accepted control carries its two-origin readback check");
    assert!(
        check.readback_matches_submission(),
        "the bytes the node reported are not the bytes it was handed",
    );
}

/// Each leaf-arrangement row, the revealed-leaf arrangement the run of
/// record declares for it, and the words the target answered it with.
const fn recorded_leaf_arrangements() -> [(&'static str, &'static [u16], &'static str); 2] {
    use vectors::live_owner_signing_negatives::run_of_record;
    [
        (
            "two-coordinators",
            &run_of_record::TWO_COORDINATORS_ARRANGEMENT,
            run_of_record::TWO_COORDINATORS_REJECT_DETAIL,
        ),
        (
            "no-coordinator",
            &run_of_record::NO_COORDINATOR_ARRANGEMENT,
            run_of_record::NO_COORDINATOR_REJECT_DETAIL,
        ),
    ]
}

/// The two leaf-arrangement mutants were each refused at the script path in
/// the recorded words, each declared the recorded revealed-leaf arrangement,
/// and the two arrangements are distinct from each other and from the
/// control's.
///
/// Split out for the same reason the consensus assertion is: the fact the
/// drive rests on — that one mutant per pair is a distinct candidate — is
/// stated once and the test body stays under the line bound.
fn assert_leaf_arrangements_drive(
    record: &vectors::live_owner_signing_negatives::OwnerSigningNegativeRecord,
) {
    use std::collections::BTreeSet;
    use vectors::live_owner_signing_negatives::{LeafArrangementObservation, run_of_record};
    let arrangements = record.leaf_arrangements();
    assert_eq!(
        arrangements.len(),
        2,
        "the two leaf-arrangement mutants — one per collision pair — were built",
    );
    let recorded = recorded_leaf_arrangements();
    // Set equality: the two built are exactly the two the run of record
    // names, so a renamed or substituted row fails rather than passing as
    // "two of something".
    let built: BTreeSet<&str> = arrangements
        .iter()
        .map(LeafArrangementObservation::row)
        .collect();
    assert_eq!(
        built,
        recorded
            .iter()
            .map(|(row, ..)| *row)
            .collect::<BTreeSet<_>>(),
        "the leaf-arrangement mutants are not the two recorded rows",
    );
    for mutant in arrangements {
        let (_, arrangement, detail) = recorded
            .iter()
            .find(|(row, ..)| *row == mutant.row())
            .expect("every built mutant is a recorded row");
        // BOUND per row to that row's own recorded layer constant, not
        // to one literal covering both: the two rows are separate
        // observations and a shared literal would hide a run in which
        // only one of them moved.
        let recorded_layer = match mutant.row() {
            "two-coordinators" => run_of_record::TWO_COORDINATORS_OBSERVED_LAYER,
            "no-coordinator" => run_of_record::NO_COORDINATOR_OBSERVED_LAYER,
            other => panic!("{other} is not a recorded leaf-arrangement row"),
        };
        assert_eq!(
            mutant.observed_layer(),
            Some(recorded_layer),
            "{} was not refused at the script path",
            mutant.row(),
        );
        assert_eq!(
            mutant.revealed_arrangement(),
            *arrangement,
            "{} revealed an arrangement the run of record does not carry",
            mutant.row(),
        );
        assert_eq!(
            mutant.observed_detail(),
            Some(*detail),
            "{} drew words the run of record does not carry",
            mutant.row(),
        );
        assert_ne!(
            mutant.revealed_arrangement(),
            run_of_record::CONTROL_ARRANGEMENT.as_slice(),
            "{} reveals the control's own arrangement and rearranges nothing",
            mutant.row(),
        );
    }
    // The separating fact is the revealed-leaf arrangement: the mutants keep
    // the control's witnessless serialization and differ only in which
    // committed leaf each input reveals, so a distinct arrangement per row
    // is what keeps no two rows resting on one observation.
    let mut arrangements: Vec<Vec<u16>> = arrangements
        .iter()
        .map(|mutant| LeafArrangementObservation::revealed_arrangement(mutant).to_vec())
        .collect();
    arrangements.sort_unstable();
    arrangements.dedup();
    assert_eq!(
        arrangements.len(),
        record.leaf_arrangements().len(),
        "two leaf-arrangement mutants share a revealed-leaf arrangement and do not separate",
    );
}

/// One consensus row's separating fact: the half-open witnessless byte
/// range its surgery declared, together with the transaction shape the
/// mutant handed the node.
type ConsensusSeparator = ((usize, usize), (usize, usize));

/// Each consensus row and the `(range, shape)` separator the run of record
/// declares for it.
///
/// The four field surgeries keep the control's 2-in-2-out shape and separate
/// by four distinct ranges; the two output-cardinality surgeries share the
/// structural range `changed_range` cannot localize past the output-count
/// varint and separate by shape; `omitted-source` separates by both.
const fn recorded_consensus_separators() -> [(&'static str, ConsensusSeparator); 7] {
    use vectors::live_owner_signing_negatives::run_of_record;
    /// The control's own shape, which the four field surgeries keep.
    const KEPT: (usize, usize) = (2, 2);
    [
        (
            "wrong-explicit-asset",
            (run_of_record::WRONG_EXPLICIT_ASSET_FIELD_RANGE, KEPT),
        ),
        (
            "confidential-asset-commitment",
            (
                run_of_record::CONFIDENTIAL_ASSET_COMMITMENT_FIELD_RANGE,
                KEPT,
            ),
        ),
        (
            "output-total-one-below-input",
            (run_of_record::OUTPUT_TOTAL_ONE_BELOW_FIELD_RANGE, KEPT),
        ),
        (
            "output-total-one-above-input",
            (run_of_record::OUTPUT_TOTAL_ONE_ABOVE_FIELD_RANGE, KEPT),
        ),
        (
            "private-output-omitted",
            (
                run_of_record::OUTPUT_CARDINALITY_FIELD_RANGE,
                run_of_record::PRIVATE_OUTPUT_OMITTED_SHAPE,
            ),
        ),
        (
            "hidden-private-u-output",
            (
                run_of_record::OUTPUT_CARDINALITY_FIELD_RANGE,
                run_of_record::HIDDEN_PRIVATE_U_OUTPUT_SHAPE,
            ),
        ),
        (
            "omitted-source",
            (
                run_of_record::OMITTED_SOURCE_FIELD_RANGE,
                run_of_record::OMITTED_SOURCE_SHAPE,
            ),
        ),
    ]
}

/// The seven consensus-conservation mutants are the seven recorded rows,
/// each refused at consensus before script in the recorded words, each on
/// the separator the run of record declares for it, and the seven
/// separators are pairwise distinct.
///
/// Split from the test body so the assertion the run rests on — that no
/// two rows share one observation — is stated once and the test stays
/// under the line bound.
fn assert_consensus_mutants_separate(
    record: &vectors::live_owner_signing_negatives::OwnerSigningNegativeRecord,
) {
    use std::collections::BTreeSet;
    use vectors::live_owner_signing_negatives::{ConsensusMutantObservation, run_of_record};
    let consensus = record.consensus_mutants();
    assert_eq!(consensus.len(), 7, "the seven consensus mutants were built");
    let recorded = recorded_consensus_separators();
    // Set equality: the seven built are exactly the seven the run of record
    // names, so a renamed or substituted row fails rather than passing as
    // "seven of something".
    let built: BTreeSet<&str> = consensus
        .iter()
        .map(ConsensusMutantObservation::row)
        .collect();
    assert_eq!(
        built,
        recorded
            .iter()
            .map(|(row, _)| *row)
            .collect::<BTreeSet<_>>(),
        "the consensus mutants are not the seven recorded rows",
    );
    for mutant in consensus {
        let (_, separator) = recorded
            .iter()
            .find(|(row, _)| *row == mutant.row())
            .expect("every built mutant is a recorded row");
        // BOUND to the consensus layer constant the seven rows' retype
        // rests on. If a rerun ever answered these at the script path
        // instead, this fails and the retype is revisited by ruling
        // rather than by drift.
        assert_eq!(
            mutant.observed_layer(),
            Some(run_of_record::CONSENSUS_MUTANT_OBSERVED_LAYER),
            "{} was not refused at consensus before script",
            mutant.row(),
        );
        assert_eq!(
            mutant.observed_detail(),
            Some(run_of_record::CONSENSUS_MUTANT_REJECT_DETAIL),
            "{} drew words the run of record does not carry",
            mutant.row(),
        );
        assert_eq!(
            mutant.separator(),
            *separator,
            "{} drifted off the separator the run of record declares",
            mutant.row(),
        );
    }
    // The separating fact is the byte range together with the shape: the
    // four field surgeries keep the control's shape and separate by range,
    // the three structural surgeries separate by shape where the
    // output-count varint defeats a localized range. The tuple is distinct
    // across all seven, so no two rows rest on one observation.
    let mut separators: Vec<((usize, usize), (usize, usize))> = consensus
        .iter()
        .map(ConsensusMutantObservation::separator)
        .collect();
    separators.sort_unstable();
    separators.dedup();
    assert_eq!(
        separators.len(),
        consensus.len(),
        "two consensus mutants share a range-and-shape separator and do not separate",
    );
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
/// the one-item shape, and the signing key is not the output key. It also
/// asserts the PAIR: that the control offered after the attempt is the
/// same candidate — measured off the two witnessless serializations — and
/// that the two submissions drew different verdicts.
///
/// It also asserts the two verdicts THEMSELVES, against phase B's own run
/// of record: the attempt's exact layer and the target's verbatim words,
/// the control's acceptance and the identity it was accepted at. The
/// earlier form recorded both and asserted neither, so that an unexpected
/// layer reached a reader rather than a panic. That protection is kept
/// where it belongs — the transcript is written to disk BEFORE any
/// assertion here runs, so the artifact carries what the node said either
/// way — and the lane now FAILS on the drifts the recorded constants
/// exist to make checkable.
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
    // The target answered, which is the precondition every assertion
    // below rests on: a run that reached no verdict at all fails here,
    // where the failure names what happened, rather than inside a
    // comparison against a figure the run never produced.
    assert!(
        record.observation().is_some(),
        "the attempt was not answered",
    );

    assert_the_pair_is_one_candidate_answered_twice(record);
    assert_the_verdicts_match_the_run_of_record(record);

    // The run says in its own bytes what it did not establish.
    assert!(rendered.contains("residual_internal_key_unspendability_stands true"));
    assert!(rendered.contains("discharges_no_residual true"));
}

/// The pair's own assertions, kept beside the run that produces them.
///
/// Extracted rather than inlined because the probe's test asserts two
/// different things — the attempt's shape and the pair's relation — and a
/// reader looking for the second should not have to find it inside the
/// first.
fn assert_the_pair_is_one_candidate_answered_twice(
    record: &vectors::live_keypath_probe::KeyPathProbeRecord,
) {
    // The control is the SAME candidate: the two witnessless
    // serializations were compared byte for byte and the comparison is
    // what is asserted, not the construction that produced them. A
    // control finalized over some other coin would be a second candidate
    // wearing the control's name.
    let control = record
        .control()
        .expect("the ceremony built the script-path control");
    assert!(
        control.shares_the_attempts_witnessless_bytes(),
        "the control and the attempt are not one candidate",
    );
    assert_eq!(
        control.witness_items(),
        3,
        "the control is not the script-path shape",
    );

    // The two verdicts are DIFFERENT, which is the whole content of the
    // pair. WHICH each of them was is bound separately, in
    // [`assert_the_verdicts_match_the_run_of_record`]: inequality alone
    // holds for an accepted attempt against a refused control just as it
    // holds for the run that happened.
    let control_observation = record
        .control_observation()
        .expect("the control was not answered");
    assert_ne!(
        record
            .observation()
            .expect("the attempt was answered")
            .layer(),
        control_observation.layer(),
        "the attempt and its control drew one verdict, so the pair separates nothing",
    );
}

/// The two verdicts and the two candidates, held against phase B's run of
/// record.
///
/// # Why this was unbound, and why binding it costs no report
///
/// The probe's first form recorded both verdicts and asserted neither, so
/// that an unexpected layer would reach a reader instead of a panic. The
/// rendered transcript is what delivers that, and it is written to disk
/// BEFORE any assertion in this test runs — so nothing here costs the
/// report on the day an answer changes. What the unasserted form actually
/// left green was every drift the run of record exists to make checkable:
/// an ACCEPTED attempt against a REFUSED control differ in layer just as
/// the run that happened does, so the pair's one assertion passed on the
/// exact inversion that makes the seventeenth refusal row unattributable.
/// Each figure below is phase B's own, and each now fails on drift.
fn assert_the_verdicts_match_the_run_of_record(
    record: &vectors::live_keypath_probe::KeyPathProbeRecord,
) {
    use target_elements_conformance::protocol::ObservedOutcomeLayer;
    use vectors::live_keypath_probe::{run_of_record, run_of_record_phase_b};

    // One chain, one issuance: the asset the probe's own run recorded.
    assert_eq!(
        record.issued_asset(),
        Some(run_of_record::ISSUED_ASSET),
        "the probe ran against an asset the run of record does not carry",
    );

    let attempt = record
        .attempt()
        .expect("the ceremony built the key-path attempt");
    assert_eq!(
        attempt.submitted_bytes().len(),
        run_of_record::SUBMITTED_BYTES,
        "the attempt handed the node a different number of bytes",
    );
    assert_eq!(
        attempt.witness_stack().len(),
        run_of_record::WITNESS_ITEMS,
        "the attempt is not the recorded one-item witness",
    );
    assert_eq!(
        attempt.witness_stack()[0].len(),
        run_of_record::WITNESS_ITEM_BYTES,
        "the attempt's one witness item is not the recorded width",
    );
    // The bytes carry a whole transaction and not only the witness item,
    // which is the cheapest check that the attempt was assembled rather
    // than merely signed. Kept beside the recorded width it reads.
    assert!(
        attempt.submitted_bytes().len() > attempt.witness_stack()[0].len(),
        "the submitted bytes are no larger than the witness item",
    );

    // The ATTEMPT's verdict, exactly. Phase B's whole content is the name
    // the refusal is filed under, so the layer is asserted as the enum AND
    // the recorded spelling is held against that enum: a constant that had
    // drifted from the vocabulary would otherwise stay green beside it.
    let observation = record.observation().expect("the attempt was answered");
    assert_eq!(
        observation.layer(),
        run_of_record_phase_b::REFUSAL_OBSERVED_LAYER,
        "the attempt was not refused at the key path",
    );
    assert_eq!(
        format!("{:?}", observation.layer()),
        run_of_record_phase_b::OBSERVED_LAYER,
        "the recorded layer name is not the vocabulary's own spelling",
    );
    assert_eq!(
        observation.detail(),
        Some(run_of_record_phase_b::REFUSAL_DETAIL),
        "the attempt drew words the run of record does not carry",
    );
    assert_eq!(
        observation.accepted_txid(),
        None,
        "a refused attempt was given an accepted identity",
    );

    // The CONTROL's verdict, exactly. A refused control makes every
    // refusal in the run unattributable, and its identity is the half of
    // the pair a reader can check against a chain.
    let control_observation = record
        .control_observation()
        .expect("the control was answered");
    assert_eq!(
        control_observation.layer(),
        ObservedOutcomeLayer::Accepted,
        "the control was not ACCEPTED, so the attempt's refusal is not attributable",
    );
    assert_eq!(
        control_observation.accepted_txid(),
        Some(run_of_record_phase_b::CONTROL_ACCEPTED_TXID),
        "the control was accepted at an identity the run of record does not carry",
    );

    // The control's own bytes. This record carries no two-origin readback
    // — the reverification the owner-signing ceremony records has no
    // counterpart in this probe — so what stands in its place is the
    // measured witnessless-bytes relation the pair rests on, held against
    // the recorded value rather than against a literal.
    let control = record
        .control()
        .expect("the ceremony built the script-path control");
    assert_eq!(
        control.submitted_bytes().len(),
        run_of_record_phase_b::CONTROL_SUBMITTED_BYTES,
        "the control handed the node a different number of bytes",
    );
    assert_eq!(
        control.witness_items(),
        run_of_record_phase_b::CONTROL_WITNESS_ITEMS,
        "the control is not the recorded script-path witness census",
    );
    assert_eq!(
        control.shares_the_attempts_witnessless_bytes(),
        run_of_record_phase_b::CONTROL_SHARES_THE_ATTEMPTS_WITNESSLESS_BYTES,
        "the pair no longer differs in the witness alone",
    );
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

/// The split PAIR's private member: one receipt in, TWO blinded receipts
/// out, no change and no fee, against a real node.
///
/// # Why the split run above does not answer this
///
/// §16.1's split pair states one semantic fixture and materializes it
/// twice. Its explicit member ran and was accepted; its private member
/// creates exactly two outputs, and neither private run this lane has
/// recorded is that shape. The split above creates THREE outputs, keeping
/// a balancing change back for the sender, and the only other recorded
/// one-in-two-out private run is the fee-bearing shape, whose second
/// output is a fee role rather than a receipt. A pair member is not
/// answered by a run of a different cardinality, and it is not answered
/// by a run whose second output is a different role.
///
/// So this run exists to be the pair member's own shape, and the pair's
/// acceptance conjunct moves on it or on nothing.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_pure_split_shape_is_submitted_to_a_real_target() {
    use vectors::live_multi_shapes::PrivateShape;

    run_one_multi_shape(PrivateShape::PureSplit, "multi-pure-split");
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

/// One blinded input, one blinded output, and one REAL fee output.
///
/// # The shape the register called expressible and unrun
///
/// The fixture registry gained a fee role in the structural-removals
/// wave and nothing downstream of it could carry one, so the shape
/// registered, derived and digested and then stopped at the projection
/// with a typed refusal naming the missing projection. The stop was
/// honest and it was not a run: a register that had recorded the shape
/// observed because its vocabulary could express it would have been
/// committing the exact error the register exists to prevent.
///
/// This run is what that removal is worth. The fee output is a fee at
/// the target and not a blinded output wearing the name -- explicit
/// value, explicit asset, empty scriptPubKey, and no witness entry of
/// its own -- and the blinded output beside it is the balancing one,
/// whose blinder is solved over no other freely chosen blinder and
/// therefore comes out as the consumed coin's own.
///
/// # What it does not establish
///
/// It is not `private-sponsor-values` and it moves no matrix row. Nobody
/// sponsors anything here: the transaction pays its own fee out of its
/// own consumed coin, in the disposable protocol asset, which is the
/// only asset whose tally that fee can close. The sponsor rows' signer
/// dependency is untouched.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_fee_bearing_one_to_one_shape_is_submitted_to_a_real_target() {
    use vectors::live_multi_shapes::PrivateShape;

    run_one_multi_shape(PrivateShape::OneToOneWithFee, "multi-one-to-one-with-fee");
}

/// TWO blinded inputs merged into ONE blinded output.
///
/// # The shape that met two walls
///
/// The first was the fixture registry's two-output floor, which refused
/// any manifest of fewer than two outputs and turned the merge away
/// before looking at it. The sole-balancing form removed that floor, and
/// the merge walked forward into a second wall the first had been
/// hiding: the only coins the ceremony could offer it were the two
/// halves of an inverse pair, whose blinders sum to zero, so the lone
/// output's forced blinder was zero -- a commitment of exactly the value
/// times the value generator, which anybody recomputes from a guessed
/// amount. The registry refused it by name, and refusing it was right.
///
/// # What makes this one different, in one sentence
///
/// It spends a THREE-output predecessor, whose blinders cancel in no
/// pair.
///
/// Three blinders summing to zero leave any two of them summing to the
/// negation of the third. The third here is a DERIVED blinder, and a
/// derived blinder is searched upward until it is nonzero and never
/// admitted zero -- so the forced blinder is nonzero for a reason that
/// can be stated. The registry would refuse a zero one by name if the
/// reasoning were wrong, which is what makes the successor registering
/// at all a proof and not a hope.
///
/// # What it establishes
///
/// The row `private-merge` of the positive private table, on an
/// acceptance of THIS shape and nothing wider. It is a two-input
/// one-output transfer and it is not a claim about merges in general.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_private_merge_shape_is_submitted_to_a_real_target() {
    use vectors::live_multi_shapes::PrivateShape;

    run_one_multi_shape(PrivateShape::PrivateMerge, "multi-private-merge");
}

/// The sponsored CONFIDENTIAL with-change shape, offered to a real node.
///
/// The shape the section 15.2 `private-sponsor-values` row moves on, and
/// the one no run in this workspace had ever offered: a blinded sponsor
/// coin in at an explicit asset, blinded receipt destinations, a
/// COMMITTED sponsor change, and an explicit reserve fee held outside
/// both balance equations.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_sponsored_confidential_with_change_shape_is_submitted_to_a_real_target() {
    use vectors::live_sponsored_private::{SponsoredPrivatePlanner, render_sponsored_private};

    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let base = environment("TRIPOD_LIVE_REPORT")
        .map(PathBuf::from)
        .expect("TRIPOD_LIVE_REPORT names where the transcript is written");
    let report = base.with_extension("sponsored-private-with-change");

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
        SponsoredPrivatePlanner::new(identifier(&genesis)).expect("the ceremony builds");
    let started = Instant::now();
    let outcome = execute_operations(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    std::fs::write(&report, render_sponsored_private(record)).expect("the transcript is written");
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
        panic!("the sponsored private ceremony refused before the node: {refusal:?}");
    }
    outcome.expect("the ceremony reached the target");

    // The blinded sponsor coin, censused by named check and never by a
    // scalar. Counted against the whole vocabulary rather than against
    // the checks this test happens to name, so a check added later is
    // one this run has to have seen hold.
    let solve = record
        .solve()
        .expect("the sponsor funding stage censused the coin");
    assert!(
        solve.every_check_held(),
        "the blinded sponsor coin failed checks: {:?}",
        solve.missing(),
    );

    // The sponsor's round trip: the adapter signed the bytes it was
    // handed, and returned a witness.
    let round = record
        .round()
        .expect("the staging pass recorded a signing request");
    assert!(
        round.echo_matches_what_was_sent(),
        "the adapter authorized bytes that are not the ones it was handed",
    );
    assert!(round.witness_items() > 0, "the sponsor returned no witness");

    // The candidate reached the node.
    assert!(record.submitted_bytes() > 0);
    assert!(record.observed_layer().is_some(), "no layer was observed");

    let check = record
        .reverification()
        .expect("an acceptance was observed and read back");
    assert!(
        check.readback_matches_submission(),
        "the bytes the node reported are not the bytes it was handed",
    );
    assert!(
        check.owner_signature_verified(),
        "an owner's signature did not verify against an independently recomputed message",
    );
    assert_eq!(
        check.sponsor_change_located(),
        Some(true),
        "the sponsor's committed change was not found in the mined bytes",
    );
}

/// The sponsor PAIR's private member: an EXPLICIT sponsor coin funded
/// exactly to the fee, no change role, one blinded destination.
///
/// # Why the with-change run above does not answer this
///
/// §16.1's sponsor pair states one semantic fixture and materializes it
/// twice. Its explicit member ran and was accepted; its private member
/// states an explicit sponsor coin funded exactly to the fee, no change
/// role, and ONE destination, and the run above is none of those three:
/// it carries a blinded sponsor coin, a committed sponsor change, and two
/// blinded destinations.
///
/// # The arithmetic that lets this shape exist
///
/// The sponsor arc observed that a COMMITTED sponsor value REQUIRES
/// committed change -- a blinded input's blinder must be absorbed by
/// something, a fee is mandatorily explicit, and the change is the only
/// remaining term. That observation does not reach this shape and does
/// not forbid it. An explicit sponsor coin is committed with the all-zero
/// blinder, so there is nothing to absorb and no change is owed: the
/// sponsor input equals the fee output in the reserve asset, and the
/// protocol asset closes over the receipts alone.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_sponsored_explicit_no_change_shape_is_submitted_to_a_real_target() {
    use vectors::live_sponsored_private::{
        SponsoredPrivatePlanner, SponsoredPrivateShape, render_sponsored_private,
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
    let report = base.with_extension("sponsored-private-explicit-no-change");

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

    let mut planner = SponsoredPrivatePlanner::for_shape(
        SponsoredPrivateShape::ExplicitWithoutChange,
        identifier(&genesis),
    )
    .expect("the ceremony builds");
    let started = Instant::now();
    let outcome = execute_operations(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    std::fs::write(&report, render_sponsored_private(record)).expect("the transcript is written");
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

    if let Some(refusal) = record.refusal() {
        panic!("the sponsored private ceremony refused before the node: {refusal:?}");
    }
    outcome.expect("the ceremony reached the target");

    // NO solve census, and its absence is asserted rather than left
    // unmentioned. The census is a statement about a BLINDED sponsor
    // coin -- that the chain reported a commitment, and that the
    // commitment is the one the registry derives -- and this shape funds
    // no such coin. A census present here would mean the committed
    // funding stage had run, which is the one thing this shape's step
    // plan removes.
    assert!(
        record.solve().is_none(),
        "an explicit sponsor coin produced a committed-coin census",
    );

    // The sponsor's round trip still happens: an explicit coin is still
    // somebody else's coin, and spending it still needs its owner's
    // authorization over the exact finalized bytes.
    let round = record
        .round()
        .expect("the staging pass recorded a signing request");
    assert!(
        round.echo_matches_what_was_sent(),
        "the adapter authorized bytes that are not the ones it was handed",
    );
    assert!(round.witness_items() > 0, "the sponsor returned no witness");

    assert!(record.submitted_bytes() > 0);
    assert!(record.observed_layer().is_some(), "no layer was observed");

    let check = record
        .reverification()
        .expect("an acceptance was observed and read back");
    assert!(
        check.readback_matches_submission(),
        "the bytes the node reported are not the bytes it was handed",
    );
    assert!(
        check.owner_signature_verified(),
        "an owner's signature did not verify against an independently recomputed message",
    );
    // The change question is NOT ASKED of this shape, and that is
    // asserted rather than left implicit. The located-check is a byte
    // scan for the reserve asset and a sponsored transaction's FEE
    // carries the reserve asset too, so on a no-change shape the scan
    // answers about the fee and a reader would take it for a change
    // output that is not there.
    //
    // What rules the change out is arithmetic over the acceptance just
    // asserted, and it is the stronger statement. The target balances
    // per asset, so the reserve sub-equation is
    // `sponsor_input == fee + change`; this sponsor's coin was funded to
    // EXACTLY the fee, so any change output at all would leave that
    // equation short and the node would have refused the candidate. It
    // accepted it.
    assert_eq!(
        check.sponsor_change_located(),
        None,
        "a shape with no change role was asked whether its change was located",
    );
}

/// Every committed output carries a range proof, and every fee output
/// carries none.
///
/// Two assertions rather than one weakened to "some entries carry
/// proofs". A blinded output that lost its proof is exactly what this
/// check exists to catch, and a fee output that GREW one would be a fee
/// that had been blinded — the failure the fee role was built to make
/// impossible, and the one worth a second assertion of its own.
/// The ENTRY CROSSING against a real node.
///
/// Explicit receipts spent into two blinded destinations. This workspace
/// has performed the shape every ceremony as a FUNDING step; what is new
/// is that the coin it spends sits at a receipt constructor's program,
/// so the transfer is governed by the covenant rather than by the
/// adapter.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_entry_crossing_shape_is_submitted_to_a_real_target() {
    use vectors::live_multi_shapes::PrivateShape;

    run_one_multi_shape(PrivateShape::EntryCrossing, "multi-entry-crossing");
}

/// The EXIT CROSSING against a real node.
///
/// Blinded receipts spent into explicit destinations beside the blinded
/// absorber a nonzero consumed blinder sum requires. It is registered
/// exactly as the six homogeneous shapes are, because it runs through
/// the same ceremony: the crossing changes what is built, not how it is
/// funded, signed or submitted.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_exit_crossing_shape_is_submitted_to_a_real_target() {
    use vectors::live_multi_shapes::PrivateShape;

    run_one_multi_shape(PrivateShape::ExitCrossing, "multi-exit-crossing");
}

fn assert_proofs_match_the_shape(
    shape: vectors::live_multi_shapes::PrivateShape,
    record: &vectors::live_multi_shapes::MultiShapeRecord,
) {
    // TWO kinds of output carry an explicit value and therefore no range
    // proof, and they are counted separately rather than added together.
    // A fee has no program and an explicit destination has one, so a
    // ceremony that built a fee where a destination belonged would still
    // satisfy a single combined count -- and that substitution is
    // precisely the one that turns a spendable receipt into value the
    // chain treats as paid away.
    let explicit = shape.fee_output_count() + shape.explicit_destination_count();
    let proving = record.output_count() - explicit;
    assert_eq!(
        record
            .output_witness_proof_bytes()
            .iter()
            .filter(|bytes| **bytes > 2)
            .count(),
        proving,
        "a committed output carried no range proof: {:?}",
        record.output_witness_proof_bytes(),
    );
    assert_eq!(
        record
            .output_witness_proof_bytes()
            .iter()
            .filter(|bytes| **bytes == 0)
            .count(),
        explicit,
        "an explicit-valued output's witness entry is empty, and only one's is: {:?}",
        record.output_witness_proof_bytes(),
    );
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

    // The predecessor is the confidential one the ceremony asked for,
    // and its COUNT is the shape's own choice of predecessor rather than
    // a constant: the merge funds a three-output predecessor because a
    // two-output one funded from an explicit input can only offer it an
    // inverse pair.
    //
    // An EXPLICITLY funded shape names a predecessor it never funds, so
    // its width is the count it asked the node for rather than the
    // fixture's. Stated as its own case rather than folded into the
    // other: the two are different questions, and a single expression
    // covering both would stop catching a confidential ceremony that
    // funded the wrong predecessor.
    let expected_coins = match shape.explicit_funding() {
        Some((outputs, _)) => outputs as usize,
        None => shape.predecessor().outputs(),
    };
    assert_eq!(
        record.coins().len(),
        expected_coins,
        "the node funded a predecessor of a different width than the shape asked for",
    );
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
    assert_proofs_match_the_shape(shape, record);

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

// --- §15.1: the positive explicit table, one run per row ---

/// One §15.1 shape, funded, submitted, mined and read back.
///
/// The explicit-lane sibling of [`run_one_multi_shape`], and deliberately
/// the same shape of function: the two lanes differ in what they build and
/// not in how a run is judged. Nothing here asserts what the node decided.
/// What it asserts is that a run completed, that the shape the ceremony
/// reports is the shape it was asked for, and — where an acceptance
/// happened — that the two origins agree.
fn run_one_explicit_shape(shape: vectors::live_explicit_shapes::ExplicitShape, extension: &str) {
    use vectors::live_explicit_shapes::{ExplicitShapePlanner, render_explicit_shape};

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

    let mut planner = ExplicitShapePlanner::for_shape(shape, identifier(&genesis))
        .expect("the shape ceremony builds");
    let started = Instant::now();
    let outcome = execute_operations(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    let rendered = render_explicit_shape(record);
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
    // one. It is never a target verdict, so the run stops here rather
    // than pretending the node said anything.
    if let Some(refusal) = record.refusal() {
        panic!("the explicit shape ceremony refused before the node: {refusal:?}");
    }
    outcome.expect("the ceremony reached the target");

    // The shape's own cardinalities, read off the record rather than off
    // the shape's name.
    assert_eq!(
        record.coins().len(),
        shape.input_count(),
        "the node funded a different number of coins than the shape asked for",
    );
    assert!(
        record
            .coins()
            .iter()
            .all(vectors::live_explicit_shapes::ObservedShapeCoin::matches_expectation),
        "the node reported a coin the ceremony did not ask for",
    );
    assert_eq!(record.input_count(), shape.input_count());
    assert_eq!(record.output_count(), shape.output_count());

    // The candidate reached the node.
    assert!(record.submitted_bytes() > 0);
    assert!(record.observed_layer().is_some(), "no layer was observed");

    // The two origins, where an acceptance was observed. Both are
    // conditions on an acceptance rather than assertions that one
    // happened.
    if let Some(check) = record.reverification() {
        assert!(
            check.readback_matches_submission(),
            "the bytes the node reported are not the bytes it was handed",
        );
        assert!(
            check.every_input_verified(),
            "an accepted signature does not verify against its recomputed message",
        );
        assert_eq!(
            check.inputs().len(),
            shape.input_count(),
            "the read-back copy carries a witness for every input",
        );
    }

    // The run says in its own bytes what it did not establish.
    assert!(rendered.contains("evidences_no_negative_case true"));
    assert!(rendered.contains("builds_no_sponsor_region true"));
}

/// §15.1 `one-input-to-one-output`.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_explicit_one_to_one_shape_is_submitted_to_a_real_target() {
    use vectors::live_explicit_shapes::ExplicitShape;

    run_one_explicit_shape(ExplicitShape::OneToOne, "explicit-one-to-one");
}

/// §15.1 `one-input-split-into-two`.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_explicit_split_shape_is_submitted_to_a_real_target() {
    use vectors::live_explicit_shapes::ExplicitShape;

    run_one_explicit_shape(ExplicitShape::SplitIntoTwo, "explicit-split");
}

/// §15.1 `several-inputs-merged-into-one`.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_explicit_merge_shape_is_submitted_to_a_real_target() {
    use vectors::live_explicit_shapes::ExplicitShape;

    run_one_explicit_shape(ExplicitShape::MergedIntoOne, "explicit-merge");
}

/// §15.1 `several-inputs-to-several-outputs`.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_explicit_several_to_several_shape_is_submitted_to_a_real_target() {
    use vectors::live_explicit_shapes::ExplicitShape;

    run_one_explicit_shape(
        ExplicitShape::SeveralToSeveral,
        "explicit-several-to-several",
    );
}

/// §15.1 `repeated-owner`: one owner authorizes two separate inputs.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_explicit_repeated_owner_shape_is_submitted_to_a_real_target() {
    use vectors::live_explicit_shapes::ExplicitShape;

    run_one_explicit_shape(ExplicitShape::RepeatedOwner, "explicit-repeated-owner");
}

/// §15.1 `several-distinct-owners`: two inputs under two published owners.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_explicit_several_owners_shape_is_submitted_to_a_real_target() {
    use vectors::live_explicit_shapes::ExplicitShape;

    run_one_explicit_shape(
        ExplicitShape::SeveralDistinctOwners,
        "explicit-several-owners",
    );
}

/// §15.1 `one-destination-owner`: every output created for one owner.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_explicit_one_destination_owner_shape_is_submitted_to_a_real_target() {
    use vectors::live_explicit_shapes::ExplicitShape;

    run_one_explicit_shape(
        ExplicitShape::OneDestinationOwner,
        "explicit-one-destination-owner",
    );
}

/// §15.1 `several-destination-owners`.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_explicit_several_destination_owners_shape_is_submitted_to_a_real_target() {
    use vectors::live_explicit_shapes::ExplicitShape;

    run_one_explicit_shape(
        ExplicitShape::SeveralDestinationOwners,
        "explicit-several-destination-owners",
    );
}

/// §15.1 `semantic-boundary-values`: the smallest destination the request
/// type admits, and the remainder.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_explicit_boundary_values_shape_is_submitted_to_a_real_target() {
    use vectors::live_explicit_shapes::ExplicitShape;

    run_one_explicit_shape(
        ExplicitShape::SemanticBoundaryValues,
        "explicit-boundary-values",
    );
}

/// §15.1 `canonical-input-normalization`: the receipts offered in the
/// reverse of their canonical order.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_explicit_normalization_shape_is_submitted_to_a_real_target() {
    use vectors::live_explicit_shapes::ExplicitShape;

    run_one_explicit_shape(
        ExplicitShape::CanonicalInputNormalization,
        "explicit-normalization",
    );
}

/// §15.1 `sponsorless`.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_explicit_sponsorless_shape_is_submitted_to_a_real_target() {
    use vectors::live_explicit_shapes::ExplicitShape;

    run_one_explicit_shape(ExplicitShape::Sponsorless, "explicit-sponsorless");
}

/// §15.1 `candidate-maximum-inputs`.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_explicit_maximum_inputs_shape_is_submitted_to_a_real_target() {
    use vectors::live_explicit_shapes::ExplicitShape;

    run_one_explicit_shape(ExplicitShape::MaximumInputs, "explicit-maximum-inputs");
}

/// §15.1 `candidate-maximum-outputs`.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_explicit_maximum_outputs_shape_is_submitted_to_a_real_target() {
    use vectors::live_explicit_shapes::ExplicitShape;

    run_one_explicit_shape(ExplicitShape::MaximumOutputs, "explicit-maximum-outputs");
}

/// The owner fee matrix's last cell: sponsorless, EXPLICIT, self-paid.
///
/// Not a §15.1 row. The explicit positive table is complete at sixteen
/// rows and a transfer that pays its own fee is none of the sixteen
/// classes, so what this run answers is the owner fee matrix and its
/// evidence is the explicit run of record.
///
/// # What only this run can establish
///
/// The explicit conservation leaf carries the fee as a TERM for
/// fee-bearing shapes, and until this run nothing had ever executed that
/// clause: the leaf was emitted at link time by a vocabulary no request
/// could select, because the explicit lane declared no fee destination.
/// A leaf that is emitted and never run is a leaf whose arithmetic has
/// been reviewed and never checked against a target, so this submission
/// is the first thing that can tell the two apart.
///
/// It is also the first sponsorless form to face RELAY on its own. The
/// sponsorless forms before it paid no fee and travelled as package
/// children, which is why the ABI builds them at the topology-restricted
/// version; a form that pays its own fee needs no package parent, and
/// what the node does with it at that version is recorded here rather
/// than predicted.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_explicit_self_paid_fee_shape_is_submitted_to_a_real_target() {
    use vectors::live_explicit_shapes::ExplicitShape;

    run_one_explicit_shape(ExplicitShape::SelfPaidFee, "explicit-self-paid-fee");
}

/// §15.3's two witness-content rows, with their control, on one chain.
///
/// # Why the control and the mutants are one test
///
/// A refusal is attributable to a row's own class only when the
/// unmutated form is ACCEPTED and the mutated form is refused. This run
/// submits the unmutated one-input one-output candidate first, then the
/// same finalized candidate twice more with its one signature position
/// offering something else — nothing at all, and then bytes of the
/// selected width that are not a signature.
///
/// Nothing here asserts what the node decided. What it asserts is that
/// all three submissions were answered and that the mutants differ from
/// the control in one run of bytes, which is the condition under which
/// the verdicts mean anything at all.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_witness_content_negatives_are_offered_beside_their_control() {
    use vectors::live_explicit_shapes::{ExplicitShapePlanner, render_explicit_shape};

    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let base = environment("TRIPOD_LIVE_REPORT")
        .map(PathBuf::from)
        .expect("TRIPOD_LIVE_REPORT names where the transcript is written");
    let report = base.with_extension("explicit-witness-negatives");

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

    let mut planner = ExplicitShapePlanner::for_witness_negatives(identifier(&genesis))
        .expect("the negative ceremony builds");
    let started = Instant::now();
    let outcome = execute_operations(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    let rendered = render_explicit_shape(record);
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

    if let Some(refusal) = record.refusal() {
        panic!("the negative ceremony refused before the node: {refusal:?}");
    }
    outcome.expect("the ceremony reached the target");

    // Every case was answered.
    assert!(
        record.observed_layer().is_some(),
        "the control was answered"
    );
    assert_eq!(
        record.negatives().len(),
        2,
        "one observation per witness-content mutation",
    );

    // The attributability condition, measured rather than argued: each
    // mutant differs from the control in exactly one run of bytes.
    for negative in record.negatives() {
        assert!(
            negative.differs_from_control_in_one_item(),
            "{:?} does not differ from the control in one run of bytes",
            negative.mutation(),
        );
    }

    // Where the control was accepted, the two origins hold for it.
    if let Some(check) = record.reverification() {
        assert!(check.readback_matches_submission());
        assert!(check.every_input_verified());
    }
}

// --- §15.1: the sponsored pair, one run per row ---

/// One sponsored shape, funded, signed, submitted, mined and read back.
///
/// The sponsored sibling of [`run_one_explicit_shape`] and deliberately
/// the same shape of function. What it asserts is that a run completed,
/// that the sponsor round trip bound to the exact finalized bytes, and —
/// where an acceptance happened — that the node's own copy agrees with
/// what was handed to it and carries the shape that was asked for.
///
/// It asserts nothing about what the node decided. A refusal is written
/// down as the target typed it and the run stops, which is what a typed
/// stop is made of.
fn run_one_sponsor_shape(
    shape: vectors::live_sponsor_shapes::SponsorShape,
    value_form: vectors::live_sponsor_shapes::SponsorValueForm,
    extension: &str,
) {
    use vectors::live_sponsor_shapes::{
        SponsorShapePlanner, SponsorValueForm, render_sponsor_shape,
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

    let mut planner = match value_form {
        SponsorValueForm::Explicit => SponsorShapePlanner::for_shape(shape, identifier(&genesis)),
        SponsorValueForm::Committed => {
            SponsorShapePlanner::for_committed_sponsor_value(shape, identifier(&genesis))
        }
    }
    .expect("the sponsored ceremony builds");
    let started = Instant::now();
    let outcome = execute_operations(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    let rendered = render_sponsor_shape(record);
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

    if let Some(refusal) = record.refusal() {
        panic!("the sponsored ceremony refused before the node: {refusal:?}");
    }
    outcome.expect("the ceremony reached the target");

    judge_one_sponsor_shape(record, &rendered, shape, value_form);
}

/// What a completed sponsored run must hold.
///
/// Split from the arranging on the rule the sponsor ceremony already
/// followed before it was lifted: a function that both arranges a run
/// and judges it makes the judging hard to read past the arranging.
fn judge_one_sponsor_shape(
    record: &vectors::live_sponsor_shapes::SponsorShapeRecord,
    rendered: &str,
    shape: vectors::live_sponsor_shapes::SponsorShape,
    value_form: vectors::live_sponsor_shapes::SponsorValueForm,
) {
    // The sponsor round trip happened, and it bound to the exact bytes.
    let round = record.round().expect("the sponsor round trip completed");
    assert!(
        round.echo_matches(),
        "the adapter signed bytes other than the ones it was sent",
    );
    assert!(
        round.replay_changed_the_control(),
        "the replay changed nothing, so no returned witness reached the control",
    );
    assert!(
        round.witness_reached_the_control(),
        "a returned witness item is absent from the replayed control",
    );

    // The binding is enforced rather than announced: the same witness,
    // bound to one mutated byte, is refused.
    let mutated = round
        .mutated_refusal()
        .expect("a signature bound to mutated bytes was refused");
    assert!(
        mutated.contains("SponsorSignatureBindingMismatch"),
        "the refusal names something other than the binding: {mutated}",
    );

    // The candidate reached the node.
    assert!(record.submitted_bytes() > 0);
    assert!(record.observed_layer().is_some(), "no layer was observed");

    // The origins, where an acceptance was observed. Every one is a
    // condition ON an acceptance rather than an assertion that one
    // happened.
    if let Some(check) = record.reverification() {
        assert!(
            check.readback_matches_submission(),
            "the bytes the node reported are not the bytes it was handed",
        );
        assert!(
            check.every_owner_verified(),
            "an accepted owner signature does not verify against its recomputed message",
        );
        assert!(
            check.sponsor_witness_in_readback(),
            "the adapter's witness is absent from the node's own copy",
        );

        // The change role, read out of the node's copy rather than off
        // the request. A with-change run that reached a node carrying no
        // change output is already a hard stop in the ceremony; this
        // states the positive half where a reader of the test sees it.
        match shape.change() {
            None => assert!(
                check.change().is_none(),
                "a without-change run carries a change output",
            ),
            Some(offered) => {
                let change = check.change().expect("a with-change run carries change");
                assert!(
                    change.is_the_declared_change(offered),
                    "the change output is not the one the deployment declares",
                );
            }
        }
    }

    judge_the_sponsor_value_form(record, rendered, value_form);

    assert!(rendered.contains("evidences_no_negative_case true"));
}

/// What the sponsor's VALUE FORM obliges the run to have observed.
///
/// Split from [`judge_one_sponsor_shape`] because it judges a different
/// axis. That function asks what any sponsored run owes — a round trip
/// bound to its own bytes, a candidate that reached the node, and the
/// change role where an acceptance happened. This asks what THIS run's
/// sponsor coin was, and the two grew independent enough that reading
/// one past the other had become the work.
fn judge_the_sponsor_value_form(
    record: &vectors::live_sponsor_shapes::SponsorShapeRecord,
    rendered: &str,
    value_form: vectors::live_sponsor_shapes::SponsorValueForm,
) {
    use vectors::live_sponsor_shapes::{
        CommittedSponsorCheck, SponsorValueForm, sponsored_run_of_record as record_of,
    };

    // The value form, judged where a reader of the test sees it.
    match value_form {
        SponsorValueForm::Explicit => {
            assert!(
                record.committed().is_none(),
                "an explicit run reported a committed sponsor census",
            );
            assert!(
                record.sponsor_funded().is_some(),
                "an explicit run observed no amount for its sponsor coin",
            );
            // The run says in its own bytes what it did not establish.
            assert!(rendered.contains("does_not_establish confidential-sponsor-values"));
        }
        SponsorValueForm::Committed => {
            let census = record
                .committed()
                .expect("a committed run reported no committed sponsor census");
            // Named one at a time, so a failure says WHICH check failed
            // rather than that some did.
            for check in CommittedSponsorCheck::ALL {
                assert!(
                    census.holds(check),
                    "the committed sponsor coin failed the check {}",
                    check.name(),
                );
            }
            assert!(census.every_check_held());
            // The coin exists on a chain: its funding transaction was
            // accepted and mined, and the node computed an identity for
            // it. That is a separate acceptance from the candidate's and
            // is not a substitute for one.
            assert_eq!(
                record.committed_funding_txid(),
                Some(record_of::COMMITTED_SPONSOR_FUNDING_TXID),
                "the committed sponsor coin was mined under another identity",
            );
            assert_eq!(
                record.committed_funding_weight(),
                Some(record_of::COMMITTED_SPONSOR_FUNDING_WEIGHT),
            );
            // No amount was observed for the coin the control spends,
            // which is the whole difference the axis makes.
            assert!(
                record.sponsor_funded().is_none(),
                "a committed run reported an amount for a coin whose value is a point",
            );
            // And the disclaimer moved with the subject: this run
            // establishes that confidential sponsor values are FUNDED,
            // so it no longer says it establishes nothing about them.
            assert!(!rendered.contains("does_not_establish confidential-sponsor-values"));
            assert!(rendered.contains("does_not_establish confidential-receipt-values"));
            assert!(rendered.contains("committed_sponsor_every_check_held true"));

            // The candidate that SPENDS the coin is refused, and the
            // refusal is the finding rather than a disappointment. An
            // ACCEPTANCE here would mean the arithmetic below is wrong,
            // which is why it is asserted rather than tolerated.
            //
            // The reserve sub-equation is the sponsor input against the
            // fee and the change. The input carries a blinder now, both
            // outputs that spend it are explicit and carry none, and
            // nothing in the transaction absorbs the difference — so the
            // target's balance check cannot close whatever the amounts
            // are.
            assert_eq!(
                record.observed_layer(),
                Some(
                    target_elements_conformance::protocol::ObservedOutcomeLayer::ConsensusRejectionBeforeScript
                ),
                "a committed sponsor value was not refused at the balance check",
            );
            assert!(record.accepted_txid().is_none());
            assert!(
                rendered.contains(record_of::COMMITTED_SPONSOR_REFUSAL),
                "the target named something other than its balance check",
            );

            // Attributability, MEASURED. A candidate names the coin it
            // spends by outpoint alone, so the value form is not in
            // these bytes at all: this submission and the explicit
            // control's are the same shape at the same width, and the
            // refusal is attributable to which coin was reached for.
            assert_eq!(
                record.submitted_bytes(),
                record_of::COMMITTED_SPONSOR_SUBMITTED_BYTES,
            );
            assert_eq!(
                record.submitted_bytes(),
                record_of::SPONSORED_CHANGE_SUBMITTED_BYTES,
            );
            assert_eq!(
                record.target_weight(),
                Some(record_of::COMMITTED_SPONSOR_TARGET_WEIGHT),
            );
            assert_eq!(
                record.target_weight(),
                Some(record_of::SPONSORED_CHANGE_TARGET_WEIGHT),
            );
        }
    }
}

/// §15.1 `sponsor-change-absent`: the sponsor funds the fee exactly.
///
/// The ceremony the sponsor wave ran, now driven from a lane rather than
/// from inside a test. It is kept because it is the lift's own control:
/// what it builds did not move, so its recorded identity must not
/// either.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_sponsored_change_absent_shape_is_submitted_to_a_real_target() {
    use vectors::live_sponsor_shapes::SponsorShape;

    run_one_sponsor_shape(
        SponsorShape::ChangeAbsent,
        vectors::live_sponsor_shapes::SponsorValueForm::Explicit,
        "sponsored-change-absent",
    );
}

/// §15.1 `sponsor-change-present`: the sponsor takes change back.
///
/// The first sponsored control in this workspace that takes change. The
/// sponsor coin is funded ABOVE the offer and the offer states the
/// residue, which is the whole of what was missing.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_sponsored_change_present_shape_is_submitted_to_a_real_target() {
    use vectors::live_sponsor_shapes::SponsorShape;

    run_one_sponsor_shape(
        SponsorShape::ChangePresent,
        vectors::live_sponsor_shapes::SponsorValueForm::Explicit,
        "sponsored-change-present",
    );
}

/// The with-change sponsored control, funded by a sponsor coin whose
/// VALUE is committed.
///
/// The first transaction this workspace offers a target that spends a
/// blinded sponsor value. Its asset stays explicit, because the covenant
/// introspects it; its fee stays explicit, because consensus defines a
/// fee by its explicitness; and the receipts stay explicit, because this
/// run varies ONE thing against the control above it and the receipts
/// are not it.
///
/// What the run has to establish before it reaches a node is that the
/// value the chain holds for the sponsor coin is the value this
/// workspace derives from published constants and no chain at all. That
/// check is in the ceremony rather than here, and it stops the run
/// rather than reporting a finding: a run whose two copies disagreed
/// would have funded something blinded while being unable to say what.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_committed_sponsor_value_shape_is_submitted_to_a_real_target() {
    use vectors::live_sponsor_shapes::{SponsorShape, SponsorValueForm};

    run_one_sponsor_shape(
        SponsorShape::ChangePresent,
        SponsorValueForm::Committed,
        "sponsored-committed-value",
    );
}

/// §15.6 `missing-sponsor-authorization`, the mutant offered FIRST.
///
/// One run, one chain, two submissions: a sponsored control whose sponsor
/// input carries no authorization, and then the unmutated control. The
/// order is the evidence — a sponsor-witness mutation leaves the identity
/// alone, so a control submitted first would make its own mutant come
/// back `txn-already-known` at a layer before script evaluation.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_missing_sponsor_authorization_negative_is_refused_behind_its_control() {
    use vectors::live_sponsor_shapes::{SponsorShape, SponsorShapePlanner, render_sponsor_shape};

    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let base = environment("TRIPOD_LIVE_REPORT")
        .map(PathBuf::from)
        .expect("TRIPOD_LIVE_REPORT names where the transcript is written");
    let report = base.with_extension("sponsored-missing-authorization");

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

    let mut planner = SponsorShapePlanner::for_missing_authorization_negative(
        SponsorShape::ChangeAbsent,
        identifier(&genesis),
    )
    .expect("the sponsored ceremony builds");
    let started = Instant::now();
    let outcome = execute_operations(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    let rendered = render_sponsor_shape(record);
    std::fs::write(&report, &rendered).expect("the transcript is written");
    std::fs::write(
        timing_path(&report),
        format!("wall_seconds {:.1}\n", wall.as_secs_f64()),
    )
    .expect("the run's wall time is written");

    if let Some(refusal) = record.refusal() {
        panic!("the sponsored ceremony refused before the node: {refusal:?}");
    }
    outcome.expect("the ceremony reached the target");

    // The mutant was offered and answered.
    assert_eq!(
        record.negatives().len(),
        1,
        "one observation per mutant offering",
    );
    let negative = &record.negatives()[0];

    // Attributability, measured rather than argued: the mutant and its
    // control come from ONE finalization and differ in the sponsor
    // witness and in nothing else.
    assert!(
        negative.differs_from_control_in_the_sponsor_witness(),
        "the mutant differs from its control somewhere other than the sponsor witness",
    );

    // A sponsored control whose sponsor input authorizes nothing must
    // not be accepted. This is the finding if it fires.
    assert_ne!(
        negative.layer(),
        target_elements_conformance::protocol::ObservedOutcomeLayer::Accepted,
        "a sponsor input carrying no authorization was ACCEPTED",
    );
    assert!(
        negative.detail().is_some_and(|detail| !detail.is_empty()),
        "the target refused and said nothing, so the row has no verdict to cite",
    );

    // And the CONTROL that followed was accepted, which is the whole of
    // what makes the refusal attributable rather than merely recorded.
    let check = record
        .reverification()
        .expect("the unmutated control was accepted behind the mutant");
    assert!(check.readback_matches_submission());
    assert!(check.every_owner_verified());
    assert!(check.sponsor_witness_in_readback());

    assert!(rendered.contains("evidences_no_negative_case false"));
}

// --- §16.1: the pairs arc, one fixture materialized twice -------------

/// The PAIRS ARC: one §16.1 semantic fixture, both its materializations
/// submitted to ONE node, and the relation over the two acceptances.
///
/// # Why this is one test and cannot be two
///
/// §6.6 asks a pair's members to carry the same exact explicit `U`, and
/// each of these ignored tests spins its own disposable chain and issues
/// its own asset. Two tests would therefore be two assets and two chains,
/// and the pair's fifth term would disagree for a reason that has nothing
/// to do with representation. So the arc issues once, submits the
/// explicit member, hands the private half the asset the first one
/// issued, and submits the private member — one ceremony, one node.
///
/// # What this run establishes that no earlier run could
///
/// The campaign already had an accepted explicit one-to-one and an
/// accepted private strict one-to-one, and they are not a pair: §16.1
/// requires ONE semantic fixture materialized twice, and those two are
/// independent ceremonies whose shapes merely match. Both members here
/// read the same fixture and neither shape carries a literal of its own,
/// so what is submitted is the pair rather than two things that resemble
/// one.
///
/// # Nothing here decides what the node should have said
///
/// The assertions are about a run that COMPLETED and about the arc's own
/// record of it. Where a member was not accepted the arc refuses before
/// it has a ledger, and the refusal is reported with the target's own
/// words in the transcript rather than asserted away.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_pairs_arc_submits_both_members_of_one_fixture_to_a_real_target() {
    use vectors::live_pair_arc::{PairArcPlanner, render_pair_arc};

    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let base = environment("TRIPOD_LIVE_REPORT")
        .map(PathBuf::from)
        .expect("TRIPOD_LIVE_REPORT names where the transcript is written");
    let report = base.with_extension("pairs-arc");

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

    let mut planner = PairArcPlanner::new(identifier(&genesis)).expect("the arc ceremony builds");
    let started = Instant::now();
    let outcome = execute_operations(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    let rendered = render_pair_arc(record);
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

    if let Some(refusal) = record.refusal() {
        panic!("the pairs arc refused before it had a ledger: {refusal:?}");
    }
    outcome.expect("the ceremony reached the target");

    // The arc's own fixture, and the fact every later claim rests on.
    assert!(record.fixture_conserves());

    let ledger = record.ledger().expect("a completed arc writes a ledger");

    // ONE asset, which is what one issuance buys and what §6.6's fifth
    // term needs.
    assert_eq!(
        record.issued_asset(),
        Some(ledger.issued_asset()),
        "the ledger names an asset the run did not issue",
    );
    assert_eq!(
        ledger.explicit().projection().explicit_asset(),
        ledger.private().projection().explicit_asset(),
        "the two members do not carry one exact explicit U",
    );

    // Both members at the acceptance bar every recorded run meets: an
    // identity the target computed, a copy read back equal to what it was
    // handed, and every input's signature verified against a message
    // recomputed here.
    for member in [ledger.explicit(), ledger.private()] {
        assert!(
            member.meets_the_acceptance_bar(),
            "{:?} did not meet the acceptance bar",
            member.member(),
        );
    }

    // The relation, in the row's own terms.
    let observation = ledger.observation();
    assert!(
        observation.projections_are_equal(),
        "a §6.6 term disagrees: {:?}",
        observation.terms(),
    );

    // And the pair is evidence of MINIMALITY rather than of similarity:
    // the private member withholds exact amounts the explicit member
    // publishes. A pair whose private half published everything would
    // establish nothing about disclosure.
    assert!(observation.terms_withheld_by_the_private_member() > 0);
    assert!(
        ledger
            .explicit()
            .projection()
            .publishes_every_exact_amount()
    );
    assert!(
        !ledger.private().projection().publishes_every_exact_amount(),
        "the private member published an exact amount",
    );
    assert!(ledger.supports_the_projection_equality_row());

    assert_the_arc_ledger_matches_the_run_of_record(record);

    // The run says in its own bytes what it did not establish.
    assert!(rendered.contains("evidences_no_negative_case true"));
    assert!(rendered.contains("builds_no_sponsor_region true"));
}

/// The arc's ledger, held against the identities and figures its own run
/// of record carries.
///
/// # Why this was unbound, and why binding it costs no report
///
/// The assertions above recompute PROPERTIES — the acceptance bar, the
/// projection relation, the disclosure asymmetry — and the strongest
/// thing they said about identity was that the two accepted txids
/// DIFFER. Two txids differ in every run, so the exact provenance the
/// phase card and `PairedRelationObserved` cite could move while the
/// lane and the static standing both stayed green. That is F3's
/// silent-drift hazard on the row this arc answers. The transcript is
/// written to disk BEFORE any assertion in this test runs, so binding
/// these costs no report on the day a figure changes: the artifact
/// carries what the node said either way, and the lane now FAILS instead
/// of passing over a record nothing reads.
fn assert_the_arc_ledger_matches_the_run_of_record(record: &vectors::live_pair_arc::PairArcRecord) {
    use vectors::live_pair_arc::{REPRESENTATION_EQUIVALENCE_TERMS, run_of_record};

    let ledger = record.ledger().expect("a completed arc writes a ledger");
    assert_eq!(
        record.ledger().is_some(),
        run_of_record::A_PAIR_ARC_LEDGER_EXISTS,
        "the flag the evidence matrix reads disagrees with the run",
    );

    // ONE asset, and the one the run of record names. §6.6's exact
    // explicit U term is a claim about THIS asset, not about some asset.
    assert_eq!(
        ledger.issued_asset(),
        run_of_record::PAIR_ISSUED_ASSET,
        "the arc issued an asset the run of record does not carry",
    );

    // The two accepted identities, each against its own recorded
    // constant. These are the figures the phase card prints and the
    // paired-relation standing rests on, and they are what a reader
    // checks against a chain.
    assert_eq!(
        Some(ledger.explicit().accepted_txid()),
        run_of_record::EXPLICIT_MEMBER_ACCEPTED_IDENTITY,
        "the explicit member was accepted at an identity the run of record does not carry",
    );
    assert_eq!(
        Some(ledger.private().accepted_txid()),
        run_of_record::PRIVATE_MEMBER_ACCEPTED_IDENTITY,
        "the private member was accepted at an identity the run of record does not carry",
    );
    assert_ne!(
        ledger.explicit().accepted_txid(),
        ledger.private().accepted_txid(),
        "the two members are one transaction",
    );

    // The two widths and the two weights, each the target's own figure.
    // The eight-fold gap between them is the REPRESENTATION, and it is
    // the measurement the arc exists to make.
    assert_eq!(
        ledger.explicit().submitted_bytes(),
        run_of_record::EXPLICIT_MEMBER_SUBMITTED_BYTES,
        "the explicit member handed the node a different number of bytes",
    );
    assert_eq!(
        ledger.private().submitted_bytes(),
        run_of_record::PRIVATE_MEMBER_SUBMITTED_BYTES,
        "the private member handed the node a different number of bytes",
    );
    assert_eq!(
        ledger.explicit().target_weight(),
        Some(run_of_record::EXPLICIT_MEMBER_TARGET_WEIGHT),
        "the target computed a weight for the explicit member the run of record does not carry",
    );
    assert_eq!(
        ledger.private().target_weight(),
        Some(run_of_record::PRIVATE_MEMBER_TARGET_WEIGHT),
        "the target computed a weight for the private member the run of record does not carry",
    );

    // The withheld count, EXACTLY, and out of the full §6.6 term list
    // rather than out of however many terms happened to be compared. Two
    // of eleven is the whole disclosure-minimality claim: zero would mean
    // the private representation published everything, and a shortened
    // term list would let a compared-nothing run report the same two.
    let observation = ledger.observation();
    assert_eq!(
        observation.terms().len(),
        REPRESENTATION_EQUIVALENCE_TERMS.len(),
        "the pair was compared on fewer §6.6 terms than the section names",
    );
    assert_eq!(
        observation.terms_withheld_by_the_private_member(),
        run_of_record::TERMS_WITHHELD_BY_THE_PRIVATE_MEMBER,
        "the private member withheld a different number of terms than the run of record carries",
    );
}
