use std::fmt;

use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Label(String);

impl Label {
    pub fn parse(value: &str, shape: LabelShape) -> Result<Self, LabelParseError> {
        let parts = value.split(':').collect::<Vec<_>>();
        if parts.iter().any(|part| !valid_segment(part)) {
            return Err(LabelParseError::Malformed(value.to_owned()));
        }
        match shape {
            LabelShape::Attestation if matches!(parts.len(), 2 | 3) => {}
            LabelShape::Attestation => return Err(LabelParseError::Shape(value.to_owned())),
            LabelShape::Planning | LabelShape::Adr | LabelShape::Model if parts.len() == 3 => {}
            LabelShape::Planning | LabelShape::Adr | LabelShape::Model => {
                return Err(LabelParseError::Shape(value.to_owned()));
            }
            LabelShape::Realization => {
                let kinds = [
                    "sec", "app", "req", "def", "inv", "lem", "obl", "trap", "rem", "intuit",
                    "rule", "pin", "res", "listing", "fig", "tab", "leaf",
                ];
                if parts.len() != 3 || !kinds.contains(&parts[0]) {
                    return Err(LabelParseError::Shape(value.to_owned()));
                }
            }
        }
        Ok(Self(value.to_owned()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Label {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LabelShape {
    Planning,
    Adr,
    Attestation,
    Realization,
    Model,
}

fn valid_segment(value: &&str) -> bool {
    !value.is_empty()
        && !value.starts_with('-')
        && !value.ends_with('-')
        && !value.contains("--")
        && value.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
}

#[derive(Clone, Debug, Error)]
pub enum LabelParseError {
    #[error("malformed label {0:?}")]
    Malformed(String),
    #[error("invalid owner-specific label shape {0:?}")]
    Shape(String),
}
