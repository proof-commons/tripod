//! Single source of truth for the workspace's generated derivative
//! artifacts (`packages/model/generated/`).
//!
//! The generation/check split is a repository law (`plans` §18.2, and
//! the maintenance gate M0.3): every generated artifact has one typed
//! source, one generator, and one non-writing checker.
//!
//! - `generate-all` (binary) **may write**: it renders every artifact
//!   from its typed source and writes it atomically under `--output`.
//! - `check-generated` (binary) and the test suites **never write**:
//!   they compute the expected bytes in memory through
//!   [`expected_artifacts`] and compare.
//!
//! The owned artifact census is [`ARTIFACT_NAMES`]; the checker also
//! rejects unexpected files in the generated directory, so directory
//! ownership stays explicit. Both binaries follow the ADR-010 output
//! contract (`adr/010-command-line-output-contract.md`).
//!
//! Generated files are *publications*, never semantic inputs: the
//! compiler, linker, and release logic must consume typed Rust, not
//! these artifacts.
//!
//! # Boundary
//!
//! The library itself **never writes**. It renders bytes in memory and
//! compares them; the only writing path in this package is the
//! `generate-all` binary, which publishes the rendered set through
//! `cli_common::publish_batch` so every artifact is staged before the
//! first destination changes. Subject files arrive as a
//! [`labels::RepositoryCensus`] built from command-line arguments
//! (ADR-014); nothing here resolves a repository path from this
//! crate's compiled location.
//!
//! # Map
//!
//! - [`ARTIFACT_NAMES`] — the owned census of generated file names.
//! - [`expected_artifacts`] — render every artifact from its typed
//!   source, in memory, returning [`ExpectedArtifact`] values.
//! - [`check`] — compare a directory against those bytes and return a
//!   [`CheckReport`] of [`ArtifactStatus`] / [`ArtifactFreshness`]
//!   entries plus any unexpected files. Never writes.
//! - [`CHECK_REPORT_SCHEMA`] — schema version of that report.
//! - [`weld`] — structural extraction of the realization document's
//!   attached manifest ([`weld::extract_appendix_toml`]) and masthead
//!   ([`weld::masthead`]), used by the release-integrity weld tests.
//!
//! # Example
//!
//! The census is the only input, so a freshness check is one call:
//!
//! ```no_run
//! // Requires a real repository tree on disk, so this compiles but
//! // does not run as a doctest.
//! let census = labels::RepositoryCensus::default();
//! let report = artifacts::check(std::path::Path::new("packages/model/generated"), &census)?;
//! assert!(report.current, "generated artifacts are stale");
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! See `README.md` in this package for the generator and checker
//! command-line contracts and the regeneration workflow.

use std::path::Path;

use anyhow::{Context, bail, ensure};
use architecture::{ARCHITECTURE, PublishedArchitecture, validate_draft};
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
    let expected = expected_artifacts(census)?;

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
