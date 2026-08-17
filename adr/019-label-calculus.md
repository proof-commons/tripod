# ADR-019: Adoption of the Label Calculus

**Status:** Decided and implemented for the authorship warrant species,
with the derivation species and the rest of the checker re-engineering
tracked as backlog DI-003
**Scope:** Every authored prose and code source of this repository, and
the labels checker that enforces the graph over them
**Adopts:** the third edition of the corrected draft archived at
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

The edition in force is the author's third, swapped in under backlog
DI-002b. It rebuilds the calculus on one Mint rule discharged by either
of two warrants — an authorship, the recorded choice of an owner's
authors, or a derivation, the present facts of an asset a registered
profile covers — and grows the adoption parameters from three to seven.
The parameter table below is restated accordingly. The two warrant
species are the substantive addition: the repository's mints all stand
on the authorship species today, and the machinery that would admit the
other is unbuilt.

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

The calculus is parametric in seven data. This repository fixes them as
follows, and a change to any row enters only by a new record.

| Parameter | Fixing |
|---|---|
| Signature Σ: owner prefixes | `A` the specification; `RZ` the realization contract; `PLAN` the planning tree; `DOC` repository documentation; `MODEL` the model crate; `ADRNNN` one owner per numbered record, derived from the filename and never written at a mint. One further owner per Cargo package. The package owners have no readable import prefix today; registering them is backlog DI-003, and until then they are citeable only within themselves |
| Owner partition Ω | Fixed by tree location, total on the carrier: `papers/attestation/main.tex` with its sections to `A`; `docs/attestation/realization.md` to `RZ`; each `adr/NNN-*.md` to its own `ADRNNN`; the rest of `plans/` to `PLAN` and the rest of `docs/` to `DOC`; `packages/model/src/` to `MODEL`; each remaining `packages/*/src/` to that package's owner. Version-control internals, build and dependency directories, and generated artifacts are outside the carrier |
| Profile signature Π | **Empty.** No inventory profile is registered, so no kind is warranted by derivation and every mint in the corpus stands on authorship. The first profile this repository invites is a test profile over the model crate's cases; registering it is backlog DI-003, and the decision that registers it claims its kind in the same commit, as the draft requires |
| Reserved kinds K | **Empty**, and necessarily so while Π is empty. A kind reserved in K that no profile governs admits neither warrant rule, so its every bare occurrence would be a hard failure by warrant totality: a nonempty K under an empty Π would reserve kinds no one could use. K grows only alongside the profile that governs it, and the kinds ADR-020 catalogues are not thereby reserved — that registry fixes what a kind means, never which authority warrants it |
| Designated typed-data classes | Architecture witness semantic tags, and the exported clause identifiers of the architecture manifest. Both target the `R13` owner. No third class is designated |
| Citation-index designations | `plans/labels/specification.md` over the `A` owner, and `plans/labels/realization.md` over the `RZ` owner. Both are generated registers and participate in nothing they index |
| Scanned-region recognition | Markdown: authored prose outside fenced blocks and double-backtick spans. Rust: line, block, and documentation comments, with string and character literals and fenced examples inside documentation comments excluded. LaTeX: authored body text with percent comments stripped, the label carried by the label macro. No other language is scanned |

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

The narrownesses stand and are honest rather than repaired here. The
synthetic-citation and anchor-harvest mechanisms are each one correct
hardwired instance of their rule rather than the rule. The checker
implements the authorship warrant species only: it has no notion of a
profile, a census, a standard place, or a derived label, so the
derivation rule, inventory discipline, and the half of warrant totality
that governs reserved kinds are unimplemented — vacuously satisfied
today, because Π and K are both empty, and unimplemented all the same.
The near-miss warnings the draft asks for are not emitted. Widening all
of this, along with the single participation scanner and the registered
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

## Adoption gate · `gate:labels:adoption`

This gate carried the implementation name until the draft's third
edition minted that same label text for its own implementation gate;
the two are distinct gates in distinct owners, and one text naming both
is a hazard for readers even where the owner-keyed registries keep it
from being a duplicate mint, so this record's gate was renamed to
adoption and its citations retargeted in the commit that swapped the
draft.

Adoption holds when:

- the draft is archived, cited as normative here, and restated nowhere;
- ADR-012 and ADR-013 are absent from the tree, and no live citation or
  cross-reference to either survives. Dated records that describe the
  repository as it stood — the reviews under `plans/reviews/` and the
  gap census — keep naming them, which is what a dated record is for;
- the adoption-parameter table fixes all seven parameters and matches
  the checker's actual owners, partition, designated classes, index
  documents, and scanned regions;
- Π and K are empty together, so that no kind is reserved without a
  profile to govern it;
- the checker admits a hyphenated area and rejects a hyphenated kind;
- the corpus-wide label check passes in continuous integration.

Every item holds as of this record.
