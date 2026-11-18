# Identity and Hash Register · `reg:identities:ownership`

This register records adjudicated identities and digests, names known census gaps,
and fixes the ownership boundary of identities that do not yet exist. The
immediate policy owner is the identity adjudication procedure in
[ADR-021](../../adr/021-identity-adjudication.md). This file
is a planning aid, not a substitute for typed identity definitions, and is
never toolchain input.

Admission of any new digest is governed by (`[ADR021-req:identity:admission-record]`). The entries below record walks through the adopted procedure in DI-004, the keyed capture freeze admission, and the fixed-form admission completed at S5. Existing digest proposals were read from their owning code and assessed at the applicable nodes; S5 supplies the typed validation and encoding warrants left open by the prospective fixed-form walk. Outcomes distinguish admission, decided no-identity stops and pre-admission work whose warrants remain owed. The classification is per identity class — one recipe, one role, one consumer-decision pattern — and never per value.

## 1. Census · `tab:identities:census`

Adjudicated digest and identity classes, one row each; known gaps follow the table. An em dash in the class column means no identity is admitted — a deliberate absence, not an omission; the outcome and its record distinguish a decided no-identity stop from pre-admission work.

| Identity | Class | Outcome |
|---|---|---|
| Git commit and tree object IDs | provenance | admitted, 2.1 |
| Document identity | provenance | admitted, 2.2 |
| Paper instance identity | provenance | admitted, 2.3 |
| anchor-set hash | semantic | admitted, 2.4 |
| Architecture semantic hash | semantic | admitted, 2.5 |
| Architecture behavioural hash | semantic | admitted, 2.6 |
| Keyed capture freeze | provenance | admitted, (`rec:identities:keyed-capture-freeze`) |
| Fixed-form keyed capture freeze | provenance | admitted, S8 capture/native/corpus consumers active, (`rec:identities:keyed-fixed-capture-stop`), (`rec:identities:keyed-fixed-s8-migration`) |
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

Eight admitted classes, nine no-identity stops and two pre-admission records. Known gaps remain for the schema-2 keyed declaration binding, confidential fixture transcript digest, operator-right candidate digest, capture-manifest and archive/census/retention/provenance digests, run content addresses, first-party subject/program byte bindings and derived test-material values; their producers and owed walks are recorded at (`tab:backlog:wave1-followups`). These computations have no matching admission or stop here; this search is not a completeness proof, and the pending walks decide which require identities or belong under existing stops.

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

Eight identity classes are admitted. Each records its subject, owner, producer,
consumer, decision, assurance class, stale condition, recipe by identifier,
migration rule, non-claims, and status — the admission fields the adopted
procedure requires, in the order
(`[ADR021-req:identity:admission-record]`) states them.

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
  Announcement-bound completion (2026-09-13): both architecture values move
  with the body — two calibrated bound rows and the announcement's bound
  references — under the unchanged recipes. The realized window check in
  `packages/realization/src/state.rs` and the model's constants already required
  these bounds. The finite presentation is completed; denotation does not move
  and none of the versioning decision procedure's verdicts fires. The deciding
  witness is `architecture_keyed_announcement_leads_equal_model_constants` in
  `packages/model/src/tests/bound_conformance_tests.rs`.
  The data census also names the cycle and maturity fields already used by the
  realized law; the derived announcement-window quantity and mirrored operation
  reads complete the presentation without a decision-procedure verdict firing.
- **Value:** the `semantic_hash` field of
  `packages/model/generated/architecture.json` at this landing, mirrored in
  the realization document's masthead.
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
- **Decision:** whether a change moved the denotation. A changed measurement
  requires a denotation record distinguishing a completed finite presentation
  from a changed law.
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
  Announcement-bound completion (2026-09-13): both architecture values move
  with the body — two calibrated bound rows and the announcement's bound
  references — under the unchanged recipes. The realized window check in
  `packages/realization/src/state.rs` and the model's constants already required
  these bounds. The finite presentation is completed; denotation does not move
  and none of the versioning decision procedure's verdicts fires. The deciding
  witness is `architecture_keyed_announcement_leads_equal_model_constants` in
  `packages/model/src/tests/bound_conformance_tests.rs`.
  The data census also names the cycle and maturity fields already used by the
  realized law; the derived announcement-window quantity and mirrored operation
  reads complete the presentation without a decision-procedure verdict firing.
- **Value:** the `behavioural_hash` field of
  `packages/model/generated/architecture.json` at this landing, mirrored in
  the realization document's masthead and pinned in
  `packages/architecture/src/tests/versioning_gate_tests.rs`.
- **Non-claims:** not the complete publication bytes, and not a second general
  architecture identity. It must not be repeated through future artifacts as
  though it identified the architecture.
- **Status:** active, as a narrow versioning witness only.

### 2.7 Keyed capture freeze · `rec:identities:keyed-capture-freeze`

- **Subject:** the complete validated selected-source `FreezeManifest` and structural `Envelope` in `packages/vectors/src/keyed_final_manifest.rs`: twelve categories of ordered source rows, each relative path and full-file SHA-256; the ordered computed inventory; key and predicted leaf digest; recipe schema; target version; and all five envelope fields. The Dependencies category binds whole `Cargo.toml` and `packages/transaction/Cargo.toml` bytes, including comments and formatting, not a projection of typed requirements or roles and not the resolved lock.
- **Owner:** vectors; the manifest types and recipe grammar are owned in `packages/vectors/src/keyed_final_manifest.rs` and `packages/vectors/src/keyed_final_manifest/encoding.rs`.
- **Producer:** `capture_freeze_digest` in `packages/vectors/src/keyed_final_native.rs` validates source bytes and every offered field before hashing. `retained_capture_freeze_digest` validates the current offer first and then renders `retained_capture_offer`; historical pins are established separately by the history checks and sealed corpus, not validated against current source bytes.
- **Consumer:** capture provenance and rows cross the capture-process and retained-archive boundary. `bind_capture_provenance` refuses an absent freeze as `Missing(Freeze)` and an unequal freeze as `Freeze`; `bind_capture_manifest` in `packages/vectors/src/keyed_final_capture.rs` requires each row's current per-key freeze; the joins in `packages/vectors/src/keyed_final_corpus.rs` require the retained freeze and refuse current-freeze substitution for that session.
- **Decision:** accept the expected current or retained per-key freeze and refuse missing or unequal bindings. Review does not bind what arrives later, digest comparison is not direct comparison of the source manifest, and the archive digest binds its claim without establishing equality to the expected sources. Neither no-identity branch of (`[ADR021-alg:identity:adjudication]`) applies.
- **Class:** provenance over the selected input set, locator-grade and not release-bound.
- **Assurance:** the explicit grammar and specified unsigned decimal primitives make encoding deterministic across toolchains and platforms. Exhaustive struct destructuring and enum matches cover every field and variant; full-file pins cover the selected bytes. Source paths, source formatting and stored vector order are recipe-defined provenance content. Category order is fixed by explicit ranks, independent of the enum's derived order. Distinct names, fixed field order, delimited collections and quoted alphabet-only strings make the encoding injective: equal encodings imply equal manifests. The prefix separates the role and identifies the recipe; SHA-256 supplies collision and second-preimage resistance under its usual assumptions at both source-pin and aggregate layers. Every consumer can recompute from the owned encoder and source/history inputs. `encoding/tests.rs` pins primitive and alphabet grammar, varies every field and stored sequence order, and checks an accepted control against eleven one-scalar inventory refusals and a complete literal mutant encoding. `tests/keyed_final/freeze_history.rs::check_encoding_history` pins both keys across four reviewed snapshots. The sealed corpus anchors the capture marker; the two earlier snapshots are reconstructions with no contemporaneous artifact. Comparisons with the former renderer are regression aids with explicit drift diagnostics, never the definition. These warrant the provenance column of (`[ADR021-tab:identity:class-properties]`).
- **Stale condition:** any inventoried source-byte change, including an editorial manifest edit, invalidates its current pin; the paired re-pin changes the current freeze. Inventory membership/order, computed inventory, key/leaf, schema, target and envelope are all inputs; a new key has its own value without reopening unrelated observations.
- **Recipe:** `keyed-capture-freeze-v1` is SHA-256 of `FreezeManifest::capture_encoding`'s UTF-8 bytes, beginning with the literal domain `keyed-capture-freeze-v1:` and ending without a newline. The owned grammar fixes literal struct/field/enum names, exhaustive field order, category ranks, ordered vectors and byte arrays, punctuation and unsigned decimal primitives. Strings contain zero or more characters from ``[A-Za-z0-9/._-]``, copied unchanged between double quotes without escaping or normalization; any other scalar returns `FreezeRefusal::FrozenInventory`, agreeing with validation's inventory refusal. Rendering is not full validation. No field is omitted and no diagnostic formatter or Unicode property table participates. Under (`[ADR021-rule:identity:recipe-permanence]`), the identifier is preserved: inspection establishes the alphabet and integer agreement, while the sealed and pinned digests and independent reconstruction establish the former structural layout for the compiler that emitted the published values; derived Debug layout is not a language stability contract. Every emitted value used paths in this alphabet, including current pins, retained pins and replayed history. Thus no published value or verification outcome changes. The historical producer is `rustc 1.102.0-nightly` dated 2026-11-07; its exact compiler revision is recorded in the sealed session provenance under `packages/vectors/fixtures/keyed-final-native/`, the same revision that `lint/hash-citation-families.tsv` carries for the engine-base capture as `engine-base-capture-rustc`. The capture compiler records who produced those bytes; it does not define the owned grammar.
- **Migration:** Under (`rule:vectors:keyed-refreeze`) and (`tab:vectors:dependency-refreeze`), retained captures keep each path's first post-capture previous digest in `RETAINED_CAPTURE`; later re-pins never substitute an intermediate digest. Current-source validation still precedes historical reconstruction. History replay checks continuity and the retained mapping; the sealed corpus fixes the capture marker and rejects current-freeze substitution. This migration handles digest replacements only; changed membership needs an explicit retained-reconstruction disposition, not a pin-only repair. No consumer transition or capture rebinding accompanies the byte-preserving encoder. A change of projection, encoding (including the path alphabet), primitive or domain requires a distinct recipe and explicit consumer transition under (`[ADR021-rule:identity:recipe-permanence]`). The distinct fixed-form capture-freeze recipe belongs to S5's admission with S8's consumer migration under ruling 211 at (`rule:phase7:fixed-form-rulings`); S2 specifies the realization and records pre-admission.
- **Non-claims:** matching values establish no source correctness, executable/toolchain/resolved-dependency identity, authenticity, independent implementation, native acceptance, semantic equivalence or deployment readiness. Included bytes are compile-time source provenance; a moved checkout must rebuild vectors. Admission permits dependent re-pins only with their own source/pin/history controls and affected-evidence disposition under (`rule:vectors:keyed-refreeze`).
- **Status:** active.

## 3. Stop records · `sec:identities:stops`

A stop is an outcome, not an omission. This section also houses pre-admission records, which defer admission rather than decide that no identity is needed. Each record states the proposal, the branch reached, the date of the walk and the condition for revisiting it. Until that condition holds, its recorded outcome stands and the proposal is not re-adjudicated from nothing. The first eleven walks were taken 2026-08-18, in DI-004, over a tree read from the owning code; where a standing policy already refused the digest, the record fixes that refusal in the procedure's own terms rather than restating it. The fixed-form walk records its own date below.

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

### 3.12 Fixed-form keyed capture freeze · `rec:identities:keyed-fixed-capture-stop`

S5 completed the 2026-11-16 pre-admission walk under (`[ADR021-alg:identity:adjudication]`); the retained label preserves its citation identity. This is a separate provenance class whose S8 consumers are now active before capture.

- **Subject, owner and producer:** vectors owns FreezeManifest, the opaque ValidatedFreeze and its identity method in packages/vectors/src/keyed_fixed_manifest.rs. Validation checks included whole-file source bytes against 46 literal pins, both ordered inventories, the independently supplied triple, the fully reconstructed descriptor, schema, target and every envelope field. Only ValidatedFreeze publishes the aggregate identity.
- **Decision and benefit:** S8's fixed-form capture/native consumers must accept only an expected current per-triple freeze, refusing absent, unequal and cross-edition values. Its retained-corpus consumer must require the capture-time freeze. The exact owners are packages/vectors/src/keyed_fixed_capture.rs, keyed_fixed_native.rs and keyed_fixed_corpus.rs, allocated at (`tab:backlog:fixed-form-paths`). Review cannot bind later process/archive arrivals; an archive hash binds a claim without comparing it to the expected inputs. Neither no-identity branch supplies that comparison.
- **Recipe and class:** `tripod-keyed-fixed-capture-freeze-v1`, provenance, locator-grade. SHA-256 covers exactly the ordered binary grammar below, owned by packages/vectors/src/keyed_fixed_manifest/encoding.rs. No diagnostic formatter participates. A frame means u32-BE byte length followed by exactly those bytes; there is no padding or separator. Boolean bytes are 00/01. Source paths are ASCII [A-Za-z0-9/._-] only, with no normalization or escaping. Counts and lengths outside u32 refuse.

| Order | Encoded fields, in order |
|---|---|
| 1 | ASCII `tripod-keyed-fixed-capture-freeze-v1`, then one 00 byte |
| 2 | Source count u32-BE; for each source in stored order: framed path, raw SHA-256 digest32. Validation requires the exact current inventory in path order: 46 entries in S5 and the S8 inventory recorded below. |
| 3 | Computed count u32-BE; each explicit code below as u16-BE in stored order. Validation requires codes 1 through 11 in S5 and codes 1 through 15 in S8. |
| 4 | K32, A32, v as u64-BE (distinct from transcript LE64), unframed transcript42, pair digest32 |
| 5 | Framed leaf, leaf version u8, internal x32, root32, tweak32, output x32, parity u8 (00 even, 01 odd), framed output program, framed control block |
| 6 | Recipe schema u32-BE, target version u32-BE; current values 1 and 2 |
| 7 | Maximum-members option: None is one 00 byte with no following value; Some is 01 followed by u64-BE |
| 8 | Five u64-BE counts: script bytes, script-witness bytes, input-witness bytes, control depth, argument items |
| 9 | Four boolean bytes: annex, issuance, peg-in, empty nonce |

| Computed code | Name | Value coverage |
|---|---|---|
| 1 | Triple | K, A, v in descriptor |
| 2 | Transcript | Descriptor transcript |
| 3 | PairDigest | Descriptor pair digest |
| 4 | Leaf | Descriptor leaf |
| 5 | Root | Descriptor root |
| 6 | Tweak | Descriptor tweak |
| 7 | Program | Descriptor output program |
| 8 | Control | Descriptor control block |
| 9 | EmissionAndClosure | Name only; emission and linker occurrence census are recomputed, not encoded values |
| 10 | MemberCoverage | Name only; member coverage is recomputed, not an encoded value |
| 11 | DerivedResources | Name only; the listed envelope counts are encoded separately, but peak and element width are recomputed, not encoded values |
| 12 | TransactionShape | Name only; values are bound through the pinned construction and byte producers |
| 13 | CaptureSubjectCensus | Name only; values are bound through the pinned capture producer |
| 14 | ReaderRecurrence | Name only; values are bound through the pinned reader producer |
| 15 | MemberMaximum | Name only; the admitted maximum is encoded separately and derived from the pinned constructor |

The code list is a census, not framed value records. Codes 1–8 carry their values through the descriptor; codes 9–15 name recomputed results whose values are bound through pinned producers rather than encoded by the code list. The selected-source inventory excludes generic program/push encoders and the pin table itself; the descriptor binds their resulting values, not an asserted transitive source closure.

| Provenance property | Warrant |
|---|---|
| Deterministic | Fixed field order, widths, byte order and explicit enum codes; exhaustive struct destructuring and enum matching. |
| Complete over named inputs | Explicit 46-source S5 inventory and the enlarged S8 inventory recorded below, including named construction, capture, reader and shared inputs. The resolved lock and transitive source closure are not claimed. Pin-table contents are the encoded inventory, avoiding a self-hash cycle. Triple, transcript/hash, leaf/version, internal key, root/tweak, output key/parity/program/control and every structural envelope field are included. |
| Free of incidentals | Relative paths, whole-file formatting and stored source/computed order are recipe content. Clock, machine, absolute checkout path, build mode and process state are excluded. |
| Domain-separated | Distinct zero-terminated recipe identifier, separate from legacy capture and schema-3 semantic identity. The history test compares both editions. |
| Recipe-identified | Identifier, projection, primitive and complete grammar are owned together. |
| Collision/second-preimage resistance | Conditional SHA-256 assumptions at source-pin and aggregate layers. Fixed widths and length framing are unambiguous. Strings outside [A-Za-z0-9/._-] refuse with Alphabet; no escaping or normalization. |
| Recomputable | Included source bytes and literal pins, independent grammar assembly in packages/vectors/tests/keyed_fixed_freeze_history.rs, field/framing/alphabet controls in the encoder child tests. The S5 and S8 snapshots and a synthetic stale-source subject are tested; no fixed-form capture is yet claimed. |

- **Migration and deadline:** activate the named three consumers, including missing/unequal/cross-edition controls, before the first fixed-form capture at S8. In the same reviewed pre-capture landing add S6 construction/commitment and later capture-used sources, computed values and admitted maximum, with pins/history and affected-evidence dispositions under (`rule:vectors:keyed-refreeze`). The present None maximum means unadmitted, never unlimited. Retained rows keep capture-time inputs and recipe; current pins never replace them.
- **Staleness:** any source bytes, inventory/order, triple, descriptor, schema, target or envelope change stales the current value. Source changes require paired pins/history. Projection, framing, alphabet, domain or primitive changes require a distinct recipe and explicit consumer transition. Legacy frozen sources, pins, history and captures are unchanged. The shared model helper outside that legacy inventory has only the three documented error-contract lines added; the first fixed-form freeze includes that documented source without a historical re-pin.
- **Non-claims of the S5 snapshot:** no authenticity, source correctness, executable/toolchain/lock identity, native acceptance, target amount or policy admission, deployment readiness or mathematical injectivity. Cryptographic premises remain conditional at (`[DOC-sec:keyed-fixed:assumptions]`). S5 gates and independent review remain required before import.

**S8 pre-capture migration** · `rec:identities:keyed-fixed-s8-migration`. The recipe, SHA-256 primitive, path alphabet, framing and descriptor projection remain those admitted at (`rec:identities:keyed-fixed-capture-stop`). The working inventory grows from 46 to 98 whole-file paths: 52 additions for S6 construction and shared byte/signing owners, S7 reader/model/evidence, the transaction manifest and amount-form source, conformance proof/protocol/executor and direct module dependencies, the unchanged adapter, new wrappers and capture owners including both integration-test sources. Three changed S5 paths are packages/vectors/Cargo.toml (existing Elements reference dependency moved into the library), packages/vectors/src/keyed_fixed_manifest.rs (behavioral inventory/envelope admission) and packages/vectors/src/keyed_fixed_manifest/encoding.rs (additive computed-code encoding). Every addition and all previous/current changed digests have 55 literal rows in packages/vectors/tests/keyed_fixed_freeze_history.rs; the unchanged 46-source S5 mapping and its 4249-byte identity remain reconstructed there. Four new computed codes name transaction shape, capture subject census, reader recurrence and member maximum, without claiming that codes alone freeze their full results. The envelope admits `Some(1299)` only for ruling 214's three-output, one-sponsor construction policy; `None` remains the historical unadmitted state. For the test triple K=zero, A=0x23 repeated, v=1, the independent 8853-byte encoding yields `6591fbb37cfc1fbcf1ffe14a5212a14bc919416f937dc51d314818070028c063`. This is a source-identity and inventory transition, not a new recipe.

**S8 post-pilot correction.** The sealed pilot retains its 98-source, 8853-byte capture-time identity `02abd95456364268db586511be03b32cc864f833753cda1d4a643921430d3b12`; the current identity above follows three changed pins in the capture owner, its journal test and its native child. Three `POST_PILOT` paired rows reconstruct the pilot mapping and its independent encoding alongside the 55 S5-to-current rows. The correction strengthens control and preparation joins and changes the zero-fee probe's submitted outputs. The pilot's actual node observations remain attached to its retained bytes and identity; they do not credit the later campaign, which requires a new current capture. The first-edition mapping and files remain byte-identical.

**S8 pilot-retention correction.** The intermediate post-pilot identity `24660fb26a6da460107ddd3b795746fb99a353e8ee9cdaf585dcd6fbca1030a5` is reconstructed from two `POST_S8A2` paired rows before the three `POST_PILOT` rows reconstruct the pilot. The current identity above follows the changed native capture child and capture-unit wrapper: observed runtime digests, the adapter handshake and microsecond timing are sealed, while the wrapper copies node logs and verifies cgroup inclusion from retained membership. The current inventory remains 98 whole-file paths and the encoding remains 8853 bytes. Neither the sealed pilot nor any earlier first-edition evidence acquires the new identity; there is no S8a2 native campaign packet to relabel.

The source diff adds no change to the first edition's frozen source, pin, script or archive bytes. S5 had no fixed-form native archive, so no old fixed-form observation is relabelled; the new capture must bind its own current per-triple identity before any submission. The native, capture and prospective corpus consumers refuse absent, unequal and old-edition identities beside matched controls; retained packets preserve their capture-time value for S8b history replay rather than acquiring current pins. The admitted property remains completeness over the *named* whole-file inventory and descriptor/envelope inputs, not the resolved lock, transitive workspace or executable. The capture binary, resolved lock, daemon, framework, zk library, image and separately recorded corpus-consumer source remain runtime/archive provenance, outside the freeze to avoid a self-identity cycle. An altered pin, source, included inventory or envelope reopens source identity and affected equivalence evidence; a changed submitted byte or invalid premise reopens the corresponding native observation, never repaired by a re-pin.
- **Non-claims of the S8 migration:** no authenticity, source correctness, executable/toolchain/lock identity, native acceptance, target policy or consensus verdict, deployment readiness or mathematical injectivity. Cryptographic premises remain conditional at (`[DOC-sec:keyed-fixed:assumptions]`). The operator capture and independent import review remain required.

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
projection — see (`[ADR021-rule:identity:no-incidentals]`).

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
  (`[ADR021-rule:identity:immediate-edges]`). Transitive upstream identities
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
  under (`[ADR021-rule:identity:recipe-permanence]`) rather than redefining the
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
