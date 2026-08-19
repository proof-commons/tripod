# ADR-021: Adoption of the Identity Adjudication Procedure

**Status:** Decided and implemented for current identity policy; the
evidence-envelope, release-manifest, and release-validator surfaces the
draft describes stay unbuilt and activate with their named consumers
**Scope:** Every first-party semantic identity, provenance identity,
generated and release artifact, evidence report, and deployment profile
of this repository
**Adopts:** the archived draft at
[plans/drafts/identity-adjudication.md](../plans/drafts/identity-adjudication.md)
as the normative statement of the identity discipline
**Amends:** the identity interpretation of reproducibility and generated
artifacts under (`[ADR011-rule:toolchain:reproducibility]`) and
(`[ADR011-rule:toolchain:generated]`)
**Retires:** ADR-016, deleted from the tree; its text remains in Git
history
**Does not establish:** authenticity, correctness, independence, or
deployment readiness merely from matching hashes

---

## Context · `sec:identity:context`

The repository has adjudicated digests under ADR-016 since the identity
policy was first written. An externally authored generic statement of
the same discipline was accepted, audited, corrected by its author, and
archived verbatim under `plans/drafts/`. The gap census at
[plans/reference/draft-gap-census.md](../plans/reference/draft-gap-census.md)
read every clause of that draft against ADR-016 and the identity
register and found the repository already satisfies the discipline
except where the draft generalizes a repository-specific mechanism into
adoption data, or asks for a surface no consumer has yet needed.

Two statements of one discipline is one fact with two owners. The
divergences between them are then invisible, and a reader cannot tell
which text binds. ADR-016 also answered the census question the draft
raised against it — whether the prescribed hash construction stays in
the record or moves into recipe records — only by continuing to
prescribe. The user ruled on 2026-08-19 that ADR-016 is deleted and the
draft promoted, with a single local-environment convention carrying
what deletion would otherwise lose.

---

## Decision · `dec:identity:adoption`

The repository adopts the archived draft as the normative statement of
the identity adjudication procedure. ADR-016 is retired and deleted.

**Rule (Normative source)** · `rule:identity:normative-source`

The draft text is the discipline. This record does not restate it, and
no other document in the tree may restate it: a clause is cited, never
copied. The benefit question, the adjudication walk, the assurance
classes, the property and mechanism tables, the admission and stop
records, the reductions, the procedural rules, and the implementation
gate are the draft's, and are cited at the draft with the `PLAN` prefix.

This record carries only what adoption of a generic text into this
repository must add: the local environment the draft leaves to its
adopting corpus, and the divergences that adoption records rather than
hides.

The draft is archived under `plans/`, so it is owned by the planning
tree. Its authority here comes from this record, not from its location:
where the draft and a planning document disagree, the draft binds.

---

## Local environment · `rule:identity:recipes`

The draft prescribes no hash construction. It fixes the properties an
identity must deliver and reduces every construction question to them
(`[PLAN-red:identity:scheme-to-properties]`), leaving the scheme a fact
of the recipe record (`[PLAN-def:identity:recipe]`). This repository's
recipe records therefore carry the construction, and this is the one
local-environment convention adoption adds.

**The convention.** Every first-party semantic identity of this
repository is computed as

\[I_X = H(D_X \parallel V_X \parallel C(P_X(X)))\]

where \(D_X\) is a domain separator naming the role, \(V_X\) identifies
the recipe, \(P_X\) is the semantic projection, \(C\) is canonical
serialization, and \(H\) is SHA-256. The domain separator folds in the
recipe identifier, so the identifier is a hashed input rather than an
envelope annotation. A new first-party semantic identity admitted under
(`[PLAN-req:identity:admission-record]`) takes this construction unless
its own recipe record states and justifies another; the draft's property
table (`[PLAN-tab:identity:class-properties]`) remains the test the
construction has to pass, and this convention is the answer that
happens to pass it everywhere in this tree today.

**Recipes in force.** Each is a code-side register in
`packages/architecture/src/`, published beside its value wherever the
value is published.

| Recipe identifier | Subject | Domain separator |
|---|---|---|
| `sha256-canonical-json-v3` | Architecture semantic hash | `tripod canonical manifest JSON v3` |
| `sha256-canonical-json-behavioural-v3` | Architecture behavioural hash | `tripod behavioural JSON v3` |
| `sha256-anchor-set-v2` | Anchor-set hash | `tripod layer-0 anchor set v2` |
| `sha256-canonical-json-deployment-v1` | Deployment-profile hash | `tripod deployment profile JSON v1` |

Retired identifiers are retained beside each in
`RETIRED_SEMANTIC_HASH_ALGORITHMS`, `RETIRED_BEHAVIOURAL_HASH_ALGORITHMS`,
and `RETIRED_ANCHOR_SET_HASH_ALGORITHMS`, so no name is ever reused.

The per-class admission and stop records for every digest the tree
computes live in [the identity register](../plans/registers/identities.md),
which is where an admission is read, not here.

---

## Current identities · `tab:identity:current`

The standing of each identity the repository holds today. This is
adoption data, not policy the draft states: the draft's illustrative
inventory (`[PLAN-ex:identity:inventory]`) is generic, and these are
this repository's actual holdings and the constraints on them.

| Identity | Purpose | Standing |
|---|---|---|
| Git commit and tree IDs | Source provenance | Retain; never protocol identity |
| Document UUID | Exact paper-input provenance in XMP | Retain; publication-only |
| Instance UUID | Paper-subtree Git-tree provenance in XMP | Retain; publication-only |
| Anchor-set hash | Exact imported attestation dependency set | Retain; domain-separated |
| Architecture semantic hash | Canonical complete architecture meaning | Retain; domain-separated |
| Architecture behavioural hash | Realization-major versioning gate only | Retain; do not propagate as a general runtime identity |
| Generated-file exact comparisons | Publication freshness | Retain; add no redundant hash |
| Deployment-profile hash | Future aggregate deployment-profile identity | Retain as pre-release infrastructure; it gives no release assurance until a real consumer validates it |
| Raw artifact/report hash fields in profile schema 2 | Provisional references | Must receive owned recipes and typed report/artifact references before production release |

Document provenance identities must not enter realization, compiler,
target, bundle, ABI, or protocol identities, and deployment calibration
must bind the exact final bundle and transaction ABI before production
release — both being this repository's naming of what
(`[PLAN-rule:identity:provenance-containment]`) requires generically.

---

## Identity chain · `rem:identity:chain`

The draft's running illustration of an immediate-dependency graph
(`[PLAN-model:identity:objects]`) uses generic names. This repository's
chain, along the immediate edges of
(`[PLAN-rule:identity:immediate-edges]`), is:

```text
ArchitectureSemanticId
    → RealizationId
    → CompilerPlanId
    → TargetPlanId
    → LinkedBundleId
    → TransactionAbiId
    → DeploymentProfileId
    → ReleaseManifestId
```

Only the first is built. The rest are the named positions later work
fills, recorded here so that a future identity arrives at a place the
graph already has rather than inventing one.

---

## Recorded separation migration · `rule:identity:separation-migration`

Every first-party semantic identity is domain-separated. The two
recipes that once predated that form — the architecture semantic hash
and the Layer-0 anchor-set hash — were migrated together under
(`[PLAN-rule:identity:recipe-permanence]`), and the earlier
grandfathered exception is superseded. No exception remains.

**Old recipes.** The architecture semantic hash was SHA-256 over the
canonical architecture JSON body, algorithm `sha256-canonical-json-v2`,
with the identifier carried beside the digest in the envelope rather
than inside the hashed input. The Layer-0 anchor-set hash was SHA-256
over the newline-joined sorted distinct anchor names, publishing no
identifier at all; it is named `sha256-anchor-set-v1` retroactively so
this record can refer to it.

**New recipes.** Each prefixes its existing hashed input with a domain
separator folding in the recipe identifier, exactly as the behavioural
and deployment-profile recipes do: `sha256-canonical-json-v3` under
`tripod canonical manifest JSON v3`, and `sha256-anchor-set-v2`
under `tripod layer-0 anchor set v2`. The anchor-set
identifier is a code-side register, not a new manifest field:
publishing it would add a hashed body field and force a schema bump,
which this migration does not make.

**Reason.** The adopted adjudication discipline requires domain
separation for every semantic identity, and the user ruling of
2026-08-16 folded the migration into the same re-pin cycle as DI-002
rather than deferring it to a separate consumer-driven event.

**Meaning or measurement.** Measurement only. No projection, canonical
encoding, digest algorithm, included field, or exclusion rule changed.
Both identities identify exactly what they identified before.

**Old and new identities.**

| Identity | Old value | New value |
|---|---|---|
| Architecture semantic hash | `4039b936…dbb196ec` | `59d102a9…c66b2a1c` |
| Anchor-set hash | `1b7dff61…f13fa1417` | `766e7d5f…e0d5b258` |

The behavioural hash did not move: its recipe and its body are
untouched, and the versioning gate keys on it, so this migration is not
a version change — consistent with the closing sentence of
(`[PLAN-rule:identity:recipe-permanence]`).

**Consumer transition.** Every consumer moved in one change set: the
typed pin, both generated manifests, the realization document's
masthead and attached appendix, and the synthetic release-profile
identity, which moved because the deployment profile binds the
architecture semantic hash as a hashed input while its own recipe
stayed `sha256-canonical-json-deployment-v1`. No dual-acceptance window
exists and none is needed; the retired identifiers are recorded in
`RETIRED_SEMANTIC_HASH_ALGORITHMS` and
`RETIRED_ANCHOR_SET_HASH_ALGORITHMS` so neither name is ever reused.

Two identities recorded here as new values have since moved again, both
times for a label rename rather than for anything about identity:
DI-006 moved the anchor set, and the
semantic hash followed it. DI-007 moved neither. The
values in the table above are the migration's own old and new pair and
are not the current pins; the current pins are the typed constants and
the realization masthead, which are the published surfaces.

---

## Hash-citation audit · `rule:identity:hash-citation-audit`

A procedure that adjudicates digests is incomplete without an instrument that keeps its tree honest about them, so this repository carries one. Every hexadecimal value in the tracked tree must be described by a rule in [the families table](../lint/hash-citation-families.tsv); the check is `check-hash-citations`, a member of the lint suite that asks git for the complete tracked set on every build. A rule names the value's group — this repository's own material, or another project's — what the value measures, and the program that writes it. A value no rule describes fails the check, and that is the check's only failure mode.

The generation program is the column that does the work, and requiring it is the same demand the procedure makes of an admitted identity: a value whose producer is not on the record cannot be regenerated once its subject moves, and quietly becomes a claim about the tree that nothing maintains. Naming "None" is a real answer and a strong one, because it says the value is authored rather than measured, and an authored value has nothing to drift from.

The audit places rather than recomputes. Whether a digest is right belongs to the test that owns it; re-deriving it here would be a second opinion about one computation rather than a new check, and it would fail for reasons that are not lint failures. What the audit establishes is narrower and is covered nowhere else: that every value has an owner on the record.

A described value is not yet a checkable one. Every rule therefore also declares how a reader establishes the value, and the declaration is a claim the audit enforces rather than prose: a value is **recomputable** when this tree carries the bytes and names the algorithm that regenerate it, **resolvable** when it names a referent outside this tree that a reader can fetch, or **defined** when it is an authored input something other than the author fixes — a specification, a published algorithm, mathematics, a stated string, or a visible pattern that is its own definition. A rule may declare none of these only by declaring the value's absence: no rule may describe a value as a digest or identifier that nothing regenerates and nothing resolves. Such a value is refused, and the refusal is the audit's second failure mode.

The demand this makes of authored constants is the one that keeps the class honest. An input a specification assigns has a definition and stays with it beside the value. An input nothing fixes is a number somebody chose, and publishing it beside checkable ones asks a reader to tell them apart by trusting the author, which is exactly what a repository resting on checkable digests cannot ask. Where such a constant is needed, it is derived rather than drawn: the value becomes the digest of a stated string, and the string and the generator are published beside it, so a reader recomputes what would otherwise be a promise. The repository's own internal key is the pattern — it is the digest of the curve generator's encoding, and nobody has to take it on faith.

No class is admitted without a way of being checked, and the case for an exception was made and refused. A property-test regression seed looks like one, because it is not offered as the digest of anything and because deriving a different value destroys the reproduction the file exists for. But what is worth preserving there is the counterexample, not the roll that happened to find it: a counterexample is preserved by writing down its inputs as a named test, which states what the seed only replays. So the seeds derive from stated strings like every other authored constant, and the counterexamples are written down as what they are.

The table's one-way growth is unchanged and now carries more weight, because a rule is a standing claim not only that a kind of value has an owner but that it has a way of being checked, and neither claim is withdrawn when today's occurrences change. What is withdrawn is a rule whose kind of value the tree no longer carries at all: a rule describing nothing is not a standing claim but a stale one, and it goes with the values it described.

The table grows one way. Rules are added when a value needs one and are not removed when the tree stops carrying that value, because a rule is a standing claim about a kind of value rather than an inventory of today's occurrences. A rule that claims nothing is reported and never enforced. The consequence is the property worth having: the audit can only ever fail as a value nothing describes — never as a value that used to be allowed and is not now — so a failure always points at something newly unaccounted for rather than at a policy that moved underneath the tree.

---

## Recorded divergences · `rem:identity:divergences`

Adoption records where the repository does not yet meet the draft, so
that absence is a documented state rather than a silence.

**The adjudication walk is not a standing procedure.** The draft makes (`[PLAN-alg:identity:adjudication]`) the required path for every proposal, with the benefit criterion (`[PLAN-crit:identity:benefit]`) as its test. DI-004 walked every digest the tree computes and recorded the outcomes, so the census is complete. The audit above holds the one part of that walk a program can settle — that no value enters without a named producer and a stated referent — and what is absent is the rest: whether a digest earns its place, which assurance class it takes, and what the deciding branch was when the outcome was to admit nothing. Those are review judgements rather than checked ones, and the backlog carries them textually.

**The release surfaces do not exist.** No evidence envelope, no release
manifest, and no release validator is built, so
(`[PLAN-case:identity:evidence]`), (`[PLAN-case:identity:release]`),
(`[PLAN-rule:identity:duties]`), and (`[PLAN-rule:identity:delegation]`)
are unimplemented. They are unimplemented rather than divergent: each
activates with its named consumer, and no consumer exists.

**The well-founded-graph rule is vacuous.** Exactly one identity in the
chain above is built, so (`[PLAN-rule:identity:well-founded-graph]`)
holds trivially and has never been tested. It becomes a real obligation
with the second link.

No other clause of the draft is unmet, and no clause is amended: the
local convention above adds a construction where the draft deliberately
leaves one open, which is adoption data and not an amendment.

---

## Adoption gate · `gate:identity:adoption`

Adoption holds when:

- the draft is archived, cited as normative here, and restated nowhere;
- ADR-016 is absent from the tree, and no live citation or
  cross-reference to it survives. Dated records that describe the
  repository as it stood — the guides, the reviews under
  `plans/reviews/`, the closed history, and the gap census — keep
  naming it, which is what a dated record is for;
- the local recipe convention names every recipe in force, and each
  named identifier matches the constant the owning code publishes;
- the current-identities table matches the identity register's census,
  and the register remains where an admission or a stop is read;
- every divergence above is either discharged or still recorded;
- the corpus-wide label check passes in continuous integration, no
  citation to a retired owner dangling.

Every item holds as of this record. The draft's own implementation gate
(`[PLAN-gate:identity:implementation]`) is discharged from the draft
and this record's adoption data together, exactly as its last bullet
requires; the three divergences above are the bullets it does not yet
clear.
