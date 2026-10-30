# Translation-Validation Vectors · `pkg:vectors:contract`

> **Status:** Candidate — compact-ASH fixtures, evidence plan and target execution, the live-transfer safety, minimality and resource studies, and the maturity run-of-record corpora, continuity, recovery, resource and refinement checks and five candidate-scope reports are delivered; one of 72 compact-ASH negative coverage rows is discharged and 71 remain outstanding
> **Phase:** [Phase 4](../phases/04-compact-ash.md) onward
> **Package:** `tripod-vectors`
> **Library:** `vectors`
> **Direct dependencies:** `architecture`, `compiler`, `linker`, `realization`, `tapscript`, `target-elements`, `target-elements-conformance`, `transaction`
> **Decision:** [D004](../decisions/004-translation-validation.md)

## Purpose · `sec:vectors:purpose`

`vectors` compares target-independent semantic expectations with concrete
target behavior for one exact linked bundle and ABI.

It owns:

- semantic fixtures;
- model-to-realization fact projection;
- semantic vectors;
- focused mutations;
- ABI-driven target materialization;
- target-native execution;
- accepted semantic projection comparison;
- relation activation and carrier coverage;
- representation safety and minimality reports;
- resource measurements;
- substrate reports;
- deterministic shrinking;
- independent-observer report comparison;
- canonical evidence schemas and identities.

It owns evidence, not semantics.

## Dependencies · `sec:vectors:dependencies`

Broad direct dependencies are permitted because this package compares assurance
boundaries.

No direct `tapscript` dependency is required when final programs and provenance
are exposed through `linker`.

Forbidden dependency:

```text
release
```

Release consumes vector reports, not the reverse.

## Typed inputs · `sec:vectors:inputs`

The package consumes:

- architecture and realization identities;
- executable model behavior;
- analyzed compiler relation/coverage scope;
- exact target;
- exact linked bundle;
- exact transaction ABI;
- explicit deterministic seeds and test fixtures;
- external observer reports through typed adapters.

## Typed outputs · `sec:vectors:outputs`

Separate reports cover:

- model/realization conformance;
- compiler-analysis completeness;
- backend pattern behavior;
- exact bundle target execution;
- relation coverage;
- representation safety;
- representation minimality;
- resource measurement;
- substrate claims;
- script integration;
- event projection;
- attestation query;
- receipt accounting.

Reports remain distinct even when one runner invokes several checks.

> Illustrative boundary; names and exact fields are not frozen.

```rust
pub fn execute_vectors(
	vectors: &TargetVectorRegistry,
	executor: &mut impl TargetExecutor,
) -> Result<TargetExecutionReport, VectorError>;
```

## Semantic fixtures · `rule:vectors:fixtures`

A positive fixture begins from a model-valid world produced through the model’s
declared valid path.

A corruption fixture is explicitly labeled as fault evidence.

Fixture identity binds:

- architecture/realization;
- operation;
- initial semantic state;
- request;
- order/checkpoint context;
- explicit fixture parameters.

It excludes target bytes and bundle identity.

## Expected result · `rule:vectors:expected`

Expected semantic behavior derives from:

- realization relations;
- executable model;
- typed expected projection adapters.

It must not derive from the candidate backend being tested.

Shared helpers are documented and are not described as independent evidence.

## Mutations · `rule:vectors:mutations`

A mutation records:

- source valid vector;
- intended relation;
- changed semantic facts or concrete fields;
- mutation layer;
- collateral relations;
- expected result;
- exact concrete transaction.

Mutation layers include:

- semantic fact;
- ABI/layout;
- target transaction;
- witness/proof;
- linked constructor/program.

Construction failure is not target-rejection evidence unless constructor
validation is the explicit claim.

## Target execution · `rule:vectors:execution`

Release-used execution runs against the selected target environment.

Reports distinguish:

- consensus result;
- policy/mempool result;
- package result;
- local diagnostic interpreter result;
- infrastructure failure.

Infrastructure failure never counts as expected rejection.

## Accepted projection · `rule:vectors:projection`

For accepted transactions, compare applicable:

- created object families;
- assets and public values;
- owners and classes;
- STATE/root successors;
- canonical deltas;
- open flows and fees;
- data outputs;
- event projections;
- disclosed facts;
- target-independent protocol observable.

Verdict equality alone is insufficient.

## Coverage · `rule:vectors:coverage`

Before execution, require equality among:

```text
realization relation scope
compiler relation scope
linked carrier scope
coverage requirement scope
```

Positive coverage requires active relation, reachable carrier, target
acceptance, and matching projection.

Target-negative coverage requires intended violation, target execution, and expected rejection; first-party refusal requires the exact owning validator to reject a retained malformed subject and accept its focused control, while missing entries remain explicit obligations in `packages/vectors/src/compact_negative_manifest.rs`.

Supplemental ABI requirements are indexed separately from compiler-derived coverage and use explicitly named test staging against the transaction-owned field validator; constructor pinning alone is not a malformed-input refusal.

Focused mutations list collateral relations honestly.

## Representation reports · `rule:vectors:representation`

Safety and minimality remain separate.

Safety vectors reject:

- wrong asset;
- confidential closed asset;
- wrong class;
- wrong recipient;
- value imbalance;
- missing authorization;
- malformed proof/opening;
- unclassified output.

Minimality vectors accept supported lower-disclosure representations with the
same semantic projection.

## Permissionless evidence · `rule:vectors:permissionless`

A permissionless construction vector runs from a public construction view plus
sponsor-local data.

It must not access hidden owner/operator fixture state.

## Resources · `rule:vectors:resources`

For each measured transaction, report:

- target/bundle/ABI;
- operation and activation case;
- family counts;
- representation;
- predicted resources;
- observed resources;
- target and policy verdict;
- measurement tool identity.

One fixture may maximize only one resource dimension.

## Independent observers · `rule:vectors:observers`

The package validates separate external candidate reports for:

1. raw events;
2. canonical query;
3. receipt accounting.

A model wrapper or second invocation is a harness test double, not independent
deployment evidence.

Each candidate declares implementation identity and shared dependencies.

## Shrinking · `rule:vectors:shrinking`

Unexpected failures are minimized deterministically when practical.

The report preserves:

- original failure;
- shrink policy and seed;
- minimized regression;
- relation and mutation provenance.

## Identity · `rule:vectors:identity`

Vector-set and report identities bind all applicable:

- architecture;
- realization;
- compiler;
- target;
- bundle;
- ABI;
- coverage policy;
- fixture/mutation set;
- tool version;
- explicit seeds.

They exclude timing, temporary paths, host identity, and secrets.

Archive candidate identity binds deployment and executed program commitments separately from optional source revision; matching source provenance alone establishes no current acceptance.

## Assurance boundary · `sec:vectors:assurance`

The package establishes finite bundle-specific translation evidence.

An earlier candidate's capture has typed historical standing for that candidate; retained producing programs support exact replay, and unavailable current-candidate replay is explicit, without promoting archived acceptance to changed bytes.

The compact archive admits once through `compact_corpus::run_of_record` with current-candidate standing bound to `FIXTURE_MERKLE_ROOT`, `FIXTURE_REFERENCE_OUTPUT_KEY` and `PINNED_PROGRAM` under ruling 128 (`rule:phase7:wave0-rulings`), replaying the same schedule that produced the capture and crediting a row only after its paired control and an exact boundary match. A second archive of the versioned second schedule admits the same way through `compact_corpus::second_run_of_record` under its own pins and the same candidate standing, and `compact_corpus::joined_campaign` requires equal keys and pins across both, so a row observed in either archive counts once and a key neither observes keeps only a tested formal closure. The admitted dispositions bind the record's table (`tab:backlog:compact-dispositions`) through `record_dispositions_bind_exactly_to_the_current_manifest`; changed candidate bytes reopen admission rather than inherit its credit.

**Table (Sponsored maturity archive standing)** · `tab:vectors:sponsored-maturity-archive`

| Archive | Admission | Fee policy and standing |
|---|---|---|
| `packages/vectors/fixtures/maturity-sponsored-run-of-record/` | Four immutable files; `MATURITY_SPONSORED_MANIFEST_SHA256` and `MATURITY_SPONSORED_RUN_ADDRESS` | Relay and mempool floors `0.00000100 BTC/kvB` reported by the node; block default `0.00000100 BTC/kvB` declared by the pinned source, no override. Replay: 69 operations, 35 fundings, 17 signings, 17 submissions, 12 accepted and five refused; zero-valued funding separately refused. Fees 128 and 141 units; exact routes and readback at (`rec:backlog:wave0-sponsored-observation`). |
| `packages/vectors/fixtures/maturity-native-run-of-record/` | Four immutable files; `MATURITY_NATIVE_MANIFEST_SHA256` and `MATURITY_NATIVE_RUN_ADDRESS` | Default relay and mempool floors `0.00000100 BTC/kvB` reported by the node; block floor `0.00000100 BTC/kvB` declared by the pinned source without override. Exact replay: 137 operations, 63 fundings, 27 signings, 47 submissions, 21 accepted and 26 refused; row bindings and phase durations at (`rec:backlog:wave0-native-observation`). |

It does not prove universal compiler correctness, cryptographic hardness,
target implementation correctness, or final deployment release.

## Exit gate · `gate:vectors:compact-ash`

The first bundle report exits when:

- all compact-ASH relation censuses agree;
- every relation has reachable positive and negative coverage;
- valid ABI transactions execute successfully;
- focused mutations reject at the intended evidence layer;
- accepted target projection matches the model;
- permissionless construction uses public data only;
- predicted and observed resources agree;
- reports are deterministic, identity-bound, and secret-free.

## Error vocabulary · `sec:vectors:errors`

See [`errors/vectors.md`](errors/vectors.md).

## Open questions · `sec:vectors:open`

- Where do shared evidence-envelope types live?
- Which exact target runner and RPC adapter are used?
- How are public model fixtures exposed without test-only internals?
- What low-level mutation API remains inaccessible to production callers?
- What coverage policy governs duplicated carriers and collateral mutations?
- How are external observer tools invoked and their independence declared?
- Which canonical binary vectors are committed versus generated for release?
