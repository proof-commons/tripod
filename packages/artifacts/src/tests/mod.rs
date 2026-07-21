//! Hermeticity and freshness tests for the generator/checker split.
//!
//! The committed generated directory itself is validated by the
//! Meson-driven `check-generated` target (ADR-014); these tests use
//! synthetic fixture repositories only.

mod weld_tests;

use crate::{ARTIFACT_NAMES, ArtifactFreshness, check, expected_artifacts};

/// A minimal synthetic repository providing the scoped label census
/// the model-label derivation needs.
fn fixture_census() -> (tempfile::TempDir, labels::RepositoryCensus) {
    let directory = tempfile::tempdir().expect("temporary repository");
    let root = directory.path();
    for child in [
        "papers/attestation/sections",
        "docs/attestation",
        "packages/model/src",
    ] {
        std::fs::create_dir_all(root.join(child)).expect("fixture directory");
    }
    std::fs::write(
        root.join("papers/attestation/main.tex"),
        "\\label{def:model:known}\n",
    )
    .expect("attestation source");
    std::fs::write(
        root.join("docs/attestation/realization.md"),
        "# Realization\n`sec:fixture`\n",
    )
    .expect("realization source");
    std::fs::write(
        root.join("packages/model/src/fixture.rs"),
        "// \u{b4}test:fixture:defined\u{b4}\n",
    )
    .expect("model source");
    let census = labels::RepositoryCensus::discover(root);
    (directory, census)
}

/// The expected artifact set covers exactly the owned census, in order.
#[test]
fn expected_artifacts_match_the_census() {
    let (_fixture, census) = fixture_census();
    let expected = expected_artifacts(&census).expect("artifacts derive");

    assert_eq!(
        expected
            .iter()
            .map(|artifact| artifact.name)
            .collect::<Vec<_>>(),
        ARTIFACT_NAMES,
    );

    for artifact in &expected {
        assert!(!artifact.bytes.is_empty(), "{} is empty", artifact.name);
    }
}

/// Derivation is deterministic: two renders produce identical bytes.
#[test]
fn expected_artifacts_are_deterministic() {
    let (_fixture, census) = fixture_census();
    let first = expected_artifacts(&census).expect("artifacts derive");
    let second = expected_artifacts(&census).expect("artifacts derive");

    for (left, right) in first.iter().zip(second.iter()) {
        assert_eq!(left.name, right.name);
        assert_eq!(left.bytes, right.bytes, "{} is nondeterministic", left.name);
    }
}

/// The checker detects staleness, missing files, and strays — and
/// checking never mutates the directory.
#[test]
fn checker_reports_stale_missing_and_unexpected() {
    let (_fixture, census) = fixture_census();
    let dir = tempfile::tempdir().expect("tempdir");

    for artifact in expected_artifacts(&census).expect("artifacts derive") {
        std::fs::write(dir.path().join(artifact.name), &artifact.bytes).expect("write");
    }

    let clean = check(dir.path(), &census).expect("check runs");
    assert!(clean.current);

    std::fs::write(dir.path().join("architecture.json"), b"corrupted").expect("write");
    std::fs::remove_file(dir.path().join("model_labels.json")).expect("remove");
    std::fs::write(dir.path().join("stray.txt"), b"stray").expect("write");

    let report = check(dir.path(), &census).expect("check runs");
    assert!(!report.current);

    let status_of = |name: &str| {
        report
            .artifacts
            .iter()
            .find(|artifact| artifact.name == name)
            .expect("census entry")
            .status
    };
    assert_eq!(status_of("architecture.json"), ArtifactFreshness::Stale);
    assert_eq!(status_of("architecture.toml"), ArtifactFreshness::Current);
    assert_eq!(status_of("model_labels.json"), ArtifactFreshness::Missing);
    assert_eq!(report.unexpected, vec!["stray.txt".to_owned()]);

    // The check must not have repaired anything.
    assert_eq!(
        std::fs::read(dir.path().join("architecture.json")).expect("read"),
        b"corrupted",
    );
    assert!(!dir.path().join("model_labels.json").exists());
}

// Collision safety (B0-010): unique staged temporaries, intact targets
// on failure, and no leftovers.

#[test]
fn concurrent_same_stem_writes_do_not_collide() {
    let dir = tempfile::tempdir().expect("tempdir");
    let json_path = dir.path().join("architecture.json");
    let toml_path = dir.path().join("architecture.toml");

    // Two same-stem targets written concurrently, repeatedly: with a
    // shared predictable temporary name these would exchange bytes.
    std::thread::scope(|scope| {
        let json_path = &json_path;
        let toml_path = &toml_path;
        scope.spawn(move || {
            for _ in 0..200 {
                crate::atomic_write(json_path, b"json-bytes").expect("json write");
            }
        });
        scope.spawn(move || {
            for _ in 0..200 {
                crate::atomic_write(toml_path, b"toml-bytes").expect("toml write");
            }
        });
    });

    assert_eq!(std::fs::read(&json_path).expect("read json"), b"json-bytes");
    assert_eq!(std::fs::read(&toml_path).expect("read toml"), b"toml-bytes");

    // No staged temporary survives success.
    let leftovers = std::fs::read_dir(dir.path())
        .expect("read dir")
        .filter_map(Result::ok)
        .filter(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            name != "architecture.json" && name != "architecture.toml"
        })
        .count();
    assert_eq!(leftovers, 0, "staged temporary files were left behind");
}

// The skip path prints its reason: a silently green skipped test
// would misreport environment-dependent coverage as evidence.
#[allow(clippy::print_stderr)]
#[test]
fn failed_write_leaves_the_original_target_intact() {
    let dir = tempfile::tempdir().expect("tempdir");
    let target = dir.path().join("artifact.json");
    std::fs::write(&target, b"original").expect("seed target");

    // Persisting over a target in an unwritable directory fails after
    // staging; the original must be untouched and nothing staged left.
    let mut permissions = std::fs::metadata(dir.path())
        .expect("metadata")
        .permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o555);
    std::fs::set_permissions(dir.path(), permissions.clone()).expect("set read-only");

    // A privileged process (root / CAP_DAC_OVERRIDE, common in rootful
    // CI containers) can write into a 0o555 directory, so the mode
    // change would not produce the failure under test. Probe and skip
    // explicitly rather than reporting an environment property as an
    // implementation failure.
    let probe = dir.path().join("permission-probe");
    if std::fs::write(&probe, b"probe").is_ok() {
        let _removed = std::fs::remove_file(&probe);
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        std::fs::set_permissions(dir.path(), permissions).expect("restore permissions");
        eprintln!(
            "skipping failed_write_leaves_the_original_target_intact: \
             process writes despite 0o555 (privileged test environment)"
        );
        return;
    }

    let result = crate::atomic_write(&target, b"replacement");

    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
    std::fs::set_permissions(dir.path(), permissions).expect("restore permissions");

    assert!(
        result.is_err(),
        "write into a read-only directory must fail"
    );
    assert_eq!(std::fs::read(&target).expect("read target"), b"original");

    let leftovers = std::fs::read_dir(dir.path())
        .expect("read dir")
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name() != "artifact.json")
        .count();
    assert_eq!(leftovers, 0, "failed write left a staged file behind");
}
