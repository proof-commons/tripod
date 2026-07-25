# Identity and Hash Register · `reg:identities:ownership`

This register inventories every identity and digest the current tree actually
produces, and fixes the ownership boundary of those that do not yet exist. The
immediate policy owner is
[ADR-016](../../adr/016-semantic-identities-and-evidence-binding.md). This file
is a planning aid, not a substitute for typed identity definitions, and is
never toolchain input.

Admission of any new digest is governed by
(`[ADR016-rule:identity:admission]`).

## 1. Status vocabulary · `tbl:identities:status`

| Status | Meaning |
|---|---|
| active | a real in-tree consumer decides something with it today |
| publication-only | provenance labelling for published documents; never enters semantic flow |
| dormant | produced and typed, but no consumer decides with it yet |
| provisional | a carried pre-production field whose recipe or role is undefined |
| historical | retired; retained only so a published value is not silently redefined |

A dormant or provisional identity is not release-ready and must not be cited as
evidence of anything.

## 2. Current inventory · `sec:identities:current`

Nine identity-bearing mechanisms exist in the tree. Each records its subject,
owner, producer, consumer, decision, assurance class, stale condition, recipe,
migration rule, non-claims, and status.

### 2.1 Git commit and tree object IDs

- **Subject:** exact committed bytes, under Git's object model.
- **Owner:** Git.
- **Producer:** Git.
- **Consumer:** `document-stamps` (date, timestamp, instance identity) and
  publication tooling.
- **Decision:** which committed source state a published paper derives from.
- **Assurance:** Git object naming; source provenance only.
- **Stale condition:** any commit touching the subject path. A dirty paper
  subtree is a hard failure, not a stale value.
- **Recipe:** Git's own object hashing; never re-derived here.
- **Migration:** follows Git. A repository-wide object-hash change replaces
  every value at once.
- **Non-claims:** no protocol, realization, compiler, target, bundle, or
  deployment meaning.
- **Status:** publication-only.

### 2.2 Document identity

- **Subject:** the exact bytes of the declared publication-input set.
- **Owner:** `document-stamps`.
- **Producer:** the document digest in `packages/document-stamps/src/lib.rs`.
- **Consumer:** the paper's PDF metadata, through the generated stamps include.
- **Decision:** whether two built PDFs came from the same exact paper input
  set.
- **Assurance:** SHA-256 over a domain-separated, length-framed encoding,
  truncated to its first 128 bits. A 128-bit prefix is adequate for provenance
  labelling and for nothing stronger.
- **Stale condition:** any change to an input's repository-relative path, Git
  mode, or bytes. Membership comes from the build's declared input set, so an
  input silently dropped from that set is a census defect, not a stale digest.
- **Recipe:** SHA-256 over the domain string followed by the framed path,
  framed Git mode, and framed bytes of each input in declared order; the first
  sixteen bytes, printed in lowercase `8-4-4-4-12` grouping.
- **Migration:** changing the domain string, the framing, or the input-set
  definition produces a different identity, and must be recorded as a
  replacement rather than a redefinition of the published one.
- **Non-claims:** not authenticity, not a signature, and deliberately not a
  version-tagged or variant-tagged identifier. It carries no protocol meaning.
- **Status:** publication-only.

### 2.3 Paper instance identity

- **Subject:** the Git tree object for the paper subtree.
- **Owner:** `document-stamps`.
- **Producer:** the instance derivation in
  `packages/document-stamps/src/lib.rs`, which peels a revision to its tree
  object.
- **Consumer:** the paper's PDF metadata.
- **Decision:** which paper-subtree instance a PDF was built from.
- **Assurance:** Git tree object naming, truncated to its first 128 bits and
  printed with the same grouping as the document identity.
- **Stale condition:** any change under the paper subtree, including changes to
  files outside the declared input set.
- **Recipe:** resolve the paper subtree path against the revision, peel it to
  the tree object, take the leading 128 bits of that object name.
- **Migration:** follows Git's object hash.
- **Non-claims:** it is not the document identity and must not be substituted
  for it — the two answer deliberately different questions, subtree state
  against declared input set. No protocol meaning.
- **Status:** publication-only.

### 2.4 anchor-set hash

- **Subject:** the set of specification anchor names the realization document cites.
- **Owner:** architecture holds the pin; labels recomputes it.
- **Producer:** the anchor-set hash in
  `packages/architecture/src/canonical.rs`.
- **Consumer:** the label and architecture weld in
  `packages/labels/src/repository.rs`, which fails the build on mismatch;
  release validation refuses a manifest that leaves the pin unset.
- **Decision:** whether the imported specification dependency set still equals the
  pinned set.
- **Assurance:** exact set equality under SHA-256. Occurrence order and repeats
  are normalized away before hashing, so the value is a property of the set
  alone.
- **Stale condition:** adding, removing, or renaming any cited anchor.
- **Recipe:** SHA-256 over the sorted distinct anchor names joined by newlines,
  with no consumer prefix on the names.
- **Migration:** a deliberate anchor-set change re-pins the manifest value in
  the same commit that changes the citations.
- **Non-claims:** it says nothing about what the anchors mean, and it is not an
  architecture identity.
- **Status:** active.

### 2.5 Architecture semantic hash

- **Subject:** the complete canonical architecture export body.
- **Owner:** architecture.
- **Producer:** the semantic hash in `packages/architecture/src/canonical.rs`,
  under an explicit algorithm identifier.
- **Consumer:** the realization architecture binding, the model ledger's query
  context, the deployment profile, and the document and artifact weld.
- **Decision:** whether two values were derived from the same complete
  architecture meaning.
- **Assurance:** exact canonical-form equality under SHA-256. Canonicalization
  carries the meaning; the digest only compares it.
- **Stale condition:** any change to the exported architecture body.
- **Recipe:** SHA-256 over the canonical JSON bytes of the architecture export
  body.
- **Migration:** the algorithm identifier is carried explicitly beside the
  value, so a recipe change is a visible measurement change.
- **Non-claims:** not authenticity, not deployment readiness, not target
  correctness, and not a substitute for validation.
- **Status:** active.

### 2.6 Architecture behavioural hash

- **Subject:** the behavioural projection of the architecture export — assets,
  roots, objects, operations, quantities, projected witnesses, clauses and
  bounds, amount limits, and tags. Dependencies, decisions, and the envelope
  are excluded.
- **Owner:** architecture.
- **Producer:** the behavioural hash in
  `packages/architecture/src/canonical.rs`, under its own algorithm identifier
  and domain-separation prefix.
- **Consumer:** the versioning gate in
  `packages/architecture/src/tests/versioning_gate_tests.rs`. That test is the
  gate: the hash moving while the realization version is unchanged fails the
  build.
- **Decision:** whether a change moved the denotation. Only a change of
  denotation may move the behavioural hash.
- **Assurance:** exact equality of the projected behavioural body under a
  domain-separated SHA-256.
- **Stale condition:** any behavioural change. Presentation changes — document
  labels, calibrated draft bound defaults — deliberately do not move it; that
  immunity is the point of the projection.
- **Recipe:** SHA-256 over the domain prefix followed by the canonical JSON
  bytes of the behavioural body.
- **Migration:** the gate is pinned per algorithm. Two earlier algorithms are
  retired, each recorded in the gate test with its pinned release value and the
  reason it was replaced, so no published value is silently redefined.
- **Non-claims:** not the complete publication bytes, and not a second general
  architecture identity. It must not be repeated through future artifacts as
  though it identified the architecture.
- **Status:** active, as a narrow versioning witness only.

### 2.7 Generated-file exact comparison

- **Subject:** the exact bytes of committed generated files.
- **Owner:** the artifact and label checkers.
- **Producer:** none. This is a comparison, not a digest.
- **Consumer:** the generated-file check lane and CI.
- **Decision:** whether a committed generated file still equals what its
  generator produces.
- **Assurance:** byte equality, which is strictly stronger than any digest over
  the same bytes.
- **Stale condition:** any regeneration difference.
- **Recipe:** not applicable. Generators write compare-if-changed and the
  checker compares directly.
- **Migration:** none.
- **Non-claims:** this is not an identity and nothing may cite it as one.
  Adding a hash beside it would mint a redundant identity with no consumer, in
  direct conflict with the admission rule.
- **Status:** active, and deliberately digest-free.

### 2.8 Deployment-profile hash

- **Subject:** the canonical deployment-profile JSON.
- **Owner:** architecture owns the type; release will own the value.
- **Producer:** the deployment-profile hash in
  `packages/architecture/src/deployment.rs`, under its own domain prefix.
- **Consumer:** architecture tests only. The release consumer does not exist.
- **Decision:** none today. Its future decision is whether two deployments
  claim the same profile.
- **Assurance:** exact canonical-form equality under a domain-separated
  SHA-256, over a schema that is itself incomplete.
- **Stale condition:** any profile field change.
- **Recipe:** SHA-256 over the domain prefix followed by the canonical JSON
  bytes of the profile.
- **Migration:** the documented schema-2 bundle and ABI binding limitation is
  release-blocking, and is repaired by the profile migration task rather than
  by appending fields.
- **Non-claims:** it is not the architecture hash, not deployment readiness,
  and it carries no present release meaning.
- **Status:** dormant until a release consumes it.

### 2.9 Profile artifact and report hash fields

- **Subject:** raw fixed-width hash fields carried inside the deployment
  profile — unit-test and property-test reports, three independent-observer
  reports, script integration, and the artifact hash rows.
- **Owner:** a future release producer.
- **Producer:** none in-tree. The fields are carried, not derived here.
- **Consumer:** profile validation, which checks presence and binding
  selectively.
- **Decision:** none. The typed roles and the digest recipes are undefined, so
  no field currently decides anything.
- **Assurance:** none beyond presence. This is the weakest entry in the
  inventory and is recorded as such.
- **Stale condition:** undefined, because the recipes are undefined.
- **Recipe:** undefined. That absence is precisely the defect the typed
  evidence envelopes task must repair.
- **Migration:** replaced by typed evidence references — role, schema, subject
  identities, producer, configuration, result status, and payload — before any
  release use.
- **Non-claims:** a raw report digest with no typed role and no bound subject
  proves nothing. Different report hashes do not demonstrate independent
  implementations, and a self-consistent local cache is not independent
  target-chain evidence.
- **Status:** provisional pre-production reference.

## 3. Inventory decisions · `rem:identities:decisions`

- Active identities with real consumers are kept: the anchor-set hash, the
  architecture semantic hash, and the behavioural hash.
- Publication-only identities stay out of semantic flow. Document provenance
  must not enter protocol, realization, compiler, target, bundle, ABI, or
  deployment semantics.
- The deployment-profile identity is dormant until release consumes it, and is
  named dormant wherever it appears.
- Exact generated-byte checking stays digest-free. No hash is added beside a
  byte comparison.
- Raw profile artifact and report hash fields are identified as provisional
  pre-production references, not evidence identities.
- No entry lacks a present or explicitly deferred consumer, so this pass
  removes nothing.

No entry carries two incompatible meanings. The semantic and behavioural hashes
have deliberately different subjects under separate domains, and the document
and instance identities answer deliberately different provenance questions.

## 4. Local handles, never identities · `rule:identities:handles`

Petgraph node and edge indices are local in-memory graph positions owned by the
graph-holding package. They are not semantic identity, not publication
identity, and not evidence identity, and they must never enter a canonical
projection — see (`[ADR016-rule:identity:classes]`).

## 5. Ownership boundary for identities not yet minted · `sec:identities:future`

These identities do not exist. The table fixes ownership and boundary only;
activation points, immediate consumers, and per-edge assurance are defined by
the future identity DAG task under
(`[ADR016-rule:identity:immediate-edges]`). No row below may be minted before a
real consumer exists.

| Identity | Owner | Binds | Does not replace |
|---|---|---|---|
| realization identity | realization | target-independent semantic graph | architecture identity |
| compiler configuration identity | compiler | analysis and policy choices | realization identity |
| analyzed-program identity | compiler | one normalized relation analysis | linked bundle |
| target-definition identity | target package | typed compatibility contract | node implementation revision |
| deployment-instance identity | target/release | network, genesis, activation/configuration | target-definition identity |
| backend configuration identity | backend | proof patterns and lowering policy | target identity |
| relocatable bundle identity | backend | unlinked target programs and relocations | linked deployment bytes |
| linked-bundle identity | linker | final programs, constructors, constants, bounds | deployment profile |
| transaction ABI identity | transaction | layouts, witnesses, metadata, bundle binding | semantic realization |
| vector-set identity | vectors | fixtures, mutations, coverage policy | execution report |
| evidence-report identity | report owner | exact claim, artifacts, tool, results | another evidence class |
| release identity | release | final manifest, assets, profile, policy | protocol denotation |

Implementation revisions remain review or test provenance unless a typed
compatibility contract explicitly makes another fact identity-relevant.

Phase-1 realization mints no public hash. It carries an explicit architecture
binding and stable typed keys for its scoped declarations only; complete
realization identity remains deferred until the complete-scope schema and
projection policy are reviewed.
