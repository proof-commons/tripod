# `tripod-artifacts`

Imported as `artifacts`. This package is the single source of truth for
the workspace's generated derivative artifacts in
`packages/model/generated/`. It owns both halves of the repository's
generation/check law (`plans` section 18.2, and the maintenance gate
M0.3): one typed source, one generator, one non-writing checker.

The generated files are **publications, never semantic inputs**. The
compiler, linker, and release logic consume typed Rust; nothing in the
workspace reads these files back as authority.

## The generation/check split

- `generate-all` (binary) **may write**. It renders every artifact from
  its typed source and publishes the whole set atomically under
  `--output`.
- `check-generated` (binary) and the library **never write**. They
  compute the expected bytes in memory and compare.

The owned census is `ARTIFACT_NAMES`:

- `architecture.json`
- `architecture.toml`
- `declassification.json`
- `model_labels.json`

The checker rejects both missing artifacts and unexpected files in the
generated directory, so directory ownership stays explicit rather than
being whatever happens to be present.

## Quickstart

Every entry point takes a `labels::RepositoryCensus` — the subject set,
assembled from command-line arguments by the build system (ADR-014).
Nothing here resolves a repository path from its own compiled location.

```rust,no_run
// Requires a real repository tree on disk: `check` reads the generated
// directory and `expected_artifacts` reads the census subjects.
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let census = labels::RepositoryCensus::default();

    // In-memory rendering: no filesystem destination is involved.
    for artifact in artifacts::expected_artifacts(&census)? {
        println!("{} is {} bytes", artifact.name, artifact.bytes.len());
    }

    // Non-writing freshness comparison.
    let report = artifacts::check(Path::new("packages/model/generated"), &census)?;
    if !report.current {
        for status in &report.artifacts {
            println!("{}: {:?}", status.name, status.status);
        }
        anyhow::bail!("generated artifacts are stale; run generate-all");
    }
    Ok(())
}
```

## Public API tour

### Root module

- `const ARTIFACT_NAMES: &[&str]` — the complete census, in the order
  the report uses.
- `const CHECK_REPORT_SCHEMA: u32` — schema version of the checker's
  report object (currently `1`).
- `struct ExpectedArtifact { name: &'static str, bytes: Vec<u8> }` — one
  rendered artifact: file name inside the generated directory, plus the
  exact expected contents.
- `expected_artifacts(census: &labels::RepositoryCensus) -> anyhow::Result<Vec<ExpectedArtifact>>`
  — render every artifact from its typed source, in memory. The
  architecture artifacts are validated on the way: the typed draft must
  validate, the publication envelope must verify, and both presentation
  encodings must round-trip to the same object. Writes nothing.
- `check(dir: &Path, census: &labels::RepositoryCensus) -> anyhow::Result<CheckReport>`
  — compare the committed directory against those bytes. Writes
  nothing.
- `enum ArtifactFreshness { Current, Stale, Missing }` — per-artifact
  verdict.
- `struct ArtifactStatus { name: String, status: ArtifactFreshness }`.
- `struct CheckReport { schema, generated_dir, artifacts, unexpected, current }`
  — `artifacts` is in census order, `unexpected` lists sorted files
  present in the directory but outside the census, and `current` is true
  only when every artifact is current *and* `unexpected` is empty.

### `weld` module

Structural extraction from the realization document, backing the
release-integrity weld tests. Extraction uses explicit unique markers,
never broad substring searches.

- `weld::masthead(document: &str) -> anyhow::Result<&str>` — every line
  before the first thematic break, where the document states its
  release identities. Errors when the document has no thematic break.
- `weld::extract_appendix_toml(document: &str) -> anyhow::Result<String>`
  — the verbatim `architecture.toml` bytes attached under the unique
  architecture appendix heading. Three structural rules are enforced:
  exactly one matching appendix heading, exactly one fenced `toml` block
  after it, and the fence must close. The returned string is the block's
  original bytes with one trailing newline.

The document claims its appendix is the generated manifest attached
verbatim; the weld tests in this crate compare that claim mechanically
against the typed architecture, so changing any attached byte or
masthead identity fails deterministically.

## Command-line contract

Both binaries follow the ADR-010 output contract
(`adr/010-command-line-output-contract.md`): JSON diagnostics on stderr,
exit classes 0/1/2, and no prose on stdout.

`generate-all` is a side-effect command and writes nothing to stdout:

```text
generate-all --repository-root DIR \
  --attestation-main FILE [--attestation-section FILE]... \
  --realization FILE [--model-source FILE]... \
  --output DIR
```

`check-generated` reports on stdout in direct mode, or to an explicit
report file plus success stamp in build mode:

```text
check-generated --repository-root DIR \
  --attestation-main FILE [--attestation-section FILE]... \
  --realization FILE [--model-source FILE]... \
  --generated-dir DIR [--report FILE --stamp FILE]
```

The direct-mode stdout object is the serialised `CheckReport`:

```text
{"schema":1,"generated_dir":"packages/model/generated","artifacts":[{"name":"architecture.json","status":"current"}],"unexpected":[],"current":true}
```

To regenerate after changing a typed source: run `generate-all` with the
same census arguments the build passes, then re-run `check-generated`.
Editing a file under the generated directory by hand is always wrong —
the checker will report it stale on the next run.

## Error handling

Both library entry points return `anyhow::Result`, with context naming
the stage that failed (deriving the published architecture, validating
the publication envelope, rendering or re-parsing an encoding, deriving
the model-label registry, reading a specific path).

Freshness is **not** an error. A stale or missing artifact, or an
unexpected file, is reported inside the returned `CheckReport`; `check`
returns `Err` only when the expected bytes cannot be derived at all or
the directory cannot be read. The binary maps a non-current report onto
the failure exit class, so the distinction is between "the check ran and
says no" and "the check could not run".

`generate-all` publishes through the shared batch publication path,
`cli_common::publish_batch`, so the whole rendered set is staged before
the first destination changes: a failure during derivation or staging
leaves every destination untouched, and the generated directory is never
left in mixed generations.

## What this package deliberately does not do

- **The library never writes.** There is no public write helper, no
  "fix" mode, and no in-place repair. Writing happens only in
  `generate-all`, through the shared publication path.
- **No implicit census.** Neither entry point discovers subjects by
  walking the repository or by locating itself on disk. If a subject is
  not in the supplied census, it does not participate.
- **No partial ownership of the directory.** The checker will not
  tolerate an unexpected file; the census is the whole contract for that
  directory.
- **No validation shortcut.** Generation cannot skip architecture
  validation: the validated wrapper is the only input the publication
  envelope accepts.
- **No consumption of its own output.** Generated artifacts are not read
  back as semantic authority anywhere in the workspace.

## Consumers

The meson build wires `check-generated` as the `generated-check` lint
lane (report plus success stamp) and `generate-all` as the regeneration
target. The `labels` package owns the label registries themselves and
derives the model-label JSON that this package publishes; the
`architecture` and `model` packages supply the typed sources. The weld
tests in this crate are the only consumer of the `weld` module.
