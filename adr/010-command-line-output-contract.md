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

Current first-party commands are public-data tools. They do not define
credential, private-key, signing-nonce, blinding-factor, or private-opening
inputs. The classification rules below prevent accidental diagnostic echo and
reserve a fail-closed shape for future code; they do not make the command
process a sandbox or secret-processing boundary. ADR-015 owns that trust
boundary.

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

On a runtime failure that occurs *before* result publication begins:

- stdout is empty;
- stderr carries JSON diagnostics;
- the process status selects the failure branch.

A failure *during* result publication — a partial write to stdout from a
broken pipe, short write, or output quota — may leave a partial record on
stdout. The result bytes and the process status are two independent effects
that cannot be committed atomically, the same physical limitation the
multi-output report/stamp design acknowledges (F1-020, F1-034). The command
still exits on the failure branch and still emits a JSON diagnostic on stderr
when stderr remains writable; it does not, and cannot, un-write bytes already
accepted by stdout. A consumer needing an all-or-nothing result reads an
explicit report file (`--report`) rather than stdout.

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

### `execwrap` child status relay

After successful wrapper startup, `execwrap` may return a child status in the
range 3-255. Codes 0-2 retain the workspace meanings:

- child 0 returns success 0;
- child 1 or 2 returns wrapper runtime failure 1;
- wrapper argument or TTY usage failure returns 2;
- child statuses 3-255 are relayed unchanged.

Signal-derived statuses such as `128 + signal` are child-status relay values,
not new workspace control-plane classes.

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

## Diagnostic data classification · `rule:output:data-classification`

Diagnostic safety is field-based. The command-line infrastructure does not
claim to discover every secret embedded in arbitrary free-form text.

Fields explicitly classified as secret-bearing are never emitted:

- credentials;
- private keys;
- tokens;
- API keys;
- passwords;
- session secrets;
- secret witness material;
- production blinding data;
- raw child argv.

Structured diagnostics prefer known-safe typed fields. Known URL fields use
the shared URL redactor.

The shared free-form redaction helpers are best-effort presentation tools,
not a completeness guarantee. No emission path may rely on them as a security
boundary; safety comes from omitting classified fields.

Raw child argv is a prohibited field class. It is not made safe by debug mode
or by heuristic redaction. Program identity may be logged only when it
originates from trusted typed configuration or an explicit safe identifier; a
caller-supplied executable path is raw child argv (it is `argv[0]`) and is
omitted like any other argument value. Argument count is known-safe metadata
and may be logged.

Likewise, the raw stderr of an argument-supplied external program is arbitrary
child output, not a typed field: it is omitted rather than relayed or
heuristically redacted, and only the process exit status is reported.

## Early startup · `rule:output:early-startup`

Early startup ends after arguments are parsed, the `--debug` state is
recorded, and JSON tracing is initialized.

Before that boundary, executables do not perform general secret sweeping.
Early records contain only static text and known-safe typed metadata, and
omit wholesale:

- raw argv and offending argument values;
- environment values;
- panic payloads;
- every field classified in (`rule:output:data-classification`).

Help and version text generated from the static command definition may be
emitted verbatim. A usage record reports the command, the clap error class,
and one fixed generic message. It does not reproduce argument values.

## Debug diagnostics · `rule:output:debug-disclosure`

Parsed `--debug` enables documented free-form diagnostic detail, including
string panic payloads.

Debug output may contain application-provided free-form text. The
infrastructure does not promise to detect secrets embedded in that text, so
operators must treat debug output as sensitive.

Fields classified as secret-bearing and raw child argv remain prohibited in
debug mode.

Debug mode may emit additional free-form application text and public input
detail. It does not change the classification of explicitly secret-bearing
fields, and it does not make out-of-contract secret inputs safe. Current
commands accept public data only (ADR-015).

### Child execution is an authority grant

Selecting an executable is granting execution authority. Argument-supplied
executables are trusted caller configuration. Omission of their argv and raw
stderr from first-party diagnostics does not sandbox or authenticate them
(ADR-015).

## Panics · `rule:output:panics`

Every executable installs the shared JSON panic hook before normal startup
work.

The hook:

- suppresses Rust's default text panic output;
- emits at most one best-effort JSON panic record;
- exits with code 1;
- omits the panic payload unless parsed `--debug` explicitly enabled it.

Payload reporting begins disabled. A panic before argument parsing therefore
fails closed under (`rule:output:early-startup`). After parsing, payload
disclosure follows (`rule:output:debug-disclosure`).

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
| `generate-label-registers` | side effect; explicit `--specification-register-output` and `--realization-register-output` |

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

### Heuristic secret sweeping as a security boundary

Rejected because a free-form scanner cannot recognize every secret shape.
Safety comes from omitting classified fields, not from rewriting text.

### Echoing offending argv in usage errors

Rejected because an unknown argument may embed an inline secret, and early
startup performs no sweeping.

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
- usage records carry no argument values;
- child argv never appears in wrapper diagnostics;
- stderr records do not interleave;
- subprocess tests cover each shipped executable;
- Clippy denies uncontrolled stdout/stderr printing and process exit outside
  the shared implementation.
