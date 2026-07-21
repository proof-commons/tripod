# ADR-013: Global Documentation and Source Label Graph

**Status:** Decided and implemented
**Scope:** Every first-party authored Markdown and Rust source, the labels
package, and the generated label registers
**Supersedes:** the ADR-012 unlinted-local-label and lint-boundary decisions

---

## Context · `sec:labels:context`

ADR-012 established layered citations but deliberately left plan-local labels
unlinted (`[ADR012-rule:labels:local]`), scoped the checker to imported
citations only (`[ADR012-rule:labels:lint-boundary]`), and tolerated repeated
bare labels by keeping the first occurrence.

Experience ahead of the realization package showed three defects:

1. an accidental bare label silently becomes a conceptual home;
2. a generated index can sustain its own citation set;
3. inferred mint-versus-citation ownership is ambiguous across namespaces.

The reference graph must therefore become mechanically complete without
making plans or comments semantic input.

## Owner-relative citation forms · `rule:labels:owner-relative`

Every participating label occurrence has exactly one of three forms:

```text
local mint:                `rule:area:name`
same-owner citation:       (`rule:area:name`)
different-owner citation:  (`[ADR010-rule:output:streams]`)
```

A bare label mints a label in the current source's owner.

A parenthesized bare label cites a mint in the same owner. It resolves
anywhere within that owner, including another file.

A parenthesized square-bracket label cites a mint owned outside the current
owner, using the external owner's registered prefix.

Square brackets identify an ownership boundary. The authority of the imported
fact is a property of the imported owner, not of the bracket.

Every citation resolves to exactly one mint. Every owner-local mint is
unique. A duplicate bare label is a hard failure, never a harmless repeat.

## Owner registry · `rule:labels:owner-registry`

| Source | Owner prefix |
|---|---|
| Layer-0 LaTeX | `A-` |
| Realization Markdown | `RZ-` |
| each `adr/NNN-*.md` | `ADRNNN-` |
| all planning Markdown | `PLAN-` |
| model Rust | `MODEL-` |
| other first-party Rust crates | one owner per Cargo package |
| non-numbered repository Markdown | `DOC-` |

A local citation never resolves into another owner. A new public prefix
requires an ADR that extends this registry. Local references inside one owner
need no public prefix.

## ADR owners · `rule:labels:adr-owners`

Each numbered ADR is an independent label owner. The owner of `adr/NNN-*.md`
is `ADRNNN`. The prefix is derived from the filename and never written at the
mint.

Inside one ADR:

- a bare valid label is a mint owned by that ADR;
- a parenthesized bare label cites only a mint in the same ADR;
- a citation to another ADR uses the parenthesized square-bracket form;
- a self-qualified square-bracket citation is invalid, because same-owner
  citations must use the local form.

Two ADRs may mint the same local text without collision; ownership
disambiguates. A citation from another owner does not transfer ownership of
the cited rule.

## Global resolution · `rule:labels:global-resolution`

The checker resolves in two passes:

1. harvest every participating source, build owner registries from mints,
   and reject duplicate mints with both locations;
2. resolve every same-owner and imported citation against the complete
   registries.

Resolution never depends on file traversal order, and forward references
across files resolve. Unknown owners, unresolved citations, malformed forms,
non-parenthesized imports, and bracket-free cross-owner tokens all fail.

## Rust sources · `rule:labels:rust-sources`

Rust occurrences use the acute delimiter so ordinary Rustdoc backtick spans
never become labels:

```text
local mint:                ´rule:verification:example´
same-owner citation:       (´rule:verification:example´)
different-owner citation:  (´[RZ-rule:translation:predicate]´)
```

The scanner processes comments and documentation comments only. String
literals, raw strings, character literals, and fenced Rustdoc examples are
ignored. An unclosed acute delimiter is a failure.

Module indexes and crate catalogs cite; the defining module mints.

## Architecture weld · `rule:labels:synthetic-citations`

Typed architecture label strings, such as witness semantic tags and invariant
clause identifiers, enter the graph as synthetic upstream citations of
Realization mints.

They are typed data, not comment syntax, and are never treated as Rust-source
mints. The existing architecture/document weld remains authoritative for
which fields participate.

## Generated registers · `rule:labels:generated-nonparticipation`

A generated register must not mint or cite the labels it indexes.

Displayed tokens render as nonparticipating double-backtick spans, and the
register files are classified as derivative output excluded from the source
reference graph while their exact bytes remain checked.

The same rule covers examples anywhere: a token shown but not meant is placed
in a fenced block or a double-backtick span.

## Layer-0 anchor derivation · `rule:labels:anchor-derivation`

The Realization upward-citation set derives from actual body citations
only. The harvest:

1. ignores fenced code and nonparticipating example spans;
2. excludes the upward-citation index section;
3. resolves every Layer-0 citation to a Layer-0 mint;
4. requires the committed index to present exactly the distinct body anchor
   set;
5. never writes;
6. computes the pinned anchor-set hash from the distinct resolved body
   citations.

A token present only in the index cannot keep itself in the release anchor
set, and removing a label's last body citation stales the committed index.
The index's per-anchor attribution column remains editorial prose.

## Source census · `rule:labels:census`

The census covers every first-party authored Markdown and Rust file. It
excludes `.git/`, `target/`, build directories, `archive/`, vendored
third-party trees, and generated binary artifacts.

Generated Markdown may be scanned as publication syntax but contributes no
source mints or citations.

Traversal failures are diagnostics. An unreadable tree must not silently
become an empty census. A file with no labels is valid; every participating
occurrence must use one of the three forms and resolve.

Register generation stays scoped: an invalid citation in one owner must not
block regenerating an unrelated owner's register, while the full repository
check still validates everything.

## Normativity unchanged · `rule:labels:normativity`

Mechanical completeness does not change authority:

- planning labels remain non-normative navigation;
- Rust labels remain documentation and provenance metadata;
- registry membership is not a protocol identity;
- renaming a label updates all citations in the same commit and requires no
  protocol or realization version bump;
- no compiler, backend, linker, transaction builder, or release validator
  consumes the documentation graph;
- the labels package and documentation checks are the only first-party
  consumers, under (`[ADR012-rule:labels:no-semantic-input]`).

## Versioning consequence · `rem:labels:versioning`

Migrating the Realization document's accidental bare repeats to citation form
changes presentation, not denotation. The document revision advances while
`realization_version`, the behavioural hash, and the architecture semantic
hash stay unchanged, provided label values themselves do not change.

A migration that changes the set of model labels is reviewed as a
source-label correction, never silently accepted as formatting.

## Supersession · `sec:labels:supersession`

This record supersedes the ADR-012 decisions that plan-local labels are
unlinted (`[ADR012-rule:labels:local]`) and that the checker validates
imported citations only (`[ADR012-rule:labels:lint-boundary]`).

It retains, unchanged in substance:

- the label shape (`[ADR012-rule:labels:shape]`);
- the layered-authority classes (`[ADR012-rule:labels:decision]`);
- the register publications (`[ADR012-rule:labels:registers]`);
- the generation/check split (`[ADR012-rule:labels:generation]`);
- the machine-use prohibition (`[ADR012-rule:labels:no-semantic-input]`).

Generated artifacts continue to follow
(`[ADR011-rule:toolchain:generated]`).

## Rejected alternatives · `sec:labels:alternatives`

### One flat global namespace

Rejected because a planning citation could silently resolve into an ADR or
realization mint without declaring the authority crossing.

### First mint wins

Rejected because an incidental bare span would silently redefine a label's
conceptual home without a diagnostic.

### Backtick labels in Rust comments

Rejected because ordinary Rustdoc spans would need a fragile "label-like"
heuristic to avoid false mints.

### Leaving plan-local labels unchecked

Rejected because the permitted dangling references hid real defects; the
cost of completeness fell once resolution became owner-aware and cross-file.

### Generated registers as participating sources

Rejected because a register row could sustain its own membership after the
body citation was removed.

## Verification · `gate:labels:implementation`

ADR-013 is implemented when:

- all three forms parse in Markdown and Rust sources;
- duplicate mints fail with both locations in every owner;
- every local and imported citation resolves to exactly one mint;
- cross-owner tokens without brackets and self-qualified imports fail;
- resolution is independent of traversal order;
- architecture label strings resolve as synthetic Realization citations;
- generated registers are nonparticipating, current, and deterministic;
- the pinned anchor-set hash derives from body citations only;
- traversal failures surface as diagnostics;
- scoped register generation ignores unrelated owners' defects;
- the migrated documents contain one mint per conceptual label;
- `scripts/check-plans.sh` and the full CI gate pass.
