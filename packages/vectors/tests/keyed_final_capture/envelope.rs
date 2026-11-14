//! Disjoint SOURCE preparation and separately timed MERGE execution.
use super::*;
/// Native and replay preparation uses the same sealed entries and disjoint handles.
/// # Panics
/// Panics on a construction, signature or observation disagreement.
pub(in crate::keyed_final_capture) fn campaign<I: Exchange>(io: I, key: [u8; 32], count: u16) -> I {
    let mut s = start(io, count);
    let mut state = Prepared {
        ordinary: s.fund(),
        parents: vec![],
    };
    let started = std::time::Instant::now();
    for i in 0..count.max(1) {
        prepare_one(&mut s, key, i, &mut state);
    }
    println!(
        "scaling SOURCE preparation count={} wall={:.6}s",
        count.max(1),
        started.elapsed().as_secs_f64()
    );
    finish(&mut s, key, count, &state);
    s.io
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Prepared {
    ordinary: (Vec<u8>, u32),
    parents: Vec<(Vec<u8>, u32, bool)>,
}
const fn start<I: Exchange>(io: I, count: u16) -> Session<I> {
    Session {
        io,
        program: vec![],
        asset: AssetId::from_internal([0; 32]),
        serial: 0,
        finals: vec![],
        target: CaptureTarget::Scaling(count),
        foreign: None,
        policy: None,
    }
}
fn prepare_one<I: Exchange>(s: &mut Session<I>, key: [u8; 32], i: u16, state: &mut Prepared) {
    let tx = TargetTransaction::decode(&state.ordinary.0).unwrap();
    let ValueField::Explicit(value) = tx.outputs()[state.ordinary.1 as usize].value() else {
        panic!("explicit sponsor")
    };
    if value < 12_000 {
        state.ordinary = s.fund();
    }
    let bytes = s.event(
        key,
        &[(state.ordinary.0.clone(), state.ordinary.1, false)],
        CaptureSchema::ScalingSource(i),
    );
    if i == 0 {
        let tx = TargetTransaction::decode(&bytes).unwrap();
        let bound = TargetTransaction::new(
            tx.version(),
            tx.inputs().to_vec(),
            tx.outputs().to_vec(),
            tx.lock_time(),
            vec![InputWitness::new(vec![vec![0; 72], vec![0; 33]])],
        )
        .unwrap();
        assert_eq!(bound.weight(), 1077);
        assert!(tx.weight() <= bound.weight());
    }
    state.ordinary = (bytes.clone(), 1);
    state.parents.push((bytes, 0, true));
    s.finals.clear(); // Member parents and signed bytes are retained in the journal, not twice in memory.
}
fn finish<I: Exchange>(s: &mut Session<I>, key: [u8; 32], count: u16, state: &Prepared) {
    if count > 0 {
        let (sponsor, index) = s.fund();
        let mut parents = state.parents.clone();
        parents.push((sponsor, index, false));
        let started = std::time::Instant::now();
        let bytes = s.event(key, &parents, CaptureSchema::ScalingMerge(count));
        let tx = TargetTransaction::decode(&bytes).unwrap();
        let mut witnesses = tx.witnesses().to_vec();
        for w in &witnesses[..usize::from(count)] {
            assert_eq!(w.stack().len(), 2);
            assert_eq!(w.stack()[0].len(), 52);
            assert_eq!(w.stack()[1].len(), 33);
        }
        witnesses[usize::from(count)] = InputWitness::new(vec![vec![0; 72], vec![0; 33]]);
        let bound = TargetTransaction::new(
            tx.version(),
            tx.inputs().to_vec(),
            tx.outputs().to_vec(),
            tx.lock_time(),
            witnesses,
        )
        .unwrap();
        assert_eq!(
            tapscript::keyed_resources::keyed_witness_bytes(1).unwrap(),
            88
        );
        if [252, 253].contains(&count) {
            // Size-only synthetic projection, never a submitted or admitted transaction.
            let shorter = TargetTransaction::new(
                bound.version(),
                bound.inputs()[1..].to_vec(),
                bound.outputs().to_vec(),
                bound.lock_time(),
                bound.witnesses()[1..].to_vec(),
            )
            .unwrap();
            let stripped =
                bound.encode_without_witness().len() - shorter.encode_without_witness().len();
            let complete = bound.encode().len() - shorter.encode().len();
            assert_eq!(complete - stripped, 91);
            assert_eq!(stripped, if count == 252 { 43 } else { 41 });
            assert_eq!(
                bound.weight() - shorter.weight(),
                if count == 252 { 263 } else { 255 }
            );
        }
        assert_eq!(
            bound.weight(),
            (if count >= 252 { 1085 } else { 1077 }) + 255 * u64::from(count)
        );
        assert!(tx.weight() <= bound.weight());
        println!(
            "scaling MERGE members={count} weight={} bound={} wall={:.6}s",
            tx.weight(),
            bound.weight(),
            started.elapsed().as_secs_f64()
        );
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Checkpoint {
    state: Prepared,
    serial: u32,
    program: Vec<u8>,
    asset: [u8; 32],
    rows: Vec<CaptureRow>,
    exchanges: Vec<(NativeOperationRequest, NativeOperationResponse)>,
}
impl Checkpoint {
    fn session<I: Exchange>(&self, io: I, count: u16) -> Session<I> {
        let mut session = start(io, count);
        session.serial = self.serial;
        session.program.clone_from(&self.program);
        session.asset = AssetId::from_internal(self.asset);
        session
    }
}
/// Each checkpoint child proves the same segment again using only recorded responses.
/// The unchanged prefix is retained, never rebuilt per case or per validator.
/// # Panics
/// Panics on any public replay, literal count, or retained-byte disagreement.
#[allow(
    clippy::too_many_lines,
    reason = "One child checks its construction and exact response replay together."
)]
pub(in crate::keyed_final_capture) fn staged(
    directory: &std::path::Path,
    key_index: usize,
    end: u16,
    merge: bool,
) {
    use super::super::{Recorded, mock::Mock, retained};
    let key = [[42; 32], vectors::keyed_final_manifest::SECOND_KEY][key_index];
    let path = directory.join(format!("state-{key_index}.json"));
    let previous: Option<Checkpoint> = path
        .exists()
        .then(|| serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap());
    let mut s = previous.as_ref().map_or_else(
        || start(Mock::new(), end),
        |p| {
            p.session(
                Mock::resume(&p.rows, &p.exchanges, &p.state.ordinary.0),
                end,
            )
        },
    );
    let offset = s.io.exchanges.len();
    let mut state = previous.as_ref().map_or_else(
        || Prepared {
            ordinary: s.fund(),
            parents: vec![],
        },
        |p| Prepared {
            ordinary: p.state.ordinary.clone(),
            parents: p.state.parents.clone(),
        },
    );
    let begin = u16::try_from(state.parents.len()).unwrap();
    if merge {
        assert_eq!(begin, end.max(1));
        finish(&mut s, key, end, &state);
    } else {
        assert!(end - begin <= 16);
        for i in begin..end {
            prepare_one(&mut s, key, i, &mut state);
        }
    }
    let replay = Recorded {
        pending: s.io.exchanges[offset..]
            .iter()
            .map(|(r, a)| {
                (
                    OperationStep::new(&r.case.step, r.subject.clone()),
                    a.clone(),
                )
            })
            .collect(),
        journal: vec![],
        rows: vec![],
    };
    let mut replay = start(replay, end);
    // The replay's starting state is exactly the previously checked public prefix.
    if let Some(p) = &previous {
        replay.serial = p.serial;
        replay.program.clone_from(&p.program);
        replay.asset = AssetId::from_internal(p.asset);
    }
    let mut replay_state = previous.as_ref().map_or_else(
        || Prepared {
            ordinary: replay.fund(),
            parents: vec![],
        },
        |p| Prepared {
            ordinary: p.state.ordinary.clone(),
            parents: p.state.parents.clone(),
        },
    );
    if merge {
        finish(&mut replay, key, end, &replay_state);
    } else {
        for i in begin..end {
            prepare_one(&mut replay, key, i, &mut replay_state);
        }
    }
    assert!(replay.io.pending.is_empty());
    assert_eq!(replay.io.journal, s.io.exchanges[offset..]);
    let row_offset = previous.as_ref().map_or(0, |p| p.rows.len());
    assert_eq!(replay.io.rows, s.io.rows[row_offset..]);
    assert_eq!(replay_state.parents, state.parents);
    let checkpoint = Checkpoint {
        state,
        serial: s.serial,
        program: s.program,
        asset: *s.asset.internal(),
        rows: s.io.rows,
        exchanges: s.io.exchanges,
    };
    if merge {
        let mut provenance = super::super::packet();
        provenance.key = Some(key);
        provenance.freeze = Some(capture_freeze_digest(key).unwrap());
        let session = retained::SessionArchive {
            key,
            provenance,
            handshake: super::super::handshake(),
            exchanges: checkpoint.exchanges,
        };
        std::fs::write(
            directory.join(format!("result-{key_index}-{end}.json")),
            serde_json::to_vec(&(checkpoint.rows, session)).unwrap(),
        )
        .unwrap();
    } else {
        std::fs::write(path, serde_json::to_vec(&checkpoint).unwrap()).unwrap();
    }
}

impl<I: Exchange> Session<I> {
    #[allow(
        clippy::too_many_lines,
        reason = "Explicit one-coordinate mutations stay together beside their sealed control."
    )]
    pub(super) fn funding_submission(&mut self, control: &FinalSubject, point: u8) -> Vec<u8> {
        let key = *control.frozen().key();
        let mut tx: elements::Transaction = elements::encode::deserialize(control.bytes()).unwrap();
        assert!([1, 2].contains(&tx.input.len()));
        let signer = tx.input.len() - 1;
        assert_eq!(tx.output.len(), 3);
        let elements::confidential::Value::Explicit(old_change) = tx.output[1].value else {
            panic!("explicit change")
        };
        if point == 0 {
            self.io.before_submission(CaptureRow {
                schema: CaptureSchema::FundingControl,
                key,
                freeze: capture_freeze_digest(key).unwrap(),
                digest: sha256(control.bytes()),
                submitted: control.bytes().to_vec(),
                boundary: CaptureBoundary::Acceptance,
                expected: ObservedOutcomeLayer::Accepted,
                sealed_control: true,
            });
            let response = self.io.exchange(control.submission_step());
            assert_eq!(response.observed_layer, ObservedOutcomeLayer::Accepted);
            let detail: serde_json::Value =
                serde_json::from_str(response.observed_detail.as_deref().unwrap()).unwrap();
            self.policy = Some(
                vectors::maturity_native::sponsored::Policy::parse(detail["policy"].clone())
                    .unwrap(),
            );
            assert_eq!(
                response.mined_readback.unwrap().raw_transaction,
                control.bytes()
            );
            return control.bytes().to_vec();
        }
        match point % 12 {
            1 | 6..=8 => {
                let value = match point % 12 {
                    1 => 0,
                    6 => 1,
                    7 => 100,
                    8 => 999,
                    _ => unreachable!(),
                };
                tx.output[0].value = elements::confidential::Value::Explicit(value);
                tx.output[1].value =
                    elements::confidential::Value::Explicit(old_change + 1000 - value);
                self.resign(control, &mut tx);
            }
            2 => {
                tx.output[1].value = elements::confidential::Value::Explicit(old_change + 1);
                self.resign(control, &mut tx);
            }
            3 => {
                tx.output.pop();
                tx.output[1].value = elements::confidential::Value::Explicit(old_change + 10_000);
                self.resign(control, &mut tx);
            }
            4 | 5 => {
                let mut witness: Vec<Vec<u8>> = tx.input[signer]
                    .witness
                    .script_witness
                    .iter()
                    .map(<[u8]>::to_vec)
                    .collect();
                if point % 12 == 4 {
                    witness[0][5] ^= 1;
                } else {
                    witness[0].resize(73, 0);
                }
                let refusal = transaction::keyed_sponsor::verify_keyed_sponsor(
                    control.frozen().frozen_bytes(),
                    signer,
                    control.frozen().inputs()[signer].parent().output(),
                    &witness,
                )
                .unwrap_err();
                if point % 12 == 5 {
                    assert_eq!(
                        refusal,
                        transaction::keyed_sponsor::KeyedSponsorRefusal::SignatureLength
                    );
                }
                println!("funding constructor-sponsor point={point} refusal={refusal:?}");
                tx.input[signer].witness.script_witness = witness.into();
            }
            9..=11 => {
                // The policy floor is read from the native session's accepted control response,
                // not from a historical fixture. Mock supplies the same typed shape.
                let policy = self.policy.as_ref().unwrap();
                let anchor = vectors::keyed_funding::reported_floor(
                    policy,
                    u64::try_from(tx.weight()).unwrap(),
                )
                .unwrap();
                let fee = match point % 12 {
                    9 => anchor.saturating_sub(1),
                    10 => anchor,
                    11 => anchor.checked_add(1).unwrap(),
                    _ => unreachable!(),
                };
                tx.output[2].value = elements::confidential::Value::Explicit(fee);
                tx.output[1].value =
                    elements::confidential::Value::Explicit(old_change + 10_000 - fee);
                self.resign(control, &mut tx);
                println!(
                    "funding fee point={point} control-weight={} reported-floor={anchor} fee={fee} final-weight={}",
                    TargetTransaction::decode(control.bytes()).unwrap().weight(),
                    tx.weight()
                );
            }
            _ => panic!("reviewed funding point"),
        }
        self.raw_submission(
            key,
            CaptureSchema::FundingPoint(point),
            elements::encode::serialize(&tx),
        )
    }
}

/// Join both keys after segment-by-segment exact replay, without repeating the prefix.
/// # Panics
/// Panics if framing, measurement binding, or any one-coordinate mutant disagrees.
pub(in crate::keyed_final_capture) fn joined(directory: &std::path::Path, count: u16) {
    use super::super::retained::{self, Archive, Refusal, SessionArchive};
    let mut rows = vec![];
    let mut sessions = vec![];
    for key in 0..2 {
        let path = directory.join(format!("scaling-{key}/result-{key}-{count}.json"));
        let (r, s): (Vec<CaptureRow>, SessionArchive) =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        rows.extend(r);
        sessions.push(s);
    }
    let target = CaptureTarget::Scaling(count);
    let manifest = CaptureManifest {
        schema: 1,
        target,
        rows,
    };
    let mut archive = Archive {
        manifest_digest: manifest_digest(&manifest).unwrap(),
        manifest,
        sessions,
        native_clause_dimensions: None,
        costs: None,
    };
    let expected: Vec<_> = archive
        .sessions
        .iter()
        .map(|s| s.provenance.clone())
        .collect();
    assert_eq!(retained::admit_framing(target, &expected, &archive), Ok(()));
    if count == 1565 {
        let index = archive
            .manifest
            .rows
            .iter()
            .position(|row| row.schema == CaptureSchema::ScalingMerge(1565))
            .unwrap();
        assert_eq!(
            archive.manifest.rows[index].boundary,
            CaptureBoundary::StructuralEnvelope
        );
        assert!(!archive.manifest.rows[index].sealed_control);
        archive.manifest.rows[index].boundary = CaptureBoundary::Acceptance;
        archive.manifest_digest = manifest_digest(&archive.manifest).unwrap();
        assert_eq!(
            retained::admit_framing(target, &expected, &archive),
            Err(Refusal::Manifest(CaptureRefusal::Boundary))
        );
        archive.manifest.rows[index].boundary = CaptureBoundary::StructuralEnvelope;
        archive.manifest_digest = manifest_digest(&archive.manifest).unwrap();
    }
    if count <= 2 {
        assert_eq!(retained::admit(target, &expected, &archive), Ok(()));
    }
    if count == 2 {
        for missing in [true, false] {
            let mut mutant = archive.clone();
            let response = &mut mutant.sessions[0]
                .exchanges
                .iter_mut()
                .find(|(r, _)| matches!(r.subject, OperationSubject::Submission(_)))
                .unwrap()
                .1;
            response.resources.transaction_weight = if missing { None } else { Some(1) };
            assert_eq!(
                retained::admit_framing(target, &expected, &mutant),
                Err(if missing {
                    Refusal::EnvelopeWeightMissing
                } else {
                    Refusal::EnvelopeWeightMismatch
                })
            );
        }
    }
    std::fs::write(
        directory.join(format!("scaling-{count}.json")),
        serde_json::to_vec(&archive).unwrap(),
    )
    .unwrap();
}

/// Full tiny funding replay and the same manifest/measurement admission as native.
/// # Panics
/// Panics on a missing coordinate or refused accepted packet.
pub(in crate::keyed_final_capture) fn funding_control(directory: &std::path::Path, point: u8) {
    use super::super::retained;
    let target = CaptureTarget::Funding(point);
    let archive = retained::mock_archive(target);
    let expected: Vec<_> = archive
        .sessions
        .iter()
        .map(|s| s.provenance.clone())
        .collect();
    assert_eq!(retained::admit(target, &expected, &archive), Ok(()));
    if point == 3 {
        let response = &archive.sessions[0].exchanges.last().unwrap().1;
        let detail: serde_json::Value =
            serde_json::from_str(response.observed_detail.as_deref().unwrap()).unwrap();
        assert_eq!(response.observed_layer, ObservedOutcomeLayer::Accepted);
        assert_eq!(detail["route"], "direct-block");
        assert_eq!(detail["relay_allowed"], false);
        assert!(detail["relay_detail"].is_string());
        let mut policy_refused = archive.clone();
        let response = &mut policy_refused.sessions[0].exchanges.last_mut().unwrap().1;
        response.observed_layer = ObservedOutcomeLayer::RelayPolicyRejection;
        response.accepted_txid = None;
        response.mined_readback = None;
        assert_eq!(
            retained::admit(target, &expected, &policy_refused),
            Err(retained::Refusal::Manifest(CaptureRefusal::ObservedOutcome))
        );
        policy_refused.sessions[0]
            .exchanges
            .last_mut()
            .unwrap()
            .1
            .observed_layer = ObservedOutcomeLayer::ScriptPathRejection;
        assert_eq!(
            retained::admit(target, &expected, &policy_refused),
            Err(retained::Refusal::Manifest(CaptureRefusal::ObservedOutcome))
        );
    }
    if point == 0 {
        let row = &archive.manifest.rows[0];
        assert!(retained::policy_response(&archive.sessions[0].exchanges, &row.submitted).is_ok());
        // Internal defensive lookup: public admission's earlier manifest join
        // already refuses this absence, independently of policy parsing.
        assert_eq!(
            retained::policy_response(&[], &row.submitted),
            Err(retained::Refusal::EnvelopePolicyExchangeMissing)
        );
        for detail in [None, Some("{".into()), Some("{\"policy\":null}".into())] {
            let mut changed = archive.clone();
            changed.sessions[0]
                .exchanges
                .last_mut()
                .unwrap()
                .1
                .observed_detail
                .clone_from(&detail);
            let refusal = retained::admit(target, &expected, &changed).unwrap_err();
            match detail.as_deref() {
                None => assert_eq!(refusal, retained::Refusal::EnvelopePolicyMissing),
                Some("{") => assert!(matches!(
                    refusal,
                    retained::Refusal::EnvelopePolicyEncoding(_)
                )),
                Some(_) => assert_eq!(
                    refusal,
                    retained::Refusal::EnvelopePolicyForm(Box::new(
                        vectors::maturity_native::MaturityNativePlanRefusal::SponsoredPolicy
                    ))
                ),
            }
        }

        for target in [CaptureTarget::Funding(24), CaptureTarget::Scaling(3)] {
            let mut invalid = archive.clone();
            invalid.manifest.target = target;
            invalid.manifest_digest = manifest_digest(&invalid.manifest).unwrap();
            assert_eq!(
                retained::admit(target, &expected, &invalid),
                Err(retained::Refusal::Manifest(
                    CaptureRefusal::UnsupportedEnvelopeCoordinate,
                ))
            );
        }
    }
    let bytes = serde_json::to_vec(&archive).unwrap();
    retained::read(target, &expected, &bytes).unwrap();
    std::fs::write(directory.join(format!("funding-{point}.json")), bytes).unwrap();
}

/// Exercise the actual transport on small representatives; wide bytes use segmented replay.
/// # Panics
/// Panics if transport or retained handshake binding disagrees.
pub(in crate::keyed_final_capture) fn transport(
    directory: &std::path::Path,
    name: &str,
    key: usize,
) {
    let (target, file) = match name {
        "source" => (CaptureTarget::Funding(0), "funding-0.json"),
        "merge" => (CaptureTarget::Funding(12), "funding-12.json"),
        "wideform" => (CaptureTarget::Scaling(2), "scaling-2.json"),
        _ => panic!("closed transport controls"),
    };
    let archive: super::super::retained::Archive =
        serde_json::from_slice(&std::fs::read(directory.join(file)).unwrap()).unwrap();
    let handshake = super::super::native::offline_transport(target, &archive.sessions[key]);
    vectors::keyed_final_native::bind_capture_journal(
        &archive.sessions[key].provenance,
        &handshake,
        &archive.sessions[key].exchanges,
    )
    .unwrap();
}
