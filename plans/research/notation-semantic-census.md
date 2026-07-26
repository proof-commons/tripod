# Research Question: Semantic Notation Census · `q:notation:semantic-census`

> **Status:** Open; prototype required
> **Blocks:** Layer-0 vocabulary integrity, the surface/core purity lint, and the symbol index's self-containment claim
> **Affected packages:** `papers/attestation`, labels
> **Depends on:** nothing; the prototype is independent of (`q:attestation:floor-bounds`)
> **Decisions:** ADR-013 owner-aware label graph, ADR-014 hand-managed census
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

The candidates compose. A plausible resolution is a declaration convention
parsed by a checker, with the lint expressions generated and the index checked
rather than generated.

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

- an exported macro added with no index row;
- an index row naming a macro that does not exist;
- an alias added with no purity-expression entry;
- an alias whose declared canonical is not what it expands to;
- a macro moved between strata without its declaration changing;
- an index row rendering a mandatory-argument macro with an empty argument;
- a collision-register row naming a font family the glyph does not use;
- a correct tree, asserting silence.

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

Pending.

## Handoff · `sec:notation-semantic-census:handoff`

A successful prototype creates a checker in the labels package with its own
build target and fixtures, an accepted declaration or manifest form documented
with the macro contract, and corrected index, register, and lint artifacts.

The corrections themselves are not blocked on this note. They are blocked on
(`q:attestation:floor-bounds`) only where they restate a bound whose scope is
still under analysis; the alias, exclusion-rule, rendering, and typography
repairs are independent and may land first. What this note must prevent is
those repairs landing as an eighth hand-maintained restatement.
