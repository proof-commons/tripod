# Research Question: Exact and Numerical Linear Algebra · `q:numerical:linear-algebra`

> **Status:** Dependency review and prototype required
> **Blocks:** release-relevant numerical solver policy, conditioning policy,
> exact certification, and any compiler/calibration analysis based on matrices
> **Does not block:** exact integer/rational semantic relations that do not use
> numerical solving
> **Affected packages:** compiler, linker where justified, vectors, release,
> future calibration tooling
> **Depends on:** (`dec:source:typed-rust`),
> (`dec:assurance:translation-validation`)
> **Imports:** (`[ADR011-rule:toolchain:dependencies]`),
> (`[ADR011-rule:toolchain:reproducibility]`),
> (`[RZ-sec:arithmetic:gadgets]`),
> (`[RZ-pin:pins:arith]`)
> **Related research:** (`q:compiler:algorithms`),
> (`q:optimization:solvers`),
> (`q:arithmetic:wide-floor`)
> **Expected handoff:** reviewed `faer` and `num-rational` dependency policy,
> exact keyed-matrix types, numerical acceptance criteria, exact certification,
> and reproducible solver reports

## Question · `sec:numerical:question`

How should the compiler-era toolchain represent, solve, diagnose, and certify
linear systems without allowing approximate floating-point analysis to become
an accidental proof of exact protocol, compiler, linker, or release semantics?

The preferred numerical linear-algebra substrate is `faer`.

The exact arithmetic substrate is:

```text
checked bounded integers
num-bigint
num-rational
num-integer
num-traits
```

The fundamental rule is:

> exact semantics define what is true; numerical linear algebra produces
> candidates, diagnostics, sensitivity analysis, and certified approximate
> results.

Raw floating-point output is never sufficient evidence for:

- semantic equality;
- conservation;
- authorization;
- object closure;
- stable identity;
- exact calibrated bound;
- release validity.

## Problem classification · `rule:numerical:classification`

Every proposed “linear solver” use must identify one of four classes.

### Numerical linear equations

\[
Ax=b,\qquad A,b,x\in\mathbb R
\]

Appropriate for:

- approximate analysis;
- sensitivity;
- measured resource-model diagnostics;
- well-conditioned least squares;
- numerical research.

`faer` is appropriate.

### Exact linear equations

\[
Ax=b,\qquad A,b,x\in\mathbb Q
\]

Appropriate for:

- exact affine semantic relations;
- exact rank and consistency;
- conservation identities;
- exact certificate verification;
- equality of canonical resource formulas.

Use exact integer/rational arithmetic.

### Linear or convex optimization

\[
\min c^\top x\quad\text{subject to linear or convex constraints}
\]

This requires an optimizer, not merely a linear-system solver.

### Integer or Boolean optimization

Variables may satisfy:

\[
x_i\in\{0,1\}\quad\text{or}\quad x_i\in\mathbb Z
\]

This includes proof selection, placement, optional program selection, and
integer calibration. It belongs to (`q:optimization:solvers`).

## Dependency posture · `tab:numerical:dependencies`

| Dependency | Status | Role |
|---|---|---|
| `faer` | preferred candidate | dense/sparse numerical linear algebra |
| `num-bigint` | existing | exact arbitrary-size integers |
| `num-rational` | recommended | exact rational coefficients and results |
| `num-integer` | existing | gcd and integer helpers |
| `num-traits` | existing | checked generic numeric support |
| `proptest` | existing | generated matrix and oracle tests |
| `rug` | optional research oracle | MPFR/GMP high-precision comparison |
| `nalgebra` | not initially required | small static matrices/geometric algebra |
| `ndarray` | not initially required | N-dimensional array ecosystem |
| `sprs` | not initially required | separate sparse matrix ecosystem |
| BLAS/LAPACK binding | not initially required | native numerical backend |

Adopt one numerical matrix ecosystem initially: `faer`.

Do not add overlapping matrix libraries without a concrete unmet requirement.

## Exact keyed systems · `rule:numerical:exact-source`

The authoritative linear system is a first-party typed value independent of
`faer`.

Conceptually:

```rust
struct ExactLinearSystem<RowId, ColumnId> {
    rows: Vec<RowId>,
    columns: Vec<ColumnId>,
    coefficients: ExactMatrix,
    right_hand_side: Vec<BigRational>,
}
```

Rows and columns are sorted by stable first-party keys.

The system rejects:

- duplicate row keys;
- duplicate column keys;
- unknown variables;
- dimension mismatch;
- noncanonical rational values;
- unsupported empty or underdetermined forms where the caller requires a
  unique solution.

Matrix row number and column number are local handles, never semantic identity.

## Exact arithmetic · `rule:numerical:exact-arithmetic`

Use reduced `BigRational` values for exact coefficients and results.

For small exact systems, the preferred production/reference algorithm is
fraction-free Bareiss elimination over `BigInt`.

For rational input:

1. reduce every rational;
2. clear denominators under a canonical policy;
3. obtain an integer augmented matrix;
4. perform fraction-free elimination;
5. determine exact rank and consistency;
6. recover the reduced rational result;
7. verify by exact substitution.

A simpler `BigRational` Gaussian-elimination implementation may remain as a
small-instance test oracle.

Exact elimination is expected to be \(O(n^3)\). The matrices under initial
semantic analysis are small enough that exactness is preferred over a more
complex probabilistic or modular solver.

## Numerical lowering · `rule:numerical:lowering`

Numerical analysis lowers the exact keyed system into a separate `faer` working
value.

The lowering records:

- row and column key order;
- exact-to-`f64` conversion;
- scaling policy;
- solver method;
- tolerance policy;
- parallelism policy;
- requested rank/conditioning behavior.

The numerical matrix is derivative working state.

It is not:

- semantic source;
- canonical publication;
- stable identity;
- exact certificate.

Every exact coefficient converted to floating point remains available for
independent residual and certification checks.

## Method selection · `rule:numerical:methods`

Select the numerical method by typed problem class.

| Problem | Preferred method |
|---|---|
| general square system | pivoted LU |
| symmetric positive definite | Cholesky after premise validation |
| symmetric indefinite | reviewed pivoted LDLᵀ where supported |
| overdetermined least squares | QR |
| rank-deficient/ill-conditioned least squares | SVD |
| repeated right-hand sides | factor once, solve repeatedly |
| large sparse system | separately reviewed sparse method |
| exact small semantic system | exact Bareiss, not `faer` |

Do not explicitly compute \(A^{-1}\) to solve \(Ax=b\).

Do not form normal equations \(A^\top Ax=A^\top b\) for ordinary least squares
unless the squared-conditioning consequence is explicitly accepted.

Use QR or SVD.

No default method hidden inside a convenience call may determine
release-relevant behavior.

## Scaling · `rule:numerical:scaling`

Scaling policy is explicit.

Candidates include:

- none;
- row equilibration;
- column equilibration;
- row and column equilibration;
- domain-specific exact scaling derived from units.

Scaling must not mix unlike semantic units without an explicit dimensionless
normalization.

The result report states both scaled and unscaled residual checks where
applicable.

## Numerical acceptance · `rule:numerical:acceptance`

A numerical solve validates at least the normwise backward error:

\[
\eta=\frac{\lVert b-Ax\rVert_\infty}{\lVert A\rVert_\infty\lVert x\rVert_\infty+\lVert b\rVert_\infty}
\]

Also validate:

- all inputs finite;
- all outputs finite;
- no NaN;
- no infinity;
- dimensions;
- decomposition success;
- rank policy;
- condition policy;
- residual independently recomputed;
- tolerance supplied by explicit typed policy.

For poorly scaled problems, add componentwise checks.

A small residual alone does not imply a small forward error when \(A\) is
ill-conditioned.

The result therefore records a typed status such as:

```text
certified for declared numerical use
diagnostic only
ill-conditioned
rank-deficient
nonfinite
decomposition failed
exact certification failed
```

Release-sensitive callers reject ambiguous rank, severe ill-conditioning, and
tolerance-sensitive plan changes.

## Conditioning and rank · `rule:numerical:conditioning`

Rank tolerance and condition limits are explicit policy inputs.

They are not inherited from an undocumented solver default.

For each use, state whether the result requires:

- full column rank;
- full row rank;
- unique solution;
- minimum-norm solution;
- rank-revealing decomposition;
- diagnostic singular values only.

A numerical rank decision must not determine exact semantic equivalence.

If exact rank matters, compute it over `BigRational`/`BigInt`.

## Certification modes · `rule:numerical:certification`

A numerical result enters one of three modes.

### Exact semantic certification

The numerical result suggests a candidate.

The candidate is transformed into an exact value by:

- declared integer quantization;
- rational reconstruction;
- exact known-domain projection;
- another explicit conversion.

Then exact arithmetic verifies:

\[
Ax=b
\]

and every domain constraint.

### Conservative bound certification

A numerical estimate is converted into a safe bound using:

- explicit uncertainty policy;
- conservative floor/ceiling direction;
- exact target limit;
- complete observed measurements.

The accepted bound is an exact integer or rational.

### Diagnostic only

The result may guide design but does not affect:

- semantic acceptance;
- identity;
- selected final bound;
- release validity.

The report labels the mode explicitly.

## Determinism · `rule:numerical:determinism`

Initial `faer` use is sequential.

Disable optional parallel execution unless a later decision proves:

- deterministic accepted result;
- canonical diagnostics;
- thread-count-independent release behavior;
- measured need.

Raw floating-point bits must not enter:

- semantic IDs;
- compiler analysis identity;
- linked bundle identity;
- ABI identity;
- deployment profile;
- release identity.

Numerical reports may include floating diagnostics only under an explicit
canonical reporting policy.

Identity-bearing results are exact integers, reduced rationals, canonical
finite selections, or conservatively quantized values.

Pivot order, iteration count, singular-value low bits, and host-specific SIMD
behavior are not semantic identity.

## Sparse analysis · `rule:numerical:sparse`

Do not use sparse algorithms merely because the semantic graph is sparse.

Sparse linear algebra is accepted only when matrix dimensions and measured cost
justify it.

Before adopting a sparse method, review:

- symbolic ordering;
- permutation determinism;
- pivoting;
- singularity detection;
- fill-in;
- tolerance/iteration policy;
- parallelism;
- recursion and stack behavior;
- sparse feature dependencies;
- reproducibility across targets.

Small exact or dense numerical systems remain preferred where simpler.

## Compiler use · `rule:numerical:compiler-use`

Appropriate compiler uses include:

- numerical sensitivity of candidate cost models;
- least-squares diagnostics over backend measurements;
- investigation of poorly conditioned parameterizations;
- candidate generation for exact affine systems.

Exact semantic relation dependence, rank, and conservation remain exact
integer/rational analyses.

`faer` is not used for:

- graph traversal;
- proof selection;
- placement;
- authorization;
- lifecycle;
- declassification;
- exact conservation proof;
- canonical ordering.

## Linker and calibration use · `rule:numerical:calibration-use`

Backend/linker code derives structural resource formulas first.

For a measured model:

\[
y=X\beta+\varepsilon
\]

`faer` may estimate \(\beta\) through QR or SVD and diagnose:

- missing terms;
- nonlinear behavior;
- incorrect fixtures;
- resource-formula drift;
- poorly conditioned measurements.

Final calibration still requires:

- structurally derived formulas;
- complete valid worst-case transactions;
- observed target measurements;
- conservative exact limit checks;
- final relink and remeasurement.

A fitted model is diagnostic evidence, not the only source of a deployment
bound.

## Dependency review · `sec:numerical:dependency-review`

Before selecting a concrete `faer` release, record:

- exact crates.io version;
- upstream repository and release tag;
- license;
- Rust 1.88 compatibility;
- exact feature set;
- transitive dependencies;
- unsafe and SIMD trust surface;
- sparse/dense APIs used;
- parallelism status;
- advisory status;
- canonical/reproducibility implications.

High-performance numerical libraries may use dependency-internal unsafe and
SIMD. First-party unsafe remains denied, but dependency unsafe is a reviewed
trust surface under ADR-011.

Prefer:

```text
default features disabled
minimum required features enabled
no parallelism initially
no unused ecosystem adapters
```

The exact feature names come from the selected release.

## Prototype · `sec:numerical:prototype`

### Stage 1 — exact keyed matrix

Implement canonical row/column keys, exact coefficients, dimension validation,
and exact substitution.

### Stage 2 — exact solver

Implement Bareiss rank, consistency, determinant where useful, and square
solve.

Compare with a simple `BigRational` elimination oracle.

### Stage 3 — `faer` wrapper

Implement explicit LU, QR, SVD, and Cholesky policies using one reviewed
release.

### Stage 4 — residual and conditioning reports

Independently recompute residuals and classify rank/conditioning.

### Stage 5 — exact/numerical cross-check

Generate small exact systems, solve numerically, and compare with exact
solutions.

### Stage 6 — resource-model fixture

Fit and diagnose one synthetic and one measured resource formula.

Prove that release acceptance still uses structural formulas and exact
conservative bounds.

### Stage 7 — sparse threshold

Measure dense versus sparse only if a real matrix size justifies the comparison.

## Required vectors · `sec:numerical:vectors`

- zero-sized and dimension mismatch;
- identity matrix;
- diagonal scaling extremes;
- full-rank integer system;
- singular system;
- inconsistent system;
- overdetermined consistent/inconsistent systems;
- underdetermined system;
- nearly dependent rows;
- symmetric positive-definite matrix;
- symmetric but non-SPD matrix;
- zero pivot requiring pivoting;
- very poor conditioning;
- nonfinite input;
- exact rational solution not exactly representable in `f64`;
- row permutation;
- column permutation and inverse result permutation;
- redundant equation;
- equation multiplied by nonzero exact scalar;
- exact coefficient mutation;
- parallel/thread-count permutation if parallelism is ever enabled;
- attempted raw float insertion into identity-bearing DTO.

## Measurements · `sec:numerical:measurements`

Record:

- matrix dimensions and density;
- exact coefficient bit lengths;
- exact solver time and intermediate growth;
- numerical decomposition time;
- solve time;
- residual;
- backward error;
- condition estimate;
- rank;
- memory;
- dense/sparse comparison where applicable;
- exact certification time;
- cross-platform or target variation;
- effect of sequential versus optional parallel execution.

## Acceptance · `gate:numerical:accept`

Accept the mathematical stack when:

- `faer` release and features pass dependency review;
- Rust 1.88 and stable lanes pass;
- exact keyed systems are independent of numerical storage;
- exact semantic equations use exact arithmetic;
- Bareiss agrees with the independent rational oracle;
- numerical methods are selected by explicit problem class;
- no explicit matrix inverse is used for solving;
- least squares uses QR/SVD rather than accidental normal equations;
- residual and conditioning policies are explicit;
- ill-conditioned and rank-deficient results cannot be falsely certified;
- release-sensitive results receive exact or conservative certification;
- raw floating-point output is excluded from semantic identity;
- repeated canonical analysis is deterministic;
- numerical reports are secret-free and reproducible under their stated scope.

## Rejection · `gate:numerical:reject`

Reject a dependency or solver policy if it:

- treats a small residual as exact proof;
- uses floating-point rank as semantic rank;
- hides tolerances in library defaults;
- computes inverse matrices for routine solves;
- uses normal equations without explicit conditioning analysis;
- accepts NaN or infinity;
- makes thread count or SIMD low bits identity-bearing;
- serializes native matrix storage as a canonical publication;
- fits a resource model and treats it as the only calibration authority;
- introduces native BLAS/LAPACK or high-precision dependencies without a
  separate toolchain review;
- cannot be cross-checked against exact small-system oracles.

## Result · `sec:numerical:result`

Pending.

## Handoff · `sec:numerical:handoff`

An accepted result updates:

- workspace dependency policy;
- compiler numerical-analysis module;
- exact coefficient and certificate types;
- resource-analysis reports;
- optimization solver certification rules;
- calibration evidence;
- release report schemas;
- test strategies;
- CI/MSRV dependency evidence.
