//! Hermetic unit tests for the stamp derivation. Git behaviour is
//! supplied by a fake runner with canned outputs; file-reading paths use
//! a synthetic temporary repository. No test resolves the live checkout.
//!
//! | Test | Property |
//! |---|---|
//! | `document_digest_is_order_independent_after_canonicalisation` | argument order does not change the document UUID |
//! | `document_digest_rejects_a_duplicate_input` | a repeated input is a hard error |
//! | `canonical_inputs_reject_paths_that_escape_the_repository` | `..` / absolute paths rejected |
//! | `document_uuid_changes_with_bytes_path_and_mode` | digest is sensitive to each framed field |
//! | `document_uuid_changes_when_an_input_is_added` | set membership matters |
//! | `instance_uuid_is_the_exact_tree_prefix` | fixed vector, no version/variant rewrite |
//! | `instance_uuid_normalises_uppercase_and_rejects_malformed` | oid hygiene |
//! | `instance_uuid_fails_hard_on_prefix_resolution` | ambiguity/mismatch aborts |
//! | `tracked_blob_mode_rejects_non_blobs` | symlinks / trees / untracked rejected |
//! | `utc_conversion_covers_the_edge_cases` | epoch 0, midnight, year/leap/century boundaries |
//! | `prepared_timestamp_has_three_consistent_representations` | one instant, three forms |
//! | `derive_rejects_a_dirty_paper_subtree` | dirty status is a hard error |
//! | `derive_rejects_a_non_sha1_object_format` | object-format guard |
//! | `derive_produces_all_four_values` | happy-path end to end |

use std::collections::HashMap;
use std::path::PathBuf;

use super::*;

// ---------------------------------------------------------------------------
// Fake Git runner
// ---------------------------------------------------------------------------

/// Matches an exact argument vector to a canned `(success, stdout)`.
#[derive(Default)]
struct FakeGit {
    responses: HashMap<Vec<String>, (bool, Vec<u8>)>,
}

impl FakeGit {
    fn with(mut self, args: &[&str], success: bool, stdout: &str) -> Self {
        let key = args.iter().map(|arg| (*arg).to_owned()).collect();
        self.responses
            .insert(key, (success, stdout.as_bytes().to_vec()));
        self
    }

    /// Register a binary-safe response (for `cat-file blob`).
    fn with_bytes(mut self, args: &[&str], success: bool, stdout: &[u8]) -> Self {
        let key = args.iter().map(|arg| (*arg).to_owned()).collect();
        self.responses.insert(key, (success, stdout.to_vec()));
        self
    }
}

/// The Git literal pathspec argument for a path, mirroring the library's
/// runtime construction.
fn literal(path: &str) -> String {
    let mut spec = String::from(":(literal)");
    spec.push_str(path);
    spec
}

impl GitRunner for FakeGit {
    fn run(&self, args: &[&str]) -> Result<GitInvocation, StampError> {
        let key: Vec<String> = args.iter().map(|arg| (*arg).to_owned()).collect();
        let (success, stdout) = self
            .responses
            .get(&key)
            .unwrap_or_else(|| panic!("unexpected git args: {args:?}"));
        Ok(GitInvocation {
            status_success: *success,
            stdout: stdout.clone(),
        })
    }
}

/// The `rev-parse` argument git receives for a peeled tree reference,
/// assembled so no literal resembles a formatting argument.
fn peeled(reference: &str) -> String {
    let mut expr = String::from(reference);
    expr.push('^');
    expr.push('{');
    expr.push_str("tree}");
    expr
}

/// The `rev-parse` argument git receives for a peeled commit reference.
fn peeled_commit(reference: &str) -> String {
    let mut expr = String::from(reference);
    expr.push('^');
    expr.push('{');
    expr.push_str("commit}");
    expr
}

/// A fixed commit oid the scenario's HEAD guard resolves to.
const HEAD_COMMIT: &str = "cccccccccccccccccccccccccccccccccccccccc";

fn input(path: &str, mode: &str, bytes: &[u8]) -> CanonicalInput {
    CanonicalInput {
        relative_path: path.to_owned(),
        git_mode: mode.to_owned(),
        bytes: bytes.to_vec(),
    }
}

// ---------------------------------------------------------------------------
// Document digest
// ---------------------------------------------------------------------------

#[test]
fn document_digest_is_order_independent_after_canonicalisation() {
    // `canonical_inputs` sorts by path, so two argument orders of the
    // same set yield the same digest. Here we mimic that by sorting.
    let mut forward = vec![
        input("papers/attestation/main.tex", "100644", b"main"),
        input("papers/attestation/references.bib", "100644", b"bib"),
    ];
    let mut reverse = vec![
        input("papers/attestation/references.bib", "100644", b"bib"),
        input("papers/attestation/main.tex", "100644", b"main"),
    ];
    forward.sort_by(|l, r| l.relative_path.cmp(&r.relative_path));
    reverse.sort_by(|l, r| l.relative_path.cmp(&r.relative_path));

    assert_eq!(document_digest(&forward), document_digest(&reverse));
}

#[test]
fn document_uuid_changes_with_bytes_path_and_mode() {
    let base = [input("a.tex", "100644", b"content")];
    let baseline = document_digest(&base);

    let changed_bytes = [input("a.tex", "100644", b"content!")];
    let changed_path = [input("b.tex", "100644", b"content")];
    let changed_mode = [input("a.tex", "100755", b"content")];

    assert_ne!(baseline, document_digest(&changed_bytes));
    assert_ne!(baseline, document_digest(&changed_path));
    assert_ne!(baseline, document_digest(&changed_mode));
}

#[test]
fn document_uuid_changes_when_an_input_is_added() {
    let one = [input("a.tex", "100644", b"a")];
    let two = [
        input("a.tex", "100644", b"a"),
        input("b.tex", "100644", b"b"),
    ];
    assert_ne!(document_digest(&one), document_digest(&two));
}

#[test]
fn framed_length_prevents_boundary_ambiguity() {
    // Without length framing, ("ab","")/("a","b") would collide. The
    // u64-BE prefixes keep the two concatenations distinct.
    let joined = [input("ab", "100644", b"")];
    let split = [input("a", "100644", b"b")];
    assert_ne!(document_digest(&joined), document_digest(&split));
}

// ---------------------------------------------------------------------------
// Instance UUID
// ---------------------------------------------------------------------------

const TREE_OID: &str = "0123456789abcdef0123456789abcdef01234567";
const TREE_PREFIX: &str = "0123456789abcdef0123456789abcdef";
const EXPECTED_INSTANCE: &str = "01234567-89ab-cdef-0123-456789abcdef";

fn instance_fake(full: &str, prefix_success: bool, prefix_resolves_to: &str) -> FakeGit {
    // The derivation lowercases the full oid before slicing the prefix,
    // mirroring git's own output normalisation, so the prefix query git
    // receives is always lowercase.
    let lower = full.to_ascii_lowercase();
    let prefix = &lower[..32];
    FakeGit::default()
        .with(
            &["rev-parse", "--verify", "HEAD:papers/attestation"],
            true,
            &format!("{full}\n"),
        )
        .with(
            &["rev-parse", "--verify", &peeled(prefix)],
            prefix_success,
            &format!("{prefix_resolves_to}\n"),
        )
}

#[test]
fn instance_uuid_is_the_exact_tree_prefix() {
    let git = instance_fake(TREE_OID, true, TREE_OID);
    let uuid = instance_uuid(&git, "HEAD", "papers/attestation").expect("derives");

    assert_eq!(uuid, EXPECTED_INSTANCE);
    // Removing the hyphens recovers the first 128 bits of the object ID.
    assert_eq!(uuid.replace('-', ""), TREE_PREFIX);
}

#[test]
fn instance_uuid_normalises_uppercase() {
    let upper = TREE_OID.to_ascii_uppercase();
    // The prefix lookup uses the lowercased prefix and resolves to the
    // lowercased full oid, matching git's own output normalisation.
    let git = instance_fake(&upper, true, TREE_OID);
    let uuid = instance_uuid(&git, "HEAD", "papers/attestation").expect("derives");
    assert_eq!(uuid, EXPECTED_INSTANCE);
}

#[test]
fn instance_uuid_rejects_a_malformed_object_id() {
    let git = FakeGit::default().with(
        &["rev-parse", "--verify", "HEAD:papers/attestation"],
        true,
        "not-a-valid-oid\n",
    );
    assert!(matches!(
        instance_uuid(&git, "HEAD", "papers/attestation"),
        Err(StampError::InvalidTreeObjectId)
    ));
}

#[test]
fn instance_uuid_fails_hard_on_prefix_resolution() {
    let ambiguous = instance_fake(TREE_OID, false, "");
    assert!(matches!(
        instance_uuid(&ambiguous, "HEAD", "papers/attestation"),
        Err(StampError::TreePrefixResolutionFailure)
    ));

    let mismatched = instance_fake(TREE_OID, true, "ffffffffffffffffffffffffffffffffffffffff");
    assert!(matches!(
        instance_uuid(&mismatched, "HEAD", "papers/attestation"),
        Err(StampError::TreePrefixResolutionFailure)
    ));
}

// ---------------------------------------------------------------------------
// Tracked-blob mode
// ---------------------------------------------------------------------------

#[test]
fn tracked_blob_accepts_a_regular_blob() {
    let git = FakeGit::default().with(
        &[
            "ls-tree",
            "HEAD",
            "--",
            &literal("papers/attestation/main.tex"),
        ],
        true,
        "100644 blob 89abcd\tpapers/attestation/main.tex\n",
    );
    let blob = tracked_blob(&git, "HEAD", "papers/attestation/main.tex").expect("blob");
    assert_eq!(blob.mode, "100644");
    assert_eq!(blob.object_id, "89abcd");
}

#[test]
fn tracked_blob_rejects_non_blobs() {
    let symlink = FakeGit::default().with(
        &["ls-tree", "HEAD", "--", &literal("link")],
        true,
        "120000 blob 89abcd\tlink\n",
    );
    assert!(matches!(
        tracked_blob(&symlink, "HEAD", "link"),
        Err(StampError::InvalidTrackedInput)
    ));

    let tree = FakeGit::default().with(
        &["ls-tree", "HEAD", "--", &literal("dir")],
        true,
        "040000 tree 89abcd\tdir\n",
    );
    assert!(matches!(
        tracked_blob(&tree, "HEAD", "dir"),
        Err(StampError::InvalidTrackedInput)
    ));

    let untracked =
        FakeGit::default().with(&["ls-tree", "HEAD", "--", &literal("ghost")], true, "");
    assert!(matches!(
        tracked_blob(&untracked, "HEAD", "ghost"),
        Err(StampError::InvalidTrackedInput)
    ));
}

#[test]
fn tracked_blob_rejects_a_path_that_does_not_match_exactly() {
    // A listing whose returned path differs from the requested one (a
    // pathspec that matched something else) is a hard failure.
    let git = FakeGit::default().with(
        &["ls-tree", "HEAD", "--", &literal("wanted.tex")],
        true,
        "100644 blob 89abcd\tother.tex\n",
    );
    assert!(matches!(
        tracked_blob(&git, "HEAD", "wanted.tex"),
        Err(StampError::InvalidTrackedInput)
    ));
}

// ---------------------------------------------------------------------------
// UTC conversion
// ---------------------------------------------------------------------------

#[test]
fn utc_conversion_covers_the_edge_cases() {
    assert_eq!(civil_from_epoch(0), (1970, 1, 1, 0, 0, 0));
    assert_eq!(civil_from_epoch(86_399), (1970, 1, 1, 23, 59, 59));
    assert_eq!(civil_from_epoch(86_400), (1970, 1, 2, 0, 0, 0));
    assert_eq!(civil_from_epoch(-1), (1969, 12, 31, 23, 59, 59));
    // Year boundary.
    assert_eq!(civil_from_epoch(1_704_067_200), (2024, 1, 1, 0, 0, 0));
    // Leap day in a 4-year leap year.
    assert_eq!(civil_from_epoch(1_709_164_800), (2024, 2, 29, 0, 0, 0));
    // 400-year leap boundary: 2000-02-29 exists.
    assert_eq!(civil_from_epoch(951_782_400), (2000, 2, 29, 0, 0, 0));
    // Project range vector from the design guide.
    assert_eq!(civil_from_epoch(1_784_118_896), (2026, 7, 15, 12, 34, 56));
}

#[test]
fn prepared_timestamp_has_three_consistent_representations() {
    let timestamp = prepared_timestamp(1_784_118_896);
    assert_eq!(timestamp.epoch, 1_784_118_896);
    assert_eq!(timestamp.iso_8601, "2026-07-15T12:34:56Z");
    assert_eq!(timestamp.pdf, "D:20260715123456Z");
}

// ---------------------------------------------------------------------------
// UUID formatting
// ---------------------------------------------------------------------------

#[test]
fn document_uuid_preserves_all_digest_bits() {
    let bytes: [u8; 16] = [
        0x6f, 0x51, 0x50, 0x8e, 0x4b, 0xf8, 0xe7, 0xa7, 0x3c, 0xdb, 0xf2, 0x02, 0x1c, 0x8a, 0x24,
        0xb2,
    ];
    let uuid = bytes_to_uuid_text(&bytes);
    assert_eq!(uuid, "6f51508e-4bf8-e7a7-3cdb-f2021c8a24b2");
    assert_eq!(uuid.replace('-', ""), hex_lower(&bytes));
}

// ---------------------------------------------------------------------------
// Full derivation via a synthetic repository
// ---------------------------------------------------------------------------

struct Scenario {
    _dir: tempfile::TempDir,
    request: StampRequest,
}

/// Build a temp directory holding the given `(path, bytes)` inputs, plus
/// a fake-git-backed request. `object_format`, `dirty`, and the derived
/// oids/epochs are configurable by the caller-supplied builder.
fn scenario(
    inputs: &[(&str, &[u8])],
    object_format: &str,
    dirty: bool,
    tree_oid: &str,
    timestamp_epoch: &str,
    date_epoch: &str,
) -> (Scenario, FakeGit) {
    let dir = tempfile::tempdir().expect("temp repo");
    let root = dir.path();
    let canonical_root = std::fs::canonicalize(root).expect("canonical root");

    let mut request_inputs = Vec::new();
    let mut fake = FakeGit::default()
        .with(
            &["rev-parse", "--show-toplevel"],
            true,
            &format!("{}\n", canonical_root.display()),
        )
        .with(
            &["rev-parse", "--show-object-format"],
            true,
            &format!("{object_format}\n"),
        )
        .with(
            &["rev-parse", "--verify", &peeled_commit("HEAD")],
            true,
            &format!("{HEAD_COMMIT}\n"),
        )
        .with(
            &[
                "status",
                "--porcelain=v1",
                "-z",
                "--untracked-files=all",
                "--",
                &literal("papers/attestation"),
            ],
            true,
            if dirty {
                " M papers/attestation/main.tex\0"
            } else {
                ""
            },
        );

    for (index, (path, bytes)) in inputs.iter().enumerate() {
        let absolute = root.join(path);
        std::fs::create_dir_all(absolute.parent().expect("parent")).expect("mkdir");
        std::fs::write(&absolute, bytes).expect("write input");
        request_inputs.push(PathBuf::from(*path));

        // Each input gets a distinct object id, and `cat-file blob`
        // returns exactly the committed bytes (equal to the worktree
        // bytes written above, so a clean scenario stays clean).
        let object_id = format!("{:040x}", index + 1);
        fake = fake
            .with(
                &["ls-tree", "HEAD", "--", &literal(path)],
                true,
                &format!("100644 blob {object_id}\t{path}\n"),
            )
            .with_bytes(&["cat-file", "blob", &object_id], true, bytes);
    }

    let prefix = &tree_oid[..32];
    fake = fake
        .with(
            &["rev-parse", "--verify", "HEAD:papers/attestation"],
            true,
            &format!("{tree_oid}\n"),
        )
        .with(
            &["rev-parse", "--verify", &peeled(prefix)],
            true,
            &format!("{tree_oid}\n"),
        )
        .with(
            &[
                "log",
                "-1",
                "--format=%ct",
                "HEAD",
                "--",
                &literal("papers/attestation"),
            ],
            true,
            &format!("{timestamp_epoch}\n"),
        );

    // The date query passes the sorted input paths, as literal
    // pathspecs, after `--`.
    let mut sorted: Vec<&str> = inputs.iter().map(|(path, _)| *path).collect();
    sorted.sort_unstable();
    let mut date_args = vec![
        "log".to_owned(),
        "-1".to_owned(),
        "--format=%ct".to_owned(),
        "HEAD".to_owned(),
        "--".to_owned(),
    ];
    for path in sorted {
        date_args.push(literal(path));
    }
    let date_refs: Vec<&str> = date_args.iter().map(String::as_str).collect();
    fake = fake.with(&date_refs, true, &format!("{date_epoch}\n"));

    let request = StampRequest {
        git: PathBuf::from("git"),
        repository_root: root.to_path_buf(),
        tree_ref: "HEAD".to_owned(),
        tree: PathBuf::from("papers/attestation"),
        inputs: request_inputs,
    };
    (Scenario { _dir: dir, request }, fake)
}

#[test]
fn derive_produces_all_four_values() {
    let (scenario, git) = scenario(
        &[
            ("papers/attestation/main.tex", b"\\documentclass{article}"),
            ("papers/attestation/references.bib", b"@misc{x}"),
        ],
        "sha1",
        false,
        TREE_OID,
        "1784118896",
        "1784000000",
    );
    let values = derive(&git, &scenario.request).expect("derives");

    assert_eq!(values.instance_uuid, EXPECTED_INSTANCE);
    assert_eq!(values.timestamp.pdf, "D:20260715123456Z");
    assert_eq!(values.date, "2026-07-14");
    assert_eq!(values.document_uuid.len(), 36);
    assert_eq!(values.document_uuid.matches('-').count(), 4);
}

#[test]
fn derive_is_input_order_independent() {
    let forward = scenario(
        &[
            ("papers/attestation/main.tex", b"main"),
            ("papers/attestation/references.bib", b"bib"),
        ],
        "sha1",
        false,
        TREE_OID,
        "1784118896",
        "1784000000",
    );
    let reverse = scenario(
        &[
            ("papers/attestation/references.bib", b"bib"),
            ("papers/attestation/main.tex", b"main"),
        ],
        "sha1",
        false,
        TREE_OID,
        "1784118896",
        "1784000000",
    );

    let a = derive(&forward.1, &forward.0.request).expect("derives");
    let b = derive(&reverse.1, &reverse.0.request).expect("derives");
    assert_eq!(a.document_uuid, b.document_uuid);
}

#[test]
fn derive_rejects_a_duplicate_input() {
    let (mut scenario, git) = scenario(
        &[("papers/attestation/main.tex", b"main")],
        "sha1",
        false,
        TREE_OID,
        "1784118896",
        "1784000000",
    );
    scenario
        .request
        .inputs
        .push(PathBuf::from("papers/attestation/main.tex"));
    assert!(matches!(
        derive(&git, &scenario.request),
        Err(StampError::DuplicateInput)
    ));
}

#[test]
fn derive_rejects_a_dirty_paper_subtree() {
    let (scenario, git) = scenario(
        &[("papers/attestation/main.tex", b"main")],
        "sha1",
        true,
        TREE_OID,
        "1784118896",
        "1784000000",
    );
    assert!(matches!(
        derive(&git, &scenario.request),
        Err(StampError::DirtyPaperSubtree)
    ));
}

#[test]
fn derive_rejects_a_non_sha1_object_format() {
    let (scenario, git) = scenario(
        &[("papers/attestation/main.tex", b"main")],
        "sha256",
        false,
        TREE_OID,
        "1784118896",
        "1784000000",
    );
    assert!(matches!(
        derive(&git, &scenario.request),
        Err(StampError::UnsupportedGitObjectFormat(_))
    ));
}

// ---------------------------------------------------------------------------
// HEAD-only revision selection
// ---------------------------------------------------------------------------

#[test]
fn head_is_accepted_as_the_selected_revision() {
    let git = FakeGit::default().with(
        &["rev-parse", "--verify", &peeled_commit("HEAD")],
        true,
        &format!("{HEAD_COMMIT}\n"),
    );
    verify_selected_revision_is_head(&git, "HEAD").expect("HEAD resolves to HEAD");
}

#[test]
fn a_branch_alias_resolving_to_head_is_accepted() {
    // A tag or branch that points at exactly HEAD is accepted: the rule
    // is commit-object equality, not literal string equality.
    let git = FakeGit::default()
        .with(
            &["rev-parse", "--verify", &peeled_commit("release")],
            true,
            &format!("{HEAD_COMMIT}\n"),
        )
        .with(
            &["rev-parse", "--verify", &peeled_commit("HEAD")],
            true,
            &format!("{HEAD_COMMIT}\n"),
        );
    verify_selected_revision_is_head(&git, "release").expect("alias resolves to HEAD");
}

#[test]
fn an_older_commit_is_rejected() {
    let git = FakeGit::default()
        .with(
            &["rev-parse", "--verify", &peeled_commit("older")],
            true,
            "0000000000000000000000000000000000000000\n",
        )
        .with(
            &["rev-parse", "--verify", &peeled_commit("HEAD")],
            true,
            &format!("{HEAD_COMMIT}\n"),
        );
    assert!(matches!(
        verify_selected_revision_is_head(&git, "older"),
        Err(StampError::SelectedRevisionIsNotHead)
    ));
}

#[test]
fn derive_rejects_a_non_head_revision_before_digesting() {
    // The HEAD guard runs before the dirty-subtree check and before any
    // input is read: this fake stubs only the checks up to the guard, so
    // if derivation proceeded it would hit an unstubbed arg and panic.
    let dir = tempfile::tempdir().expect("temp repo");
    let root = dir.path();
    let canonical_root = std::fs::canonicalize(root).expect("canonical root");

    let git = FakeGit::default()
        .with(
            &["rev-parse", "--show-toplevel"],
            true,
            &format!("{}\n", canonical_root.display()),
        )
        .with(&["rev-parse", "--show-object-format"], true, "sha1\n")
        .with(
            &["rev-parse", "--verify", &peeled_commit("older")],
            true,
            "0000000000000000000000000000000000000000\n",
        )
        .with(
            &["rev-parse", "--verify", &peeled_commit("HEAD")],
            true,
            &format!("{HEAD_COMMIT}\n"),
        );

    let request = StampRequest {
        git: PathBuf::from("git"),
        repository_root: root.to_path_buf(),
        tree_ref: "older".to_owned(),
        tree: PathBuf::from("papers/attestation"),
        inputs: vec![PathBuf::from("papers/attestation/main.tex")],
    };

    assert!(matches!(
        derive(&git, &request),
        Err(StampError::SelectedRevisionIsNotHead)
    ));
}

#[test]
fn canonical_inputs_reject_paths_that_escape_the_repository() {
    let (mut scenario, git) = scenario(
        &[("papers/attestation/main.tex", b"main")],
        "sha1",
        false,
        TREE_OID,
        "1784118896",
        "1784000000",
    );
    scenario
        .request
        .inputs
        .push(PathBuf::from("../outside.tex"));
    assert!(matches!(
        derive(&git, &scenario.request),
        Err(StampError::InvalidInputPath)
    ));
}

#[test]
fn a_leading_curdir_alias_is_an_invalid_path() {
    // A leading `./` is a genuine `CurDir` component, rejected outright
    // before any git lookup.
    let (mut scenario, git) = scenario(
        &[("papers/attestation/main.tex", b"main")],
        "sha1",
        false,
        TREE_OID,
        "1784118896",
        "1784000000",
    );
    scenario
        .request
        .inputs
        .push(PathBuf::from("./papers/attestation/main.tex"));
    assert!(matches!(
        derive(&git, &scenario.request),
        Err(StampError::InvalidInputPath)
    ));
}

#[test]
fn a_normalising_alias_cannot_bypass_duplicate_detection() {
    // A mid-path `/./` normalizes to the canonical path, so it collapses
    // onto the genuine input and is caught as a duplicate rather than
    // silently double-counted in the digest.
    let (mut scenario, git) = scenario(
        &[("papers/attestation/main.tex", b"main")],
        "sha1",
        false,
        TREE_OID,
        "1784118896",
        "1784000000",
    );
    scenario
        .request
        .inputs
        .push(PathBuf::from("papers/attestation/./main.tex"));
    assert!(matches!(
        derive(&git, &scenario.request),
        Err(StampError::DuplicateInput)
    ));
}

#[test]
fn input_outside_the_paper_subtree_is_rejected() {
    let (mut scenario, git) = scenario(
        &[("papers/attestation/main.tex", b"main")],
        "sha1",
        false,
        TREE_OID,
        "1784118896",
        "1784000000",
    );
    scenario
        .request
        .inputs
        .push(PathBuf::from("docs/attestation/human.md"));
    assert!(matches!(
        derive(&git, &scenario.request),
        Err(StampError::InputOutsidePaperTree)
    ));
}

#[test]
fn worktree_bytes_disagreeing_with_the_committed_blob_are_dirty() {
    let (scenario, git) = scenario(
        &[("papers/attestation/main.tex", b"worktree")],
        "sha1",
        false,
        TREE_OID,
        "1784118896",
        "1784000000",
    );

    // Override the committed blob for the single input (object id of the
    // first input) so it disagrees with the worktree bytes written above.
    let object_id = format!("{:040x}", 1);
    let git = git.with_bytes(&["cat-file", "blob", &object_id], true, b"committed");

    assert!(matches!(
        derive(&git, &scenario.request),
        Err(StampError::DirtyPaperSubtree)
    ));
}

#[test]
fn a_pathspec_metacharacter_filename_is_looked_up_literally() {
    // A filename containing a pathspec metacharacter must resolve
    // through a literal pathspec; the fake only answers the literal form.
    let (scenario, git) = scenario(
        &[("papers/attestation/fig[1].tex", b"figure")],
        "sha1",
        false,
        TREE_OID,
        "1784118896",
        "1784000000",
    );
    derive(&git, &scenario.request).expect("literal pathspec resolves the exact file");
}

// ---------------------------------------------------------------------------
// Template rendering and idempotent output
// ---------------------------------------------------------------------------

fn render_values() -> AttestationStampValues {
    AttestationStampValues {
        date: "2030-01-02".to_owned(),
        timestamp: PreparedTimestamp {
            epoch: 1_893_542_400,
            iso_8601: "2030-01-02T00:00:00Z".to_owned(),
            pdf: "D:20300102000000Z".to_owned(),
        },
        document_uuid: "11111111-2222-3333-4444-555555555555".to_owned(),
        instance_uuid: "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee".to_owned(),
    }
}

#[test]
fn render_stamps_fills_all_four_placeholders() {
    let template = concat!(
        r"\newcommand{\AttestationDate}{@ATTESTATION_DATE@}",
        "\n",
        r"\newcommand{\AttestationTimestamp}{@ATTESTATION_TIMESTAMP@}",
        "\n",
        r"\newcommand{\AttestationDocumentUUID}{@ATTESTATION_DOCUMENT_UUID@}",
        "\n",
        r"\newcommand{\AttestationInstanceUUID}{@ATTESTATION_INSTANCE_UUID@}",
        "\n",
    );
    let rendered = render_stamps(template, &render_values()).expect("renders");

    assert!(rendered.contains("{2030-01-02}"));
    assert!(rendered.contains("{D:20300102000000Z}"));
    assert!(rendered.contains("{11111111-2222-3333-4444-555555555555}"));
    assert!(rendered.contains("{aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee}"));
    assert!(!rendered.contains("@ATTESTATION_"));
}

#[test]
fn render_stamps_rejects_an_unresolved_placeholder() {
    let template = "@ATTESTATION_DATE@ and @ATTESTATION_UNKNOWN@";
    assert!(matches!(
        render_stamps(template, &render_values()),
        Err(StampError::UnresolvedPlaceholder)
    ));
}

/// Single-path convenience over the stage/publish primitives: the same
/// compare-if-changed, atomic-rename semantics the two-output render path
/// composes, exercised in isolation.
fn write_if_changed(path: &std::path::Path, bytes: &[u8]) -> Result<(), StampError> {
    publish(stage_if_changed(path, bytes)?)
}

#[test]
fn write_if_changed_creates_an_absent_file() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("out");
    write_if_changed(&path, b"content").expect("writes");
    assert_eq!(std::fs::read(&path).expect("read"), b"content");
}

#[test]
fn write_if_changed_skips_identical_content() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("out");
    write_if_changed(&path, b"same").expect("writes");

    // Pin an old mtime; a rewrite would move it to ~now, a skip preserves it.
    let old = std::time::SystemTime::UNIX_EPOCH;
    std::fs::File::options()
        .write(true)
        .open(&path)
        .expect("open")
        .set_times(std::fs::FileTimes::new().set_modified(old))
        .expect("set mtime");

    write_if_changed(&path, b"same").expect("writes");

    let mtime = std::fs::metadata(&path)
        .expect("metadata")
        .modified()
        .expect("mtime");
    assert_eq!(mtime, old, "identical content must not be rewritten");
    assert_eq!(std::fs::read(&path).expect("read"), b"same");
}

#[test]
fn write_if_changed_rewrites_changed_content() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("out");
    write_if_changed(&path, b"one").expect("writes");
    write_if_changed(&path, b"two").expect("rewrites");
    assert_eq!(std::fs::read(&path).expect("read"), b"two");
}

// F1-034: render stages both outputs before publishing either, so a
// staging failure on one leaves neither final output changed. Two
// independent paths cannot be renamed as one transaction, so a failure
// during the second final rename can still leave a partial pair — the
// build fails and the next invocation repairs it; these tests pin the
// guarantees that do hold.

const RENDER_TEMPLATE: &str = concat!(
    r"\newcommand{\AttestationDate}{@ATTESTATION_DATE@}",
    "\n",
    r"\newcommand{\AttestationTimestamp}{@ATTESTATION_TIMESTAMP@}",
    "\n",
    r"\newcommand{\AttestationDocumentUUID}{@ATTESTATION_DOCUMENT_UUID@}",
    "\n",
    r"\newcommand{\AttestationInstanceUUID}{@ATTESTATION_INSTANCE_UUID@}",
    "\n",
);

fn expected_epoch_bytes(values: &AttestationStampValues) -> String {
    format!("{}\n", values.timestamp.epoch)
}

#[test]
fn render_leaves_stamps_untouched_when_epoch_staging_fails() {
    let dir = tempfile::tempdir().expect("temp dir");
    let stamps = dir.path().join("stamps.tex");
    // The epoch parent directory does not exist, so staging the epoch
    // output fails before anything is published.
    let epoch = dir.path().join("absent").join("source-date-epoch");

    std::fs::write(&stamps, b"OLD STAMPS").expect("seed stamps");

    let result = render_outputs(RENDER_TEMPLATE, &render_values(), &stamps, &epoch);

    assert!(matches!(result, Err(StampError::OutputWriteFailed)));
    assert_eq!(std::fs::read(&stamps).expect("read"), b"OLD STAMPS");
    assert!(
        !epoch.exists(),
        "epoch must not be created on staging failure"
    );
}

#[test]
fn render_leaves_epoch_untouched_when_stamps_staging_fails() {
    let dir = tempfile::tempdir().expect("temp dir");
    // The stamps parent directory does not exist, so staging the stamps
    // output fails before anything is published.
    let stamps = dir.path().join("absent").join("stamps.tex");
    let epoch = dir.path().join("source-date-epoch");

    std::fs::write(&epoch, b"OLD EPOCH").expect("seed epoch");

    let result = render_outputs(RENDER_TEMPLATE, &render_values(), &stamps, &epoch);

    assert!(matches!(result, Err(StampError::OutputWriteFailed)));
    assert_eq!(std::fs::read(&epoch).expect("read"), b"OLD EPOCH");
    assert!(
        !stamps.exists(),
        "stamps must not be created on staging failure"
    );
}

#[test]
fn render_leaves_stamps_untouched_when_epoch_publish_fails() {
    let dir = tempfile::tempdir().expect("temp dir");
    let stamps = dir.path().join("stamps.tex");
    // The epoch destination is an existing directory: staging succeeds
    // but the rename onto it fails. Because the epoch is published first,
    // the stamps output is never published.
    let epoch = dir.path().join("source-date-epoch");
    std::fs::create_dir(&epoch).expect("epoch dir");

    std::fs::write(&stamps, b"OLD STAMPS").expect("seed stamps");

    let result = render_outputs(RENDER_TEMPLATE, &render_values(), &stamps, &epoch);

    assert!(matches!(result, Err(StampError::OutputWriteFailed)));
    assert_eq!(std::fs::read(&stamps).expect("read"), b"OLD STAMPS");
    assert!(epoch.is_dir(), "the epoch directory is not clobbered");
}

#[test]
fn render_repairs_a_partial_prior_state() {
    let dir = tempfile::tempdir().expect("temp dir");
    let stamps = dir.path().join("stamps.tex");
    let epoch = dir.path().join("source-date-epoch");

    let values = render_values();
    let expected_stamps = render_stamps(RENDER_TEMPLATE, &values).expect("renders");

    // A prior run published stamps.tex but not the epoch (a late
    // failure): the pair is incoherent going in.
    std::fs::write(&stamps, &expected_stamps).expect("seed stamps");
    std::fs::write(&epoch, b"STALE\n").expect("seed epoch");

    render_outputs(RENDER_TEMPLATE, &values, &stamps, &epoch).expect("rerun repairs");

    assert_eq!(
        std::fs::read_to_string(&stamps).expect("read"),
        expected_stamps
    );
    assert_eq!(
        std::fs::read_to_string(&epoch).expect("read"),
        expected_epoch_bytes(&values),
    );
}

#[test]
fn render_is_a_no_op_on_an_identical_rerun() {
    let dir = tempfile::tempdir().expect("temp dir");
    let stamps = dir.path().join("stamps.tex");
    let epoch = dir.path().join("source-date-epoch");
    let values = render_values();

    render_outputs(RENDER_TEMPLATE, &values, &stamps, &epoch).expect("first write");

    // Pin an old mtime on both; a rewrite would move it to ~now.
    let old = std::time::SystemTime::UNIX_EPOCH;
    for path in [&stamps, &epoch] {
        std::fs::File::options()
            .write(true)
            .open(path)
            .expect("open")
            .set_times(std::fs::FileTimes::new().set_modified(old))
            .expect("set mtime");
    }

    render_outputs(RENDER_TEMPLATE, &values, &stamps, &epoch).expect("identical rerun");

    for path in [&stamps, &epoch] {
        let mtime = std::fs::metadata(path)
            .expect("metadata")
            .modified()
            .expect("mtime");
        assert_eq!(mtime, old, "identical rerun must not rewrite {path:?}");
    }
}
