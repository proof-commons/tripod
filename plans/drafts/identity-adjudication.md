# An Adjudication Procedure for Identities, Digests, and Evidence

This document lays down, self-containedly, a discipline for digests and identities in a corpus of validated typed objects, generated artifacts, evidence reports, deployment profiles, and releases. It is organized around one question — _is there a benefit from hashing this?_ — because a digest is justified only by a decision it makes possible or cheaper. One Formulation states the question, one Procedure walks every proposal through it, and a Case analysis receives the outcome, including the documented stop in which the right answer is no digest at all. The discipline prescribes no hash construction anywhere: it specifies the properties an identity must deliver, each for the benefit it provides, and leaves every scheme to the recipe record. It is generic: nothing here names a particular repository, tool, or algorithm. This document is self-contained and cites only itself; where an adopting corpus uses it beside other disciplines, their alignment is fixed by the corpus's recorded adoption decisions, not by this text. Acceptance of this document presupposes acceptance of no other document: where another discipline's artifact is consumed — a labeling convention, a set of reserved kinds, a format's declared environment classes — this discipline consumes the artifact as adoption data and asks nothing of its provenance.

The document practices the labeling discipline it assumes: it is a source in the corpus it governs. The label at each heading or environment head is that environment's mint; a parenthesized label in running text is a same-owner citation; material in fenced blocks and double-backtick spans is displayed without participating. The document title is publication metadata, not an environment head; it mints nothing and participates in nothing. Every label here has area `identity`, each environment's kind names its genre, and environments carry no numbers: the mint at each head is the sole name of its environment, and every internal reference is a citation. A recipe _warrants_ a property in the ordinary sense of vouching for it; the word bears no relation to any labeling discipline's warrants.

## The question · `sec:identity:question`

**Formulation (The benefit question)** · `formul:identity:benefit-question`

Given an object or artifact a contributor proposes to hash: is there a benefit from hashing it — which decision becomes possible or cheaper, and for which consumer — and only then, which class of identity should carry it? This document fixes the procedure that answers both questions for every typed object, generated artifact, evidence report, deployment profile, and release of an adopting corpus. Matching hashes establish none of authenticity, correctness, independence, or deployment readiness (`warn:identity:non-claims`).

**Model (Objects and boundaries)** · `model:identity:objects`

The corpus holds authoritative typed objects; artifacts rendered from them; evidence reports about them; profiles aggregating requirements; and releases aggregating everything. As a running illustration, one adopting corpus's identities form a graph of immediate dependencies — the names below are that corpus's, not this document's — of which one path runs:

```text
ModelId → PlanId → BuildId → BundleId → InterfaceId → ProfileId → ReleaseId
```

An arrow points from dependency to dependent: the right side binds the left as an immediate identity dependency (`rule:identity:immediate-edges`), with fan-in where an object aggregates several — a release binds its profile, its required evidence reports, and its distributed artifacts alike. The boundaries that matter to identity are package, process, cache, publication, distribution, deployment, and signature. An identity earns its place only at such a boundary; inside one owner, the typed value itself is the comparison.

**Definition (Digest; recipe)** · `def:identity:recipe`

A digest is the output of a fixed function over a fixed presentation of a value, and it evidences equality with respect to one recipe, under that recipe's collision assumptions — nothing else. Exact equality belongs to direct typed comparison and exact byte comparison; a digest buys their effect across a boundary at the price of a collision assumption, and the assurance class states whether that price is acceptable. A recipe names its projection (which content enters), its canonicalization (how that content is presented), its primitive (the digest function), its domain separator (which role it serves), and its recipe identifier (under which consumers recompute and migrations occur). A corpus's chosen constructions are named in recipe records and prescribed nowhere else.

**Table (Identity properties and their benefits)** · `tab:identity:properties`

The following are the properties an identity can be required to deliver, each demanded not for its own sake but for the benefit it provides:

| Property | Benefit it provides | Failure without it |
| --- | --- | --- |
| Deterministic over meaning | consumers compare identities instead of re-deriving objects; cache and reuse become mechanical | equal objects hash apart; the digest decides nothing |
| Complete over semantic content | a change of meaning always changes the recipe input, and matching digests then fail except with the recipe's residual collision probability; staleness is detectable | silent semantic drift under a stable identity |
| Free of incidental content | re-serialization, reordering, and rebuilds change nothing; presentation invariance holds | false staleness; consumers learn to ignore the identity |
| Domain-separated by role | an identity cannot be replayed as a claim of another kind; evidence roles stay distinct | a digest quoted in one role masquerades in another |
| Recipe-identified | consumers know exactly how to recompute; recipes change only by explicit migration | ambiguous verification; silent redefinition |
| Collision- and second-preimage-resistant | a matched identity computationally pins the object the decision was about | substitution and cache poisoning under a matching digest |
| Recomputable by any consumer | verification without trusting the producer; recomputation is the check | the digest is decoration over a producer's claim |

Any construction delivering the required properties qualifies; which one a corpus chose is a fact of the recipe record (`red:identity:scheme-to-properties`).

**Table (Assurance classes and required properties)** · `tab:identity:class-properties`

Each admitted identity belongs to one assurance class, and each class requires a stated subset of the properties of (`tab:identity:properties`): a cell reads _required_, optionally with the scope or mechanism over which the property is required; _not applicable_; or a condition, stated in the cell:

| Property | Semantic | Artifact | Provenance | Evidence | Release |
| --- | --- | --- | --- | --- | --- |
| Deterministic | required | required, over exact bytes | required | required | required |
| Complete | over the semantic projection | over the exact bytes | over the named revision, tree, or input set | over the full typed report subject | over the entire canonical manifest |
| Free of incidentals | required | not applicable beyond path and role metadata | recipe-defined | required | required |
| Domain-separated | required | bound through the manifest role | required | required — load-bearing | required — load-bearing |
| Recipe-identified | required | required — an algorithm alone underspecifies projection, framing, and normalization | required | required | required |
| Collision-resistant | required | required | required where release-bound; per threat model otherwise | required | required — carries the signing benefit |
| Recomputable | required | required | required where release-bound | required | required |

A provenance identity that is only a locator — an internal name for a revision or input set, never release-bound — carries the weaker conditional row; release reachability forces the stronger.

**Table (Assurance mechanisms)** · `tab:identity:mechanisms`

Distinct mechanisms establish distinct things, and none substitutes for another:

| Mechanism | Establishes | Does not establish |
| --- | --- | --- |
| Type | Representable shape | Cross-field validity |
| Validator | Declared constraints and invariants | Authenticity or implementation correctness |
| Test, proof, or execution | Evidence for a scoped claim | Identity or universal correctness |
| Semantic identity | Computational equality of a canonical typed projection | Validity or authenticity by itself |
| Artifact digest | Computational equality of exact bytes | Meaning or semantic correctness |
| Provenance identity | A named source revision, tree, or input set | Correctness of that source |
| Evidence-report identity | One typed report applies to named subjects | Honesty or implementation independence |
| Signature | A named authority approved an identity | Correctness of the signed object |
| Reproducible build | Independent builds produced equal bytes | An uncompromised toolchain |

A hash never replaces the owning type, the validator, or the evidence requirement.

**Example (Illustrative inventory)** · `ex:identity:inventory`

For illustration only, a typical corpus might already hold the following. The Class column carries only the five assurance classes of (`sec:identity:cases`); an em dash marks an entry that has none, because no digest exists or because none has yet been admitted (`req:identity:admission-record`):

| Identity | Class | Standing |
| --- | --- | --- |
| Revision and tree identifiers | provenance | retained; never a protocol identity |
| Publication metadata identifiers | provenance | retained; publication-only, contained by (`rule:identity:provenance-containment`) |
| An upstream anchor-set hash | semantic | retained |
| A model semantic hash | semantic | retained |
| A behavioural hash gating major versions | semantic | retained; not propagated as a runtime identity |
| Generated-file exact comparisons | — | freshness by exact bytes under (`case:identity:artifact`); no digest |
| A profile hash awaiting its consumer | — | pre-admission; no assurance until a named consumer decides from it |
| Raw hash fields in a profile schema | — | pre-admission; must gain owned recipes and typed references before release |

## The procedure · `sec:identity:procedure`

**Procedure (Adjudication)** · `alg:identity:adjudication`

Every proposal walks one tree:

```text
proposed digest
    ↓
typed and validated?                  ── no → validate before any
    ↓ yes                                     identity is admitted
which decision would equality change?
    none                              ── → no identity: stop
    ↓ a named consumer's
is that equality already given on the path —
reviewed at this boundary · compared anyhow · parent-assured?
    yes                               ── → no identity: stop
    ↓ no
record the admission facts
    ↓
dispatch on class:
    semantic | artifact | provenance | evidence | release

artifact branch:  canonical renderer → artifact bytes
                  → exact freshness comparison
                  → byte digest only when independently
                    distributed or release-bound

evidence branch:  execution → typed report payload
                  → validated report envelope → report identity
                  → deployment profile or release manifest
```

The benefit nodes are decided by (`crit:identity:benefit`); an accepted walk records (`req:identity:admission-record`) before it dispatches; the two stopping branches terminate in (`case:identity:no-identity`).

**Criterion (Benefit)** · `crit:identity:benefit`

A digest benefits the corpus if and only if a named consumer's decision becomes possible or cheaper through mechanical equality that nothing already on that path provides — where what a path can already provide is fixed by (`tab:identity:mechanisms`). Equality a standing review supersedes is no benefit at the review's own boundary: the review judges content, the digest only equality, and the weaker check cannot add to the stronger. Across a later boundary of (`model:identity:objects`) — cache, publication, distribution, deployment, signature — a digest may still bind what was reviewed to what arrives, and that binding is a distinct benefit the review does not provide. Equality a direct typed comparison already performs is no benefit, and equality the parent identity already carries is no benefit.

**Requirement (Admission record)** · `req:identity:admission-record`

Admission is per identity class — one recipe, one role, one consumer-decision pattern; individual identity values flow through their class's record and are never admitted one by one. An accepted class records: the complete typed object or exact artifact bytes identified; the package owning the recipe; the producer; the present consumer, arriving in the same change set or within a recorded, deadline-bound migration — "the same implementation series" means exactly this; that consumer's exact accept, reject, cache, or reuse decision; the assurance class — semantic, artifact, provenance, evidence, or release, as fixed by (`sec:identity:cases`); the recipe, by identifier (`def:identity:recipe`), warranting every property its assurance class requires (`tab:identity:class-properties`); the exact stale conditions; the migration behavior; and the explicit non-claims. Fields are validated as parts of their owning object and are never independently hashed merely to detect changes.

**Requirement (Stop record)** · `req:identity:stop-record`

A no-identity outcome is recorded: the proposal, the deciding branch of (`alg:identity:adjudication`), the date, and any condition under which the walk is retaken. The absence of a digest is then the corpus's documented state, and the same proposal is not re-adjudicated from nothing.

## Case analysis · `sec:identity:cases`

**Case (Semantic identity)** · `case:identity:semantic`

One meaning with several possible encodings, or consumption across a package, process, cache, or publication boundary. The identity is computed over the canonical projection of the validated object under its recipe (`def:identity:recipe`) and delivers the full semantic column of (`tab:identity:class-properties`). No construction is prescribed; the recipe records the one in use.

**Case (Artifact digest)** · `case:identity:artifact`

Exact bytes, independently distributed or release-bound. The required properties are the artifact column of (`tab:identity:class-properties`), and a release-manifest entry binds artifact role, canonical relative path, schema or media type, recipe identifier, and digest. An artifact digest does not become a semantic identity unless one reviewed canonical byte encoding is explicitly defined as the semantic object. Committed generated publications take the freshness sub-branch: exact expected-byte comparison decides them, and no digest is added — an implementation may realize exact comparison through a transient internal hash, which is an optimization, not an identity, and never persists or publishes.

**Case (Provenance identity)** · `case:identity:provenance`

A named source: a revision, a tree, an exact canonical input set. The provenance column of (`tab:identity:class-properties`) carries it, locator-grade or release-bound as its reach requires. It names material, claims nothing about the material's correctness, and stays separate from semantic and artifact identity.

**Case (Evidence identity)** · `case:identity:evidence`

A validated typed report envelope binding at least report role, report schema, exact subject identities, producer or implementation identity, configuration where relevant, result status, and canonical payload or payload digest. The evidence column of (`tab:identity:class-properties`) applies in full, and domain separation by role is the load-bearing property: it is what keeps one report from being quoted as another kind of claim. A raw digest without role and subject binding is not evidence identity.

**Case (Release identity)** · `case:identity:release`

The canonical release manifest aggregating the deployment profile, required evidence references, distributed artifacts with their byte digests, release policy, and explicit source revision and date. The release column of (`tab:identity:class-properties`) applies in full; domain separation and collision resistance carry the signing benefit. If signing is introduced, the release-manifest identity is the signing root, and internal objects are not signed separately unless they hold an independently defined authority boundary of their own, which then carries its own typed role and verification policy.

**Case (No identity)** · `case:identity:no-identity`

The affirmative stop, as first-class as any admission. No digest is created when direct typed comparison suffices; when the parent identity already provides the assurance; when the value crosses no package, process, cache, publication, distribution, deployment, or signature boundary of (`model:identity:objects`) and has no independent lifecycle of its own; when importance is the only motive, importance being no consumer; when the value is ephemeral local evidence, such as ordinary CI logs nobody consumes as release evidence; or when the purpose never required byte-equal provenance, because every change is reviewed at that boundary and the review judges more than equality. Failing (`crit:identity:benefit`) is a result, not an omission: the walk ends here deliberately, the stop is recorded under (`req:identity:stop-record`), and the absence of a digest is then the corpus's documented state.

## Reductions · `sec:identity:reductions`

**Reduction (Scheme to properties)** · `red:identity:scheme-to-properties`

Every construction question reduces to the property table. A scheme is adequate exactly when it delivers the properties of (`tab:identity:properties`) that its class requires; which adequate scheme a corpus chose is a fact of its recipe record; and the discipline prescribes none. Schemes therefore migrate freely under (`rule:identity:recipe-permanence`) while every benefit the properties buy stands still.

**Reduction (Mesh to chain)** · `red:identity:mesh-to-chain`

All-to-all identity binding reduces to immediate typed edges. Binding every object to every transitive dependency duplicates what the chain already carries and erects a quadratic consistency mesh; binding each object to its immediate dependencies alone (`rule:identity:immediate-edges`) preserves transitive assurance by composition — the composition grounded by (`rule:identity:well-founded-graph`) — and gives each edge one owner. Human-readable manifests may display a whole chain; authoritative validation follows the edges.

**Reduction (Field hashes to object validation)** · `red:identity:fields-to-object`

Hashing a field reduces to validating its object. Fields have no independent lifecycle, cross-field validity is a property of the whole, and a field digest evidences nothing the owning validator does not already establish (`tab:identity:mechanisms`).

## Rules of the procedure · `sec:identity:rules`

**Rule (Validation before admission)** · `rule:identity:admission-order`

No semantic, evidence, or release identity — a deployment profile's identity falling under whichever of these its class record names — is admitted or published before the owning validator has run. Bytes may be hashed earlier — for transport, lookup, streaming, or content addressing — but such a value has no assurance standing until the applicable validation succeeds (`tab:identity:mechanisms`): a self-consistent invalid object rehashes perfectly, and a digest downstream of no validation binds garbage exactly.

**Rule (No incidental content)** · `rule:identity:no-incidentals`

No graph-library index, source order, path, line number, solver variable number, matrix position, traversal order, thread schedule, temporary path, or floating-point working value enters a semantic identity. This enforces the free-of-incidentals property of (`tab:identity:properties`) at the source: processing accidents must not masquerade as meaning.

**Rule (Recipe permanence)** · `rule:identity:recipe-permanence`

A published recipe (`def:identity:recipe`) is never silently redefined. Changing its projection — the fields it includes and the rules by which it excludes — its canonicalization, its primitive, or its domain separator creates a new recipe identifier. A migration records old and new recipes, the reason, whether meaning changed or only measurement, old and new identities where applicable, and the consumer transition policy. A recipe migration does not itself decide semantic versioning; the owning versioning rule does.

**Rule (Producer and consumer duties)** · `rule:identity:duties`

The owning producer validates the complete typed object (`rule:identity:admission-order`), derives its canonical projection, computes its identity, and publishes object and recipe identifier together wherever external consumption exists. An immediate consumer parses external bytes into a typed value where necessary, rejects unknown fields and unsupported schemas, runs the owner's validator, recomputes the identity, compares the required immediate dependency identity (`rule:identity:immediate-edges`), and only then consumes the typed value.

**Rule (Immediate edges)** · `rule:identity:immediate-edges`

An independently consumed parent binds only its immediate identity dependencies, and a child receives an identity of its own only when it has an independent lifecycle; otherwise the parent includes the canonical typed child value directly.

**Rule (Well-founded graph)** · `rule:identity:well-founded-graph`

The authoritative identity-dependency graph is finite and acyclic: every release-reachable identity terminates, along the immediate edges of (`rule:identity:immediate-edges`), in directly validated typed values or exact artifact bytes. A cycle at the model level is broken in the identity projection before any identity on it is admitted.

**Rule (Delegated release validation)** · `rule:identity:delegation`

The release validator traverses the typed identity graph (`rule:identity:well-founded-graph`), delegates to package-owned validators (`rule:identity:duties`), verifies immediate edges (`rule:identity:immediate-edges`), required evidence roles, and artifact bytes, and reimplements nothing.

**Rule (Provenance containment)** · `rule:identity:provenance-containment`

Publication provenance identifiers never enter semantic, interface, or protocol identity. Before production release, deployment calibration binds the exact final bundle and interface, and evidence fields bind typed report roles and subjects (`case:identity:evidence`) rather than bare digest arrays.

## Myths · `sec:identity:myths`

**Myth (Hashes validate)** · `myth:identity:hashes-validate`

Corrected: admission awaits validation, always (`rule:identity:admission-order`). A hash of an invalid object is a fast way to remember the mistake.

**Myth (A digest authenticates)** · `myth:identity:digest-authenticates`

Corrected: an unkeyed digest evidences equality under one recipe, never authenticity. Authenticity requires an independently trusted expected identity, or a signature over the accepted release root (`case:identity:release`) — nothing smaller, except an object holding an independently defined authority boundary of its own.

**Myth (Inequality is independence)** · `myth:identity:inequality-independence`

Corrected: different bytes do not prove independent implementation or judgment. Independence is a reviewed provenance claim recording implementation identity, shared code and dependencies, operator, and execution environment where relevant. Equal report hashes are not rejected to manufacture an appearance of independence; distinct report roles are distinguished by typed envelopes and domain separation (`case:identity:evidence`).

**Myth (Important values deserve hashes)** · `myth:identity:importance`

Corrected: importance is not a consumer (`crit:identity:benefit`). A value's weight argues for validation and review — the mechanisms that judge content — not for another digest.

**Myth (Reviewed objects still need digests)** · `myth:identity:reviewed-anyhow`

Corrected: review judges content, a digest only equality. Under a standing review, at the review's own decision point, the digest adds a maintenance surface, not assurance, and fails (`crit:identity:benefit`) on its own terms. Binding the reviewed object across a later boundary of (`model:identity:objects`) is a different proposal, walked separately.

**Myth (The scheme is the security)** · `myth:identity:scheme-worship`

Corrected: benefits flow from the properties a recipe warrants, not from any particular concatenation (`red:identity:scheme-to-properties`). A familiar-looking scheme without the properties is false comfort; an unfamiliar one with them is sound.

**Myth (One hash can rule them all)** · `myth:identity:one-hash`

Corrected: semantic meaning, artifact bytes, provenance, evidence, and release aggregation have different stale conditions and different consumers (`sec:identity:cases`); one undifferentiated hash serves none of them.

**Myth (More digests, more assurance)** · `myth:identity:more-is-safer`

Corrected: each digest is a standing obligation — a recipe, stale conditions, a migration path, a consumer (`req:identity:admission-record`). Proliferation multiplies obligations while assurance stays where it always was: with types, validators, and evidence.

**Warning (Non-claims)** · `warn:identity:non-claims`

A matching identity evidences equality under one recipe (`def:identity:recipe`), computationally and nothing else: no logical identity, no authenticity, no correctness, no independence, no deployment readiness. Every admission record states its own non-claims.

## Moral · `sec:identity:moral`

**Moral (Types validate; hashes bind)** · `moral:identity:types-validate-hashes-bind`

One aggregate identity per independently meaningful object; one byte digest per independently distributed artifact; one root for future authentication; typed evidence references instead of ambiguous raw hashes; immediate rather than all-to-all binding (`rule:identity:immediate-edges`); no field-level proliferation. Types and validators remain the correctness mechanism; hashes remain the comparison and binding mechanism; and no digest without a decision it changes (`crit:identity:benefit`) — the documented stop (`case:identity:no-identity`) being as sound an outcome as any admission.

## Implementation gate · `sec:identity:gate`

**Gate (Implementation)** · `gate:identity:implementation`

Implementation is blocked until all of the following hold:

- every existing digest is classified by object, owner, producer, consumer, decision, assurance class, stale condition, migration, and non-claims;
- no new digest enters without passing (`crit:identity:benefit`) and recording its class under (`req:identity:admission-record`);
- every no-identity outcome is recorded with its deciding branch and revisit condition (`req:identity:stop-record`);
- every admitted recipe demonstrably delivers the properties its assurance class requires (`tab:identity:class-properties`), and no scheme is prescribed outside a recipe record;
- semantic, artifact, provenance, evidence, and release identities use distinct typed roles;
- committed generated publications keep exact freshness comparisons and acquire no redundant digests (`case:identity:artifact`);
- identity graphs bind immediate dependencies only (`rule:identity:immediate-edges`), and the graph is acyclic, every release-reachable identity terminating in validated values or exact bytes (`rule:identity:well-founded-graph`);
- local handles and other incidental content remain absent from semantic identity (`rule:identity:no-incidentals`);
- evidence reports bind typed roles and exact subjects (`case:identity:evidence`);
- deployment calibration binds the final bundle and interface before production (`rule:identity:provenance-containment`);
- producers and consumers discharge their duties (`rule:identity:duties`), and release validation delegates to package-owned validators (`rule:identity:delegation`);
- any future signature authenticates the release-manifest identity, or the identity of an object holding a declared independent authority boundary (`case:identity:release`);
- the gate is dischargeable from this document and the corpus's adoption data alone; no check consults another document;
- the corpus's full checks pass and leave no uncommitted generated changes.
