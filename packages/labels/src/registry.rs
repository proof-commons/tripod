use std::collections::BTreeMap;

use crate::{label::Label, owner::LabelOwner, source::SourceLocation};

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
}
