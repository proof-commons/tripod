use std::path::Path;

use crate::{
    diagnostic::{LabelDiagnostic, LabelErrorCode},
    source::SourceLocation,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InlineCodeContext {
    Bare,
    Parenthesized,
    Asymmetric,
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
}

pub fn scan_markdown(path: &Path, source: &str) -> MarkdownScan {
    let mut scan = MarkdownScan::default();
    let mut fence: Option<(char, usize, usize)> = None;
    let mut home = None;
    for (index, raw) in source.lines().enumerate() {
        let line = index + 1;
        if let Some((marker, length, _)) = fence {
            if fence_close(raw, marker, length) {
                fence = None;
            }
            continue;
        }
        if let Some((marker, length)) = fence_open(raw) {
            fence = Some((marker, length, line));
            continue;
        }
        if let Some(title) = heading(raw) {
            home = Some(title);
        }
        scan_line(path, raw, line, home.as_deref(), &mut scan);
    }
    if let Some((_, _, line)) = fence {
        scan.diagnostics.push(LabelDiagnostic::error(
            LabelErrorCode::UnclosedMarkdownFence,
            &SourceLocation::new(path, line, 1),
            "Markdown fence is not closed",
        ));
    }
    scan
}

pub(crate) fn fence_open(line: &str) -> Option<(char, usize)> {
    let trimmed = after_fence_indent(line)?;
    let marker = trimmed.chars().next()?;
    if !matches!(marker, '`' | '~') {
        return None;
    }
    let length = trimmed.chars().take_while(|value| *value == marker).count();
    (length >= 3).then_some((marker, length))
}
pub(crate) fn fence_close(line: &str, marker: char, length: usize) -> bool {
    let Some(trimmed) = after_fence_indent(line) else {
        return false;
    };
    let actual = trimmed.chars().take_while(|value| *value == marker).count();
    actual >= length && trimmed[actual..].trim().is_empty()
}
fn after_fence_indent(line: &str) -> Option<&str> {
    let indent = line
        .chars()
        .take_while(|character| *character == ' ')
        .count();
    (indent <= 3).then_some(&line[indent..])
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
        scan.code_spans[first_span + index].context = context(line, start, after, &ranges);
    }
}
fn context(line: &str, start: usize, after: usize, ranges: &[(usize, usize)]) -> InlineCodeContext {
    if is_parenthesized_group(line, start, after, ranges) {
        return InlineCodeContext::Parenthesized;
    }
    let open = line[..start].trim_end().ends_with('(');
    let close = line[after..].trim_start().starts_with(')');
    if (open && !close && !line[after..].contains(')'))
        || (close && !open && !line[..start].contains('('))
    {
        InlineCodeContext::Asymmetric
    } else {
        InlineCodeContext::Bare
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
