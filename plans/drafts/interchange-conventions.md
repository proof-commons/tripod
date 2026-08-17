# The Interchange Conventions

_A model-theoretic statement_

This document consolidates, as one system, the conventions adopted piecewise — the encoding layer, namespace labels, versions, the envelope, dispatch, and evolution — and supersedes the fragments it integrates. It states the system as a calculus of satisfaction: two languages, one of data and one of description; a satisfaction judgment between them; a registry assigning theories to envelope coordinates; and acceptance defined, not derived. Normative references: RFC 8949 (CBOR), RFC 8610 (CDDL), RFC 5234 (ABNF). Throughout, **§4.2** abbreviates RFC 8949 §4.2, _Core Deterministic Encoding Requirements_. Semantic Versioning 2.0.0 is a genealogical reference only; nothing normative depends on it. This document cites no label of any other document of an adopting corpus, and its only normative dependencies are the external standards named above; where an adopting corpus uses it beside other disciplines, their alignment is fixed by the corpus's recorded adoption decisions, not by this text; and acceptance of this document presupposes acceptance of no other. Where another discipline's artifact is consumed — a labeling convention, an identity recipe, a format's declared environment classes — this document consumes the artifact as adoption data and asks nothing of its provenance.

The document practices the labeling discipline it assumes: it is a source in the corpus it governs. The label at each heading or environment head is that environment's mint; a parenthesized label in running text is a same-owner citation; material in fenced blocks and double-backtick spans is displayed without participating. The document title is publication metadata, not an environment head; it mints nothing and participates in nothing. Every label here has area `interchange`, each environment's kind names its genre, and environments carry no numbers: the mint at each head is the sole name of its environment. External section numbers, such as §4.2, quote other corpora; only internal numbering is absent. A Demonstration is part of the environment it closes and mints nothing of its own. And the namespace labels of this document are unrelated to the documentation labels of any corpus labeling discipline; only the word is shared.

## The two languages · `sec:interchange:languages`

**Language (Data)** · `lang:interchange:data-language`

The data language is the set of canonical names and the structures they denote. A name is a byte sequence that is a single CBOR data item encoded under §4.2: preferred serialization throughout — shortest-form heads for integers and lengths, and for floating-point values where admitted — no indefinite-length encodings, and map keys pairwise distinct and sorted bytewise-lexicographically on their encoded forms. Membership is exact: a byte sequence not of this form belongs to the language nowhere and denotes nothing. The language contains one name per structure, made precise by (`metathm:interchange:unique-names`). A _document_ is a structure of a particular form: a map whose keys are unsigned integers, in which key 0 is present and holds a namespace label, and key 1 is present and holds a version; beyond keys 0 and 1 a document may carry any values, under unsigned-integer keys only — content is open; the key space is disciplined. This prose definition is normative for membership where it and any operationalization could be read to differ.

**Language (Description)** · `lang:interchange:description-language`

The description language is CDDL (RFC 8610); its sentences are theories, and the control operators employed — `.size`, `.regexp`, `.gt` — are those of its §3.8. A theory constrains structure only. No theory of the description language can enforce determinism, because determinism is a property of the data language's names (`lang:interchange:data-language`), not of its structures. The two languages are therefore independent, and both are load-bearing.

**Grammar (Namespace labels)** · `gram:interchange:label-grammar`

Σ is the set of thirty-six characters comprising the lowercase Latin letters `a`–`z` and the decimal digits `0`–`9`; Σ⁻ is Σ together with the hyphen. An _atom_ is a nonempty finite word over Σ⁻ whose first and last characters lie in Σ: a single character of Σ is an atom, and hyphens occur only in the interior. A _namespace label_ is a word a₁`.`a₂`.`⋯`.`aₙ with n ≥ 2, each aᵢ an atom; the dot is a separator, not a character of any atom. A namespace label occupies at most 255 bytes in UTF-8 — equivalently, at most 255 characters. In ABNF (RFC 5234), normatively:

```abnf
namespace-label = atom 1*( "." atom )
atom            = alnum [ *( alnum / "-" ) alnum ]
alnum           = %x30-39 / %x61-7A        ; 0-9 / a-z
```

Reading atoms left to right descends a rooted tree: authority over a prefix confers authority over its subtree, so uniqueness of whole labels reduces to uniqueness at each branching, which the reverse-DNS convention inherits from an existing global scheme. The requirement n ≥ 2 places every label strictly below the root: no one claims a bare top-level word. Allocation is decentralized here; assignment into the registry is a separate act of (`sig:interchange:theory-assignment`).

**Definition (Versions)** · `def:interchange:versions`

A _version_ is a triple (M, m, p) of unsigned integers — _major_, _minor_, _patch_ — represented as the three-element array [M, m, p]. Versions are ordered lexicographically: by major, then by minor, then by patch. The order is for selection, not acceptance: an emitter targeting a namespace chooses the greatest assigned (M, m) it supports, while acceptance itself (`def:interchange:acceptance`) compares by equality only.

**Notation (Ground terms)** · `ntn:interchange:ground-terms`

An _unsigned integer_ (`uint`) is a CBOR major-type-0 value: an integer in [0, 2⁶⁴). Byte sequences are compared bytewise-lexicographically. For a map d and a set K of keys, d↾K is the restriction of d to the keys in K; the _content_ of a document d is d↾{k : k > 1}. For a theory S, **L(S)** is its model class — the set of data-language structures satisfying S under (`judg:interchange:satisfaction`) — and **L₂(S)** = { content(d) : d ∈ L(S) } is its content class.

**Requirement (Fixed once)** · `req:interchange:fixed-once`

Each convention here is fixed once for the whole system. Local deviation is not an option the system offers.

## Satisfaction · `sec:interchange:satisfaction`

**Judgment (Satisfaction)** · `judg:interchange:satisfaction`

Form: d ⊨ S — the structure d satisfies the theory S. Satisfaction is structural only: it inspects the shape and values of d against S and nothing else — not d's history, not the registry's later state, not the reader's present holdings.

**Schema (Global)** · `schema:interchange:global`

The base theory, satisfied by every document of the data language before any assignment is consulted:

```cddl
global = {
  0 => namespace-label,
  1 => version,
  * (uint .gt 1) => any
}

version = [major: uint, minor: uint, patch: uint]

namespace-label = namespace-form .size (3..255)

namespace-form = tstr .regexp "[a-z0-9]([a-z0-9-]*[a-z0-9])?(\\.[a-z0-9]([a-z0-9-]*[a-z0-9])?)+"
```

The `.regexp` operationalizes the shape fixed by the ABNF of (`gram:interchange:label-grammar`), which is normative for shape where the two could be read to differ; the length bound is carried by `.size` — with 3 the length of the shortest label — and by the sentence of that Grammar, and the ABNF says nothing about it. The prose definition of a document in (`lang:interchange:data-language`) is normative for membership where it and this schema could be read to differ; `global` operationalizes it for satisfaction. The `.gt 1` on the wildcard is exposition more than enforcement — deterministic maps already exclude duplicate keys — but it lets the base theory say what the data language says: the envelope is not the content's to redefine.

**Definition (Acceptance)** · `def:interchange:acceptance`

A document d with envelope (ℓ, [M, m, p]) is _accepted_ under the registry R exactly when (ℓ, M, m) lies in the domain of R and d ⊨ R(ℓ, M, m); otherwise it is _rejected whole_. Acceptance is always relative to a registry state: a reader holding a state in which (ℓ, M, m) is unassigned rejects whole, and that rejection is a fact about the state, not about the document — by (`inv:interchange:permanence`) a later state can only turn rejection into acceptance, never the reverse. Equality does all the work: of ℓ as bytes, of M and m as integers. No other comparison participates, and p does not occur in the condition at all (`inv:interchange:patch-identity`). And bytes outside the data language (`lang:interchange:data-language`) denote no structure: a non-canonical input is not a defective document to be repaired — it is never accepted and re-canonicalized — but no document at all, with nothing for ⊨ to hold of.

## Theories · `sec:interchange:theories`

**Grammar (Assignable fragment)** · `gram:interchange:assignable-fragment`

An assigned theory is written in the assignable fragment of the description language (`lang:interchange:description-language`): one map rule pinning keys 0 and 1, enumerating its content keys — each a literal unsigned integer greater than 1 — with types drawn from a fixed constructor set — the primitive types, `.size`- and `.regexp`-constrained strings, bounded arrays and maps of fragment types, finite unions, and optionality — with no recursion and no controls beyond those named here. Every fragment theory extends the base theory (`schema:interchange:global`) semantically: its model class is included in the base theory's. Within the fragment, content-class inclusion is machine-checkable key by key by a sound criterion: every key of the earlier theory appears in the later with an admitting type, every key required later is required earlier, and type admission is decided structurally over the constructor set.

**Signature (Theory assignment)** · `sig:interchange:theory-assignment`

The registry **R** is a partial map from triples (namespace label ℓ, major M, minor m) to theories of the assignable fragment (`gram:interchange:assignable-fragment`), maintained by an owner. Every assigned theory extends the base theory (`schema:interchange:global`) — L(R(ℓ, M, m)) ⊆ L(global) — achieved by pinning key 0 to ℓ, pinning key 1 to [M, m, uint] with patch free, enumerating its content keys, each a literal uint greater than 1, and being closed: it admits nothing it does not name. Allocation of the label tree is decentralized (`gram:interchange:label-grammar`); assignment into R is the owner's separate act, and allocation and consolidation of the published registry material are obligations of that owner (`cav:interchange:governance-obligations`), not theorems.

**Invariant (Restraint)** · `inv:interchange:restraint`

An assigned theory admits floating-point values, tags, or simple values other than `false`, `true`, and `null` only by explicit provision, and every such provision fixes the canonical form of what it admits.

**Invariant (Permanence)** · `inv:interchange:permanence`

Theories are never revised and never withdrawn: once (ℓ, M, m) is assigned, its theory object is immutable in R.

**Invariant (Patch identity)** · `inv:interchange:patch-identity`

For fixed (ℓ, M, m), all patch revisions share the single assigned theory object: the model class is identical across patches, and only exposition moves.

**Invariant (Minor inclusion)** · `inv:interchange:minor-inclusion`

For fixed ℓ and M, and minors m < m′ both assigned: L₂(R(ℓ, M, m)) ⊆ L₂(R(ℓ, M, m′)), and every content key defined at m keeps its meaning at m′. The inclusion is machine-checkable within the assignable fragment (`gram:interchange:assignable-fragment`); meaning preservation is a governance obligation (`cav:interchange:governance-obligations`), of the same standing as allocation.

**Law (Major boundary)** · `law:interchange:major-boundary`

A revision of the described system whose new assignment would violate (`inv:interchange:patch-identity`) or (`inv:interchange:minor-inclusion`) is major, whatever else it claims to be. Symmetrically: a revision changing the model class at all is at least minor, whatever else it claims to be.

**Example (Registered theory)** · `ex:interchange:registered-theory`

An assigned theory, illustrated for a namespace at major 1, minor 2, exhibiting the assigned shape — envelope pinned, patch free, remainder closed — with a key added at minor 2, optional as (`metathm:interchange:conservativity`) requires:

```cddl
example = {
  0 => "com.company.example",
  1 => [1, 2, uint],
  2 => tstr,           ; defined since 1.0
  ? 7 => bstr,         ; added at 1.2 — necessarily optional
}
```

Extension is by restriction: the envelope keys are pinned, and the base theory's wildcard is replaced by the enumerated content keys.

## Metatheory · `sec:interchange:metatheory`

**Meta-theorem (Unique names)** · `metathm:interchange:unique-names`

Every structure of the admitted data model has exactly one name in the data language. Consequently two conforming encoders given equal items emit equal bytes, and byte equality of names decides equality of structures system-wide. Application-level equivalence between distinct structures — an integer and a float of equal magnitude, a tagged and an untagged reading — is not identified unless an assigned theory identifies it. Demonstration. Membership in (`lang:interchange:data-language`) already requires canonical form, and §4.2 removes each degree of freedom the format offers: head widths are fixed by preferred serialization, framing by the prohibition of indefinite lengths, map order by the sorting rule, floating-point width by shortest form where (`inv:interchange:restraint`) admits floats at all. Induction over the structure of items. ∎ Everything the system builds on byte equality — content addressing, signatures, deduplication — rests here, and it stands because bytes outside the data language never reach ⊨ at all: they are refused at the door, not repaired there.

**Meta-theorem (One spelling, one encoding)** · `metathm:interchange:one-spelling`

The map from namespace labels to their canonical names is injective, and string equality, byte equality, and encoded-item equality coincide on labels. Demonstration. Every character of Σ⁻ together with the dot is printable ASCII, on which UTF-8 acts as the identity, one byte per character; injectivity follows. No character of the alphabet (`gram:interchange:label-grammar`) participates in any Unicode canonical or compatibility decomposition, so every label is a fixed point of NFC and NFD alike, and no normalization can produce a second byte form; there is no case to fold and no ignorable to strip. §4.2 then fixes a unique text-string name of those bytes (`lang:interchange:data-language`). ∎

**Meta-theorem (Bounded determination)** · `metathm:interchange:bounded-determination`

The theory assigned to a document is determined by at most a 296-byte prefix of its name, together with R: nothing beyond the envelope need be examined before the theory governing the document is known. Openness in transit and strictness on receipt coexist without tension. Demonstration. For unsigned integers under preferred serialization (`lang:interchange:data-language`), bytewise-lexicographic order of names coincides with numeric order: the head classes — immediate values 0–23, then one-, two-, four-, and eight-byte arguments — begin with strictly increasing initial bytes (`0x00`–`0x17`, `0x18`, `0x19`, `0x1a`, `0x1b`), oversized heads are forbidden, and within a class arguments are big-endian of equal length, where bytewise and numeric order agree. Hence the entries at keys 0 and 1 — the two least keys — stand first in every document's name. For the bound: a map head is at most 9 bytes; key 0 is 1 byte; a label is at most 2 + 255 bytes; key 1 is 1 byte; the version is a 1-byte array head plus three uint heads of at most 9 bytes each; and 9 + 1 + 257 + 1 + 1 + 27 = 296. Acceptance (`def:interchange:acceptance`) consults only that envelope and R. ∎

**Meta-theorem (Conservativity)** · `metathm:interchange:conservativity`

A later minor is conservative over the shared key vocabulary: it imposes no requirement that models of an earlier minor lack. In particular, a content key absent from some earlier assigned minor of the same major is optional in every later minor's theory. Demonstration. A content valid at the earlier minor — of which there is at least one, the fragment's constructors being inhabited — lacks the key; by (`inv:interchange:minor-inclusion`) it lies in the later minor's content class; a theory requiring the key would exclude it. ∎

**Meta-theorem (Absoluteness)** · `metathm:interchange:absoluteness`

d ⊨ S is a fact of d and S alone. With permanence, an archived document's satisfaction can never change: its assigned theory outlives every later revision, archives never rot, and nothing is ever republished. A new document may address a pair its reader does not yet hold, and is then rejected whole — so progress is paid for by readers updating, never by archives. Demonstration. Satisfaction is structural (`judg:interchange:satisfaction`); the theory object is immutable (`inv:interchange:permanence`) and shared across patches (`inv:interchange:patch-identity`); nothing in either varies with the registry's later growth or the reader's state. ∎

**Meta-theorem (Acceptance monotonicity)** · `metathm:interchange:acceptance-monotonicity`

Registry growth moves acceptance in one direction only: a document rejected for an unknown coordinate under one registry state is accepted under a later state only if that state assigns its coordinate — and then exactly when it satisfies the assigned theory — and a document once accepted is accepted under every later state. Demonstration. Acceptance (`def:interchange:acceptance`) consults the domain of R and the assigned theory alone; the domain only grows, and each assigned theory object is immutable (`inv:interchange:permanence`), so no later state can remove a coordinate or alter what its theory admits. ∎

## Caveats · `sec:interchange:caveats`

**Caveat (Extra-logical governance)** · `cav:interchange:governance-obligations`

Allocation of namespaces, consolidation of the published registry material, and preservation of meaning across minors are obligations of the owner, outside the logic: the calculus checks inclusion (`inv:interchange:minor-inclusion`), never intent. The registry is modeled as a partial map (`sig:interchange:theory-assignment`), so functionality is true of the map by construction; what governance owes is that the maintained and published material determines one such map and contains no conflicting assignment records. Allocation should also mind confusable spellings — `l` against `1`, `0` against `o` — which the grammar admits and equality will not conflate: distinctness of labels is byte-distinctness, and nearness to another owner's label is a governance concern, not a logical one. These obligations are of one standing, and none of them is a theorem. This document states no adoption gate: its conditions are the invariants themselves, checked by the owner's registry machinery.

**Caveat (Genealogy)** · `cav:interchange:genealogy`

Semantic Versioning 2.0.0 is genealogical only: its discipline is retained in the invariants and the Law (`law:interchange:major-boundary`); its grammar is discarded. Prerelease identifiers and build metadata are not forbidden but unrepresentable. Experimental status is carried solely by namespace designation, and no exemption attaches to major zero: there is no anything-goes phase.

**Warning (Lower bound)** · `warn:interchange:lower-bound`

The Law (`law:interchange:major-boundary`) is a lower bound, not a license: major permits breakage; it does not invite it.

**Remark (Self-application)** · `rem:interchange:self-application`

Nothing prevents this specification, or the registry itself, from traveling as documents in a reserved namespace. But the base theory (`schema:interchange:global`) is constitutionally prior: it is the one theory reached without dispatch, because it is what makes dispatch (`def:interchange:acceptance`) possible.

## References · `sec:interchange:references`

RFC 8949, _Concise Binary Object Representation (CBOR)_, §4.2 in particular. RFC 8610, _Concise Data Definition Language (CDDL)_; the control operators `.size`, `.regexp`, and `.gt` are those of its §3.8. RFC 5234, _Augmented BNF for Syntax Specifications_. Semantic Versioning 2.0.0 (semver.org), genealogical only. This section is authored, annotated prose — a References division, not a generated bibliography register.
