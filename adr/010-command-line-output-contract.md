# ADR-010: Command-Line Output Contract

**Status:** Decided and implemented
**Scope:** Every first-party executable shipped by this workspace
**Implementation:** `packages/cli-common`
**Excludes:** tests, examples, benches, build-script protocol output, and
library crates without executables

---

## Context · `sec:output:context`

Workspace commands are consumed by shell scripts, Meson, CI, and later
toolchain packages. Plain-text diagnostics mixed with result data make those
interfaces ambiguous and can corrupt pipelines.

The workspace therefore assigns one machine-readable meaning to each standard
stream and one shared set of process exit classes.

## Streams · `rule:output:streams`

Standard output carries result data only.

Standard error carries control-plane records:

- diagnostics;
- logs;
- warnings;
- progress;
- help;
- version;
- usage errors;
- panic reports;
- TTY refusal.

Both streams contain JSON records or nothing.

A command with no stdout result leaves stdout empty.

### Child-byte relay

`execwrap` may reproduce a child process's raw stdout or stderr bytes. Those
bytes are the wrapper's result data, not wrapper diagnostics.

Wrapper diagnostics still use JSON on stderr.

## Result encoding · `rule:output:json`

A non-streaming result is:

```text
one JSON object
one trailing newline
```

A streaming result uses NDJSON:

```text
one complete JSON object per line
```

A command using streaming output must document that choice.

On runtime failure:

- stdout is empty;
- stderr carries JSON diagnostics;
- the process status selects the failure branch.

JSON presentation is deterministic where result identity requires it.

## Assets · `rule:output:assets`

Files, documents, bundles, reports, manifests, and other assets are written
only to paths supplied by arguments such as:

```text
--output
--output-dir
```

Assets are never dumped to stdout as an undocumented byte stream.

A check command does not repair or regenerate tracked assets.

Generation and checking remain separate operations.

The `execwrap` child-byte relay is the deliberate exception described in
(`rule:output:streams`).

## TTY refusal · `rule:output:tty`

A command producing stdout result data refuses to run when stdout is attached
to a terminal.

It returns:

```text
exit 2
JSON tty_refusal record on stderr
empty stdout
```

The caller may pipe to:

```text
jq
cat
less
a file
another process
```

Commands with no stdout result data do not refuse terminal stdout.

## Exit classes · `rule:output:exit-codes`

| Class | Code | Meaning |
|---|---:|---|
| success | 0 | Command completed successfully. |
| failure | 1 | Runtime, startup, execution, internal, or validation failure. |
| usage | 2 | Invalid arguments, clap usage failure, or TTY refusal. |

A command-specific status may be added only when its command contract
documents it. Codes 0-2 retain the meanings above.

Binaries return `ExitCode` from `main`. They do not use `Result` termination,
which could write Rust's plain-text `Error: ...` format.

## Control-plane records · `rule:output:control-plane`

Shared early control records contain:

```text
schema
command
kind
message
optional structured fields
```

Shared kinds include:

```text
help
version
usage_error
tty_refusal
panic
status
diagnostic
```

Multiple stderr records use NDJSON and a non-interleaving locked writer.

Help and version:

```text
exit 0
JSON on stderr
empty stdout
```

Usage failure:

```text
exit 2
JSON on stderr
empty stdout
```

## Diagnostics · `rule:output:diagnostics`

Runtime diagnostics use JSON `tracing` output on stderr.

Filter precedence is:

1. nonempty `RUST_LOG`;
2. parsed `--debug`;
3. command default.

Plain `println!` and `eprintln!` are prohibited for first-party command
diagnostics. Controlled emission remains centralized in `cli-common`.

A diagnostic must not be treated as result data merely because it is useful to
a human reader.

## Redaction · `rule:output:redaction`

Commands must not emit raw:

- credentials;
- private keys;
- tokens;
- API keys;
- passwords;
- session secrets;
- credential-bearing URLs;
- secret witness material;
- production blinding data.

Structured diagnostics should prefer known-safe typed fields.

Untrusted free-form text must pass through the shared redaction helpers before
emission.

Child argv is not logged. Program identity and argument count may be logged
when safe.

Redaction is a call-site and shared-infrastructure obligation. A JSON envelope
does not make unsafe text safe.

## Panics · `rule:output:panics`

Every executable installs the shared JSON panic hook before normal startup
work.

The hook:

- suppresses Rust's default text panic output;
- emits at most one best-effort JSON panic record;
- exits with code 1;
- omits the panic payload unless parsed `--debug` explicitly enabled it.

Payload reporting begins disabled. A panic before argument parsing therefore
fails closed.

Panic payloads remain subject to secret-handling rules even in debug mode.

## Shared implementation · `rule:output:implementation`

`cli-common` owns:

- clap help/version/usage wrapping;
- control-plane records;
- JSON stderr writing;
- tracing initialization;
- panic handling;
- stdout TTY refusal;
- stdout JSON emission;
- exit classes;
- shared `--debug`;
- redaction helpers;
- runners for stdout-result and side-effect commands.

New executables use this package rather than reimplementing the contract.

Existing command classifications are:

| Command | Class |
|---|---|
| `execwrap` | side effect plus deliberate child-byte relay |
| `flatten-latex-main` | side effect; writes `--output` |
| `generate-all` | side effect; writes `--output` |
| `check-generated` | one JSON stdout result |
| `check-labels` | one JSON stdout result |
| `generate-label-registers` | side effect; explicit `--output-root` |

## Rejected alternatives · `sec:output:alternatives`

### Human text on stderr

Rejected because machine callers cannot reliably distinguish diagnostics,
usage, progress, and panic output.

### Result data mixed with logs on stdout

Rejected because diagnostics can corrupt a data pipeline.

### Pretty-print automatically on a terminal

Rejected because result commands have one machine contract. Interactive users
must opt into a consumer such as `jq`.

### Panic payloads enabled before parsing

Rejected because payloads may contain secrets.

### Per-command exit conventions

Rejected because shared automation needs stable branch classes.

## Verification · `gate:output:verification`

The contract is implemented when:

- help and version use JSON stderr and exit 0;
- invalid argv uses JSON stderr and exit 2;
- result commands refuse terminal stdout;
- successful single results are one JSON line;
- failure leaves result stdout empty;
- side-effect commands write only explicit assets;
- panic output is JSON-only;
- panic payloads default to hidden;
- child argv and credential URLs do not leak;
- stderr records do not interleave;
- subprocess tests cover each shipped executable;
- Clippy denies uncontrolled stdout/stderr printing and process exit outside
  the shared implementation.
