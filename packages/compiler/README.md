# `tripod-compiler`

`compiler` analyzes a validated realization into a deterministic
target-independent compilation plan. It emits no target program.

The package contract is [plans/packages/compiler.md](../../plans/packages/compiler.md).

## Input

The crate consumes typed values only: a validated scoped realization, an
explicit compilation scope, a typed analysis policy, and optionally abstract
target capabilities.

It does not consume:

- generated architecture, realization, or declassification publications;
- model source, labels, or tests;
- plans, ADR, or target reference prose;
- target bytecode or disassembly;
- environment or filesystem state.

## Output

The analyzed value will carry architecture and realization bindings, explicit
scope, the normalized relation graph, source provenance, proof alternatives,
disclosure analysis, fact-source requirements, constructibility, lifecycle,
placement and layout requirements, target capability requirements, and coverage
requirements.

No target opcode, stack index, tapleaf, transaction position, or target byte
enters compiler core. Concrete positions are backend output.

## Identity

The crate mints no public compiler digest. An analysis identity activates only
once a real cross-process, cached, or published consumer exists; until then
typed comparison is the boundary.

## State

The crate implements typed input binding (P2-004), exact scoped relation and
expression graph construction (C1-005/P2-005), and conservative checked
constant folding (P2-006). The analysis structures are crate-private: proof
planning, source/disclosure analysis, lifecycle, placement, layout, coverage,
and the pilot analyzed program remain unimplemented, and no complete analyzed
program is exposed. No public value this crate produces today can be mistaken
for a completed analysis.
