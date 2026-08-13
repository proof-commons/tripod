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
compiler-to-target capability adapter
```

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
