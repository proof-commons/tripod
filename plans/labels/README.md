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
(`inf:labels:import`).

## Registers · `tbl:labels-index:files`

| Register | Owner | Source |
|---|---|---|
| [Specification](specification.md) | the Attestation specification | `papers/attestation/main.tex` and `sections/*.tex` |
| [Realization](realization.md) | the realization document | `docs/attestation/realization.md` |

The registers are derivative indexes. Their source documents remain
authoritative.

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
```

Update mode may write only the two owned register files.

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
