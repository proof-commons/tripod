//! `flatten-latex-main` library: produce a single flat `.tex` from a paper's
//! `main.tex`, inlining `\input{}` / `\subfile{}` and embedding the
//! `\addbibresource{}` BibTeX file inside `\begin{filecontents*}` blocks.
//!
//! Port of the original `bin/create_flat_main.pl` script. The binary entry
//! point lives in [`src/bin/flatten-latex-main.rs`](../bin/flatten-latex-main.rs).

use std::fs::{File, create_dir_all};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};

#[cfg(test)]
mod tests;

/// Behavior switches for [`flatten`].
#[derive(Debug, Default, Clone)]
pub struct FlattenOptions {
    /// Fail on a missing `\addbibresource{}` file instead of emitting
    /// a warning comment. Release builds should set this: a silently
    /// dropped bibliography is a broken artifact, not a warning.
    pub strict_bibliography: bool,
}

/// Flatten `main_file` into `output_file`, resolving `\input{}` / `\subfile{}`
/// and `\addbibresource{}` paths relative to `paper_dir`.
///
/// `include_dirs` is an ordered list of additional directories searched as a
/// fallback when a referenced file is not found under `paper_dir`. This lets
/// callers (e.g. meson) inline shared macros that are staged into a build
/// directory rather than sitting next to the paper's other sources.
///
/// The flattener is intentionally blind to the surrounding repository layout:
/// callers (e.g. meson) supply the paper directory and output path directly.
///
/// The output is **reproducible** (same inputs, byte-identical output —
/// no timestamps) and **atomic** (written to a uniquely named temporary
/// in the output directory and renamed, so a failed flatten never leaves
/// a partial output file and concurrent flattens never share a staging
/// path).
///
/// # Errors
///
/// Returns any I/O failure encountered while reading inputs or writing the
/// flattened output, including a missing `main_file` or `\input{}` target,
/// an include cycle, or (with [`FlattenOptions::strict_bibliography`]) a
/// missing bibliography file.
pub fn flatten(
    paper_dir: &Path,
    main_file: &Path,
    output_file: &Path,
    include_dirs: &[PathBuf],
    options: &FlattenOptions,
) -> Result<()> {
    let output_parent = output_file
        .parent()
        .ok_or_else(|| anyhow!("output path has no parent"))?;
    if !output_parent.as_os_str().is_empty() && !output_parent.exists() {
        create_dir_all(output_parent)
            .with_context(|| format!("creating {}", output_parent.display()))?;
    }

    // Atomic output: write to a uniquely named temporary in the output
    // directory, rename on success. A fixed staging name would let two
    // concurrent flattens of the same output truncate or rename each
    // other's bytes; the unique temporary is removed automatically on
    // failure (drop) and on success (persist consumes it).
    let staging_dir = if output_parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        output_parent
    };

    let staged = tempfile::Builder::new()
        .prefix(".flatten-staged-")
        .tempfile_in(staging_dir)
        .with_context(|| format!("creating staging file in {}", staging_dir.display()))?;

    write_flattened(
        paper_dir,
        main_file,
        staged.as_file(),
        include_dirs,
        options,
    )?;

    staged
        .persist(output_file)
        .with_context(|| format!("renaming into {}", output_file.display()))?;

    Ok(())
}

fn write_flattened(
    paper_dir: &Path,
    main_file: &Path,
    staged: &File,
    include_dirs: &[PathBuf],
    options: &FlattenOptions,
) -> Result<()> {
    let mut writer = BufWriter::new(staged);

    // The header is fixed: no timestamps or environment-dependent
    // values, so the same inputs flatten to byte-identical output.
    writeln!(writer, "% ========== Flattened document ==========")?;
    writeln!(
        writer,
        "% This is an auto-generated file. Do not edit directly."
    )?;
    writeln!(
        writer,
        "% Original structure is preserved with include commands commented out."
    )?;
    writeln!(
        writer,
        "% ==============================================================="
    )?;
    writeln!(writer)?;

    let mut context = FlattenContext {
        paper_dir,
        include_dirs,
        options,
        include_stack: Vec::new(),
    };

    context.process_file(main_file, &mut writer, true)?;

    writer.flush()?;
    Ok(())
}

/// Resolve a referenced file name against `paper_dir` first, then each entry in
/// `include_dirs`. When `name` has no extension, a `default_ext` variant is
/// also tried in every directory. Returns the first existing candidate, or the
/// best `paper_dir`-relative guess (preserving the original error messages)
/// when nothing is found.
fn resolve_reference(
    name: &str,
    paper_dir: &Path,
    include_dirs: &[PathBuf],
    default_ext: &str,
) -> PathBuf {
    let needs_ext = !name.contains('.');
    for dir in std::iter::once(paper_dir).chain(include_dirs.iter().map(PathBuf::as_path)) {
        let candidate = dir.join(name);
        if candidate.exists() {
            return candidate;
        }
        if needs_ext {
            let with_ext = dir.join(format!("{name}.{default_ext}"));
            if with_ext.exists() {
                return with_ext;
            }
        }
    }
    if needs_ext {
        paper_dir.join(format!("{name}.{default_ext}"))
    } else {
        paper_dir.join(name)
    }
}

/// Recursive flatten state: resolution roots, options, and the active
/// include chain used for cycle detection.
struct FlattenContext<'a> {
    paper_dir: &'a Path,
    include_dirs: &'a [PathBuf],
    options: &'a FlattenOptions,
    include_stack: Vec<PathBuf>,
}

impl FlattenContext<'_> {
    fn process_file<W: Write>(
        &mut self,
        file_path: &Path,
        writer: &mut W,
        is_main: bool,
    ) -> Result<()> {
        if !file_path.exists() {
            return Err(anyhow!(
                "could not find file to process: {}",
                file_path.display()
            ));
        }

        // Cycle detection over canonical paths: re-entering a file on
        // the active include chain would recurse forever.
        let canonical = file_path
            .canonicalize()
            .with_context(|| format!("canonicalizing {}", file_path.display()))?;
        if self.include_stack.contains(&canonical) {
            let chain = self
                .include_stack
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(" -> ");
            bail!(
                "include cycle detected: {} re-enters via {chain}",
                canonical.display(),
            );
        }
        self.include_stack.push(canonical);

        let result = self.process_lines(file_path, writer, is_main);

        self.include_stack.pop();
        result
    }

    fn process_lines<W: Write>(
        &mut self,
        file_path: &Path,
        writer: &mut W,
        is_main: bool,
    ) -> Result<()> {
        let input =
            File::open(file_path).with_context(|| format!("opening {}", file_path.display()))?;
        let reader = BufReader::new(input);

        for line_result in reader.lines() {
            let line = line_result.with_context(|| format!("reading {}", file_path.display()))?;
            self.dispatch_line(&line, writer, is_main)?;
        }
        Ok(())
    }

    fn dispatch_line<W: Write>(&mut self, line: &str, writer: &mut W, is_main: bool) -> Result<()> {
        let trimmed = line.trim_start();
        if trimmed.starts_with("\\begin{document}") || trimmed.starts_with("\\end{document}") {
            if is_main {
                writeln!(writer, "{line}")?;
            } else {
                writeln!(writer, "% {line}")?;
            }
            return Ok(());
        }
        if trimmed.starts_with("\\documentclass") {
            if is_main {
                writeln!(writer, "{line}")?;
            } else {
                writeln!(writer, "% {line}")?;
            }
            return Ok(());
        }
        if trimmed.starts_with("\\usepackage{subfiles}") {
            writeln!(writer, "% {line}")?;
            return Ok(());
        }
        if let Some((bib, rest)) = extract_braced_with_rest(trimmed, "\\addbibresource") {
            ensure_only_trailing_comment(line, "\\addbibresource", rest)?;
            return self.handle_bibliography(line, writer, &bib);
        }
        if let Some((included, surviving_tokens, false_branch)) =
            extract_if_file_exists_include(trimmed)
        {
            return self.handle_conditional_include(
                line,
                writer,
                &included,
                &surviving_tokens,
                &false_branch,
            );
        }
        if let Some((included, rest)) = extract_include_with_rest(trimmed) {
            ensure_only_trailing_comment(line, "include", rest)?;
            // An include annotated with `% flatten-ignore` is emitted commented
            // out and not recursed into. This is for build-generated includes
            // (e.g. a zref-xr cross-document bridge) that exist only in the build
            // tree and cannot resolve in a standalone, single-paper flatten.
            // Genuine \input typos still hard-fail, preserving strictness.
            if line.contains("flatten-ignore") {
                writeln!(writer, "% {line}")?;
                writeln!(
                    writer,
                    "% --- flatten-ignore: external include '{included}' omitted from flat build ---"
                )?;
                return Ok(());
            }
            return self.handle_include(line, writer, &included);
        }
        // A line the flattener does not handle must not smuggle an
        // active include through to the "flattened" output: silent
        // publication corruption is worse than a build failure.
        ensure_no_active_include(line)?;
        writeln!(writer, "{line}")?;
        Ok(())
    }
}

/// Byte position of the first unescaped `%`, if any.
const fn comment_start(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index += 2,
            b'%' => return Some(index),
            _ => index += 1,
        }
    }
    None
}

/// The semantic (pre-comment) part of a line.
fn active_part(line: &str) -> &str {
    comment_start(line).map_or(line, |index| &line[..index])
}

/// Reject semantic content after a recognized include command: the
/// flattener comments out the whole original line, so any trailing
/// tokens (`\input{a}\label{b}`) would be silently lost. Trailing
/// whitespace and comments are fine.
fn ensure_only_trailing_comment(line: &str, command: &str, rest: &str) -> Result<()> {
    if active_part(rest).trim().is_empty() {
        Ok(())
    } else {
        bail!(
            "unsupported include syntax in {line:?}: content after the {command} command \
             on the same line cannot be preserved; put the include alone on its line"
        )
    }
}

/// Reject a line carrying an active (uncommented) include command that
/// the dispatcher did not recognize — an embedded `\input`, a prefixed
/// include, or a malformed conditional. Emitting it verbatim would
/// leave an unresolved include in output that claims to be flat.
fn ensure_no_active_include(line: &str) -> Result<()> {
    let active = active_part(line);

    for command in ["\\input", "\\subfile", "\\addbibresource"] {
        let mut search = active;
        while let Some(position) = search.find(command) {
            let after = &search[position + command.len()..];
            // A longer control sequence (e.g. `\inputline`) is a
            // different macro, not an include.
            if !after.starts_with(|character: char| character.is_ascii_alphabetic()) {
                bail!(
                    "unsupported include syntax in {line:?}: {command} embedded in a line \
                     the flattener cannot preserve; put the include alone on its line"
                );
            }
            search = after;
        }
    }

    Ok(())
}

/// Extract `command{value}` at line start, returning the value and the
/// unparsed remainder of the line so the caller can reject trailing
/// semantic content.
fn extract_braced_with_rest<'a>(line: &'a str, command: &str) -> Option<(String, &'a str)> {
    let after = line.strip_prefix(command)?;
    take_braced(after.trim_start())
}

fn extract_include_with_rest(line: &str) -> Option<(String, &str)> {
    extract_braced_with_rest(line, "\\input")
        .or_else(|| extract_braced_with_rest(line, "\\subfile"))
}

fn extract_if_file_exists_include(line: &str) -> Option<(String, String, String)> {
    let after = line.strip_prefix("\\IfFileExists")?;
    let (_, after_probe) = take_braced(after.trim_start())?;
    let (true_branch, after_true) = take_braced(after_probe.trim_start())?;
    let (false_branch, after_false) = take_braced(after_true.trim_start())?;
    if !active_part(after_false).trim().is_empty() {
        return None;
    }
    extract_include_survivors(&true_branch)
        .map(|(included, survivors)| (included, survivors, false_branch))
}

fn take_braced(input: &str) -> Option<(String, &str)> {
    let input = input.trim_start();
    let body_start = input.find('{')? + 1;
    if body_start != 1 {
        return None;
    }

    let mut depth = 1usize;
    for (offset, ch) in input[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let body_end = body_start + offset;
                    let rest_start = body_end + ch.len_utf8();
                    return Some((
                        input[body_start..body_end].to_string(),
                        &input[rest_start..],
                    ));
                }
            }
            _ => {}
        }
    }
    None
}

fn extract_include_survivors(branch: &str) -> Option<(String, String)> {
    extract_include_survivors_for_command(branch, "\\input")
        .or_else(|| extract_include_survivors_for_command(branch, "\\subfile"))
}

fn extract_include_survivors_for_command(branch: &str, command: &str) -> Option<(String, String)> {
    let command_start = branch.find(command)?;
    let after_command = &branch[command_start + command.len()..];
    let braced = after_command.trim_start();
    let (included, rest) = take_braced(braced)?;
    let include_end = branch.len() - rest.len();

    let mut surviving_tokens = String::new();
    surviving_tokens.push_str(&branch[..command_start]);
    surviving_tokens.push_str(&branch[include_end..]);
    Some((included, surviving_tokens))
}

impl FlattenContext<'_> {
    fn handle_bibliography<W: Write>(
        &self,
        original_line: &str,
        writer: &mut W,
        bib_file: &str,
    ) -> Result<()> {
        writeln!(writer, "% {original_line}")?;
        let bib_path = resolve_reference(bib_file, self.paper_dir, self.include_dirs, "bib");

        if bib_path.exists() {
            writeln!(
                writer,
                "% --- BEGIN embedded bibliography from: {bib_file} ---"
            )?;
            writeln!(writer, "\\begin{{filecontents*}}{{{bib_file}}}")?;
            let bib =
                File::open(&bib_path).with_context(|| format!("opening {}", bib_path.display()))?;
            let reader = BufReader::new(bib);
            for line_result in reader.lines() {
                let line =
                    line_result.with_context(|| format!("reading {}", bib_path.display()))?;
                writeln!(writer, "{line}")?;
            }
            writeln!(writer, "\\end{{filecontents*}}")?;
            writeln!(writer, "\\addbibresource{{{bib_file}}}")?;
            writeln!(
                writer,
                "% --- END embedded bibliography from: {bib_file} ---"
            )?;
        } else if self.options.strict_bibliography {
            bail!(
                "bibliography file '{bib_file}' not found at '{}' (strict mode)",
                bib_path.display(),
            );
        } else {
            writeln!(
                writer,
                "% WARNING: Bibliography file '{bib_file}' not found at '{}'",
                bib_path.display()
            )?;
        }
        Ok(())
    }

    fn handle_conditional_include<W: Write>(
        &mut self,
        original_line: &str,
        writer: &mut W,
        included: &str,
        surviving_tokens: &str,
        false_branch: &str,
    ) -> Result<()> {
        writeln!(writer, "% {original_line}")?;
        let included_path = resolve_reference(included, self.paper_dir, self.include_dirs, "tex");
        if !included_path.exists() {
            // The probed file is absent, so LaTeX would take the false
            // branch. An empty false branch mirrors as a comment; a
            // nonempty one would have to be emitted (and possibly
            // flattened) to preserve semantics, which this line-based
            // flattener does not support — fail rather than silently
            // dropping it.
            if !false_branch.trim().is_empty() {
                bail!(
                    "unsupported conditional include in {original_line:?}: '{included}' is \
                     absent and the nonempty false branch would be silently dropped"
                );
            }
            writeln!(
                writer,
                "% --- flatten: conditional include '{included}' absent; false branch taken ---"
            )?;
            return Ok(());
        }
        writeln!(writer, "% --- BEGIN included content from: {included} ---")?;
        self.process_file(&included_path, writer, false)?;
        writeln!(writer, "% --- END included content from: {included} ---")?;
        if !surviving_tokens.trim().is_empty() {
            ensure_no_active_include(surviving_tokens)?;
            writeln!(writer, "{surviving_tokens}")?;
        }
        Ok(())
    }

    fn handle_include<W: Write>(
        &mut self,
        original_line: &str,
        writer: &mut W,
        included: &str,
    ) -> Result<()> {
        writeln!(writer, "% {original_line}")?;
        let included_path = resolve_reference(included, self.paper_dir, self.include_dirs, "tex");
        writeln!(writer, "% --- BEGIN included content from: {included} ---")?;
        self.process_file(&included_path, writer, false)?;
        writeln!(writer, "% --- END included content from: {included} ---")?;
        Ok(())
    }
}
