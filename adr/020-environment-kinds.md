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
DI-002b. Its own generator-derived headline table reports 333 names over
349 rows and 208 kinds, with three declared hybrids and four device
classes; the counts are derived from its tables and hand-maintained
nowhere, this sentence quoting them rather than fixing them. The edition
also adds machinery an adopting corpus must answer for: it names an
acceptee who owns the local extensions, the evidence, the statuses, and
a generated companion register; it prints an attestation status at each
row, marked by a dagger where the edition's evidence is borderline; and
it derives homonymy from the effective relation rather than declaring
it. The sections below answer for each.

---

## Decision · `dec:kinds:adoption`

The repository adopts the archived registry as its kind vocabulary.

**Rule (Normative source)** · `rule:kinds:normative-source`

The draft is the registry. This record does not restate the catalogue;
it records only this repository's migration, its local extensions, and
the sense it fixes for one ambiguous word.

**Rule (Acceptee)** · `rule:kinds:acceptee`

This repository is the acceptee named by this adoption, and the only
one. It owns the local extension set recorded below, which the registry
writes X_A; the evidence for it, held first-hand in the corpus itself;
the attestation statuses it assigns; and the companion register the
registry requires of an acceptee. The adopted component of the evidence
base is the edition's own, taken by reference here and not restated.

The effective relation is therefore C_A, the registry's own rows
together with X_A. A local extension is never a row of the registry, and
becomes one only if a later edition expressly incorporates it.

**Rule (Extension entry)** · `rule:kinds:extension-entry`

A kind the registry does not carry is a local extension, an entry of
X_A. It enters the register below by the same decision that introduces
it, and it must be distinct — as a token and as a concept — from every
registry entry. A kind naming a genre the registry already classifies is
not an extension but a defect, and is migrated. An entry records a
name-and-kind pair, the catalogued sense that pair carries here, and the
first-hand evidence for it: an occurrence in this corpus, located.

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

The `ver` migration moved three mints in the attestation verification
appendix and the generated attestation register that follows them, six sites
in two files and no other occurrence in the tree. Its identity
consequence was measured rather than assumed, and the measurement is
recorded in the backlog: the anchor-set recipe was first reproduced
against the standing pin, then recomputed after the rename, and returned
the same value, because the realization contract cites the walkthrough
section and none of these three sub-anchors. Nothing was re-pinned.

Two round-one decisions the third edition could have unsettled did not
move. Postcondition is still a registry row carrying `postc`, so the
migration of `post` stands as taken. Requirement is still `req`, so the
repository's `req` remains the registry's own token and needs no
decision at all — it leaves X_A only because it was never an extension.

---

## Recorded extension set X_A · `tab:kinds:extensions`

The pairs this repository adds to the registry's rows, across the
surfaces this record governs: the planning tree, the decision records,
the documentation tree, and the Rust packages. Each was checked against
the whole registry for token distinctness and genre distinctness, and
each carries first-hand evidence — an occurrence in this corpus, located
— as the registry requires of an acceptee's own rows. The count is the
occurrences of the token in the tree, mints and citations together, at
the time of this record. The attestation LaTeX surface is not yet in scope
and carries three further tokens, named under the consequences below.

| Name | Kind | Catalogued sense | First-hand evidence |
|---|---|---|---|
| Branch | `branch` | A named control-flow branch of a typed source, headed at the branch and cited by the tests that cover it | `branch:operations:settle-distribution` at `packages/model/src/ops/settlement.rs:23`; 30 occurrences |
| Candidate | `candidate` | A proposed option under evaluation, stated so that a later decision can select among a stated field | `candidate:guide11:explicit-only` at `plans/guides/guide_eleven_concept.md:990`; 59 occurrences |
| Error vocabulary | `err` | The enumerated error surface a package presents at its boundary. Near the registry's Erratum, `errat`, and a different genre: an erratum corrects a document after publication, where this states what a package can return | `err:labels:vocabulary` at `plans/packages/errors/labels.md:1`; 10 occurrences, one per package |
| Leaf | `leaf` | A terminal node of a decision or derivation tree, cited where the path through it is argued | `leaf:authorization:cadence-band` at `docs/attestation/realization.md:1121`; 4 occurrences |
| Milestone | `milestone` | A named point in a delivery schedule, reached or not | `milestone:realization:crate` at `plans/packages/realization.md:272`; 28 occurrences |
| Obligation | `obl` | A standing obligation on an implementation, discharged by evidence rather than satisfied once. Near the registry's Requirement, `req`, and distinct: a requirement states what the system must do, an obligation states what its builder must keep proving | `obl:oracle:adversary` at `docs/attestation/realization.md:1650`; 26 occurrences |
| Operation | `op` | An operation of the protocol's state machine, the unit a transaction performs | `op:realization:compact-ash` at `plans/packages/realization.md:203`; 4 occurrences |
| Phase | `phase` | A numbered stage of the delivery plan, with an entry and an exit | `phase:roadmap:cycle` at `plans/phases/11-cycle.md:1`; 16 occurrences |
| Pin | `pin` | A committed value, recipe, or reference frozen against drift and enforced by a gate. No registry row names the frozen commitment; Version records what a version comprises, which is a different act | `pin:pins:denotation` at `docs/attestation/realization.md:127`; 98 occurrences |
| Reference | `ref` | A cited external work or upstream artifact, summarized for use here | `ref:elements:tapscript` at `plans/reference/elements-tapscript.md:1`; 1 occurrence |
| Residual risk | `res` | An accepted and catalogued exposure — R-dust, R-op, R-conv, R-CSV — recorded rather than removed. The registry's Result, `result`, is a stated result; the token `res` does not abbreviate it and is assigned to nothing | `res:trust:dust` at `docs/attestation/realization.md:1796`; 33 occurrences |
| Task | `task` | A tracked unit of backlog work: kept, statused, and cited long after writing, a records-family genre. The registry's Task rows are `exer`, work assigned to a reader, and `job`, a code asset that runs; both misdescribe a backlog item, and the token `task` is assigned to nothing | `task:phase2:pilots` at `plans/backlog.md:1231`; 8 occurrences |
| Trap | `trap` | A catalogued way to get the design wrong, stated once at its canonical site and returned to by every later argument that could fall into it. Near the registry's Pitfall, `warn`, and distinct in the same way a Refrain is distinct from a Remark: a warning is a caution in the flow of the text, where a trap is a named, cited entry of a standing register of design errors | `trap:architecture:two-clocks` at `docs/attestation/realization.md:327`; 165 occurrences |

**Rows that left the register.** The second edition's register carried
four tokens the third edition's registry now carries as rows of its own,
so they are no longer extensions and are struck from X_A without any
change to the corpus: `pkg` (Package, among the assets), `q` (Question),
`req` (Requirement), and `test` (Test, among the assets). Each was
already the registry's own token in use here; the register was simply
listing them redundantly, and the third edition settled it. No mint
moved.

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
kind is unrelated — a proposed design option under evaluation — and no
occurrence of it asserts anything about attestation. The status word
appears in this record only where the attestation section says so.

---

## Attestation, homonymy, and the companion register · `rem:kinds:attestation`

The third edition prints a status at each row and makes the acceptee
answer for it. This repository accepts the edition's own statuses by
reference, strengthens none of them, and weakens none. The edition's
daggered rows — Yoga, Meta-question, and Schema in its data-shape sense
— stand as borderline here, and none of the three is in use in this
corpus. The edition's one candidate is Record read as a member-bearing
aggregate; it lies outside the relation and so outside C_A, and this
repository mints no `rec` at all, so nothing here rests on a candidate.
Every pair of X_A is firm on the first-hand evidence located in the
register above.

Homonymy is derived, not declared, and two of the pairs above sit under
homonymous names. Test carries `quiz` among the examples and `test`
among the assets; this repository uses the asset sense throughout, and
the kind token at each label says so. Structure carries three senses in
the registry and this repository mints none of them. The register that
would present Hom(C_A) in full is generated, and this repository has not
built its generator; the gap is recorded under the consequences.

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

Three narrownesses remain. The kind vocabulary is still split between a
realization list and a model list compiled into Rust, with no vocabulary
at all for the prose owners; making the registry itself the checker's
vocabulary is backlog DI-003. The companion attestation register the
third edition requires of an acceptee — its evidence and status rows
under a total recorded ordering, with Hom(C_A) presented as a view of
the same base — is not built; it is a generated register with a
generator, which is DI-003's work, and until it exists the evidence for
X_A is the located occurrences in this record rather than a maintained
artifact. And the LaTeX label surface of the attestation paper carries three
further tokens the census did not reach — `motto`, a true collision with
the registry's Motto, `slogan`; `invest`; and `abs` — which the same
treatment will settle when that surface enters scope. That surface's
fourth unadjudicated token, `ver`, was reached and settled in round two
above; the three named here remain open, and this record adjudicates
none of them.

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

The registry's own adoption gate carries one further item this record
cannot yet discharge: the companion attestation register, generated and
maintained by regeneration, presenting Hom(C_A) as a view of the same
evidence base. It is unbuilt, is recorded as such under the consequences
above, and is tracked as backlog DI-003. Nothing in the corpus depends
on it today; what depends on it is the claim that the evidence for X_A
is maintained rather than stated once.
