# Architecture Decision Records

This directory records implemented or repository-wide engineering policy.

ADRs govern current first-party code. They are subordinate to normative
protocol artifacts but take precedence over planning documents.

## Authority · `sec:adr:authority`

The relevant authority order is:

1. Attestation economic specification;
2. realization conformance contract;
3. typed architecture;
4. implemented ADRs;
5. planning decisions;
6. package, phase, research, and backlog documents.

Plans may propose future policy. An ADR records policy accepted for the
repository itself.

## Record index · `tbl:adr:index`

| Record | Status | Policy |
|---|---|---|
| [ADR-010](010-command-line-output-contract.md) | Decided and implemented | First-party command-line streams, diagnostics, and exit classes. |
| [ADR-011](011-toolchain-and-dependency-policy.md) | Decided and implemented | Rust, dependency, locking, unsafe-code, target-compatibility, and reproducibility policy. |
| [ADR-012](012-documentation-labels.md) | Implemented; superseded in part by ADR-013 | Documentation labels, citation layers, registers, and lint boundaries. |
| [ADR-013](013-global-label-graph.md) | Decided and implemented | Owner-aware global label graph across Markdown and Rust sources. |
| [ADR-014](014-meson-lint-census-and-stamps.md) | Decided and implemented | Build-system-owned lint census, argument-passed inputs, and stamp-file dependency edges. |
| [ADR-015](015-public-data-and-execution-trust.md) | Decided and implemented | Public-data interfaces, untrusted execution, crash artifacts, and future secret boundaries. |
| [ADR-016](016-semantic-identities-and-evidence-binding.md) | Proposed | Admission, ownership, flow, consumption, and assurance meaning of semantic identities, artifact digests, provenance identities, evidence reports, and release roots. |

## Numbering · `rule:adr:numbering`

ADR numbers are permanent and never reused.

A replacement receives a new number and names the records it supersedes.
Implemented ADR identities remain stable even when their prose is rewritten
for density.

## Labels · `rule:adr:labels`

Each numbered ADR is an independent label owner under
(`[ADR013-rule:labels:adr-owners]`); mint discipline follows
(`[ADR012-rule:labels:mint]`).

A plan citing an ADR label uses the ADR owner prefix, for example:

```text
(`[ADR010-rule:output:streams]`)
(`[ADR011-rule:toolchain:locked]`)
(`[ADR013-rule:labels:owner-relative]`)
```

ADR labels are policy-owned upstream labels and are linted when imported.

## Writing rules · `rule:adr:writing`

Each ADR states:

- context;
- decision;
- consequences;
- rejected alternatives;
- implementation or verification gate.

ADRs do not repeat package plans, roadmap sequencing, or long implementation
catalogues.

## Machine use · `rem:adr:machine-use`

ADR Markdown is not compiler, linker, transaction, or release input.

Implemented policy is represented in code, configuration, and tests. ADRs
explain and govern that implementation.

