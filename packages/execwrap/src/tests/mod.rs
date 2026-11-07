//! # `execwrap` library tests
//!
//! | Test                                                | What it covers                          |
//! |-----------------------------------------------------|-----------------------------------------|
//! | `routing_config_detects_combined_redirects`         | `--redirect` triggers both streams      |
//! | `routing_config_detects_single_stream_redirects`    | per-stream redirects only flag one      |
//! | `collect_file_specs_rejects_ambiguous_routing`      | conflicting paths are an error          |
//! | `aliased_redirection_paths_are_rejected`            | `a.log` vs `./a.log` conflict           |
//! | `symlink_and_hardlink_aliases_are_rejected`         | link aliases resolve to one identity    |
//! | `two_routes_to_one_log_are_refused_because_they_would_interleave` | the documented ADR-017 local exception |
//! | `parent_symlink_alias_is_rejected`                  | symlinked parent dirs resolve           |
//! | `symlinked_ancestor_with_missing_directories_is_rejected` | pending suffixes share identity   |
//! | `parent_dir_through_missing_component_still_resolves_symlinks` | `..` re-enters resolution    |
//! | `missing_directories_under_distinct_ancestors_validate` | distinct pending targets accepted  |
//! | `dangling_symlink_alias_is_rejected`                | dangling link aliases its target        |
//! | `dangling_symlink_chain_resolves_to_final_target`   | chains resolve to the last target       |
//! | `symlink_loop_is_an_error`                          | link loops error instead of hanging     |
//! | `distinct_redirection_targets_validate`             | different files remain accepted         |
//! | `ambiguous_alias_does_not_truncate_existing_output` | usage failure precedes truncation       |
//! | `dangling_alias_failure_leaves_target_intact`       | preflight only stats, never creates     |
//! | `typesetter_passes_data_through`                    | raw mode writes bytes unchanged         |
//! | `typesetter_preserves_arbitrary_bytes`              | raw mode is byte-exact on binary data   |
//! | `calligraphic_adds_prefix_per_line`                 | prefix mode formats per-line            |
//! | `calligraphic_sanitizes_control_bytes`              | sanitization is calligraphic-only       |
//! | `calligraphic_buffers_partial_lines_until_finalize` | partial lines flush at finalize         |
//! | `calligraphic_preserves_code_point_split_across_two_writes` | chunked multibyte UTF-8 survives |
//! | `calligraphic_preserves_code_point_split_across_three_writes` | byte-at-a-time UTF-8 survives  |
//! | `calligraphic_replaces_invalid_utf8_after_buffering` | genuinely invalid bytes still map to `\u{fffd}` |
//! | `calligraphic_output_is_chunking_invariant`         | same bytes, any chunking, same output   |
//! | `routing_preflight_capture_preserves_control_and_refusal` | scoped routing span and DEBUG refusal |
//! | `run_capture_preserves_result_bytes_and_refusal`    | scoped child span, typed completion, unchanged bytes |
//! | `child_stream_capture_propagates_into_readers_and_preserves_bytes` | worker dispatcher and stream totals |
//! | `log_notification_events_preserve_callbacks_and_assets` | path-free notices and unchanged callbacks |
//! | `writer_finalize_capture_preserves_success_and_failure_bytes` | finalize outcomes without byte changes |
//! | `mock_capture_preserves_assets_and_typed_failure`   | mock completion and fixed failure kind |
//! | `tracing_name_vocabulary_is_snake_case`             | all categorical trace names and enum mappings |

use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::prelude::*;

use super::writer::{StreamProps, Writer};
use super::{ExecError, RoutingConfig, Stream, collect_file_specs, file_identity};

#[derive(Clone, Debug)]
struct TraceRecord {
    target: &'static str,
    level: Level,
    name: &'static str,
    fields: std::collections::BTreeMap<String, String>,
}

#[derive(Default)]
struct TraceFields(std::collections::BTreeMap<String, String>);

impl Visit for TraceFields {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.0
            .insert(field.name().to_string(), format!("{value:?}"));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.0.insert(field.name().to_string(), value.to_string());
    }
}

struct CaptureLayer(Arc<Mutex<Vec<TraceRecord>>>);

static CAPTURE_INSTALL_LOCK: Mutex<()> = Mutex::new(());
const CAPTURE_CHILD_MARKER: &str = "TRIPOD_EXECWRAP_CAPTURE_CHILD";

impl<S: Subscriber> Layer<S> for CaptureLayer {
    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        _id: &tracing::span::Id,
        _ctx: Context<'_, S>,
    ) {
        let mut fields = TraceFields::default();
        attrs.record(&mut fields);
        let metadata = attrs.metadata();
        self.0.lock().expect("capture lock").push(TraceRecord {
            target: metadata.target(),
            level: *metadata.level(),
            name: metadata.name(),
            fields: fields.0,
        });
    }

    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut fields = TraceFields::default();
        event.record(&mut fields);
        let metadata = event.metadata();
        self.0.lock().expect("capture lock").push(TraceRecord {
            target: metadata.target(),
            level: *metadata.level(),
            name: "event",
            fields: fields.0,
        });
    }
}

fn with_capture<T>(run: impl FnOnce() -> T) -> (T, Vec<TraceRecord>) {
    let _install_guard = CAPTURE_INSTALL_LOCK.lock().expect("capture install lock");
    let records = Arc::new(Mutex::new(Vec::new()));
    let subscriber = tracing_subscriber::registry().with(CaptureLayer(Arc::clone(&records)));
    let value = tracing::subscriber::with_default(subscriber, || {
        tracing::callsite::rebuild_interest_cache();
        run()
    });
    let events = records.lock().expect("capture lock").clone();
    (value, events)
}

fn isolate_capture_test(test_name: &str) -> bool {
    if std::env::var_os(CAPTURE_CHILD_MARKER).is_some() {
        return false;
    }
    let output = std::process::Command::new(std::env::current_exe().expect("test executable"))
        .arg("--exact")
        .arg(test_name)
        .arg("--nocapture")
        .env(CAPTURE_CHILD_MARKER, "1")
        .output()
        .expect("run isolated capture test");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let passed = stdout
        .lines()
        .filter(|line| line.starts_with("test result: ok. 1 passed; 0 failed;"))
        .count();
    assert!(
        output.status.success() && passed == 1,
        "isolated capture test {test_name} failed or selected the wrong tests\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    true
}

macro_rules! isolated_capture {
    ($name:ident) => {
        if isolate_capture_test(concat!("tests::", stringify!($name))) {
            return;
        }
    };
}

fn event<'a>(records: &'a [TraceRecord], target: &str, message: &str) -> &'a TraceRecord {
    records
        .iter()
        .find(|record| {
            record.target == target
                && record.name == "event"
                && record
                    .fields
                    .get("message")
                    .is_some_and(|value| value.contains(message))
        })
        .expect("expected trace event")
}

fn span<'a>(records: &'a [TraceRecord], target: &str, name: &str) -> &'a TraceRecord {
    records
        .iter()
        .find(|record| record.target == target && record.name == name)
        .expect("expected trace span")
}

fn no_high_severity(records: &[TraceRecord]) {
    assert!(
        records
            .iter()
            .all(|record| record.level != Level::WARN && record.level != Level::ERROR)
    );
}

#[test]
fn tracing_name_vocabulary_is_snake_case() {
    let variant_names = [
        (Stream::Stdout.trace_label(), "stdout"),
        (Stream::Stderr.trace_label(), "stderr"),
        (super::RouteKind::CombinedRaw.trace_label(), "combined_raw"),
        (
            super::RouteKind::CombinedPrefixed.trace_label(),
            "combined_prefixed",
        ),
        (super::RouteKind::StdoutOnly.trace_label(), "stdout_only"),
        (super::RouteKind::StderrOnly.trace_label(), "stderr_only"),
        (super::MockChild::Xelatex.trace_label(), "xelatex"),
        (super::MockChild::Biber.trace_label(), "biber"),
        (super::MockChild::Latexmk.trace_label(), "latexmk"),
    ];
    for (name, expected) in variant_names {
        assert_eq!(name, expected);
        assert_snake_case_name(name);
    }

    for name in [
        "completed",
        "refused",
        "failed",
        "empty_command",
        "ambiguous_redirection",
        "unrouted_stream",
        "io",
        "spawn",
        "reader_panic",
        "read",
        "write",
        "finalize",
        "raw",
        "prefixed",
        "routing",
        "setup",
        "child",
        "mock",
        "change_directory",
        "data_loss",
        "child_exit",
        "injected_failure",
        "child_status",
        "asset",
        "success",
        "failure",
        "usage",
    ] {
        assert_snake_case_name(name);
    }
}

fn assert_snake_case_name(name: &str) {
    assert!(
        name.bytes()
            .next()
            .is_some_and(|first| first.is_ascii_lowercase())
            && name.split('_').all(|part| {
                !part.is_empty()
                    && part
                        .bytes()
                        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
            }),
        "trace name does not match lower snake case: {name}"
    );
}

#[test]
fn routing_preflight_capture_preserves_control_and_refusal() {
    isolated_capture!(routing_preflight_capture_preserves_control_and_refusal);
    let dir = tempfile::tempdir().expect("tempdir");
    let output = dir.path().join("out.log");
    let control = RoutingConfig {
        redirect: Some(output.clone()),
        ..RoutingConfig::default()
    };
    let refusal = RoutingConfig {
        redirect_output: Some(output),
        ..control.clone()
    };
    assert!(super::preflight_routing(&control).is_ok());
    let (accepted, accepted_records) = with_capture(|| super::preflight_routing(&control));
    assert!(accepted.is_ok());
    let accepted_event = event(
        &accepted_records,
        "execwrap::routing",
        "routing preflight completed",
    );
    assert_eq!(
        accepted_event.fields.get("outcome").map(String::as_str),
        Some("completed")
    );
    assert_eq!(
        span(&accepted_records, "execwrap::routing", "routing_preflight").level,
        Level::DEBUG
    );
    assert!(matches!(
        super::preflight_routing(&refusal),
        Err(ExecError::AmbiguousRedirection { .. })
    ));
    let (rejected, rejected_records) = with_capture(|| super::preflight_routing(&refusal));
    assert!(matches!(
        rejected,
        Err(ExecError::AmbiguousRedirection { .. })
    ));
    let refused_event = event(
        &rejected_records,
        "execwrap::routing",
        "routing preflight completed",
    );
    assert_eq!(refused_event.level, Level::DEBUG);
    assert_eq!(
        refused_event.fields.get("outcome").map(String::as_str),
        Some("refused")
    );
    assert_eq!(
        refused_event.fields.get("refusal_kind").map(String::as_str),
        Some("ambiguous_redirection")
    );
    no_high_severity(&rejected_records);
    assert!(!dir.path().join("out.log").exists());
}

#[test]
fn unrouted_preflight_capture_refuses_before_side_effects() {
    isolated_capture!(unrouted_preflight_capture_refuses_before_side_effects);
    let dir = tempfile::tempdir().expect("tempdir");
    let log = dir.path().join("both.log");
    let control = RoutingConfig {
        redirect: Some(log.clone()),
        ..RoutingConfig::default()
    };
    assert!(super::preflight_routing(&control).is_ok());
    let (accepted, accepted_records) = with_capture(|| super::preflight_routing(&control));
    assert!(accepted.is_ok());
    assert_eq!(
        event(
            &accepted_records,
            "execwrap::routing",
            "routing preflight completed"
        )
        .fields
        .get("outcome")
        .map(String::as_str),
        Some("completed")
    );
    let refusal = RoutingConfig::default();
    assert!(matches!(
        super::preflight_routing(&refusal),
        Err(ExecError::UnroutedStream {
            stream: Stream::Stdout
        })
    ));
    let (rejected, rejected_records) = with_capture(|| super::preflight_routing(&refusal));
    assert!(matches!(
        rejected,
        Err(ExecError::UnroutedStream {
            stream: Stream::Stdout
        })
    ));
    let refused = event(
        &rejected_records,
        "execwrap::routing",
        "routing preflight completed",
    );
    assert_eq!(refused.level, Level::DEBUG);
    assert_eq!(
        refused.fields.get("outcome").map(String::as_str),
        Some("refused")
    );
    assert_eq!(
        refused.fields.get("refusal_kind").map(String::as_str),
        Some("unrouted_stream")
    );
    no_high_severity(&rejected_records);
    assert!(!log.exists());

    let output_only = RoutingConfig {
        redirect_output: Some(log),
        ..RoutingConfig::default()
    };
    assert!(matches!(
        super::preflight_routing(&output_only),
        Err(ExecError::UnroutedStream {
            stream: Stream::Stderr
        })
    ));
}

#[test]
fn capture_rebuilds_interest_after_uncaptured_call_on_two_threads() {
    isolated_capture!(capture_rebuilds_interest_after_uncaptured_call_on_two_threads);
    let dir = tempfile::tempdir().expect("tempdir");
    let control = RoutingConfig {
        redirect: Some(dir.path().join("out.log")),
        ..RoutingConfig::default()
    };
    assert!(super::preflight_routing(&control).is_ok());
    let handles: Vec<_> = (0..2)
        .map(|_| {
            let control = control.clone();
            std::thread::spawn(move || {
                let (result, records) = with_capture(|| super::preflight_routing(&control));
                assert!(result.is_ok());
                assert_eq!(
                    records
                        .iter()
                        .filter(|record| {
                            record.target == "execwrap::routing"
                                && record.name == "routing_preflight"
                        })
                        .count(),
                    1
                );
            })
        })
        .collect();
    for handle in handles {
        handle.join().expect("capture thread");
    }
}

#[test]
fn run_capture_preserves_result_bytes_and_refusal() {
    isolated_capture!(run_capture_preserves_result_bytes_and_refusal);
    let dir = tempfile::tempdir().expect("tempdir");
    let output = dir.path().join("out.log");
    let control = RoutingConfig {
        redirect: Some(output.clone()),
        ..RoutingConfig::default()
    };
    let command = [
        std::ffi::OsString::from("sh"),
        std::ffi::OsString::from("-c"),
        std::ffi::OsString::from("printf 'output\\n'"),
    ];
    let plain = super::run(&command, &control, |_| {}).expect("plain run");
    let plain_bytes = std::fs::read(&output).expect("read output");
    let (observed, observed_records) = with_capture(|| super::run(&command, &control, |_| {}));
    let observed = observed.expect("observed run");
    assert_eq!(
        (plain.exit_code, plain.data_loss),
        (observed.exit_code, observed.data_loss)
    );
    assert_eq!(plain_bytes, std::fs::read(&output).expect("read output"));
    assert_eq!(plain_bytes, b"output\n");
    assert!(
        observed_records
            .iter()
            .any(|record| record.name == "child_run" && record.target == "execwrap::run")
    );
    let completed = event(&observed_records, "execwrap::run", "child completed");
    assert_eq!(
        completed.fields.get("outcome").map(String::as_str),
        Some("completed")
    );
    let started = event(&observed_records, "execwrap::run", "child started");
    assert_eq!(
        started.fields.get("argument_count").map(String::as_str),
        Some("2")
    );

    let refusal = RoutingConfig {
        redirect_output: Some(output.clone()),
        ..control
    };
    assert!(matches!(
        super::run(&command, &refusal, |_| {}),
        Err(ExecError::AmbiguousRedirection { .. })
    ));
    let before = std::fs::read(&output).expect("read output");
    let (rejected, rejected_records) = with_capture(|| super::run(&command, &refusal, |_| {}));
    assert!(matches!(
        rejected,
        Err(ExecError::AmbiguousRedirection { .. })
    ));
    assert_eq!(std::fs::read(&output).expect("read output"), before);
    let refused = event(&rejected_records, "execwrap::run", "child completed");
    assert_eq!(refused.level, Level::DEBUG);
    assert_eq!(
        refused.fields.get("outcome").map(String::as_str),
        Some("refused")
    );
    assert_eq!(
        refused.fields.get("refusal_kind").map(String::as_str),
        Some("ambiguous_redirection")
    );
    no_high_severity(&rejected_records);
}

#[test]
fn routing_config_detects_combined_redirects() {
    let cfg = RoutingConfig {
        redirect: Some(PathBuf::from("a.log")),
        ..RoutingConfig::default()
    };
    assert!(cfg.is_stream_redirected(Stream::Stdout));
    assert!(cfg.is_stream_redirected(Stream::Stderr));
}

#[test]
fn routing_config_detects_single_stream_redirects() {
    let cfg = RoutingConfig {
        redirect_output: Some(PathBuf::from("out.log")),
        ..RoutingConfig::default()
    };
    assert!(cfg.is_stream_redirected(Stream::Stdout));
    assert!(!cfg.is_stream_redirected(Stream::Stderr));
}

#[test]
fn collect_file_specs_rejects_ambiguous_routing() {
    let cfg = RoutingConfig {
        redirect: Some(PathBuf::from("a.log")),
        redirect_output: Some(PathBuf::from("a.log")),
        ..RoutingConfig::default()
    };
    let error = collect_file_specs(&cfg).expect_err("expected ambiguity");
    assert!(matches!(error, ExecError::AmbiguousRedirection { .. }));
}

#[test]
fn aliased_redirection_paths_are_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");

    let cfg = RoutingConfig {
        redirect: Some(dir.path().join("a.log")),
        redirect_output: Some(dir.path().join("./a.log")),
        ..RoutingConfig::default()
    };

    assert!(matches!(
        super::preflight_routing(&cfg),
        Err(ExecError::AmbiguousRedirection { .. }),
    ));
}

#[test]
fn symlink_and_hardlink_aliases_are_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");

    let target = dir.path().join("a.log");
    std::fs::write(&target, b"keep").expect("write target");

    let symlink = dir.path().join("sym.log");
    std::os::unix::fs::symlink(&target, &symlink).expect("symlink");

    let symlinked = RoutingConfig {
        redirect: Some(target.clone()),
        redirect_output: Some(symlink),
        ..RoutingConfig::default()
    };

    assert!(matches!(
        super::preflight_routing(&symlinked),
        Err(ExecError::AmbiguousRedirection { .. }),
    ));

    let hard_link = dir.path().join("hard.log");
    std::fs::hard_link(&target, &hard_link).expect("hard link");

    let hard_linked = RoutingConfig {
        redirect: Some(target.clone()),
        redirect_error: Some(hard_link),
        ..RoutingConfig::default()
    };

    assert!(matches!(
        super::preflight_routing(&hard_linked),
        Err(ExecError::AmbiguousRedirection { .. }),
    ));

    // Preflight only stats: the existing target is untouched.
    assert_eq!(std::fs::read(&target).expect("read target"), b"keep");
}

#[test]
fn two_routes_to_one_log_are_refused_because_they_would_interleave() {
    // ADR-017 records this package as the sole local exception to
    // lexical output-role comparison, and the hazard is what justifies
    // it: two routes resolving to one file are opened independently,
    // each truncating, then written concurrently as the child produces
    // output. The log ends up interleaved and partially overwritten
    // while every I/O call succeeds.
    //
    // Lexical comparison cannot see this pair — the two names differ
    // in every component and neither is a `.`/`..` spelling of the
    // other — so refusing it needs the resolved identity.
    let dir = tempfile::tempdir().expect("tempdir");
    let stdout_log = dir.path().join("stdout.log");
    std::fs::write(&stdout_log, b"previous run").expect("write log");
    let stderr_log = dir.path().join("stderr.log");
    std::fs::hard_link(&stdout_log, &stderr_log).expect("hard link");

    let interleaving = RoutingConfig {
        redirect_output: Some(stdout_log.clone()),
        redirect_error: Some(stderr_log),
        ..RoutingConfig::default()
    };

    assert!(matches!(
        super::preflight_routing(&interleaving),
        Err(ExecError::AmbiguousRedirection { .. }),
    ));

    // The refusal precedes truncation, so the previous log survives.
    // Nothing here closes a host race: the check only precedes the
    // open, it does not hold the path still.
    assert_eq!(
        std::fs::read(&stdout_log).expect("read log"),
        b"previous run"
    );
}

#[test]
fn parent_symlink_alias_is_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");

    let real = dir.path().join("real");
    std::fs::create_dir(&real).expect("create real dir");

    let alias = dir.path().join("alias");
    std::os::unix::fs::symlink(&real, &alias).expect("symlink dir");

    // The target file does not exist yet; only the parents alias.
    let cfg = RoutingConfig {
        redirect: Some(real.join("x.log")),
        redirect_output: Some(alias.join("x.log")),
        ..RoutingConfig::default()
    };

    assert!(matches!(
        super::preflight_routing(&cfg),
        Err(ExecError::AmbiguousRedirection { .. }),
    ));
}

#[test]
fn symlinked_ancestor_with_missing_directories_is_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");

    let real = dir.path().join("real");
    std::fs::create_dir(&real).expect("create real dir");

    let alias = dir.path().join("alias");
    std::os::unix::fs::symlink(&real, &alias).expect("symlink dir");

    // Two missing directory levels below the symlinked ancestor:
    // canonicalizing the complete parent fails on both routes, so a
    // lexical fallback would accept them as distinct identities and
    // open one file under two routing kinds.
    let cfg = RoutingConfig {
        redirect: Some(real.join("new/sub/x.log")),
        redirect_output: Some(alias.join("new/sub/x.log")),
        ..RoutingConfig::default()
    };

    assert!(matches!(
        super::preflight_routing(&cfg),
        Err(ExecError::AmbiguousRedirection { .. }),
    ));

    // Rejection happens before any directory or file is created.
    assert!(!real.join("new").exists());
}

#[test]
fn parent_dir_through_missing_component_still_resolves_symlinks() {
    let dir = tempfile::tempdir().expect("tempdir");

    let real = dir.path().join("real");
    std::fs::create_dir(&real).expect("create real dir");

    let alias = dir.path().join("alias");
    std::os::unix::fs::symlink(&real, &alias).expect("symlink dir");

    // `newdir` does not exist; the `..` climbs back out of the missing
    // suffix, so the following symlinked component must be resolved
    // through the filesystem again, not appended lexically.
    let cfg = RoutingConfig {
        redirect: Some(dir.path().join("newdir/../alias/out.txt")),
        redirect_output: Some(real.join("out.txt")),
        ..RoutingConfig::default()
    };

    assert!(matches!(
        super::preflight_routing(&cfg),
        Err(ExecError::AmbiguousRedirection { .. }),
    ));
}

#[test]
fn missing_directories_under_distinct_ancestors_validate() {
    let dir = tempfile::tempdir().expect("tempdir");

    let first = dir.path().join("first");
    let second = dir.path().join("second");
    std::fs::create_dir(&first).expect("create first dir");
    std::fs::create_dir(&second).expect("create second dir");

    let cfg = RoutingConfig {
        redirect_output: Some(first.join("new/sub/x.log")),
        redirect_error: Some(second.join("new/sub/x.log")),
        ..RoutingConfig::default()
    };

    let specs = collect_file_specs(&cfg).expect("distinct pending targets are valid");
    assert_eq!(specs.len(), 2);
}

#[test]
fn dangling_symlink_alias_is_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");

    // sym.log dangles: real.log does not exist. File::create on the
    // link would still create/truncate real.log, so the two paths are
    // one identity.
    let real = dir.path().join("real.log");
    let symlink = dir.path().join("sym.log");
    std::os::unix::fs::symlink(&real, &symlink).expect("symlink");

    let cfg = RoutingConfig {
        redirect_output: Some(symlink.clone()),
        redirect_error: Some(real),
        ..RoutingConfig::default()
    };

    assert!(matches!(
        super::preflight_routing(&cfg),
        Err(ExecError::AmbiguousRedirection { .. }),
    ));

    // A dangling link on its own is still a valid single target.
    let alone = RoutingConfig {
        redirect_output: Some(symlink),
        ..RoutingConfig::default()
    };
    let specs = collect_file_specs(&alone).expect("single dangling target is valid");
    assert_eq!(specs.len(), 1);
}

#[test]
fn dangling_symlink_chain_resolves_to_final_target() {
    let dir = tempfile::tempdir().expect("tempdir");

    let real = dir.path().join("real.log");
    let sym1 = dir.path().join("sym1.log");
    let sym2 = dir.path().join("sym2.log");
    std::os::unix::fs::symlink(&real, &sym1).expect("symlink sym1");
    std::os::unix::fs::symlink(&sym1, &sym2).expect("symlink sym2");

    assert_eq!(
        file_identity(&sym2).expect("chain identity"),
        file_identity(&real).expect("target identity"),
        "a dangling chain must share the final target's identity"
    );
}

#[test]
fn symlink_loop_is_an_error() {
    let dir = tempfile::tempdir().expect("tempdir");

    let loop_a = dir.path().join("loop_a.log");
    let loop_b = dir.path().join("loop_b.log");
    std::os::unix::fs::symlink(&loop_a, &loop_b).expect("symlink loop_b");
    std::os::unix::fs::symlink(&loop_b, &loop_a).expect("symlink loop_a");

    assert!(
        file_identity(&loop_a).is_err(),
        "a symlink loop must error, not hang"
    );

    let cfg = RoutingConfig {
        redirect: Some(loop_a),
        ..RoutingConfig::default()
    };
    assert!(matches!(
        super::preflight_routing(&cfg),
        Err(ExecError::Io(_)),
    ));
}

#[test]
fn distinct_redirection_targets_validate() {
    let dir = tempfile::tempdir().expect("tempdir");

    let cfg = RoutingConfig {
        redirect_output: Some(dir.path().join("out.log")),
        redirect_error: Some(dir.path().join("err.log")),
        ..RoutingConfig::default()
    };

    super::preflight_routing(&cfg).expect("distinct targets are valid");
}

#[test]
fn ambiguous_alias_does_not_truncate_existing_output() {
    let dir = tempfile::tempdir().expect("tempdir");

    let target = dir.path().join("a.log");
    std::fs::write(&target, b"previous log contents").expect("write target");

    let symlink = dir.path().join("sym.log");
    std::os::unix::fs::symlink(&target, &symlink).expect("symlink");

    let cfg = RoutingConfig {
        redirect: Some(target.clone()),
        redirect_output: Some(symlink),
        ..RoutingConfig::default()
    };

    let command = [std::ffi::OsString::from("true")];

    assert!(matches!(
        super::run(&command, &cfg, |_notification| {}),
        Err(ExecError::AmbiguousRedirection { .. }),
    ));

    assert_eq!(
        std::fs::read(&target).expect("read target"),
        b"previous log contents",
        "usage failure must precede any truncation"
    );
}

#[test]
fn dangling_alias_failure_leaves_target_intact() {
    let dir = tempfile::tempdir().expect("tempdir");

    let real = dir.path().join("real.log");
    let symlink = dir.path().join("sym.log");
    std::os::unix::fs::symlink(&real, &symlink).expect("symlink");

    let cfg = RoutingConfig {
        redirect_output: Some(symlink),
        redirect_error: Some(real.clone()),
        ..RoutingConfig::default()
    };

    // Preflight only stats: rejecting the dangling alias must not
    // create the missing target.
    assert!(matches!(
        super::preflight_routing(&cfg),
        Err(ExecError::AmbiguousRedirection { .. }),
    ));
    assert!(!real.exists(), "preflight must not create the target");

    // Once the target exists the alias is still rejected, and its
    // contents survive both preflight and a full run attempt.
    std::fs::write(&real, b"previous log contents").expect("write target");

    assert!(matches!(
        super::preflight_routing(&cfg),
        Err(ExecError::AmbiguousRedirection { .. }),
    ));

    let command = [std::ffi::OsString::from("true")];
    assert!(matches!(
        super::run(&command, &cfg, |_notification| {}),
        Err(ExecError::AmbiguousRedirection { .. }),
    ));

    assert_eq!(
        std::fs::read(&real).expect("read target"),
        b"previous log contents",
        "usage failure must precede any truncation"
    );
}

struct SharedSink {
    inner: Arc<Mutex<Vec<u8>>>,
}

impl Write for SharedSink {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut guard = self.inner.lock().expect("test sink lock");
        Write::write(&mut *guard, buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn make_writer(
    add_prefix: bool,
    props: StreamProps,
    wrap_length: usize,
) -> (Writer, Arc<Mutex<Vec<u8>>>) {
    let buffer = Arc::new(Mutex::new(Vec::<u8>::new()));
    let shared = Arc::clone(&buffer);
    let sink = SharedSink { inner: shared };
    let writer = Writer::new(sink, add_prefix, wrap_length, props, false);
    (writer, buffer)
}

#[test]
fn typesetter_passes_data_through() {
    let (mut writer, buffer) = make_writer(false, StreamProps::default(), 80);
    writer
        .write(Stream::Stdout, b"hello\nworld\n")
        .expect("write");
    writer.finalize().expect("finalize");
    let written = buffer.lock().expect("lock").clone();
    assert_eq!(&*written, b"hello\nworld\n");
}

#[test]
fn typesetter_preserves_arbitrary_bytes() {
    // Raw mode is byte-exact: invalid UTF-8, NUL, control bytes, and
    // large chunks pass through unchanged.
    let (mut writer, buffer) = make_writer(false, StreamProps::default(), 80);

    let mut payload: Vec<u8> = vec![0xff, 0xfe, 0x00, 0x01, 0x1b, b'[', b'3', b'1', b'm'];
    payload.extend(std::iter::repeat_n(0xa5_u8, 64 * 1024));
    payload.push(b'\n');

    writer.write(Stream::Stdout, &payload).expect("write");
    writer.write(Stream::Stderr, &payload).expect("write");
    writer.finalize().expect("finalize");

    let written = buffer.lock().expect("lock").clone();
    let mut expected = payload.clone();
    expected.extend_from_slice(&payload);
    assert_eq!(written, expected);
}

#[test]
fn calligraphic_sanitizes_control_bytes() {
    let props = StreamProps {
        line1_prefix_stdout: ">>> ".to_string(),
        ..StreamProps::default()
    };
    let (mut writer, buffer) = make_writer(true, props, 80);
    writer
        .write(Stream::Stdout, b"a\x00b\xffc\n")
        .expect("write");
    writer.finalize().expect("finalize");
    let written = String::from_utf8(buffer.lock().expect("lock").clone()).expect("utf8");
    assert_eq!(written, ">>> a?b\u{fffd}c\n");
}

#[test]
fn calligraphic_adds_prefix_per_line() {
    let props = StreamProps {
        line1_prefix_stdout: ">>> ".to_string(),
        wrap_prefix_stdout: "    ".to_string(),
        ..StreamProps::default()
    };
    let (mut writer, buffer) = make_writer(true, props, 80);
    writer
        .write(Stream::Stdout, b"line one\nline two\n")
        .expect("write");
    writer.finalize().expect("finalize");
    let written = String::from_utf8(buffer.lock().expect("lock").clone()).expect("utf8");
    assert_eq!(written, ">>> line one\n>>> line two\n");
}

#[test]
fn calligraphic_buffers_partial_lines_until_finalize() {
    let props = StreamProps {
        line1_prefix_stdout: ">>> ".to_string(),
        ..StreamProps::default()
    };
    let (mut writer, buffer) = make_writer(true, props, 80);
    writer.write(Stream::Stdout, b"partial").expect("write");
    assert!(buffer.lock().expect("lock").is_empty());
    writer.finalize().expect("finalize");
    let written = String::from_utf8(buffer.lock().expect("lock").clone()).expect("utf8");
    assert_eq!(written, ">>> partial\n");
}

fn prefix_props() -> StreamProps {
    StreamProps {
        line1_prefix_stdout: ">>> ".to_string(),
        ..StreamProps::default()
    }
}

#[test]
fn calligraphic_preserves_code_point_split_across_two_writes() {
    // "€\n" is E2 82 AC 0A; a pipe read may deliver any split of it.
    let (mut writer, buffer) = make_writer(true, prefix_props(), 80);
    writer.write(Stream::Stdout, &[0xe2]).expect("write");
    writer
        .write(Stream::Stdout, &[0x82, 0xac, b'\n'])
        .expect("write");
    writer.finalize().expect("finalize");
    let written = String::from_utf8(buffer.lock().expect("lock").clone()).expect("utf8");
    assert_eq!(written, ">>> \u{20ac}\n");
}

#[test]
fn calligraphic_preserves_code_point_split_across_three_writes() {
    let (mut writer, buffer) = make_writer(true, prefix_props(), 80);
    for byte in [0xe2_u8, 0x82, 0xac, b'\n'] {
        writer.write(Stream::Stdout, &[byte]).expect("write");
    }
    writer.finalize().expect("finalize");
    let written = String::from_utf8(buffer.lock().expect("lock").clone()).expect("utf8");
    assert_eq!(written, ">>> \u{20ac}\n");
}

#[test]
fn calligraphic_replaces_invalid_utf8_after_buffering() {
    // A lone continuation byte and a truncated sequence at end of
    // stream are genuinely invalid and must still become U+FFFD.
    let (mut writer, buffer) = make_writer(true, prefix_props(), 80);
    writer.write(Stream::Stdout, &[b'a', 0xac]).expect("write");
    writer.write(Stream::Stdout, b"b\n").expect("write");
    writer.write(Stream::Stdout, &[0xe2, 0x82]).expect("write");
    writer.finalize().expect("finalize");
    let written = String::from_utf8(buffer.lock().expect("lock").clone()).expect("utf8");
    assert_eq!(written, ">>> a\u{fffd}b\n>>> \u{fffd}\n");
}

#[test]
fn calligraphic_output_is_chunking_invariant() {
    // The same byte stream must render identically no matter how the
    // OS chunks the pipe reads.
    let payload = "una línea con € y 日本語\nsecond line\ntail without newline"
        .as_bytes()
        .to_vec();

    let mut renderings = Vec::new();
    for chunk_size in [1_usize, 2, 3, 5, 7, payload.len()] {
        let (mut writer, buffer) = make_writer(true, prefix_props(), 80);
        for chunk in payload.chunks(chunk_size) {
            writer.write(Stream::Stdout, chunk).expect("write");
        }
        writer.finalize().expect("finalize");
        renderings.push(buffer.lock().expect("lock").clone());
    }

    let first = renderings[0].clone();
    assert!(
        renderings.iter().all(|rendering| *rendering == first),
        "output depends on write chunking"
    );
    let text = String::from_utf8(first).expect("utf8");
    assert!(text.contains('\u{20ac}'));
    assert!(!text.contains('\u{fffd}'));
}

// ---------------------------------------------------------------------------
// Mocked TeX children (mock_mode)
// ---------------------------------------------------------------------------

#[test]
fn mock_xelatex_writes_bcf_and_aux() {
    let dir = tempfile::tempdir().expect("temp dir");
    crate::run_mock_child(crate::MockChild::Xelatex, dir.path()).expect("mock runs");
    assert!(dir.path().join("main.bcf").is_file());
    assert!(dir.path().join("main.aux").is_file());
}

#[test]
fn mock_biber_writes_bbl() {
    let dir = tempfile::tempdir().expect("temp dir");
    crate::run_mock_child(crate::MockChild::Biber, dir.path()).expect("mock runs");
    assert!(dir.path().join("main.bbl").is_file());
}

#[test]
fn mock_latexmk_writes_pdf_and_aux() {
    let dir = tempfile::tempdir().expect("temp dir");
    crate::run_mock_child(crate::MockChild::Latexmk, dir.path()).expect("mock runs");
    assert!(dir.path().join("main.pdf").is_file());
    assert!(dir.path().join("main.aux").is_file());
}

#[test]
fn mock_child_is_compare_if_changed() {
    let dir = tempfile::tempdir().expect("temp dir");
    crate::run_mock_child(crate::MockChild::Latexmk, dir.path()).expect("mock runs");
    let pdf = dir.path().join("main.pdf");

    let old = std::time::SystemTime::UNIX_EPOCH;
    std::fs::File::options()
        .write(true)
        .open(&pdf)
        .expect("open")
        .set_times(std::fs::FileTimes::new().set_modified(old))
        .expect("set mtime");

    crate::run_mock_child(crate::MockChild::Latexmk, dir.path()).expect("mock runs");
    let mtime = std::fs::metadata(&pdf)
        .expect("metadata")
        .modified()
        .expect("mtime");
    assert_eq!(mtime, old, "identical mock output must not be rewritten");
}

#[test]
fn child_stream_capture_propagates_into_readers_and_preserves_bytes() {
    isolated_capture!(child_stream_capture_propagates_into_readers_and_preserves_bytes);
    let dir = tempfile::tempdir().expect("tempdir");
    let output = dir.path().join("both.log");
    let cfg = RoutingConfig {
        redirect: Some(output.clone()),
        ..RoutingConfig::default()
    };
    let command = [
        std::ffi::OsString::from("sh"),
        std::ffi::OsString::from("-c"),
        std::ffi::OsString::from("printf 'capture\\n'"),
    ];
    let plain = super::run(&command, &cfg, |_| {}).expect("plain run");
    let plain_bytes = std::fs::read(&output).expect("read output");
    let (observed, records) = with_capture(|| super::run(&command, &cfg, |_| {}));
    let observed = observed.expect("observed run");
    assert_eq!(
        (plain.exit_code, plain.data_loss),
        (observed.exit_code, observed.data_loss)
    );
    assert_eq!(plain_bytes, std::fs::read(&output).expect("read output"));
    assert_eq!(plain_bytes, b"capture\n");
    assert_eq!(
        records
            .iter()
            .filter(|record| record.name == "child_stream_read")
            .count(),
        2
    );
    let streams: Vec<_> = records
        .iter()
        .filter(|record| record.target == "execwrap::stream" && record.name == "event")
        .collect();
    assert_eq!(streams.len(), 2);
    let stdout = streams
        .iter()
        .find(|record| record.fields.get("stream").map(String::as_str) == Some("stdout"))
        .expect("stdout stream event");
    assert_eq!(
        stdout.fields.get("read_bytes").map(String::as_str),
        Some("8")
    );
    assert_eq!(
        stdout.fields.get("written_bytes").map(String::as_str),
        Some("8")
    );
    assert_eq!(
        stdout.fields.get("outcome").map(String::as_str),
        Some("completed")
    );
}

#[test]
fn log_notification_events_preserve_callbacks_and_assets() {
    isolated_capture!(log_notification_events_preserve_callbacks_and_assets);
    let dir = tempfile::tempdir().expect("tempdir");
    let command = [
        std::ffi::OsString::from("sh"),
        std::ffi::OsString::from("-c"),
        std::ffi::OsString::from("printf 'notice\\n'"),
    ];
    let output = dir.path().join("direct.log");
    let cfg = RoutingConfig {
        redirect: Some(output.clone()),
        notify_on_write: true,
        ..RoutingConfig::default()
    };
    let mut plain_notifications = Vec::new();
    let plain = super::run(&command, &cfg, |notification| {
        plain_notifications.push(matches!(
            notification,
            super::Notification::WroteToLog { .. }
        ));
    })
    .expect("plain run");
    let plain_bytes = std::fs::read(&output).expect("plain bytes");
    let mut observed_notifications = Vec::new();
    let (observed, records) = with_capture(|| {
        super::run(&command, &cfg, |notification| {
            observed_notifications.push(matches!(
                notification,
                super::Notification::WroteToLog { .. }
            ));
        })
    });
    let observed = observed.expect("observed run");
    assert_eq!(plain.exit_code, observed.exit_code);
    assert_eq!(plain.data_loss, observed.data_loss);
    assert_eq!(plain_notifications, observed_notifications);
    assert_eq!(plain_notifications, [false]);
    assert_eq!(plain_bytes, std::fs::read(&output).expect("observed bytes"));
    assert_eq!(plain_bytes, b"notice\n");
    let notice = event(&records, "execwrap::routing", "log write started");
    assert_eq!(notice.level, Level::INFO);
    assert_eq!(
        notice.fields.get("stream").map(String::as_str),
        Some("stdout")
    );
}

struct FailingFlushSink {
    inner: Arc<Mutex<Vec<u8>>>,
}

impl Write for FailingFlushSink {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut guard = self.inner.lock().expect("sink lock");
        guard.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::other("injected flush failure"))
    }
}

#[test]
fn writer_finalize_capture_preserves_success_and_failure_bytes() {
    isolated_capture!(writer_finalize_capture_preserves_success_and_failure_bytes);
    let (mut plain_writer, plain_buffer) = make_writer(true, prefix_props(), 80);
    plain_writer.write(Stream::Stdout, b"tail").expect("write");
    plain_writer.finalize().expect("plain finalize");
    let (mut observed_writer, observed_buffer) = make_writer(true, prefix_props(), 80);
    observed_writer
        .write(Stream::Stdout, b"tail")
        .expect("write");
    let (observed_result, records) = with_capture(|| observed_writer.finalize());
    observed_result.expect("observed finalize");
    assert_eq!(
        *plain_buffer.lock().expect("plain lock"),
        *observed_buffer.lock().expect("observed lock")
    );
    let writer_span = span(&records, "execwrap::writer", "writer_finalize");
    assert_eq!(writer_span.level, Level::DEBUG);
    let completed = event(&records, "execwrap::writer", "writer finalized");
    assert_eq!(
        completed.fields.get("outcome").map(String::as_str),
        Some("completed")
    );

    let plain_buffer = Arc::new(Mutex::new(Vec::new()));
    let mut plain_writer = Writer::new(
        FailingFlushSink {
            inner: Arc::clone(&plain_buffer),
        },
        false,
        80,
        StreamProps::default(),
        false,
    );
    plain_writer.write(Stream::Stdout, b"keep").expect("write");
    let plain_error = plain_writer.finalize().expect_err("plain failure");
    let observed_buffer = Arc::new(Mutex::new(Vec::new()));
    let mut observed_writer = Writer::new(
        FailingFlushSink {
            inner: Arc::clone(&observed_buffer),
        },
        false,
        80,
        StreamProps::default(),
        false,
    );
    observed_writer
        .write(Stream::Stdout, b"keep")
        .expect("write");
    let (observed_error, records) = with_capture(|| observed_writer.finalize());
    assert_eq!(
        plain_error.kind(),
        observed_error.expect_err("observed failure").kind()
    );
    assert_eq!(
        *plain_buffer.lock().expect("plain lock"),
        *observed_buffer.lock().expect("observed lock")
    );
    let failed = event(&records, "execwrap::writer", "writer finalized");
    assert_eq!(failed.level, Level::DEBUG);
    assert_eq!(
        failed.fields.get("outcome").map(String::as_str),
        Some("failed")
    );
    assert_eq!(
        failed.fields.get("error_kind").map(String::as_str),
        Some("io")
    );
    no_high_severity(&records);
}

#[test]
fn mock_capture_preserves_assets_and_typed_failure() {
    isolated_capture!(mock_capture_preserves_assets_and_typed_failure);
    let dir = tempfile::tempdir().expect("tempdir");
    crate::run_mock_child(crate::MockChild::Biber, dir.path()).expect("plain mock");
    let plain_bytes = std::fs::read(dir.path().join("main.bbl")).expect("plain bytes");
    let (observed, records) =
        with_capture(|| crate::run_mock_child(crate::MockChild::Biber, dir.path()));
    observed.expect("observed mock");
    assert_eq!(
        plain_bytes,
        std::fs::read(dir.path().join("main.bbl")).expect("observed bytes")
    );
    let mock_span = span(&records, "execwrap::mock", "mock_child_run");
    assert_eq!(mock_span.level, Level::INFO);
    let completed = event(&records, "execwrap::mock", "mock child completed");
    assert_eq!(
        completed.fields.get("outcome").map(String::as_str),
        Some("completed")
    );

    let blocked = dir.path().join("blocked");
    std::fs::write(&blocked, b"keep").expect("block output directory");
    let plain_error =
        crate::run_mock_child(crate::MockChild::Biber, &blocked).expect_err("plain failure");
    let (observed_error, records) =
        with_capture(|| crate::run_mock_child(crate::MockChild::Biber, &blocked));
    assert_eq!(
        plain_error.kind(),
        observed_error.expect_err("observed failure").kind()
    );
    assert_eq!(std::fs::read(&blocked).expect("blocked bytes"), b"keep");
    let failed = event(&records, "execwrap::mock", "mock child completed");
    assert_eq!(failed.level, Level::INFO);
    assert_eq!(
        failed.fields.get("outcome").map(String::as_str),
        Some("failed")
    );
    assert_eq!(
        failed.fields.get("error_kind").map(String::as_str),
        Some("io")
    );
    no_high_severity(&records);
}
