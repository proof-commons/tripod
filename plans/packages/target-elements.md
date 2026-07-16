# Elements Target Contract · `pkg:target-elements:contract`

> **Status:** Planned
> **Phase:** [Phase 3](../phases/03-target-foundation.md)
> **Package:** `tripod-target-elements`
> **Library:** `target_elements`
> **Decisions:** [D001](../decisions/001-typed-rust-source.md),
> [D003](../decisions/003-tapscript-first.md),
> [D004](../decisions/004-translation-validation.md),
> [D005](../decisions/005-value-representation.md)
> **Policy:** (`[ADR011-rule:toolchain:target-compatibility]`)

## Purpose · `sec:target-elements:purpose`

`target-elements` defines the typed compatibility contract consumed by the
Elements backend, transaction builder, vectors, and release tooling.

It owns target facts such as:

- tapscript execution domain and leaf version;
- opcode discriminants used by the project;
- exact stack contracts;
- failure behavior;
- transaction introspection shapes;
- asset/value encodings;
- byte-order rules;
- sighash capabilities;
- relative timelock behavior;
- confidential transaction capabilities;
- issuance/reissuance interfaces;
- consensus and policy resource interfaces;
- target evidence requirements.

It owns no attestation-contract operation semantics.

## Dependencies · `sec:target-elements:dependencies`

The package has no required first-party protocol dependency.

It may use narrowly reviewed generic dependencies for:

- typed target transaction structures;
- serialization;
- hashing;
- errors.

Forbidden dependencies:

```text
architecture
realization
model
compiler
tapscript
linker
transaction
vectors
release
artifacts
```

A downstream adapter maps compiler capability requirements to this package
without introducing a cycle.

## Typed inputs · `sec:target-elements:inputs`

The target contract is authored as typed Rust from reviewed deployed
capabilities.

Review material may include upstream source and node behavior, but no
implementation source revision becomes protocol or target identity.

A deployment instance separately supplies:

- network ID;
- genesis ID;
- network flavor;
- activation/configuration evidence binding.

## Typed outputs · `sec:target-elements:outputs`

The package exposes:

1. `TargetDefinition`
   - compatibility-contract identity;
   - opcodes, encodings, execution domain, limits, and capabilities.

2. `DeploymentInstance`
   - target-definition binding;
   - network/genesis identity;
   - activation/configuration binding.

3. `ElementsTarget`
   - validated combination used by backend and release tooling.

Exact type names remain implementation-owned.

## Identity · `rule:target-elements:identity`

The target-definition identity binds the typed compatibility contract:

- schema;
- execution domain;
- leaf version;
- relied-upon opcode semantics;
- stack contracts;
- encodings;
- sighash/timelock interfaces;
- CT and issuance interfaces;
- resource interfaces;
- evidence requirement registry.

It excludes:

- implementation source revision;
- node binary version;
- local source path;
- RPC endpoint;
- test date;
- host configuration.

Node version/build/configuration used by integration tests is report
provenance.

A change may stale target evidence without changing target-definition identity
when the typed contract is unchanged.

## Capability status · `rule:target-elements:capabilities`

Capability status is not a single boolean.

The typed contract distinguishes:

- unsupported;
- primitive source/deployment support;
- conditional support;
- approved complete proof pattern.

A low-level EC or hash primitive does not establish an authenticated-opening or
constructor proof by itself.

## Opcode contract · `rule:target-elements:opcodes`

Every backend-used opcode defines:

- numeric code;
- typed name;
- execution domain;
- operand order and encoding;
- result order and encoding;
- success and failure stack effects;
- malformed-input behavior;
- resource cost;
- review provenance.

Backend code consumes these typed declarations rather than duplicating raw
constants.

## Encodings · `rule:target-elements:encodings`

The package models asset and value representation independently.

It defines exact classes for:

- explicit asset;
- confidential asset;
- explicit value;
- confidential value;
- null values/nonces where applicable.

Byte order is field-specific.

Unknown encodings fail closed.

## Sighash and timelocks · `rule:target-elements:authorization`

The target contract exposes dimensions required to select authorization:

- output commitment;
- input commitment/extension behavior;
- issuance commitment;
- script-path behavior;
- transaction field commitment.

It also defines relative-timelock semantics and prerequisites.

It does not choose the attestation contract's signature or cadence policy.

## Confidential transactions · `rule:target-elements:ct`

The package defines target capabilities for:

- CT value conservation;
- commitment equality;
- explicit value introspection;
- issuance/reissuance;
- authenticated opening only after a complete tested proof pattern exists.

Closed-asset confidentiality remains prohibited by D005’s initial deployment
policy, which is enforced downstream.

## Resources · `rule:target-elements:resources`

Consensus and policy limits are separate typed values.

The target contract supplies measurement rules for:

- transaction weight;
- witness bytes;
- stack and altstack;
- element size;
- crypto budget;
- script execution;
- policy/standardness;
- package behavior where applicable.

It does not choose protocol batch bounds.

## Evidence registry · `rule:target-elements:evidence`

Every relied-upon capability maps to one or more target evidence requirements.

The static contract states:

```text
evidence required
```

A deployment report states:

```text
evidence verified
```

Mutable test status never changes the target definition.

## Assurance boundary · `sec:target-elements:assurance`

This package establishes a complete typed target contract and deployment
binding.

It does not establish:

- backend pattern correctness;
- target node correctness;
- production activation;
- resource feasibility of a protocol operation;
- deployment evidence completion.

## Milestones · `tbl:target-elements:milestones`

| Label | Deliverable |
|---|---|
| `milestone:target-elements:review` | Capability review and provenance |
| `milestone:target-elements:opcodes` | Typed opcode registry |
| `milestone:target-elements:encodings` | Encoding and byte-order rules |
| `milestone:target-elements:authorization` | Sighash and timelocks |
| `milestone:target-elements:ct` | CT and issuance capabilities |
| `milestone:target-elements:resources` | Consensus/policy limits |
| `milestone:target-elements:evidence` | Evidence requirement registry |
| `milestone:target-elements:regtest` | Validated development instance |
| `milestone:target-elements:identity` | Deterministic identities |

## Exit gate · `gate:target-elements:exit`

Phase 3 target work exits when:

- every backend-used primitive is typed and tested;
- development and production instances are distinct;
- network/genesis binding validates;
- capability dependencies close;
- evidence requirements are complete;
- implementation source revisions remain test provenance only;
- target-native tests cover every capability used by the first backend;
- no protocol operation policy appears in the package;
- output is deterministic and workspace checks remain clean.

## Error vocabulary · `sec:target-elements:errors`

See [`errors/target-elements.md`](errors/target-elements.md).

## Open questions · `sec:target-elements:open`

- Which Rust Elements library/version provides transaction and encoding types?
- Where does the compiler-target capability adapter live?
- Are consensus and policy separate sub-identities?
- What exactly is the initial-witness push policy scope?
- Which sighash profile is selected?
- How is production activation evidenced separately from regtest behavior?
