use std::{collections::BTreeSet, fs, path::Path};

use crate::{
    RepositoryPaths,
    diagnostic::{LabelDiagnostic, LabelErrorCode},
    label::{Label, LabelShape},
    owner::LabelOwner,
    registry::{LabelMint, LabelRegistry},
    source::{SourceLocation, relative_to},
};

pub const MODEL_TYPES: &[&str] = &["def", "rule", "thm", "test", "branch", "protocol", "lst"];
pub const REALIZATION_TYPES: &[&str] = &[
    "sec", "subsec", "app", "req", "inv", "lem", "obl", "trap", "rem", "ins", "rule", "pin", "res",
    "fig", "tbl", "leaf",
];

#[derive(Default)]
pub struct ModelHarvest {
    pub registry: LabelRegistry,
    pub realization_citations: Vec<(Label, SourceLocation)>,
    pub diagnostics: Vec<LabelDiagnostic>,
}
pub fn harvest_model(paths: &RepositoryPaths) -> ModelHarvest {
    let mut result = ModelHarvest::default();
    let mut files = Vec::new();
    collect(&paths.model_src, &mut files);
    files.sort();
    for path in files {
        let relative = relative_to(&paths.root, &path);
        let Ok(source) = fs::read_to_string(&path) else {
            continue;
        };
        for (index, line) in source.lines().enumerate() {
            harvest_line(&relative, index + 1, line, &mut result);
        }
    }
    result
}

fn harvest_line(path: &Path, line_number: usize, line: &str, result: &mut ModelHarvest) {
    let mut opening = None;
    for (offset, character) in line.char_indices() {
        if character != '\u{b4}' {
            continue;
        }
        if let Some(start) = opening.take() {
            let location =
                SourceLocation::new(path, line_number, line[..start].chars().count() + 1);
            harvest_label(
                &line[start + character.len_utf8()..offset],
                location,
                result,
            );
        } else {
            opening = Some(offset);
        }
    }
    if let Some(offset) = opening {
        result.diagnostics.push(LabelDiagnostic::error(
            LabelErrorCode::UnclosedInlineCode,
            &SourceLocation::new(path, line_number, line[..offset].chars().count() + 1),
            "unclosed acute label delimiter",
        ));
    }
}

fn harvest_label(value: &str, location: SourceLocation, result: &mut ModelHarvest) {
    let kind = value.split(':').next().unwrap_or_default();
    if MODEL_TYPES.contains(&kind) {
        match Label::parse(value, LabelShape::Model) {
            Ok(label) => {
                let mint = LabelMint {
                    owner: LabelOwner::Model,
                    label,
                    location,
                    home: None,
                };
                drop(result.registry.insert(mint));
            }
            Err(error) => result.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::InvalidLabel,
                &location,
                error.to_string(),
            )),
        }
    } else if REALIZATION_TYPES.contains(&kind) {
        match Label::parse(value, LabelShape::Realization) {
            Ok(label) => result.realization_citations.push((label, location)),
            Err(error) => result.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::InvalidLabel,
                &location,
                error.to_string(),
            )),
        }
    } else {
        result.diagnostics.push(LabelDiagnostic::error(
            LabelErrorCode::InvalidLabel,
            &location,
            format!("unknown model label type {kind}"),
        ));
    }
}
fn collect(directory: &Path, files: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = fs::read_dir(directory) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                collect(&path, files);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
        }
    }
}
pub fn model_labels(registry: &LabelRegistry) -> BTreeSet<String> {
    registry.labels().map(ToString::to_string).collect()
}
