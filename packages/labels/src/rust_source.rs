use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use crate::{
    RepositoryCensus,
    diagnostic::{LabelDiagnostic, LabelErrorCode},
    label::{Label, LabelShape},
    markdown::nested_fence_diagnostic,
    nearmiss,
    owner::{ImportedLabel, LabelOwner},
    participation::{
        CommentSegment, comment_segments, fence_close, fence_open, nested_fence, region_spans,
    },
    registry::{LabelMint, LabelRegistry},
    repository::{CitationClass, CitationOrigin, LabelCitation},
    source::{SourceLocation, relative_to},
};

pub const MODEL_TYPES: &[&str] = &["def", "rule", "thm", "test", "branch", "proto", "listing"];
pub const REALIZATION_TYPES: &[&str] = &[
    "sec", "app", "req", "inv", "lem", "obl", "trap", "rem", "intuit", "pin", "res", "fig", "tab",
    "leaf",
];

/// Harvest of one Rust-source label owner (the model crate or another
/// first-party crate).
#[derive(Default)]
pub struct RustHarvest {
    pub registry: LabelRegistry,
    pub citations: Vec<LabelCitation>,
    pub diagnostics: Vec<LabelDiagnostic>,
}

pub fn harvest_model(paths: &RepositoryCensus) -> RustHarvest {
    harvest_rust_crate(&paths.root, &paths.model_sources, &LabelOwner::Model)
}

/// Harvest every first-party crate other than the model crate as its
/// own label owner (ADR-019: one owner per Cargo package). Crates with
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

fn harvest_file(path: &Path, source: &str, owner: &LabelOwner, result: &mut RustHarvest) {
    let segments = comment_segments(path, source, &mut result.diagnostics);
    // Fenced Rustdoc examples are nonparticipating, using the same
    // fence recognition as the Markdown scanner (backtick and tilde
    // markers alike). Only documentation comments open a fence, and a
    // fence lives inside one contiguous documentation block: it cannot
    // suppress ordinary comments, other blocks, or another item's
    // documentation, and a block that ends with its fence open is
    // diagnosed at the opening line rather than silently swallowing
    // the rest of the file.
    let mut fence: Option<(char, usize)> = None;
    let mut fence_line = 0;
    let mut fence_block = 0;
    // The logical region the calculus asks pairing to be settled in: a
    // maximal run of consecutive participating segments of one comment
    // block. A block boundary and a fenced example each end it, so no
    // acute span crosses either.
    let mut region: Vec<&CommentSegment> = Vec::new();
    for segment in &segments {
        if fence.is_some() && segment.block != fence_block {
            result.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::UnclosedMarkdownFence,
                &SourceLocation::new(path, fence_line, 1),
                "documentation fence is not closed",
            ));
            fence = None;
        }
        if let Some((marker, length)) = fence {
            if fence_close(&segment.text, marker, length) {
                fence = None;
            }
            harvest_region(path, &mut region, owner, result);
            continue;
        }
        if segment.kind.is_documentation()
            && let Some(open) = fence_open(&segment.text)
        {
            fence = Some(open);
            fence_line = segment.line;
            fence_block = segment.block;
            harvest_region(path, &mut region, owner, result);
            continue;
        }
        // A container-nested fence is outside the accepted grammar in
        // documentation comments too, so its content would be
        // harvested as ordinary comment text.
        if segment.kind.is_documentation()
            && let Some(column) = nested_fence(&segment.text)
        {
            result.diagnostics.push(nested_fence_diagnostic(
                path,
                segment.line,
                segment.column + column - 1,
            ));
        }
        nearmiss::comment(path, segment, &mut result.diagnostics);
        if region
            .last()
            .is_some_and(|last| last.block != segment.block)
        {
            harvest_region(path, &mut region, owner, result);
        }
        region.push(segment);
    }
    harvest_region(path, &mut region, owner, result);
    if fence.is_some() {
        result.diagnostics.push(LabelDiagnostic::error(
            LabelErrorCode::UnclosedMarkdownFence,
            &SourceLocation::new(path, fence_line, 1),
            "documentation fence is not closed",
        ));
    }
}

/// Harvest one logical comment region, draining it.
///
/// Locating and classifying the region's acute spans is
/// [`region_spans`], shared with the Python front-end (ADR-023); what
/// remains here is what a Rust owner does with a located span.
fn harvest_region(
    path: &Path,
    region: &mut Vec<&CommentSegment>,
    owner: &LabelOwner,
    result: &mut RustHarvest,
) {
    for span in region_spans(path, region, &mut result.diagnostics) {
        harvest_label(
            &span.value,
            span.location,
            span.parenthesized,
            owner,
            result,
        );
    }
    region.clear();
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
            Ok(imported) => result.citations.push(LabelCitation {
                source_owner: owner.clone(),
                target: imported,
                origin: CitationOrigin::Source {
                    owner: owner.clone(),
                    location,
                },
                class: CitationClass::AuthoredImported,
            }),
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
            Ok(label) if parenthesized => result.citations.push(LabelCitation {
                source_owner: owner.clone(),
                target: ImportedLabel {
                    owner: owner.clone(),
                    label,
                },
                origin: CitationOrigin::Source {
                    owner: owner.clone(),
                    location,
                },
                class: CitationClass::AuthoredSameOwner,
            }),
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
                error.code(),
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
