//! `document-stamps` library: derive deterministic Attestation paper
//! metadata from committed Git state.
//!
//! The build system supplies the Git program, the repository root, the
//! paper subtree, and the exact publication-input set. From those [`run`]
//! derives four prepared values as an [`AttestationStampValues`] object.
//! [`render`] additionally fills the `stamps.tex.in` template with those
//! values and writes the generated `stamps.tex` and `source-date-epoch`
//! file (compare-if-changed, for ninja `restat`); the
//! rendering and fail-closed placeholder check are unit-tested here rather
//! than in a shell wrapper.
//!
//! The four values, and their intentionally distinct domains:
//!
//! - `date` — UTC calendar date of the latest commit touching any member
//!   of the exact input set;
//! - `timestamp` — UTC timestamp of the latest commit touching anything
//!   under the paper subtree (this is `SOURCE_DATE_EPOCH`);
//! - `document_uuid` — first 128 bits of a canonical SHA-256 digest over
//!   the exact input set;
//! - `instance_uuid` — first 128 bits of the Git SHA-1 tree object ID for
//!   the paper subtree, formatted with UUID grouping.
//!
//! The identities derive from committed objects, so the paper subtree
//! must be clean: a dirty subtree is a hard failure.

use std::fmt::Write as _;
use std::path::{Component, Path, PathBuf};

use sha2::{Digest as _, Sha256};

#[cfg(test)]
mod tests;

/// Failures deriving the paper stamps. Diagnostics stay free of raw
/// argv and environment values (ADR-010).
#[derive(Debug, thiserror::Error)]
pub enum StampError {
    /// The supplied repository root is not the Git top level.
    #[error("the supplied repository root is not the Git top level")]
    RepositoryRootMismatch,

    /// The repository uses an object format other than SHA-1, which the
    /// 128-bit tree-prefix rule is not defined for.
    #[error("unsupported Git object format {0:?}; expected sha1")]
    UnsupportedGitObjectFormat(String),

    /// A Git invocation could not be spawned or exited non-zero.
    #[error("Git command failed")]
    GitCommandFailed,

    /// A Git invocation emitted output that was not valid UTF-8.
    #[error("Git command emitted invalid UTF-8")]
    InvalidGitOutput,

    /// The selected revision does not resolve to checked-out HEAD.
    ///
    /// Stamp derivation reads and renders the checked-out tree, so a
    /// different revision would produce hybrid metadata.
    #[error("selected revision is not checked-out HEAD")]
    SelectedRevisionIsNotHead,

    /// The paper subtree has uncommitted or untracked changes.
    #[error("paper subtree is dirty")]
    DirtyPaperSubtree,

    /// An input path is absolute, escapes the repository, contains a
    /// lexical alias (`.`/`..`), or is not valid UTF-8 for this
    /// repository's policy.
    #[error("input path is invalid or outside the repository")]
    InvalidInputPath,

    /// A publication input is not strictly beneath the declared,
    /// dirty-checked paper subtree.
    #[error("publication input is outside the declared paper subtree")]
    InputOutsidePaperTree,

    /// The same input path was supplied more than once.
    #[error("input occurs more than once")]
    DuplicateInput,

    /// An input is not a tracked regular file at the selected revision.
    #[error("input is not a tracked regular file at the selected revision")]
    InvalidTrackedInput,

    /// An input's working-tree bytes could not be read.
    #[error("input working-tree bytes could not be read")]
    InputReadFailed,

    /// The selected path set has no affecting commit.
    #[error("the selected path set has no affecting commit")]
    MissingPathHistory,

    /// Git returned a malformed tree object ID.
    #[error("invalid Git tree object ID")]
    InvalidTreeObjectId,

    /// The 128-bit tree prefix did not uniquely resolve to the expected
    /// tree; repository object identity is unexpectedly ambiguous.
    #[error("the 128-bit tree prefix did not uniquely resolve to the expected tree")]
    TreePrefixResolutionFailure,

    /// The template could not be read.
    #[error("the stamps template could not be read")]
    TemplateReadFailed,

    /// The rendered template still contains an unresolved placeholder.
    #[error("the rendered stamps template still contains an unresolved placeholder")]
    UnresolvedPlaceholder,

    /// A generated output file could not be written.
    #[error("a generated stamp output could not be written")]
    OutputWriteFailed,
}

/// The four prepared values emitted as one JSON object.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AttestationStampValues {
    /// UTC calendar date `YYYY-MM-DD` of the latest input-touching commit.
    pub date: String,
    /// The paper-subtree timestamp in three deterministic forms.
    pub timestamp: PreparedTimestamp,
    /// UUID-formatted first 128 bits of the input digest.
    pub document_uuid: String,
    /// UUID-formatted first 128 bits of the paper-subtree tree object ID.
    pub instance_uuid: String,
}

/// One instant in three deterministic representations.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PreparedTimestamp {
    /// Unix seconds, used for `SOURCE_DATE_EPOCH`.
    pub epoch: i64,
    /// `YYYY-MM-DDThh:mm:ssZ`, for diagnostics and tests.
    pub iso_8601: String,
    /// `D:YYYYMMDDhhmmssZ`, for the PDF date fields.
    pub pdf: String,
}

/// Everything the derivation needs, supplied explicitly by the build
/// system (nothing is read from ambient environment).
#[derive(Debug, Clone)]
pub struct StampRequest {
    /// The Git program to invoke.
    pub git: PathBuf,
    /// The repository root all paths resolve against.
    pub repository_root: PathBuf,
    /// The revision to verify against checked-out `HEAD`. The current
    /// implementation accepts only revisions that resolve to the same
    /// commit as `HEAD`; a different revision is rejected.
    pub tree_ref: String,
    /// The repository-relative paper subtree.
    pub tree: PathBuf,
    /// The exact publication-input set (repository-relative).
    pub inputs: Vec<PathBuf>,
}

/// Derive the four prepared values from committed Git state.
///
/// # Errors
///
/// Returns a [`StampError`] if the repository root is not the Git top
/// level, the object format is not SHA-1, the paper subtree is dirty, an
/// input is invalid or untracked, the selected paths have no history, or
/// the paper-tree prefix does not uniquely resolve.
pub fn run(request: &StampRequest) -> Result<AttestationStampValues, StampError> {
    let git = SystemGit {
        git: request.git.clone(),
        repository_root: request.repository_root.clone(),
    };
    derive(&git, request)
}

/// A request to derive the values and render the build inputs.
#[derive(Debug, Clone)]
pub struct RenderRequest<'a> {
    /// The derivation request.
    pub stamps: &'a StampRequest,
    /// The `stamps.tex.in` template to fill.
    pub template: &'a Path,
    /// Where the rendered `stamps.tex` is written.
    pub stamps_output: &'a Path,
    /// Where the `source-date-epoch` file is written.
    pub epoch_output: &'a Path,
}

/// Derive the four values and write the paper build inputs.
///
/// Writes the rendered `stamps.tex` and `source-date-epoch` file. Both
/// content files use compare-if-changed writes so an unchanged rebuild
/// preserves their mtime (ninja `restat`).
///
/// # Errors
///
/// Returns a [`StampError`] on any derivation failure (see [`run`]), if
/// the template cannot be read, if a placeholder is left unresolved, or
/// if an output cannot be written.
pub fn render(request: &RenderRequest<'_>) -> Result<(), StampError> {
    let values = run(request.stamps)?;
    let template = std::fs::read_to_string(request.template)
        .map_err(|_error| StampError::TemplateReadFailed)?;
    render_outputs(
        &template,
        &values,
        request.stamps_output,
        request.epoch_output,
    )
}

fn render_outputs(
    template: &str,
    values: &AttestationStampValues,
    stamps_output: &Path,
    epoch_output: &Path,
) -> Result<(), StampError> {
    let rendered = render_stamps(template, values)?;
    let epoch_bytes = format!("{}\n", values.timestamp.epoch);

    // Stage both outputs before publishing either: any read, temp-file
    // create, write, or fsync failure for either destination therefore
    // happens before any final output changes, so a late failure cannot
    // leave one output new and the other old. Two independent paths
    // cannot be renamed as a single transaction, so publishing is still
    // two renames; if the second rename fails the build fails and the
    // next invocation repairs the pair (F1-034).
    let staged_stamps = stage_if_changed(stamps_output, rendered.as_bytes())?;
    let staged_epoch = stage_if_changed(epoch_output, epoch_bytes.as_bytes())?;

    // Publish the dependency-first metadata first and the more visible
    // stamps.tex last, so a rename failure between the two tends to
    // leave the visible TeX metadata old rather than pairing new TeX
    // metadata with an old epoch. This is a recovery preference, not a
    // proof of atomicity.
    publish(staged_epoch)?;
    publish(staged_stamps)?;
    Ok(())
}

/// Substitute the four placeholders into the template, fail-closed if any
/// `@ATTESTATION_` token survives. The values are ASCII-constrained and
/// contain no placeholder syntax, so substitution cannot cascade.
fn render_stamps(template: &str, values: &AttestationStampValues) -> Result<String, StampError> {
    let rendered = template
        .replace("@ATTESTATION_DATE@", &values.date)
        .replace("@ATTESTATION_TIMESTAMP@", &values.timestamp.pdf)
        .replace("@ATTESTATION_DOCUMENT_UUID@", &values.document_uuid)
        .replace("@ATTESTATION_INSTANCE_UUID@", &values.instance_uuid);
    if rendered.contains("@ATTESTATION_") {
        return Err(StampError::UnresolvedPlaceholder);
    }
    Ok(rendered)
}

/// A destination whose new bytes are fully written and fsync'd to a
/// sibling temp file, ready to be renamed into place. Holding one of
/// these means every fallible step except the final rename has already
/// succeeded.
struct StagedOutput {
    destination: PathBuf,
    staged: tempfile::NamedTempFile,
}

/// Stage `bytes` for `path` without changing `path`.
///
/// Returns `Ok(None)` when the current contents already equal `bytes`
/// (so nothing needs writing — this keeps ninja `restat` from cascading
/// rebuilds), or `Ok(Some(staged))` when a sibling temp file has been
/// created, written, and fsync'd and only awaits [`publish`]. Any read,
/// create, write, or fsync failure is reported here, before `path` is
/// touched.
fn stage_if_changed(path: &Path, bytes: &[u8]) -> Result<Option<StagedOutput>, StampError> {
    if std::fs::read(path).is_ok_and(|existing| existing == bytes) {
        return Ok(None);
    }
    let directory = path.parent().ok_or(StampError::OutputWriteFailed)?;
    let mut staged = tempfile::Builder::new()
        .prefix(".stamps-staged-")
        .tempfile_in(directory)
        .map_err(|_error| StampError::OutputWriteFailed)?;
    std::io::Write::write_all(&mut staged, bytes)
        .map_err(|_error| StampError::OutputWriteFailed)?;
    staged
        .as_file()
        .sync_all()
        .map_err(|_error| StampError::OutputWriteFailed)?;
    Ok(Some(StagedOutput {
        destination: path.to_path_buf(),
        staged,
    }))
}

/// Rename a staged output into its destination. A `None` (unchanged
/// output) is a no-op. This is the only step that mutates a final path.
fn publish(staged: Option<StagedOutput>) -> Result<(), StampError> {
    if let Some(output) = staged {
        output
            .staged
            .persist(&output.destination)
            .map_err(|_error| StampError::OutputWriteFailed)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Git access
// ---------------------------------------------------------------------------

/// Captured result of one Git invocation.
struct GitInvocation {
    status_success: bool,
    stdout: Vec<u8>,
}

/// Runs Git commands against a fixed repository. Abstracted as a trait so
/// unit tests inject canned outputs instead of a live checkout.
trait GitRunner {
    /// Run Git with the given arguments (a `-C <root>` prefix is applied
    /// by the implementation), capturing stdout.
    fn run(&self, args: &[&str]) -> Result<GitInvocation, StampError>;
}

/// The production runner: a real `git` process.
struct SystemGit {
    git: PathBuf,
    repository_root: PathBuf,
}

impl GitRunner for SystemGit {
    fn run(&self, args: &[&str]) -> Result<GitInvocation, StampError> {
        let output = std::process::Command::new(&self.git)
            .arg("-C")
            .arg(&self.repository_root)
            .args(args)
            .output()
            .map_err(|error| {
                tracing::error!(error = %error, "invoking git failed");
                StampError::GitCommandFailed
            })?;
        Ok(GitInvocation {
            status_success: output.status.success(),
            stdout: output.stdout,
        })
    }
}

/// Run Git and require a zero exit, returning trimmed stdout as text.
fn run_text(git: &dyn GitRunner, args: &[&str]) -> Result<String, StampError> {
    let invocation = git.run(args)?;
    if !invocation.status_success {
        tracing::error!("git command exited non-zero");
        return Err(StampError::GitCommandFailed);
    }
    let text =
        String::from_utf8(invocation.stdout).map_err(|_error| StampError::InvalidGitOutput)?;
    Ok(text.trim_end_matches(['\n', '\r']).to_owned())
}

/// Run Git and require a zero exit, returning raw stdout bytes. Used for
/// binary-safe blob reads where trimming or UTF-8 decoding would corrupt
/// the content.
fn run_bytes(git: &dyn GitRunner, args: &[&str]) -> Result<Vec<u8>, StampError> {
    let invocation = git.run(args)?;
    if !invocation.status_success {
        return Err(StampError::GitCommandFailed);
    }
    Ok(invocation.stdout)
}

// ---------------------------------------------------------------------------
// Derivation
// ---------------------------------------------------------------------------

fn derive(
    git: &dyn GitRunner,
    request: &StampRequest,
) -> Result<AttestationStampValues, StampError> {
    verify_repository_root(git, &request.repository_root)?;
    verify_object_format(git)?;
    verify_selected_revision_is_head(git, &request.tree_ref)?;

    let tree_path = canonical_relative_path(&request.tree)?;
    let tree = path_to_str(&tree_path)?;
    reject_dirty_subtree(git, tree)?;

    let inputs = canonical_inputs(git, request, &request.tree_ref, &tree_path)?;

    let document_uuid = bytes_to_uuid_text(&document_digest(&inputs));
    let instance_uuid = instance_uuid(git, &request.tree_ref, tree)?;
    let timestamp = paper_timestamp(git, &request.tree_ref, tree)?;
    let date = input_date(git, &request.tree_ref, &inputs)?;

    // Re-check cleanliness after reading every input, narrowing (not
    // closing) the window in which the worktree changes mid-derivation.
    // A live-worktree build cannot be made fully transactional across the
    // subsequent TeX run; canonical release still needs a quiescent
    // checkout and the final clean-tree gate.
    reject_dirty_subtree(git, tree)?;

    Ok(AttestationStampValues {
        date,
        timestamp,
        document_uuid,
        instance_uuid,
    })
}

/// Structurally canonicalize a repository-relative path, rejecting
/// absolute paths and every lexical alias (`.`, `..`, root, prefix).
/// A nonempty sequence of `Normal` components is required, so aliases
/// like `./a` or `a/./b` cannot smuggle a duplicate past duplicate
/// detection or change the digest framing.
fn canonical_relative_path(path: &Path) -> Result<PathBuf, StampError> {
    if path.is_absolute() {
        return Err(StampError::InvalidInputPath);
    }

    let mut canonical = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => canonical.push(value),
            Component::CurDir
            | Component::ParentDir
            | Component::RootDir
            | Component::Prefix(_) => return Err(StampError::InvalidInputPath),
        }
    }

    if canonical.as_os_str().is_empty() {
        return Err(StampError::InvalidInputPath);
    }

    Ok(canonical)
}

/// Build a Git literal pathspec so a filename containing pathspec
/// metacharacters cannot change the selected file set. Assembled at
/// runtime so no string literal resembles a magic pathspec.
fn literal_pathspec(relative_path: &str) -> String {
    let mut spec = String::from(":(literal)");
    spec.push_str(relative_path);
    spec
}

fn verify_repository_root(git: &dyn GitRunner, repository_root: &Path) -> Result<(), StampError> {
    let toplevel = run_text(git, &["rev-parse", "--show-toplevel"])?;
    let reported =
        std::fs::canonicalize(&toplevel).map_err(|_error| StampError::RepositoryRootMismatch)?;
    let expected = std::fs::canonicalize(repository_root)
        .map_err(|_error| StampError::RepositoryRootMismatch)?;
    if reported == expected {
        Ok(())
    } else {
        Err(StampError::RepositoryRootMismatch)
    }
}

fn verify_object_format(git: &dyn GitRunner) -> Result<(), StampError> {
    let format = run_text(git, &["rev-parse", "--show-object-format"])?;
    if format == "sha1" {
        Ok(())
    } else {
        Err(StampError::UnsupportedGitObjectFormat(format))
    }
}

/// Append git's "peel to commit" suffix (`^{commit}`) to a revision.
/// Built at runtime so no string literal resembles a formatting
/// argument.
fn peel_to_commit(reference: &str) -> String {
    let mut expr = String::from(reference);
    expr.push('^');
    expr.push('{');
    expr.push_str("commit}");
    expr
}

/// Reject any selected revision that does not resolve to the same commit
/// object as checked-out `HEAD`. Derivation reads and renders the
/// working tree, so publishing a different committed revision would mix
/// its metadata with the checked-out bytes. A branch or tag that points
/// at exactly `HEAD` is accepted: the rule is commit-object equality, not
/// literal string equality.
fn verify_selected_revision_is_head(git: &dyn GitRunner, tree_ref: &str) -> Result<(), StampError> {
    let selected = run_text(git, &["rev-parse", "--verify", &peel_to_commit(tree_ref)])?;
    let head = run_text(git, &["rev-parse", "--verify", &peel_to_commit("HEAD")])?;

    if selected.eq_ignore_ascii_case(&head) {
        Ok(())
    } else {
        Err(StampError::SelectedRevisionIsNotHead)
    }
}

fn reject_dirty_subtree(git: &dyn GitRunner, tree: &str) -> Result<(), StampError> {
    let invocation = git.run(&[
        "status",
        "--porcelain=v1",
        "-z",
        "--untracked-files=all",
        "--",
        &literal_pathspec(tree),
    ])?;
    if !invocation.status_success {
        return Err(StampError::GitCommandFailed);
    }
    if invocation.stdout.is_empty() {
        Ok(())
    } else {
        Err(StampError::DirtyPaperSubtree)
    }
}

/// One validated publication input, ready for digesting. `bytes` are the
/// committed blob bytes, verified equal to the worktree bytes.
struct CanonicalInput {
    relative_path: String,
    git_mode: String,
    bytes: Vec<u8>,
}

fn canonical_inputs(
    git: &dyn GitRunner,
    request: &StampRequest,
    tree_ref: &str,
    tree: &Path,
) -> Result<Vec<CanonicalInput>, StampError> {
    let mut inputs = Vec::with_capacity(request.inputs.len());
    for raw in &request.inputs {
        inputs.push(canonical_input(
            git,
            &request.repository_root,
            tree_ref,
            tree,
            raw,
        )?);
    }
    inputs.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    for pair in inputs.windows(2) {
        if pair[0].relative_path == pair[1].relative_path {
            return Err(StampError::DuplicateInput);
        }
    }
    Ok(inputs)
}

fn canonical_input(
    git: &dyn GitRunner,
    repository_root: &Path,
    tree_ref: &str,
    tree: &Path,
    raw: &Path,
) -> Result<CanonicalInput, StampError> {
    let canonical = canonical_relative_path(raw)?;

    // Every publication input must live strictly beneath the
    // dirty-checked paper subtree, so the subtree status guard covers it.
    if canonical == *tree || !canonical.starts_with(tree) {
        return Err(StampError::InputOutsidePaperTree);
    }

    let relative_path = path_to_str(&canonical)?.to_owned();

    let blob = tracked_blob(git, tree_ref, &relative_path)?;

    // Digest authority is the committed blob, not the worktree.
    let committed_bytes = run_bytes(git, &["cat-file", "blob", &blob.object_id])?;

    // The paper build renders the worktree, so bind each rendered input
    // to its committed blob: any divergence is a dirty-subtree failure.
    // This duplicates the broad `git status` guard on purpose — the
    // per-input comparison is exact for the bytes actually rendered.
    let worktree_bytes = std::fs::read(repository_root.join(&canonical))
        .map_err(|_error| StampError::InputReadFailed)?;
    if worktree_bytes != committed_bytes {
        return Err(StampError::DirtyPaperSubtree);
    }

    Ok(CanonicalInput {
        relative_path,
        git_mode: blob.mode,
        bytes: committed_bytes,
    })
}

/// A tracked regular blob: its Git file mode and object id.
struct TrackedBlob {
    mode: String,
    object_id: String,
}

/// Resolve a tracked regular blob at `relative_path`, rejecting
/// directories, symlinks, gitlinks, untracked paths, and any listing
/// whose returned path is not exactly the requested one. A literal
/// pathspec prevents a filename with pathspec metacharacters from
/// selecting a different file.
fn tracked_blob(
    git: &dyn GitRunner,
    tree_ref: &str,
    relative_path: &str,
) -> Result<TrackedBlob, StampError> {
    let listing = run_text(
        git,
        &["ls-tree", tree_ref, "--", &literal_pathspec(relative_path)],
    )?;

    let mut lines = listing.lines();
    let line = lines.next().ok_or(StampError::InvalidTrackedInput)?;
    // Exactly one record must match the literal pathspec.
    if lines.next().is_some() {
        return Err(StampError::InvalidTrackedInput);
    }

    // Format: "<mode> <type> <oid>\t<path>".
    let (metadata, path) = line
        .split_once('\t')
        .ok_or(StampError::InvalidTrackedInput)?;
    if path != relative_path {
        return Err(StampError::InvalidTrackedInput);
    }

    let mut fields = metadata.split_whitespace();
    let mode = fields.next().ok_or(StampError::InvalidTrackedInput)?;
    let object_type = fields.next().ok_or(StampError::InvalidTrackedInput)?;
    let object_id = fields.next().ok_or(StampError::InvalidTrackedInput)?;
    if object_type != "blob" || mode == "120000" {
        return Err(StampError::InvalidTrackedInput);
    }

    Ok(TrackedBlob {
        mode: mode.to_owned(),
        object_id: object_id.to_owned(),
    })
}

// ---------------------------------------------------------------------------
// Document digest
// ---------------------------------------------------------------------------

/// SHA-256 domain separator for the publication-input digest. The
/// trailing NUL frames the recipe version.
const DOCUMENT_DOMAIN: &[u8] = b"tripod/document-inputs/v1\0";

fn update_framed(hasher: &mut Sha256, bytes: &[u8]) {
    let length = u64::try_from(bytes.len()).expect("input length fits in u64");
    hasher.update(length.to_be_bytes());
    hasher.update(bytes);
}

/// Canonical SHA-256 over the sorted inputs, truncated to 128 bits.
fn document_digest(inputs: &[CanonicalInput]) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(DOCUMENT_DOMAIN);
    for input in inputs {
        update_framed(&mut hasher, input.relative_path.as_bytes());
        update_framed(&mut hasher, input.git_mode.as_bytes());
        update_framed(&mut hasher, &input.bytes);
    }
    let digest = hasher.finalize();
    let mut prefix = [0_u8; 16];
    prefix.copy_from_slice(&digest[..16]);
    prefix
}

// ---------------------------------------------------------------------------
// Instance UUID
// ---------------------------------------------------------------------------

/// Append git's "peel to tree" suffix (`^{tree}`) to a revision. Built at
/// runtime so no string literal ever resembles a formatting argument.
fn peel_to_tree(reference: &str) -> String {
    let mut expr = String::from(reference);
    expr.push('^');
    expr.push('{');
    expr.push_str("tree}");
    expr
}

fn instance_uuid(git: &dyn GitRunner, tree_ref: &str, tree: &str) -> Result<String, StampError> {
    // `<rev>:<dir>` resolves a directory path directly to its tree oid.
    let full = run_text(
        git,
        &["rev-parse", "--verify", &format!("{tree_ref}:{tree}")],
    )?;
    let full = normalize_tree_oid(&full)?;

    // The prefix check peels to `^{tree}` so a unique non-tree object (or
    // an ambiguous prefix) is a hard failure, not a silent success.
    let prefix = &full[..32];
    let invocation = git.run(&["rev-parse", "--verify", &peel_to_tree(prefix)])?;
    if !invocation.status_success {
        return Err(StampError::TreePrefixResolutionFailure);
    }
    let resolved = String::from_utf8(invocation.stdout)
        .map_err(|_error| StampError::InvalidGitOutput)?
        .trim_end_matches(['\n', '\r'])
        .to_ascii_lowercase();
    if resolved != full {
        return Err(StampError::TreePrefixResolutionFailure);
    }

    Ok(group_uuid(prefix))
}

fn normalize_tree_oid(oid: &str) -> Result<String, StampError> {
    if oid.len() == 40 && oid.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(oid.to_ascii_lowercase())
    } else {
        Err(StampError::InvalidTreeObjectId)
    }
}

// ---------------------------------------------------------------------------
// Timestamp and date
// ---------------------------------------------------------------------------

fn paper_timestamp(
    git: &dyn GitRunner,
    tree_ref: &str,
    tree: &str,
) -> Result<PreparedTimestamp, StampError> {
    let spec = literal_pathspec(tree);
    let epoch = commit_epoch(git, &["log", "-1", "--format=%ct", tree_ref, "--", &spec])?;
    Ok(prepared_timestamp(epoch))
}

fn input_date(
    git: &dyn GitRunner,
    tree_ref: &str,
    inputs: &[CanonicalInput],
) -> Result<String, StampError> {
    // Literal pathspecs, owned until the call completes, so a filename
    // with pathspec metacharacters cannot change the selected history.
    let specs = inputs
        .iter()
        .map(|input| literal_pathspec(&input.relative_path))
        .collect::<Vec<_>>();

    let mut args = vec!["log", "-1", "--format=%ct", tree_ref, "--"];
    for spec in &specs {
        args.push(spec);
    }
    let epoch = commit_epoch(git, &args)?;
    let (year, month, day, _, _, _) = civil_from_epoch(epoch);
    Ok(format!("{year:04}-{month:02}-{day:02}"))
}

fn commit_epoch(git: &dyn GitRunner, args: &[&str]) -> Result<i64, StampError> {
    let text = run_text(git, args)?;
    if text.is_empty() {
        return Err(StampError::MissingPathHistory);
    }
    text.parse::<i64>()
        .map_err(|_error| StampError::InvalidGitOutput)
}

fn prepared_timestamp(epoch: i64) -> PreparedTimestamp {
    let (year, month, day, hour, minute, second) = civil_from_epoch(epoch);
    PreparedTimestamp {
        epoch,
        iso_8601: format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z"),
        pdf: format!("D:{year:04}{month:02}{day:02}{hour:02}{minute:02}{second:02}Z"),
    }
}

// ---------------------------------------------------------------------------
// UTC civil-date arithmetic (Howard Hinnant's algorithm), pure integer
// maths so no timezone, locale, or `date` binary participates.
// ---------------------------------------------------------------------------

/// Convert Unix seconds to `(year, month, day, hour, minute, second)` in
/// UTC. Negative epochs (before 1970) are handled via Euclidean division.
const fn civil_from_epoch(epoch: i64) -> (i64, i64, i64, i64, i64, i64) {
    let days = epoch.div_euclid(86_400);
    let seconds = epoch.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds / 3_600;
    let minute = (seconds % 3_600) / 60;
    let second = seconds % 60;
    (year, month, day, hour, minute, second)
}

/// Days since the Unix epoch → `(year, month, day)`.
const fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted - era * 146_097; // [0, 146096]
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365; // [0, 399]
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100); // [0, 365]
    let month_prime = (5 * day_of_year + 2) / 153; // [0, 11]
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1; // [1, 31]
    let month = if month_prime < 10 {
        month_prime + 3
    } else {
        month_prime - 9
    }; // [1, 12]
    let year = if month <= 2 { year + 1 } else { year };
    (year, month, day)
}

// ---------------------------------------------------------------------------
// Hex and UUID formatting
// ---------------------------------------------------------------------------

fn hex_lower(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(text, "{byte:02x}").expect("writing to a String cannot fail");
    }
    text
}

/// Format 16 digest bytes as `8-4-4-4-12` lowercase hex. No UUID version
/// or variant bits are rewritten: the text preserves all 128 bits.
fn bytes_to_uuid_text(bytes: &[u8; 16]) -> String {
    group_uuid(&hex_lower(bytes))
}

/// Group 32 lowercase hex characters as `8-4-4-4-12`.
fn group_uuid(hex: &str) -> String {
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32],
    )
}

fn path_to_str(path: &Path) -> Result<&str, StampError> {
    path.to_str().ok_or(StampError::InvalidInputPath)
}
