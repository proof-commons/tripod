# A Taxonomy of Environments in Formal Writing, as a Kind Registry

This document constitutes the taxonomy of environments as a kind registry: a catalogue of the environments of formal writing — headed like the theorem family, displayed like the equation, structural like the section, kept like the record, carried by the code like the test — from papers, monographs, textbooks, lecture notes, standards, specifications, decision records, and the labeled assets of their code, in which every distinct genre carries one kind, a short token fit to serve as the first segment of a label. Synonymous surface names share a kind, and a homonymous name carries one kind per catalogued sense: a Language fixes the token shape, two Signatures name the registry's data and its acceptee, one Inference rule composes hybrid kinds, three Judgments classify names, validate heads, and record attestation, two Definitions derive homonymy and presentation reduction, five Invariants govern the assignment and its evidence, one Requirement binds the acceptee's companion register, fourteen Conventions lay out the registry itself, a Diagram charts their dependencies, a Table presents the headline counts, Caveats bound it, four rejected Ansätze delimit it negatively, and a Gate blocks adoption until met. The catalogue's headline counts — its names, its rows, its kinds, its declared hybrids, its device classes — are derived from the tables by the register generator and presented in (`tab:kinds:headline-counts`), maintained by hand nowhere, this sentence included; the preamble's inventory of this document's own environments is, by contrast, hand-maintained prose, checked by the gate like every other head. The registry is normative in its kind assignments and evidence-conditioned in its inventory of names: the Convention tables lay down the classification relation, and each edition carries its own evidence base, from which the attestation status printed at each row derives. Adoption names a corpus acceptee, who owns that corpus's evidence, extensions, and generated companion register; this document materializes no index of any corpus's facts, its own headline counts excepted, which derive from its tables alone, and the registry classifies this document's own environment heads. This document is self-contained and cites only itself; where an adopting corpus uses it beside other disciplines, their alignment is fixed by the corpus's recorded adoption decisions, not by this text. Acceptance of this document presupposes acceptance of no other document: where another discipline's artifact is consumed — a set of reserved kinds, a labeling convention, a format's declared environment classes — this registry consumes the artifact as adoption data and asks nothing of its provenance.

The document practices the discipline it serves. The label at each heading or environment head is that environment's mint; a parenthesized label in running text is a same-owner citation; material in fenced blocks, double-backtick spans, and plain code spans that are not label-shaped — including every kind token in the tables — is displayed without participating. The document title is publication metadata, not an environment head; it mints nothing and participates in nothing. Every label minted here has area `kinds`, and each heading anchor carries `sec` — the kind this registry assigns to the section — so the document's skeleton is classified by its own tables. Environments carry no numbers: the mint at each head is the sole name of its environment.

## Syntax · `sec:kinds:syntax`

**Language (Kinds and names)** · `lang:kinds:kind-language`

A kind is a word over lowercase letters and digits, fit to stand as the first segment of a label; an environment name is the surface heading an author writes, in natural language. Many names may share one kind, and one name may carry several kinds, one per catalogued sense; an occurrence carries exactly one, selected at its label. A kind token is the most standard written abbreviation the conventions of formal writing attest for its genre; where that abbreviation is entrenched for another reading — a negation, a summation, an error, an exponential, a version — the registry deviates deliberately, and the deviation is recorded.

```text
kind  ::=  word              word  ::=  [a-z0-9]+

a kind in use, as a label's first segment:   `thm:analysis:mean-value`
a citation of that label:                    (`thm:analysis:mean-value`)
```

## Registry and authorities · `sec:kinds:registry`

**Signature (Registry data)** · `sig:kinds:registry-data`

Let N be the set of exact catalogue names and K the set of kind tokens. The ordinary rows of the Convention tables — every row other than the hybrid rows and the device rows of (`conv:kinds:hybrids`) — determine B ⊆ N × K; the declared hybrid triples determine H, and (`inf:kinds:hybrid`) derives the hybrid rows. The base classification relation C is exactly the union of the ordinary and derived hybrid rows; no heading occurrence, presentation device, attestation record, generated presentation, or candidate pair contributes a member to C. The registry authority owns B, H, and the edition evidence base of (`judg:kinds:attestation`); a change to any of them is a new edition of the registry, not a consequence of adoption. An adopting corpus may record local extension rows X_A; its effective relation is C_A = C ∪ X_A. A local extension is not a row of C and becomes one only when a later edition expressly incorporates it.

**Signature (Acceptee)** · `sig:kinds:acceptee`

Adoption names exactly one acceptee A: the authority responsible for accepting and maintaining the adopted registry material. A owns the local extension set X_A; the evidence base E_A; the status map σ_A; the register generator G_A; and the generated companion register Ê_A. E_A has two parts: an adopted component — the edition evidence base of this registry, taken by reference in the recorded adoption decision — and an owned component, held first-hand, covering X_A and any status A strengthens. Each owned record identifies a name-and-kind pair (n, k), an exact quoted spelling, a source, a locator, and context enough to adjudicate the catalogued sense. For the corpus in which this registry itself travels, the registry authority is the acceptee, and the edition evidence base is E_A entire.

**Diagram (Dependencies)** · `diag:kinds:dependencies`

An arrow points from dependency to dependent.

```text
lang:kinds:kind-language
    └─→ sig:kinds:registry-data
            ├─→ conv:kinds:* (fourteen)
            │       ├─→ inf:kinds:hybrid ─┐
            │       └────────────────────┴─→ judg:kinds:classification
            │                                   ├─→ inv:kinds:one-kind
            │                                   ├─→ inv:kinds:distinctness
            │                                   ├─→ def:kinds:homonymy
            │                                   │       └─→ cav:kinds:homonymy
            │                                   └─→ def:kinds:presentation-reduction
            │                                           └─→ judg:kinds:head-validation
            │                                                   ├─→ inv:kinds:totality
            │                                                   └─→ inv:kinds:catalogued-pairs
            └─→ sig:kinds:acceptee
                    ├─→ (X_A) ─→ judg:kinds:classification        [C_A]
                    └─→ judg:kinds:attestation
                            ├─→ req:kinds:attestation-register
                            ├─→ inv:kinds:attestation-coverage
                            └─→ cav:kinds:attestation-limits

every normative node ─→ gate:kinds:adoption
   [gate consults: this document + adoption data only]
```

**Table (Headline counts)** · `tab:kinds:headline-counts`

A generated region: the five counts below are derived from the Convention tables by the register generator — names counted distinct after the dagger normalization of (`judg:kinds:attestation`), rows counted over C with the derived hybrid rows included and the device rows excluded — and the table is maintained only by regeneration.

| Measure          | Count |
| ---------------- | ----- |
| Names            | 333   |
| Rows             | 349   |
| Kinds            | 208   |
| Declared hybrids | 3     |
| Device classes   | 4     |

## Assignment · `sec:kinds:assignment`

**Judgment (Classification)** · `judg:kinds:classification`

Form: C_A ⊢ n ▹ k — "under the effective registry C_A, exact catalogue name n is classified by kind k," holding exactly when (n, k) ∈ C_A. Unqualified, n ▹ k abbreviates C ⊢ n ▹ k. Classification is a relation, not a function: one name may carry several kinds, one per catalogued sense, and several names one kind. The label at a head declares the intended kind; the registry validates the pair. The Convention rows and the declared instances of (`inf:kinds:hybrid`) are the only sources of C, and recorded extensions the only further source of C_A: attestation evidence, presentation reduction, and generated presentations derive no classification pair.

**Inference rule (Hybrid kinds)** · `inf:kinds:hybrid`

```text
n₁ ▹ k₁      n₂ ▹ k₂      n = n₁ "–" n₂      (n, n₁, n₂) declared
──────────────────────────────────────────────────────────  Hybrid
                          n ▹ k₁k₂
```

A hybrid environment concatenates its parts' kinds in order: a Definition–Proposition is classified `defprop` because Definition is classified `def` and Proposition `prop`. The declared triples are exactly the hybrid rows of the registry, and the parts are non-hybrid names. Side conditions, checked at declaration: the composed token is not otherwise assigned (`inv:kinds:distinctness`), and no two declared hybrids compose one token. A local hybrid belongs to X_A, declared by the same recorded adoption decision that introduces it (`sig:kinds:acceptee`); in either case its parts are non-hybrid names, its token otherwise unused, and no two declarations compose one token. The composition is not decoded: a hybrid token names its compound genre directly.

**Definition (Homonymy)** · `def:kinds:homonymy`

For any classification relation R, Hom(R) = { (n, k) ∈ R : there is k′ ≠ k with (n, k′) ∈ R }; a name is homonymous under R exactly when it occurs in Hom(R). A row is the catalogue of its sense: two rows sharing a name catalogue two senses, which the owning Convention's prose names where the distinction is not evident from the tables' subject matter. Homonymy is derived, never declared: no row here, and no acceptee record, states it directly. This document materializes no homonym index; presenting Hom(C_A) is the acceptee's obligation (`req:kinds:attestation-register`), because homonymy is a fact of the effective relation, which exists only per corpus.

## Results and assertions · `sec:kinds:results`

**Convention (Results and assertions)** · `conv:kinds:results`

Statements presented or proposed as true: the theorem family proper, together with conjectural and interrogative variants. Emphasis names — a Main Theorem, a Key Lemma — classify by their base under the modifier rule of (`def:kinds:presentation-reduction`).

| Environment       | Kind        |
| ----------------- | ----------- |
| Assertion         | `claim`     |
| Bound             | `bound`     |
| Characterization  | `crit`      |
| Claim             | `claim`     |
| Conjecture        | `conj`      |
| Consequence       | `cor`       |
| Corollary         | `cor`       |
| Criterion         | `crit`      |
| Estimate          | `bound`     |
| Fact              | `fact`      |
| Folklore          | `folk`      |
| Generalization    | `gen`       |
| Guess             | `guess`     |
| Hypothesis        | `hyp`       |
| Identity          | `ident`     |
| Inequality        | `bound`     |
| Law               | `law`       |
| Lemma             | `lem`       |
| Meta-conjecture   | `metaconj`  |
| Meta-question †   | `metaq`     |
| Meta-theorem      | `metathm`   |
| Open Problem      | `open`      |
| Open Question     | `open`      |
| Paradox           | `paradox`   |
| Prediction        | `pred`      |
| Principle         | `prin`      |
| Property          | `property`  |
| Proposition       | `prop`      |
| Question          | `q`         |
| Research question | `open`      |
| Result            | `result`    |
| Speculation       | `guess`     |
| Statement         | `stmt`      |
| Theorem           | `thm`       |
| Theorem schema    | `thmschema` |
| Thesis            | `thesis`    |
| Variant           | `variant`   |

## Proofs and arguments · `sec:kinds:proofs`

**Convention (Proofs and arguments)** · `conv:kinds:proofs`

Environments that justify, derive, or deliberately fail to justify a statement. Answer and Solution are one genre, `sol`, by (`inv:kinds:one-kind`) — deliberately so, although Question and Problem part: the response genre does not inherit the split of what it responds to.

| Environment           | Kind      |
| --------------------- | --------- |
| Answer                | `sol`     |
| Argument              | `arg`     |
| Bogus proof           | `fallacy` |
| Calculation           | `calc`    |
| Check                 | `verif`   |
| Computation           | `calc`    |
| Demonstration         | `pf`      |
| Derivation            | `calc`    |
| Disproof              | `refut`   |
| Explanation           | `expl`    |
| False proof           | `fallacy` |
| Heuristic argument    | `heur`    |
| Hint                  | `hint`    |
| Idea of proof         | `sketch`  |
| Justification         | `just`    |
| Objection             | `obj`     |
| Outline of proof      | `sketch`  |
| Plausibility argument | `heur`    |
| Proof                 | `pf`      |
| Proof idea            | `sketch`  |
| Proof outline         | `sketch`  |
| Proof sketch          | `sketch`  |
| Refutation            | `refut`   |
| Reply                 | `reply`   |
| Sanity check          | `verif`   |
| Sketch                | `sketch`  |
| Sketch of proof       | `sketch`  |
| Solution              | `sol`     |
| Strategy              | `strat`   |
| Verification          | `verif`   |

## Definitions, axioms, requirements, and setup · `sec:kinds:setup`

**Convention (Definitions, axioms, requirements, and setup)** · `conv:kinds:setup`

Environments that fix meaning, notation, assumptions, requirements, or the ambient context. Construction, Model, and Structure are one classical genre in their fixed-object sense — the posited tuple, the satisfying structure — and converge on `constr` by (`inv:kinds:one-kind`); Model in its computational sense is catalogued separately (`conv:kinds:computation`), and Structure in its declared-shape sense is Schema's doublet. Working hypothesis and Standing hypothesis are catalogued expressly as overrides of the modifier rule of (`def:kinds:presentation-reduction`): their base, Hypothesis, is conjectural, but the ambient sense is assumptive, and the rows say so.

| Environment         | Kind       |
| ------------------- | ---------- |
| Abuse of notation   | `abuse`    |
| Ansatz              | `ansatz`   |
| Axiom               | `ax`       |
| Axiom schema        | `axschema` |
| Convention          | `conv`     |
| Definition          | `def`      |
| Grammar             | `gram`     |
| Indexing convention | `conv`     |
| Inference rule      | `inf`      |
| Invariant           | `inv`      |
| Judgment            | `judg`     |
| Language            | `lang`     |
| Model               | `constr`   |
| Nomenclature        | `term`     |
| Notation            | `ntn`      |
| Postcondition       | `postc`    |
| Postulate           | `ax`       |
| Precondition        | `pre`      |
| Requirement         | `req`      |
| Rule                | `rule`     |
| Schema              | `schema`   |
| Setting             | `setup`    |
| Setup               | `setup`    |
| Sign convention     | `conv`     |
| Signature           | `sig`      |
| Situation           | `setup`    |
| Specification       | `spec`     |
| Standing hypothesis | `assum`    |
| Structure           | `constr`   |
| Structure           | `schema`   |
| Terminology         | `term`     |
| Working hypothesis  | `assum`    |

## Remarks and meta-commentary · `sec:kinds:commentary`

**Convention (Remarks and meta-commentary)** · `conv:kinds:commentary`

Environments that comment on the work rather than doing it: asides, warnings, morals, corrections, and editorial notes. A Preview looks ahead within the document; an Outlook looks beyond it — two genres, two kinds. An Interlude is an Aside at section scale, and scale is presentation.

| Environment              | Kind      |
| ------------------------ | --------- |
| Acknowledgment           | `ack`     |
| Addendum                 | `adden`   |
| Aside                    | `aside`   |
| Caution                  | `warn`    |
| Caveat                   | `cav`     |
| Comment                  | `rem`     |
| Corollary (of the proof) | `por`     |
| Corrigendum              | `errat`   |
| Dictum                   | `slogan`  |
| Digression               | `aside`   |
| Discussion               | `disc`    |
| Erratum                  | `errat`   |
| Excursus                 | `aside`   |
| Expectation              | `pred`    |
| Fallacy                  | `fallacy` |
| Fun fact                 | `fact`    |
| Heuristic                | `heur`    |
| Historical note          | `hist`    |
| Historical remark        | `hist`    |
| Insight                  | `intuit`  |
| Interlude                | `aside`   |
| Intermezzo               | `aside`   |
| Intuition                | `intuit`  |
| Maxim                    | `slogan`  |
| Misconception            | `myth`    |
| Moral                    | `moral`   |
| Motivation               | `mot`     |
| Myth                     | `myth`    |
| N.B.                     | `rem`     |
| Note                     | `rem`     |
| Observation              | `obs`     |
| Outlook                  | `outlook` |
| Overview                 | `preview` |
| Perspective              | `persp`   |
| Philosophy               | `persp`   |
| Pitfall                  | `warn`    |
| Porism                   | `por`     |
| Preview                  | `preview` |
| Punchline                | `moral`   |
| Recall                   | `recall`  |
| Refrain                  | `refrain` |
| Remark                   | `rem`     |
| Reminder                 | `recall`  |
| Roadmap                  | `preview` |
| Rule of thumb            | `heur`    |
| Saying                   | `slogan`  |
| Scholium                 | `schol`   |
| Sidebar                  | `aside`   |
| Slogan                   | `slogan`  |
| Summary                  | `summ`    |
| Takeaway                 | `moral`   |
| Upshot                   | `moral`   |
| Warning                  | `warn`    |

## Examples and exercises · `sec:kinds:examples`

**Convention (Examples and exercises)** · `conv:kinds:examples`

Instances, illustrations, and work assigned to the reader. Toy, Worked, Running, and Numerical examples classify by their base under the modifier rule of (`def:kinds:presentation-reduction`); a Non-example, a Counterexample, and a Special case are not modifications of Example but genres of their own.

| Environment      | Kind        |
| ---------------- | ----------- |
| Activity         | `exer`      |
| Anecdote         | `story`     |
| Application      | `appl`      |
| Assignment       | `exer`      |
| Case study       | `casestudy` |
| Challenge        | `puzzle`    |
| Counterexample   | `cex`       |
| Demonstration    | `ex`        |
| Drill            | `exer`      |
| Example          | `ex`        |
| Exercise         | `exer`      |
| Exploration      | `proj`      |
| Homework         | `exer`      |
| Illustration     | `ex`        |
| Legend           | `story`     |
| Non-example      | `nonex`     |
| Parable          | `story`     |
| Practice         | `exer`      |
| Problem          | `prob`      |
| Project          | `proj`      |
| Puzzle           | `puzzle`    |
| Quiz             | `quiz`      |
| Research problem | `open`      |
| Riddle           | `puzzle`    |
| Special case     | `spcase`    |
| Story            | `story`     |
| Task             | `exer`      |
| Test             | `quiz`      |
| Vignette         | `story`     |
| Warm-up          | `exer`      |

## Algorithms, computation, and structured reasoning · `sec:kinds:computation`

**Convention (Algorithms, computation, and structured reasoning)** · `conv:kinds:computation`

Procedural, computational, and case-analytic scaffolding. A Thought experiment executes nothing and is its own genre, not an Experiment.

| Environment           | Kind       |
| --------------------- | ---------- |
| Algorithm             | `alg`      |
| Assumption            | `assum`    |
| Case                  | `case`     |
| Code                  | `listing`  |
| Computational note    | `impl`     |
| Condition             | `cond`     |
| Construction          | `constr`   |
| Data                  | `data`     |
| Experiment            | `expt`     |
| Formulation           | `formul`   |
| Gate                  | `gate`     |
| Given data            | `data`     |
| Implementation remark | `impl`     |
| Listing               | `listing`  |
| Model                 | `model`    |
| Observation           | `data`     |
| Problem formulation   | `formul`   |
| Procedure             | `alg`      |
| Protocol              | `proto`    |
| Pseudocode            | `listing`  |
| Reduction             | `red`      |
| Scenario              | `scenario` |
| Scheme                | `scheme`   |
| Simulation            | `expt`     |
| Step                  | `step`     |
| Story                 | `scenario` |
| Thought experiment    | `gedanken` |
| Use case              | `scenario` |

## Displays and floats · `sec:kinds:displays`

**Convention (Displays and floats)** · `conv:kinds:displays`

The displayed and floating objects of the document: captioned, cited, and set off from the running text. A bare caption is an attached name; a caption carrying content of its own is catalogued in (`conv:kinds:apparatus`). Placement — floating to a page top, gathered at the end — is presentation (`def:kinds:presentation-reduction`): the kind names the object, not where it lands.

| Environment  | Kind      |
| ------------ | --------- |
| Array        | `mat`     |
| Chart        | `fig`     |
| Diagram      | `diag`    |
| Equation     | `eq`      |
| Exhibit      | `exhibit` |
| Figure       | `fig`     |
| Graph        | `fig`     |
| Illustration | `fig`     |
| Image        | `fig`     |
| Matrix       | `mat`     |
| Photograph   | `fig`     |
| Picture      | `fig`     |
| Plot         | `fig`     |
| Scheme       | `fig`     |
| Table        | `tab`     |

## Apparatus and annotation · `sec:kinds:apparatus`

**Convention (Apparatus and annotation)** · `conv:kinds:apparatus`

Environments that annotate other environments. The governing test: apparatus that adds decodable content of its own is a genre; apparatus that only names, places, or contains is presentation (`def:kinds:presentation-reduction`). A Legend decodes symbols — a mapping, not a name; a content-bearing Caption instructs the reading of its float; an Epigraph is the attested quotation genre; a Credit records provenance of the displayed material; a Gloss decodes a piece of displayed content, where a Remark comments on the work.

| Environment    | Kind       |
| -------------- | ---------- |
| Annotation     | `gloss`    |
| Attribution    | `credit`   |
| Caption        | `caption`  |
| Courtesy line  | `credit`   |
| Credit         | `credit`   |
| Epigraph       | `epigraph` |
| Gloss          | `gloss`    |
| Key            | `legend`   |
| Legend         | `legend`   |
| Marginal gloss | `gloss`    |
| Source line    | `credit`   |

## Structure and sectioning · `sec:kinds:structure`

**Convention (Structure and sectioning)** · `conv:kinds:structure`

The divisions of the document itself: the sectioning ladder and its appendages. The ladder's rungs are genres — the conventions of formal writing head and refer to each rung in its own right — while nesting within a rung is the sub- prefix, iterated at need, and is presentation (`def:kinds:presentation-reduction`): a subsection is a section, nested, and a subclause a clause. Rank is genre; scale alone is not. A named division — an Introduction, a Conclusion, a Preliminaries — is a division wearing a name, by the same caveat, so any name may pair with a rung's kind as presentation; where such a name is also a catalogued environment — a Discussion, a Motivation — the two pairs are distinct senses, and the label's kind token declares which is meant (`cav:kinds:homonymy`). A lecture headed by its schedule — "Week 5" — carries a numbering device on a Lecture, and an annex qualified "(normative)" carries lettering and a status note, both devices, base `app`.

| Environment | Kind   |
| ----------- | ------ |
| Annex       | `app`  |
| Appendix    | `app`  |
| Book        | `book` |
| Chapter     | `chap` |
| Clause      | `sec`  |
| Lecture     | `lect` |
| Module      | `sec`  |
| Paragraph   | `para` |
| Part        | `part` |
| Review      | `sec`  |
| Section     | `sec`  |
| Unit        | `unit` |
| Volume      | `vol`  |

## Front and back matter · `sec:kinds:front-matter`

**Convention (Front and back matter)** · `conv:kinds:front-matter`

The framing environments outside the sectioning ladder: what a document says about itself before and after it says anything else. Placement at front or back is presentation (`def:kinds:presentation-reduction`); the genre is the frame. Glossary, Index, Bibliography, and the Lists of figures, symbols, and tables are generated registers — kept and cited like the artifacts of (`conv:kinds:records`), classified by no kind of their own.

| Environment | Kind    |
| ----------- | ------- |
| Abstract    | `abst`  |
| Afterword   | `adden` |
| Dedication  | `dedic` |
| Epilogue    | `adden` |
| Foreword    | `pref`  |
| Postscript  | `adden` |
| Preamble    | `pref`  |
| Preface     | `pref`  |
| Prologue    | `pref`  |
| Supplement  | `adden` |
| Synopsis    | `abst`  |

## Records and archives · `sec:kinds:records`

**Convention (Records and archives)** · `conv:kinds:records`

The kept documents of a project or institution: records of decisions, events, meetings, accounts, changes, and versions, maintained over time and cited long after writing. The numbered decision record is the attested exemplar of `rec`; an Entry is the dated unit a log, journal, or ledger accumulates; and recurrence over time is the genre itself here, never a presentation device. A Version is the recorded statement of what a version comprises and changes — the version value it states is data, not an environment; an Amendment modifies, an Addendum extends, a Corrigendum corrects — three genres; and a Version history is no environment but a log or register that Version statements accumulate. Proposal carries the full word as its token: the standard abbreviation is entrenched for Proposition, and the deviation is deliberate. This registry is its own instance — a kept, cited register, classified `reg` by its own table.

| Environment   | Kind         |
| ------------- | ------------ |
| Agenda        | `agenda`     |
| Amendment     | `amend`      |
| Annals        | `chron`      |
| Catalogue     | `reg`        |
| Changelog     | `log`        |
| Chronicle     | `chron`      |
| Decision      | `dec`        |
| Diary         | `jour`       |
| Dossier       | `dossier`    |
| Entry         | `entry`      |
| Inventory     | `reg`        |
| Journal       | `jour`       |
| Ledger        | `ledger`     |
| Log           | `log`        |
| Memo          | `memo`       |
| Memorandum    | `memo`       |
| Minutes       | `minutes`    |
| Postmortem    | `postmortem` |
| Proposal      | `proposal`   |
| Protocol      | `minutes`    |
| Record        | `rec`        |
| Register      | `reg`        |
| Registry      | `reg`        |
| Release notes | `relnotes`   |
| Report        | `rep`        |
| Retrospective | `retro`      |
| Review        | `rep`        |
| Revision      | `ver`        |
| Version       | `ver`        |

## Assets and inventory · `sec:kinds:assets`

**Convention (Assets and inventory)** · `conv:kinds:assets`

The labeled constructs of code: the units a codebase names and keeps — the packages and the modules and namespaces within them; the functions, types, classes, and interfaces they expose; the tests, suites, benchmarks, fixtures, and fuzz targets that exercise them; the services, endpoints, jobs, and pipelines that run them; and the migrations, flags, settings, metrics, events, and error codes they operate by. The family's mark is that the name is the code's own: an asset is headed by the identifier it already bears, where every other family's head is a heading an author composes.

| Environment          | Kind         |
| -------------------- | ------------ |
| API item             | `api`        |
| Alert                | `alert`      |
| Benchmark            | `bench`      |
| CLI command          | `cli`        |
| Class                | `class`      |
| Dataset              | `dataset`    |
| Endpoint             | `endpoint`   |
| Environment variable | `envvar`     |
| Error code           | `errcode`    |
| Event                | `event`      |
| Feature flag         | `flag`       |
| Fixture              | `fixture`    |
| Function             | `func`       |
| Fuzz target          | `fuzz`       |
| Interface            | `iface`      |
| Job                  | `job`        |
| Library              | `lib`        |
| Lint rule            | `lint`       |
| Macro                | `macro`      |
| Metric               | `metric`     |
| Migration            | `migr`       |
| Module               | `mod`        |
| Namespace            | `ns`         |
| Package              | `pkg`        |
| Pipeline             | `pipeline`   |
| Query                | `query`      |
| Role                 | `role`       |
| Route                | `endpoint`   |
| Runnable example     | `runex`      |
| Schema †             | `dataschema` |
| Script               | `script`     |
| Service              | `svc`        |
| Setting              | `setting`    |
| Snapshot             | `snapshot`   |
| Struct               | `class`      |
| Structure            | `class`      |
| Suite                | `suite`      |
| Task                 | `job`        |
| Test                 | `test`       |
| Type                 | `type`       |
| Workflow             | `pipeline`   |

## Hybrids, variants, and numbering devices · `sec:kinds:hybrids`

**Convention (Hybrids, variants, and numbering devices)** · `conv:kinds:hybrids`

Compound environments, classified by the declared instances of (`inf:kinds:hybrid`), beside the device classes — families of spellings, not single names — that are presentation rather than genre (`def:kinds:presentation-reduction`).

| Environment                                        | Kind      |
| -------------------------------------------------- | --------- |
| Definition–Proposition                             | `defprop` |
| Definition–Theorem                                 | `defthm`  |
| Lemma–Definition                                   | `lemdef`  |
| Lettered main theorems (Theorem A, Theorem B, …)   | —         |
| Named theorem notes (e.g., Theorem (Riemann–Roch)) | —         |
| Restated theorems (Theorem 1.1, restated)          | —         |
| Starred/unnumbered variants (theorem*, etc.)       | —         |

## Whimsical and rare · `sec:kinds:whimsy`

**Convention (Whimsical and rare)** · `conv:kinds:whimsy`

Attested but unusual environments, mostly from lecture notes and playful authors. The dagger marks borderline attestation, per (`judg:kinds:attestation`).

| Environment | Kind      |
| ----------- | --------- |
| Confession  | `confess` |
| Curiosity   | `fact`    |
| Desideratum | `goal`    |
| Dream       | `dream`   |
| Fantasy     | `dream`   |
| Goal        | `goal`    |
| Hope        | `hope`    |
| Joke        | `joke`    |
| Miracle     | `miracle` |
| Motto       | `slogan`  |
| Prayer      | `hope`    |
| Promise     | `promise` |
| Sorites     | `sorites` |
| Surprise    | `miracle` |
| Wish        | `hope`    |
| Yoga †      | `yoga`    |

## Presentation and head validation · `sec:kinds:presentation`

**Definition (Presentation reduction)** · `def:kinds:presentation-reduction`

For an authored head h, base_A(h) is the exact catalogue name obtained after removing the devices this registry admits: numbering, lettering, attached names, stars and unnumbering, restatement, continuation, iterated sub- prefixes, placement, containment, and the catalogued emphasis and status modifiers — Main, Key, Fundamental, Working, Standing, Blanket, Concrete, Motivating, Numerical, Toy, Worked, and Running. Which environment class a document format declares for a head, and how a head maps to one, are adoption data. An expressly catalogued overriding row takes precedence over reduction: Working hypothesis and Standing hypothesis reduce to themselves and carry `assum`. Where a name is both an exact row and a division head, the label's kind token selects the sense (`cav:kinds:homonymy`), and reduction supplies the rung. For named divisions — an Introduction, a Conclusion — the underlying rung supplied by the format is the base. Reduction changes no member of C_A and creates no pair.

**Judgment (Head validation)** · `judg:kinds:head-validation`

Form: C_A ⊢ h ✓ k, holding exactly when h is an exact catalogue name with C_A ⊢ h ▹ k, or h is not an overriding row, base_A(h) = n, and C_A ⊢ n ▹ k. Generated registers are not authored heads and form no judgment. Validation consumes a classification; it never extends the relation.

Placement devices follow: a Footnote, an Endnote, and a Marginal note are placed remarks, classified by what they say, not where they sit. Proof variants follow: an Alternative proof, a Second proof, and a Proof of a named theorem are presentation of the proof genre, exactly parallel to restatement. Containers follow: a Box, a Panel, and a Callout hold an environment and are not one; and alt text is a rendering of its float's content, not a genre. In particular a restated theorem is its original returned to, not a new environment: it refers, and names nothing new. A Refrain is the genre built on this mechanic: it is stated once and thereafter returned to, and the returns name nothing new.

## Invariants · `sec:kinds:invariants`

**Invariant (One kind per concept)** · `inv:kinds:one-kind`

Synonymous names — word-order variants, register variants, classical doublets — classify to one kind: Proof sketch, Sketch of proof, Outline of proof, and Idea of proof all carry `sketch`; Axiom and Postulate carry `ax`; Setting in its ambient sense, Setup, and Situation carry `setup`; Dictum, Maxim, Motto, Saying, and Slogan carry `slogan`; Struct and Structure carry `class` among the assets. A concept never has two kinds — though a name may carry several, one per distinct concept it names.

**Invariant (Distinct concepts, distinct kinds)** · `inv:kinds:distinctness`

Two distinct genres never share a token, however close their words: Schema and Scheme, Proposition and Property, Identity and Equation, Record and Report, Question and Quiz all part. Nor do they share a kind merely because their functions are close: a Criterion characterizes, a Verification checks, a Postcondition holds, and a Gate blocks until met — four genres, four kinds. Distinctness is the registry's: two names share a kind exactly when the registry treats them as one citation-relevant genre, an equivalence that may collapse narrower presentational or pedagogical subgenres without claiming the names interchangeable in every context. Kind tokens are pairwise distinct across the registry, one per concept, and distinctness is byte-distinctness: near-misses in spelling — `rec`, `reg`, `rep`, and `req`; `prop` and `proposal`; `app` and `appl`; `obs` and `obj`; `crit` and `credit`; `legend` and `ledger`; `ver` and `verif`; `setting` and `setup`; `schema` and `dataschema` — are distinct tokens, and their nearness is an editorial concern, never a logical one.

**Invariant (Totality)** · `inv:kinds:totality`

Every participating authored head is the subject of exactly one head-validation judgment for the kind its label carries — by an exact pair of C_A or by reduction through exactly one base pair (`judg:kinds:head-validation`). Generated registers are outside this requirement.

**Invariant (Catalogued pairs)** · `inv:kinds:catalogued-pairs`

Every exact pair used as the base of a head-validation judgment belongs to C_A: a pair outside C is usable only as a member of the acceptee's recorded X_A, and an unrecorded pair fails. Reduction does not exempt its base pair, and an attestation record supplies no missing classification.

## Attestation and evidence · `sec:kinds:attestation`

**Judgment (Attestation status)** · `judg:kinds:attestation`

Form: A ⊢ (n, k) ⇑ q, with q ∈ {firm, borderline, candidate}, holding exactly when σ_A(n, k) = q and E_A carries evidence for the pair. The dagger printed at a row is a status mark on the row, never a character of the name: the exact catalogue name is the row's name with the mark removed. Evidence is held by reference for base rows, first-hand for extensions and strengthenings (`sig:kinds:acceptee`). Firm accepts the evidence as ordinary attestation; borderline accepts the pair while qualifying its evidence; candidate retains evidence without admitting the pair to C_A. An acceptee may strengthen a base row's edition status — borderline to firm, recorded first-hand — and never weakens one. Attestation is an acceptee-owned recorded judgment: not derived from classification, and deriving no classification.

**Requirement (Companion attestation register)** · `req:kinds:attestation-register`

The acceptee owns, generates, and maintains Ê_A = G_A(E_A, σ_A), under a total recorded ordering — name, kind, source, locator, then a record sequence number as tiebreak. Beside its evidence and status rows, the register presents exactly Hom(C_A) (`def:kinds:homonymy`) as its homonym section, derived from the same pairs: the corpus consults its own register, never another's and never a materialization elsewhere, for which names require the label's kind token to disambiguate. Both presentations are views of one evidence base; neither creates a classification, an attestation judgment, or a homonymy fact. Maintenance is by regeneration and exactness check; hand-editing Ê_A is a failure.

**Invariant (Attestation coverage)** · `inv:kinds:attestation-coverage`

Edition clause: a base row of C carries the dagger (†) exactly when the registry authority's edition evidence records borderline status; the dagger changes only by edition. Corpus clause: for every (n, k) ∈ C_A, exactly one of firm and borderline holds under A, no weaker than the edition status of a base row; no member of C_A is a candidate, and every candidate lies outside C_A. Loss of the last supporting evidence fails coverage; it removes no row, changes no kind, and never lets Ê_A stand as evidence for itself.

For the present edition, the daggered rows are Yoga, Meta-question, and Schema in its data-shape sense, and the edition's one candidate is Record in the member-bearing-aggregate sense, outside C accordingly.

## Caveats · `sec:kinds:caveats`

**Caveat (Homonymy)** · `cav:kinds:homonymy`

A surface name alone need not determine a kind: for a name in Hom(C_A), the kind token at the label is the author's declaration of the catalogued sense, and the registry validates the pair. This caveat enumerates nothing — the acceptee's register presents Hom(C_A) (`req:kinds:attestation-register`) — and its examples are explanatory, and of three species. Homonyms proper: Structure's senses — the fixed object beside Model, the declared shape beside Schema, and the member-bearing aggregate beside Class — with the paper-roadmap head a Preview or a named division. Ambiguities that are not homonymy: the division-head pattern, where a Discussion or a Motivation heading a division takes the rung's kind by reduction, and no pair is created. And second senses outside C entirely, no environment at all — the mathematical graph, the periodical journal, the legal article, the statute, the lexicographic lemma, the cryptographic signature, the runtime metric value, the version value, the contents table, and the word registry itself, which names distinct artifacts across a corpus's disciplines, the sense fixed by each document's preamble. Homonymy is distinct from synonymy, and the two are independent facts: of the Construction–Model–Structure genre, one kind throughout in its fixed-object sense, Construction rows nowhere in Hom while Model and Structure both do, each carrying a second catalogued sense besides; and several names under one kind — `heur`, `assum`, `por`, `fact`, `pred` — row nowhere in Hom at all.

**Caveat (Limits of attestation)** · `cav:kinds:attestation-limits`

An attestation judgment records that an authority accepts a located occurrence as evidence for one name-and-kind pair. It establishes no frequency, merit, recommendation, synonymy, distinctness, or exhaustiveness — those are empirical, editorial, or matters of the authored assignment. The register is evidence presentation, not classification authority: repetition strengthens no assignment, and a candidate licenses no use of its pair.

## Rejected Ansätze · `sec:kinds:rejected-ansaetze`

**Ansatz (One kind per name)** · `ansatz:kinds:kind-per-name`

Give every surface name its own token. Then Proof sketch and Sketch of proof carry different kinds, references fragment by spelling, and a renamed heading orphans its readers — against (`inv:kinds:one-kind`). Rejected.

**Ansatz (Kinds from families)** · `ansatz:kinds:family-kinds`

Issue one token per family. Then a theorem, a lemma, and a conjecture all carry the same kind, a reference can no longer say what it refers to, and (`inv:kinds:distinctness`) holds only vacuously. Rejected.

**Ansatz (Numbering as kinds)** · `ansatz:kinds:numbering`

Encode letters and numbers in the token — a kind for Theorem A, a kind for Theorem 1.1 restated. Then presentation enters denotation and every restatement names anew, against the devices this registry declares presentation (`def:kinds:presentation-reduction`). Rejected.

**Ansatz (Functional names)** · `ansatz:kinds:functional-names`

Require every name to carry exactly one kind, and legislate local usage into line — Protocol rewritten as Minutes, Illustration as Figure, wherever the foreign sense is meant. Then (`judg:kinds:classification`) validates spellings instead of pairs, the catalogue stops describing what authors attestedly write, and genuinely ambiguous language is outlawed rather than catalogued — while the label's kind token was already the author's disambiguation, and (`inv:kinds:catalogued-pairs`) already checks it against the effective relation (`def:kinds:homonymy`). Rejected.

## Adoption gate · `sec:kinds:gate`

**Gate (Adoption)** · `gate:kinds:adoption`

Adoption is blocked until all of the following hold:

- every participating authored head validates by exactly one exact pair or one reduction (`judg:kinds:head-validation`) and is the subject of exactly one such judgment (`inv:kinds:totality`), with overriding rows taking precedence (`def:kinds:presentation-reduction`);
- kind tokens are pairwise distinct, one per concept (`inv:kinds:distinctness`), and synonym classes convergent (`inv:kinds:one-kind`);
- every base pair in use lies in C and every local pair in the recorded X_A (`inv:kinds:catalogued-pairs`);
- the hybrid rows are exactly the declared instances of (`inf:kinds:hybrid`), tokens collision-free;
- homonymy is derived by (`def:kinds:homonymy`) and declared nowhere, and this document materializes no homonym index;
- the adoption decision names exactly one acceptee, owning the extensions, evidence, statuses, generator, and register (`sig:kinds:acceptee`);
- every pair of C_A has exactly one firm or borderline status, no weaker than its edition floor; daggers match the edition's evidence exactly; every candidate lies outside C_A (`inv:kinds:attestation-coverage`);
- the companion register is the current, deterministic, totally ordered output of the acceptee's evidence and statuses, presents exactly Hom(C_A) as a view of the same base, and is maintained only by regeneration (`req:kinds:attestation-register`);
- the headline counts are derived from the tables by the register generator, presented only in (`tab:kinds:headline-counts`), never hand-maintained, and are this document's only generated region;
- this document's own heads and heading anchors are classified by the registry they define, with exactly one label per head, no environment numbering, and every parenthesized reference resolving;
- the gate is dischargeable from this document and the corpus's adoption data alone; no check consults another document.
