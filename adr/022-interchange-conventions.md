# ADR-022: Adoption of the Interchange Conventions

**Status:** Decided; normative now and unimplemented by design — no
registry, no namespace allocation, and no encoder or decoder is built
until an externally consumed document exists
**Scope:** Every document this repository emits for consumption outside
itself
**Adopts:** the archived draft at
[plans/drafts/interchange-conventions.md](../plans/drafts/interchange-conventions.md),
the fourth and forward-compatibility edition, as the normative statement
of the wire-format discipline
**Excludes:** first-party command-line streams and the native executor
protocol, both of which ADR-010 owns
**Does not establish:** any namespace claim, any assigned theory, any
identity over an interchange document, or any obligation on a document
that never leaves this repository

---

## Context · `sec:interchange:context`

The repository emits no externally consumed document today. An
externally authored generic statement of a wire-format discipline was
accepted, audited, corrected by its author, and archived verbatim under
`plans/drafts/`. The gap census at
[plans/reference/draft-gap-census.md](../plans/reference/draft-gap-census.md)
read every clause of that draft against the tree and found no
implementation at all — correctly, because nothing here encodes CBOR or
CDDL and the draft names no consumer.

The edition in force is the author's fourth, supplied 2026-08-18, which
adds forward compatibility as a construction rather than a hope:
ceiling-based acceptance over downward-closed holding, tolerant
validation against the floor's open companion, and stamps naming a
never-assigned coordinate rejected as checkably false claims. Within a
held major no conforming document is ever rejected.

The user ruled on 2026-08-19 that the draft is adopted wholesale, takes
priority, and that overlapping text is deleted rather than maintained
beside it. Adopting it before a consumer is the point rather than a
defect of timing: the draft fixes each convention once for the whole
system (`[PLAN-req:interchange:fixed-once]`).

---

## Decision · `dec:interchange:adoption`

The repository adopts the archived draft as the normative statement of
the interchange conventions: the wire-format discipline for every
document it emits for consumption outside itself.

**Rule (Normative source)** · `rule:interchange:normative-source`

The draft text is the discipline. This record does not restate it, and
no other document in the tree may restate it: a clause is cited, never
copied. The two languages, the satisfaction judgment, the base theory,
acceptance and the open companion, the assignable fragment, the registry
signature, the four invariants, the Law, the six meta-theorems and the
caveats are the draft's, and are cited at the draft with the `PLAN`
prefix.

This record carries only what adoption of a generic text into this
repository must add: the boundary against the record that owns
first-party streams, the recorded stop that keeps the one live wire
surface out of scope, the alignment decisions the draft's preamble
leaves to its adopting corpus, and the standing of an adopted discipline
with no implementation.

The draft is archived under `plans/`, so it is owned by the planning
tree. Its authority here comes from this record, not from its location:
where the draft and a planning document disagree, the draft binds.
Inside `plans/` its clauses are cited in the local parenthesized form,
the draft being PLAN-owned; from this record, from crate sources, and
from DOC-owned READMEs they are cited with the `PLAN` prefix.

---

## Boundary against ADR-010 · `rule:interchange:boundary`

Two records govern structured output in this repository, and they divide
by what is being produced, not by encoding.

ADR-010 governs **first-party command-line streams**: what a workspace
executable writes to standard output and standard error, its JSON result
encoding (`[ADR010-rule:output:json]`), its shared control-plane records
(`[ADR010-rule:output:control-plane]`), its diagnostics and its exit
classes. Those streams are consumed by shell scripts, Meson and CI
inside this repository, by an invoking process that chose the invocation.

ADR-022 governs **externally consumed documents**: artifacts that leave
this repository for a reader that did not invoke it. Their encoding,
their envelope, their versioning and their acceptance are the adopted
draft's.

The two do not overlap, and neither defers to the other. A stream is not
a document: it has exactly one reader, fixed at invocation; it carries no
namespace label and no version triple; it is dispatched by nothing,
because the caller already knows what it asked for; and it is not
archived, so nothing later validates it against a theory assigned today.
A document is the converse in each respect, which is what the envelope
and the registry exist to serve. Neither record amends the other or is
read onto the other's surface, and a surface that ever falls under both
is adjudicated at that moment, never assumed into one of them.

---

## The executor protocol keeps its stop · `rem:interchange:executor-stop`

The one live wire surface in the tree is the native executor protocol in
`packages/target-elements-conformance/src/executor.rs` and its mock
peer. It does not adopt these conventions, and adoption does not reach
it.

The reasoning is the stop record DI-004 wrote at section 3.10 of
[the identity register](../plans/registers/identities.md), restated here
as this record's own because the register's subject was a digest and
this record's is the wire format. A digest or signature over the
protocol's frames was proposed; no consumer's decision would change,
because the protocol is a line-delimited JSON pipe between two
first-party processes in one repository, with no archive, no
third-party reader and no version negotiation, and ADR-010 owns its
shape. The revisit condition is the register's and is unchanged: a frame
stream that becomes archived, read by a third party, or consumed by a
release as evidence stops satisfying the branch and is walked again — at
which point it has become an externally consumed document, and this
record rather than the exclusion applies to it.

The protocol's revision counter versions that pipe internally. It is not
a version in the adopted sense (`[PLAN-def:interchange:versions]`) and
claims none of that definition's consequences.

---

## Alignment records · `rem:interchange:alignment`

The draft cites no label of any other document of an adopting corpus and
presupposes acceptance of no other; where it consumes another
discipline's artifact it takes it as adoption data and asks nothing of
its provenance. Alignment is therefore the corpus's to record. These are
this corpus's.

**Labeling.** The draft's namespace labels
(`[PLAN-gram:interchange:label-grammar]`) and the documentation labels of
the calculus adopted by (`[ADR019-dec:labels:adoption]`) are unrelated,
as the draft itself disclaims: only the word is shared. This corpus
records that it means the disclaimer. A namespace label never names a
documentation environment, no documentation label is ever an envelope
value, and neither grammar constrains the other — the label checker has
no jurisdiction over a namespace label, and a namespace registry would
have none over a mint.

**Identity.** Byte equality of canonical names
(`[PLAN-metathm:interchange:unique-names]`) is what content addressing
over interchange documents would rest on, but the conventions mint no
identity and this adoption admits none. A digest over an interchange
document is an identity question, owned by the procedure adopted at
(`[ADR021-dec:identity:adoption]`) and constructed under
(`[ADR021-rule:identity:recipes]`). The first such document proposing a
digest walks that adjudication and takes an admission record in the
identity register, as any other proposal does.

---

## No implementation until a consumer · `rule:interchange:no-implementation`

Adoption is normative now and builds nothing now. No registry
(`[PLAN-sig:interchange:theory-assignment]`), no namespace allocation, no
canonical encoder and no validating decoder is built until a real
externally consumed document exists. Until then the governance
obligations of (`[PLAN-cav:interchange:governance-obligations]`) bind
nothing, there being no assignment to allocate, consolidate or preserve
meaning across.

What adoption buys before that day is that the first such document
arrives at a settled discipline instead of inventing one under its own
pressure: the envelope, the version triple, the acceptance rule and the
additive minor regime are already fixed, so the open question at that
moment is the document's content alone.

The expected first consumer class is the externally consumed artifacts of
the compact-attestation era that begins at Guide 12 — attestation-shaped
documents leaving this repository for a reader that is not this
workspace. Naming the class charters nothing: no such artifact is
specified, scheduled, or required by this record, and the first one
actually proposed carries the registry, allocation and encoder work with
it, chartered there rather than here.

---

## Adoption gate · `gate:interchange:adoption`

Adoption holds when:

- the draft is archived, cited as normative here, and restated nowhere;
- the boundary above holds in both directions: no ADR-010 surface claims
  an interchange envelope, and no interchange clause is read onto a
  command-line stream;
- the executor protocol keeps the recorded stop above, with the revisit
  condition still the identity register's;
- no namespace is allocated, no theory assigned, and no encoder or
  decoder built, while no consumer is named;
- the corpus-wide label check passes, every `PLAN` citation here
  resolving at the draft and every ADR citation at its owner.

Every item holds as of this record. The draft states no adoption gate of
its own: its conditions are its invariants, checked by the owner's
registry machinery (`[PLAN-cav:interchange:governance-obligations]`) —
machinery this repository does not have and does not owe until it
assigns its first coordinate.
