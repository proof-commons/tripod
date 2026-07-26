# Research Question: Semantic Notation Census · `q:notation:semantic-census`

> **Status:** Design accepted; implementation pending
> **Blocks:** Layer-0 vocabulary integrity, the surface/core purity lint, and the symbol index's self-containment claim
> **Affected packages:** `papers/attestation`, labels
> **Depends on:** the corrected scopes from (`q:attestation:floor-bounds`), which the claim weld carries
> **Decisions:** ADR-013 owner-aware label graph, ADR-014 hand-managed census; declaration convention plus typed checker plus derived lint accepted, handwritten manifest rejected
> **Imports:** (`[A-app:symbols]`),
> (`[A-prop:containment:floor-ceiling]`),
> (`[A-prop:interface:bootstrap-capacity]`),
> (`[A-def:interface:net-live-share]`)
> **Expected handoff:** a single semantic vocabulary source welded mechanically to the symbol index, the collision register, and the purity lint

## Question · `sec:notation-semantic-census:question`

Can the attestation semantic vocabulary be single-sourced and mechanically welded
to every artifact that restates it, without parsing arbitrary TeX?

The macro file is the declared vocabulary root of the whole stack. One concept
is currently described independently in seven places, none of which is derived
from any other:

```text
1  the macro definition comment
2  the normative prose definition
3  the theorem or proposition statement
4  the verification-suite table row
5  the symbol-index row
6  the collision-register row
7  the review-blocking purity grep expression
```

Nothing checks that the seven agree. This note asks whether one of them can
become the source and the rest derivations or checked consumers.

## Why this is not a style question · `sec:notation-semantic-census:stakes`

The duplication has already produced a substantive failure. The floor-ceiling
proposition (`[A-prop:containment:floor-ceiling]`) states its scope
correctly — the value holds before external deposits. Four independent
restatements dropped that scope, and one of them is a verification-suite
invariant asserting a false general claim. The mathematics is owned by
(`q:attestation:floor-bounds`); what belongs here is the observation that a
correct proposition and four incorrect restatements of it coexisted with every
existing gate green.

The same duplication produced a lint that does not enforce what it claims:

- the purity grep names the canonical fee-flow macro but not its retained
  alias, so a downstream published formula can reference the same forbidden
  surface quantity through the alias and pass the review-blocking check;
- the alias appears in no symbol-index row, so the index cannot be used to
  discover it either;
- three further exported or presentation macros are absent from the index,
  which declares no exclusion rule and so cannot be read as either complete or
  deliberately partial.

At the time of writing the alias and two phase helpers have zero uses anywhere
in the repository, so the escape is latent rather than exploited. It is
review-blocking policy that is unenforced, not a current violation.

## Observed drift instances · `tbl:notation-semantic-census:drift`

Each row is a fixture the prototype must detect. All were found by reading, not
by any gate.

| Kind | Instance | Detected by |
|---|---|---|
| scope dropped | genesis floor ceiling restated as a bootstrapping ceiling in the macro comment, index, and verification rows | nothing |
| scope dropped | the capacity bound's "before external deposits" condition present in the macro comment, absent from the index row | nothing |
| lint gap | retained surface alias absent from the purity expression that names its canonical macro | nothing |
| index gap | exported alias and three helpers absent from the index, with no exclusion rule declared | nothing |
| contract contradicted | two macros documented as always applied, used bare 65 and 6 times respectively in normative prose | nothing |
| contract contradicted | the unit conversion documented as applied at exactly two points, consumed at three | nothing |
| typography error | one glyph's font family described as another glyph's in the collision register | nothing |
| invalid rendering | operation macros invoked with empty mandatory arguments in the index, rendering malformed signatures | nothing |

The bare-versus-applied row is the clearest evidence that the duplication is
the problem rather than any individual author: the macro file's own design
principles state that empty braces yield the bare symbol, while two macro
comments in the same file state that the same symbols are always applied. The
file contradicts itself, and both readings have consumers.

## Constraints · `rule:notation-semantic-census:constraints`

A candidate must respect the existing repository discipline:

- the macro file compiles standalone and is the root of the import chain; it
  may not gain a dependency on a generated artifact;
- generated artifacts are one-way derivatives, so the symbol index may be
  generated from the vocabulary but the vocabulary may not be generated from
  the index;
- the census is hand-managed and explicit under ADR-014; a new declared file
  must fail the build until it is listed;
- checks report under the bounded-diagnostics policy and fail closed;
- no candidate may require a TeX engine run to perform the check, since the
  lint must run in lanes that do not build the papers.

## Candidate matrix · `tbl:notation-semantic-census:candidates`

| Mint | Candidate | Strength | Main risk |
|---|---|---|---|
| `candidate:notation:declaration-convention` | A fixed comment grammar above each macro carrying stratum, glyph, canonical-or-alias, domain, and scope; parsed lexically | No TeX semantics needed; source stays the single file | A comment grammar can drift from the definition it labels unless the definition is parsed too |
| `candidate:notation:manifest` | A checked manifest file listing every semantic macro and its attributes, welded to the macro file and the index | Typed, easy to consume from Rust, matches the register discipline | A third artifact to keep current; needs its own weld or it becomes an eighth restatement |
| `candidate:notation:generated-index` | Generate the symbol-index appendix from the vocabulary source | Removes one restatement entirely rather than checking it | Appendix prose is currently richer than any attribute set; generation may flatten it |
| `candidate:notation:generated-lint` | Derive the purity expressions from the vocabulary source instead of maintaining them by hand | Closes the alias gap by construction | The expressions are consumed by review instructions and shell lanes, not only by code |
| `candidate:notation:checker-only` | Parse macro definitions lexically and check the index, register, and lint agree; change no authoring workflow | Smallest change; immediate value | Proves agreement of restatements without reducing their number |

**Selected:** (`candidate:notation:declaration-convention`),
(`candidate:notation:checker-only`), and (`candidate:notation:generated-lint`),
composed. The macro file remains the declared vocabulary source.

**Rejected:** (`candidate:notation:manifest`). An independent handwritten
manifest would become another restatement requiring another weld, which is the
defect this note exists to remove. (`candidate:notation:generated-index`) is
deferred rather than rejected: the appendix prose is currently richer than any
attribute set, so the index is checked, not generated, until the attribute set
demonstrably carries the same meaning.

## Accepted design · `sec:notation-semantic-census:design`

### Declaration convention · `rule:notation-semantic-census:declaration`

Each semantic macro in Part B carries one adjacent machine-readable
declaration. Every Part-B exported command must have exactly one; nothing is
silently omitted.

```tex
% @tripod-notation
% name=ratefloor
% stratum=surface
% arity=1
% canonical=ratefloor
% index=required
% formula-policy=citation-only
% application=bare-or-applied
% scope=dynamic-settlement-floor
\NewDocumentCommand{\ratefloor}{m}{...}
```

An alias is explicit, and so is a presentation helper's exclusion:

```tex
% @tripod-notation
% name=feespigot
% stratum=surface
% arity=0
% alias-of=feeflow
% index=exclude:redundant-alias
% formula-policy=citation-only
% application=bare
```

### What the checker derives · `rule:notation-semantic-census:checks`

A `labels::notation` module extracts name, arity, stratum, canonical-or-alias
status, alias target, index requirement or exclusion reason, purity policy,
application policy, and declared scope — all lexically, with no TeX run. It
then checks the declaration against the definition it labels:

- the declared command exists and its actual arity matches;
- aliases expand to the declared canonical, chains terminate, and no alias
  crosses an authority or purity class;
- every semantic command is declared exactly once, and no declaration names a
  nonexistent command;
- every required symbol has exactly one symbol-index row;
- every excluded symbol carries a reason;
- every alias inherits its canonical's purity restriction.

The macro file stays excluded from label minting but enters its own role, so
notation parsing is distinct from label harvesting:

```text
--attestation-macros papers/attestation/macros_attestation.tex
```

### Derived purity · `rule:notation-semantic-census:purity`

The forbidden set is computed from declarations rather than maintained by
hand:

```text
forbidden in consumer formulas =
    every surface macro
    ∪ every core macro
    ∪ every alias of either
```

This closes the alias gap by construction. The hand-maintained grep expression
is either removed in favour of the typed checker, or confined to a delimited
block whose contents are rendered from the declarations and compared exactly.
It must not survive as an unverified parallel policy list.

### Index contract · `rule:notation-semantic-census:index-rule`

The appendix declares its own rule, and the checker verifies exactly that rule:

```text
Every Part-B semantic macro appears exactly once unless its
@tripod-notation declaration carries index=exclude:<reason>.
```

An omission is then never indistinguishable from policy.

### Application-policy correction · `rule:notation-semantic-census:application`

The macro comments are corrected rather than forcing dozens of valid bare uses
to change. For both the floor and the confidence-price handles the established
convention is:

```text
empty argument     current, generic, or context-unspecified value
nonempty argument  explicitly indexed or applied value
```

so "ALWAYS APPLIED" becomes "bare-or-applied according to context". Likewise
the "exactly two conversion points" comment names three semantic roles: burn
valuation, redemption payout, and deposit/cycle issuance.

## Scope-sensitive claim weld · `rule:notation-semantic-census:claim-weld`

The notation checker proves vocabulary structure, not prose truth. For the
small number of scope-sensitive published claims, typed claim markers are added
rather than pretending a general prose checker exists.

```tex
% @tripod-claim
% id=floor-factor
% scope=pre-external-deposits
% required-homes=macro,proposition,symbol-index,verification
```

Each required home carries a matching non-rendering marker. The checker
verifies exactly one marker per required home, an identical scope token in
every home, no undeclared home, and no duplicate home.

This is the check that would have rejected the present drift, where the
proposition said "before external deposits" while the macro comment,
verification table, and symbol index promoted it to all of bootstrapping.

The limitation is stated honestly: the weld proves the declared scopes agree;
it does not understand arbitrary English. Where practical the visible scope
phrase itself comes from one shared presentation macro, reducing even that
residual.

## Required prototype · `sec:notation-semantic-census:prototype`

### Stage 1 — extraction

Extract every macro definition from the macro file lexically: name, argument
count, stratum from its section, and whether it expands to another macro. This
stage alone answers whether the alias and helper set can be recovered without
TeX semantics.

### Stage 2 — census comparison

Compare the extracted set against the symbol-index rows and the purity
expressions. Report every macro that is exported but unindexed, indexed but
undefined, or absent from an expression naming its canonical macro.

### Stage 3 — attribute weld

Add the chosen declaration or manifest form and check that each declared
attribute agrees with the definition it describes: a macro declared as an alias
must expand to its named canonical, and a macro declared with a stratum must
sit in that stratum's section.

### Stage 4 — exclusion rule

Require the index to declare its own exclusion rule and check the exclusions
are exactly the macros the rule names, so a silent omission is not
indistinguishable from a deliberate one.

### Stage 5 — regression fixtures

Encode every row of the drift table as a fixture and confirm each is reported.
A fixture that the chosen candidate structurally cannot detect must be recorded
as an accepted limitation, not dropped.

## Vectors · `sec:notation-semantic-census:vectors`

- an exported macro with no declaration;
- a declaration with no definition;
- a wrong declared arity;
- an alias with the wrong canonical target;
- an alias omitted from purity coverage;
- a required index row missing;
- a duplicate index row;
- an excluded index symbol with no reason;
- an indexed but undefined macro;
- a mandatory-argument operation rendered with empty arguments;
- a must-applied macro used bare, if any remain after the application-policy
  correction;
- a scope-sensitive claim restated under a conflicting scope marker;
- unrelated ordinary TeX, asserting it does not participate;
- a correct tree, asserting silence;
- diagnostics asserted to be deterministically ordered.

The last vector matters as much as the others: this checker runs on every
build, and a check that cannot be silent will be disabled.

## Acceptance · `gate:notation-semantic-census:accept`

Accept only when:

- one artifact is the declared vocabulary source and the others are derived
  from it or checked against it;
- every exported macro is either indexed or excluded by a declared rule;
- every alias is registered with the same authority and restrictions as its
  canonical, in both the index and every purity expression;
- the purity expressions are derived, or checked to cover the full exported
  set;
- each drift fixture is reported, or recorded as an accepted limitation with
  its reason;
- a correct tree produces no diagnostics;
- the macro file still compiles standalone with no dependency on any generated
  artifact.

## Rejection · `gate:notation-semantic-census:reject`

Reject a candidate if:

- it adds an artifact restating the vocabulary without welding it;
- it requires a TeX run to check;
- it enforces agreement only for macros an author remembers to declare;
- it silently accepts an exported macro that appears in no index row and no
  exclusion list;
- it replaces the appendix's prose with generated attributes at a net loss of
  meaning for a reader who never opens the macro file.

## Result · `sec:notation-semantic-census:result`

Design accepted; implementation pending.

The accepted composition is the declaration convention, the typed lexical
checker, and the derived purity set, with the symbol index checked rather than
generated and the scope-sensitive claim weld added. A handwritten manifest is
rejected as a further restatement.

A set of source defects is independent of the checker and can be corrected
without waiting for it:

```text
include feespigot in purity coverage by alias derivation
index or explicitly exclude feespigot, bootrange, normrange
replace \depositop{}{}, \burnop{}{}, \redeemop{}
   with \depositop{\delta}{a}, \burnop{x}{a}, \redeemop{x}
correct "blackboard vs calligraphic" for plain A to
   "italic Latin vs calligraphic"
correct the bare/applied comments to bare-or-applied
correct the conversion-point count from two to three
```

The last correction — narrowing the description of 1/(1-ζ) — waits on
(`q:attestation:floor-bounds`), which supplies the scope the corrected text
must state.

## Handoff · `sec:notation-semantic-census:handoff`

A successful prototype creates a checker in the labels package with its own
build target and fixtures, an accepted declaration or manifest form documented
with the macro contract, and corrected index, register, and lint artifacts.

The corrections themselves are not blocked on this note. They are blocked on
(`q:attestation:floor-bounds`) only where they restate a bound whose scope is
still under analysis; the alias, exclusion-rule, rendering, and typography
repairs are independent and may land first. What this note must prevent is
those repairs landing as an eighth hand-maintained restatement.
