# Tracing design for `tripod-architecture`

## Purpose and shape · `sec:tracing-architecture:purpose`

`architecture` is the typed, static normative manifest and identifier vocabulary at the root of the first-party dependency graph; it has no binary, no first-party dependencies, and no file, environment, network, or standard-stream I/O.

`src/ids.rs` defines finite IDs, `src/spec.rs` declares the manifest and lookup helpers, `src/validate.rs` checks drafts and releases, `src/canonical.rs` derives validated identities, `src/export.rs` builds and checks publication envelopes and renders artifact strings, and `src/deployment.rs` validates separately bound deployment evidence and derives its dormant profile identity; `src/lib.rs` re-exports the public surface and `src/tests/` holds conformance and mutation tests.

The main callers are `realization` and `model` for validated IDs and semantic binding, `compiler`, `linker`, and `vectors` for architecture-owned IDs, and `artifacts` for validated publication rendering; errors cross these API boundaries as typed `Vec<ManifestError>`, `Vec<DeploymentError>`, `EnvelopeError`, or serialization errors, leaving presentation to the caller.

## Output inventory · `sec:tracing-architecture:inventory`

This inventory records the design-time baseline before architecture instrumentation; its source line numbers and tracing-anchor entry describe that baseline, while the stream-write and bespoke timing counts remain zero.

| Location | Baseline operation | Class | Stream effect |
|---|---|---|---|
| All `packages/architecture/src/*.rs` and `src/tests/*.rs` | No `println!`, `eprintln!`, `print!`, `eprint!`, `stdout()`, `stderr()`, or direct stream write | No structured result, diagnostic, progress, timing, or counter emission | None; baseline count is zero in every class. |
| `src/validate.rs:69-96` | `write!` in `Display for ManifestError` | Diagnostic formatting | Writes to the supplied `fmt::Formatter`, never directly to stderr. |
| `src/deployment.rs:253-316` | `write!` in `Display for DeploymentError` | Diagnostic formatting | Writes to the supplied `fmt::Formatter`, never directly to stderr. |
| `src/export.rs:318-336` | `write!` in `Display for EnvelopeError` | Diagnostic formatting | Writes to the supplied `fmt::Formatter`, never directly to stderr. |
| `src/lib.rs:19-20` | `use tracing as _` dependency anchor; no span or event call | No baseline emission | None; the first instrumentation commit removed the anchor. |
| All package source and tests | No bespoke counter, progress reporter, `Instant`, `elapsed`, or timing accumulator | No current counter, progress, or timing emission | None. |

`PublishedArchitecture::to_artifact_json` and `to_artifact_toml` in `src/export.rs:130-144` return `String` values; `canonical_json_bytes` in `src/canonical.rs:284-288` returns bytes. These are structured data values for explicit asset publication, not stream writes.

## Standard output and standard error · `sec:tracing-architecture:streams`

The library emits no stdout result and owns no subscriber. Its validated wrappers, defect vectors, hashes, envelope values, and render strings remain typed return values; `artifacts` writes the publication JSON and TOML to explicit asset paths. A command choosing stdout uses `cli-common` for one JSON object plus newline or documented NDJSON, terminal refusal, empty stdout before failed publication, and the existing partial-write rule (`[ADR010-rule:output:streams]`, `[ADR010-rule:output:json]`, `[ADR010-rule:output:assets]`). A TOML render string never becomes an undocumented stdout format.

No current architecture code writes stderr, so there is no direct writer to migrate. Each selected public operation below emits one structured completion event inside its span; its typed error remains the caller's source of detail, and only the executable's `cli-common` boundary emits a user-facing ERROR when that command fails. The library never installs a subscriber, sets a global default, or installs a panic hook.

An executable's first action is `cli_common::install_command_subscriber(command_name)`, which installs one JSON subscriber and the panic hook before argument parsing. Its always-enabled control route emits help, version, usage, TTY-refusal, and panic records with ADR-010's wire schema through a shared locked writer; `RUST_LOG=off` cannot suppress them. Its filtered diagnostic route writes library events and enabled span close records with the usual tracing JSON envelope, and runners reload the filter after parsing under the nonempty `RUST_LOG`, then `--debug`, then command-default precedence; an invalid directive falls back to INFO with a warning that omits its text. The shared INFO command span starts after parsing and parents architecture spans when called; help and usage get only control records. Enabled span closes carry `busy_ns: u64` and `idle_ns: u64`; their inclusive durations and parentage supply timing, without a local stopwatch or a duration event.

ADR-010 requires every first-party stderr byte to come from the shared subscriber, including early control records; it retains their control schema, omits raw argv and panic payloads before parsing, uses a nonblocking panic path, and keeps the shared exit classes (`[ADR010-rule:output:early-startup]`, `[ADR010-rule:output:control-plane]`, `[ADR010-rule:output:panics]`). `execwrap` routes child stdout and stderr only to explicit files, with no raw child-byte relay through the wrapper's standard streams (`[ADR010-rule:output:streams]`). Consuming binaries own their startup and stream assertions; architecture returns typed data and leaves subscriber-owned presentation at the executable boundary, preserving ADR-026's separation of returned domain data from span durations and event counts (`[ADR026-dec:tracing:core]`).

## Spans · `sec:tracing-architecture:spans`

| Public operation and boundary | Span name | Explicit target | Level | Entry fields (type) | Duration value |
|---|---|---|---|---|---|
| `validate_draft` | `validate_draft` | `architecture::validate` | DEBUG | `schema_version: u32`, `operation_count: u64` | Measures the complete multi-pass manifest scan. |
| `validate_architecture_release` | `validate_architecture_release` | `architecture::validate` | DEBUG | `schema_version: u32` | Measures draft consistency plus the anchor-set pin and final-status check. |
| `validate_deployment_profile_structure` and `validate_deployment_profile` | `validate_deployment_profile_structure` and `validate_deployment_profile` | `architecture::deployment` | DEBUG | `profile_schema_version: u32`, `calibration_count: u64`, `dependency_count: u64` | Measures evidence scans and architecture binding, including nested identity work. |
| `validate_production_deployment_release` | `validate_production_deployment_release` | `architecture::deployment` | DEBUG | `profile_schema_version: u32` | Measures structural checks preceding the schema-2 production refusal. |
| `semantic_hash`, `semantic_hash_hex`, `behavioural_hash`, `behavioural_hash_hex` | `semantic_hash`, `semantic_hash_hex`, `behavioural_hash`, `behavioural_hash_hex` respectively | `architecture::canonical` | DEBUG | `schema_version: u32` | Measures canonical projection, serialization, and SHA-256, with hex encoding included by the hex entry point. |
| `deployment_profile_hash` and `deployment_profile_hash_hex` | `deployment_profile_hash` and `deployment_profile_hash_hex` respectively | `architecture::deployment` | DEBUG | `profile_schema_version: u32` | Measures validated profile projection and hashing. |
| `PublishedArchitecture::from_architecture` | `build_envelope` | `architecture::export` | DEBUG | `schema_version: u32` | Measures in-memory envelope construction and both hashes; durable publication belongs to `artifacts` and `cli-common`. |
| `PublishedArchitecture::verify_hashes`, `validate_envelope`, `validate_release_envelope`, `validate_against_expected` | `verify_hashes`, `validate_envelope`, `validate_release_envelope`, `validate_against_expected` respectively | `architecture::export` | DEBUG | `schema_version: u32` | Measures hash verification, metadata checking, and expected-publication comparison. |
| `PublishedArchitecture::to_artifact_json` and `to_artifact_toml` | `render_artifact` | `architecture::export` | DEBUG | `schema_version: u32`, `format: &'static str` (`json` or `toml`) | Measures presentation serialization without changing artifact bytes. |

These validators and hashers may be called repeatedly by other packages, so all architecture spans are DEBUG. The public entry point calls an uninstrumented internal core when another public method in this crate would otherwise produce a second completion event: release validation calls the draft core; deployment structure validation calls the architecture release and identity cores; the profile wrapper calls the structure core; hash hex wrappers call the hash core; and envelope validation calls verification cores. Rendering has no nested public validation call. A direct call to any listed public entry point still has its own one span and one event. Constant-time ID accessors, `Display`, and small record helpers remain uninstrumented.

## Events · `sec:tracing-architecture:events`

| Public boundary | Explicit target | Level | Fixed message | Fields (type) |
|---|---|---|---|---|
| `validate_draft` | `architecture::validate` | DEBUG | `draft validation completed` | `outcome: &'static str`, `defect_count: u64`; on refusal `refusal_kind: &'static str` |
| `validate_architecture_release` | `architecture::validate` | DEBUG | `release validation completed` | `outcome: &'static str`, `defect_count: u64`; on refusal `refusal_kind: &'static str` |
| Structural `validate_deployment_profile_structure` or wrapper `validate_deployment_profile` | `architecture::deployment` | DEBUG | `deployment structure validation completed` | `outcome: &'static str`, `defect_count: u64`; on refusal `refusal_kind: &'static str` |
| `validate_production_deployment_release` | `architecture::deployment` | DEBUG | `production release validation completed` | `outcome: &'static str` (`refused` under schema 2), `refusal_kind: &'static str`, `defect_count: u64` |
| Manifest or profile hash and hex entry points | `architecture::canonical` or `architecture::deployment` | DEBUG | `identity computation completed` | `identity_kind: &'static str` (`semantic`, `behavioural`, or `profile`), `outcome: &'static str`; on serialization failure `error_kind: &'static str` (`serialization`) |
| `PublishedArchitecture::from_architecture` | `architecture::export` | DEBUG | `envelope construction completed` | `outcome: &'static str`; on serialization failure `error_kind: &'static str` (`serialization`) |
| `PublishedArchitecture` envelope verification and validation entry points | `architecture::export` | DEBUG | `envelope validation completed` | `outcome: &'static str`, `defect_count: u64` (`0` or `1`); on typed refusal `refusal_kind: &'static str`; on serialization failure `error_kind: &'static str` (`serialization`) |
| `PublishedArchitecture::to_artifact_json` or `to_artifact_toml` | `architecture::export` | DEBUG | `artifact rendering completed` | `outcome: &'static str`; on success `rendered_bytes: u64`; on serialization failure `error_kind: &'static str` (`serialization`) |

Every listed operation emits exactly one completion event before its span closes. `outcome` is exactly `completed` for a returned typed result, `refused` for an expected typed rejection, or `failed` for an operational error; it never encodes whether another package later admits a result. `refusal_kind` is present exactly on a refusal, using the first returned defect's stable payload-free variant name, and `defect_count` is the length of that returned vector; for a single `EnvelopeError`, the count is one. `error_kind` is present exactly on a failure. Counts become `u64` with saturating conversion at emission, without changing domain types or behavior.

`validate_production_deployment_release` always returns a refusal under profile schema 2; if structural defects precede `ProductionReleaseUnsupported`, the event names the first structural defect, and otherwise it names `production_release_unsupported`. That is an expected DEBUG result, not a deployability claim. Serialization errors are `failed` at DEBUG with `error_kind = "serialization"`; the command boundary, if any, owns the one terminal ERROR. No per-defect event floods stderr, and a nested same-crate core emits no duplicate completion event.

## Naming · `sec:tracing-architecture:naming`

Targets are explicit literal Rust crate-stem module names: `architecture::validate`, `architecture::canonical`, `architecture::export`, and `architecture::deployment`; no binary role or Cargo package prefix enters them. Span names are lower_snake_case operations without a package prefix or dots, and messages are fixed lowercase English phrases. Span fields are entry context only: `schema_version: u32`, `profile_schema_version: u32`, existing collection counts, and `format` for rendering; completed counts and outcomes belong to events.

`outcome` has only `completed`, `refused`, and `failed`; `identity_kind` has only `semantic`, `behavioural`, and `profile`; `format` has only `json` and `toml`. `refusal_kind` and `error_kind` are stable payload-free variant or failure names; `ManifestError`, `DeploymentError`, and `EnvelopeError` each provide a static `name()` mapping that never formats a payload. `operation_code: u16` means the architecture-owned `OperationId::code()` wherever a caller needs one, but this package does not add it to operations without one specific ID. Event count keys end in `_count: u64`, byte quantities in `_bytes: u64`, and `phase` is reserved for a fixed step when the span name cannot already express it; no architecture event here needs `phase`.

No event field contains a manifest body, profile evidence, hash, amount, nonce, script, witness, key, command result, caller-supplied path, raw child output, or free-form error text. The library does not repeat `command`, `output_mode`, `exit_class`, `result_published`, `schema`, or control-record `kind`; those belong to `cli-common` and its command span or control route (`[ADR010-rule:output:data-classification]`).

## Cost · `sec:tracing-architecture:cost`

Create one span per listed public operation, not per manifest member or canonical JSON node. Entry fields read existing slice lengths and schema numbers; a completion event reads its existing result and error vector. Do not derive a hash, traverse or clone a manifest, compute a defect histogram, allocate a string, format an error, or sample a clock solely for telemetry. A future derived field must be guarded by `tracing::enabled!` or lazily formatted; with no subscriber the added cost is the callsite check (`[ADR026-dec:tracing:core]`).

## Testing · `sec:tracing-architecture:testing`

Use a small package-local in-memory `tracing-subscriber` capture under `tracing::subscriber::with_default`, never a global subscriber or new shared test-helper crate. For each validator with a completing branch, pair a completed control with one subject one coordinate away that returns a typed refusal at that validator's boundary; assert the target, DEBUG level, fixed message, `outcome`, first `refusal_kind`, and `defect_count`, with no WARN or ERROR from the refused call. Production-release validation has no completing branch under schema 2: a structurally valid profile is the control with only `ProductionReleaseUnsupported`, and a one-coordinate mutation adds its structural defect ahead of that policy refusal. A direct draft call and a nested release call must each yield exactly one completion event for their external invocation; a deployment wrapper must likewise avoid duplicate structure or identity events.

Every capture test re-executes itself as the only selected test in a fresh child process with its exact test path and a child marker; the parent requires a successful exit and exactly one passed test, printing child output on failure. Inside the child, the scoped subscriber retains a process-wide install lock and rebuilds callsite interest before the subject runs, so concurrent uncaptured tests in the parent cannot change the capture verdict. A regression first calls a traced operation without capture and then captures it from two threads, requiring one span per call.

Assert identical returned defects, hashes, canonical bytes, and JSON/TOML artifact bytes with and without capture; test span presence and parentage rather than elapsed thresholds. The enabled production subscriber's numeric close records, early control records under `RUST_LOG=off`, panic behavior, and stdout bytes belong in subprocess tests of the consuming binaries. `artifacts` owns the `generate-all` and `check-generated` stream and asset assertions; this pure library has no stdout or stderr handle to capture.

## Implementation plan · `sec:tracing-architecture:plan`

1. `feat(architecture): trace manifest validation` added DEBUG draft and release spans, one event per external call, the uninstrumented draft core, a static `ManifestError::name()` mapping, and scoped control/refusal tests; it removed the `use tracing as _;` anchor.
2. `feat(architecture): trace deployment validation` added structure, wrapper, and production spans and completion events, a static `DeploymentError::name()` mapping, and tests for a valid structural profile, one-coordinate refusals, and the schema-2 policy refusal; it reused the uninstrumented structure and identity cores and depended on step 1's event vocabulary.
3. `feat(architecture): trace canonical identities` added DEBUG manifest and profile identity spans/events and core-versus-wrapper separation; scoped tests assert identical canonical bytes and digests with and without capture.
4. `feat(architecture): trace publication envelopes` added `build_envelope`, envelope validation, and `render_artifact` spans/events with fixed `format`, a static `EnvelopeError::name()` mapping, and scoped tests for typed outcomes and byte-identical JSON/TOML; it depended on step 3's identity cores.
5. `artifacts` owns binary startup and tests actual JSON stdout, JSON-only stderr, `RUST_LOG=off` control records, and explicit assets against the implemented `cli-common` subscriber and amended ADR-010; this is an `artifacts` commit, not an architecture change. `cli-common` retains compatibility delegates while binaries migrate and removes them when that migration is complete.

## Open questions for the other packages · `sec:tracing-architecture:questions`

- Resolved target and names: literal `architecture::<module>` targets and lower_snake_case span names are fixed; `phase` is used only when the span name does not already identify the step, and `outcome` is `completed`, `refused`, or `failed`.
- Resolved diagnostic ownership: one library completion event reports a typed outcome, and `cli-common` alone emits the command's terminal ERROR; a nested same-crate core does not emit another completion event.
- Resolved timing: the subscriber emits enabled span close records with inclusive `busy_ns` and `idle_ns`; a collector derives exclusive time from parentage and never sums nested spans as separate work.
- Resolved startup: under amended ADR-010, `cli-common` installs the subscriber and panic hook before parsing and routes required control records through an unfiltered layer; architecture installs neither.
