//! The Wave-11 operation lane: one compact-ASH run against a real node.
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

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding, LeafVersion,
    reviewed_elements_tapscript, validate_reviewed_development_binding,
};
use target_elements_conformance::executor::{
    DEFAULT_EXECUTOR_TIMEOUT, ExecutorConfiguration, ExecutorTrust, execute_operations,
};
use target_elements_conformance::protocol::ObservedOutcomeLayer;
use vectors::comparison::{compare, read_accepted};
use vectors::fixture::{OPERATION, positive_semantic_census};
use vectors::materialize::{TargetVectorId, vector_id};
use vectors::matrix::EvidenceBoundary;
use vectors::operation::{CompactAshOperationPlanner, OperationTranscript};
use vectors::plan::{ProjectionComparison, derive_evidence_plan};

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

fn hex(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(text, "{byte:02x}");
    }
    text
}

/// The §17.4 comparison for every vector the target accepted.
///
/// Performed here rather than inside the planner, because it needs both
/// halves and the planner holds only one: the expectation came from the
/// realization layer's own arithmetic over the fixture, and the
/// observation is read back out of the bytes the target took and the
/// coins the target said it created. A vector the target did not accept
/// gets no entry at all — §1.4's second verdict does not exist for a
/// transaction that never reached the first.
fn compare_projections(
    transcript: &OperationTranscript,
) -> BTreeMap<TargetVectorId, (ProjectionComparison, String)> {
    let mut verdicts = BTreeMap::new();
    let (Some(program), Some(asset)) =
        (transcript.constructor_program(), transcript.issued_asset())
    else {
        return verdicts;
    };
    let census = positive_semantic_census().expect("the positive census builds");

    for submission in transcript.submissions() {
        if submission.layer() != ObservedOutcomeLayer::Accepted {
            continue;
        }
        let id = submission.vector();
        let Some(case) = census.iter().find(|case| vector_id(case) == id) else {
            continue;
        };
        // Only the coins this vector was funded with. Handing over every
        // coin the run created would let one vector's input read as
        // another's, and the sponsor count is derived from exactly the
        // inputs that are not in this set.
        let mine: BTreeMap<_, _> = transcript
            .funded()
            .get(&id)
            .map(|outpoints| {
                outpoints
                    .iter()
                    .filter_map(|outpoint| {
                        transcript
                            .coins()
                            .get(outpoint)
                            .map(|amount| (*outpoint, *amount))
                    })
                    .collect()
            })
            .unwrap_or_default();

        let verdict = match read_accepted(submission.bytes(), &mine, asset, program, OPERATION) {
            // A transaction that could not be read has not disagreed
            // about anything, so the comparison was not performed. It
            // must not be recorded as a difference, which would file a
            // reading failure as a protocol finding.
            Err(refusal) => (
                ProjectionComparison::NotPerformed,
                format!("unreadable: {refusal:?}"),
            ),
            Ok(observed) => {
                let differed = compare(case.expected(), &observed);
                if differed.is_empty() {
                    (ProjectionComparison::Matched, "matched".to_owned())
                } else {
                    let names: Vec<&str> = differed.iter().map(|term| term.name()).collect();
                    (
                        ProjectionComparison::Differed,
                        format!("differed: {}", names.join(" ")),
                    )
                }
            }
        };
        verdicts.insert(id, verdict);
    }
    verdicts
}

/// Write everything the run established, and nothing it did not.
/// How many §19 coverage rows this run's outcomes actually discharge.
///
/// The plan decides, not this lane: each outcome states a vector, the
/// layer the target put it at, and what the projection comparison found,
/// and `CoverageObservation::is_discharged` is what turns the triple
/// into coverage or refuses to. An acceptance whose projection was never
/// compared discharges nothing, which is why the comparison above
/// distinguishes not-performed from differed.
fn coverage(
    transcript: &OperationTranscript,
    projections: &BTreeMap<TargetVectorId, (ProjectionComparison, String)>,
) -> (usize, usize, usize) {
    let bundle = vectors::bundle::fixture_bundle().expect("the fixture bundle builds");
    let mut plan = derive_evidence_plan(&bundle).expect("the evidence plan derives");
    let outcomes: Vec<_> = transcript
        .submissions()
        .iter()
        .map(|submission| {
            let projection = projections
                .get(&submission.vector())
                .map_or(ProjectionComparison::NotPerformed, |(verdict, _)| *verdict);
            (submission.vector(), submission.layer(), projection)
        })
        .collect();
    plan.discharge(&outcomes);
    (
        plan.census().coverage_requirements(),
        plan.observed_rows(),
        plan.discharged_rows(),
    )
}

fn render(
    transcript: &OperationTranscript,
    wall: Duration,
    outcome_text: &str,
    projections: &BTreeMap<TargetVectorId, (ProjectionComparison, String)>,
) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    let _ = writeln!(out, "  \"wall_seconds\": {:.1},", wall.as_secs_f64());
    let _ = writeln!(out, "  \"run\": {outcome_text},");
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
    let _ = writeln!(
        out,
        "  \"projections_compared\": {}, \"projections_matched\": {},",
        projections.len(),
        projections
            .values()
            .filter(|(verdict, _)| *verdict == ProjectionComparison::Matched)
            .count()
    );
    let (requirements, observed, discharged) = coverage(transcript, projections);
    let _ = writeln!(
        out,
        "  \"coverage_requirements\": {requirements}, \"coverage_observed\": {observed}, \"coverage_discharged\": {discharged},"
    );

    out.push_str(&render_divergences(transcript));

    out.push_str("  \"submissions\": [\n");
    for (index, submission) in transcript.submissions().iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        let _ = write!(
            out,
            "    {{\"ordinal\": {}, \"ash_inputs\": {}, \"sponsors\": {}, \"layer\": {}, \"txid\": {}, \"projection\": {}, \"detail\": {}, \"bytes\": {}}}",
            submission.vector().fixture().ordinal(),
            submission.vector().ash_inputs(),
            submission.vector().sponsors(),
            quote(&submission.layer().to_string()),
            submission
                .accepted_txid()
                .map_or_else(|| "null".to_owned(), quote),
            projections
                .get(&submission.vector())
                .map_or_else(|| quote("not performed"), |(_, text)| quote(text)),
            submission.detail().map_or_else(|| "null".to_owned(), quote),
            submission.bytes().len(),
        );
    }
    out.push_str("\n  ],\n");
    out.push_str(&render_mutations(transcript));
    out.push_str("}\n");
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
/// whether those two agree — as separate fields, because a row
/// recording only the agreement would hide which way a disagreement
/// went.
///
/// Whether the mutation could even reach its expected boundary is a
/// further fact: an arm that unbalances the closed asset is answered by
/// consensus before any script runs, so a script-path expectation is
/// unreachable for it whatever the covenant would have said.
fn render_mutations(transcript: &OperationTranscript) -> String {
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
    let mut out = String::new();
    let _ = writeln!(
        out,
        "  \"mutation_control\": {{\"subject_ordinal\": {}, \"layer\": {}, \"mutations_attributable\": {}}},",
        subject.map_or_else(
            || "null".to_owned(),
            |id| id.fixture().ordinal().to_string()
        ),
        control.map_or_else(
            || "null".to_owned(),
            |submission| quote(&submission.layer().to_string())
        ),
        // The whole negative half's licence, stated once: the control
        // was accepted, so the mutations submitted before it differ from
        // an accepted transaction by exactly what each one changed.
        control.is_some_and(|submission| submission.layer() == ObservedOutcomeLayer::Accepted),
    );
    out.push_str("  \"mutations\": [\n");
    for (index, mutant) in transcript.mutants().iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        let expected = mutant
            .mutation()
            .expected_boundary()
            .expect("the mutation stages a class §18 names");
        let observed = mutant.layer();
        let _ = write!(
            out,
            "    {{\"mutation\": {}, \"class\": {}, \"origin_ordinal\": {}, \"expected_boundary\": {}, \"observed_layer\": {}, \"boundary_matched\": {}, \"target_verdict\": {}, \"preserves_value_balance\": {}, \"detail\": {}, \"bytes\": {}}}",
            quote(&format!("{:?}", mutant.mutation())),
            quote(mutant.mutation().class_name()),
            mutant.origin().fixture().ordinal(),
            quote(&format!("{expected:?}")),
            quote(&observed.to_string()),
            matches_boundary(expected, observed),
            observed.is_target_verdict(),
            mutant.mutation().preserves_value_balance(),
            mutant.detail().map_or_else(|| "null".to_owned(), quote),
            mutant.bytes().len(),
        );
    }
    out.push_str("\n  ]\n");
    out
}

/// Whether an observed layer is the §1.5 boundary a class expected.
///
/// The two vocabularies are different types and this is the only place
/// they are put side by side. It is a total function over the observed
/// layer rather than a lookup with a fallback, so a layer nobody
/// considered fails to match rather than matching by accident.
fn matches_boundary(expected: EvidenceBoundary, observed: ObservedOutcomeLayer) -> bool {
    // Exhaustive over the expected side, which is this workspace's own
    // vocabulary: a boundary added to §1.5 has to be given an
    // observable layer here or this stops compiling. The observed side
    // is then a single equality, so no layer can satisfy a boundary by
    // falling through a wildcard.
    let required = match expected {
        EvidenceBoundary::ConsensusRejectionBeforeScript => {
            ObservedOutcomeLayer::ConsensusRejectionBeforeScript
        }
        EvidenceBoundary::ScriptPathRejection => ObservedOutcomeLayer::ScriptPathRejection,
        EvidenceBoundary::RelayPolicyRejection => ObservedOutcomeLayer::RelayPolicyRejection,
        EvidenceBoundary::AcceptedTransaction => ObservedOutcomeLayer::Accepted,
        // Boundaries no submission reaches. A pre-target refusal happens
        // before a target is asked, an infrastructure failure is not a
        // target fact, and a report-layer verdict is made after
        // acceptance rather than by the target — so no observed layer
        // satisfies one of these.
        EvidenceBoundary::SemanticRequestRejection
        | EvidenceBoundary::CompilerPlanRejection
        | EvidenceBoundary::BackendEmissionRejection
        | EvidenceBoundary::LinkerRejection
        | EvidenceBoundary::AbiConstructionRejection
        | EvidenceBoundary::ExecutorInfrastructureFailure
        | EvidenceBoundary::ReportSemanticProjectionRejection => return false,
    };
    observed == required
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
    );

    let mut planner = CompactAshOperationPlanner::new().expect("the planner builds");
    let started = Instant::now();
    let outcome = execute_operations(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let transcript = planner.transcript();
    let outcome_text = match &outcome {
        Ok(_) => quote("completed"),
        Err(error) => quote(&format!("refused: {error}")),
    };
    let projections = compare_projections(transcript);
    let out = render(transcript, wall, &outcome_text, &projections);
    std::fs::write(&report, &out).expect("the transcript is written");

    // A run that reached the target at all answered every step it asked
    // for. Nothing here says what the answers were.
    if outcome.is_ok() {
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
            projections.len(),
            accepted,
            "an accepted transaction went uncompared"
        );

        check_negative_half(transcript, accepted);
    }
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
