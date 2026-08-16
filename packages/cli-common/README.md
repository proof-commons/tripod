# `cli-common`

`cli-common` is the shared scaffolding every command-line tool in this
workspace is built on. It owns the ADR-010 command-line output
contract: what may appear on stdout, what must appear on stderr, which
exit classes exist, and what must never be printed at all.

It is infrastructure, not policy about any particular subject. It
contains no repository knowledge: a helper binary receives its subjects
as explicit arguments (ADR-014) and uses this crate only to parse argv,
emit machine-readable records, publish outputs, and exit correctly.

## The output contract

Three rules govern every binary that uses this crate.

1. **Stdout carries result data only.** At most one JSON object plus a
   trailing newline. A side-effect command writes nothing to stdout.
2. **Everything else is JSON on stderr.** Diagnostics, status, help
   text, version strings, usage errors, and panic reports are all
   control-plane records, never prose on stdout.
3. **Three exit classes.** `0` success, `1` failure, `2` usage
   (including argv errors and terminal-stdout refusal).

Result data is refused when stdout is a terminal, because a JSON result
is machine input:

```text
$ my-tool
{"schema":1,"command":"my-tool","kind":"tty_refusal","message":"stdout is a terminal; pipe to a file or another process","fields":{"type":"tty_refusal","exit_code":2,"stream":"stdout"}}
$ echo $?
2
```

Help and version are control-plane data too, so they land on stderr
while stdout stays empty and the exit class is success:

```text
$ my-tool --help 2>help.json >result.json
$ echo $?
0
$ cat result.json    # empty
$ head -c 60 help.json
{"schema":1,"command":"my-tool","kind":"help","message":"h
```

A usage error reports the clap error *class* and never the offending
argument text, which may embed an inline secret:

```text
{"schema":1,"command":"my-tool","kind":"usage_error","message":"invalid command-line arguments","fields":{"type":"usage_error","exit_code":2,"clap_error_kind":"missing_required_argument"}}
```

## Quickstart

A complete helper binary. `BaseArgs` contributes the shared `--debug`
flag; `run_stdout_json_command` installs the JSON panic hook,
initialises JSON stderr tracing with `RUST_LOG` / `--debug` precedence,
refuses terminal stdout, writes the returned value as the single JSON
result object, and maps a returned error onto exit class 1 after
passing its display text through redaction.

```rust,no_run
// Compiled but not executed: `main` parses process argv and installs a
// process-wide panic hook, neither of which belongs in a test harness.
use std::process::ExitCode;

use clap::Parser;
use cli_common::{BaseArgs, parse_args_or_exit, run_stdout_json_command};
use serde::Serialize;

const COMMAND_NAME: &str = "word-count";

#[derive(Parser)]
#[command(name = "word-count", version, about = "Count words in a phrase")]
struct Args {
    #[command(flatten)]
    base: BaseArgs,

    /// The phrase to measure.
    #[arg(long, value_name = "TEXT")]
    phrase: String,
}

#[derive(Serialize)]
struct Report {
    schema: u32,
    words: usize,
}

fn main() -> ExitCode {
    // Installed before parsing: a panic anywhere in the process must
    // still fail closed with one JSON record, never Rust's text hook.
    cli_common::install_json_panic_hook(COMMAND_NAME);
    let args = parse_args_or_exit::<Args>();

    run_stdout_json_command(
        COMMAND_NAME,
        args.base.debug,
        tracing::Level::INFO,
        || -> Result<Report, std::io::Error> {
            Ok(Report {
                schema: 1,
                words: args.phrase.split_whitespace().count(),
            })
        },
    )
}
```

Records are ordinary values, so their shape can be asserted directly:

```rust
use cli_common::{CommandExit, ControlPlaneRecord, ControlPlaneRecordKind};

let record = ControlPlaneRecord::usage_error("my-tool", "missing_required_argument");

assert_eq!(record.kind, ControlPlaneRecordKind::UsageError);
assert_eq!(record.message, "invalid command-line arguments");
assert_eq!(CommandExit::Usage.code(), 2);
```

## Public API tour

The crate is a single flat module; `publication` is private and its
published items are re-exported here.

### Exit classes

- `enum CommandExit { Success, Failure, Usage }` — `code(self) -> u8`
  (0/1/2), `exit_code(self) -> ExitCode`, `from_code(u8) -> Option<Self>`.
- `exit_with(exit: CommandExit) -> !` — terminate with that class.

### Control-plane records

- `struct ControlPlaneRecord { schema, command, kind, message, fields }`
  — `schema` is always `CONTROL_PLANE_SCHEMA` (currently `1`).
- Constructors: `new`, `help`, `version`, `usage_error`, `tty_refusal`,
  `panic`, `status`, `diagnostic`. `usage_error` fixes its own message;
  the caller supplies only the clap error class.
- `enum ControlPlaneRecordKind { Help, Version, UsageError, TtyRefusal, Panic, Status, Diagnostic }` and the matching
  `enum ControlPlaneFields` payloads; `enum StandardStream { Stdout, Stderr }`.
- `emit_control_plane_record(&ControlPlaneRecord) -> io::Result<()>` —
  one JSON line on stderr under a process-wide write lock. Does not
  depend on `tracing`, so it is usable before a subscriber exists.
- `struct CliExit { record, exit }` — a record paired with the class to
  exit under; `new`, `exit_code`.

### Argument parsing

- `parse_args_from<T: Parser, I, A>(args: I) -> Result<T, CliExit>` —
  clap help, version, and usage failures become a `CliExit` rather than
  clap's own text output. The caller writes the record and returns the
  exit code.
- `parse_args_or_exit<T: Parser>() -> T` — the same over process argv,
  emitting the record and exiting directly.
- `struct BaseArgs { pub debug: bool }` — flatten into a binary's `Args`
  with `#[command(flatten)]` to gain `--debug`.
- `struct CheckOutputArgs { pub report: Option<PathBuf>, pub stamp: Option<PathBuf> }`
  — the reciprocal `--report` / `--stamp` pair for checker binaries.

### Redaction

- `is_sensitive_field_name(&str) -> bool` — name matches a secret marker
  (token, password, api key, credential, private key, and similar).
- `redact_field_value(field_name, value) -> Cow<str>` — replaces the
  value of a sensitive field, otherwise redacts URL credentials.
- `redact_url(&str) -> Cow<str>` — strips userinfo credentials, secret
  query parameters, and OAuth-style fragment pieces from a URL of any
  scheme; a non-URL is returned unchanged.
- `redact_database_url(&str) -> Cow<str>` — retained name, delegates to
  `redact_url`.
- `redact_text(&str) -> String` — conservative sanitisation of free-form
  diagnostic text: `key=value` tokens with sensitive keys, and
  URL-shaped tokens, are scrubbed with whitespace preserved. A
  URL-shaped token that fails to parse fails closed to `REDACTED`.
- `const REDACTED: &str` — the placeholder these paths substitute.

### Process scaffolding

- `install_json_panic_hook(command_name: &str)` — replaces Rust's text
  panic hook with one best-effort JSON record, then exits class 1.
- `set_panic_payload_reporting_enabled(bool)` — payload reporting starts
  **disabled**; call with the parsed `--debug` value once argv is known.
- `tracing_filter(debug: bool, default_level: tracing::Level) -> TracingFilter`
  — resolves the directive with `RUST_LOG` first, then `--debug`, then
  the supplied default; the chosen `TracingFilterSource` is reported.
- `init_json_tracing`, `try_init_json_tracing`,
  `init_json_tracing_with_debug`, `try_init_json_tracing_with_debug` —
  JSON subscriber on stderr. The `try_` forms return `false` when a
  subscriber is already installed.
- `stdout_tty_refusal_record(&str) -> Option<ControlPlaneRecord>` and
  `refuse_if_stdout_is_tty(&str)` — the terminal check as a value, and
  as an emit-and-exit.
- `emit<T: Serialize>(&T) -> io::Result<()>` — one JSON object plus a
  trailing newline on stdout.

### Runners

Each installs the panic hook, sets payload reporting from `debug`, and
initialises JSON tracing before running the closure.

- `run_stdout_json_command(name, debug, default_level, run) -> ExitCode`
  — refuses terminal stdout, then emits the closure's `Ok` value.
- `run_no_stdout_command(name, debug, default_level, run) -> ExitCode`
  — side-effect command; deliberately performs **no** TTY refusal
  because it produces no stdout result data.
- `run_no_stdout_command_async(...) -> ExitCode` — the same, awaiting a
  returned future.
- `run_check_command(name, debug, default_level, &CheckOutputArgs, run) -> ExitCode`
  — the ADR-014 checker shape described below.

### Output roles and stamps

- `destination_identity(&Path) -> DestinationIdentity` — lexical
  normalisation to an absolute path (relative paths resolve against the
  process working directory). Not canonicalisation: no symlink
  resolution, no device or inode comparison.
- `ensure_distinct_outputs(&[(&str, &Path)]) -> Result<(), AliasedOutputs>`
  — refuses two output roles naming one destination, before any
  semantic work or file mutation.
- `struct AliasedOutputs { first, second }` — the offending role pair.
- `touch_stamp(&Path) -> io::Result<()>` — create an empty stamp, or
  re-date an existing empty one. An existing **nonempty** stamp is
  refused rather than truncated.
- `finish_check_command(name, &CheckOutputArgs, &report) -> Result<(), CheckResultError>`
  — publishes a successful check result under the active mode.
- `enum CheckResultError { TtyRefusal, InvalidMode, Io, AliasedOutputs }`.

A checker runs in one of exactly two modes:

- **direct** (neither `--report` nor `--stamp`): the JSON report goes to
  stdout, refused on a terminal, and no stamp is touched;
- **build** (both): the report is published compare-if-changed to its
  explicit path and only then is the stamp touched, so a failed report
  write never leaves a fresh success stamp. Stdout stays empty.

`clap` enforces the reciprocal requirement, so a lone `--report` or
`--stamp` is a usage error before the command runs.

### Publication

- `enum PublicationMode { Public, Executable }` — `octal(self) -> u32`
  yields `0o644` / `0o755`.
- `set_publication_mode(&File, PublicationMode) -> io::Result<()>` —
  applied exactly, not masked by the umask, before a staged file is
  renamed into place.
- `struct PublicationAsset<'a> { role, path, bytes }` — one output of a
  multi-output command; `role` is a stable diagnostic name, never file
  contents.
- `publish_batch(&[PublicationAsset]) -> Result<Vec<PublicationResult>, BatchPublicationError>`
  — derives, compares, and stages **every** member before the first
  rename, so a staging failure changes no destination.
- `struct PublicationChange { bytes_changed, mode_changed }` with
  `unchanged(self) -> bool`, and `struct PublicationResult { role, path, change }`
  in input order.
- `enum BatchPublicationError { AliasedOutputs, Stage, Publish }`.

Publication freshness has two components: bytes **and** mode. A
destination whose bytes are current but whose mode is wrong is repaired
in place, leaving the bytes and their modification time untouched, and
the repair is reported separately so it is never mistaken for new
content.

## Error handling

Fallible entry points return `io::Result` (`emit`,
`emit_control_plane_record`, `touch_stamp`, `set_publication_mode`) or a
typed error: `CliExit` for parse control flow, `AliasedOutputs` for role
collisions, `CheckResultError` for check publication, and
`BatchPublicationError` for batch publication. The batch variants say
how far the operation got: `AliasedOutputs` before any filesystem
effect, `Stage` with every final destination untouched, and `Publish`
after staging succeeded, where earlier members of the batch may already
carry new bytes.

The runners are the boundary where an arbitrary error becomes an exit
code. They log the error's display text through `redact_text` centrally,
so an individual command does not have to remember that its own error
type might embed a credential URL.

Atomicity limits are stated, not implied. `publish_batch` performs all
staging before the first rename, but multiple renames are not one
filesystem transaction: a failure partway leaves a partial publication,
the command fails, and the next successful run repairs the set.

## What this crate deliberately does not do

- **No human-readable output mode.** There is no `--format=text`, no
  pretty printer, and no partially structured mode. Prose on stdout
  would break the contract even for a successful run.
- **No echoing of argv.** Usage-error records carry the clap error class
  and nothing else. The rendered clap message reproduces the offending
  argument, which may embed an inline secret, so it is dropped wholesale
  rather than heuristically filtered. Help and version text is generated
  from the static command definition and is therefore kept verbatim.
- **No panic payloads by default.** Payload reporting is off until a
  parsed `--debug` enables it, so a panic before argument parsing
  reports thread and location only.
- **No credential-bearing text.** Redaction is applied by the crate's
  own paths; it is a safety net for untrusted echo text, not a licence
  to pass arbitrary strings through. Structured records should prefer
  known-safe fields to free text.
- **No filesystem identity claims.** `destination_identity` is lexical.
  It catches two roles spelled differently for one destination; it makes
  no claim about symlinks, hard links, mounts, or a host replacing a
  path between the check and the write.
- **No path discovery.** Nothing here resolves a repository path from a
  compiled location; subjects arrive as arguments.
- **No transactional multi-file publication.** See the atomicity limit
  above.

## Consumers

Every first-party command-line tool in the workspace depends on this
crate: `tripod-artifacts` (`generate-all`, `check-generated`),
`tripod-document-stamps` (`attestation-stamps`), `execwrap`,
`flatten-latex-main`, `labels` (`check-labels`, `check-plans`,
`generate-label-registers`, `census-audit`, `check-forbidden-text`), and
`target-elements-conformance`.

The publication helpers additionally back the generated-artifact
batch publication and the checker report/stamp lanes wired in the meson
build.

## Acknowledgements

We thank the Torrust developers for the inspiration for this command-line tooling.
