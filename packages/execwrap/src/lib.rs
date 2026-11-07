#![doc = include_str!("../README.md")]

use std::ffi::OsString;
use std::fs::{File, create_dir_all};
use std::io::{self, Read};
use std::os::unix::process::ExitStatusExt as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{Sender, channel};
use std::thread;

use thiserror::Error;

pub mod writer;

#[cfg(test)]
mod tests;

pub use writer::{StreamProps, Writer};

/// One of the two captured streams.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stream {
    /// Standard output of the wrapped child process.
    Stdout,
    /// Standard error of the wrapped child process.
    Stderr,
}

impl Stream {
    /// Uppercase label, e.g. `"STDOUT"`.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Stdout => "STDOUT",
            Self::Stderr => "STDERR",
        }
    }

    const fn trace_label(self) -> &'static str {
        match self {
            Self::Stdout => "stdout",
            Self::Stderr => "stderr",
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::Stdout => 0,
            Self::Stderr => 1,
        }
    }
}

/// Errors returned from [`run`].
#[derive(Debug, Error)]
pub enum ExecError {
    /// An I/O error happened while preparing pipes, writers, or files.
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    /// The child process could not be spawned.
    ///
    /// The failing program path is caller-controlled `argv[0]` — raw
    /// child argv under ADR-010 — so it is deliberately not carried on
    /// this variant, keeping it out of every `Display` and diagnostic.
    #[error("failed to spawn child process: {source}")]
    Spawn {
        /// Underlying I/O error.
        #[source]
        source: io::Error,
    },

    /// The command vector was empty (no program to run).
    #[error("empty command")]
    EmptyCommand,

    /// The user requested ambiguous routing for the same file.
    #[error("ambiguous redirection for {path}: configured for multiple routing kinds")]
    AmbiguousRedirection {
        /// Conflicting output path.
        path: PathBuf,
    },

    /// A child stream has no explicit file route.
    #[error("unrouted child {stream:?} stream")]
    UnroutedStream {
        /// Child stream that needs a route.
        stream: Stream,
    },
}

/// User-provided routing configuration.
#[derive(Debug, Default, Clone)]
pub struct RoutingConfig {
    /// `--redirect` target: capture stdout and stderr merged in raw form.
    pub redirect: Option<PathBuf>,
    /// `--redirect-output` target: capture stdout only.
    pub redirect_output: Option<PathBuf>,
    /// `--redirect-error` target: capture stderr only.
    pub redirect_error: Option<PathBuf>,
    /// `--redirect-prefixed` target: capture both streams with prefixes.
    pub redirect_prefixed: Option<PathBuf>,
    /// Emit notifications when log files are written.
    pub notify_on_write: bool,
    /// Enable debug-level diagnostic logging.
    pub debug: bool,
    /// Suppress info-level diagnostic logging.
    pub quiet: bool,
}

impl RoutingConfig {
    /// True when the stream is being written to a log file.
    #[must_use]
    pub const fn is_stream_redirected(&self, stream: Stream) -> bool {
        if self.redirect.is_some() || self.redirect_prefixed.is_some() {
            return true;
        }
        match stream {
            Stream::Stdout => self.redirect_output.is_some(),
            Stream::Stderr => self.redirect_error.is_some(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RouteKind {
    CombinedRaw,
    CombinedPrefixed,
    StdoutOnly,
    StderrOnly,
}

impl RouteKind {
    const fn trace_label(self) -> &'static str {
        match self {
            Self::CombinedRaw => "combined_raw",
            Self::CombinedPrefixed => "combined_prefixed",
            Self::StdoutOnly => "stdout_only",
            Self::StderrOnly => "stderr_only",
        }
    }
}

#[derive(Debug)]
struct FileSpec {
    path: PathBuf,
    kind: RouteKind,
    identity: FileIdentity,
}

/// Filesystem identity of a redirection target.
///
/// This is the **sole** package-local exception to lexical output-role
/// comparison `(´[ADR017-rule:path:local-checks]´)`, which requires the
/// exception to state four things:
///
/// **The exact hazard.** Two redirection routes resolving to one file
/// are opened independently, each with truncation, and then written
/// concurrently by the wrapper as the child produces output. The
/// result is one log with interleaved and partially overwritten
/// bytes, and no I/O call fails at any point. The damage is done to
/// the operation's own output, not to repository state.
///
/// **The additional check.** Each target is resolved to a filesystem
/// identity before anything is opened: device and inode for an
/// existing file (so symlinks and hard links collapse), and resolved
/// parent directory plus file name for one that does not exist yet.
/// A dangling symlink resolves to its ultimate target, because
/// creating the file would follow the link.
///
/// **Why lexical uniqueness is insufficient here.** Lexical
/// comparison catches `a.log` against `./a.log`, but not two names
/// hard-linked to one file, nor two paths beneath a symlinked
/// directory, nor a dangling link and the name it points at. Those
/// all produce the interleaving above, and unlike an ordinary output
/// role — which the caller controls end to end — these paths are
/// routinely assembled by build glue where such aliases arise by
/// accident.
///
/// **The remaining host race.** None of this closes a
/// time-of-check/time-of-use window: the host may replace or remount
/// any of these paths between the check and the open. This is a
/// preflight guard against configuration mistakes and nothing more.
/// It is a package-local correctness measure, never a
/// repository-wide filesystem security claim.
#[derive(Debug, PartialEq, Eq)]
enum FileIdentity {
    /// The target exists: device and inode numbers (symlinks
    /// followed, hard links share the pair).
    Existing(u64, u64),
    /// The target does not exist yet: resolved parent directory plus
    /// final file name.
    Pending(PathBuf, OsString),
}

/// Bound on manual resolution of dangling symlink chains, matching
/// the kernel's nested-link limit behind `ELOOP`.
const SYMLINK_FOLLOW_LIMIT: usize = 40;

fn file_identity(path: &Path) -> io::Result<FileIdentity> {
    use std::os::unix::fs::MetadataExt as _;

    let mut current = path.to_path_buf();
    let mut hops = 0_usize;
    loop {
        match std::fs::metadata(&current) {
            Ok(metadata) => return Ok(FileIdentity::Existing(metadata.dev(), metadata.ino())),

            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                // NotFound covers a dangling final symlink, and
                // `File::create` on that link would follow it: the
                // identity must be that of the link's ultimate
                // target, never the link's own name.
                let is_dangling_link = std::fs::symlink_metadata(&current)
                    .is_ok_and(|link| link.file_type().is_symlink());
                if !is_dangling_link {
                    return pending_identity(&current);
                }
                if hops == SYMLINK_FOLLOW_LIMIT {
                    // `ErrorKind::FilesystemLoop` is not stable yet.
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "too many levels of symbolic links in redirection target",
                    ));
                }
                hops += 1;

                let target = std::fs::read_link(&current)?;
                current = if target.is_absolute() {
                    target
                } else {
                    // A relative link target resolves against the
                    // link's own directory, not the process cwd.
                    match current.parent() {
                        Some(parent) if !parent.as_os_str().is_empty() => parent.join(target),
                        _ => target,
                    }
                };
            }

            Err(error) => return Err(error),
        }
    }
}

/// Identity of a target that does not exist and is not a symlink.
fn pending_identity(path: &Path) -> io::Result<FileIdentity> {
    let file_name = path
        .file_name()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "redirection path has no file name",
            )
        })?
        .to_os_string();

    let parent = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };

    let parent = resolve_pending_directory(parent)?;

    Ok(FileIdentity::Pending(parent, file_name))
}

/// Absolute identity of a directory whose trailing components may not
/// exist yet: every existing prefix component is resolved through the
/// filesystem (following symlinks), and only the missing suffix is
/// resolved lexically. Canonicalizing the whole parent — falling back
/// to a purely lexical form when any component is missing — would let
/// two routes through a symlinked existing ancestor with directories
/// still to be created below it register as distinct identities.
fn resolve_pending_directory(path: &Path) -> io::Result<PathBuf> {
    use std::path::Component;

    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };

    let mut resolved = PathBuf::new();
    // Number of trailing components that could not be resolved through
    // the filesystem. While zero, `resolved` is canonical and every
    // new component is re-resolved — so a `..` that pops back out of
    // the missing suffix returns to filesystem-backed resolution
    // instead of leaving later symlinked components lexical.
    let mut missing = 0_usize;
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            // Exact in both modes: a canonical prefix is symlink-free,
            // and a missing suffix directory is created literally.
            Component::ParentDir => {
                resolved.pop();
                missing = missing.saturating_sub(1);
            }
            other => {
                if missing > 0 {
                    resolved.push(other);
                    missing += 1;
                    continue;
                }
                resolved.push(other);
                match resolved.canonicalize() {
                    Ok(canonical) => resolved = canonical,
                    // Missing (or unreadable) component: lexical until
                    // a `..` climbs back into existing space.
                    Err(_) => missing = 1,
                }
            }
        }
    }

    Ok(resolved)
}

fn collect_file_specs(cfg: &RoutingConfig) -> Result<Vec<FileSpec>, ExecError> {
    let candidates = [
        (cfg.redirect.as_ref(), RouteKind::CombinedRaw),
        (cfg.redirect_prefixed.as_ref(), RouteKind::CombinedPrefixed),
        (cfg.redirect_output.as_ref(), RouteKind::StdoutOnly),
        (cfg.redirect_error.as_ref(), RouteKind::StderrOnly),
    ];

    let mut specs: Vec<FileSpec> = Vec::new();
    for (maybe_path, kind) in candidates {
        let Some(path) = maybe_path else { continue };
        let identity = file_identity(path)?;
        if let Some(existing) = specs.iter().find(|s| s.identity == identity) {
            if existing.kind != kind {
                return Err(ExecError::AmbiguousRedirection { path: path.clone() });
            }
        } else {
            specs.push(FileSpec {
                path: path.clone(),
                kind,
                identity,
            });
        }
    }
    Ok(specs)
}

/// Validate a routing configuration before any side effect.
///
/// Targets are only stat'ed — nothing is opened, created, or
/// truncated — so the binary can report an invalid argument
/// combination as a usage failure before touching any file.
///
/// # Errors
///
/// Returns [`ExecError::AmbiguousRedirection`] when one file — by
/// filesystem identity, not merely lexical path equality — is
/// configured for multiple routing kinds, or [`ExecError::UnroutedStream`]
/// when either child stream has no file route.
pub fn preflight_routing(cfg: &RoutingConfig) -> Result<(), ExecError> {
    let route_count = if tracing::enabled!(target: "execwrap::routing", tracing::Level::DEBUG) {
        configured_route_count(cfg)
    } else {
        0
    };
    let span = tracing::span!(
        target: "execwrap::routing",
        tracing::Level::DEBUG,
        "routing_preflight",
        route_count
    );
    let _entered = span.enter();
    let result = validate_routes(cfg).map(|_specs| ());
    match &result {
        Ok(()) => tracing::debug!(
            target: "execwrap::routing",
            outcome = "completed",
            route_count,
            "routing preflight completed"
        ),
        Err(ExecError::AmbiguousRedirection { .. }) => tracing::debug!(
            target: "execwrap::routing",
            outcome = "refused",
            route_count,
            refusal_kind = "ambiguous_redirection",
            "routing preflight completed"
        ),
        Err(ExecError::UnroutedStream { .. }) => tracing::debug!(
            target: "execwrap::routing",
            outcome = "refused",
            route_count,
            refusal_kind = "unrouted_stream",
            "routing preflight completed"
        ),
        Err(_) => tracing::debug!(
            target: "execwrap::routing",
            outcome = "failed",
            route_count,
            error_kind = "io",
            "routing preflight completed"
        ),
    }
    result
}

fn validate_routes(cfg: &RoutingConfig) -> Result<Vec<FileSpec>, ExecError> {
    for stream in [Stream::Stdout, Stream::Stderr] {
        if !cfg.is_stream_redirected(stream) {
            return Err(ExecError::UnroutedStream { stream });
        }
    }
    collect_file_specs(cfg)
}

fn configured_route_count(cfg: &RoutingConfig) -> u64 {
    [
        cfg.redirect.is_some(),
        cfg.redirect_prefixed.is_some(),
        cfg.redirect_output.is_some(),
        cfg.redirect_error.is_some(),
    ]
    .into_iter()
    .map(u64::from)
    .sum()
}

fn count_u64(len: usize) -> u64 {
    u64::try_from(len).unwrap_or(u64::MAX)
}

/// Open `path` for writing in binary mode, creating parent directories as needed.
///
/// # Errors
///
/// Returns the underlying I/O error from directory creation or file open.
pub fn safe_open(path: &Path) -> io::Result<File> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        create_dir_all(parent)?;
    }
    File::create(path)
}

/// Notification emitted by the wrapper while running.
#[derive(Debug, Clone)]
pub enum Notification {
    /// `--notify-on-write` informational record: writing to a log file.
    WritingToLog { path: PathBuf, stream: Stream },
    /// `--notify-on-write` deferred record: wrote to a log file by end of run.
    WroteToLog { path: PathBuf },
}

/// Outcome of running the wrapped command.
#[derive(Debug, Clone)]
pub struct ExecOutcome {
    /// Numeric exit code. Signal terminations are reported as `128 + signal`.
    pub exit_code: i32,
    /// True when captured output may be incomplete: a log-file write,
    /// a child-stream read, or a writer finalization failed. Callers
    /// must treat a run with data loss as failed even when the child
    /// exited 0 — silently succeeding after losing logs is data loss,
    /// not success.
    pub data_loss: bool,
}

/// Run the wrapped command using the provided routing configuration.
///
/// `notify` is called for each notification record (only when `notify_on_write`
/// is enabled).
///
/// # Errors
///
/// Returns [`ExecError`] for setup failures (pipes, writers, spawn). A non-zero
/// exit code from the child is *not* an error: it is returned through
/// [`ExecOutcome::exit_code`].
///
/// # Panics
///
/// Panics if the child's stdout/stderr pipes cannot be taken after being
/// configured as piped, which should never happen.
#[allow(
    clippy::cognitive_complexity,
    clippy::too_many_lines,
    clippy::option_if_let_else
)]
pub fn run(
    command: &[OsString],
    cfg: &RoutingConfig,
    notify: impl FnMut(&Notification),
) -> Result<ExecOutcome, ExecError> {
    let enabled = tracing::enabled!(target: "execwrap::run", tracing::Level::INFO);
    let argument_count = if enabled {
        count_u64(command.len().saturating_sub(1))
    } else {
        0
    };
    let route_count = if enabled {
        configured_route_count(cfg)
    } else {
        0
    };
    let span = tracing::span!(
        target: "execwrap::run",
        tracing::Level::INFO,
        "child_run",
        argument_count,
        route_count
    );
    let _entered = span.enter();
    let result = run_inner(command, cfg, notify);
    match &result {
        Ok((outcome, normal_exit_code)) => {
            if let Some(child_exit_code) = normal_exit_code {
                tracing::info!(
                    target: "execwrap::run",
                    outcome = "completed",
                    child_exit_code,
                    data_loss = outcome.data_loss,
                    "child completed"
                );
            } else {
                tracing::info!(
                    target: "execwrap::run",
                    outcome = "completed",
                    data_loss = outcome.data_loss,
                    "child completed"
                );
            }
        }
        Err(ExecError::EmptyCommand) => tracing::debug!(
            target: "execwrap::run",
            outcome = "refused",
            refusal_kind = "empty_command",
            "child completed"
        ),
        Err(ExecError::AmbiguousRedirection { .. }) => tracing::debug!(
            target: "execwrap::run",
            outcome = "refused",
            refusal_kind = "ambiguous_redirection",
            "child completed"
        ),
        Err(ExecError::UnroutedStream { .. }) => tracing::debug!(
            target: "execwrap::run",
            outcome = "refused",
            refusal_kind = "unrouted_stream",
            "child completed"
        ),
        Err(ExecError::Spawn { .. }) => tracing::info!(
            target: "execwrap::run",
            outcome = "failed",
            error_kind = "spawn",
            "child completed"
        ),
        Err(ExecError::Io(_)) => tracing::info!(
            target: "execwrap::run",
            outcome = "failed",
            error_kind = "io",
            "child completed"
        ),
    }
    result.map(|(outcome, _normal_exit_code)| outcome)
}

#[allow(
    clippy::cognitive_complexity,
    clippy::too_many_lines,
    clippy::option_if_let_else
)]
fn run_inner(
    command: &[OsString],
    cfg: &RoutingConfig,
    mut notify: impl FnMut(&Notification),
) -> Result<(ExecOutcome, Option<i32>), ExecError> {
    let program = command.first().ok_or(ExecError::EmptyCommand)?;
    let args = &command[1..];

    let specs = validate_routes(cfg)?;

    // Build writers per file path, then index them per stream.
    let mut file_writers: Vec<FileWriter> = Vec::new();
    let mut stdout_subscribers: Vec<usize> = Vec::new();
    let mut stderr_subscribers: Vec<usize> = Vec::new();

    for spec in &specs {
        let file = safe_open(&spec.path).map_err(ExecError::Io)?;
        let is_prefixed = matches!(spec.kind, RouteKind::CombinedPrefixed);
        let props = if is_prefixed {
            StreamProps {
                line1_prefix_stdout: ">>> ".to_string(),
                wrap_prefix_stdout: "    ".to_string(),
                line1_prefix_stderr: "*** ".to_string(),
                wrap_prefix_stderr: "    ".to_string(),
            }
        } else {
            StreamProps::default()
        };
        let writer = Writer::new(file, is_prefixed, 80, props, cfg.debug);
        let idx = file_writers.len();
        file_writers.push(FileWriter {
            path: spec.path.clone(),
            kind: spec.kind,
            writer,
        });

        match spec.kind {
            RouteKind::StdoutOnly => stdout_subscribers.push(idx),
            RouteKind::StderrOnly => stderr_subscribers.push(idx),
            RouteKind::CombinedRaw | RouteKind::CombinedPrefixed => {
                stdout_subscribers.push(idx);
                stderr_subscribers.push(idx);
            }
        }
    }

    // Track first-write notifications.
    let mut first_write_stdout = !stdout_subscribers.is_empty();
    let mut first_write_stderr = !stderr_subscribers.is_empty();

    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| ExecError::Spawn { source })?;

    // ADR-010 redaction: neither argv nor the program path is logged,
    // even under --debug. Command lines routinely carry tokens,
    // passwords, and credential URLs, and the program is itself
    // caller-controlled argv[0]; no field-aware sanitizer guards this
    // call site, so only known-safe metadata (argument count and pid)
    // is recorded.
    tracing::info!(
        target: "execwrap::run",
        argument_count = count_u64(args.len()),
        pid = child.id(),
        "child started",
    );

    let stdout = child.stdout.take().expect("stdout pipe configured above");
    let stderr = child.stderr.take().expect("stderr pipe configured above");

    let (tx, rx) = channel::<ChannelMessage>();

    let stdout_handle = spawn_reader(Stream::Stdout, stdout, tx.clone());
    let stderr_handle = spawn_reader(Stream::Stderr, stderr, tx);

    let mut data_loss = false;
    let track_streams = tracing::enabled!(target: "execwrap::stream", tracing::Level::DEBUG);
    let mut stream_metrics = [StreamMetrics::default(), StreamMetrics::default()];

    for message in rx {
        match message {
            ChannelMessage::Data { stream, data } => {
                if track_streams {
                    let metrics = &mut stream_metrics[stream.index()];
                    metrics.read_bytes = metrics.read_bytes.saturating_add(count_u64(data.len()));
                }
                let subscribers = match stream {
                    Stream::Stdout => &stdout_subscribers,
                    Stream::Stderr => &stderr_subscribers,
                };
                let first = match stream {
                    Stream::Stdout => {
                        let f = first_write_stdout;
                        first_write_stdout = false;
                        f
                    }
                    Stream::Stderr => {
                        let f = first_write_stderr;
                        first_write_stderr = false;
                        f
                    }
                };
                for &idx in subscribers {
                    let entry = &mut file_writers[idx];
                    if first && cfg.notify_on_write {
                        tracing::info!(
                            target: "execwrap::routing",
                            route_kind = entry.kind.trace_label(),
                            stream = stream.trace_label(),
                            "log write started"
                        );
                        notify(&Notification::WritingToLog {
                            path: entry.path.clone(),
                            stream,
                        });
                    }
                    if entry.writer.write(stream, &data).is_err() {
                        data_loss = true;
                        if track_streams {
                            stream_metrics[stream.index()].fail("write");
                        }
                    }
                }
            }
            ChannelMessage::Eof { stream: _stream } => {}
            ChannelMessage::Error {
                stream,
                error: _error,
            } => {
                data_loss = true;
                if track_streams {
                    stream_metrics[stream.index()].fail("read");
                }
            }
        }
    }

    // A panicked reader thread may have dropped bytes on the floor.
    for (stream, handle) in [
        (Stream::Stdout, stdout_handle),
        (Stream::Stderr, stderr_handle),
    ] {
        if handle.join().is_err() {
            data_loss = true;
            if track_streams {
                stream_metrics[stream.index()].fail("reader_panic");
            }
        }
    }

    for entry in &mut file_writers {
        if entry.writer.finalize().is_err() {
            data_loss = true;
            if track_streams {
                match entry.kind {
                    RouteKind::CombinedRaw | RouteKind::CombinedPrefixed => {
                        stream_metrics[0].fail("finalize");
                        stream_metrics[1].fail("finalize");
                    }
                    RouteKind::StdoutOnly => stream_metrics[0].fail("finalize"),
                    RouteKind::StderrOnly => stream_metrics[1].fail("finalize"),
                }
            }
        }
    }

    if track_streams {
        for stream in [Stream::Stdout, Stream::Stderr] {
            let written_bytes = file_writers.iter().fold(0_u64, |total, entry| {
                total.saturating_add(entry.writer.written_bytes(stream))
            });
            let metrics = stream_metrics[stream.index()];
            if let Some(error_kind) = metrics.error_kind {
                tracing::debug!(
                    target: "execwrap::stream",
                    stream = stream.trace_label(),
                    outcome = "failed",
                    read_bytes = metrics.read_bytes,
                    written_bytes,
                    error_kind,
                    "child stream completed"
                );
            } else {
                tracing::debug!(
                    target: "execwrap::stream",
                    stream = stream.trace_label(),
                    outcome = "completed",
                    read_bytes = metrics.read_bytes,
                    written_bytes,
                    "child stream completed"
                );
            }
        }
    }

    let status = child.wait().map_err(ExecError::Io)?;

    let exit_code = if let Some(code) = status.code() {
        code
    } else if let Some(signal) = status.signal() {
        128 + signal
    } else {
        1
    };

    Ok((
        ExecOutcome {
            exit_code,
            data_loss,
        },
        status.code(),
    ))
}

struct FileWriter {
    path: PathBuf,
    kind: RouteKind,
    writer: Writer,
}

#[derive(Clone, Copy, Default)]
struct StreamMetrics {
    read_bytes: u64,
    error_kind: Option<&'static str>,
}

impl StreamMetrics {
    const fn fail(&mut self, kind: &'static str) {
        if self.error_kind.is_none() {
            self.error_kind = Some(kind);
        }
    }
}

enum ChannelMessage {
    Data { stream: Stream, data: Vec<u8> },
    Eof { stream: Stream },
    Error { stream: Stream, error: io::Error },
}

fn spawn_reader<R: Read + Send + 'static>(
    stream: Stream,
    mut source: R,
    tx: Sender<ChannelMessage>,
) -> thread::JoinHandle<()> {
    let dispatch = tracing::dispatcher::get_default(tracing::Dispatch::clone);
    let parent = tracing::Span::current();
    thread::spawn(move || {
        tracing::dispatcher::with_default(&dispatch, || {
            let _parent = parent.enter();
            let span = tracing::span!(
                target: "execwrap::stream",
                tracing::Level::DEBUG,
                "child_stream_read",
                stream = stream.trace_label()
            );
            let _entered = span.enter();
            let mut buffer = [0_u8; 8192];
            loop {
                match source.read(&mut buffer) {
                    Ok(0) => {
                        let _ignored = tx.send(ChannelMessage::Eof { stream });
                        break;
                    }
                    Ok(read) => {
                        let data = buffer[..read].to_vec();
                        if tx.send(ChannelMessage::Data { stream, data }).is_err() {
                            break;
                        }
                    }
                    Err(ref error) if error.kind() == io::ErrorKind::Interrupted => {}
                    Err(error) => {
                        let _ignored = tx.send(ChannelMessage::Error { stream, error });
                        break;
                    }
                }
            }
        });
    })
}

// ---------------------------------------------------------------------------
// Mocked TeX children (ADR-014 mock_mode). In mock_mode the Meson graph
// selects these instead of the real toolchain so the graph, dependencies,
// restat behaviour, and failure propagation can be exercised without a TeX
// toolchain. The real TeX command wiring is XeLaTeX's own concern; this only
// fabricates the deterministic outputs each step's downstream target needs.
// ---------------------------------------------------------------------------

/// A mocked external TeX child, selected by the build in `mock_mode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum MockChild {
    /// Simulates `xelatex -no-pdf`: writes `main.bcf` and `main.aux`.
    Xelatex,
    /// Simulates `biber`: writes `main.bbl`.
    Biber,
    /// Simulates `latexmk`: writes `main.pdf` and `main.aux`.
    Latexmk,
}

impl MockChild {
    const fn trace_label(self) -> &'static str {
        match self {
            Self::Xelatex => "xelatex",
            Self::Biber => "biber",
            Self::Latexmk => "latexmk",
        }
    }
}

const MOCK_BCF: &[u8] = b"% mock main.bcf\n";
const MOCK_AUX: &[u8] = b"\\relax\n% mock main.aux\n";
const MOCK_BBL: &[u8] = b"% mock main.bbl\n";
const MOCK_PDF: &[u8] = b"MOCK TRIPOD ATTESTATION PDF\n";

/// Fabricate the deterministic outputs a mocked TeX child produces, under
/// `outdir`.
///
/// Compare-if-changed writes keep a rebuild byte- and mtime-stable so the
/// mock exercises ninja `restat` the same way the real build does.
///
/// # Errors
///
/// Returns an I/O error if `outdir` cannot be created or an output written.
pub fn run_mock_child(kind: MockChild, outdir: &Path) -> io::Result<()> {
    let outputs: &[(&str, &[u8])] = match kind {
        MockChild::Xelatex => &[("main.bcf", MOCK_BCF), ("main.aux", MOCK_AUX)],
        MockChild::Biber => &[("main.bbl", MOCK_BBL)],
        MockChild::Latexmk => &[("main.pdf", MOCK_PDF), ("main.aux", MOCK_AUX)],
    };
    let output_count = if tracing::enabled!(target: "execwrap::mock", tracing::Level::INFO) {
        count_u64(outputs.len())
    } else {
        0
    };
    let span = tracing::span!(
        target: "execwrap::mock",
        tracing::Level::INFO,
        "mock_child_run",
        child_kind = kind.trace_label(),
        output_count
    );
    let _entered = span.enter();
    let result = (|| {
        create_dir_all(outdir)?;
        for (name, bytes) in outputs {
            write_if_changed_under(outdir, name, bytes)?;
        }
        Ok(())
    })();
    if result.is_ok() {
        tracing::info!(
            target: "execwrap::mock",
            outcome = "completed",
            child_kind = kind.trace_label(),
            output_count,
            "mock child completed"
        );
    } else {
        tracing::info!(
            target: "execwrap::mock",
            outcome = "failed",
            child_kind = kind.trace_label(),
            output_count,
            error_kind = "io",
            "mock child completed"
        );
    }
    result
}

/// Write `bytes` to `dir/name` only if the current contents differ.
fn write_if_changed_under(dir: &Path, name: &str, bytes: &[u8]) -> io::Result<()> {
    let path = dir.join(name);
    if std::fs::read(&path).is_ok_and(|existing| existing == bytes) {
        return Ok(());
    }
    std::fs::write(&path, bytes)
}
