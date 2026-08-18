//! The one participation scanner every label check consumes.
//!
//! # Why this module exists
//!
//! The calculus defines participation once (`judg:labels:participation`):
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
//! (recorded as DI-F01). The survey below is the refactor's contract:
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
//! | `repository::harvest_realization` | `delimiter_len != 1` skip, restated inline | [`MarkdownScan::participating_spans`] |
//! | `repository::harvest_adrs` | `delimiter_len != 1` skip, restated inline | [`MarkdownScan::participating_spans`] |
//! | `repository::harvest_markdown_owner` | `delimiter_len != 1` skip, restated inline | [`MarkdownScan::participating_spans`] |
//! | `repository::harvest_attestation_citations` | `delimiter_len != 1` skip, restated inline; **and** computed the generated-index region boundary by walking raw `source.lines()` with no fence awareness | [`MarkdownScan::participating_spans`]; the region walk now consults [`ProseParticipation::participates`] (DI-F01 resolution 1) |
//! | `plans::without_fenced_lines` | a second fence open/close loop, blanking fenced lines for the link check only | [`ProseParticipation::blanked`] |
//! | `rust_source::comment_segments` | comment/literal segmentation for Rust: the scanned-region recognition | unchanged in place, documented here as the Rust front-end; its fence handling calls this module's [`fence_open`]/[`fence_close`]/[`nested_fence`] |
//! | `latex::comments` | strips percent comments, honouring backslash escaping: the LaTeX front-end | unchanged in place, documented here; LaTeX has no fenced or double-delimited displayed material, so its region model is total |
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
//! [`crate::markdown::classify`] and the harvesters. Participation
//! decides only whether the grammar gets to look at the span at all.

use crate::markdown::InlineCodeSpan;

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
