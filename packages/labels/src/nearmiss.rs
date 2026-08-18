//! Near-miss span warnings (ADR-019 total resolution).
//!
//! The calculus asks the checker to warn on spans a reader would take
//! for a label where the grammar does not: an interior differing from a
//! label only in letter case, only in bracketing, or only in spacing,
//! and — in scanned code text, where the acute delimiter carries the
//! label syntax — a backtick span whose interior is label-shaped.
//!
//! Every family here warns and nothing here fails. Two properties keep
//! that honest, and both are structural rather than promised:
//!
//! 1. A near miss is never an occurrence. Each family repairs the span
//!    before testing it, and each repair changes the text: labels are
//!    lowercase, carry no whitespace, and wear no brackets. A span that
//!    any owner's grammar already accepts is therefore unchanged by the
//!    repair, still fails the repaired test, and is never warned about.
//! 2. A near miss is never reported where a defect already is. The
//!    bracket family declines square-bracketed interiors, which belong
//!    to the imported-citation grammar and fail there on their own, and
//!    the prose families see only spans the harvest let fall to text.
//!
//! Displayed spans are silent: prose warnings walk the participating
//! spans of a scan, which excludes fenced blocks and double-backtick
//! spans, and comment warnings are raised only for segments outside a
//! documentation fence.
//!
//! The repaired form must also carry an adopted kind. That single
//! condition is what keeps the families narrow: an ordinary colon-
//! bearing span becomes a candidate only when its first segment is a
//! token of the registry or of the recorded extension set.

use std::path::Path;

use crate::{
    adoption::kind_is_adopted,
    diagnostic::{LabelDiagnostic, LabelErrorCode},
    label::{Label, LabelShape},
    markdown::{MarkdownScan, run},
    owner::ImportedLabel,
    participation::CommentSegment,
    source::SourceLocation,
};

/// The near-miss families the calculus names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NearMiss {
    /// Lowercasing the interior yields a label.
    Casing,
    /// Removing the interior's whitespace yields a label.
    Spacing,
    /// The interior is a label or an imported citation wrapped in
    /// brackets that are not the citation form.
    Brackets,
    /// A backtick span in scanned comment text whose interior is
    /// label-shaped, where the acute delimiter was meant.
    BacktickForAcute,
}

impl NearMiss {
    /// The warning text, naming the repair rather than the rule.
    const fn message(self) -> &'static str {
        match self {
            Self::Casing => {
                "near-miss span: this differs from a label only in letter case, and a label is \
                 lowercase throughout"
            }
            Self::Spacing => {
                "near-miss span: this differs from a label only in spacing, and a label carries \
                 no whitespace"
            }
            Self::Brackets => {
                "near-miss span: this is a label in brackets that are not the citation form, \
                 which parenthesizes the span itself"
            }
            Self::BacktickForAcute => {
                "near-miss span: this backtick span in scanned comment text is label-shaped, \
                 where the acute delimiter carries the label syntax"
            }
        }
    }
}

/// Whether a value is exactly a label of an adopted kind.
///
/// The three-segment planning shape is the strictest of the owner
/// shapes and is used for every surface: a near miss is a warning, and
/// a warning earns its place by being rare.
fn label_shaped(value: &str) -> bool {
    Label::parse(value.trim(), LabelShape::Planning)
        .is_ok_and(|label| kind_is_adopted(label.kind()))
}

/// Whether a value is exactly the interior of an imported citation.
fn imported_shaped(value: &str) -> bool {
    ImportedLabel::parse(value.trim()).is_ok()
}

/// The interior of a bracketed value, for bracket pairs that are not
/// the imported-citation form.
///
/// Square brackets are deliberately absent: they open the imported
/// grammar, which diagnoses its own defects, and a warning there would
/// double-report an error.
fn wrapped(value: &str) -> Option<&str> {
    let value = value.trim();
    for (open, close) in [('(', ')'), ('{', '}')] {
        if let Some(interior) = value.strip_prefix(open).and_then(|v| v.strip_suffix(close)) {
            return Some(interior);
        }
    }
    None
}

/// Classify one prose span that the grammar has already read as text.
///
/// Returns the first matching family in a fixed order, so one span
/// yields at most one warning and the order never depends on input.
pub fn classify(content: &str) -> Option<NearMiss> {
    // An occurrence is not a near miss. The harvest has normally
    // decided this already; the test is repeated so the classifier is
    // sound when called on its own.
    if label_shaped(content) {
        return None;
    }
    let lowered = content.to_ascii_lowercase();
    if lowered != content && label_shaped(&lowered) {
        return Some(NearMiss::Casing);
    }
    let tight = content
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    if tight != content.trim() && label_shaped(&tight) {
        return Some(NearMiss::Spacing);
    }
    if let Some(interior) = wrapped(content)
        && (label_shaped(interior) || imported_shaped(interior))
    {
        return Some(NearMiss::Brackets);
    }
    None
}

/// Warn on the near-miss spans of one scanned prose source.
///
/// Called once per scanned file, beside the scan's own diagnostics.
pub fn prose(scan: &MarkdownScan, diagnostics: &mut Vec<LabelDiagnostic>) {
    for span in scan.participating_spans() {
        if let Some(family) = classify(&span.content) {
            diagnostics.push(LabelDiagnostic::warning(
                LabelErrorCode::NearMissSpan,
                &span.location,
                family.message(),
            ));
        }
    }
}

/// Warn on label-shaped backtick spans of one scanned comment segment.
///
/// Only single-backtick spans are considered, matching the prose rule
/// that a wider delimiter shows rather than means. An unclosed
/// delimiter ends the walk without comment: backticks are not the
/// comment surface's label syntax, so their pairing is not this
/// checker's affair.
pub fn comment(path: &Path, segment: &CommentSegment, diagnostics: &mut Vec<LabelDiagnostic>) {
    let line = &segment.text;
    let bytes = line.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() {
        if bytes[cursor] != b'`' {
            cursor += 1;
            continue;
        }
        let start = cursor;
        let length = run(bytes, cursor);
        cursor += length;
        let mut end = cursor;
        while end < bytes.len() && !(bytes[end] == b'`' && run(bytes, end) == length) {
            end += 1;
        }
        if end == bytes.len() {
            return;
        }
        let interior = &line[cursor..end];
        if length == 1 && comment_label_shaped(interior) {
            diagnostics.push(LabelDiagnostic::warning(
                LabelErrorCode::NearMissSpan,
                &SourceLocation::new(
                    path,
                    segment.line,
                    segment.column + line[..start].chars().count(),
                ),
                NearMiss::BacktickForAcute.message(),
            ));
        }
        cursor = end + length;
    }
}

/// Whether a comment backtick interior is label-shaped: a local label,
/// or a square-bracketed imported citation. Both forms are written with
/// acute delimiters in comment text, so both are near misses here.
fn comment_label_shaped(interior: &str) -> bool {
    let interior = interior.trim();
    if label_shaped(interior) {
        return true;
    }
    interior
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .is_some_and(imported_shaped)
}
