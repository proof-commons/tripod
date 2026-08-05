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

Implemented: typed input binding (P2-004); exact scoped relation and
expression DAGs (C1-005/P2-005); conservative checked constant folding
(P2-006); proof-obligation classification, exact feasible proof-plan
enumeration under explicit search limits, authenticatable source
requirements, authorization-case constructibility analysis, plan-specific
disclosure analysis, and representation lifecycle analysis
(C1-008/P2-007/P2-008/P2-009), oracle-checked; typed execution cases,
relation discharge classification, carrier eligibility, exact feasible
placement, and target-independent layout requirements (C1-009/P2-010);
relation-indexed coverage requirements, typed coverage dependencies with a
forbidden-cycle SCC policy, and independent placement and coverage oracles
(P2-011/C1-010/C1-013).

Not implemented: target capability adapter, concrete target layout, and the
complete pilot analyzed program. The analysis structures are crate-private, no complete
analyzed program is exposed, and no compiler-plan identity exists — typed
comparison remains the boundary. No public value this crate produces today
can be mistaken for a completed analysis.
