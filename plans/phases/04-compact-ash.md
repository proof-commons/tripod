# Phase 4 — End-to-End Compact ASH · `phase:roadmap:compact-ash`

> **Status:** Exited — the Phase-4 exit gate passed 2026-08-21 and is
> recorded in the backlog's section 2.8: twelve accepted and matched
> live submissions, honest coverage 100 of 211, the section-29 audit at
> 94 of 105, the local gate 47 of 47 with byte-identical document
> reproduction. See [the Guide-12 completion
> record](../history/guide-12-completion-report.md).
> **Entry:** (`gate:phase2:exit`), (`gate:phase3:exit`)
> **Packages:** compiler, target-elements, tapscript, linker, transaction,
> vectors
> **Primary semantic operation:** `compact-ash`

## Goal · `sec:phase4:goal`

Prove the complete compiler-to-target pipeline on the smallest root-free,
permissionless protocol operation.

```text
architecture
→ realization
→ compiler
→ tapscript
→ linker
→ transaction ABI
→ target transaction
→ relation-indexed evidence
```

## Deliverables · `sec:phase4:deliverables`

### Backend emission

Emit:

- local ASH participation program;
- one canonical ASH coordinator program;
- explicit closed `U` checks;
- public/openable value checks;
- family count/range checks;
- one ASH successor check;
- exact ownerless conservation;
- sponsor isolation;
- no specialized event projection;
- symbols, relocations, witness roles, and resource formulas.

### Linking

Produce a candidate linked bundle containing:

- linked ASH constructor;
- operation programs;
- deterministic target program tree/control recipes;
- complete relation carriers;
- candidate bound assignment;
- linked resource formulas;
- candidate identity.

### Transaction ABI

Produce a candidate ABI with:

```text
ASH input family
optional sponsor suffix
exactly one ASH output
optional sponsor change
canonical coordinator
```

Construct valid transactions using public data plus sponsor-local data only.

### Evidence

Produce semantic, target, coverage, and resource reports bound to the exact
candidate bundle and ABI.

## Required vectors · `sec:phase4:vectors`

Positive:

- minimum two ASH inputs;
- larger bounded batch;
- sponsorless;
- sponsored;
- canonical input normalization;
- exact ASH successor value.

Negative:

- zero or one ASH input;
- count above candidate maximum;
- duplicate input;
- wrong asset;
- wrong constructor;
- wrong coordinator;
- malformed family range;
- no ASH output;
- two ASH outputs;
- wrong output value;
- undeclared closed-asset output;
- confidential/unclassified `U`;
- sponsor overlap;
- hidden owner/operator signature;
- wrong program or control path.

## Resource evidence · `sec:phase4:resources`

Measure complete valid transactions for:

- minimum batch;
- larger batch;
- candidate maximum;
- maximum sponsor candidate;
- deepest relevant control path;
- highest witness/stack objective.

Compare predicted and observed:

- transaction weight;
- witness bytes;
- stack/altstack;
- element size;
- target operation cost;
- crypto budget;
- policy result.

## Assurance boundary · `sec:phase4:assurance`

Success establishes translation validation for one exact compact-ASH bundle.

It does not establish later operations, final calibration, production target
activation, or deployment release.

## Exit gate · `gate:phase4:exit`

Phase 4 exits when:

- every compact-ASH semantic relation survives analysis, emission, and linking;
- every relation has a reachable carrier;
- one typed deterministic ABI drives clients, vectors, and measurement;
- valid target transactions accept with matching semantic projection;
- focused target mutations reject;
- permissionless construction uses no hidden secret;
- predicted and observed resources agree;
- candidate scope/status is explicit;
- reports are deterministic and secret-free;
- checks leave the repository clean.
