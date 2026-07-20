# D008: Exact Semantics; Certified Numerical Analysis · `dec:math:exact-certified`

> **Status:** Accepted
> **Class:** Mathematical assurance
> **Depends on:** (`dec:source:typed-rust`)
> **Imports:** (`[ADR011-rule:toolchain:dependencies]`),
> (`[ADR011-rule:toolchain:reproducibility]`),
> (`[RZ-sec:arithmetic:gadgets]`),
> (`[RZ-obl:oracle:determinism]`)
> **Supersedes:** no prior decision

## Choice · `rule:math:exact-certified`

Use exact arithmetic for semantic, identity-bearing, calibration, and release
claims.

Numerical linear algebra may generate candidates, diagnostics, sensitivity
information, conditioning information, and certified approximate results. A raw
floating-point result is never sufficient evidence for exact semantic or release
claims.

## Exact Claims · `rule:math:exact-claims`

The following use checked integers, arbitrary-precision integers, reduced exact
rationals, exact finite search, or independently checked certificates:

- semantic arithmetic and equality;
- conservation;
- cardinality;
- authorization;
- object and relation identity;
- graph coverage;
- proof-plan feasibility;
- carrier placement;
- target-resource limit acceptance;
- calibrated integer bounds;
- canonical publication values;
- deployment and release validation.

A numerical residual does not prove exact equality.

## Problem Classes · `rule:math:problem-classes`

Keep these classes distinct:

```text
graph storage and traversal:
    Petgraph

checked bounded protocol arithmetic:
    first-party typed integer domains

arbitrary-precision exact arithmetic:
    num-bigint, num-integer, num-traits

exact rational arithmetic:
    num-rational when a concrete consumer exists

numerical linear algebra:
    faer when a concrete consumer exists

finite proof/placement selection:
    exact enumeration or branch-and-bound initially

SAT/LP/MILP:
    separately reviewed only after measured need

target arithmetic:
    exact target relation plus independent host reference

cryptographic mathematics:
    reviewed target/cryptographic libraries and target-native evidence
```

A matrix solver is not a graph algorithm. A numerical linear solver is not a
proof-selection optimizer. A graph library is not a semantic AST.

## Numerical Dependency · `rule:math:faer`

`faer` is the preferred private numerical linear-algebra dependency when an
implemented analysis has a concrete numerical problem.

It is not added to packages that have no numerical consumer.

Before adoption, the selected release and features receive the dependency review
required by ADR-011, including MSRV, license, transitive graph, unsafe and SIMD
boundary, determinism, parallelism, and advisory status.

## Numerical Acceptance · `rule:math:numerical-acceptance`

A release-sensitive numerical analysis records:

- exact typed source problem;
- numerical lowering;
- method;
- scaling;
- tolerance;
- decomposition status;
- rank policy;
- condition policy;
- independently recomputed residual;
- backward error;
- certification mode.

For $Ax=b$, the minimum normwise diagnostic is:

$$
\eta =
\frac{\lVert b-Ax\rVert_\infty}
{\lVert A\rVert_\infty\lVert x\rVert_\infty+\lVert b\rVert_\infty}.
$$

A small $\eta$ is diagnostic evidence. Exact semantic acceptance additionally
requires exact reconstruction, exact substitution, or another reviewed
certificate.

## Identity · `rule:math:identity`

The following never enter semantic or release identity:

- raw `f32` or `f64` bits;
- matrix row or column position;
- pivot order;
- solver variable number;
- iteration count;
- thread schedule;
- low-order platform-dependent numerical differences.

Identity-bearing results are exact typed values, reduced rationals, canonical
finite selections, or conservatively converted exact bounds.

## Determinism · `rule:math:determinism`

Numerical execution is sequential initially.

Parallel numerical execution requires a later demonstrated need and tests
showing that accepted exact results, canonical diagnostics, and identity-bearing
outputs remain unchanged across thread counts and schedules.

## Initial Phase-1 Policy · `rule:math:phase1`

Phase 1 requires no numerical dependency.

Compact ASH and live transfer use:

- checked counts;
- checked protocol amounts;
- exact equality;
- exact owner sets;
- exact finite typed relations;
- direct Petgraph graphs.

`num-rational` and `faer` are added only when a concrete implemented analysis
requires them.

## Does Not Authorize · `sec:math:limits`

This decision does not authorize:

- approximate conservation;
- floating semantic identity;
- hidden solver tolerances;
- explicit matrix inversion for routine solves;
- normal equations without explicit conditioning review;
- floating-point rank as exact semantic rank;
- a numerical fit as the sole calibration authority;
- solver timeout as infeasibility;
- a heuristic fallback after exact-search exhaustion;
- native numerical dependencies without toolchain review.

## Verification · `gate:math:exact-certified`

The decision is implemented when:

- exact semantic code uses checked or arbitrary-precision arithmetic;
- numerical results affecting release have exact or conservative certification;
- no raw floating value enters semantic identity;
- no package adds unused numerical dependencies;
- every nontrivial exact algorithm has an independent small-instance oracle;
- MSRV and stable lanes pass;
- dependency and advisory review passes;
- all repository checks remain green and clean.