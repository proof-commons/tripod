use std::fs;

use serde::Serialize;

use crate::{
    LabelDiagnostic, LabelErrorCode,
    census::{CensusGroup, RepositoryCensus},
    diagnostic::sort_diagnostics,
    render,
    repository::RepositoryLabels,
    source::{SourceLocation, relative_to},
};

pub const CHECK_REPORT_SCHEMA: u32 = 3;
#[derive(Debug, Serialize)]
pub struct CheckReport {
    pub schema: u32,
    pub attestation_labels: usize,
    pub realization_labels: usize,
    pub adr_labels: usize,
    pub model_labels: usize,
    pub planning_labels: usize,
    pub doc_labels: usize,
    pub crate_labels: usize,
    pub imported_citations: usize,
    pub valid: bool,
}
pub fn check_repository(paths: &RepositoryCensus) -> (CheckReport, Vec<LabelDiagnostic>) {
    let mut labels = RepositoryLabels::harvest_sources(paths);
    // Full census verification (ADR-014): the argument census must
    // equal the on-disk discovery for every group.
    labels.diagnostics.extend(paths.verify(CensusGroup::ALL));
    current(
        paths,
        &paths.specification_register,
        &render::specification_register(&labels.registries.attestation),
        &mut labels.diagnostics,
    );
    current(
        paths,
        &paths.realization_register,
        &render::realization_register(&labels.registries.realization),
        &mut labels.diagnostics,
    );
    if let Ok(expected) = render::model_labels_json(&labels.registries.model) {
        current(
            paths,
            &paths.model_labels_json,
            &expected,
            &mut labels.diagnostics,
        );
    }
    sort_diagnostics(&mut labels.diagnostics);
    let valid = !labels.has_errors();
    let report = CheckReport {
        schema: CHECK_REPORT_SCHEMA,
        attestation_labels: labels.registries.attestation.len(),
        realization_labels: labels.registries.realization.len(),
        adr_labels: labels
            .registries
            .adrs
            .values()
            .map(crate::registry::LabelRegistry::len)
            .sum(),
        model_labels: labels.registries.model.len(),
        planning_labels: labels.registries.plan.len(),
        doc_labels: labels.registries.doc.len(),
        crate_labels: labels
            .registries
            .crates
            .values()
            .map(crate::registry::LabelRegistry::len)
            .sum(),
        imported_citations: labels.imported_citation_count(),
        valid,
    };
    (report, labels.diagnostics)
}
fn current(
    paths: &RepositoryCensus,
    path: &std::path::Path,
    expected: &str,
    diagnostics: &mut Vec<LabelDiagnostic>,
) {
    let location = SourceLocation::new(relative_to(&paths.root, path), 1, 1);
    match fs::read_to_string(path) {
        Ok(actual) if actual == expected => {}
        Ok(_) => diagnostics.push(LabelDiagnostic::error(
            LabelErrorCode::GeneratedRegisterStale,
            &location,
            "generated label publication is stale",
        )),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::GeneratedRegisterMissing,
                &location,
                "generated label publication is missing",
            ));
        }
        Err(error) => diagnostics.push(LabelDiagnostic::error(
            LabelErrorCode::Io,
            &location,
            error.to_string(),
        )),
    }
}
