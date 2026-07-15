//! Model-register label discipline.
//!
//! Implements `´test:verification:model-label-register´`: every
//! acute-delimited label token in the model source is three lowercase
//! kebab-case segments over a declared type vocabulary; tokens whose
//! type is document-owned are citations and must resolve verbatim in
//! the realization document; tokens whose type is model-owned are
//! definitions-by-use, indexed at `generated/model_labels.json` so the
//! register has a single reviewable definition site.
//!
//! The harvest and rendering live in [`crate::artifacts`]; the index
//! freshness test is a pure check — regeneration is `generate-all`'s
//! job, never a test side effect.

use std::collections::BTreeSet;

use crate::artifacts::{
    DOCUMENT_TYPES, MODEL_TYPES, harvest_labels, model_labels_json, package_root, split_label,
};

const DOC_RELATIVE_PATH: &str = "../../docs/attestation/realization.md";

const INDEX_RELATIVE_PATH: &str = "generated/model_labels.json";

#[test]
fn every_model_label_is_a_well_formed_triple_over_the_type_registry() {
    for label in harvest_labels() {
        let segments = split_label(&label);

        assert_eq!(segments.len(), 3, "label {label} is not a triple");

        assert!(
            MODEL_TYPES.contains(&segments[0]) || DOCUMENT_TYPES.contains(&segments[0]),
            "label {label} has unknown type prefix",
        );

        for segment in &segments[1..] {
            assert!(
                !segment.is_empty()
                    && segment
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
                "label {label} has a malformed segment",
            );
        }
    }
}

#[test]
fn document_typed_labels_resolve_in_the_realization_document() {
    let cites = harvest_labels()
        .into_iter()
        .filter(|label| DOCUMENT_TYPES.contains(&split_label(label)[0]))
        .collect::<BTreeSet<_>>();

    if cites.is_empty() {
        return;
    }

    // A dangling cite is a failure: the code is citing the document.
    let document = std::fs::read_to_string(package_root().join(DOC_RELATIVE_PATH))
        .expect("realization.md must exist once model comments cite document labels");

    for label in cites {
        assert!(
            document.contains(&label),
            "model comment cites {label}, which the realization document does not define",
        );
    }
}

/// The committed label index equals the in-memory harvest. Pure check:
/// the test never writes; a stale index is regenerated with
/// `cargo run -p tripod-artifacts --bin generate-all`.
#[test]
fn model_label_index_is_current() {
    let rendered = model_labels_json();

    let index_path = package_root().join(INDEX_RELATIVE_PATH);
    let committed = std::fs::read_to_string(&index_path).unwrap_or_default();

    assert_eq!(
        committed, rendered,
        "generated/model_labels.json is stale; run \
         `cargo run -p tripod-artifacts --bin generate-all` \
         and commit the diff",
    );
}

// Delimiter strictness (B0-012): the scanner rejects malformed
// delimiter structure instead of silently dropping tokens.

#[test]
#[should_panic(expected = "unmatched label delimiter at fixture.rs:2")]
fn unmatched_opening_delimiter_is_rejected() {
    let mut labels = std::collections::BTreeSet::new();
    crate::artifacts::scan_labels(
        "fixture.rs",
        "// fine line\n// cites \u{b4}rule:verification:important-check with no closer\n",
        &mut labels,
    );
}

#[test]
#[should_panic(expected = "unmatched label delimiter")]
fn odd_delimiter_count_is_rejected() {
    let mut labels = std::collections::BTreeSet::new();
    crate::artifacts::scan_labels(
        "fixture.rs",
        "// \u{b4}def:a:b\u{b4} then a stray \u{b4} delimiter\n",
        &mut labels,
    );
}

#[test]
fn multiple_labels_on_one_line_are_all_harvested() {
    let mut labels = std::collections::BTreeSet::new();
    crate::artifacts::scan_labels(
        "fixture.rs",
        "// \u{b4}def:a:b\u{b4} and \u{b4}rule:c:d\u{b4}\n// no labels here\n",
        &mut labels,
    );

    assert_eq!(
        labels.into_iter().collect::<Vec<_>>(),
        vec!["def:a:b".to_owned(), "rule:c:d".to_owned()],
    );
}
