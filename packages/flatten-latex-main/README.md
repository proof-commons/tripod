# `flatten-latex-main`

`flatten-latex-main` turns a paper's multi-file LaTeX source into a
single self-contained `.tex` file. It inlines `\input{}` and
`\subfile{}` references and embeds the bibliography named by
`\addbibresource{}` inside a `filecontents` block, so the result
compiles on its own with no surrounding directory.

The workspace uses it to produce the standalone source artifact that
accompanies the published paper. It is the Rust replacement for the
original Perl flattening script `bin/create_flat_main.pl`, and the
binary follows the ADR-010 output contract through `cli-common`.

## The fixed-list model

This is the single most important thing to understand about the
package: **the flattener never looks anything up on the filesystem.**

It never learns the paper directory or the build directory. The caller
supplies a fixed list of files that may be inlined, and a reference
inside the source resolves only by matching its **trailing path
components** against that list — the filename first, then each
enclosing folder the reference names. Exactly one match is required:

- zero matches is a hard "not supplied" error;
- two or more matches is an ambiguity error.

Component-wise comparison means `01_intro.tex` never matches
`x01_intro.tex`. A reference whose final component has no extension also
tries the default extension (`tex` for includes, `bib` for
bibliographies), so `\input{sections/01_intro}` resolves.

Because a reference can only ever name a file already on the list, an
absolute include such as `\input{/etc/passwd}` and a traversal such as
`\input{../../secret}` cannot be read or published. Both spellings are
rejected outright as well.

The list entries themselves are caller-granted capabilities. Each is
validated as an existing regular file before anything is read; a
symbolic link is refused, because the flattener inlines files named on
its list and never a link's resolved target. That is a role check, not
a filesystem boundary: beneath a supplied entry the host owns what the
path resolves to, and nothing here closes a time-of-check/time-of-use
race. The entry point file is trusted and need not appear on the list.

## Quickstart

```rust,no_run
// Reads the supplied files and writes the output file, so this compiles
// but is not run as a doctest.
use std::path::{Path, PathBuf};

use flatten_latex_main::{FlattenOptions, flatten};

fn main() -> anyhow::Result<()> {
    // The fixed list of inlineable files, normally assembled by the
    // build system rather than by hand.
    let allowed = vec![
        PathBuf::from("papers/attestation/sections/01_intro.tex"),
        PathBuf::from("papers/attestation/sections/02_model.tex"),
        PathBuf::from("papers/attestation/references.bib"),
    ];

    flatten(
        Path::new("papers/attestation/main.tex"),
        &allowed,
        Path::new("build/main-flat.tex"),
        &FlattenOptions {
            // Release builds should set this: a silently dropped
            // bibliography is a broken artifact, not a warning.
            strict_bibliography: true,
        },
    )
}
```

## Public API tour

The crate is a single flat module with one entry point.

- `flatten(main_file: &Path, allowed_files: &[PathBuf], output_file: &Path, options: &FlattenOptions) -> anyhow::Result<()>`
  — flatten `main_file` into `output_file`, resolving every reference
  against `allowed_files`. Validation of the entry point and of every
  list entry happens before any read or staging, so a rejected call
  leaves an existing output untouched and no staging file behind.
- `struct FlattenOptions { pub strict_bibliography: bool }` —
  `Default` gives lenient bibliography handling. Implements `Debug`,
  `Default`, and `Clone`.

## Output shape

The output opens with a fixed four-line comment header — no timestamps,
no environment-dependent values — so the same inputs always flatten to
byte-identical output.

Each original directive is preserved as a comment and replaced by the
material it named:

| Source line                        | In the flattened output                                            |
|------------------------------------|--------------------------------------------------------------------|
| `\input{...}` / `\subfile{...}`    | commented out, then the file's content between BEGIN/END markers    |
| `\addbibresource{...}`             | commented out, then the bibliography inside a `filecontents` block followed by a fresh `\addbibresource` |
| `\documentclass`                   | kept in the entry file, commented out in every included file        |
| `\begin{document}` / `\end{document}` | kept in the entry file, commented out in every included file     |
| `\usepackage{subfiles}`            | always commented out                                                |
| anything else                      | copied through unchanged                                            |

Includes are recursive, and the active include chain is tracked so a
cycle is reported as an error rather than expanding forever.

Two special cases:

- An include annotated `% flatten-ignore` is commented out and not
  recursed into, with a marker comment recording the omission. This
  exists for build-generated includes that only exist in the build tree
  and cannot resolve in a standalone single-paper flatten. A genuine
  typo in an ordinary include still fails hard.
- `\IfFileExists{probe}{true}{false}` is supported in a restricted exact
  form. Branch selection follows the **probe**, exactly as LaTeX selects
  it, under the fixed-list model: the probed file "exists" precisely
  when it resolves on the list. An absent probe with an empty false
  branch becomes a marker comment; an absent probe with a nonempty false
  branch, or a probe and include resolving to different supplied files,
  is an error rather than a silent guess.

## Error handling

Every failure is an `anyhow::Error` naming the offending reference or
line. The catalogue:

- **entry point or list entry invalid** — missing, not a regular file,
  or a symbolic link;
- **reference not supplied** — matched no file on the list;
- **reference ambiguous** — matched more than one, with all matches
  named;
- **reference malformed** — absolute, containing `..`, or empty;
- **include cycle** — the re-entered file and the active chain;
- **unsupported include syntax** — semantic content after a recognised
  include on the same line, or an include command embedded in a line the
  dispatcher cannot preserve. The flattener comments out the whole
  original line, so trailing tokens would be silently lost; a line
  carrying an active include the dispatcher did not recognise is refused
  rather than emitted verbatim, because an unresolved include in output
  that claims to be flat is silent publication corruption;
- **unsupported conditional** — the two cases above;
- **missing bibliography** — an error under `strict_bibliography`,
  otherwise a warning comment in the output;
- **I/O failures** — reported with the path being read or written.

Failure never leaves a partial artifact. The output is written to a
uniquely named temporary in the destination directory and renamed only
on success; on failure the temporary is removed when it is dropped. The
staging name is unique per invocation, so two concurrent flattens
targeting one directory never share a staging path.

## Command-line contract

```text
flatten-latex-main --main FILE --output FILE \
  --file FILE [--file FILE]... [--strict-bib] [--debug]
```

`--file` is repeated once per inlineable file and is required. The
binary is a side-effect command: nothing goes to stdout, diagnostics are
JSON on stderr, and the exit classes are the ADR-010 baseline (0 success,
1 failure, 2 usage).

## What this package deliberately does not do

- **No directory search.** No include path, no working-directory
  fallback, no extension probing beyond the single default-extension
  retry. If a file is not on the list, it does not exist as far as the
  flattener is concerned.
- **No symlink following for supplied files.** A link on the list is
  refused, not resolved.
- **No filesystem security claim.** The allowlist confines what source
  text may select; it does not confine what the host maps beneath a
  supplied path, and it does not close a time-of-check/time-of-use
  window.
- **No LaTeX evaluation.** It is a line-based rewriter, not a TeX
  interpreter. Macro-expanded includes, includes built from variables,
  and general conditional evaluation are unsupported and are refused
  rather than guessed at.
- **No silent degradation.** An unresolvable include, an ambiguous
  reference, or an unpreservable line fails the build. The one lenient
  path is a missing bibliography with `strict_bibliography` unset, which
  emits a warning comment — and release builds are expected to set the
  flag.
- **No timestamps or environment values in the output.**
- **No PDF or compilation.** It produces flattened source only.

## Consumers

The meson build under `papers/attestation/` invokes the binary to
produce the flattened standalone source that ships alongside the paper,
passing the inlineable file set from its hand-managed census (ADR-014)
rather than by globbing. The package depends on `cli-common` for the
ADR-010 output contract.
