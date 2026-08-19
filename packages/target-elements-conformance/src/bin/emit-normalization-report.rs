//! `emit-normalization-report`: the §14 safety report, from a run record.
//!
//! # Why the judgement happens here and not in the runner
//!
//! Three §10.4 rows are refused by the report layer: the target accepts
//! the transaction and the claim does not. Deciding that means comparing
//! the claim with the transaction, and the claim lives in this crate. A
//! runner that made the comparison would be a second implementation of
//! the claim, free to drift from the first — and a drifted claim agrees
//! with whatever it is compared against.
//!
//! So the runner is transport: it records the adapter's responses
//! verbatim. This command reads that record, rebuilds the matrix and its
//! expectations from [`target_elements_conformance::normalization`], and
//! produces the typed report. The expectations are never read from the
//! run record, so a run cannot supply the answer it is checked against.

use std::collections::BTreeMap;
use std::io::{Read as _, Write as _};

use target_elements_conformance::conservation_report::ConservationReportRole;
use target_elements_conformance::declassification::normalization_declassifications;
use target_elements_conformance::normalization::{
    NormalizationClaim, NormalizationMutation, canonical_mutation_matrix,
};
use target_elements_conformance::normalization_report::{NormalizationReport, outcome_of};
use target_elements_conformance::protocol::NativeNormalizationResponse;

fn main() {
    let mut arguments = std::env::args().skip(1);
    let Some(path) = arguments.next() else {
        eprintln!("usage: emit-normalization-report RUN-RECORD");
        std::process::exit(2);
    };

    let mut text = String::new();
    std::fs::File::open(&path)
        .and_then(|mut file| file.read_to_string(&mut text))
        .unwrap_or_else(|error| {
            eprintln!("the run record at {path} could not be read: {error}");
            std::process::exit(1);
        });

    let record: serde_json::Value = serde_json::from_str(&text).unwrap_or_else(|error| {
        eprintln!("the run record is not JSON: {error}");
        std::process::exit(1);
    });

    let responses: Vec<NativeNormalizationResponse> =
        serde_json::from_value(record["responses"].clone()).unwrap_or_else(|error| {
            eprintln!("the run record's responses are not this protocol's: {error}");
            std::process::exit(1);
        });

    // Indexed by the mutation each response answers, so a run that
    // answered rows out of order, or skipped one, is visible as a missing
    // row rather than as a silent misalignment with the matrix.
    let mut answered: BTreeMap<String, NativeNormalizationResponse> = BTreeMap::new();
    for response in responses {
        response.validate_shape().unwrap_or_else(|defect| {
            eprintln!(
                "a response for {} contradicts itself: {defect:?}",
                response.case.normalization
            );
            std::process::exit(1);
        });
        answered.insert(response.case.normalization.clone(), response);
    }

    let mut rows = Vec::new();
    let mut profile = None;
    for row in canonical_mutation_matrix() {
        let spelling = wire_spelling(row.mutation);
        let Some(response) = answered.get(&spelling) else {
            eprintln!("the run answered no row for {spelling}");
            std::process::exit(1);
        };
        let outcome = outcome_of(row.mutation, row.expected, response);
        // The profile the run observed, taken from the unmutated row: it
        // is the one row whose signature was never disturbed, so it is
        // the only one whose witness reports the profile the owner
        // actually authorized under.
        if row.mutation == NormalizationMutation::None {
            profile = outcome.authorization_profile;
        }
        rows.push(outcome);
    }

    let report = NormalizationReport {
        role: ConservationReportRole::Experimental,
        claim: NormalizationClaim::canonical(),
        disclosures: normalization_declassifications(),
        authorization_profile: profile,
        observed_genesis: string_at(&record, "environment", "genesis_id"),
        observed_network: string_at(&record, "environment", "network_id"),
        declared_tip: optional_string_at(&record, "handshake", "intended_executed_tip"),
        binary_reported_revision: optional_string_at(
            &record,
            "handshake",
            "binary_reported_revision",
        ),
        rows,
    };

    let rendered = serde_json::to_string_pretty(&report).expect("the report serializes");
    let mut out = std::io::stdout();
    out.write_all(rendered.as_bytes())
        .and_then(|()| out.write_all(b"\n"))
        .expect("stdout accepts the report");
}

fn wire_spelling(mutation: NormalizationMutation) -> String {
    serde_json::to_value(mutation)
        .expect("a mutation serializes")
        .as_str()
        .expect("a mutation is a string")
        .to_owned()
}

fn string_at(record: &serde_json::Value, section: &str, field: &str) -> String {
    record[section][field]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

fn optional_string_at(record: &serde_json::Value, section: &str, field: &str) -> Option<String> {
    record[section][field].as_str().map(ToOwned::to_owned)
}
