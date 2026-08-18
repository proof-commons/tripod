//! The companion attestation register the adopted kind registry
//! requires of an acceptee (ADR-020).
//!
//! This repository is the acceptee. It owns the local extension set,
//! the evidence for it, the statuses it assigns, this generator, and
//! the register the generator writes. The register is a view of that
//! evidence base and status map, and it presents the homonymy of the
//! effective relation beside them. It creates nothing: not a
//! classification, not an attestation, not a homonymy fact.
//!
//! Every row is derived, on each run, from two committed documents —
//! the archived registry draft and the adopting decision record — and
//! from a fresh census of the corpus. Nothing is held in this module
//! but the derivation.
//!
//! The register is totally ordered by name, then kind, then source,
//! then locator, then the sequence number of the record in its source
//! table. The order is a property of the derived key alone, so it is
//! independent of the order the sources are read in.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use crate::{
    adoption::{EXTENSION_SOURCE, REGISTRY_SOURCE},
    diagnostic::{LabelDiagnostic, LabelErrorCode},
    source::SourceLocation,
};

/// Which document recorded a pair.
///
/// The two sources are the registry's own rows and this corpus's
/// recorded extension set, written C and `X_A` by the registry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Source {
    /// A row of the adopted registry's base relation.
    Base,
    /// A row of this corpus's recorded extension set.
    Extension,
}

impl Source {
    /// The register's short name for the source.
    pub const fn token(self) -> &'static str {
        match self {
            Self::Base => "C",
            Self::Extension => "X_A",
        }
    }

    /// The document the rows were read from.
    pub const fn document(self) -> &'static str {
        match self {
            Self::Base => REGISTRY_SOURCE,
            Self::Extension => EXTENSION_SOURCE,
        }
    }
}

/// An attestation status. Only the two admitting statuses are derived
/// here: a candidate is not a member of the effective relation, so no
/// row of either source carries one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    Firm,
    Borderline,
}

impl Status {
    pub const fn token(self) -> &'static str {
        match self {
            Self::Firm => "firm",
            Self::Borderline => "borderline",
        }
    }
}

/// The register's total recorded ordering, as a key.
///
/// The field order is the ordering: name, kind, source, locator, then
/// the record's sequence number in its source table as the tiebreak.
/// Deriving the comparator from the field order keeps the ordering one
/// fact rather than two.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RecordKey {
    pub name: String,
    pub kind: String,
    pub source: Source,
    pub locator: String,
    pub sequence: usize,
}

/// One recorded pair, with what its source says about it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Record {
    pub key: RecordKey,
    pub status: Status,
    /// The catalogued sense, for a first-hand row. Base rows hold their
    /// evidence by reference to the edition and carry none here.
    pub sense: Option<String>,
    /// An exact quoted spelling of the pair in this corpus, for a
    /// first-hand row.
    pub spelling: Option<String>,
}

/// The derived evidence base and status map, ready to render.
#[derive(Clone, Debug, Default)]
pub struct AttestationBase {
    /// Every recorded pair of the effective relation, in register
    /// order.
    pub records: Vec<Record>,
    /// Mints per kind in this corpus, the census the first-hand rows
    /// are answerable to.
    pub census: BTreeMap<String, usize>,
}

impl AttestationBase {
    /// The first-hand rows: this corpus's recorded extensions.
    pub fn extensions(&self) -> impl Iterator<Item = &Record> {
        self.records
            .iter()
            .filter(|record| record.key.source == Source::Extension)
    }

    /// The rows held by reference whose status is weaker than firm.
    /// The edition's remaining base rows are firm and are stated by
    /// reference rather than enumerated.
    pub fn borderline_base(&self) -> impl Iterator<Item = &Record> {
        self.records.iter().filter(|record| {
            record.key.source == Source::Base && record.status == Status::Borderline
        })
    }

    /// The number of base rows, for the by-reference statement.
    pub fn base_count(&self) -> usize {
        self.records
            .iter()
            .filter(|record| record.key.source == Source::Base)
            .count()
    }

    /// Hom of the effective relation: every recorded pair whose name
    /// carries another kind. Derived from the same records the evidence
    /// and status sections present, never declared.
    pub fn homonyms(&self) -> Vec<&Record> {
        let mut kinds_by_name: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for record in &self.records {
            kinds_by_name
                .entry(&record.key.name)
                .or_default()
                .insert(&record.key.kind);
        }
        self.records
            .iter()
            .filter(|record| {
                kinds_by_name
                    .get(record.key.name.as_str())
                    .is_some_and(|kinds| kinds.len() > 1)
            })
            .collect()
    }

    /// The mint count of a kind in this corpus.
    pub fn mints(&self, kind: &str) -> usize {
        self.census.get(kind).copied().unwrap_or_default()
    }
}

/// Derive the evidence base from the two source documents and a census.
///
/// A source that cannot be read, or that yields no rows, is a failure:
/// a register derived from an empty relation would present a vacuous
/// homonymy and a vacuous coverage claim.
pub fn derive(
    root: &Path,
    census: BTreeMap<String, usize>,
) -> Result<AttestationBase, Vec<LabelDiagnostic>> {
    let mut diagnostics = Vec::new();
    let base = read_source(root, REGISTRY_SOURCE, &mut diagnostics)
        .map(|text| base_records(&text))
        .unwrap_or_default();
    let extension = read_source(root, EXTENSION_SOURCE, &mut diagnostics)
        .map(|text| extension_records(&text))
        .unwrap_or_default();
    for (rows, source) in [(&base, Source::Base), (&extension, Source::Extension)] {
        if rows.is_empty() {
            diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::GeneratedRegisterStale,
                &SourceLocation::new(Path::new(source.document()), 1, 1),
                "no attestation rows were derived from this source; refusing a vacuous register",
            ));
        }
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let mut records = base;
    records.extend(extension);
    records.sort();
    Ok(AttestationBase { records, census })
}

fn read_source(
    root: &Path,
    relative: &str,
    diagnostics: &mut Vec<LabelDiagnostic>,
) -> Option<String> {
    match fs::read_to_string(root.join(relative)) {
        Ok(text) => Some(text),
        Err(error) => {
            diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::Io,
                &SourceLocation::new(Path::new(relative), 1, 1),
                error.to_string(),
            ));
            None
        }
    }
}

/// The status mark the registry prints at a borderline row. It marks
/// the row, never the name: the exact catalogue name is the row's name
/// with the mark removed.
const BORDERLINE_MARK: char = '\u{2020}';

/// Read the registry's base rows from the Convention tables.
///
/// A row is a name and a kind token. Device rows, whose kind cell is a
/// dash rather than a token, classify nothing and are skipped, and the
/// header and rule rows carry no token either. The locator is the
/// Convention's own label, which is the table's home in the draft.
fn base_records(text: &str) -> Vec<Record> {
    let mut records = Vec::new();
    let mut home = String::new();
    let mut inside = false;
    for line in text.lines() {
        if let Some(label) = convention_label(line) {
            home = label;
            inside = true;
            continue;
        }
        if line.starts_with("## ") {
            inside = false;
            continue;
        }
        if !inside || !line.starts_with('|') {
            continue;
        }
        let cells = table_cells(line);
        if cells.len() != 2 {
            continue;
        }
        let Some(kind) = backticked(cells[1]) else {
            continue;
        };
        let marked = cells[0].contains(BORDERLINE_MARK);
        let name = cells[0].replace(BORDERLINE_MARK, "").trim().to_owned();
        if name.is_empty() {
            continue;
        }
        records.push(Record {
            key: RecordKey {
                name,
                kind,
                source: Source::Base,
                locator: home.clone(),
                sequence: records.len(),
            },
            status: if marked {
                Status::Borderline
            } else {
                Status::Firm
            },
            sense: None,
            spelling: None,
        });
    }
    records
}

/// The label of a Convention environment heading, if the line is one.
fn convention_label(line: &str) -> Option<String> {
    if !line.starts_with("**Convention (") {
        return None;
    }
    let (_, label) = line.rsplit_once(" \u{b7} ")?;
    backticked_token(label.trim())
}

/// The label text heading the adopting record's extension table. Held
/// as data rather than written as a citation: this crate is a different
/// owner, and the string names a location in a document, not a fact
/// this crate cites.
const EXTENSION_TABLE_MINT: &str = "tab:kinds:extensions";

/// Read the recorded extension rows from the adopting record's table.
///
/// Each row carries a name, a kind, the catalogued sense, and the
/// first-hand evidence: an exact quoted spelling and the locator that
/// places it in this corpus. Every recorded extension is firm on that
/// evidence, which is the adopting record's own statement about the set
/// it records.
fn extension_records(text: &str) -> Vec<Record> {
    let mut records = Vec::new();
    let mut inside = false;
    for line in text.lines() {
        if line.contains(EXTENSION_TABLE_MINT) {
            inside = true;
            continue;
        }
        if inside && line.starts_with("## ") {
            break;
        }
        if !inside || !line.starts_with('|') {
            continue;
        }
        let cells = table_cells(line);
        if cells.len() != 4 {
            continue;
        }
        let Some(kind) = backticked(cells[1]) else {
            continue;
        };
        let quoted = backticked_tokens(cells[3]);
        records.push(Record {
            key: RecordKey {
                name: cells[0].trim().to_owned(),
                kind,
                source: Source::Extension,
                locator: quoted.get(1).cloned().unwrap_or_default(),
                sequence: records.len(),
            },
            status: Status::Firm,
            sense: Some(cells[2].trim().to_owned()),
            spelling: quoted.first().cloned(),
        });
    }
    records
}

fn table_cells(line: &str) -> Vec<&str> {
    line.trim()
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .collect()
}

/// A cell that is exactly one backticked kind token. The token shape is
/// the registry's: lowercase letters and digits, never a hyphen, since
/// a registry of words admits no hyphenated member.
fn backticked(cell: &str) -> Option<String> {
    let token = backticked_token(cell)?;
    let shaped = token
        .chars()
        .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit());
    shaped.then_some(token)
}

/// A cell that is exactly one backticked token of any shape.
fn backticked_token(cell: &str) -> Option<String> {
    let token = cell.trim().strip_prefix('`')?.strip_suffix('`')?;
    let shaped = !token.is_empty()
        && token
            .chars()
            .all(|character| !character.is_whitespace() && character != '`');
    shaped.then(|| token.to_owned())
}

/// Every backticked token of a cell, in order.
fn backticked_tokens(cell: &str) -> Vec<String> {
    cell.split('`')
        .skip(1)
        .step_by(2)
        .filter(|token| !token.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}
