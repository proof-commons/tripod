//! Derivations behind the model's generated artifacts:
//! `generated/declassification.json` and `generated/model_labels.json`.
//!
//! This module is the single typed source for both files. The
//! `tripod-artifacts` crate's `generate-all` binary renders
//! and writes them; its `check-generated` binary and the test suite
//! compare committed bytes against these derivations without writing.
//! Tests never modify the checkout.
//!
//! # Declassification
//!
//! Implements the derivation demanded by `sec:representation` and O9
//! (`obl:oracle:disclosure`): each operation's declassification set
//! D_o is **derived from the manifest's transition read-sets, never
//! authored**. The manifest is welded to the model bidirectionally
//! (O4), so a fact enters D_o only because the transition actually
//! consumes or publishes it:
//!
//! - `read:<quantity>` — the transition reads a public quantity
//!   (D_state: floor, payout, issuance formulas re-value it);
//! - `issuance:<asset>:<condition>` — a boundary delta mints a public
//!   amount under a formula-bound condition (D_state);
//! - `destruction:<asset>:<condition>` — a boundary delta destroys a
//!   public amount (D_state / D_audit);
//! - `record:<tag>` — a tagged data output is a public payload
//!   (D_audit);
//! - `projection:<projection>` — a public event projection (D_audit);
//!   the universal `transition-certificate` projection is excluded,
//!   since it is derived history carried by every operation, not a
//!   value disclosure.
//!
//! **Lateral deltas are deliberately absent**: lateral owner-to-owner
//! movement is the private membrane (`trap:representation:two-axes`).
//! Transfer, relabel, and compact-ash therefore declassify nothing;
//! burn declassifies its record and event (the fresh-ASH aggregate),
//! not source denominations; redeem declassifies its reads and its
//! destruction; admission declassifies its principal-bound issuance.
//!
//! The committed artifact is **provisional and derivative**: it is a
//! publication of this derivation for independent readers, never a
//! semantic input. The planned compiler must receive declassification
//! from the typed realization dependency graph, not by ingesting the
//! JSON (see `plans/README.md`, "Declassification is derived, not
//! authored").
//!
//! # Model labels
//!
//! Implements `´test:verification:model-label-register´`: every
//! acute-delimited label token in the model source is three lowercase
//! kebab-case segments over a declared type vocabulary; tokens whose
//! type is document-owned are citations into the realization document;
//! tokens whose type is model-owned are definitions-by-use, indexed at
//! `generated/model_labels.json` so the register has a single
//! reviewable definition site.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use architecture::ARCHITECTURE;

/// Label types the model register owns (definitions-by-use).
///
/// `rule` and `lst` also appear in the document's type list; the
/// overlap is harmless because model labels are never cited across
/// artifacts — the namespaces are disjoint by ownership, not by
/// string.
///
/// Tie-break: acute delimiters ALWAYS mean model-owned. To cite a
/// document `rule:`/`lst:` label from code, use plain brackets
/// (checklist-guarded), never acute delimiters.
pub const MODEL_TYPES: &[&str] = &["def", "rule", "thm", "test", "branch", "protocol", "lst"];

/// Document-owned label types: an acute-delimited token with one of
/// these types is a citation into the realization document.
pub const DOCUMENT_TYPES: &[&str] = &[
    "sec", "subsec", "app", "req", "inv", "lem", "obl", "trap", "rem", "ins", "pin", "res", "fig",
    "tbl", "leaf",
];

/// The universal projection every operation carries; derived history,
/// not a value disclosure.
const UNIVERSAL_PROJECTION: &str = "transition-certificate";

/// One row of the derived declassification map.
pub struct DeclassificationRow {
    /// Operation identifier.
    pub operation: &'static str,
    /// Sorted, deduplicated declassification facts.
    pub declassifies: Vec<String>,
}

impl DeclassificationRow {
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "operation": self.operation,
            "declassifies": self.declassifies,
        })
    }
}

/// Derive the declassification map from the typed architecture's
/// operation declarations. Deterministic: rows and facts are sorted.
#[must_use]
pub fn declassification_rows() -> Vec<DeclassificationRow> {
    let mut rows = ARCHITECTURE
        .operations
        .iter()
        .map(|operation| {
            let mut declassifies = Vec::new();

            for quantity in operation.reads {
                declassifies.push(format!("read:{}", quantity.as_str()));
            }

            for issuance in operation.issuances {
                declassifies.push(format!(
                    "issuance:{}:{}",
                    issuance.asset.as_str(),
                    issuance.condition.as_str(),
                ));
            }

            for delta in operation.canonical_deltas {
                if delta.kind.as_str() == "destruction" {
                    declassifies.push(format!(
                        "destruction:{}:{}",
                        delta.asset.as_str(),
                        delta.condition.as_str(),
                    ));
                }
            }

            for data_output in operation.data_outputs {
                declassifies.push(format!("record:{}", data_output.tag.as_str()));
            }

            for projection in operation.projections {
                let name = projection.projection.as_str();

                if name != UNIVERSAL_PROJECTION {
                    declassifies.push(format!("projection:{name}"));
                }
            }

            declassifies.sort();
            declassifies.dedup();

            DeclassificationRow {
                operation: operation.id.as_str(),
                declassifies,
            }
        })
        .collect::<Vec<_>>();

    rows.sort_by_key(|row| row.operation);
    rows
}

/// Render `generated/declassification.json` exactly as committed:
/// pretty-printed JSON array plus a trailing newline.
#[must_use]
pub fn declassification_json() -> String {
    let rows = declassification_rows()
        .iter()
        .map(DeclassificationRow::to_json)
        .collect::<Vec<_>>();

    serde_json::to_string_pretty(&rows).expect("declassification map serializes") + "\n"
}

/// The model package root (compile-time manifest directory).
#[must_use]
pub fn package_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn rust_sources(dir: &Path, sources: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("source directory is readable") {
        let path = entry.expect("directory entry is readable").path();

        if path.is_dir() {
            rust_sources(&path, sources);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            sources.push(path);
        }
    }
}

/// Every acute-delimited label token in the model source, deduplicated.
///
/// # Panics
///
/// Panics if the model source tree under the package root is not
/// readable; the harvest is meaningful only on a source checkout.
#[must_use]
pub fn harvest_labels() -> BTreeSet<String> {
    let mut sources = Vec::new();
    rust_sources(&package_root().join("src"), &mut sources);

    let mut labels = BTreeSet::new();

    for path in sources {
        let text = std::fs::read_to_string(&path).expect("source file is readable");
        scan_labels(&path.display().to_string(), &text, &mut labels);
    }

    labels
}

/// Scan one source text for acute-delimited label tokens.
///
/// Delimiters come in pairs on a single line, so odd-index chunks of a
/// per-line split are exactly the delimited tokens. An unmatched
/// delimiter is a harvest failure, not a silently dropped label: an
/// acute-delimited token whose closing delimiter was mistyped must not
/// vanish from the register.
///
/// # Panics
///
/// Panics with `origin` and the line number when a line carries an odd
/// number of acute delimiters.
pub fn scan_labels(origin: &str, text: &str, labels: &mut BTreeSet<String>) {
    for (line_number, line) in text.lines().enumerate() {
        let chunks = line.split('\u{b4}').collect::<Vec<_>>();

        assert!(
            chunks.len() % 2 == 1,
            "unmatched label delimiter at {origin}:{}: {line:?}",
            line_number + 1,
        );

        if chunks.len() < 3 {
            continue;
        }

        for (index, chunk) in chunks.iter().enumerate() {
            if index % 2 == 1 {
                labels.insert((*chunk).to_owned());
            }
        }
    }
}

/// Split a label into its `type:segment:segment` components.
#[must_use]
pub fn split_label(label: &str) -> Vec<&str> {
    label.split(':').collect()
}

/// Render `generated/model_labels.json` exactly as committed: the
/// sorted model-owned label register, pretty-printed plus a trailing
/// newline.
#[must_use]
pub fn model_labels_json() -> String {
    let index = harvest_labels()
        .into_iter()
        .filter(|label| MODEL_TYPES.contains(&split_label(label)[0]))
        .collect::<Vec<_>>();

    serde_json::to_string_pretty(&index).expect("label index serializes") + "\n"
}
