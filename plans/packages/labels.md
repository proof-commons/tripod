# Repository Label Tooling · `pkg:labels:contract`

> **Status:** Active
> **Package:** `tripod-labels`
> **Library:** `labels`
> **Direct dependency:** `architecture`
> **Policy:** (`[ADR012-rule:labels:decision]`)
> **Assurance class:** Documentation conformance

## Purpose · `sec:labels-package:purpose`

`labels` provides one Rust implementation for repository documentation-label
harvesting, imported-citation validation, deterministic registries, and source
weld checks.

It owns Layer-0 LaTeX, Realization Markdown, ADR, model Rust, and planning
imported labels, plus the generated Layer-0 and v13 planning registers.

Plan-local labels remain non-normative and unlinted.

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
deterministic Layer-0 and v13 registers, and the model-label JSON derivation.

## Forbidden use · `sec:labels-package:forbidden`

No semantic package consumes label registries or planning labels. The package
does not define protocol semantics, make plan labels stable, lint plan-local
citations, write during checks, or include ambient time or host paths in output.

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

## Exit gate · `gate:labels-package:exit`

The package is complete when source labels and imported citations validate,
plan-local labels remain ignored, generated registers are current and
deterministic, checks are non-writing, artifacts remains the model-publication
writer, and CI/Meson invoke this package directly.