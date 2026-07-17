use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use crate::{
    RepositoryCensus,
    diagnostic::{LabelDiagnostic, LabelErrorCode},
    label::{Label, LabelShape},
    markdown::{fence_close, fence_open},
    owner::{ImportedLabel, LabelOwner},
    registry::{LabelMint, LabelRegistry},
    source::{SourceLocation, relative_to},
};

pub const MODEL_TYPES: &[&str] = &["def", "rule", "thm", "test", "branch", "protocol", "lst"];
pub const REALIZATION_TYPES: &[&str] = &[
    "sec", "subsec", "app", "req", "inv", "lem", "obl", "trap", "rem", "ins", "pin", "res", "fig",
    "tbl", "leaf",
];

/// Harvest of one Rust-source label owner (the model crate or another
/// first-party crate).
#[derive(Default)]
pub struct RustHarvest {
    pub registry: LabelRegistry,
    pub citations: Vec<(Label, SourceLocation)>,
    pub imports: Vec<(ImportedLabel, SourceLocation)>,
    pub diagnostics: Vec<LabelDiagnostic>,
}

pub fn harvest_model(paths: &RepositoryCensus) -> RustHarvest {
    harvest_rust_crate(&paths.root, &paths.model_sources, &LabelOwner::Model)
}

/// Harvest every first-party crate other than the model crate as its
/// own label owner (ADR-013: one owner per Cargo package). Crates with
/// no participating labels have empty registries, which is valid.
pub fn harvest_crates(paths: &RepositoryCensus) -> BTreeMap<String, RustHarvest> {
    paths
        .crate_sources
        .iter()
        .map(|(name, sources)| {
            (
                name.clone(),
                harvest_rust_crate(&paths.root, sources, &LabelOwner::Crate(name.clone())),
            )
        })
        .collect()
}

fn harvest_rust_crate(root: &Path, sources: &[PathBuf], owner: &LabelOwner) -> RustHarvest {
    let mut result = RustHarvest::default();
    let mut files = sources.to_vec();
    files.sort();
    for path in files {
        let relative = relative_to(root, &path);
        match fs::read_to_string(&path) {
            Ok(source) => harvest_file(&relative, &source, owner, &mut result),
            Err(error) => result.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::Io,
                &SourceLocation::new(&relative, 1, 1),
                error.to_string(),
            )),
        }
    }
    result
}

/// One comment text run on one source line, positioned by the first
/// character after the comment marker.
struct CommentSegment {
    line: usize,
    column: usize,
    text: String,
}

/// Extract comment and documentation-comment text from Rust source.
///
/// Labels participate only in comments (ADR-013 Rust-sources rule):
/// string literals, raw strings, byte/C strings, and character
/// literals are code, so an acute span inside them — for example a
/// label-bearing test fixture — must not mint or cite.
fn comment_segments(
    path: &Path,
    source: &str,
    diagnostics: &mut Vec<LabelDiagnostic>,
) -> Vec<CommentSegment> {
    let chars = source.chars().collect::<Vec<_>>();
    let mut segments = Vec::new();
    let mut i = 0;
    let mut line = 1;
    let mut column = 1;
    let mut advance = |i: &mut usize, line: &mut usize, column: &mut usize| {
        if chars[*i] == '\n' {
            *line += 1;
            *column = 1;
        } else {
            *column += 1;
        }
        *i += 1;
    };
    while i < chars.len() {
        match chars[i] {
            '/' if chars.get(i + 1) == Some(&'/') => {
                // Line comment: skip the marker run (`//`, `///`,
                // `//!`) and capture to end of line.
                while i < chars.len() && (chars[i] == '/' || chars[i] == '!') {
                    advance(&mut i, &mut line, &mut column);
                }
                let (start_line, start_column) = (line, column);
                let mut text = String::new();
                while i < chars.len() && chars[i] != '\n' {
                    text.push(chars[i]);
                    advance(&mut i, &mut line, &mut column);
                }
                segments.push(CommentSegment {
                    line: start_line,
                    column: start_column,
                    text,
                });
            }
            '/' if chars.get(i + 1) == Some(&'*') => {
                // Block comment; Rust block comments nest, and every
                // `/*` / `*/` consumes both characters so overlapping
                // sequences like `*/*` cannot be double-counted.
                advance(&mut i, &mut line, &mut column);
                advance(&mut i, &mut line, &mut column);
                let mut depth = 1_usize;
                let (mut start_line, mut start_column) = (line, column);
                let mut text = String::new();
                while i < chars.len() && depth > 0 {
                    if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
                        depth += 1;
                        text.push_str("/*");
                        advance(&mut i, &mut line, &mut column);
                        advance(&mut i, &mut line, &mut column);
                        continue;
                    }
                    if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                        depth -= 1;
                        if depth == 0 {
                            segments.push(CommentSegment {
                                line: start_line,
                                column: start_column,
                                text: std::mem::take(&mut text),
                            });
                        } else {
                            text.push_str("*/");
                        }
                        advance(&mut i, &mut line, &mut column);
                        advance(&mut i, &mut line, &mut column);
                        continue;
                    }
                    if chars[i] == '\n' {
                        segments.push(CommentSegment {
                            line: start_line,
                            column: start_column,
                            text: std::mem::take(&mut text),
                        });
                        advance(&mut i, &mut line, &mut column);
                        start_line = line;
                        start_column = column;
                    } else {
                        text.push(chars[i]);
                        advance(&mut i, &mut line, &mut column);
                    }
                }
                if depth > 0 {
                    diagnostics.push(LabelDiagnostic::error(
                        LabelErrorCode::UnclosedInlineCode,
                        &SourceLocation::new(path, start_line, start_column),
                        "unterminated block comment",
                    ));
                }
            }
            '"' => skip_string(&chars, &mut i, &mut line, &mut column, &mut advance),
            'b' | 'c' | 'r' => match string_prefix(&chars, i) {
                Some(prefix) => {
                    for _ in 0..prefix.len {
                        advance(&mut i, &mut line, &mut column);
                    }
                    if prefix.raw {
                        skip_raw_string(
                            &chars,
                            &mut i,
                            &mut line,
                            &mut column,
                            prefix.hashes,
                            &mut advance,
                        );
                    } else {
                        // Non-raw prefixed string (`b"..."`, `c"..."`):
                        // the quote was consumed; finish it with
                        // escapes.
                        finish_plain_string(&chars, &mut i, &mut line, &mut column, &mut advance);
                    }
                }
                None => advance(&mut i, &mut line, &mut column),
            },
            '\'' => skip_char_or_lifetime(&chars, &mut i, &mut line, &mut column, &mut advance),
            // Everything else — including identifier runs — advances
            // one character; `string_prefix`'s lookbehind already
            // rejects prefix letters inside identifiers.
            _ => advance(&mut i, &mut line, &mut column),
        }
    }
    segments
}

/// A string-literal prefix (`r`, `b`, `br`, `c`, `cr`, with optional
/// `#`s for raw forms) starting at `i`.
struct StringPrefix {
    /// Characters from `i` through the opening quote, inclusive.
    len: usize,
    raw: bool,
    hashes: usize,
}

fn string_prefix(chars: &[char], i: usize) -> Option<StringPrefix> {
    if i > 0 && (chars[i - 1].is_alphanumeric() || chars[i - 1] == '_') {
        return None;
    }
    let mut j = i;
    if matches!(chars.get(j), Some('b' | 'c')) {
        j += 1;
    }
    let raw = matches!(chars.get(j), Some('r'));
    let mut hashes = 0;
    if raw {
        j += 1;
        while matches!(chars.get(j), Some('#')) {
            j += 1;
            hashes += 1;
        }
    }
    if j == i {
        return None;
    }
    (chars.get(j) == Some(&'"')).then_some(StringPrefix {
        len: j - i + 1,
        raw,
        hashes,
    })
}

fn skip_string(
    chars: &[char],
    i: &mut usize,
    line: &mut usize,
    column: &mut usize,
    advance: &mut impl FnMut(&mut usize, &mut usize, &mut usize),
) {
    advance(i, line, column);
    finish_plain_string(chars, i, line, column, advance);
}

fn finish_plain_string(
    chars: &[char],
    i: &mut usize,
    line: &mut usize,
    column: &mut usize,
    advance: &mut impl FnMut(&mut usize, &mut usize, &mut usize),
) {
    while *i < chars.len() {
        match chars[*i] {
            '\\' => {
                advance(i, line, column);
                if *i < chars.len() {
                    advance(i, line, column);
                }
            }
            '"' => {
                advance(i, line, column);
                return;
            }
            _ => advance(i, line, column),
        }
    }
}

fn skip_raw_string(
    chars: &[char],
    i: &mut usize,
    line: &mut usize,
    column: &mut usize,
    hashes: usize,
    advance: &mut impl FnMut(&mut usize, &mut usize, &mut usize),
) {
    while *i < chars.len() {
        if chars[*i] == '"'
            && chars[*i + 1..]
                .iter()
                .take_while(|value| **value == '#')
                .count()
                >= hashes
        {
            advance(i, line, column);
            for _ in 0..hashes {
                advance(i, line, column);
            }
            return;
        }
        advance(i, line, column);
    }
}

fn skip_char_or_lifetime(
    chars: &[char],
    i: &mut usize,
    line: &mut usize,
    column: &mut usize,
    advance: &mut impl FnMut(&mut usize, &mut usize, &mut usize),
) {
    advance(i, line, column);
    if chars.get(*i) == Some(&'\\') {
        // Escaped char literal: consume to the closing quote.
        while *i < chars.len() && chars[*i] != '\'' {
            if chars[*i] == '\\' {
                advance(i, line, column);
            }
            if *i < chars.len() {
                advance(i, line, column);
            }
        }
        if *i < chars.len() {
            advance(i, line, column);
        }
    } else if chars.get(*i + 1) == Some(&'\'') {
        // Plain char literal `'x'`.
        advance(i, line, column);
        advance(i, line, column);
    }
    // Otherwise a lifetime: the quote is already consumed and the
    // identifier run is ordinary code.
}

fn harvest_file(path: &Path, source: &str, owner: &LabelOwner, result: &mut RustHarvest) {
    let segments = comment_segments(path, source, &mut result.diagnostics);
    // Fenced Rustdoc examples are nonparticipating, using the same
    // fence recognition as the Markdown scanner (backtick and tilde
    // markers alike). Fences open and close within comment text.
    let mut fence: Option<(char, usize)> = None;
    let mut fence_line = 0;
    for segment in segments {
        if let Some((marker, length)) = fence {
            if fence_close(&segment.text, marker, length) {
                fence = None;
            }
            continue;
        }
        if let Some(open) = fence_open(&segment.text) {
            fence = Some(open);
            fence_line = segment.line;
            continue;
        }
        harvest_segment(path, &segment, owner, result);
    }
    if fence.is_some() {
        result.diagnostics.push(LabelDiagnostic::error(
            LabelErrorCode::UnclosedMarkdownFence,
            &SourceLocation::new(path, fence_line, 1),
            "documentation fence is not closed",
        ));
    }
}

fn harvest_segment(
    path: &Path,
    segment: &CommentSegment,
    owner: &LabelOwner,
    result: &mut RustHarvest,
) {
    let line = &segment.text;
    let mut opening = None;
    for (offset, character) in line.char_indices() {
        if character != '\u{b4}' {
            continue;
        }
        if let Some(start) = opening.take() {
            let location = SourceLocation::new(
                path,
                segment.line,
                segment.column + line[..start].chars().count(),
            );
            let after = &line[offset + character.len_utf8()..];
            let open = line[..start].ends_with('(');
            let close = after.starts_with(')');
            // One-sided parentheses are a citation typo, mirroring the
            // Markdown scanner: silently treating the span as a bare
            // mint would let a dangling citation self-satisfy.
            if (open && !close && !after.contains(')'))
                || (close && !open && !line[..start].contains('('))
            {
                result.diagnostics.push(LabelDiagnostic::error(
                    LabelErrorCode::AsymmetricCitation,
                    &location,
                    "label citation has an unmatched parenthesis",
                ));
                continue;
            }
            harvest_label(
                &line[start + character.len_utf8()..offset],
                location,
                open && close,
                owner,
                result,
            );
        } else {
            opening = Some(offset);
        }
    }
    if let Some(offset) = opening {
        result.diagnostics.push(LabelDiagnostic::error(
            LabelErrorCode::UnclosedInlineCode,
            &SourceLocation::new(
                path,
                segment.line,
                segment.column + line[..offset].chars().count(),
            ),
            "unclosed acute label delimiter",
        ));
    }
}

fn harvest_label(
    value: &str,
    location: SourceLocation,
    parenthesized: bool,
    owner: &LabelOwner,
    result: &mut RustHarvest,
) {
    if let Some(token) = value.strip_prefix('[').and_then(|v| v.strip_suffix(']')) {
        if !parenthesized {
            result.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::InvalidImportedCitationForm,
                &location,
                "imported citation must be parenthesized",
            ));
            return;
        }
        match ImportedLabel::parse(token) {
            Ok(imported) => result.imports.push((imported, location)),
            Err(error) => result.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::UnknownOwner,
                &location,
                error.to_string(),
            )),
        }
        return;
    }
    let kind = value.split(':').next().unwrap_or_default();
    if MODEL_TYPES.contains(&kind) {
        match Label::parse(value, LabelShape::Model) {
            Ok(label) if parenthesized => result.citations.push((label, location)),
            Ok(label) => {
                let mint = LabelMint {
                    owner: owner.clone(),
                    label,
                    location,
                    home: None,
                };
                result
                    .registry
                    .insert_or_diagnose(mint, "Rust", &mut result.diagnostics);
            }
            Err(error) => result.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::InvalidLabel,
                &location,
                error.to_string(),
            )),
        }
    } else if *owner == LabelOwner::Model && REALIZATION_TYPES.contains(&kind) {
        // In the model crate, a bare realization-shaped token is a
        // citation missing its explicit owner. Other crates get the
        // generic unknown-type diagnostic below: suggesting an R13
        // import for a crate-local label would be wrong advice.
        result.diagnostics.push(LabelDiagnostic::error(
            LabelErrorCode::InvalidImportedCitationForm,
            &location,
            format!("realization citation must use the imported form (´[RZ-{value}]´)"),
        ));
    } else {
        result.diagnostics.push(LabelDiagnostic::error(
            LabelErrorCode::InvalidLabel,
            &location,
            format!("unknown Rust label type {kind}"),
        ));
    }
}

pub fn model_labels(registry: &LabelRegistry) -> BTreeSet<String> {
    registry.labels().map(ToString::to_string).collect()
}
