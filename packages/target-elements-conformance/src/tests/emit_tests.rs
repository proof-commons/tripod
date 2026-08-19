//! The emitted documents, checked without spawning a command.
//!
//! These exist because of `G12-R03`. The four `emit-*` commands built
//! their documents inside `main`, so nothing in this crate could observe
//! what they published; the row's disposition said as much, and so did
//! `G12-R05`'s. The library seam is what makes these ordinary tests
//! possible, and the subprocess contract — exit classes, JSON-only
//! stderr, the terminal refusal — is a separate question the Wave-2
//! harness asks of the binaries themselves.

use crate::emit::{
    conservation_matrix_document, lifecycle_report_document, normalization_matrix_document,
    normalization_report_document,
};
use crate::protocol::NATIVE_PROTOCOL_SCHEMA;

/// The conservation matrix states this revision and separates its shapes.
///
/// The separation is the property worth pinning: a request carries the
/// row identity and the subject and no expected layer, so an executor
/// receiving one has nothing to echo.
#[test]
fn the_conservation_matrix_keeps_the_expectation_out_of_its_requests() {
    let document = conservation_matrix_document();
    assert_eq!(document["schema"], NATIVE_PROTOCOL_SCHEMA);

    let requests = document["requests"]
        .as_array()
        .expect("the document carries requests");
    assert!(!requests.is_empty(), "the matrix asks the target something");
    for request in requests {
        let members: Vec<_> = request
            .as_object()
            .expect("a request is an object")
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            members,
            vec!["case", "schema", "subject"],
            "a request carries the identity and the subject, and nothing else",
        );
    }

    // Every executed row is asked, and no deferred row is: a row this
    // wave cannot materialize is not put to a target.
    let rows = document["rows"]
        .as_array()
        .expect("the document carries rows");
    let executed = rows.iter().filter(|row| row["deferral"].is_null()).count();
    assert_eq!(requests.len(), executed);
}

/// The normalization matrix states its claim and asks it unmutated once.
#[test]
fn the_normalization_matrix_states_a_conserving_claim() {
    let document = normalization_matrix_document().expect("the canonical claim conserves");
    assert_eq!(document["schema"], NATIVE_PROTOCOL_SCHEMA);

    let requests = document["requests"]
        .as_array()
        .expect("the document carries requests");
    let unmutated = requests
        .iter()
        .filter(|request| request["subject"]["mutation"] == "none")
        .count();
    assert_eq!(
        unmutated, 1,
        "exactly one row is the claim as stated, which is the row the \
         lifecycle lane publishes from",
    );
    for request in requests {
        assert!(
            request["subject"].get("expected").is_none(),
            "no expectation crosses the request boundary",
        );
    }
}

/// A normalization run that is not a census of the matrix is refused.
///
/// The report is a statement about the canonical matrix, so a record
/// answering none of it cannot produce a partial report — it produces no
/// report.
#[test]
fn a_normalization_report_refuses_a_record_that_is_not_a_census() {
    let empty = serde_json::json!({"responses": []});
    let refusal = normalization_report_document(&empty)
        .expect_err("a record answering nothing is not a census");
    assert!(
        refusal.contains("census"),
        "the refusal names what is wrong: {refusal}",
    );

    let wrong_shape = serde_json::json!({"responses": [{"not": "a response"}]});
    assert!(
        normalization_report_document(&wrong_shape).is_err(),
        "a response this protocol does not define is refused",
    );
}

/// A lifecycle run whose record is not a public handoff is refused.
#[test]
fn a_lifecycle_report_refuses_a_record_without_a_handoff() {
    let empty = serde_json::json!({});
    let refusal = lifecycle_report_document(&empty)
        .expect_err("a record carrying no handoff builds no report");
    assert!(
        refusal.contains("public handoff"),
        "the refusal names what is wrong: {refusal}",
    );
}

/// A lifecycle check that is not a protocol record is refused, not defaulted.
///
/// The reading this replaced defaulted an absent check name to the empty
/// string and an absent agreement to `false`, so a malformed record
/// produced a report of well-formed checks nobody had made. A check is
/// evidence, and a defaulted one is evidence of nothing.
#[test]
fn a_lifecycle_report_refuses_a_malformed_check() {
    let record = serde_json::json!({
        "process_a": {"subject": {"handoff": handoff_wire(), "authorization_profile": null}},
        "process_b_runs": [{
            "attempt": 1,
            "pid": 2,
            "wallet_name": "a-wallet",
            "rows": [{
                "row": "accepted",
                "observed_outcome": "verified",
                // Neither a name nor an agreement: the shape the old
                // reading turned into a check that read as made.
                "checks": [{"expected": "aa", "observed": "aa"}],
            }],
        }],
        "stale_row": {"unbuilt_reason": null},
        "destruction": {"scope": "the creator's wallet"},
        "environment": {"genesis_id": "bb", "network_id": "aa", "handshake": {}},
    });
    let refusal =
        lifecycle_report_document(&record).expect_err("a malformed check builds no report");
    assert!(
        refusal.contains("check"),
        "the refusal names the check it could not read: {refusal}",
    );
}

/// One published record, in the adapter's own wire spelling.
fn handoff_wire() -> serde_json::Value {
    serde_json::json!({
        "schema": crate::lifecycle::HANDOFF_SCHEMA,
        "chain_name": "elementsregtest",
        "network_id": "aa",
        "genesis_id": "bb",
        "txid": "cc",
        "output_index": 0,
        "block_hash": "dd",
        "block_height": 101,
        "raw_transaction": "00",
        "claimed_explicit_amount": 100_000_u64,
        "claimed_explicit_asset": "ee",
        "claimed_owner_address": "an-address",
    })
}
