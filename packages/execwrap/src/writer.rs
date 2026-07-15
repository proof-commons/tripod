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
    debug: bool,
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
        debug: bool,
    ) -> Self {
        Self {
            file: Box::new(file),
            add_prefix,
            wrap_length,
            props,
            debug,
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
            if self.debug {
                tracing::debug!(bytes = raw.len(), "writer typesetter");
            }
            self.file.write_all(raw)
        }
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
            if self.debug {
                tracing::debug!(line = %line.trim_end(), "writer calligraphic");
            }
            let wrapped = wrap_with_prefixes(&line, &line1, &wrap, self.wrap_length);
            self.file.write_all(wrapped.as_bytes())?;
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
