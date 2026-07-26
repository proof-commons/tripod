# Research Question: Semantic Notation Census · `q:notation:semantic-census`

> **Status:** Resolved — no additional notation census required; the diagnosis was wrong
> **Blocks:** nothing further; the surviving work is attestation label minting and editorial correction
> **Affected packages:** `papers/attestation`
> **Depends on:** the corrected scopes from (`q:attestation:floor-bounds`)
> **Decisions:** ADR-013 owner-aware label graph is sufficient; a second semantic registry is rejected
> **Expected handoff:** precise Layer-0 semantic labels for the floor-bound correction, plus editorial repairs to the macro comments and symbol index

## Question · `sec:notation-semantic-census:question`

Can the attestation semantic vocabulary be single-sourced and mechanically welded
to every artifact that restates it, without parsing arbitrary TeX?

The answer is yes, and it is the wrong question. It treats a content-authoring
problem as a notation-tooling problem. The corrected diagnosis and the evidence
for it are in the result below.

## Why the question was opened · `sec:notation-semantic-census:stakes`

The duplication that prompted this note is real. One concept is described in
seven unwelded places: the macro comment, the prose definition, the theorem
statement, the verification row, the symbol-index row, the collision-register
row, and the purity grep. Nothing checks that they agree, and the floor-ceiling
drift is what that permitted — a correctly scoped proposition coexisting with
four restatements that dropped its scope, one of them a verification invariant
asserting a false general claim, with every gate green. That proposition has
since been removed and replaced by separately scoped claims.

That observation stands. What does not stand is the inference that the repair
is a second vocabulary registry.

## Observed drift instances · `tbl:notation-semantic-census:drift`

Retained as evidence. All were found by reading, not by any gate.

| Kind | Instance | Disposition |
|---|---|---|
| scope dropped | genesis floor ceiling restated as a bootstrapping ceiling in the macro comment, index, and verification rows | attestation label minting plus editorial correction |
| scope dropped | the capacity bound's "before external deposits" condition present in the macro comment, absent from the index row | editorial correction |
| lint gap | retained surface alias absent from the purity expression that names its canonical macro | editorial correction to the expression |
| index gap | exported alias and three helpers absent from the index, with no exclusion rule declared | editorial correction |
| contract contradicted | two macros documented as always applied, used bare 65 and 6 times respectively | editorial correction to the comments |
| contract contradicted | the unit conversion documented as applied at exactly two points, consumed at three | editorial correction |
| typography error | one glyph's font family described as another glyph's in the collision register | editorial correction |
| invalid rendering | operation macros invoked with empty mandatory arguments in the index | editorial correction |

Every row is a documentation defect. Exactly one — the first — also has a
semantic consequence, and that consequence is addressed by giving the distinct
claims distinct attestation labels, not by parsing macros.

## Withdrawn design · `sec:notation-semantic-census:withdrawn`

An earlier revision of this note accepted a composition of a `@tripod-notation`
declaration grammar above each macro, a `labels::notation` lexical extractor, a
derived purity set, an index inclusion/exclusion contract, and `@tripod-claim`
scope markers welding a claim's scope token across its required homes.

That design is withdrawn, not deferred. It is recorded here only so a future
reader does not re-propose it without seeing why it was rejected:

- it introduces a second semantic-source system alongside the label graph,
  which is the defect this note was opened to remove, not a cure for it;
- the macro file and symbol index are presentation, and promoting them to
  parallel semantic registries entrenches the confusion;
- the claim-scope weld proves only that declared scope *tokens* agree, which is
  a weak proxy for the property actually wanted;
- the cost is a new parser, a new grammar, a new build role, and a fixture
  suite, for a defect class the existing graph already covers once the labels
  are precise.

The candidate matrix that produced it is superseded by the result below.

## The corrected diagnosis · `rule:notation-semantic-census:diagnosis`

Attestation did not consistently give important semantic claims their own precise
labels. Several distinct facts were bundled under one informal idea of a "floor
ceiling":

```text
1  the floor definition            φ = Ω/Y
2  the pre-maturity inequality     φ ≤ Ω/Y_T
3  the genesis special value       1/(1-ζ)
4  the false claim that 3 is a fixed ceiling throughout bootstrapping
5  the pre-deposit capacity        A_max = E₀·ln(1/(1-ζ))
```

These are five facts with different scopes. A label should answer *what exact
proposition, equation, definition, or qualification does this denote*, not
*where is roughly related material discussed*. When distinct claims share a
broad home, importing the wrong scope becomes easy.

The rule is not "label everything" — an intermediate algebra line inside one
proof needs no public label. It is **label every independently consumed
semantic fact**: one imported by a consumer, depended on by name by another
Attestation result, defining an interface or monetary-surface quantity, carrying a
scope that matters, distinguishing two claims that share notation, asserted as
a verification invariant, or whose alteration would change a released
consumer's dependency.

## Audit of the second half of the diagnosis · `tbl:notation-semantic-census:audit`

The diagnosis also proposed that the realization often cites a broad nearby
definition or section instead of the exact upstream claim. Measured against the
document, that half does not hold.

| Measurement | Result |
|---|---|
| distinct attestation anchors imported by the R13 body | 38 |
| imports by label kind | `def`, `prop`, `thm`, `post`, `lem`, `rem`, `alg`, `open` only |
| section, appendix, table, or equation imports | 0 |
| symbol-index or collision-register imports | 0 |
| containment or seigniorage anchors imported | 0 |

R13 never cites an Attestation section, appendix, or symbol-index anchor for a
semantic premise; every import is claim-bearing. The one place it imports the
affected material is precise and correctly scoped: it states that only the live
portion is burnable *before external deposits* and cites
(`[A-prop:interface:bootstrap-capacity]`), which is the proposition carrying
exactly that scope.

The consumer-side citation repair the diagnosis anticipated therefore has no
current instances. The imprecision is Attestation-internal.

## Result · `sec:notation-semantic-census:result`

Resolved. The problem was misdiagnosed.

The repository does not need a second semantic-vocabulary registry, nor a
parser for macro strata, aliases, symbol-index rows, and prose scopes. ADR-013's
owner-aware label graph is sufficient: Layer 0 mints, consumers import with the
`A-` prefix, and the graph already enforces ownership, uniqueness, form,
resolution, index/body agreement, and the anchor-set binding.

The actual defect is incomplete attestation semantic label minting. The
consumer-side half of the original diagnosis is not supported by the document:
R13's imports are claim-bearing and, where they touch the affected material,
correctly scoped.

The macro file and symbol index remain presentation. They should be corrected
where false, and must not become a parallel semantic source.

### Label lifecycle rule · `rule:notation-semantic-census:lifecycle`

One rule survives from this note as a genuine constraint on the correction:

> A stable semantic label must never silently change meaning.

The lifetime-envelope anchor denoted *a finite lifetime envelope exists*.
Withdrawing that theorem removes the label; the negative result gets a new one.
Reusing the key would preserve navigation while reversing semantics, which is
strictly worse than a broken link because nothing would report it.

Adding new labels is safe for consumers. A consumer's anchor-set hash moves
only when the consumer's own distinct import set changes, not when Attestation
mints something new.

## Handoff · `sec:notation-semantic-census:handoff`

Two independent workstreams, neither requiring new machinery.

**Attestation semantic labels**, owned by (`q:attestation:floor-bounds`): give each
surviving claim from that resolution one precise home, and retire rather than
repurpose the label of the withdrawn theorem. The semantic map lives in that
note, since the mathematics determines the homes.

**Editorial correction**, a focused backlog task: every drift-table row above,
corrected when the surrounding text is touched. Independent of the floor-bound
result except the last, which needs the corrected scope first:

```text
include feespigot in the purity expression naming feeflow
index or explicitly exclude feespigot, bootrange, normrange
replace \depositop{}{}, \burnop{}{}, \redeemop{}
   with \depositop{\delta}{a}, \burnop{x}{a}, \redeemop{x}
correct "blackboard vs calligraphic" for plain A to
   "italic Latin vs calligraphic"
correct the always-applied comments to bare-or-applied
correct the conversion-point count from two to three
narrow every description of 1/(1-ζ) to its real scope
```
