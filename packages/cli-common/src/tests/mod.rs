//! # CLI-common tests
//!
//! | Test                                  | What it covers                                   |
//! |---------------------------------------|--------------------------------------------------|
//! | `emit_writes_one_json_line`           | `emit` produces compact JSON + trailing newline. |
//! | `emit_propagates_serialisation_error` | A non-`Serialize`-friendly value surfaces `Err`. |
//! | `emit_propagates_writer_error`        | A failing writer surfaces `Err` (broken pipe).   |
//! | `partial_stdout_write_during_emit_is_not_reverted` | A mid-write pipe break leaves a partial prefix. |
//! | `emit_to_real_stdout_succeeds`        | `emit` itself writes to the captured stdout.     |
//! | `base_args_parses_default`            | `BaseArgs::debug` defaults to `false`.           |
//! | `base_args_parses_long_flag`          | `--debug` flips `BaseArgs::debug` to `true`.     |
//! | `command_exit_codes_match_contract`   | Baseline process statuses are fixed.             |
//! | `control_record_serialises_shape`      | Shared stderr record shape is stable.            |
//! | `json_line_writer_appends_newline`      | JSON record helper writes one complete line.     |
//! | `usage_error_record_carries_fields`    | Usage records include exit code and clap kind.   |
//! | `tty_refusal_record_carries_fields`    | TTY refusal records identify stdout and code 2.  |
//! | `panic_record_omits_payload_without_debug` | Panic records hide payloads without debug.    |
//! | `panic_record_carries_debug_payload`    | Panic records can expose string payloads.        |
//! | `panic_payload_reporting_fails_closed_then_follows_debug_flag` | Startup payload gate fails closed. |
//! | `panic_payload_message_extracts_string_payloads` | String panic payloads are downcast.     |
//! | `parse_args_from_returns_help_record`  | Clap help becomes JSON stderr control data.      |
//! | `parse_args_from_returns_version_record` | Clap version becomes JSON stderr control data. |
//! | `parse_args_from_returns_usage_record` | Clap argv errors become JSON usage records.      |
//! | `tracing_filter_prefers_rust_log`      | `RUST_LOG` wins over `--debug`.                  |
//! | `tracing_filter_uses_debug_flag`       | `--debug` selects debug without `RUST_LOG`.      |
//! | `tracing_filter_uses_default_level`    | Default level is used last.                      |
//! | `redacts_sensitive_field_values`       | Secret-like field names are hidden.              |
//! | `redacts_database_url_secrets`         | DB credentials and query secrets are removed.    |
//! | `redacts_credentials_in_any_url_scheme` | HTTPS and custom-scheme credentials are removed. |
//! | `redacts_secrets_in_free_form_text`    | key=value and URL secrets scrubbed from text.    |
//! | `redacts_key_equals_credential_url_token` | `--key=credential-url` tokens are scrubbed.   |
//! | `redacts_credential_url_in_error_chain_text` | key=URL inside a longer sentence is scrubbed. |
//! | `redacts_sensitive_url_fragments`      | OAuth-style fragment secrets are dropped.        |
//! | `unparseable_url_ish_token_fails_closed` | Unparseable `://` tokens become `[redacted]`.  |
//! | `keeps_non_sensitive_key_value_tokens` | Safe key=value tokens survive untouched.         |
//! | `redacts_runner_style_error_display`   | Runner-shaped error text loses credential URLs.  |
//! | `usage_error_text_never_reproduces_secrets` | Usage records omit argument values.         |
//! | `keeps_public_key_fields_visible`      | Public key metadata is not treated as secret.    |
//!
//! `refuse_if_stdout_is_tty` and `init_json_tracing` mutate
//! global process state (the `process::exit` path and the
//! installed tracing subscriber) and are deliberately
//! exercised only by the per-binary integration tests of the
//! helper binaries — invoking them here would either abort
//! the test process or install a global subscriber that
//! interferes with the rest of the test binary's output.

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::io::{self, Write};

use clap::Parser;
use serde::Serialize;
use serde_json::json;

use crate::{
    BaseArgs, CONTROL_PLANE_SCHEMA, CommandExit, ControlPlaneFields, ControlPlaneRecord,
    ControlPlaneRecordKind, REDACTED, StandardStream, TracingFilterSource,
    panic_payload_message_from_payload, panic_payload_reporting_enabled, parse_args_from,
    redact_database_url, redact_field_value, set_panic_payload_reporting_enabled,
    tracing_filter_from_rust_log, write_json_line,
};

/// A `Write` that fails every call with `BrokenPipe`.
///
/// Used to cover the `emit` error branch without actually
/// closing stdout.
struct FailingWriter;

impl Write for FailingWriter {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "test: pipe closed",
        ))
    }

    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "test: pipe closed",
        ))
    }
}

/// A `Write` that accepts a bounded prefix of bytes and then fails —
/// modeling a stdout pipe that breaks partway through result emission.
struct PrefixThenFailWriter {
    written: Vec<u8>,
    limit: usize,
}

impl Write for PrefixThenFailWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.written.len() >= self.limit {
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "test: pipe closed mid-write",
            ));
        }
        let take = buf.len().min(self.limit - self.written.len());
        self.written.extend_from_slice(&buf[..take]);
        Ok(take)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Parser)]
#[command(name = "fixture-helper", version = "1.2.3", about = "Fixture helper")]
#[allow(dead_code)]
struct FixtureCli {
    #[arg(long)]
    name: Option<String>,

    #[command(flatten)]
    base: BaseArgs,
}

/// Pure helper that mirrors `emit` but writes to an arbitrary
/// `Write`. Lets us assert the on-the-wire bytes without
/// touching the real `stdout()` lock (which would interleave
/// with cargo's own captured-output machinery).
fn emit_to<W: Write, T: Serialize>(mut w: W, value: &T) -> io::Result<()> {
    let json = serde_json::to_string(value)?;
    w.write_all(json.as_bytes())?;
    w.write_all(b"\n")?;
    w.flush()
}

#[test]
fn emit_writes_one_json_line() {
    // BTreeMap to fix key order so the assertion is stable.
    let mut value: BTreeMap<&str, u32> = BTreeMap::new();
    value.insert("a", 1);
    value.insert("b", 2);

    let mut buf = Vec::new();
    emit_to(&mut buf, &value).expect("emit must succeed for a writable buffer");

    assert_eq!(buf, b"{\"a\":1,\"b\":2}\n");
}

#[test]
fn emit_propagates_writer_error() {
    let value = serde_json::json!({"k": "v"});
    let err = emit_to(FailingWriter, &value).expect_err("emit must surface writer errors");
    assert_eq!(err.kind(), io::ErrorKind::BrokenPipe);
}

#[test]
fn partial_stdout_write_during_emit_is_not_reverted() {
    // A pipe that breaks partway through result emission: `emit` surfaces
    // the error, but the prefix it already wrote cannot be un-written.
    // ADR-010 therefore promises empty stdout only *before* result
    // publication begins, never during it (`rule:output:json`).
    let value = serde_json::json!({
        "key": "a moderately long value that exceeds the prefix limit"
    });

    let mut writer = PrefixThenFailWriter {
        written: Vec::new(),
        limit: 8,
    };
    let err = emit_to(&mut writer, &value).expect_err("a mid-write pipe break surfaces an error");

    assert_eq!(err.kind(), io::ErrorKind::BrokenPipe);
    assert_eq!(
        writer.written.len(),
        8,
        "the partial prefix was already published to stdout and cannot be reverted",
    );
}

#[test]
fn emit_propagates_serialisation_error() {
    // `serde_json::Map` keys must be strings; a map keyed by a
    // non-string serialises to an `io::Error` wrapped serde
    // failure when bridged through `serde_json::to_string`.
    //
    // Easier route: a custom `Serialize` impl that always errors.
    use serde::Serializer;

    struct AlwaysFails;
    impl Serialize for AlwaysFails {
        fn serialize<S: Serializer>(&self, _s: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("test: forced failure"))
        }
    }

    let mut buf = Vec::new();
    let err = emit_to(&mut buf, &AlwaysFails).expect_err("must surface ser error");
    // `serde_json::Error` converts to `io::Error` via `From`,
    // which preserves a non-Other kind only for IO sources;
    // for ser errors the kind is `InvalidData` or `Other`
    // depending on serde-json version. Don't pin the kind —
    // the contract is "an error is returned and nothing is
    // written".
    assert!(buf.is_empty(), "no bytes should be written on ser failure");
    drop(err); // kind not asserted, see comment above.
}

#[test]
fn emit_to_real_stdout_succeeds() {
    // Cover the real `emit` (which writes to `io::stdout`)
    // — under cargo test, stdout is captured and a write is
    // always allowed, so this just exercises the happy path.
    crate::emit(&serde_json::json!({"k": "v"})).expect("emit must succeed under captured stdout");
}

#[test]
fn base_args_parses_default() {
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        base: BaseArgs,
    }

    let parsed = Cli::try_parse_from(["prog"]).expect("no args is valid");
    assert!(!parsed.base.debug);
}

#[test]
fn base_args_parses_long_flag() {
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        base: BaseArgs,
    }

    let parsed = Cli::try_parse_from(["prog", "--debug"]).expect("--debug is valid");
    assert!(parsed.base.debug);
}

#[test]
fn command_exit_codes_match_contract() {
    assert_eq!(CommandExit::Success.code(), 0);
    assert_eq!(CommandExit::Failure.code(), 1);
    assert_eq!(CommandExit::Usage.code(), 2);

    assert_eq!(CommandExit::from_code(0), Some(CommandExit::Success));
    assert_eq!(CommandExit::from_code(1), Some(CommandExit::Failure));
    assert_eq!(CommandExit::from_code(2), Some(CommandExit::Usage));
    assert_eq!(CommandExit::from_code(3), None);
}

#[test]
fn control_record_serialises_shape() {
    let record = ControlPlaneRecord::new("fixture", ControlPlaneRecordKind::Status, "ready", None);
    let value = serde_json::to_value(record).unwrap();

    assert_eq!(value["schema"], json!(CONTROL_PLANE_SCHEMA));
    assert_eq!(value["command"], json!("fixture"));
    assert_eq!(value["kind"], json!("status"));
    assert_eq!(value["message"], json!("ready"));
    assert!(value.get("fields").is_none());
}

#[test]
fn json_line_writer_appends_newline() {
    let record = ControlPlaneRecord::status("fixture", "ready");
    let mut buf = Vec::new();

    write_json_line(&mut buf, &record).expect("record should serialize");

    assert!(buf.ends_with(b"\n"));
    let line = std::str::from_utf8(&buf).expect("JSON must be UTF-8");
    let value: serde_json::Value =
        serde_json::from_str(line).expect("record must be a JSON object");
    assert_eq!(value["schema"], json!(CONTROL_PLANE_SCHEMA));
    assert_eq!(value["kind"], json!("status"));
}

#[test]
fn usage_error_record_carries_fields() {
    let record = ControlPlaneRecord::usage_error("fixture", "unknown_argument");

    assert_eq!(record.kind, ControlPlaneRecordKind::UsageError);
    assert_eq!(
        record.fields,
        Some(ControlPlaneFields::UsageError {
            exit_code: CommandExit::Usage.code(),
            clap_error_kind: "unknown_argument".to_string(),
        })
    );

    let value = serde_json::to_value(record).unwrap();
    assert_eq!(value["fields"]["type"], json!("usage_error"));
    assert_eq!(value["fields"]["exit_code"], json!(2));
}

#[test]
fn tty_refusal_record_carries_fields() {
    let record = ControlPlaneRecord::tty_refusal("fixture");

    assert_eq!(record.kind, ControlPlaneRecordKind::TtyRefusal);
    assert_eq!(
        record.fields,
        Some(ControlPlaneFields::TtyRefusal {
            exit_code: CommandExit::Usage.code(),
            stream: StandardStream::Stdout,
        })
    );
}

#[test]
fn panic_record_omits_payload_without_debug() {
    let record =
        ControlPlaneRecord::panic("fixture", Some("main"), Some("src/main.rs:12:34"), None);
    let value = serde_json::to_value(record).unwrap();

    assert_eq!(value["kind"], json!("panic"));
    assert_eq!(value["fields"]["type"], json!("panic"));
    assert_eq!(value["fields"]["exit_code"], json!(1));
    assert_eq!(value["fields"]["thread"], json!("main"));
    assert!(value["fields"].get("payload").is_none());
}

#[test]
fn panic_record_carries_debug_payload() {
    let record = ControlPlaneRecord::panic(
        "fixture",
        Some("main"),
        Some("src/main.rs:12:34"),
        Some("panic with \"quoted\" detail"),
    );
    let line = serde_json::to_string(&record).unwrap();

    assert!(line.contains(r#""payload":"panic with \"quoted\" detail""#));

    let value: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert_eq!(
        value["fields"]["payload"],
        json!("panic with \"quoted\" detail")
    );
}

#[test]
fn panic_payload_reporting_fails_closed_then_follows_debug_flag() {
    // Fail closed (ADR-010): payloads may contain secrets, so the gate
    // starts disabled and opens only via the parsed --debug value.
    assert!(!panic_payload_reporting_enabled());

    set_panic_payload_reporting_enabled(true);
    assert!(panic_payload_reporting_enabled());

    set_panic_payload_reporting_enabled(false);
    assert!(!panic_payload_reporting_enabled());
}

#[test]
fn panic_payload_message_extracts_string_payloads() {
    let borrowed_payload: &(dyn std::any::Any + Send) = &"borrowed panic";
    let owned_payload: &(dyn std::any::Any + Send) = &String::from("owned panic");
    let numeric_payload: &(dyn std::any::Any + Send) = &1_u8;

    assert_eq!(
        panic_payload_message_from_payload(borrowed_payload),
        Some("borrowed panic")
    );
    assert_eq!(
        panic_payload_message_from_payload(owned_payload),
        Some("owned panic")
    );
    assert_eq!(panic_payload_message_from_payload(numeric_payload), None);
}

#[test]
fn parse_args_from_returns_help_record() {
    let Err(exit) = parse_args_from::<FixtureCli, _, _>(["fixture-helper", "--help"]) else {
        panic!("help should stop parsing");
    };

    assert_eq!(exit.exit, CommandExit::Success);
    assert_eq!(exit.record.command, "fixture-helper");
    assert_eq!(exit.record.kind, ControlPlaneRecordKind::Help);

    let Some(ControlPlaneFields::Help { text }) = exit.record.fields else {
        panic!("help record should carry help text");
    };
    assert!(text.contains("Fixture helper"));
    assert!(text.contains("--debug"));
}

#[test]
fn parse_args_from_returns_version_record() {
    let Err(exit) = parse_args_from::<FixtureCli, _, _>(["fixture-helper", "--version"]) else {
        panic!("version should stop parsing");
    };

    assert_eq!(exit.exit, CommandExit::Success);
    assert_eq!(exit.record.command, "fixture-helper");
    assert_eq!(exit.record.kind, ControlPlaneRecordKind::Version);

    let Some(ControlPlaneFields::Version { version }) = exit.record.fields else {
        panic!("version record should carry version text");
    };
    assert_eq!(version, "fixture-helper 1.2.3");
}

#[test]
fn parse_args_from_returns_usage_record() {
    let Err(exit) = parse_args_from::<FixtureCli, _, _>(["fixture-helper", "--no-such-flag"])
    else {
        panic!("unknown flags should stop parsing");
    };

    assert_eq!(exit.exit, CommandExit::Usage);
    assert_eq!(exit.record.command, "fixture-helper");
    assert_eq!(exit.record.kind, ControlPlaneRecordKind::UsageError);
    assert_eq!(exit.record.message, "invalid command-line arguments");

    let Some(ControlPlaneFields::UsageError {
        exit_code,
        clap_error_kind,
    }) = exit.record.fields
    else {
        panic!("usage record should carry usage fields");
    };
    assert_eq!(exit_code, CommandExit::Usage.code());
    assert_eq!(clap_error_kind, "unknown_argument");
}

#[test]
fn tracing_filter_prefers_rust_log() {
    let filter = tracing_filter_from_rust_log(
        Some(OsStr::new("warn,tower_http=debug")),
        true,
        tracing::Level::INFO,
    );

    assert_eq!(filter.directive, "warn,tower_http=debug");
    assert_eq!(filter.source, TracingFilterSource::RustLog);
}

#[test]
fn tracing_filter_uses_debug_flag() {
    let filter = tracing_filter_from_rust_log(Some(OsStr::new("   ")), true, tracing::Level::INFO);

    assert_eq!(filter.directive, "debug");
    assert_eq!(filter.source, TracingFilterSource::DebugFlag);
}

#[test]
fn tracing_filter_uses_default_level() {
    let filter = tracing_filter_from_rust_log(None, false, tracing::Level::WARN);

    assert_eq!(filter.directive, "warn");
    assert_eq!(filter.source, TracingFilterSource::DefaultLevel);
}

#[test]
fn redacts_sensitive_field_values() {
    assert_eq!(
        redact_field_value("tracker.token", "MyAccessToken"),
        REDACTED
    );
    assert_eq!(redact_field_value("smtp_password", "secret"), REDACTED);
    assert_eq!(redact_field_value("auth.private_key_pem", "PEM"), REDACTED);
}

#[test]
fn redacts_database_url_secrets() {
    let redacted = redact_database_url(
        "mysql://user:pass@example.test/db?ssl-mode=required&token=abc&password=def",
    );

    assert_eq!(redacted, "mysql://example.test/db?ssl-mode=required");
}

#[test]
fn keeps_public_key_fields_visible() {
    assert_eq!(
        redact_field_value("auth.public_key_path", "/etc/app/public.pem"),
        "/etc/app/public.pem"
    );
}

#[test]
fn redacts_credentials_in_any_url_scheme() {
    assert_eq!(
        crate::redact_url("https://user:password@example.test/path?api_key=secret&page=2"),
        "https://example.test/path?page=2"
    );

    assert_eq!(
        crate::redact_url("rpc://alice:hunter2@node.test:7041/"),
        "rpc://node.test:7041/"
    );

    // A safe public URL is returned unchanged (borrowed).
    assert_eq!(
        crate::redact_url("https://example.test/docs?page=2"),
        "https://example.test/docs?page=2"
    );

    // A non-URL value is untouched.
    assert_eq!(crate::redact_url("not a url"), "not a url");

    // A non-database field value carrying a credential URL is
    // scrubbed on the generic field path too.
    assert_eq!(
        redact_field_value("endpoint", "https://user:password@example.test/?api_key=x"),
        "https://example.test/"
    );
}

#[test]
fn redacts_secrets_in_free_form_text() {
    let text = "error: unexpected argument '--api-token=SHOULD_NOT_APPEAR' found\n\
                endpoint https://user:pw@example.test/ retried";

    let redacted = crate::redact_text(text);

    assert!(!redacted.contains("SHOULD_NOT_APPEAR"));
    assert!(!redacted.contains("user:pw"));
    assert!(redacted.contains(&format!("--api-token={REDACTED}")));
    assert!(redacted.contains("https://example.test/"));
    assert!(redacted.contains("error: unexpected argument"));
    // Whitespace and safe text survive untouched.
    assert!(redacted.contains('\n'));
}

#[test]
fn redacts_key_equals_credential_url_token() {
    // The whole token fails `Url::parse` (the `--endpoint=` prefix is
    // not a scheme), so the value side must be redacted on its own.
    let redacted =
        crate::redact_text("--endpoint=https://user:password@example.test/path?api_key=secret");

    assert_eq!(redacted, "--endpoint=https://example.test/path");
    assert!(!redacted.contains("password"));
    assert!(!redacted.contains("secret"));
}

#[test]
fn redacts_credential_url_in_error_chain_text() {
    let text = "request failed: endpoint=https://user:pw@example.test/ (attempt 2 of 3)";

    let redacted = crate::redact_text(text);

    assert!(!redacted.contains("user:pw"));
    assert!(redacted.contains("endpoint=https://example.test/"));
    assert!(redacted.contains("request failed:"));
    assert!(redacted.contains("(attempt 2 of 3)"));
}

#[test]
fn redacts_sensitive_url_fragments() {
    // OAuth implicit-grant style: the secret rides in the fragment,
    // which query-pair scrubbing never sees. Safe pieces survive.
    assert_eq!(
        crate::redact_url("https://example.test/cb#access_token=abc&state=xyz"),
        "https://example.test/cb#state=xyz"
    );

    // A fully sensitive fragment is cleared outright.
    assert_eq!(
        crate::redact_url("https://example.test/cb#access_token=abc"),
        "https://example.test/cb"
    );

    // Fragment secrets are also caught on the free-text path.
    let redacted = crate::redact_text("redirected to https://example.test/cb#id_token=SECRET");
    assert!(!redacted.contains("SECRET"));
}

#[test]
fn unparseable_url_ish_token_fails_closed() {
    // "://user:pw@" has no scheme, so `Url::parse` fails even after
    // substring extraction; the token must not be echoed verbatim.
    assert_eq!(
        crate::redact_text("connect to ://user:pw@ failed"),
        format!("connect to {REDACTED} failed")
    );

    // "https://" alone fails parsing too (empty host).
    assert_eq!(crate::redact_text("https://"), REDACTED);
}

#[test]
fn keeps_non_sensitive_key_value_tokens() {
    let text = "--retries=5 mode=fast";

    assert_eq!(crate::redact_text(text), text);
}

#[test]
fn redacts_runner_style_error_display() {
    // The generic runners pass arbitrary `Display` error text through
    // `redact_text` before logging; this covers that text shape.
    let error_display =
        "command failed: error connecting to https://svc:token123@api.example.test/v1: timeout";

    let redacted = crate::redact_text(error_display);

    assert!(!redacted.contains("token123"));
    assert!(redacted.contains("https://api.example.test/v1"));
    assert!(redacted.contains("command failed:"));
    assert!(redacted.contains("timeout"));
}

#[test]
fn usage_error_text_never_reproduces_secrets() {
    let Err(exit) =
        parse_args_from::<FixtureCli, _, _>(["fixture-helper", "--api-token=SHOULD_NOT_APPEAR"])
    else {
        panic!("unknown secret-bearing argument must fail parsing");
    };

    assert_eq!(exit.exit, CommandExit::Usage);

    let rendered = serde_json::to_string(&exit.record).unwrap();

    assert!(
        !rendered.contains("SHOULD_NOT_APPEAR"),
        "usage record leaked the secret: {rendered}"
    );
}

#[test]
fn touch_stamp_creates_empty_and_redates_an_empty_stamp() {
    let dir = tempfile::tempdir().expect("tempdir");
    let stamp = dir.path().join("suite.ok");

    crate::touch_stamp(&stamp).expect("first touch creates");
    let created = std::fs::metadata(&stamp).expect("stamp exists");
    assert_eq!(created.len(), 0, "a stamp is empty by contract");

    let before = created.modified().expect("mtime");
    std::thread::sleep(std::time::Duration::from_millis(20));

    crate::touch_stamp(&stamp).expect("second touch re-dates an empty stamp");
    let metadata = std::fs::metadata(&stamp).expect("metadata");
    assert_eq!(metadata.len(), 0);
    assert!(metadata.modified().expect("mtime") > before);
}

#[test]
fn touch_stamp_refuses_a_nonempty_stamp_without_destroying_it() {
    // F3-010: bytes in a stamp mean something other than a first-party
    // command wrote it. The touch refuses (no fresh success fact, the
    // target stays dirty) and preserves the foreign bytes for
    // inspection rather than truncating them.
    let dir = tempfile::tempdir().expect("tempdir");
    let stamp = dir.path().join("suite.ok");
    std::fs::write(&stamp, b"foreign").expect("seed stamp");
    let before = std::fs::metadata(&stamp)
        .expect("metadata")
        .modified()
        .expect("mtime");

    let error = crate::touch_stamp(&stamp).expect_err("a nonempty stamp is refused");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(error.to_string().contains("empty by contract"), "{error}");
    assert_eq!(std::fs::read(&stamp).expect("read"), b"foreign");
    assert_eq!(
        std::fs::metadata(&stamp)
            .expect("metadata")
            .modified()
            .expect("mtime"),
        before,
    );
}

#[test]
fn build_mode_writes_report_then_touches_stamp() {
    let dir = tempfile::tempdir().expect("tempdir");
    let report = dir.path().join("out/report.json");
    let stamp = dir.path().join("suite.ok");
    let output = crate::CheckOutputArgs {
        report: Some(report.clone()),
        stamp: Some(stamp.clone()),
    };

    crate::finish_check_command("demo", &output, &serde_json::json!({"valid": true}))
        .expect("build-mode publish");

    assert_eq!(
        std::fs::read(&report).expect("report exists"),
        b"{\"valid\":true}\n",
    );
    assert!(stamp.exists(), "the success stamp is touched");
}

#[test]
fn build_mode_leaves_an_unchanged_report_untouched_on_rerun() {
    let dir = tempfile::tempdir().expect("tempdir");
    let report = dir.path().join("report.json");
    let output = crate::CheckOutputArgs {
        report: Some(report.clone()),
        stamp: Some(dir.path().join("suite.ok")),
    };
    let value = serde_json::json!({"valid": true});

    crate::finish_check_command("demo", &output, &value).expect("first publish");

    let old = std::time::SystemTime::UNIX_EPOCH;
    std::fs::File::options()
        .write(true)
        .open(&report)
        .expect("open report")
        .set_times(std::fs::FileTimes::new().set_modified(old))
        .expect("pin mtime");

    crate::finish_check_command("demo", &output, &value).expect("second publish");

    let mtime = std::fs::metadata(&report)
        .expect("metadata")
        .modified()
        .expect("mtime");
    assert_eq!(mtime, old, "an unchanged report must not be rewritten");
}

// SR3-04: the staged report is owner-only until publication sets the
// intended mode, so a check report must arrive publicly readable and
// stay that way across an unchanged rerun.
#[cfg(unix)]
#[test]
fn build_mode_publishes_a_publicly_readable_report() {
    use std::os::unix::fs::PermissionsExt as _;

    let dir = tempfile::tempdir().expect("tempdir");
    let report = dir.path().join("report.json");
    let output = crate::CheckOutputArgs {
        report: Some(report.clone()),
        stamp: Some(dir.path().join("suite.ok")),
    };
    let mode = |path: &std::path::Path| {
        std::fs::metadata(path)
            .expect("metadata")
            .permissions()
            .mode()
            & 0o777
    };

    crate::finish_check_command("demo", &output, &serde_json::json!({"valid": true}))
        .expect("absent destination");
    assert_eq!(mode(&report), 0o644, "a new report must be public");

    crate::finish_check_command("demo", &output, &serde_json::json!({"valid": false}))
        .expect("changed destination");
    assert_eq!(mode(&report), 0o644, "a rewritten report stays public");

    crate::finish_check_command("demo", &output, &serde_json::json!({"valid": false}))
        .expect("unchanged destination");
    assert_eq!(mode(&report), 0o644, "an unchanged report stays public");
}

#[test]
fn a_half_specified_output_mode_is_invalid() {
    let dir = tempfile::tempdir().expect("tempdir");
    let output = crate::CheckOutputArgs {
        report: Some(dir.path().join("report.json")),
        stamp: None,
    };
    assert!(matches!(
        crate::finish_check_command("demo", &output, &serde_json::json!({})),
        Err(crate::CheckResultError::InvalidMode),
    ));
}

#[test]
fn a_failed_report_write_leaves_the_stamp_untouched() {
    // Point the report at a path whose parent is a regular file, so
    // directory creation fails and the report is never published. The
    // stamp must not appear: it is the success fact.
    let dir = tempfile::tempdir().expect("tempdir");
    let blocker = dir.path().join("blocker");
    std::fs::write(&blocker, b"not a directory").expect("seed blocker");
    let report = blocker.join("nested/report.json");
    let stamp = dir.path().join("suite.ok");
    let output = crate::CheckOutputArgs {
        report: Some(report),
        stamp: Some(stamp.clone()),
    };

    assert!(matches!(
        crate::finish_check_command("demo", &output, &serde_json::json!({})),
        Err(crate::CheckResultError::Io(_)),
    ));
    assert!(
        !stamp.exists(),
        "a failed report write leaves no success stamp"
    );
}

// F2-005 output-role uniqueness: a multi-output command must never exit
// success with one role's bytes overwriting another's, however the two
// destinations are spelled.

#[test]
fn identical_output_paths_are_aliased() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("out");

    let error = crate::ensure_distinct_outputs(&[("first", &out), ("second", &out)])
        .expect_err("identical destinations");
    assert_eq!(error.first, "first");
    assert_eq!(error.second, "second");
}

#[test]
fn lexical_alias_spellings_are_aliased() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir(dir.path().join("sub")).expect("mkdir");
    let plain = dir.path().join("out");
    let dotted = dir.path().join("sub/../out");

    let error = crate::ensure_distinct_outputs(&[("first", &plain), ("second", &dotted)])
        .expect_err("lexical alias");
    assert_eq!(
        (error.first.as_str(), error.second.as_str()),
        ("first", "second")
    );
}

#[test]
fn relative_and_absolute_spellings_of_one_destination_are_aliased() {
    // A relative role path resolves against the process working
    // directory, its documented base, so the two spellings collide.
    let cwd = std::env::current_dir().expect("working directory");
    let relative = std::path::Path::new("./out");

    crate::ensure_distinct_outputs(&[("first", relative), ("second", &cwd.join("out"))])
        .expect_err("one destination named two ways");
}

// ADR-017: role uniqueness is lexical. Host-level aliasing is not
// detected, because detecting some aliases while a host may replace or
// remount a path at any moment establishes no boundary. These two
// cases record that as the decided behaviour rather than an oversight.

#[cfg(unix)]
#[test]
fn hard_linked_destinations_are_not_treated_as_one_role() {
    let dir = tempfile::tempdir().expect("tempdir");
    let first = dir.path().join("first-name");
    std::fs::write(&first, b"content").expect("write");
    let second = dir.path().join("second-name");
    std::fs::hard_link(&first, &second).expect("hard link");

    // A hard link is not semantic identity; first-party outputs are
    // identified by role, path, schema, and bytes. Callers must not
    // alias distinct roles this way.
    crate::ensure_distinct_outputs(&[("first", &first), ("second", &second)])
        .expect("lexically distinct paths pass; the caller owns hard-link aliasing");
}

#[cfg(unix)]
#[test]
fn symlinked_parent_directories_are_not_resolved() {
    let dir = tempfile::tempdir().expect("tempdir");
    let real = dir.path().join("real");
    std::fs::create_dir(&real).expect("mkdir");
    let linked = dir.path().join("linked");
    std::os::unix::fs::symlink(&real, &linked).expect("symlink");

    crate::ensure_distinct_outputs(&[
        ("first", &real.join("out")),
        ("second", &linked.join("out")),
    ])
    .expect("no symlink resolution; the host owns what these paths resolve to");
}

#[test]
fn distinct_destinations_validate() {
    let dir = tempfile::tempdir().expect("tempdir");

    crate::ensure_distinct_outputs(&[
        ("first", &dir.path().join("one")),
        ("second", &dir.path().join("two")),
    ])
    .expect("distinct destinations are valid");
}

#[test]
fn build_mode_rejects_aliased_report_and_stamp() {
    let dir = tempfile::tempdir().expect("tempdir");
    let shared = dir.path().join("shared");
    let output = crate::CheckOutputArgs {
        report: Some(shared.clone()),
        stamp: Some(dir.path().join("sub/../shared")),
    };
    std::fs::create_dir(dir.path().join("sub")).expect("mkdir");

    let error = crate::finish_check_command("demo", &output, &serde_json::json!({"valid": true}))
        .expect_err("aliased report/stamp");

    // The failure precedes publication: no report bytes, no stamp.
    assert!(matches!(error, crate::CheckResultError::AliasedOutputs(_)));
    assert!(
        !shared.exists(),
        "nothing may be published on alias failure"
    );
}

// --- T3: batch-staged publication ---

mod publication {
    use std::path::Path;

    use crate::{BatchPublicationError, PublicationAsset, publish_batch};

    fn asset<'a>(role: &'a str, path: &'a Path, bytes: &'a [u8]) -> PublicationAsset<'a> {
        PublicationAsset { role, path, bytes }
    }

    /// Write a destination already carrying the public publication
    /// mode, so a byte-equality test is not perturbed by the umask
    /// (freshness is bytes and mode together).
    fn write_public(path: &Path, bytes: &[u8]) {
        std::fs::write(path, bytes).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o644)).unwrap();
        }
    }

    fn staged_leftovers(directory: &Path) -> Vec<String> {
        std::fs::read_dir(directory)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| name.starts_with(".publication-staged-"))
            .collect()
    }

    #[test]
    fn aliased_outputs_fail_before_any_write() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.txt");

        let error = publish_batch(&[asset("first", &path, b"a"), asset("second", &path, b"b")])
            .unwrap_err();

        assert!(matches!(error, BatchPublicationError::AliasedOutputs(_)));
        assert!(!path.exists());
        assert_eq!(staged_leftovers(dir.path()), [] as [std::string::String; 0]);
    }

    #[test]
    fn unchanged_destinations_keep_bytes_and_mtimes() {
        let dir = tempfile::tempdir().unwrap();
        let first = dir.path().join("first.txt");
        let second = dir.path().join("second.txt");
        write_public(&first, b"one");
        write_public(&second, b"two");
        let first_mtime = std::fs::metadata(&first).unwrap().modified().unwrap();
        let second_mtime = std::fs::metadata(&second).unwrap().modified().unwrap();

        let results = publish_batch(&[
            asset("first", &first, b"one"),
            asset("second", &second, b"two"),
        ])
        .unwrap();

        assert!(results.iter().all(|result| result.change.unchanged()));
        assert_eq!(
            std::fs::metadata(&first).unwrap().modified().unwrap(),
            first_mtime
        );
        assert_eq!(
            std::fs::metadata(&second).unwrap().modified().unwrap(),
            second_mtime
        );
    }

    #[test]
    fn only_the_changed_member_is_rewritten() {
        let dir = tempfile::tempdir().unwrap();
        let current = dir.path().join("current.txt");
        let stale = dir.path().join("stale.txt");
        write_public(&current, b"kept");
        write_public(&stale, b"old");
        let kept_mtime = std::fs::metadata(&current).unwrap().modified().unwrap();

        let results = publish_batch(&[
            asset("current", &current, b"kept"),
            asset("stale", &stale, b"new"),
        ])
        .unwrap();

        assert_eq!(
            results
                .iter()
                .map(|result| result.change.bytes_changed)
                .collect::<Vec<_>>(),
            [false, true]
        );
        assert_eq!(std::fs::read(&stale).unwrap(), b"new");
        assert_eq!(
            std::fs::metadata(&current).unwrap().modified().unwrap(),
            kept_mtime
        );
    }

    #[test]
    fn a_staging_failure_changes_no_final_destination() {
        let dir = tempfile::tempdir().unwrap();
        let first = dir.path().join("first.txt");
        write_public(&first, b"old");
        // The second output's parent is a regular file, so directory
        // creation (and therefore staging) must fail.
        let blocker = dir.path().join("blocker");
        std::fs::write(&blocker, b"file").unwrap();
        let second = blocker.join("second.txt");

        let error = publish_batch(&[
            asset("first", &first, b"new"),
            asset("second", &second, b"payload"),
        ])
        .unwrap_err();

        assert!(matches!(
            error,
            BatchPublicationError::Stage { ref role, .. } if role == "second"
        ));
        assert_eq!(std::fs::read(&first).unwrap(), b"old");
        assert_eq!(staged_leftovers(dir.path()), [] as [std::string::String; 0]);
    }

    #[test]
    fn a_late_publish_failure_is_reported_and_repairable() {
        let dir = tempfile::tempdir().unwrap();
        let first = dir.path().join("first.txt");
        // A directory occupying the second destination defeats the
        // final rename only — staging succeeds, so the first member is
        // already published: exactly the documented residual window.
        let second = dir.path().join("second.txt");
        std::fs::create_dir(&second).unwrap();

        let error = publish_batch(&[
            asset("first", &first, b"one"),
            asset("second", &second, b"two"),
        ])
        .unwrap_err();

        assert!(matches!(
            error,
            BatchPublicationError::Publish { ref role, .. } if role == "second"
        ));
        assert_eq!(std::fs::read(&first).unwrap(), b"one");

        // A subsequent successful run repairs the complete set.
        std::fs::remove_dir(&second).unwrap();
        let results = publish_batch(&[
            asset("first", &first, b"one"),
            asset("second", &second, b"two"),
        ])
        .unwrap();

        assert_eq!(
            results
                .iter()
                .map(|result| result.change.bytes_changed)
                .collect::<Vec<_>>(),
            [false, true]
        );
        assert_eq!(std::fs::read(&second).unwrap(), b"two");
        assert_eq!(staged_leftovers(dir.path()), [] as [std::string::String; 0]);
    }

    #[test]
    fn a_prior_mixed_generation_is_repaired() {
        let dir = tempfile::tempdir().unwrap();
        let fresh = dir.path().join("fresh.txt");
        let stale = dir.path().join("stale.txt");
        write_public(&fresh, b"generation-2");
        write_public(&stale, b"generation-1");

        let results = publish_batch(&[
            asset("fresh", &fresh, b"generation-2"),
            asset("stale", &stale, b"generation-2"),
        ])
        .unwrap();

        assert_eq!(
            results
                .iter()
                .map(|result| result.change.bytes_changed)
                .collect::<Vec<_>>(),
            [false, true]
        );
        assert_eq!(std::fs::read(&stale).unwrap(), b"generation-2");
    }

    // --- SR3-04: published files carry the intended mode ---
    //
    // Staging creates an owner-only temporary, so without an explicit
    // mode the renamed publication would become 0o600. Git records only
    // the executable bit, so nothing downstream would notice.

    #[cfg(unix)]
    fn mode_of(path: &Path) -> u32 {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777
    }

    #[cfg(unix)]
    #[test]
    fn publication_modes_name_their_octal_contract() {
        use crate::PublicationMode;

        assert_eq!(PublicationMode::Public.octal(), 0o644);
        assert_eq!(PublicationMode::Executable.octal(), 0o755);
    }

    #[cfg(unix)]
    #[test]
    fn an_absent_destination_is_published_publicly_readable() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("out.txt");

        let results = publish_batch(&[asset("out", &out, b"data")]).unwrap();

        assert!(results[0].change.bytes_changed);
        assert_eq!(mode_of(&out), 0o644, "a new publication must be public");
    }

    #[cfg(unix)]
    #[test]
    fn a_changed_public_destination_stays_publicly_readable() {
        use std::os::unix::fs::PermissionsExt as _;

        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("out.txt");
        std::fs::write(&out, b"old").unwrap();
        std::fs::set_permissions(&out, std::fs::Permissions::from_mode(0o644)).unwrap();

        let results = publish_batch(&[asset("out", &out, b"new")]).unwrap();

        assert!(results[0].change.bytes_changed);
        assert_eq!(std::fs::read(&out).unwrap(), b"new");
        assert_eq!(
            mode_of(&out),
            0o644,
            "republishing must not narrow an existing public destination"
        );
    }

    #[cfg(unix)]
    #[test]
    fn an_unchanged_public_destination_keeps_its_mode() {
        use std::os::unix::fs::PermissionsExt as _;

        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("out.txt");
        std::fs::write(&out, b"same").unwrap();
        std::fs::set_permissions(&out, std::fs::Permissions::from_mode(0o644)).unwrap();

        let results = publish_batch(&[asset("out", &out, b"same")]).unwrap();

        assert!(results[0].change.unchanged());
        assert_eq!(mode_of(&out), 0o644);
    }

    #[cfg(unix)]
    #[test]
    fn an_executable_publication_mode_is_applied_to_a_staged_file() {
        use crate::{PublicationMode, set_publication_mode};

        let dir = tempfile::tempdir().unwrap();
        let staged = dir.path().join("tool");
        let file = std::fs::File::create(&staged).unwrap();

        set_publication_mode(&file, PublicationMode::Executable).unwrap();
        drop(file);

        assert_eq!(mode_of(&staged), 0o755);
    }

    #[test]
    fn missing_destinations_and_parents_are_created() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("deep").join("out.txt");

        let results = publish_batch(&[asset("out", &nested, b"data")]).unwrap();

        assert!(results[0].change.bytes_changed);
        assert_eq!(std::fs::read(&nested).unwrap(), b"data");
    }

    // --- R2-N04: publication mode is part of compare-if-changed
    // freshness ---
    //
    // Byte equality alone left a destination stuck at a wrong mode
    // forever: every run saw equal bytes and skipped the repair. The
    // rows below are the review's freshness matrix.

    #[cfg(unix)]
    #[test]
    fn an_owner_only_destination_with_current_bytes_is_mode_repaired() {
        use std::os::unix::fs::PermissionsExt as _;

        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("out.txt");
        std::fs::write(&out, b"same").unwrap();
        std::fs::set_permissions(&out, std::fs::Permissions::from_mode(0o600)).unwrap();
        let mtime = std::fs::metadata(&out).unwrap().modified().unwrap();

        let results = publish_batch(&[asset("out", &out, b"same")]).unwrap();

        assert!(
            !results[0].change.bytes_changed,
            "a mode repair must not rewrite content"
        );
        assert!(results[0].change.mode_changed, "the repair must be visible");
        assert_eq!(mode_of(&out), 0o644);
        assert_eq!(std::fs::read(&out).unwrap(), b"same");
        assert_eq!(
            std::fs::metadata(&out).unwrap().modified().unwrap(),
            mtime,
            "a mode-only repair must leave the bytes' mtime alone"
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_run_after_a_mode_repair_is_a_no_op() {
        use std::os::unix::fs::PermissionsExt as _;

        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("out.txt");
        std::fs::write(&out, b"same").unwrap();
        std::fs::set_permissions(&out, std::fs::Permissions::from_mode(0o600)).unwrap();

        let repaired = publish_batch(&[asset("out", &out, b"same")]).unwrap();
        assert!(repaired[0].change.mode_changed);

        let again = publish_batch(&[asset("out", &out, b"same")]).unwrap();

        assert!(
            again[0].change.unchanged(),
            "a repaired destination is current in both components"
        );
        assert_eq!(mode_of(&out), 0o644);
        assert_eq!(staged_leftovers(dir.path()), [] as [std::string::String; 0]);
    }

    #[cfg(unix)]
    #[test]
    fn stale_bytes_are_republished_rather_than_mode_repaired() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("out.txt");
        write_public(&out, b"old");

        let results = publish_batch(&[asset("out", &out, b"new")]).unwrap();

        assert!(results[0].change.bytes_changed);
        assert!(
            !results[0].change.mode_changed,
            "a republished member carries its mode from staging, not from a repair"
        );
        assert_eq!(mode_of(&out), 0o644);
    }

    #[cfg(unix)]
    #[test]
    fn an_executable_destination_missing_its_x_bit_is_not_current() {
        use std::os::unix::fs::PermissionsExt as _;

        use crate::PublicationMode;
        use crate::publication::{DestinationState, inspect_destination, repair_publication_mode};

        let dir = tempfile::tempdir().unwrap();
        let tool = dir.path().join("tool");
        std::fs::write(&tool, b"binary").unwrap();
        std::fs::set_permissions(&tool, std::fs::Permissions::from_mode(0o644)).unwrap();

        assert_eq!(
            inspect_destination(&tool, b"binary", PublicationMode::Executable),
            DestinationState::ModeOnly,
            "equal bytes without the executable bit are not a current publication"
        );

        repair_publication_mode(&tool, PublicationMode::Executable).unwrap();

        assert_eq!(mode_of(&tool), 0o755);
        assert_eq!(std::fs::read(&tool).unwrap(), b"binary");
        assert_eq!(
            inspect_destination(&tool, b"binary", PublicationMode::Executable),
            DestinationState::Current
        );
    }

    #[cfg(unix)]
    #[test]
    fn an_absent_destination_is_stale_in_every_publication_class() {
        use crate::PublicationMode;
        use crate::publication::{DestinationState, inspect_destination};

        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing");

        for mode in [PublicationMode::Public, PublicationMode::Executable] {
            assert_eq!(
                inspect_destination(&missing, b"bytes", mode),
                DestinationState::StaleBytes
            );
        }
    }

    // --- R2-N04: the JSON checker report shares the freshness rule ---

    #[cfg(unix)]
    mod report {
        use std::os::unix::fs::PermissionsExt as _;

        use super::mode_of;
        use crate::write_json_report_if_changed;

        fn value() -> serde_json::Value {
            serde_json::json!({"ok": true})
        }

        #[test]
        fn an_absent_report_is_published_publicly_readable() {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("report.json");

            let change = write_json_report_if_changed(&path, &value()).unwrap();

            assert!(change.bytes_changed);
            assert!(!change.mode_changed);
            assert_eq!(mode_of(&path), 0o644);
        }

        #[test]
        fn a_current_report_with_a_correct_mode_is_untouched() {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("report.json");
            write_json_report_if_changed(&path, &value()).unwrap();
            let mtime = std::fs::metadata(&path).unwrap().modified().unwrap();

            let change = write_json_report_if_changed(&path, &value()).unwrap();

            assert!(change.unchanged());
            assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), mtime);
        }

        #[test]
        fn a_stale_report_is_rewritten() {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("report.json");
            std::fs::write(&path, b"{\"ok\":false}\n").unwrap();
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();

            let change = write_json_report_if_changed(&path, &value()).unwrap();

            assert!(change.bytes_changed);
            assert!(!change.mode_changed);
            assert_eq!(std::fs::read(&path).unwrap(), b"{\"ok\":true}\n");
        }

        #[test]
        fn an_owner_only_current_report_is_mode_repaired_without_rewriting() {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("report.json");
            write_json_report_if_changed(&path, &value()).unwrap();
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
            let mtime = std::fs::metadata(&path).unwrap().modified().unwrap();

            let change = write_json_report_if_changed(&path, &value()).unwrap();

            assert!(!change.bytes_changed);
            assert!(change.mode_changed);
            assert_eq!(mode_of(&path), 0o644);
            assert_eq!(std::fs::read(&path).unwrap(), b"{\"ok\":true}\n");
            assert_eq!(
                std::fs::metadata(&path).unwrap().modified().unwrap(),
                mtime,
                "a mode-only report repair must not rewrite the report"
            );

            let again = write_json_report_if_changed(&path, &value()).unwrap();
            assert!(again.unchanged(), "the run after a repair is a no-op");
        }
    }
}
