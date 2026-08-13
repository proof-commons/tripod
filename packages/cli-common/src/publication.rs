//! Batch-staged publication of multi-output generated asset sets.
//!
//! Implements the T3 publication discipline: derive every output's
//! bytes first, compare every destination, stage every changed member
//! as a flushed sibling temporary, and only after **all** staging
//! succeeds begin the final renames. A staging failure therefore
//! changes no final destination, and unchanged destinations keep their
//! bytes and mtimes (compare-if-changed).
//!
//! The final renames are still not one filesystem transaction: a
//! failure during a later rename can leave a partial publication. The
//! command fails in that case, and a subsequent successful invocation
//! repairs the complete set (ADR-017 — no stronger atomicity is
//! claimed).
//!
//! Staged temporaries are created owner-only (`0o600`), so every
//! staging path sets the intended final mode with
//! [`set_publication_mode`] *before* the rename; otherwise a public
//! report, generated artifact, PDF mirror, or flattened source would
//! silently become unreadable to other users, and Git — which records
//! only the executable bit — would not notice. Unchanged destinations
//! are deliberately left completely untouched, so this repairs the
//! mode of every member it actually republishes, not of a destination
//! that already carries the expected bytes.

use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

use crate::AliasedOutputs;

/// Intended final permission class of a published file.
///
/// Publication stages into an owner-only temporary, so the class is
/// applied explicitly rather than inherited from the umask.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublicationMode {
    /// Public data: reports, generated text and data, PDF mirrors, and
    /// flattened source.
    Public,
    /// A published executable.
    Executable,
}

impl PublicationMode {
    /// The Unix mode this publication class carries.
    #[must_use]
    pub const fn octal(self) -> u32 {
        match self {
            Self::Public => 0o644,
            Self::Executable => 0o755,
        }
    }
}

/// Set the intended publication mode on a staged file before it is
/// renamed into its final destination.
///
/// The mode is set exactly, not masked by the process umask: a
/// publication's readability is a property of the publication, not of
/// the environment that happened to produce it.
///
/// # Errors
///
/// Propagates the underlying `fchmod` failure.
pub fn set_publication_mode(file: &std::fs::File, mode: PublicationMode) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        file.set_permissions(std::fs::Permissions::from_mode(mode.octal()))
    }

    // Non-Unix hosts have no equivalent of these bits; the publication
    // discipline is otherwise unchanged.
    #[cfg(not(unix))]
    {
        let _ = (file, mode);
        Ok(())
    }
}

/// One output of a multi-output generation command.
pub struct PublicationAsset<'a> {
    /// Stable diagnostic role of this output (never file contents).
    pub role: &'a str,
    pub path: &'a Path,
    pub bytes: &'a [u8],
}

/// Publication outcome for one asset, in input order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicationResult {
    pub role: String,
    pub path: PathBuf,
    /// False when the destination already carried the expected bytes
    /// and was deliberately left untouched.
    pub changed: bool,
}

/// Failure publishing a batch of generated outputs.
#[derive(Debug, thiserror::Error)]
pub enum BatchPublicationError {
    /// Two output roles resolve to one destination; refused before any
    /// derivation-visible effect.
    #[error(transparent)]
    AliasedOutputs(#[from] AliasedOutputs),

    /// Comparing or staging one output failed; no final destination
    /// was changed and every staged temporary was removed.
    #[error("staging output {role} failed")]
    Stage {
        role: String,
        #[source]
        source: io::Error,
    },

    /// A final rename failed after staging succeeded; earlier members
    /// of this batch may already carry new bytes. The publication set
    /// is repaired by the next successful run.
    #[error("publishing output {role} failed")]
    Publish {
        role: String,
        #[source]
        source: io::Error,
    },
}

/// Publish `assets` with all-staged-before-first-rename discipline.
///
/// Results are returned in input order; callers wanting deterministic
/// publication order sort their asset list before calling.
///
/// # Errors
///
/// [`BatchPublicationError::AliasedOutputs`] before any filesystem
/// effect when two roles share one destination;
/// [`BatchPublicationError::Stage`] with every final destination
/// untouched; [`BatchPublicationError::Publish`] when a final rename
/// fails after staging (earlier members may already be published).
pub fn publish_batch(
    assets: &[PublicationAsset<'_>],
) -> Result<Vec<PublicationResult>, BatchPublicationError> {
    // Phase 1 — validate: lexically distinct destinations and usable
    // parent directories before anything is derived on disk.
    let roles = assets
        .iter()
        .map(|asset| (asset.role, asset.path))
        .collect::<Vec<_>>();
    crate::ensure_distinct_outputs(&roles)?;

    let stage_error = |role: &str, source: io::Error| BatchPublicationError::Stage {
        role: role.to_owned(),
        source,
    };

    for asset in assets {
        let directory = asset.path.parent().unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(directory).map_err(|error| stage_error(asset.role, error))?;
    }

    // Phase 2 — compare and stage every changed member. Temporaries
    // stay in memory; dropping them on any failure removes every
    // staged file while all final destinations remain untouched.
    let mut staged = Vec::new();

    for asset in assets {
        if std::fs::read(asset.path).is_ok_and(|current| current == asset.bytes) {
            staged.push(None);
            continue;
        }

        let directory = asset.path.parent().unwrap_or_else(|| Path::new("."));
        let mut temporary = tempfile::Builder::new()
            .prefix(".publication-staged-")
            .tempfile_in(directory)
            .map_err(|error| stage_error(asset.role, error))?;
        temporary
            .write_all(asset.bytes)
            .and_then(|()| temporary.flush())
            .and_then(|()| temporary.as_file().sync_all())
            .and_then(|()| set_publication_mode(temporary.as_file(), PublicationMode::Public))
            .map_err(|error| stage_error(asset.role, error))?;

        staged.push(Some(temporary));
    }

    // Phase 3 — publish. Only rename steps remain; see the module
    // documentation for the residual multi-rename window.
    let mut results = Vec::new();

    for (asset, temporary) in assets.iter().zip(staged) {
        let changed = match temporary {
            None => false,
            Some(temporary) => {
                temporary
                    .persist(asset.path)
                    .map_err(|error| BatchPublicationError::Publish {
                        role: asset.role.to_owned(),
                        source: error.error,
                    })?;
                true
            }
        };

        results.push(PublicationResult {
            role: asset.role.to_owned(),
            path: asset.path.to_path_buf(),
            changed,
        });
    }

    Ok(results)
}
