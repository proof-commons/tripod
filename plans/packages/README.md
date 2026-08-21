# Package Contracts

This directory defines planned package ownership, typed inputs and outputs,
dependency direction, assurance boundaries, and exit gates.

## Dependency spine · `rule:packages:spine`

```text
architecture
  -> realization
  -> compiler
  -> target/backend
  -> linker
  -> transaction
  -> vectors
  -> release
```

Evidence and final assembly packages may depend broadly. Semantic packages must
remain narrow.

## Direct-dependency plan · `tab:packages:dependencies`

| Package | Planned/current first-party direct dependencies |
|---|---|
| `realization` | `architecture` |
| `model` | `architecture`; `realization` for conformance projection |
| `labels` | `architecture` |
| `compiler` | `realization`; `architecture` when public APIs name architecture-owned IDs |
| `target-elements` | none |
| `tapscript` | `compiler`, `target-elements` |
| `simplicity` | none while parked |
| `linker` | `tapscript`, `target-elements` |
| `transaction` | `linker`, `target-elements` |
| `vectors` | `architecture`, `realization`, `model`, `compiler`, `target-elements`, `linker`, `transaction` |
| `release` | `architecture`, `realization`, `compiler`, `target-elements`, `linker`, `transaction`, `vectors`, `artifacts` |

This is planning ownership, not dependency configuration. Once crates exist,
Cargo metadata and dependency review establish actual dependency conformance.
The model's runtime transition acceptance remains independent of realization;
its realization dependency is for post-execution conformance projection.

## Index · `tab:packages:index`

| Package | Status | Direct role |
|---|---|---|
| [realization.md](realization.md) | Active | Target-independent semantic declaration. |
| [labels.md](labels.md) | Active | Repository-wide documentation label registries and checks. |
| [compiler.md](compiler.md) | Active — complete scoped analyzed pilots implemented internally; the public target-requirement boundary is the only public analysis surface | Relation, proof, disclosure, lifecycle, placement, and layout analysis. |
| [target-elements.md](target-elements.md) | Active — reviewed contract with welds, status closure, and development target-native primitive evidence; production support not claimed | Typed Liquid/Elements compatibility contract. |
| [tapscript.md](tapscript.md) | Active — static adapter, typed instruction core, and abstract stack validator implemented; backend patterns absent | First production target backend. |
| target-elements-conformance | Active — native fixture census, secretless executor protocol, and gate command implemented; contract prose lives in the package README | Development target-native primitive evidence harness. |
| [simplicity.md](simplicity.md) | Parked | Future second backend. |
| [linker.md](linker.md) | Candidate | Constructor, relocation, and bundle resolution. |
| [transaction.md](transaction.md) | Candidate — compact-ASH ABI, first-party explicit encoder/decoder, and construction pipeline implemented; sighash, blinding, and issuance delegated | Canonical transaction and witness ABI. |
| [vectors.md](vectors.md) | Candidate — section-18 matrix, canonical/experimental subject separation, semantic fixtures, target materialization, the evidence plan, live target execution, focused mutations, and the resource study implemented; coverage stands at 100 of 211 | Translation-validation evidence. |
| [release.md](release.md) | Planned | Final evidence/profile/publication gate. |
| [errors/](errors/README.md) | Active | Illustrative typed error vocabularies for planned package boundaries. |

## Contract form · `rule:packages:form`

Each package contract states purpose, allowed direct dependencies, forbidden
dependencies and inputs, typed inputs, typed outputs, identity ownership,
assurance boundary, determinism requirements, milestones, exit gate, and
unresolved research.

Cross-cutting rationale is cited from decisions rather than repeated.

## Dependency rule · `rule:packages:dependencies`

Package plans list expected minimal direct dependencies. A package directly
depends on the package owning every public type it names; it must not hide real
type ownership through transitive re-exports. A new shared package is introduced
only when at least two concrete consumers demonstrate one stable common
abstraction. Planning convenience alone is insufficient.

## Generated direction · `rule:packages:generated`

Generated publications flow out of typed packages and never back into
first-party semantics. This follows (`[ADR011-rule:toolchain:generated]`).

## Machine use · `rem:packages:machine-use`

Package-contract Markdown is not a package manifest, schema, or compiler input.