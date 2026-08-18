use std::path::Path;

use crate::{
    diagnostic::{LabelDiagnostic, LabelErrorCode},
    participation::{ProseParticipation, nested_fence},
    source::SourceLocation,
};

/// How one delimited span sits relative to the parenthesized-citation
/// grammar (ADR-019).
///
/// Only [`Self::Bare`] mints. The two failure contexts are diagnosed by
/// the harvesting layer: a span adjacent to a parenthesis is an
/// attempted citation, and an attempted citation must never fall back
/// to a mint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InlineCodeContext {
    Bare,
    Parenthesized,
    /// A parenthesis is adjacent on exactly one side and no candidate
    /// partner exists on the line: a dropped parenthesis.
    Asymmetric,
    /// A parenthesis is adjacent, and a partner exists, but the group
    /// content is not the exact citation grammar — typically prose
    /// inside the group.
    MalformedGroup,
}
impl InlineCodeContext {
    /// The diagnostic owed by a failed citation attempt, naming
    /// `subject` (for example `label citation`).
    ///
    /// Returns `None` for the two well-formed contexts.
    pub(crate) fn defect(self, subject: &str) -> Option<(LabelErrorCode, String)> {
        match self {
            Self::Bare | Self::Parenthesized => None,
            Self::Asymmetric => Some((
                LabelErrorCode::AsymmetricCitation,
                format!("{subject} has an unmatched parenthesis"),
            )),
            Self::MalformedGroup => Some((
                LabelErrorCode::MalformedCitationGroup,
                format!(
                    "{subject} is adjacent to a parenthesis but is not a parenthesized citation group"
                ),
            )),
        }
    }
}

#[derive(Clone, Debug)]
pub struct InlineCodeSpan {
    pub content: String,
    pub delimiter_len: usize,
    pub location: SourceLocation,
    pub context: InlineCodeContext,
    pub home: Option<String>,
}
#[derive(Clone, Debug, Default)]
pub struct MarkdownScan {
    pub code_spans: Vec<InlineCodeSpan>,
    pub diagnostics: Vec<LabelDiagnostic>,
    /// The line-level participation view this scan was built on, kept
    /// so a consumer that also reasons about regions — the generated
    /// index boundary in the Realization harvest, for one — asks the
    /// same scanner rather than re-walking the raw source.
    pub participation: ProseParticipation,
}

impl MarkdownScan {
    /// The spans a grammar rule may look at: every span on a
    /// participating line whose delimiter width means it, in the single
    /// place that decision is made.
    pub fn participating_spans(&self) -> impl Iterator<Item = &InlineCodeSpan> {
        self.code_spans.iter().filter(|span| span.participates())
    }
}

pub fn scan_markdown(path: &Path, source: &str) -> MarkdownScan {
    let participation = ProseParticipation::of(source);
    let mut scan = MarkdownScan {
        participation,
        ..MarkdownScan::default()
    };
    let mut home = None;
    for (index, raw) in source.lines().enumerate() {
        let line = index + 1;
        if !scan.participation.participates(line) {
            continue;
        }
        if let Some(column) = nested_fence(raw) {
            scan.diagnostics
                .push(nested_fence_diagnostic(path, line, column));
        }
        if let Some(title) = heading(raw) {
            home = Some(title);
        }
        scan_line(path, raw, line, home.as_deref(), &mut scan);
    }
    if let Some(line) = scan.participation.unclosed_fence() {
        scan.diagnostics.push(LabelDiagnostic::error(
            LabelErrorCode::UnclosedMarkdownFence,
            &SourceLocation::new(path, line, 1),
            "Markdown fence is not closed",
        ));
    }
    scan
}

pub(crate) fn nested_fence_diagnostic(path: &Path, line: usize, column: usize) -> LabelDiagnostic {
    LabelDiagnostic::error(
        LabelErrorCode::NestedMarkdownFence,
        &SourceLocation::new(path, line, column),
        "fenced block inside a blockquote, list item, or indented container is outside the \
         accepted Markdown grammar; move it to the top level",
    )
}

fn heading(line: &str) -> Option<String> {
    let value = line.trim_start();
    let count = value.chars().take_while(|value| *value == '#').count();
    if !(1..=6).contains(&count) || !value[count..].starts_with(char::is_whitespace) {
        return None;
    }
    Some(
        value[count..]
            .trim()
            .split(" · `")
            .next()?
            .trim()
            .to_owned(),
    )
}
fn scan_line(
    path: &Path,
    line: &str,
    line_number: usize,
    home: Option<&str>,
    scan: &mut MarkdownScan,
) {
    let bytes = line.as_bytes();
    let mut cursor = 0;
    let mut ranges = Vec::new();
    let first_span = scan.code_spans.len();
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
            scan.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::UnclosedInlineCode,
                &SourceLocation::new(path, line_number, line[..start].chars().count() + 1),
                "inline code delimiter is not closed",
            ));
            return;
        }
        let after = end + length;
        let location = SourceLocation::new(path, line_number, line[..start].chars().count() + 1);
        scan.code_spans.push(InlineCodeSpan {
            content: line[cursor..end].to_owned(),
            delimiter_len: length,
            location,
            context: InlineCodeContext::Bare,
            home: home.map(str::to_owned),
        });
        ranges.push((start, after));
        cursor = after;
    }
    for (index, (start, after)) in ranges.iter().copied().enumerate() {
        scan.code_spans[first_span + index].context = classify(line, start, after, &ranges);
    }
}

/// Classify one delimited span by parsing its immediate syntactic
/// group.
///
/// `start` and `after` bound the span including its delimiters, and
/// `ranges` holds every delimited span on the line in order. The
/// citation grammar is exact: a parenthesized group whose content is
/// delimited spans separated only by whitespace or commas. When the
/// grammar fails but a parenthesis is adjacent, the occurrence is an
/// attempted citation and is diagnosed — the presence of an unrelated
/// parenthesis elsewhere on the line never demotes it to a mint.
pub(crate) fn classify(
    line: &str,
    start: usize,
    after: usize,
    ranges: &[(usize, usize)],
) -> InlineCodeContext {
    if is_parenthesized_group(line, start, after, ranges) {
        return InlineCodeContext::Parenthesized;
    }
    let open = line[..start].trim_end().ends_with('(');
    let close = line[after..].trim_start().starts_with(')');
    match (open, close) {
        // No adjacent parenthesis: an ordinary occurrence, whatever
        // else the line contains.
        (false, false) => InlineCodeContext::Bare,
        // One adjacent parenthesis with no candidate partner anywhere
        // on the line: the partner was dropped.
        (true, false) if !line[after..].contains(')') => InlineCodeContext::Asymmetric,
        (false, true) if !line[..start].contains('(') => InlineCodeContext::Asymmetric,
        // An adjacent parenthesis with a partner on the line, yet the
        // group is not the exact grammar.
        _ => InlineCodeContext::MalformedGroup,
    }
}
fn is_parenthesized_group(
    line: &str,
    start: usize,
    after: usize,
    ranges: &[(usize, usize)],
) -> bool {
    let Some(open) = line[..start].rfind('(') else {
        return false;
    };
    let Some(close) = line[after..].find(')').map(|offset| after + offset) else {
        return false;
    };
    let mut cursor = open + 1;
    let mut contains_span = false;
    for (span_start, span_after) in ranges {
        if *span_start < cursor || *span_after > close {
            continue;
        }
        if !separator(&line[cursor..*span_start]) {
            return false;
        }
        contains_span = true;
        cursor = *span_after;
    }
    contains_span && separator(&line[cursor..close])
}
fn separator(value: &str) -> bool {
    value
        .chars()
        .all(|character| character.is_whitespace() || character == ',')
}
fn run(bytes: &[u8], start: usize) -> usize {
    bytes[start..]
        .iter()
        .take_while(|value| **value == b'`')
        .count()
}
