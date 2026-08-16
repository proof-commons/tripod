# ADR-019: Adoption of the Label Calculus

**Status:** Decided and implemented, with the checker re-engineering
tracked as backlog DI-003
**Scope:** Every authored prose and code source of this repository, and
the labels checker that enforces the graph over them
**Adopts:** the corrected draft archived at
[plans/drafts/label-calculus.md](../plans/drafts/label-calculus.md) as
the normative statement of the calculus
**Retires:** ADR-012 and ADR-013, deleted from the tree; their text
remains in Git history
**Does not establish:** any semantic, build, or release meaning for a
label — the graph stays documentation

---

## Context · `sec:labels:context`

The repository has enforced an owner-partitioned label graph since
ADR-012, restated as a calculus in ADR-013. An externally authored
generic statement of the same calculus was accepted, audited, corrected
by its author, and archived verbatim under `plans/drafts/`. The gap
census at
[plans/reference/draft-gap-census.md](../plans/reference/draft-gap-census.md)
read every clause of that draft against the implementation and found the
repository already satisfies the calculus except where the draft
generalizes a repository-specific mechanism into an adoption parameter.

Two statements of one calculus is one fact with two owners. The
divergences between them are then invisible, and a reader cannot tell
which text binds.

---

## Decision · `dec:labels:adoption`

The repository adopts the archived draft as the normative statement of
the label calculus. ADR-012 and ADR-013 are retired and deleted.

**Rule (Normative source)** · `rule:labels:normative-source`

The draft text is the calculus. This record does not restate it, and no
other document in the tree may restate it: a clause is cited, never
copied. This record carries only what adoption of a generic text into
this repository must add — the adoption parameters, the amendments, and
the migration record.

The draft is archived under `plans/`, so it is owned by the planning
tree. Its authority here comes from this record, not from its location:
where the draft and a planning document disagree, the draft binds.

---

## Adoption parameters · `tab:labels:parameters`

The calculus is parametric in three data. This repository fixes them as
follows, and a change to any row enters only by a new record.

| Parameter | Fixing |
|---|---|
| Signature: owner prefixes | `A` the specification; `RZ` the realization contract; `PLAN` the planning tree; `DOC` repository documentation; `MODEL` the model crate; `ADRNNN` one owner per numbered record, derived from the filename and never written at a mint |
| Signature: code packages | One owner per Cargo package. These owners have no readable import prefix today; registering them is backlog DI-003, and until then they are citeable only within themselves |
| Designated typed-data classes | Architecture witness semantic tags, and the exported clause identifiers of the architecture manifest. Both target the `R13` owner. No third class is designated |
| Citation-index documents | `plans/labels/specification.md` over the `A` owner, and `plans/labels/realization.md` over the `RZ` owner. Both are generated registers and participate in nothing they index |

---

## Amendments · `rule:labels:areas`

The draft fixes kind and area as words without hyphens. This repository
amends the area segment only: **an area may hyphenate words exactly as a
name may**. The corpus already carries hyphenated areas, `labels-index`
among them, and the concession costs nothing the calculus relies on —
the colon, not the hyphen, is the segment separator.

The kind segment takes no such concession. A kind is a word, and the
checker rejects a hyphenated kind. Kinds are the vocabulary
[ADR-020](020-environment-kinds.md) governs, and a registry of words
admits no hyphenated member.

No other clause of the draft is amended.

---

## Consequences · `rem:labels:consequences`

The calculus was already implemented, so adoption changes little in
code. What it changes is where a reader looks: a question about minting,
resolution, participation, or ownership is answered by the draft, and by
this record only where a parameter or an amendment is at issue.

Two known narrownesses stand and are honest rather than repaired here:
the synthetic-citation and anchor-harvest mechanisms are each one
correct hardwired instance of their rule rather than the rule. Widening
them, along with the single participation scanner and the registered
package prefixes, is backlog DI-003.

Adoption is not a migration of label values. Under the draft's own
presentation-invariance meta-theorem nothing that hashes the corpus
moves, and no software version bumps.

---

## Rejected alternatives · `rem:labels:rejected`

**Keep ADR-013 and cite the draft as commentary.** The two texts already
disagree in the places the census names; keeping both leaves the
disagreement unresolved and the reader without an authority.

**Copy the draft into this record.** The draft is a corrected verbatim
artifact of an external author. Copying it forks it at the first
correction and destroys the provenance that made it worth adopting.

**Amend the draft text to admit hyphenated areas.** The archive is
verbatim. An adopting corpus records its amendments in its own record,
which is what this section is.

---

## Implementation gate · `gate:labels:implementation`

Adoption holds when:

- the draft is archived, cited as normative here, and restated nowhere;
- ADR-012 and ADR-013 are absent from the tree, and no live citation or
  cross-reference to either survives. Dated records that describe the
  repository as it stood — the reviews under `plans/reviews/` and the
  gap census — keep naming them, which is what a dated record is for;
- the adoption-parameter table matches the checker's actual owners,
  designated classes, and index documents;
- the checker admits a hyphenated area and rejects a hyphenated kind;
- the corpus-wide label check passes in continuous integration.

Every item holds as of this record.
