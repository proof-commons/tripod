use crate::{
    AnnouncementLeadBounds, Cycle, Maturity, ProtocolAmount, StateMetadata, StateMetadataRefusal,
    StateRepresentationNonce, announce_maturity, decode_state_metadata, encode_state_metadata,
};

use super::tracing_tests::{assert_no_warning_or_error, capture, in_capture_child};

fn metadata() -> StateMetadata {
    StateMetadata {
        omega: ProtocolAmount::new(1).unwrap(),
        y_l: ProtocolAmount::new(2).unwrap(),
        y_t: ProtocolAmount::new(3).unwrap(),
        q: ProtocolAmount::new(4).unwrap(),
        cycle: Cycle::new(5),
        maturity: Maturity::Unannounced,
    }
}

#[test]
fn state_codec_emits_only_debug_refusal_and_preserves_bytes() {
    if !in_capture_child(
        "tests::tracing_refusal_tests::state_codec_emits_only_debug_refusal_and_preserves_bytes",
    ) {
        return;
    }
    let encoded = encode_state_metadata(&metadata(), StateRepresentationNonce::new(7));
    let plain = decode_state_metadata(&encoded);
    let (captured, records) = capture(|| decode_state_metadata(&encoded));
    assert_eq!(plain, captured);
    assert!(records.is_empty());

    let mut wrong_domain = encoded;
    wrong_domain[0] ^= 1;
    let plain = decode_state_metadata(&wrong_domain);
    let (captured, records) = capture(|| decode_state_metadata(&wrong_domain));
    assert_eq!(plain, captured);
    assert_eq!(captured, Err(StateMetadataRefusal::WrongDomain));
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].target, "realization::state_codec");
    assert_eq!(records[0].level, tracing::Level::DEBUG);
    assert_eq!(
        records[0].fields.get("message").map(String::as_str),
        Some("state metadata refused")
    );
    assert_eq!(
        records[0].fields.get("outcome").map(String::as_str),
        Some("refused")
    );
    assert_eq!(
        records[0].fields.get("refusal_kind").map(String::as_str),
        Some("wrong-domain")
    );
    assert_eq!(
        records[0].fields.get("input_bytes").map(String::as_str),
        Some("86")
    );
    assert_eq!(records[0].types.get("input_bytes"), Some(&"u64"));
    assert_eq!(records[0].types.get("refusal_kind"), Some(&"str"));
    assert_no_warning_or_error(&records);
}

#[test]
fn maturity_transition_emits_only_debug_refusal_and_preserves_successor() {
    if !in_capture_child(
        "tests::tracing_refusal_tests::maturity_transition_emits_only_debug_refusal_and_preserves_successor",
    ) {
        return;
    }
    let predecessor = metadata();
    let bounds = AnnouncementLeadBounds::new(Cycle::new(1), Cycle::new(3)).unwrap();
    let plain = announce_maturity(&predecessor, Cycle::new(6), bounds);
    let (captured, records) = capture(|| announce_maturity(&predecessor, Cycle::new(6), bounds));
    assert_eq!(plain, captured);
    assert!(records.is_empty());

    let plain = announce_maturity(&predecessor, Cycle::new(5), bounds);
    let (captured, records) = capture(|| announce_maturity(&predecessor, Cycle::new(5), bounds));
    assert_eq!(plain, captured);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].target, "realization::state");
    assert_eq!(records[0].level, tracing::Level::DEBUG);
    assert_eq!(
        records[0].fields.get("message").map(String::as_str),
        Some("maturity transition refused")
    );
    assert_eq!(
        records[0].fields.get("outcome").map(String::as_str),
        Some("refused")
    );
    assert_eq!(
        records[0].fields.get("refusal_kind").map(String::as_str),
        Some("announcement-below-minimum")
    );
    assert_no_warning_or_error(&records);
}
