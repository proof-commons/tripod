//! Offline pre-capture controls. Native observations are ignored until the
//! separately reviewed operator packet runs on its isolated node.

use architecture::keyed_fixed::{KeyedFixedManifest, KeyedFixedOperationId};
use compiler::{
    AnalysisPolicy, ProofSearchLimits, bind_input,
    keyed_fixed_plan::{PlacementSearchLimits, plan},
};
use elements::hashes::{sha256, sha256d};
use linker::{
    keyed_fixed_bundle::link_keyed_fixed_bundle,
    keyed_fixed_reference::keyed_fixed_parameter_offers,
};
use realization::keyed_fixed::{KeyedFixedParameters, KeyedFixedValue};
use tapscript::keyed_fixed_bundle::{KeyedFixedBundleSelection, emit_keyed_fixed_bundle};
use tapscript::keyed_fixed_constructor::KeyedFixedLeafRecipe;
use transaction::keyed_fixed_construct::{
    KeyedFixedConstructionRequest, KeyedFixedOutputOffer, construct_keyed_fixed,
};
use transaction::keyed_fixed_inputs::{KeyedFixedInputOffer, KeyedFixedInputRole};
use transaction::keyed_fixed_request::{keyed_fixed_merge_request, keyed_fixed_source_request};
use transaction::keyed_fixed_taproot::commit_keyed_fixed_bundle;
use transaction::{
    AssetField, AssetId, InputWitness, NonceField, Outpoint, TargetInput, TargetOutput,
    TargetTransaction, Txid, ValueField,
};
use vectors::keyed_fixed_capture::{CaptureBoundary, CaptureFamily, CaptureSubject, subjects};
use vectors::keyed_fixed_capture::{CaptureFreezeRefusal, bind_capture_freeze};
use vectors::keyed_fixed_capture::{
    CaptureProvenance, JournalBinding, JournalRefusal, JournalRow, PairControl, bind_journal,
    validate_journal_transport,
};
use vectors::keyed_fixed_capture::{
    WorkloadRefusal, WorkloadSamples, bind_workload_samples, workload_shape,
};
use vectors::keyed_fixed_constructor::{PublicKeyedCurve, construct as independent_commitment};
use vectors::keyed_fixed_corpus::{CorpusFreezeRefusal, bind_corpus_freeze};
use vectors::keyed_fixed_manifest::FreezeManifest;
use vectors::keyed_fixed_native::{NativeFreezeRefusal, ObservedFreeze, bind_native_freeze};
use vectors::keyed_fixed_stack_model::{self as model, Facts, Field, Input, Output, ProgramFact};

// Raw explicit asset payload from the retained first-edition funded parent;
// the native runner must compare this with its own observed policy asset.
const POLICY_ASSET: [u8; 32] = [
    0x23, 0x0f, 0x4f, 0x5d, 0x4b, 0x7c, 0x6f, 0xa8, 0x45, 0x80, 0x6e, 0xe4, 0xf6, 0x77, 0x13, 0x45,
    0x9e, 0x1b, 0x69, 0xe8, 0xe6, 0x0f, 0xce, 0xe2, 0xe4, 0x94, 0x0c, 0x7a, 0x0d, 0x5d, 0xe1, 0xb2,
];

fn parameters(key_byte: u8, value: u64) -> KeyedFixedParameters {
    KeyedFixedParameters {
        key: [key_byte; 32],
        asset: POLICY_ASSET,
        value: KeyedFixedValue::new(value).unwrap(),
    }
}

#[test]
fn fixed_capture_freeze_consumers() {
    let first = parameters(42, 60);
    let second = parameters(43, 1000);
    let first_freeze = FreezeManifest::offer(first)
        .unwrap()
        .validate(first)
        .unwrap();
    let second_freeze = FreezeManifest::offer(second)
        .unwrap()
        .validate(second)
        .unwrap();
    let expected = first_freeze.identity();
    let unequal = second_freeze.identity();
    let repository = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source_snapshot: Vec<_> = first_freeze
        .manifest()
        .sources
        .iter()
        .map(|source| {
            (
                source.path.clone(),
                std::fs::read(repository.join(&source.path)).unwrap(),
            )
        })
        .collect();
    let historical = first_freeze
        .manifest()
        .reconstruct_capture_history(first, &source_snapshot, expected)
        .unwrap();
    assert_eq!(historical.parameters(), first);
    assert_eq!(historical.identity(), expected);
    assert!(
        first_freeze
            .manifest()
            .reconstruct_capture_history(first, &source_snapshot, unequal)
            .is_err()
    );
    let mut substituted_snapshot = source_snapshot;
    substituted_snapshot[0].1[0] ^= 1;
    assert!(
        first_freeze
            .manifest()
            .reconstruct_capture_history(first, &substituted_snapshot, expected)
            .is_err()
    );
    assert_ne!(expected, unequal);
    let old_edition = vectors::keyed_final_manifest::FreezeManifest::offer(first.key)
        .capture_encoding()
        .unwrap();
    let old_edition = elements::hashes::sha256::Hash::hash(old_edition.as_bytes()).to_byte_array();
    assert_ne!(expected, old_edition);

    assert_eq!(
        bind_native_freeze(&first_freeze, Some(ObservedFreeze(expected))),
        Ok(ObservedFreeze(expected))
    );
    assert_eq!(
        bind_native_freeze(&first_freeze, None),
        Err(NativeFreezeRefusal::Missing)
    );
    for identity in [unequal, old_edition] {
        assert_eq!(
            bind_native_freeze(&first_freeze, Some(ObservedFreeze(identity))),
            Err(NativeFreezeRefusal::Unequal)
        );
    }

    assert_eq!(
        bind_capture_freeze(&first_freeze, Some(expected)),
        Ok(expected)
    );
    assert_eq!(
        bind_capture_freeze(&first_freeze, None),
        Err(CaptureFreezeRefusal::Missing)
    );
    for identity in [unequal, old_edition] {
        assert_eq!(
            bind_capture_freeze(&first_freeze, Some(identity)),
            Err(CaptureFreezeRefusal::Unequal)
        );
    }

    assert_eq!(
        bind_corpus_freeze(&historical, Some(expected)),
        Ok(expected)
    );
    assert_eq!(
        bind_corpus_freeze(&historical, None),
        Err(CorpusFreezeRefusal::Missing)
    );
    for identity in [unequal, old_edition] {
        assert_eq!(
            bind_corpus_freeze(&historical, Some(identity)),
            Err(CorpusFreezeRefusal::Unequal)
        );
    }
}

#[test]
fn fixed_capture_policy_envelope() {
    use CaptureSubject as S;
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    assert_eq!(
        transaction::keyed_fixed_construct::keyed_fixed_policy_member_maximum(&target),
        Ok(1299)
    );
    let count_prefix = |n: u64| if n + 1 < 253 { 1_u64 } else { 3 };
    let independent_weight = |n: u64| {
        let stripped = 239 + count_prefix(n) + 41 * n;
        4 * stripped + 117 + 143 * n
    };
    assert_eq!(count_prefix(251), 1);
    assert_eq!(count_prefix(252), 3);
    assert_eq!(independent_weight(1299), 399_878);
    assert_eq!(independent_weight(1300), 400_185);
    assert_eq!(independent_weight(0), 1_077);
    assert!(independent_weight(1299) <= 400_000);
    assert!(independent_weight(1300) > 400_000);
    let floor = |size: u64, rate: u64| size * rate / 1000 + u64::from(size * rate % 1000 != 0);
    assert_eq!(floor(145, 100), 15);
    assert_eq!(floor(177, 100), 18);
    assert!(14 < floor(145, 100));
    assert!(15 >= floor(145, 100));
    let scaling = subjects(CaptureFamily::Scaling);
    assert!(scaling.contains(&S::PublicSource(42)));
    assert!(scaling.contains(&S::WeightSource));
    assert_eq!(
        vectors::keyed_fixed_capture::expected_layer(S::KeyPathAttempt),
        Some(target_elements_conformance::protocol::ObservedOutcomeLayer::ScriptPathRejection)
    );
    for count in [1, 2, 251, 252, 253, 650, 1299, 1300] {
        assert!(scaling.contains(&S::WeightMerge(count)));
    }
    let policy = subjects(CaptureFamily::Policy);
    assert!(policy.contains(&S::Floor(14)));
    assert!(policy.contains(&S::Floor(15)));
    assert_eq!(
        S::WeightMerge(1300).boundary(),
        Some(CaptureBoundary::TypedConstruction)
    );
    assert_eq!(
        S::WeightSource.boundary(),
        Some(CaptureBoundary::Acceptance)
    );
    assert_eq!(
        S::StructuralWeight1300.boundary(),
        Some(CaptureBoundary::RelayPolicy)
    );
    assert_eq!(S::DirectBlock1300.boundary(), None);
    assert_eq!(S::PresentNonceFloor(17).boundary(), None);
    assert_eq!(S::Floor(14).boundary(), Some(CaptureBoundary::RelayPolicy));
    assert_eq!(S::ZeroFeeFloor.boundary(), Some(CaptureBoundary::FeePolicy));
    assert_eq!(S::Floor(13).boundary(), None);
    assert_eq!(S::WeightMerge(1301).boundary(), None);
}

#[test]
fn fixed_capture_workload_shapes() {
    for events in [4, 16, 64] {
        let linear = workload_shape(events, false);
        assert_eq!(linear.events(), events);
        assert_eq!(linear.edges(), events);
        assert_eq!(linear.parents[0], Vec::<usize>::new());
        for (index, parents) in linear.parents.iter().enumerate().skip(1) {
            assert_eq!(parents, &[index - 1]);
        }
        let branching = workload_shape(events, true);
        assert_eq!(branching.events(), events);
        assert_eq!(branching.edges(), events + 1);
        assert_eq!(branching.parents[0], Vec::<usize>::new());
        assert_eq!(branching.parents[1], Vec::<usize>::new());
        assert_eq!(branching.parents[2], [0, 1]);
        for (index, parents) in branching.parents.iter().enumerate().skip(3) {
            assert_eq!(parents, &[index - 1]);
        }
        let samples = WorkloadSamples {
            event_bytes: vec![240; events],
            parent_bytes: vec![90; 2],
            cold_read_micros: 11,
            warm_read_micros: [3, 4, 3, 4, 3],
        };
        assert_eq!(
            bind_workload_samples(&branching, &samples),
            Ok(events * 240 + 180)
        );
        let mut incomplete = samples;
        incomplete.event_bytes.pop();
        assert_eq!(
            bind_workload_samples(&branching, &incomplete),
            Err(WorkloadRefusal::IncompleteFundedBytes)
        );
    }
    assert_eq!(workload_shape(5, true).events(), 0);
}

#[test]
#[allow(clippy::too_many_lines)] // One exact journal row and adjacent substitutions must share the same validated freeze.
fn fixed_capture_journal_binding() {
    use target_elements_conformance::protocol::{
        MinedFundingReadback, NATIVE_PROTOCOL_SCHEMA, NativeOperationRequest,
        NativeOperationResponse, NativeResourceObservation, ObservedOutcomeLayer, OperationCaseId,
        OperationStepKind, OperationSubject, TargetSubmissionSubject,
    };
    let program = [vec![0, 20], vec![7; 20]].concat();
    let parent_bytes = parent(&program, POLICY_ASSET, 1_000_000, 3);
    let parent_tx = TargetTransaction::decode(&parent_bytes).unwrap();
    let point = Outpoint::new(
        Txid::from_internal(
            sha256d::Hash::hash(&parent_tx.encode_without_witness()).to_byte_array(),
        ),
        0,
    )
    .unwrap();
    let transaction = TargetTransaction::new(
        2,
        vec![TargetInput::new(point, 123)],
        vec![
            TargetOutput::new(
                AssetField::Explicit(AssetId::from_internal(POLICY_ASSET)),
                ValueField::Explicit(999_000),
                NonceField::Null,
                program,
            ),
            TargetOutput::new(
                AssetField::Explicit(AssetId::from_internal(POLICY_ASSET)),
                ValueField::Explicit(1_000),
                NonceField::Null,
                vec![],
            ),
        ],
        0,
        vec![InputWitness::new(vec![])],
    )
    .unwrap();
    let submitted = transaction.encode();
    let reference: elements::Transaction = elements::encode::deserialize(&submitted).unwrap();
    let txid = reference.txid().to_string();
    let case = OperationCaseId {
        operation: OperationStepKind::Submit,
        step: "control".into(),
    };
    let request = NativeOperationRequest {
        schema: NATIVE_PROTOCOL_SCHEMA,
        case: case.clone(),
        subject: OperationSubject::Submission(Box::new(TargetSubmissionSubject {
            transaction_bytes: submitted.clone(),
        })),
    };
    let response = NativeOperationResponse {
        schema: NATIVE_PROTOCOL_SCHEMA,
        case,
        observed_layer: ObservedOutcomeLayer::Accepted,
        observed_detail: Some("typed mock; no native observation".into()),
        issued_asset: None,
        funded_outputs: vec![],
        confidential_funded_outputs: vec![],
        mined_readback: Some(MinedFundingReadback {
            transaction_id: txid.clone(),
            witness_transaction_id: reference.wtxid().to_string(),
            block_hash: "control-block".into(),
            block_height: 1,
            raw_transaction: submitted.clone(),
        }),
        accepted_txid: Some(txid),
        sponsor_witness: vec![],
        script_path_witness: vec![],
        signer_public_key: None,
        signed_profile: None,
        signing_genesis: None,
        signature_bound_to: None,
        resources: NativeResourceObservation {
            transaction_weight: Some(u64::try_from(reference.weight()).unwrap()),
            ..NativeResourceObservation::default()
        },
    };
    let freeze = FreezeManifest::offer(parameters(42, 60))
        .unwrap()
        .validate(parameters(42, 60))
        .unwrap();
    let provenance = CaptureProvenance {
        freeze_identity: freeze.identity(),
        executable_digest: [2; 32],
        executor_digest: [3; 32],
        dependency_lock_digest: [4; 32],
        daemon_digest: [5; 32],
        framework_revision: "reviewed".into(),
        zk_library_digest: [6; 32],
        image_identity: "immutable-image".into(),
    };
    let control = JournalRow {
        request,
        response,
        submitted: submitted.clone(),
        submitted_digest: target_elements_conformance::constructor::tagged::sha256(&submitted),
        parent_bytes: vec![parent_bytes.clone()],
        parent_digests: vec![target_elements_conformance::constructor::tagged::sha256(
            &parent_bytes,
        )],
        readback: Some(submitted.clone()),
        expected_provenance: provenance.clone(),
        observed_provenance: provenance,
    };
    assert_eq!(validate_journal_transport(&freeze, &control), Ok(()));
    let mut changed = control.clone();
    changed.expected_provenance.freeze_identity[0] ^= 1;
    changed.observed_provenance.freeze_identity[0] ^= 1;
    assert_eq!(
        validate_journal_transport(&freeze, &changed),
        Err(JournalRefusal::Provenance)
    );
    let mut changed = control.clone();
    changed.observed_provenance.daemon_digest[0] ^= 1;
    assert_eq!(
        validate_journal_transport(&freeze, &changed),
        Err(JournalRefusal::Provenance)
    );
    let mut changed = control.clone();
    changed.request.case.step.push('x');
    assert_eq!(
        validate_journal_transport(&freeze, &changed),
        Err(JournalRefusal::Case)
    );
    let mut changed = control.clone();
    changed.submitted[0] ^= 1;
    assert_eq!(
        validate_journal_transport(&freeze, &changed),
        Err(JournalRefusal::Submission)
    );
    let mut changed = control.clone();
    changed.parent_bytes[0][0] ^= 1;
    assert_eq!(
        validate_journal_transport(&freeze, &changed),
        Err(JournalRefusal::Parent)
    );
    let mut changed = control.clone();
    changed.parent_bytes[0] = parent(
        &[vec![0, 20], vec![8; 20]].concat(),
        POLICY_ASSET,
        1_000_000,
        4,
    );
    changed.parent_digests[0] = hash(&changed.parent_bytes[0]);
    assert_eq!(
        validate_journal_transport(&freeze, &changed),
        Err(JournalRefusal::Parent)
    );
    let mut changed = control.clone();
    changed.readback = None;
    assert_eq!(
        validate_journal_transport(&freeze, &changed),
        Err(JournalRefusal::Readback)
    );
    let mut changed = control.clone();
    changed
        .response
        .mined_readback
        .as_mut()
        .unwrap()
        .raw_transaction[0] ^= 1;
    assert_eq!(
        validate_journal_transport(&freeze, &changed),
        Err(JournalRefusal::Readback)
    );
    let mut changed = control;
    changed.response.observed_detail = Some("OP_EQUALVERIFY".into());
    assert_eq!(validate_journal_transport(&freeze, &changed), Ok(()));
    // Generic node text remains retained data and supplies no instruction offset.

    let prepared = changed.expected_provenance;
    let positive = signed_mock_journal(
        60,
        60,
        31,
        false,
        ObservedOutcomeLayer::Accepted,
        prepared.clone(),
    );
    let positive_binding = JournalBinding {
        family: CaptureFamily::Pilot,
        subject: CaptureSubject::PublicSource(42),
        sponsor_index: 0,
        prepared_provenance: &prepared,
        control: None,
    };
    assert_eq!(bind_journal(&freeze, &positive, &positive_binding), Ok(()));
    let scaling_funding = JournalBinding {
        family: CaptureFamily::Scaling,
        ..positive_binding
    };
    assert_eq!(bind_journal(&freeze, &positive, &scaling_funding), Ok(()));
    let mut direct_acceptance = positive.clone();
    direct_acceptance.response.observed_detail = Some(
        serde_json::json!({"route":"direct-block","relay_allowed":false,"relay_detail":"fee"})
            .to_string(),
    );
    assert_eq!(
        bind_journal(&freeze, &direct_acceptance, &positive_binding),
        Err(JournalRefusal::Outcome)
    );
    let mut broken_sponsor = positive.clone();
    let decoded = TargetTransaction::decode(&broken_sponsor.submitted).unwrap();
    let mut witness = decoded.witnesses()[0].stack().to_vec();
    witness[0][5] ^= 1;
    let mutated = TargetTransaction::new(
        decoded.version(),
        decoded.inputs().to_vec(),
        decoded.outputs().to_vec(),
        decoded.lock_time(),
        vec![InputWitness::new(witness)],
    )
    .unwrap()
    .encode();
    let reference: elements::Transaction = elements::encode::deserialize(&mutated).unwrap();
    let OperationSubject::Submission(subject) = &mut broken_sponsor.request.subject else {
        panic!("submission")
    };
    subject.transaction_bytes = mutated.clone();
    broken_sponsor.submitted = mutated.clone();
    broken_sponsor.submitted_digest = hash(&mutated);
    broken_sponsor.readback = Some(mutated.clone());
    let readback = broken_sponsor.response.mined_readback.as_mut().unwrap();
    readback.raw_transaction = mutated;
    readback.witness_transaction_id = reference.wtxid().to_string();
    assert_eq!(
        bind_journal(&freeze, &broken_sponsor, &positive_binding),
        Err(JournalRefusal::Sponsor)
    );
    let negative = signed_mock_journal(
        59,
        60,
        32,
        true,
        ObservedOutcomeLayer::ScriptPathRejection,
        prepared.clone(),
    );
    let merge_control = signed_mock_journal(
        60,
        60,
        37,
        true,
        ObservedOutcomeLayer::Accepted,
        prepared.clone(),
    );
    let paired = JournalBinding {
        family: CaptureFamily::Clauses,
        subject: CaptureSubject::ValueMinus,
        sponsor_index: 1,
        prepared_provenance: &prepared,
        control: Some(PairControl {
            subject: CaptureSubject::OneMemberMerge(42),
            freeze: &freeze,
            row: &merge_control,
            sponsor_index: 1,
        }),
    };
    assert_eq!(bind_journal(&freeze, &negative, &paired), Ok(()));
    let mut control_from_other_preparation = merge_control.clone();
    control_from_other_preparation
        .expected_provenance
        .daemon_digest[0] ^= 1;
    control_from_other_preparation.observed_provenance =
        control_from_other_preparation.expected_provenance.clone();
    let other_preparation = JournalBinding {
        control: Some(PairControl {
            row: &control_from_other_preparation,
            ..paired.control.unwrap()
        }),
        ..paired
    };
    assert_eq!(
        bind_journal(&freeze, &negative, &other_preparation),
        Err(JournalRefusal::Control)
    );
    let mut direct_control = merge_control.clone();
    direct_control.response.observed_detail = Some(
        serde_json::json!({"route":"direct-block","relay_allowed":false,"relay_detail":"fee"})
            .to_string(),
    );
    let refused_direct_control = JournalBinding {
        control: Some(PairControl {
            row: &direct_control,
            ..paired.control.unwrap()
        }),
        ..paired
    };
    assert_eq!(
        bind_journal(&freeze, &negative, &refused_direct_control),
        Err(JournalRefusal::Control)
    );
    let accepted_negative = signed_mock_journal(
        59,
        60,
        34,
        true,
        ObservedOutcomeLayer::Accepted,
        prepared.clone(),
    );
    assert_eq!(
        bind_journal(&freeze, &accepted_negative, &paired),
        Err(JournalRefusal::Outcome)
    );
    let no_pair = JournalBinding {
        control: None,
        ..paired
    };
    assert_eq!(
        bind_journal(&freeze, &negative, &no_pair),
        Err(JournalRefusal::Control)
    );
    let legacy = JournalBinding {
        family: CaptureFamily::LegacyCopy,
        ..positive_binding
    };
    assert_eq!(
        bind_journal(&freeze, &positive, &legacy),
        Err(JournalRefusal::Census)
    );
    let mut drifted = prepared.clone();
    drifted.daemon_digest[0] ^= 1;
    let drift = JournalBinding {
        prepared_provenance: &drifted,
        ..positive_binding
    };
    assert_eq!(
        bind_journal(&freeze, &positive, &drift),
        Err(JournalRefusal::Provenance)
    );
    let other = parameters(43, 1000);
    let other_freeze = FreezeManifest::offer(other)
        .unwrap()
        .validate(other)
        .unwrap();
    let wrong_control = JournalBinding {
        control: Some(PairControl {
            freeze: &other_freeze,
            ..paired.control.unwrap()
        }),
        ..paired
    };
    assert_eq!(
        bind_journal(&freeze, &negative, &wrong_control),
        Err(JournalRefusal::Control)
    );
    let floor_14 = parameters(42, 14);
    let floor_15 = parameters(42, 15);
    let freeze_14 = FreezeManifest::offer(floor_14)
        .unwrap()
        .validate(floor_14)
        .unwrap();
    let freeze_15 = FreezeManifest::offer(floor_15)
        .unwrap()
        .validate(floor_15)
        .unwrap();
    let mut prepared_14 = prepared.clone();
    prepared_14.freeze_identity = freeze_14.identity();
    let mut prepared_15 = prepared.clone();
    prepared_15.freeze_identity = freeze_15.identity();
    let mut subject_14 = signed_mock_journal(
        14,
        14,
        35,
        false,
        ObservedOutcomeLayer::Accepted,
        prepared_14.clone(),
    );
    subject_14.response.observed_detail = Some(
        serde_json::json!({"route":"direct-block","relay_allowed":false,"relay_detail":"dust"})
            .to_string(),
    );
    let control_15 = signed_mock_journal(
        15,
        15,
        36,
        false,
        ObservedOutcomeLayer::Accepted,
        prepared_15,
    );
    let floor_binding = JournalBinding {
        family: CaptureFamily::Policy,
        subject: CaptureSubject::Floor(14),
        sponsor_index: 0,
        prepared_provenance: &prepared_14,
        control: Some(PairControl {
            subject: CaptureSubject::Floor(15),
            freeze: &freeze_15,
            row: &control_15,
            sponsor_index: 0,
        }),
    };
    assert_eq!(
        bind_journal(&freeze_14, &subject_14, &floor_binding),
        Ok(())
    );
    let wrong_floor_label = JournalBinding {
        subject: CaptureSubject::Floor(14),
        prepared_provenance: &control_15.expected_provenance,
        control: None,
        ..floor_binding
    };
    assert_eq!(
        bind_journal(&freeze_15, &control_15, &wrong_floor_label),
        Err(JournalRefusal::Triple)
    );
    let foreign_floor = JournalBinding {
        control: Some(PairControl {
            freeze: &other_freeze,
            ..floor_binding.control.unwrap()
        }),
        ..floor_binding
    };
    assert_eq!(
        bind_journal(&freeze_14, &subject_14, &foreign_floor),
        Err(JournalRefusal::Control)
    );
    let mut foreign_provenance = prepared.clone();
    foreign_provenance.freeze_identity = other_freeze.identity();
    let foreign_row = signed_mock_journal(
        60,
        60,
        33,
        false,
        ObservedOutcomeLayer::Accepted,
        foreign_provenance.clone(),
    );
    let foreign_binding = JournalBinding {
        prepared_provenance: &foreign_provenance,
        ..positive_binding
    };
    assert_eq!(
        bind_journal(&other_freeze, &foreign_row, &foreign_binding),
        Err(JournalRefusal::Triple)
    );
}

fn hash(bytes: &[u8]) -> [u8; 32] {
    sha256::Hash::hash(bytes).to_byte_array()
}

#[allow(clippy::too_many_lines)] // An independently signed control and its mock readback share the exact bytes.
fn signed_mock_journal(
    amount: u64,
    bound_value: u64,
    salt: u32,
    spend_member: bool,
    layer: target_elements_conformance::protocol::ObservedOutcomeLayer,
    provenance: CaptureProvenance,
) -> JournalRow {
    use elements::secp256k1_zkp::{Message, PublicKey, Secp256k1, SecretKey};
    use target_elements_conformance::protocol::{
        MinedFundingReadback, NATIVE_PROTOCOL_SCHEMA, NativeOperationRequest,
        NativeOperationResponse, NativeResourceObservation, ObservedOutcomeLayer, OperationCaseId,
        OperationStepKind, OperationSubject, TargetSubmissionSubject,
    };
    let secp = Secp256k1::new();
    let key = SecretKey::from_slice(&[1; 32]).unwrap();
    let public = PublicKey::from_secret_key(&secp, &key).serialize();
    let mut sponsor_program = vec![0, 20];
    sponsor_program.extend(elements::hashes::hash160::Hash::hash(&public).to_byte_array());
    let parent_bytes = parent(&sponsor_program, POLICY_ASSET, 1_000_000, salt);
    let parent_tx = TargetTransaction::decode(&parent_bytes).unwrap();
    let point = Outpoint::new(
        Txid::from_internal(
            sha256d::Hash::hash(&parent_tx.encode_without_witness()).to_byte_array(),
        ),
        0,
    )
    .unwrap();
    let q = parameters(42, bound_value);
    let program = independent_commitment(
        &target_elements::reviewed_elements_tapscript().unwrap(),
        q,
        &PublicKeyedCurve,
    )
    .unwrap()
    .output_program;
    let member_parent =
        spend_member.then(|| parent(&program, POLICY_ASSET, bound_value, salt + 100));
    let member_input = member_parent.as_ref().map(|bytes| {
        let parent_tx = TargetTransaction::decode(bytes).unwrap();
        TargetInput::new(
            Outpoint::new(
                Txid::from_internal(
                    sha256d::Hash::hash(&parent_tx.encode_without_witness()).to_byte_array(),
                ),
                0,
            )
            .unwrap(),
            0x8000_0002,
        )
    });
    let sponsor_index = usize::from(member_input.is_some());
    let mut inputs = member_input.into_iter().collect::<Vec<_>>();
    inputs.push(TargetInput::new(point, 0x8000_0001));
    let mut witnesses = if sponsor_index == 1 {
        let freeze = FreezeManifest::offer(q).unwrap().validate(q).unwrap();
        vec![InputWitness::new(vec![
            freeze.manifest().descriptor.leaf.clone(),
            freeze.manifest().descriptor.control.clone(),
        ])]
    } else {
        Vec::new()
    };
    witnesses.push(InputWitness::new(vec![]));
    let unsigned = TargetTransaction::new(
        2,
        inputs,
        vec![
            if matches!(amount, 17 | 18) {
                TargetOutput::new(
                    AssetField::Explicit(AssetId::from_internal(POLICY_ASSET)),
                    ValueField::Explicit(amount),
                    NonceField::Commitment(public),
                    program,
                )
            } else {
                output_with_program(POLICY_ASSET, amount, program)
            },
            output_with_program(
                POLICY_ASSET,
                1_000_000 + u64::from(sponsor_index == 1) * bound_value - amount - 1_000,
                sponsor_program,
            ),
            output_with_program(POLICY_ASSET, 1_000, vec![]),
        ],
        0,
        witnesses.clone(),
    )
    .unwrap();
    let digest = transaction::keyed_fixed_sponsor::keyed_fixed_sponsor_digest(
        &unsigned.encode_without_witness(),
        sponsor_index,
        &parent_tx.outputs()[0],
    )
    .unwrap();
    let mut signature = secp
        .sign_ecdsa(&Message::from_digest(digest), &key)
        .serialize_der()
        .to_vec();
    signature.push(1);
    witnesses[sponsor_index] = InputWitness::new(vec![signature, public.to_vec()]);
    let signed = TargetTransaction::new(
        unsigned.version(),
        unsigned.inputs().to_vec(),
        unsigned.outputs().to_vec(),
        unsigned.lock_time(),
        witnesses,
    )
    .unwrap();
    let submitted = signed.encode();
    let reference: elements::Transaction = elements::encode::deserialize(&submitted).unwrap();
    let txid = reference.txid().to_string();
    let case = OperationCaseId {
        operation: OperationStepKind::Submit,
        step: format!("signed-{salt}"),
    };
    let accepted = layer == ObservedOutcomeLayer::Accepted;
    let readback = accepted.then(|| MinedFundingReadback {
        transaction_id: txid.clone(),
        witness_transaction_id: reference.wtxid().to_string(),
        block_hash: "mock-block".into(),
        block_height: 1,
        raw_transaction: submitted.clone(),
    });
    JournalRow {
        request: NativeOperationRequest {
            schema: NATIVE_PROTOCOL_SCHEMA,
            case: case.clone(),
            subject: OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: submitted.clone(),
            })),
        },
        response: NativeOperationResponse {
            schema: NATIVE_PROTOCOL_SCHEMA,
            case,
            observed_layer: layer,
            observed_detail: Some(if accepted {
                serde_json::json!({"route":"relay-and-template","relay_allowed":true}).to_string()
            } else {
                serde_json::json!({"detail":{"txid":txid},"route":"relay-and-template","relay_allowed":true}).to_string()
            }),
            issued_asset: None,
            funded_outputs: vec![],
            confidential_funded_outputs: vec![],
            mined_readback: readback.clone(),
            accepted_txid: accepted.then_some(txid),
            sponsor_witness: vec![],
            script_path_witness: vec![],
            signer_public_key: None,
            signed_profile: None,
            signing_genesis: None,
            signature_bound_to: None,
            resources: NativeResourceObservation {
                transaction_weight: Some(u64::try_from(reference.weight()).unwrap()),
                ..NativeResourceObservation::default()
            },
        },
        submitted: submitted.clone(),
        submitted_digest: hash(&submitted),
        parent_bytes: member_parent
            .iter()
            .cloned()
            .chain([parent_bytes.clone()])
            .collect(),
        parent_digests: member_parent
            .iter()
            .map(|bytes| hash(bytes))
            .chain([hash(&parent_bytes)])
            .collect(),
        readback: readback.map(|value| value.raw_transaction),
        expected_provenance: provenance.clone(),
        observed_provenance: provenance,
    }
}

#[test]
#[allow(clippy::too_many_lines)] // The packet census and adjacent refusal controls share one mocked preparation.
fn fixed_capture_sponsor_packet() {
    use target_elements_conformance::protocol::ObservedOutcomeLayer;
    use vectors::keyed_fixed_capture::{CaptureRole, validate_s8c_census};

    let sponsor = subjects(CaptureFamily::Sponsor);
    let legacy = subjects(CaptureFamily::LegacyCopy);
    assert_eq!(sponsor.len(), 6);
    assert_eq!(legacy.len(), 3);
    let rows = sponsor
        .iter()
        .enumerate()
        .map(|(index, subject)| (*subject, subject.role(), 3 * (index + 1)))
        .collect::<Vec<_>>();
    assert_eq!(
        validate_s8c_census(CaptureFamily::Sponsor, &rows, 18),
        Ok(())
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row.1 == CaptureRole::Primary)
            .count(),
        3
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row.1 == CaptureRole::SupportingMember)
            .count(),
        2
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row.1 == CaptureRole::SupportingPayment)
            .count(),
        1
    );
    let legacy_rows = legacy
        .iter()
        .enumerate()
        .map(|(index, subject)| (*subject, subject.role(), 3 * (index + 1)))
        .collect::<Vec<_>>();
    assert_eq!(
        validate_s8c_census(CaptureFamily::LegacyCopy, &legacy_rows, 9),
        Ok(())
    );
    let mut wrong_role = rows.clone();
    wrong_role[4].1 = CaptureRole::Primary;
    assert_eq!(
        validate_s8c_census(CaptureFamily::Sponsor, &wrong_role, 18),
        Err(JournalRefusal::Census)
    );
    assert_eq!(
        validate_s8c_census(CaptureFamily::Sponsor, &rows, 17),
        Err(JournalRefusal::Census)
    );

    let freeze = FreezeManifest::offer(parameters(42, 60))
        .unwrap()
        .validate(parameters(42, 60))
        .unwrap();
    let provenance = CaptureProvenance {
        freeze_identity: freeze.identity(),
        executable_digest: [2; 32],
        executor_digest: [3; 32],
        dependency_lock_digest: [4; 32],
        daemon_digest: [5; 32],
        framework_revision: "mock".into(),
        zk_library_digest: [6; 32],
        image_identity: "mock-image".into(),
    };
    let refusal = signed_mock_journal(
        59,
        60,
        101,
        true,
        ObservedOutcomeLayer::ScriptPathRejection,
        provenance,
    );
    assert_eq!(validate_journal_transport(&freeze, &refusal), Ok(()));
    let mut foreign_txid = refusal;
    let mut detail: serde_json::Value =
        serde_json::from_str(foreign_txid.response.observed_detail.as_deref().unwrap()).unwrap();
    detail["detail"]["txid"] = serde_json::json!("00");
    foreign_txid.response.observed_detail = Some(detail.to_string());
    assert_eq!(
        validate_journal_transport(&freeze, &foreign_txid),
        Err(JournalRefusal::Response)
    );

    native::mocked_failed_response_retention();
}

const fn output_with_program(asset: [u8; 32], value: u64, program: Vec<u8>) -> TargetOutput {
    TargetOutput::new(
        AssetField::Explicit(AssetId::from_internal(asset)),
        ValueField::Explicit(value),
        NonceField::Null,
        program,
    )
}

fn independent_leaf(q: KeyedFixedParameters) -> Vec<u8> {
    let transcript = [
        q.asset.to_vec(),
        vec![1],
        q.value.get().to_le_bytes().to_vec(),
        vec![1],
    ]
    .concat();
    [
        vec![32],
        q.key.to_vec(),
        vec![0x75, 32],
        hash(&transcript).to_vec(),
        vec![
            0x76, 0, 0xce, 0x7e, 0xc4, 0, 0xcf, 0x7e, 0xc6, 0x88, 0xcd, 0xc8, 0x7e, 0xc4, 0xcd,
            0xc9, 0x7e, 0xc6, 0x88, 0xcd, 0xc7, 0x75, 4, 0, 0, 0, 0, 0x88, 0x75, 0xcd, 0xca, 0,
            0xd1, 0x7b, 0x88, 0x88, 0x51,
        ],
    ]
    .concat()
}

fn parent(program: &[u8], asset: [u8; 32], value: u64, salt: u32) -> Vec<u8> {
    TargetTransaction::new(
        2,
        vec![TargetInput::new(
            Outpoint::new(Txid::from_internal([1; 32]), salt).unwrap(),
            123,
        )],
        vec![TargetOutput::new(
            AssetField::Explicit(AssetId::from_internal(asset)),
            ValueField::Explicit(value),
            NonceField::Null,
            program.to_vec(),
        )],
        0,
        vec![InputWitness::new(vec![])],
    )
    .unwrap()
    .encode()
}

fn input_offer(
    parent: &[u8],
    index: u32,
    role: KeyedFixedInputRole,
) -> KeyedFixedInputOffer<'_, '_> {
    let decoded = TargetTransaction::decode(parent).unwrap();
    KeyedFixedInputOffer {
        parent,
        input_index: index,
        outpoint: Outpoint::new(
            Txid::from_internal(
                sha256d::Hash::hash(&decoded.encode_without_witness()).to_byte_array(),
            ),
            0,
        )
        .unwrap(),
        sequence: 123,
        flags: 0,
        role,
        capability: None,
    }
}

fn linked(
    q: KeyedFixedParameters,
    operation: KeyedFixedOperationId,
) -> linker::keyed_fixed_bundle::LinkedKeyedFixedBundle {
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    let manifest = KeyedFixedManifest::contract().unwrap();
    let declaration = realization::keyed_fixed::KeyedFixedRealizationDraft::derive(&manifest)
        .unwrap()
        .validate(&manifest)
        .unwrap();
    let limit = std::num::NonZeroU64::new(100_000).unwrap();
    let input = bind_input(
        &manifest,
        declaration,
        vec![operation],
        AnalysisPolicy::strict(ProofSearchLimits::new(limit, limit)),
    )
    .unwrap();
    let plan = plan(&input, PlacementSearchLimits::new(limit, limit)).unwrap();
    let selection = KeyedFixedBundleSelection {
        binding: plan.project().binding.clone(),
        scope: plan.project().scope.clone(),
        proof: 0,
        placements: plan.project().plans[0]
            .operations
            .keys()
            .map(|id| (*id, 0))
            .collect(),
    };
    let recipe = KeyedFixedLeafRecipe::new(q);
    let emission = emit_keyed_fixed_bundle(&target, &plan, selection, recipe).unwrap();
    link_keyed_fixed_bundle(
        &target,
        &emission,
        &keyed_fixed_parameter_offers(&target, recipe),
    )
    .unwrap()
}

#[test]
#[allow(clippy::too_many_lines)] // Two triples and both operations share one public reconstruction comparison.
fn fixed_capture_subject_bytes() {
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    for (key, value) in [(42, 60), (43, 1000)] {
        let q = parameters(key, value);
        let independent = independent_commitment(&target, q, &PublicKeyedCurve).unwrap();
        assert_eq!(independent.leaf, independent_leaf(q));
        assert_eq!(independent.output_program.len(), 34);
        assert_eq!(independent.control.len(), 33);
        for member_count in [0_u64, 1, 2] {
            let operation = if member_count == 0 {
                KeyedFixedOperationId::Source
            } else {
                KeyedFixedOperationId::Merge
            };
            let commitment = commit_keyed_fixed_bundle(&target, &linked(q, operation)).unwrap();
            assert_eq!(commitment.leaf(), independent.leaf);
            assert_eq!(commitment.output_program(), independent.output_program);
            assert_eq!(commitment.control().as_slice(), independent.control);
            let sponsor_program = [vec![0, 20], vec![7; 20]].concat();
            let sponsor = parent(&sponsor_program, q.asset, 1_000_000, 1);
            let mut offers = vec![input_offer(&sponsor, 0, KeyedFixedInputRole::Collateral)];
            let members: Vec<_> = (0..member_count)
                .map(|index| {
                    parent(
                        commitment.output_program(),
                        q.asset,
                        value,
                        2 + u32::try_from(index).unwrap(),
                    )
                })
                .collect();
            for (index, member) in members.iter().enumerate() {
                offers.push(input_offer(
                    member,
                    u32::try_from(index + 1).unwrap(),
                    KeyedFixedInputRole::Member,
                ));
            }
            let source;
            let merge;
            let request = if operation == KeyedFixedOperationId::Source {
                source = keyed_fixed_source_request(&target, q, &commitment, &offers).unwrap();
                KeyedFixedConstructionRequest::Source(&source)
            } else {
                merge = keyed_fixed_merge_request(&target, q, &commitment, &offers).unwrap();
                KeyedFixedConstructionRequest::Merge(&merge)
            };
            let tx = construct_keyed_fixed(
                &target,
                request,
                KeyedFixedOutputOffer {
                    policy_asset: AssetId::from_internal(q.asset),
                    successor_value: value,
                    additional_outputs: vec![TargetOutput::new(
                        AssetField::Explicit(AssetId::from_internal(q.asset)),
                        ValueField::Explicit(1_000_000 + member_count * value - value - 1_000),
                        NonceField::Null,
                        sponsor_program,
                    )],
                    fee: Some(1_000),
                },
            )
            .unwrap();
            let bytes = tx.transaction().encode();
            let target_decoded: elements::Transaction =
                elements::encode::deserialize(&bytes).unwrap();
            assert_eq!(elements::encode::serialize(&target_decoded), bytes);
            assert_eq!(
                tx.transaction().outputs()[0].program(),
                independent.output_program
            );
            assert_eq!(
                tx.transaction().outputs()[0].value(),
                ValueField::Explicit(value)
            );
            for witness in &tx.transaction().witnesses()[1..] {
                assert_eq!(
                    witness.stack(),
                    [independent.leaf.clone(), independent.control.clone()]
                );
            }
        }
    }
}

fn model_output(program: &[u8], asset: [u8; 32], value: u64) -> Output {
    Output {
        program: ProgramFact::Witness {
            version: 1,
            payload: program[2..].to_vec(),
        },
        asset: Field {
            payload: asset.to_vec(),
            prefix: 1,
        },
        value: Field {
            payload: value.to_le_bytes().to_vec(),
            prefix: 1,
        },
    }
}

#[test]
#[allow(clippy::too_many_lines)] // Each separately funded coordinate retains its progressing model control and exact site.
fn fixed_capture_structural_pairs() {
    let q = parameters(42, 60);
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    let descriptor = independent_commitment(&target, q, &PublicKeyedCurve).unwrap();
    let program = descriptor.output_program;
    let leaf = independent_leaf(q);
    let ordinary = model_output(&program, q.asset, q.value.get());
    let control = Facts {
        current: 1,
        inputs: vec![
            Input {
                txid: [7; 32],
                birth: 0,
                flags: 0,
                output: model_output(&program, q.asset, 1_000_000),
            },
            Input {
                txid: [8; 32],
                birth: 0,
                flags: 0,
                output: ordinary.clone(),
            },
        ],
        outputs: vec![ordinary],
    };
    assert!(model::evaluate(&leaf, &control).is_ok());
    let fee = 1_000;
    let other = [0x91; 32];
    for (asset, value, change, other_asset_funding, site) in [
        (other, 60, 1_000_060 - fee, 60, 12),
        (q.asset, 59, 1_000_001 - fee, 0, 12),
        (q.asset, 61, 999_999 - fee, 0, 12),
    ] {
        // Balance each asset separately; a cross-asset scalar sum would hide
        // the very earlier refusal this model pair is meant to exclude.
        assert_eq!(
            1_000_000 + 60,
            change + fee + if asset == q.asset { value } else { 0 }
        );
        assert_eq!(other_asset_funding, if asset == other { value } else { 0 });
        let mut subject = control.clone();
        subject.outputs[0] = model_output(&program, asset, value);
        assert_eq!(
            model::evaluate(&leaf, &subject),
            Err(model::Refusal {
                instruction: site,
                kind: "unequal"
            })
        );
    }
    let mut wrong_asset_parent = control.clone();
    wrong_asset_parent.inputs[1].output.asset.payload = other.to_vec();
    assert_eq!(
        model::evaluate(&leaf, &wrong_asset_parent),
        Err(model::Refusal {
            instruction: 21,
            kind: "unequal"
        })
    );
    let mut wrong_value_parent = control.clone();
    wrong_value_parent.inputs[1].output.value.payload = 59_u64.to_le_bytes().to_vec();
    assert_eq!(
        model::evaluate(&leaf, &wrong_value_parent),
        Err(model::Refusal {
            instruction: 21,
            kind: "unequal"
        })
    );
    let mut later_parent = control;
    later_parent.inputs[1].birth = 1;
    assert_eq!(
        model::evaluate(&leaf, &later_parent),
        Err(model::Refusal {
            instruction: 26,
            kind: "unequal"
        })
    );
}

#[test]
#[allow(clippy::too_many_lines)] // Proof, tally and signature controls bind one serialized structural shape.
fn fixed_capture_proofs_and_sponsor() {
    use elements::bitcoin::hashes::Hash as _;
    use elements::secp256k1_zkp::{
        Generator, Message, PedersenCommitment, PublicKey, Secp256k1, SecretKey, Tag, ZERO_TWEAK,
        verify_commitments_sum_to_equal,
    };
    use target_elements_conformance::confidential_funding::RangeproofVerifier;
    use target_elements_conformance::confidential_oracles::{
        ReferenceProofMaterial, ReferenceRangeproofVerifier,
    };
    use transaction::bytes::OutputWitness;

    let secp = Secp256k1::new();
    let generator = Generator::new_unblinded(&secp, Tag::from(POLICY_ASSET));
    let proof_owner = ReferenceProofMaterial::new();
    let proof_checker = ReferenceRangeproofVerifier::new();
    let mut one = [0_u8; 32];
    one[31] = 1;
    let two = [2_u8; 32];
    let three = [3_u8; 32];
    let mut negative_one = elements::secp256k1_zkp::constants::CURVE_ORDER;
    negative_one[31] -= 1;
    let key = SecretKey::from_slice(&one).unwrap();
    let public = PublicKey::from_secret_key(&secp, &key).serialize();
    let mut sponsor_program = vec![0, 20];
    sponsor_program.extend(elements::hashes::hash160::Hash::hash(&public).to_byte_array());
    let member_program = independent_commitment(
        &target_elements::reviewed_elements_tapscript().unwrap(),
        parameters(42, 60),
        &PublicKeyedCurve,
    )
    .unwrap()
    .output_program;
    let change_program = sponsor_program.clone();
    let commitment = |value, blinder: &[u8; 32]| {
        proof_owner
            .value_commitment(&POLICY_ASSET, value, blinder)
            .unwrap()
    };
    let first_commitment = commitment(60, &one);
    let second_commitment = commitment(930, &negative_one);
    let first_nonce = proof_owner.nonce_commitment(&two).unwrap();
    let second_nonce = proof_owner.nonce_commitment(&three).unwrap();
    let first_proof = proof_owner
        .range_proof(
            &POLICY_ASSET,
            60,
            &one,
            &two,
            &first_commitment,
            &member_program,
        )
        .unwrap();
    let second_proof = proof_owner
        .range_proof(
            &POLICY_ASSET,
            930,
            &negative_one,
            &three,
            &second_commitment,
            &change_program,
        )
        .unwrap();
    let printed_asset = vectors::maturity_observer::encode_hex(
        &POLICY_ASSET.iter().rev().copied().collect::<Vec<_>>(),
    );
    assert!(proof_checker.verifies(
        &first_proof,
        &first_commitment,
        &printed_asset,
        &member_program,
    ));
    assert!(proof_checker.verifies(
        &second_proof,
        &second_commitment,
        &printed_asset,
        &change_program,
    ));
    let mut broken_proof = first_proof.clone();
    broken_proof[0] ^= 1;
    assert!(!proof_checker.verifies(
        &broken_proof,
        &first_commitment,
        &printed_asset,
        &member_program,
    ));
    let input_commitment = PedersenCommitment::new(&secp, 1_000, ZERO_TWEAK, generator);
    let fee_commitment = PedersenCommitment::new(&secp, 10, ZERO_TWEAK, generator);
    let first = PedersenCommitment::from_slice(&first_commitment).unwrap();
    let second = PedersenCommitment::from_slice(&second_commitment).unwrap();
    assert!(verify_commitments_sum_to_equal(
        &secp,
        &[input_commitment],
        &[first, second, fee_commitment],
    ));
    let wrong_tally = PedersenCommitment::from_slice(&commitment(929, &negative_one)).unwrap();
    assert!(!verify_commitments_sum_to_equal(
        &secp,
        &[input_commitment],
        &[first, wrong_tally, fee_commitment],
    ));
    let sponsor_parent = parent(&sponsor_program, POLICY_ASSET, 1_000, 91);
    let sponsor_outpoint =
        input_offer(&sponsor_parent, 0, KeyedFixedInputRole::Collateral).outpoint;
    let outputs = vec![
        TargetOutput::new(
            AssetField::Explicit(AssetId::from_internal(POLICY_ASSET)),
            ValueField::Commitment(first_commitment),
            NonceField::Commitment(first_nonce),
            member_program,
        ),
        TargetOutput::new(
            AssetField::Explicit(AssetId::from_internal(POLICY_ASSET)),
            ValueField::Commitment(second_commitment),
            NonceField::Commitment(second_nonce),
            change_program,
        ),
        output_with_program(POLICY_ASSET, 10, vec![]),
    ];
    let unsigned = TargetTransaction::with_output_witnesses(
        2,
        vec![TargetInput::new(sponsor_outpoint, 0x8000_0001)],
        outputs.clone(),
        0,
        vec![InputWitness::new(vec![])],
        vec![
            OutputWitness::range_proof_only(first_proof),
            OutputWitness::range_proof_only(second_proof),
            OutputWitness::empty(),
        ],
    )
    .unwrap();
    let parent = TargetTransaction::decode(&sponsor_parent).unwrap();
    let spent = &parent.outputs()[0];
    let hybrid_digest = transaction::keyed_fixed_sponsor::keyed_fixed_sponsor_digest(
        &unsigned.encode_without_witness(),
        0,
        spent,
    )
    .unwrap();
    let reference: elements::Transaction =
        elements::encode::deserialize(&unsigned.encode()).unwrap();
    let script = elements::Script::new_p2pkh(
        &elements::PubkeyHash::from_slice(&spent.program()[2..]).unwrap(),
    );
    assert_eq!(
        hybrid_digest,
        elements::sighash::SighashCache::new(&reference)
            .segwitv0_sighash(
                0,
                &script,
                elements::confidential::Value::Explicit(1_000),
                elements::EcdsaSighashType::All
            )
            .to_byte_array()
    );
    let mut hybrid_signature = secp
        .sign_ecdsa(&Message::from_digest(hybrid_digest), &key)
        .serialize_der()
        .to_vec();
    hybrid_signature.push(1);
    let hybrid_stack = vec![hybrid_signature, public.to_vec()];
    assert_eq!(
        transaction::keyed_fixed_sponsor::verify_keyed_fixed_sponsor(
            &unsigned.encode_without_witness(),
            0,
            spent,
            &hybrid_stack
        ),
        Ok(())
    );
    let mut proof_changed = unsigned.output_witnesses().to_vec();
    let mut proof_bytes = broken_proof;
    proof_bytes[0] ^= 2;
    proof_changed[0] = OutputWitness::range_proof_only(proof_bytes);
    let proof_subject = TargetTransaction::with_output_witnesses(
        unsigned.version(),
        unsigned.inputs().to_vec(),
        unsigned.outputs().to_vec(),
        unsigned.lock_time(),
        unsigned.witnesses().to_vec(),
        proof_changed,
    )
    .unwrap();
    assert_eq!(
        transaction::keyed_fixed_sponsor::keyed_fixed_sponsor_digest(
            &proof_subject.encode_without_witness(),
            0,
            spent
        )
        .unwrap(),
        hybrid_digest
    );
    assert_eq!(
        transaction::keyed_fixed_sponsor::verify_keyed_fixed_sponsor(
            &proof_subject.encode_without_witness(),
            0,
            spent,
            &hybrid_stack
        ),
        Ok(())
    );
    let mut changed_outputs = unsigned.outputs().to_vec();
    changed_outputs[1] = TargetOutput::new(
        changed_outputs[1].asset(),
        ValueField::Commitment(commitment(929, &negative_one)),
        changed_outputs[1].nonce(),
        changed_outputs[1].program().to_vec(),
    );
    let wrong_balance = TargetTransaction::with_output_witnesses(
        unsigned.version(),
        unsigned.inputs().to_vec(),
        changed_outputs,
        unsigned.lock_time(),
        unsigned.witnesses().to_vec(),
        unsigned.output_witnesses().to_vec(),
    )
    .unwrap();
    assert_eq!(
        transaction::keyed_fixed_sponsor::verify_keyed_fixed_sponsor(
            &wrong_balance.encode_without_witness(),
            0,
            spent,
            &hybrid_stack
        ),
        Err(transaction::keyed_fixed_sponsor::KeyedFixedSponsorRefusal::InvalidSignature)
    );
    let mut changed_outputs = unsigned.outputs().to_vec();
    changed_outputs[0] = TargetOutput::new(
        changed_outputs[0].asset(),
        changed_outputs[0].value(),
        NonceField::Commitment(second_nonce),
        changed_outputs[0].program().to_vec(),
    );
    let wrong_nonce = TargetTransaction::with_output_witnesses(
        unsigned.version(),
        unsigned.inputs().to_vec(),
        changed_outputs,
        unsigned.lock_time(),
        unsigned.witnesses().to_vec(),
        unsigned.output_witnesses().to_vec(),
    )
    .unwrap();
    assert_eq!(
        transaction::keyed_fixed_sponsor::verify_keyed_fixed_sponsor(
            &wrong_nonce.encode_without_witness(),
            0,
            spent,
            &hybrid_stack
        ),
        Err(transaction::keyed_fixed_sponsor::KeyedFixedSponsorRefusal::InvalidSignature)
    );
    let explicit_signable = TargetTransaction::new(
        unsigned.version(),
        unsigned.inputs().to_vec(),
        vec![
            output_with_program(POLICY_ASSET, 60, outputs[0].program().to_vec()),
            output_with_program(POLICY_ASSET, 930, outputs[1].program().to_vec()),
            output_with_program(POLICY_ASSET, 10, vec![]),
        ],
        unsigned.lock_time(),
        unsigned.witnesses().to_vec(),
    )
    .unwrap();
    let digest = transaction::keyed_fixed_sponsor::keyed_fixed_sponsor_digest(
        &explicit_signable.encode_without_witness(),
        0,
        spent,
    )
    .unwrap();
    let mut signature = secp
        .sign_ecdsa(&Message::from_digest(digest), &key)
        .serialize_der()
        .to_vec();
    signature.push(1);
    let stack = vec![signature, public.to_vec()];
    assert_eq!(
        transaction::keyed_fixed_sponsor::verify_keyed_fixed_sponsor(
            &explicit_signable.encode_without_witness(),
            0,
            spent,
            &stack,
        ),
        Ok(()),
    );
    for committed in [false, true] {
        for present in [false, true] {
            let value = if committed {
                ValueField::Commitment(first_commitment)
            } else {
                ValueField::Explicit(60)
            };
            let change_value = if committed {
                ValueField::Commitment(second_commitment)
            } else {
                ValueField::Explicit(930)
            };
            let nonce = if present {
                NonceField::Commitment(first_nonce)
            } else {
                NonceField::Null
            };
            let change_nonce = if present {
                NonceField::Commitment(second_nonce)
            } else {
                NonceField::Null
            };
            let formed = TargetTransaction::with_output_witnesses(
                unsigned.version(),
                unsigned.inputs().to_vec(),
                vec![
                    TargetOutput::new(
                        outputs[0].asset(),
                        value,
                        nonce,
                        outputs[0].program().to_vec(),
                    ),
                    TargetOutput::new(
                        outputs[1].asset(),
                        change_value,
                        change_nonce,
                        outputs[1].program().to_vec(),
                    ),
                    outputs[2].clone(),
                ],
                unsigned.lock_time(),
                unsigned.witnesses().to_vec(),
                vec![
                    if committed {
                        unsigned.output_witnesses()[0].clone()
                    } else {
                        OutputWitness::empty()
                    },
                    if committed {
                        unsigned.output_witnesses()[1].clone()
                    } else {
                        OutputWitness::empty()
                    },
                    OutputWitness::empty(),
                ],
            )
            .unwrap();
            let frozen = formed.encode_without_witness();
            let digest =
                transaction::keyed_fixed_sponsor::keyed_fixed_sponsor_digest(&frozen, 0, spent)
                    .unwrap();
            let independent: elements::Transaction =
                elements::encode::deserialize(&formed.encode()).unwrap();
            assert_eq!(
                digest,
                elements::sighash::SighashCache::new(&independent)
                    .segwitv0_sighash(
                        0,
                        &script,
                        elements::confidential::Value::Explicit(1_000),
                        elements::EcdsaSighashType::All
                    )
                    .to_byte_array()
            );
            let mut signature = secp
                .sign_ecdsa(&Message::from_digest(digest), &key)
                .serialize_der()
                .to_vec();
            signature.push(1);
            let witness = vec![signature, public.to_vec()];
            assert_eq!(
                transaction::keyed_fixed_sponsor::verify_keyed_fixed_sponsor(
                    &frozen, 0, spent, &witness
                ),
                Ok(())
            );
            let mut changed = formed.outputs().to_vec();
            changed[0] = TargetOutput::new(
                changed[0].asset(),
                changed[0].value(),
                if present {
                    NonceField::Commitment(second_nonce)
                } else {
                    NonceField::Commitment(first_nonce)
                },
                changed[0].program().to_vec(),
            );
            let changed = TargetTransaction::with_output_witnesses(
                formed.version(),
                formed.inputs().to_vec(),
                changed,
                formed.lock_time(),
                formed.witnesses().to_vec(),
                formed.output_witnesses().to_vec(),
            )
            .unwrap();
            let changed_frozen = changed.encode_without_witness();
            assert_eq!(
                transaction::keyed_fixed_sponsor::verify_keyed_fixed_sponsor(
                    &changed_frozen,
                    0,
                    spent,
                    &witness
                ),
                Err(transaction::keyed_fixed_sponsor::KeyedFixedSponsorRefusal::InvalidSignature)
            );
            let changed_digest = transaction::keyed_fixed_sponsor::keyed_fixed_sponsor_digest(
                &changed_frozen,
                0,
                spent,
            )
            .unwrap();
            let mut replacement = secp
                .sign_ecdsa(&Message::from_digest(changed_digest), &key)
                .serialize_der()
                .to_vec();
            replacement.push(1);
            assert_eq!(
                transaction::keyed_fixed_sponsor::verify_keyed_fixed_sponsor(
                    &changed_frozen,
                    0,
                    spent,
                    &[replacement, public.to_vec()]
                ),
                Ok(())
            );
        }
    }
    let mut wrong_stack = stack;
    let mut unequal_digest = digest;
    unequal_digest[0] ^= 1;
    wrong_stack[0] = secp
        .sign_ecdsa(&Message::from_digest(unequal_digest), &key)
        .serialize_der()
        .to_vec();
    wrong_stack[0].push(1);
    assert_eq!(
        transaction::keyed_fixed_sponsor::verify_keyed_fixed_sponsor(
            &explicit_signable.encode_without_witness(),
            0,
            spent,
            &wrong_stack,
        ),
        Err(transaction::keyed_fixed_sponsor::KeyedFixedSponsorRefusal::InvalidSignature)
    );
    assert_eq!(outputs.len(), 3);
}

#[path = "keyed_fixed_capture/native.rs"]
mod native;

#[test]
#[ignore = "operator-only fixed-form native capture"]
fn native_pilot() {
    native::run(CaptureFamily::Pilot);
}

#[test]
#[ignore = "operator-only fixed-form native capture"]
fn native_clauses() {
    native::run(CaptureFamily::Clauses);
}

#[test]
#[ignore = "operator-only fixed-form native capture"]
fn native_policy() {
    native::run(CaptureFamily::Policy);
}

#[test]
#[ignore = "operator-only fixed-form native capture"]
fn native_scaling() {
    native::run(CaptureFamily::Scaling);
}

#[test]
#[ignore = "operator-only fixed-form native capture"]
fn native_histories() {
    native::run(CaptureFamily::Histories);
}

#[test]
#[ignore = "operator-only fixed-form sponsor capture"]
fn native_sponsor() {
    native::run(CaptureFamily::Sponsor);
}

#[test]
#[ignore = "operator-only fixed-form native capture"]
fn native_legacy_copy() {
    native::run(CaptureFamily::LegacyCopy);
}
