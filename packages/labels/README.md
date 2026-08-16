# `tripod-labels`

`labels` is the repository's documentation-conformance package. It harvests
typed labels, validates imported citations and architecture/document welds, and
renders deterministic label publications. It is documentation tooling only; no
semantic package consumes its registries or planning labels.

This README is the crate documentation: it is included verbatim as the
rendered landing page, and every example below runs as a doctest. It is meant
to be enough to use the public API correctly on its own.

The library crate is named `labels`; the Cargo package is
`tripod-labels`.

## Trust position

A label is a documentation cross-reference and nothing else. Plan-local labels
remain non-normative and non-identity-bearing, but their mints, citations, and
owner crossings are mechanically checked under ADR-013. Nothing this crate
produces is a protocol input, a compiler input, or an identity anything depends
on.

## Inputs

- `architecture::ARCHITECTURE`;
- Attestation LaTeX and realization Markdown;
- ADR and planning Markdown;
- model Rust source; and
- committed label registers and model-label publication.

Every subject file arrives by role-tagged command-line argument
(ADR-014): the build system states census membership from its
hand-managed per-directory lists, and the binaries re-verify that
census against the on-disk tree before trusting it. Nothing in this
package resolves a repository path from its own compiled location.

## Outputs

The library returns owner-aware registries and diagnostics. It also renders the
specification register, realization register, and the derived model-label JSON.

`check-labels` is non-writing and emits one JSON report on stdout. It validates
source labels, imported citations, the architecture-to-document weld, the
pinned attestation anchor set, and generated-publication freshness.

`check-plans` is non-writing and emits one JSON report on stdout. It checks
plan-tree structure and hygiene over `adr/` and `plans/`: census agreement,
README ownership indexing, headings, links, drafting-scaffold and placeholder
markers, phase-gate consistency, and the Markdown weight budget.

`generate-label-registers` writes only:

- `plans/labels/specification.md`; and
- `plans/labels/realization.md`.

The complete source-reference relation is stored and validated as a direct
Petgraph graph. Stable owner/label keys are semantic identities; Petgraph
indices are local handles only. The `artifacts` package remains responsible for
writing `packages/model/generated/model_labels.json`.

## The label vocabulary

A label is an owner-local token whose segments are lowercase letters, digits,
and hyphens, and whose arity is fixed by its owner's `LabelShape`.

| `LabelShape` | Arity |
|---|---|
| `Attestation` | 2 or 3 segments |
| `Planning`, `Adr`, `Model` | exactly 3 segments |
| `Realization` | 2 segments with head in `sec`, `app`; or 3 with head in `subsec`, `req`, `def`, `inv`, `lem`, `obl`, `trap`, `rem`, `ins`, `rule`, `pin`, `res`, `lst`, `fig`, `tbl`, `leaf` |

A citation that crosses owners carries the target owner's prefix.
`LabelOwner` has seven variants, each with a prefix and exactly one shape:

| `LabelOwner` | Prefix | Shape |
|---|---|---|
| `Attestation` | `A-` | `Attestation` |
| `Realization` | `RZ-` | `Realization` |
| `Adr(u16)` | `ADR012-` (zero-padded to three digits) | `Adr` |
| `Model` | `MODEL-` | `Model` |
| `Plan` | `PLAN-` | `Planning` |
| `Doc` | `DOC-` | `Planning` |
| `Crate(String)` | the crate name plus a hyphen | `Planning` |

## Quickstart: parse a label and an imported citation

Both parsers are pure and touch no filesystem.

```rust
use labels::owner::{ImportedLabel, LabelOwner};
use labels::{Label, LabelShape};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // An owner-local label is parsed against its owner's arity grammar.
    let local = Label::parse("def:model:classes", LabelShape::Attestation)?;
    assert_eq!(local.as_str(), "def:model:classes");

    // A citation naming another owner carries that owner's prefix.
    let imported = ImportedLabel::parse("ADR012-rule:labels:decision")?;
    assert_eq!(imported.owner, LabelOwner::Adr(12));
    assert_eq!(imported.label.as_str(), "rule:labels:decision");
    assert_eq!(imported.owner.prefix(), "ADR012-");
    Ok(())
}
```

## Quickstart: mint into a registry and render a register

The mint-to-render pipeline is pure as well, so a consumer can drive it without
a repository on disk.

```rust
use labels::owner::LabelOwner;
use labels::registry::{LabelMint, LabelRegistry};
use labels::render::specification_register;
use labels::source::SourceLocation;
use labels::{Label, LabelShape};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mint = |name: &str| -> Result<LabelMint, Box<dyn std::error::Error>> {
        Ok(LabelMint {
            owner: LabelOwner::Attestation,
            label: Label::parse(name, LabelShape::Attestation)?,
            location: SourceLocation::new("papers/attestation/model.tex", 12, 1),
            home: None,
        })
    };

    let mut registry = LabelRegistry::default();
    registry
        .insert(mint("def:model:classes")?)
        .expect("the first mint of a label is accepted");

    // A duplicate is refused and handed back, never silently overwritten.
    assert!(registry.insert(mint("def:model:classes")?).is_err());
    assert_eq!(registry.len(), 1);

    // The register renders deterministically from the registry alone.
    let register = specification_register(&registry);
    assert!(register.contains("``[A-def:model:classes]``"));
    Ok(())
}
```

Register tokens render as double-backtick spans deliberately: the label scanner
treats those as nonparticipating examples, so a generated register can never
mint or cite the labels it indexes (the ADR-013 generated-register
nonparticipation rule).

## Quickstart: scan Markdown for label tokens

```rust
use labels::markdown::{InlineCodeContext, scan_markdown};
use std::path::Path;

// The path is a diagnostic label only; the text is the input.
let scan = scan_markdown(Path::new("fixture.md"), "# Fixture · `sec:fixture`\n");

assert!(scan.diagnostics.is_empty());
assert_eq!(scan.code_spans.len(), 1);

// Only a bare span mints; a parenthesized one is a citation.
assert_eq!(scan.code_spans[0].context, InlineCodeContext::Bare);
```

## What needs a repository on disk

Everything shown above is pure. The entry points that read the tree are
`check_repository`, `generate_registers`, `repository::model_labels_json`,
`plans::check_plans`, `RepositoryCensus::discover`, and the three harvesters
(`latex::harvest_attestation`, `rust_source::harvest_model`,
`rust_source::harvest_crates`).

There is no in-memory constructor for a `CheckReport`, and none is offered:
the gate's subject is a real checkout, and a report assembled from synthetic
counts would claim a verdict about a tree nobody looked at.

## Public-API tour

Fourteen public modules. Five items are re-exported at the crate root —
`Label`, `LabelShape`, `LabelDiagnostic`, `LabelErrorCode`, `Severity`,
`CensusGroup`, `RepositoryCensus`, `CheckReport`, `check_repository`,
`group_crate_sources`, `generate_registers`, and `model_labels_json` —
everything else is reached by module path.

### `label` — the validated token

```text
Label::parse(value: &str, shape: LabelShape) -> Result<Label, LabelParseError>
Label::as_str(&self) -> &str
```

`Label` implements `Display` and the full ordering and hashing traits, so it is
usable as a map key. `LabelShape` has five variants and no methods.
`LabelParseError` has two variants: `Malformed` (bad characters) and `Shape`
(wrong arity for the owner).

### `owner` — owners and cross-owner citations

```text
LabelOwner::prefix(&self) -> String
LabelOwner::shape(&self) -> LabelShape
ImportedLabel::parse(value: &str) -> Result<ImportedLabel, OwnerParseError>
```

`ImportedLabel` has two public fields, `owner: LabelOwner` and `label: Label`.
`OwnerParseError` has two variants: `Unknown` (no such owner prefix) and
`Label` (the local part failed to parse).

### `source` — locations

```text
SourceLocation::new(relative_path: impl Into<PathBuf>, line: usize, column: usize) -> Self
SourceLocation::display_path(&self) -> String
slash_path(path: &Path) -> String
relative_to(root: &Path, path: &Path) -> PathBuf
```

Lines and columns are one-based, and the path is repository-relative.

### `diagnostic` — located defects

```text
LabelDiagnostic::error(code: LabelErrorCode, location: &SourceLocation, message: impl Into<String>) -> Self
LabelDiagnostic::is_error(&self) -> bool
sort_diagnostics(diagnostics: &mut [LabelDiagnostic])
```

`LabelDiagnostic` has six public fields: `code`, `severity`, `message`, `path`,
`line`, `column`. Note that `error` is the **only** public constructor — there
is no `warning` constructor.

`LabelErrorCode` is a flat vocabulary of **21 codes** with no prefix families;
they are enumerated in declaration order in `src/diagnostic.rs`, and that order
is what `sort_diagnostics` sorts by. Representative codes: `Io`,
`UnclosedMarkdownFence`, `DuplicateMint`, `UnknownImportedLabel`,
`CensusUnreadable`.

`Severity` is `Error` or `Warning`. Only `Error` fails a lane.

### `registry` — the mint store

```text
LabelRegistry::insert(&mut self, mint: LabelMint) -> Result<(), LabelMint>
LabelRegistry::insert_or_diagnose(&mut self, mint: LabelMint, owner_name: &str, diagnostics: &mut Vec<LabelDiagnostic>)
LabelRegistry::contains(&self, label: &Label) -> bool
LabelRegistry::get(&self, label: &Label) -> Option<&LabelMint>
LabelRegistry::len(&self) -> usize
LabelRegistry::is_empty(&self) -> bool
LabelRegistry::iter(&self) -> impl Iterator<Item = (&Label, &LabelMint)>
LabelRegistry::labels(&self) -> impl Iterator<Item = &Label>
```

`insert` refuses a duplicate and hands the offered mint back rather than
overwriting; `insert_or_diagnose` wraps it so that every owner's harvest reports
a duplicate in the same message shape, naming the original location.
`LabelMint` has four public fields: `owner`, `label`, `location`, and `home`
(the owning heading, where one is known).

`RegistrySet` collects every owner at once, with public fields `attestation`,
`realization`, `adrs: BTreeMap<u16, _>`, `model`, `plan`, `doc`, and
`crates: BTreeMap<String, _>`.

### `render` — the three pure renderers

```text
specification_register(registry: &LabelRegistry) -> String
realization_register(registry: &LabelRegistry) -> String
model_labels_json(registry: &LabelRegistry) -> Result<String, serde_json::Error>
```

All three are pure functions of the registry, perform no I/O, and are
deterministic: the registry is a `BTreeMap`, so output order is the label
order and nothing else.

### `markdown` — the accepted grammar

```text
scan_markdown(path: &Path, source: &str) -> MarkdownScan
```

`MarkdownScan` carries `code_spans: Vec<InlineCodeSpan>` and
`diagnostics: Vec<LabelDiagnostic>`. `InlineCodeSpan` carries `content`,
`delimiter_len`, `location`, `context`, and `home`.

`InlineCodeContext` decides what a span means, and only `Bare` mints:

| Variant | Meaning |
|---|---|
| `Bare` | a mint |
| `Parenthesized` | a citation |
| `Asymmetric` | a defect: unbalanced citation parentheses |
| `MalformedGroup` | a defect: a malformed citation group |

### `census` — the role-grouped file census

```text
RepositoryCensus::discover(root: impl AsRef<Path>) -> Self
RepositoryCensus::verify(&self, groups: &[CensusGroup]) -> Vec<LabelDiagnostic>
RepositoryCensus::resolve(root: &Path, path: PathBuf) -> PathBuf
group_crate_sources(root: &Path, files: impl IntoIterator<Item = PathBuf>) -> Result<BTreeMap<String, Vec<PathBuf>>, String>
parse_tracked_listing(listing: &str) -> Result<Vec<TrackedEntry<'_>>, String>
audit_census<'a>(tracked: ..., declared: ..., excluded: ..., exclude_pattern: &Regex) -> CensusAuditReport
```

`RepositoryCensus` has thirteen public fields naming each role: `root`,
`attestation_main`, `attestation_sections`, `realization`, `adrs`, `plans`, `docs`,
`model_sources`, `crate_sources`, the three generated-artifact paths, and
`traversal`.

`CensusGroup` has seven variants and three censuses:
`CensusGroup::ALL` (every group), `REGISTER_SCOPED` (Attestation and Realization —
what `generate_registers` needs), and `SCOPED` (those two plus Model — what
`model_labels_json` needs).

`group_crate_sources` performs no filesystem access: it is pure path arithmetic
requiring each path to sit under `packages/<name>/src`, and it rejects `model`,
which has its own role.

### `check` — the label gate

```text
check_repository(paths: &RepositoryCensus) -> (CheckReport, Vec<LabelDiagnostic>)
```

Reads the tree and writes nothing. It harvests every owner, verifies the census
over `CensusGroup::ALL`, then re-renders the three generated artifacts and
byte-compares them against disk, so a stale register is a defect rather than
something a later run quietly fixes. Diagnostics come back sorted, and
`report.valid` is false exactly when some diagnostic is an error.

`CheckReport` is a plain serializable summary with ten public count fields and
`valid`. `CHECK_REPORT_SCHEMA` is 3, advanced for the renamed `attestation_labels` count field.

### `plans` — the plan-tree lane

```text
check_plans(root: &Path, subjects: &[PathBuf]) -> anyhow::Result<PlansOutcome>
```

`PlansOutcome` carries `report: PlansReport`, `failures: Vec<String>`, and
`warnings: Vec<String>`. `check_plans` returns `Err` **only** for environmental
faults such as an unreadable file; every tree defect is a `failures` entry
instead. `PLANS_REPORT_SCHEMA` is 2.

### `repository` — harvest, graph, and the writing entry point

```text
RepositoryLabels::harvest_sources(paths: &RepositoryCensus) -> Self
RepositoryLabels::has_errors(&self) -> bool
RepositoryLabels::imported_citation_count(&self) -> usize
build_label_graph(registries: &RegistrySet, citations: ..., diagnostics: &mut Vec<LabelDiagnostic>) -> (DiGraph<...>, BTreeMap<LabelGraphNodeId, NodeIndex<u32>>)
project_label_graph(graph: &DiGraph<...>) -> LabelGraphProjection
generate_registers(paths: &RepositoryCensus, specification_output: &Path, realization_output: &Path) -> Result<Vec<GeneratedRegister>, GenerateError>
model_labels_json(paths: &RepositoryCensus) -> Result<String, GenerateError>
```

`generate_registers` is the crate's **only** writing entry point. It enforces
role-distinct outputs, refuses to write at all if any diagnostic is an error
(`GenerateError::Validation`), and publishes both files atomically as a batch,
returning one `GeneratedRegister { path, bytes }` per file.

The citation graph keys nodes by `LabelGraphNodeId`, a stable owner/label
identity; Petgraph `NodeIndex` values are local handles only and must never be
persisted or compared across runs. `project_label_graph` gives the sorted,
comparable snapshot.

`GenerateError` has five variants: `AliasedOutputs`, `Publication`,
`Validation(Vec<LabelDiagnostic>)`, `Io`, and `Json`. Its `diagnostics()`
accessor reaches the validation diagnostics.

### `latex`, `rust_source`, `forbidden`

```text
latex::harvest_attestation(paths: &RepositoryCensus) -> (LabelRegistry, Vec<LabelDiagnostic>)
rust_source::harvest_model(paths: &RepositoryCensus) -> RustHarvest
rust_source::harvest_crates(paths: &RepositoryCensus) -> BTreeMap<String, RustHarvest>
rust_source::model_labels(registry: &LabelRegistry) -> BTreeSet<String>
forbidden::forbidden_needle() -> String
forbidden::parse_grep_matches(output: &str) -> Vec<ForbiddenTextMatch>
```

`rust_source` also publishes the two type censuses `MODEL_TYPES` and
`REALIZATION_TYPES`. `RustHarvest` carries `registry`, `citations`, and
`diagnostics`.

`ForbiddenTextMatch` carries `path`, `line`, and `policy` — the policy
identifier, never the offending token itself, so the audit's own report cannot
reproduce what it forbids.

## Error handling

The crate separates two kinds of wrong, and the distinction is load-bearing.

A **diagnostic** is a located defect in the documentation being checked. It is
returned as data in a `Vec<LabelDiagnostic>` and never aborts a run: a lane
collects every defect in the tree and reports them together, because a checker
that stopped at the first defect would need as many runs as there are defects.
`LabelDiagnostic::is_error` and `Severity` decide whether a lane fails.

An **error** is an environmental or input fault, and only a few entry points
return one:

| Error | Returned by | Variants |
|---|---|---|
| `LabelParseError` | `Label::parse` | `Malformed`, `Shape` |
| `owner::OwnerParseError` | `ImportedLabel::parse` | `Unknown`, `Label` |
| `repository::GenerateError` | `generate_registers`, `repository::model_labels_json` | `AliasedOutputs`, `Publication`, `Validation`, `Io`, `Json` |
| `serde_json::Error` | `render::model_labels_json` | upstream |
| `String` | `group_crate_sources`, `parse_tracked_listing` | a message |
| `anyhow::Error` | `plans::check_plans` | environmental faults only |

`check_repository` returns no `Result` at all: an unreadable file becomes a
`LabelErrorCode::Io` diagnostic, so a fault in the tree is reported in the same
place as a defect in it.

## Binaries

Five binaries, auto-discovered from `src/bin/`. All accept `--debug` and
`--repository-root <DIR>`; the checking ones accept `--report <FILE>` and
`--stamp <FILE>` together (each requires the other) and emit one JSON report on
stdout.

| Binary | Arguments beyond the shared ones | Writes |
|---|---|---|
| `check-labels` | `--attestation-main`, `--attestation-section` (repeatable), `--realization`, `--adr` (repeatable), `--plan` (repeatable), `--doc` (repeatable), `--model-source` (repeatable), `--crate-source` (repeatable), `--specification-register`, `--realization-register`, `--model-labels-json` | nothing |
| `check-plans` | `--subject <FILE>` (repeatable, required) | nothing |
| `census-audit` | `--git <PROGRAM>`, `--exclude-pattern <REGEX>`, `--excluded <FILE>` (repeatable), and positional `<FILE>...` for the declared census | nothing |
| `check-forbidden-text` | `--git <PROGRAM>` | nothing |
| `generate-label-registers` | `--attestation-main`, `--attestation-section` (repeatable), `--realization`, `--specification-register-output`, `--realization-register-output` | the two registers |

`census-audit` is the only binary taking positional arguments.

## What this package deliberately does not do

- It is not a semantic input. No protocol, model, or compiler package consumes
  its registries, and a planning label carries no identity anything depends on.
- It resolves no repository path from its own compiled location: every subject
  arrives as a role-tagged argument (ADR-014), and the binaries re-verify the
  declared census against the tree before trusting it.
- It writes nothing except the two registers, and only through
  `generate_registers`. Every other lane is non-writing. The `artifacts` package
  remains responsible for writing
  `packages/model/generated/model_labels.json` — this crate only derives the
  content.
- It does not fix what it finds. A stale register is reported, never
  regenerated, by `check_repository`.
- It parses no general Markdown or LaTeX. The scanners accept exactly the
  grammar the label rules are stated over.
- It reproduces no forbidden token: the placeholder audit reports a policy
  identifier and a location, never the matched text.
