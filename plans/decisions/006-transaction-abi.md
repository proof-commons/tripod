# D006: Generate a Canonical Transaction and Witness ABI · `dec:abi:canonical-transactions`

> **Status:** Accepted
> **Class:** Backend ABI
> **Depends on:** (`dec:architecture:realization-layer`),
> (`dec:backend:tapscript-first`),
> (`dec:assurance:translation-validation`),
> (`dec:representation:value-parametric`)
> **Imports:** (`[RZ-rule:translation:collections]`),
> (`[RZ-rule:translation:cross-utxo]`),
> (`[RZ-rule:translation:certificate-leaf]`)
> **Supersedes:** none

## Choice · `rule:abi:choice`

Generate one deterministic target- and bundle-specific transaction/witness ABI
for every supported semantic operation.

The ABI derives from:

```text
architecture cardinalities
+
realization relations
+
compiler proof/placement/layout requirements
+
target capabilities
+
backend lowering
+
linked constructors and programs
+
calibrated deployment values
```

Clients, vectors, and calibration consume the same typed ABI.

They do not infer transaction shape from script bytes or planning prose.

## Semantic/layout split · `rule:abi:semantic-layout`

Architecture and realization define:

- allowed object families;
- cardinalities;
- semantic relations;
- authorization;
- state and observable effects.

The backend ABI defines:

- concrete input/output family order;
- fixed positions and bounded ranges;
- coordinator selection;
- optional-family encoding;
- sponsor region;
- data-output order;
- target program selection;
- witness item order;
- metadata encoding;
- target transaction constraints.

A positional layout is implementation evidence unless the upstream semantic
contract makes the position observable.

## Counts and ranges · `rule:abi:ranges`

Counts are untrusted proposals.

Target enforcement authenticates:

- transaction input/output totals;
- family minimum and calibrated maximum;
- family start and end;
- range disjointness;
- constructor in every protocol slot;
- complete protocol-family coverage;
- sponsor/protocol separation.

No output region may remain unclassified where it could carry a closed
protocol asset.

## Obligation placement · `rule:abi:placement`

Every target-enforced relation has at least one executable carrier.

The placement record identifies:

- semantic relation;
- selected proof;
- target program;
- input family or coordinator;
- authenticated facts;
- local or global scope;
- deliberate duplicate enforcement;
- target capabilities.

An unplaced relation is a compilation failure.

A relation placed only on an optional program that need not execute is not
enforced.

## Coordinator rule · `rule:abi:coordinator`

Repeated families may designate one canonical coordinator, normally the
lowest-index member of the authenticated family range.

Local programs may enforce:

- current object recognition;
- local class;
- local owner authorization;
- operation participation.

The coordinator may enforce:

- family census;
- global conservation;
- output closure;
- sponsor boundary;
- state assignment;
- event shape.

The exact split is target- and operation-specific.

## Witness ABI · `rule:abi:witness`

Each target program has a typed witness schema defining:

- item role;
- encoding;
- length and domain;
- public or secret status;
- witness availability;
- source relation;
- target program;
- canonical ordering.

Witness publications describe roles, not production secrets.

Permissionless operations may use only public, deployment, bundle, or
constructor-local sponsor facts.

## Package ownership · `tab:abi:ownership`

| Package | ABI responsibility |
|---|---|
| architecture | semantic families and bounds |
| realization | semantic relations and availability |
| compiler | placement and layout requirements |
| backend | concrete target lowering and preliminary witness roles |
| linker | final constructor/program/control resolution |
| transaction | final typed ABI and transaction construction |
| vectors | valid and malformed ABI-driven transactions |
| release | bundle/ABI/calibration identity validation |

## Calibration · `rule:abi:calibration`

Calibration uses complete valid ABI transactions.

It must not infer a deployment bound from isolated script size alone.

For each candidate bound:

1. link candidate bundle;
2. derive candidate ABI;
3. construct worst-case transactions;
4. execute and measure them;
5. choose the next candidate deterministically.

After final values are selected, relink and remeasure the exact final bundle
and ABI.

The orchestration lives above linker and transaction so the Cargo dependency
graph remains acyclic.

## Identity · `rule:abi:identity`

ABI identity binds:

- schema;
- target;
- linked bundle;
- calibrated bounds;
- operation layouts;
- constructor and metadata schemas;
- witness schemas;
- representation requirements;
- target transaction constraints.

A layout or witness change moves ABI and normally bundle/configuration
identity. It does not move architecture or realization identity when semantic
behavior is unchanged.

## Does not authorize · `sec:abi:limits`

This decision does not authorize:

- concrete transaction positions in architecture;
- target indexes in realization;
- wallets reverse-engineering scripts;
- handwritten independent builders;
- trusting witness counts without constructor checks;
- one universal ABI for all backends;
- zero-value placeholders for absent protocol objects;
- treating the model’s internal output order as target ABI automatically;
- complete global checks existing only in the transaction builder.

## Supersession · `rule:abi:supersession`

A future target may support flexible layouts only if it still provides:

- authenticated family membership;
- complete obligation placement;
- deterministic client construction;
- mutation/vector materialization;
- calibration fixtures;
- bundle/client compatibility identity.

## Verification · `gate:abi:first-operation`

The decision is demonstrated when `compact-ash` has one linked, typed,
deterministic ABI used unchanged by:

- transaction construction;
- target vectors;
- worst-case calibration;
- release identity checks.
