//! Canonical grammar for keyed-capture-freeze-v1 (UTF-8, no final newline).
//!
//! The literal prefix is `keyed-capture-freeze-v1:`. Structs use their literal
//! name, ` { `, named fields separated by `, `, and ` }`. A field is its literal
//! name followed by `: ` and its value. `FreezeManifest` fields are frozen,
//! computed, key, `leaf_digest`, `recipe_schema`, `target_version`, envelope, in that
//! order. `SourceIdentity` fields are path, digest; `Envelope` fields are handles,
//! `signature_bytes`, `weight_units`, sponsors, outputs. Every field is included.
//!
//! The frozen map uses `{` and `}`, with entries separated by `, ` and each
//! category followed by `: ` and its source vector. Categories use the exact
//! names and ranks in `frozen_name`, independent of derived
//! `Ord`. Computed categories use the exact names in `computed_name`. Vectors and
//! byte arrays use `[` and `]`, with elements separated by `, ` in stored order;
//! an empty collection has no interior space. Bytes are unsigned decimal
//! integers, not hexadecimal or text. All integers use primitive `Display`'s
//! base-ten ASCII digits, without sign, padding, separators or leading zeroes
//! (except zero itself). No other standard formatting routine is used.
//!
//! Strings contain zero or more characters from `[A-Za-z0-9/._-]`, copied between double
//! quotes, with no escaping or normalization. A scalar outside this alphabet
//! refuses with `FreezeRefusal::FrozenInventory`; no encoding is returned.
//! This preserves the former rendering on every admitted path. Comparisons
//! with the former `Debug` rendering are regression aids, never the definition.

use super::{ComputedItem, Envelope, FreezeManifest, FreezeRefusal, FrozenItem, SourceIdentity};

pub(super) fn render(manifest: &FreezeManifest) -> Result<String, FreezeRefusal> {
    let FreezeManifest {
        frozen,
        computed,
        key,
        leaf_digest,
        recipe_schema,
        target_version,
        envelope,
    } = manifest;
    let mut out = String::from("keyed-capture-freeze-v1:FreezeManifest { frozen: {");
    let mut categories: Vec<_> = frozen.iter().collect();
    categories.sort_by_key(|(kind, _)| frozen_name(**kind).0);
    for (index, (kind, sources)) in categories.into_iter().enumerate() {
        separator(&mut out, index);
        out.push_str(frozen_name(*kind).1);
        out.push_str(": [");
        for (index, source) in sources.iter().enumerate() {
            separator(&mut out, index);
            let SourceIdentity { path, digest } = source;
            out.push_str("SourceIdentity { path: ");
            string(&mut out, path)?;
            out.push_str(", digest: ");
            bytes(&mut out, digest);
            out.push_str(" }");
        }
        out.push(']');
    }
    out.push_str("}, computed: [");
    for (index, item) in computed.iter().enumerate() {
        separator(&mut out, index);
        out.push_str(computed_name(*item));
    }
    out.push_str("], key: ");
    bytes(&mut out, key);
    out.push_str(", leaf_digest: ");
    bytes(&mut out, leaf_digest);
    out.push_str(", recipe_schema: ");
    out.push_str(&recipe_schema.to_string());
    out.push_str(", target_version: ");
    out.push_str(&target_version.to_string());
    out.push_str(", envelope: Envelope { handles: ");
    let Envelope {
        handles,
        signature_bytes,
        weight_units,
        sponsors,
        outputs,
    } = envelope;
    out.push_str(&handles.to_string());
    out.push_str(", signature_bytes: ");
    out.push_str(&signature_bytes.to_string());
    out.push_str(", weight_units: ");
    out.push_str(&weight_units.to_string());
    out.push_str(", sponsors: ");
    out.push_str(&sponsors.to_string());
    out.push_str(", outputs: ");
    out.push_str(&outputs.to_string());
    out.push_str(" } }");
    Ok(out)
}

fn separator(out: &mut String, index: usize) {
    if index != 0 {
        out.push_str(", ");
    }
}

fn bytes(out: &mut String, bytes: &[u8]) {
    out.push('[');
    for (index, byte) in bytes.iter().enumerate() {
        separator(out, index);
        out.push_str(&byte.to_string());
    }
    out.push(']');
}

fn string(out: &mut String, value: &str) -> Result<(), FreezeRefusal> {
    if !value.bytes().all(
        |byte| matches!(byte, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b'.' | b'_' | b'-'),
    ) {
        return Err(FreezeRefusal::FrozenInventory);
    }
    out.push('"');
    out.push_str(value);
    out.push('"');
    Ok(())
}

const fn frozen_name(kind: FrozenItem) -> (u8, &'static str) {
    match kind {
        FrozenItem::Declarations => (0, "Declarations"),
        FrozenItem::Realization => (1, "Realization"),
        FrozenItem::Plans => (2, "Plans"),
        FrozenItem::Selection => (3, "Selection"),
        FrozenItem::Grammar => (4, "Grammar"),
        FrozenItem::Recipe => (5, "Recipe"),
        FrozenItem::Target => (6, "Target"),
        FrozenItem::Abi => (7, "Abi"),
        FrozenItem::Dependencies => (8, "Dependencies"),
        FrozenItem::FundingSigning => (9, "FundingSigning"),
        FrozenItem::PolicyEnvelope => (10, "PolicyEnvelope"),
        FrozenItem::References => (11, "References"),
    }
}

const fn computed_name(kind: ComputedItem) -> &'static str {
    match kind {
        ComputedItem::Leaf => "Leaf",
        ComputedItem::LeafHash => "LeafHash",
        ComputedItem::Root => "Root",
        ComputedItem::Tweak => "Tweak",
        ComputedItem::OutputProgram => "OutputProgram",
        ComputedItem::Parity => "Parity",
        ComputedItem::Control => "Control",
        ComputedItem::Commitment => "Commitment",
        ComputedItem::RequestAssociation => "RequestAssociation",
        ComputedItem::HandleSets => "HandleSets",
        ComputedItem::Signatures => "Signatures",
        ComputedItem::TransactionIdentities => "TransactionIdentities",
        ComputedItem::PolicyReading => "PolicyReading",
        ComputedItem::ConstructionChoices => "ConstructionChoices",
        ComputedItem::EmissionAndClosure => "EmissionAndClosure",
        ComputedItem::FinalResources => "FinalResources",
    }
}

#[cfg(test)]
mod tests;
