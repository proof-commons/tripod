# A Taxonomy of Environments in Formal Writing, as a Kind Registry

This document constitutes the taxonomy of environments as a kind
registry: a catalogue of the environments of formal writing — headed
like the theorem family, displayed like the equation, structural like
the section, kept like the record — from papers, monographs, textbooks,
lecture notes, standards, specifications, and decision records, in
which every distinct genre carries one kind, a short token fit to serve
as the first segment of a label. The catalogue's 248 attested names
classify into 148 kinds. Synonymous surface names share a kind: a Judgment
classifies names, one Inference rule composes hybrid kinds, Invariants
govern the assignment, eleven Conventions lay out the registry itself,
Caveats bound it, three rejected Ansätze delimit it negatively, and a
Postcondition gates adoption. The catalogue is descriptive — every name
is attested in real formal writing — while the kind assignment is laid
down, and the registry classifies this document's own environment
heads.

The document practices the discipline it serves. The label at each
heading or environment head is that environment's mint; a parenthesized
label in running text is a same-owner citation; material in fenced
blocks, double-backtick spans, and plain code spans that are not
label-shaped — including every kind token in the tables — is displayed
without participating. Every label minted here has area `kinds`, and each
heading anchor carries `sec` — the kind this registry assigns to the
section — so the document's skeleton is classified by its own tables. Environments
carry no numbers: the mint at each head is the sole name of its
environment.

## Syntax · `sec:kinds:syntax`

**Language (Kinds and names)** · `lang:kinds:kind-language`
A kind is a word over lowercase letters and digits, fit to stand as the
first segment of a label; an environment name is the surface heading an
author writes, in natural language. Many names may share one kind;
no name has two.

```text
kind  ::=  word              word  ::=  [a-z0-9]+

a kind in use, as a label's first segment:   `thm:analysis:mean-value`
a citation of that label:                    (`thm:analysis:mean-value`)
```

## Assignment · `sec:kinds:assignment`

**Judgment (Classification)** · `judg:kinds:classification`
Form: n ▹ k — "surface name n is classified by kind k." The eight
Conventions of the registry are the complete graph of ▹: a name is
classified exactly as its row states, and there are no other
derivations except by (`inf:kinds:hybrid`).

**Inference rule (Hybrid kinds)** · `inf:kinds:hybrid`

```text
n₁ ▹ k₁      n₂ ▹ k₂      n = n₁ "–" n₂
────────────────────────────────────────  Hybrid
              n ▹ k₁k₂
```

A hybrid environment concatenates its parts' kinds in order: a
Definition–Proposition is classified `defprop` because Definition is
classified `def` and Proposition `prop`. The hybrid rows of the
registry are exactly this rule's instances.

## Invariants · `sec:kinds:invariants`

**Invariant (One kind per concept)** · `inv:kinds:one-kind`
Synonymous names — word-order variants, register variants, classical
doublets — classify to one kind: Proof sketch, Sketch of proof, Outline
of proof, and Idea of proof all carry `sketch`; Axiom and Postulate
carry `ax`; Setting, Setup, and Situation carry `setup`; Dictum, Maxim,
Motto, Saying, and Slogan carry `slogan`. A concept never has two
kinds.

**Invariant (Distinct concepts, distinct kinds)** ·
`inv:kinds:distinctness`
Two distinct genres never share a token, however close their words:
Schema and Scheme, Proposition and Property, Identity and Equation,
Application and Appendix, Record and Report, Question and Quiz all
part. Nor however
close their function: a Criterion characterizes, a Verification
checks, a Postcondition holds, and a Gate blocks until met — four
genres, four kinds. Kind tokens are pairwise distinct across the registry, one per
concept.

**Invariant (Totality)** · `inv:kinds:totality`
Every catalogued name is classified by exactly one kind, or is
expressly declared a presentation device under
(`cav:kinds:presentation`). Nothing is left unclassified.

## Results and assertions · `sec:kinds:results`

**Convention (Results and assertions)** · `conv:kinds:results`
Statements presented or proposed as true: the theorem family proper,
together with conjectural and interrogative variants.

| Environment | Kind |
|---|---|
| Assertion | `claim` |
| Bound | `bound` |
| Claim | `claim` |
| Conjecture | `conj` |
| Consequence | `cor` |
| Corollary | `cor` |
| Criterion | `crit` |
| Estimate | `bound` |
| Fact | `fact` |
| Folklore | `folk` |
| Generalization | `gen` |
| Guess | `guess` |
| Hypothesis | `hyp` |
| Identity | `ident` |
| Inequality | `bound` |
| Key Lemma | `lem` |
| Law | `law` |
| Lemma | `lem` |
| Main Theorem | `thm` |
| Meta-conjecture | `metaconj` |
| Meta-question | `metaq` |
| Meta-theorem | `mthm` |
| Open Problem | `open` |
| Open Question | `open` |
| Paradox | `paradox` |
| Prediction | `pred` |
| Principle | `prin` |
| Property | `prpt` |
| Proposition | `prop` |
| Question | `q` |
| Result | `result` |
| Speculation | `guess` |
| Statement | `stmt` |
| Subclaim | `claim` |
| Sublemma | `lem` |
| Theorem | `thm` |
| Theorem schema | `thmschema` |
| Thesis | `thesis` |
| Variant | `variant` |

## Proofs and arguments · `sec:kinds:proofs`

**Convention (Proofs and arguments)** · `conv:kinds:proofs`
Environments that justify, derive, or deliberately fail to justify a
statement.

| Environment | Kind |
|---|---|
| Answer | `sol` |
| Argument | `arg` |
| Bogus proof | `fallacy` |
| Calculation | `calc` |
| Computation | `calc` |
| Demonstration | `pf` |
| Derivation | `calc` |
| Disproof | `refut` |
| Explanation | `expl` |
| False proof | `fallacy` |
| Heuristic argument | `heur` |
| Hint | `hint` |
| Idea of proof | `sketch` |
| Justification | `just` |
| Outline of proof | `sketch` |
| Plausibility argument | `heur` |
| Proof | `pf` |
| Proof sketch | `sketch` |
| Refutation | `refut` |
| Sanity check | `ver` |
| Sketch of proof | `sketch` |
| Solution | `sol` |
| Strategy | `strat` |
| Verification | `ver` |

## Definitions, axioms, and setup · `sec:kinds:setup`

**Convention (Definitions, axioms, and setup)** · `conv:kinds:setup`
Environments that fix meaning, notation, assumptions, or the ambient
context.

| Environment | Kind |
|---|---|
| Abuse of notation | `abuse` |
| Ansatz | `ansatz` |
| Axiom | `ax` |
| Axiom schema | `axschema` |
| Blanket assumption | `assum` |
| Convention | `conv` |
| Definition | `def` |
| Grammar | `gram` |
| Indexing convention | `conv` |
| Inference rule | `inf` |
| Invariant | `inv` |
| Judgment | `judg` |
| Language | `lang` |
| Nomenclature | `term` |
| Notation | `ntn` |
| Postcondition | `postc` |
| Postulate | `ax` |
| Precondition | `pre` |
| Rule | `rule` |
| Schema | `schema` |
| Setting | `setup` |
| Setup | `setup` |
| Signature | `sig` |
| Sign convention | `conv` |
| Situation | `setup` |
| Specification | `spec` |
| Standing assumption | `assum` |
| Terminology | `term` |
| Working definition | `def` |

## Remarks and meta-commentary · `sec:kinds:commentary`

**Convention (Remarks and meta-commentary)** · `conv:kinds:commentary`
Environments that comment on the work rather than doing it:
asides, warnings, morals, corrections, and editorial notes.

| Environment | Kind |
|---|---|
| Acknowledgment | `ack` |
| Aside | `aside` |
| Caution | `warn` |
| Caveat | `cav` |
| Comment | `rem` |
| Corrigendum | `errat` |
| Dictum | `slogan` |
| Digression | `aside` |
| Discussion | `disc` |
| Erratum | `errat` |
| Excursus | `aside` |
| Expectation | `pred` |
| Fallacy | `fallacy` |
| Fun fact | `fact` |
| Heuristic | `heur` |
| Historical note | `hist` |
| Historical remark | `hist` |
| Insight | `intuit` |
| Interlude | `aside` |
| Intermezzo | `aside` |
| Intuition | `intuit` |
| Maxim | `slogan` |
| Misconception | `myth` |
| Moral | `moral` |
| Motivation | `mot` |
| Myth | `myth` |
| N.B. | `rem` |
| Note | `rem` |
| Observation | `obs` |
| Outlook | `outlook` |
| Perspective | `persp` |
| Philosophy | `persp` |
| Pitfall | `warn` |
| Porism | `por` |
| Preview | `outlook` |
| Recall | `recall` |
| Refrain | `refrain` |
| Remark | `rem` |
| Reminder | `recall` |
| Rule of thumb | `heur` |
| Saying | `slogan` |
| Scholium | `schol` |
| Sidebar | `aside` |
| Slogan | `slogan` |
| Summary | `summ` |
| Takeaway | `moral` |
| Warning | `warn` |

## Examples and exercises · `sec:kinds:examples`

**Convention (Examples and exercises)** · `conv:kinds:examples`
Instances, illustrations, and work assigned to the reader.

| Environment | Kind |
|---|---|
| Activity | `exer` |
| Application | `app` |
| Case study | `app` |
| Challenge | `puzzle` |
| Counterexample | `cex` |
| Example | `ex` |
| Example (continued) | `ex` |
| Exercise | `exer` |
| Exploration | `proj` |
| Illustration | `ex` |
| Non-example | `nonex` |
| Problem | `prob` |
| Project | `proj` |
| Puzzle | `puzzle` |
| Quiz | `quiz` |
| Research problem | `open` |
| Running example | `ex` |
| Special case | `spcase` |
| Story | `story` |
| Subexample | `ex` |
| Task | `exer` |
| Toy example | `ex` |
| Warm-up | `exer` |
| Worked example | `ex` |

## Algorithms, computation, and structured reasoning · `sec:kinds:computation`

**Convention (Algorithms, computation, and structured reasoning)** ·
`conv:kinds:computation`
Procedural, computational, and case-analytic scaffolding.

| Environment | Kind |
|---|---|
| Algorithm | `alg` |
| Assumption | `assum` |
| Case | `case` |
| Code | `listing` |
| Computational note | `impl` |
| Condition | `cond` |
| Construction | `constr` |
| Data | `data` |
| Experiment | `expt` |
| Formulation | `formul` |
| Gate | `gate` |
| Given data | `data` |
| Implementation remark | `impl` |
| Listing | `listing` |
| Model | `model` |
| Numerical example | `ex` |
| Problem formulation | `formul` |
| Procedure | `alg` |
| Protocol | `proto` |
| Pseudocode | `listing` |
| Reduction | `red` |
| Scheme | `scheme` |
| Step | `step` |
| Subcase | `case` |
| Substep | `step` |

## Displays and floats · `sec:kinds:displays`

**Convention (Displays and floats)** · `conv:kinds:displays`
The displayed and floating objects of the document: captioned, cited,
and set off from the running text. A caption is an attached name, and
placement — floating to a page top, gathered at the end — is
presentation too (`cav:kinds:presentation`): the kind names the
object, not where it lands.

| Environment | Kind |
|---|---|
| Array | `mat` |
| Chart | `fig` |
| Diagram | `diag` |
| Equation | `eq` |
| Exhibit | `exhibit` |
| Figure | `fig` |
| Matrix | `mat` |
| Plot | `fig` |
| Table | `tab` |

## Structure and sectioning · `sec:kinds:structure`

**Convention (Structure and sectioning)** · `conv:kinds:structure`
The divisions of the document itself: the sectioning ladder and its
appendages. Nesting is the sub- prefix, iterated at need, and is
presentation (`cav:kinds:presentation`): a subsection is a section,
nested. A named section — an Introduction, a Conclusion — is a section
wearing a name, by the same caveat.

| Environment | Kind |
|---|---|
| Appendix | `appx` |
| Chapter | `chap` |
| Paragraph | `para` |
| Part | `part` |
| Section | `sec` |
| Subparagraph | `para` |
| Subsection | `sec` |
| Subsubsection | `sec` |

## Records and archives · `sec:kinds:records`

**Convention (Records and archives)** · `conv:kinds:records`
The kept documents of a project or institution: records of decisions,
events, meetings, accounts, and changes, maintained over time and
cited long after writing. The numbered decision record is the attested
exemplar of `rec`; an Entry is the dated unit a log, journal, or
ledger accumulates; and recurrence over time is the genre itself here,
never a presentation device. This registry is its own instance — a
kept, cited register, classified `reg` by its own table.

| Environment | Kind |
|---|---|
| Annals | `chron` |
| Changelog | `log` |
| Chronicle | `chron` |
| Decision | `dec` |
| Diary | `jour` |
| Dossier | `dossier` |
| Entry | `entry` |
| Journal | `jour` |
| Ledger | `ledger` |
| Log | `log` |
| Memo | `memo` |
| Memorandum | `memo` |
| Minutes | `minutes` |
| Postmortem | `postmortem` |
| Record | `rec` |
| Register | `reg` |
| Registry | `reg` |
| Report | `rep` |
| Retrospective | `retro` |

## Hybrids, variants, and numbering devices · `sec:kinds:hybrids`

**Convention (Hybrids, variants, and numbering devices)** ·
`conv:kinds:hybrids`
Compound environments, classified by (`inf:kinds:hybrid`), beside
typographical and numbering conventions that are presentation rather
than genre (`cav:kinds:presentation`).

| Environment | Kind |
|---|---|
| Addendum | `adden` |
| Corollary (of the proof) | `por` |
| Definition–Proposition | `defprop` |
| Definition–Theorem | `defthm` |
| Lemma–Definition | `lemdef` |
| Lettered main theorems (Theorem A, Theorem B, …) | — |
| Named theorem notes (e.g., Theorem (Riemann–Roch)) | — |
| Restated theorems (Theorem 1.1, restated) | — |
| Starred/unnumbered variants (theorem*, etc.) | — |

## Whimsical and rare · `sec:kinds:whimsy`

**Convention (Whimsical and rare)** · `conv:kinds:whimsy`
Attested but unusual environments, mostly from lecture notes and
playful authors.

| Environment | Kind |
|---|---|
| Confession | `confess` |
| Curiosity | `fact` |
| Dream | `dream` |
| Fantasy | `dream` |
| Goal | `goal` |
| Hope | `hope` |
| Joke | `joke` |
| Miracle | `miracle` |
| Motto | `slogan` |
| Prayer | `hope` |
| Promise | `promise` |
| Sorites | `sorites` |
| Surprise | `miracle` |
| Wish | `hope` |
| Yoga | `yoga` |

## Caveats · `sec:kinds:caveats`

**Caveat (Presentation devices)** · `cav:kinds:presentation`
Numbering, lettering (Theorem A), attached names (Theorem
(Riemann–Roch)), stars and unnumbering (theorem*), restatement
("restated"), continuation ("continued"), and the sub- prefix,
iterated at need (Sublemma, Subclaim, Subexample, Subcase, Substep,
Subsection, Subsubsection, Subparagraph), are presentation, not genre:
each such occurrence carries the kind of its base environment. In particular a restated theorem cites its original
rather than minting anew — replacing exactly these devices is what
labels are for. A Refrain is the genre built on this mechanic: it
mints once, at its first statement, and every later return of the
refrain is a citation.

**Caveat (One word, several homes)** · `cav:kinds:homonyms`
Some words are catalogued in more than one family; the concept, not the
family, carries the kind. Heuristic commentary and heuristic argument
are one genre, `heur`; Fallacy names the same exhibit whether filed
with the false proofs or the misconceptions, `fallacy`; an Assumption
is `assum` whether ambient or case-analytic; a Porism and a Corollary
of the proof are one classical genre, `por`. Hypothesis, by contrast,
is classified where the catalogue attests it — among the conjectural
statements as `hyp` — and an ambient hypothesis is written as an
assumption. Illustration is catalogued among the examples, as the
worked instance; the pictorial illustration is a `fig`. The minutes of
a meeting are `minutes` even where local usage calls them a protocol,
Protocol remaining the procedural genre. And a Historical note
comments while a Chronicle records: commentary and record part even
when they concern the same past.

**Caveat (Attestation)** · `cav:kinds:attestation`
The catalogue is descriptive: every name is attested, but the whimsical
kinds — `dream`, `miracle`, `yoga` and their neighbours — are rare, and
Yoga and Meta-question are borderline, flagged rather than firmly
attested. Adopting the registry does not oblige a corpus to use any
kind; it fixes what each kind means when used.

## Rejected Ansätze · `sec:kinds:rejected-ansaetze`

**Ansatz (One kind per name)** · `ansatz:kinds:kind-per-name`
Give every surface name its own token. Then Proof sketch and Sketch of
proof mint under different kinds, citations fragment by spelling, and a
renamed heading orphans its readers — against (`inv:kinds:one-kind`).
Rejected.

**Ansatz (Kinds from families)** · `ansatz:kinds:family-kinds`
Issue one token per family. Then a theorem, a lemma, and a conjecture
all carry the same kind, a citation can no longer say what it cites,
and (`inv:kinds:distinctness`) holds only vacuously. Rejected.

**Ansatz (Numbering as kinds)** · `ansatz:kinds:numbering`
Encode letters and numbers in the token — a kind for Theorem A, a kind
for Theorem 1.1 restated. Then presentation enters denotation and every
restatement mints anew, against (`cav:kinds:presentation`). Rejected.

## Postcondition · `sec:kinds:postcondition`

**Postcondition (Adoption)** · `postc:kinds:adoption`
A corpus has adopted the registry when all of the following hold:

- every catalogued name is classified by exactly one kind, or expressly
  declared a presentation device;
- kind tokens are pairwise distinct across the registry, one per
  concept;
- synonym classes are convergent: word-order variants, register
  variants, and classical doublets share one kind;
- the hybrid rows are exactly the instances of the Hybrid rule;
- this document's own environment heads and heading anchors are
  classified by the registry they define;
- the document carries exactly one mint per environment and no
  environment numbering, and every citation in it resolves.
