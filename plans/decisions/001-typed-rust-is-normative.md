# D001: Typed Rust Is Normative

> **Status:** ACCEPTED
> **Scope:** Source-of-truth and data-flow policy for all planned realization,
> compiler, backend, linker, transaction, evidence, and release packages
> **Decision class:** Source of truth
> **Applies to:** `architecture`, `realization`, `model`, `compiler`,
> `target-elements`, `tapscript`, `simplicity`, `linker`, `transaction`,
> `vectors`, `release`, and generated-artifact tooling
> **Depends on:** ADR-011; the existing typed-architecture and generated-artifact
> direction
> **Supersedes:** none
> **Superseded by:** none
> **Related normative constraints:** the register-authority rule, typed
> architecture authority, generated-appendix weld, and code-generation
> obligations in `docs/attestation/realization.md`
> **Related research:** none
> **Promoted ADR:** none
> **Machine-consumed by the toolchain:** no

---

## 1. Context

The repository already has a strong source-of-truth direction.

The typed architecture is declared in Rust:

```text
packages/architecture/src/spec.rs
packages/architecture/src/ids.rs
packages/architecture/src/validate.rs
```

Derivative publications are generated from that declaration:

```text
packages/model/generated/architecture.json
packages/model/generated/architecture.toml
```

The executable model imports the typed architecture as a Rust dependency and
derives architecture-facing policy from it. It does not deserialize the
generated JSON or TOML to decide model behavior.

The repository also publishes derivative model artifacts:

```text
packages/model/generated/declassification.json
packages/model/generated/model_labels.json
```

Those files are useful for review, interoperability, documentation, and stale
artifact checks. They are not currently intended to replace the typed Rust
declarations from which they are derived.

The planned compiler-era package graph introduces several new opportunities to
accidentally reverse this direction:

- the compiler could deserialize `architecture.json` or
  `architecture.toml`;
- declassification analysis could ingest `declassification.json`;
- operation semantics could be reconstructed by scraping model Rust source;
- the linker could read Markdown tables for constructor or layout policy;
- the target package could scrape an opcode survey;
- release logic could infer required evidence by parsing prose;
- package APIs could accept generic JSON values instead of typed domain
  structures;
- generated files could become trusted merely because they are easy to inspect
  or exchange.

Such reverse dependencies would create several semantic authorities:

```text
typed Rust
generated JSON/TOML
model source text
planning Markdown
realization prose
backend configuration files
```

They would then require permanent cross-source synchronization and would make
it unclear which artifact wins when two sources disagree.

That is precisely the drift the typed architecture and generated-artifact
discipline are intended to prevent.

The compiler needs a source that is:

- typed;
- exhaustively checked;
- refactor-safe;
- deterministic;
- directly usable by Rust packages;
- independent of presentation syntax;
- suitable for bidirectional validation;
- capable of carrying stable semantic identifiers and provenance.

Typed Rust values satisfy those requirements. Generated publication files and
source-text scraping do not.

---

## 2. Decision

First-party semantic and release tooling consumes **typed Rust values derived
from normative Rust declarations**.

The canonical direction is:

```text
typed Rust declarations
        ↓
validated and derived typed Rust values
        ↓
typed compiler analysis
        ↓
typed target/backend artifacts
        ↓
typed linked and transaction artifacts
        ↓
typed evidence and release values
        ↓
optional canonical publication serialization
```

The reverse direction is prohibited for first-party semantic behavior:

```text
generated JSON or TOML
generated declassification or label files
Markdown
LaTeX
model source text
copied external reference prose
        ↓
first-party compiler semantics, linker policy, or release requirements
```

Specifically:

1. `architecture::ARCHITECTURE` and its typed validators remain the source of
   finite architecture declarations.

2. The future `realization` package derives or constructs one validated typed
   `RealizationSpec` from the typed architecture and target-independent
   normative Rust declarations.

3. The executable model is checked against the typed realization. The
   compiler does not scrape model source to reconstruct formulas, reads,
   writes, guards, or transitions.

4. The compiler consumes typed realization values, typed compilation policy,
   typed deployment parameters, and typed target-capability values.

5. Target packages encode source-verified target facts as typed Rust. Backends
   do not parse planning/reference Markdown for opcode semantics.

6. Backends produce typed relocatable programs.

7. The linker consumes typed relocatable programs and typed deployment
   constants.

8. The transaction package consumes typed linked-bundle and ABI values.

9. The vector harness consumes typed model, realization, compiler, target,
   backend, linker, and transaction values.

10. Release logic consumes typed identities, linked artifacts, evidence
    reports, and deployment profiles.

11. Serialization is a one-way publication boundary. A generated artifact may
    be parsed in interoperability and conformance tests, but the parsed value
    must be compared with an independently derived typed expected value and
    must not become the semantic input to first-party compilation or release.

12. Planning and reference Markdown remains informational and is never
    machine-consumed semantic configuration.

---

## 3. Required consequences

### 3.1 Typed package APIs

Every planned package must expose typed inputs and outputs.

Preferred library APIs have forms such as:

```rust
pub fn derive(
    architecture: &architecture::Architecture,
) -> Result<RealizationSpec, RealizationError>;
```

```rust
pub fn analyze<T: TargetCapabilities>(
    request: CompilationRequest<'_, T>,
) -> Result<AnalyzedProgram, CompileError>;
```

```rust
pub fn emit(
    program: &AnalyzedProgram,
    target: &ElementsTarget,
    configuration: &TapscriptConfiguration,
) -> Result<RelocatableProgram, EmitError>;
```

```rust
pub fn link(
    program: &RelocatableProgram,
    deployment: &DeploymentParameters,
) -> Result<LinkedBundle, LinkError>;
```

> Illustrative APIs; their exact signatures are not frozen by this decision.

The requirement is the typed direction, not these names.

APIs should prefer domain types such as:

```text
OperationId
RelationId
ArchitectureIdentity
RealizationIdentity
DeploymentParameters
ElementsTarget
LinkedBundle
EvidenceReports
DeploymentProfile
```

over generic:

```text
serde_json::Value
BTreeMap<String, String>
PathBuf pointing to a manifest
untyped byte blobs with undocumented meaning
```

Raw bytes remain appropriate for final encoded artifacts after their typed
meaning is established.

### 3.2 Typed identity ownership

Stable semantic identifiers must be owned by the package that defines their
meaning.

Expected ownership is:

```text
architecture:
    architecture IDs and discriminants

realization:
    semantic fact, expression, relation, observable,
    constructibility, lifecycle, and proof-alternative IDs

compiler:
    analyzed/lowered node and plan IDs

target package:
    exact target identity and capability IDs

linker:
    constructor, relocation, linked-program, and bundle identities

transaction:
    layout and witness-ABI identities

vectors:
    vector, mutation, relation-coverage, and report identities

release:
    release assembly records over identities owned elsewhere
```

A publication string is a rendering of a typed identifier, not the identifier's
source of truth.

### 3.3 Generated artifacts remain derivative

Every generated artifact must have:

1. one typed source;
2. one explicit generator;
3. one non-writing checker;
4. canonical ordering;
5. deterministic bytes;
6. a documented schema;
7. unknown-field rejection where it is ingested externally;
8. a stale-artifact test;
9. no reverse semantic dependency.

The current generated-artifact ownership pattern under:

```text
packages/artifacts/
```

is the initial implementation model.

Future generators may publish files such as:

```text
realization.json
declassification.json
operation_layouts.json
obligation_placement.json
witness_abi.json
resource_report.json
relation_coverage.json
target_identity.json
linked_bundle_manifest.json
```

These names are illustrative. Adding such a file does not authorize a
first-party package to read it as semantic input.

### 3.4 Declassification is derived from typed dependencies

The future disclosure/declassification analysis must derive from:

- typed facts;
- typed expressions;
- operation dependency edges;
- state assignments;
- public observables;
- constructibility requirements;
- target-independent representation alternatives.

The allowed direction is:

```text
typed operation relation graph
        ↓
typed dependency analysis
        ↓
typed declassification result
        ↓
optional declassification.json
```

The prohibited direction is:

```text
declassification.json
        ↓
compiler disclosure policy
```

The compiler also must not recover the dependency graph by parsing:

```text
packages/model/src/ops/*.rs
packages/model/src/shape.rs
packages/model/src/kernel.rs
```

Model conformance must establish that executable behavior agrees with the typed
realization declaration.

### 3.5 Target facts become typed values

The current Elements tapscript Markdown survey is reference material.

The future target package must encode exact target facts directly:

```rust
pub const INITIAL_ELEMENTS_TARGET: ElementsTarget = ElementsTarget {
    // exact source and capability pin
};
```

> Illustrative declaration; exact target identity is not frozen here.

The typed target must bind:

- the typed compatibility-contract identity (upstream review provenance
  recorded beside it; ADR-011);
- network/genesis identity policy;
- activation set;
- leaf version;
- opcode assignments;
- semantics relied upon by the backend;
- asset/value prefix rules;
- sighash semantics;
- consensus and policy resource limits.

The backend must not scrape `plans/reference/elements-tapscript.md`.

### 3.6 Dynamic external inputs are parsed at explicit boundaries

This decision does not prohibit all external input.

Deployment-specific facts may originate outside the source tree, including:

- network and genesis identities;
- keys;
- issued asset IDs;
- ceremony outputs;
- calibrated bounds;
- artifact hashes;
- independent report hashes;
- output directories;
- release status requests.

Such input must cross an explicit parsing and validation boundary:

```text
external bytes or command-line arguments
        ↓
parser
        ↓
typed Rust value
        ↓
validation
        ↓
semantic/release API
```

The parser is not the source of architecture semantics. It constructs a typed
instance of a schema owned by Rust code.

### 3.7 Tests preserve independent derivation

A publication round-trip test should have this shape:

```text
derive expected typed value from normative Rust
serialize expected value
parse committed/publication bytes
validate parsed value
compare parsed value with expected typed value
compare publication bytes with canonical serialization
```

It should not have this shape:

```text
parse committed publication
use parsed value to compile
declare success because compilation succeeded
```

The first detects stale or malformed publications. The second trusts the
artifact it is supposed to check.

### 3.8 Documentation references source rather than duplicating it

Plans and package documentation should link to typed ownership and generated
artifacts rather than copying large manifests, current hashes, or full tables.

When prose must summarize a typed declaration, it should explicitly identify
the typed source as authoritative.

---

## 4. Prohibited consequences

This decision prohibits the following first-party semantic paths.

### 4.1 Publication-file compilation

Future package APIs must not require arguments such as:

```rust
pub struct CompilationRequest {
    pub architecture_json_path: PathBuf,
    pub architecture_toml_path: PathBuf,
    pub declassification_json_path: PathBuf,
}
```

unless the API is specifically an external interoperability adapter whose
output is immediately compared with the normative typed source and is not used
for first-party release compilation.

The normal first-party compiler API consumes typed values.

### 4.2 Semantic source scraping

The compiler must not:

- parse Rust syntax from model operation files;
- grep model labels to infer relation ownership;
- inspect source comments to derive facts;
- infer read sets from function bodies;
- use proc-macro token streams as a substitute for a typed semantic
  declaration;
- inspect tests to determine operation semantics.

Code generation from typed Rust macros or const declarations remains permitted
when those typed declarations are the normative source. The prohibited action
is recovering semantics from implementation source text.

### 4.3 Markdown-driven target behavior

The target/backend must not:

- parse opcode numbers from a plan;
- parse activation status from Markdown;
- derive stack semantics from copied reference prose;
- use a planning table as a release pin;
- treat a source survey as deployment evidence.

Reference Markdown may guide the implementation review. Typed target code and
deployment tests govern behavior.

### 4.4 Parallel handwritten semantic tables

A package must not create a second operation table that independently repeats:

- input/output families;
- root use;
- bounds;
- open-flow roles;
- value-flow classes;
- authorization modes;
- projections;
- architecture IDs;
- semantic formulas;
- disclosure dependencies.

Where a target-specific mapping is necessary, it must map from typed semantic
IDs and be validated for complete coverage. It must not redefine the semantic
fact itself.

### 4.5 Generic data structures as hidden schemas

Using a generic JSON value or string map internally must not hide a semantic
schema.

For example, this is not an acceptable substitute for a typed operation
relation:

```rust
BTreeMap<String, serde_json::Value>
```

Generic structures may exist inside serialization adapters, but they must
convert immediately to or from validated typed values.

### 4.6 Generated-file repair during checks

Tests and check commands must not regenerate tracked files as a side effect.

The split is:

```text
generate:
    may write explicit caller-selected outputs

check:
    derives expected typed values and compares without writing
```

A stale artifact is a test/check failure, not an invitation for the test to
modify the checkout.

### 4.7 Plans as executable configuration

No package may read this decision record, any file under `plans/`, or any
planning index to determine:

- package dependencies;
- operation semantics;
- proof selection;
- disclosure;
- target capabilities;
- layout;
- evidence requirements;
- release status.

Implemented machine configuration must be represented in typed source or an
explicit typed external configuration schema.

---

## 5. Alternatives considered

### 5.1 Deserialize generated architecture TOML as compiler input

#### Proposal

Treat:

```text
packages/model/generated/architecture.toml
```

as the compiler's manifest input.

#### Advantages

- easy to inspect;
- language-neutral;
- apparently decouples compiler from architecture implementation;
- resembles conventional compiler manifest ingestion;
- supports potential external producers.

#### Rejection

The file is already derivative of the typed architecture. Making it
first-party compiler input reverses the source direction and creates two
authorities:

```text
typed architecture
generated TOML consumed by compiler
```

A stale or manually edited publication could then alter compilation even while
the typed architecture remained unchanged.

External interoperability can be supported through a separate adapter that:

1. parses the publication;
2. validates the envelope;
3. compares it with the expected typed architecture identity;
4. produces a typed value under an explicitly different trust model.

That does not justify using the publication as the normal first-party semantic
path.

### 5.2 Use generated JSON as a stable language-neutral IR

#### Proposal

Serialize the realization or compiler relation graph to JSON and make every
later stage consume it.

#### Advantages

- clear process boundaries;
- easy artifact inspection;
- language-independent backend implementations;
- convenient caching.

#### Rejection for the initial toolchain

Using serialized IR between isolated processes can be valid, but only if the
serialized schema is a canonical encoding of a typed IR and the consumer
validates the typed identity and schema.

The source of truth remains the typed Rust structure. The JSON does not become
the place semantics are authored.

The initial first-party implementation should pass typed values in-process
where practical. A future process boundary may use typed canonical wire
artifacts as transport, but their schema and identity remain owned by Rust
types.

### 5.3 Scrape the executable model for operation reads and writes

#### Proposal

Analyze model source to infer:

- which state fields each operation reads;
- which facts become public;
- which formulas apply;
- which relations a compiler must enforce.

#### Advantages

- appears to avoid duplicate semantic declarations;
- follows executable behavior;
- may discover actual dependencies automatically.

#### Rejection

Rust source analysis would be incomplete and brittle:

- helper functions obscure dependencies;
- control flow changes inferred sets;
- source-level refactoring could move compiler semantics;
- macro expansion complicates analysis;
- semantic intent is not identical to syntactic reads;
- proof alternatives, constructibility, lifecycle, and disclosure reasons are
  not recoverable reliably;
- source text would become an accidental language.

The accepted alternative is a typed target-independent realization shared with
model conformance.

### 5.4 Author compiler policy directly in the compiler

#### Proposal

Write one compiler-side table for each operation, independently of the model.

#### Advantages

- direct control over code generation;
- no additional realization package;
- backend implementation can begin quickly.

#### Rejection

This duplicates the semantic theory and invites drift between:

- architecture;
- model;
- compiler;
- documents.

The compiler would then be difficult to validate relation-by-relation because
its internal table would be both the implementation and the purported source
of expected obligations.

The target-independent realization layer is the accepted alternative.

### 5.5 Treat planning Markdown as configuration

#### Proposal

Keep proof methods, target capabilities, package sequencing, or layout policy in
Markdown tables and have tooling read them.

#### Advantages

- easy human editing;
- avoids building a typed configuration layer;
- keeps plans and implementation visibly synchronized.

#### Rejection

Plans mix accepted direction, future work, rationale, and unresolved questions.
They are not stable machine schemas.

Executable configuration must have:

- typed validation;
- deterministic identity;
- explicit versioning;
- fail-closed unknown-field behavior;
- tests.

Planning Markdown is intentionally outside those responsibilities.

### 5.6 Make an external schema the only normative source

#### Proposal

Move the architecture and realization into a language-neutral schema, generate
Rust from it, and make the schema authoritative.

#### Advantages

- language neutrality;
- potential multiple implementation languages;
- external tooling interoperability;
- conventional schema-first workflow.

#### Rejection for the current system

The repository already has a mature typed Rust architecture, exhaustive enum
welds, validators, canonical hashes, and model integration.

Replacing that source would be a major migration with substantial new trust and
generation machinery. It is unnecessary for the first compiler and would delay
the actual missing semantic layer.

The decision may be reconsidered only if a concrete multi-language need
outweighs the migration and authority costs.

---

## 6. Assurance and evidence consequences

### 6.1 Architecture evidence remains typed-source-derived

Architecture tests must continue to establish:

- complete typed declarations;
- structural validation;
- hash stability;
- generated publication equality;
- document appendix equality;
- release validation.

The compiler cannot upgrade a malformed or stale publication into valid
architecture evidence.

### 6.2 Model conformance becomes explicit

Introducing `realization` creates a new evidence boundary:

```text
typed realization relation
        ↔
executable model behavior
```

Model conformance tests must demonstrate agreement without collapsing the two
roles into one source-scraping implementation.

A green model test remains model evidence. It does not prove backend emission
or target semantics.

### 6.3 Compiler relation coverage gains source provenance

Because the compiler consumes typed relations, every lowered relation can
carry provenance back to:

- architecture operation;
- realization relation;
- expression dependencies;
- invariant or observable obligation.

That provenance supports relation-indexed evidence and prevents unnamed checks
from appearing only in emitted code.

### 6.4 Publication checks become stronger

A publication checker can compare:

```text
parsed artifact
canonical re-rendering
typed expected value
declared hashes
```

This is stronger than trusting either the serialized body or its embedded hash
alone.

### 6.5 External interoperability remains possible

An independent implementation may consume published JSON/TOML or other
canonical artifacts.

Its trust model is:

```text
parse publication
validate schema and envelope
verify identity/hash
implement independently
produce evidence
```

The first-party compiler's typed input rule does not prevent external
interoperability. It prevents derivative first-party publications from
becoming a competing internal semantic authority.

### 6.6 Release evidence remains separately typed

Independent report hashes, target identities, calibrated bounds, and artifact
hashes enter release through typed deployment/profile structures.

Release does not parse prose to decide which reports are required.

---

## 7. Determinism and identity consequences

### 7.1 Typed structural identities

Semantic identity must derive from canonical typed structure, not from:

- source file path;
- line number;
- declaration insertion order where order is not semantic;
- debug formatting;
- serialized object-key order;
- host environment;
- current time.

Packages that publish stable identities must define:

- domain separator;
- schema version;
- included typed fields;
- excluded presentation fields;
- collection ordering;
- algorithm identifier;
- migration policy.

### 7.2 Publication identity is not source identity

A generated file's byte hash identifies that publication encoding. It does not
replace:

- architecture semantic hash;
- architecture behavioural hash;
- realization identity;
- compiler configuration identity;
- target identity;
- linked-bundle identity;
- deployment-profile identity.

Different identity domains must remain explicitly separated.

### 7.3 Caching

A future cache may key typed stages by their typed identities:

```text
architecture identity
realization identity
compiler configuration identity
target identity
deployment parameter identity
```

It must not use a file modification time or an unverified publication path as
the semantic cache key.

### 7.4 Reproducible generation

A generator must produce the same bytes from the same typed value and explicit
rendering configuration.

No generated artifact may contain:

- ambient timestamps;
- host paths;
- nondeterministic map order;
- random identifiers;
- unpinned tool versions;
- environment-dependent formatting.

---

## 8. Implementation and migration

### 8.1 Preserve the current architecture direction

No migration is required for the existing architecture/model relationship.

Continue:

```text
architecture typed source
    ↓
PublishedArchitecture
    ↓
architecture.json / architecture.toml
```

Continue checking committed files against typed expected values.

### 8.2 Introduce realization as typed code

Phase 1 adds:

```text
packages/realization/
```

It should depend on `architecture` and expose typed semantic structures.

Do not first create a `realization.json` and then generate Rust from it.

### 8.3 Add model-conformance tests incrementally

For pilot operations:

1. derive the typed operation realization;
2. construct accepted model transitions;
3. project semantic facts from model inputs/outputs;
4. evaluate declared relations;
5. compare declared projections with model certificates;
6. mutate facts and require relation failure.

Do not rewrite the entire model before these tests demonstrate that the
realization vocabulary is adequate.

### 8.4 Move declassification derivation

The current model artifact derivation is provisional.

Migration path:

1. define operation facts and dependencies in `realization`;
2. derive typed declassification there;
3. update the artifact generator to serialize that typed result;
4. compare committed bytes through the non-writing checker;
5. remove any compiler-facing dependency on the old model-derived publication;
6. retain compatibility tests as needed.

The final compiler path must never read the JSON.

### 8.5 Encode target facts in `target-elements`

The reference survey informs implementation review.

The target package then encodes the exact selected claims as typed constants
and tests them against a supported target environment, recording the node
version as test provenance.

No code generator should parse the survey.

### 8.6 Keep CLI parsing outside semantic libraries

Future command-line binaries may accept paths and external values, but they
must:

1. parse through ADR-010-compliant entrypoints;
2. convert into typed values;
3. validate;
4. call library APIs;
5. write assets only to caller-selected paths.

Semantic libraries remain independent of process argv and presentation output.

### 8.7 Promote implemented policy when appropriate

Once future packages implement this rule consistently, promote it to a root
ADR.

The ADR should govern current code and may supersede this planning record's
policy text.

---

## 9. Risks and limitations

### 9.1 Rust-language coupling

The decision couples the first-party semantic implementation to Rust.

This is intentional for the initial toolchain because:

- the architecture and model already use Rust;
- the workspace has strong Rust policy;
- type sharing reduces drift;
- the missing work is compiler implementation, not language-neutral schema
  design.

Mitigation:

- publish canonical derivative artifacts;
- document schemas;
- provide independent vectors;
- allow external implementations to consume verified publications;
- keep target-independent semantics free of Rust implementation accidents.

### 9.2 Shared-code correlated failures

If model conformance and compiler analysis reuse too much implementation code,
they can share one bug and agree incorrectly.

This decision does not require collapsing the model into the compiler.

Mitigations include:

- model and compiler retain distinct roles;
- relation-indexed mutations;
- target differential tests;
- independent indexer/auditor implementations;
- canonical publications for external review;
- separate substrate evidence.

### 9.3 Typed declarations can still be wrong

Typed Rust prevents many classes of drift and malformed state. It does not
prove that the typed semantics are economically correct.

Normative review, architecture validation, model tests, mutation tests,
property tests, document welds, versioning gates, and deployment evidence
remain necessary.

### 9.4 Publication consumers use a different path

Independent external implementations may consume JSON/TOML while first-party
code consumes typed Rust.

That is acceptable if both paths bind to the same published identities and the
external implementation independently validates the publication.

The repository must not falsely claim that the first-party typed path alone
constitutes independent interoperability evidence.

### 9.5 Procedural temptation

Generated files are convenient to inspect and pass across processes. Future
contributors may be tempted to add “temporary” parsers that become semantic
dependencies.

Mitigation:

- package plans list forbidden inputs;
- API reviews reject publication paths in semantic requests;
- tests assert no generated-file dependency where practical;
- root ADR promotion makes the rule repository-wide once implemented.

---

## 10. Supersession conditions

This decision may be superseded if the project deliberately adopts a different
normative source architecture, for example:

1. a language-neutral schema becomes the sole normative source;
2. Rust declarations are generated from that schema;
3. the migration preserves or deliberately versions every semantic identity;
4. validators and model/compiler consumers are updated;
5. generated-source and bootstrap trust are explicitly audited;
6. publication and external interoperability improve enough to justify the
   migration;
7. a new accepted decision and, once implemented, root ADR govern the change.

It may also be refined if first-party packages move into different programming
languages. Such a refinement must preserve the deeper rule:

> one typed normative semantic source, one-way generated publications, and no
> source-text scraping.

This decision is not superseded merely because:

- a CLI accepts JSON deployment input;
- a canonical IR is serialized between processes;
- an external implementation consumes architecture TOML;
- a test parses generated files;
- a publication schema is added.

Those cases remain compatible when the typed source and identity validation
stay authoritative.

---

## 11. References

### Normative and implemented source

- [`../../docs/attestation/realization.md`](../../docs/attestation/realization.md)
- [`../../packages/architecture/src/spec.rs`](../../packages/architecture/src/spec.rs)
- [`../../packages/architecture/src/validate.rs`](../../packages/architecture/src/validate.rs)
- [`../../packages/architecture/src/export.rs`](../../packages/architecture/src/export.rs)
- [`../../packages/model/src/manifest.rs`](../../packages/model/src/manifest.rs)
- [`../../packages/model/src/artifacts.rs`](../../packages/model/src/artifacts.rs)
- [`../../packages/artifacts/src/lib.rs`](../../packages/artifacts/src/lib.rs)

### Repository policy

- [ADR-010](../../adr/010-command-line-output-contract.md)
- [ADR-011](../../adr/011-toolchain-and-dependency-policy.md)

### Planning architecture

- [`../README.md`](../README.md)
- [`../toolchain-architecture.md`](../toolchain-architecture.md)
- [`../roadmap.md`](../roadmap.md)
- [`../packages/realization.md`](../packages/realization.md)
- [`../packages/compiler.md`](../packages/compiler.md)

---

## 12. Decision summary

> First-party attestation-contract semantic tooling begins with validated typed
> Rust declarations and passes typed values through realization, compiler,
> target, backend, linker, transaction, evidence, and release stages.
> Generated JSON/TOML, Markdown, LaTeX, model source text, and copied target
> references remain one-way publications or review material; they never flow
> back into first-party semantic or release policy.
