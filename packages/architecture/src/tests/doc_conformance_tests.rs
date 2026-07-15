//! Doc-conformance tests: the register-1 → document weld.
//!
//! The manifest is generated; the document is hand-written. These
//! tests guard the hand-written side: every document label the
//! manifest carries MUST appear verbatim in the realization markdown.
//!
//! The v13 markdown has landed; these tests are live and gate the
//! document's label freeze.

use std::path::Path;

use crate::*;

const DOC_RELATIVE_PATH: &str = "../../docs/attestation/realization.md";

/// The document's frozen label-type prefixes. The gated equality test
/// below holds this list equal to the document's used set exactly, so
/// a type stays here only if the document defines a label with it.
/// Revision 13a mints the `def` type (`def:versioning:denotation-law`).
const TYPES: &[&str] = &[
    "sec", "app", "req", "def", "inv", "lem", "obl", "trap", "rem", "intuit", "rule", "pin", "res",
    "listing", "fig", "tab", "leaf",
];

fn doc_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(DOC_RELATIVE_PATH)
}

/// Every document label the manifest carries: witness semantic tags
/// plus the invariant-clause registry.
fn manifest_document_labels() -> Vec<&'static str> {
    let mut labels = ARCHITECTURE
        .witnesses
        .iter()
        .map(|witness| witness.semantic_tag)
        .collect::<Vec<_>>();

    labels.extend(ARCHITECTURE.clauses.iter().map(|clause| clause.as_str()));

    labels.sort_unstable();
    labels.dedup();
    labels
}

#[test]
fn manifest_labels_are_well_formed_document_labels() {
    // Shape check runs unconditionally: every label is a
    // `type:section:name` triple over the document's frozen type
    // prefixes, lowercase kebab-case segments.
    for label in manifest_document_labels() {
        let segments = label.split(':').collect::<Vec<_>>();

        assert_eq!(segments.len(), 3, "label {label} is not a triple");

        assert!(
            TYPES.contains(&segments[0]),
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
fn every_manifest_label_appears_verbatim_in_the_document() {
    let document = std::fs::read_to_string(doc_path())
        .expect("realization.md must exist once the label freeze is declared");

    for label in manifest_document_labels() {
        assert!(
            document.contains(label),
            "manifest label {label} does not appear verbatim in the realization document",
        );
    }
}

/// Every backticked `type:section:name` label token in the document
/// text, plus the two-segment-by-construction `sec:…` and `app:…`
/// anchors. Mints are bare backticked labels; internal cites wrap the
/// same code span in round brackets — both land in a backtick span, so
/// one harvest sees them all. External cites (`[A-…]`, bracketed
/// model-typed spans) carry `[` inside the code span and fail the
/// segment check.
fn document_label_types(document: &str) -> std::collections::BTreeSet<String> {
    let mut types = std::collections::BTreeSet::new();

    // Backticks pair on a single line, so odd-index chunks of a
    // per-line split are exactly the code spans. Fenced code blocks
    // are skipped (their backticks are fence syntax, not spans); on
    // every prose line an unmatched backtick is a harvest failure —
    // a mistyped label span must not silently vanish.
    let mut in_fence = false;

    for (line_number, line) in document.lines().enumerate() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }

        let chunks = line.split('`').collect::<Vec<_>>();

        assert!(
            chunks.len() % 2 == 1,
            "unmatched backtick in realization document line {}: {line:?}",
            line_number + 1,
        );

        if chunks.len() < 3 {
            continue;
        }

        for (index, token) in chunks.iter().enumerate() {
            if index % 2 == 0 {
                continue;
            }

            let segments = token.split(':').collect::<Vec<_>>();
            let segments_well_formed = segments.iter().all(|segment| {
                !segment.is_empty()
                    && segment
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            });
            // A section or appendix label names its division and may
            // name the document it divides, so it carries two or three
            // segments; every other type is a triple.
            let admissible_len =
                if segments.first() == Some(&"sec") || segments.first() == Some(&"app") {
                    segments.len() == 2 || segments.len() == 3
                } else {
                    segments.len() == 3
                };
            if admissible_len && segments_well_formed {
                types.insert(segments[0].to_owned());
            }
        }
    }

    types
}

#[test]
fn document_label_types_equal_the_frozen_type_list() {
    let document = std::fs::read_to_string(doc_path())
        .expect("realization.md must exist once the label freeze is declared");

    let used = document_label_types(&document);
    let frozen = TYPES
        .iter()
        .map(ToString::to_string)
        .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(
        used, frozen,
        "TYPES must equal the document's used set exactly: \
         trim unused types or use them before the label freeze",
    );
}

/// Every `[A-…]` cite token in the document, with the `A-` prefix
/// stripped: the Layer-0 anchor names over which the manifest's
/// `anchor_set_hash` is computed. Body and §16 index occurrences land
/// in one set; `anchor_set_hash` deduplicates and sorts internally.
fn document_anchor_names(document: &str) -> Vec<&str> {
    let mut names = Vec::new();
    let mut rest = document;

    while let Some(start) = rest.find("[A-") {
        rest = &rest[start + "[A-".len()..];

        let Some(end) = rest.find(']') else { break };
        let name = &rest[..end];

        let well_formed = !name.is_empty()
            && name.split(':').all(|segment| {
                !segment.is_empty()
                    && segment
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            });

        if well_formed {
            names.push(name);
        }

        rest = &rest[end..];
    }

    names
}

#[test]
fn pinned_anchor_set_hash_matches_the_document_harvest() {
    let document = std::fs::read_to_string(doc_path())
        .expect("realization.md must exist once the label freeze is declared");

    let names = document_anchor_names(&document);

    assert!(!names.is_empty(), "the document cites no Layer-0 anchors");

    let pinned = ARCHITECTURE
        .document
        .specification
        .anchor_set_hash
        .expect("the anchor-set hash must be pinned at release");

    assert_eq!(
        anchor_set_hash(names.iter().copied()),
        pinned,
        "the pinned Layer-0 anchor-set hash does not match the set harvested \
         from the realization document",
    );
}

// Harvester structure checks (B0-012): fenced code is skipped, and an
// unmatched inline backtick in prose is a hard failure.

#[test]
fn document_harvester_skips_fences_and_collects_prose_spans() {
    let synthetic = "\
prose citing `rem:alpha:beta` here\n\
```toml\n\
value = \"unbalanced ` backtick inside fence is fine\n\
```\n\
more prose `pin:gamma:delta` (`sec:closing`)\n";

    let types = document_label_types(synthetic);

    assert!(types.contains("rem"));
    assert!(types.contains("pin"));
    assert!(types.contains("sec"));
}

#[test]
#[should_panic(expected = "unmatched backtick in realization document line 2")]
fn document_harvester_rejects_unmatched_prose_backtick() {
    document_label_types("fine line\nbroken `rem:alpha:beta span\n");
}
