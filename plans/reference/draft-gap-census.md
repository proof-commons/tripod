# Draft-versus-implementation gap census

A clause-by-clause census of the four adopted drafts in `plans/drafts/`
against the incumbent decision records and the label checker as it is
written today. Every disposition below was read out of the code, not
out of a module doc comment; where the code did not settle a question,
the row says so.

## How to read this census · `sec:gapcensus:method`

Dispositions are: **implemented**, the repository already does what the
clause says and the module doing it is named; **divergent**, the
repository does something here but not what the clause says;
**absent**, nothing answers the clause; **undetermined**, the code did
not settle it, with the reason given.

Draft clauses are named by parenthesized same-owner citation. The
drafts live under `plans/`, so they belong to the same `PLAN` label
owner as this file and a local citation resolves to them; a bare code
span of the same text would be a second mint of a label the draft
already mints, which the checker rejects.

## Label calculus · `sec:gapcensus:calculus`

This draft is the generic restatement of ADR-013, which is itself
marked decided and implemented. Most clauses are therefore satisfied by
construction; the deltas are exactly the places where the draft
generalizes a repository-specific mechanism into an adoption parameter.

| Clause | Disposition | Evidence |
|---|---|---|
| (`lang:labels:label-language`) | divergent | `Label::parse` in `packages/labels/src/label.rs` admits hyphens in every segment, so kind and area range over a wider alphabet than the words the clause fixes. |
| (`gram:labels:well-formed`) | implemented | `classify` in `packages/labels/src/markdown.rs` parses the immediate syntactic group; a non-label-shaped span falls out of `harvest_markdown_owner` unparsed. |
| (`sig:labels:owners`) | divergent | The signature is a hardcoded chain in `ImportedLabel::parse`, `packages/labels/src/owner.rs`, admitting six prefix families. `LabelOwner::prefix` emits a prefix for `Crate` owners that `parse` cannot read back, so per-package owners are unciteable across owners. |
| (`judg:labels:minting`), (`judg:labels:resolution`) | implemented | `RepositoryCensus::discover` in `census.rs` excludes the generated registers; resolution exists only as an edge built by `build_label_graph` in `repository.rs`. |
| (`judg:labels:participation`), prose | implemented | Fences skipped in `scan_markdown`; double-backtick spans skipped by the `delimiter_len` guard in four harvest sites of `repository.rs`. |
| (`judg:labels:participation`), code | implemented | `comment_segments` in `rust_source.rs` scans comments only, skipping plain, raw, byte, C and character literals; fenced documentation examples are skipped in `harvest_file`. |
| unclosed acute delimiter | implemented | `harvest_segment` in `rust_source.rs` raises `UnclosedInlineCode` for an unpaired delimiter. |
| (`inf:labels:mint`) | implemented for the authorship species only | Bare spans mint into the owner's own registry in `harvest_markdown_owner` and `harvest_label`. The rule is shared by both warrant species; the checker discharges only the authorship premise, and no code path discharges a derivation, so the shared rule is half-realized. |
| (`inf:labels:same-owner-citation`) | implemented | `mint_nodes` in `build_label_graph` is keyed on the owner-and-label pair, so a same-owner citation resolves across that owner's files and nowhere else. |
| (`inf:labels:imported-citation`) | implemented | `import` in `repository.rs`; unknown prefixes raise `UnknownOwner`, and `diagnose_bracket_free_owner_token` fails a cross-owner token written without brackets. |
| self-qualified import refused | implemented | Checked twice: eagerly for the realization owner in `harvest_realization_import`, and universally in `build_label_graph`, which drops the citation and raises `InvalidImportedCitationForm`. |
| (`inf:labels:synthetic-citation`) | divergent | `add_architecture_citations` in `repository.rs` designates exactly two typed-data classes, both hardcoded: architecture witness semantic tags and clause identifiers, both targeting the realization owner. There is no registry of designated classes and no way to designate a third. |
| (`inf:labels:anchor-harvest`) | divergent | `harvest_attestation_citations` implements one hardwired instance: the realization document as index and Layer-0 as upstream, with the index region opened by a literal heading match on the anchors mint. The clause's general index and upstream pair is not parameterized. |
| (`inv:labels:unique-mint`) | implemented | `LabelRegistry::insert_or_diagnose` in `registry.rs` reports the duplicate site and the first mint site. One weaker path, `report_duplicate_graph_node`, reports a single location for graph-node collisions. |
| (`inv:labels:total-resolution`) | implemented | `report_unresolved_citation` in `repository.rs` errors per class, and `validate_citation_outdegrees` rejects a citation resolving to more than one mint. |
| (`inv:labels:two-pass`) | implemented | `harvest_sources` fills every registry and queues citations; `validate` drains the queue afterwards. |
| (`metathm:labels:order-independence`), (`metathm:labels:no-self-support`) | implemented | The first follows from the staging above. For the second, index lines are excluded from the body scan and only body tokens enter the anchor set; `validate_attestation_anchor_pin` recomputes the pin over body names alone. |
| (`metathm:labels:presentation-invariance`) | undetermined | A meta-theorem about migrations, not a checkable property; no code asserts or could assert it. |
| (`cav:labels:non-normativity`), (`cav:labels:coexistence`) | implemented | No compiler, linker or release path reads the registries; registries are per owner, and `Walk::unreadable` in `census.rs` records `CensusUnreadable` rather than yielding an empty group. |
| the five rejected Ansätze, (`ansatz:labels:flat-namespace`) through (`ansatz:labels:participating-registers`) | implemented | All five stay refused: resolution is owner-keyed; a second mint errors; only acute spans in comments are scanned in code; planning sources resolve under the same total rule; and the registers are dropped from discovery while `check::current` still compares their bytes. |

### Gate checklist · `sec:gapcensus:calculus-gate`

Each item of (`gate:labels:implementation`) against the current
checker. The draft ends in a Gate that blocks implementation until met,
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
| generated registers nonparticipating, current, deterministic | holds |
| pinned anchor-set hash from body citations only | holds |
| traversal failures surface as diagnostics | holds |
| scoped generation ignores unrelated owners' defects | holds, in `derive_register_sources` and `derive_model_sources` |
| every mint on exactly one warrant, one warrant species per kind, reserved kinds without a profile fail | absent: the checker has no notion of a warrant, and adopts neither a profile signature nor a reserved-kind set |
| covered assets carry their derived label at the standard place, no inventory label outliving its asset | absent, for the same reason |
| near-miss spans warned without being treated as occurrences | absent: a label-shaped span that parses as no form is silently text |
| gate dischargeable from this document and the adoption data alone | holds |
| one mint per environment, every citation resolves | holds for the drafts as committed: the gate is green |
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

The hybrid rule (`inf:kinds:hybrid`) has no applicability today: no
repository kind concatenates two catalogued kinds, and no compound
environment head is in use. What adoption still costs is a registry
extension of about a dozen tokens, and, for `task` and `res`, either a
rename or a recorded local extension. The realization and model kind
lists remain compiled into `label.rs` and `rust_source.rs`, so any
further rename is a code change and a register regeneration in one
commit.

The second edition's internal defect — a classification judgment
(`judg:kinds:classification`) claiming eight Conventions against a
document carrying eleven — is repaired: the third edition's preamble
claims fourteen and the document carries fourteen.

## Identity adjudication · `sec:gapcensus:identity`

ADR-016 already decides most of this draft, and the register
`plans/registers/identities.md` already records each identity in the
draft's admission shape. Deltas are marked where the draft asks for
something ADR-016 does not.

| Clause | Disposition | Evidence |
|---|---|---|
| (`rule:identity:admission-order`), (`myth:identity:hashes-validate`) | implemented | ADR-016 states it, and the producers in `packages/architecture/src/canonical.rs` and `deployment.rs` hash validated exports only. |
| (`tab:identity:mechanisms`), (`warn:identity:non-claims`), (`req:identity:admission-record`) | implemented | The mechanism table is reproduced as ADR-016's; every register entry carries subject, owner, producer, consumer, decision, assurance, stale condition, recipe, migration, non-claims and status. |
| (`crit:identity:benefit`) | divergent | ADR-016 lists rejection grounds but not the benefit criterion as a test. The draft's reviewed-anyhow ground, that a standing review supersedes equality, has no counterpart in ADR-016. |
| (`alg:identity:adjudication`) | absent | No decision walk exists as a required procedure; admission today is a checklist applied in review. |
| (`case:identity:no-identity`), (`req:identity:stop-record`) | implemented | The register carries eleven stop records, each with its proposal, deciding branch, date and revisit condition, beside six admission records. The dormant and provisional statuses remain, now as the standing of two of the stops rather than as the whole treatment of refusal. |
| (`def:identity:recipe`), recipe identifiers | implemented | Recipe identifiers exist as algorithm constants for the semantic, behavioural, anchor-set and deployment hashes in `canonical.rs` and `deployment.rs`, with superseded recipes retained beside each. The anchor set gained `ANCHOR_SET_HASH_ALGORITHM` in the DI-004 separation migration; its identifier is a code-side register rather than a manifest field, which the migration record states. |
| (`rule:identity:recipe-permanence`) | implemented | The superseded behavioural algorithm list in `canonical.rs` is a migration record, and the identifier is published beside every value. |
| (`tab:identity:properties`), domain separation | implemented | Every first-party semantic identity prefixes a domain. The two that once did not were migrated together in DI-004 under (`[ADR016-rule:identity:separation-migration]`), which supersedes the grandfather clause; no exception remains for the property table to conflict with. |
| (`case:identity:artifact`), freshness sub-branch | implemented | Generated publications are compared byte for byte in `check::current` and carry no digest. |
| (`rule:identity:no-incidentals`), (`rule:identity:immediate-edges`), (`red:identity:mesh-to-chain`), (`red:identity:fields-to-object`) | implemented | ADR-016 and the register forbid local handles in semantic identity, bind the future graph by immediate edges only, and refuse field-level hashing on the draft's own grounds. |
| (`case:identity:evidence`), (`rule:identity:duties`), (`rule:identity:delegation`), (`case:identity:release`) | absent | No evidence envelope, no release manifest and no release validator exist yet; ADR-016 marks these as activating with their consumers. |
| (`rule:identity:provenance-containment`) | implemented | Publication-only provenance identities are contained by the register's status vocabulary. |
| (`gate:identity:implementation`) | divergent | Every item ADR-016 already gates is met, and DI-004 discharged three more: every existing digest is classified, every no-identity outcome is recorded, and every admitted recipe carries an identifier delivering its class's properties. The benefit walk as a standing requirement on new proposals, and the release surfaces, are not. |

## Interchange conventions · `sec:gapcensus:interchange`

No implementation exists, and this is correct: nothing in the
repository encodes CBOR or CDDL, and the draft names no consumer. The
strings CBOR and CDDL occur only in the drafts and the backlog.

| Clause | Disposition | What adoption would demand |
|---|---|---|
| (`lang:interchange:data-language`) | absent | A canonical encoder and a validating decoder for the deterministic encoding profile, with non-canonical input refused rather than repaired. |
| (`lang:interchange:description-language`), (`schema:interchange:global`) | absent | A description-language registry carrying the base theory, and a tool checking a document against an assigned theory. |
| (`gram:interchange:label-grammar`) | absent | A namespace-label validator and an allocation record for whichever labels the repository claims. |
| (`sig:interchange:theory-assignment`), (`inv:interchange:permanence`), (`inv:interchange:patch-identity`), (`inv:interchange:minor-inclusion`), (`law:interchange:major-boundary`) | absent | An owned registry with immutable assignments, a machine check of content-class inclusion across minors, and a version-bump gate wired to it. |
| (`def:interchange:acceptance`), (`metathm:interchange:bounded-determination`) | absent | Envelope-first dispatch with whole-document rejection; the bound then follows from the encoder. |

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

1. The identity draft prescribes no hash construction and leaves every
   scheme to a recipe record, while ADR-016 prescribes the
   domain-separated form and requires it of every future semantic
   identity. Does adoption move the prescription out of the record and
   into recipe records, or does ADR-016 keep it?
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
