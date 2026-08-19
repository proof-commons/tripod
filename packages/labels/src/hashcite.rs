//! Hash-citation accounting over the tracked tree.
//!
//! Every hexadecimal value the repository tracks should be describable:
//! something writes it, and it measures something. This module finds
//! those values and asks a committed rule table which one of them each
//! occurrence is. A value no rule describes is the audit's single
//! failure mode, and it is the one worth having — a new digest arriving
//! with no named generator is the moment the tree stops being able to
//! say where its own numbers come from.
//!
//! The audit places; it does not recompute. Whether a manifest digest is
//! *correct* is the manifest's own test's job, and an audit that
//! recomputed it would duplicate that suite and then fail for reasons
//! that are not lint failures. What this establishes is narrower and not
//! covered anywhere else: that every value has an owner on the record.
//!
//! Growth is one-way. Rules are added when a value needs one and are
//! never removed when the tree stops carrying that value, because a rule
//! is a standing claim about a kind of value rather than an inventory of
//! today's occurrences. That is what makes the failure legible: the
//! audit can only ever fail as "a value nothing describes", never as "a
//! value that used to be allowed".

use std::collections::BTreeSet;

use regex::Regex;
use serde::Serialize;

/// Report schema version for the hash-citation audit.
pub const HASH_CITATION_SCHEMA: u32 = 1;

/// Shortest hexadecimal run read as a value.
///
/// Seven, because seven is what git abbreviates to by default: an
/// eight-digit floor cannot see the commonest short form there is.
pub const VALUE_FLOOR: usize = 7;

/// Longest hexadecimal run read as a value — past this a run is no
/// digest this tree writes.
pub const VALUE_CEILING: usize = 64;

/// Whether a rule's subject belongs to this repository or to another.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Group {
    /// The value derives from this repository's own material.
    Internal,
    /// The value names something outside this repository.
    External,
}

/// A token form a rule may argue from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shape {
    /// Drawn from a handful of characters, so plainly not a measurement.
    LowEntropy,
    /// The underscored `0x` form fingerprints are written for reading in.
    Underscored,
    /// Embedded in a name rather than standing free as a word.
    Filename,
}

/// How a rule decides whether it claims an occurrence.
#[derive(Debug, Clone)]
enum Matcher {
    /// The path matches a glob.
    PathGlob(String),
    /// The path matches a glob and the line's first word is this key.
    PathContext(String, String),
    /// The line's first word is this key.
    ContextKey(String),
    /// The line's first word matches this pattern in full.
    ContextKeyPattern(Regex),
    /// This pattern occurs somewhere in the line.
    Context(Regex),
    /// The value itself, compared without regard to case.
    Token(String),
    /// One value in one file: the narrowest claim a rule can make.
    Site(String, String),
    /// An abbreviation of this value: the citation names it at any length.
    ///
    /// A commit is cited at whatever length the sentence around it wanted, so a
    /// rule naming one has to admit its abbreviations. It admits only genuine
    /// prefixes of the full value it states, which is why the rule carries the
    /// whole thing: a reader resolves what the rule names, not what the
    /// abbreviation happens to match.
    PrefixOf(String),
    /// The value has this form.
    Shape(Shape),
}

/// One row of the families table.
#[derive(Debug, Clone)]
pub struct Rule {
    /// Stable identifier, reported when the rule claims an occurrence.
    pub id: String,
    /// This repository's material, or another project's.
    pub group: Group,
    /// Where the value's correctness is established — never here.
    pub verification: String,
    /// What the value measures.
    pub referent: String,
    /// What writes it.
    pub generation_program: String,
    /// Why the row reads as it does.
    pub notes: String,
    matcher: Matcher,
}

/// A condition that stops the table being read at all.
#[derive(Debug, thiserror::Error)]
pub enum RuleTableError {
    /// The first line does not name the eight columns.
    #[error("the header must name the eight columns, beginning with `id`")]
    Header,
    /// A row has the wrong number of fields.
    #[error("line {line}: expected 8 tab-separated columns, found {found}")]
    Columns {
        /// One-based line number in the table.
        line: usize,
        /// Column count actually found.
        found: usize,
    },
    /// A row names a match kind this audit does not implement.
    #[error("line {line}: unknown match kind `{kind}`")]
    MatchKind {
        /// One-based line number in the table.
        line: usize,
        /// The unknown kind as written.
        kind: String,
    },
    /// A row's group is neither INTERNAL nor EXTERNAL.
    #[error("line {line}: group must be INTERNAL or EXTERNAL, found `{group}`")]
    Group {
        /// One-based line number in the table.
        line: usize,
        /// The group as written.
        group: String,
    },
    /// A row names a token form this audit does not implement.
    #[error("line {line}: unknown shape `{shape}`")]
    Shape {
        /// One-based line number in the table.
        line: usize,
        /// The unknown shape as written.
        shape: String,
    },
    /// An INTERNAL row leaves its generation program unstated.
    #[error("line {line}: an INTERNAL rule must name its generation program")]
    MissingGenerator {
        /// One-based line number in the table.
        line: usize,
    },
    /// A row's pattern does not compile.
    #[error("line {line}: `{value}` is not a usable pattern: {message}")]
    Pattern {
        /// One-based line number in the table.
        line: usize,
        /// The pattern as written.
        value: String,
        /// What the pattern compiler said about it.
        message: String,
    },
    /// A path glob uses a construct this audit deliberately does not read.
    #[error("line {line}: a path glob may use only `*` and `?`")]
    GlobSyntax {
        /// One-based line number in the table.
        line: usize,
    },
    /// A `pathctx` row does not separate its glob from its key.
    #[error("line {line}: a pathctx value must be written `<glob>::<key>`")]
    PathContextSyntax {
        /// One-based line number in the table.
        line: usize,
    },
    /// A `site` row does not separate its path from its value.
    #[error("line {line}: a site value must be written `<path>@<value>`")]
    SiteSyntax {
        /// One-based line number in the table.
        line: usize,
    },
    /// Two rows share an identifier, so a report could not say which claimed a value.
    #[error("line {line}: duplicate rule id `{id}`")]
    DuplicateId {
        /// One-based line number in the table.
        line: usize,
        /// The repeated identifier.
        id: String,
    },
    /// A rule declares no way for a reader to establish the value.
    ///
    /// Describing a value establishes that somebody owns it. It does not
    /// establish that anybody can check it, and the two are different: a rule
    /// could name a generator that no longer exists, and this audit would pass
    /// a value nothing regenerates because a rule described it. So the
    /// verification column is a closed vocabulary of things a reader can do,
    /// and a word outside it is refused rather than recorded.
    #[error(
        "line {line}: `{verification}` is not a way a reader establishes a value; \
         a rule may not describe a value as a digest or identifier that nothing \
         regenerates and nothing resolves"
    )]
    UndeclaredVerification {
        /// One-based line number in the table.
        line: usize,
        /// The word as written.
        verification: String,
    },
    /// The table declares nothing, which would excuse nothing.
    #[error("the table declares no rules")]
    Empty,
}

/// The ways a reader can establish a value, and the whole of them.
///
/// Each word names something the reader does: recompute it from bytes this
/// tree carries under an algorithm this tree names, read it back out of a
/// transaction, verify a signature against it, decode it, resolve it, or read
/// the definition that fixes an authored input. A rule declaring anything else
/// is declaring that the value cannot be checked, which is the one thing a rule
/// may not say.
const VERIFICATIONS: &[&str] = &[
    "RECOMPUTABLE",
    "CARRIED",
    "SIGNATURE-VERIFIED",
    "HEX-TEXT-DECODES",
    "RESOLVABLE",
    "SOURCE-STATED",
    "VECTORS-SUITE",
    "MODULE-TEST",
    "CARGO-TEST",
    "MODEL-EXPORT",
    "DOCUMENT-BUILD",
    "EXTERNAL-ELEMENTS",
    "DERIVED",
    "AUTHORED",
];

/// One hexadecimal value found at one place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Occurrence {
    /// Repository-relative path of the file the value sits in.
    pub path: String,
    /// One-based line number, or zero when the value is part of the name.
    pub line: u32,
    /// Zero-based character offset of the value on its line.
    pub column: u32,
    /// The value as written.
    pub token: String,
    /// The whitespace-delimited word the value sits inside.
    pub word: String,
    /// The line, trimmed — or the path, for a value in a name.
    pub context: String,
    /// The form the value takes, reported so a refusal is legible.
    pub shape: &'static str,
}

/// One occurrence no rule described.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HashCitationRefusal {
    /// Repository-relative path of the file the value sits in.
    pub path: String,
    /// One-based line number, or zero when the value is part of the name.
    pub line: u32,
    /// Zero-based character offset of the value on its line.
    pub column: u32,
    /// The value as written.
    pub value: String,
    /// The form the value takes.
    pub shape: String,
}

/// How much of the tree one rule accounted for.
///
/// Published per rule rather than summed, because the distribution is
/// the reviewable fact: a rule that suddenly claims an order of
/// magnitude more than it did is a rule that has stopped meaning what
/// its row says, and no total would show that.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuleClaims {
    /// The rule's identifier, as the table spells it.
    pub id: String,
    /// Occurrences this rule was the first to claim.
    pub occurrences: usize,
}

/// The hash-citation audit result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HashCitationReport {
    /// Report schema version.
    pub schema: u32,
    /// Tracked files read as text.
    pub files_scanned: usize,
    /// Occurrences the scan found.
    pub occurrences: usize,
    /// Occurrences a rule claimed.
    pub placed: usize,
    /// Every rule in table order, with what it claimed. A rule showing
    /// zero is reported and never enforced.
    pub claims: Vec<RuleClaims>,
    /// Occurrences no rule described (empty on a clean tree).
    pub refusals: Vec<HashCitationRefusal>,
    /// True only when every occurrence is described by a rule.
    pub valid: bool,
}

// -------------------------------------------------------------------------
// the table
// -------------------------------------------------------------------------

/// Read the families table.
///
/// Comment and blank lines are skipped; every other line must carry the
/// eight columns exactly. A malformed row is an error rather than a
/// skipped line, because a rule table that silently drops what it cannot
/// read would excuse by accident.
pub fn load_rules(text: &str) -> Result<Vec<Rule>, RuleTableError> {
    let mut lines = text.lines().enumerate();
    let Some((_, header)) = lines.next() else {
        return Err(RuleTableError::Header);
    };
    if !header.starts_with("id\t") {
        return Err(RuleTableError::Header);
    }

    let mut rules = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, raw) in lines {
        let line = index + 1;
        if raw.trim().is_empty() || raw.starts_with('#') {
            continue;
        }
        let fields: Vec<_> = raw.split('\t').collect();
        if fields.len() != 8 {
            return Err(RuleTableError::Columns {
                line,
                found: fields.len(),
            });
        }
        let rule = parse_rule(line, &fields)?;
        if !seen.insert(rule.id.clone()) {
            return Err(RuleTableError::DuplicateId { line, id: rule.id });
        }
        rules.push(rule);
    }
    if rules.is_empty() {
        return Err(RuleTableError::Empty);
    }
    Ok(rules)
}

fn parse_rule(line: usize, fields: &[&str]) -> Result<Rule, RuleTableError> {
    let group = match fields[3] {
        "INTERNAL" => Group::Internal,
        "EXTERNAL" => Group::External,
        other => {
            return Err(RuleTableError::Group {
                line,
                group: other.to_owned(),
            });
        }
    };
    if group == Group::Internal && fields[6].trim().is_empty() {
        return Err(RuleTableError::MissingGenerator { line });
    }
    if !VERIFICATIONS.contains(&fields[4]) {
        return Err(RuleTableError::UndeclaredVerification {
            line,
            verification: fields[4].to_owned(),
        });
    }
    Ok(Rule {
        id: fields[0].to_owned(),
        group,
        verification: fields[4].to_owned(),
        referent: fields[5].to_owned(),
        generation_program: fields[6].to_owned(),
        notes: fields[7].to_owned(),
        matcher: parse_matcher(line, fields[1], fields[2])?,
    })
}

fn parse_matcher(line: usize, kind: &str, value: &str) -> Result<Matcher, RuleTableError> {
    match kind {
        "pathglob" => {
            check_glob(line, value)?;
            Ok(Matcher::PathGlob(value.to_owned()))
        }
        "pathctx" => {
            let (glob, key) = value
                .rsplit_once("::")
                .ok_or(RuleTableError::PathContextSyntax { line })?;
            check_glob(line, glob)?;
            Ok(Matcher::PathContext(glob.to_owned(), key.to_owned()))
        }
        "ctxkey" => Ok(Matcher::ContextKey(value.to_owned())),
        // Anchored, because a key pattern claims the whole key: an
        // unanchored `id` would claim `request-id` and every other key
        // that merely contains it.
        "ctxkeyre" => Ok(Matcher::ContextKeyPattern(compile(
            line,
            value,
            &format!("^(?:{value})$"),
        )?)),
        "regex" => Ok(Matcher::Context(compile(line, value, value)?)),
        "token" => Ok(Matcher::Token(value.to_lowercase())),
        "prefixof" => Ok(Matcher::PrefixOf(value.to_lowercase())),
        "site" => {
            let (path, token) = value
                .rsplit_once('@')
                .ok_or(RuleTableError::SiteSyntax { line })?;
            Ok(Matcher::Site(path.to_owned(), token.to_lowercase()))
        }
        "shape" => Ok(Matcher::Shape(match value {
            "low-entropy" => Shape::LowEntropy,
            "underscored" => Shape::Underscored,
            "filename" => Shape::Filename,
            other => {
                return Err(RuleTableError::Shape {
                    line,
                    shape: other.to_owned(),
                });
            }
        })),
        other => Err(RuleTableError::MatchKind {
            line,
            kind: other.to_owned(),
        }),
    }
}

fn compile(line: usize, value: &str, pattern: &str) -> Result<Regex, RuleTableError> {
    Regex::new(pattern).map_err(|error| RuleTableError::Pattern {
        line,
        value: value.to_owned(),
        message: error.to_string(),
    })
}

// Character classes are refused rather than implemented: the tree's globs
// need none, and a class is the one glob construct whose reading differs
// between tools, so refusing it keeps this table meaning one thing.
fn check_glob(line: usize, pattern: &str) -> Result<(), RuleTableError> {
    if pattern.contains('[') || pattern.contains(']') {
        return Err(RuleTableError::GlobSyntax { line });
    }
    Ok(())
}

// -------------------------------------------------------------------------
// matching
// -------------------------------------------------------------------------

impl Rule {
    /// Does this rule claim the occurrence?
    #[must_use]
    pub fn claims(&self, occurrence: &Occurrence) -> bool {
        match &self.matcher {
            Matcher::PathGlob(glob) => glob_matches(glob, &occurrence.path),
            Matcher::PathContext(glob, key) => {
                glob_matches(glob, &occurrence.path) && context_key(occurrence) == key
            }
            Matcher::ContextKey(key) => context_key(occurrence) == key,
            Matcher::ContextKeyPattern(pattern) => pattern.is_match(context_key(occurrence)),
            Matcher::Context(pattern) => pattern.is_match(&occurrence.context),
            Matcher::Token(value) => occurrence.token.to_lowercase() == *value,
            Matcher::PrefixOf(value) => value.starts_with(&occurrence.token.to_lowercase()),
            Matcher::Site(path, value) => {
                occurrence.path == *path && occurrence.token.to_lowercase() == *value
            }
            Matcher::Shape(shape) => has_shape(*shape, occurrence),
        }
    }
}

/// The first rule that claims the occurrence, in table order.
#[must_use]
pub fn first_claim<'a>(rules: &'a [Rule], occurrence: &Occurrence) -> Option<&'a Rule> {
    rules.iter().find(|rule| rule.claims(occurrence))
}

/// The first whitespace-delimited word of the occurrence's line.
///
/// The capture and report formats are `<key> <value>` records, so the
/// first word names what the value on that line *is*. That makes the key
/// the natural unit of classification for the corpus that dominates this
/// population, and it is why so many rules are stated over keys rather
/// than over paths.
fn context_key(occurrence: &Occurrence) -> &str {
    occurrence
        .context
        .split_whitespace()
        .next()
        .unwrap_or_default()
}

fn has_shape(shape: Shape, occurrence: &Occurrence) -> bool {
    match shape {
        Shape::LowEntropy => {
            let alphabet: BTreeSet<_> = occurrence.token.to_lowercase().chars().collect();
            alphabet.len() <= 4
        }
        Shape::Underscored => is_underscored(&occurrence.token),
        Shape::Filename => is_filename_site(&occurrence.token, &occurrence.word),
    }
}

/// Is the whole value written in the underscored fingerprint form?
fn is_underscored(token: &str) -> bool {
    let characters: Vec<_> = token.chars().collect();
    underscored_spans(&characters)
        .first()
        .is_some_and(|&(start, end)| start == 0 && end == characters.len())
}

/// Is the value part of a name, rather than a word standing on its own?
fn is_filename_site(token: &str, word: &str) -> bool {
    let core = word.trim_matches(|c| PUNCTUATION.contains(c));
    if core == token || !core.contains(token) {
        return false;
    }
    core.contains('/') || has_file_extension(core)
}

// Punctuation a value can be wrapped in without ceasing to be part of the
// word it sits in: brackets, quotes, the markup emphasis marks, and the
// separators a sentence puts against a name.
const PUNCTUATION: &str = "()[]{}<>,.;:'\"`*_|";

fn has_file_extension(core: &str) -> bool {
    let Some((_, extension)) = core.rsplit_once('.') else {
        return false;
    };
    let mut characters = extension.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    first.is_ascii_alphabetic()
        && (1..=9).contains(&characters.clone().count())
        && characters.all(|c| c.is_ascii_alphanumeric())
}

// -------------------------------------------------------------------------
// globbing
// -------------------------------------------------------------------------

/// Match a path against a `*`/`?` glob, where `*` spans separators.
///
/// Separator-spanning is deliberate and is what the table's rules assume:
/// `plans/*` is a claim about the plans tree, not about its top level.
#[must_use]
pub fn glob_matches(pattern: &str, text: &str) -> bool {
    let pattern: Vec<_> = pattern.chars().collect();
    let text: Vec<_> = text.chars().collect();
    let (mut p, mut t) = (0usize, 0usize);
    let mut star: Option<usize> = None;
    let mut resume = 0usize;
    while t < text.len() {
        if p < pattern.len() && (pattern[p] == '?' || pattern[p] == text[t]) {
            p += 1;
            t += 1;
        } else if p < pattern.len() && pattern[p] == '*' {
            star = Some(p);
            p += 1;
            resume = t;
        } else if let Some(position) = star {
            p = position + 1;
            resume += 1;
            t = resume;
        } else {
            return false;
        }
    }
    pattern[p..].iter().all(|&c| c == '*')
}

// -------------------------------------------------------------------------
// scanning
// -------------------------------------------------------------------------

const fn is_word_character(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Spans of the underscored `0x` fingerprint form, `0x053e_323b_8934_f2b5`.
///
/// This form is scanned before plain runs because its underscores would
/// otherwise break it into pieces none of which is the value.
fn underscored_spans(characters: &[char]) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut i = 0;
    while i + 1 < characters.len() {
        if characters[i] == '0' && characters[i + 1] == 'x' {
            let digits = i + 2;
            let mut end = digits;
            while end < characters.len() && characters[end].is_ascii_hexdigit() {
                end += 1;
            }
            if end - digits >= 2 {
                let mut groups = 0;
                while end < characters.len() && characters[end] == '_' {
                    let mut next = end + 1;
                    while next < characters.len() && characters[next].is_ascii_hexdigit() {
                        next += 1;
                    }
                    if next == end + 1 {
                        break;
                    }
                    end = next;
                    groups += 1;
                }
                if groups > 0 {
                    spans.push((i, end));
                    i = end;
                    continue;
                }
            }
        }
        i += 1;
    }
    spans
}

/// Spans of a long value written for a human with its middle removed.
///
/// The ellipsis sits between two hexadecimal runs and both halves belong
/// to one value, so neither half is scanned: a rule about half a digest
/// would be a rule about nothing.
fn elided_spans(characters: &[char]) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut i = 0;
    while i < characters.len() {
        let separator = if characters[i] == '\u{2026}' {
            1
        } else if characters[i..].starts_with(&['.', '.', '.']) {
            3
        } else {
            0
        };
        if separator > 0 {
            let mut left = i;
            while left > 0 && characters[left - 1].is_whitespace() {
                left -= 1;
            }
            let left_end = left;
            while left > 0 && characters[left - 1].is_ascii_hexdigit() {
                left -= 1;
            }
            let mut right = i + separator;
            while right < characters.len() && characters[right].is_whitespace() {
                right += 1;
            }
            let right_start = right;
            while right < characters.len() && characters[right].is_ascii_hexdigit() {
                right += 1;
            }
            if left_end - left >= 4 && right - right_start >= 4 {
                spans.push((left, right));
                i = right;
                continue;
            }
        }
        i += 1;
    }
    spans
}

/// Every value on one line, as `(start, end)` character offsets.
///
/// A run must stand free of any longer identifier: a hexadecimal
/// substring of a longer word is part of a name, not a value cited in
/// its own right. That is why the test is over whole word runs rather
/// than over every hexadecimal substring.
fn value_spans(characters: &[char]) -> Vec<(usize, usize)> {
    let underscored = underscored_spans(characters);
    let mut spans = underscored.clone();
    let mut i = 0;
    while i < characters.len() {
        if !is_word_character(characters[i]) {
            i += 1;
            continue;
        }
        let start = i;
        while i < characters.len() && is_word_character(characters[i]) {
            i += 1;
        }
        let length = i - start;
        if (VALUE_FLOOR..=VALUE_CEILING).contains(&length)
            && characters[start..i].iter().all(char::is_ascii_hexdigit)
            && !underscored
                .iter()
                .any(|&(from, to)| from <= start && i <= to)
        {
            spans.push((start, i));
        }
    }
    spans.sort_unstable();
    spans
}

/// The form a value takes, for a refusal a reader can act on.
#[must_use]
pub fn shape_name(token: &str) -> &'static str {
    if token.starts_with("0x") {
        return "fingerprint";
    }
    match token.len() {
        64 => "sha256",
        40 => "object-id",
        7..=12 => "short-object-id",
        _ => "hex-run",
    }
}

fn surrounding_word(characters: &[char], start: usize, end: usize) -> String {
    let mut from = start;
    while from > 0 && !characters[from - 1].is_whitespace() {
        from -= 1;
    }
    let mut to = end;
    while to < characters.len() && !characters[to].is_whitespace() {
        to += 1;
    }
    characters[from..to].iter().collect()
}

/// Every value in one tracked file, including its name.
///
/// The name is scanned as well as the body because a record named after
/// the tree it was taken over carries the value in its identity, and a
/// scan that read only bodies would be blind to exactly the population
/// the name rule exists to account for.
#[must_use]
pub fn scan_file(path: &str, text: &str) -> Vec<Occurrence> {
    let mut found = Vec::new();
    collect(path, path, 0, &mut found);
    for (number, raw) in text.lines().enumerate() {
        if raw.trim().is_empty() {
            continue;
        }
        let line = u32::try_from(number + 1).unwrap_or(u32::MAX);
        collect(path, raw, line, &mut found);
    }
    found
}

fn collect(path: &str, raw: &str, line: u32, found: &mut Vec<Occurrence>) {
    let characters: Vec<_> = raw.chars().collect();
    let elided = elided_spans(&characters);
    for (start, end) in value_spans(&characters) {
        if elided.iter().any(|&(from, to)| from <= start && end <= to) {
            continue;
        }
        let token: String = characters[start..end].iter().collect();
        // Decimal digits are hexadecimal digits, so an all-digit run
        // matches the value pattern while being an ordinary number. Form
        // cannot separate the two, and the audit takes the reading that
        // does not invent a measurement.
        if token.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        found.push(Occurrence {
            path: path.to_owned(),
            line,
            column: u32::try_from(start).unwrap_or(u32::MAX),
            shape: shape_name(&token),
            word: surrounding_word(&characters, start, end),
            context: if line == 0 {
                path.to_owned()
            } else {
                raw.trim().to_owned()
            },
            token,
        });
    }
}

/// Adjudicate a scanned population against the table.
#[must_use]
pub fn adjudicate(
    rules: &[Rule],
    occurrences: &[Occurrence],
    files_scanned: usize,
) -> HashCitationReport {
    let mut claimed = vec![0usize; rules.len()];
    let mut refusals = Vec::new();
    let mut placed = 0usize;
    for occurrence in occurrences {
        if let Some(index) = rules.iter().position(|rule| rule.claims(occurrence)) {
            claimed[index] += 1;
            placed += 1;
        } else {
            refusals.push(HashCitationRefusal {
                path: occurrence.path.clone(),
                line: occurrence.line,
                column: occurrence.column,
                value: occurrence.token.clone(),
                shape: occurrence.shape.to_owned(),
            });
        }
    }
    let claims = rules
        .iter()
        .zip(&claimed)
        .map(|(rule, &occurrences)| RuleClaims {
            id: rule.id.clone(),
            occurrences,
        })
        .collect();
    HashCitationReport {
        schema: HASH_CITATION_SCHEMA,
        files_scanned,
        occurrences: occurrences.len(),
        placed,
        claims,
        valid: refusals.is_empty(),
        refusals,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str =
        "id\tmatch_kind\tmatch_value\tgroup\tverification\treferent\tgeneration_program\tnotes\n";

    fn table(rows: &str) -> Vec<Rule> {
        load_rules(&format!("{HEADER}{rows}")).expect("table loads")
    }

    // Every value below is an authored specimen that names no commit of
    // this repository. A fixture that cited a real one would be the very
    // thing this module exists to find, sitting inside its own test.
    //
    // Bound rather than asserted through `is_empty`, so a failure prints
    // the count it actually found.
    fn count(path: &str, line: &str) -> usize {
        scan_file(path, line).len()
    }

    fn occurrence(path: &str, line: &str) -> Occurrence {
        let mut found = scan_file(path, line);
        assert_eq!(found.len(), 1, "expected exactly one value in {line:?}");
        found.remove(0)
    }

    #[test]
    fn a_run_inside_a_longer_identifier_is_not_a_value() {
        // The guard is what separates a citation from a name fragment.
        assert_eq!(count("a.md", "zzz1234567 and deadbeefzz"), 0);
        assert_eq!(count("a.md", "at deadbeef1 today"), 1);
    }

    #[test]
    fn the_floor_and_ceiling_bound_the_population() {
        assert_eq!(count("a.md", "abcdef is short"), 0);
        let long: String = std::iter::repeat_n('a', 65).collect();
        assert_eq!(count("a.md", &format!("value {long}")), 0);
        let ceiling: String = std::iter::repeat_n('a', 64).collect();
        assert_eq!(count("a.md", &format!("value {ceiling}")), 1);
    }

    #[test]
    fn an_all_digit_run_is_a_number_and_not_a_value() {
        assert_eq!(count("a.md", "cap 2100000000000000 units"), 0);
    }

    #[test]
    fn both_halves_of_an_elided_value_are_left_alone() {
        let elided = "suite deadbeef1234\u{2026}5678deadbeef taken";
        assert_eq!(count("a.md", elided), 0);
        assert_eq!(count("a.md", "suite deadbeef1234...5678deadbeef taken"), 0);
    }

    #[test]
    fn the_underscored_fingerprint_form_is_one_value() {
        let found = scan_file("a.md", "grounds 0x053e_323b_8934_f2b5 holds");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].token, "0x053e_323b_8934_f2b5");
        assert_eq!(found[0].shape, "fingerprint");
    }

    #[test]
    fn a_value_in_the_name_is_found_with_line_zero() {
        let found = scan_file("plans/reviews/review-9-tree-abcdef0.md", "");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].line, 0);
        assert_eq!(found[0].column, 28);
    }

    #[test]
    fn a_path_glob_spans_separators() {
        assert!(glob_matches("plans/*", "plans/history/backlog-history.md"));
        assert!(glob_matches(
            "packages/*/src/*",
            "packages/vectors/src/a/b.rs"
        ));
        assert!(!glob_matches("plans/*", "docs/attestation/realization.md"));
    }

    #[test]
    fn the_first_matching_rule_wins() {
        let rules = table(concat!(
            "narrow\ttoken\tdeadbeef1\tINTERNAL\tAUTHORED\tItself.\tNone.\tThe specific rule.\n",
            "broad\tpathglob\tplans/*\tINTERNAL\tRECOMPUTABLE\tA record.\tThe record.\tThe general rule.\n",
        ));
        let found = occurrence("plans/backlog.md", "row deadbeef1 closed");
        assert_eq!(first_claim(&rules, &found).expect("claimed").id, "narrow");
    }

    #[test]
    fn a_key_pattern_claims_the_whole_key_and_not_a_fragment() {
        let rules = table(
            "ids\tctxkeyre\t(response-)?(request-id|id)\tINTERNAL\tHEX-TEXT-DECODES\tA label.\tThe adapter.\tKeys.\n",
        );
        assert!(first_claim(&rules, &occurrence("c.capture", "request-id 12 deadbeef1")).is_some());
        assert!(first_claim(&rules, &occurrence("c.capture", "other-id 12 deadbeef1")).is_none());
    }

    #[test]
    fn a_value_no_rule_describes_is_refused() {
        let rules = table(
            "records\tpathglob\tplans/*\tINTERNAL\tRECOMPUTABLE\tA record.\tThe record.\tRecords.\n",
        );
        let found = scan_file("papers/note.md", "digest deadbeef1 unexplained");
        let report = adjudicate(&rules, &found, 1);
        assert!(!report.valid);
        assert_eq!(report.refusals.len(), 1);
        assert_eq!(report.refusals[0].value, "deadbeef1");
        assert_eq!(report.refusals[0].shape, "short-object-id");
        assert_eq!(report.claims.len(), 1);
        assert_eq!(report.claims[0].id, "records");
        assert_eq!(report.claims[0].occurrences, 0);
    }

    #[test]
    fn the_name_shape_claims_an_embedded_value_and_not_a_free_word() {
        let rules =
            table("names\tshape\tfilename\tINTERNAL\tAUTHORED\tThe artifact.\tNone.\tNames.\n");
        let embedded = occurrence("plans/a.md", "see review-9-tree-abcdef0.md for it");
        assert!(first_claim(&rules, &embedded).is_some());
        let free = occurrence("plans/a.md", "see abcdef0 for it");
        assert!(first_claim(&rules, &free).is_none());
    }

    #[test]
    fn an_internal_rule_must_name_its_generation_program() {
        let table = format!(
            "{HEADER}nameless\tpathglob\tplans/*\tINTERNAL\tRECOMPUTABLE\tA record.\t\tNo generator.\n"
        );
        assert!(matches!(
            load_rules(&table),
            Err(RuleTableError::MissingGenerator { .. })
        ));
    }

    #[test]
    fn a_prefix_rule_claims_the_abbreviations_of_the_value_it_names() {
        let rules = table(
            "tip\tprefixof\tb7fc5d080a7e9ccc0ef48c3ba11db243e794bdb0\tEXTERNAL\tRESOLVABLE\t             An upstream commit.\tThe upstream project.\tCited at any length.\n",
        );

        for cited in [
            "b7fc5d0",
            "b7fc5d080a7e",
            "b7fc5d080a7e9ccc0ef48c3ba11db243e794bdb0",
        ] {
            let occurrence = occurrence("plans/a.md", &format!("built from {cited} exactly"));
            assert!(
                first_claim(&rules, &occurrence).is_some(),
                "{cited} is an abbreviation of the commit the rule names",
            );
        }

        // A run that merely shares no prefix with it is a different value, and
        // a rule naming one commit must not excuse another.
        let other = occurrence("plans/a.md", "built from b7fc5d180a7e exactly");
        assert!(first_claim(&rules, &other).is_none());
    }

    #[test]
    fn a_rule_must_declare_how_a_reader_establishes_the_value() {
        let unchecked = format!(
            "{HEADER}recorded\tpathglob\tplans/*\tINTERNAL\tRECORDED\tA measurement.\tA wave.\tRecorded.\n"
        );
        assert!(matches!(
            load_rules(&unchecked),
            Err(RuleTableError::UndeclaredVerification { verification, .. })
                if verification == "RECORDED"
        ));

        // The refusal is of the declaration, not of the row's subject: the same
        // rule with a word that names a reader's check is admitted.
        let checked = format!(
            "{HEADER}recomputed\tpathglob\tplans/*\tINTERNAL\tRECOMPUTABLE\tA measurement.\tA wave.\tRecomputed.\n"
        );
        assert!(load_rules(&checked).is_ok());
    }

    #[test]
    fn re_running_a_ceremony_is_not_something_a_reader_can_do() {
        // The word this audit shipped with for corpus material said where the
        // value came from. A reader without a node and the ceremony cannot act
        // on it, so it is refused like every other non-declaration.
        let table = format!(
            "{HEADER}capture\tpathglob\tplans/*\tINTERNAL\tCAPTURE-RUN\tCeremony material.\tThe harness.\tRe-run it.\n"
        );
        assert!(matches!(
            load_rules(&table),
            Err(RuleTableError::UndeclaredVerification { .. })
        ));
    }

    #[test]
    fn a_malformed_row_stops_the_table_rather_than_being_skipped() {
        let table = format!("{HEADER}short\tpathglob\tplans/*\n");
        assert!(matches!(
            load_rules(&table),
            Err(RuleTableError::Columns { found: 3, .. })
        ));
    }

    #[test]
    fn an_unknown_match_kind_is_refused_rather_than_ignored() {
        let table = format!(
            "{HEADER}odd\tresidue\tSOMETHING\tINTERNAL\tAUTHORED\tItself.\tNone.\tUnknown kind.\n"
        );
        assert!(matches!(
            load_rules(&table),
            Err(RuleTableError::MatchKind { .. })
        ));
    }
}
