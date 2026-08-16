# A Calculus of Documentation and Source Labels

This document lays down, self-containedly, the reference graph of an
authored corpus — prose documents and source code — as a small calculus:
a Language of labels and a Grammar of their occurrences, a Signature of
owners, Judgments asserting minting, resolution, and participation,
Inference rules deriving them, and Invariants every derivation must
satisfy. Meta-theorems record what
holds of the calculus; Caveats bound its authority; five rejected
Ansätze delimit it negatively; a single Postcondition gates
implementation. The calculus is parametric in three data — the
Signature, the typed-data classes that cite synthetically, and the
documents that maintain citation indexes — and a corpus adopts it by
fixing these and running a checker.

The document practices the discipline it defines: wherever it lives, it
is a source in the corpus it governs. The label at each heading or
environment head is that environment's mint; a parenthesized label in
running text is a same-owner citation; material in fenced blocks and
double-backtick spans is displayed without participating. Environments
carry no numbers: replacing numbering is part of what a label is for,
so the mint at each head is the sole name of its environment, and this
document refers to its own environments only by citation. A
Demonstration is part of the environment it closes and mints nothing of
its own.

## Syntax · `sec:labels:syntax`

**Language (Labels)** · `lang:labels:label-language`
A label is a colon-joined triple of kind, area, and name. Kind and area
are words over lowercase letters and digits; the name may hyphenate such
words. The kind alphabet is open; this document employs `sec`, `lang`,
`gram`, `sig`, `judg`, `inf`, `inv`, `mthm`, `cav`, `ansatz`, and
`postc`, and every label it mints has area `labels`. A label occurs in
exactly one of three forms, in either of two concrete syntaxes — one
for prose, one for code comments:

```text
label       ::=  kind ":" area ":" name
kind, area  ::=  word                 name  ::=  word ("-" word)*
word        ::=  [a-z0-9]+            PREFIX ::=  [A-Z][A-Z0-9]*

                   mint        same-owner citation   imported citation
Prose occurrence:   `label`     (`label`)             (`[PREFIX-label]`)
Code occurrence:    ´label´     (´label´)             (´[PREFIX-label]´)
```

The three forms name, in order, a mint, a same-owner citation, and an
imported citation; their semantics is fixed by the rules of
(`sec:labels:inference-rules`). For illustration, in a corpus whose
specification is registered under ``SPEC``, a tokenizer's defining
comment, a same-owner design note, and a document of another owner
might carry:

```text
mint, in a code comment:        ´def:parser:tokenizer´
citation, prose, same owner:    (`def:parser:tokenizer`)
citation, from another owner:   (`[SPEC-def:parser:tokenizer]`)
```

**Grammar (Well-formed occurrences)** · `gram:labels:well-formed`
Exactly the displayed productions generate occurrences, and an
occurrence is atomic: forms do not nest, and no other bracketing,
prefixing, or spacing is an occurrence. A backtick or acute span that is
not label-shaped is ordinary text and can never become a label by
accident.

**Signature (Owners)** · `sig:labels:owners`
The Signature Σ is a partial map from registered prefixes to owners,
fixed at adoption. Owners partition the corpus. A family of numbered
records may register one owner per record, the prefix derived from the
filename and never written at a mint. Σ is closed: a new prefix enters
it only through a recorded decision. Local references within one owner
need no prefix. For illustration:

| Source | Owner prefix |
|---|---|
| the specification | `SPEC` |
| the user guide | `GUIDE` |
| each numbered record `records/NNN-*.md` | `RECNNN` |
| each code package | one prefix per package, derived from the package name |
| working notes | `NOTES` |

## Judgments · `sec:labels:judgments`

**Judgment (Minting)** · `judg:labels:minting`
Form: O ⊢ o ⇓ ℓ — "in owner O, occurrence o mints label ℓ." Minting
judgments are formed only over occurrences with part(o)
(`judg:labels:participation`), drawn from the carrier: every authored
prose and code source of the corpus, excluding version-control
internals, build and dependency directories, archived and vendored
trees, and generated artifacts. Generated prose may be read as
publication syntax but forms no minting judgment. A file with no
occurrences is vacuously in good standing.

**Judgment (Resolution)** · `judg:labels:resolution`
Form: c ↦ ⟨O, ℓ⟩ — "citation occurrence c resolves to the mint of ℓ in
owner O," the pair naming its mint uniquely by
(`inv:labels:unique-mint`). The judgment holds exactly when derived by a
rule of (`sec:labels:inference-rules`); there are no other derivations.

**Judgment (Participation)** · `judg:labels:participation`
Form: part(o). In prose, occurrences in authored text participate;
fenced blocks and double-backtick spans do not — a token shown but not
meant is placed in one of these. A generated register participates in
nothing it indexes: it is derivative output, excluded from the source
graph while its exact bytes remain checked. In code, only comments and
documentation comments are scanned; string and character literals and
fenced documentation examples are not, and an opening acute delimiter
without its close in scanned text is a hard failure. The defining
source mints; indexes and catalogs cite.

## Inference rules · `sec:labels:inference-rules`

**Inference rule (Local mint)** · `inf:labels:local-mint`

```text
part(o)      o is bare with label ℓ      owner(o) = O
─────────────────────────────────────────────────────  Mint
                      O ⊢ o ⇓ ℓ
```

A bare participating occurrence mints its label in the owner of its own
source — in that owner and never another.

**Inference rule (Local citation)** · `inf:labels:local-citation`

```text
part(c)      c = (ℓ)      owner(c) = O      O ⊢ o ⇓ ℓ
─────────────────────────────────────────────────────  Cite
                     c ↦ ⟨O, ℓ⟩
```

A parenthesized bare occurrence cites within its own owner and resolves
anywhere within that owner, across files and across the two concrete
syntaxes. It never resolves into another owner. In Cite, Import, and
Synthetic the minting premise is read existentially: some occurrence of
the named owner mints the label, unique by (`inv:labels:unique-mint`).

**Inference rule (Import)** · `inf:labels:import`

```text
part(c)      c = ([P-ℓ])      owner(c) = O
Σ(P) = O′      O′ ≠ O         O′ ⊢ o ⇓ ℓ
──────────────────────────────────────────  Import
                c ↦ ⟨O′, ℓ⟩
```

Side conditions: the prefix is registered by (`sig:labels:owners`), and
the named owner differs from the current one — a self-qualified import
is underivable. The bracket is the syntax of the ownership boundary and
nothing else: the authority of the imported fact is a property of O′,
never of the bracket.

**Inference rule (Synthetic citation)** ·
`inf:labels:synthetic-citation`
A corpus may designate classes of typed data strings — identifiers
carried in schemas, manifests, or machine-checked design artifacts —
as citing a target owner. Such strings are data, not comment syntax.
Each derives a synthetic citation of a mint of its target owner T:

```text
a is of a designated class targeting T      T ⊢ o ⇓ value(a)
────────────────────────────────────────────────────────────  Synthetic
                     a ↦ ⟨T, value(a)⟩
```

No such string is ever a source mint. The designation of which fields
participate is among the adoption parameters and remains authoritative.

**Inference rule (Anchor harvest)** · `inf:labels:anchor-harvest`
A corpus may designate citation indexes: a document D maintains a
committed index of its citations into an upstream owner U.

```text
c ∈ body(D)      part(c)      c ↦ ⟨U, ℓ⟩
─────────────────────────────────────────  Anchor
             ℓ ∈ Anchors(D, U)
```

Side conditions: body(D) excludes the citation-index section itself
together with all nonparticipating material; the harvest never writes. The committed index
presents exactly the distinct set Anchors(D, U), and any pinned hash of
the anchor set is computed from that set alone. Attribution and
commentary columns of the index remain editorial prose.

## Invariants · `sec:labels:invariants`

**Invariant (Unique mint)** · `inv:labels:unique-mint`
For every owner O and label ℓ there is at most one occurrence o with
O ⊢ o ⇓ ℓ. A second bare occurrence is a violation, reported with both
locations — never a harmless repeat.

**Invariant (Total resolution)** · `inv:labels:total-resolution`
Every participating citation, and every designated typed-data string of
(`inf:labels:synthetic-citation`), is the conclusion of exactly one
rule, with exactly one mint. Unknown owners, unresolved citations, malformed
forms, non-parenthesized imports, and bracket-free cross-owner tokens
all fail.

**Invariant (Two-pass adequacy)** · `inv:labels:two-pass`
Derivation is staged. First, every carrier source is harvested and the
minting registries of all owners completed, duplicates failing by
(`inv:labels:unique-mint`); only then is any resolution judgment
derived, against the completed registries.

## Metatheory · `sec:labels:metatheory`

**Meta-theorem (Order independence)** ·
`mthm:labels:order-independence`
Derivability of every judgment is independent of the order in which
carrier sources are traversed, and forward references across files
resolve.
Demonstration. By (`inv:labels:two-pass`) every resolution consults
registries already completed from the whole carrier, so no derivation
can observe traversal order. ∎

**Meta-theorem (No self-support)** · `mthm:labels:no-self-support`
No token can keep itself in an anchor set from the index alone, and
removing a label's last body citation stales the committed index.
Demonstration. Anchors(D, U) is derived by
(`inf:labels:anchor-harvest`) from body citations only, and the index
section lies outside body(D); an index row therefore derives nothing,
while exactness of presentation fails the moment the set shrinks. ∎

**Meta-theorem (Presentation invariance)** ·
`mthm:labels:presentation-invariance`
A migration that re-forms occurrences while preserving every label value
changes presentation, not denotation: every minting and resolution
judgment stands, every version identifier and content hash of the
governed artifacts stands, and the document revision alone advances.
Demonstration. Labels are the denotation and occurrence form the
presentation; if no label value changes, the registries and every
derivation over them are unchanged, hence so is every hash computed from
them. ∎

## Caveats · `sec:labels:caveats`

**Caveat (Non-normativity)** · `cav:labels:non-normativity`
The calculus is documentation, never semantic input. Labels are
navigation and provenance metadata, and membership in a registry
confers no identity outside the graph. No compiler, build system,
packager, or release tool consumes the graph; the checker and the
register generator are its only consumers. Renaming a label updates all
citations in the same commit and bumps no version of the software the
corpus documents, and a migration that changes the set of labels
carried by a governed artifact is reviewed as a source correction,
never accepted silently as formatting.

**Caveat (Coexistence and diagnostics)** · `cav:labels:coexistence`
Two owners may mint the same label text without collision: ownership
disambiguates, and a citation transfers no ownership. Traversal failures
are diagnostics — an unreadable tree must never become an empty carrier.
Scoped register generation ignores unrelated owners' defects, while the
corpus-wide check still validates everything.

## Rejected Ansätze · `sec:labels:rejected-ansaetze`

**Ansatz (Flat namespace)** · `ansatz:labels:flat-namespace`
Take one global namespace: no Signature, every match resolves. Then the
side condition of (`inf:labels:import`) vanishes, and a working note's
citation resolves into any owner's mint with no declared crossing.
Rejected.

**Ansatz (First mint wins)** · `ansatz:labels:first-mint-wins`
Order the mints and keep the first. Then (`inv:labels:unique-mint`) is
abandoned, and an incidental bare span silently becomes — or moves — a
label's conceptual home, with no diagnostic. Rejected.

**Ansatz (Code-font delimiters in code)** ·
`ansatz:labels:code-font-delimiters`
Scan the ordinary code-font spans of documentation comments for labels.
Then (`judg:labels:participation`) is undecidable without a fragile
label-like heuristic, and ordinary documentation makes false mints.
Rejected.

**Ansatz (Unchecked local labels)** ·
`ansatz:labels:unchecked-locals`
Exempt one class of sources — say the working notes — from resolution.
Then (`inv:labels:total-resolution`) fails on that class and dangling
references hide real defects, while owner-aware cross-file resolution
has already made totality cheap. Rejected.

**Ansatz (Participating registers)** ·
`ansatz:labels:participating-registers`
Let generated registers mint and cite. Then (`judg:labels:minting`)
holds of derivative text, and a register row sustains its own membership
after the last body citation is gone, contradicting
(`mthm:labels:no-self-support`). Rejected.

## Postcondition · `sec:labels:postcondition`

**Postcondition (Implementation)** · `postc:labels:implementation`
After implementation, all of the following hold:

- the three occurrence forms parse in both concrete syntaxes;
- duplicate mints fail with both locations in every owner;
- every local and imported citation resolves to exactly one mint;
- bracket-free cross-owner tokens and self-qualified imports fail;
- resolution is independent of traversal order;
- every designated typed-data string resolves as a synthetic citation
  of its target owner;
- generated registers are nonparticipating, current, and deterministic;
- every pinned anchor-set hash derives from body citations only;
- traversal failures surface as diagnostics;
- scoped register generation ignores unrelated owners' defects;
- this document carries exactly one mint per environment and no
  environment numbering, and every citation in the corpus resolves;
- the corpus-wide check passes in continuous integration.
