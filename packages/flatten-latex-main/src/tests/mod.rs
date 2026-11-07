//! # `flatten-latex-main` library tests
//!
//! | Test                                       | What it covers                          |
//! |--------------------------------------------|-----------------------------------------|
//! | `extract_braced_returns_inner_value_and_rest` | Parses `\cmd{value}` plus remainder     |
//! | `extract_braced_rejects_other_command`     | Returns None when prefix mismatches     |
//! | `extract_include_handles_input`            | `\input{}` parsed                       |
//! | `extract_include_handles_subfile`          | `\subfile{}` parsed                     |
//! | `flatten_writes_inlined_output`            | End-to-end flatten on a temp tree       |
//! | `flatten_ignores_marked_include`           | `% flatten-ignore` include is skipped   |
//! | `flatten_inlines_a_supplied_file_anywhere` | A listed file is matched by name, not dir |
//! | `flatten_expands_conditional_include`       | Conditional include keeps survivor code |
//! | `flatten_is_reproducible`                   | Same inputs, byte-identical output      |
//! | `flatten_detects_include_cycles`            | Self/mutual includes fail, not recurse  |
//! | `flatten_rejects_trailing_tokens_after_include` | `\input{a}\label{b}` fails, not drops |
//! | `flatten_allows_trailing_comment_after_include` | `\input{a} % note` still flattens   |
//! | `flatten_rejects_embedded_include`          | prefixed/mid-line includes fail         |
//! | `flatten_rejects_nonempty_false_branch`     | absent conditional keeps no silent drop |
//! | `conditional_with_absent_probe_takes_the_false_branch_even_if_include_exists` | Branch selection follows the probe |
//! | `conditional_with_existing_probe_and_absent_include_is_an_error` | True branch failure is never silently dropped |
//! | `conditional_probe_and_include_naming_distinct_files_is_rejected_atomically` | Divergent probe/include pairs are refused |
//! | `flatten_passes_commented_includes`         | `% \input{a}` is inert                  |
//! | `flatten_failure_leaves_no_partial_output`  | Atomic output on failed flatten         |
//! | `flatten_failure_preserves_existing_output` | Failed flatten keeps prior output bytes |
//! | `concurrent_flattens_never_mix_output`      | Same-output races yield one full result |
//! | `strict_bibliography_fails_on_missing_bib`  | Release mode hard-fails an unlisted bib |
//! | `flatten_embeds_a_supplied_bibliography`     | A listed bib is inlined verbatim        |
//! | `flatten_rejects_absolute_include`          | `\input{/etc/passwd}` refused           |
//! | `flatten_rejects_parent_traversal`          | `\input{../../secret}` refused          |
//! | `flatten_rejects_unlisted_file_on_disk`     | A real adjacent file not on the list is refused |
//! | `flatten_rejects_symlink_escape`            | An unlisted symlink is never followed   |
//! | `flatten_aborts_on_ambiguous_reference`     | A reference matching two listed files aborts |
//! | `flatten_rejects_allowlisted_symlink`       | Listing a symlink never authorizes its target |
//! | `flatten_rejects_symlinked_main`            | The entry point must be a regular file  |
//! | `flatten_rejects_symlink_cycles_without_recursing` | A symlink loop is refused, not chased |
//! | `flatten_detects_cycles_through_hard_link_aliases` | Cycle detection is by allowlist path |
//! | `file_beneath_nested_symlinked_ancestors_is_a_caller_capability` | A supplied path's ancestors are the host's business |
//! | `main_beneath_symlinked_ancestor_is_accepted` | The entry point is an explicit caller path |
//! | `a_reference_off_the_allowlist_remains_unreachable_beneath_a_symlink` | Source text still selects only supplied files |
//! | `failed_confinement_preserves_output_and_stages_nothing` | Validation precedes staging |
//! | `flatten_disambiguates_by_folder`           | Folder components resolve a duplicate filename |

// These tests embed LaTeX include literals such as `\input{section}`. When a
// brace's contents happen to name an in-scope binding, clippy suspects a
// missed `format!`; here they are deliberate TeX source, not format strings.
#![allow(clippy::literal_string_with_formatting_args)]

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::tempdir;

use super::{FlattenOptions, extract_braced_with_rest, extract_include_with_rest, flatten};

mod tracing_tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::process::Command;
    use std::sync::{Arc, Mutex};

    use serde_json::{Value, json};
    use tempfile::tempdir;
    use tracing::field::{Field, Visit};
    use tracing::span::{Attributes, Id};
    use tracing::{Event, Subscriber};
    use tracing_subscriber::layer::{Context, SubscriberExt};
    use tracing_subscriber::registry::LookupSpan;
    use tracing_subscriber::{Layer, Registry};

    use super::super::{ClassifiedError, FailureClass, classification};
    use super::{FlattenOptions, flatten, write_file};

    // Only one capture may install or drop a scoped dispatcher at a time.
    static CAPTURE_TESTS: Mutex<()> = Mutex::new(());

    fn capture_test_guard() -> std::sync::MutexGuard<'static, ()> {
        CAPTURE_TESTS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn run_capture_test_in_child(test_name: &str) -> bool {
        const CHILD_MARKER: &str = "FLATTEN_LATEX_MAIN_CAPTURE_TEST_CHILD";
        if std::env::var(CHILD_MARKER).as_deref() == Ok(test_name) {
            return true;
        }
        let output = Command::new(std::env::current_exe().expect("current test executable"))
            .args(["--exact", test_name])
            .env(CHILD_MARKER, test_name)
            .output()
            .expect("run isolated capture test");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success()
                && stdout
                    .lines()
                    .any(|line| line.starts_with("test result: ok. 1 passed; 0 failed;")),
            "isolated capture test {test_name} failed or selected no test: status={:?}\nstdout:\n{stdout}\nstderr:\n{stderr}",
            output.status
        );
        false
    }

    #[derive(Clone, Debug)]
    struct Record {
        kind: &'static str,
        target: &'static str,
        level: tracing::Level,
        name: &'static str,
        fields: BTreeMap<String, Value>,
    }

    #[derive(Default)]
    struct Fields(BTreeMap<String, Value>);

    impl Visit for Fields {
        fn record_u64(&mut self, field: &Field, value: u64) {
            self.0.insert(field.name().to_owned(), json!(value));
        }

        fn record_bool(&mut self, field: &Field, value: bool) {
            self.0.insert(field.name().to_owned(), json!(value));
        }

        fn record_str(&mut self, field: &Field, value: &str) {
            self.0.insert(field.name().to_owned(), json!(value));
        }

        fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
            self.0
                .insert(field.name().to_owned(), json!(format!("{value:?}")));
        }
    }

    struct CaptureLayer(Arc<Mutex<Vec<Record>>>);

    impl<S> Layer<S> for CaptureLayer
    where
        S: Subscriber + for<'lookup> LookupSpan<'lookup>,
    {
        fn on_new_span(&self, attrs: &Attributes<'_>, _id: &Id, _ctx: Context<'_, S>) {
            let mut fields = Fields::default();
            attrs.record(&mut fields);
            let metadata = attrs.metadata();
            self.0.lock().expect("capture lock").push(Record {
                kind: "span",
                target: metadata.target(),
                level: *metadata.level(),
                name: metadata.name(),
                fields: fields.0,
            });
        }

        fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
            let mut fields = Fields::default();
            event.record(&mut fields);
            let metadata = event.metadata();
            self.0.lock().expect("capture lock").push(Record {
                kind: "event",
                target: metadata.target(),
                level: *metadata.level(),
                name: metadata.name(),
                fields: fields.0,
            });
        }

        fn on_close(&self, id: Id, ctx: Context<'_, S>) {
            let span = ctx.span(&id).expect("span still present at close");
            let metadata = span.metadata();
            self.0.lock().expect("capture lock").push(Record {
                kind: "close",
                target: metadata.target(),
                level: *metadata.level(),
                name: metadata.name(),
                fields: BTreeMap::new(),
            });
        }
    }

    fn capture<T>(operation: impl FnOnce() -> T) -> (T, Vec<Record>) {
        let _capture_test = capture_test_guard();
        let records = Arc::new(Mutex::new(Vec::new()));
        let subscriber = Registry::default().with(CaptureLayer(Arc::clone(&records)));
        let result = tracing::subscriber::with_default(subscriber, || {
            tracing::callsite::rebuild_interest_cache();
            operation()
        });
        let captured = records.lock().expect("capture lock").clone();
        (result, captured)
    }

    fn completion(records: &[Record]) -> &Record {
        let events: Vec<_> = records
            .iter()
            .filter(|record| {
                record.kind == "event"
                    && record.target == "flatten_latex_main::flatten"
                    && record.fields.contains_key("outcome")
            })
            .collect();
        assert_eq!(
            events.len(),
            1,
            "one completion event per call: {records:?}"
        );
        assert_eq!(
            events[0].fields.get("message"),
            Some(&json!("flatten completed"))
        );
        events[0]
    }

    fn assert_span(records: &[Record], name: &str, target: &str, level: tracing::Level) {
        assert!(
            records.iter().any(|record| record.kind == "span"
                && record.name == name
                && record.target == target
                && record.level == level),
            "missing span {name}"
        );
        assert!(
            records.iter().any(|record| record.kind == "close"
                && record.name == name
                && record.target == target
                && record.level == level),
            "missing close {name}"
        );
    }

    fn assert_file_spans(records: &[Record]) {
        let file_spans: Vec<_> = records
            .iter()
            .filter(|record| record.kind == "span" && record.name == "process_file")
            .collect();
        assert_eq!(file_spans.len(), 2);
        assert!(
            file_spans
                .iter()
                .any(|record| record.fields.get("depth") == Some(&json!(0))
                    && record.fields.get("is_main") == Some(&json!(true)))
        );
        assert!(
            file_spans
                .iter()
                .any(|record| record.fields.get("depth") == Some(&json!(1))
                    && record.fields.get("is_main") == Some(&json!(false)))
        );
    }

    fn assert_same_error_surface(plain: &anyhow::Error, traced: &anyhow::Error) {
        let plain_chain: Vec<_> = plain.chain().map(ToString::to_string).collect();
        let traced_chain: Vec<_> = traced.chain().map(ToString::to_string).collect();
        assert_eq!(plain_chain, traced_chain, "error chains differ");
        assert_eq!(format!("{plain:#}"), format!("{traced:#}"));
        assert_eq!(
            format!("{plain:#}").matches(&plain.to_string()).count(),
            1,
            "the outer message appears once in the chain rendering"
        );
        assert!(plain.downcast_ref::<ClassifiedError>().is_some());
        assert!(traced.downcast_ref::<ClassifiedError>().is_some());
        assert_eq!(
            format!("{:?}", classification(plain)),
            format!("{:?}", classification(traced))
        );
    }

    #[test]
    fn classification_preserves_error_chain_with_and_without_capture() {
        if !run_capture_test_in_child(
            "tests::tracing_tests::classification_preserves_error_chain_with_and_without_capture",
        ) {
            return;
        }
        let dir = tempdir().expect("tempdir");
        let main = dir.path().join("main.tex");
        let section = dir.path().join("section.tex");
        let output = dir.path().join("flat.tex");
        let options = FlattenOptions::default();
        write_file(&main, "\\input{missing}\n");
        write_file(&section, "Section text\n");

        let plain = flatten(&main, std::slice::from_ref(&section), &output, &options)
            .expect_err("uncaptured missing include");
        let (captured, records) =
            capture(|| flatten(&main, std::slice::from_ref(&section), &output, &options));
        let traced = captured.expect_err("captured missing include");
        assert_same_error_surface(&plain, &traced);
        assert!(matches!(
            classification(&plain),
            Some(FailureClass::Refused {
                kind: "missing_include",
                depth: Some(0)
            })
        ));
        assert_eq!(
            completion(&records).fields.get("outcome"),
            Some(&json!("refused"))
        );

        write_file(&main, "Main text\n");
        let blocker = dir.path().join("blocker");
        write_file(&blocker, "not a directory");
        let blocked_output = blocker.join("child").join("flat.tex");
        let plain = flatten(&main, &[], &blocked_output, &options)
            .expect_err("uncaptured publication failure");
        let (captured, records) = capture(|| flatten(&main, &[], &blocked_output, &options));
        let traced = captured.expect_err("captured publication failure");
        assert_same_error_surface(&plain, &traced);
        assert!(matches!(
            classification(&plain),
            Some(FailureClass::Failed {
                kind: "io",
                phase: "write"
            })
        ));
        assert_eq!(
            completion(&records).fields.get("outcome"),
            Some(&json!("failed"))
        );
    }

    #[test]
    fn scoped_capture_preserves_completed_and_refused_results() {
        if !run_capture_test_in_child(
            "tests::tracing_tests::scoped_capture_preserves_completed_and_refused_results",
        ) {
            return;
        }
        let dir = tempdir().expect("tempdir");
        let main = dir.path().join("main.tex");
        let section = dir.path().join("section.tex");
        let bib = dir.path().join("refs.bib");
        let plain = dir.path().join("plain.tex");
        let traced = dir.path().join("traced.tex");
        write_file(&main, "\\input{section}\n\\addbibresource{refs.bib}\n");
        write_file(&section, "Section text\n");
        write_file(&bib, "@book{one, title={One}}\n");
        let allowed = vec![section, bib];
        let options = FlattenOptions::default();

        let plain_result = flatten(&main, &allowed, &plain, &options);
        let (traced_result, records) = capture(|| flatten(&main, &allowed, &traced, &options));
        assert!(plain_result.is_ok() && traced_result.is_ok());
        assert_eq!(
            fs::read(&plain).expect("plain asset"),
            fs::read(&traced).expect("traced asset")
        );
        assert_span(
            &records,
            "flatten",
            "flatten_latex_main::flatten",
            tracing::Level::INFO,
        );
        assert_span(
            &records,
            "validate_inputs",
            "flatten_latex_main::input",
            tracing::Level::DEBUG,
        );
        assert_span(
            &records,
            "write_flattened",
            "flatten_latex_main::output",
            tracing::Level::DEBUG,
        );
        assert_span(
            &records,
            "process_file",
            "flatten_latex_main::input",
            tracing::Level::DEBUG,
        );
        let root = records
            .iter()
            .find(|record| record.kind == "span" && record.name == "flatten")
            .expect("root span");
        assert_eq!(root.fields.get("allowed_file_count"), Some(&json!(2)));
        assert_eq!(root.fields.get("strict_bibliography"), Some(&json!(false)));
        assert_file_spans(&records);
        let completed = completion(&records);
        assert_eq!(completed.level, tracing::Level::INFO);
        for (field, value) in [
            ("outcome", json!("completed")),
            ("include_count", json!(1)),
            ("omitted_include_count", json!(0)),
            ("bibliography_count", json!(1)),
            ("file_count", json!(2)),
            ("max_include_depth", json!(1)),
        ] {
            assert_eq!(completed.fields.get(field), Some(&value), "field {field}");
        }

        write_file(&main, "\\input{missing}\n\\addbibresource{refs.bib}\n");
        let plain_error =
            flatten(&main, &allowed, &plain, &options).expect_err("missing include refused");
        let (traced_result, refused_records) =
            capture(|| flatten(&main, &allowed, &traced, &options));
        let traced_error = traced_result.expect_err("missing include refused");
        assert_eq!(plain_error.to_string(), traced_error.to_string());
        assert_eq!(
            fs::read(&plain).expect("prior plain asset"),
            fs::read(&traced).expect("prior traced asset")
        );
        let refused = completion(&refused_records);
        assert_eq!(refused.level, tracing::Level::DEBUG);
        assert_eq!(refused.fields.get("outcome"), Some(&json!("refused")));
        assert_eq!(
            refused.fields.get("refusal_kind"),
            Some(&json!("missing_include"))
        );
        assert_eq!(refused.fields.get("depth"), Some(&json!(0)));
        assert!(!refused_records.iter().any(|record| record.kind == "event"
            && matches!(record.level, tracing::Level::WARN | tracing::Level::ERROR)));
    }

    #[test]
    fn lenient_bibliography_warns_without_changing_asset() {
        if !run_capture_test_in_child(
            "tests::tracing_tests::lenient_bibliography_warns_without_changing_asset",
        ) {
            return;
        }
        let dir = tempdir().expect("tempdir");
        let main = dir.path().join("main.tex");
        write_file(&main, "\\addbibresource{missing.bib}\n");
        let plain = dir.path().join("plain.tex");
        let traced = dir.path().join("traced.tex");
        let options = FlattenOptions::default();
        assert!(flatten(&main, &[], &plain, &options).is_ok());
        let (result, records) = capture(|| flatten(&main, &[], &traced, &options));
        assert!(result.is_ok());
        let bytes = fs::read(&traced).expect("asset");
        assert_eq!(fs::read(&plain).expect("plain asset"), bytes);
        assert!(String::from_utf8(bytes).expect("utf8 asset").contains(
            "% WARNING: Bibliography file 'missing.bib' not found among the supplied files"
        ));
        let warnings: Vec<_> = records
            .iter()
            .filter(|record| record.kind == "event" && record.level == tracing::Level::WARN)
            .collect();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].target, "flatten_latex_main::resolve");
        assert_eq!(
            warnings[0].fields.get("message"),
            Some(&json!("bibliography omitted"))
        );
        assert_eq!(
            warnings[0].fields.get("strict_bibliography"),
            Some(&json!(false))
        );
        assert_eq!(
            completion(&records).fields.get("bibliography_count"),
            Some(&json!(0))
        );
    }

    #[test]
    fn validation_refusal_is_debug_and_preserves_prior_output() {
        if !run_capture_test_in_child(
            "tests::tracing_tests::validation_refusal_is_debug_and_preserves_prior_output",
        ) {
            return;
        }
        let dir = tempdir().expect("tempdir");
        let main = dir.path().join("main.tex");
        let missing = dir.path().join("missing.tex");
        let plain = dir.path().join("plain.tex");
        let traced = dir.path().join("traced.tex");
        let options = FlattenOptions::default();
        write_file(&main, "Main text\n");
        assert!(flatten(&main, &[], &plain, &options).is_ok());
        let (control, control_records) = capture(|| flatten(&main, &[], &traced, &options));
        assert!(control.is_ok());
        assert_eq!(
            fs::read(&plain).expect("plain asset"),
            fs::read(&traced).expect("traced asset")
        );
        assert_span(
            &control_records,
            "validate_inputs",
            "flatten_latex_main::input",
            tracing::Level::DEBUG,
        );

        let plain_error = flatten(&missing, &[], &plain, &options).expect_err("missing main");
        let (result, records) = capture(|| flatten(&missing, &[], &traced, &options));
        let traced_error = result.expect_err("missing main");
        assert_eq!(plain_error.to_string(), traced_error.to_string());
        assert_eq!(
            fs::read(&plain).expect("prior plain asset"),
            fs::read(&traced).expect("prior traced asset")
        );
        assert_span(
            &records,
            "validate_inputs",
            "flatten_latex_main::input",
            tracing::Level::DEBUG,
        );
        let refused = completion(&records);
        assert_eq!(refused.level, tracing::Level::DEBUG);
        assert_eq!(refused.fields.get("outcome"), Some(&json!("refused")));
        assert_eq!(
            refused.fields.get("refusal_kind"),
            Some(&json!("invalid_input"))
        );
        assert_eq!(refused.fields.get("depth"), Some(&json!(0)));
        assert!(!records.iter().any(|record| record.kind == "event"
            && matches!(record.level, tracing::Level::WARN | tracing::Level::ERROR)));
    }

    #[test]
    fn uncaptured_call_then_two_thread_captures_each_record_one_span() {
        if !run_capture_test_in_child(
            "tests::tracing_tests::uncaptured_call_then_two_thread_captures_each_record_one_span",
        ) {
            return;
        }
        let dir = tempdir().expect("tempdir");
        let main = dir.path().join("main.tex");
        let plain = dir.path().join("plain.tex");
        write_file(&main, "Main text\n");
        flatten(&main, &[], &plain, &FlattenOptions::default()).expect("uncaptured flatten");
        let expected = fs::read(plain).expect("uncaptured asset");

        let handles: Vec<_> = (0..2)
            .map(|index| {
                let main = main.clone();
                let output = dir.path().join(format!("captured-{index}.tex"));
                std::thread::spawn(move || {
                    let (result, records) =
                        capture(|| flatten(&main, &[], &output, &FlattenOptions::default()));
                    result.expect("captured flatten");
                    let root_spans = records
                        .iter()
                        .filter(|record| {
                            record.kind == "span"
                                && record.name == "flatten"
                                && record.target == "flatten_latex_main::flatten"
                        })
                        .count();
                    assert_eq!(root_spans, 1, "one root span per capture: {records:?}");
                    fs::read(output).expect("captured asset")
                })
            })
            .collect();
        for handle in handles {
            assert_eq!(handle.join().expect("capture thread"), expected);
        }
    }
}

/// Write `content` to `path`, creating parent directories as needed.
fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dir");
    }
    fs::write(path, content).expect("write file");
}

#[test]
fn extract_braced_returns_inner_value_and_rest() {
    let parsed = extract_braced_with_rest("\\addbibresource{refs.bib} % note", "\\addbibresource");
    assert_eq!(
        parsed,
        Some(("refs.bib".to_string(), " % note")),
        "value and remainder are both returned"
    );
}

#[test]
fn extract_braced_rejects_other_command() {
    let parsed = extract_braced_with_rest("\\addbibresource{refs.bib}", "\\input");
    assert!(parsed.is_none());
}

#[test]
fn extract_include_handles_input() {
    let parsed = extract_include_with_rest("\\input{section}");
    assert_eq!(parsed, Some(("section".to_string(), "")));
}

#[test]
fn extract_include_handles_subfile() {
    let parsed = extract_include_with_rest("\\subfile{chapter}");
    assert_eq!(parsed, Some(("chapter".to_string(), "")));
}

#[test]
fn tracing_category_names_are_snake_case() {
    const OUTCOMES: &[&str] = &["completed", "refused", "failed"];
    const REFUSAL_KINDS: &[&str] = &[
        "invalid_input",
        "invalid_output",
        "invalid_reference",
        "missing_include",
        "ambiguous_reference",
        "include_cycle",
        "unsupported_syntax",
        "missing_bibliography",
    ];
    const ERROR_KINDS: &[&str] = &["io", "publication"];
    const PHASES: &[&str] = &["validate", "write", "rename"];
    for name in OUTCOMES
        .iter()
        .chain(REFUSAL_KINDS)
        .chain(ERROR_KINDS)
        .chain(PHASES)
    {
        let mut bytes = name.bytes();
        assert!(
            matches!(bytes.next(), Some(first) if first.is_ascii_lowercase())
                && !name.ends_with('_')
                && !name.contains("__")
                && bytes
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'),
            "invalid tracing category {name}"
        );
    }
}

#[test]
fn flatten_writes_inlined_output() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let paper_dir = root.join("papers/network");
    let main = paper_dir.join("main.tex");
    let section = paper_dir.join("section.tex");
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{section}\n\\end{document}\n",
    );
    write_file(
        &section,
        "\\documentclass[../main.tex]{subfiles}\n\\begin{document}\nhello world\n\\end{document}\n",
    );

    let output = root.join("flat/blueprint_flat.tex");
    flatten(&main, &[section], &output, &FlattenOptions::default()).expect("flatten");
    let text = fs::read_to_string(output).expect("read output");
    assert!(text.contains("\\documentclass{article}"));
    assert!(text.contains("% \\documentclass[../main.tex]{subfiles}"));
    assert!(text.contains("\\begin{document}"));
    assert!(text.contains("% \\begin{document}"));
    assert!(text.contains("% --- BEGIN included content from: section ---"));
    assert!(text.contains("hello world"));
    assert!(text.contains("% \\end{document}"));
    assert!(text.contains("% --- END included content from: section ---"));
}

#[test]
fn flatten_ignores_marked_include() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("papers/network/main.tex");
    // The bridge file is neither on disk nor on the allowlist; the flattener
    // must still succeed because the include is annotated `% flatten-ignore`.
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{generated_bridge} % flatten-ignore\n\\end{document}\n",
    );

    let output = root.join("flat/blueprint_flat.tex");
    flatten(&main, &[], &output, &FlattenOptions::default()).expect("flatten");
    let text = fs::read_to_string(output).expect("read output");
    assert!(text.contains("% \\input{generated_bridge} % flatten-ignore"));
    assert!(text.contains("flatten-ignore: external include 'generated_bridge' omitted"));
    assert!(!text.contains("BEGIN included content from: generated_bridge"));
}

#[test]
fn flatten_inlines_a_supplied_file_anywhere() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("papers/network/main.tex");
    // The macro file lives in a completely separate directory; the flattener
    // finds it purely by matching the reference name against the supplied
    // list, with no knowledge of either directory.
    let staged = root.join("build/staged/macros_token.tex");
    write_file(
        &main,
        "\\documentclass{article}\n\\input{macros_token.tex}\n\\begin{document}\n\\end{document}\n",
    );
    write_file(&staged, "\\newcommand{\\foo}{bar}\n");

    let output = root.join("flat/blueprint_flat.tex");
    flatten(&main, &[staged], &output, &FlattenOptions::default()).expect("flatten");
    let text = fs::read_to_string(output).expect("read output");
    assert!(text.contains("% --- BEGIN included content from: macros_token.tex ---"));
    assert!(text.contains("\\newcommand{\\foo}{bar}"));
}

#[test]
fn flatten_expands_conditional_include() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("papers/economic/main.tex");
    let bridge = root.join("build/papers/economic/bridge.tex");
    write_file(
        &main,
        "\\documentclass{article}\n\\newif\\ifbridge\n\\bridgefalse\n\\IfFileExists{bridge.tex}{\\input{bridge}\\bridgetrue}{}\n\\begin{document}\n\\end{document}\n",
    );
    write_file(&bridge, "\\newcommand{\\linked}{yes}\n");

    let output = root.join("flat/economic_flat.tex");
    flatten(&main, &[bridge], &output, &FlattenOptions::default()).expect("flatten");
    let text = fs::read_to_string(output).expect("read output");
    assert!(text.contains("% \\IfFileExists{bridge.tex}{\\input{bridge}\\bridgetrue}{}"));
    assert!(text.contains("% --- BEGIN included content from: bridge ---"));
    assert!(text.contains("\\newcommand{\\linked}{yes}"));
    assert!(text.contains("% --- END included content from: bridge ---\n\\bridgetrue"));
    assert!(!text.contains("\\IfFileExists{bridge.tex}{%"));
}

// --- F3-009 conditional branch selection follows the probe, not the
// nested include: the two need not name the same file, and guessing
// from the include can flatten the wrong branch. ---

#[test]
fn conditional_with_absent_probe_takes_the_false_branch_even_if_include_exists() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("paper/main.tex");
    let bridge = root.join("build/bridge.tex");
    write_file(&bridge, "\\newcommand{\\linked}{yes}\n");
    // LaTeX probes probe-me.tex, which does not exist under the
    // fixed-list model, so the (empty) false branch is taken even
    // though the nested include would resolve.
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\IfFileExists{probe-me.tex}{\\input{bridge}}{}\n\\end{document}\n",
    );

    let output = root.join("flat.tex");
    flatten(&main, &[bridge], &output, &FlattenOptions::default()).expect("flatten");
    let text = fs::read_to_string(output).expect("read output");
    assert!(text.contains("% --- flatten: conditional probe 'probe-me.tex' absent"));
    assert!(!text.contains("BEGIN included content from: bridge"));
    assert!(!text.contains("\\newcommand{\\linked}{yes}"));
}

#[test]
fn conditional_with_existing_probe_and_absent_include_is_an_error() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("paper/main.tex");
    let probe = root.join("paper/probe-me.tex");
    write_file(&probe, "probe body\n");
    // LaTeX would take the true branch and then fail on the missing
    // include; the flattener must fail too, never silently take the
    // false branch.
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\IfFileExists{probe-me.tex}{\\input{missing}}{}\n\\end{document}\n",
    );

    let output = root.join("flat.tex");
    let error = flatten(&main, &[probe], &output, &FlattenOptions::default())
        .expect_err("an existing probe with an absent include must fail");
    assert!(
        error
            .to_string()
            .contains("probe 'probe-me.tex' exists but the true branch include"),
        "{error:#}"
    );
    assert!(!output.exists());
}

#[test]
fn conditional_probe_and_include_naming_distinct_files_is_rejected_atomically() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("paper/main.tex");
    let probe = root.join("paper/probe-me.tex");
    let other = root.join("paper/other.tex");
    write_file(&probe, "probe body\n");
    write_file(&other, "other body\n");
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\IfFileExists{probe-me.tex}{\\input{other}}{}\n\\end{document}\n",
    );

    let output = root.join("flat.tex");
    write_file(&output, "previous good output\n");
    let error = flatten(&main, &[probe, other], &output, &FlattenOptions::default())
        .expect_err("a divergent probe/include pair is outside exact restricted support");
    assert!(
        error
            .to_string()
            .contains("resolve to different supplied files"),
        "{error:#}"
    );
    assert_eq!(
        fs::read_to_string(&output).expect("existing output"),
        "previous good output\n",
    );
}

#[test]
fn flatten_is_reproducible() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("paper/main.tex");
    let section = root.join("paper/section.tex");
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{section}\n\\end{document}\n",
    );
    write_file(&section, "content\n");

    let first = root.join("first.tex");
    let second = root.join("second.tex");
    let allow = [section];
    flatten(&main, &allow, &first, &FlattenOptions::default()).expect("flatten");
    flatten(&main, &allow, &second, &FlattenOptions::default()).expect("flatten");

    assert_eq!(
        fs::read(first).expect("read first"),
        fs::read(second).expect("read second"),
        "flatten output is not byte-identical across runs",
    );
}

#[test]
fn flatten_detects_include_cycles() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("paper/main.tex");
    let a = root.join("paper/a.tex");
    let b = root.join("paper/b.tex");
    // a -> b -> a: must fail with a cycle diagnostic, not recurse.
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{a}\n\\end{document}\n",
    );
    write_file(&a, "\\input{b}\n");
    write_file(&b, "\\input{a}\n");

    let output = root.join("flat.tex");
    let error =
        flatten(&main, &[a, b], &output, &FlattenOptions::default()).expect_err("cycle must fail");
    assert!(error.to_string().contains("include cycle"), "{error}");
}

/// Build `main.tex` holding `body` between the standard document markers, plus
/// a trivial `section.tex`. Returns the main path and the `[section.tex]`
/// allowlist.
fn paper_with_body(root: &Path, body: &str) -> (PathBuf, Vec<PathBuf>) {
    let main = root.join("paper/main.tex");
    let section = root.join("paper/section.tex");
    write_file(
        &main,
        &format!("\\documentclass{{article}}\n\\begin{{document}}\n{body}\n\\end{{document}}\n"),
    );
    write_file(&section, "hello section\n");
    (main, vec![section])
}

#[test]
fn flatten_rejects_trailing_tokens_after_include() {
    let dir = tempdir().expect("tempdir");
    let (main, allow) = paper_with_body(dir.path(), "\\input{section}\\label{after-section}");

    let output = dir.path().join("flat.tex");
    let error = flatten(&main, &allow, &output, &FlattenOptions::default())
        .expect_err("trailing tokens after an include must fail, not silently drop");

    assert!(error.to_string().contains("unsupported include syntax"));
    assert!(!output.exists());
}

#[test]
fn flatten_allows_trailing_comment_after_include() {
    let dir = tempdir().expect("tempdir");
    let (main, allow) = paper_with_body(dir.path(), "\\input{section} % layer note");

    let output = dir.path().join("flat.tex");
    flatten(&main, &allow, &output, &FlattenOptions::default())
        .expect("trailing comment is not semantic content");

    let text = fs::read_to_string(output).expect("read output");
    assert!(text.contains("hello section"));
}

#[test]
fn flatten_rejects_embedded_include() {
    let dir = tempdir().expect("tempdir");
    let (main, allow) = paper_with_body(dir.path(), "prefix text \\input{section}");

    let error = flatten(
        &main,
        &allow,
        &dir.path().join("flat.tex"),
        &FlattenOptions::default(),
    )
    .expect_err("an include the dispatcher cannot handle must not pass through");

    assert!(error.to_string().contains("unsupported include syntax"));
}

#[test]
fn flatten_rejects_nonempty_false_branch() {
    let dir = tempdir().expect("tempdir");
    let (main, allow) = paper_with_body(
        dir.path(),
        "\\IfFileExists{missing-optional.tex}{\\input{missing-optional}}{\\fallbackmacro}",
    );

    let error = flatten(
        &main,
        &allow,
        &dir.path().join("flat.tex"),
        &FlattenOptions::default(),
    )
    .expect_err("a dropped nonempty false branch must fail the flatten");

    assert!(error.to_string().contains("false branch"));
}

#[test]
fn flatten_passes_commented_includes() {
    let dir = tempdir().expect("tempdir");
    let (main, allow) = paper_with_body(dir.path(), "% \\input{not-really-included}");

    let output = dir.path().join("flat.tex");
    flatten(&main, &allow, &output, &FlattenOptions::default())
        .expect("a commented include is inert");

    let text = fs::read_to_string(output).expect("read output");
    assert!(text.contains("% \\input{not-really-included}"));
}

#[test]
fn flatten_failure_leaves_no_partial_output() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("paper/main.tex");
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{missing-section}\n\\end{document}\n",
    );

    let output = root.join("flat.tex");
    flatten(&main, &[], &output, &FlattenOptions::default())
        .expect_err("an unlisted include must fail");

    assert!(!output.exists(), "failed flatten left a partial output");
    assert_eq!(
        staged_leftovers(root),
        0,
        "failed flatten left a temporary file"
    );
}

/// Count leftover staging files (any name marking a temporary) under `root`.
fn staged_leftovers(root: &Path) -> usize {
    fs::read_dir(root)
        .expect("read root")
        .filter_map(std::result::Result::ok)
        .filter(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            Path::new(&name)
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("tmp"))
                || name.starts_with(".flatten-staged-")
        })
        .count()
}

#[test]
fn flatten_failure_preserves_existing_output() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("paper/main.tex");
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{missing-section}\n\\end{document}\n",
    );

    let output = root.join("flat.tex");
    fs::write(&output, "previous good output\n").expect("write prior output");

    flatten(&main, &[], &output, &FlattenOptions::default())
        .expect_err("an unlisted include must fail");

    assert_eq!(
        fs::read_to_string(&output).expect("read output"),
        "previous good output\n",
        "failed flatten must not disturb an existing output"
    );
    assert_eq!(
        staged_leftovers(root),
        0,
        "failed flatten left a temporary file"
    );
}

#[test]
fn concurrent_flattens_never_mix_output() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();

    // Two papers with distinct, multi-line content flattened to the same
    // output path. Any staging-path collision shows up as mixed or truncated
    // bytes; correct staging yields exactly one paper's complete flatten.
    let mut mains = Vec::new();
    let mut expected = Vec::new();
    for index in 0..2 {
        let main = root.join(format!("paper-{index}/main.tex"));
        let body = (0..200).fold(String::new(), |mut body, line| {
            use std::fmt::Write as _;
            let _ = writeln!(body, "paper {index} line {line}");
            body
        });
        write_file(
            &main,
            &format!("\\documentclass{{article}}\n\\begin{{document}}\n{body}\\end{{document}}\n"),
        );

        let reference = root.join(format!("reference-{index}.tex"));
        flatten(&main, &[], &reference, &FlattenOptions::default()).expect("reference flatten");
        expected.push(fs::read(reference).expect("read reference"));
        mains.push(main);
    }

    let output = root.join("flat.tex");
    for _round in 0..20 {
        std::thread::scope(|scope| {
            for main in &mains {
                let output = output.clone();
                scope.spawn(move || {
                    flatten(main, &[], &output, &FlattenOptions::default())
                        .expect("concurrent flatten");
                });
            }
        });

        let bytes = fs::read(&output).expect("read output");
        assert!(
            expected.contains(&bytes),
            "concurrent flattens produced mixed output"
        );
    }

    assert_eq!(
        staged_leftovers(root),
        0,
        "concurrent flattens left a temporary file"
    );
}

#[test]
fn strict_bibliography_fails_on_missing_bib() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("paper/main.tex");
    write_file(
        &main,
        "\\documentclass{article}\n\\addbibresource{references.bib}\n\\begin{document}\n\\end{document}\n",
    );

    // Default mode: warning comment, successful flatten (bib not supplied).
    let output = root.join("flat.tex");
    flatten(&main, &[], &output, &FlattenOptions::default()).expect("lenient flatten succeeds");
    let text = fs::read_to_string(&output).expect("read output");
    assert!(text.contains("% WARNING: Bibliography file 'references.bib' not found"));

    // Strict (release) mode: hard failure, no output.
    let strict_output = root.join("strict.tex");
    let error = flatten(
        &main,
        &[],
        &strict_output,
        &FlattenOptions {
            strict_bibliography: true,
        },
    )
    .expect_err("strict flatten fails");
    assert!(error.to_string().contains("strict mode"), "{error}");
    assert!(!strict_output.exists());
}

#[test]
fn flatten_embeds_a_supplied_bibliography() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("paper/main.tex");
    let bib = root.join("paper/references.bib");
    write_file(
        &main,
        "\\documentclass{article}\n\\addbibresource{references.bib}\n\\begin{document}\n\\end{document}\n",
    );
    write_file(&bib, "@misc{key, title = {A Title}}\n");

    let output = root.join("flat.tex");
    flatten(
        &main,
        &[bib],
        &output,
        &FlattenOptions {
            strict_bibliography: true,
        },
    )
    .expect("a supplied bibliography is embedded");
    let text = fs::read_to_string(&output).expect("read output");
    assert!(text.contains("% --- BEGIN embedded bibliography from: references.bib ---"));
    assert!(text.contains("\\begin{filecontents*}{references.bib}"));
    assert!(text.contains("@misc{key, title = {A Title}}"));
    assert!(text.contains("\\addbibresource{references.bib}"));
}

// --- Path-confinement regressions (Finding 1 / ADR-015). The flattener does
// no directory lookup, so a reference can only ever name a file on the
// supplied list; an absolute path, a `..` escape, a real adjacent file, and a
// symlink target are all unreachable unless explicitly supplied. ---

#[test]
fn flatten_rejects_absolute_include() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("paper/main.tex");
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{/etc/passwd}\n\\end{document}\n",
    );

    let output = root.join("flat.tex");
    let error = flatten(&main, &[], &output, &FlattenOptions::default())
        .expect_err("an absolute include must be refused");
    assert!(error.to_string().contains("absolute path"), "{error}");
    assert!(!output.exists());
}

#[test]
fn flatten_rejects_parent_traversal() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("paper/main.tex");
    // A secret exists outside the paper tree; a `..` escape must not reach it.
    write_file(&root.join("outside-secret.tex"), "TOP SECRET\n");
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{../../outside-secret}\n\\end{document}\n",
    );

    let output = root.join("flat.tex");
    let error = flatten(&main, &[], &output, &FlattenOptions::default())
        .expect_err("a parent-directory escape must be refused");
    assert!(error.to_string().contains("parent-directory"), "{error}");
    assert!(!output.exists());
}

#[test]
fn flatten_rejects_unlisted_file_on_disk() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("paper/main.tex");
    // section.tex exists right next to main.tex, but is NOT on the allowlist.
    // A directory-based flattener would inline it; this one must refuse.
    write_file(&root.join("paper/section.tex"), "adjacent but unlisted\n");
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{section}\n\\end{document}\n",
    );

    let output = root.join("flat.tex");
    let error = flatten(&main, &[], &output, &FlattenOptions::default())
        .expect_err("a file not on the supplied list must be refused even if it exists on disk");
    assert!(
        error
            .to_string()
            .contains("does not match any file supplied"),
        "{error}"
    );
    assert!(!output.exists());
}

#[cfg(unix)]
#[test]
fn flatten_rejects_symlink_escape() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("paper/main.tex");
    let secret = root.join("outside-secret.tex");
    write_file(&secret, "TOP SECRET\n");
    // A symlink inside the paper tree pointing outside it. It is not on the
    // allowlist, so it is never followed.
    let link = root.join("paper/link.tex");
    fs::create_dir_all(link.parent().unwrap()).expect("mkdir");
    std::os::unix::fs::symlink(&secret, &link).expect("symlink");
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{link.tex}\n\\end{document}\n",
    );

    let output = root.join("flat.tex");
    let error = flatten(&main, &[], &output, &FlattenOptions::default())
        .expect_err("an unlisted symlink must be refused");
    assert!(
        error
            .to_string()
            .contains("does not match any file supplied"),
        "{error}"
    );
    assert!(!output.exists());
    let text = fs::read_to_string(&output).unwrap_or_default();
    assert!(!text.contains("TOP SECRET"));
}

#[test]
fn flatten_aborts_on_ambiguous_reference() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("paper/main.tex");
    // Two supplied files share the basename referenced by \input, and the
    // reference names only the basename — an ambiguous match must abort.
    let first = root.join("a/dup.tex");
    let second = root.join("b/dup.tex");
    write_file(&first, "first dup\n");
    write_file(&second, "second dup\n");
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{dup.tex}\n\\end{document}\n",
    );

    let output = root.join("flat.tex");
    let error = flatten(&main, &[first, second], &output, &FlattenOptions::default())
        .expect_err("an ambiguous reference must abort");
    assert!(error.to_string().contains("ambiguously matches"), "{error}");
    assert!(!output.exists());
}

#[test]
fn flatten_disambiguates_by_folder() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("paper/main.tex");
    // Two files share a basename, but the reference names a folder component,
    // so the match is unique: filename first, then folder.
    let a_dup = root.join("a/dup.tex");
    let b_dup = root.join("b/dup.tex");
    write_file(&a_dup, "from a\n");
    write_file(&b_dup, "from b\n");
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{b/dup.tex}\n\\end{document}\n",
    );

    let output = root.join("flat.tex");
    flatten(&main, &[a_dup, b_dup], &output, &FlattenOptions::default())
        .expect("a folder-qualified reference resolves uniquely");
    let text = fs::read_to_string(output).expect("read output");
    assert!(text.contains("from b"));
    assert!(!text.contains("from a"));
}

// --- F2-004 strict regular-file confinement: an allowlisted path must
// be an existing regular file; allowlisting never authorizes a
// symlink's resolved target, and aliases cannot re-enter an include. ---

#[cfg(unix)]
#[test]
fn flatten_rejects_allowlisted_symlink() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("paper/main.tex");
    let secret = root.join("outside-secret.tex");
    write_file(&secret, "TOP SECRET\n");
    // The symlink IS on the allowlist: the confinement claim is exactly
    // that listing the link path does not authorize its target.
    let link = root.join("paper/link.tex");
    fs::create_dir_all(link.parent().unwrap()).expect("mkdir");
    std::os::unix::fs::symlink(&secret, &link).expect("symlink");
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{link.tex}\n\\end{document}\n",
    );

    let output = root.join("flat.tex");
    let error = flatten(&main, &[link], &output, &FlattenOptions::default())
        .expect_err("an allowlisted symlink must be refused");
    assert!(error.to_string().contains("symbolic link"), "{error}");
    assert!(!output.exists());
}

#[cfg(unix)]
#[test]
fn flatten_rejects_symlinked_main() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let real = root.join("real-main.tex");
    write_file(
        &real,
        "\\documentclass{article}\n\\begin{document}\n\\end{document}\n",
    );
    let main = root.join("paper/main.tex");
    fs::create_dir_all(main.parent().unwrap()).expect("mkdir");
    std::os::unix::fs::symlink(&real, &main).expect("symlink");

    let output = root.join("flat.tex");
    let error = flatten(&main, &[], &output, &FlattenOptions::default())
        .expect_err("a symlinked main entry point must be refused");
    assert!(error.to_string().contains("symbolic link"), "{error}");
    assert!(!output.exists());
}

#[cfg(unix)]
#[test]
fn flatten_rejects_symlink_cycles_without_recursing() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let first = root.join("paper/a.tex");
    let second = root.join("paper/b.tex");
    fs::create_dir_all(first.parent().unwrap()).expect("mkdir");
    // A symlink loop: validation rejects the links by their own file
    // type before any open could chase the cycle.
    std::os::unix::fs::symlink(&second, &first).expect("symlink");
    std::os::unix::fs::symlink(&first, &second).expect("symlink");
    let main = root.join("paper/main.tex");
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{a.tex}\n\\end{document}\n",
    );

    let output = root.join("flat.tex");
    let error = flatten(&main, &[first, second], &output, &FlattenOptions::default())
        .expect_err("a symlink loop must be refused, not followed");
    assert!(error.to_string().contains("symbolic link"), "{error}");
    assert!(!output.exists());
}

#[cfg(unix)]
#[test]
fn flatten_detects_cycles_through_hard_link_aliases() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("paper/main.tex");
    // a.tex includes b.tex, and b.tex is a hard link to a.tex: two
    // allowlist entries, one regular file.
    let first = root.join("paper/a.tex");
    write_file(&first, "\\input{b.tex}\n");
    let second = root.join("paper/b.tex");
    fs::hard_link(&first, &second).expect("hard link");
    // Path equality is sufficient and no filesystem identity is
    // consulted (ADR-017): every frame is an allowlist entry, so the
    // chain main -> a -> b re-enters b on the next include and the
    // cycle closes one frame later than an inode comparison would.
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{a.tex}\n\\end{document}\n",
    );

    let output = root.join("flat.tex");
    let error = flatten(&main, &[first, second], &output, &FlattenOptions::default())
        .expect_err("a hard-link alias cycle must be detected");
    assert!(error.to_string().contains("include cycle"), "{error}");
    assert!(!output.exists());
}

// --- ADR-017: a supplied allowlist entry is a caller-granted
// capability. The flattener no longer walks its ancestors for
// symlinks. That walk could not see bind mounts, FUSE aliases, or a
// replacement between the check and the open, so it claimed a boundary
// the repository cannot hold. These two tests record the decided
// behaviour so it is not mistaken for a regression and restored. What
// still fails closed is the source-derived side: a reference that
// names no allowlist entry remains unreachable. ---

#[cfg(unix)]
#[test]
fn file_beneath_nested_symlinked_ancestors_is_a_caller_capability() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("paper/main.tex");
    // Two levels of symlinked directory between the paper tree and the
    // real file. The caller put this path on the allowlist, so the
    // caller granted read authority to whatever the host resolves it
    // to; the flattener inlines it.
    let outside = root.join("outside/deeper/section.tex");
    write_file(&outside, "SUPPLIED BODY\n");
    let first = root.join("paper/level-one");
    fs::create_dir_all(first.parent().unwrap()).expect("mkdir");
    std::os::unix::fs::symlink(root.join("outside"), &first).expect("symlink");
    let listed = first.join("deeper/section.tex");
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{section.tex}\n\\end{document}\n",
    );

    let output = root.join("flat.tex");
    flatten(&main, &[listed], &output, &FlattenOptions::default())
        .expect("an allowlisted path is a caller-granted capability");

    let text = fs::read_to_string(&output).expect("flattened output");
    assert!(text.contains("SUPPLIED BODY"), "{text}");
}

#[cfg(unix)]
#[test]
fn main_beneath_symlinked_ancestor_is_accepted() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let real_main = root.join("real/main.tex");
    write_file(
        &real_main,
        "\\documentclass{article}\n\\begin{document}\n\\end{document}\n",
    );
    let alias = root.join("paper");
    std::os::unix::fs::symlink(root.join("real"), &alias).expect("symlink");
    let main = alias.join("main.tex");

    let output = root.join("flat.tex");
    flatten(&main, &[], &output, &FlattenOptions::default())
        .expect("the entry point is an explicit caller-supplied path");
    assert!(output.exists());
}

#[cfg(unix)]
#[test]
fn a_reference_off_the_allowlist_remains_unreachable_beneath_a_symlink() {
    // The confinement that survives: source text selects only what the
    // caller supplied. A symlinked directory inside the paper tree does
    // not widen the allowlist, because resolution never touches the
    // filesystem.
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("paper/main.tex");
    let outside = root.join("outside/secret.tex");
    write_file(&outside, "TOP SECRET\n");
    let alias = root.join("paper/reachable");
    fs::create_dir_all(alias.parent().unwrap()).expect("mkdir");
    std::os::unix::fs::symlink(root.join("outside"), &alias).expect("symlink");
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{secret.tex}\n\\end{document}\n",
    );

    let output = root.join("flat.tex");
    let error = flatten(&main, &[], &output, &FlattenOptions::default())
        .expect_err("a reference matching no supplied file must fail");
    assert!(
        error.to_string().contains("does not match any file"),
        "{error}"
    );
    assert!(!output.exists());
    let text = fs::read_to_string(&output).unwrap_or_default();
    assert!(!text.contains("TOP SECRET"));
}

#[cfg(unix)]
#[test]
fn failed_confinement_preserves_output_and_stages_nothing() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("paper/main.tex");
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\end{document}\n",
    );
    let secret = root.join("outside-secret.tex");
    write_file(&secret, "TOP SECRET\n");
    let link = root.join("paper/link.tex");
    std::os::unix::fs::symlink(&secret, &link).expect("symlink");

    let output = root.join("out/flat.tex");
    write_file(&output, "previous good output\n");

    let error = flatten(&main, &[link], &output, &FlattenOptions::default())
        .expect_err("confinement failure");
    assert!(error.to_string().contains("symbolic link"), "{error}");

    // The prior output survives byte-for-byte and validation failed
    // before staging, so no staging remnant exists anywhere under the
    // output directory.
    assert_eq!(
        fs::read_to_string(&output).expect("existing output"),
        "previous good output\n",
    );
    let staged: Vec<_> = fs::read_dir(output.parent().unwrap())
        .expect("output dir")
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with(".flatten-staged-")
        })
        .collect();
    assert!(staged.is_empty(), "{staged:?}");
}
