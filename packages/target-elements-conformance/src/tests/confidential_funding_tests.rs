//! The confidential funding arm: negotiation, framing, and readback
//! binding.
//!
//! # What none of this establishes
//!
//! Nothing about a target. No node is spawned, no transaction is built,
//! no proof is generated, and no answer below came from anything that
//! executed anything. The executor's side is written out in advance and
//! the three collaborators the binding needs — the decoder, the
//! inclusion observation, and the rangeproof verifier — are scripted
//! stubs, which is what lets a test drive a refusal that a real run
//! could only reach by being wrong.
//!
//! The commitments here are the exception worth naming: they are real
//! points, produced by the package's own first-party bignum oracle over
//! published constants. That is arithmetic rather than evidence, and it
//! is used because a commitment refusal has to be able to tell an
//! admitted encoding from an inadmissible one.
//!
//! Every value in this module is public disposable development
//! material. No key is derived from any of it and none exists
//! `(´[ADR015-rule:security:test-material]´)`.

use serde_json::{Map, Value};

use target_elements::ReproducibilityContract;

use crate::commitment_oracle::commitment::commitment;
use crate::confidential_funding::{
    ConfidentialFixtureResolution, ConfidentialFramingRefusal, ConfidentialFundingExchange,
    ConfidentialFundingObservation, ConfidentialFundingOracles, ConfidentialFundingRefusal,
    ConfidentialReadbackDecoder, ConfidentialWireRecord, DecodedAssetField, DecodedFundingOutput,
    DecodedFundingTransaction, DecodedValueField, FixtureResolutionRefused, FundingProfileKind,
    MinedInclusionOracle, MinedReadbackFact, RangeproofVerifier, ReadbackDecodeRefused,
    bind_confidential_funding, check_before_sending, check_selected_profiles,
    frame_confidential_subject, negotiate_confidential_funding, resolve_binding,
};
use crate::protocol::{
    ConfidentialFixtureDigest, ConfidentialFixtureHandle, ConfidentialFundedOutput,
    ConfidentialFundingBinding, ConfidentialFundingDestination, ExecutorCapability,
    ExecutorHandshake, FundingRepresentationProfile, MinedFundingReadback, NATIVE_PROTOCOL_SCHEMA,
    NativeOperationResponse, NativeResourceObservation, ObservedOutcomeLayer, OperationCaseId,
    OperationStepKind, OperationSubject, TargetConfidentialFundingSubject, TargetFundingSubject,
    WireOutpoint,
};

use super::support::{
    TEST_DESTINATION_PROGRAMS, TEST_FIXTURE_DIGEST, TEST_FIXTURE_HANDLE,
    confidential_advertisement, confidential_binding, confidential_handshake, confidential_subject,
    nonmock_handshake,
};

/// The disposable protocol asset these cases fund in.
const ASSET: &str = "aa11aa11aa11aa11aa11aa11aa11aa11aa11aa11aa11aa11aa11aa11aa11aa11";

/// The asset identifier the first-party oracle derives a generator from.
const ASSET_BYTES: [u8; 32] = [0xaa; 32];

/// The transaction identity the scripted chain recomputes.
const TXID: &str = "1111111111111111111111111111111111111111111111111111111111111111";

/// The witness identity the scripted chain recomputes.
const WTXID: &str = "2222222222222222222222222222222222222222222222222222222222222222";

/// The block the scripted chain says holds it.
const BLOCK: &str = "3333333333333333333333333333333333333333333333333333333333333333";

/// One real value commitment, over published constants.
fn point(amount: u64, blinder: u8) -> Vec<u8> {
    commitment(&ASSET_BYTES, amount, &[blinder; 32])
        .expect("the first-party oracle commits to a public amount")
        .to_vec()
}

/// A nonce field carrying the transported-point prefix the reviewed
/// encoding admits.
fn nonce(byte: u8) -> Vec<u8> {
    let mut field = vec![0x02_u8; 33];
    field[32] = byte;
    field
}

/// Lowercase hexadecimal, which is how the target renders a script.
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// A registry that resolves exactly the registered case.
struct ScriptedRegistry;

impl ConfidentialFixtureResolution for ScriptedRegistry {
    fn resolve(
        &self,
        handle: &ConfidentialFixtureHandle,
        digest: &ConfidentialFixtureDigest,
    ) -> Result<(), FixtureResolutionRefused> {
        if handle.as_str() != TEST_FIXTURE_HANDLE {
            return Err(FixtureResolutionRefused::UnknownHandle);
        }
        if digest.bytes() != &TEST_FIXTURE_DIGEST {
            return Err(FixtureResolutionRefused::DigestMismatch);
        }
        Ok(())
    }
}

/// A decoder whose reading is written out in advance.
struct ScriptedDecoder {
    reading: Option<DecodedFundingTransaction>,
}

impl ConfidentialReadbackDecoder for ScriptedDecoder {
    fn decode(&self, _raw: &[u8]) -> Result<DecodedFundingTransaction, ReadbackDecodeRefused> {
        self.reading.clone().ok_or(ReadbackDecodeRefused)
    }
}

/// An inclusion observation written out in advance.
struct ScriptedInclusion {
    holds: bool,
}

impl MinedInclusionOracle for ScriptedInclusion {
    fn contains(&self, block_hash: &str, block_height: u32, transaction_id: &str) -> bool {
        self.holds && block_hash == BLOCK && block_height == 101 && transaction_id == TXID
    }
}

/// A rangeproof verifier written out in advance.
struct ScriptedRangeproofs {
    verifies: bool,
}

impl RangeproofVerifier for ScriptedRangeproofs {
    fn verifies(
        &self,
        proof: &[u8],
        _value_commitment: &[u8],
        _explicit_asset: &str,
        _output_program: &[u8],
    ) -> bool {
        self.verifies && !proof.is_empty()
    }
}

/// One whole exchange, valid before a test breaks one thing in it.
struct Scenario {
    subject: TargetConfidentialFundingSubject,
    recorded: ConfidentialFundingBinding,
    response: NativeOperationResponse,
    decoded: DecodedFundingTransaction,
    decodes: bool,
    included: bool,
    proofs_verify: bool,
}

impl Scenario {
    /// The valid case: two destinations, both parities, both mined.
    fn valid() -> Self {
        let mut subject = confidential_subject();
        subject.asset = Some(ASSET.to_owned());
        let outputs: Vec<ConfidentialFundedOutput> = (0..2_u32)
            .map(|index| ConfidentialFundedOutput {
                outpoint: WireOutpoint {
                    txid: TXID.to_owned(),
                    vout: index,
                },
                explicit_asset: ASSET.to_owned(),
                value_commitment: point(
                    1_000 + u64::from(index),
                    0x11 + u8::try_from(index).expect("two outputs fit a byte"),
                ),
                nonce: nonce(u8::try_from(index).expect("two outputs fit a byte")),
                script: hex(&TEST_DESTINATION_PROGRAMS[index as usize]),
                output_witness_index: index,
                surjection_proof: Vec::new(),
                rangeproof: vec![0x33; 64],
            })
            .collect();
        let decoded = DecodedFundingTransaction {
            transaction_id: TXID.to_owned(),
            witness_transaction_id: WTXID.to_owned(),
            outputs: outputs
                .iter()
                .enumerate()
                .map(|(index, output)| DecodedFundingOutput {
                    asset: DecodedAssetField::Explicit(ASSET.to_owned()),
                    value: DecodedValueField::Commitment(output.value_commitment.clone()),
                    nonce: output.nonce.clone(),
                    program: TEST_DESTINATION_PROGRAMS[index].to_vec(),
                    surjection_proof: Vec::new(),
                    rangeproof: output.rangeproof.clone(),
                })
                .collect(),
        };
        let response = NativeOperationResponse {
            schema: NATIVE_PROTOCOL_SCHEMA,
            case: OperationCaseId {
                operation: OperationStepKind::FundConfidential,
                step: "predecessor".to_owned(),
            },
            observed_layer: ObservedOutcomeLayer::Accepted,
            observed_detail: None,
            issued_asset: None,
            funded_outputs: Vec::new(),
            confidential_funded_outputs: outputs,
            mined_readback: Some(MinedFundingReadback {
                transaction_id: TXID.to_owned(),
                witness_transaction_id: WTXID.to_owned(),
                block_hash: BLOCK.to_owned(),
                block_height: 101,
                raw_transaction: vec![0x02, 0x00, 0x00, 0x00],
            }),
            accepted_txid: None,
            sponsor_witness: Vec::new(),
            signature_bound_to: None,
            resources: NativeResourceObservation::default(),
        };
        Self {
            recorded: subject.binding.clone(),
            subject,
            response,
            decoded,
            decodes: true,
            included: true,
            proofs_verify: true,
        }
    }

    /// What the binding makes of this scenario.
    fn bind(&self) -> Result<ConfidentialFundingObservation, ConfidentialFundingRefusal> {
        let advertisement = confidential_advertisement();
        let decoder = ScriptedDecoder {
            reading: self.decodes.then(|| self.decoded.clone()),
        };
        let inclusion = ScriptedInclusion {
            holds: self.included,
        };
        let rangeproofs = ScriptedRangeproofs {
            verifies: self.proofs_verify,
        };
        bind_confidential_funding(
            &ConfidentialFundingExchange {
                requested: &self.subject,
                recorded: &self.recorded,
                advertisement: &advertisement,
            },
            &self.response,
            &ConfidentialFundingOracles {
                decoder: &decoder,
                inclusion: &inclusion,
                rangeproofs: &rangeproofs,
            },
        )
    }

    /// The refusal this scenario draws.
    fn refusal(&self) -> ConfidentialFundingRefusal {
        self.bind().expect_err("the scenario is refused")
    }
}

/// The subject as a wire record.
fn subject_record(subject: &TargetConfidentialFundingSubject) -> Map<String, Value> {
    serde_json::to_value(subject)
        .expect("the subject serializes")
        .as_object()
        .expect("a subject is an object")
        .clone()
}

#[test]
fn the_valid_exchange_binds_every_output_from_the_decoded_bytes() {
    let scenario = Scenario::valid();
    let observation = scenario.bind().expect("the valid exchange binds");
    let ConfidentialFundingObservation::Funded(funding) = observation else {
        panic!("an accepted confidential step reports what it created");
    };
    assert_eq!(funding.transaction_id, TXID);
    assert_eq!(funding.witness_transaction_id, WTXID);
    assert_eq!(funding.outputs.len(), 2);
    for (index, output) in funding.outputs.iter().enumerate() {
        assert_eq!(output.output_program, TEST_DESTINATION_PROGRAMS[index]);
        assert_eq!(output.explicit_asset, ASSET);
        assert_eq!(output.outpoint.txid, TXID);
    }
}

#[test]
fn a_revision_four_executor_is_refused_rather_than_translated_for() {
    let mut handshake = confidential_handshake();
    handshake.protocol_schema = NATIVE_PROTOCOL_SCHEMA - 1;
    assert_eq!(
        negotiate_confidential_funding(&handshake),
        Err(ConfidentialFundingRefusal::ProtocolSchemaUnsupported {
            required: NATIVE_PROTOCOL_SCHEMA,
            offered: NATIVE_PROTOCOL_SCHEMA - 1,
        }),
        "a peer speaking the old revision must be refused, not reconciled",
    );
}

#[test]
fn a_capability_without_an_advertisement_and_an_advertisement_without_one_both_refuse() {
    let plain = nonmock_handshake();
    assert_eq!(
        negotiate_confidential_funding(&plain),
        Err(ConfidentialFundingRefusal::ConfidentialFundingCapabilityAbsent),
    );

    let mut claiming = nonmock_handshake();
    claiming
        .capabilities
        .insert(ExecutorCapability::ConfidentialValueTestFunding);
    assert_eq!(
        negotiate_confidential_funding(&claiming),
        Err(ConfidentialFundingRefusal::CapabilityAdvertisementDisagrees),
        "a capability with no advertisement states selections nothing offers",
    );

    let mut advertising = nonmock_handshake();
    advertising.confidential_funding = Some(confidential_advertisement());
    assert_eq!(
        negotiate_confidential_funding(&advertising),
        Err(ConfidentialFundingRefusal::CapabilityAdvertisementDisagrees),
        "an advertisement with no capability offers what nothing claims",
    );

    let honest = confidential_handshake();
    assert!(negotiate_confidential_funding(&honest).is_ok());
}

#[test]
fn an_unadvertised_profile_refuses_before_construction() {
    let subject = confidential_subject();

    let mut without_representation = confidential_advertisement();
    without_representation.representation_profiles.clear();
    assert_eq!(
        check_selected_profiles(&without_representation, &subject.binding.profiles),
        Err(ConfidentialFundingRefusal::HybridRepresentationUnsupported),
    );

    let mut without_custody = confidential_advertisement();
    without_custody.custody_profiles.clear();
    assert_eq!(
        check_selected_profiles(&without_custody, &subject.binding.profiles),
        Err(ConfidentialFundingRefusal::FundingProfileUnsupported {
            kind: FundingProfileKind::Custody,
            name: "central_public_fixtures".to_owned(),
        }),
    );

    let mut without_materializer = confidential_advertisement();
    without_materializer.materializer_profiles.clear();
    assert_eq!(
        check_selected_profiles(&without_materializer, &subject.binding.profiles),
        Err(ConfidentialFundingRefusal::FundingProfileUnsupported {
            kind: FundingProfileKind::Materializer,
            name: "guide_ctf_deterministic_v1".to_owned(),
        }),
    );

    // The contract the plan selected is refused where the executor does
    // not support it, rather than the run being moved onto the contract
    // the executor happens to hold.
    let mut recorded_only = confidential_advertisement();
    recorded_only.reproducibility_contracts =
        std::collections::BTreeSet::from([ReproducibilityContract::RecordedRandomness]);
    assert_eq!(
        check_selected_profiles(&recorded_only, &subject.binding.profiles),
        Err(ConfidentialFundingRefusal::FundingProfileUnsupported {
            kind: FundingProfileKind::ReproducibilityContract,
            name: "byte_identity".to_owned(),
        }),
    );
}

#[test]
fn a_confidential_request_with_no_destination_refuses() {
    let mut subject = confidential_subject();
    subject.destinations.clear();
    assert_eq!(
        check_before_sending(&confidential_handshake(), &subject, &ScriptedRegistry),
        Err(ConfidentialFundingRefusal::DestinationSetEmpty),
    );
}

#[test]
fn an_unknown_handle_and_a_drifted_digest_both_refuse_before_cryptographic_work() {
    let mut unknown = confidential_binding();
    unknown.fixture_handle = ConfidentialFixtureHandle::new("ctf-v1/no-such-case".to_owned());
    assert_eq!(
        resolve_binding(&ScriptedRegistry, &unknown),
        Err(ConfidentialFundingRefusal::UnknownFixtureHandle {
            handle: unknown.fixture_handle.clone(),
        }),
    );

    let mut drifted = confidential_binding();
    drifted.fixture_digest = ConfidentialFixtureDigest::new([0x00; 32]);
    assert_eq!(
        resolve_binding(&ScriptedRegistry, &drifted),
        Err(ConfidentialFundingRefusal::FixtureDigestMismatch {
            handle: drifted.fixture_handle.clone(),
        }),
    );

    assert!(
        check_before_sending(
            &confidential_handshake(),
            &confidential_subject(),
            &ScriptedRegistry,
        )
        .is_ok()
    );
}

#[test]
fn the_confidential_and_explicit_funding_subjects_are_untagged_disjoint() {
    let confidential = subject_record(&confidential_subject());
    let explicit =
        serde_json::to_value(OperationSubject::Funding(Box::new(TargetFundingSubject {
            issue_asset: true,
            asset: None,
            output_program: vec![0x51, 0x20],
            outputs: 2,
            amount_per_output: 100,
        })))
        .expect("the subject serializes");
    let explicit = explicit.as_object().expect("a subject is an object");

    let shared: Vec<&String> = confidential
        .keys()
        .filter(|member| explicit.contains_key(*member))
        .collect();
    assert_eq!(
        shared,
        vec![&"asset".to_owned(), &"issue_asset".to_owned()],
        "the two funding subjects share exactly the asset question",
    );
    assert!(
        confidential.contains_key("destinations") && confidential.contains_key("binding"),
        "the confidential subject carries the two members no other subject declares",
    );
    assert!(
        !confidential.contains_key("output_program")
            && !confidential.contains_key("outputs")
            && !confidential.contains_key("amount_per_output"),
        "the confidential subject declares none of the explicit arm's own members",
    );
}

#[test]
fn an_untagged_record_under_this_revision_is_unknown_rather_than_explicit() {
    // The explicit arm's record, framed as the arm the case identity
    // named. Nothing falls back: the refusal names a member the
    // confidential subject does not declare.
    let explicit = serde_json::to_value(TargetFundingSubject {
        issue_asset: true,
        asset: None,
        output_program: vec![0x51, 0x20],
        outputs: 2,
        amount_per_output: 100,
    })
    .expect("the subject serializes");
    let explicit = explicit
        .as_object()
        .expect("a subject is an object")
        .clone();
    let refusal = frame_confidential_subject(&explicit).expect_err("an explicit record is unknown");
    let ConfidentialFramingRefusal::Refused(ConfidentialFundingRefusal::UnknownWireMember {
        record,
        member,
    }) = refusal
    else {
        panic!("an untagged record is refused by name, not by fallback");
    };
    assert_eq!(record, ConfidentialWireRecord::Subject);
    assert!(
        ["output_program", "outputs", "amount_per_output"].contains(&member.as_str()),
        "the refusal named {member}, which is not one of the explicit arm's own members",
    );
}

#[test]
fn an_undeclared_member_of_any_confidential_record_is_refused() {
    for (record, path) in [
        (ConfidentialWireRecord::Subject, Vec::new()),
        (ConfidentialWireRecord::Binding, vec!["binding"]),
        (
            ConfidentialWireRecord::Profiles,
            vec!["binding", "profiles"],
        ),
    ] {
        let mut value = Value::Object(subject_record(&confidential_subject()));
        let mut cursor = &mut value;
        for step in &path {
            cursor = cursor
                .as_object_mut()
                .expect("a record is an object")
                .get_mut(*step)
                .expect("the record declares that member");
        }
        cursor
            .as_object_mut()
            .expect("a record is an object")
            .insert("commentary".to_owned(), Value::from("extra"));
        let record_object = value.as_object().expect("a subject is an object").clone();
        assert_eq!(
            frame_confidential_subject(&record_object),
            Err(ConfidentialFramingRefusal::Refused(
                ConfidentialFundingRefusal::UnknownWireMember {
                    record,
                    member: "commentary".to_owned(),
                }
            )),
        );
    }
}

#[test]
fn an_unassigned_representation_tag_and_an_unknown_profile_are_told_apart() {
    let mut value = Value::Object(subject_record(&confidential_subject()));
    let profiles = value
        .get_mut("binding")
        .and_then(|binding| binding.get_mut("profiles"))
        .and_then(Value::as_object_mut)
        .expect("the binding declares profiles");
    profiles.insert(
        "representation".to_owned(),
        Value::from("confidential_asset_confidential_value"),
    );
    let record = value.as_object().expect("a subject is an object").clone();
    assert_eq!(
        frame_confidential_subject(&record),
        Err(ConfidentialFramingRefusal::Refused(
            ConfidentialFundingRefusal::UnknownRepresentationTag {
                tag: "confidential_asset_confidential_value".to_owned(),
            }
        )),
        "an unassigned arm tag is its own refusal",
    );

    let mut value = Value::Object(subject_record(&confidential_subject()));
    let profiles = value
        .get_mut("binding")
        .and_then(|binding| binding.get_mut("profiles"))
        .and_then(Value::as_object_mut)
        .expect("the binding declares profiles");
    profiles.insert(
        "reproducibility_contract".to_owned(),
        Value::from("semantic_only"),
    );
    let record = value.as_object().expect("a subject is an object").clone();
    assert_eq!(
        frame_confidential_subject(&record),
        Err(ConfidentialFramingRefusal::Refused(
            ConfidentialFundingRefusal::FundingProfileUnsupported {
                kind: FundingProfileKind::ReproducibilityContract,
                name: "semantic_only".to_owned(),
            }
        )),
        "an unknown contract is a profile refusal and never a downgrade",
    );
}

#[test]
fn a_well_framed_record_reads_back_to_the_subject_that_wrote_it() {
    let subject = confidential_subject();
    assert_eq!(
        frame_confidential_subject(&subject_record(&subject)),
        Ok(subject),
    );
}

#[test]
fn explicit_fallback_from_a_confidential_request_is_a_response_arm_mismatch() {
    let mut scenario = Scenario::valid();
    scenario.response.funded_outputs = vec![crate::protocol::FundedOutput {
        outpoint: WireOutpoint {
            txid: TXID.to_owned(),
            vout: 0,
        },
        asset: ASSET.to_owned(),
        amount_satoshis: 1_000,
        script: hex(&TEST_DESTINATION_PROGRAMS[0]),
    }];
    assert_eq!(
        scenario.refusal(),
        ConfidentialFundingRefusal::ResponseArmMismatch {
            requested: OperationStepKind::FundConfidential,
            returned: OperationStepKind::Fund,
        },
    );

    let mut other_kind = Scenario::valid();
    other_kind.response.case.operation = OperationStepKind::Submit;
    assert_eq!(
        other_kind.refusal(),
        ConfidentialFundingRefusal::ResponseArmMismatch {
            requested: OperationStepKind::FundConfidential,
            returned: OperationStepKind::Submit,
        },
    );
}

#[test]
fn a_recorded_binding_that_differs_from_the_request_refuses() {
    let mut handle = Scenario::valid();
    handle.recorded.fixture_handle = ConfidentialFixtureHandle::new("ctf-v1/other-case".to_owned());
    assert_eq!(
        handle.refusal(),
        ConfidentialFundingRefusal::ResponseFixtureBindingMismatch,
    );

    let mut digest = Scenario::valid();
    digest.recorded.fixture_digest = ConfidentialFixtureDigest::new([0x00; 32]);
    assert_eq!(
        digest.refusal(),
        ConfidentialFundingRefusal::ResponseFixtureBindingMismatch,
    );

    let mut profiles = Scenario::valid();
    profiles.recorded.profiles.reproducibility_contract =
        ReproducibilityContract::RecordedRandomness;
    assert_eq!(
        profiles.refusal(),
        ConfidentialFundingRefusal::ResponseProfileBindingMismatch,
        "a record claiming another contract than the request selected is refused",
    );
}

#[test]
fn a_refusal_carrying_a_funded_observation_is_refused() {
    let mut scenario = Scenario::valid();
    scenario.response.observed_layer = ObservedOutcomeLayer::ConsensusRejectionBeforeScript;
    assert_eq!(
        scenario.refusal(),
        ConfidentialFundingRefusal::RefusalCarriesFundedObservation,
        "a refusal that carried outpoints and mined bytes is a verdict wearing a refusal's name",
    );

    // The other half: a target refusal with nothing to show for it is
    // an ordinary negative answer.
    let mut honest = Scenario::valid();
    honest.response.observed_layer = ObservedOutcomeLayer::ConsensusRejectionBeforeScript;
    honest.response.confidential_funded_outputs.clear();
    honest.response.mined_readback = None;
    assert_eq!(
        honest.bind(),
        Ok(ConfidentialFundingObservation::NoFundedObservation),
    );
}

#[test]
fn a_materializer_that_refused_is_typed_by_the_layer_it_refused_at() {
    for cause in [
        ObservedOutcomeLayer::FixtureConstructionFailure,
        ObservedOutcomeLayer::ExecutorInfrastructureFailure,
    ] {
        let mut scenario = Scenario::valid();
        scenario.response.observed_layer = cause;
        scenario.response.confidential_funded_outputs.clear();
        scenario.response.mined_readback = None;
        scenario.response.observed_detail = Some("the materializer refused".to_owned());
        assert_eq!(
            scenario.refusal(),
            ConfidentialFundingRefusal::DeterministicMaterializationRefused { cause },
            "a step that did not happen is not a verdict about anything",
        );
    }
}

#[test]
fn an_absent_readback_and_undecodable_bytes_are_told_apart() {
    let mut absent = Scenario::valid();
    absent.response.mined_readback = None;
    assert_eq!(
        absent.refusal(),
        ConfidentialFundingRefusal::MinedReadbackMismatch {
            fact: MinedReadbackFact::ReadbackAbsent,
            index: None,
        },
    );

    let mut undecodable = Scenario::valid();
    undecodable.decodes = false;
    assert_eq!(
        undecodable.refusal(),
        ConfidentialFundingRefusal::RawTransactionDecodeFailed,
    );
}

#[test]
fn a_request_echo_cannot_replace_readback_for_any_output_fact() {
    // Each case leaves the response's own report untouched and changes
    // what the chain holds. A binding that read the echo would see
    // nothing wrong with any of them.
    let mut identity = Scenario::valid();
    identity.decoded.transaction_id = WTXID.to_owned();
    assert_eq!(
        identity.refusal(),
        ConfidentialFundingRefusal::MinedReadbackMismatch {
            fact: MinedReadbackFact::TransactionIdentity,
            index: None,
        },
    );

    let mut witness = Scenario::valid();
    witness.decoded.witness_transaction_id = TXID.to_owned();
    assert_eq!(
        witness.refusal(),
        ConfidentialFundingRefusal::MinedReadbackMismatch {
            fact: MinedReadbackFact::WitnessTransactionIdentity,
            index: None,
        },
    );

    let mut commitment = Scenario::valid();
    commitment.decoded.outputs[1].value = DecodedValueField::Commitment(point(7, 0x44));
    assert_eq!(
        commitment.refusal(),
        ConfidentialFundingRefusal::MinedReadbackMismatch {
            fact: MinedReadbackFact::ValueCommitment,
            index: Some(1),
        },
    );

    let mut nonce_field = Scenario::valid();
    nonce_field.decoded.outputs[0].nonce = nonce(0xee);
    assert_eq!(
        nonce_field.refusal(),
        ConfidentialFundingRefusal::MinedReadbackMismatch {
            fact: MinedReadbackFact::NonceField,
            index: Some(0),
        },
    );

    let mut proof = Scenario::valid();
    proof.decoded.outputs[0].rangeproof = vec![0x44; 64];
    assert_eq!(
        proof.refusal(),
        ConfidentialFundingRefusal::MinedReadbackMismatch {
            fact: MinedReadbackFact::RangeproofBytes,
            index: Some(0),
        },
    );
}

#[test]
fn a_script_rendering_that_disagrees_with_the_decoded_program_refuses() {
    let mut scenario = Scenario::valid();
    scenario.response.confidential_funded_outputs[1].script = hex(&[0x51, 0x20, 0x00, 0x00]);
    assert_eq!(
        scenario.refusal(),
        ConfidentialFundingRefusal::MinedReadbackMismatch {
            fact: MinedReadbackFact::OutputScript,
            index: Some(1),
        },
    );
}

#[test]
fn an_outpoint_naming_another_transaction_or_witness_entry_refuses() {
    let mut foreign = Scenario::valid();
    foreign.response.confidential_funded_outputs[0]
        .outpoint
        .txid = BLOCK.to_owned();
    assert_eq!(
        foreign.refusal(),
        ConfidentialFundingRefusal::MinedReadbackMismatch {
            fact: MinedReadbackFact::OutpointTransaction,
            index: Some(0),
        },
    );

    let mut witness_index = Scenario::valid();
    witness_index.response.confidential_funded_outputs[0].output_witness_index = 1;
    assert_eq!(
        witness_index.refusal(),
        ConfidentialFundingRefusal::MinedReadbackMismatch {
            fact: MinedReadbackFact::OutputWitnessIndex,
            index: Some(0),
        },
    );

    // An output index naming no decoded output at all is the same
    // question asked of the bytes rather than of the record.
    let mut beyond = Scenario::valid();
    beyond.response.confidential_funded_outputs[1].outpoint.vout = 9;
    beyond.response.confidential_funded_outputs[1].output_witness_index = 9;
    assert_eq!(
        beyond.refusal(),
        ConfidentialFundingRefusal::MinedReadbackMismatch {
            fact: MinedReadbackFact::OutputWitnessIndex,
            index: Some(1),
        },
    );
}

#[test]
fn the_block_the_answer_names_is_checked_independently() {
    let mut scenario = Scenario::valid();
    scenario.included = false;
    assert_eq!(
        scenario.refusal(),
        ConfidentialFundingRefusal::MinedInclusionUnverified,
    );
}

#[test]
fn a_count_or_a_repeated_coin_refuses() {
    let mut short = Scenario::valid();
    short.response.confidential_funded_outputs.pop();
    assert_eq!(
        short.refusal(),
        ConfidentialFundingRefusal::OutputCountMismatch {
            requested: 2,
            observed: 1,
        },
    );

    let mut twice = Scenario::valid();
    let first = twice.response.confidential_funded_outputs[0].clone();
    twice.response.confidential_funded_outputs[1] = first.clone();
    assert_eq!(
        twice.refusal(),
        ConfidentialFundingRefusal::DuplicateFundedOutpoint {
            outpoint: first.outpoint,
        },
    );
}

#[test]
fn a_program_or_an_asset_the_request_did_not_state_refuses() {
    let mut program = Scenario::valid();
    program.decoded.outputs[0].program = vec![0x51, 0x20, 0x00, 0x00];
    assert_eq!(
        program.refusal(),
        ConfidentialFundingRefusal::OutputProgramMismatch { index: 0 },
    );

    let mut asset = Scenario::valid();
    asset.decoded.outputs[1].asset = DecodedAssetField::Explicit("bb".repeat(32));
    assert_eq!(
        asset.refusal(),
        ConfidentialFundingRefusal::ExplicitAssetMismatch { index: 1 },
    );

    // A step that neither states an asset nor reports issuing one has
    // named no protocol asset at all.
    let mut unnamed = Scenario::valid();
    unnamed.subject.asset = None;
    unnamed.subject.issue_asset = true;
    assert_eq!(
        unnamed.refusal(),
        ConfidentialFundingRefusal::ExplicitAssetMismatch { index: 0 },
    );
}

#[test]
fn the_two_halves_of_the_hybrid_tuple_are_refused_apart() {
    // The two variants are separate because a run that confused them
    // would have learned the opposite of what it reported.
    let mut committed_asset = Scenario::valid();
    committed_asset.decoded.outputs[0].asset = DecodedAssetField::Commitment(point(5, 0x22));
    assert_eq!(
        committed_asset.refusal(),
        ConfidentialFundingRefusal::ConfidentialAssetReturned { index: 0 },
    );

    let mut explicit_value = Scenario::valid();
    explicit_value.decoded.outputs[1].value = DecodedValueField::Explicit(1_000);
    assert_eq!(
        explicit_value.refusal(),
        ConfidentialFundingRefusal::ExplicitValueReturned { index: 1 },
    );
}

#[test]
fn an_inadmissible_commitment_or_nonce_encoding_refuses() {
    let mut short = Scenario::valid();
    short.response.confidential_funded_outputs[0]
        .value_commitment
        .truncate(32);
    assert_eq!(
        short.refusal(),
        ConfidentialFundingRefusal::CommitmentEncodingInvalid { index: 0 },
    );

    let mut off_curve = Scenario::valid();
    let mut bogus = vec![0x08_u8; 33];
    bogus[1] = 0xff;
    off_curve.response.confidential_funded_outputs[0].value_commitment = bogus.clone();
    off_curve.decoded.outputs[0].value = DecodedValueField::Commitment(bogus);
    assert_eq!(
        off_curve.refusal(),
        ConfidentialFundingRefusal::CommitmentEncodingInvalid { index: 0 },
        "a prefix the encoding admits is not on its own a point",
    );

    let mut null_nonce = Scenario::valid();
    null_nonce.decoded.outputs[0].nonce = Vec::new();
    assert_eq!(
        null_nonce.refusal(),
        ConfidentialFundingRefusal::NonceEncodingInvalid { index: 0 },
        "an absent nonce field carries no nonce for anything to be derived against",
    );

    let mut wrong_prefix = Scenario::valid();
    let mut field = nonce(0);
    field[0] = 0x0a;
    wrong_prefix.response.confidential_funded_outputs[0].nonce = field.clone();
    wrong_prefix.decoded.outputs[0].nonce = field;
    assert_eq!(
        wrong_prefix.refusal(),
        ConfidentialFundingRefusal::NonceEncodingInvalid { index: 0 },
    );
}

#[test]
fn the_proofs_the_representation_fixes_are_each_refused_in_their_own_way() {
    let mut empty = Scenario::valid();
    empty.response.confidential_funded_outputs[0]
        .rangeproof
        .clear();
    assert_eq!(
        empty.refusal(),
        ConfidentialFundingRefusal::RangeproofMissing { index: 0 },
    );

    let mut invalid = Scenario::valid();
    invalid.proofs_verify = false;
    assert_eq!(
        invalid.refusal(),
        ConfidentialFundingRefusal::RangeproofInvalid { index: 0 },
    );

    let mut surjection = Scenario::valid();
    surjection.decoded.outputs[1].surjection_proof = vec![0x01];
    assert_eq!(
        surjection.refusal(),
        ConfidentialFundingRefusal::SurjectionProofUnexpected { index: 1 },
    );

    let mut reported_surjection = Scenario::valid();
    reported_surjection.response.confidential_funded_outputs[0].surjection_proof = vec![0x01];
    assert_eq!(
        reported_surjection.refusal(),
        ConfidentialFundingRefusal::SurjectionProofUnexpected { index: 0 },
    );
}

#[test]
fn every_refusal_renders_a_line_of_its_own() {
    // A vocabulary whose members render alike is a vocabulary a report
    // cannot tell apart. Nothing here reads a target; what is held is
    // that the words differ.
    let subject = confidential_subject();
    let rendered = [
        ConfidentialFundingRefusal::ProtocolSchemaUnsupported {
            required: 5,
            offered: 4,
        },
        ConfidentialFundingRefusal::ConfidentialFundingCapabilityAbsent,
        ConfidentialFundingRefusal::CapabilityAdvertisementDisagrees,
        ConfidentialFundingRefusal::HybridRepresentationUnsupported,
        ConfidentialFundingRefusal::UnknownRepresentationTag {
            tag: "unassigned".to_owned(),
        },
        ConfidentialFundingRefusal::FundingProfileUnsupported {
            kind: FundingProfileKind::Representation,
            name: "unknown".to_owned(),
        },
        ConfidentialFundingRefusal::UnknownWireMember {
            record: ConfidentialWireRecord::Subject,
            member: "commentary".to_owned(),
        },
        ConfidentialFundingRefusal::UnknownFixtureHandle {
            handle: subject.binding.fixture_handle.clone(),
        },
        ConfidentialFundingRefusal::FixtureDigestMismatch {
            handle: subject.binding.fixture_handle.clone(),
        },
        ConfidentialFundingRefusal::DestinationSetEmpty,
        ConfidentialFundingRefusal::DeterministicMaterializationRefused {
            cause: ObservedOutcomeLayer::FixtureConstructionFailure,
        },
        ConfidentialFundingRefusal::ResponseArmMismatch {
            requested: OperationStepKind::FundConfidential,
            returned: OperationStepKind::Fund,
        },
        ConfidentialFundingRefusal::ResponseFixtureBindingMismatch,
        ConfidentialFundingRefusal::ResponseProfileBindingMismatch,
        ConfidentialFundingRefusal::OutputCountMismatch {
            requested: 2,
            observed: 1,
        },
        ConfidentialFundingRefusal::OutputProgramMismatch { index: 0 },
        ConfidentialFundingRefusal::ExplicitAssetMismatch { index: 0 },
        ConfidentialFundingRefusal::ConfidentialAssetReturned { index: 0 },
        ConfidentialFundingRefusal::ExplicitValueReturned { index: 0 },
        ConfidentialFundingRefusal::CommitmentEncodingInvalid { index: 0 },
        ConfidentialFundingRefusal::NonceEncodingInvalid { index: 0 },
        ConfidentialFundingRefusal::RangeproofMissing { index: 0 },
        ConfidentialFundingRefusal::RangeproofInvalid { index: 0 },
        ConfidentialFundingRefusal::SurjectionProofUnexpected { index: 0 },
        ConfidentialFundingRefusal::DuplicateFundedOutpoint {
            outpoint: WireOutpoint {
                txid: TXID.to_owned(),
                vout: 0,
            },
        },
        ConfidentialFundingRefusal::RawTransactionDecodeFailed,
        ConfidentialFundingRefusal::MinedInclusionUnverified,
        ConfidentialFundingRefusal::MinedReadbackMismatch {
            fact: MinedReadbackFact::TransactionIdentity,
            index: None,
        },
        ConfidentialFundingRefusal::RefusalCarriesFundedObservation,
    ];
    let lines: std::collections::BTreeSet<String> =
        rendered.iter().map(ToString::to_string).collect();
    assert_eq!(
        lines.len(),
        rendered.len(),
        "two refusals render the same line",
    );
    for line in &lines {
        assert!(!line.is_empty(), "a refusal rendered nothing");
    }
}

#[test]
fn the_arm_the_capability_and_the_step_kind_correspond_in_one_place() {
    let subject = OperationSubject::ConfidentialFunding(Box::new(confidential_subject()));
    assert_eq!(subject.kind(), OperationStepKind::FundConfidential);
    assert_eq!(
        subject.required_capability(),
        ExecutorCapability::ConfidentialValueTestFunding,
    );

    // The gate is per step: an executor holding every other operation
    // capability is still refused this one.
    let mut executor: ExecutorHandshake = nonmock_handshake();
    executor
        .capabilities
        .insert(ExecutorCapability::TestFundingCeremony);
    executor
        .capabilities
        .insert(ExecutorCapability::TargetTransactionSubmission);
    assert!(!executor.runs_operation_step(&subject));
    assert!(confidential_handshake().runs_operation_step(&subject));
}

#[test]
fn the_advertised_representation_is_the_hybrid_one_and_nothing_wider() {
    let advertisement = confidential_advertisement();
    assert_eq!(
        advertisement
            .representation_profiles
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        vec![FundingRepresentationProfile::ExplicitAssetConfidentialValue],
    );
    assert!(
        advertisement
            .reproducibility_contracts
            .contains(&ReproducibilityContract::ByteIdentity),
        "the reference contract is the one the test ceremony runs under",
    );
}

#[test]
fn the_confidential_records_round_trip_and_refuse_an_undeclared_member() {
    let subject = confidential_subject();
    let text = serde_json::to_string(&subject).expect("the subject serializes");
    assert_eq!(
        serde_json::from_str::<TargetConfidentialFundingSubject>(&text).expect("it parses"),
        subject,
    );

    let advertisement = confidential_advertisement();
    let text = serde_json::to_string(&advertisement).expect("the advertisement serializes");
    assert_eq!(
        serde_json::from_str::<crate::protocol::ConfidentialFundingAdvertisement>(&text)
            .expect("it parses"),
        advertisement,
    );

    let mut value = serde_json::to_value(&advertisement).expect("the advertisement serializes");
    value
        .as_object_mut()
        .expect("an advertisement is an object")
        .insert("commentary".to_owned(), Value::from("extra"));
    assert!(
        serde_json::from_value::<crate::protocol::ConfidentialFundingAdvertisement>(value).is_err(),
        "an undeclared member must be refused rather than ignored",
    );

    let destination = ConfidentialFundingDestination {
        output_program: TEST_DESTINATION_PROGRAMS[0].to_vec(),
    };
    let text = serde_json::to_string(&destination).expect("the destination serializes");
    assert_eq!(
        serde_json::from_str::<ConfidentialFundingDestination>(&text).expect("it parses"),
        destination,
    );
}

#[test]
fn the_reproducibility_contract_travels_as_the_contract_s_own_code() {
    // One word for the contract in the whole workspace: the wire reads
    // and writes the vocabulary's own code rather than a second
    // spelling of it.
    let subject = confidential_subject();
    let value = serde_json::to_value(&subject).expect("the subject serializes");
    let carried = value
        .get("binding")
        .and_then(|binding| binding.get("profiles"))
        .and_then(|profiles| profiles.get("reproducibility_contract"))
        .and_then(Value::as_str)
        .expect("the profiles carry the contract");
    assert_eq!(carried, ReproducibilityContract::ByteIdentity.code());
}
