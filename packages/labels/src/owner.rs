use thiserror::Error;

use crate::label::{Label, LabelParseError, LabelShape};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LabelOwner {
    Attestation,
    Realization,
    Adr(u16),
    Model,
    Plan,
    Doc,
    /// One owner per first-party Cargo package other than the model
    /// crate, named by its `packages/` directory. Crate labels have no
    /// public import prefix until a real cross-owner citation needs
    /// one (ADR-013 owner registry).
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
            Self::Crate(name) => format!("{name}-"),
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
    pub fn parse(value: &str) -> Result<Self, OwnerParseError> {
        let (owner, label) = if let Some(label) = value.strip_prefix("A-") {
            (LabelOwner::Attestation, label)
        } else if let Some(label) = value.strip_prefix("RZ-") {
            (LabelOwner::Realization, label)
        } else if let Some(label) = value.strip_prefix("MODEL-") {
            (LabelOwner::Model, label)
        } else if let Some(label) = value.strip_prefix("PLAN-") {
            (LabelOwner::Plan, label)
        } else if let Some(label) = value.strip_prefix("DOC-") {
            (LabelOwner::Doc, label)
        } else if let Some(rest) = value.strip_prefix("ADR") {
            let Some((number, label)) = rest.split_once('-') else {
                return Err(OwnerParseError::Unknown(value.to_owned()));
            };
            if number.len() != 3 {
                return Err(OwnerParseError::Unknown(value.to_owned()));
            }
            (
                LabelOwner::Adr(
                    number
                        .parse()
                        .map_err(|_| OwnerParseError::Unknown(value.to_owned()))?,
                ),
                label,
            )
        } else {
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
