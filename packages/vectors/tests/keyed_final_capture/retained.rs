//! Test-local retention and replay. No maintained descriptor or production reader.
use super::{Recorded, driver};
use serde::{Deserialize, Serialize};
use target_elements_conformance::executor::OperationStep;
use target_elements_conformance::protocol::*;
use vectors::keyed_final_capture::*;
use vectors::keyed_final_native::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// One key's transport transcript and measured capture provenance.
pub struct SessionArchive {
    pub key: [u8; 32],
    pub provenance: CaptureProvenance,
    pub handshake: ExecutorHandshake,
    pub exchanges: Vec<(NativeOperationRequest, NativeOperationResponse)>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Test-local retained packet, admitted by the same reader for mock and native.
pub struct Archive {
    pub manifest: CaptureManifest,
    pub manifest_digest: [u8; 32],
    pub sessions: Vec<SessionArchive>,
    /// No per-clause native wall, CPU, stalls or validation-charge observation exists.
    pub native_clause_dimensions: Option<String>,
    pub costs: Option<super::cost::Costs>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
/// Admission conditions; owner errors retain their original identity.
pub enum Refusal {
    Encoding(String),
    SessionCensus,
    Provenance(ProvenanceRefusal),
    Journal(JournalRefusal),
    Manifest(CaptureRefusal),
    Replay,
    ReconstructedManifest,
    InventedDimension,
    CostPresence,
    CostMembers,
    CostSubjects,
    CostNativeDimensions,
    ObservedWeight,
    EnvelopeWeightMissing,
    EnvelopeWeightMismatch,
    EnvelopePolicyMissing,
    EnvelopePolicyExchangeMissing,
    EnvelopePolicyEncoding(String),
    EnvelopePolicyForm(Box<vectors::maturity_native::MaturityNativePlanRefusal>),
    HandshakeExecutable,
    HandshakeFramework,
    HandshakeIntended,
    HandshakeUpstream,
    HandshakeBinary,
    HandshakeTopics,
    HandshakeDomain,
    HandshakeLeafVersion,
    HandshakeCapability,
    RevisionSyntax(target_elements_conformance::provenance::ProvenanceSyntaxDefect),
}

/// Decode a retained packet and check its provenance, byte joins and replay.
/// # Errors
/// Returns the first framing or admission condition, retaining owner errors.
pub fn read(
    target: CaptureTarget,
    expected: &[CaptureProvenance],
    bytes: &[u8],
) -> Result<Archive, Refusal> {
    let archive = serde_json::from_slice(bytes)
        .map_err(|e: serde_json::Error| Refusal::Encoding(e.to_string()))?;
    admit(target, expected, &archive)?;
    Ok(archive)
}

#[allow(
    clippy::too_many_lines,
    reason = "The admission precedence is explicit, with owner refusals before replay and comparisons."
)]
/// Validate an already decoded packet against independently supplied provenance.
/// # Errors
/// Returns the first condition in the documented admission order.
pub fn admit(
    target: CaptureTarget,
    expected: &[CaptureProvenance],
    archive: &Archive,
) -> Result<(), Refusal> {
    admit_core(target, expected, archive, true)
}

// Test-only decomposition: segment children prove every replay operation; this
// function checks their joint envelope without rebuilding the prefix per child.
pub(super) fn admit_framing(
    target: CaptureTarget,
    expected: &[CaptureProvenance],
    archive: &Archive,
) -> Result<(), Refusal> {
    admit_core(target, expected, archive, false)
}
#[allow(
    clippy::too_many_lines,
    reason = "Owner precedence, byte checks and replay remain in one admission path."
)]
fn admit_core(
    target: CaptureTarget,
    expected: &[CaptureProvenance],
    archive: &Archive,
    replay_enabled: bool,
) -> Result<(), Refusal> {
    if archive.sessions.len() != 2
        || expected.len() != 2
        || archive.sessions.iter().map(|s| s.key).collect::<Vec<_>>()
            != [[42; 32], vectors::keyed_final_manifest::SECOND_KEY]
    {
        return Err(Refusal::SessionCensus);
    }
    if archive.native_clause_dimensions.is_some() {
        return Err(Refusal::InventedDimension);
    }
    let mut exchanges = vec![];
    for (session, expected) in archive.sessions.iter().zip(expected) {
        if session.provenance.key != Some(session.key) {
            return Err(Refusal::SessionCensus);
        }
        bind_capture_provenance(expected, &session.provenance).map_err(Refusal::Provenance)?;
        bind_capture_journal(&session.provenance, &session.handshake, &session.exchanges)
            .map_err(Refusal::Journal)?;
        handshake_provenance(&session.handshake, expected)?;
        exchanges.extend(session.exchanges.clone());
    }
    bind_capture_manifest(
        target,
        &expected[0],
        &archive.sessions[0].provenance,
        &archive.sessions[0].handshake,
        &archive.manifest,
        archive.manifest_digest,
        &exchanges,
    )
    .map_err(Refusal::Manifest)?;
    if matches!(
        target,
        CaptureTarget::Scaling(_) | CaptureTarget::Funding(_)
    ) {
        for (request, response) in &exchanges {
            if let OperationSubject::Submission(subject) = &request.subject {
                let weight = response
                    .resources
                    .transaction_weight
                    .ok_or(Refusal::EnvelopeWeightMissing)?;
                let tx: elements::Transaction =
                    elements::encode::deserialize(&subject.transaction_bytes)
                        .map_err(|_| Refusal::EnvelopeWeightMismatch)?;
                if weight != u64::try_from(tx.weight()).unwrap() {
                    return Err(Refusal::EnvelopeWeightMismatch);
                }
            }
        }
    }
    if matches!(target, CaptureTarget::Funding(_)) {
        for row in archive
            .manifest
            .rows
            .iter()
            .filter(|r| r.schema == CaptureSchema::FundingControl)
        {
            let response = policy_response(&exchanges, &row.submitted)?;
            let raw = response
                .observed_detail
                .as_deref()
                .ok_or(Refusal::EnvelopePolicyMissing)?;
            let detail: serde_json::Value = serde_json::from_str(raw)
                .map_err(|e| Refusal::EnvelopePolicyEncoding(e.to_string()))?;
            vectors::maturity_native::sponsored::Policy::parse(detail["policy"].clone())
                .map_err(|e| Refusal::EnvelopePolicyForm(Box::new(e)))?;
        }
    }
    if replay_enabled {
        let replay = std::panic::catch_unwind(|| {
            let mut rows = vec![];
            for session in &archive.sessions {
                let pending = session
                    .exchanges
                    .iter()
                    .map(|(request, response)| {
                        (
                            OperationStep::new(&request.case.step, request.subject.clone()),
                            response.clone(),
                        )
                    })
                    .collect();
                let io = Recorded {
                    pending,
                    journal: vec![],
                    rows: vec![],
                };
                let io = driver::campaign(io, session.key, target);
                assert!(
                    io.pending.is_empty(),
                    "extra exchange after final public reconstruction"
                );
                rows.extend(io.rows);
            }
            rows
        })
        .map_err(|_| Refusal::Replay)?;
        if replay != archive.manifest.rows {
            return Err(Refusal::ReconstructedManifest);
        }
    }
    if archive.costs.is_some() != (target == CaptureTarget::Cost) {
        return Err(Refusal::CostPresence);
    }
    if let Some(offered) = &archive.costs {
        for session in &archive.sessions {
            for (request, response) in &session.exchanges {
                if let (OperationSubject::Submission(subject), Some(weight)) =
                    (&request.subject, response.resources.transaction_weight)
                {
                    let tx: elements::Transaction =
                        elements::encode::deserialize(&subject.transaction_bytes).unwrap();
                    if u64::try_from(tx.weight()).unwrap() != weight {
                        return Err(Refusal::ObservedWeight);
                    }
                }
            }
        }
        let derived = super::cost::derive(&archive.sessions);
        if offered.members != derived.members {
            return Err(Refusal::CostMembers);
        }
        if offered.subjects != derived.subjects {
            return Err(Refusal::CostSubjects);
        }
        if offered.native_clause_wall_seconds.is_some()
            || offered.native_clause_cpu_seconds.is_some()
            || offered.native_clause_stalls_seconds.is_some()
            || offered.native_validation_charge.is_some()
        {
            return Err(Refusal::CostNativeDimensions);
        }
    }
    Ok(())
}

pub(super) fn policy_response<'a>(
    exchanges: &'a [(NativeOperationRequest, NativeOperationResponse)],
    submitted: &[u8],
) -> Result<&'a NativeOperationResponse, Refusal> {
    exchanges
        .iter()
        .find_map(|(request, response)| match &request.subject {
            OperationSubject::Submission(subject) if subject.transaction_bytes == submitted => {
                Some(response)
            }
            _ => None,
        })
        .ok_or(Refusal::EnvelopePolicyExchangeMissing)
}

fn handshake_provenance(
    handshake: &ExecutorHandshake,
    expected: &CaptureProvenance,
) -> Result<(), Refusal> {
    use target_elements_conformance::provenance::{FullRevisionId, RevisionId};
    if Some(&handshake.node_version) != expected.executable.as_ref() {
        return Err(Refusal::HandshakeExecutable);
    }
    if handshake.framework_revision != expected.framework {
        return Err(Refusal::HandshakeFramework);
    }
    let full = FullRevisionId::new(expected.framework.as_deref().unwrap())
        .map_err(Refusal::RevisionSyntax)?;
    if handshake.intended_executed_tip.as_deref() != Some(full.as_str()) {
        return Err(Refusal::HandshakeIntended);
    }
    if handshake.upstream_base.as_deref() != Some(full.as_str()) {
        return Err(Refusal::HandshakeUpstream);
    }
    let binary = handshake
        .binary_reported_revision
        .as_deref()
        .ok_or(Refusal::HandshakeBinary)?;
    if !RevisionId::new(binary)
        .map_err(Refusal::RevisionSyntax)?
        .matches_full(&full)
    {
        return Err(Refusal::HandshakeBinary);
    }
    if !handshake.included_local_topics.is_empty() {
        return Err(Refusal::HandshakeTopics);
    }
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    let domain = WireExecutionDomain::of(target.definition().execution_domain()).unwrap();
    if !handshake.supported_domains.contains(&domain) {
        return Err(Refusal::HandshakeDomain);
    }
    if !handshake
        .supported_leaf_versions
        .contains(&target.definition().leaf_version().get())
    {
        return Err(Refusal::HandshakeLeafVersion);
    }
    if ![
        ExecutorCapability::TestSponsorAuthorization,
        ExecutorCapability::TargetTransactionSubmission,
    ]
    .iter()
    .all(|capability| handshake.capabilities.contains(capability))
    {
        return Err(Refusal::HandshakeCapability);
    }
    Ok(())
}

/// Produce a scripted, non-native control through the shared public driver.
/// # Panics
/// Panics if the public pipeline or the scripted control violates its test contract.
pub fn mock_archive(target: CaptureTarget) -> Archive {
    let mut rows = vec![];
    let mut sessions = vec![];
    for key in [[42; 32], vectors::keyed_final_manifest::SECOND_KEY] {
        let mock = driver::campaign(super::mock::Mock::for_target(target), key, target);
        rows.extend(mock.rows);
        let mut provenance = super::packet();
        provenance.key = Some(key);
        provenance.freeze = Some(capture_freeze_digest(key).unwrap());
        sessions.push(SessionArchive {
            key,
            provenance,
            handshake: super::handshake(),
            exchanges: mock.exchanges,
        });
    }
    let manifest = CaptureManifest {
        schema: 1,
        target,
        rows,
    };
    let costs = (target == CaptureTarget::Cost).then(|| super::cost::derive(&sessions));
    Archive {
        manifest_digest: manifest_digest(&manifest).unwrap(),
        manifest,
        sessions,
        native_clause_dimensions: None,
        costs,
    }
}

/// Accepted packets and coordinate mutants, bounded by the caller's child guard.
/// # Panics
/// Panics if any control differs from its declared result or retention fails.
pub fn controls(target: CaptureTarget, directory: &std::path::Path) {
    let started = std::time::Instant::now();
    let archive = mock_archive(target);
    let expected: Vec<_> = archive
        .sessions
        .iter()
        .map(|s| s.provenance.clone())
        .collect();
    assert_eq!(admit(target, &expected, &archive), Ok(()));
    read(target, &expected, &serde_json::to_vec(&archive).unwrap()).unwrap();
    assert!(matches!(
        read(target, &expected, b"{"),
        Err(Refusal::Encoding(_))
    ));
    owner_controls(target, &expected, &archive);
    let mut missing = archive.clone();
    missing.sessions.pop();
    assert_eq!(
        admit(target, &expected, &missing),
        Err(Refusal::SessionCensus)
    );
    let mut invented = archive.clone();
    invented.native_clause_dimensions = Some("0 native cycles".into());
    assert_eq!(
        admit(target, &expected, &invented),
        Err(Refusal::InventedDimension)
    );
    let mut swapped = archive.clone();
    // Source A and B have the same broad shape/outcome, but are not interchangeable.
    let a = swapped
        .manifest
        .rows
        .iter()
        .position(|r| r.schema == CaptureSchema::SourceA)
        .unwrap();
    let b = swapped
        .manifest
        .rows
        .iter()
        .position(|r| r.schema == CaptureSchema::SourceB)
        .unwrap();
    swapped.manifest.rows[a].schema = CaptureSchema::SourceB;
    swapped.manifest.rows[b].schema = CaptureSchema::SourceA;
    swapped.manifest_digest = manifest_digest(&swapped.manifest).unwrap();
    assert_eq!(
        admit(target, &expected, &swapped),
        Err(Refusal::ReconstructedManifest)
    );
    let mut extra = archive.clone();
    let pair = extra.sessions[0].exchanges[0].clone();
    extra.sessions[0].exchanges.push(pair);
    assert_eq!(admit(target, &expected, &extra), Err(Refusal::Replay));
    if target == CaptureTarget::Cost {
        cost_controls(&expected, &archive);
    }
    std::fs::write(
        directory.join("archive.json"),
        serde_json::to_vec(&archive).unwrap(),
    )
    .unwrap();
    let wall = started.elapsed().as_secs_f64();
    println!(
        "capture {target:?} mock-admission wall={wall:.6}s split_or_report={}",
        wall > 10.0
    );
}

/// Replay one key's complete transport, retaining its observed handshake.
/// # Panics
/// Panics if the shared packet, transport assertions or retention fail.
pub fn transport_controls(target: CaptureTarget, directory: &std::path::Path, index: usize) {
    let archive: Archive =
        serde_json::from_slice(&std::fs::read(directory.join("archive.json")).unwrap()).unwrap();
    let handshake = super::native::offline_transport(target, &archive.sessions[index]);
    std::fs::write(
        directory.join(format!("handshake-{index}.json")),
        serde_json::to_vec(&handshake).unwrap(),
    )
    .unwrap();
}

/// Preserve the original joint admission assertion after both transport children.
/// # Panics
/// Panics if the retained handshakes or full two-key admission fail.
pub fn transported_controls(target: CaptureTarget, directory: &std::path::Path) {
    let mut archive: Archive =
        serde_json::from_slice(&std::fs::read(directory.join("archive.json")).unwrap()).unwrap();
    let expected: Vec<_> = archive
        .sessions
        .iter()
        .map(|session| session.provenance.clone())
        .collect();
    for (index, session) in archive.sessions.iter_mut().enumerate() {
        session.handshake = serde_json::from_slice(
            &std::fs::read(directory.join(format!("handshake-{index}.json"))).unwrap(),
        )
        .unwrap();
    }
    assert_eq!(admit(target, &expected, &archive), Ok(()));
}

fn owner_controls(target: CaptureTarget, expected: &[CaptureProvenance], archive: &Archive) {
    type Change = fn(&mut ExecutorHandshake);
    let mut subject = archive.clone();
    subject.sessions[0].provenance.node = None;
    assert_eq!(
        admit(target, expected, &subject),
        Err(Refusal::Provenance(ProvenanceRefusal::Missing(
            ProvenanceField::Node
        )))
    );
    let mut subject = archive.clone();
    subject.sessions[0].handshake.protocol_schema = 8;
    assert_eq!(
        admit(target, expected, &subject),
        Err(Refusal::Journal(JournalRefusal::HandshakeRevision))
    );
    let mut subject = archive.clone();
    subject.manifest_digest[0] ^= 1;
    assert_eq!(
        admit(target, expected, &subject),
        Err(Refusal::Manifest(CaptureRefusal::ManifestDigest))
    );
    let cases: [(Refusal, Change); 9] = [
        (Refusal::HandshakeDomain, |h| h.supported_domains.clear()),
        (Refusal::HandshakeLeafVersion, |h| {
            h.supported_leaf_versions.clear();
        }),
        (Refusal::HandshakeCapability, |h| h.capabilities.clear()),
        (Refusal::HandshakeExecutable, |h| h.node_version.push('x')),
        (Refusal::HandshakeFramework, |h| h.framework_revision = None),
        (Refusal::HandshakeIntended, |h| {
            h.intended_executed_tip = None;
        }),
        (Refusal::HandshakeUpstream, |h| h.upstream_base = None),
        (Refusal::HandshakeBinary, |h| {
            h.binary_reported_revision = Some("2".repeat(12));
        }),
        (Refusal::HandshakeTopics, |h| {
            h.included_local_topics.insert("extra".into());
        }),
    ];
    for (refusal, change) in cases {
        let mut subject = archive.clone();
        change(&mut subject.sessions[0].handshake);
        assert_eq!(admit(target, expected, &subject), Err(refusal));
    }
    let mut subject = archive.clone();
    subject.sessions[0].handshake.binary_reported_revision = Some("!".into());
    assert!(matches!(
        admit(target, expected, &subject),
        Err(Refusal::RevisionSyntax(_))
    ));
}
fn cost_controls(expected: &[CaptureProvenance], archive: &Archive) {
    type Mutation = fn(&mut Archive);
    let cases: [(Refusal, Mutation); 7] = [
        (Refusal::CostPresence, |a| a.costs = None),
        (Refusal::CostMembers, |a| {
            a.costs.as_mut().unwrap().members[0].model_primitives += 1;
        }),
        (Refusal::CostMembers, |a| {
            a.costs.as_mut().unwrap().members[0].primitive_unit = "seconds".into();
        }),
        (Refusal::CostMembers, |a| {
            a.costs.as_mut().unwrap().members[0].subject[0] ^= 1;
        }),
        (Refusal::CostSubjects, |a| {
            a.costs.as_mut().unwrap().subjects[0].direct_complete_bytes += 1;
        }),
        (Refusal::CostNativeDimensions, |a| {
            a.costs.as_mut().unwrap().native_clause_cpu_seconds = Some(0);
        }),
        (Refusal::ObservedWeight, |a| {
            let (_, response) = a.sessions[0]
                .exchanges
                .iter_mut()
                .find(|(request, _)| matches!(request.subject, OperationSubject::Submission(_)))
                .unwrap();
            response.resources.transaction_weight = Some(1);
        }),
    ];
    for (refusal, mutate) in cases {
        let mut subject = archive.clone();
        mutate(&mut subject);
        assert_eq!(admit(CaptureTarget::Cost, expected, &subject), Err(refusal));
    }
}
