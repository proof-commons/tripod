//! Buffered, per-stream log writer used by [`crate::run`].
//!
//! Two-mode design:
//!
//! - **Typesetter mode (`add_prefix = false`)** writes the child's
//!   bytes **unchanged**: no UTF-8 conversion, no control-byte
//!   replacement, no truncation. Raw redirection must be byte-exact.
//! - **Calligraphic mode (`add_prefix = true`)** is a text
//!   presentation: it byte-buffers per stream, splits on newline
//!   bytes, sanitizes each completed line (lossy UTF-8, control bytes
//!   replaced), and re-wraps it with a per-stream prefix. Sanitization
//!   runs on complete lines, never on individual read chunks, so a
//!   multibyte code point split across pipe reads survives intact and
//!   the output does not depend on OS chunk boundaries.

use std::io::{self, Write};

use crate::Stream;

/// Per-stream prefix properties for the calligraphic mode.
#[derive(Debug, Default, Clone)]
pub struct StreamProps {
    /// Prefix for the first line of each wrapped block on stdout.
    pub line1_prefix_stdout: String,
    /// Prefix for continuation lines on stdout.
    pub wrap_prefix_stdout: String,
    /// Prefix for the first line of each wrapped block on stderr.
    pub line1_prefix_stderr: String,
    /// Prefix for continuation lines on stderr.
    pub wrap_prefix_stderr: String,
}

/// Two-mode buffered writer.
pub struct Writer {
    file: Box<dyn Write + Send>,
    add_prefix: bool,
    wrap_length: usize,
    props: StreamProps,
    track_written_bytes: bool,
    stdout_written_bytes: u64,
    stderr_written_bytes: u64,
    stdout_buffer: Vec<u8>,
    stderr_buffer: Vec<u8>,
}

impl Writer {
    /// Build a new writer.
    ///
    /// `add_prefix` selects calligraphic mode. `wrap_length` is used by
    /// the line-wrapper in calligraphic mode only.
    #[must_use]
    pub fn new<W: Write + Send + 'static>(
        file: W,
        add_prefix: bool,
        wrap_length: usize,
        props: StreamProps,
        _debug: bool,
    ) -> Self {
        Self {
            file: Box::new(file),
            add_prefix,
            wrap_length,
            props,
            track_written_bytes: tracing::enabled!(
                target: "execwrap::stream",
                tracing::Level::DEBUG
            ),
            stdout_written_bytes: 0,
            stderr_written_bytes: 0,
            stdout_buffer: Vec::new(),
            stderr_buffer: Vec::new(),
        }
    }

    /// Write a chunk of bytes attributed to `stream`.
    ///
    /// In typesetter (raw) mode the bytes are written unchanged. In
    /// calligraphic mode they are sanitized and line-wrapped.
    ///
    /// # Errors
    ///
    /// Returns any underlying I/O error from the destination file.
    pub fn write(&mut self, stream: Stream, raw: &[u8]) -> io::Result<()> {
        if self.add_prefix {
            self.write_calligraphic(stream, raw)
        } else {
            self.file.write_all(raw)?;
            if self.track_written_bytes {
                self.record_written(stream, raw.len());
            }
            Ok(())
        }
    }

    /// Bytes confirmed written for one stream, excluding partial failed writes.
    pub(crate) const fn written_bytes(&self, stream: Stream) -> u64 {
        match stream {
            Stream::Stdout => self.stdout_written_bytes,
            Stream::Stderr => self.stderr_written_bytes,
        }
    }

    fn record_written(&mut self, stream: Stream, len: usize) {
        let total = match stream {
            Stream::Stdout => &mut self.stdout_written_bytes,
            Stream::Stderr => &mut self.stderr_written_bytes,
        };
        *total = total.saturating_add(u64::try_from(len).unwrap_or(u64::MAX));
    }

    fn write_calligraphic(&mut self, stream: Stream, raw: &[u8]) -> io::Result<()> {
        let (line1, wrap) = match stream {
            Stream::Stdout => (
                self.props.line1_prefix_stdout.clone(),
                self.props.wrap_prefix_stdout.clone(),
            ),
            Stream::Stderr => (
                self.props.line1_prefix_stderr.clone(),
                self.props.wrap_prefix_stderr.clone(),
            ),
        };

        let buffer = match stream {
            Stream::Stdout => &mut self.stdout_buffer,
            Stream::Stderr => &mut self.stderr_buffer,
        };
        buffer.extend_from_slice(raw);

        // Split on the newline byte, which can never be a UTF-8
        // continuation byte, so a multibyte code point straddling two
        // reads stays whole inside one buffered line.
        while let Some(newline_index) = buffer.iter().position(|&byte| byte == b'\n') {
            let line_bytes: Vec<u8> = buffer.drain(..=newline_index).collect();
            let line = sanitize(&line_bytes);
            let wrapped = wrap_with_prefixes(&line, &line1, &wrap, self.wrap_length);
            self.file.write_all(wrapped.as_bytes())?;
            if self.track_written_bytes {
                let total = match stream {
                    Stream::Stdout => &mut self.stdout_written_bytes,
                    Stream::Stderr => &mut self.stderr_written_bytes,
                };
                *total = total.saturating_add(u64::try_from(wrapped.len()).unwrap_or(u64::MAX));
            }
        }
        Ok(())
    }

    /// Flush any partial calligraphic buffer for both streams, then
    /// flush the destination file.
    ///
    /// # Errors
    ///
    /// Returns any underlying I/O error from the destination file.
    pub fn finalize(&mut self) -> io::Result<()> {
        let mode = if self.add_prefix { "prefixed" } else { "raw" };
        let span = tracing::span!(
            target: "execwrap::writer",
            tracing::Level::DEBUG,
            "writer_finalize",
            mode
        );
        let _entered = span.enter();
        let result = self.finalize_inner();
        if result.is_ok() {
            tracing::debug!(
                target: "execwrap::writer",
                outcome = "completed",
                "writer finalized"
            );
        } else {
            tracing::debug!(
                target: "execwrap::writer",
                outcome = "failed",
                error_kind = "io",
                "writer finalized"
            );
        }
        result
    }

    fn finalize_inner(&mut self) -> io::Result<()> {
        if self.add_prefix {
            for stream in [Stream::Stdout, Stream::Stderr] {
                let (line1, wrap) = match stream {
                    Stream::Stdout => (
                        self.props.line1_prefix_stdout.clone(),
                        self.props.wrap_prefix_stdout.clone(),
                    ),
                    Stream::Stderr => (
                        self.props.line1_prefix_stderr.clone(),
                        self.props.wrap_prefix_stderr.clone(),
                    ),
                };
                let buffer = match stream {
                    Stream::Stdout => &mut self.stdout_buffer,
                    Stream::Stderr => &mut self.stderr_buffer,
                };
                if buffer.is_empty() {
                    continue;
                }
                let mut line = sanitize(&std::mem::take(buffer));
                line.push('\n');
                let wrapped = wrap_with_prefixes(&line, &line1, &wrap, self.wrap_length);
                self.file.write_all(wrapped.as_bytes())?;
                if self.track_written_bytes {
                    self.record_written(stream, wrapped.len());
                }
            }
        }
        self.file.flush()
    }
}

/// Lossy text conversion for the calligraphic presentation only:
/// invalid UTF-8 becomes replacement characters and control bytes
/// (except newline, tab, carriage return) become `?`. Never applied
/// to raw (typesetter) routing. Callers must pass complete buffered
/// lines, not read chunks: lossy conversion of a partial chunk would
/// corrupt a multibyte code point split across reads.
fn sanitize(raw: &[u8]) -> String {
    let lossy = String::from_utf8_lossy(raw).into_owned();
    lossy
        .chars()
        .map(|character| {
            if character == '\n'
                || character == '\t'
                || character == '\r'
                || !character.is_control()
            {
                character
            } else {
                '?'
            }
        })
        .collect()
}

fn wrap_with_prefixes(line: &str, line1_prefix: &str, wrap_prefix: &str, width: usize) -> String {
    let trimmed = line.strip_suffix('\n').unwrap_or(line);
    let options = textwrap::Options::new(width)
        .initial_indent(line1_prefix)
        .subsequent_indent(wrap_prefix)
        .break_words(true);
    let mut wrapped = textwrap::wrap(trimmed, &options).join("\n");
    wrapped.push('\n');
    wrapped
}
