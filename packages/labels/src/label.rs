use std::fmt;

use thiserror::Error;

use crate::shape;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Label(String);

impl Label {
    pub fn parse(value: &str, shape: LabelShape) -> Result<Self, LabelParseError> {
        let parts = value.split(':').collect::<Vec<_>>();
        if parts.iter().any(|part| !valid_segment(part)) {
            return Err(LabelParseError::Malformed(value.to_owned()));
        }
        // The kind is a word, never a hyphenated one: it ranges over the
        // ADR-020 registry, and a registry of words admits no hyphenated
        // member. Area and name may hyphenate — the area concession is
        // this repository's amendment recorded in ADR-019.
        if parts.first().is_some_and(|kind| kind.contains('-')) {
            return Err(LabelParseError::Malformed(value.to_owned()));
        }
        // The arity rule is one decision, held in `shape` and applied to
        // every owner alike: a label is three-part, with no surface
        // exemption and no enumerated residue — the realization
        // contract's divisions migrated to the three-part form.
        if !shape::arity_admitted(parts.len()) {
            return Err(LabelParseError::Arity(value.to_owned()));
        }
        if shape == LabelShape::Realization {
            // ADR-020 retired `subsec`: a subsection is a section
            // nested, and the sub- prefix is a presentation device, so
            // what `subsec` labelled is a three-part `sec`. `app` joins
            // the same list for the same reason: an appendix carries
            // divisions exactly as a section does.
            let kinds = [
                "sec", "app", "req", "def", "inv", "lem", "obl", "trap", "rem", "intuit", "rule",
                "pin", "res", "listing", "fig", "tab", "leaf",
            ];
            if !kinds.contains(&parts[0]) {
                return Err(LabelParseError::Shape(value.to_owned()));
            }
        }
        Ok(Self(value.to_owned()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    /// The kind segment: the first of the colon-joined triple, and the
    /// segment the ADR-020 registry governs. Every parsed label has one,
    /// so the split cannot fail.
    pub fn kind(&self) -> &str {
        self.0.split(':').next().unwrap_or(&self.0)
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
    /// The token is not three-part. Reported with the expected form,
    /// since the writer of a two-segment label has a name for what is
    /// missing but not always a name for the rule.
    #[error("label {0:?} is not three-part; a label is written {form}", form = shape::EXPECTED_FORM)]
    Arity(String),
    #[error("invalid owner-specific label shape {0:?}")]
    Shape(String),
}

impl LabelParseError {
    /// The diagnostic code a reported parse failure carries.
    pub const fn code(&self) -> crate::diagnostic::LabelErrorCode {
        match self {
            Self::Arity(_) => crate::diagnostic::LabelErrorCode::MalformedLabelShape,
            Self::Malformed(_) | Self::Shape(_) => crate::diagnostic::LabelErrorCode::InvalidLabel,
        }
    }
}
