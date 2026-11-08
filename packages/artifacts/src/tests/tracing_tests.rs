//! Scoped capture of the library's public tracing boundaries.

use std::io::Write;
use std::process::Command;
use std::sync::{Arc, Mutex};

use serde_json::Value;
use tracing_subscriber::fmt::format::FmtSpan;

use super::fixture_census;
use crate::{ARTIFACT_NAMES, ArtifactFreshness, check, expected_artifacts, weld};

static CAPTURE_INSTALL: Mutex<()> = Mutex::new(());
const CAPTURE_CHILD_ENV: &str = "TRIPOD_ARTIFACTS_CAPTURE_CHILD";

fn run_isolated_capture_test(name: &str) -> bool {
    if std::env::var_os(CAPTURE_CHILD_ENV).is_some() {
        return false;
    }
    let output = Command::new(std::env::current_exe().expect("current test executable"))
        .arg("--exact")
        .arg(name)
        .env(CAPTURE_CHILD_ENV, "1")
        .output()
        .expect("run isolated capture test");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success() && stdout.contains("test result: ok. 1 passed; 0 failed;"),
        "isolated capture test {name} failed or selected no test (status {}):\nstdout:\n{stdout}\nstderr:\n{stderr}",
        output.status
    );
    true
}

struct CaptureWriter(Arc<Mutex<Vec<u8>>>);

impl Write for CaptureWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0
            .lock()
            .expect("capture lock")
            .extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn capture<T>(run: impl FnOnce() -> T) -> (T, Vec<Value>) {
    let _install = CAPTURE_INSTALL.lock().expect("capture install lock");
    let bytes = Arc::new(Mutex::new(Vec::new()));
    let writer_bytes = Arc::clone(&bytes);
    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_max_level(tracing::Level::TRACE)
        .with_span_events(FmtSpan::CLOSE)
        .with_writer(move || CaptureWriter(Arc::clone(&writer_bytes)))
        .finish();
    let result = tracing::subscriber::with_default(subscriber, || {
        tracing::callsite::rebuild_interest_cache();
        run()
    });
    let records = bytes
        .lock()
        .expect("capture lock")
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice(line).expect("tracing JSON"))
        .collect();
    (result, records)
}

fn event<'a>(records: &'a [Value], target: &str, message: &str, level: &str) -> &'a Value {
    let matches = records
        .iter()
        .filter(|record| {
            record["target"] == target
                && record["fields"]["message"] == message
                && record["level"] == level
        })
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1, "exactly one {message} event");
    matches[0]
}

fn span_closed(records: &[Value], target: &str, name: &str) {
    let count = records
        .iter()
        .filter(|record| {
            record["target"] == target
                && record["fields"]["message"] == "close"
                && record["span"]["name"] == name
        })
        .count();
    assert_eq!(count, 1, "expected one {name} close record");
}

fn no_warn_or_error(records: &[Value]) {
    assert!(
        records
            .iter()
            .all(|record| record["level"] != "WARN" && record["level"] != "ERROR")
    );
}

fn error_text<T>(result: anyhow::Result<T>) -> String {
    match result {
        Ok(_) => panic!("expected a refusal or failure"),
        Err(error) => error.to_string(),
    }
}

#[test]
fn shared_refusal_names_use_snake_case_wire_names() {
    use architecture::{
        AssetId, BoundId, DecisionId, ManifestError, ObjectId, OperationId, QuantityId, RootId,
    };
    use labels::LabelErrorCode;

    let valid = |name: &str| {
        name.split('_').all(|word| {
            word.bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_lowercase())
                && word
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        })
    };
    let manifest_errors = [
        ManifestError::DuplicateId {
            category: "asset",
            id: "test".to_owned(),
        },
        ManifestError::MissingId {
            category: "asset",
            id: "test".to_owned(),
        },
        ManifestError::InvalidAsset {
            asset: AssetId::Lbtc,
            reason: "test",
        },
        ManifestError::InvalidRoot {
            root: RootId::State,
            reason: "test",
        },
        ManifestError::InvalidObject {
            object: ObjectId::State,
            reason: "test",
        },
        ManifestError::InvalidOperation {
            operation: OperationId::Cycle,
            reason: "test",
        },
        ManifestError::InvalidQuantity {
            quantity: QuantityId::Floor,
            reason: "test",
        },
        ManifestError::InvalidBound {
            bound: BoundId::AdmissionBatchMax,
            reason: "test",
        },
        ManifestError::InvalidDecision {
            decision: DecisionId::NoOnchainAttestationAccumulator,
            reason: "test",
        },
        ManifestError::InvalidDocument { reason: "test" },
        ManifestError::UnpinnedSpecification,
        ManifestError::DraftPublication,
    ];
    let label_codes = [
        LabelErrorCode::Io,
        LabelErrorCode::UnclosedMarkdownFence,
        LabelErrorCode::NestedMarkdownFence,
        LabelErrorCode::UnclosedInlineCode,
        LabelErrorCode::AsymmetricCitation,
        LabelErrorCode::MalformedCitationGroup,
        LabelErrorCode::DuplicateMint,
        LabelErrorCode::MissingMint,
        LabelErrorCode::UnknownOwner,
        LabelErrorCode::UnknownImportedLabel,
        LabelErrorCode::InvalidImportedCitationForm,
        LabelErrorCode::InvalidLabel,
        LabelErrorCode::InvalidStatusTag,
        LabelErrorCode::DuplicateLatexLabel,
        LabelErrorCode::ArchitectureLabelMissing,
        LabelErrorCode::AttestationAnchorSetMismatch,
        LabelErrorCode::AttestationIndexStale,
        LabelErrorCode::GeneratedRegisterMissing,
        LabelErrorCode::GeneratedRegisterStale,
        LabelErrorCode::CensusStale,
        LabelErrorCode::CensusUnreadable,
        LabelErrorCode::UnknownKind,
        LabelErrorCode::ReservedKindWithoutProfile,
        LabelErrorCode::InventoryKindOutOfPlace,
        LabelErrorCode::KindVocabularyDrift,
        LabelErrorCode::UncataloguedHeadPair,
        LabelErrorCode::UnregisteredOwner,
        LabelErrorCode::NearMissSpan,
        LabelErrorCode::MalformedLabelShape,
    ];
    assert_eq!(manifest_errors.len(), 12);
    assert_eq!(label_codes.len(), 29);
    for name in manifest_errors
        .iter()
        .map(ManifestError::name)
        .chain(label_codes.iter().copied().map(LabelErrorCode::as_str))
    {
        assert!(valid(name), "invalid refusal_kind: {name}");
    }
    assert_eq!(ManifestError::DraftPublication.name(), "draft_publication");
    assert_eq!(LabelErrorCode::Io.as_str(), "io");
}

#[test]
fn capture_rebuilds_interest_after_uncaptured_call_across_threads() {
    if run_isolated_capture_test(
        "tests::tracing_tests::capture_rebuilds_interest_after_uncaptured_call_across_threads",
    ) {
        return;
    }
    weld::masthead("# Warm\n---\n").expect("uncaptured warmup");
    std::thread::scope(|scope| {
        let first = scope.spawn(|| capture(|| weld::masthead("# A\n---\n").expect("first")));
        let second = scope.spawn(|| capture(|| weld::masthead("# B\n---\n").expect("second")));
        for handle in [first, second] {
            let (masthead, records) = handle.join().expect("capture thread");
            assert!(masthead == "# A" || masthead == "# B");
            span_closed(&records, "artifacts::weld", "extract_weld");
        }
    });
}

#[test]
fn rendering_completion_and_refusal_are_scoped_and_preserve_bytes() {
    if run_isolated_capture_test(
        "tests::tracing_tests::rendering_completion_and_refusal_are_scoped_and_preserve_bytes",
    ) {
        return;
    }
    let (fixture, census) = fixture_census();
    let plain = expected_artifacts(&census).expect("plain render");
    let (captured, records) = capture(|| expected_artifacts(&census).expect("captured render"));
    for (left, right) in plain.iter().zip(&captured) {
        assert_eq!(left.name, right.name);
        assert_eq!(left.bytes, right.bytes);
    }
    let completion = event(
        &records,
        "artifacts::render",
        "expected artifacts derived",
        "INFO",
    );
    assert_eq!(completion["fields"]["outcome"], "completed");
    assert_eq!(
        completion["fields"]["artifact_count"].as_u64(),
        Some(u64::try_from(ARTIFACT_NAMES.len()).unwrap_or(u64::MAX))
    );
    let bytes = captured.iter().fold(0_u64, |sum, artifact| {
        sum.saturating_add(u64::try_from(artifact.bytes.len()).unwrap_or(u64::MAX))
    });
    assert_eq!(completion["fields"]["rendered_bytes"].as_u64(), Some(bytes));
    span_closed(&records, "artifacts::render", "expected_artifacts");
    no_warn_or_error(&records);

    std::fs::write(
        fixture.path().join("packages/model/src/fixture.rs"),
        "// \u{b4}test:fixture:defined\u{b4}\n// \u{b4}test:fixture:defined\u{b4}\n",
    )
    .expect("duplicate one mint");
    let plain_error = error_text(expected_artifacts(&census));
    let (captured_error, records) = capture(|| error_text(expected_artifacts(&census)));
    assert_eq!(plain_error, captured_error);
    let refusal = event(
        &records,
        "artifacts::render",
        "expected artifact derivation refused",
        "DEBUG",
    );
    assert_eq!(refusal["fields"]["outcome"], "refused");
    assert_eq!(refusal["fields"]["refusal_kind"], "duplicate_mint");
    span_closed(&records, "artifacts::render", "expected_artifacts");
    no_warn_or_error(&records);
}

#[test]
fn checking_completion_counts_and_refusal_preserve_reports() {
    if run_isolated_capture_test(
        "tests::tracing_tests::checking_completion_counts_and_refusal_preserve_reports",
    ) {
        return;
    }
    let (fixture, census) = fixture_census();
    let directory = tempfile::tempdir().expect("generated directory");
    for artifact in expected_artifacts(&census).expect("render") {
        std::fs::write(directory.path().join(artifact.name), artifact.bytes)
            .expect("seed artifact");
    }
    let assert_check = |stale: u64, missing: u64, unexpected: u64, current: bool| {
        let plain = check(directory.path(), &census).expect("plain check");
        let (captured, records) =
            capture(|| check(directory.path(), &census).expect("captured check"));
        assert_eq!(
            serde_json::to_vec(&plain).expect("plain report"),
            serde_json::to_vec(&captured).expect("captured report")
        );
        assert_eq!(captured.current, current);
        let completion = event(
            &records,
            "artifacts::check",
            "generated artifacts checked",
            "INFO",
        );
        assert_eq!(completion["fields"]["outcome"], "completed");
        assert_eq!(
            completion["fields"]["artifact_count"].as_u64(),
            Some(u64::try_from(ARTIFACT_NAMES.len()).unwrap_or(u64::MAX))
        );
        assert_eq!(completion["fields"]["stale_count"], stale);
        assert_eq!(completion["fields"]["missing_count"], missing);
        assert_eq!(completion["fields"]["unexpected_count"], unexpected);
        assert_eq!(completion["fields"]["current"], current);
        span_closed(&records, "artifacts::check", "check");
        span_closed(&records, "artifacts::check", "render_expected");
        assert!(
            !records
                .iter()
                .any(|record| record["fields"]["message"] == "expected artifacts derived")
        );
        no_warn_or_error(&records);
    };
    assert_check(0, 0, 0, true);
    std::fs::write(directory.path().join("architecture.json"), b"corrupted")
        .expect("stale one asset");
    assert_check(1, 0, 0, false);
    std::fs::remove_file(directory.path().join("model_labels.json")).expect("missing one asset");
    assert_check(1, 1, 0, false);
    std::fs::write(directory.path().join("stray.txt"), b"stray").expect("unexpected one asset");
    assert_check(1, 1, 1, false);
    let report = check(directory.path(), &census).expect("check report");
    assert_eq!(report.artifacts[0].status, ArtifactFreshness::Stale);

    std::fs::write(
        fixture.path().join("packages/model/src/fixture.rs"),
        "// \u{b4}test:fixture:defined\u{b4}\n// \u{b4}test:fixture:defined\u{b4}\n",
    )
    .expect("duplicate one mint");
    let plain_error = error_text(check(directory.path(), &census));
    let (captured_error, records) = capture(|| error_text(check(directory.path(), &census)));
    assert_eq!(plain_error, captured_error);
    let refusal = event(
        &records,
        "artifacts::check",
        "generated artifact check refused",
        "DEBUG",
    );
    assert_eq!(refusal["fields"]["outcome"], "refused");
    assert_eq!(refusal["fields"]["refusal_kind"], "duplicate_mint");
    no_warn_or_error(&records);

    let (_fresh_fixture, fresh_census) = fixture_census();
    let missing_directory = directory.path().join("absent-directory");
    let plain_error = error_text(check(&missing_directory, &fresh_census));
    let (captured_error, records) =
        capture(|| error_text(check(&missing_directory, &fresh_census)));
    assert_eq!(plain_error, captured_error);
    let failure = event(
        &records,
        "artifacts::check",
        "generated artifact check failed",
        "INFO",
    );
    assert_eq!(failure["fields"]["outcome"], "failed");
    assert_eq!(failure["fields"]["error_kind"], "io");
    no_warn_or_error(&records);
}

#[test]
fn weld_completion_and_structural_refusal_preserve_text_and_errors() {
    if run_isolated_capture_test(
        "tests::tracing_tests::weld_completion_and_structural_refusal_preserve_text_and_errors",
    ) {
        return;
    }
    let document = concat!(
        "# Masthead\n---\n",
        "## Appendix `app:realization:architecture`\n",
        "```toml\nx = 1\n```\n"
    );
    let plain = weld::masthead(document).expect("plain masthead");
    let (captured, records) = capture(|| weld::masthead(document).expect("captured masthead"));
    assert_eq!(plain, captured);
    let completion = event(
        &records,
        "artifacts::weld",
        "document weld extracted",
        "DEBUG",
    );
    assert_eq!(completion["fields"]["part"], "masthead");
    span_closed(&records, "artifacts::weld", "extract_weld");
    no_warn_or_error(&records);

    let plain = weld::extract_appendix_toml(document).expect("plain appendix");
    let (captured, records) =
        capture(|| weld::extract_appendix_toml(document).expect("captured appendix"));
    assert_eq!(plain, captured);
    let completion = event(
        &records,
        "artifacts::weld",
        "document weld extracted",
        "DEBUG",
    );
    assert_eq!(completion["fields"]["part"], "appendix");
    span_closed(&records, "artifacts::weld", "extract_weld");
    no_warn_or_error(&records);

    let refused_document = document.replace("```toml", "```text");
    let plain_error = error_text(weld::extract_appendix_toml(&refused_document));
    let (captured_error, records) =
        capture(|| error_text(weld::extract_appendix_toml(&refused_document)));
    assert_eq!(plain_error, captured_error);
    let refusal = event(
        &records,
        "artifacts::weld",
        "document weld extraction refused",
        "DEBUG",
    );
    assert_eq!(refusal["fields"]["part"], "appendix");
    assert_eq!(refusal["fields"]["refusal_kind"], "unexpected_fence");
    no_warn_or_error(&records);

    let plain_error = error_text(weld::masthead("# Masthead"));
    let (captured_error, records) = capture(|| error_text(weld::masthead("# Masthead")));
    assert_eq!(plain_error, captured_error);
    let refusal = event(
        &records,
        "artifacts::weld",
        "document weld extraction refused",
        "DEBUG",
    );
    assert_eq!(refusal["fields"]["part"], "masthead");
    assert_eq!(refusal["fields"]["refusal_kind"], "missing_thematic_break");
    no_warn_or_error(&records);
}
