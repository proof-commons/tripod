# D007: Petgraph Is the Repository-Wide Graph Substrate · `dec:graph:petgraph`

> **Status:** Accepted
> **Class:** Cross-package implementation substrate
> **Depends on:** (`dec:source:typed-rust`)
> **Imports:** (`[ADR011-rule:toolchain:dependencies]`),
> (`[ADR011-rule:toolchain:reproducibility]`),
> (`[ADR013-rule:labels:global-resolution]`)
> **Supersedes:** every planned or provisional first-party graph container,
> graph wrapper, adjacency-map implementation, and alternate graph substrate

## Choice · `rule:graph:petgraph`

Use `petgraph` as the repository-wide graph data structure and graph algorithm
dependency.

Every first-party structure whose semantics are nodes connected by typed edges
uses a concrete `petgraph` graph type directly.

This includes, as applicable:

- documentation citation graphs;
- realization expression and relation graphs;
- compiler dependency, proof, constructibility, lifecycle, placement, and
  coverage graphs;
- linker symbol, constructor, relocation, reachability, carrier, and resource
  graphs;
- condensation DAGs;
- target-program trees and other graph-shaped target structures;
- later graph analyses added by first-party packages.

No first-party graph wrapper, generic graph adapter, adjacency-list container,
adjacency-map substitute, alternate graph library, or bespoke traversal
framework is introduced.

## Dependency · `rule:graph:dependency`

The selected release is:

```text
petgraph 0.8.3
```

It is workspace-owned and exact-version pinned.

Optional features are enabled only for a named current consumer; see
(`rule:graph:features`).

A package directly depending on Petgraph names the workspace dependency:

```toml
petgraph = { workspace = true }
```

Dependency resolution follows (`[ADR011-rule:toolchain:locked]`): the version range in the workspace manifest is the reviewable unit, and no lockfile pins it before v1.

## Direct use · `rule:graph:direct-use`

Graph-owning packages use concrete Petgraph types directly, for example:

```rust
petgraph::graph::DiGraph<Node, Edge, u32>
petgraph::graph::UnGraph<Node, Edge, u32>
petgraph::stable_graph::StableDiGraph<Node, Edge, u32>
petgraph::graphmap::DiGraphMap<Node, Edge>
```

according to the graph's actual mutation, identity, and lookup needs.

Do not introduce types such as:

```text
CanonicalGraph
FrozenGraph
SemanticGraph
GraphAdapter
GraphStore
ProjectGraph
```

that merely wrap or reproduce Petgraph's storage or traversal API.

Domain-specific values may contain Petgraph graphs and may add semantic
validation, but they do not reimplement or abstract the graph substrate.

## Semantic identity · `rule:graph:identity`

Petgraph indices are process-local handles.

The following are never semantic identity:

- `NodeIndex`;
- `EdgeIndex`;
- graph insertion position;
- traversal order;
- iterator order;
- SCC return order;
- solver or matrix position derived from a graph handle.

First-party typed stable keys remain the identity of semantic nodes and edges.

A graph-owning package maintains whatever direct key/index lookup is needed to
map between:

```text
typed stable key
↔
Petgraph local index
```

Such a lookup table is indexing metadata, not a second graph implementation.

## Canonical construction · `rule:graph:canonical-construction`

When graph construction influences diagnostics, deterministic evaluation,
publication, evidence, or identity:

1. collect and validate typed nodes;
2. sort nodes by first-party stable key;
3. insert them into the Petgraph graph in that order;
4. collect and validate typed edges;
5. sort edges by typed source key, target key, and edge key;
6. insert them in that order;
7. run Petgraph algorithms;
8. translate results back to typed stable keys;
9. normalize any mathematically unordered result explicitly.

Canonical ordering is a property of project inputs and results. It is not an
alternative graph container.

## Algorithms · `rule:graph:algorithms`

Use Petgraph algorithms and visitors wherever they provide the required
operation, including:

- topological sorting;
- cycle detection;
- strongly connected components;
- condensation;
- reachability;
- depth-first traversal;
- breadth-first traversal;
- path existence;
- graph transformation;
- graph generation for tests.

Do not implement first-party substitutes for an available Petgraph algorithm
merely to control incidental iteration order.

Where an algorithm returns a mathematically unordered result, normalize the
result by typed stable key after the Petgraph operation.

A specialized project algorithm may operate over a Petgraph graph when the
algorithm itself is project-specific, such as proof-plan search, placement,
package-merge, or typed carrier coverage. It must not reimplement graph storage
or ordinary graph traversal.

## Feature surface · `rule:graph:features`

An optional feature is enabled only for a named current consumer. A feature is
not carried to advertise intent, to anticipate a future need, or to keep one
maximal configuration for its own sake: an unused feature is unreviewed
dependency surface, and enabling it states a capability the repository does not
exercise.

The reviewed surface for this release is:

```text
serde-1     enabled — narrow reviewed exception, below
rayon       not enabled
dot_parser  not enabled
unstable    not enabled
generate    not enabled
```

`dot_parser` additionally pulled GPL-2.0-or-later crates into the first-party build graph, which ADR-011's dependency rule does not admit without a separate policy decision, and `rayon` added nondeterministic parallelism with no consumer. Neither is enabled, so neither trade-off is taken.

Feature availability never changes semantic authority. Enabling a feature
grants no license to serialize a Petgraph container as a canonical
publication, to emit nondeterministic identity-bearing output, to treat DOT as
a semantic input, or to treat an unstable API as a release contract.

### `serde-1` as a narrow reviewed exception

`serde-1` is retained ahead of a production consumer, deliberately and as the
single exception to the rule above. Its scope is fixed:

- noncanonical diagnostics and internal caches only;
- never a semantic, publication, or release input;
- guarded by an explicit test in the labels package, so removing the feature
  breaks a named test rather than silently changing behaviour;
- removed if no diagnostic serializer materializes and the guard test becomes
  its only justification.

Every other optional feature returns under the ordinary rule: a named consumer
first, then the feature.

## Serialization · `rule:graph:serialization`

Petgraph serialization may be used for tests, diagnostics, caches, or explicitly
noncanonical internal artifacts.

A semantic or release publication does not serialize a Petgraph container as
its authoritative schema.

Canonical publications render first-party typed values:

```text
stable node key
typed node value
stable source key
stable target key
typed edge value
```

under an owned schema and canonical ordering.

## Parallelism · `rule:graph:parallelism`

Parallel Petgraph facilities are not enabled. Enabling them requires a measured
need and evidence that identity-bearing results are unchanged; the conditions
below govern any such future use.

Any identity-bearing or canonical result must remain identical across:

- thread counts;
- task schedules;
- graph insertion permutations;
- repeated clean runs.

Parallel execution that changes a selected plan, diagnostic identity,
publication, bundle, ABI, vector, or release result is non-conforming.

## Existing code · `rule:graph:migration`

Existing first-party code described as a graph must use Petgraph directly.

In particular, the repository-wide documentation citation graph and the new
realization graphs are migrated before Phase 1 exits.

Typed registries, ordered histories, finite tables, maps, and sets remain their
native structures when their semantics are not graph storage. Calling a
collection a "registry," "table," "history," or "tree" does not evade this
decision when its actual semantics are a general node/edge graph.

## Package policy · `rule:graph:package-policy`

Petgraph is workspace-owned.

A package adds a direct dependency when it constructs, stores, traverses, or
publishes a graph.

Do not add unused direct dependencies to crates that own no graph.

No shared first-party graph crate is created.

## Assurance · `sec:graph:assurance`

Petgraph supplies graph storage and standard graph algorithms.

First-party code remains responsible for:

- typed node and edge semantics;
- stable semantic keys;
- source provenance;
- graph-specific validation;
- canonical insertion;
- canonical result normalization;
- complexity policy;
- interpretation of cycles and SCCs;
- proof, lifecycle, authorization, and coverage meaning;
- canonical publications.

Finding an SCC does not make a semantic cycle valid. Reachability does not prove
execution in every required case. Those remain package-owned semantic claims.

## Does not authorize · `sec:graph:limits`

This decision does not authorize:

- Petgraph indices as semantic IDs;
- traversal order as canonical order;
- direct graph serialization as a protocol or release schema;
- DOT or generated graph input as semantic source;
- an SCC as automatic acceptance of a dependency cycle;
- parallel scheduling as identity;
- replacing typed ASTs with untyped node strings;
- using one universal graph schema for unrelated semantic relations;
- weakening exact project-specific analyses into generic graph heuristics.

## Verification · `gate:graph:petgraph`

The decision is implemented when:

- `petgraph` is workspace-owned at the reviewed release, with optional
  features limited to those having a named current consumer;
- every graph-owning first-party package depends on it directly;
- no first-party graph wrapper or alternate graph container exists;
- realization expression and relation graphs use Petgraph directly;
- documentation citation resolution uses Petgraph directly;
- graph indices do not appear in semantic IDs or canonical publications;
- node and edge insertion permutations yield identical typed results;
- standard algorithms use Petgraph implementations;
- identity-bearing results remain stable across repeated and parallel runs;
- Rust 1.88 and stable workspace lanes pass;
- dependency, license, advisory, and lockfile review passes;
- all repository checks remain green and clean.