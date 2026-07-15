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
| [ADR-012](012-documentation-labels.md) | Decided; implementation pending | Documentation labels, citation layers, registers, and lint boundaries. |

## Numbering · `rule:adr:numbering`

ADR numbers are permanent and never reused.

A replacement receives a new number and names the records it supersedes.
Implemented ADR identities remain stable even when their prose is rewritten
for density.

## Labels · `rule:adr:labels`

ADRs mint labels under (`[ADR012-rule:labels:mint]`).

A plan citing an ADR label uses the ADR owner prefix, for example:

```text
(`[ADR010-rule:output:streams]`)
(`[ADR011-rule:toolchain:locked]`)
(`[ADR012-rule:labels:external-citation]`)
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

