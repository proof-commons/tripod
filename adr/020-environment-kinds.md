# ADR-020: Adoption of the Environment-Kind Registry

**Status:** Decided; the kind-token migration adjudicated below lands
across the commits of this batch
**Scope:** The kind segment of every label minted in this repository,
and the checker's kind vocabulary
**Adopts:** the corrected draft archived at
[plans/drafts/environment-kinds.md](../plans/drafts/environment-kinds.md)
as the normative kind registry
**Depends on:** [ADR-019](019-label-calculus.md), which fixes the label
form the kind segment opens
**Does not establish:** any obligation to use a catalogued kind — the
registry fixes what a kind means when used, never that it must be

---

## Context · `sec:kinds:context`

Before this record the repository had no kind vocabulary worth the name.
The checker validated a closed list of kinds for the realization owner
and a second closed list for the model crate, both compiled into Rust;
every other prose owner accepted any word at all. Kinds accordingly
drifted: the corpus carried both `tbl` and `tab` for a table, and both
`post` and `postc` for a postcondition, which is one concept wearing two
tokens in each case.

An externally authored registry classifying 248 attested environment
names into 148 kinds was accepted, audited, corrected, and archived
verbatim. The gap census compared it against every kind in use and found
forty-three assigned identically, a short list divergent, and a longer
list absent.

---

## Decision · `dec:kinds:adoption`

The repository adopts the archived registry as its kind vocabulary.

**Rule (Normative source)** · `rule:kinds:normative-source`

The draft is the registry. This record does not restate the catalogue;
it records only this repository's migration, its local extensions, and
the sense it fixes for one ambiguous word.

**Rule (Extension entry)** · `rule:kinds:extension-entry`

A kind the registry does not carry is a local extension. It enters the
register below by the same decision that introduces it, and it must be
distinct — as a token and as a concept — from every registry entry. A
kind naming a genre the registry already classifies is not an extension
but a defect, and is migrated.

---

## Migration record · `tab:kinds:migration`

The census flagged eight tokens as conflicting. Each was adjudicated
against the **concept** it names, not against the spelling of its token,
because the registry classifies genres and not words. A token collides
when the registry assigns the same genre a different kind; a token is an
extension when the genre is uncatalogued and the token is unclaimed.

Six collided and are migrated, on 2026-08-16. Two did not.

| Repository kind | Registry position | Adjudication | Outcome |
|---|---|---|---|
| `subsec` | Subsection is a section nested; the sub- prefix is a declared presentation device | Same genre, two tokens. A subsection is a document division exactly as a section is, and the repository already carried both | migrated to `sec` |
| `tbl` | Table is `tab` | Same genre. The corpus carried both spellings, which the one-kind invariant forbids on its own | migrated to `tab` |
| `lst` | Listing, Code, and Pseudocode are all `listing` | Same genre: a displayed block of code | migrated to `listing` |
| `post` | Postcondition is `postc` | Same genre. The corpus carried both spellings | migrated to `postc` |
| `protocol` | Protocol is `proto` | Same genre: the procedural exchange. The long form is the same word, not a different concept | migrated to `proto` |
| `ins` | Insight and Intuition are `intuit` | Same genre. The mints so labelled are recognition lessons and standing intuitions, which is what the registry classifies | migrated to `intuit` |
| `task` | Task is classified `exer`, among exercises | **Different genre.** The registry's Task is work assigned to a reader. The repository's mints name backlog work items — the kept, tracked units of a work register, a records-family genre. The token `task` is assigned to nothing by the registry | local extension, not migrated |
| `res` | Result is `result` | **Different genre.** The token does not abbreviate Result. Every mint so labelled names a catalogued **residual risk** — R-dust, R-op, R-conv, R-CSV — an accepted-and-recorded exposure, not a stated result. The registry catalogues no Residual, and the token `res` is assigned to nothing | local extension, not migrated |

Each migration renamed every mint and every citation of the token in one
commit, as the calculus requires, and moved the checker's compiled kind
lists in the same commit so the gate never went red.

---

## Local-extension register · `tab:kinds:extensions`

Kinds this repository uses that the registry does not carry. Each was
checked against the whole registry for token and concept distinctness.

| Kind | Genre it names |
|---|---|
| `branch` | A named control-flow branch of a typed source, cited by tests |
| `candidate` | A proposed option under evaluation, before selection |
| `err` | An entry of a package's error vocabulary. Distinct from `errat`, which is a correction notice about a document |
| `leaf` | A terminal node of a decision or derivation tree |
| `milestone` | A dated point in a delivery schedule |
| `obl` | A standing obligation on an implementation |
| `op` | An operation of the protocol's state machine |
| `phase` | A numbered stage of the delivery plan |
| `pin` | A committed value or reference frozen against drift |
| `pkg` | A first-party package as a unit of plan and ownership |
| `q` | An open question awaiting a ruling. Retained as the registry's own token for Question |
| `ref` | A cited external work or upstream artifact |
| `req` | A stated requirement on the system |
| `res` | A catalogued residual risk, accepted and recorded |
| `task` | A tracked unit of backlog work |
| `test` | A named test case in a typed source |
| `trap` | A known way to get the design wrong, recorded to be avoided |

---

## Registry-sense disambiguation · `rem:kinds:disambiguation`

The registry warns that the word registry names distinct artifacts
across a corpus, its sense fixed by each document's preamble. This
repository carries three such artifacts, and they are not the same
thing:

- the **kind registry**, which is the adopted draft;
- the **minting registries** of the checker, one per owner, which are
  runtime data structures and are never documents;
- the **identity register** at `plans/registers/identities.md`, a kept
  and cited document classified `reg` by the registry's own table.

Where this repository says registry without qualification inside a
labels context, it means the kind registry.

---

## Consequences · `rem:kinds:consequences`

The kind segment now has a meaning a reader can look up, and the
checker's vocabulary is a stated list rather than an accident of which
owner was implemented first.

The migration was a rename of label values, not a re-presentation, so
the presentation-invariance meta-theorem does **not** cover it: the
generated registers were regenerated, and the attestation anchor-set pin was
recomputed and reviewed as a source correction rather than accepted as
formatting.

Two narrownesses remain. The kind vocabulary is still split between a
realization list and a model list compiled into Rust, with no vocabulary
at all for the prose owners; making the registry itself the checker's
vocabulary is backlog DI-003. And the LaTeX label surface of the attestation
paper carries three further tokens the census did not reach — `motto`,
`invest`, and `abs` — which the same treatment will settle when that
surface enters scope.

---

## Rejected alternatives · `rem:kinds:rejected`

**Record every divergent token as a local extension.** This is what the
one-kind invariant exists to forbid. A corpus carrying both `tbl` and
`tab` has not extended the registry; it has two names for one thing.

**Migrate on token spelling.** Then `task` becomes `exer` and a backlog
item is filed as a reader's exercise, and `res` becomes `result` and an
accepted risk is filed as a proved statement. Both would record a
falsehood in the label. The registry classifies genres; so does this
migration.

---

## Implementation gate · `gate:kinds:implementation`

Adoption holds when:

- the registry is archived, cited as normative here, and restated
  nowhere;
- no migrated token survives anywhere in the corpus as a kind;
- every kind in use is either carried by the registry or listed in the
  local-extension register;
- no extension collides with a registry token or a registry genre;
- the checker's compiled kind lists carry the migrated tokens and not
  the retired ones;
- the corpus-wide label check passes in continuous integration.

Every item holds once the migration commits of this batch have landed.
