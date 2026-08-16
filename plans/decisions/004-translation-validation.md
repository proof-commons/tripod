# D004: Validate Every Released Bundle · `dec:assurance:translation-validation`

> **Status:** Accepted
> **Class:** Assurance
> **Depends on:** (`dec:architecture:realization-layer`),
> (`dec:backend:tapscript-first`)
> **Imports:** (`[RZ-rem:oracle:thesis]`),
> (`[RZ-rule:translation:certificate-leaf]`),
> (`[RZ-sec:oracle:boundary]`)
> **Supersedes:** none

## Choice · `rule:translation-validation:choice`

Do not initially claim a formally verified compiler.

Accept each deployment candidate through evidence bound to the exact emitted
and linked bundle.

```text
typed realization
    ↓
candidate compiler/backend/linker output
    ↓
exact transaction ABI and target transactions
    ↓
relation-indexed execution evidence
    +
target dependency evidence
    +
independent observer evidence
    ↓
deployment release
```

Compiler source review and unit tests remain required. They are not sufficient
release evidence.

## Relation coverage · `rule:translation-validation:relations`

Each target-independent semantic relation has:

- deterministic relation ID;
- source provenance;
- selected target proof;
- at least one executable carrier;
- activated accepting coverage;
- focused rejecting coverage;
- expected target verdict;
- observed target verdict;
- accepted semantic projection comparison.

Conditional relations additionally require:

- inactive valid case;
- active valid case;
- active invalid case.

A broad operation test does not substitute for relation coverage.

## Evidence boundaries · `tab:translation-validation:evidence`

| Evidence | Subject |
|---|---|
| architecture tests | finite declarations and identities |
| realization/model conformance | abstract semantic relation |
| compiler tests | analysis completeness |
| backend-pattern tests | reusable target proof pattern |
| bundle vectors | exact emitted and linked artifact |
| transaction tests | concrete ABI and witness construction |
| target reports | selected substrate claims |
| event report | raw public event recognition |
| query report | canonical attestation computation |
| accounting report | residue and receipt-accounting projection |
| deployment profile | final cross-binding |

Passing one row does not silently satisfy another.

## Mutation rule · `rule:translation-validation:mutation`

A rejecting mutation records:

- valid source vector;
- intended violated relation;
- changed facts;
- other relations affected;
- concrete target transaction;
- target carrier reached;
- actual target result.

When strict independence is not established, the report says:

```text
focused mutation with collateral relations
```

Construction failure is not counted as target rejection unless the test claims
constructor-level validation.

## Identity binding · `rule:translation-validation:identity`

Every release report binds the applicable:

- architecture;
- realization;
- compiler configuration;
- target;
- linked bundle;
- transaction ABI;
- vector set;
- report schema and tool identity.

A report for changed bytes, target, ABI, bounds, or vector policy is stale
unless its typed scope explicitly excludes that change.

## Independent observers · `rule:translation-validation:observers`

Deployment requires separate reports for:

1. attestation event projection;
2. attestation query;
3. receipt accounting.

Another invocation, wrapper, or checkpoint rebuild of the model’s reference
implementation is a differential-harness test, not independent deployment
evidence.

The independence boundary must be documented.

## Consequences · `sec:translation-validation:consequences`

- Evidence is developed operation by operation.
- Every compiler relation remains visible through target emission and linking.
- Missing carrier or coverage is a compilation/evidence failure.
- Target-native execution is required for release-used target claims.
- Reports are deterministic and nonsecret.
- Missing, failed, incomplete, unsupported, or stale required evidence blocks
  release.
- Formal proofs may strengthen individual patterns without erasing unrelated
  deployment boundaries.

## Does not authorize · `sec:translation-validation:limits`

This decision does not authorize:

- claiming finite vectors are universal proof;
- generating expected and actual results through one candidate path;
- accepting test doubles as independent implementations;
- replacing target execution with mocks;
- one opaque “all tests passed” report;
- release waivers;
- reusing evidence across changed identities without a typed reuse rule;
- changing expected results to match an unexpected target acceptance.

## Supersession · `rule:translation-validation:supersession`

A formally verified translation chain may replace part of this evidence model
only when its exact theorem scope covers the corresponding compiler, backend,
linker, transaction, and target semantics.

Uncovered deployment and independent-observer evidence remains required.

## Verification · `gate:translation-validation:release`

The decision is implemented when a complete linked pilot bundle has no missing
relation carrier, positive vector, negative vector, identity binding, target
result, or semantic projection comparison.
