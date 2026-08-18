use thiserror::Error;

use crate::{
    adoption,
    label::{Label, LabelParseError, LabelShape},
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LabelOwner {
    Attestation,
    Realization,
    Adr(u16),
    Model,
    Plan,
    Doc,
    /// One owner per first-party Cargo package other than the model
    /// crate, named by its `packages/` directory. The import prefix of
    /// each is registered in the adoption data's package table.
    Crate(String),
}

impl LabelOwner {
    pub fn prefix(&self) -> String {
        match self {
            Self::Attestation => "A-".to_owned(),
            Self::Realization => "RZ-".to_owned(),
            Self::Adr(number) => format!("ADR{number:03}-"),
            Self::Model => "MODEL-".to_owned(),
            Self::Plan => "PLAN-".to_owned(),
            Self::Doc => "DOC-".to_owned(),
            Self::Crate(name) => format!(
                "{}-",
                adoption::package_prefix(name)
                    .map_or_else(|| adoption::derive_package_prefix(name), str::to_owned)
            ),
        }
    }
    pub const fn shape(&self) -> LabelShape {
        match self {
            Self::Attestation => LabelShape::Attestation,
            Self::Realization => LabelShape::Realization,
            Self::Adr(_) => LabelShape::Adr,
            Self::Model | Self::Crate(_) => LabelShape::Model,
            Self::Plan | Self::Doc => LabelShape::Planning,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImportedLabel {
    pub owner: LabelOwner,
    pub label: Label,
}

impl ImportedLabel {
    /// Parse an imported-citation token into its owner and label.
    ///
    /// The prefix is everything before the first hyphen, which is exact:
    /// a prefix is capitals and digits only, so the first hyphen is
    /// always the boundary, and every hyphen after it belongs to the
    /// label's name segment. The prefix becomes an owner only through
    /// the adoption signature, so an unregistered prefix has no owner.
    pub fn parse(value: &str) -> Result<Self, OwnerParseError> {
        let Some((prefix, label)) = value.split_once('-') else {
            return Err(OwnerParseError::Unknown(value.to_owned()));
        };
        let Some(owner) = adoption::owner_for_prefix(prefix) else {
            return Err(OwnerParseError::Unknown(value.to_owned()));
        };
        Ok(Self {
            label: Label::parse(label, owner.shape())?,
            owner,
        })
    }
}

#[derive(Clone, Debug, Error)]
pub enum OwnerParseError {
    #[error("unknown imported-label owner in {0:?}")]
    Unknown(String),
    #[error(transparent)]
    Label(#[from] LabelParseError),
}
