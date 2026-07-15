use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use thiserror::Error;

use crate::{
    diagnostic::{LabelDiagnostic, LabelErrorCode, sort_diagnostics},
    label::{Label, LabelShape},
    latex::harvest_attestation,
    markdown::{InlineCodeContext, scan_markdown},
    owner::{ImportedLabel, LabelOwner},
    registry::{LabelMint, LabelRegistry, RegistrySet},
    render,
    rust_source::{ModelHarvest, harvest_model},
    source::{SourceLocation, relative_to},
};

#[derive(Clone, Debug)]
pub struct RepositoryPaths {
    pub root: PathBuf,
    pub attestation_main: PathBuf,
    pub attestation_sections: PathBuf,
    pub realization: PathBuf,
    pub adr_dir: PathBuf,
    pub plans_dir: PathBuf,
    pub model_src: PathBuf,
    pub specification_register: PathBuf,
    pub realization_register: PathBuf,
    pub model_labels_json: PathBuf,
}
impl RepositoryPaths {
    pub fn from_root(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref().to_path_buf();
        Self {
            attestation_main: root.join("papers/attestation/main.tex"),
            attestation_sections: root.join("papers/attestation/sections"),
            realization: root.join("docs/attestation/realization.md"),
            adr_dir: root.join("adr"),
            plans_dir: root.join("plans"),
            model_src: root.join("packages/model/src"),
            specification_register: root.join("plans/labels/specification.md"),
            realization_register: root.join("plans/labels/realization.md"),
            model_labels_json: root.join("packages/model/generated/model_labels.json"),
            root,
        }
    }
    pub fn workspace_default() -> Self {
        Self::from_root(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
    }
}

#[derive(Default)]
pub struct RepositoryLabels {
    pub registries: RegistrySet,
    pub diagnostics: Vec<LabelDiagnostic>,
    realization_internal: Vec<(Label, SourceLocation)>,
    adr_internal: Vec<(u16, Label, SourceLocation)>,
    imports: Vec<(ImportedLabel, SourceLocation)>,
    attestation_anchor_names: Vec<String>,
}
impl RepositoryLabels {
    pub fn harvest_sources(paths: &RepositoryPaths) -> Self {
        let mut result = Self::default();
        let (attestation, diagnostics) = harvest_attestation(paths);
        result.registries.attestation = attestation;
        result.diagnostics.extend(diagnostics);
        harvest_realization(paths, &mut result);
        harvest_adrs(paths, &mut result);
        harvest_imports(paths, &mut result);
        let model = harvest_model(paths);
        add_model(model, &mut result);
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

fn add_model(model: ModelHarvest, result: &mut RepositoryLabels) {
    for (label, location) in model.realization_citations {
        result.realization_internal.push((label, location));
    }
    result.registries.model = model.registry;
    result.diagnostics.extend(model.diagnostics);
}
fn harvest_realization(paths: &RepositoryPaths, result: &mut RepositoryLabels) {
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
                if result.registries.realization.insert(mint).is_err() { /* Existing source repeats some locator spans; first mint remains canonical. */
                }
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
fn harvest_adrs(paths: &RepositoryPaths, result: &mut RepositoryLabels) {
    for path in files(&paths.adr_dir) {
        let Some(number) = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.get(..3))
            .and_then(|number| number.parse::<u16>().ok())
        else {
            continue;
        };
        let relative = relative_to(&paths.root, &path);
        let Ok(source) = fs::read_to_string(&path) else {
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
                    if registry.insert(mint).is_err() {
                        result.diagnostics.push(LabelDiagnostic::error(
                            LabelErrorCode::DuplicateMint,
                            &span.location,
                            "duplicate ADR label mint",
                        ));
                    }
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
fn harvest_imports(paths: &RepositoryPaths, result: &mut RepositoryLabels) {
    for path in files(&paths.plans_dir) {
        if path == paths.specification_register || path == paths.realization_register {
            continue;
        }
        let relative = relative_to(&paths.root, &path);
        let Ok(source) = fs::read_to_string(&path) else {
            continue;
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
            } else if looks_imported(&span.content) {
                result.diagnostics.push(LabelDiagnostic::error(
                    LabelErrorCode::InvalidImportedCitationForm,
                    &span.location,
                    "imported citation must use square brackets",
                ));
            }
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
fn harvest_attestation_citations(path: &Path, source: &str, result: &mut RepositoryLabels) {
    let mut offset = 0;
    while let Some(found) = source[offset..].find("[A-") {
        let start = offset + found;
        let body = start + "[A-".len();
        let Some(close) = source[body..].find(']') else {
            break;
        };
        let value = &source[body..body + close];
        if let Ok(label) = Label::parse(value, LabelShape::Attestation) {
            let location = SourceLocation::new(
                path,
                source[..start].lines().count() + 1,
                source[..start]
                    .lines()
                    .last()
                    .map_or(1, |line| line.chars().count() + 1),
            );
            result.attestation_anchor_names.push(value.to_owned());
            result.imports.push((
                ImportedLabel {
                    owner: LabelOwner::Attestation,
                    label,
                },
                location,
            ));
        }
        offset = body + close + 1;
    }
}
fn validate(result: &mut RepositoryLabels) {
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
    for (imported, location) in &result.imports {
        let registry = match imported.owner {
            LabelOwner::Attestation => Some(&result.registries.attestation),
            LabelOwner::Realization => Some(&result.registries.realization),
            LabelOwner::Adr(number) => result.registries.adrs.get(&number),
            LabelOwner::Model => Some(&result.registries.model),
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
    validate_architecture_weld(result);
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
fn files(root: &Path) -> Vec<PathBuf> {
    let mut output = Vec::new();
    walk(root, &mut output);
    output.sort();
    output
}
fn walk(root: &Path, output: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, output);
            } else if path.extension().is_some_and(|extension| extension == "md") {
                output.push(path);
            }
        }
    }
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
pub fn generate_registers(
    paths: &RepositoryPaths,
    output_root: &Path,
) -> Result<Vec<GeneratedRegister>, GenerateError> {
    let labels = RepositoryLabels::harvest_sources(paths);
    if labels.has_errors() {
        return Err(GenerateError::Validation(labels.diagnostics));
    }
    let outputs = [
        (
            output_root.join("plans/labels/specification.md"),
            render::specification_register(&labels.registries.attestation),
        ),
        (
            output_root.join("plans/labels/realization.md"),
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
pub fn model_labels_json(paths: &RepositoryPaths) -> Result<String, GenerateError> {
    let labels = RepositoryLabels::harvest_sources(paths);
    if labels.has_errors() {
        return Err(GenerateError::Validation(labels.diagnostics));
    }
    Ok(render::model_labels_json(&labels.registries.model)?)
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
