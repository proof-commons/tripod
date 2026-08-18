//! Head validation: the judgment that a head's name and its declared
//! kind are a catalogued pair.
//!
//! ADR-020 adopts the environment-kind registry, and the registry's
//! adoption gate asks that every participating authored head validate by
//! exactly one exact pair or one reduction. The checker held a label's
//! kind token to the adopted vocabulary from the first, but nothing held
//! a head's **name** to the kind beside it: a head reading Theorem and
//! labelled with the kind of a table passed, because both sides were
//! separately catalogued. This module closes that gap.
//!
//! # What is a head
//!
//! An environment head is the repository's `**Genre (Title)**` leader
//! followed by the head's own label. Recognition is deliberately narrow,
//! because a false rejection blocks an author: a line qualifies only
//! when the bold leader opens the line, carries a parenthesized title,
//! and is followed by the head separator and a parsable label. Bold
//! prose emphasis — a leader ending in a full stop, a leader with no
//! title, a leader with no label after it — is not a head and forms no
//! judgment, which is the reading the registry's own totality invariant
//! takes: it quantifies over authored heads, not over bold spans.
//!
//! # Scope
//!
//! Markdown owners only. The LaTeX surface of the Attestation specification declares
//! its environments through the format rather than through a bold
//! leader, and ADR-020 records that surface as not yet in scope at all;
//! Rust sources carry no environment heads. Fenced blocks and the other
//! non-participating regions are excluded through the same scan every
//! other Markdown rule consults, so an example head inside a fence is
//! text, exactly as an example label inside one is.
//!
//! # Reduction
//!
//! The registry admits a list of presentation devices, and this module
//! implements the two the corpus can reach today: the iterated `sub-`
//! prefix, and the catalogued emphasis and status modifiers. The others
//! — numbering, lettering, attached names, stars and unnumbering,
//! restatement, continuation, placement, containment, and the rung a
//! named division takes from its format — are unimplemented, and no head
//! in the governed corpus triggers one. Each is a widening, never a
//! narrowing: implementing one can only admit heads this module now
//! rejects, so leaving them out is visible as a rejection rather than
//! silent as an acceptance.
//!
//! Overriding rows take precedence over reduction, as the registry
//! directs: Working hypothesis and Standing hypothesis reduce to
//! themselves, so a head reading Working hypothesis and labelled with
//! the conjectural kind of its base fails rather than reducing.

use std::{collections::BTreeSet, path::Path};

use crate::{
    adoption::{catalogued_senses, pair_is_catalogued},
    diagnostic::{LabelDiagnostic, LabelErrorCode},
    label::{Label, LabelShape},
    markdown::MarkdownScan,
    source::SourceLocation,
};

/// The separator between a head's bold leader and its label.
const HEAD_SEPARATOR: &str = "** · ";

/// The catalogued emphasis and status modifiers, which are presentation
/// and not genre. Quoted from the registry's reduction definition.
const MODIFIERS: &[&str] = &[
    "Main",
    "Key",
    "Fundamental",
    "Working",
    "Standing",
    "Blanket",
    "Concrete",
    "Motivating",
    "Numerical",
    "Toy",
    "Worked",
    "Running",
];

/// The rows the registry catalogues expressly as overrides of the
/// modifier rule: they reduce to themselves and carry their own kind.
const OVERRIDING_ROWS: &[&str] = &["Working hypothesis", "Standing hypothesis"];

/// Depth cap on iterated reduction. Far above any attested head, and
/// present only so a pathological name cannot spin.
const REDUCTION_DEPTH: usize = 8;

/// One recognized environment head.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthoredHead {
    /// The genre word or phrase: the bold leader before its title.
    pub genre: String,
    /// The parenthesized title, kept for the diagnostic so a reader can
    /// find the head without counting lines.
    pub title: String,
    /// The kind token the head's own label declares.
    pub kind: String,
    /// The label as written.
    pub label: String,
}

/// Recognize an environment head on one line, or decline it.
///
/// Declining is the common case and is never a defect: most bold spans
/// in this corpus are prose emphasis.
#[must_use]
pub fn parse_head(line: &str) -> Option<AuthoredHead> {
    let rest = line.strip_prefix("**")?;
    let close = rest.find(HEAD_SEPARATOR)?;
    let leader = &rest[..close];
    if leader.is_empty() || leader.contains("**") {
        return None;
    }
    // The leader is a genre and a parenthesized title, in that order.
    let open = leader.find(" (")?;
    if !leader.ends_with(')') {
        return None;
    }
    let genre = leader[..open].trim();
    let title = leader[open + 2..leader.len() - 1].trim();
    if genre.is_empty() || !genre.starts_with(|character: char| character.is_uppercase()) {
        return None;
    }
    let after = &rest[close + HEAD_SEPARATOR.len()..];
    let label = delimited_label(after)?;
    let parsed = Label::parse(&label, LabelShape::Planning).ok()?;
    Some(AuthoredHead {
        genre: genre.to_owned(),
        title: title.to_owned(),
        kind: parsed.kind().to_owned(),
        label,
    })
}

/// The label a head carries, taken from the delimited span that opens
/// the text after the separator. Both delimiters this corpus mints in
/// are accepted, since which one is written is a syntax question the
/// citation rules own and not a head-validation one.
fn delimited_label(after: &str) -> Option<String> {
    let mut characters = after.chars();
    let delimiter = characters.next()?;
    if delimiter != '`' && delimiter != '\u{b4}' {
        return None;
    }
    let body = &after[delimiter.len_utf8()..];
    let end = body.find(delimiter)?;
    Some(body[..end].to_owned())
}

/// Whether the head's name and kind stand in the effective relation,
/// directly or through one reduction.
#[must_use]
pub fn head_validates(genre: &str, kind: &str) -> bool {
    if pair_is_catalogued(genre, kind) {
        return true;
    }
    if is_overriding_row(genre) {
        return false;
    }
    reductions(genre)
        .iter()
        .any(|base| pair_is_catalogued(base, kind))
}

fn is_overriding_row(genre: &str) -> bool {
    OVERRIDING_ROWS
        .iter()
        .any(|row| row.eq_ignore_ascii_case(genre))
}

/// Every name reachable from an authored head by the reduction devices
/// this module implements, the head's own name excluded.
fn reductions(genre: &str) -> BTreeSet<String> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut frontier = vec![genre.to_owned()];
    for _ in 0..REDUCTION_DEPTH {
        let mut next = Vec::new();
        for name in std::mem::take(&mut frontier) {
            for reduced in [strip_sub_prefix(&name), strip_modifier(&name)]
                .into_iter()
                .flatten()
            {
                if seen.insert(reduced.clone()) {
                    next.push(reduced);
                }
            }
        }
        if next.is_empty() {
            break;
        }
        frontier = next;
    }
    seen.remove(genre);
    seen
}

/// One `sub-` prefix removed, in either spelling. Iteration is the
/// caller's, so a sub-subsection reduces through a subsection.
fn strip_sub_prefix(name: &str) -> Option<String> {
    let lower = name.to_lowercase();
    let rest = lower
        .strip_prefix("sub-")
        .or_else(|| lower.strip_prefix("sub"))?;
    if rest.is_empty() {
        return None;
    }
    Some(capitalize(rest))
}

/// One leading catalogued modifier removed. The modifier must be a
/// whole word: a name merely beginning with a modifier's letters — a
/// Mainframe, a Keyword — is not a modified name and reduces to nothing.
fn strip_modifier(name: &str) -> Option<String> {
    for modifier in MODIFIERS {
        if name.len() <= modifier.len()
            || !name.is_char_boundary(modifier.len())
            || !name[..modifier.len()].eq_ignore_ascii_case(modifier)
        {
            continue;
        }
        let rest = &name[modifier.len()..];
        if !rest.starts_with([' ', '-']) {
            continue;
        }
        let rest = rest[1..].trim_start();
        if !rest.is_empty() {
            return Some(capitalize(rest));
        }
    }
    None
}

fn capitalize(value: &str) -> String {
    let mut characters = value.chars();
    characters.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(characters).collect()
    })
}

/// Validate every head of one Markdown document.
///
/// The scan supplies the participation view, so a head inside a fenced
/// block or any other non-participating region forms no judgment.
pub fn validate_document(path: &Path, source: &str, scan: &MarkdownScan) -> Vec<LabelDiagnostic> {
    let mut diagnostics = Vec::new();
    for (index, raw) in source.lines().enumerate() {
        let line = index + 1;
        if !scan.participation.participates(line) {
            continue;
        }
        let Some(head) = parse_head(raw) else {
            continue;
        };
        if head_validates(&head.genre, &head.kind) {
            continue;
        }
        let location = SourceLocation::new(path, line, 1);
        diagnostics.push(LabelDiagnostic::error(
            LabelErrorCode::UncataloguedHeadPair,
            &location,
            uncatalogued_message(&head),
        ));
    }
    diagnostics
}

/// The diagnostic a failed head owes: the head, the label, and what the
/// catalogue does carry for the name, so the author can see at once
/// whether the head or the kind is the side that is wrong.
fn uncatalogued_message(head: &AuthoredHead) -> String {
    let AuthoredHead {
        genre,
        title,
        kind,
        label,
    } = head;
    let senses = catalogued_senses(genre);
    let carried = if senses.is_empty() {
        reductions(genre)
            .iter()
            .find_map(|base| {
                let reduced = catalogued_senses(base);
                (!reduced.is_empty()).then(|| {
                    format!(
                        "the name is in no catalogue row, and its base {base} carries {}",
                        reduced.join(", ")
                    )
                })
            })
            .unwrap_or_else(|| "the name is in no catalogue row, and reduces to none".to_owned())
    } else {
        format!("the catalogued senses of {genre} are {}", senses.join(", "))
    };
    format!(
        "the head {genre} ({title}) declares kind {kind} at {label}, which the effective \
         relation does not pair with the name {genre}; {carried}"
    )
}
