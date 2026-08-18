# ADR-020: Adoption of the Environment-Kind Registry

**Status:** Decided and implemented; both kind-token migration rounds
recorded below are complete
**Scope:** The kind segment of every label minted in this repository,
and the checker's kind vocabulary
**Adopts:** the third edition of the corrected draft archived at
[plans/drafts/environment-kinds.md](../plans/drafts/environment-kinds.md)
as the normative kind registry
**Names as acceptee:** this repository, the acceptee A of the adopted
registry, owning its local extensions, evidence, statuses, and companion
register
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

An externally authored registry classifying attested environment names
into kinds was accepted, audited, corrected, and archived verbatim. The
gap census compared it against every kind in use and found forty-three
assigned identically, a short list divergent, and a longer list absent.

The edition in force is the author's third, swapped in under backlog
DI-002b. Its generator-derived headline table reports 333 names over 349
rows and 208 kinds, with three declared hybrids and four device classes;
this sentence quotes those counts rather than fixing them. The edition
adds machinery an adopting corpus must answer for: an acceptee owning
the extensions, the evidence, the statuses, and a generated companion
register; an attestation status at each row, daggered where the
edition's evidence is borderline; and homonymy derived from the
effective relation rather than declared. The sections below answer for
each.

---

## Decision · `dec:kinds:adoption`

The repository adopts the archived registry as its kind vocabulary.

**Rule (Normative source)** · `rule:kinds:normative-source`

The draft is the registry. This record does not restate the catalogue;
it records only this repository's migration, its local extensions, and
the sense it fixes for one ambiguous word.

**Rule (Acceptee)** · `rule:kinds:acceptee`

This repository is the acceptee named by this adoption, and the only
one. It owns the extension set recorded below, which the registry writes
X_A; the evidence for it, held first-hand in the corpus; the attestation
statuses it assigns; and the companion register an acceptee owes. The
adopted component of the evidence base is the edition's own, taken by
reference and not restated. The effective relation is therefore C_A, the
registry's rows together with X_A; an extension is never a row of the
registry, and becomes one only if a later edition incorporates it.

**Rule (Extension entry)** · `rule:kinds:extension-entry`

A kind the registry does not carry is a local extension, an entry of
X_A. It enters below by the same decision that introduces it, and it
must be distinct — as a token and as a concept — from every registry
entry. A kind naming a genre the registry already classifies is not an
extension but a defect, and is migrated. An entry records a
name-and-kind pair, the sense that pair carries here, and its
first-hand evidence: an occurrence in this corpus, located.

---

## Migration record · `tab:kinds:migration`

The census flagged eight tokens as conflicting. Each was adjudicated
against the **concept** it names, not against the spelling of its token,
because the registry classifies genres and not words. A token collides
when the registry assigns the same genre a different kind; a token is an
extension when the genre is uncatalogued and the token is unclaimed.

Six collided and were migrated on 2026-08-16. Two did not.

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

**Round two.** The third-edition swap of DI-002b widened the registry
from 148 kinds to 208 and reached two tokens round one had not.

| Repository kind | Registry position | Adjudication | Outcome |
|---|---|---|---|
| `mthm` | Meta-theorem is `metathm` | Same genre, two spellings of one abbreviation. The occurrences were the gap census's citations of the calculus's own meta-theorems, which the third edition mints under the registry token | migrated to `metathm` in the census rewrite that accompanied the swap |
| `ver` | Version and Revision are `ver`; Verification, Check, and Sanity check are `verif` | **Different genre, colliding token.** The three mints are walkthrough anchors of the attestation verification appendix — verifications, not version statements — and the registry's own distinctness note pairs `ver` and `verif` as a near-miss to be kept apart | migrated to `verif` |

The `ver` migration moved three mints and the generated attestation register
that follows them, six sites in two files and no other occurrence in the
tree. Its identity consequence was measured, not assumed, and the
measurement is recorded in the backlog: the anchor-set recipe was
reproduced against the standing pin, recomputed after the rename, and
returned the same value, because the realization contract cites the
walkthrough section and none of these three sub-anchors. Nothing was
re-pinned.

Two round-one decisions the third edition could have unsettled did not
move: Postcondition is still `postc`, so the migration of `post` stands
as taken, and Requirement is still `req`, so the repository's `req` is
the registry's own token and needs no decision at all.

**Round three.** The three-part-label migration of 2026-08-18 carried
the last colliding token off the attestation surface.

| Repository kind | Registry position | Adjudication | Outcome |
|---|---|---|---|
| `abs` | Abstract is `abst`, which Synopsis shares | Same genre, two spellings of one abbreviation: the mint is the paper's own abstract environment, exactly what the registry's Abstract names | migrated to `abst` |

The mint is one, in the paper's front matter, and it is not an anchor
the realization contract cites; the anchor-set pin moved in that
migration for a different reason, recorded in the backlog with the
retired value and its reproduction.

---

## Recorded extension set X_A · `tab:kinds:extensions`

The pairs this repository adds to the registry's rows, across the
surfaces this record governs: the planning tree, the decision records,
the documentation tree, and the Rust packages. Each was checked against
the whole registry for token and genre distinctness, and each carries
first-hand evidence — an occurrence in this corpus, located — as the
registry requires of an acceptee's own rows. The count is occurrences of
the token in the tree, mints and citations together, at this record. The
attestation LaTeX surface entered scope with the three-part-label migration,
recorded under the consequences below.

| Name | Kind | Sense | First-hand evidence |
|---|---|---|---|
| Branch | `branch` | A named control-flow branch of a typed source, cited by the tests that cover it | `branch:operations:settle-distribution`, `packages/model/src/ops/settlement.rs:23`; 30 |
| Candidate | `candidate` | A proposed option under evaluation, before selection | `candidate:guide11:explicit-only`, `plans/guides/guide_eleven_concept.md:990`; 59 |
| Error vocabulary | `err` | The enumerated error surface a package presents at its boundary. Near Erratum, `errat`, and a different genre: an erratum corrects a document after publication; this states what a package can return | `err:labels:vocabulary`, `plans/packages/errors/labels.md:1`; 10, one per package |
| Leaf | `leaf` | A terminal node of a decision or derivation tree | `leaf:authorization:cadence-band`, `docs/attestation/realization.md:1121`; 4 |
| Milestone | `milestone` | A named point in a delivery schedule, reached or not | `milestone:realization:crate`, `plans/packages/realization.md:272`; 28 |
| Motto | `motto` | The document's own opening declaration, its thesis in one line, placed once by the front matter. Near two registry rows and neither: Epigraph, `epigraph`, carries another's words; Motto is filed `slogan`, the saying classified by its wording | `motto:attestation:motto`, `papers/attestation/sections/00_title.tex:95`; 1 |
| Obligation | `obl` | A standing obligation on an implementation, discharged by evidence rather than satisfied once. Near Requirement, `req`, and distinct: a requirement states what the system must do, an obligation what its builder must keep proving | `obl:oracle:adversary`, `docs/attestation/realization.md:1650`; 26 |
| Operation | `op` | An operation of the protocol's state machine | `op:realization:compact-ash`, `plans/packages/realization.md:203`; 4 |
| Phase | `phase` | A numbered stage of the delivery plan, with an entry and an exit | `phase:roadmap:cycle`, `plans/phases/11-cycle.md:1`; 16 |
| Pin | `pin` | A committed value, recipe, or reference frozen against drift and enforced by a gate. No registry row names the frozen commitment; Version records what a version comprises, a different act | `pin:pins:denotation`, `docs/attestation/realization.md:127`; 98 |
| Reference | `ref` | A cited external work or upstream artifact | `ref:elements:tapscript`, `plans/reference/elements-tapscript.md:1`; 1 |
| Residual risk | `res` | An accepted and catalogued exposure — R-dust, R-op, R-conv, R-CSV — recorded rather than removed. Result, `result`, is a stated result; the token `res` does not abbreviate it and is assigned to nothing | `res:trust:dust`, `docs/attestation/realization.md:1796`; 33 |
| Task | `task` | A tracked unit of backlog work: kept, statused, cited long after writing, a records-family genre. The registry's Task rows are `exer`, work set for a reader, and `job`, a code asset that runs; both misdescribe a backlog item, and `task` is assigned to nothing | `task:phase2:pilots`, `plans/backlog.md:1231`; 8 |
| Trap | `trap` | A catalogued way to get the design wrong, stated once at its canonical site and returned to by later arguments. Near Pitfall, `warn`, and distinct: a warning cautions in the flow of the text, where a trap is a named, cited entry of a standing register of design errors | `trap:architecture:two-clocks`, `docs/attestation/realization.md:327`; 165 |

**Motto.** The token is unclaimed and the genre uncatalogued, so the
acceptee records the pair on the evidence above, under the attestation
principle: a local deviation stated, not left silent.

**Rows that left the set.** Four tokens the second edition's register
carried are rows of the third edition's registry, so they are no longer
extensions and are struck from X_A with no change to the corpus: `pkg`
(Package), `q` (Question), `req` (Requirement), and `test` (Test). Each
was already the registry's own token in use here; the register was
listing them redundantly. No mint moved.

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

One further word needs the same treatment. The third edition uses
candidate for an attestation status: a pair whose evidence is kept but
which is not admitted to the relation. This repository's `candidate`
kind is unrelated — a proposed design option under evaluation — and
asserts nothing about attestation.

---

## Attestation and homonymy · `rem:kinds:attestation`

This repository accepts the edition's statuses by reference and neither
strengthens nor weakens one. The edition's daggered rows — Yoga,
Meta-question, and Schema in its data-shape sense — stand as borderline
here, and none is in use. Its one candidate is Record read as a
member-bearing aggregate, outside the relation and so outside C_A; this
repository mints no `rec` at all, so nothing here rests on a candidate.
Every pair of X_A is firm on the evidence located above.

Homonymy is derived, not declared. Of the tokens in use, Test carries
`quiz` among the examples and `test` among the assets, and this
repository uses the asset sense throughout, the kind token at each label
saying so. The register presenting Hom(C_A) in full is generated at
[plans/labels/attestation.md](../plans/labels/attestation.md): sixteen
names, thirty-four pairs. Task is there because this repository's `task`
joins the registry's `exer` and `job` under that name — a homonymy of
the effective relation that exists only here, which is why the corpus
consults its own register and never another's. Motto joins it by
decision: the entry above sets this corpus's `motto` beside the
registry's `slogan` under that name, and the kind token says which
sense a label means.

## Consequences · `rem:kinds:consequences`

The kind segment now has a meaning a reader can look up, and the
checker's vocabulary is a stated list rather than an accident of which
owner was implemented first.

The migration was a rename of label values, not a re-presentation, so
the presentation-invariance meta-theorem does **not** cover it, and the
identities that hash label strings moved with it. The generated
registers were regenerated; the attestation anchor-set pin was recomputed
over the same thirty-eight anchors under new names; and because welded
witness semantic tags carry renamed labels, the architecture semantic
hash and the deployment-profile identity were re-pinned, with the
document's welded appendix and masthead re-welded to the regenerated
manifest. Each was reviewed as a source correction rather than accepted
as formatting, which is what the calculus demands of a rename. The
recomputation recipe was checked by reproducing the retired pin exactly
before the new one was taken.

Backlog DI-003 closed two of the three narrownesses this record opened.
The registry is now the checker's own kind vocabulary, welded in both
directions to the documents the tables were extracted from, so it
cannot go stale silently. And the companion attestation register the
third edition requires of an acceptee is built: its evidence and status
rows stand under the recorded ordering with Hom(C_A) presented as a
view of the same base, derived on each run from the extension table
above, the registry's Convention tables, and a mint census of the
corpus, so the evidence for X_A is maintained rather than stated once.

The narrowness is closed. Of the four tokens the attestation LaTeX surface
carried unadjudicated, `ver` was settled in round two above and `motto`
by the X_A entry recorded here; the three-part-label migration settled
the last two. `abs` migrated to the registry's own `abst`, the kind
Abstract carries. `invest` is adjudicated by having nothing to
adjudicate: no label mints it, and its macro was moved to the
three-part form with the rest, so the token enters the corpus only by a
first use — which now fails the check until a decision admits the kind,
which is the fail-closed answer this record wants. Every attestation kind
in use is therefore registry-carried or X_A-recorded, and that surface
is enforced rather than reported: the checker's reported-only arm is
deleted, and an unregistered kind is an error whoever mints it.

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
- every kind in use is either carried by the registry or recorded in
  X_A, and every X_A entry carries a located first-hand occurrence;
- no extension collides with a registry token or a registry genre;
- exactly one acceptee is named, and it owns the extensions, the
  evidence, and the statuses;
- no pair in use is a candidate, and no edition status is weakened;
- the checker's compiled kind lists carry the migrated tokens and not
  the retired ones;
- the corpus-wide label check passes in continuous integration.

Every item holds as of this record.

The registry's own adoption gate carried one item this record could not
discharge: the companion attestation register, generated, maintained by
regeneration, and presenting Hom(C_A). DI-003 built it, and the claim
that the evidence for X_A is maintained rather than stated once now
holds.

That gate carries one further item this record names here rather than
leaving implied. It asks that every participating authored head
validate by exactly one exact pair or one reduction. The check is now
built, over the Markdown owners this record governs: the decision
records and the authored planning tree, the verbatim archive excluded
as another author's heads and no such head elsewhere. The one hundred
and forty-six environment heads under `adr/` and `plans/` all carry
catalogued pairs; the hand reading is now a check.
