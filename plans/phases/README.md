# Implementation Phases

This directory owns one compact card per implementation phase.

The root [roadmap](../roadmap.md) owns phase order and dependencies. Each phase
card owns its entry conditions, deliverables, evidence, and exit gate.

## Index · `tab:phases:index`

| Phase | Status | Result |
|---|---|---|
| [00-baseline.md](00-baseline.md) | Complete | Reproducible compiler-era baseline. |
| [01-realization.md](01-realization.md) | Complete | Typed realization pilots. |
| [02-compiler.md](02-compiler.md) | Complete | Target-independent compiler analysis. |
| [03-target-foundation.md](03-target-foundation.md) | Active | Typed target and foundational prototypes. |
| [04-compact-ash.md](04-compact-ash.md) | Planned | First complete backend operation. |
| [05-live-transfer.md](05-live-transfer.md) | Planned | Owner authorization and value-representation evidence. |
| [06-state-and-maturity.md](06-state-and-maturity.md) | Planned | STATE constructor and maturity announcement. |
| [07-burn-and-clear.md](07-burn-and-clear.md) | Planned | Burn, ASH, and clear pipeline. |
| [08-redemption.md](08-redemption.md) | Planned | Wide arithmetic and redemption. |
| [09-requests-and-admission.md](09-requests-and-admission.md) | Planned | Request, cancellation, and admission. |
| [10-settlement.md](10-settlement.md) | Planned | Settlement prototype and implementation. |
| [11-cycle.md](11-cycle.md) | Planned | Cycle after dependent seams. |
| [12-release.md](12-release.md) | Planned | Calibration, evidence, profile, and release. |

## Phase form · `rule:phases:form`

Each card contains goal, entry conditions, deliverables, required evidence, and
an exit gate. Package responsibilities are cited from package contracts, not
repeated.

## Gate rule · `rule:phases:gates`

A phase is complete only when its exit gate passes. Code existing in the
repository is not sufficient. Research results must be accepted before
prototype-dependent production APIs freeze.

## Labels · `rule:phases:labels`

Primary phase labels use `phase:roadmap:<name>` and exit gates use
`gate:<phase>:exit`. Both are plan-local, non-normative, and
non-identity-bearing; uniqueness and citation checks still apply.