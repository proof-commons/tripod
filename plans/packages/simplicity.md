# Simplicity Backend · `pkg:simplicity:contract`

> **Status:** Parked
> **Package:** `tripod-simplicity`
> **Decision:** [D003](../decisions/003-tapscript-first.md)

## Purpose · `sec:simplicity:purpose`

A future Simplicity backend will consume the same target-independent compiler
relations and produce separate target programs, bundle identity, ABI, resource
reports, and execution evidence.

Its first architectural role is to test that the shared boundary is genuinely
backend-independent.

## Current obligation · `rule:simplicity:parked`

While parked:

- no workspace crate is required;
- no Simplicity dependency is added;
- no release claims Simplicity support;
- no placeholder target or bundle identity is published;
- no target-independent type may nevertheless assume tapscript stacks,
  tapleaves, control blocks, or taptrees.

## Reactivation · `gate:simplicity:reactivation`

Work may begin after:

- realization and compiler relations cover the selected deployment scope;
- tapscript has demonstrated linking and transaction ABI boundaries;
- relation-indexed semantic vectors are target-independent in practice;
- an exact typed Simplicity target can be defined;
- a concrete assurance or deployment need justifies implementation.

The first pilot should be an already implemented semantic operation, normally
compact ASH.

## Dependencies · `sec:simplicity:dependencies`

After reactivation, expected dependencies are:

```text
compiler
future typed Simplicity target contract
```

It must not depend on `tapscript`.

## Typed outputs · `sec:simplicity:outputs`

A future backend owns:

- typed target programs;
- target proof choices;
- target constructor artifacts;
- target placement/layout;
- target witness/program-input ABI requirements;
- resource formulas;
- relation carriers;
- relocatable bundle identity.

## Assurance boundary · `sec:simplicity:assurance`

Target-language formal semantics may strengthen pattern evidence.

They do not automatically prove:

- compiler selection;
- linking;
- transaction construction;
- deployment activation;
- independent observer output.

D004 remains in force unless superseded by a proof covering those boundaries.

## Exit gate · `gate:simplicity:pilot`

The first pilot exits only when it consumes unchanged semantic relation IDs,
passes target-native positive and negative vectors, matches the model’s public
semantic projection, and exposes any leaked tapscript assumptions found in
shared interfaces.

## Error vocabulary · `sec:simplicity:errors`

See [`errors/simplicity.md`](errors/simplicity.md).
