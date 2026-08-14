# `tripod-tapscript`

`tapscript` adapts compiler-owned abstract target requirements to target-owned
primitive contracts, backend-pattern obligations, structural obligations, and
external evidence requirements.

The package contract is
[plans/packages/tapscript.md](../../plans/packages/tapscript.md). That contract
describes the eventual backend; this crate is its first, narrow stage.

## Dependencies

Two first-party packages and no third-party ones:

```text
compiler
target-elements
```

`architecture`, `realization`, and `model` are deliberately absent. An
assessment names compiler-owned abstract capabilities and target-owned
primitives; the compiler already publishes the architecture-owned facts it is
willing to project, and citing those packages here would reach around that
projection rather than consume it. `linker`, `transaction`, `vectors`,
`release`, and `artifacts` are absent because no target program, bundle, or
publication exists to hand them.

## Implemented

```text
package boundary
static capability adapter over the reviewed target contract
external-evidence-role adapter
```

## Static, not deployment-aware

Every assessment here is a statement about the reviewed *static* target
contract. Nothing in the crate accepts a development binding, and nothing reads
a network identity, a genesis identity, an activation declaration, or a
deployment resource override. The static contract, the deployment declaration,
and target-native evidence are three different values; a function that took one
and answered for another would be a false claim about binding-aware assessment.
A deployment-aware assessment is deferred until a consumer for one exists.

## Both published censuses are answered

The compiler publishes what an analysis requires as two censuses — abstract
capabilities and external-evidence roles — and the adapter answers both, with
exact equality in both directions. A compiler evidence role therefore cannot
disappear at this boundary: the mapping is exhaustive, so a new role stops this
crate compiling until its target obligation is stated.

## Not implemented

```text
target program type
instruction builder
stack scheduler
backend proof patterns
constructors
relocatable bundle
```

## Support is not a boolean

An assessment is a typed multi-state result, never `Supported(bool)`. It
separates a reviewed negative fact from a missing primitive, a missing
primitive from a backend pattern that nobody has written, a backend pattern
from a structural obligation the compiler and the ABI owe, and a structural
obligation from external evidence only the target's own rules can produce.
Collapsing any of those distinctions would let an assessment read as progress
that has not happened.

## Nothing here is claimed to work against a node

No target program has been emitted, no transaction has been built, and no
evidence requirement the target contract names has been discharged. An
assessment states what a future backend would have to establish. It states
nothing about whether it has been established.
