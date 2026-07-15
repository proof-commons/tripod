use serde::Serialize;

use crate::source::SourceLocation;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LabelErrorCode {
    Io,
    UnclosedMarkdownFence,
    UnclosedInlineCode,
    AsymmetricCitation,
    DuplicateMint,
    MissingMint,
    UnknownOwner,
    UnknownImportedLabel,
    InvalidImportedCitationForm,
    InvalidLabel,
    DuplicateLatexLabel,
    ArchitectureLabelMissing,
    AttestationAnchorSetMismatch,
    GeneratedRegisterMissing,
    GeneratedRegisterStale,
    ModelLabelPublicationStale,
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
