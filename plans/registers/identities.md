# Identity and Hash Register · `reg:identities:ownership`

This register inventories every identity and digest the current tree actually
produces, and fixes the ownership boundary of those that do not yet exist. The
immediate policy owner is the adopted identity adjudication procedure at
[plans/drafts/identity-adjudication.md](../drafts/identity-adjudication.md),
adopted by [ADR-021](../../adr/021-identity-adjudication.md). This file
is a planning aid, not a substitute for typed identity definitions, and is
never toolchain input.

Admission of any new digest is governed by
(`req:identity:admission-record`). Every entry below was walked through the
adopted adjudication procedure in DI-004: each digest the tree computes was
found from the owning code, put to the benefit criterion, and recorded either
as an admission record or as a stop. The classification is per identity class —
one recipe, one role, one consumer-decision pattern — and never per value.

## 1. Census · `tab:identities:census`

Every digest and identity the tree computes or carries, one row each. An em
dash in the class column is a deliberate absence, not an omission: no identity
is admitted there, and the stop record says on which branch the walk ended.

| Identity | Class | Outcome |
|---|---|---|
| Git commit and tree object IDs | provenance | admitted, 2.1 |
| Document identity | provenance | admitted, 2.2 |
| Paper instance identity | provenance | admitted, 2.3 |
| anchor-set hash | semantic | admitted, 2.4 |
| Architecture semantic hash | semantic | admitted, 2.5 |
| Architecture behavioural hash | semantic | admitted, 2.6 |
| Generated-file exact comparison | — | stopped, 3.1 |
| Field-level digests | — | stopped, 3.2 |
| Deployment-profile hash | — | pre-admission, 3.3 |
| Profile artifact, report, dependency and calibration hash fields | — | pre-admission, 3.4 |
| Native conformance report identity | — | stopped, 3.5 |
| Compiler plan and analyzed-program identity | — | stopped, 3.6 |
| Realization identity | — | stopped, 3.7 |
| Target-protocol tagged hashes and sighashes | — | stopped, 3.8 |
| Received chain identifiers | — | stopped, 3.9 |
| Executor wire-frame identity | — | stopped, 3.10 |
| CI lane outputs | — | stopped, 3.11 |

Six admitted classes, eleven recorded stops. No digest the tree computes is
outside this table.

### 1.1 Status vocabulary · `tab:identities:status`

| Status | Meaning |
|---|---|
| active | a real in-tree consumer decides something with it today |
| publication-only | provenance labelling for published documents; never enters semantic flow |
| dormant | produced and typed, but no consumer decides with it yet |
| provisional | a carried pre-production field whose recipe or role is undefined |
| historical | retired; retained only so a published value is not silently redefined |

A dormant or provisional identity is not release-ready and must not be cited as
evidence of anything.

## 2. Admission records · `sec:identities:current`

Six identity classes are admitted. Each records its subject, owner, producer,
consumer, decision, assurance class, stale condition, recipe by identifier,
migration rule, non-claims, and status — the admission fields the adopted
procedure requires, in the order
(`req:identity:admission-record`) states them.

### 2.1 Git commit and tree object IDs

- **Subject:** exact committed bytes, under Git's object model.
- **Owner:** Git.
- **Producer:** Git.
- **Consumer:** `document-stamps` (date, timestamp, instance identity) and
  publication tooling.
- **Decision:** which committed source state a published paper derives from.
- **Class:** provenance, locator-grade. Never release-bound, so the weaker
  conditional collision row applies.
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
- **Class:** provenance over an exact canonical input set, locator-grade. The
  128-bit truncation is admissible only because the value is never
  release-bound; a release-bound provenance identity could not carry it.
- **Assurance:** SHA-256 over a domain-separated, length-framed encoding,
  truncated to its first 128 bits. A 128-bit prefix is adequate for provenance
  labelling and for nothing stronger.
- **Stale condition:** any change to an input's repository-relative path, Git
  mode, or bytes. Membership comes from the build's declared input set, so an
  input silently dropped from that set is a census defect, not a stale digest.
- **Recipe:** validate and deduplicate the publication inputs, sort them by
  canonical repository-relative path, then hash the domain separator
  `tripod/document-inputs/v1` followed by a NUL byte, and for each
  input in that sorted order its length-framed repository-relative path,
  length-framed Git mode, and length-framed committed blob bytes. Take the
  first 128 bits and render them in lowercase `8-4-4-4-12` grouping without
  rewriting any bit as a version or variant field. Sorting is what makes the
  identity independent of argument order; a repeated path is refused rather
  than hashed twice.
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
- **Class:** provenance over a named tree, locator-grade, on the same
  never-release-bound condition as the document identity.
- **Assurance:** Git tree object naming, truncated to its first 128 bits and
  printed with the same grouping as the document identity.
- **Stale condition:** any change under the paper subtree, including changes to
  files outside the declared input set.
- **Recipe:** resolve the paper subtree path against the revision to its tree
  object name, take the leading 32 hex characters, then re-resolve that prefix
  peeled to a tree and require it to equal the full object name. Render the
  prefix in the same `8-4-4-4-12` grouping as the document identity. The
  round-trip is part of the recipe, not a convenience: an ambiguous prefix or a
  unique non-tree object is a hard failure rather than a silently truncated
  identity.
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
- **Class:** semantic. The recipe carries the full semantic column: it is
  deterministic over the set, complete over it, free of occurrence order and
  repetition, domain-separated, recipe-identified, and recomputable by the
  labels checker, which is what the weld does on every build.
- **Assurance:** exact set equality under SHA-256. Occurrence order and repeats
  are normalized away before hashing, so the value is a property of the set
  alone.
- **Stale condition:** adding, removing, or renaming any cited specification anchor.
- **Recipe:** `sha256-anchor-set-v2`: SHA-256 over the domain prefix followed by
  the sorted distinct anchor names joined by newlines, with no consumer prefix
  on the names.
- **Migration:** a deliberate anchor-set change re-pins the manifest value in
  the same commit that changes the citations. The recipe itself migrated once,
  in DI-004, when the domain prefix was added; the retired `sha256-anchor-set-v1`
  and both values are recorded in
  (`[ADR021-rule:identity:separation-migration]`). DI-008: the specification's
  v1.0.0 release renamed the paper's own-division label area to `attestation`,
  moving two anchor names; the retired value was
  reproduced from the pre-rename set before the new value was taken;
  the recipe and its domain prefix are unchanged (the prefix is a frozen recipe
  string, not prose).
- **Non-claims:** it says nothing about what the anchors mean, and it is not an
  architecture identity.
- **Status:** active.

### 2.5 Architecture semantic hash

- **Subject:** the complete canonical architecture export body.
- **Owner:** architecture.
- **Producer:** the semantic hash in `packages/architecture/src/canonical.rs`,
  under an explicit algorithm identifier.
- **Consumer:** the realization architecture binding, the model ledger's
  attestation context, the deployment profile, and the document and artifact
  weld.
- **Decision:** whether two values were derived from the same complete
  architecture meaning. Each consumer's decision is a rejection: the ledger
  refuses any attestation context whose manifest hash is not this build's
  expected typed hash, on every indexer construction path, so a
  wrong-architecture or placeholder context cannot reach a query result;
  profile validation refuses a profile bound to a different architecture hash;
  and the weld fails the build when a published masthead or manifest disagrees.
- **Class:** semantic, and the only admitted identity that crosses a process
  boundary today — the ledger's schema-13 checkpoint bytes carry it, so a
  consumer that never re-derives the architecture still compares meaning.
- **Assurance:** exact canonical-form equality under SHA-256. Canonicalization
  carries the meaning; the digest only compares it.
- **Stale condition:** any change to the exported architecture body.
- **Recipe:** `sha256-canonical-json-v3`: SHA-256 over the domain prefix
  followed by the canonical JSON bytes of the architecture export body.
- **Migration:** the algorithm identifier is carried explicitly beside the
  value, so a recipe change is a visible measurement change. The recipe
  migrated once, in DI-004, when the domain prefix was added; the retired
  `sha256-canonical-json-v2` and both values are recorded in
  (`[ADR021-rule:identity:separation-migration]`). DI-008: the value moved with
  its body — the specification binding's key, version, and anchor-set pin are
  body fields — retiring its predecessor; the
  behavioural hash was unchanged across the same commit, witnessing that the
  denotation did not move.
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
- **Class:** semantic, over a projection rather than the whole export. Its
  completeness is claimed over the behavioural projection alone, which is why
  it is not a second architecture identity.
- **Assurance:** exact equality of the projected behavioural body under a
  domain-separated SHA-256.
- **Stale condition:** any behavioural change. Presentation changes — document
  labels, calibrated draft bound defaults — deliberately do not move it; that
  immunity is the point of the projection.
- **Recipe:** `sha256-canonical-json-behavioural-v3`: SHA-256 over the domain
  prefix followed by the canonical JSON bytes of the behavioural body.
- **Migration:** the gate is pinned per algorithm. Two earlier algorithms are
  retired, each recorded in the gate test with its pinned release value and the
  reason it was replaced, so no published value is silently redefined.
- **Non-claims:** not the complete publication bytes, and not a second general
  architecture identity. It must not be repeated through future artifacts as
  though it identified the architecture.
- **Status:** active, as a narrow versioning witness only.

## 3. Stop records · `sec:identities:stops`

A stop is an outcome, not an omission. Each record states the proposal, the
branch of the adjudication procedure that decided it, the date the walk was
taken, and the condition under which it is retaken. Until that condition holds,
the absence of a digest is this repository's decided state and the same
proposal is not re-adjudicated from nothing. All eleven walks were taken
2026-08-18, in DI-004, over a tree read from the owning code; where a standing
policy already refused the digest, the record fixes that refusal in the
procedure's own terms rather than restating it.

### 3.1 Generated-file exact comparison

- **Proposal:** a digest beside each committed generated file, so freshness is
  decided by comparing digests rather than bytes.
- **Branch:** the artifact branch's freshness sub-branch. Byte equality is
  strictly stronger than any digest over the same bytes, and the generators
  write compare-if-changed while the checker in `packages/artifacts/src/lib.rs`
  compares the committed bytes against the regenerated ones directly.
- **Revisit:** only if a generated file becomes independently distributed or
  release-bound, at which point the walk is an artifact-digest walk with a
  manifest row, not a freshness walk.
- **Non-claim:** the comparison is not an identity and nothing may cite it as
  one. A hash added beside it would be a redundant identity with no consumer.

### 3.2 Field-level digests

- **Proposal:** hash individual fields of a typed object to detect change.
- **Branch:** equality the parent already carries, reducing to object
  validation. Fields have no independent lifecycle and cross-field validity is
  a property of the whole, so a field digest evidences nothing the owning
  validator does not already establish.
- **Revisit:** when a field acquires an independent lifecycle of its own — its
  own transport, cache, publication, or signature boundary — which makes it an
  object, not a field, and starts a fresh walk.

### 3.3 Deployment-profile hash

- **Proposal:** admit the canonical deployment-profile hash produced by
  `packages/architecture/src/deployment.rs` under
  `sha256-canonical-json-deployment-v1`.
- **Branch:** no named consumer's decision changes. Architecture tests compute
  and compare it; no release consumes it, so equality decides nothing today.
  The value exists and is domain-separated, but it is pre-admission and carries
  no assurance.
- **Revisit:** when a release manifest consumes it. Activation is additionally
  blocked until the schema-2 residual is repaired: the profile cannot yet bind
  a linked bundle or a transaction ABI, and the calibration cannot bind the ABI
  under which its measurement was taken, so bundle equality alone does not
  prove the measured transaction shape used the final ABI.
- **Non-claims:** it is not the architecture hash, not deployment readiness,
  and it carries no present release meaning. Status: dormant.

### 3.4 Profile artifact, report, dependency and calibration hash fields

- **Proposal:** treat the raw fixed-width hash fields carried inside the
  deployment profile as evidence and artifact identities. The tree carries four
  families: seven artifact rows, six test and independent-observer report
  rows, one evidence hash per substrate dependency, and the calibration's
  evidence and script-bundle hashes.
- **Branch:** validation before admission, and no decision. Profile validation
  checks presence, nonzeroness, and the single binding that the calibration's
  script-bundle hash equals the released emitted-script-bundle artifact row.
  Beyond that the typed roles and the digest recipes are undefined, so no field
  decides anything and no stale condition can be stated.
- **Revisit:** before any production release, and not later. Each family is
  replaced by typed evidence or artifact references — role, schema, exact
  subject identities, producer, configuration, result status, and payload or
  payload digest — under an owned recipe. This is the register's weakest
  material and is recorded as such.
- **Non-claims:** a raw report digest with no typed role and no bound subject
  proves nothing. Different report hashes do not demonstrate independent
  implementations, and a self-consistent local cache is not independent
  target-chain evidence.

### 3.5 Native conformance report identity

- **Proposal:** a report digest, or a field reserved for one, on the native
  evidence reports in `packages/target-elements-conformance/src/report.rs` and
  `prototype_report.rs`.
- **Branch:** equality already given on the path. The reports are deterministic
  by construction — no clock, elapsed time, hostname, user, process identifier,
  temporary path, executor path, or environment value, and every collection
  ordered — so a report is compared by its typed content and its exact bytes
  within one repository, and no cache, publication, or distribution boundary
  lies between producer and consumer.
- **Revisit:** when a report crosses such a boundary — a release consuming it
  as required evidence, or a third party receiving it — at which point the walk
  is an evidence-identity walk binding role, schema, and exact subjects, not a
  bare digest.

### 3.6 Compiler plan and analyzed-program identity

- **Proposal:** a plan hash, analyzed-program digest, or report identity on the
  compiler's analyzed types.
- **Branch:** no boundary and typed comparison suffices. Nothing persists or
  transports an analyzed program, so the typed value is the comparison. The
  refusal is enforced structurally rather than by convention: a compile-time
  probe in `packages/compiler/src/tests/pilot_program_tests.rs` resolves
  differently the moment any analyzed type gains a hash implementation, and an
  exhaustive destructuring of the stable projection stops compiling if a digest
  field is added anywhere in it.
- **Revisit:** when an analysis result is cached across processes, published as
  an artifact, or consumed by a separately versioned backend. The one hash
  reachable inside the analysis is the architecture binding's semantic hash,
  which the analysis binds as the identity of its source; the compiler mints
  none of its own.

### 3.7 Realization identity

- **Proposal:** publish a realization hash in Phase 1.
- **Branch:** no boundary. A scoped realization is consumed as an in-process
  typed value, and the derivation carries an architecture binding — schema
  version, realization version, and the architecture semantic hash — rather
  than an identity of its own.
- **Revisit:** when a consumer receives a realization as external bytes across
  a process, cache, or publication boundary. The complete-scope schema and
  projection policy are reviewed first; a field reserved for a future digest
  before then would itself be a speculative identity.

### 3.8 Target-protocol tagged hashes and sighashes

- **Proposal:** register the digests the target constructor and the conformance
  fixtures compute — leaf and branch tagged hashes under the target's own tags,
  output-key tweaks, signature hashes, and the streaming-hash primitives the
  fixture census exercises — as identities of this repository.
- **Branch:** the walk does not reach a benefit question, because these are not
  identities of corpus objects. They are the modelled protocol's own values,
  computed under the target's recipes to construct programs the target accepts
  and to compare against what the target computes. Their equality decisions are
  the target's; the transcribed tags carry their upstream provenance beside
  each constant.
- **Revisit:** never as corpus identity. A first-party object that wanted an
  identity would take its own walk under its own recipe, and no target-protocol
  digest may be quoted as a corpus semantic, artifact, evidence, or release
  identity.

### 3.9 Received chain identifiers

- **Proposal:** admit the network identity, genesis identity, and block hashes
  carried by the attestation context, the deployment profile, and the
  executor's reported environment.
- **Branch:** nothing is proposed to be hashed. These are received typed
  fields naming target-chain material, validated by their owners — nonzero
  network and genesis identities, and prefix consistency across a validated
  chain view — under no recipe of ours.
- **Revisit:** none. They stay provenance about the chain, never enter a
  semantic identity, and are not evidence that the named chain state is
  correct.

### 3.10 Executor wire-frame identity

- **Proposal:** a digest or signature over the native executor protocol's
  frames.
- **Branch:** no consumer's decision changes. The protocol is a line-delimited
  JSON pipe between two first-party processes in one repository, with no
  archive, no third-party reader, and no version negotiation; ADR-010 owns its
  shape.
- **Revisit:** when a frame stream is archived, read by a third party, or
  consumed by a release as evidence.

### 3.11 CI lane outputs

- **Proposal:** a persistent identity over the lane records and timing reports
  that `scripts/ci.py` emits.
- **Branch:** ephemeral local evidence. Nothing consumes a lane record as
  release evidence, and importance is not a consumer.
- **Revisit:** when a lane record is named as required release evidence, which
  makes it an evidence-identity walk with a typed envelope.

## 4. Inventory decisions · `rem:identities:decisions`

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
- Every admitted class is domain-separated. The two recipes that once were not
  migrated together in DI-004 under
  (`[ADR021-rule:identity:separation-migration]`), which supersedes the
  grandfather clause; no exception remains.
- The census found no unclassified digest and admitted nothing new. Its whole
  yield on the admission side is sharper records; on the stop side it is eight
  refusals that were practice, or enforced in code, without ever being written
  down as decided outcomes.

No entry carries two incompatible meanings. The semantic and behavioural hashes
have deliberately different subjects under separate domains, and the document
and instance identities answer deliberately different provenance questions.

## 5. Local handles, never identities · `rule:identities:handles`

Petgraph node and edge indices are local in-memory graph positions owned by the
graph-holding package. They are not semantic identity, not publication
identity, and not evidence identity, and they must never enter a canonical
projection — see (`rule:identity:no-incidentals`).

## 6. Future immediate-edge identity DAG · `sec:identities:future`

None of the identities in this section exists. This section fixes, for each,
the phase that may activate it, the immediate consumer whose existence is the
activation condition, the immediate edges it binds, its assurance, and what it
does not claim. The chain is:

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

### 6.1 Activation rules · `rule:identities:activation`

- No identity below is minted before a real consumer exists. An activation
  phase is permission, not a schedule: reaching the phase without the named
  consumer does not activate the identity.
- A parent binds only its immediate identity dependencies, under
  (`rule:identity:immediate-edges`). Transitive upstream identities
  are never repeated as an all-to-all mesh. A human-readable manifest may
  display the complete chain; authoritative validation follows immediate typed
  edges only.
- A child receives its own identity only when it is separately consumed,
  transported, cached, signed, versioned, or published. Otherwise the parent
  includes the canonical typed child value directly.
- No canonical projection contains a Petgraph index, source order, path, line
  number, solver variable number, matrix position, traversal order, thread
  schedule, temporary path, or floating working value.
- Every identity carries an explicit recipe identifier from its first
  publication, and any later change of projection, encoding, domain separator,
  algorithm, included fields, or exclusion rules mints a new recipe identifier
  under (`rule:identity:recipe-permanence`) rather than redefining the
  old one.

### 6.2 Edges

**ArchitectureSemanticId** — the chain root, and the one link that is already
active; see 2.5 above. It binds no upstream identity.

**RealizationId**

- **Activation phase:** not Phase 2. Activated only when a validated scoped
  realization crosses a process, cache, or publication boundary — that is, when
  a consumer receives realization as external bytes rather than as an
  in-process typed value.
- **Immediate consumer:** the compiler, and only in that transported form.
- **Binds:** the architecture semantic identity, plus the canonical projection
  of the scoped realization.
- **Assurance:** canonical-projection equality. The scoped realization is
  already ownership-validated at derivation, so the identity compares meaning
  that validation has already accepted.
- **Non-claims:** not architecture identity, not semantic correctness, and no
  target meaning whatsoever.
- **Child identities:** none. Per-operation realizations have no independent
  lifecycle, so the parent carries their canonical typed values directly.

**CompilerPlanId**

- **Activation phase:** Phase 2 at the earliest, and only on a real boundary —
  an analysis result cached across processes, published as an artifact, or
  consumed by a separately versioned backend.
- **Immediate consumer:** the target or backend package.
- **Binds:** the realization identity where one exists, otherwise the
  architecture semantic identity together with the typed realization value, and
  the compiler configuration identity.
- **Assurance:** canonical analysis-projection equality.
- **Non-claims:** it does not attest that the analysis is correct, does not
  identify a linked bundle, and carries no target-specific detail.
- **Child identities:** none while the relation DAG, proof plan, disclosure and
  source analysis, lifecycle analysis, placement and layout requirements, and
  coverage requirements are consumed only as parts of one plan.

**TargetPlanId**

- **Activation phase:** Phase 3, when a target-specific lowering plan is
  transported or cached separately from the compiler plan.
- **Immediate consumer:** the backend, and the linker through it.
- **Binds:** the compiler plan identity, the target-definition identity, and
  the backend configuration identity.
- **Non-claims:** no deployment-instance meaning and no bytes.

**LinkedBundleId**

- **Activation phase:** the linker phase, when linked bundles are distributed,
  cached, or referenced by an ABI.
- **Immediate consumer:** the transaction ABI, the deployment profile, and the
  release manifest.
- **Binds:** the target plan identity, and the relocatable bundle identity
  where the linker consumes relocatable bundles separately.
- **Assurance:** this is where semantic identity and artifact digest meet. The
  distributed bytes additionally carry an artifact digest with role, canonical
  path, schema, algorithm, and byte digest; the two are separate entries and
  neither substitutes for the other.
- **Non-claims:** not deployment readiness, and not calibration.

**TransactionAbiId**

- **Activation phase:** the transaction phase, when external construction
  consumes the ABI.
- **Immediate consumer:** external constructors, the deployment profile, and
  the release manifest.
- **Binds:** the linked bundle identity.
- **Non-claims:** not semantic realization. An ABI is a construction contract,
  not a meaning.

**DeploymentProfileId**

- **Activation phase:** the release phase. The value already exists and is
  dormant; activation is additionally blocked because schema 2 cannot bind a
  bundle or an ABI, which the profile migration task must repair first.
- **Immediate consumer:** the release manifest.
- **Binds:** the linked bundle identity, the transaction ABI identity, and
  typed evidence references — not raw report digests.
- **Non-claims:** not release identity, and not evidence of independence.

**ReleaseManifestId**

- **Activation phase:** the release package, as the sole aggregate release
  root.
- **Immediate consumer:** release distribution and, if signing is introduced,
  the signer.
- **Binds:** the deployment profile identity, required evidence references,
  distributed artifact rows with their byte digests, release policy, the
  explicit source revision, and the explicit release date.
- **Assurance:** if release signing is introduced, this identity is the signing
  root. Internal fields and intermediate objects are not signed separately
  unless they carry an independent operational authority boundary.
- **Non-claims:** not protocol denotation. Two matching manifests do not
  demonstrate independent implementation, evidence independence, or
  correctness.

### 6.3 Phase-2 consequence · `rule:identities:phase2`

Phase 2 proceeds without minting a public realization or compiler identity. No
speculative hash field enters compiler core: where no persistent cross-process
consumer exists, typed comparison remains the boundary, and a field reserved
for a future digest is itself a speculative identity.

### 6.4 Ownership boundary · `tab:identities:boundary`

Ownership and boundary for every unminted identity, including those outside the
chain above.

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
