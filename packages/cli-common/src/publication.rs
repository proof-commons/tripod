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
//! only the executable bit — would not notice.
//!
//! Publication freshness therefore has two components, not one
//! (R2-N04). A destination is current when its bytes equal the derived
//! bytes **and** its mode equals the mode the publication class
//! requires. Comparing bytes alone left a byte-current destination
//! stuck at a wrong mode forever: every subsequent run saw equal bytes
//! and skipped the repair. A mode-only mismatch is now repaired in
//! place — the bytes and their modification time are untouched — and
//! the repair is reported separately from a byte rewrite through
//! [`PublicationChange`], so a mode repair is never mistaken for new
//! content. A run after the repair is a no-op.

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

/// Whether a destination already carries the required bytes and mode.
///
/// Read by the compare-if-changed paths before anything is staged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DestinationState {
    /// Absent, unreadable, or carrying different bytes: republish.
    StaleBytes,
    /// Bytes already current, mode wrong: repair the mode only.
    ModeOnly,
    /// Bytes and mode both current: leave completely untouched.
    Current,
}

/// Compare a destination against the bytes and mode a publication
/// requires.
///
/// An unreadable destination is reported as [`DestinationState::StaleBytes`]
/// so the caller republishes it through the ordinary staged path and
/// surfaces any real failure there.
pub fn inspect_destination(path: &Path, bytes: &[u8], mode: PublicationMode) -> DestinationState {
    if !std::fs::read(path).is_ok_and(|current| current == bytes) {
        return DestinationState::StaleBytes;
    }

    if destination_mode_matches(path, mode) {
        DestinationState::Current
    } else {
        DestinationState::ModeOnly
    }
}

/// Whether the destination's permission bits already equal `mode`.
#[cfg(unix)]
fn destination_mode_matches(path: &Path, mode: PublicationMode) -> bool {
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::metadata(path)
        .is_ok_and(|metadata| metadata.permissions().mode() & 0o777 == mode.octal())
}

/// Non-Unix hosts carry no equivalent bits, so mode never differs.
#[cfg(not(unix))]
fn destination_mode_matches(_path: &Path, _mode: PublicationMode) -> bool {
    true
}

/// Repair a byte-current destination's mode without touching its bytes.
///
/// Applied to the destination itself rather than through a staged
/// rename: the published bytes and their modification time must not
/// change, so downstream `restat` consumers stay clean.
///
/// # Errors
///
/// Propagates the underlying `chmod` failure.
#[cfg(unix)]
pub fn repair_publication_mode(path: &Path, mode: PublicationMode) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode.octal()))
}

/// Non-Unix hosts have nothing to repair.
///
/// # Errors
///
/// Never fails on these hosts.
#[cfg(not(unix))]
pub fn repair_publication_mode(_path: &Path, _mode: PublicationMode) -> io::Result<()> {
    Ok(())
}

/// One output of a multi-output generation command.
pub struct PublicationAsset<'a> {
    /// Stable diagnostic role of this output (never file contents).
    pub role: &'a str,
    pub path: &'a Path,
    pub bytes: &'a [u8],
}

/// What a compare-if-changed publication actually altered.
///
/// The two components are reported separately so a mode repair is
/// never read as new content (R2-N04).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PublicationChange {
    /// The destination received new bytes, staged with the required
    /// mode already applied.
    pub bytes_changed: bool,
    /// A byte-current destination's mode was repaired in place; its
    /// bytes and their modification time were not touched.
    pub mode_changed: bool,
}

impl PublicationChange {
    /// A destination that already carried the required bytes and mode.
    #[must_use]
    pub const fn unchanged(self) -> bool {
        !self.bytes_changed && !self.mode_changed
    }
}

/// Publication outcome for one asset, in input order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicationResult {
    pub role: String,
    pub path: PathBuf,
    /// Empty when the destination already carried the expected bytes
    /// and mode and was deliberately left untouched.
    pub change: PublicationChange,
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

/// Publication class every batch asset carries.
///
/// Batch publication covers reports, generated text, and generated
/// data only; published executables are synced by
/// `scripts/cargo-bin-sync.sh`, not by this path.
const ASSET_MODE: PublicationMode = PublicationMode::Public;

/// What phase 2 decided to do with one destination.
enum PlannedPublication {
    /// Bytes and mode already current: touch nothing.
    Current,
    /// Bytes current, mode wrong: repair the mode in place.
    ModeOnly,
    /// Republish these staged bytes over the destination.
    Stage(tempfile::NamedTempFile),
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
    // staged file while all final destinations remain untouched. A
    // byte-current member whose mode is wrong is planned as a mode-only
    // repair: nothing is staged, because its bytes must not move.
    let mut staged = Vec::new();

    for asset in assets {
        match inspect_destination(asset.path, asset.bytes, ASSET_MODE) {
            DestinationState::Current => {
                staged.push(PlannedPublication::Current);
                continue;
            }
            DestinationState::ModeOnly => {
                staged.push(PlannedPublication::ModeOnly);
                continue;
            }
            DestinationState::StaleBytes => {}
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
            .and_then(|()| set_publication_mode(temporary.as_file(), ASSET_MODE))
            .map_err(|error| stage_error(asset.role, error))?;

        staged.push(PlannedPublication::Stage(temporary));
    }

    // Phase 3 — publish. Only rename and mode-repair steps remain; see
    // the module documentation for the residual multi-rename window.
    let mut results = Vec::new();

    for (asset, planned) in assets.iter().zip(staged) {
        let publish_error = |error: io::Error| BatchPublicationError::Publish {
            role: asset.role.to_owned(),
            source: error,
        };

        let change = match planned {
            PlannedPublication::Current => PublicationChange::default(),
            PlannedPublication::ModeOnly => {
                repair_publication_mode(asset.path, ASSET_MODE).map_err(publish_error)?;
                PublicationChange {
                    bytes_changed: false,
                    mode_changed: true,
                }
            }
            PlannedPublication::Stage(temporary) => {
                temporary
                    .persist(asset.path)
                    .map_err(|error| publish_error(error.error))?;
                PublicationChange {
                    bytes_changed: true,
                    mode_changed: false,
                }
            }
        };

        results.push(PublicationResult {
            role: asset.role.to_owned(),
            path: asset.path.to_path_buf(),
            change,
        });
    }

    Ok(results)
}
