//! The one participation scanner every label check consumes.
//!
//! # Why this module exists
//!
//! The calculus defines participation once `(´[ADR019-judg:labels:participation]´)`:
//! in prose, authored text participates while fenced blocks and
//! double-backtick spans do not; in code, only comments and
//! documentation comments are scanned, never string or character
//! literals nor fenced documentation examples; a generated register
//! participates in nothing it indexes. One comment is one logical
//! region and one prose block is another, and delimiter pairing is
//! settled within a region before any span in it is parsed.
//!
//! Before this module the repository re-derived fragments of that
//! judgment in six places, which is how the scanners came to disagree
//! (recorded as DI-F01 and DI-F02). The survey below is the contract:
//! every row names a decision site, the logic it used to carry, and
//! where that decision lives now. A new check must appear in this
//! table, consuming this module, rather than growing a seventh
//! recognizer.
//!
//! # Survey of participation decision sites
//!
//! | Site | Decision it made | Now |
//! | --- | --- | --- |
//! | `markdown::scan_markdown` | own fence open/close loop over lines; emitted every backtick span regardless of delimiter width | consumes [`ProseParticipation`] for line roles; span width still recorded, filtered by [`InlineCodeSpan::participates`] |
//! | `repository::harvest_realization` | `delimiter_len != 1` skip, restated inline | [`crate::markdown::MarkdownScan::participating_spans`] |
//! | `repository::harvest_adrs` | `delimiter_len != 1` skip, restated inline | [`crate::markdown::MarkdownScan::participating_spans`] |
//! | `repository::harvest_markdown_owner` | `delimiter_len != 1` skip, restated inline | [`crate::markdown::MarkdownScan::participating_spans`] |
//! | `repository::harvest_attestation_citations` | `delimiter_len != 1` skip, restated inline; **and** computed the generated-index region boundary by walking raw `source.lines()` with no fence awareness | [`crate::markdown::MarkdownScan::participating_spans`]; the region walk now consults [`ProseParticipation::participates`] (DI-F02) |
//! | `plans::without_fenced_lines` | a second fence open/close loop, blanking fenced lines for the link check only | [`ProseParticipation::blanked`] |
//! | `rust_source::comment_segments` | comment/literal segmentation for Rust: the scanned-region recognition | moved here as [`comment_segments`], the Rust front-end; the harvester keeps only its fence handling, which calls [`fence_open`]/[`fence_close`]/[`nested_fence`] |
//! | `rust_source::harvest_region` | located and classified the acute spans of one comment region, for Rust alone | moved here as [`region_spans`], shared by both code front-ends; the harvesters keep only what they do with a located span |
//! | the Python front-end (ADR-023) | new: comment/string segmentation for Python | [`python_comment_segments`], written here rather than beside its harvester, so the second code language cannot grow a second recognizer |
//! | `latex::comments` | strips percent comments, honouring backslash escaping | moved here as [`latex_participating`], the LaTeX front-end; LaTeX has no fenced or double-delimited displayed material, so its region model is total |
//! | `repository::harvest_plans` | skipped the two generated registers by census role | unchanged; register non-participation is a census-role decision, not a scanning one, and is recorded here so the whole judgment is visible in one place |
//! | `forbidden::*` | none: `git grep -F` over every tracked file | unchanged, deliberately. The forbidden-token ban is total by policy — a banned token is banned inside a fence and inside a string literal too — so this audit is participation-blind by design, not by omission |
//!
//! # Sites that remain participation-blind
//!
//! These read raw text today and are recorded, not changed, because
//! their subjects are not label occurrences and rewiring them would
//! alter what they report:
//!
//! - `plans::check_file` scaffolding, placeholder, confidence,
//!   deleted-path, and machine-input markers scan the whole file. Only
//!   the Markdown link check is fence-aware.
//! - `plans::verify_task_status_agreement` walks raw backlog lines for
//!   table rows and task headings.
//!
//! # What is deliberately not here
//!
//! Span *grammar* — whether a participating span is a mint, a
//! parenthesized citation, or an unparenthesized import — belongs to
//! [`crate::markdown::InlineCodeContext`] and the harvesters. Participation
//! decides only whether the grammar gets to look at the span at all.

use std::path::Path;

use crate::{
    diagnostic::{LabelDiagnostic, LabelErrorCode},
    markdown::{InlineCodeContext, InlineCodeSpan, classify},
    source::SourceLocation,
};

/// Why a stretch of source is displayed rather than participating.
///
/// Recorded so a caller can explain a skip, and so the reasons stay a
/// closed set: a new way of displaying material must be added here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Displayed {
    /// Inside a fenced block, or the fence marker line itself.
    Fence,
    /// A span delimited by two or more backticks: shown, not meant.
    DoubleBacktick,
}

/// Participation role of one physical line of a prose source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineRole {
    /// Authored prose: spans on this line are parsed.
    Participating,
    /// A fence marker, or a line inside a fenced block.
    Fenced,
}

impl LineRole {
    /// The reason this line is displayed, or `None` when it
    /// participates.
    #[must_use]
    pub const fn displayed(self) -> Option<Displayed> {
        match self {
            Self::Participating => None,
            Self::Fenced => Some(Displayed::Fence),
        }
    }
}

/// The line-level participation view of one prose source.
///
/// Fenced blocks are the only construct that spans lines, so the
/// logical regions the calculus asks for are exactly the maximal runs
/// of participating lines; a span never crosses a fence boundary
/// because a fence marker line is itself displayed. Callers that need
/// spans go through [`crate::markdown::scan_markdown`], which is built
/// on this view; callers that are line-oriented use
/// [`Self::participates`] or [`Self::blanked`].
#[derive(Clone, Debug, Default)]
pub struct ProseParticipation {
    roles: Vec<LineRole>,
    /// One-based line of an opening fence that never closed, if any.
    /// An unclosed fence leaves every following line displayed, which
    /// is a hard failure of the file rather than a silent truncation,
    /// so the opening line is retained for the diagnostic.
    unclosed_fence: Option<usize>,
}

impl ProseParticipation {
    /// Classify every line of `source`.
    #[must_use]
    pub fn of(source: &str) -> Self {
        let mut roles = Vec::new();
        let mut fence: Option<(char, usize, usize)> = None;
        for (index, raw) in source.lines().enumerate() {
            if let Some((marker, length, _)) = fence {
                if fence_close(raw, marker, length) {
                    fence = None;
                }
                roles.push(LineRole::Fenced);
                continue;
            }
            if let Some((marker, length)) = fence_open(raw) {
                fence = Some((marker, length, index + 1));
                roles.push(LineRole::Fenced);
                continue;
            }
            roles.push(LineRole::Participating);
        }
        Self {
            roles,
            unclosed_fence: fence.map(|(_, _, line)| line),
        }
    }

    /// Whether the one-based `line` carries authored prose.
    ///
    /// A line past the end of the source does not participate: there is
    /// no authored text there to mean anything.
    #[must_use]
    pub fn participates(&self, line: usize) -> bool {
        line.checked_sub(1)
            .and_then(|index| self.roles.get(index))
            .is_some_and(|role| *role == LineRole::Participating)
    }

    /// The role of the one-based `line`, or `None` past the end.
    #[must_use]
    pub fn role(&self, line: usize) -> Option<LineRole> {
        line.checked_sub(1)
            .and_then(|index| self.roles.get(index))
            .copied()
    }

    /// The one-based line of an opening fence that never closed.
    #[must_use]
    pub const fn unclosed_fence(&self) -> Option<usize> {
        self.unclosed_fence
    }

    /// `source` with every displayed line blanked, so a line-oriented
    /// or whole-text check skips displayed material while every
    /// participating line keeps its original line number.
    #[must_use]
    pub fn blanked(&self, source: &str) -> String {
        source
            .lines()
            .enumerate()
            .map(|(index, raw)| {
                if self.participates(index + 1) {
                    raw
                } else {
                    ""
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl InlineCodeSpan {
    /// Whether this span participates.
    ///
    /// A span reaches here only from a participating line, so the one
    /// remaining question is delimiter width: the calculus places a
    /// token that is shown but not meant in a double-backtick span, so
    /// exactly the single-backtick spans participate.
    #[must_use]
    pub const fn participates(&self) -> bool {
        self.delimiter_len == 1
    }

    /// Why this span is displayed, or `None` when it participates.
    #[must_use]
    pub const fn displayed(&self) -> Option<Displayed> {
        if self.participates() {
            None
        } else {
            Some(Displayed::DoubleBacktick)
        }
    }
}

// ---------------------------------------------------------------------
// LaTeX front-end.
// ---------------------------------------------------------------------

/// `source` with every LaTeX comment removed, line structure intact.
///
/// LaTeX inverts the prose rule: a percent sign begins a comment that
/// runs to end of line, and a comment is exactly the material that does
/// not participate. An escaped percent is literal text, so escaping is
/// counted rather than merely looked for one character back — the
/// backslash run's parity decides, since a doubled backslash escapes
/// itself and leaves the percent live.
///
/// The format has no fenced or double-delimited displayed material, so
/// what survives this strip is the whole participating region and the
/// line view has nothing further to say about it.
#[must_use]
pub fn latex_participating(source: &str) -> String {
    source.lines().fold(String::new(), |mut output, line| {
        let mut slash = 0;
        let mut end = line.len();
        for (index, character) in line.char_indices() {
            if character == '\\' {
                slash += 1;
            } else if character == '%' && slash % 2 == 0 {
                end = index;
                break;
            } else {
                slash = 0;
            }
        }
        output.push_str(&line[..end]);
        output.push('\n');
        output
    })
}

// ---------------------------------------------------------------------
// Fence recognition: the single grammar every front-end shares.
// ---------------------------------------------------------------------

/// Recognize a fence opener, returning its marker and run length.
///
/// The accepted Markdown grammar admits at most three leading spaces
/// (ADR-019 fenced material); anything deeper is a container, handled
/// by [`nested_fence`].
#[must_use]
pub fn fence_open(line: &str) -> Option<(char, usize)> {
    let trimmed = after_fence_indent(line)?;
    let marker = trimmed.chars().next()?;
    if !matches!(marker, '`' | '~') {
        return None;
    }
    let length = trimmed.chars().take_while(|value| *value == marker).count();
    (length >= 3).then_some((marker, length))
}

/// Whether `line` closes a fence opened with `marker` repeated
/// `length` times.
#[must_use]
pub fn fence_close(line: &str, marker: char, length: usize) -> bool {
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

/// Detect a fence the top-level recognizer cannot see, returning the
/// one-based column of its marker run.
///
/// A fence hidden behind a blockquote marker, a list bullet, or deeper
/// indentation is outside the accepted grammar, so its content would be
/// scanned as ordinary prose. Rather than grow a container parser, the
/// repository rejects such a fence: a label-shaped token inside one can
/// then never participate silently, because the document carrying it
/// fails.
#[must_use]
pub fn nested_fence(line: &str) -> Option<usize> {
    if fence_open(line).is_some() {
        return None;
    }
    let bytes = line.as_bytes();
    let mut offset = 0;
    let mut contained = false;
    loop {
        let indent = offset;
        while bytes.get(offset) == Some(&b' ') {
            offset += 1;
        }
        // Indentation deeper than a top-level fence is itself a
        // container: a list continuation or an indented block.
        if offset - indent > 3 {
            contained = true;
        }
        match bytes.get(offset) {
            Some(b'>') => {
                offset += 1;
                contained = true;
            }
            Some(b'-' | b'*' | b'+') if bytes.get(offset + 1) == Some(&b' ') => {
                offset += 2;
                contained = true;
            }
            Some(digit) if digit.is_ascii_digit() => {
                let mut end = offset;
                while bytes.get(end).is_some_and(u8::is_ascii_digit) {
                    end += 1;
                }
                if !matches!(bytes.get(end), Some(b'.' | b')')) || bytes.get(end + 1) != Some(&b' ')
                {
                    break;
                }
                offset = end + 2;
                contained = true;
            }
            _ => break,
        }
    }
    if !contained {
        return None;
    }
    let rest = &line[offset..];
    let marker = rest.chars().next()?;
    if !matches!(marker, '`' | '~') {
        return None;
    }
    let length = rest.chars().take_while(|value| *value == marker).count();
    (length >= 3).then(|| line[..offset].chars().count() + 1)
}

// ---------------------------------------------------------------------
// Rust front-end: the scanned-region recognition.
//
// The calculus fixes "which regions count as comments in each language"
// at adoption and then consumes it without asking about its provenance.
// For Rust that recognition is exactly the segmentation below: comments
// and documentation comments are scanned, while string literals (plain,
// raw, byte, and C), character literals, and lifetimes are code and are
// skipped whole. A label-bearing test fixture inside a string literal
// therefore mints nothing.
//
// Fenced documentation examples are excluded on top of this, by the
// harvester, using the fence recognition above.
// ---------------------------------------------------------------------

/// Syntactic class of one comment, retained so fence handling can
/// distinguish documentation from ordinary comments (ADR-019 excludes
/// fenced Rustdoc examples, not arbitrary comment text).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommentKind {
    OrdinaryLine,
    OuterDocLine,
    InnerDocLine,
    OrdinaryBlock,
    OuterDocBlock,
    InnerDocBlock,
}

impl CommentKind {
    #[must_use]
    pub const fn is_documentation(self) -> bool {
        matches!(
            self,
            Self::OuterDocLine | Self::InnerDocLine | Self::OuterDocBlock | Self::InnerDocBlock
        )
    }
}

/// One comment text run on one source line, positioned by the first
/// character after the comment marker.
pub struct CommentSegment {
    pub line: usize,
    pub column: usize,
    pub text: String,
    pub kind: CommentKind,
    /// Contiguous comment-block identity: consecutive same-kind line
    /// comments with no intervening code share a block; every block
    /// comment is its own block. A documentation fence never outlives
    /// its block.
    pub block: usize,
}

/// Extract comment and documentation-comment text from Rust source.
///
/// Labels participate only in comments (ADR-019 Rust-sources rule):
/// string literals, raw strings, byte/C strings, and character
/// literals are code, so an acute span inside them — for example a
/// label-bearing test fixture — must not mint or cite.
pub fn comment_segments(
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
    // Block-identity bookkeeping: consecutive same-kind line comments
    // with no intervening code continue one block; anything else opens
    // a new one.
    let mut next_block = 0_usize;
    let mut previous_line_comment: Option<(usize, CommentKind, usize)> = None;
    let mut code_since_comment = false;
    while i < chars.len() {
        match chars[i] {
            '/' if chars.get(i + 1) == Some(&'/') => {
                // Line comment: classify the marker run before
                // consuming it. Exactly three slashes are outer
                // documentation; `//!` is inner documentation; two or
                // four-plus slashes are ordinary comments (rustdoc
                // treats `////` as ordinary).
                let slashes = chars[i..]
                    .iter()
                    .take_while(|character| **character == '/')
                    .count();
                let kind = if slashes == 3 {
                    CommentKind::OuterDocLine
                } else if slashes == 2 && chars.get(i + 2) == Some(&'!') {
                    CommentKind::InnerDocLine
                } else {
                    CommentKind::OrdinaryLine
                };
                // Skip the marker run (`//`, `///`, `//!`) and capture
                // to end of line.
                while i < chars.len() && (chars[i] == '/' || chars[i] == '!') {
                    advance(&mut i, &mut line, &mut column);
                }
                let (start_line, start_column) = (line, column);
                let block = match previous_line_comment {
                    Some((previous_line, previous_kind, block))
                        if previous_kind == kind
                            && start_line == previous_line + 1
                            && !code_since_comment =>
                    {
                        block
                    }
                    _ => {
                        next_block += 1;
                        next_block
                    }
                };
                previous_line_comment = Some((start_line, kind, block));
                code_since_comment = false;
                let mut text = String::new();
                while i < chars.len() && chars[i] != '\n' {
                    text.push(chars[i]);
                    advance(&mut i, &mut line, &mut column);
                }
                segments.push(CommentSegment {
                    line: start_line,
                    column: start_column,
                    text,
                    kind,
                    block,
                });
            }
            '/' if chars.get(i + 1) == Some(&'*') => {
                // Block comment; Rust block comments nest, and every
                // `/*` / `*/` consumes both characters so overlapping
                // sequences like `*/*` cannot be double-counted.
                // Classify before consuming: `/*!` is inner
                // documentation; `/**` is outer documentation unless it
                // is `/**/` (empty) or `/***` (ordinary, like `////`).
                let kind = match chars.get(i + 2) {
                    Some('!') => CommentKind::InnerDocBlock,
                    Some('*')
                        if chars.get(i + 3) != Some(&'*') && chars.get(i + 3) != Some(&'/') =>
                    {
                        CommentKind::OuterDocBlock
                    }
                    _ => CommentKind::OrdinaryBlock,
                };
                next_block += 1;
                let block = next_block;
                code_since_comment = false;
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
                                kind,
                                block,
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
                            kind,
                            block,
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
            '"' => {
                code_since_comment = true;
                skip_string(&chars, &mut i, &mut line, &mut column, &mut advance);
            }
            'b' | 'c' | 'r' => {
                code_since_comment = true;
                if let Some(prefix) = string_prefix(&chars, i) {
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
                } else {
                    advance(&mut i, &mut line, &mut column);
                }
            }
            '\'' => {
                code_since_comment = true;
                skip_char_or_lifetime(&chars, &mut i, &mut line, &mut column, &mut advance);
            }
            // Everything else — including identifier runs — advances
            // one character; `string_prefix`'s lookbehind already
            // rejects prefix letters inside identifiers. Whitespace is
            // not code for block continuity; anything else is.
            _ => {
                if !chars[i].is_whitespace() {
                    code_since_comment = true;
                }
                advance(&mut i, &mut line, &mut column);
            }
        }
    }
    segments
}

// ---------------------------------------------------------------------
// Python front-end: the scanned-region recognition of ADR-023.
//
// Python's rule is the Rust rule with a smaller grammar under it. A
// number sign opens a comment that runs to end of line, and string
// literals are code: single-, double-, and triple-quoted, with or
// without a prefix, and docstrings among them, since a docstring is a
// string literal in an expression statement and nothing lexical tells
// it from any other string. The executor this record was written for
// carries protocol text, RPC arguments, and fixture strings, so scanning
// literals would manufacture citations out of data.
//
// Three simplifications hold because Python is not Rust, and each is a
// narrowing of what the scanner must get right rather than an
// assumption about the source:
//
//   1. A quote is always a string delimiter. Python has no character
//      literal and no lifetime, so the Rust lookbehind that keeps a
//      prefix letter out of an identifier has nothing to do here: the
//      quote itself is the trigger, whatever letters precede it.
//   2. A backslash escapes the next character in every string form,
//      raw included. A raw string keeps the backslash in its value but
//      still cannot be terminated by the quote after it, so delimiter
//      pairing needs no raw/cooked distinction.
//   3. There is no block comment and so no nesting.
//
// Two deliberate narrownesses, stated rather than repaired. A
// single-quoted string that reaches end of line is closed there: it is
// a syntax error in Python, and ending it at the newline keeps one
// stray quote from swallowing the rest of the file. An unterminated
// triple-quoted string runs to end of file, as an unterminated Rust
// string does. Neither is diagnosed here; a file in that state does not
// run, and the label check is not Python's parser.
// ---------------------------------------------------------------------

/// Extract comment text from Python source (ADR-023 Python rule).
///
/// Every segment is [`CommentKind::OrdinaryLine`]: Python has one
/// comment form, and this repository claims no documentation-comment
/// convention over runs of number signs. Block identity follows the
/// Rust rule — consecutive comment lines with no code between them
/// continue one region.
#[must_use]
pub fn python_comment_segments(source: &str) -> Vec<CommentSegment> {
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
    let mut next_block = 0_usize;
    let mut previous_line_comment: Option<(usize, usize)> = None;
    let mut code_since_comment = false;
    while i < chars.len() {
        match chars[i] {
            '#' => {
                // One number sign is the marker; any further ones are
                // the comment's own text.
                advance(&mut i, &mut line, &mut column);
                let (start_line, start_column) = (line, column);
                let block = match previous_line_comment {
                    Some((previous_line, block))
                        if start_line == previous_line + 1 && !code_since_comment =>
                    {
                        block
                    }
                    _ => {
                        next_block += 1;
                        next_block
                    }
                };
                previous_line_comment = Some((start_line, block));
                code_since_comment = false;
                let mut text = String::new();
                while i < chars.len() && chars[i] != '\n' {
                    text.push(chars[i]);
                    advance(&mut i, &mut line, &mut column);
                }
                segments.push(CommentSegment {
                    line: start_line,
                    column: start_column,
                    text,
                    kind: CommentKind::OrdinaryLine,
                    block,
                });
            }
            quote @ ('"' | '\'') => {
                code_since_comment = true;
                let triple = chars.get(i + 1) == Some(&quote) && chars.get(i + 2) == Some(&quote);
                let width = if triple { 3 } else { 1 };
                for _ in 0..width {
                    advance(&mut i, &mut line, &mut column);
                }
                skip_python_string(
                    &chars,
                    &mut i,
                    &mut line,
                    &mut column,
                    &mut advance,
                    quote,
                    triple,
                );
            }
            _ => {
                if !chars[i].is_whitespace() {
                    code_since_comment = true;
                }
                advance(&mut i, &mut line, &mut column);
            }
        }
    }
    segments
}

/// Consume a Python string literal whose opening delimiter is already
/// past, stopping after its closing delimiter.
fn skip_python_string(
    chars: &[char],
    i: &mut usize,
    line: &mut usize,
    column: &mut usize,
    advance: &mut impl FnMut(&mut usize, &mut usize, &mut usize),
    quote: char,
    triple: bool,
) {
    while *i < chars.len() {
        if chars[*i] == '\\' {
            // The escape and whatever follows it, newline included: a
            // backslash-continued line stays inside the literal.
            advance(i, line, column);
            if *i < chars.len() {
                advance(i, line, column);
            }
            continue;
        }
        if chars[*i] == quote {
            if !triple {
                advance(i, line, column);
                return;
            }
            if chars.get(*i + 1) == Some(&quote) && chars.get(*i + 2) == Some(&quote) {
                for _ in 0..3 {
                    advance(i, line, column);
                }
                return;
            }
        }
        if chars[*i] == '\n' && !triple {
            // Unterminated single-quoted string: closed at the newline
            // rather than allowed to swallow the file.
            return;
        }
        advance(i, line, column);
    }
}

// ---------------------------------------------------------------------
// The shared region walk over located acute spans.
// ---------------------------------------------------------------------

/// One acute span of a scanned code region, located and classified.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegionSpan {
    /// The span's interior, both acute delimiters removed.
    pub value: String,
    pub location: SourceLocation,
    /// Whether the immediate syntactic group parenthesizes the span,
    /// which is what separates a citation from a mint.
    pub parenthesized: bool,
}

/// Locate and classify the acute spans of one logical comment region.
///
/// The region's text is its segments joined by newlines, the comment
/// leaders already resolved away by the front-end. Joining on a newline
/// keeps pairing region-wide, as the calculus asks, while never
/// fabricating a label out of two lines: no label carries a newline, so
/// an acute whose candidate interior crosses a line break opens
/// nothing.
///
/// Citation-defect and unclosed-delimiter diagnostics are pushed here;
/// what a well-formed span means is the caller's, and differs by owner.
#[must_use]
pub fn region_spans(
    path: &Path,
    region: &[&CommentSegment],
    diagnostics: &mut Vec<LabelDiagnostic>,
) -> Vec<RegionSpan> {
    let mut located = Vec::new();
    if region.is_empty() {
        return located;
    }
    let mut text = String::new();
    let mut starts = Vec::new();
    for segment in region {
        if !text.is_empty() {
            text.push('\n');
        }
        starts.push((text.len(), segment.line, segment.column));
        text.push_str(&segment.text);
    }
    let scan = acute_scan(&text);
    // A region byte offset back to the position it came from: the
    // segment it falls in, advanced by the characters before it.
    let locate = |offset: usize| {
        let (start, line, column) = starts
            .iter()
            .rev()
            .find(|(start, _, _)| *start <= offset)
            .copied()
            .unwrap_or((0, 1, 1));
        SourceLocation::new(path, line, column + text[start..offset].chars().count())
    };
    for (start, after) in scan.spans.iter().copied() {
        let location = locate(start);
        // Classification parses the immediate syntactic group. An
        // attempted citation whose group is malformed is diagnosed,
        // never demoted to a bare mint: a dangling citation must not
        // be able to self-satisfy by minting the label it cites.
        //
        // The group is read within the span's own line, not the whole
        // region: distinguishing a dropped parenthesis from a malformed
        // group turns on whether a partner is in sight, and a
        // parenthesis a line away is not. Every span lies inside one
        // line, since an interior carrying a newline is not
        // label-shaped and so opens nothing.
        let line_start = text[..start].rfind('\n').map_or(0, |offset| offset + 1);
        let line_end = text[after..]
            .find('\n')
            .map_or(text.len(), |offset| after + offset);
        let line = &text[line_start..line_end];
        let ranges = scan
            .spans
            .iter()
            .filter(|(span, _)| (line_start..line_end).contains(span))
            .map(|(span, end)| (span - line_start, end - line_start))
            .collect::<Vec<_>>();
        let context = classify(line, start - line_start, after - line_start, &ranges);
        if let Some((code, message)) = context.defect("label citation") {
            diagnostics.push(LabelDiagnostic::error(code, &location, message));
            continue;
        }
        located.push(RegionSpan {
            value: text[start + ACUTE.len_utf8()..after - ACUTE.len_utf8()].to_owned(),
            location,
            parenthesized: context == InlineCodeContext::Parenthesized,
        });
    }
    if let Some(offset) = scan.unclosed {
        diagnostics.push(LabelDiagnostic::error(
            LabelErrorCode::UnclosedInlineCode,
            &locate(offset),
            "unclosed acute label delimiter",
        ));
    }
    located
}

// ---------------------------------------------------------------------
// Acute span model: the code syntax's delimiter classification.
// ---------------------------------------------------------------------

/// The acute accent, which carries the label syntax in scanned code
/// text as the backtick carries it in prose.
pub const ACUTE: char = '\u{b4}';

/// Whether text is shaped like a label or a bracketed import, which is
/// the condition under which an acute opens.
///
/// The test is lexical silhouette, never resolution: a token that is
/// trying to be a label opens a span and is then diagnosed by the
/// grammar if it parses as nothing, while text that is not trying is
/// left alone. Whitespace, an empty interior, characters outside the
/// label alphabet, and the absence of a colon each disqualify — which
/// is what separates a real occurrence from an apostrophe accident.
#[must_use]
pub fn label_shaped_text(value: &str) -> bool {
    let core = value
        .strip_prefix('[')
        .and_then(|inner| inner.strip_suffix(']'))
        .unwrap_or(value);
    !core.is_empty()
        && core.contains(':')
        && core.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == ':' || character == '-'
        })
}

/// The acute-delimited spans of one scanned code region.
#[derive(Clone, Debug, Default)]
pub struct AcuteScan {
    /// Each span as (byte offset of the opening acute, byte offset just
    /// past the closing acute), in source order.
    pub spans: Vec<(usize, usize)>,
    /// Byte offset of an opening acute whose region ended before it
    /// closed. An opening acute declares intent to mint or cite, so its
    /// loss is a hard failure rather than a silent demotion to text.
    pub unclosed: Option<usize>,
}

/// Pair the acute delimiters of one logical code region.
///
/// The acute classifies locally: it opens exactly when label-shaped
/// text follows it, an opening acute unclosed when the region ends is a
/// hard failure, and an acute that opens nothing is ordinary text. The
/// candidate interior runs to the next acute, or to the end of the
/// region when none follows — so the opening decision never depends on
/// a delimiter being found, which is what lets an unclosed opener be
/// reported at all. A stray closing acute opens nothing and is
/// therefore silent: it is overwhelmingly an apostrophe accident.
#[must_use]
pub fn acute_scan(text: &str) -> AcuteScan {
    let mut scan = AcuteScan::default();
    let mut cursor = 0;
    while let Some(relative) = text[cursor..].find(ACUTE) {
        let open = cursor + relative;
        let interior = open + ACUTE.len_utf8();
        let next = text[interior..].find(ACUTE).map(|offset| interior + offset);
        if label_shaped_text(&text[interior..next.unwrap_or(text.len())]) {
            if let Some(close) = next {
                scan.spans.push((open, close + ACUTE.len_utf8()));
                cursor = close + ACUTE.len_utf8();
            } else {
                // An opener whose region ended first: intent to mint or
                // cite, lost. The scan stops at the failure.
                scan.unclosed = Some(open);
                return scan;
            }
        } else {
            // Opens nothing, so it is text; the next acute is still a
            // candidate opener in its own right.
            cursor = interior;
        }
    }
    scan
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
