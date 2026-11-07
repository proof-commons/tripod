#![doc = include_str!("../README.md")]

use std::path::Path;

use anyhow::{Context, bail, ensure};
use architecture::{ARCHITECTURE, PublishedArchitecture, validate_draft};
use labels::repository::GenerateError;
use serde::Serialize;

pub mod weld;

/// Schema version for the `check-generated` stdout report.
pub const CHECK_REPORT_SCHEMA: u32 = 1;

/// The complete census of generated artifacts, exactly as the files
/// appear in the generated directory. The checker rejects both missing
/// and unexpected entries.
pub const ARTIFACT_NAMES: &[&str] = &[
    "architecture.json",
    "architecture.toml",
    "declassification.json",
    "model_labels.json",
];

/// One rendered artifact: file name plus exact expected bytes.
pub struct ExpectedArtifact {
    /// File name inside the generated directory.
    pub name: &'static str,
    /// Exact expected file contents.
    pub bytes: Vec<u8>,
}

macro_rules! refusal_kind_table {
    ($function:ident($value:ident: $ty:ty), $names:ident, { $($variant:pat => $name:literal),+ $(,)? }) => {
        const fn $function($value: $ty) -> &'static str {
            match $value {
                $($variant => $name,)+
            }
        }

        #[cfg(test)]
        const $names: &[&str] = &[$($name,)+];
    };
}

refusal_kind_table!(manifest_refusal_kind(error: &architecture::ManifestError), MANIFEST_REFUSAL_KINDS, {
    architecture::ManifestError::DuplicateId { .. } => "duplicate_id",
    architecture::ManifestError::MissingId { .. } => "missing_id",
    architecture::ManifestError::InvalidAsset { .. } => "invalid_asset",
    architecture::ManifestError::InvalidRoot { .. } => "invalid_root",
    architecture::ManifestError::InvalidObject { .. } => "invalid_object",
    architecture::ManifestError::InvalidOperation { .. } => "invalid_operation",
    architecture::ManifestError::InvalidQuantity { .. } => "invalid_quantity",
    architecture::ManifestError::InvalidBound { .. } => "invalid_bound",
    architecture::ManifestError::InvalidDecision { .. } => "invalid_decision",
    architecture::ManifestError::InvalidDocument { .. } => "invalid_document",
    architecture::ManifestError::UnpinnedSpecification => "unpinned_specification",
    architecture::ManifestError::DraftPublication => "draft_publication",
});

refusal_kind_table!(label_refusal_kind(code: labels::LabelErrorCode), LABEL_REFUSAL_KINDS, {
    labels::LabelErrorCode::Io => "io",
    labels::LabelErrorCode::UnclosedMarkdownFence => "unclosed_markdown_fence",
    labels::LabelErrorCode::NestedMarkdownFence => "nested_markdown_fence",
    labels::LabelErrorCode::UnclosedInlineCode => "unclosed_inline_code",
    labels::LabelErrorCode::AsymmetricCitation => "asymmetric_citation",
    labels::LabelErrorCode::MalformedCitationGroup => "malformed_citation_group",
    labels::LabelErrorCode::DuplicateMint => "duplicate_mint",
    labels::LabelErrorCode::MissingMint => "missing_mint",
    labels::LabelErrorCode::UnknownOwner => "unknown_owner",
    labels::LabelErrorCode::UnknownImportedLabel => "unknown_imported_label",
    labels::LabelErrorCode::InvalidImportedCitationForm => "invalid_imported_citation_form",
    labels::LabelErrorCode::InvalidLabel => "invalid_label",
    labels::LabelErrorCode::InvalidStatusTag => "invalid_status_tag",
    labels::LabelErrorCode::DuplicateLatexLabel => "duplicate_latex_label",
    labels::LabelErrorCode::ArchitectureLabelMissing => "architecture_label_missing",
    labels::LabelErrorCode::AttestationAnchorSetMismatch => "attestation_anchor_set_mismatch",
    labels::LabelErrorCode::AttestationIndexStale => "attestation_index_stale",
    labels::LabelErrorCode::GeneratedRegisterMissing => "generated_register_missing",
    labels::LabelErrorCode::GeneratedRegisterStale => "generated_register_stale",
    labels::LabelErrorCode::CensusStale => "census_stale",
    labels::LabelErrorCode::CensusUnreadable => "census_unreadable",
    labels::LabelErrorCode::UnknownKind => "unknown_kind",
    labels::LabelErrorCode::ReservedKindWithoutProfile => "reserved_kind_without_profile",
    labels::LabelErrorCode::InventoryKindOutOfPlace => "inventory_kind_out_of_place",
    labels::LabelErrorCode::KindVocabularyDrift => "kind_vocabulary_drift",
    labels::LabelErrorCode::UncataloguedHeadPair => "uncatalogued_head_pair",
    labels::LabelErrorCode::UnregisteredOwner => "unregistered_owner",
    labels::LabelErrorCode::NearMissSpan => "near_miss_span",
    labels::LabelErrorCode::MalformedLabelShape => "malformed_label_shape",
});

fn refusal_kind(
    error: &anyhow::Error,
    architecture_kind: Option<&'static str>,
) -> Option<&'static str> {
    if let Some(kind) = architecture_kind {
        return Some(kind);
    }
    match error.downcast_ref::<GenerateError>() {
        Some(GenerateError::Validation(diagnostics)) => diagnostics
            .first()
            .map(|diagnostic| label_refusal_kind(diagnostic.code)),
        _ => None,
    }
}

fn error_kind(error: &anyhow::Error) -> &'static str {
    if error.downcast_ref::<std::io::Error>().is_some()
        || matches!(
            error.downcast_ref::<GenerateError>(),
            Some(GenerateError::Io(_))
        )
    {
        "io"
    } else if error.downcast_ref::<serde_json::Error>().is_some()
        || error.downcast_ref::<toml::de::Error>().is_some()
        || error.downcast_ref::<toml::ser::Error>().is_some()
        || matches!(
            error.downcast_ref::<GenerateError>(),
            Some(GenerateError::Json(_))
        )
    {
        "serialization"
    } else {
        "internal"
    }
}

/// Render every generated artifact from its typed source, in memory.
///
/// The model-label publication derives from the scoped label census
/// (attestation, realization, and model sources), which arrives by
/// argument (ADR-014): nothing here resolves a repository path from
/// this crate's compiled location.
///
/// The architecture artifacts are validated before rendering: the
/// typed draft must validate, the publication envelope must verify,
/// and both presentation encodings must round-trip to the same DTO.
///
/// # Errors
///
/// Returns a validation or serialization failure; nothing is written.
pub fn expected_artifacts(
    census: &labels::RepositoryCensus,
) -> anyhow::Result<Vec<ExpectedArtifact>> {
    let artifact_count = u64::try_from(ARTIFACT_NAMES.len()).unwrap_or(u64::MAX);
    let span = tracing::span!(
        target: "artifacts::render",
        tracing::Level::INFO,
        "expected_artifacts",
        artifact_count
    );
    let _entered = span.enter();
    let mut architecture_kind = None;
    let result = render_expected(census, &mut architecture_kind);
    match &result {
        Ok(artifacts) => {
            if tracing::enabled!(target: "artifacts::render", tracing::Level::INFO) {
                let rendered_bytes = artifacts.iter().fold(0_u64, |sum, artifact| {
                    sum.saturating_add(u64::try_from(artifact.bytes.len()).unwrap_or(u64::MAX))
                });
                tracing::info!(
                    target: "artifacts::render",
                    outcome = "completed",
                    artifact_count,
                    rendered_bytes,
                    "expected artifacts derived"
                );
            }
        }
        Err(error) => {
            if tracing::enabled!(target: "artifacts::render", tracing::Level::INFO)
                || tracing::enabled!(target: "artifacts::render", tracing::Level::DEBUG)
            {
                if let Some(refusal_kind) = refusal_kind(error, architecture_kind) {
                    tracing::debug!(
                        target: "artifacts::render",
                        outcome = "refused",
                        refusal_kind,
                        "expected artifact derivation refused"
                    );
                } else {
                    tracing::info!(
                        target: "artifacts::render",
                        outcome = "failed",
                        error_kind = error_kind(error),
                        "expected artifact derivation failed"
                    );
                }
            }
        }
    }
    result
}

fn render_expected(
    census: &labels::RepositoryCensus,
    architecture_kind: &mut Option<&'static str>,
) -> anyhow::Result<Vec<ExpectedArtifact>> {
    // The validated wrapper is the only input the publication envelope
    // accepts, so generation cannot skip validation (R2-N03).
    let validated = match validate_draft(&ARCHITECTURE) {
        Ok(validated) => validated,
        Err(errors) => {
            let rendered = errors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n");
            if tracing::enabled!(target: "artifacts::render", tracing::Level::INFO)
                || tracing::enabled!(target: "artifacts::render", tracing::Level::DEBUG)
                || tracing::enabled!(target: "artifacts::check", tracing::Level::INFO)
                || tracing::enabled!(target: "artifacts::check", tracing::Level::DEBUG)
            {
                let kind = errors
                    .first()
                    .map_or("invalid_draft", manifest_refusal_kind);
                *architecture_kind = Some(kind);
            }
            bail!("typed architecture failed draft validation:\n{rendered}");
        }
    };

    let published = PublishedArchitecture::from_architecture(&validated)
        .context("deriving published architecture")?;
    published
        .validate_envelope()
        .context("validating publication envelope")?;

    let json = published
        .to_artifact_json()
        .context("rendering architecture.json")?;
    let toml = published
        .to_artifact_toml()
        .context("rendering architecture.toml")?;

    // Both presentation encodings must round-trip to the same DTO.
    let from_json: PublishedArchitecture =
        serde_json::from_str(&json).context("re-parsing architecture.json")?;
    let from_toml: PublishedArchitecture =
        toml::from_str(&toml).context("re-parsing architecture.toml")?;
    ensure!(
        from_json == published && from_toml == published,
        "serialized architecture artifacts do not round-trip",
    );
    from_json
        .validate_envelope()
        .context("validating re-parsed JSON envelope")?;
    from_toml
        .validate_envelope()
        .context("validating re-parsed TOML envelope")?;

    let model_labels =
        labels::model_labels_json(census).context("deriving model-label registry")?;

    Ok(vec![
        ExpectedArtifact {
            name: "architecture.json",
            bytes: json.into_bytes(),
        },
        ExpectedArtifact {
            name: "architecture.toml",
            bytes: toml.into_bytes(),
        },
        ExpectedArtifact {
            name: "declassification.json",
            bytes: model::artifacts::declassification_json().into_bytes(),
        },
        ExpectedArtifact {
            name: "model_labels.json",
            bytes: model_labels.into_bytes(),
        },
    ])
}

/// Freshness of one artifact in the generated directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactFreshness {
    /// Committed bytes equal the expected bytes.
    Current,
    /// Committed bytes differ from the expected bytes.
    Stale,
    /// The artifact file is absent.
    Missing,
}

/// Per-artifact entry of the check report.
#[derive(Debug, Serialize)]
pub struct ArtifactStatus {
    /// File name inside the generated directory.
    pub name: String,
    /// Freshness verdict.
    pub status: ArtifactFreshness,
}

/// Stdout result object of `check-generated`.
#[derive(Debug, Serialize)]
pub struct CheckReport {
    /// Report schema version.
    pub schema: u32,
    /// Checked directory.
    pub generated_dir: String,
    /// Per-artifact freshness, in census order.
    pub artifacts: Vec<ArtifactStatus>,
    /// Files present in the directory but outside the owned census.
    pub unexpected: Vec<String>,
    /// True when every artifact is current and no unexpected file
    /// exists.
    pub current: bool,
}

/// Compare the committed generated directory against the in-memory
/// expected artifacts. Never writes.
///
/// # Errors
///
/// Returns an error when the expected artifacts cannot be derived or
/// the directory cannot be read; freshness problems are reported in
/// the returned [`CheckReport`], not as errors.
pub fn check(dir: &Path, census: &labels::RepositoryCensus) -> anyhow::Result<CheckReport> {
    let artifact_count = u64::try_from(ARTIFACT_NAMES.len()).unwrap_or(u64::MAX);
    let span = tracing::span!(
        target: "artifacts::check",
        tracing::Level::INFO,
        "check",
        artifact_count
    );
    let _entered = span.enter();
    let mut architecture_kind = None;
    let result = check_core(dir, census, &mut architecture_kind);
    match &result {
        Ok(report) => {
            if tracing::enabled!(target: "artifacts::check", tracing::Level::INFO) {
                let stale_count = u64::try_from(
                    report
                        .artifacts
                        .iter()
                        .filter(|artifact| artifact.status == ArtifactFreshness::Stale)
                        .count(),
                )
                .unwrap_or(u64::MAX);
                let missing_count = u64::try_from(
                    report
                        .artifacts
                        .iter()
                        .filter(|artifact| artifact.status == ArtifactFreshness::Missing)
                        .count(),
                )
                .unwrap_or(u64::MAX);
                let unexpected_count = u64::try_from(report.unexpected.len()).unwrap_or(u64::MAX);
                tracing::info!(
                    target: "artifacts::check",
                    outcome = "completed",
                    artifact_count,
                    stale_count,
                    missing_count,
                    unexpected_count,
                    current = report.current,
                    "generated artifacts checked"
                );
            }
        }
        Err(error) => {
            if tracing::enabled!(target: "artifacts::check", tracing::Level::INFO)
                || tracing::enabled!(target: "artifacts::check", tracing::Level::DEBUG)
            {
                if let Some(refusal_kind) = refusal_kind(error, architecture_kind) {
                    tracing::debug!(
                        target: "artifacts::check",
                        outcome = "refused",
                        refusal_kind,
                        "generated artifact check refused"
                    );
                } else {
                    tracing::info!(
                        target: "artifacts::check",
                        outcome = "failed",
                        error_kind = error_kind(error),
                        "generated artifact check failed"
                    );
                }
            }
        }
    }
    result
}

fn check_core(
    dir: &Path,
    census: &labels::RepositoryCensus,
    architecture_kind: &mut Option<&'static str>,
) -> anyhow::Result<CheckReport> {
    let expected = {
        let span = tracing::span!(
            target: "artifacts::check",
            tracing::Level::DEBUG,
            "render_expected",
            artifact_count = u64::try_from(ARTIFACT_NAMES.len()).unwrap_or(u64::MAX)
        );
        let _entered = span.enter();
        render_expected(census, architecture_kind)?
    };

    let mut artifacts = Vec::new();
    for artifact in &expected {
        let path = dir.join(artifact.name);
        let status = match std::fs::read(&path) {
            Ok(committed) if committed == artifact.bytes => ArtifactFreshness::Current,
            Ok(_) => ArtifactFreshness::Stale,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                ArtifactFreshness::Missing
            }
            Err(error) => {
                return Err(error).context(format!("reading {}", path.display()));
            }
        };
        artifacts.push(ArtifactStatus {
            name: artifact.name.to_owned(),
            status,
        });
    }

    let mut unexpected = Vec::new();
    for entry in std::fs::read_dir(dir).context(format!("listing {}", dir.display()))? {
        let entry = entry.context("reading directory entry")?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !ARTIFACT_NAMES.contains(&name.as_str()) {
            unexpected.push(name);
        }
    }
    unexpected.sort();

    let current = unexpected.is_empty()
        && artifacts
            .iter()
            .all(|artifact| artifact.status == ArtifactFreshness::Current);

    Ok(CheckReport {
        schema: CHECK_REPORT_SCHEMA,
        generated_dir: dir.display().to_string(),
        artifacts,
        unexpected,
        current,
    })
}

#[cfg(test)]
mod tests;
