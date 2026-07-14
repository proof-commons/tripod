# ADR-010: Command-Line Output Contract

**Status:** Decided and implemented
**Scope:** every first-party executable in this workspace
**Implemented by:** `packages/cli-common`

Adapted from an upstream architecture decision record ("global command-line
output contract"); summarized and re-scoped for this repository.

---

## Context

The workspace ships several command-line executables — build helpers on the
paper build path (`execwrap`, `flatten-latex-main`) and generated-artifact
tools (`generate-all`, `check-generated`) — and will grow more as the
realization/compiler toolchain lands. If each command decides independently
what stdout and stderr mean, shell and build integration becomes brittle and
diagnostics can corrupt data streams.

## Decision

Adopt one repository-wide JSON-only command-line output contract for every
first-party executable. New executables are governed by this contract from
creation.

## Contract

### Streams

- Stdout and stderr are both **machine-readable** streams: JSON records or
  nothing. Plain human-readable text is not a valid output format on either
  stream.
- **Stdout is reserved for result data** intended for a caller to consume.
  Everything else — diagnostics, logs, progress, warnings, help, usage,
  status — goes to **stderr as JSON control-plane records**.
- A command with no stdout result data leaves stdout empty.
- **Assets are not stdout.** A command that produces files (rendered
  artifacts, flattened documents, generated manifests) writes them to paths
  supplied by arguments such as `--output`, never by dumping bytes to stdout.
  One deliberate exception: `execwrap` exists to relay a wrapped child's
  byte streams; its pass-through and `--redirect*` routing reproduce the
  child's bytes verbatim and are the command's result data, not diagnostics.

### JSON output

- When a command emits stdout result data, the default wire format is exactly
  **one JSON object followed by one trailing newline**.
- Streaming stdout result data uses **NDJSON**: one complete JSON object per
  line. A command using NDJSON documents that exception in its own contract.
- On success (exit 0) the result object is emitted on stdout; on failure
  stdout is empty and the exit code is the branch signal for callers.
- Stderr, when it emits multiple records over time, is NDJSON: one complete
  JSON object per line, written through a non-interleaving locked writer.

### TTY refusal

A command that emits stdout result data **refuses to run when stdout is
attached to a terminal** (exit 2, JSON diagnostic on stderr). The refusal is
unconditional — result data is for another process or file; pipe to `jq`,
`less`, or `cat` to inspect interactively. Commands without stdout result
data do not refuse a terminal.

### Exit codes

| Class   | Code | Meaning |
|---------|------|---------|
| success | 0    | Command succeeded. |
| failure | 1    | Runtime, startup, internal, or execution failure. |
| usage   | 2    | Usage failure: clap argv errors, TTY refusal, invalid arguments. |

Command-specific non-usage exit codes may be documented by the owning
command's contract; the baseline classes stay stable.

### Diagnostics

- Diagnostics use `tracing` with the JSON writer on stderr.
- Filter precedence: a non-empty `RUST_LOG` wins; otherwise the shared
  `--debug` flag selects debug level; otherwise the command's default level.
- No `println!`/`eprintln!` for progress, status, or errors. The workspace
  denies `clippy::print_stdout`, `clippy::print_stderr`, and `clippy::exit`;
  the only allowed exceptions live inside `cli-common`'s controlled emitters.
- Help/version requests write a JSON control record to stderr and exit 0;
  argv errors write one and exit 2. Raw clap text is always wrapped.
- Panics emit one JSON diagnostic record on stderr through the shared panic
  hook and exit 1; Rust's default text panic output is not used. Panic string
  payloads are included only under `--debug`, since payloads may contain
  secrets.

### Control-plane record schema

Shared stderr control-plane records carry: `schema` (number, currently 1),
`command` (string), `kind` (`help` | `version` | `usage_error` |
`tty_refusal` | `panic` | `status` | `diagnostic`), `message` (string), and
an optional kind-specific `fields` object. See `cli_common::ControlPlaneRecord`.

### Redaction

Before anything reaches JSON stderr: never log raw credential-bearing URLs,
private keys, tokens, session secrets, API keys, or passwords. The shared
helpers (`redact_field_value`, `redact_database_url`) replace secret-like
fields with `[redacted]` and strip userinfo and secret query parameters from
database URLs.

## Shared infrastructure

`packages/cli-common` is the single implementation of this contract: JSON
clap wrapping (`parse_args_or_exit`), control-plane emission
(`emit_control_plane_record`), the JSON panic hook, locked JSON stderr
tracing, TTY refusal, the stdout emitter (`emit`), the shared `--debug` flag
(`BaseArgs`), exit-class centralization (`CommandExit`, `exit_with`), and
runners for the two command classes (`run_stdout_json_command`,
`run_no_stdout_command`). Binaries return `ExitCode` from `main` — `Result`
termination would write raw `Error: ...` text to stderr.

## Command classification

| Command | Class | Notes |
|---|---|---|
| `execwrap` | no stdout result data | side-effect wrapper; child-byte relay is exempt as result data |
| `flatten-latex-main` | no stdout result data | writes the flattened `.tex` via `--output` |
| `generate-all` | no stdout result data | writes generated artifacts via `--output` |
| `check-generated` | stdout result data | one JSON report object; TTY refusal applies |

## Out of scope

Tests, benches, examples, build-script Cargo protocol output, and library
crates with no shipped binary.
