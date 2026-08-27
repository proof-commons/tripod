# Compiler Roadmap

The roadmap orders the build-out of Tripod, the contract closure compiler, along its exemplar: each phase carries one slice of the attestation contract from realization to target bytes, and the compiler is accepted only as far as the exemplar proves it. Phase details live under [phases/](phases/README.md).

## Sequence · `rule:roadmap:sequence`

```text
0  baseline
1  realization
2  compiler
3  target foundation
4  compact-ash
5  live transfer
6  STATE and maturity
7  burn and clear
8  redemption
9  requests and admission
10 settlement
11 cycle
12 release
```

## Dependency rationale · `rule:roadmap:dependencies`

The order is intentional:

- semantic declarations precede compiler analysis;
- compiler analysis precedes target emission;
- target prototypes precede production backend APIs;
- `compact-ash` proves the complete pipeline first;
- transfer proves owner authorization and representation latitude;
- STATE operations wait for constructor continuity;
- redemption waits for wide arithmetic;
- settlement waits for arithmetic and a batch-size-2 layout prototype;
- cycle arrives after its issuance, STATE, arithmetic, distribution, maturity,
  and resource dependencies;
- final release waits for independent event, query, and accounting evidence.

## Status · `phase:roadmap:current`

```text
Current: Phase 6 - STATE constructor and maturity announcement
```

## Phase index · `tab:roadmap:phases`

| Phase | Card |
|---:|---|
| 0 | [phases/00-baseline.md](phases/00-baseline.md) |
| 1 | [phases/01-realization.md](phases/01-realization.md) |
| 2 | [phases/02-compiler.md](phases/02-compiler.md) |
| 3 | [phases/03-target-foundation.md](phases/03-target-foundation.md) |
| 4 | [phases/04-compact-ash.md](phases/04-compact-ash.md) |
| 5 | [phases/05-live-transfer.md](phases/05-live-transfer.md) |
| 6 | [phases/06-state-and-maturity.md](phases/06-state-and-maturity.md) |
| 7 | [phases/07-burn-and-clear.md](phases/07-burn-and-clear.md) |
| 8 | [phases/08-redemption.md](phases/08-redemption.md) |
| 9 | [phases/09-requests-and-admission.md](phases/09-requests-and-admission.md) |
| 10 | [phases/10-settlement.md](phases/10-settlement.md) |
| 11 | [phases/11-cycle.md](phases/11-cycle.md) |
| 12 | [phases/12-release.md](phases/12-release.md) |

## Change rule · `rule:roadmap:change`

A phase-order change must state changed dependency, supporting evidence,
affected package contracts, affected research notes, effect on current backlog,
and normative-versioning impact, if any.

## Parallel work · `rule:roadmap:parallel`

Allowed before the consuming phase: source review, throwaway target
experiments, isolated constructor/arithmetic/declassification prototypes, vector
design, independent observer implementation, and report-schema experiments.

Not allowed before the consuming gate: target fields in `RealizationSpec`, a
stable prototype ABI, release capability claims, calibrated bounds from draft
defaults, production-marked target proof patterns, or test doubles labeled
independent.

## Deferred work · `sec:roadmap:deferred`

Deferred until after the first tapscript deployment:

- production Simplicity backend;
- confidential closed protocol asset identity;
- general optimizer infrastructure;
- broad formal target semantics;
- private entitlement arithmetic;
- arbitrary sponsor sidecars;
- wallet-balance activation in the model; and
- protocol changes not required by implementation evidence.

Deferred work re-enters only through an accepted decision or research note and
a roadmap change.

## Cross-phase laws · `rule:roadmap:laws`

Every phase preserves:

- typed semantic ownership;
- one-way generated artifacts;
- deterministic explicit-input output;
- non-writing checks;
- locked builds;
- clean-tree verification;
- separate evidence classes;
- fail-closed unresolved requirements.