# `tripod-document-stamps`

Imported as `document_stamps`; ships the `attestation-stamps` binary.

This package derives the Attestation paper's identity metadata —
its date, its timestamp, and two UUIDs — from committed Git state, and
renders those values into the build inputs the TeX pipeline consumes.
It exists so the published paper carries deterministic, reproducible
identities instead of wall-clock values captured at build time.

## The four values

| Value           | Domain                                                            |
|-----------------|-------------------------------------------------------------------|
| `date`          | UTC calendar date of the latest commit touching any member of the **exact input set** |
| `timestamp`     | UTC timestamp of the latest commit touching **anything under the paper subtree** (this is the source date epoch) |
| `document_uuid` | First 128 bits of a canonical digest over the exact input set      |
| `instance_uuid` | First 128 bits of the Git tree object ID for the paper subtree, formatted with UUID grouping |

The two domains are deliberately different. `date` answers "when did the
published content last change", so it follows only the declared inputs.
`timestamp` answers "how old is this build's source tree", so it follows
the whole subtree and is the value reproducible builds need.

`document_uuid` is content identity: a digest over each input's
repository-relative path, its Git file mode, and its committed bytes,
each length-framed under a versioned domain separator, over the inputs
sorted by path. `instance_uuid` is tree identity: the subtree's own Git
object, so two checkouts of the same commit agree.

All four derive from **committed** objects, so the paper subtree must be
clean. A dirty or untracked file under the subtree is a hard failure,
not a warning, and each input's working-tree bytes are additionally
compared against its committed blob.

## Quickstart

Every input is supplied explicitly; nothing is read from the ambient
environment.

```rust,no_run
// Requires a clean Git checkout and a `git` executable on disk, so this
// compiles but is not run as a doctest.
use std::path::{Path, PathBuf};

use document_stamps::{RenderRequest, StampRequest};

fn main() -> Result<(), document_stamps::StampError> {
    let request = StampRequest {
        git: PathBuf::from("/usr/bin/git"),
        repository_root: PathBuf::from("."),
        tree_ref: "HEAD".to_owned(),
        tree: PathBuf::from("papers/attestation"),
        inputs: vec![
            PathBuf::from("papers/attestation/main.tex"),
            PathBuf::from("papers/attestation/sections/01_intro.tex"),
        ],
    };

    // Derivation only: reads Git, writes nothing.
    let values = document_stamps::run(&request)?;
    println!("date {} epoch {}", values.date, values.timestamp.epoch);
    println!("document {}", values.document_uuid);
    println!("instance {}", values.instance_uuid);

    // Derivation plus publication of the two build inputs.
    document_stamps::render(&RenderRequest {
        stamps: &request,
        template: Path::new("papers/attestation/stamps.tex.in"),
        stamps_output: Path::new("build/stamps.tex"),
        epoch_output: Path::new("build/source-date-epoch"),
    })
}
```

## Public API tour

The crate is a single flat module.

### Requests

- `struct StampRequest { git: PathBuf, repository_root: PathBuf,
  tree_ref: String, tree: PathBuf, inputs: Vec<PathBuf> }`
  — `git` is the executable to invoke; `repository_root` must be the Git
  top level and every other path is relative to it; `tree` is the paper
  subtree; `inputs` is the exact publication-input set. `tree_ref` is
  verified against checked-out `HEAD`: a branch or tag pointing at
  exactly `HEAD` is accepted, because the rule is commit-object
  equality, not string equality. Any other revision is rejected, since
  derivation renders the working tree and mixing in another commit's
  metadata would produce hybrid output.
- `struct RenderRequest<'a> { stamps: &'a StampRequest, template: &'a Path,
  stamps_output: &'a Path, epoch_output: &'a Path }`.

### Entry points

- `run(request: &StampRequest) -> Result<AttestationStampValues, StampError>`
  — derive the four values. Reads Git; writes nothing.
- `render(request: &RenderRequest<'_>) -> Result<(), StampError>`
  — derive, substitute the four placeholders into the template, and
  publish the rendered TeX file and the epoch file.

### Values

- `struct AttestationStampValues { date: String, timestamp: PreparedTimestamp,
  document_uuid: String, instance_uuid: String }` — serialisable as the
  binary's single JSON result object. Both UUID fields are lowercase hex
  grouped `8-4-4-4-12`; no version or variant bits are rewritten, so the
  text preserves all 128 derived bits.
- `struct PreparedTimestamp { epoch: i64, iso_8601: String, pdf: String }`
  — one instant in three deterministic forms: Unix seconds for the
  source date epoch, a `Z`-suffixed ISO form for diagnostics and tests,
  and the PDF date form for the document's own date fields.

### Errors

`enum StampError` — `AliasedOutputs`, `RepositoryRootMismatch`,
`UnsupportedGitObjectFormat`, `GitCommandFailed`, `InvalidGitOutput`,
`SelectedRevisionIsNotHead`, `DirtyPaperSubtree`, `InvalidInputPath`,
`InputOutsidePaperTree`, `DuplicateInput`, `InvalidTrackedInput`,
`InputReadFailed`, `MissingPathHistory`, `InvalidTreeObjectId`,
`TreePrefixResolutionFailure`, `TemplateReadFailed`,
`UnresolvedPlaceholder`, `OutputWriteFailed`.

## Command-line contract

`attestation-stamps` has two modes, selected by whether the three render
arguments are present. They are all-or-nothing.

**JSON mode** — no `--template`. One JSON object on stdout, refused when
stdout is a terminal:

```text
attestation-stamps --git PROGRAM --repository-root DIR \
  [--tree-ref REV] --tree DIR --input FILE [--input FILE]...
```

```text
{"date":"2026-08-16","timestamp":{"epoch":1755300000,"iso_8601":"2026-08-16T00:40:00Z","pdf":"D:20260816004000Z"},"document_uuid":"...","instance_uuid":"..."}
```

**Render mode** — writes the two build inputs and produces no stdout:

```text
attestation-stamps --git PROGRAM --repository-root DIR \
  [--tree-ref REV] --tree DIR --input FILE... \
  --template FILE --stamps-output FILE --epoch-output FILE
```

The template is filled by substituting four `@ATTESTATION_...@`
placeholders (date, timestamp, document UUID, instance UUID). If any
placeholder token survives substitution, rendering fails rather than
publishing a half-filled file.

## Error handling

Every fallible entry point returns `StampError`, which names a failure
*class* and never echoes a path, an argument, or Git output — those may
carry values ADR-010 keeps out of diagnostics. The underlying details
are logged through `tracing` where they are safe.

Publication is staged before it is performed. Both outputs are written
and fsync'd to sibling temporaries first, so any read, create, write, or
fsync failure happens before either final path changes. Only then are
the two renames performed, epoch first and the more visible TeX file
last. Two independent paths cannot be renamed as one transaction: if the
second rename fails, the build fails and the next successful invocation
repairs the pair. This is a recovery preference, not a proof of
atomicity, and it is stated rather than implied.

Both outputs are compare-if-changed, so an unchanged rebuild leaves
their bytes and modification times alone and does not cascade
downstream.

Cleanliness is checked before and again after every input is read. That
narrows, but does not close, the window in which the working tree
changes mid-derivation; a canonical release still needs a quiescent
checkout and the final clean-tree gate.

## What this package deliberately does not do

- **No ambient inputs.** It does not search for `git`, infer the
  repository root, read environment variables, or consult a timezone
  database, locale, or `date` binary. UTC calendar arithmetic is pure
  integer maths.
- **No wall-clock values.** Every value derives from committed objects.
  Build time never enters the output.
- **No dirty-tree tolerance.** There is no force flag and no "best
  effort" derivation from uncommitted content.
- **No arbitrary revision.** Only a revision resolving to checked-out
  `HEAD` is accepted, because the rendered bytes come from the working
  tree.
- **No implicit input set.** Inputs are listed exactly; each must be a
  tracked regular file strictly beneath the declared subtree.
  Directories, symlinks, submodule links, duplicates, absolute paths,
  and paths containing `.` or `..` are all rejected. Git pathspecs are
  built as literals so a filename containing pathspec metacharacters
  cannot widen the selection.
- **No path echoing in errors.** Failure classes are typed; offending
  values stay out of the message.
- **No transactional two-file publication.** See the limit above.
- **No non-SHA-1 repositories.** The 128-bit tree-prefix rule is not
  defined for another object format, so one is refused rather than
  approximated.

## Consumers

The meson build under `papers/attestation/` runs `attestation-stamps` in
render mode to produce the generated `stamps.tex` and the
`source-date-epoch` file consumed by the reproducible TeX build. The
JSON mode exists for inspection and for tests. The package depends on
`cli-common` for the ADR-010 output contract and for output-role
uniqueness checking.
