#![doc = include_str!("../README.md")]

use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::future::Future;
use std::io::{self, IsTerminal, Write};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, TryLockError};

use clap::{CommandFactory, Parser};
use serde::{Deserialize, Serialize};
use tracing_subscriber::fmt::MakeWriter;

mod publication;

pub use publication::{
    BatchPublicationError, PublicationAsset, PublicationChange, PublicationMode, PublicationResult,
    publish_batch, set_publication_mode,
};

#[cfg(test)]
mod tests;

/// Schema version for shared ADR-010 control-plane records.
pub const CONTROL_PLANE_SCHEMA: u32 = 1;

/// Placeholder used when a diagnostic value is intentionally hidden.
pub const REDACTED: &str = "[redacted]";

static PANIC_REPORTED: AtomicBool = AtomicBool::new(false);
// Fail closed: panic payloads may contain secrets, so they are omitted
// until a parsed `--debug` flag explicitly enables them (ADR-010). A
// panic before debug status is known therefore reports thread and
// location but no payload.
static PANIC_PAYLOAD_REPORTING_ENABLED: AtomicBool = AtomicBool::new(false);
static STDERR_WRITE_LOCK: Mutex<()> = Mutex::new(());

/// Baseline exit-code classes shared by ADR-010 command-line tools.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommandExit {
    /// Successful completion.
    Success,
    /// Runtime, startup, internal, or command execution failure.
    Failure,
    /// Command-line usage failure, including clap errors and TTY refusal.
    Usage,
}

impl CommandExit {
    /// Return the numeric process status for this exit class.
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::Success => 0,
            Self::Failure => 1,
            Self::Usage => 2,
        }
    }

    /// Return this class as a standard library [`ExitCode`].
    #[must_use]
    pub fn exit_code(self) -> ExitCode {
        ExitCode::from(self.code())
    }

    /// Map a numeric status back to a shared exit class.
    #[must_use]
    pub const fn from_code(code: u8) -> Option<Self> {
        match code {
            0 => Some(Self::Success),
            1 => Some(Self::Failure),
            2 => Some(Self::Usage),
            _ => None,
        }
    }
}

/// JSON record kinds emitted on stderr as ADR-010 control-plane data.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ControlPlaneRecordKind {
    /// Help text requested by the caller.
    Help,
    /// Version information requested by the caller.
    Version,
    /// Command-line usage or argv parsing failure.
    UsageError,
    /// Refusal to write stdout result data directly to a terminal.
    TtyRefusal,
    /// Panic diagnostic emitted by the shared panic hook.
    Panic,
    /// Non-error status update.
    Status,
    /// Runtime diagnostic or error.
    Diagnostic,
}

/// Standard streams referenced by control-plane records.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StandardStream {
    /// Standard output.
    Stdout,
    /// Standard error.
    Stderr,
}

/// Structured fields attached to a control-plane record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ControlPlaneFields {
    /// Help text returned by clap or a command-specific renderer.
    Help { text: String },
    /// Version string returned by clap or the command.
    Version { version: String },
    /// Details for an argv parsing or usage error.
    UsageError {
        exit_code: u8,
        clap_error_kind: String,
    },
    /// Details for stdout TTY refusal.
    TtyRefusal {
        exit_code: u8,
        stream: StandardStream,
    },
    /// Details for a panic diagnostic. The panic payload is only exposed when debug diagnostics are enabled.
    Panic {
        exit_code: u8,
        thread: Option<String>,
        location: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        payload: Option<String>,
    },
}

/// Shared JSON control-plane record written to stderr.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ControlPlaneRecord {
    /// Record schema version.
    pub schema: u32,
    /// Binary or entrypoint name.
    pub command: String,
    /// Machine-readable record kind.
    pub kind: ControlPlaneRecordKind,
    /// Short human-readable message, still carried inside JSON.
    pub message: String,
    /// Kind-specific structured fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<ControlPlaneFields>,
}

impl ControlPlaneRecord {
    /// Build a generic control-plane record.
    #[must_use]
    pub fn new(
        command: &str,
        kind: ControlPlaneRecordKind,
        message: &str,
        fields: Option<ControlPlaneFields>,
    ) -> Self {
        Self {
            schema: CONTROL_PLANE_SCHEMA,
            command: command.to_string(),
            kind,
            message: message.to_string(),
            fields,
        }
    }

    /// Build a help record.
    #[must_use]
    pub fn help(command: &str, text: &str) -> Self {
        Self::new(
            command,
            ControlPlaneRecordKind::Help,
            "help requested",
            Some(ControlPlaneFields::Help {
                text: text.to_string(),
            }),
        )
    }

    /// Build a version record.
    #[must_use]
    pub fn version(command: &str, version: &str) -> Self {
        Self::new(
            command,
            ControlPlaneRecordKind::Version,
            "version requested",
            Some(ControlPlaneFields::Version {
                version: version.to_string(),
            }),
        )
    }

    /// Build an argv usage-error record.
    ///
    /// The message is fixed here, not caller-supplied: a usage record
    /// carries no argument values (ADR-010), and argument text may
    /// embed inline secrets. The machine-readable detail is the error
    /// class alone.
    #[must_use]
    pub fn usage_error(command: &str, clap_error_kind: &str) -> Self {
        Self::new(
            command,
            ControlPlaneRecordKind::UsageError,
            "invalid command-line arguments",
            Some(ControlPlaneFields::UsageError {
                exit_code: CommandExit::Usage.code(),
                clap_error_kind: clap_error_kind.to_string(),
            }),
        )
    }

    /// Build a stdout TTY-refusal record.
    #[must_use]
    pub fn tty_refusal(command: &str) -> Self {
        Self::new(
            command,
            ControlPlaneRecordKind::TtyRefusal,
            "stdout is a terminal; pipe to a file or another process",
            Some(ControlPlaneFields::TtyRefusal {
                exit_code: CommandExit::Usage.code(),
                stream: StandardStream::Stdout,
            }),
        )
    }

    /// Build a panic diagnostic record.
    #[must_use]
    pub fn panic(
        command: &str,
        thread: Option<&str>,
        location: Option<&str>,
        payload: Option<&str>,
    ) -> Self {
        Self::new(
            command,
            ControlPlaneRecordKind::Panic,
            "unexpected panic",
            Some(ControlPlaneFields::Panic {
                exit_code: CommandExit::Failure.code(),
                thread: thread.map(str::to_string),
                location: location.map(str::to_string),
                payload: payload.map(str::to_string),
            }),
        )
    }

    /// Build a non-error status record.
    #[must_use]
    pub fn status(command: &str, message: &str) -> Self {
        Self::new(command, ControlPlaneRecordKind::Status, message, None)
    }

    /// Build a runtime diagnostic record.
    #[must_use]
    pub fn diagnostic(command: &str, message: &str) -> Self {
        Self::new(command, ControlPlaneRecordKind::Diagnostic, message, None)
    }
}

/// Control-plane record and exit class produced while parsing argv.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliExit {
    /// Record to write to stderr.
    pub record: ControlPlaneRecord,
    /// Process exit class to use after writing the record.
    pub exit: CommandExit,
}

impl CliExit {
    /// Build a new parse-control exit value.
    #[must_use]
    pub const fn new(record: ControlPlaneRecord, exit: CommandExit) -> Self {
        Self { record, exit }
    }

    /// Return this exit class as a standard library [`ExitCode`].
    #[must_use]
    pub fn exit_code(&self) -> ExitCode {
        self.exit.exit_code()
    }
}

/// Source used to choose the JSON tracing filter directive.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TracingFilterSource {
    /// The `RUST_LOG` environment variable supplied the directive.
    RustLog,
    /// The shared `--debug` flag selected debug logging.
    DebugFlag,
    /// The caller-supplied default level was used.
    DefaultLevel,
}

/// Resolved JSON tracing filter directive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TracingFilter {
    /// Directive passed to `tracing-subscriber`'s environment filter.
    pub directive: String,
    /// Input source that selected the directive.
    pub source: TracingFilterSource,
}

impl TracingFilter {
    /// Build a resolved tracing filter directive.
    #[must_use]
    pub fn new(directive: impl Into<String>, source: TracingFilterSource) -> Self {
        Self {
            directive: directive.into(),
            source,
        }
    }
}

/// Return true when a diagnostic field name is likely to contain a secret.
#[must_use]
pub fn is_sensitive_field_name(field_name: &str) -> bool {
    const SENSITIVE_MARKERS: &[&str] = &[
        "admin_token",
        "api_key",
        "apikey",
        "credential",
        "credentials",
        "jwt_secret",
        "mailer_password",
        "passwd",
        "password",
        "private_key",
        "pwd",
        "secret",
        "session_secret",
        "smtp_password",
        "token",
    ];

    let normalized = normalize_name(field_name);
    SENSITIVE_MARKERS
        .iter()
        .any(|marker| normalized.contains(marker))
}

/// Redact a diagnostic field value according to the ADR-010 policy.
#[must_use]
pub fn redact_field_value<'a>(field_name: &str, value: &'a str) -> Cow<'a, str> {
    if is_sensitive_field_name(field_name) {
        Cow::Borrowed(REDACTED)
    } else {
        redact_url(value)
    }
}

/// Redact credentials and secret query parameters from a database URL.
///
/// Retained name; delegates to the scheme-agnostic [`redact_url`].
#[must_use]
pub fn redact_database_url(value: &str) -> Cow<'_, str> {
    redact_url(value)
}

/// Redact userinfo credentials and secret query parameters from a URL
/// of any scheme. Non-URL values are returned unchanged; safe host,
/// path, and query data are preserved.
///
/// ADR-010 forbids credential-bearing URLs on JSON stderr regardless
/// of scheme: an HTTPS endpoint or custom RPC URL can carry secrets
/// exactly like a database URL.
///
/// Fragments are treated like query strings: OAuth-style fragments
/// (`#access_token=...&state=...`) are split on `&` and any piece
/// whose key is sensitive is dropped, keeping the safe pieces.
#[must_use]
pub fn redact_url(value: &str) -> Cow<'_, str> {
    let Ok(mut url) = url::Url::parse(value) else {
        return Cow::Borrowed(value);
    };

    let username_changed = !url.username().is_empty() && url.set_username("").is_ok();
    let password_changed = url.password().is_some() && url.set_password(None).is_ok();

    let mut safe_query_pairs = Vec::new();
    let mut removed_query_pair = false;
    for (key, query_value) in url.query_pairs() {
        if is_sensitive_query_key(&key) {
            removed_query_pair = true;
        } else {
            safe_query_pairs.push((key.into_owned(), query_value.into_owned()));
        }
    }

    if removed_query_pair {
        url.set_query(None);
        if !safe_query_pairs.is_empty() {
            let mut query_pairs = url.query_pairs_mut();
            for (key, query_value) in safe_query_pairs {
                query_pairs.append_pair(&key, &query_value);
            }
        }
    }

    // OAuth-style implicit-grant responses put credentials in the
    // fragment (`#access_token=...`), which `query_pairs` never sees.
    let mut removed_fragment_piece = false;
    if let Some(fragment) = url.fragment().map(str::to_owned)
        && !fragment.is_empty()
    {
        let mut pieces: Vec<&str> = fragment.split('&').collect();
        let original_piece_count = pieces.len();
        pieces.retain(|piece| {
            let key = piece.split_once('=').map_or(*piece, |(key, _value)| key);
            !is_sensitive_query_key(key)
        });
        if pieces.len() < original_piece_count {
            removed_fragment_piece = true;
            if pieces.is_empty() {
                url.set_fragment(None);
            } else {
                url.set_fragment(Some(&pieces.join("&")));
            }
        }
    }

    let changed =
        username_changed || password_changed || removed_query_pair || removed_fragment_piece;

    if changed {
        Cow::Owned(url.to_string())
    } else {
        Cow::Borrowed(value)
    }
}

/// Redact secrets from free-form diagnostic text (for example a clap
/// error message that reproduces the offending argument).
///
/// Two token shapes are scrubbed, whitespace preserved:
///
/// - `key=value` tokens (including `--key=value`) whose key is
///   sensitive per [`is_sensitive_field_name`] have the value
///   replaced; non-sensitive keys with URL-shaped values (such as
///   `--endpoint=https://user:pw@host/`) have the value redacted;
/// - URL-shaped tokens (containing `://`) have their URL substring
///   passed through [`redact_url`], failing closed to `[redacted]`
///   when the substring does not parse as a URL.
///
/// This is conservative sanitization of untrusted echo text, not an
/// escape hatch: structured records should still prefer known-safe
/// fields over free text.
#[must_use]
pub fn redact_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut token = String::new();

    for character in text.chars() {
        if character.is_whitespace() {
            if !token.is_empty() {
                out.push_str(&redact_text_token(&token));
                token.clear();
            }
            out.push(character);
        } else {
            token.push(character);
        }
    }

    if !token.is_empty() {
        out.push_str(&redact_text_token(&token));
    }

    out
}

fn redact_text_token(token: &str) -> Cow<'_, str> {
    // Peel surrounding quotes/brackets that clap and error chains wrap
    // around echoed values, so the inner value is still recognized.
    const WRAPPERS: &[char] = &['\'', '"', '`', '(', ')', '<', '>', '[', ']', ',', ';'];

    let trimmed = token.trim_matches(WRAPPERS);
    if trimmed.is_empty() {
        return Cow::Borrowed(token);
    }

    let prefix_len = token.find(trimmed).unwrap_or(0);
    let prefix = &token[..prefix_len];
    let suffix = &token[prefix_len + trimmed.len()..];

    let scheme_separator = trimmed.find("://");

    // `key=value` tokens are recognised only when the '=' sits left of
    // any "://", so a URL query string (`?a=b`) is not mistaken for a
    // key=value token, while `--endpoint=https://user:pw@host/` is.
    if let Some((key, value)) = trimmed.split_once('=')
        && scheme_separator.is_none_or(|separator| key.len() < separator)
    {
        let name = key.trim_start_matches('-');
        if !value.is_empty() && is_sensitive_field_name(name) {
            return Cow::Owned(format!("{prefix}{key}={REDACTED}{suffix}"));
        }
        if let Cow::Owned(redacted) = redact_url_occurrence(value) {
            return Cow::Owned(format!("{prefix}{key}={redacted}{suffix}"));
        }
        return Cow::Borrowed(token);
    }

    if scheme_separator.is_some()
        && let Cow::Owned(redacted) = redact_url_occurrence(trimmed)
    {
        return Cow::Owned(format!("{prefix}{redacted}{suffix}"));
    }

    Cow::Borrowed(token)
}

/// Redact a URL embedded at any position inside `text`.
///
/// Locates the first `://`, scans backward over scheme characters to
/// the scheme start, and applies [`redact_url`] to the substring from
/// there to the end of `text`, splicing any prefix back on. If the
/// substring does not parse as a URL this fails closed (ADR-010) and
/// replaces it with [`REDACTED`]: an unparseable URL-shaped value may
/// still embed credentials verbatim.
fn redact_url_occurrence(text: &str) -> Cow<'_, str> {
    let Some(separator) = text.find("://") else {
        return Cow::Borrowed(text);
    };

    // Scheme characters are ASCII (RFC 3986: ALPHA / DIGIT / "+" /
    // "-" / "."), so a byte-wise backward scan stays on UTF-8
    // boundaries.
    let bytes = text.as_bytes();
    let mut scheme_start = separator;
    while scheme_start > 0 {
        let byte = bytes[scheme_start - 1];
        if byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'-' | b'.') {
            scheme_start -= 1;
        } else {
            break;
        }
    }

    let prefix = &text[..scheme_start];
    let candidate = &text[scheme_start..];

    if url::Url::parse(candidate).is_err() {
        return Cow::Owned(format!("{prefix}{REDACTED}"));
    }

    match redact_url(candidate) {
        Cow::Owned(redacted) => Cow::Owned(format!("{prefix}{redacted}")),
        Cow::Borrowed(_) => Cow::Borrowed(text),
    }
}

fn normalize_name(name: &str) -> String {
    let mut normalized = String::with_capacity(name.len());
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_lowercase());
        } else {
            normalized.push('_');
        }
    }
    normalized
}

fn is_sensitive_query_key(key: &str) -> bool {
    let normalized = normalize_name(key);
    normalized == "key" || normalized.ends_with("_key") || is_sensitive_field_name(key)
}

/// Resolve the JSON tracing filter with ADR-010 precedence.
///
/// `RUST_LOG` wins when set and non-empty. Otherwise `--debug` selects `debug`,
/// and the caller-supplied default level is used last.
#[must_use]
pub fn tracing_filter(debug: bool, default_level: tracing::Level) -> TracingFilter {
    tracing_filter_from_rust_log(
        std::env::var_os("RUST_LOG").as_deref(),
        debug,
        default_level,
    )
}

fn tracing_filter_from_rust_log(
    rust_log: Option<&OsStr>,
    debug: bool,
    default_level: tracing::Level,
) -> TracingFilter {
    if let Some(value) = rust_log {
        let directive = value.to_string_lossy();
        let trimmed = directive.trim();
        if !trimmed.is_empty() {
            return TracingFilter::new(trimmed, TracingFilterSource::RustLog);
        }
    }

    if debug {
        TracingFilter::new("debug", TracingFilterSource::DebugFlag)
    } else {
        TracingFilter::new(
            level_directive(default_level),
            TracingFilterSource::DefaultLevel,
        )
    }
}

fn level_directive(level: tracing::Level) -> String {
    level.as_str().to_ascii_lowercase()
}

/// Parse argv with `clap`, converting help, version, and usage errors into
/// ADR-010 JSON control-plane records.
///
/// # Errors
///
/// Returns [`CliExit`] when parsing should stop and the caller should write the
/// enclosed record to stderr before exiting with the enclosed exit class.
/// The error value is intentionally returned by value because this path is only
/// used when parsing stops, and boxing it would complicate the public helper API.
#[allow(clippy::result_large_err)]
pub fn parse_args_from<T, I, A>(args: I) -> Result<T, CliExit>
where
    T: Parser,
    I: IntoIterator<Item = A>,
    A: Into<OsString> + Clone,
{
    T::try_parse_from(args).map_err(|clap_error| cli_exit_from_clap_error::<T>(&clap_error))
}

/// Parse process argv with `clap`, writing ADR-010 control-plane records and
/// exiting when parsing is a help, version, or usage-control path.
#[must_use]
pub fn parse_args_or_exit<T>() -> T
where
    T: Parser,
{
    match parse_args_from::<T, _, _>(std::env::args_os()) {
        Ok(args) => args,
        Err(exit) => {
            let _ignored = emit_control_plane_record(&exit.record);
            exit_with(exit.exit);
        }
    }
}

fn cli_exit_from_clap_error<T>(clap_error: &clap::Error) -> CliExit
where
    T: CommandFactory,
{
    let command_name = T::command().get_name().to_string();
    let text = clap_error.to_string().trim_end().to_string();

    match clap_error.kind() {
        clap::error::ErrorKind::DisplayHelp => CliExit::new(
            ControlPlaneRecord::help(&command_name, &text),
            CommandExit::Success,
        ),
        clap::error::ErrorKind::DisplayVersion => CliExit::new(
            ControlPlaneRecord::version(&command_name, &text),
            CommandExit::Success,
        ),
        // Usage errors happen before the early-startup boundary, so the
        // record omits argument values wholesale instead of relying on
        // heuristic redaction: clap's rendered text reproduces the
        // offending argument, which may embed an inline secret
        // (`--api-token=...`, a credential URL). Help/version text is
        // generated from the static command definition and stays verbatim.
        _ => CliExit::new(
            ControlPlaneRecord::usage_error(
                &command_name,
                &clap_error_kind_name(clap_error.kind()),
            ),
            CommandExit::Usage,
        ),
    }
}

fn clap_error_kind_name(kind: clap::error::ErrorKind) -> String {
    debug_name_to_snake_case(&format!("{kind:?}"))
}

fn debug_name_to_snake_case(name: &str) -> String {
    let mut normalized = String::with_capacity(name.len());
    let mut previous_was_lower_or_digit = false;

    for character in name.chars() {
        if character.is_ascii_uppercase() {
            if !normalized.is_empty() && previous_was_lower_or_digit {
                normalized.push('_');
            }
            normalized.push(character.to_ascii_lowercase());
            previous_was_lower_or_digit = false;
        } else {
            normalized.push(character);
            previous_was_lower_or_digit =
                character.is_ascii_lowercase() || character.is_ascii_digit();
        }
    }

    normalized
}

/// Write one ADR-010 control-plane JSON record to stderr.
///
/// This helper does not depend on `tracing`, so it is safe for clap help,
/// clap parse errors, early startup failures, TTY refusal, and panic hooks.
///
/// # Errors
///
/// Returns an error if serialisation or writing to stderr fails.
pub fn emit_control_plane_record(record: &ControlPlaneRecord) -> io::Result<()> {
    let mut writer = LockedStderrWriter::new();
    write_json_line(&mut writer, record)
}

fn try_emit_control_plane_record(record: &ControlPlaneRecord) -> io::Result<bool> {
    let Some(mut writer) = LockedStderrWriter::try_new() else {
        return Ok(false);
    };

    write_json_line(&mut writer, record)?;
    Ok(true)
}

fn write_json_line<W: Write, T: serde::Serialize>(writer: &mut W, value: &T) -> io::Result<()> {
    let json = serde_json::to_string(value)?;
    writer.write_all(json.as_bytes())?;
    writer.write_all(b"\n")?;
    writer.flush()
}

/// Install a JSON-only panic hook for ADR-010 command-line entrypoints.
///
/// The hook emits one best-effort control-plane record on stderr and terminates
/// the process with exit code 1 without invoking Rust's default text panic hook.
pub fn install_json_panic_hook(command_name: &str) {
    let command_name = command_name.to_string();
    std::panic::set_hook(Box::new(move |panic_info| {
        if !PANIC_REPORTED.swap(true, Ordering::SeqCst) {
            let current_thread = std::thread::current();
            let thread_name = current_thread.name();
            let location = panic_info.location().map(|location| {
                format!(
                    "{}:{}:{}",
                    location.file(),
                    location.line(),
                    location.column()
                )
            });
            let payload = panic_payload_reporting_enabled()
                .then(|| panic_payload_message(panic_info))
                .flatten();
            let record =
                ControlPlaneRecord::panic(&command_name, thread_name, location.as_deref(), payload);
            let _ignored = try_emit_control_plane_record(&record);
        }

        exit_with(CommandExit::Failure);
    }));
}

/// Enable or disable string panic payloads in JSON panic diagnostics.
///
/// Payload reporting starts **disabled** (fail closed: payloads may
/// contain secrets), so a panic before argument parsing omits its
/// payload. Call this with the parsed `--debug` value once arguments
/// are available.
pub fn set_panic_payload_reporting_enabled(enabled: bool) {
    PANIC_PAYLOAD_REPORTING_ENABLED.store(enabled, Ordering::SeqCst);
}

fn panic_payload_reporting_enabled() -> bool {
    PANIC_PAYLOAD_REPORTING_ENABLED.load(Ordering::SeqCst)
}

fn panic_payload_message<'a>(info: &'a std::panic::PanicHookInfo<'_>) -> Option<&'a str> {
    panic_payload_message_from_payload(info.payload())
}

fn panic_payload_message_from_payload(payload: &(dyn std::any::Any + Send)) -> Option<&str> {
    payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
}

/// Exit the current process with an ADR-010 exit class.
#[allow(clippy::exit)]
pub fn exit_with(exit: CommandExit) -> ! {
    std::process::exit(i32::from(exit.code()));
}

/// Return a TTY-refusal record when stdout is attached to a terminal.
#[must_use]
pub fn stdout_tty_refusal_record(command_name: &str) -> Option<ControlPlaneRecord> {
    if io::stdout().is_terminal() {
        Some(ControlPlaneRecord::tty_refusal(command_name))
    } else {
        None
    }
}

/// Refuse to run if stdout is a terminal (ADR-010).
///
/// Emits a JSON control-plane record on stderr and exits with code 2.
pub fn refuse_if_stdout_is_tty(binary_name: &str) {
    if let Some(record) = stdout_tty_refusal_record(binary_name) {
        let _ignored = emit_control_plane_record(&record);
        exit_with(CommandExit::Usage);
    }
}

/// Initialise `tracing-subscriber` with JSON output on stderr.
pub fn init_json_tracing(level: tracing::Level) {
    let _installed = try_init_json_tracing(level);
}

/// Try to initialise `tracing-subscriber` with JSON output on stderr.
///
/// Returns `false` if another subscriber is already installed.
#[must_use]
pub fn try_init_json_tracing(level: tracing::Level) -> bool {
    let filter = tracing_filter(false, level);
    try_init_json_tracing_with_directive(&filter.directive)
}

/// Initialise JSON stderr tracing with `RUST_LOG` / `--debug` precedence.
pub fn init_json_tracing_with_debug(debug: bool, default_level: tracing::Level) {
    let _installed = try_init_json_tracing_with_debug(debug, default_level);
}

/// Try to initialise JSON stderr tracing with `RUST_LOG` / `--debug` precedence.
///
/// Returns `false` if another subscriber is already installed.
#[must_use]
pub fn try_init_json_tracing_with_debug(debug: bool, default_level: tracing::Level) -> bool {
    let filter = tracing_filter(debug, default_level);
    try_init_json_tracing_with_directive(&filter.directive)
}

fn try_init_json_tracing_with_directive(filter_directive: &str) -> bool {
    let fallback_directive = level_directive(tracing::Level::INFO);
    let filter = tracing_subscriber::EnvFilter::try_new(filter_directive)
        .unwrap_or_else(|_error| tracing_subscriber::EnvFilter::new(fallback_directive));

    tracing_subscriber::fmt()
        .json()
        .with_env_filter(filter)
        .with_writer(LockedStderr)
        .try_init()
        .is_ok()
}

/// Serialise `value` as one JSON object + trailing newline to stdout.
///
/// # Errors
///
/// Returns an error if serialisation or writing to stdout fails.
pub fn emit<T: serde::Serialize>(value: &T) -> io::Result<()> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    write_json_line(&mut out, value)
}

/// Run a stdout-producing single-JSON-object command.
///
/// The runner installs the JSON panic hook, initialises JSON stderr tracing,
/// refuses terminal stdout, writes successful result data to stdout, and maps
/// failures to ADR-010 baseline exit classes.
pub fn run_stdout_json_command<Output, CommandError, Run>(
    command_name: &str,
    debug: bool,
    default_level: tracing::Level,
    run: Run,
) -> ExitCode
where
    Output: serde::Serialize,
    CommandError: std::fmt::Display,
    Run: FnOnce() -> Result<Output, CommandError>,
{
    set_panic_payload_reporting_enabled(debug);
    install_json_panic_hook(command_name);
    init_json_tracing_with_debug(debug, default_level);

    if let Some(record) = stdout_tty_refusal_record(command_name) {
        let _ignored = emit_control_plane_record(&record);
        return CommandExit::Usage.exit_code();
    }

    match run() {
        Ok(output) => match emit(&output) {
            Ok(()) => CommandExit::Success.exit_code(),
            Err(error) => {
                tracing::error!(error = %redact_text(&error.to_string()), "failed to write JSON to stdout");
                CommandExit::Failure.exit_code()
            }
        },
        Err(error) => {
            // ADR-010: arbitrary `Display` errors may embed credential
            // URLs (connection strings, HTTP endpoints), so the display
            // text passes through `redact_text` centrally here instead
            // of trusting each command to sanitise its own errors.
            tracing::error!(error = %redact_text(&error.to_string()), "command failed");
            CommandExit::Failure.exit_code()
        }
    }
}

/// Run a side-effect command that does not emit stdout result data.
///
/// The runner installs the JSON panic hook, initialises JSON stderr tracing, and
/// maps failures to ADR-010 baseline exit classes. It deliberately does not
/// perform stdout TTY refusal because the command has no stdout result data.
pub fn run_no_stdout_command<CommandError, Run>(
    command_name: &str,
    debug: bool,
    default_level: tracing::Level,
    run: Run,
) -> ExitCode
where
    CommandError: std::fmt::Display,
    Run: FnOnce() -> Result<(), CommandError>,
{
    set_panic_payload_reporting_enabled(debug);
    install_json_panic_hook(command_name);
    init_json_tracing_with_debug(debug, default_level);

    match run() {
        Ok(()) => CommandExit::Success.exit_code(),
        Err(error) => {
            // ADR-010: the display text passes through `redact_text`
            // because arbitrary errors may embed credential URLs.
            tracing::error!(error = %redact_text(&error.to_string()), "command failed");
            CommandExit::Failure.exit_code()
        }
    }
}

/// Run an async side-effect command that does not emit stdout result data.
///
/// The runner installs the JSON panic hook, initialises JSON stderr tracing, and
/// maps failures to ADR-010 baseline exit classes. It deliberately does not
/// perform stdout TTY refusal because the command has no stdout result data.
pub async fn run_no_stdout_command_async<CommandError, Run, RunFuture>(
    command_name: &str,
    debug: bool,
    default_level: tracing::Level,
    run: Run,
) -> ExitCode
where
    CommandError: std::fmt::Display,
    Run: FnOnce() -> RunFuture,
    RunFuture: Future<Output = Result<(), CommandError>>,
{
    set_panic_payload_reporting_enabled(debug);
    install_json_panic_hook(command_name);
    init_json_tracing_with_debug(debug, default_level);

    match run().await {
        Ok(()) => CommandExit::Success.exit_code(),
        Err(error) => {
            // ADR-010: the display text passes through `redact_text`
            // because arbitrary errors may embed credential URLs.
            tracing::error!(error = %redact_text(&error.to_string()), "command failed");
            CommandExit::Failure.exit_code()
        }
    }
}

/// Run an ADR-014 checker that produces a JSON report on success.
///
/// Installs the JSON panic hook and stderr tracing, runs the check, and
/// publishes the result through [`finish_check_command`] under the
/// active output mode: direct-mode stdout (refused on a terminal, exit
/// class 2) or a build-mode report asset plus success stamp. The `run`
/// closure must emit its own per-item diagnostics on stderr and return
/// `Err` when the semantic check fails; that maps to the failure exit
/// class and leaves both stdout and the stamp untouched.
pub fn run_check_command<Report, CommandError, Run>(
    command_name: &str,
    debug: bool,
    default_level: tracing::Level,
    output: &CheckOutputArgs,
    run: Run,
) -> ExitCode
where
    Report: serde::Serialize,
    CommandError: std::fmt::Display,
    Run: FnOnce() -> Result<Report, CommandError>,
{
    set_panic_payload_reporting_enabled(debug);
    install_json_panic_hook(command_name);
    init_json_tracing_with_debug(debug, default_level);

    // Direct mode (no --report/--stamp) refuses terminal stdout *before*
    // any semantic work: the JSON result would have nowhere safe to go,
    // so running the check first would be wasted work and could touch
    // the filesystem on the way. Build mode has an explicit report/stamp
    // destination and is exempt. `finish_check_command` still performs
    // its own refusal for direct callers.
    if output.report.is_none()
        && output.stamp.is_none()
        && let Some(record) = stdout_tty_refusal_record(command_name)
    {
        let _ignored = emit_control_plane_record(&record);
        return CommandExit::Usage.exit_code();
    }

    let report = match run() {
        Ok(report) => report,
        Err(error) => {
            // ADR-010: arbitrary errors may embed credential URLs.
            tracing::error!(error = %redact_text(&error.to_string()), "command failed");
            return CommandExit::Failure.exit_code();
        }
    };

    match finish_check_command(command_name, output, &report) {
        Ok(()) => CommandExit::Success.exit_code(),
        Err(CheckResultError::TtyRefusal) => CommandExit::Usage.exit_code(),
        Err(error) => {
            tracing::error!(error = %error, "failed to publish the check result");
            CommandExit::Failure.exit_code()
        }
    }
}

struct LockedStderr;

impl<'writer> MakeWriter<'writer> for LockedStderr {
    type Writer = LockedStderrWriter;

    fn make_writer(&'writer self) -> Self::Writer {
        LockedStderrWriter::new()
    }
}

struct LockedStderrWriter {
    _guard: MutexGuard<'static, ()>,
    stderr: io::Stderr,
}

impl LockedStderrWriter {
    fn new() -> Self {
        Self {
            _guard: STDERR_WRITE_LOCK
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            stderr: io::stderr(),
        }
    }

    fn try_new() -> Option<Self> {
        match STDERR_WRITE_LOCK.try_lock() {
            Ok(guard) => Some(Self {
                _guard: guard,
                stderr: io::stderr(),
            }),
            Err(TryLockError::Poisoned(poisoned)) => Some(Self {
                _guard: poisoned.into_inner(),
                stderr: io::stderr(),
            }),
            Err(TryLockError::WouldBlock) => None,
        }
    }
}

impl Write for LockedStderrWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stderr.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stderr.flush()
    }
}

/// Common `--debug` flag for all helpers. Flatten into each
/// binary's `Args` struct via `#[command(flatten)]`.
#[derive(clap::Args)]
pub struct BaseArgs {
    /// Enable debug-level logging on stderr.
    #[arg(long)]
    pub debug: bool,
}

/// Touch an ADR-014 stamp file: create it empty when absent, update
/// the modification time of an existing empty stamp, and refuse a
/// nonempty one.
///
/// A stamp is empty by contract — the JSON report is a separate asset
/// — so bytes in an existing stamp mean something other than this
/// command wrote it. Refusing (rather than truncating) is
/// non-destructive: the foreign bytes survive for inspection, no
/// fresh success stamp appears, and the build graph keeps the target
/// dirty. A failing command must not call this at all.
///
/// This strict rule governs only checker `--stamp` outputs, whose
/// mtime is a freshness oracle. `build_always_stale` build-dir markers
/// (the generator and cargo-lane wrappers, publication mirrors) are a
/// distinct class: they are never consulted for freshness, so they use
/// plain shell `touch` by design and deliberately do not route here
/// (reviewed as F4-003 — not a policy bypass).
///
/// # Errors
///
/// Returns the underlying I/O error, or an `InvalidData` error for an
/// existing nonempty stamp.
pub fn touch_stamp(path: &std::path::Path) -> io::Result<()> {
    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
    {
        Ok(_created) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            let file = std::fs::File::options().write(true).open(path)?;
            let length = file.metadata()?.len();
            if length > 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "stamp {} holds {length} byte(s); a stamp is empty by contract, \
                         refusing to re-date foreign bytes",
                        path.display(),
                    ),
                ));
            }
            let now = std::time::SystemTime::now();
            file.set_times(
                std::fs::FileTimes::new()
                    .set_accessed(now)
                    .set_modified(now),
            )
        }
        Err(error) => Err(error),
    }
}

/// Output arguments shared by ADR-014 checker binaries.
///
/// A checker runs in one of two modes:
///
/// - **direct** — no `--report`/`--stamp`: the JSON result goes to
///   stdout (refused on a terminal) and no stamp is touched;
/// - **build** — both `--report` and `--stamp`: the result is published
///   as an explicit report asset (compare-if-changed), the stamp is
///   touched only after a successful report write, and stdout stays
///   empty.
///
/// `clap` enforces the reciprocal requirement, so a lone `--report` or
/// `--stamp` is a usage error before the command runs.
#[derive(clap::Args, Debug)]
pub struct CheckOutputArgs {
    /// Build mode: write the JSON result to this explicit report file.
    #[arg(long, value_name = "FILE", requires = "stamp")]
    pub report: Option<std::path::PathBuf>,

    /// Build mode: touch this success stamp after publishing the report.
    #[arg(long, value_name = "FILE", requires = "report")]
    pub stamp: Option<std::path::PathBuf>,
}

/// Identity of one output destination for role-uniqueness validation.
///
/// One lexically normalized absolute path
/// (`[ADR017-rule:path:output-roles]`). Deliberately *not* filesystem
/// canonicalization, symlink resolution, device/inode comparison,
/// hard-link detection, or mount identity: those establish no boundary
/// the repository can hold, because the host may replace or remount a
/// path at any time.
///
/// This is a correctness guard against configuration mistakes — two
/// roles spelled differently for one destination. Callers must not
/// alias distinct output roles through symlinks, hard links, mounts,
/// or namespaces; a hard link is not semantic identity, and
/// first-party outputs are identified by role, path, schema, and
/// bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestinationIdentity(std::path::PathBuf);

/// Compute the destination identity of a possibly not-yet-existing path.
///
/// A relative path is resolved against the process working directory,
/// its documented base. Normalization then removes `.` components and
/// resolves `..` components without consulting the filesystem, so the
/// result is defined for a destination that does not exist yet and is
/// unaffected by what the host has mounted beneath it.
#[must_use]
pub fn destination_identity(path: &std::path::Path) -> DestinationIdentity {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(path)
    };

    let mut normalized = std::path::PathBuf::new();
    for component in absolute.components() {
        match component {
            std::path::Component::CurDir => {}
            // `pop` on a root leaves the root in place, so a `..`
            // chain cannot escape above it.
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other),
        }
    }
    DestinationIdentity(normalized)
}

/// Two output roles named one destination.
#[derive(Debug, thiserror::Error)]
#[error("output roles --{first} and --{second} name the same destination")]
pub struct AliasedOutputs {
    pub first: String,
    pub second: String,
}

/// Reject duplicated destinations among a command's output roles
/// before any semantic work or file mutation: a multi-output command
/// must never exit success with one role's bytes overwriting
/// another's.
///
/// The comparison is lexical. It catches two roles naming one
/// destination — including through `.` and `..` spellings — and makes
/// no claim about host-level aliasing or about the destination staying
/// put afterwards.
///
/// # Errors
///
/// Returns [`AliasedOutputs`] naming the first offending role pair.
pub fn ensure_distinct_outputs(roles: &[(&str, &std::path::Path)]) -> Result<(), AliasedOutputs> {
    let identities = roles
        .iter()
        .map(|(_, path)| destination_identity(path))
        .collect::<Vec<_>>();

    for (index, (first, _)) in roles.iter().enumerate() {
        for (offset, (second, _)) in roles.iter().enumerate().skip(index + 1) {
            if identities[index] == identities[offset] {
                return Err(AliasedOutputs {
                    first: (*first).to_owned(),
                    second: (*second).to_owned(),
                });
            }
        }
    }

    Ok(())
}

/// Failure publishing a check result through [`finish_check_command`].
#[derive(Debug, thiserror::Error)]
pub enum CheckResultError {
    /// Direct mode refused because stdout is a terminal (exit class 2).
    #[error("stdout is a terminal; refusing to write result data")]
    TtyRefusal,

    /// Exactly one of `--report`/`--stamp` was supplied. `clap` normally
    /// prevents this; the variant keeps the finalizer total.
    #[error("invalid checker output mode: --report and --stamp are all-or-nothing")]
    InvalidMode,

    /// The report could not be published or the stamp could not be
    /// touched.
    #[error("failed to publish the check result")]
    Io(#[from] io::Error),

    /// Build mode named one destination for both the report and the
    /// stamp; a stamp holding JSON bytes would contradict its
    /// empty-stamp role.
    #[error("aliased checker outputs: {0}")]
    AliasedOutputs(#[from] AliasedOutputs),
}

/// Publish a successful check result under the active output mode.
///
/// In direct mode the report is written to stdout (refused on a
/// terminal). In build mode the report is written compare-if-changed to
/// its explicit path and only then is the success stamp touched, so a
/// failed report publication never leaves a fresh stamp behind. Callers
/// must invoke this only when the semantic check succeeded; a failed
/// check leaves stdout empty and emits diagnostics on stderr.
///
/// # Errors
///
/// Returns [`CheckResultError::TtyRefusal`] in direct mode when stdout is
/// a terminal, [`CheckResultError::InvalidMode`] if only one build-mode
/// path is set, or [`CheckResultError::Io`] on a write failure.
pub fn finish_check_command<T: serde::Serialize>(
    command_name: &str,
    output: &CheckOutputArgs,
    report: &T,
) -> Result<(), CheckResultError> {
    match (&output.report, &output.stamp) {
        (None, None) => {
            if let Some(record) = stdout_tty_refusal_record(command_name) {
                let _ignored = emit_control_plane_record(&record);
                return Err(CheckResultError::TtyRefusal);
            }
            emit(report)?;
            Ok(())
        }

        (Some(report_path), Some(stamp_path)) => {
            ensure_distinct_outputs(&[("report", report_path), ("stamp", stamp_path)])?;
            // The checker CLI contract reports the check result, not
            // the publication's freshness components; a mode-repaired
            // report is still a current report.
            let _change = write_json_report_if_changed(report_path, report)?;
            touch_stamp(stamp_path)?;
            Ok(())
        }

        _ => Err(CheckResultError::InvalidMode),
    }
}

/// Write `value` as one JSON object plus trailing newline to `path`,
/// skipping the write when the current contents already match.
///
/// The report is staged in a sibling temp file and atomically renamed,
/// so a partial write never leaves a truncated report. The staged file
/// carries the public publication mode before the rename, so a report
/// does not inherit the owner-only mode of the temporary. The
/// compare-if-changed skip keeps ninja `restat` from cascading rebuilds
/// on unchanged results.
///
/// Freshness is bytes **and** mode (R2-N04): a report whose bytes are
/// current but whose mode is not `0o644` is repaired in place, leaving
/// its bytes and their modification time alone. The typed
/// [`PublicationChange`] says which of the two happened.
fn write_json_report_if_changed<T: serde::Serialize>(
    path: &std::path::Path,
    value: &T,
) -> io::Result<PublicationChange> {
    let mut bytes = serde_json::to_vec(value)?;
    bytes.push(b'\n');

    match publication::inspect_destination(path, &bytes, publication::PublicationMode::Public) {
        publication::DestinationState::Current => return Ok(PublicationChange::default()),
        publication::DestinationState::ModeOnly => {
            publication::repair_publication_mode(path, publication::PublicationMode::Public)?;
            return Ok(PublicationChange {
                bytes_changed: false,
                mode_changed: true,
            });
        }
        publication::DestinationState::StaleBytes => {}
    }

    let directory = path.parent().unwrap_or_else(|| std::path::Path::new("."));
    std::fs::create_dir_all(directory)?;

    let mut staged = tempfile::Builder::new()
        .prefix(".check-report-staged-")
        .tempfile_in(directory)?;
    staged.write_all(&bytes)?;
    staged.as_file().sync_all()?;
    publication::set_publication_mode(staged.as_file(), publication::PublicationMode::Public)?;
    staged.persist(path).map_err(|error| error.error)?;
    Ok(PublicationChange {
        bytes_changed: true,
        mode_changed: false,
    })
}
