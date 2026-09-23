//! Exact STATE constructor continuity across three public byte sources.
//!
//! A submit-ready candidate is built through the public transaction chain.
//! Two admitted archives are replayed through the public native planner.
//! Their submitted bytes pass through projection, report assembly, validation,
//! and canonical rendering in one integration binary.
//! Independent reconstructions pin the bytes and the report's determinism.
//! Each refusal changes one input axis and identifies the refusing boundary.
//!
//! These host comparisons run no node and submit no transaction.
//! A projection supplies neither target acceptance nor a row standing.
//! Caller-stated branch context supplies no root-freshness proof.
//! Even the accepted archive's host projection retains an unmet acceptance
//! obligation because that projection makes no target observation.

use linker::{CandidateDeploymentIdentity, CandidateLinkedMaturityBundle};
use realization::{Cycle, announce_maturity};
use tapscript::{CandidateStateConstructor, StateWitnessSchedule, state_output_program_at_nonce};
use target_elements::ReviewedElementsTapscriptDefinition;
use target_elements_conformance::constructor::tagged::sha256;
use target_elements_conformance::executor::TargetOperationPlanner;
use target_elements_conformance::protocol::{FundedOutput, OperationSubject, WireOutpoint};
use transaction::bytes::{AssetField, AssetId, Outpoint, Txid, ValueField};
use transaction::live_request::{RequestedForm, SponsorChangeRequest};
use transaction::operator_right::{BranchContext, OperatorRightRegistry};
use transaction::operator_signing::{OPERATOR_SIGHASH_TYPE_BYTE, OperatorSigningResponse};
use transaction::state_abi::derive_maturity_announcement_abi;
use transaction::state_construct::construct_maturity_announcement;
use transaction::state_finalize::finalize_maturity_announcement;
use transaction::state_request::MaturityAnnouncementRequest;
use transaction::state_signing::OperatorSigningStarted;
use transaction::state_view::{MaturityViewStatement, PublicMaturityStateView};
use vectors::live_capability::OracleLiveCurve;
use vectors::maturity_closure::{
    MaturityDeployment, MaturityWitnessSelection, OracleStateCurve, closure_target,
    linked_maturity_bundle,
};
use vectors::maturity_continuity::{
    MaturityByteSource, MaturityContinuityRefusal, MaturityFundedPredecessor,
    MaturityProjectionInput, ValidatedMaturityContinuity, project_maturity_continuity,
};
use vectors::maturity_continuity_report::{
    MaturityContinuityReport, MaturityContinuityReportRefusal, assemble_maturity_continuity_report,
    render_maturity_continuity_report, validate_maturity_continuity_report,
};
use vectors::maturity_corpus::{
    MATURITY_VARIABLE_RUN_ADDRESS, ValidatedMaturityCorpus, maturity_run_of_record,
    maturity_variable_run_of_record,
};
use vectors::maturity_evidence::{
    MaturityConstructorMaterial, MaturityConstructorMaterialAbsence,
    MaturityExecutorProvenanceExpectation, MaturityTargetBinding,
    derive_maturity_evidence_plan_with,
};
use vectors::maturity_native::{MaturityAcceptanceObligation, MaturityAnnouncementPlanner};
use vectors::maturity_operator::{OPERATOR_HANDLE, OperatorVerifier};

// --- Owned public sources ------------------------------------------------

/// One byte source and the context independently supplied for projection.
struct Source {
    origin: MaturityByteSource,
    bytes: Vec<u8>,
    funded: MaturityFundedPredecessor,
    bundle: CandidateLinkedMaturityBundle,
    identity: CandidateDeploymentIdentity,
    branch: BranchContext,
    successor: CandidateStateConstructor,
}

impl Source {
    /// Borrow the owned source as the projector's public input.
    fn input(&self) -> MaturityProjectionInput<'_> {
        MaturityProjectionInput {
            source: self.origin.clone(),
            submitted_bytes: &self.bytes,
            funded: &self.funded,
            branch: self.branch,
            bundle: &self.bundle,
            identity: &self.identity,
        }
    }

    /// Run the public projector over the owned source.
    fn project(&self) -> Result<ValidatedMaturityContinuity, MaturityContinuityRefusal> {
        project_maturity_continuity(self.input())
    }
}

// --- Public builders and local wire conversions -------------------------

/// The reviewed target used by the public announcement chain.
///
/// # Panics
///
/// Panics only if the reviewed target ceases to validate.
fn target() -> ReviewedElementsTapscriptDefinition {
    closure_target().expect("the reviewed target validates")
}

/// Derive the planner's requested cycle from the linked predecessor and lead window.
///
/// # Panics
///
/// Panics only if the linked predecessor's window overflows the cycle domain.
fn announced_cycle(bundle: &CandidateLinkedMaturityBundle) -> Cycle {
    let predecessor = bundle.instances()[0].metadata().semantic.cycle;
    let (earliest, latest) = bundle
        .deployment()
        .lead_bounds()
        .bounds()
        .window(predecessor)
        .expect("the linked predecessor has an admitted lead window");
    Cycle::new(earliest.get().saturating_add(1)).min(latest)
}

/// Build a fresh submit-ready candidate through the published signer chain.
///
/// # Panics
///
/// Panics only if the fixed published deployment, its view, signer, or target
/// ceases to admit the public construction and authorization chain.
fn node_free() -> Source {
    let bundle = linked_maturity_bundle(
        MaturityDeployment::PublishedSignerHeld,
        MaturityWitnessSelection::retained_whole_metadata(),
    )
    .expect("the published signer deployment links");
    let retained = &bundle.instances()[0];
    let parameters = MaturityDeployment::PublishedSignerHeld
        .parameters()
        .expect("the deployment resolves its singleton");
    let outpoint =
        Outpoint::new(Txid::from_internal([0x71; 32]), 0).expect("output zero is admitted");
    let branch = BranchContext::new([0x9b; 32], 12).expect("the branch identifier is nonzero");
    let funded = MaturityFundedPredecessor {
        outpoint,
        asset: AssetId::from_internal(*parameters.singleton()),
        amount: 1,
        program: retained.constructor().output_program(),
    };
    let statements = vec![
        MaturityViewStatement::CurrentStateOutpoint(outpoint),
        MaturityViewStatement::AssetAndAmount(
            AssetField::Explicit(funded.asset),
            ValueField::Explicit(funded.amount),
        ),
        MaturityViewStatement::PredecessorMetadata(retained.metadata().semantic),
        MaturityViewStatement::PredecessorRepresentationNonce(retained.metadata().representation),
        MaturityViewStatement::CurrentRootBinding(branch),
        MaturityViewStatement::PredecessorProgram(funded.program.clone()),
        MaturityViewStatement::AcceptedLinkedBundle(Box::new(bundle.clone())),
    ];
    let target = target();
    let view = PublicMaturityStateView::new(statements)
        .expect("the seven view statements are distinct")
        .validate(&target, &OracleStateCurve)
        .expect("the retained program validates the stated metadata");
    let abi = derive_maturity_announcement_abi(&target, &view)
        .expect("the validated view derives its ABI");
    let request = MaturityAnnouncementRequest::new(
        announced_cycle(&bundle),
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
    )
    .expect("sponsorless without change is admitted");
    let construction =
        construct_maturity_announcement(&target, &abi, &view, &request, &OracleStateCurve)
            .expect("the linked deployment constructs the announcement");
    let finalized = finalize_maturity_announcement(construction);
    let successor = finalized.construction().successor_constructor().clone();
    let started =
        OperatorSigningStarted::open(&finalized, &target, &OracleLiveCurve::new(target.clone()))
            .expect("the candidate freezes its operator request");
    let binding = started.request().binding();
    let signature = OPERATOR_HANDLE
        .material()
        .expect("the published handle resolves its material")
        .sign(started.request().message().with_vector_grown(), &[0; 32])
        .expect("the published scalar signs the frozen message")
        .to_vec();
    let response = OperatorSigningResponse::new(
        started.request().input_index(),
        signature,
        OPERATOR_SIGHASH_TYPE_BYTE,
        started.protected_bytes().to_vec(),
        binding.key().clone(),
        binding.deployment().clone(),
        binding.capability_revision(),
    );
    let mut registry = OperatorRightRegistry::default();
    let right = registry
        .issue(
            started.construction_right_scope(),
            started.protected_bytes(),
        )
        .expect("a fresh registry issues the candidate's scope");
    let ready = started
        .authorize(&mut registry, right, [response], &OperatorVerifier)
        .expect("the published signature authorizes the candidate")
        .bind_for_submission(&target)
        .expect("the authorized candidate binds its witness");
    Source {
        origin: MaturityByteSource::NodeFreeSubmitReady,
        bytes: ready.bytes().to_vec(),
        funded,
        identity: bundle.deployment().identity().clone(),
        branch,
        bundle,
        successor,
    }
}

/// Decode lowercase or uppercase pairs from a target-printed hex field.
fn decode_hex(text: &str) -> Option<Vec<u8>> {
    let (pairs, remainder) = text.as_bytes().as_chunks::<2>();
    if !remainder.is_empty() {
        return None;
    }
    pairs
        .iter()
        .map(|pair| {
            let digits = std::str::from_utf8(pair).ok()?;
            u8::from_str_radix(digits, 16).ok()
        })
        .collect()
}

/// Convert a printed wire outpoint to the transaction's internal byte order.
fn outpoint_of(wire: &WireOutpoint) -> Option<Outpoint> {
    let raw = decode_hex(&wire.txid)?;
    let mut internal = <[u8; 32]>::try_from(raw.as_slice()).ok()?;
    internal.reverse();
    Outpoint::new(Txid::from_internal(internal), wire.vout).ok()
}

/// Convert a printed asset identifier to internal byte order.
fn asset_of(text: &str) -> Option<AssetId> {
    let mut raw = decode_hex(text)?;
    raw.reverse();
    <[u8; 32]>::try_from(raw.as_slice())
        .ok()
        .map(AssetId::from_internal)
}

/// Convert the archive's sole funding output into a projection premise.
///
/// # Panics
///
/// Panics only if the admitted archive's printed funding fields cease to decode.
fn funded_of(output: &FundedOutput) -> MaturityFundedPredecessor {
    MaturityFundedPredecessor {
        outpoint: outpoint_of(&output.outpoint).expect("the funded outpoint decodes"),
        asset: asset_of(&output.asset).expect("the funded asset decodes"),
        amount: output.amount_satoshis,
        program: decode_hex(&output.script).expect("the funded program decodes"),
    }
}

/// Replay an admitted corpus through a fresh public planner.
///
/// # Panics
///
/// Panics only if an admitted exchange differs from the planner's own next
/// step, or if its one funding output or submission subject is absent.
fn archived(corpus: &ValidatedMaturityCorpus, selection: MaturityWitnessSelection) -> Source {
    let identity = corpus.evidence().identity().clone();
    let branch = corpus.evidence().branch();
    let mut planner =
        MaturityAnnouncementPlanner::new(identity.clone(), branch, selection).expect("planner");
    let mut next = planner.next_step(None).expect("issuance step");
    for (step, response) in &corpus.exchanges()[..2] {
        assert_eq!(next.as_ref(), Some(step));
        next = planner
            .next_step(Some((step.case(), response)))
            .expect("the archive's settled exchange replays");
    }
    assert_eq!(next.as_ref(), Some(&corpus.exchanges()[2].0));
    let OperationSubject::Submission(submission) = corpus.exchanges()[2].0.subject() else {
        panic!("the third exchange is a submission")
    };
    assert_eq!(
        planner.submission_bytes(),
        Some(submission.transaction_bytes.as_slice())
    );
    let [output] = corpus.exchanges()[1].1.funded_outputs.as_slice() else {
        panic!("the funding exchange has exactly one output")
    };
    Source {
        origin: MaturityByteSource::ArchivedSubmission {
            run_address: corpus.report().run_address().to_owned(),
        },
        bytes: submission.transaction_bytes.clone(),
        funded: funded_of(output),
        bundle: planner.bundle().clone(),
        identity,
        branch,
        successor: planner
            .announcement()
            .expect("the replayed planner has an announcement")
            .construction()
            .successor_constructor()
            .clone(),
    }
}

/// Reconstruct the historical whole-metadata archive through public loading.
///
/// # Panics
///
/// Panics only if the admitted archive ceases to validate or replay.
fn archived_whole() -> Source {
    archived(
        maturity_run_of_record().expect("the historical archive validates"),
        MaturityWitnessSelection::retained_whole_metadata(),
    )
}

/// Reconstruct the accepted variable-metadata archive through public loading.
///
/// # Panics
///
/// Panics only if the admitted archive ceases to validate or replay.
fn archived_variable() -> Source {
    let corpus = maturity_variable_run_of_record().expect("the variable archive validates");
    assert_eq!(corpus.report().run_address(), MATURITY_VARIABLE_RUN_ADDRESS);
    archived(
        corpus,
        MaturityWitnessSelection::Retained(StateWitnessSchedule::VariableMetadata),
    )
}

/// Reconstruct every source independently in candidate, whole, variable order.
///
/// # Panics
///
/// Panics only if one of the three fixed public source builders refuses.
fn sources() -> [Source; 3] {
    [node_free(), archived_whole(), archived_variable()]
}

/// Derive the report binding from the public evidence plan.
///
/// # Panics
///
/// Panics only if the fixed absent-material evidence plan ceases to derive.
fn binding() -> MaturityTargetBinding {
    derive_maturity_evidence_plan_with(
        MaturityExecutorProvenanceExpectation::NotStatedByTheOperator,
        MaturityConstructorMaterial::Absent(
            MaturityConstructorMaterialAbsence::NotSuppliedToDerivation,
        ),
    )
    .expect("the evidence plan derives without constructor material")
    .binding()
    .clone()
}

/// State the operator's absent executor-provenance expectation.
const fn provenance() -> MaturityExecutorProvenanceExpectation {
    MaturityExecutorProvenanceExpectation::NotStatedByTheOperator
}

/// Project the three owned sources through the public projector.
///
/// # Panics
///
/// Panics only if one of the public builders produces bytes the projector refuses.
fn projections(sources: &[Source; 3]) -> Vec<ValidatedMaturityContinuity> {
    sources
        .iter()
        .map(|source| source.project().expect("the public source projects"))
        .collect()
}

/// Assemble, independently validate, and render a three-source report.
///
/// # Panics
///
/// Panics only if the assembled public report differs from its source projections.
fn checked_report(records: &[ValidatedMaturityContinuity]) -> (MaturityContinuityReport, String) {
    let references = [&records[0], &records[1], &records[2]];
    let binding = binding();
    let provenance = provenance();
    let report =
        assemble_maturity_continuity_report(&references, binding.clone(), provenance.clone());
    let validated =
        validate_maturity_continuity_report(&report, &references, &binding, &provenance)
            .expect("the assembled report recomputes from the sources");
    (report, render_maturity_continuity_report(&validated))
}

// --- Positive projections and independent reconstruction ----------------

/// Check the complete constructor and semantic fact set for one public source.
///
/// # Panics
///
/// Panics only if the public source fails projection or constructor recomputation.
fn assert_positive(source: &Source, schedule: StateWitnessSchedule, width: usize) {
    let record = source.project().expect("the public source projects");
    assert_eq!(record.source(), &source.origin);
    assert_eq!(record.submitted_bytes(), source.bytes);
    assert_eq!(record.schedule(), schedule);
    assert_eq!(record.transaction().witnesses()[0].stack()[4].len(), width);
    assert_eq!(
        record.predecessor().encoded_metadata(),
        source.bundle.instances()[0].metadata()
    );
    assert_eq!(
        record.successor().encoded_metadata(),
        source.successor.encoded_metadata()
    );
    assert_eq!(record.funded(), &source.funded);
    assert_eq!(record.byte_identity(), &sha256(&source.bytes));
    assert!(record.static_comparison().static_root().agrees());
    assert!(record.static_comparison().control_block().agrees());
    assert_eq!(record.static_comparison().descriptor_result(), &Ok(()));
    assert_eq!(record.static_comparison().constructor_result(), &Ok(()));
    assert!(record.semantic_comparison().transition_agrees());
    assert_eq!(
        record.semantic_comparison().reconstruction_agrees(),
        Some(true)
    );
    assert_eq!(
        record.predecessor_leastness().host_least_constructor(),
        source.bundle.instances()[0].constructor()
    );
    assert_eq!(
        record.successor_leastness().host_least_constructor(),
        &source.successor
    );
    for leastness in [record.predecessor_leastness(), record.successor_leastness()] {
        assert!(leastness.is_host_least());
        assert_eq!(leastness.reconstructed_facts_match(), Some(true));
    }
    let target = target();
    for projection in [record.predecessor(), record.successor()] {
        assert_eq!(
            projection.output_program(),
            state_output_program_at_nonce(
                &target,
                projection.encoded_metadata(),
                source.bundle.static_subtree(),
                source.bundle.policy().internal_key(),
                &OracleStateCurve,
            )
            .expect("the fixed nonce reconstructs its output program")
        );
    }
    assert_eq!(
        record.expected_successor(),
        &announce_maturity(
            &record.predecessor().encoded_metadata().semantic,
            record.requested_cycle(),
            record.bounds(),
        )
        .expect("the witnessed announcement lies in the lead window")
    );
}

#[test]
fn submit_ready_candidate_projects_both_constructors_and_transition() {
    assert_positive(&node_free(), StateWitnessSchedule::WholeMetadata, 86);
}

#[test]
fn historical_archive_projects_both_constructors_and_transition() {
    assert_positive(&archived_whole(), StateWitnessSchedule::WholeMetadata, 86);
}

#[test]
fn accepted_variable_archive_projects_both_constructors_and_transition() {
    assert_positive(
        &archived_variable(),
        StateWitnessSchedule::VariableMetadata,
        53,
    );
}

#[test]
fn each_request_uses_the_predecessor_and_lead_window_rule() {
    for source in sources() {
        let record = source.project().expect("the source projects");
        let predecessor = source.bundle.instances()[0].metadata().semantic.cycle;
        let bounds = source.bundle.deployment().lead_bounds().bounds();
        let window = bounds.window(predecessor).expect("the lead window exists");
        assert_eq!(predecessor, Cycle::new(5));
        assert_eq!(
            (bounds.minimum(), bounds.maximum()),
            (Cycle::new(4), Cycle::new(6))
        );
        assert_eq!(window, (Cycle::new(9), Cycle::new(11)));
        assert_eq!(announced_cycle(&source.bundle), Cycle::new(10));
        assert_eq!(record.bounds(), bounds);
        assert_eq!(record.requested_cycle(), announced_cycle(&source.bundle));
    }
}

#[test]
fn funding_branch_schedule_and_origin_distinguish_the_sources() {
    let [candidate, whole, variable] = sources();
    assert_ne!(candidate.funded.outpoint, whole.funded.outpoint);
    assert_ne!(candidate.branch, whole.branch);
    assert_ne!(whole.origin, variable.origin);
    assert_ne!(candidate.origin, whole.origin);
    assert_ne!(candidate.origin, variable.origin);
    assert_ne!(
        whole.bundle.record().schedule(),
        variable.bundle.record().schedule()
    );
    assert!(matches!(
        candidate.origin,
        MaturityByteSource::NodeFreeSubmitReady
    ));
    assert!(matches!(
        whole.origin,
        MaturityByteSource::ArchivedSubmission { .. }
    ));
    assert!(matches!(
        variable.origin,
        MaturityByteSource::ArchivedSubmission { .. }
    ));
}

#[test]
fn independent_public_reconstructions_repeat_exact_bytes_and_identity() {
    let first = sources();
    let second = sources();
    for (left, right) in first.iter().zip(&second) {
        assert_eq!(left.bytes, right.bytes);
        assert_eq!(
            left.project().expect("first projection").byte_identity(),
            &sha256(&right.bytes)
        );
        assert_eq!(
            left.project().expect("first projection").byte_identity(),
            right.project().expect("second projection").byte_identity()
        );
    }
}

// --- Canonical report and outstanding acceptance ------------------------

#[test]
fn independently_rebuilt_reports_render_identical_source_bindings() {
    let first = projections(&sources());
    let second = projections(&sources());
    let (report, rendered) = checked_report(&first);
    let (_, rebuilt) = checked_report(&second);
    assert_eq!(rendered, rebuilt);
    assert_eq!(report.entries().len(), 3);
    let lines = rendered.lines().collect::<Vec<_>>();
    assert_eq!(
        lines
            .iter()
            .copied()
            .filter(|line| line.starts_with("schedule "))
            .collect::<Vec<_>>(),
        [
            "schedule whole-metadata",
            "schedule whole-metadata",
            "schedule variable-metadata"
        ]
    );
    assert_eq!(
        lines
            .iter()
            .copied()
            .filter(|line| line.starts_with("source_class "))
            .collect::<Vec<_>>(),
        [
            "source_class node-free-submit-ready",
            "source_class archived-submission",
            "source_class archived-submission"
        ]
    );
    let addresses = lines
        .iter()
        .copied()
        .filter(|line| line.starts_with("source_address "))
        .collect::<Vec<_>>();
    assert_eq!(addresses.len(), 3);
    assert_eq!(addresses[0], "source_address -");
    for (line, record) in addresses[1..].iter().zip(&first[1..]) {
        let MaturityByteSource::ArchivedSubmission { run_address } = record.source() else {
            panic!("archive origin")
        };
        let printed = line
            .strip_prefix("source_address ")
            .expect("address prefix");
        assert_eq!(decode_hex(printed).as_deref(), Some(run_address.as_bytes()));
    }
    let identities = lines
        .iter()
        .copied()
        .filter(|line| line.starts_with("byte_identity "))
        .collect::<Vec<_>>();
    assert_eq!(identities.len(), 3);
    for (line, record) in identities.iter().zip(&first) {
        let printed = line
            .strip_prefix("byte_identity ")
            .expect("identity prefix");
        assert_eq!(
            decode_hex(printed).as_deref(),
            Some(record.byte_identity().as_slice())
        );
    }
}

#[test]
fn accepted_archive_projection_and_report_keep_acceptance_outstanding() {
    let records = projections(&sources());
    for record in &records {
        assert!(matches!(
            record.acceptance_obligation(),
            MaturityAcceptanceObligation::Outstanding { .. }
        ));
    }
    let (report, rendered) = checked_report(&records);
    assert!(matches!(
        report.acceptance(),
        MaturityAcceptanceObligation::Outstanding { .. }
    ));
    for exact in [
        "acceptance outstanding",
        "acceptance_route RelayWitnessRestructure",
        "acceptance_route BlockLayerSubmissionSubject",
        "completeness partial-required-rows-outstanding",
    ] {
        assert!(rendered.lines().any(|line| line == exact));
    }
    assert!(!rendered.contains("Accepted"));
    assert!(!rendered.contains("Observed"));
}

// --- One-axis source refusals -------------------------------------------

#[test]
fn whole_bytes_under_variable_context_refuse_the_metadata_width() {
    let whole = archived_whole();
    let variable = archived_variable();
    let mut input = variable.input();
    input.submitted_bytes = &whole.bytes;
    assert!(matches!(
        project_maturity_continuity(input),
        Err(MaturityContinuityRefusal::WitnessWidth {
            index: 4,
            expected: 53,
            actual: 86
        })
    ));
}

#[test]
fn variable_bytes_under_whole_context_refuse_the_metadata_width() {
    let whole = archived_whole();
    let variable = archived_variable();
    let mut input = whole.input();
    input.submitted_bytes = &variable.bytes;
    assert!(matches!(
        project_maturity_continuity(input),
        Err(MaturityContinuityRefusal::WitnessWidth {
            index: 4,
            expected: 86,
            actual: 53
        })
    ));
}

#[test]
fn changed_funded_output_index_refuses_the_spent_outpoint() {
    let source = node_free();
    let mut funded = source.funded.clone();
    funded.outpoint = Outpoint::new(funded.outpoint.txid(), 1).expect("output one is admitted");
    let mut input = source.input();
    input.funded = &funded;
    assert!(matches!(
        project_maturity_continuity(input),
        Err(MaturityContinuityRefusal::SpentOutpoint { decoded, funded: stated })
            if decoded == source.funded.outpoint && stated == funded.outpoint
    ));
}

#[test]
fn successor_program_in_funding_refuses_retained_context() {
    let source = node_free();
    let mut funded = source.funded.clone();
    funded.program = source.successor.output_program();
    assert_ne!(funded.program, source.funded.program);
    let mut input = source.input();
    input.funded = &funded;
    assert!(matches!(
        project_maturity_continuity(input),
        Err(MaturityContinuityRefusal::RetainedContext { .. })
    ));
}

#[test]
fn other_deployment_identity_refuses_retained_context() {
    let source = node_free();
    let other = CandidateDeploymentIdentity::new([0x17; 32], [0x21; 32])
        .expect("the other network and genesis form an identity");
    assert_ne!(other, source.identity);
    let mut input = source.input();
    input.identity = &other;
    assert!(matches!(
        project_maturity_continuity(input),
        Err(MaturityContinuityRefusal::RetainedContext { .. })
    ));
}

#[test]
fn truncated_candidate_refuses_transaction_decode() {
    let source = node_free();
    let shortened = &source.bytes[..source.bytes.len() - 1];
    let mut input = source.input();
    input.submitted_bytes = shortened;
    assert!(matches!(
        project_maturity_continuity(input),
        Err(MaturityContinuityRefusal::Decode(_))
    ));
}

// --- One-axis report refusals -------------------------------------------

#[test]
fn reordered_sources_refuse_report_source_order() {
    let records = projections(&sources());
    let binding = binding();
    let provenance = provenance();
    let ordered = [&records[0], &records[1], &records[2]];
    let report = assemble_maturity_continuity_report(&ordered, binding.clone(), provenance.clone());
    let swapped = [&records[1], &records[0], &records[2]];
    assert_eq!(
        validate_maturity_continuity_report(&report, &swapped, &binding, &provenance),
        Err(MaturityContinuityReportRefusal::SourceOrderDiffers { position: 0 })
    );
}

#[test]
fn omitted_source_refuses_report_source_count() {
    let records = projections(&sources());
    let binding = binding();
    let provenance = provenance();
    let ordered = [&records[0], &records[1], &records[2]];
    let report = assemble_maturity_continuity_report(&ordered, binding.clone(), provenance.clone());
    assert_eq!(
        validate_maturity_continuity_report(&report, &ordered[..2], &binding, &provenance),
        Err(MaturityContinuityReportRefusal::SourceCountDiffers {
            stated: 3,
            recomputed: 2
        })
    );
}
