use serde::Serialize;

use crate::source::SourceLocation;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LabelErrorCode {
    Io,
    UnclosedMarkdownFence,
    /// A fence hidden behind a blockquote, list, or indentation
    /// container, which the accepted Markdown grammar does not carry.
    NestedMarkdownFence,
    UnclosedInlineCode,
    AsymmetricCitation,
    /// An attempted citation adjacent to a parenthesis whose group is
    /// not the exact parenthesized-citation grammar.
    MalformedCitationGroup,
    DuplicateMint,
    MissingMint,
    UnknownOwner,
    UnknownImportedLabel,
    InvalidImportedCitationForm,
    InvalidLabel,
    InvalidStatusTag,
    DuplicateLatexLabel,
    ArchitectureLabelMissing,
    AttestationAnchorSetMismatch,
    AttestationIndexStale,
    GeneratedRegisterMissing,
    GeneratedRegisterStale,
    CensusStale,
    /// A census traversal could not read a directory or an entry, so
    /// the discovered membership of its group is unknown.
    CensusUnreadable,
    /// A minted kind lies outside the adopted vocabulary: neither a
    /// registry token nor a recorded extension.
    UnknownKind,
    /// A kind reserved for derivation that no profile governs, so no
    /// warrant rule admits its occurrence.
    ReservedKindWithoutProfile,
    /// An inventory-kind token away from its profile's standard place,
    /// where the derivation warrant cannot reach it.
    InventoryKindOutOfPlace,
    /// The checker's committed kind vocabulary disagrees with the
    /// document it was extracted from.
    KindVocabularyDrift,
    /// A Cargo package carries no registered owner prefix, or a
    /// registration names no package.
    UnregisteredOwner,
    /// A span a reader would take for a label that the grammar reads as
    /// text: a warning, never an occurrence and never a failure.
    NearMissSpan,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LabelDiagnostic {
    pub code: LabelErrorCode,
    pub severity: Severity,
    pub message: String,
    pub path: String,
    pub line: usize,
    pub column: usize,
}

impl LabelDiagnostic {
    pub fn error(
        code: LabelErrorCode,
        location: &SourceLocation,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity: Severity::Error,
            message: message.into(),
            path: location.display_path(),
            line: location.line,
            column: location.column,
        }
    }

    /// A diagnostic that reports without failing the check. Reserved for
    /// facts a recorded decision has placed outside the enforcing scope,
    /// which must stay visible without turning the gate red.
    pub fn warning(
        code: LabelErrorCode,
        location: &SourceLocation,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity: Severity::Warning,
            message: message.into(),
            path: location.display_path(),
            line: location.line,
            column: location.column,
        }
    }

    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

pub fn sort_diagnostics(diagnostics: &mut [LabelDiagnostic]) {
    diagnostics.sort_by(|left, right| {
        (
            left.severity,
            &left.path,
            left.line,
            left.column,
            left.code,
            &left.message,
        )
            .cmp(&(
                right.severity,
                &right.path,
                right.line,
                right.column,
                right.code,
                &right.message,
            ))
    });
}
