//! `emit-conservation-matrix`: the canonical §8.4 matrix, as JSON.
//!
//! The matrix is owned by
//! [`target_elements_conformance::conservation`], and this command is the
//! only way anything outside the crate obtains it. That is the point: a
//! runner that re-derived the fixture language in another language would
//! drift from it, and the drift would be invisible precisely where it
//! mattered — a blinding factor computed by a slightly different recipe
//! still looks like a blinding factor.
//!
//! # It emits the rows, and separately the requests
//!
//! `rows` carries the whole matrix, expectations included, for a reader
//! and for the report. `requests` carries what may be sent to an
//! executor: the row identity and its subject, and no expected layer
//! `(´[PLAN-rule:guide11-exec:request-subject]´)`. Two shapes rather than
//! one, so that the boundary is a property of the data rather than a
//! discipline a runner is trusted to observe.
//!
//! Deferred rows appear in `rows` and never in `requests`: a row this
//! wave established it cannot materialize is not asked of a target.

use std::io::Write;

use target_elements_conformance::conservation::canonical_conservation_matrix;
use target_elements_conformance::protocol::NATIVE_PROTOCOL_SCHEMA;

fn main() {
    let matrix = canonical_conservation_matrix();

    let requests: Vec<_> = matrix
        .iter()
        .filter(|row| row.is_executed())
        .map(|row| {
            serde_json::json!({
                "schema": NATIVE_PROTOCOL_SCHEMA,
                "case": row.id,
                "subject": row.subject(),
            })
        })
        .collect();

    let document = serde_json::json!({
        "schema": NATIVE_PROTOCOL_SCHEMA,
        "rows": matrix,
        "requests": requests,
    });

    let rendered = serde_json::to_string_pretty(&document).expect("the matrix serializes");
    let mut out = std::io::stdout();
    out.write_all(rendered.as_bytes())
        .and_then(|()| out.write_all(b"\n"))
        .expect("stdout accepts the matrix");
}
