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
    /// An environment head declares a kind the effective relation does
    /// not pair with the head's own name, directly or by reduction.
    UncataloguedHeadPair,
    /// A Cargo package carries no registered owner prefix, or a
    /// registration names no package.
    UnregisteredOwner,
    /// A span a reader would take for a label that the grammar reads as
    /// text: a warning, never an occurrence and never a failure.
    NearMissSpan,
    /// A label-intended occurrence that is not three-part. The rule is
    /// universal: see the `shape` module for the surfaces it governs.
    MalformedLabelShape,
}

impl LabelErrorCode {
    /// Stable wire spelling used by structured diagnostics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Io => "io",
            Self::UnclosedMarkdownFence => "unclosed_markdown_fence",
            Self::NestedMarkdownFence => "nested_markdown_fence",
            Self::UnclosedInlineCode => "unclosed_inline_code",
            Self::AsymmetricCitation => "asymmetric_citation",
            Self::MalformedCitationGroup => "malformed_citation_group",
            Self::DuplicateMint => "duplicate_mint",
            Self::MissingMint => "missing_mint",
            Self::UnknownOwner => "unknown_owner",
            Self::UnknownImportedLabel => "unknown_imported_label",
            Self::InvalidImportedCitationForm => "invalid_imported_citation_form",
            Self::InvalidLabel => "invalid_label",
            Self::InvalidStatusTag => "invalid_status_tag",
            Self::DuplicateLatexLabel => "duplicate_latex_label",
            Self::ArchitectureLabelMissing => "architecture_label_missing",
            Self::AttestationAnchorSetMismatch => "attestation_anchor_set_mismatch",
            Self::AttestationIndexStale => "attestation_index_stale",
            Self::GeneratedRegisterMissing => "generated_register_missing",
            Self::GeneratedRegisterStale => "generated_register_stale",
            Self::CensusStale => "census_stale",
            Self::CensusUnreadable => "census_unreadable",
            Self::UnknownKind => "unknown_kind",
            Self::ReservedKindWithoutProfile => "reserved_kind_without_profile",
            Self::InventoryKindOutOfPlace => "inventory_kind_out_of_place",
            Self::KindVocabularyDrift => "kind_vocabulary_drift",
            Self::UncataloguedHeadPair => "uncatalogued_head_pair",
            Self::UnregisteredOwner => "unregistered_owner",
            Self::NearMissSpan => "near_miss_span",
            Self::MalformedLabelShape => "malformed_label_shape",
        }
    }
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

#[cfg(test)]
mod trace_name_tests {
    use super::LabelErrorCode;

    #[test]
    fn every_label_diagnostic_trace_name_is_snake_case() {
        use LabelErrorCode::*;
        let codes = [
            Io,
            UnclosedMarkdownFence,
            NestedMarkdownFence,
            UnclosedInlineCode,
            AsymmetricCitation,
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
            CensusUnreadable,
            UnknownKind,
            ReservedKindWithoutProfile,
            InventoryKindOutOfPlace,
            KindVocabularyDrift,
            UncataloguedHeadPair,
            UnregisteredOwner,
            NearMissSpan,
            MalformedLabelShape,
        ];
        let snake_case = regex::Regex::new(r"^[a-z][a-z0-9]*(_[a-z0-9]+)*$")
            .expect("valid tracing-name expression");
        for code in codes {
            let name = code.as_str();
            assert!(snake_case.is_match(name), "invalid tracing name: {name}");
        }
        assert_eq!(Io.as_str(), "io");
        assert_eq!(
            InvalidImportedCitationForm.as_str(),
            "invalid_imported_citation_form"
        );
    }
}
