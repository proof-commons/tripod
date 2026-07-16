use std::collections::BTreeMap;

use crate::{
    diagnostic::{LabelDiagnostic, LabelErrorCode},
    label::Label,
    owner::LabelOwner,
    source::SourceLocation,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabelMint {
    pub owner: LabelOwner,
    pub label: Label,
    pub location: SourceLocation,
    pub home: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LabelRegistry {
    entries: BTreeMap<Label, LabelMint>,
}

impl LabelRegistry {
    pub fn insert(&mut self, mint: LabelMint) -> Result<(), LabelMint> {
        if self.entries.contains_key(&mint.label) {
            return Err(mint);
        }
        self.entries.insert(mint.label.clone(), mint);
        Ok(())
    }
    pub fn contains(&self, label: &Label) -> bool {
        self.entries.contains_key(label)
    }
    pub fn get(&self, label: &Label) -> Option<&LabelMint> {
        self.entries.get(label)
    }
    /// Insert a mint, emitting one `DuplicateMint` diagnostic naming
    /// the original location when the label is already minted. Every
    /// owner harvest routes duplicates through here so the message
    /// shape cannot drift between owners.
    pub fn insert_or_diagnose(
        &mut self,
        mint: LabelMint,
        owner_name: &str,
        diagnostics: &mut Vec<LabelDiagnostic>,
    ) {
        if let Err(duplicate) = self.insert(mint) {
            let original = self
                .get(&duplicate.label)
                .map(|mint| format!("{}:{}", mint.location.display_path(), mint.location.line))
                .unwrap_or_default();
            diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::DuplicateMint,
                &duplicate.location,
                format!(
                    "duplicate {owner_name} label mint {}; first minted at {original}",
                    duplicate.label,
                ),
            ));
        }
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn iter(&self) -> impl Iterator<Item = (&Label, &LabelMint)> {
        self.entries.iter()
    }
    pub fn labels(&self) -> impl Iterator<Item = &Label> {
        self.entries.keys()
    }
}

#[derive(Clone, Debug, Default)]
pub struct RegistrySet {
    pub attestation: LabelRegistry,
    pub realization: LabelRegistry,
    pub adrs: BTreeMap<u16, LabelRegistry>,
    pub model: LabelRegistry,
    pub plan: LabelRegistry,
    pub doc: LabelRegistry,
    pub crates: BTreeMap<String, LabelRegistry>,
}
