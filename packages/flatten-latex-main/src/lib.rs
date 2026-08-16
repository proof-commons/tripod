//! `flatten-latex-main` library: produce a single flat `.tex` from a paper's
//! `main.tex`, inlining `\input{}` / `\subfile{}` and embedding the
//! `\addbibresource{}` BibTeX file inside `\begin{filecontents*}` blocks.
//!
//! Port of the original `bin/create_flat_main.pl` script. The binary entry
//! point lives in [`src/bin/flatten-latex-main.rs`](../bin/flatten-latex-main.rs).
//!
//! # Boundary
//!
//! The flattener is **directory-blind**. It performs no filesystem
//! lookup to locate a referenced file and never learns the paper or
//! build directory: a reference resolves only by matching its trailing
//! path components against the caller's fixed allowlist, and exactly
//! one match is required. An absolute include or a parent-directory
//! escape therefore cannot be read or published, because the allowlist
//! is what confines the paths source text may select.
//!
//! The allowlist entries are caller-granted capabilities. Each is
//! validated as an existing regular file (a symlink is refused) before
//! anything is read, but that is a role check, not a filesystem
//! boundary: beneath a supplied entry the host owns what the path
//! resolves to, and nothing here closes a time-of-check/time-of-use
//! race. `main_file` is the trusted entry point and need not appear on
//! the list.
//!
//! Output is reproducible — the same inputs flatten to byte-identical
//! output, with no timestamps or environment-dependent values — and is
//! staged in a uniquely named temporary beside the destination, then
//! renamed, so a failed flatten leaves no partial file and concurrent
//! flattens never share a staging path.
//!
//! # Map
//!
//! - [`flatten`] — the single entry point: entry file, allowlist,
//!   output path, options.
//! - [`FlattenOptions`] — currently one switch,
//!   [`FlattenOptions::strict_bibliography`].
//!
//! Failures are `anyhow::Error` values carrying the offending
//! reference or line.
//!
//! # Example
//!
//! ```no_run
//! // Reads the supplied files and writes the output, so this compiles
//! // but does not run as a doctest.
//! use std::path::{Path, PathBuf};
//!
//! use flatten_latex_main::{FlattenOptions, flatten};
//!
//! let allowed = vec![
//!     PathBuf::from("papers/attestation/sections/01_intro.tex"),
//!     PathBuf::from("papers/attestation/references.bib"),
//! ];
//!
//! flatten(
//!     Path::new("papers/attestation/main.tex"),
//!     &allowed,
//!     Path::new("build/main-flat.tex"),
//!     &FlattenOptions { strict_bibliography: true },
//! )?;
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! See `README.md` in this package for the resolution rules, the
//! rewriting table, and the failure catalogue.

use std::ffi::{OsStr, OsString};
use std::fs::{File, create_dir_all};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Component, Path, PathBuf};

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

/// Flatten `main_file` into `output_file`, inlining `\input{}` / `\subfile{}`
/// and `\addbibresource{}` references.
///
/// The flattener is **directory-blind**: it performs no filesystem lookup to
/// locate a referenced file and never learns the paper or build directory. It
/// works only from `allowed_files` — the fixed set of inlineable files the
/// caller (e.g. meson) supplies. A reference resolves by matching its trailing
/// path components (filename first, then enclosing folders) against that list;
/// exactly one match is required. Zero matches is a hard "not supplied" error
/// and two or more is an ambiguity error.
///
/// Because a reference can only ever name a file already on `allowed_files`, an
/// absolute include (`\input{/etc/passwd}`) and a parent-directory escape
/// (`\input{../../secret}`) cannot be read or published: the allowlist is what
/// confines the paths source text may select (`[ADR017-rule:path:derived-references]`).
///
/// The allowlist entries themselves are caller-granted capabilities. Each is
/// validated as an existing regular file before anything is read or staged,
/// but that is a role check, not a filesystem boundary: beneath a supplied
/// entry the host owns what the path resolves to, and the flattener neither
/// walks ancestors for aliases nor claims to close a
/// time-of-check/time-of-use race (`[ADR017-rule:path:toctou]`). `main_file`
/// is the trusted entry point and need not appear in `allowed_files`.
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
/// flattened output, a reference that matches zero or multiple supplied files,
/// a missing `main_file`, an include cycle, or (with
/// [`FlattenOptions::strict_bibliography`]) a bibliography reference not on the
/// supplied list.
pub fn flatten(
    main_file: &Path,
    allowed_files: &[PathBuf],
    output_file: &Path,
    options: &FlattenOptions,
) -> Result<()> {
    // Confinement before any read or staging: a failed validation must
    // leave an existing output untouched and no staging file behind.
    ensure_regular_file(main_file, "main file")?;
    for allowed in allowed_files {
        ensure_regular_file(allowed, "supplied file")?;
    }

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

    write_flattened(main_file, allowed_files, staged.as_file(), options)?;

    staged
        .persist(output_file)
        .with_context(|| format!("renaming into {}", output_file.display()))?;

    Ok(())
}

/// File-type validation of a supplied path: the flattener inlines a
/// regular file, so a directory, device, or socket is a caller error
/// reported before anything is read.
///
/// This is a role check, not a filesystem boundary
/// (`[ADR017-rule:path:explicit-paths]`). The allowlist is what
/// confines *what source text may select*; beneath a supplied entry
/// the host owns what the path resolves to. The ancestor-by-ancestor
/// symlink walk this function once performed has been removed: it
/// could not see bind mounts, FUSE aliases, or a replacement between
/// the check and the open, so it bought complexity rather than a
/// boundary. Nothing here closes a time-of-check/time-of-use race.
fn ensure_regular_file(path: &Path, role: &str) -> Result<()> {
    let metadata = std::fs::symlink_metadata(path)
        .with_context(|| format!("inspecting {role} {}", path.display()))?;

    if metadata.file_type().is_symlink() {
        bail!(
            "{role} {} is a symbolic link; the flattener inlines regular files named \
             on its fixed list, and never a link's resolved target",
            path.display(),
        );
    }

    if !metadata.file_type().is_file() {
        bail!("{role} {} is not a regular file", path.display());
    }

    Ok(())
}

fn write_flattened(
    main_file: &Path,
    allowed_files: &[PathBuf],
    staged: &File,
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
        allowed: allowed_files,
        options,
        include_stack: Vec::new(),
    };

    context.process_file(main_file, &mut writer, true)?;

    writer.flush()?;
    Ok(())
}

/// Resolve a LaTeX reference name to exactly one file on the supplied
/// allowlist by matching trailing path components — the filename first, then
/// each enclosing folder the reference names. The flattener performs no
/// filesystem lookup: a reference resolves only if it names a supplied file,
/// so an absolute path, a `..` escape, or a symlink target off the list is
/// unreachable.
///
/// A reference whose final component has no extension also tries the
/// `default_ext` variant.
///
/// # Errors
///
/// Fails when the reference contains a `..` component, matches no supplied
/// file, or matches more than one supplied file (ambiguous).
fn resolve_reference<'a>(
    name: &str,
    allowed: &'a [PathBuf],
    default_ext: &str,
) -> Result<&'a Path> {
    let mut wanted = reference_components(name)?;
    let mut matches = suffix_matches(allowed, &wanted);

    // A reference without an extension (e.g. `\input{section}`) also tries
    // `section.<default_ext>`.
    if matches.is_empty() && !name.contains('.') {
        if let Some(last) = wanted.last_mut() {
            last.push(".");
            last.push(default_ext);
        }
        matches = suffix_matches(allowed, &wanted);
    }

    match matches.as_slice() {
        [single] => Ok(*single),
        [] => bail!(
            "reference {name:?} does not match any file supplied to the flattener; \
             the flattener inlines only files on the caller's fixed --file list"
        ),
        many => bail!(
            "reference {name:?} ambiguously matches {} supplied files: {}",
            many.len(),
            many.iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", "),
        ),
    }
}

/// Split a LaTeX reference (always `/`-separated) into trailing-match
/// components, rejecting an absolute reference or a `..` traversal outright.
fn reference_components(name: &str) -> Result<Vec<OsString>> {
    if name.starts_with('/') {
        bail!("reference {name:?} is an absolute path; references must name a supplied file");
    }
    let mut components = Vec::new();
    for part in name.split('/') {
        match part {
            "" | "." => {}
            ".." => bail!(
                "reference {name:?} contains a parent-directory ('..') component; \
                 references must name a supplied file, not a path to traverse"
            ),
            other => components.push(OsString::from(other)),
        }
    }
    if components.is_empty() {
        bail!("reference {name:?} is empty");
    }
    Ok(components)
}

/// Every supplied file whose path ends with exactly `wanted` (compared by
/// path component, so `01_interface.tex` never matches `x01_interface.tex`).
fn suffix_matches<'a>(allowed: &'a [PathBuf], wanted: &[OsString]) -> Vec<&'a Path> {
    allowed
        .iter()
        .filter(|candidate| path_ends_with_components(candidate, wanted))
        .map(PathBuf::as_path)
        .collect()
}

fn path_ends_with_components(path: &Path, wanted: &[OsString]) -> bool {
    let normal: Vec<&OsStr> = path
        .components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part),
            _ => None,
        })
        .collect();
    if wanted.len() > normal.len() {
        return false;
    }
    normal[normal.len() - wanted.len()..]
        .iter()
        .zip(wanted.iter())
        .all(|(have, want)| *have == want.as_os_str())
}

/// Recursive flatten state: the fixed allowlist of inlineable files, options,
/// and the active include chain used for cycle detection.
struct FlattenContext<'a> {
    allowed: &'a [PathBuf],
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
        // Cycle detection over the active include chain, by path.
        // Every reference resolves to an entry of the caller's fixed
        // allowlist, so every frame below the entry point is one of a
        // finite set of paths: an unbounded chain must repeat a path,
        // and comparing paths therefore terminates every cycle. The
        // former device/inode comparison detected nothing this does
        // not (`[ADR017-rule:path:output-roles]`).
        let frame = file_path.to_path_buf();
        if self.include_stack.contains(&frame) {
            let chain = self
                .include_stack
                .iter()
                .map(|entry| entry.display().to_string())
                .collect::<Vec<_>>()
                .join(" -> ");
            bail!(
                "include cycle detected: {} re-enters via {chain}",
                frame.display(),
            );
        }
        self.include_stack.push(frame);

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
        if let Some(conditional) = extract_if_file_exists_include(trimmed) {
            return self.handle_conditional_include(line, writer, &conditional);
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

/// Resolve one conditional probe against the fixed list.
///
/// `Ok(Some)` for exactly one match, `Ok(None)` for no match (LaTeX
/// takes the false branch), and `Err` for a probe the flattener must
/// not guess about: an absolute path, a traversal, or an ambiguous
/// match.
fn resolve_probe<'a>(
    name: &str,
    allowed: &'a [PathBuf],
    default_ext: &str,
) -> Result<Option<&'a Path>> {
    let mut wanted = reference_components(name)?;
    let mut matches = suffix_matches(allowed, &wanted);

    if matches.is_empty() && !name.contains('.') {
        if let Some(last) = wanted.last_mut() {
            last.push(".");
            last.push(default_ext);
        }
        matches = suffix_matches(allowed, &wanted);
    }

    match matches.as_slice() {
        [single] => Ok(Some(*single)),
        [] => Ok(None),
        many => bail!(
            "conditional probe {name:?} ambiguously matches {} supplied files: {}",
            many.len(),
            many.iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", "),
        ),
    }
}

fn extract_include_with_rest(line: &str) -> Option<(String, &str)> {
    extract_braced_with_rest(line, "\\input")
        .or_else(|| extract_braced_with_rest(line, "\\subfile"))
}

/// One parsed `\\IfFileExists{probe}{true}{false}` line whose true
/// branch carries exactly one include.
struct ConditionalInclude {
    probe: String,
    included: String,
    surviving_tokens: String,
    false_branch: String,
}

fn extract_if_file_exists_include(line: &str) -> Option<ConditionalInclude> {
    let after = line.strip_prefix("\\IfFileExists")?;
    let (probe, after_probe) = take_braced(after.trim_start())?;
    let (true_branch, after_true) = take_braced(after_probe.trim_start())?;
    let (false_branch, after_false) = take_braced(after_true.trim_start())?;
    if !active_part(after_false).trim().is_empty() {
        return None;
    }
    extract_include_survivors(&true_branch).map(|(included, surviving_tokens)| ConditionalInclude {
        probe,
        included,
        surviving_tokens,
        false_branch,
    })
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

        match resolve_reference(bib_file, self.allowed, "bib") {
            Ok(bib_path) => {
                let bib_path = bib_path.to_path_buf();
                writeln!(
                    writer,
                    "% --- BEGIN embedded bibliography from: {bib_file} ---"
                )?;
                writeln!(writer, "\\begin{{filecontents*}}{{{bib_file}}}")?;
                let bib = File::open(&bib_path)
                    .with_context(|| format!("opening {}", bib_path.display()))?;
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
            }
            Err(_) if self.options.strict_bibliography => {
                bail!(
                    "bibliography file '{bib_file}' not found among the supplied files (strict mode)"
                );
            }
            Err(_) => {
                writeln!(
                    writer,
                    "% WARNING: Bibliography file '{bib_file}' not found among the supplied files"
                )?;
            }
        }
        Ok(())
    }

    fn handle_conditional_include<W: Write>(
        &mut self,
        original_line: &str,
        writer: &mut W,
        conditional: &ConditionalInclude,
    ) -> Result<()> {
        let ConditionalInclude {
            probe,
            included,
            surviving_tokens,
            false_branch,
        } = conditional;
        writeln!(writer, "% {original_line}")?;
        // Branch selection follows the PROBE, exactly as LaTeX selects
        // it, under the fixed-list model: a file exists iff it resolves
        // on the supplied list. Deciding by the nested include instead
        // would flatten `\IfFileExists{a}{\input{b}}{}` under different
        // branch semantics from TeX when only one of the two exists.
        let Some(probe_path) = resolve_probe(probe, self.allowed, "tex")? else {
            // The probed file is not on the supplied list, so LaTeX
            // takes the false branch. An empty false branch mirrors as
            // a comment; a nonempty one would have to be emitted (and
            // possibly flattened) to preserve semantics, which this
            // line-based flattener does not support — fail rather than
            // silently dropping it.
            if !false_branch.trim().is_empty() {
                bail!(
                    "unsupported conditional include in {original_line:?}: probe '{probe}' is \
                     absent and the nonempty false branch would be silently dropped"
                );
            }
            writeln!(
                writer,
                "% --- flatten: conditional probe '{probe}' absent; false branch taken ---"
            )?;
            return Ok(());
        };
        let included_path = resolve_reference(included, self.allowed, "tex")
            .with_context(|| {
                format!(
                    "conditional include in {original_line:?}: probe '{probe}' exists but the \
                     true branch include cannot be flattened"
                )
            })?
            .to_path_buf();
        // Restricted exact support: the probe and the include must name
        // one supplied file. Divergent probe/include pairs would need
        // real conditional evaluation to preserve semantics.
        if included_path != probe_path {
            bail!(
                "unsupported conditional include in {original_line:?}: probe '{probe}' and \
                 include '{included}' resolve to different supplied files"
            );
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
        let included_path = resolve_reference(included, self.allowed, "tex")?.to_path_buf();
        writeln!(writer, "% --- BEGIN included content from: {included} ---")?;
        self.process_file(&included_path, writer, false)?;
        writeln!(writer, "% --- END included content from: {included} ---")?;
        Ok(())
    }
}
