# D001: Typed Rust Is the First-Party Semantic Source · `dec:source:typed-rust`

> **Status:** Accepted
> **Class:** Source ownership
> **Imports:** (`[ADR011-rule:toolchain:generated]`),
> (`[RZ-rem:overview:register-authority]`)
> **Supersedes:** none

## Choice · `rule:source:typed-rust`

First-party semantic tooling consumes validated typed Rust values.

The direction is:

```text
typed declarations
    ↓
validated typed derivations
    ↓
compiler/backend/linker/release values
    ↓
optional canonical publications
```

Generated JSON, TOML, Markdown, LaTeX, indexes, reports, and disassembly are
one-way publications.

They are not first-party semantic inputs.

## Owned sources · `tab:source:owners`

| Owner | Typed source |
|---|---|
| finite architecture | `architecture::ARCHITECTURE` |
| target-independent semantics | future `realization::RealizationSpec` |
| compiler analysis | future typed compiler IR |
| target compatibility | future typed target package |
| linked deployment | future typed linked bundle |
| transaction ABI | future typed ABI |
| deployment release | typed deployment profile and evidence values |

The executable model remains executable reference behavior. It is not a syntax
tree for the compiler.

## Constraints · `rule:source:constraints`

First-party semantic packages must not derive behavior by parsing:

- generated architecture publications;
- generated declassification reports;
- planning or ADR Markdown;
- realization prose;
- Attestation LaTeX;
- model source text;
- model comments or labels;
- tests;
- target reference Markdown;
- backend disassembly.

External bytes may enter through explicit parsers when they represent
deployment inputs or interoperability publications. Such input is converted
immediately into validated typed values.

## Consequences · `sec:source:consequences`

- Architecture publications remain review and interoperability artifacts.
- Declassification derives from typed semantic dependencies.
- Compiler semantics are not reconstructed from model implementation details.
- Target capabilities are typed rather than scraped from references.
- Release tooling validates publications against independently derived typed
  expected values.
- Cache and artifact identities derive from typed structure rather than file
  timestamps or paths.
- Generators and non-writing checkers remain separate.

## Does not authorize · `sec:source:limits`

This decision does not authorize:

- treating all Rust implementation code as normative;
- making model internals compiler APIs;
- accepting one unvalidated serialized IR as authority;
- generating both candidate and expected behavior through one code path and
  calling the result independent evidence;
- preventing external implementations from consuming canonical publications;
- hiding a schema inside generic string maps or `serde_json::Value`.

Typed source must still be reviewed, validated, versioned, and tested.

## Migration · `rule:source:migration`

New packages expose typed inputs and outputs first.

A publication is added only when a real consumer or review need exists. It
must have:

1. one typed source;
2. one canonical renderer;
3. one non-writing checker;
4. deterministic bytes;
5. an owned schema;
6. no reverse dependency.

## Supersession · `rule:source:supersession`

This decision may be replaced by another single typed semantic source, such as
a language-neutral formal schema, only if the migration preserves:

- one authoritative source;
- typed validation;
- semantic identities;
- model and compiler consumption;
- one-way publications;
- no source-text scraping.

## Verification · `gate:source:typed-rust`

The decision is implemented when package dependency review and tests establish
that no first-party semantic or release path consumes generated publications,
planning prose, or model source text.
