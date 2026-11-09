//! Test-only admission of the compressed original scaling capture.
//!
//! One bounded decompression and one full replay are shared by all tests. The
//! decoder is a dev-dependency; no compression API or codec enters the library.
//! Local refusal tests call the same validators used by complete admission,
//! changing only one retained response or claim, without replaying 7,861 steps
//! for each mutant. Immutable acquisition pins precede semantic interpretation.

use std::collections::BTreeSet;
use std::io::{Read, Write as _};
use std::sync::OnceLock;
use std::time::Instant;

use serde_json::Value;
use target_elements::reviewed_elements_tapscript;
use target_elements_conformance::constructor::tagged::sha256;
use target_elements_conformance::protocol::{NativeOperationResponse, OperationSubject};
use transaction::bytes::TargetTransaction;

use crate::keyed_birth::Wave1TargetPacket;
use crate::keyed_birth_corpus::{bind_packet, capture_record, hex, manifest_text, response_detail};
use crate::keyed_birth_native::identity;
use crate::keyed_scaling::{
    COUNTS, EXCHANGE_COUNT, INPUT_WEIGHT, SOURCE_COUNT, ScalingClass, ScalingPlanner,
    ScalingRecord, replay_scaling_capture,
};

const COMPRESSED: &[u8] =
    include_bytes!("../fixtures/keyed-scaling-run-of-record/5910802756ef.report.capture.lzma");
const COMPRESSED_PIN: &str = "2c9d4c5df9eb73446a410fa7aaf6b496309cad1df5b5c733377bcb73a1e68ca4";
const OUTPUT_PIN: &str = "e751ebaf5ea088883f9a6c5d4a9393418ca8649d95e5b045426920733ad4033f";
const MAX_OUTPUT: usize = 164_375_392;
const DECODER_MEMORY_KIB: u32 = 16 * 1024;
const TIMING: &str =
    include_str!("../fixtures/keyed-scaling-run-of-record/5910802756ef.report.timing");
const CEREMONY: &str =
    include_str!("../fixtures/keyed-scaling-run-of-record/5910802756ef.report.capture.timing");
const REPORT: &str = include_str!("../fixtures/keyed-scaling-run-of-record/RUN-REPORT");
const MANIFEST: &str = include_str!("../fixtures/keyed-scaling-run-of-record/MANIFEST.sha256");
const OTHER_PINS: [&str; 4] = [
    "399b6af5dbe1f277e2c8b0bd629bdbf3d32449c1d1830ed846661c6154ef2079",
    "88fc195b97991d3099513822e690cf5dda7172019bc15f6a61b7c19115efdbb2",
    "d0449f65a2d1aff5aabd842dd5c5a7a9268b43bf1e9219ce5b8b6bb1a5e87591",
    "7d615baaa3121a4745170137c3dcd99434a58cbcc911d4319d738232fa1b4acf",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Refusal {
    Digest,
    Bound,
    Decode,
    Framing,
    Binding,
    Dimension,
    Replay,
    Subject,
}
type Result<T> = std::result::Result<T, Refusal>;

fn require(condition: bool, refusal: Refusal) -> Result<()> {
    if condition { Ok(()) } else { Err(refusal) }
}

fn verify_digest(bytes: &[u8], pin: &str) -> Result<()> {
    require(hex(&sha256(bytes)) == pin, Refusal::Digest)
}

// Never append the probe byte beyond the limit. Decoder dictionary/probability
// storage has its own 16 MiB bound, independent of this output bound.
fn read_bounded(mut reader: impl Read, limit: usize) -> Result<Vec<u8>> {
    let mut output = Vec::with_capacity(limit.min(MAX_OUTPUT));
    let mut buffer = vec![0_u8; 64 * 1024].into_boxed_slice();
    loop {
        let available = (limit - output.len()).saturating_add(1).min(buffer.len());
        let count = reader
            .read(&mut buffer[..available])
            .map_err(|_| Refusal::Decode)?;
        if count == 0 {
            return Ok(output);
        }
        require(count <= limit - output.len(), Refusal::Bound)?;
        output.extend_from_slice(&buffer[..count]);
    }
}

fn decode(compressed: &[u8]) -> Result<String> {
    let _span = tracing::info_span!(
        "decode_scaling_archive",
        compressed_bytes = compressed.len(),
        maximum_output = MAX_OUTPUT
    )
    .entered();
    verify_digest(compressed, COMPRESSED_PIN)?;
    let mut decoder = lzma_rust2::LzmaReader::new_mem_limit(compressed, DECODER_MEMORY_KIB, None)
        .map_err(|_| Refusal::Decode)?;
    decoder.set_relaxed_end_condition(false);
    let output = read_bounded(decoder, MAX_OUTPUT)?;
    validate_decoded_output(&output)?;
    tracing::info!(
        output_bytes = output.len(),
        "scaling archive decoded and pinned"
    );
    String::from_utf8(output).map_err(|_| Refusal::Decode)
}

fn validate_decoded_output(output: &[u8]) -> Result<()> {
    require(output.len() == MAX_OUTPUT, Refusal::Bound)?;
    verify_digest(output, OUTPUT_PIN)
}

fn capture() -> Result<&'static str> {
    static CAPTURE: OnceLock<Result<String>> = OnceLock::new();
    CAPTURE
        .get_or_init(|| decode(COMPRESSED))
        .as_ref()
        .map(String::as_str)
        .map_err(|e| *e)
}

struct Admitted {
    packet: Wave1TargetPacket,
    planner: ScalingPlanner,
    claims: Vec<Value>,
}

fn admitted() -> Result<&'static Admitted> {
    static ADMITTED: OnceLock<Result<Admitted>> = OnceLock::new();
    ADMITTED
        .get_or_init(admit_run_of_record)
        .as_ref()
        .map_err(|e| *e)
}

fn admit_run_of_record() -> Result<Admitted> {
    let started = Instant::now();
    let _span = tracing::info_span!("admit_scaling_run_of_record").entered();
    let capture = capture()?;
    for (text, pin) in [TIMING, CEREMONY, REPORT, MANIFEST]
        .into_iter()
        .zip(OTHER_PINS)
    {
        verify_digest(text.as_bytes(), pin)?;
    }
    require(
        MANIFEST
            == manifest_text(&[
                ("5910802756ef.report.capture", capture),
                ("5910802756ef.report.timing", TIMING),
                ("RUN-REPORT", REPORT),
                ("5910802756ef.report.capture.timing", CEREMONY),
            ]),
        Refusal::Digest,
    )?;
    let (header, _, payload) = capture_record(capture, capture).map_err(|_| Refusal::Framing)?;
    require(
        payload["keyed_scaling_capture_schema"] == 1
            && payload["eligible_for_review"] == true
            && payload["admitted"] == false
            && payload["construction_refusal"] == "None"
            && payload["replay_refusal"] == "None",
        Refusal::Dimension,
    )?;
    let journal = &payload["wire_journal"];
    let operations = journal["operations"].as_array().ok_or(Refusal::Dimension)?;
    require(operations.len() == EXCHANGE_COUNT, Refusal::Dimension)?;
    for operation in operations {
        complete_response(&operation["response"])?;
    }
    let target = reviewed_elements_tapscript().map_err(|_| Refusal::Binding)?;
    let planner = replay_scaling_capture(&target, journal).map_err(|_| Refusal::Replay)?;
    require(planner.eligible(), Refusal::Subject)?;
    let first = planner.exchanges().first().ok_or(Refusal::Dimension)?;
    let policy = response_detail(&first.1).map_err(|_| Refusal::Dimension)?;
    let packet = bind_packet(
        header,
        &payload,
        planner.exchanges(),
        &payload,
        &policy["policy"],
    )
    .map_err(|_| Refusal::Binding)?;
    require(
        packet.executor().adapter_version == "2.1.1"
            && journal["handshake"]["adapter_version"] == "2.1.1",
        Refusal::Binding,
    )?;
    let claims = payload["report"]["records"]
        .as_array()
        .ok_or(Refusal::Dimension)?;
    require(
        claims.len() == SOURCE_COUNT + 5 && claims.len() == planner.records().len(),
        Refusal::Dimension,
    )?;
    for (record, claim) in planner.records().iter().zip(claims) {
        validate_record(&planner, record, claim)?;
    }
    validate_handles(&planner)?;
    require(payload["report"] == planner.report_json(), Refusal::Subject)?;
    let admitted = Admitted {
        packet,
        planner,
        claims: claims.clone(),
    };
    tracing::info!(
        records = admitted.planner.records().len(),
        wall_seconds = started.elapsed().as_secs_f64(),
        "scaling admission complete"
    );
    timing("shared_decompression_and_admission", started);
    Ok(admitted)
}

// The archive promises complete wire responses, including nullable/defaultable
// members. Do not let serde's compatibility defaults conceal a missing field.
fn complete_response(value: &Value) -> Result<NativeOperationResponse> {
    let object = value.as_object().ok_or(Refusal::Dimension)?;
    let fields = [
        "schema",
        "case",
        "observed_layer",
        "observed_detail",
        "issued_asset",
        "funded_outputs",
        "confidential_funded_outputs",
        "mined_readback",
        "accepted_txid",
        "sponsor_witness",
        "script_path_witness",
        "signer_public_key",
        "signed_profile",
        "signing_genesis",
        "signature_bound_to",
        "resources",
    ];
    require(
        object.len() == fields.len() && fields.iter().all(|field| object.contains_key(*field)),
        Refusal::Dimension,
    )?;
    serde_json::from_value(value.clone()).map_err(|_| Refusal::Dimension)
}

fn validate_record(planner: &ScalingPlanner, record: &ScalingRecord, claim: &Value) -> Result<()> {
    let _span = tracing::debug_span!("validate_scaling_record", row = record.row, preparation = ?record.preparation).entered();
    let n = *COUNTS.get(record.row).ok_or(Refusal::Subject)?;
    let neighbour = record.preparation.is_none() && n == 1565;
    let expected = if neighbour {
        ScalingClass::WeightPolicyRefusedBlockMined
    } else {
        ScalingClass::RelayMined
    };
    require(
        planner.classification(record) == expected
            && claim["class"] == serde_json::to_value(expected).map_err(|_| Refusal::Subject)?,
        Refusal::Subject,
    )?;
    let OperationSubject::Submission(subject) = record.submission.0.subject() else {
        return Err(Refusal::Subject);
    };
    let tx = TargetTransaction::decode(&subject.transaction_bytes).map_err(|_| Refusal::Subject)?;
    let weight = record
        .submission
        .1
        .resources
        .transaction_weight
        .ok_or(Refusal::Dimension)?;
    require(
        weight == tx.weight() && claim["node_weight"] == weight,
        Refusal::Subject,
    )?;
    let count = if record.preparation.is_some() { 0 } else { n };
    let worst = 1077 + INPUT_WEIGHT * count as u64 + if count >= 252 { 8 } else { 0 };
    require(weight <= worst, Refusal::Subject)?;
    if record.preparation.is_none() {
        require(
            weight == [0, 1331, 1587, 200_494, 400_159, 399_905][record.row],
            Refusal::Subject,
        )?;
    }
    let detail = response_detail(&record.submission.1).map_err(|_| Refusal::Dimension)?;
    if neighbour {
        require(
            detail["relay_allowed"] == false
                && detail["relay_detail"] == "tx-size"
                && detail["route"] == "direct-block"
                && detail["detail"]["template"]["member"] == false,
            Refusal::Subject,
        )?;
    } else {
        require(
            detail["relay_allowed"] == true
                && detail["route"] == "relay-and-template"
                && detail["detail"]["template"]["member"] == true,
            Refusal::Subject,
        )?;
    }
    // classify() checks exact txid/wtxid, raw mined bytes, inclusion and policy.
    let resources = &record.submission.1.resources;
    require(
        resources.peak_stack_items.is_none()
            && resources.peak_altstack_items.is_none()
            && resources.maximum_element_bytes.is_none()
            && resources.validation_budget_used.is_none(),
        Refusal::Dimension,
    )
}

fn validate_handles(planner: &ScalingPlanner) -> Result<()> {
    let _span = tracing::info_span!("validate_scaling_handle_census").entered();
    let mut prepared = BTreeSet::new();
    let mut consumed = BTreeSet::new();
    let mut all_sources = BTreeSet::new();
    let mut standalone = None;
    for record in planner.records() {
        let OperationSubject::Submission(subject) = record.submission.0.subject() else {
            return Err(Refusal::Subject);
        };
        let tx =
            TargetTransaction::decode(&subject.transaction_bytes).map_err(|_| Refusal::Subject)?;
        if record.preparation.is_some() {
            let id = identity(&tx).to_target_display();
            require(all_sources.insert(id.clone()), Refusal::Subject)?;
            if record.row == 0 {
                require(standalone.replace(id).is_none(), Refusal::Subject)?;
            } else {
                require(prepared.insert((record.row, id)), Refusal::Subject)?;
            }
        } else {
            require(
                tx.inputs().len() == COUNTS[record.row] + 1,
                Refusal::Subject,
            )?;
            for input in &tx.inputs()[..COUNTS[record.row]] {
                let outpoint = input.outpoint();
                require(
                    outpoint.index() == 0
                        && consumed.insert((record.row, outpoint.txid().to_target_display())),
                    Refusal::Subject,
                )?;
            }
        }
    }
    require(
        prepared == consumed
            && prepared.len() == SOURCE_COUNT - 1
            && standalone.is_some_and(|id| consumed.iter().all(|(_, used)| *used != id)),
        Refusal::Subject,
    )
}

fn timing(name: &str, started: Instant) {
    // Test-only output, bypassing libtest capture so the gate's retained
    // libtest stream contains successful per-test durations as well as failures.
    let _result = writeln!(
        std::io::stdout().lock(),
        "scaling-admission {name} wall={:.6}s",
        started.elapsed().as_secs_f64()
    );
}

fn timed(name: &str, test: impl FnOnce(&Admitted)) {
    let accepted = admitted().expect("accepted archive beside each refusal");
    let started = Instant::now();
    test(accepted);
    timing(name, started);
}

fn neighbour(accepted: &Admitted) -> (&ScalingRecord, &Value) {
    accepted
        .planner
        .records()
        .iter()
        .zip(&accepted.claims)
        .find(|(r, _)| r.row == 4 && r.preparation.is_none())
        .expect("neighbour census")
}

#[test]
fn exact_scaling_admission() {
    timed("exact_scaling_admission", |a| {
        assert_eq!(a.packet.executor().adapter_version, "2.1.1");
        assert_eq!(a.planner.records().len(), 3920);
        assert_eq!(a.planner.exchanges().len(), 7861);
    });
}

#[test]
fn changed_compressed_byte_refused() {
    timed("changed_compressed_byte_refused", |_| {
        let mut bytes = COMPRESSED.to_vec();
        bytes[20] ^= 1;
        assert_eq!(decode(&bytes), Err(Refusal::Digest));
    });
}

#[test]
fn changed_uncompressed_digest_refused() {
    timed("changed_uncompressed_digest_refused", |_| {
        let mut output = capture().expect("decoded").as_bytes().to_vec();
        assert_eq!(validate_decoded_output(&output), Ok(()));
        output[0] ^= 1;
        assert_eq!(validate_decoded_output(&output), Err(Refusal::Digest));
    });
}

#[test]
fn output_over_bound_refused() {
    timed("output_over_bound_refused", |_| {
        assert_eq!(
            read_bounded(capture().expect("decoded").as_bytes(), MAX_OUTPUT - 1),
            Err(Refusal::Bound)
        );
    });
}

#[test]
fn changed_neighbour_class_refused() {
    timed("changed_neighbour_class_refused", |a| {
        let (record, claim) = neighbour(a);
        let mut claim = claim.clone();
        claim["class"] = Value::String("ConsensusWeightRefused".into());
        assert_eq!(
            validate_record(&a.planner, record, &claim),
            Err(Refusal::Subject)
        );
    });
}

#[test]
fn changed_neighbour_reason_refused() {
    timed("changed_neighbour_reason_refused", |a| {
        let (record, claim) = neighbour(a);
        let mut record = record.clone();
        let mut detail = response_detail(&record.submission.1).expect("detail");
        detail["relay_detail"] = Value::String("bad-witness-nonstandard".into());
        record.submission.1.observed_detail = Some(detail.to_string());
        assert_eq!(
            validate_record(&a.planner, &record, claim),
            Err(Refusal::Subject)
        );
    });
}

#[test]
fn changed_node_weight_refused() {
    timed("changed_node_weight_refused", |a| {
        let (record, claim) = neighbour(a);
        let mut record = record.clone();
        record.submission.1.resources.transaction_weight = Some(400_160);
        assert_eq!(
            validate_record(&a.planner, &record, claim),
            Err(Refusal::Subject)
        );
    });
}

#[test]
fn missing_response_field_refused() {
    timed("missing_response_field_refused", |a| {
        let (record, _) = neighbour(a);
        let mut value = serde_json::to_value(&record.submission.1).expect("wire response");
        value
            .as_object_mut()
            .expect("object")
            .remove("sponsor_witness");
        assert_eq!(complete_response(&value), Err(Refusal::Dimension));
    });
}
