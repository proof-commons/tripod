# Draft-versus-implementation gap census

A clause-by-clause census of the four externally authored texts now carried
normatively by ADR-019 through ADR-022, against the incumbent decision records
and the label checker as it was written when this dated reading was made. Every
disposition below was read out of the code, not out of a module doc comment;
where the code did not settle a question, the row says so.

## How to read this census · `sec:gapcensus:method`

Dispositions are: **implemented**, the repository already does what the
clause says and the module doing it is named; **divergent**, the
repository does something here but not what the clause says;
**absent**, nothing answers the clause; **undetermined**, the code did
not settle it, with the reason given.

The cited clauses now belong to their numbered ADR owners, so this PLAN-owned
census names each by an imported ADR citation. The rows remain historical
evidence of the reading that justified adoption.

## Label calculus · `sec:gapcensus:calculus`

The label-calculus text was the generic restatement of ADR-013, which was itself
marked decided and implemented. Most clauses are therefore satisfied by
construction; the deltas are exactly the places where the draft
generalizes a repository-specific mechanism into an adoption parameter.

This reading is dated: the calculus's fourth edition is now the normative body of [ADR-019](../../adr/019-label-calculus.md), and it moves the clauses two of the rows below turn on. A generated register no longer stands outside the graph — participation holds of it in full, its occurrences are mints and citations like any others, and the exclusion that once sat in (`[ADR019-judg:labels:participation]`) now sits at the harvest instead, under the edition's new (`[ADR019-inv:labels:generated-compliance]`): what a register presents, it never feeds. Two dispositions are inverted by that move rather than merely reworded. The minting-and-resolution row reads the census exclusion of the registers as satisfying its clause, and the Ansätze row reads their removal from discovery as the refusal of participating registers; under the fourth edition each is a divergence, because the checker buys the no-feeding half by dropping the registers from discovery altogether and forfeits the participation the text now requires in the same stroke. That standing is recorded, not claimed away: ADR-019's status line and its implementation-standing entry both carry it, and closing it is a checker change rather than an amendment. The rows are kept as the reading that justified adoption.

| Clause | Disposition | Evidence |
|---|---|---|
| (`[ADR019-lang:labels:label-language]`) | divergent | `Label::parse` in `packages/labels/src/label.rs` admits hyphens in every segment, so kind and area range over a wider alphabet than the words the clause fixes. |
| (`[ADR019-gram:labels:well-formed]`) | implemented | `classify` in `packages/labels/src/markdown.rs` parses the immediate syntactic group; a non-label-shaped span falls out of `harvest_markdown_owner` unparsed. |
| (`[ADR019-sig:labels:owners]`) | divergent | The signature is a hardcoded chain in `ImportedLabel::parse`, `packages/labels/src/owner.rs`, admitting six prefix families. `LabelOwner::prefix` emits a prefix for `Crate` owners that `parse` cannot read back, so per-package owners are unciteable across owners. |
| (`[ADR019-judg:labels:minting]`), (`[ADR019-judg:labels:resolution]`) | implemented | `RepositoryCensus::discover` in `census.rs` excludes the generated registers; resolution exists only as an edge built by `build_label_graph` in `repository.rs`. |
| (`[ADR019-judg:labels:participation]`), prose | implemented | Fences skipped in `scan_markdown`; double-backtick spans skipped by the `delimiter_len` guard in four harvest sites of `repository.rs`. |
| (`[ADR019-judg:labels:participation]`), code | implemented | `comment_segments` in `rust_source.rs` scans comments only, skipping plain, raw, byte, C and character literals; fenced documentation examples are skipped in `harvest_file`. |
| acute classifies locally | implemented | `acute_scan` in `participation.rs` opens an acute only where `label_shaped_text` follows; `harvest_region` in `rust_source.rs` raises `UnclosedInlineCode` for an opener whose region ends first, and an acute that opens nothing stays text. |
| (`[ADR019-inf:labels:mint]`) | implemented for the authorship species only | Bare spans mint into the owner's own registry in `harvest_markdown_owner` and `harvest_label`. The rule is shared by both warrant species; the checker discharges only the authorship premise, and no code path discharges a derivation, so the shared rule is half-realized. |
| (`[ADR019-inf:labels:same-owner-citation]`) | implemented | `mint_nodes` in `build_label_graph` is keyed on the owner-and-label pair, so a same-owner citation resolves across that owner's files and nowhere else. |
| (`[ADR019-inf:labels:imported-citation]`) | implemented | `import` in `repository.rs`; unknown prefixes raise `UnknownOwner`, and `diagnose_bracket_free_owner_token` fails a cross-owner token written without brackets. |
| self-qualified import refused | implemented | Checked twice: eagerly for the realization owner in `harvest_realization_import`, and universally in `build_label_graph`, which drops the citation and raises `InvalidImportedCitationForm`. |
| (`[ADR019-inf:labels:synthetic-citation]`) | divergent | `add_architecture_citations` in `repository.rs` designates exactly two typed-data classes, both hardcoded: architecture witness semantic tags and clause identifiers, both targeting the realization owner. There is no registry of designated classes and no way to designate a third. |
| (`[ADR019-inf:labels:anchor-harvest]`) | divergent | `harvest_attestation_citations` implements one hardwired instance: the realization document as index and Attestation as upstream, with the index region opened by a literal heading match on the anchors mint. The clause's general index and upstream pair is not parameterized. |
| (`[ADR019-inv:labels:unique-mint]`) | implemented | `LabelRegistry::insert_or_diagnose` in `registry.rs` reports the duplicate site and the first mint site. One weaker path, `report_duplicate_graph_node`, reports a single location for graph-node collisions. |
| (`[ADR019-inv:labels:total-resolution]`) | implemented | `report_unresolved_citation` in `repository.rs` errors per class, and `validate_citation_outdegrees` rejects a citation resolving to more than one mint. |
| (`[ADR019-inv:labels:two-pass]`) | implemented | `harvest_sources` fills every registry and queues citations; `validate` drains the queue afterwards. |
| (`[ADR019-metathm:labels:order-independence]`), (`[ADR019-metathm:labels:no-self-support]`) | implemented | The first follows from the staging above. For the second, index lines are excluded from the body scan and only body tokens enter the anchor set; `validate_attestation_anchor_pin` recomputes the pin over body names alone. |
| (`[ADR019-metathm:labels:presentation-invariance]`) | undetermined | A meta-theorem about migrations, not a checkable property; no code asserts or could assert it. |
| (`[ADR019-cav:labels:non-normativity]`), (`[ADR019-cav:labels:coexistence]`) | implemented | No compiler, linker or release path reads the registries; registries are per owner, and `Walk::unreadable` in `census.rs` records `CensusUnreadable` rather than yielding an empty group. |
| the five rejected Ansätze, (`[ADR019-ansatz:labels:flat-namespace]`) through (`[ADR019-ansatz:labels:participating-registers]`) | implemented | All five stay refused: resolution is owner-keyed; a second mint errors; only acute spans in comments are scanned in code; planning sources resolve under the same total rule; and the registers are dropped from discovery while `check::current` still compares their bytes. |

### Gate checklist · `sec:gapcensus:calculus-gate`

Each item of (`[ADR019-gate:labels:implementation]`) against the current
checker. The normative text ends in a Gate that blocks implementation until met,
not in a postcondition discharged after the fact, so an unmet item
below is a block on adoption rather than a defect of work already done.

| Gate item | Status |
|---|---|
| three forms parse in both syntaxes | holds |
| duplicate mints fail with both locations | holds for registry mints; the graph-node path reports one |
| every same-owner and imported citation resolves to one mint | holds |
| bracket-free cross-owner tokens and self-qualified imports fail | holds, but the import form is not suggested when the label mints in another owner |
| resolution independent of traversal order | holds |
| designated typed-data strings resolve synthetically | holds for the two hardcoded classes only |
| every generated occurrence a unique mint or a resolving citation, generated mints standing on their warrant, no generated region feeding what it presents, registers current and deterministic | partial under the fourth edition: no-feeding, currency and determinism hold; the two occurrence clauses do not, the registers being dropped from discovery rather than harvested |
| pinned anchor-set hash from body citations only | holds |
| traversal failures surface as diagnostics | holds |
| scoped generation ignores unrelated owners' defects | holds, in `derive_register_sources` and `derive_model_sources` |
| every mint on exactly one warrant, one warrant species per kind, reserved kinds without a profile fail | absent: the checker has no notion of a warrant, and adopts neither a profile signature nor a reserved-kind set |
| covered assets carry their derived label at the standard place, no inventory label outliving its asset | absent, for the same reason |
| near-miss spans warned without being treated as occurrences | absent: a label-shaped span that parses as no form is silently text |
| gate dischargeable from this document and the adoption data alone | holds |
| one mint per environment, every citation resolves | held for the four source texts as committed: the gate was green |
| corpus-wide check passes in CI | holds |

Two real gaps here. The narrowness of the synthetic-citation and
anchor-harvest mechanisms is the older one: each is a correct instance
of its rule, neither is the rule. The larger one is the derivation
authority, which the third edition adds whole and the checker does not
have at all.

## Environment kinds · `sec:gapcensus:kinds`

The registry is descriptive and binds nothing until adopted, so no
clause here is implemented in code: `Label::parse` validates a kind
vocabulary only for the realization owner, and every other prose owner
accepts any word as a kind. The census below is therefore of the
repository's kind usage against the registry's assignments, harvested
from every mint in `plans/`, `adr/`, `docs/` and `packages/`.

This reading is dated in the same way: the registry's fourth edition is now the normative body of [ADR-020](../../adr/020-environment-kinds.md), adding a fifteenth Convention over works and publication and extending the records Convention with the deliberative rows — twenty rows and twelve kind tokens, none of which collides with a kind this repository mints. No disposition below is inverted by it, and the counts quoted in this section are the third edition's, as read.

Forty-eight of the sixty-one kinds in current use are assigned
identically by the third-edition registry, among them `sec`, `rule`,
`rem`, `inv`, `def`, `dec`, `lem`, `inf`, `judg`, `cav`, `metathm`,
`ansatz`, `conv`, `thm`, `case`, `tab`, `gate`, `crit`, `red`, `moral`,
`postc`, and — newly catalogued in this edition — `req`, `pkg` and
`test`. Two label migrations retired the
repository's divergent spellings: `subsec`, `tbl`, `lst`, `protocol`,
`post` and `ins` now occur nowhere in the tree, so the rows that once
recorded them are struck. The remaining thirteen part:

| Repository kind | Registry position | Disposition |
|---|---|---|
| `task` | Task is classified `exer`, among exercises | divergent in meaning: the repository uses it for work items |
| `res` | Result is `result` | divergent if the repository token abbreviates Result |
| `err` | Erratum is `errat`; an error-vocabulary genre is uncatalogued | absent from the registry |
| `trap`, `pin`, `obl`, `candidate`, `milestone`, `phase`, `op`, `leaf`, `ref`, `branch` | no row | absent from the registry |

The hybrid rule (`[ADR020-inf:kinds:hybrid]`) has no applicability today: no
repository kind concatenates two catalogued kinds, and no compound
environment head is in use. What adoption still costs is a registry
extension of about a dozen tokens, and, for `task` and `res`, either a
rename or a recorded local extension. The realization and model kind
lists remain compiled into `label.rs` and `rust_source.rs`, so any
further rename is a code change and a register regeneration in one
commit.

The second edition's internal defect — a classification judgment
(`[ADR020-judg:kinds:classification]`) claiming eight Conventions against a
document carrying eleven — is repaired: the third edition's preamble
claims fourteen and the document carries fourteen.

## Identity adjudication · `sec:gapcensus:identity`

ADR-016 already decides most of this draft, and the register
`plans/registers/identities.md` already records each identity in the
draft's admission shape. Deltas are marked where the draft asks for
something ADR-016 does not.

This reading is superseded: ADR-016 is deleted and the externally
authored text is adopted through
[ADR-021](../../adr/021-identity-adjudication.md). The rows below are kept as
the reading that justified the promotion; where one names ADR-016 as the
deciding text, ADR-021's normative body now decides and its Amendments section
supplies the local recipe convention.

| Clause | Disposition | Evidence |
|---|---|---|
| (`[ADR021-rule:identity:admission-order]`), (`[ADR021-myth:identity:hashes-validate]`) | implemented | ADR-016 states it, and the producers in `packages/architecture/src/canonical.rs` and `deployment.rs` hash validated exports only. |
| (`[ADR021-tab:identity:mechanisms]`), (`[ADR021-warn:identity:non-claims]`), (`[ADR021-req:identity:admission-record]`) | implemented | The mechanism table is reproduced as ADR-016's; every register entry carries subject, owner, producer, consumer, decision, assurance, stale condition, recipe, migration, non-claims and status. |
| (`[ADR021-crit:identity:benefit]`) | divergent | ADR-016 lists rejection grounds but not the benefit criterion as a test. The draft's reviewed-anyhow ground, that a standing review supersedes equality, has no counterpart in ADR-016. |
| (`[ADR021-alg:identity:adjudication]`) | absent | No decision walk exists as a required procedure; admission today is a checklist applied in review. |
| (`[ADR021-case:identity:no-identity]`), (`[ADR021-req:identity:stop-record]`) | implemented | The register carries eleven stop records, each with its proposal, deciding branch, date and revisit condition, beside six admission records. The dormant and provisional statuses remain, now as the standing of two of the stops rather than as the whole treatment of refusal. |
| (`[ADR021-def:identity:recipe]`), recipe identifiers | implemented | Recipe identifiers exist as algorithm constants for the semantic, behavioural, anchor-set and deployment hashes in `canonical.rs` and `deployment.rs`, with superseded recipes retained beside each. The anchor set gained `ANCHOR_SET_HASH_ALGORITHM` in the DI-004 separation migration; its identifier is a code-side register rather than a manifest field, which the migration record states. |
| (`[ADR021-rule:identity:recipe-permanence]`) | implemented | The superseded behavioural algorithm list in `canonical.rs` is a migration record, and the identifier is published beside every value. |
| (`[ADR021-tab:identity:properties]`), domain separation | implemented | Every first-party semantic identity prefixes a domain. The two that once did not were migrated together in DI-004 under (`[ADR021-rule:identity:separation-migration]`), which supersedes the grandfather clause; no exception remains for the property table to conflict with. |
| (`[ADR021-case:identity:artifact]`), freshness sub-branch | implemented | Generated publications are compared byte for byte in `check::current` and carry no digest. |
| (`[ADR021-rule:identity:no-incidentals]`), (`[ADR021-rule:identity:immediate-edges]`), (`[ADR021-red:identity:mesh-to-chain]`), (`[ADR021-red:identity:fields-to-object]`) | implemented | ADR-016 and the register forbid local handles in semantic identity, bind the future graph by immediate edges only, and refuse field-level hashing on the draft's own grounds. |
| (`[ADR021-case:identity:evidence]`), (`[ADR021-rule:identity:duties]`), (`[ADR021-rule:identity:delegation]`), (`[ADR021-case:identity:release]`) | absent | No evidence envelope, no release manifest and no release validator exist yet; ADR-016 marks these as activating with their consumers. |
| (`[ADR021-rule:identity:provenance-containment]`) | implemented | Publication-only provenance identities are contained by the register's status vocabulary. |
| (`[ADR021-gate:identity:implementation]`) | divergent | Every item ADR-016 already gates is met, and DI-004 discharged three more: every existing digest is classified, every no-identity outcome is recorded, and every admitted recipe carries an identifier delivering its class's properties. The benefit walk as a standing requirement on new proposals, and the release surfaces, are not. |

## Interchange conventions · `sec:gapcensus:interchange`

No implementation existed, and this was correct: nothing in the repository
encoded CBOR or CDDL, and the source text named no consumer. At the time of
this reading, the strings CBOR and CDDL occurred only in the source texts and
the backlog.

| Clause | Disposition | What adoption would demand |
|---|---|---|
| (`[ADR022-lang:interchange:data-language]`) | absent | A canonical encoder and a validating decoder for the deterministic encoding profile, with non-canonical input refused rather than repaired. |
| (`[ADR022-lang:interchange:description-language]`), (`[ADR022-schema:interchange:global]`) | absent | A description-language registry carrying the base theory, and a tool checking a document against an assigned theory. |
| (`[ADR022-gram:interchange:label-grammar]`) | absent | A namespace-label validator and an allocation record for whichever labels the repository claims. |
| (`[ADR022-sig:interchange:theory-assignment]`), (`[ADR022-inv:interchange:permanence]`), (`[ADR022-inv:interchange:patch-identity]`), (`[ADR022-inv:interchange:minor-inclusion]`), (`[ADR022-law:interchange:major-boundary]`) | absent | An owned registry with immutable assignments, a machine check of content-class inclusion across minors, and a version-bump gate wired to it. |
| (`[ADR022-def:interchange:acceptance]`), (`[ADR022-metathm:interchange:bounded-determination]`) | absent | Envelope-first dispatch with whole-document rejection; the bound then follows from the encoder. |

The one live wire surface, the native executor protocol in
`packages/target-elements-conformance/src/executor.rs` and its mock
peer, deliberately does not adopt these conventions and should stay as
it is: a line-delimited JSON pipe between two first-party processes in
one repository, with no archive, no third-party reader, no version
negotiation and no signature over its frames. Under the draft's own
benefit reasoning no consumer's decision would change, and ADR-010
already owns that shape.

## Findings register for DI-003 · `sec:gapcensus:findings`

Ranked checker-engineering deltas, each naming the module that would
change. The first three are the standing findings from backlog section
13.2, restated against what this reading found.

| Rank | Finding | Module |
|---|---|---|
| 1 | DI-F01, one participation scanner. Two independent parsers share only `classify`; fence tracking, delimiter pairing and literal skipping drift separately. | `markdown.rs`, `rust_source.rs` |
| 2 | DI-F02, owner signatures as data. The prefix map is a hardcoded chain, so a new owner is a code change, and per-package owners have a prefix they cannot be cited by. | `owner.rs` |
| 3 | DI-F03, import handling. Realization keeps a bespoke import path with its own fallback and self-import check, duplicating the universal one. | `repository.rs` |
| 4 | Synthetic citations are two hardwired architecture classes, not a designated-class table; no third class can be added without editing the harvest. | `repository.rs`, `architecture/src/spec.rs` |
| 5 | Anchor harvest is one hardwired index-and-upstream pair opened by a literal heading match; a second index needs a second implementation. | `repository.rs` |
| 6 | Kind vocabulary is split three ways: a closed realization list, a closed model list, and no vocabulary for the prose owners. | `label.rs`, `rust_source.rs` |
| 7 | The segment alphabet admits hyphens in kind and area, which both the draft and ADR-013 restrict to words. | `label.rs` |
| 8 | Duplicate reporting has two shapes: the registry path names both locations, the graph-node path names one. | `registry.rs`, `repository.rs` |

## Questions needing a ruling · `sec:gapcensus:rulings`

These are contradictions between an adopted draft and current
repository practice. They are recorded as questions, not resolved here.

1. Settled: the identity draft prescribes no hash construction and
   leaves every scheme to a recipe record, while ADR-016 prescribed the
   domain-separated form and required it of every future semantic
   identity. ADR-016 is deleted and the draft promoted; ADR-021 keeps
   the construction as the one local
   recipe convention, which is adoption data rather than a competing
   prescription.
2. Settled: the identity draft's property table requires domain
   separation of every admitted identity, with no exception clause,
   and ADR-016 once grandfathered the architecture semantic hash and
   the anchor-set hash. With no exception clause to stand on, both
   recipes migrate rather than persist as a recorded divergence, and DI-004 carried
   the migration; the grandfather clause is gone and the ADR records
   the separation migration in its place.
3. The kind registry classifies Task as `exer`, while the repository
   uses `task` for work items. The alternative to a rename is to record
   the repository's token as a local extension. Ten further kinds the
   repository relies on, `trap` and `obl` and `pin` among them, have no
   row at all. Settled since: the Table question, the repository having
   migrated to `tab` throughout.
4. Settled: the repository used both `tab` and `tbl`, and both `post`
   and `postc`, which the one-kind invariant forbids independently of
   adoption. The `tbl` and `post` spellings are gone from the tree.
5. The label calculus fixes kind and area over an alphabet without
   hyphens; the checker admits hyphens everywhere. Tightening is a
   corpus-wide validation change; loosening the draft is a text change.
6. The label calculus's signature has one owner per code package and
   assumes such an owner can be cited. Today those owners have no
   readable prefix. Granting them one is a signature change of the kind
   the draft says enters only by recorded decision.

## Residual scope · `sec:gapcensus:residuals`

Not covered: the LaTeX label surface and the forbidden-token scanner,
both outside the drafts' subject matter; and the interchange draft's
metatheory, checked by reading rather than by anything runnable.

Also not covered, and owed a census of its own: the clauses the third
edition adds. The calculus's derivation authority is the whole of it —
the profile signature, the reserved-kind set, the two warrant rules and
the judgment of derivation, warrant totality, inventory discipline, the
warrant-lapse meta-theorem, and the three Ansätze that delimit
derivation negatively. The checker has none of this, so every such
clause would read absent; the rows are omitted rather than filled with
one repeated word. The identity draft's new stop record and
well-founded-graph rule, and the interchange draft's new metatheory,
are uncensused for the same reason.
