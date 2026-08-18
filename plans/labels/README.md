# Upstream Label Registers

This directory contains generated indexes of labels owned by normative
documents above the planning layer.

## Purpose · `sec:labels-index:purpose`

Planning documents use two cross-reference classes:

```text
local planning citation:
    (`pkg:realization:contract`)

imported upstream citation:
    (`[RZ-sec:realization:representation]`)
```

Plan-local labels are non-normative, but they are mechanically complete:
every planning label has one mint (`inv:labels:unique-mint`)
and every local citation resolves across the planning tree
(`inv:labels:total-resolution`).

Imported labels are authoritative references and are linted under
(`inf:labels:imported-citation`).

## Registers · `tab:labels-index:files`

| Register | Owner | Source |
|---|---|---|
| [Specification](specification.md) | the Attestation specification | `papers/attestation/main.tex` and `sections/*.tex` |
| [Realization](realization.md) | the realization document | `docs/attestation/realization.md` |
| [Attestation](attestation.md) | This repository, as acceptee | `plans/drafts/environment-kinds.md`, `adr/020-environment-kinds.md`, and the corpus census |

The registers are derivative indexes. Their source documents remain
authoritative.

The first two index upstream labels. The third is the companion
attestation register the adopted kind registry requires of an acceptee:
it views this repository's evidence base and status map, and presents
the homonymy of the effective relation. It indexes no labels and is not
a citation source.

## Citation prefixes · `rule:labels-index:prefixes`

Use:

```text
A-      the Attestation specification
RZ-     the realization document
```

Examples:

```text
(`[A-def:model:classes]`)
(`[RZ-sec:realization:representation]`)
(`[RZ-obl:oracle:ledger]`)
```

ADR labels are linted directly from ADR source and do not require a committed
aggregate register.

## Updating · `rule:labels-index:update`

An explicit update command regenerates the registers.

Run:

```sh
meson compile -C build generate-label-registers
meson compile -C build generate-attestation-register
```

The first may write only the two upstream register files; the second
only the attestation register. The two generators are separate because
their derivations are: the upstream registers are scoped to the specification
and realization sources, while the attestation register's mint census
answers to the whole corpus.

## Checking · `rule:labels-index:check`

CI uses non-writing check mode:

```sh
meson compile -C build lint
```

The checker derives expected bytes in memory and fails when a register is
missing or stale.

## Machine-use prohibition · `rem:labels-index:machine-use`

No compiler, backend, linker, transaction builder, vector generator, or release
validator consumes these Markdown registers.

The documentation checker is their only first-party machine consumer.
