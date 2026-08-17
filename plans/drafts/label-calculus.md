# A Calculus of Documentation and Source Labels

This document lays down, self-containedly, the reference graph of a corpus — prose documents and source code — as a small calculus with one minting story and two authorities. Every label exists on a _warrant_: an authorship, the recorded choice of an owner's authors naming a concept, or a derivation, the present facts of an asset computed by a registered profile. One Mint rule admits both; two warrant rules discharge it; and citations consume mints without ever seeing warrants, which is why one graph holds both authorities. Judgments assert warranting, minting, derivation, resolution, and participation; Invariants govern every derivation; Meta-theorems record what holds, chief among them that a citation breaks exactly when its target's warrant lapses; Caveats bound the calculus's authority; eight rejected Ansätze delimit it negatively; and a single Gate blocks implementation until met. The calculus is parametric in seven data — the Signature of owners, the owner partition, the profile signature, the reserved kinds, the typed-data classes that cite synthetically, the documents that maintain citation indexes, and the scanned-region recognition: per language, which comment and documentation-comment regions are scanned — and a corpus adopts it by fixing these and running a checker. This document is self-contained and cites only itself; where an adopting corpus uses it beside other disciplines, their alignment is fixed by the corpus's recorded adoption decisions, not by this text. Acceptance of this document presupposes acceptance of no other document: where another discipline's artifact is consumed — a set of reserved kinds, an identity recipe, a format's declared environment classes — this calculus consumes the artifact as adoption data and asks nothing of its provenance.

The document practices the discipline it defines: wherever it lives, it is a source in the corpus it governs. The label at each heading or environment head is that environment's mint; a parenthesized label in running text is a same-owner citation; material in fenced blocks and double-backtick spans is displayed without participating. The document title is publication metadata, not an environment head; it mints nothing and participates in nothing. Environments carry no numbers: replacing numbering is part of what a label is for, so the mint at each head is the sole name of its environment, and this document refers to its own environments only by citation. A Demonstration is part of the environment it closes and mints nothing of its own.

## Syntax · `sec:labels:syntax`

**Language (Labels)** · `lang:labels:label-language`

A label is a colon-joined triple of kind, area, and name. Kind and area are words over lowercase letters and digits; the name may hyphenate such words. The kind alphabet is open; this document employs `sec`, `lang`, `gram`, `sig`, `judg`, `inf`, `inv`, `metathm`, `cav`, `ansatz`, and `gate`, and every label it mints has area `labels`. A label occurs in exactly one of three forms, in either of two concrete syntaxes — one for prose, one for code comments:

```text
label       ::=  kind ":" area ":" name
kind, area  ::=  word                 name  ::=  word ("-" word)*
word        ::=  [a-z0-9]+            PREFIX ::=  [A-Z][A-Z0-9]*

                   mint        same-owner citation   imported citation
Prose occurrence:   `label`     (`label`)             (`[PREFIX-label]`)
Code occurrence:    ´label´     (´label´)             (´[PREFIX-label]´)
```

The three forms name, in order, a mint, a same-owner citation, and an imported citation; their semantics is fixed by the rules of (`sec:labels:inference-rules`). For illustration, in a corpus whose specification is registered under `SPEC`, a tokenizer's defining comment, a same-owner design note, and a document of another owner might carry:

```text
mint, in a code comment:        ´def:parser:tokenizer´
citation, prose, same owner:    (`def:parser:tokenizer`)
citation, from another owner:   (`[SPEC-def:parser:tokenizer]`)
```

**Grammar (Well-formed occurrences)** · `gram:labels:well-formed`

Exactly the displayed productions generate occurrences, and an occurrence is atomic: forms do not nest, and no other bracketing, prefixing, or spacing is an occurrence. A span is logical, never a run of bytes: the source's own structure — a quotation block's markers, a list's continuation indentation, a comment's leaders — is resolved away before spans are determined, so one span may run across lines nowhere contiguous in the file, while no span crosses a boundary that structure itself closes. An occurrence exists only where a span parses completely as one of the three forms; a span that parses as no form is ordinary text, and there is no partially well-formed occurrence. A backtick or acute span that is not label-shaped can never become a label by accident.

**Signature (Owners and partition)** · `sig:labels:owners`

The Signature Σ is a partial map from registered prefixes to owners, fixed at adoption. Owners partition the corpus, and adoption fixes the partition itself: a map Ω from carrier sources and covered assets to owners, total on the carrier, with each registered prefix naming one owner in Ω's range. owner(·) is Ω throughout. A family of numbered records may register one owner per record, the prefix derived from the filename and never written at a mint. Σ is closed under its registered families: a family admits prefixes by its derivation rule, and a new prefix outside every family, or a new family, enters only through a recorded decision. Local references within one owner need no prefix. For illustration:

| Source | Owner prefix |
| --- | --- |
| the specification | `SPEC` |
| the user guide | `GUIDE` |
| each numbered record `records/NNN-*.md` | `RECNNN` |
| each code package | one prefix per package, derived from the package name |
| working notes | `NOTES` |

**Signature (Profiles)** · `sig:labels:profiles`

The profile signature Π is the registered family of inventory profiles, each governing one kind. A profile fixes its kind token; its census — which assets it covers, a covered asset being whatever the recognizing harness, build, or language says it is, and possibly a container such as a module or namespace definition; its classification rule, from which the area derives; its name transformation, from the asset's bare identifier to the name segment; and its standard place — the position where the label is carried: in the asset's header, in its documentation comment, or in the owner's own prose, one choice per profile, several covering profiles stacking their labels there in a fixed order. A kind governed by Π is an _inventory kind_, warranted only by derivation (`inf:labels:derivation-warrant`); every kind outside K is authored, and a kind reserved in K that no profile governs admits neither warrant (`sig:labels:reserved-kinds`). Extending Π is a recorded decision, as for (`sig:labels:owners`), and the decision that extends Π claims its kind: in the same commit, every authored mint of that kind is renamed to an authored kind or retired under (`inf:labels:authorship-warrant`), or superseded by the derivation at the standard place, and every citation follows; a claimed kind with surviving authored mints is a hard failure of the deciding commit (`inv:labels:warrant-totality`), not of the mints. For illustration, a test profile with kind `test`, areas `unit`, `interunit`, and `integration`, and hyphenation of the function identifier:

```text
carried at the standard place of a covered test:
    ´test:integration:decode-roundtrip´
cited from its own package:
    (´test:integration:decode-roundtrip´)
imported by the documentation:
    (`[CODEC-test:integration:decode-roundtrip]`)
```

**Signature (Reserved kinds)** · `sig:labels:reserved-kinds`

Adoption may fix, beside Π, a set K of reserved kinds, each intended for derivation only. Every kind governed by Π (`sig:labels:profiles`) lies in K; a bare occurrence of a reserved kind not governed by any profile is a hard failure awaiting its derivation (`inf:labels:derivation-warrant`), never an authored mint. A corpus that also adopts a registry of kinds populates K from it by its own recorded decision; this calculus consumes the set and asks nothing of its provenance.

## Judgments · `sec:labels:judgments`

**Judgment (Warrant)** · `judg:labels:warrant`

Form: w warrants ℓ at o — "on authority w, the label ℓ may stand bare at occurrence o." A warrant is one of exactly two things: an _authorship_, the recorded choice of the owner's authors, which the occurrence itself embodies; or a _derivation_, the present facts of a covered asset, which no one records because the asset is the record. Warrants attach to kinds, never to media: both concrete syntaxes serve both authorities, so an authored label may stand in a code comment, and a profile may set its standard place in the owner's prose. The line between the authorities is the warrant alone.

**Judgment (Minting)** · `judg:labels:minting`

Form: O ⊢ o ⇓ ℓ — "in owner O, occurrence o mints label ℓ." Minting is warranted occurrence and nothing else (`judg:labels:warrant`). Minting judgments are formed only over occurrences with part(o) (`judg:labels:participation`), drawn from the carrier: every authored prose and code source of the corpus, excluding version-control internals, build and dependency directories, archived and vendored trees, and generated artifacts. A generated region within a carrier source — a committed index or register — remains in the carrier as bytes, checked for exactness, while participating in nothing. Generated prose may be read as publication syntax but forms no minting judgment. A file with no occurrences is vacuously in good standing.

**Judgment (Derivation)** · `judg:labels:derivation`

Form: derive_p(a) = ℓ — "under profile p, asset a derives label ℓ": the kind is p's, the area is p's classification of a, the name is p's transformation of a's bare identifier. The derivation reads these facts and no others. File, module, path, and position never enter it, and no notion of asset identity across change is needed anywhere in the calculus: a moved or renamed asset simply derives a different label, and the difference does the detecting.

**Judgment (Resolution)** · `judg:labels:resolution`

Form: c ↦ ⟨O, ℓ⟩ — "citation occurrence c resolves to the mint of ℓ in owner O," the pair naming its mint uniquely by (`inv:labels:unique-mint`). The judgment holds exactly when derived by a citation rule of (`sec:labels:inference-rules`); there are no other derivations. Citations consume mints, never warrants: no citation can tell on which authority its target stands.

**Judgment (Participation)** · `judg:labels:participation`

Form: part(o). In prose, occurrences in authored text participate; fenced blocks and double-backtick spans do not — a token shown but not meant is placed in one of these. A generated register participates in nothing it indexes: it is derivative output, excluded from the source graph while its exact bytes remain checked. In code, only comments and documentation comments are scanned; string and character literals and fenced documentation examples are not. Which regions count as comments in each language is fixed at adoption as the scanned-region recognition; the judgment consumes it and asks nothing of its provenance. One comment is one logical region, its leaders resolved away (`gram:labels:well-formed`), and one prose block is another; delimiter pairing is settled within a region before any span in it is parsed. The two rules following are therefore scanning preconditions, prior to (`gram:labels:well-formed`), since a span whose boundaries are undetermined cannot be asked whether it parses — neither is a counterexample to the text-never-failure clause of (`inv:labels:total-resolution`), which governs delimited spans alone. In scanned code text the acute belongs to the label syntax and classifies locally: it opens exactly when label-shaped text follows it, an opening acute unclosed when its region ends is a hard failure, and an acute that opens nothing is text — an opening acute declares intent to mint or cite and its loss is an error, while a stray closing acute is overwhelmingly an apostrophe accident. In prose the backtick belongs to the document format, so no such local classification is available: an unpaired backtick leaves its block's spans undefined and is a hard failure of the file, bounded by that block — reported there, with the rest of the file resolved normally, as a traversal failure is (`cav:labels:coexistence`). The prose syntax defers to that format's span rules wherever it defines them. The defining source mints; indexes and catalogs cite.

## Inference rules · `sec:labels:inference-rules`

**Inference rule (Mint)** · `inf:labels:mint`

```text
part(o)    o is bare with label ℓ    owner(o) = O    w warrants ℓ at o
─────────────────────────────────────────────────────────  Mint
                         O ⊢ o ⇓ ℓ
```

Both authorities mint through this one rule; they differ only in how the warrant premise is discharged (`judg:labels:warrant`), and the label's kind decides which of the two warrant rules is admissible.

**Inference rule (Authorship warrant)** · `inf:labels:authorship-warrant`

```text
kind(ℓ) ∉ K      o is the choice of O's authors
────────────────────────────────────────────────  Authorship
              authorship warrants ℓ at o
```

For authored kinds — those outside K, hence neither governed by Π nor reserved to it (`sig:labels:reserved-kinds`) — the occurrence itself is the choice, and the choice is the warrant: granted freely, lapsing only by a recorded decision. The premise is exclusion from K and not merely from Π: a kind reserved in K that no profile governs admits neither warrant rule, and its bare occurrence is the hard failure of (`inv:labels:warrant-totality`), never an authored mint. A renaming updates all citations in the same commit; a retirement removes or re-points them in the same commit. The area of an authored label records the concept's home at creation and never chases prose moves.

**Inference rule (Derivation warrant)** · `inf:labels:derivation-warrant`

```text
kind(ℓ) = kind(p), p ∈ Π      a ∈ census(p)      o at place_p(a)
owner(o) = owner(a)           ℓ = derive_p(a)
────────────────────────────────────────────────  Derivation
              derivation warrants ℓ at o
```

The profile p is drawn from (`sig:labels:profiles`) and the derived label from (`judg:labels:derivation`). Writing the label is attestation, not naming: an occurrence at the standard place whose text differs from the derivation warrants nothing and is a hard failure, not a mint of something else. The owner is the asset's package, never the module — so movement within the package changes nothing, and movement across packages changes the owner.

**Inference rule (Same-owner citation)** · `inf:labels:same-owner-citation`

```text
part(c)      c = (ℓ)      owner(c) = O      O ⊢ o ⇓ ℓ
─────────────────────────────────────────────────────  Cite
                     c ↦ ⟨O, ℓ⟩
```

A citation of the unprefixed form cites within its own owner and resolves anywhere within that owner, across files and across the two concrete syntaxes. It never resolves into another owner. In Cite, Import, and Synthetic the minting premise is read existentially: some occurrence of the named owner mints the label, unique by (`inv:labels:unique-mint`).

**Inference rule (Imported citation)** · `inf:labels:imported-citation`

```text
part(c)      c = ([P-ℓ])      owner(c) = O
Σ(P) = O′      O′ ≠ O         O′ ⊢ o ⇓ ℓ
──────────────────────────────────────────  Import
                c ↦ ⟨O′, ℓ⟩
```

Side conditions: the prefix is registered by (`sig:labels:owners`), and the named owner differs from the current one — a self-qualified import is underivable. The minting premise is existential here too, unique in O′ by (`inv:labels:unique-mint`). The bracket is the syntax of the ownership boundary and nothing else: the authority of the imported fact is a property of O′, never of the bracket.

**Inference rule (Synthetic citation)** · `inf:labels:synthetic-citation`

A corpus may designate classes of typed data strings — identifiers carried in schemas, manifests, or machine-checked design artifacts — as citing a target owner. Such strings are data, not comment syntax. Each derives a synthetic citation of a mint of its target owner T:

```text
a is of a designated class targeting T      T ⊢ o ⇓ value(a)
────────────────────────────────────────────────────────────  Synthetic
                     a ↦ ⟨T, value(a)⟩
```

No such string is ever a source mint. The designation of which fields participate is among the adoption parameters and remains authoritative.

**Inference rule (Anchor harvest)** · `inf:labels:anchor-harvest`

A corpus may designate citation indexes: a document D maintains a committed index of its citations into an upstream owner U.

```text
c ∈ body(D)      part(c)      c ↦ ⟨U, ℓ⟩
─────────────────────────────────────────  Anchor
             ℓ ∈ Anchors(D, U)
```

Side conditions: U ranges over the owners designated upstream of D, and citations into other owners are harvested by no index; body(D) excludes the citation-index section itself together with all nonparticipating material; the harvest never writes. The committed index presents exactly the distinct set Anchors(D, U), ordered bytewise-lexicographically on labels, and any pinned hash of the anchor set is computed from that ordered presentation alone. That ordering fixes a projection and a canonicalization and nothing further: a corpus pinning such a hash owes the remaining terms of its identity — the function, the separation of its role from every other, and the identifier under which consumers recompute — to whatever content-identity discipline it has adopted, this calculus prescribing none (`cav:labels:non-normativity`). Attribution and commentary columns of the index remain editorial prose.

## Invariants · `sec:labels:invariants`

**Invariant (Warrant totality)** · `inv:labels:warrant-totality`

Every mint stands on exactly one warrant, and every kind admits at most one warrant species: kinds in K admit derivation only, and only where a profile governs them; kinds outside K admit authorship only. A reserved kind no profile governs admits none, deliberately. A bare participating occurrence for which no warrant rule is admissible — an inventory-kind token away from any standard place, a reserved-kind token no profile governs (`sig:labels:reserved-kinds`), an authored-kind token whose kind a profile has since claimed — is a hard failure, not a mint.

**Invariant (Unique mint)** · `inv:labels:unique-mint`

For every owner O and label ℓ there is at most one occurrence o with O ⊢ o ⇓ ℓ. A second bare occurrence is a violation, reported with both locations — never a harmless repeat.

**Invariant (Total resolution)** · `inv:labels:total-resolution`

Every participating citation, and every designated typed-data string of (`inf:labels:synthetic-citation`), is the conclusion of exactly one rule, with exactly one mint. Unknown owners, unresolved citations, non-parenthesized imports, and bracket-free cross-owner tokens all fail. A delimited span that parses as no form is text, never a failure, by (`gram:labels:well-formed`), delimiter pairing having been settled first (`judg:labels:participation`); a parenthesized span whose interior is label-shaped but resolves nowhere fails, and never lapses into text. The checker should warn on near-miss spans — label-shaped interiors with wrong casing, brackets, or spacing, and in scanned code text label-shaped backtick spans where an acute was meant — without treating them as occurrences.

**Invariant (Inventory discipline)** · `inv:labels:inventory`

For every profile p of (`sig:labels:profiles`), within each owner: every asset of p's census carries exactly one label of p's kind, at p's standard place; the derivation is injective — two covered assets of one owner never derive one label of one kind, a collision being a naming defect of the assets and surfaced as such; and no label of p's kind occurs without a covered asset — labels do not outlive what they name. Labels of distinct kinds coexist freely on one asset, one facet each.

**Invariant (Two-pass adequacy)** · `inv:labels:two-pass`

Derivation is staged. First, the adoption data are loaded — Σ, Ω, Π, K, the typed-data classes, the index designations, and the scanned-region recognition; then every carrier source is harvested — the minting registries of all owners completed, the censuses and derivations of all profiles computed, duplicates failing by (`inv:labels:unique-mint`); only then is any resolution judgment derived, against the completed registries.

## Metatheory · `sec:labels:metatheory`

**Meta-theorem (Warrant lapse)** · `metathm:labels:warrant-lapse`

Fix Σ and the owner partition (`sig:labels:owners`), Π (`sig:labels:profiles`) and K (`sig:labels:reserved-kinds`), and participation (`judg:labels:participation`); fix a citation that resolves, and let a transition preserve the citation's own text, owner, and participation and preserve (`inv:labels:unique-mint`) and (`inv:labels:inventory`). Then the citation dangles exactly when its target's warrant lapses, and each authority lapses its own way. An authorship lapses only by a recorded renaming or retirement, and the deciding commit re-points or removes the citations it breaks. A derivation lapses when the asset's class, classification, or name changes — and exactly the citations of that facet dangle — when the asset changes package, and exactly the imports under the old package's prefix dangle — or when the asset leaves the census, deletion included: the label, forbidden to outlive its asset (`inv:labels:inventory`), lapses with it, and every citation of it dangles. Moving an asset within its package, or changing only what it does, lapses nothing. Breakage is scoped to the facet or the boundary that moved, and refactoring inside them is free. Demonstration. A mint stands exactly while its warrant holds (`inf:labels:mint`) and stands on that one warrant alone (`inv:labels:warrant-totality`); the derivation (`judg:labels:derivation`) reads class, classification, and bare identifier only, so exactly those changes, and departure from the census — census membership being the Derivation rule's own premise (`inf:labels:derivation-warrant`) — change or remove the derived label, and by (`inv:labels:inventory`) the carried label follows; by (`inv:labels:unique-mint`) the lapsed label has no other mint, so by (`inv:labels:total-resolution`) its citations fail; and ownership enters a citation only through the import prefix. ∎

**Meta-theorem (Order independence)** · `metathm:labels:order-independence`

Derivability of every judgment is independent of the order in which carrier sources are traversed, and forward references across files resolve. Demonstration. By (`inv:labels:two-pass`) every resolution consults registries already completed from the whole carrier, so no derivation can observe traversal order. ∎

**Meta-theorem (No self-support)** · `metathm:labels:no-self-support`

No token can keep itself in an anchor set from the index alone, and removing a label's last body citation stales the committed index. Demonstration. Anchors(D, U) is derived by (`inf:labels:anchor-harvest`) from body citations only, and the index section lies outside body(D); an index row therefore derives nothing, while exactness of presentation fails the moment the set shrinks. ∎

**Meta-theorem (Presentation invariance)** · `metathm:labels:presentation-invariance`

A migration that re-forms occurrences while preserving every label value changes presentation, not denotation: every minting and resolution judgment stands, every version identifier and content hash of the governed artifacts — the software and generated outputs the corpus documents, together with every hash computed from label sets and anchor sets alone — stands, and the document revision alone advances. Demonstration. Labels are the denotation and occurrence form the presentation; if no label value changes, the registries and every derivation over them are unchanged — minting (`judg:labels:minting`), resolution (`judg:labels:resolution`), and the harvested sets of (`inf:labels:anchor-harvest`) alike — hence so is every hash computed from them. Hashes over the edited sources' exact bytes are presentation-level and advance with the revision; they are not computed from the registries. ∎

## Caveats · `sec:labels:caveats`

**Caveat (Non-normativity)** · `cav:labels:non-normativity`

The calculus is documentation, never semantic input. Labels are navigation and provenance metadata, and membership in a registry confers no identity outside the graph. No compiler, build system, packager, or release tool consumes the graph; the checker and the register generator are its only consumers. Renaming a label updates all citations in the same commit and bumps no version of the software the corpus documents, by the presentation–denotation split of (`metathm:labels:presentation-invariance`); and a migration that changes the set of labels carried by a governed artifact is reviewed as a source correction, never accepted silently as formatting.

**Caveat (Assets and non-claims)** · `cav:labels:assets`

The calculus defines no asset. A test is what the harness recognizes, a module is what the language provides, and a profile only reads the facts they expose. An inventory citation therefore claims exactly that an asset of that class, classification, and name exists in that owner — never that its content is unchanged, its behavior right, or, for a container, that its membership is anything at all: members come and go without disturbing a citation of the container, and a document wanting the roster wants a generated register, nonparticipating as ever (`judg:labels:participation`). Pinning content is a matter for a content-identity discipline, not for names.

**Caveat (Coexistence and diagnostics)** · `cav:labels:coexistence`

Two owners may mint the same label text without collision: ownership disambiguates, and a citation transfers no ownership. Traversal failures are diagnostics — an unreadable tree must never become an empty carrier. Scoped register generation ignores unrelated owners' defects, while the corpus-wide check still validates everything.

## Rejected Ansätze · `sec:labels:rejected-ansaetze`

**Ansatz (Flat namespace)** · `ansatz:labels:flat-namespace`

Take one global namespace: no Signature, every match resolves. Then the side condition of (`inf:labels:imported-citation`) vanishes, and a working note's citation resolves into any owner's mint with no declared crossing. Rejected.

**Ansatz (First mint wins)** · `ansatz:labels:first-mint-wins`

Order the mints and keep the first. Then (`inv:labels:unique-mint`) is abandoned and the warrant contest is decided by luck: an incidental bare span silently becomes — or moves — a label's conceptual home, with no diagnostic. Rejected.

**Ansatz (Code-font delimiters in code)** · `ansatz:labels:code-font-delimiters`

Scan the ordinary code-font spans of documentation comments for labels. Then (`judg:labels:participation`) is undecidable without a fragile label-like heuristic, and ordinary documentation makes false mints. Rejected.

**Ansatz (Unchecked local labels)** · `ansatz:labels:unchecked-locals`

Exempt one class of sources — say the working notes — from resolution. Then (`inv:labels:total-resolution`) fails on that class and dangling references hide real defects, while owner-aware cross-file resolution has already made totality cheap. Rejected.

**Ansatz (Participating registers)** · `ansatz:labels:participating-registers`

Let generated registers mint and cite. Then (`judg:labels:minting`) holds of derivative text, and a register row sustains its own membership after the last body citation is gone, contradicting (`metathm:labels:no-self-support`). Rejected.

**Ansatz (Authored asset labels)** · `ansatz:labels:authored-asset-labels`

Let authors name their tests' labels freely — warrants by fiat where facts were available. Then the label drifts from the asset the moment either moves, nothing lints the drift, and a citation pins prose to a chosen name rather than to anything in the code. Derivation (`inf:labels:derivation-warrant`) is the whole point: the label is evidence, not prose. Rejected.

**Ansatz (Paths in the derivation)** · `ansatz:labels:path-derivation`

Derive the name from the module path or file. Then every refactor breaks every citation, breakage stops meaning anything, and authors learn not to cite (`judg:labels:derivation`). Location participates as ownership — the package — or not at all. Rejected.

**Ansatz (Membership in the derivation)** · `ansatz:labels:membership-derivation`

Let a container's label derive from its contents. Then adding one test breaks every citation of the suite, and the container's label becomes a content hash wearing a name. A label pins what a thing is called, never what it holds (`cav:labels:assets`). Rejected.

## Implementation gate · `sec:labels:gate`

**Gate (Implementation)** · `gate:labels:implementation`

Implementation is blocked until all of the following hold:

- the three occurrence forms of (`lang:labels:label-language`) parse in both concrete syntaxes, and a span that parses as no form is text, per (`gram:labels:well-formed`);
- duplicate mints fail with both locations in every owner (`inv:labels:unique-mint`);
- every same-owner and imported citation resolves to exactly one mint, and resolution is total (`inv:labels:total-resolution`);
- bracket-free cross-owner tokens and self-qualified imports fail (`inf:labels:imported-citation`), and an unresolved same-owner citation whose label mints in another owner is reported with the import form suggested;
- resolution is independent of traversal order (`metathm:labels:order-independence`);
- every designated typed-data string resolves as a synthetic citation of its target owner (`inf:labels:synthetic-citation`);
- generated registers are nonparticipating, current, and deterministic (`judg:labels:participation`);
- every pinned anchor-set hash derives from the ordered body-citation set alone (`inf:labels:anchor-harvest`), and no register sustains its own membership (`metathm:labels:no-self-support`);
- traversal failures surface as diagnostics, and scoped register generation ignores unrelated owners' defects (`cav:labels:coexistence`);
- every mint stands on exactly one warrant, every kind admits at most one warrant species, and reserved kinds without a governing profile fail (`inv:labels:warrant-totality`);
- every covered asset carries, per covering profile, exactly its derived label at the standard place, and no inventory label outlives its asset (`inv:labels:inventory`);
- this document carries exactly one mint per environment and no environment numbering, and every citation in the corpus resolves;
- the checker warns on near-miss spans, including label-shaped backtick spans in scanned code comments where the acute syntax was intended, without treating them as occurrences (`inv:labels:total-resolution`);
- the gate is dischargeable from this document and the corpus's adoption data alone; no check consults another document;
- the corpus-wide check passes in continuous integration.
