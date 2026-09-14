//! Operator boundary checks using public fixture bytes and a deterministic double.
//!
//! The announcement candidate arrives with the synthetic bite. A finalized live
//! transfer stands in for one frozen script-path candidate here.

use std::cell::Cell;

use linker::backend::StackItem;
use linker::live_backend::{
    EstablishedOperatorProfile, OperatorKey, OperatorKeyCurveValidity, OperatorProfileDisposition,
};
use linker::{CandidateDeploymentIdentity, OperatorDeploymentBinding};
use sha2::{Digest, Sha256};
use tapscript::{operator_key_encoding_closure, selected_operator_profile};
use target_elements::{EncodingClass, LeafVersion, TargetContractVersion};

use super::live_support::{
    FixtureCurve, LIVE_INTERNAL_KEY, RefusingCurve, operator_finalized_fixture,
};
use super::{outpoint, reviewed_target};
use crate::bytes::{InputWitness, TargetInput, TargetOutput, TargetTransaction};
use crate::live_finalize::FinalizedLiveTransfer;
use crate::operator_signing::{
    OperatorEvidenceStanding, OperatorSigningInput, OperatorSigningRefusal, OperatorSigningRequest,
    OperatorSigningResponse, ScriptPathSignatureVerifier, ScriptPathVerifierRejection,
    authorize_operator,
};
use crate::script_path_signing::{LiveDeployment, ScriptPathCensusRefusal, SpentOutputCensusEntry};
use crate::taproot::{Digest32, leaf_hash};

const GENESIS: Digest32 = [0x21; 32];
const VERIFIER_NAME: &str = "deterministic public-data test double; not Schnorr";

fn operator_key(byte: u8) -> OperatorKey {
    let target = reviewed_target();
    let closure = operator_key_encoding_closure(target.definition().authorization());
    OperatorKey::new(&closure, closure.approved(), vec![byte; 32])
        .expect("public fixture bytes have the approved encoding")
}

fn identity(network: u8, genesis: Digest32) -> CandidateDeploymentIdentity {
    CandidateDeploymentIdentity::new([network; 32], genesis).expect("nonzero fixture identifiers")
}

struct Fixture {
    finalized: FinalizedLiveTransfer,
    binding: OperatorDeploymentBinding,
}

impl Fixture {
    fn new() -> Self {
        let target = reviewed_target();
        let internal = StackItem::encoded(
            &target,
            EncodingClass::XOnlyPublicKey,
            LIVE_INTERNAL_KEY.to_vec(),
        )
        .expect("fixture internal key encoding");
        let profile = EstablishedOperatorProfile::establish(selected_operator_profile(), &target)
            .expect("the selected profile is established");
        let binding = OperatorDeploymentBinding::bind(
            &target,
            operator_key(0x33),
            profile,
            identity(0x11, GENESIS),
            &internal,
        )
        .expect("the deployment binds");
        Self {
            finalized: operator_finalized_fixture(),
            binding,
        }
    }

    fn input(&self) -> OperatorSigningInput {
        let record = &self.finalized.receipts()[0];
        OperatorSigningInput::new(
            u32::from(record.position()),
            leaf_hash(LeafVersion::TAPSCRIPT, record.leaf_script()),
            LeafVersion::TAPSCRIPT,
            record.leaf_script().to_vec(),
            record.control_block().to_vec(),
        )
    }

    fn spent(&self) -> Vec<SpentOutputCensusEntry> {
        self.finalized
            .receipts()
            .iter()
            .map(|record| {
                SpentOutputCensusEntry::new(
                    record.asset(),
                    record.value(),
                    record.program().to_vec(),
                )
            })
            .collect()
    }

    fn freeze(&self) -> OperatorSigningRequest<'_> {
        self.freeze_candidate(self.finalized.protected().clone())
    }

    fn freeze_candidate(&self, candidate: TargetTransaction) -> OperatorSigningRequest<'_> {
        OperatorSigningRequest::freeze(
            &reviewed_target(),
            &self.binding,
            candidate,
            self.spent(),
            LiveDeployment::new(GENESIS),
            self.input(),
            &FixtureCurve,
        )
        .expect("the fixture freezes")
    }

    fn parts(&self, request: &OperatorSigningRequest<'_>) -> ResponseParts {
        ResponseParts {
            index: 0,
            signature: signature(
                self.binding.key().bytes(),
                request.message().with_vector_grown(),
            ),
            type_byte: 0,
            echo: self.finalized.protected().encode(),
            operator: operator_key(0x33),
            deployment: identity(0x11, GENESIS),
            revision: TargetContractVersion::V2,
        }
    }
}

struct ResponseParts {
    index: u32,
    signature: Vec<u8>,
    type_byte: u8,
    echo: Vec<u8>,
    operator: OperatorKey,
    deployment: CandidateDeploymentIdentity,
    revision: TargetContractVersion,
}

impl ResponseParts {
    fn response(self) -> OperatorSigningResponse {
        OperatorSigningResponse::new(
            self.index,
            self.signature,
            self.type_byte,
            self.echo,
            self.operator,
            self.deployment,
            self.revision,
        )
    }
}

/// A deterministic test function, not Schnorr and not signature evidence.
fn signature(key: &[u8], message: &Digest32) -> Vec<u8> {
    let first = Sha256::new()
        .chain_update(key)
        .chain_update(message)
        .finalize();
    let second = Sha256::new()
        .chain_update(message)
        .chain_update(key)
        .finalize();
    first.iter().chain(second.iter()).copied().collect()
}

/// A test-only double, not cryptography or native evidence.
#[derive(Default)]
struct TestVerifier {
    calls: Cell<usize>,
}

impl ScriptPathSignatureVerifier for TestVerifier {
    fn verify(
        &self,
        key: &[u8],
        message: &Digest32,
        offered: &[u8],
    ) -> Result<(), ScriptPathVerifierRejection> {
        self.calls.set(self.calls.get() + 1);
        if offered == signature(key, message) {
            Ok(())
        } else {
            Err(ScriptPathVerifierRejection::new(
                "test function mismatch".to_owned(),
            ))
        }
    }

    fn description(&self) -> &str {
        VERIFIER_NAME
    }
}

fn refused(change: impl FnOnce(&mut ResponseParts)) -> OperatorSigningRefusal {
    let fixture = Fixture::new();
    let request = fixture.freeze();
    let mut parts = fixture.parts(&request);
    change(&mut parts);
    let verifier = TestVerifier::default();
    let refusal = authorize_operator(request, [parts.response()], &verifier)
        .expect_err("refusal returns no accepted candidate or witness");
    assert_eq!(verifier.calls.get(), 0, "bindings fail before verification");
    refusal
}

fn with_outputs(candidate: &TargetTransaction, outputs: Vec<TargetOutput>) -> TargetTransaction {
    TargetTransaction::with_output_witnesses(
        candidate.version(),
        candidate.inputs().to_vec(),
        outputs,
        candidate.lock_time(),
        candidate.witnesses().to_vec(),
        candidate.output_witnesses().to_vec(),
    )
    .expect("the changed candidate retains its cardinalities")
}

#[test]
fn freeze_records_one_input_and_the_discharged_curve_obligation() {
    let fixture = Fixture::new();
    let request = fixture.freeze();
    assert!(request.curve_validity_discharged());
    assert_eq!(
        request.binding().key().curve_validity(),
        OperatorKeyCurveValidity::Unverified
    );
    assert_ne!(request.binding().key().bytes(), LIVE_INTERNAL_KEY);
    assert_eq!(request.candidate(), fixture.finalized.protected());
    assert_eq!(request.frozen_bytes(), request.candidate().encode());
    assert_eq!(request.census().signing_inputs().len(), 1);
    assert_eq!(request.input_index(), 0);
    assert_eq!(
        request.census().output_witnesses(),
        request.candidate().output_witnesses()
    );
    assert_eq!(request.census().genesis_block_hash(), &GENESIS);
    assert_eq!(request.census().spent_outputs(), fixture.spent());
    assert!(request.message().candidates_are_distinct());
}

#[test]
fn wrong_genesis_precedes_curve_and_census_checks() {
    let fixture = Fixture::new();
    let result = OperatorSigningRequest::freeze(
        &reviewed_target(),
        &fixture.binding,
        fixture.finalized.protected().clone(),
        Vec::new(),
        LiveDeployment::new([0x22; 32]),
        fixture.input(),
        &RefusingCurve,
    );
    assert_eq!(
        result.expect_err("no frozen request"),
        OperatorSigningRefusal::GenesisMismatch {
            bound: GENESIS,
            offered: [0x22; 32],
        }
    );
}

#[test]
fn freeze_refuses_a_key_the_curve_rejects() {
    let fixture = Fixture::new();
    let result = OperatorSigningRequest::freeze(
        &reviewed_target(),
        &fixture.binding,
        fixture.finalized.protected().clone(),
        Vec::new(),
        LiveDeployment::new(GENESIS),
        fixture.input(),
        &RefusingCurve,
    );
    assert_eq!(
        result.expect_err("curve precedes cardinality"),
        OperatorSigningRefusal::OperatorKeyIsNotACurvePoint {
            key: vec![0x33; 32],
        }
    );
}

#[test]
fn freeze_refuses_spent_output_cardinality_mismatch() {
    let fixture = Fixture::new();
    let result = OperatorSigningRequest::freeze(
        &reviewed_target(),
        &fixture.binding,
        fixture.finalized.protected().clone(),
        Vec::new(),
        LiveDeployment::new(GENESIS),
        fixture.input(),
        &FixtureCurve,
    );
    assert_eq!(
        result.expect_err("no frozen request"),
        OperatorSigningRefusal::Census(ScriptPathCensusRefusal::SpentOutputCardinalityMismatch {
            inputs: 1,
            spent_outputs: 0
        },)
    );
}

#[test]
fn freeze_refuses_an_altered_control_path() {
    let fixture = Fixture::new();
    let input = fixture.input();
    // This fixture has one receipt input. Alter one sibling path element.
    let mut control = input.control_block().to_vec();
    assert!(control.len() > 33);
    control[33] ^= 1;
    let wrong = OperatorSigningInput::new(
        input.input_index(),
        *input.tapleaf_hash(),
        input.leaf_version(),
        input.leaf_script().to_vec(),
        control,
    );
    let result = OperatorSigningRequest::freeze(
        &reviewed_target(),
        &fixture.binding,
        fixture.finalized.protected().clone(),
        fixture.spent(),
        LiveDeployment::new(GENESIS),
        wrong,
        &FixtureCurve,
    );
    assert_eq!(
        result.expect_err("the leaf no longer commits"),
        OperatorSigningRefusal::Census(ScriptPathCensusRefusal::LeafHashDoesNotCommit {
            input_index: 0
        },)
    );
}

#[test]
fn freeze_refuses_a_substituted_leaf_script() {
    let fixture = Fixture::new();
    let input = fixture.input();
    let mut script = input.leaf_script().to_vec();
    script.push(0x51);
    let computed = leaf_hash(input.leaf_version(), &script);
    let wrong = OperatorSigningInput::new(
        0,
        *input.tapleaf_hash(),
        input.leaf_version(),
        script,
        input.control_block().to_vec(),
    );
    let result = OperatorSigningRequest::freeze(
        &reviewed_target(),
        &fixture.binding,
        fixture.finalized.protected().clone(),
        fixture.spent(),
        LiveDeployment::new(GENESIS),
        wrong,
        &FixtureCurve,
    );
    assert_eq!(
        result.expect_err("the script must be the selected leaf"),
        OperatorSigningRefusal::WrongLeaf {
            input_index: 0,
            expected: *input.tapleaf_hash(),
            computed
        }
    );
}

#[test]
fn freeze_refuses_a_malformed_control_block() {
    let fixture = Fixture::new();
    let input = fixture.input();
    let wrong = OperatorSigningInput::new(
        0,
        *input.tapleaf_hash(),
        input.leaf_version(),
        input.leaf_script().to_vec(),
        vec![0; 32],
    );
    let result = OperatorSigningRequest::freeze(
        &reviewed_target(),
        &fixture.binding,
        fixture.finalized.protected().clone(),
        fixture.spent(),
        LiveDeployment::new(GENESIS),
        wrong,
        &FixtureCurve,
    );
    assert_eq!(
        result.expect_err("no frozen request"),
        OperatorSigningRefusal::Census(ScriptPathCensusRefusal::ControlBlockMalformed {
            input_index: 0,
            offered: 32
        },)
    );
}

#[test]
fn freeze_refuses_an_out_of_range_input() {
    let fixture = Fixture::new();
    let input = fixture.input();
    let wrong = OperatorSigningInput::new(
        1,
        *input.tapleaf_hash(),
        input.leaf_version(),
        input.leaf_script().to_vec(),
        input.control_block().to_vec(),
    );
    let result = OperatorSigningRequest::freeze(
        &reviewed_target(),
        &fixture.binding,
        fixture.finalized.protected().clone(),
        fixture.spent(),
        LiveDeployment::new(GENESIS),
        wrong,
        &FixtureCurve,
    );
    assert_eq!(
        result.expect_err("no frozen request"),
        OperatorSigningRefusal::Census(ScriptPathCensusRefusal::SigningInputOutOfRange {
            input_index: 1,
            inputs: 1
        },)
    );
}

#[test]
fn positive_authorization_has_the_witness_abi_and_only_in_process_standing() {
    let fixture = Fixture::new();
    let request = fixture.freeze();
    let response = fixture.parts(&request).response();
    assert_eq!(response.input_index(), request.input_index());
    assert_eq!(response.type_byte(), 0);
    assert_eq!(response.echo(), request.frozen_bytes());
    assert_eq!(response.operator(), fixture.binding.key());
    assert_eq!(response.deployment(), fixture.binding.deployment());
    assert_eq!(
        response.capability_revision(),
        fixture.binding.capability_revision()
    );
    let expected = vec![
        response.signature().to_vec(),
        fixture.input().leaf_script().to_vec(),
        fixture.input().control_block().to_vec(),
    ];
    let verifier = TestVerifier::default();
    let accepted = authorize_operator(request, [response], &verifier).expect("the double accepts");
    assert_eq!(accepted.witness().stack(), expected);
    assert_eq!(
        accepted.request().candidate(),
        fixture.finalized.protected()
    );
    assert_eq!(
        accepted.standing(),
        &OperatorEvidenceStanding::InProcessVerified {
            verifier: VERIFIER_NAME.to_owned(),
        }
    );
    assert_eq!(verifier.calls.get(), 1);
}

#[test]
fn missing_response_returns_no_artifact() {
    let fixture = Fixture::new();
    let result = authorize_operator(fixture.freeze(), [], &TestVerifier::default());
    assert_eq!(
        result.expect_err("no witness"),
        OperatorSigningRefusal::MissingResponse { input_index: 0 }
    );
}

#[test]
fn second_response_for_the_frozen_input_is_duplicate() {
    let fixture = Fixture::new();
    let request = fixture.freeze();
    let answer = fixture.parts(&request).response();
    let result = authorize_operator(request, [answer.clone(), answer], &TestVerifier::default());
    assert_eq!(
        result.expect_err("no witness"),
        OperatorSigningRefusal::DuplicateResponse { input_index: 0 }
    );
}

#[test]
fn second_response_for_another_input_is_unexpected() {
    let fixture = Fixture::new();
    let request = fixture.freeze();
    let first = fixture.parts(&request).response();
    let mut extra = fixture.parts(&request);
    extra.index = 7;
    let result = authorize_operator(request, [first, extra.response()], &TestVerifier::default());
    assert_eq!(
        result.expect_err("no witness"),
        OperatorSigningRefusal::UnexpectedResponse { input_index: 7 }
    );
}

#[test]
fn sole_response_for_another_input_is_wrong_input() {
    assert_eq!(
        refused(|parts| parts.index = 7),
        OperatorSigningRefusal::WrongInput {
            expected: 0,
            offered: 7
        }
    );
}

#[test]
fn another_operator_is_refused_before_other_binding_faults() {
    assert_eq!(
        refused(|parts| {
            parts.operator = operator_key(0x44);
            parts.deployment = identity(0x22, GENESIS);
            parts.revision = TargetContractVersion::V1;
        }),
        OperatorSigningRefusal::WrongOperator {
            bound: operator_key(0x33),
            offered: operator_key(0x44)
        }
    );
}

#[test]
fn another_network_is_refused_before_stale_revision() {
    assert_eq!(
        refused(|parts| {
            parts.deployment = identity(0x22, GENESIS);
            parts.revision = TargetContractVersion::V1;
        }),
        OperatorSigningRefusal::WrongDeployment {
            bound: Box::new(identity(0x11, GENESIS)),
            offered: Box::new(identity(0x22, GENESIS)),
        }
    );
}

#[test]
fn another_response_genesis_is_refused() {
    assert_eq!(
        refused(|parts| parts.deployment = identity(0x11, [0x22; 32])),
        OperatorSigningRefusal::WrongDeployment {
            bound: Box::new(identity(0x11, GENESIS)),
            offered: Box::new(identity(0x11, [0x22; 32])),
        }
    );
}

#[test]
fn offered_v1_is_a_named_stale_profile_refusal() {
    // A stale binding cannot be constructed through the reviewed public API.
    // The response can offer V1, so it exercises the binding's revision check.
    assert_eq!(
        refused(|parts| parts.revision = TargetContractVersion::V1),
        OperatorSigningRefusal::WrongProfile(OperatorProfileDisposition::StaleRevision {
            pinned: TargetContractVersion::V2,
            offered: TargetContractVersion::V1,
        })
    );
}

#[test]
fn nondefault_type_is_refused_before_an_empty_signature() {
    assert_eq!(
        refused(|parts| {
            parts.type_byte = 1;
            parts.signature.clear();
        }),
        OperatorSigningRefusal::WrongTypeByte { offered: 1 }
    );
}

#[test]
fn empty_signature_has_its_own_refusal() {
    assert_eq!(
        refused(|parts| parts.signature.clear()),
        OperatorSigningRefusal::EmptySignature { input_index: 0 }
    );
}

#[test]
fn a_63_byte_signature_is_malformed() {
    assert_eq!(
        refused(|parts| parts.signature.truncate(63)),
        OperatorSigningRefusal::MalformedSignature { offered: 63 }
    );
}

#[test]
fn a_65_byte_signature_is_malformed() {
    assert_eq!(
        refused(|parts| parts.signature.push(1)),
        OperatorSigningRefusal::MalformedSignature { offered: 65 }
    );
}

#[test]
fn echo_with_a_changed_successor_program_is_refused() {
    let fixture = Fixture::new();
    let candidate = fixture.finalized.protected();
    let mut outputs = candidate.outputs().to_vec();
    let index = outputs
        .iter()
        .position(|output| !output.is_fee())
        .expect("a successor");
    let output = &outputs[index];
    outputs[index] = TargetOutput::new(output.asset(), output.value(), output.nonce(), vec![0x51]);
    let echo = with_outputs(candidate, outputs).encode();
    assert_eq!(
        refused(|parts| parts.echo = echo),
        OperatorSigningRefusal::BoundToOtherBytes { input_index: 0 }
    );
}

#[test]
fn echo_with_a_sponsor_input_added_is_refused() {
    let fixture = Fixture::new();
    let candidate = fixture.finalized.protected();
    let mut inputs = candidate.inputs().to_vec();
    inputs.push(TargetInput::new(outpoint(0xe1, 0), 0xffff_fffe));
    let mut witnesses = candidate.witnesses().to_vec();
    witnesses.push(InputWitness::new(Vec::new()));
    let echo = TargetTransaction::with_output_witnesses(
        candidate.version(),
        inputs,
        candidate.outputs().to_vec(),
        candidate.lock_time(),
        witnesses,
        candidate.output_witnesses().to_vec(),
    )
    .expect("the added input has its own witness")
    .encode();
    assert_eq!(
        refused(|parts| parts.echo = echo),
        OperatorSigningRefusal::BoundToOtherBytes { input_index: 0 }
    );
}

#[test]
fn echo_with_the_fee_role_changed_is_refused() {
    let fixture = Fixture::new();
    let candidate = fixture.finalized.protected();
    let mut outputs = candidate.outputs().to_vec();
    let index = outputs
        .iter()
        .position(TargetOutput::is_fee)
        .expect("a self-paid fee");
    let output = &outputs[index];
    outputs[index] = TargetOutput::new(output.asset(), output.value(), output.nonce(), vec![0x51]);
    assert!(!outputs[index].is_fee());
    let echo = with_outputs(candidate, outputs).encode();
    assert_eq!(
        refused(|parts| parts.echo = echo),
        OperatorSigningRefusal::BoundToOtherBytes { input_index: 0 }
    );
}

#[test]
fn signature_over_another_transaction_fails_even_with_the_correct_echo() {
    let fixture = Fixture::new();
    let request = fixture.freeze();
    let candidate = request.candidate();
    let changed = TargetTransaction::with_output_witnesses(
        candidate.version(),
        candidate.inputs().to_vec(),
        candidate.outputs().to_vec(),
        candidate.lock_time() + 1,
        candidate.witnesses().to_vec(),
        candidate.output_witnesses().to_vec(),
    )
    .expect("a different lock time");
    let another = fixture.freeze_candidate(changed);
    assert_ne!(request.message(), another.message());
    let mut parts = fixture.parts(&request);
    parts.signature = signature(
        fixture.binding.key().bytes(),
        another.message().with_vector_grown(),
    );
    let message = *request.message().with_vector_grown();
    let verifier = TestVerifier::default();
    let refusal =
        authorize_operator(request, [parts.response()], &verifier).expect_err("no witness");
    assert_eq!(
        refusal,
        OperatorSigningRefusal::SignatureDoesNotVerifyForFrozenMessage {
            input_index: 0,
            message,
            rejection: ScriptPathVerifierRejection::new("test function mismatch".to_owned()),
        }
    );
    assert_eq!(verifier.calls.get(), 1);
}

#[test]
fn signature_under_another_key_does_not_pass_the_committed_identity_check() {
    let fixture = Fixture::new();
    let request = fixture.freeze();
    let mut parts = fixture.parts(&request);
    parts.signature = signature(
        operator_key(0x44).bytes(),
        request.message().with_vector_grown(),
    );
    let refusal = authorize_operator(request, [parts.response()], &TestVerifier::default())
        .expect_err("a correct identity field does not repair the signature");
    assert!(matches!(
        refusal,
        OperatorSigningRefusal::SignatureDoesNotVerifyForFrozenMessage { .. }
    ));
    assert_eq!(
        ScriptPathVerifierRejection::new("reason".to_owned()).reason(),
        "reason"
    );
}
