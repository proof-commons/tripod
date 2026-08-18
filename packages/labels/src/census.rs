//! The argument-passed repository census and its on-disk verifier
//! (ADR-014).
//!
//! Checker and generator binaries receive every subject file by
//! command-line argument; nothing in this crate resolves a repository
//! path from its own compiled location. The discovery walk survives
//! here as a *verifier*: [`RepositoryCensus::verify`] re-discovers the
//! subjects on disk and hard-fails when the argument census and
//! reality disagree, preserving the ADR-019 guarantee that a new file
//! cannot silently sit outside the label graph.
//!
//! [`RepositoryCensus::discover`] is also the fixture constructor for
//! unit tests, which build synthetic repositories in temporary
//! directories.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    fs, io,
    path::{Path, PathBuf},
};

use crate::{
    diagnostic::{LabelDiagnostic, LabelErrorCode, sort_diagnostics},
    source::{SourceLocation, relative_to},
};

/// Directory names excluded from on-disk census discovery (ADR-019):
/// build products, archives, and vendored trees. Hidden directories
/// are excluded unconditionally.
const EXCLUDED_CENSUS_DIRS: &[&str] = &["archive", "build", "builddir", "target", "vendor"];

/// The complete role-grouped file census of one repository checkout.
///
/// Group membership is stated by the build system (or discovered for
/// test fixtures); classification and shape policy stay in this crate.
/// Relative paths are resolved against `root`.
#[derive(Clone, Debug, Default)]
pub struct RepositoryCensus {
    pub root: PathBuf,
    pub attestation_main: PathBuf,
    pub attestation_sections: Vec<PathBuf>,
    pub realization: PathBuf,
    pub adrs: Vec<PathBuf>,
    pub plans: Vec<PathBuf>,
    pub docs: Vec<PathBuf>,
    pub model_sources: Vec<PathBuf>,
    pub crate_sources: BTreeMap<String, Vec<PathBuf>>,
    /// Generated register publications and the model-label artifact.
    /// Inputs of the currency weld only; empty in scoped censuses that
    /// never touch them.
    pub specification_register: PathBuf,
    pub realization_register: PathBuf,
    pub attestation_register: PathBuf,
    pub model_labels_json: PathBuf,
    /// Traversal failures recorded while discovering this census,
    /// keyed by the group whose walk failed.
    ///
    /// A build-argument census carries none: only
    /// [`RepositoryCensus::discover`] fills this. An unreadable tree
    /// must never become an empty carrier, so
    /// [`RepositoryCensus::verify`] reports these before comparing
    /// declared and discovered members (ADR-014).
    pub traversal: BTreeMap<CensusGroup, Vec<LabelDiagnostic>>,
}

/// Census groups, for scoped verification: an unrelated group's
/// staleness must not block a scoped derivation (ADR-019).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CensusGroup {
    Attestation,
    Realization,
    Adr,
    Plan,
    Doc,
    Model,
    Crates,
}

impl CensusGroup {
    pub const ALL: &[Self] = &[
        Self::Attestation,
        Self::Realization,
        Self::Adr,
        Self::Plan,
        Self::Doc,
        Self::Model,
        Self::Crates,
    ];
    /// The groups feeding the scoped register derivation.
    pub const REGISTER_SCOPED: &[Self] = &[Self::Attestation, Self::Realization];
    /// The groups feeding the scoped model-label derivation.
    pub const SCOPED: &[Self] = &[Self::Attestation, Self::Realization, Self::Model];
}

impl RepositoryCensus {
    /// Discover the census by walking the canonical repository layout
    /// under `root`. Production use is the verifier below; tests use
    /// it to enumerate synthetic fixture repositories. Missing
    /// directories yield empty groups — verification reports the
    /// disagreement, so an unreadable tree cannot silently become an
    /// empty census.
    pub fn discover(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref().to_path_buf();
        let paper = root.join("papers/attestation");
        let mut census = Self {
            attestation_main: paper.join("main.tex"),
            realization: root.join("docs/attestation/realization.md"),
            specification_register: root.join("plans/labels/specification.md"),
            realization_register: root.join("plans/labels/realization.md"),
            attestation_register: root.join("plans/labels/attestation.md"),
            model_labels_json: root.join("packages/model/generated/model_labels.json"),
            root,
            ..Self::default()
        };
        let root = census.root.clone();

        let mut walk = Walk::new(&root);
        census.attestation_sections =
            files_with_extension(&paper.join("sections"), "tex", &mut walk);
        census
            .traversal
            .insert(CensusGroup::Attestation, walk.finish());

        let mut walk = Walk::new(&root);
        for path in markdown_files(&root.join("adr"), &mut walk) {
            if adr_number(&path).is_some() {
                census.adrs.push(path);
            }
        }
        census.traversal.insert(CensusGroup::Adr, walk.finish());

        let mut walk = Walk::new(&root);
        census.plans = markdown_files(&root.join("plans"), &mut walk)
            .into_iter()
            .filter(|path| {
                *path != census.specification_register
                    && *path != census.realization_register
                    && *path != census.attestation_register
            })
            .collect();
        census.traversal.insert(CensusGroup::Plan, walk.finish());

        let mut walk = Walk::new(&root);
        let mut docs = Vec::new();
        walk_docs(&census, &root, &mut docs, &mut walk);
        docs.sort();
        census.docs = docs;
        census.traversal.insert(CensusGroup::Doc, walk.finish());

        let mut walk = Walk::new(&root);
        census.model_sources = rust_files(&root.join("packages/model/src"), &mut walk);
        census.traversal.insert(CensusGroup::Model, walk.finish());

        let mut walk = Walk::new(&root);
        for entry in walk.entries(&root.join("packages")) {
            let Some(name) = entry.path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let source = entry.path.join("src");
            if name == "model" || !entry.is_dir || !source.is_dir() {
                continue;
            }
            census
                .crate_sources
                .insert(name.to_owned(), rust_files(&source, &mut walk));
        }
        census.traversal.insert(CensusGroup::Crates, walk.finish());

        census
    }

    /// Verify this argument census against a fresh on-disk discovery
    /// for the given groups. Every disagreement is a
    /// [`LabelErrorCode::CensusStale`] diagnostic naming the path: the
    /// remedy is a build reconfigure, never a silent reclassification.
    pub fn verify(&self, groups: &[CensusGroup]) -> Vec<LabelDiagnostic> {
        let discovered = Self::discover(&self.root);
        let mut diagnostics = Vec::new();
        for group in groups {
            // An unreadable tree must never become an empty carrier:
            // a group whose walk failed is reported before its
            // membership is compared, because a directory that could
            // not be read may hide any number of subjects (ADR-014).
            // A census that was itself discovered carries the same
            // failures as the fresh walk, so identical records are
            // reported once.
            for source in [self, &discovered] {
                for failure in source.traversal.get(group).into_iter().flatten() {
                    if !diagnostics.contains(failure) {
                        diagnostics.push(failure.clone());
                    }
                }
            }
            let (declared, found): (Vec<&Path>, Vec<&Path>) = match group {
                CensusGroup::Attestation => (
                    std::iter::once(self.attestation_main.as_path())
                        .chain(self.attestation_sections.iter().map(PathBuf::as_path))
                        .collect(),
                    std::iter::once(discovered.attestation_main.as_path())
                        .chain(discovered.attestation_sections.iter().map(PathBuf::as_path))
                        .collect(),
                ),
                CensusGroup::Realization => (
                    vec![self.realization.as_path()],
                    vec![discovered.realization.as_path()],
                ),
                CensusGroup::Adr => (as_paths(&self.adrs), as_paths(&discovered.adrs)),
                CensusGroup::Plan => (as_paths(&self.plans), as_paths(&discovered.plans)),
                CensusGroup::Doc => (as_paths(&self.docs), as_paths(&discovered.docs)),
                CensusGroup::Model => (
                    as_paths(&self.model_sources),
                    as_paths(&discovered.model_sources),
                ),
                CensusGroup::Crates => (
                    self.crate_sources
                        .values()
                        .flatten()
                        .map(PathBuf::as_path)
                        .collect(),
                    discovered
                        .crate_sources
                        .values()
                        .flatten()
                        .map(PathBuf::as_path)
                        .collect(),
                ),
            };
            compare(&self.root, &declared, &found, &mut diagnostics);
        }
        diagnostics
    }

    /// Resolve a path argument against the census root: absolute paths
    /// pass through, relative ones are repository-relative.
    pub fn resolve(root: &Path, path: PathBuf) -> PathBuf {
        if path.is_absolute() {
            path
        } else {
            root.join(path)
        }
    }
}

/// Schema version for the `census-audit` stdout report.
pub const CENSUS_AUDIT_SCHEMA: u32 = 2;

/// The only Git modes the tracked repository may contain: an ordinary
/// blob and an executable blob (ADR-017 repository shape). A symlink
/// (`120000`) or gitlink (`160000`) fails the audit.
pub const ALLOWED_TRACKED_MODES: [&str; 2] = ["100644", "100755"];

/// One entry of the mode-bearing tracked-file listing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrackedEntry<'a> {
    /// The Git mode as printed by the listing, e.g. `100644`.
    pub mode: &'a str,
    /// The repository-relative path.
    pub path: &'a str,
}

impl TrackedEntry<'_> {
    /// Whether this entry is an ordinary tracked blob.
    #[must_use]
    pub fn mode_is_allowed(&self) -> bool {
        ALLOWED_TRACKED_MODES.contains(&self.mode)
    }
}

/// One tracked entry whose Git mode is not an ordinary blob.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct TrackedModeDefect {
    /// The repository-relative path.
    pub path: String,
    /// The rejected Git mode.
    pub mode: String,
}

/// Parse a NUL-separated `git ls-files --stage -z` listing.
///
/// Each record is `<mode> <object> <stage>\t<path>`. A malformed
/// record is an error rather than a skipped entry: silently dropping
/// one would remove exactly the tracked symlink this audit exists to
/// catch.
///
/// # Errors
///
/// Returns the offending record when it carries no tab separator or no
/// mode field.
pub fn parse_tracked_listing(listing: &str) -> Result<Vec<TrackedEntry<'_>>, String> {
    listing
        .split('\0')
        .filter(|record| !record.is_empty())
        .map(|record| {
            let (metadata, path) = record.split_once('\t').ok_or_else(|| {
                format!("tracked listing record has no path separator: {record:?}")
            })?;
            let mode = metadata
                .split(' ')
                .next()
                .filter(|mode| !mode.is_empty())
                .ok_or_else(|| format!("tracked listing record has no mode: {record:?}"))?;
            Ok(TrackedEntry { mode, path })
        })
        .collect()
}

/// Result of auditing the hand-managed build census against the
/// tracked file set (ADR-014).
#[derive(Debug, serde::Serialize)]
pub struct CensusAuditReport {
    /// Report schema version.
    pub schema: u32,
    /// Tracked files considered.
    pub tracked: usize,
    /// Tracked files that are lint subjects after exclusions.
    pub subjects: usize,
    /// Files the build census declares.
    pub declared: usize,
    /// Tracked subjects absent from the declared census: each needs a
    /// line in its directory's meson.build list.
    pub missing_from_census: Vec<String>,
    /// Declared files that are not tracked subjects: stale list
    /// entries, or files that were never `git add`ed.
    pub not_tracked: Vec<String>,
    /// Tracked entries whose Git mode is not an ordinary blob —
    /// symlinks and gitlinks (ADR-017). This covers the complete
    /// tracked set, including paths excluded from lint subjects: lint
    /// exclusion does not exempt a path from the repository-shape
    /// rule.
    pub disallowed_modes: Vec<TrackedModeDefect>,
    /// True when the declared census and the tracked subjects agree
    /// and every tracked entry is an ordinary blob.
    pub valid: bool,
}

/// Audit the declared census against the tracked file list.
///
/// A tracked file is a lint subject unless the categorical exclusion
/// pattern matches it or it appears in the explicit per-directory
/// exclusion list. The declared census must equal the subject set
/// exactly; both directions of disagreement are reported.
///
/// Repository shape is audited over the complete tracked set rather
/// than over subjects: this is the single central owner of the rule
/// that the repository carries no tracked symlink, gitlink, or
/// submodule (ADR-014 tracked-entry modes), so first-party tools do
/// not repeat alias analysis for build-supplied repository paths.
pub fn audit_census<'a>(
    tracked: impl IntoIterator<Item = TrackedEntry<'a>>,
    declared: impl IntoIterator<Item = &'a str>,
    excluded: impl IntoIterator<Item = &'a str>,
    exclude_pattern: &regex::Regex,
) -> CensusAuditReport {
    let excluded: BTreeSet<&str> = excluded.into_iter().collect();
    let declared: BTreeSet<&str> = declared.into_iter().collect();
    let mut tracked_count = 0;
    let mut disallowed_modes = Vec::new();
    let subjects: BTreeSet<&str> = tracked
        .into_iter()
        .inspect(|entry| {
            tracked_count += 1;
            if !entry.mode_is_allowed() {
                disallowed_modes.push(TrackedModeDefect {
                    path: entry.path.to_owned(),
                    mode: entry.mode.to_owned(),
                });
            }
        })
        .map(|entry| entry.path)
        .filter(|path| !exclude_pattern.is_match(path) && !excluded.contains(path))
        .collect();
    disallowed_modes.sort();
    let missing_from_census: Vec<String> = subjects
        .difference(&declared)
        .map(ToString::to_string)
        .collect();
    let not_tracked: Vec<String> = declared
        .difference(&subjects)
        .map(ToString::to_string)
        .collect();
    let valid =
        missing_from_census.is_empty() && not_tracked.is_empty() && disallowed_modes.is_empty();
    CensusAuditReport {
        schema: CENSUS_AUDIT_SCHEMA,
        tracked: tracked_count,
        subjects: subjects.len(),
        declared: declared.len(),
        missing_from_census,
        not_tracked,
        disallowed_modes,
        valid,
    }
}

/// Group role-tagged crate source arguments by crate name, derived
/// from the mandatory `packages/<name>/src/` prefix.
///
/// # Errors
///
/// Returns the offending path when a subject violates the declared
/// role.
pub fn group_crate_sources(
    root: &Path,
    files: impl IntoIterator<Item = PathBuf>,
) -> Result<BTreeMap<String, Vec<PathBuf>>, String> {
    let mut crates: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    for file in files {
        let file = RepositoryCensus::resolve(root, file);
        let relative = relative_to(root, &file);
        let mut components = relative.components().map(|component| {
            component
                .as_os_str()
                .to_str()
                .unwrap_or_default()
                .to_owned()
        });
        match (
            components.next().as_deref(),
            components.next(),
            components.next().as_deref(),
        ) {
            (Some("packages"), Some(name), Some("src")) if name != "model" => {
                crates.entry(name).or_default().push(file);
            }
            _ => {
                return Err(format!(
                    "crate source outside packages/<name>/src: {}",
                    relative.display()
                ));
            }
        }
    }
    Ok(crates)
}

fn as_paths(paths: &[PathBuf]) -> Vec<&Path> {
    paths.iter().map(PathBuf::as_path).collect()
}

fn compare(
    root: &Path,
    declared: &[&Path],
    found: &[&Path],
    diagnostics: &mut Vec<LabelDiagnostic>,
) {
    let declared: BTreeSet<&Path> = declared.iter().copied().collect();
    let found: BTreeSet<&Path> = found.iter().copied().collect();
    for missing in declared.difference(&found) {
        diagnostics.push(stale(
            root,
            missing,
            "census names a file absent on disk; rerun meson setup",
        ));
    }
    for extra in found.difference(&declared) {
        diagnostics.push(stale(
            root,
            extra,
            "file is outside the build census; rerun meson setup",
        ));
    }
}

fn stale(root: &Path, path: &Path, message: &str) -> LabelDiagnostic {
    LabelDiagnostic::error(
        LabelErrorCode::CensusStale,
        &SourceLocation::new(relative_to(root, path), 1, 1),
        message,
    )
}

fn adr_number(path: &Path) -> Option<u16> {
    path.file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.get(..3))
        .and_then(|number| number.parse::<u16>().ok())
}

/// One directory entry of a census walk, classified by following
/// symbolic links exactly as the previous `is_dir` test did.
struct Entry {
    path: PathBuf,
    is_dir: bool,
}

/// Failure-recording traversal for one census group.
///
/// Every unreadable directory and every unreadable entry becomes a
/// [`LabelErrorCode::CensusUnreadable`] diagnostic naming the path.
/// Nothing is dropped silently: a suppressed failure would let an
/// unreadable subject tree agree with an empty declared group and pass
/// verification (ADR-014).
struct Walk<'a> {
    root: &'a Path,
    diagnostics: Vec<LabelDiagnostic>,
}

impl<'a> Walk<'a> {
    const fn new(root: &'a Path) -> Self {
        Self {
            root,
            diagnostics: Vec::new(),
        }
    }

    /// Read one directory, sorted, recording failures.
    ///
    /// An absent directory is an empty group rather than a failure:
    /// the declared census disagreement already names a removed
    /// subject. Every other error kind — permissions above all — is a
    /// diagnostic, because the directory may hold subjects that cannot
    /// be seen.
    fn entries(&mut self, directory: &Path) -> Vec<Entry> {
        let mut entries = Vec::new();
        let iterator = match fs::read_dir(directory) {
            Ok(iterator) => iterator,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return entries,
            Err(error) => {
                self.unreadable(directory, &error);
                return entries;
            }
        };
        for entry in iterator {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    self.unreadable(directory, &error);
                    continue;
                }
            };
            let path = entry.path();
            match fs::metadata(&path) {
                Ok(metadata) => entries.push(Entry {
                    is_dir: metadata.is_dir(),
                    path,
                }),
                Err(error) => self.unreadable(&path, &error),
            }
        }
        entries.sort_by(|left, right| left.path.cmp(&right.path));
        entries
    }

    fn unreadable(&mut self, path: &Path, error: &dyn Display) {
        self.diagnostics.push(LabelDiagnostic::error(
            LabelErrorCode::CensusUnreadable,
            &SourceLocation::new(relative_to(self.root, path), 1, 1),
            format!("census traversal could not read this path: {error}"),
        ));
    }

    /// The recorded failures in a deterministic order.
    fn finish(self) -> Vec<LabelDiagnostic> {
        let mut diagnostics = self.diagnostics;
        sort_diagnostics(&mut diagnostics);
        diagnostics
    }
}

fn files_with_extension(directory: &Path, extension: &str, walk: &mut Walk<'_>) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = walk
        .entries(directory)
        .into_iter()
        .filter(|entry| {
            entry
                .path
                .extension()
                .is_some_and(|found| found == extension)
        })
        .map(|entry| entry.path)
        .collect();
    files.sort();
    files
}

fn markdown_files(directory: &Path, walk: &mut Walk<'_>) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_by_extension(directory, "md", &mut files, walk);
    files.sort();
    files
}

fn rust_files(directory: &Path, walk: &mut Walk<'_>) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_by_extension(directory, "rs", &mut files, walk);
    files.sort();
    files
}

fn collect_by_extension(
    directory: &Path,
    extension: &str,
    files: &mut Vec<PathBuf>,
    walk: &mut Walk<'_>,
) {
    for entry in walk.entries(directory) {
        if entry.is_dir {
            collect_by_extension(&entry.path, extension, files, walk);
        } else if entry
            .path
            .extension()
            .is_some_and(|found| found == extension)
        {
            files.push(entry.path);
        }
    }
}

/// Discover every authored Markdown file outside the trees owned
/// elsewhere: the `DOC` owner census (ADR-019).
fn walk_docs(
    census: &RepositoryCensus,
    directory: &Path,
    output: &mut Vec<PathBuf>,
    walk: &mut Walk<'_>,
) {
    let adr_dir = census.root_join_adr();
    let plans_dir = census.root_join_plans();
    for entry in walk.entries(directory) {
        let path = entry.path;
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if entry.is_dir {
            if name.starts_with('.') || EXCLUDED_CENSUS_DIRS.contains(&name) || path == plans_dir {
                continue;
            }
            walk_docs(census, &path, output, walk);
        } else if path.extension().is_some_and(|extension| extension == "md") {
            // Files with their own owner: the realization document and
            // numbered ADRs.
            if path == census.realization {
                continue;
            }
            if path.parent() == Some(adr_dir.as_path()) && adr_number(&path).is_some() {
                continue;
            }
            output.push(path);
        }
    }
}

impl RepositoryCensus {
    fn root_join_adr(&self) -> PathBuf {
        self.root.join("adr")
    }
    fn root_join_plans(&self) -> PathBuf {
        self.root.join("plans")
    }
}
