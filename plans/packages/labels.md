# Repository Label Tooling · `pkg:labels:contract`

> **Status:** Active
> **Package:** `tripod-labels`
> **Library:** `labels`
> **Direct dependencies:** `architecture`, `petgraph`
> **Policy:** (`[ADR012-rule:labels:decision]`),
> [D007](../decisions/007-petgraph-graph-substrate.md)
> **Assurance class:** Documentation conformance

## Purpose · `sec:labels-package:purpose`

`labels` provides one Rust implementation for repository documentation-label
harvesting, imported-citation validation, deterministic registries, and source
weld checks.

It owns Attestation LaTeX, realization Markdown, ADR, first-party crate Rust,
and planning imported labels, plus the two generated upstream planning registers.

Plan-local labels remain non-normative; under
(`[ADR013-rule:labels:global-resolution]`) they are linted for one mint
per label and complete citation resolution.

## Inputs · `sec:labels-package:inputs`

The package reads typed architecture labels and documentation source files:

- `architecture::ARCHITECTURE`;
- Attestation LaTeX;
- realization Markdown;
- ADR and planning Markdown;
- model Rust source;
- committed generated registries.

## Outputs · `sec:labels-package:outputs`

It produces typed owner registries, diagnostics, one JSON check report,
deterministic upstream registers, and the model-label JSON derivation.
The package owns a direct Petgraph graph whose nodes are mints and citations
and whose edges resolve each citation to exactly one mint.

## Scoped derivations · `rule:labels-package:scopes`

The package keeps one parser implementation but separates derivation scopes:

```rust
derive_register_sources(paths)  // attestation and realization
derive_model_sources(paths)     // and model sources
check_repository(paths)         // every owner
```

Each verifies only the census groups it reads, so a stale plan or ADR census
cannot block an upstream derivation. `generate-label-registers` derives from
the two upstream sources alone; `model_labels_json` adds model Rust to them,
because a model label's citations must resolve upstream. `check-labels` runs
every owner and the publication-freshness check. Planning or ADR prose must
not block model-label publication.

## Source census · `rule:labels-package:census`

Repository-wide means every first-party source class with an explicit label
policy, not arbitrary checkout recursion. The census includes Attestation LaTeX,
the realization, ADR and planning Markdown, first-party Markdown consumers such
as root/package READMEs and `human.md`, and first-party crate Rust comments
(string literals do not participate). It excludes `target/`,
build directories, `archive/`, `.git/`, generated binary artifacts, and
third-party/vendor trees. Traversal errors are diagnostics; an unreadable tree
must not silently become an empty census.

`MODEL-` is an explicit imported owner for future planning citations. Existing
unprefixed model spans in the realization remain a document-specific legacy
form. Owner-qualified spans in the realization retain that document's existing
syntax; planning and ADR imports require the parenthesized square-bracket form.

## Forbidden use · `sec:labels-package:forbidden`

No semantic package consumes label registries or planning labels. The package
does not define protocol semantics, make plan labels release-stable, write
during checks, or include ambient time or host paths in output.

## Commands · `sec:labels-package:commands`

```text
check-labels
    non-writing; one JSON stdout result on success; JSON stderr diagnostics

generate-label-registers
    writes only plans/labels/specification.md and plans/labels/realization.md
```

## Meson · `sec:labels-package:meson`

Meson invokes one direct non-writing `labels-check` test and exposes one
explicit non-default `generate-label-registers` run target. It owns no separate
parser or generated source.

## Error vocabulary · `sec:labels-package:errors`

See [`errors/labels.md`](errors/labels.md).

## Exit gate · `gate:labels-package:exit`

The package is complete when source labels and imported citations validate,
planning and documentation citations resolve, generated registers are current and
deterministic, checks are non-writing, artifacts remains the model-publication
writer, and CI/Meson invoke this package directly.