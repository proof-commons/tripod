use std::{
    collections::BTreeSet,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use thiserror::Error;

use crate::{
    census::RepositoryCensus,
    diagnostic::{LabelDiagnostic, LabelErrorCode, sort_diagnostics},
    label::{Label, LabelShape},
    latex::harvest_attestation,
    markdown::{InlineCodeContext, fence_close, fence_open, scan_markdown},
    owner::{ImportedLabel, LabelOwner},
    registry::{LabelMint, LabelRegistry, RegistrySet},
    render,
    rust_source::{RustHarvest, harvest_crates, harvest_model},
    source::{SourceLocation, relative_to},
};

#[derive(Default)]
pub struct RepositoryLabels {
    pub registries: RegistrySet,
    pub diagnostics: Vec<LabelDiagnostic>,
    realization_internal: Vec<(Label, SourceLocation)>,
    adr_internal: Vec<(u16, Label, SourceLocation)>,
    model_internal: Vec<(Label, SourceLocation)>,
    plan_internal: Vec<(Label, SourceLocation)>,
    doc_internal: Vec<(Label, SourceLocation)>,
    crate_internal: Vec<(String, Label, SourceLocation)>,
    imports: Vec<(ImportedLabel, SourceLocation)>,
    attestation_anchor_names: Vec<String>,
    attestation_index_names: BTreeSet<String>,
    attestation_index_location: Option<SourceLocation>,
}
impl RepositoryLabels {
    pub fn harvest_sources(paths: &RepositoryCensus) -> Self {
        let mut result = Self::default();
        let (attestation, diagnostics) = harvest_attestation(paths);
        result.registries.attestation = attestation;
        result.diagnostics.extend(diagnostics);
        harvest_realization(paths, &mut result);
        harvest_adrs(paths, &mut result);
        harvest_plans(paths, &mut result);
        harvest_docs(paths, &mut result);
        let model = harvest_model(paths);
        add_model(model, &mut result);
        for (name, harvest) in harvest_crates(paths) {
            result.crate_internal.extend(
                harvest
                    .citations
                    .into_iter()
                    .map(|(label, location)| (name.clone(), label, location)),
            );
            result.imports.extend(harvest.imports);
            result.diagnostics.extend(harvest.diagnostics);
            result.registries.crates.insert(name, harvest.registry);
        }
        validate(&mut result);
        sort_diagnostics(&mut result.diagnostics);
        result
    }
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(LabelDiagnostic::is_error)
    }
    pub const fn imported_citation_count(&self) -> usize {
        self.imports.len()
    }
}

fn add_model(model: RustHarvest, result: &mut RepositoryLabels) {
    result.model_internal.extend(model.citations);
    result.imports.extend(model.imports);
    result.registries.model = model.registry;
    result.diagnostics.extend(model.diagnostics);
}
fn harvest_realization(paths: &RepositoryCensus, result: &mut RepositoryLabels) {
    let relative = relative_to(&paths.root, &paths.realization);
    let Ok(source) = fs::read_to_string(&paths.realization) else {
        result.diagnostics.push(LabelDiagnostic::error(
            LabelErrorCode::Io,
            &SourceLocation::new(relative, 1, 1),
            "cannot read the realization",
        ));
        return;
    };
    harvest_attestation_citations(&relative, &source, result);
    let scan = scan_markdown(&relative, &source);
    result.diagnostics.extend(scan.diagnostics);
    for span in scan.code_spans {
        if span.delimiter_len != 1 {
            continue;
        }
        if let Some(token) = square(&span.content) {
            if !token.starts_with("A-") {
                if let Ok(imported) = ImportedLabel::parse(token) {
                    result.imports.push((imported, span.location));
                } else if let Ok(label) = Label::parse(token, LabelShape::Model) {
                    result.imports.push((
                        ImportedLabel {
                            owner: LabelOwner::Model,
                            label,
                        },
                        span.location,
                    ));
                }
            }
            continue;
        }
        let Ok(label) = Label::parse(span.content.trim(), LabelShape::Realization) else {
            continue;
        };
        match span.context {
            InlineCodeContext::Bare => {
                let mint = LabelMint {
                    owner: LabelOwner::Realization,
                    label,
                    location: span.location.clone(),
                    home: span.home,
                };
                result.registries.realization.insert_or_diagnose(
                    mint,
                    "the realization",
                    &mut result.diagnostics,
                );
            }
            InlineCodeContext::Parenthesized => {
                result.realization_internal.push((label, span.location));
            }
            InlineCodeContext::Asymmetric => result.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::AsymmetricCitation,
                &span.location,
                "label citation has an unmatched parenthesis",
            )),
        }
    }
}
fn harvest_adrs(paths: &RepositoryCensus, result: &mut RepositoryLabels) {
    for path in &paths.adrs {
        let Some(number) = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.get(..3))
            .and_then(|number| number.parse::<u16>().ok())
        else {
            continue;
        };
        let relative = relative_to(&paths.root, path);
        let Ok(source) = fs::read_to_string(path) else {
            continue;
        };
        let scan = scan_markdown(&relative, &source);
        result.diagnostics.extend(scan.diagnostics);
        let mut registry = LabelRegistry::default();
        for span in scan.code_spans {
            if span.delimiter_len != 1 {
                continue;
            }
            if let Some(token) = square(&span.content) {
                if span.context == InlineCodeContext::Parenthesized {
                    import(token, &span.location, result);
                } else {
                    result.diagnostics.push(LabelDiagnostic::error(
                        LabelErrorCode::InvalidImportedCitationForm,
                        &span.location,
                        "imported citation must be parenthesized",
                    ));
                }
                continue;
            }
            let Ok(label) = Label::parse(span.content.trim(), LabelShape::Adr) else {
                continue;
            };
            match span.context {
                InlineCodeContext::Bare => {
                    let mint = LabelMint {
                        owner: LabelOwner::Adr(number),
                        label,
                        location: span.location.clone(),
                        home: span.home,
                    };
                    registry.insert_or_diagnose(
                        mint,
                        &format!("ADR{number:03}"),
                        &mut result.diagnostics,
                    );
                }
                InlineCodeContext::Parenthesized => {
                    result.adr_internal.push((number, label, span.location));
                }
                InlineCodeContext::Asymmetric => result.diagnostics.push(LabelDiagnostic::error(
                    LabelErrorCode::AsymmetricCitation,
                    &span.location,
                    "label citation has an unmatched parenthesis",
                )),
            }
        }
        result.registries.adrs.insert(number, registry);
    }
}
// Planning and repository-documentation Markdown are complete label
// owners (ADR-013): a bare planning-shaped label mints, a parenthesized
// one cites, and citations resolve against the complete owner registry
// across files — not merely against imports.

/// The two Markdown owners this harvest serves. A dedicated enum keeps
/// the registry and citation-sink dispatch exhaustive: a future owner
/// must choose destinations explicitly instead of falling into a
/// catch-all.
#[derive(Clone, Copy)]
enum MarkdownOwner {
    Plan,
    Doc,
}

impl MarkdownOwner {
    const fn owner(self) -> LabelOwner {
        match self {
            Self::Plan => LabelOwner::Plan,
            Self::Doc => LabelOwner::Doc,
        }
    }
    const fn name(self) -> &'static str {
        match self {
            Self::Plan => "PLAN",
            Self::Doc => "DOC",
        }
    }
}

fn harvest_plans(paths: &RepositoryCensus, result: &mut RepositoryLabels) {
    for path in &paths.plans {
        if *path == paths.specification_register || *path == paths.realization_register {
            // Generated registers are derivative publications and must
            // not contribute source mints or citations.
            continue;
        }
        harvest_markdown_owner(paths, path, MarkdownOwner::Plan, result);
    }
}

/// Harvest every authored Markdown file outside the trees owned
/// elsewhere as the `DOC` owner. Membership comes from the argument
/// census; the census verifier keeps that census welded to the walked
/// reality (ADR-014), so a newly added README still cannot silently
/// sit outside the label graph.
fn harvest_docs(paths: &RepositoryCensus, result: &mut RepositoryLabels) {
    for path in &paths.docs {
        harvest_markdown_owner(paths, path, MarkdownOwner::Doc, result);
    }
}

fn harvest_markdown_owner(
    paths: &RepositoryCensus,
    path: &Path,
    owner: MarkdownOwner,
    result: &mut RepositoryLabels,
) {
    let relative = relative_to(&paths.root, path);
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            result.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::Io,
                &SourceLocation::new(relative, 1, 1),
                error.to_string(),
            ));
            return;
        }
    };
    let scan = scan_markdown(&relative, &source);
    result.diagnostics.extend(scan.diagnostics);
    for span in scan.code_spans {
        if span.delimiter_len != 1 {
            continue;
        }
        if let Some(token) = square(&span.content) {
            if span.context == InlineCodeContext::Parenthesized {
                import(token, &span.location, result);
            } else {
                result.diagnostics.push(LabelDiagnostic::error(
                    LabelErrorCode::InvalidImportedCitationForm,
                    &span.location,
                    "imported citation must be parenthesized",
                ));
            }
            continue;
        }
        if looks_imported(&span.content) {
            result.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::InvalidImportedCitationForm,
                &span.location,
                "imported citation must use square brackets",
            ));
            continue;
        }
        let Ok(label) = Label::parse(span.content.trim(), LabelShape::Planning) else {
            continue;
        };
        let registry = match owner {
            MarkdownOwner::Plan => &mut result.registries.plan,
            MarkdownOwner::Doc => &mut result.registries.doc,
        };
        match span.context {
            InlineCodeContext::Bare => {
                let mint = LabelMint {
                    owner: owner.owner(),
                    label,
                    location: span.location.clone(),
                    home: span.home,
                };
                registry.insert_or_diagnose(mint, owner.name(), &mut result.diagnostics);
            }
            InlineCodeContext::Parenthesized => match owner {
                MarkdownOwner::Plan => result.plan_internal.push((label, span.location)),
                MarkdownOwner::Doc => result.doc_internal.push((label, span.location)),
            },
            InlineCodeContext::Asymmetric => result.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::AsymmetricCitation,
                &span.location,
                "label citation has an unmatched parenthesis",
            )),
        }
    }
}
fn import(token: &str, location: &SourceLocation, result: &mut RepositoryLabels) {
    match ImportedLabel::parse(token) {
        Ok(value) => result.imports.push((value, location.clone())),
        Err(error) => result.diagnostics.push(LabelDiagnostic::error(
            LabelErrorCode::UnknownOwner,
            location,
            error.to_string(),
        )),
    }
}
/// Harvest attestation citations from the realization body only.
///
/// Fenced code is ignored, and tokens inside the generated §17
/// upward-citation index are collected separately: the pinned
/// anchor-set hash and the imported-citation set derive from body
/// occurrences, so a token present only in the index cannot keep
/// itself in the release anchor set. The committed index is instead
/// welded to the body by set equality.
fn harvest_attestation_citations(path: &Path, source: &str, result: &mut RepositoryLabels) {
    let mut fence: Option<(char, usize)> = None;
    let mut in_index = false;
    let mut index_location = SourceLocation::new(path, 1, 1);
    let mut index_names: BTreeSet<String> = BTreeSet::new();
    for (number, line) in source.lines().enumerate() {
        if let Some((marker, length)) = fence {
            if fence_close(line, marker, length) {
                fence = None;
            }
            continue;
        }
        if let Some(open) = fence_open(line) {
            fence = Some(open);
            continue;
        }
        if line.starts_with("## ") {
            // Matched on the locator mint form so a heading merely
            // citing (`sec:anchors`) cannot open the index region.
            in_index = line.contains(" · `sec:realization:anchors`");
            if in_index {
                index_location = SourceLocation::new(path, number + 1, 1);
            }
        }
        let nonparticipating = nonparticipating_ranges(line);
        let mut offset = 0;
        while let Some(found) = line[offset..].find("[A-") {
            let start = offset + found;
            let body = start + "[A-".len();
            let Some(close) = line[body..].find(']') else {
                break;
            };
            let value = &line[body..body + close];
            let example = nonparticipating
                .iter()
                .any(|(from, to)| start >= *from && start < *to);
            if !example && let Ok(label) = Label::parse(value, LabelShape::Attestation) {
                if in_index {
                    index_names.insert(value.to_owned());
                } else {
                    result.attestation_anchor_names.push(value.to_owned());
                    result.imports.push((
                        ImportedLabel {
                            owner: LabelOwner::Attestation,
                            label,
                        },
                        SourceLocation::new(path, number + 1, line[..start].chars().count() + 1),
                    ));
                }
            }
            offset = body + close + 1;
        }
    }
    result.attestation_index_names = index_names;
    result.attestation_index_location = Some(index_location);
}

/// Byte ranges of inline code spans with a delimiter run of two or
/// more backticks on one line: nonparticipating example spans under
/// ADR-013, excluded from the raw anchor scan.
fn nonparticipating_ranges(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut ranges = Vec::new();
    let mut cursor = 0;
    while cursor < bytes.len() {
        if bytes[cursor] != b'`' {
            cursor += 1;
            continue;
        }
        let start = cursor;
        let length = backtick_run(bytes, cursor);
        cursor += length;
        let mut end = cursor;
        while end < bytes.len() && !(bytes[end] == b'`' && backtick_run(bytes, end) == length) {
            end += 1;
        }
        if end == bytes.len() {
            break;
        }
        if length >= 2 {
            ranges.push((start, end + length));
        }
        cursor = end + length;
    }
    ranges
}

fn backtick_run(bytes: &[u8], start: usize) -> usize {
    bytes[start..]
        .iter()
        .take_while(|value| **value == b'`')
        .count()
}

fn validate(result: &mut RepositoryLabels) {
    validate_references(result);
    validate_attestation_index(result);
    validate_architecture_weld(result);
}

/// Weld the committed §17 upward-citation index to the body anchor
/// set. Full-check only: the scoped derivations neither read nor
/// write the index, and a stale index must not block regenerating
/// registers or the model-label publication whose content does not
/// depend on it.
fn validate_attestation_index(result: &mut RepositoryLabels) {
    let Some(location) = result.attestation_index_location.clone() else {
        return;
    };
    let body_names: BTreeSet<String> = result.attestation_anchor_names.iter().cloned().collect();
    if result.attestation_index_names != body_names {
        result.diagnostics.push(LabelDiagnostic::error(
            LabelErrorCode::AttestationIndexStale,
            &location,
            "the committed upward-citation index does not present exactly the body's attestation anchor set",
        ));
    }
}
fn validate_references(result: &mut RepositoryLabels) {
    for (label, location) in &result.realization_internal {
        if !result.registries.realization.contains(label) {
            result.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::MissingMint,
                location,
                format!("unresolved Realization citation {label}"),
            ));
        }
    }
    for (number, label, location) in &result.adr_internal {
        let resolved = result
            .registries
            .adrs
            .get(number)
            .is_some_and(|registry| registry.contains(label));
        if !resolved {
            result.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::MissingMint,
                location,
                format!("unresolved ADR{number:03} citation {label}"),
            ));
        }
    }
    for (label, location) in &result.model_internal {
        if !result.registries.model.contains(label) {
            result.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::MissingMint,
                location,
                format!("unresolved model citation {label}"),
            ));
        }
    }
    for (label, location) in &result.plan_internal {
        if !result.registries.plan.contains(label) {
            result.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::MissingMint,
                location,
                format!("unresolved planning citation {label}"),
            ));
        }
    }
    for (label, location) in &result.doc_internal {
        if !result.registries.doc.contains(label) {
            result.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::MissingMint,
                location,
                format!("unresolved documentation citation {label}"),
            ));
        }
    }
    for (name, label, location) in &result.crate_internal {
        let resolved = result
            .registries
            .crates
            .get(name)
            .is_some_and(|registry| registry.contains(label));
        if !resolved {
            result.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::MissingMint,
                location,
                format!("unresolved {name} crate citation {label}"),
            ));
        }
    }
    for (imported, location) in &result.imports {
        let registry = match imported.owner {
            LabelOwner::Attestation => Some(&result.registries.attestation),
            LabelOwner::Realization => Some(&result.registries.realization),
            LabelOwner::Adr(number) => result.registries.adrs.get(&number),
            LabelOwner::Model => Some(&result.registries.model),
            LabelOwner::Plan => Some(&result.registries.plan),
            LabelOwner::Doc => Some(&result.registries.doc),
            LabelOwner::Crate(ref name) => result.registries.crates.get(name),
        };
        match registry {
            None => result.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::UnknownOwner,
                location,
                "imported owner has no registry",
            )),
            Some(registry) if !registry.contains(&imported.label) => {
                result.diagnostics.push(LabelDiagnostic::error(
                    LabelErrorCode::UnknownImportedLabel,
                    location,
                    format!(
                        "unresolved imported label {}{}",
                        imported.owner.prefix(),
                        imported.label
                    ),
                ));
            }
            Some(_) => {}
        }
    }
}
fn validate_architecture_weld(result: &mut RepositoryLabels) {
    let location = SourceLocation::new("packages/architecture/src/spec.rs", 1, 1);
    let architecture = &architecture::ARCHITECTURE;
    for label in architecture
        .witnesses
        .iter()
        .map(|witness| witness.semantic_tag)
        .chain(architecture.clauses.iter().map(|clause| clause.as_str()))
    {
        match Label::parse(label, LabelShape::Realization) {
            Ok(label) if result.registries.realization.contains(&label) => {}
            Ok(_) => result.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::ArchitectureLabelMissing,
                &location,
                format!("architecture manifest label {label} is missing from Realization"),
            )),
            Err(error) => result.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::InvalidLabel,
                &location,
                error.to_string(),
            )),
        }
    }
    if let Some(pinned) = architecture.document.specification.anchor_set_hash {
        let actual = architecture::anchor_set_hash(
            result.attestation_anchor_names.iter().map(String::as_str),
        );
        if actual != pinned {
            result.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::AttestationAnchorSetMismatch,
                &location,
                "the pinned attestation anchor-set hash does not match the realization's citations",
            ));
        }
    }
}
fn square(value: &str) -> Option<&str> {
    value.strip_prefix('[')?.strip_suffix(']')
}
fn looks_imported(value: &str) -> bool {
    ImportedLabel::parse(value).is_ok()
}
#[derive(Clone, Debug)]
pub struct GeneratedRegister {
    pub path: PathBuf,
    pub bytes: usize,
}
#[derive(Debug, Error)]
pub enum GenerateError {
    #[error("label source validation failed")]
    Validation(Vec<LabelDiagnostic>),
    #[error("I/O failure: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON rendering failed: {0}")]
    Json(#[from] serde_json::Error),
}
impl GenerateError {
    pub fn diagnostics(&self) -> &[LabelDiagnostic] {
        match self {
            Self::Validation(diagnostics) => diagnostics,
            _ => &[],
        }
    }
}
/// Generate the specification and realization registers from their owning
/// upstream sources only, writing to the argument-supplied output
/// paths (ADR-014: assets go only where arguments route them).
///
/// An unrelated planning or ADR defect must not block regenerating an
/// upstream register (ADR-013 scoped-derivation rule);
/// `check_repository` remains the full repository-wide gate.
pub fn generate_registers(
    paths: &RepositoryCensus,
    specification_output: &Path,
    realization_output: &Path,
) -> Result<Vec<GeneratedRegister>, GenerateError> {
    let labels = derive_model_sources(paths);
    if labels.has_errors() {
        return Err(GenerateError::Validation(labels.diagnostics));
    }
    let outputs = [
        (
            specification_output.to_path_buf(),
            render::specification_register(&labels.registries.attestation),
        ),
        (
            realization_output.to_path_buf(),
            render::realization_register(&labels.registries.realization),
        ),
    ];
    let mut written = Vec::new();
    for (path, contents) in outputs {
        write(&path, contents.as_bytes())?;
        written.push(GeneratedRegister {
            path,
            bytes: contents.len(),
        });
    }
    Ok(written)
}
pub fn model_labels_json(paths: &RepositoryCensus) -> Result<String, GenerateError> {
    let labels = derive_model_sources(paths);
    if labels.has_errors() {
        return Err(GenerateError::Validation(labels.diagnostics));
    }
    Ok(render::model_labels_json(&labels.registries.model)?)
}

fn derive_model_sources(paths: &RepositoryCensus) -> RepositoryLabels {
    let mut result = RepositoryLabels::default();
    // Scoped census verification (ADR-014): a stale plan or ADR census
    // must not block an upstream derivation, but a stale scoped census
    // would silently change the derived registers.
    result
        .diagnostics
        .extend(paths.verify(crate::census::CensusGroup::SCOPED));
    let (attestation, diagnostics) = harvest_attestation(paths);
    result.registries.attestation = attestation;
    result.diagnostics.extend(diagnostics);
    harvest_realization(paths, &mut result);
    add_model(harvest_model(paths), &mut result);
    // Imports naming owners outside this scope are validated by
    // check_repository; validating them here against registries that
    // were never harvested would fail the scoped derivation on a
    // defect in an unrelated owner.
    result.imports.retain(|(imported, _)| {
        matches!(
            imported.owner,
            LabelOwner::Attestation | LabelOwner::Realization | LabelOwner::Model
        )
    });
    validate_references(&mut result);
    sort_diagnostics(&mut result.diagnostics);
    result
}
fn write(path: &Path, bytes: &[u8]) -> Result<(), std::io::Error> {
    let directory = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(directory)?;
    let mut staged = tempfile::Builder::new()
        .prefix(".labels-staged-")
        .tempfile_in(directory)?;
    staged.write_all(bytes)?;
    staged.as_file().sync_all()?;
    staged
        .persist(path)
        .map(|_| ())
        .map_err(|error| error.error)
}
