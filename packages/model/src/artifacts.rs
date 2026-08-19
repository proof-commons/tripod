//! Derivations behind the model's generated artifacts:
//! `generated/declassification.json` and `generated/model_labels.json`.
//!
//! This module is the typed source for declassification. The
//! `tripod-artifacts` crate's `generate-all` binary renders
//! and writes the derived publication; its `check-generated` binary
//! and the test suite compare committed bytes without writing.
//! Tests never modify the checkout.
//!
//! # Declassification
//!
//! Implements the derivation demanded by
//! `(´[RZ-sec:realization:representation]´)` and O9
//! `(´[RZ-obl:oracle:disclosure]´)`: each operation's declassification set
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
//! movement is the private membrane `(´[RZ-trap:representation:two-axes]´)`.
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
use architecture::ARCHITECTURE;

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
