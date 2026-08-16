# An Adjudication Procedure for Identities, Digests, and Evidence

This document lays down, self-containedly, a discipline for digests and
identities in a corpus of validated typed objects, generated artifacts,
evidence reports, deployment profiles, and releases. It is organized
around one question — *is there a benefit from hashing this?* — because
a digest is justified only by a decision it makes possible or cheaper.
One Formulation states the question, one Procedure walks every proposal
through it, and a Case analysis receives the outcome, including the
documented stop in which the right answer is no digest at all. The
discipline prescribes no hash construction anywhere: it specifies the
properties an identity must deliver, each for the benefit it provides,
and leaves every scheme to the recipe record. It is generic: nothing
here names a particular repository, tool, or algorithm.

The document practices the labeling discipline it assumes: it is a
source in the corpus it governs. The label at each heading or
environment head is that environment's mint; a parenthesized label in
running text is a same-owner citation; material in fenced blocks is
displayed without participating. Every label here has area `identity`,
each environment's kind names its genre, and environments carry no
numbers: the mint at each head is the sole name of its environment, and
every internal reference is a citation.

## The question · `sec:identity:question`

**Formulation (The benefit question)** ·
`formul:identity:benefit-question`
Given an object or artifact a contributor proposes to hash: is there a
benefit from hashing it — which decision becomes possible or cheaper,
and for which consumer — and only then, which class of identity should
carry it? This document fixes the procedure that answers both
questions for every typed object, generated artifact, evidence report,
deployment profile, and release of an adopting corpus. Matching hashes
establish none of authenticity, correctness, independence, or
deployment readiness (`warn:identity:non-claims`).

**Model (Objects and boundaries)** · `model:identity:objects`
The corpus holds authoritative typed objects; artifacts rendered from
them; evidence reports about them; profiles aggregating requirements;
and releases aggregating everything. As a running illustration, its
identities form a chain of immediate dependencies:

```text
ModelId → PlanId → BuildId → BundleId → InterfaceId → ProfileId → ReleaseId
```

The boundaries that matter to identity are package, process, cache,
publication, distribution, and signature. An identity earns its place
only at such a boundary; inside one owner, the typed value itself is
the comparison.

**Definition (Digest; recipe)** · `def:identity:recipe`
A digest is the output of a fixed function over a fixed presentation
of a value, and it establishes equality with respect to one recipe —
nothing else. A recipe names its projection (which content enters),
its canonicalization (how that content is presented), its primitive
(the digest function), its domain separator (which role it serves),
and its recipe identifier (under which consumers recompute and
migrations occur). A corpus's chosen constructions are named in recipe
records and prescribed nowhere else.

**Table (Identity properties and their benefits)** ·
`tab:identity:properties`
An admitted identity delivers the following properties, each demanded
not for its own sake but for the benefit it provides:

| Property | Benefit it provides | Failure without it |
|---|---|---|
| Deterministic over meaning | consumers compare identities instead of re-deriving objects; cache and reuse become mechanical | equal objects hash apart; the digest decides nothing |
| Complete over semantic content | a change of meaning always changes the identity; staleness is detectable | silent semantic drift under a stable identity |
| Free of incidental content | re-serialization, reordering, and rebuilds change nothing; presentation invariance holds | false staleness; consumers learn to ignore the identity |
| Domain-separated by role | an identity cannot be replayed as a claim of another kind; evidence roles stay distinct | a digest quoted in one role masquerades in another |
| Recipe-identified | consumers know exactly how to recompute; recipes change only by explicit migration | ambiguous verification; silent redefinition |
| Collision- and second-preimage-resistant | a matched identity actually pins the object the decision was about | substitution and cache poisoning under a matching digest |
| Recomputable by any consumer | verification without trusting the producer; recomputation is the check | the digest is decoration over a producer's claim |

Any construction delivering the required properties qualifies; which
one a corpus chose is a fact of the recipe record
(`red:identity:scheme-to-properties`).

**Table (Assurance mechanisms)** · `tab:identity:mechanisms`
Distinct mechanisms establish distinct things, and none substitutes
for another:

| Mechanism | Establishes | Does not establish |
|---|---|---|
| Type | Representable shape | Cross-field validity |
| Validator | Declared constraints and invariants | Authenticity or implementation correctness |
| Test, proof, or execution | Evidence for a scoped claim | Identity or universal correctness |
| Semantic identity | Equality of a canonical typed projection | Validity or authenticity by itself |
| Artifact digest | Equality of exact bytes | Meaning or semantic correctness |
| Provenance identity | A named source revision, tree, or input set | Correctness of that source |
| Evidence-report identity | One typed report applies to named subjects | Honesty or implementation independence |
| Signature | A named authority approved an identity | Correctness of the signed object |
| Reproducible build | Independent builds produced equal bytes | An uncompromised toolchain |

A hash never replaces the owning type, the validator, or the evidence
requirement.

**Data (Illustrative inventory)** · `data:identity:inventory`
For illustration only, a typical corpus might already hold:

| Identity | Class | Standing |
|---|---|---|
| Revision and tree identifiers | provenance | retained; never a protocol identity |
| Publication metadata identifiers | provenance | retained; publication-only, contained by (`rule:identity:containment`) |
| An upstream anchor-set hash | semantic | retained |
| A model semantic hash | semantic | retained |
| A behavioural hash gating major versions | semantic | retained; not propagated as a runtime identity |
| Generated-file exact comparisons | freshness, no digest | retained under (`case:identity:artifact`); no redundant hash |
| A profile hash awaiting its consumer | pre-admission | no assurance until a named consumer decides from it |
| Raw hash fields in a profile schema | pre-admission | must gain owned recipes and typed references before release |

## The procedure · `sec:identity:procedure`

**Procedure (Adjudication)** · `alg:identity:adjudication`
Every proposal walks one tree:

```text
proposed digest
    ↓
typed and validated?                  ── no → validate; never hash
    ↓ yes
which decision would equality change?
    none                              ── → no identity: stop
    ↓ a named consumer's
is that equality already given on the path —
reviewed anyhow · compared anyhow · parent-assured?
    yes                               ── → no identity: stop
    ↓ no
record the admission facts
    ↓
dispatch on boundary:
    semantic | bytes | provenance | evidence | release

bytes branch:     canonical renderer → artifact bytes
                  → exact freshness comparison
                  → byte digest only when independently
                    distributed or release-bound

evidence branch:  execution → typed report payload
                  → validated report envelope → report identity
                  → deployment profile or release manifest
```

The benefit nodes are decided by (`crit:identity:benefit`); the two
stopping branches terminate in (`case:identity:none`); an accepted
walk records (`cond:identity:admission-record`) before it dispatches.

**Criterion (Benefit)** · `crit:identity:benefit`
A digest benefits the corpus if and only if a named consumer's
decision becomes possible or cheaper through mechanical equality that
nothing already on that path provides — where what a path can already
provide is fixed by (`tab:identity:mechanisms`). Equality a standing
review supersedes is no benefit: the review judges content, the digest
only equality, and the weaker check cannot add to the stronger.
Equality a direct typed comparison already performs is no benefit, and
equality the parent identity already carries is no benefit.

**Condition (Admission record)** · `cond:identity:admission-record`
An accepted digest records: the complete typed object or exact
artifact bytes identified; the package owning the recipe; the
producer; the present consumer, arriving no later than the same
implementation series; that consumer's exact accept, reject, cache, or
reuse decision; the assurance class — semantic equality, byte
integrity, provenance, evidence binding, or release authentication;
the recipe, by identifier, warranting every property of
(`tab:identity:properties`); the exact stale conditions; the migration
behavior; and the explicit non-claims. Fields are validated as parts
of their owning object and are never independently hashed merely to
detect changes.

## Case analysis · `sec:identity:cases`

**Case (Semantic identity)** · `case:identity:semantic`
One meaning with several possible encodings, or consumption across a
package, process, cache, or publication boundary. The identity is
computed over the canonical projection of the validated object and
delivers the full property table (`tab:identity:properties`). No
construction is prescribed; the recipe records the one in use.

**Case (Artifact digest)** · `case:identity:artifact`
Exact bytes, independently distributed or release-bound. The property
set restricts to bytes — determinism over the bytes, collision
resistance, recomputability — and a release-manifest entry binds
artifact role, canonical relative path, schema or media type,
algorithm, and digest. An artifact digest does not become a semantic
identity unless one reviewed canonical byte encoding is explicitly
defined as the semantic object. Committed generated publications take
the freshness sub-branch: exact expected-byte comparison decides them,
and no digest is added.

**Case (Provenance identity)** · `case:identity:provenance`
A named source: a revision, a tree, an exact canonical input set.
Determinism and recipe identification carry it. It names material,
claims nothing about the material's correctness, and stays separate
from semantic and artifact identity.

**Case (Evidence identity)** · `case:identity:evidence`
A validated typed report envelope binding at least report role, report
schema, exact subject identities, producer or implementation identity,
configuration where relevant, result status, and canonical payload or
payload digest. Domain separation by role is the load-bearing
property: it is what keeps one report from being quoted as another
kind of claim. A raw digest without role and subject binding is not
evidence identity.

**Case (Release identity)** · `case:identity:release`
The canonical release manifest aggregating the deployment profile,
required evidence references, distributed artifacts with their byte
digests, release policy, and explicit source revision and date. Domain
separation and collision resistance carry the signing benefit: if
signing is introduced, the release-manifest identity is the signing
root, and internal objects are not signed separately unless they hold
an independent operational authority boundary.

**Case (No identity)** · `case:identity:none`
The affirmative stop, as first-class as any admission. No digest is
created when direct typed comparison suffices; when the parent
identity already provides the assurance; when the value has no
independent storage, transport, signature, cache, reuse, disclosure,
or versioning boundary; when importance is the only motive, importance
being no consumer; when the value is ephemeral local evidence, such as
ordinary CI logs nobody consumes as release evidence; or when the
purpose never required byte-equal provenance, because every change is
reviewed anyhow and the review judges more than equality. Failing
(`crit:identity:benefit`) is a result, not an omission: the walk ends
here deliberately, and the absence of a digest is then the corpus's
documented state.

## Reductions · `sec:identity:reductions`

**Reduction (Scheme to properties)** ·
`red:identity:scheme-to-properties`
Every construction question reduces to the property table. A scheme is
adequate exactly when it delivers the properties of
(`tab:identity:properties`); which adequate scheme a corpus chose is a
fact of its recipe record; and the discipline prescribes none. Schemes
therefore migrate freely under (`rule:identity:recipe-permanence`)
while every benefit the properties buy stands still.

**Reduction (Mesh to chain)** · `red:identity:mesh-to-chain`
All-to-all identity binding reduces to immediate typed edges. Binding
every object to every transitive dependency duplicates what the chain
already carries and erects a quadratic consistency mesh; binding each
object to its immediate dependencies alone
(`rule:identity:immediate-edges`) preserves transitive assurance by
composition and gives each edge one owner. Human-readable manifests
may display a whole chain; authoritative validation follows the edges.

**Reduction (Field hashes to object validation)** ·
`red:identity:fields-to-object`
Hashing a field reduces to validating its object. Fields have no
independent lifecycle, cross-field validity is a property of the
whole, and a field digest evidences nothing the owning validator does
not already establish (`tab:identity:mechanisms`).

## Rules of the procedure · `sec:identity:rules`

**Rule (Validation before hashing)** · `rule:identity:validation-order`
The owning validator runs before any identity is computed. A
self-consistent invalid object rehashes perfectly; a digest downstream
of no validation binds garbage exactly.

**Rule (No incidental content)** · `rule:identity:no-incidentals`
No graph-library index, source order, path, line number, solver
variable number, matrix position, traversal order, thread schedule,
temporary path, or floating-point working value enters a semantic
identity. This enforces the free-of-incidentals property of
(`tab:identity:properties`) at the source: processing accidents must
not masquerade as meaning.

**Rule (Recipe permanence)** · `rule:identity:recipe-permanence`
A published recipe is never silently redefined. Changing its
projection, canonicalization, primitive, domain separator, included
fields, or exclusion rules creates a new recipe identifier. A
migration records old and new recipes, the reason, whether meaning
changed or only measurement, old and new identities where applicable,
and the consumer transition policy. A recipe migration does not itself
decide semantic versioning; the owning versioning rule does.

**Rule (Producer and consumer duties)** · `rule:identity:duties`
The owning producer validates the complete typed object, derives its
canonical projection, computes its identity, and publishes object and
recipe identifier together wherever external consumption exists. An
immediate consumer parses external bytes into a typed value where
necessary, rejects unknown fields and unsupported schemas, runs the
owner's validator, recomputes the identity, compares the required
immediate dependency identity, and only then consumes the typed value.

**Rule (Immediate edges)** · `rule:identity:immediate-edges`
An independently consumed parent binds only its immediate identity
dependencies, and a child receives an identity of its own only when it
has an independent lifecycle; otherwise the parent includes the
canonical typed child value directly.

**Rule (Delegated release validation)** · `rule:identity:delegation`
The release validator traverses the typed identity graph, delegates to
package-owned validators, verifies immediate edges, required evidence
roles, and artifact bytes, and reimplements nothing.

**Rule (Provenance containment)** · `rule:identity:containment`
Publication provenance identifiers never enter semantic, interface, or
protocol identity. Before production release, deployment calibration
binds the exact final bundle and interface, and evidence fields bind
typed report roles and subjects rather than bare digest arrays.

## Myths · `sec:identity:myths`

**Myth (Hashes validate)** · `myth:identity:hashes-validate`
Corrected: validation precedes hashing, always
(`rule:identity:validation-order`). A hash of an invalid object is a
fast way to remember the mistake.

**Myth (A digest authenticates)** · `myth:identity:digest-authenticates`
Corrected: an unkeyed digest proves equality under one recipe, never
authenticity. Authenticity requires an independently trusted expected
identity, or a signature over the accepted release root
(`case:identity:release`) — and nothing smaller.

**Myth (Inequality is independence)** ·
`myth:identity:inequality-independence`
Corrected: different bytes do not prove independent implementation or
judgment. Independence is a reviewed provenance claim recording
implementation identity, shared code and dependencies, operator, and
execution environment where relevant. Equal report hashes are not
rejected to manufacture an appearance of independence; distinct
report roles are distinguished by typed envelopes and domain
separation (`case:identity:evidence`).

**Myth (Important values deserve hashes)** · `myth:identity:importance`
Corrected: importance is not a consumer. A value's weight argues for
validation and review — the mechanisms that judge content — not for
another digest.

**Myth (Reviewed objects still need digests)** ·
`myth:identity:reviewed-anyhow`
Corrected: review judges content, a digest only equality. Under a
standing review the digest adds a maintenance surface, not assurance,
and fails (`crit:identity:benefit`) on its own terms.

**Myth (The scheme is the security)** · `myth:identity:scheme-worship`
Corrected: benefits flow from the properties a recipe warrants, not
from any particular concatenation
(`red:identity:scheme-to-properties`). A familiar-looking scheme
without the properties is false comfort; an unfamiliar one with them
is sound.

**Myth (One hash can rule them all)** · `myth:identity:one-hash`
Corrected: semantic meaning, artifact bytes, provenance, evidence, and
release aggregation have different stale conditions and different
consumers (`sec:identity:cases`); one undifferentiated hash serves
none of them.

**Myth (More digests, more assurance)** · `myth:identity:more-is-safer`
Corrected: each digest is a standing obligation — a recipe, stale
conditions, a migration path, a consumer. Proliferation multiplies
obligations while assurance stays where it always was: with types,
validators, and evidence.

**Warning (Non-claims)** · `warn:identity:non-claims`
A matching identity establishes equality under one recipe
(`def:identity:recipe`) and nothing else: no authenticity, no
correctness, no independence, no deployment readiness. Every
admission record states its own non-claims.

## Moral · `sec:identity:moral`

**Moral (Types validate; hashes bind)** · `moral:identity:types-validate`
One aggregate identity per independently meaningful object; one byte
digest per independently distributed artifact; one root for future
authentication; typed evidence references instead of ambiguous raw
hashes; immediate rather than all-to-all binding; no field-level
proliferation. Types and validators remain the correctness mechanism;
hashes remain the comparison and binding mechanism; and no digest
without a decision it changes.

## Gate · `sec:identity:gate`

**Gate (Implementation)** · `gate:identity:implementation`
The discipline is implemented when:

- every existing digest is classified by object, owner, producer,
  consumer, decision, assurance class, stale condition, migration, and
  non-claims;
- no new digest enters without passing (`crit:identity:benefit`) and
  recording (`cond:identity:admission-record`);
- every admitted recipe demonstrably delivers the properties of
  (`tab:identity:properties`), and no scheme is prescribed outside a
  recipe record;
- semantic, artifact, provenance, evidence, and release identities use
  distinct typed roles;
- committed generated publications keep exact freshness comparisons
  and acquire no redundant digests;
- identity graphs bind immediate dependencies only;
- local handles and other incidental content remain absent from
  semantic identity;
- evidence reports bind typed roles and exact subjects;
- deployment calibration binds the final bundle and interface before
  production;
- release validation delegates to package-owned validators;
- any future signature authenticates the release-manifest identity;
- the corpus's full checks remain green and clean.
