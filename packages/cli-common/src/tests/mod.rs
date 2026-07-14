//! # CLI-common tests
//!
//! | Test                                  | What it covers                                   |
//! |---------------------------------------|--------------------------------------------------|
//! | `emit_writes_one_json_line`           | `emit` produces compact JSON + trailing newline. |
//! | `emit_propagates_serialisation_error` | A non-`Serialize`-friendly value surfaces `Err`. |
//! | `emit_propagates_writer_error`        | A failing writer surfaces `Err` (broken pipe).   |
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
//! | `usage_error_text_never_reproduces_secrets` | Clap echo of secret argv is sanitized.      |
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
    let record = ControlPlaneRecord::usage_error("fixture", "unknown argument", "unknown_argument");

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
    assert!(exit.record.message.contains("--no-such-flag"));

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
