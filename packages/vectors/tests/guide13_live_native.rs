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
