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
fn tracked_blob_mode_accepts_a_regular_blob() {
    let git = FakeGit::default().with(
        &["ls-tree", "HEAD", "--", "papers/attestation/main.tex"],
        true,
        "100644 blob 89abcd\tpapers/attestation/main.tex\n",
    );
    assert_eq!(
        tracked_blob_mode(&git, "HEAD", "papers/attestation/main.tex").expect("mode"),
        "100644"
    );
}

#[test]
fn tracked_blob_mode_rejects_non_blobs() {
    let symlink = FakeGit::default().with(
        &["ls-tree", "HEAD", "--", "link"],
        true,
        "120000 blob 89abcd\tlink\n",
    );
    assert!(matches!(
        tracked_blob_mode(&symlink, "HEAD", "link"),
        Err(StampError::InvalidTrackedInput)
    ));

    let tree = FakeGit::default().with(
        &["ls-tree", "HEAD", "--", "dir"],
        true,
        "040000 tree 89abcd\tdir\n",
    );
    assert!(matches!(
        tracked_blob_mode(&tree, "HEAD", "dir"),
        Err(StampError::InvalidTrackedInput)
    ));

    let untracked = FakeGit::default().with(&["ls-tree", "HEAD", "--", "ghost"], true, "");
    assert!(matches!(
        tracked_blob_mode(&untracked, "HEAD", "ghost"),
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
            &[
                "status",
                "--porcelain=v1",
                "-z",
                "--untracked-files=all",
                "--",
                "papers/attestation",
            ],
            true,
            if dirty {
                " M papers/attestation/main.tex\0"
            } else {
                ""
            },
        );

    for (path, bytes) in inputs {
        let absolute = root.join(path);
        std::fs::create_dir_all(absolute.parent().expect("parent")).expect("mkdir");
        std::fs::write(&absolute, bytes).expect("write input");
        request_inputs.push(PathBuf::from(*path));
        fake = fake.with(
            &["ls-tree", "HEAD", "--", path],
            true,
            &format!("100644 blob deadbeef\t{path}\n"),
        );
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
                "papers/attestation",
            ],
            true,
            &format!("{timestamp_epoch}\n"),
        );

    // The date query passes the sorted input paths after `--`.
    let mut date_args = vec![
        "log".to_owned(),
        "-1".to_owned(),
        "--format=%ct".to_owned(),
        "HEAD".to_owned(),
        "--".to_owned(),
    ];
    let mut sorted: Vec<&str> = inputs.iter().map(|(path, _)| *path).collect();
    sorted.sort_unstable();
    for path in sorted {
        date_args.push(path.to_owned());
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
