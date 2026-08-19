//! `emit-normalization-matrix`: the canonical §10.4 matrix, as JSON.
//!
//! The matrix and the claim are owned by
//! [`target_elements_conformance::normalization`], and this command is
//! the only way anything outside the crate obtains them — the same
//! reasoning `emit-conservation-matrix` rests on: a runner that restated
//! the claim in another language would drift from it, and a drifted
//! claim is undetectable exactly where it matters, because a wrong
//! amount still looks like an amount.
//!
//! # It emits the rows, and separately the requests
//!
//! `rows` carries the whole matrix, expectations and reasoning included,
//! for a reader and for the report. `requests` carries what may be sent
//! to an executor: the row identity and its subject, and no expected
//! layer `(´[PLAN-rule:guide11-exec:request-subject]´)`.
//!
//! The separation matters more here than for a conservation row. Three
//! of these rows are answered by the report layer rather than by the
//! target, and an executor that knew which ones could report a
//! disagreement it never observed.

use std::io::Write;

use target_elements_conformance::normalization::{
    NormalizationClaim, NormalizationSubject, canonical_mutation_matrix,
};
use target_elements_conformance::protocol::{NATIVE_PROTOCOL_SCHEMA, NormalizationCaseId};

fn main() {
    let matrix = canonical_mutation_matrix();
    let claim = NormalizationClaim::canonical();
    assert!(
        claim.conserves(),
        "the canonical claim conserves the value it consumes"
    );

    let requests: Vec<_> = matrix
        .iter()
        .map(|row| {
            let spelling = serde_json::to_value(row.mutation)
                .expect("a mutation serializes")
                .as_str()
                .expect("a mutation is a string")
                .to_owned();
            serde_json::json!({
                "schema": NATIVE_PROTOCOL_SCHEMA,
                "case": NormalizationCaseId { normalization: spelling },
                "subject": NormalizationSubject {
                    claim: claim.clone(),
                    mutation: row.mutation,
                },
            })
        })
        .collect();

    let document = serde_json::json!({
        "schema": NATIVE_PROTOCOL_SCHEMA,
        "claim": claim,
        "rows": matrix,
        "requests": requests,
    });

    let rendered = serde_json::to_string_pretty(&document).expect("the matrix serializes");
    let mut out = std::io::stdout();
    out.write_all(rendered.as_bytes())
        .and_then(|()| out.write_all(b"\n"))
        .expect("stdout accepts the matrix");
}
