# Contributing to Tripod · `sec:contributing:tripod`

Start here as a contributor, worker or reviewer. [README.md](README.md) introduces the compiler and its attestation exemplar; [AGENTS.md](AGENTS.md) owns the detailed working rules, build cadence and citation restrictions; [SECURITY.md](SECURITY.md) owns the security guidance. This guide routes those rules and the invariant statements without replacing their owners or adopting a proposal by indexing it.

## Work in steps · `sec:contributing:steps`

For dispatched work, the worker charter and working brief supply the lane clone, base, hub lane ref, permitted files, compute route and verification bar. Work in that clone on its own lane branch, commit as needed within the brief, and ship only to the named lane ref. Worker commits are unsigned; the loop reviews and rebuilds the landing into a few meaningful signed commits. Preserve pushed history: do not amend pushed commits; append corrections, and merge a moved base into a slice lane rather than rewriting its published work. The charter distinguishes slice work from the single-bite convention; use the form of the dispatched step. The import and full-gate expectations remain those of AGENTS.md, and the loop owns the gate of record on the landing.

Read the owning statement before changing or reviewing its implementation. The subject-to-authority map is (`[PLAN-rule:plans:authority]`); the planning ownership and update rules are (`[PLAN-rule:plans:laws]`) and (`[PLAN-rule:plans:update]`). A planning label provides navigation, not semantic authority. The [backlog](plans/backlog.md) holds current execution, the [roadmap](plans/roadmap.md) and [phase cards](plans/phases/README.md) hold sequencing and gates, and [cross-package registers](plans/registers/README.md) locate identity and artifact ownership. [Label registers](plans/labels/README.md) index specification and realization labels and the kind registry's companion attestation; their source documents remain authoritative.

## Text, declarations and citations · `sec:contributing:text`

Keep a fact with its owner and point to it from elsewhere. Follow the [ADR writing rules](adr/README.md) for adopted engineering policy and the planning rules above for planning changes. AGENTS.md states the forbidden-text rule, source-provenance limits and patch-tag convention. Declare every new tracked file in its directory's `meson.build`, under the explicit lint census of [ADR-014](adr/014-meson-lint-census-and-stamps.md); root Markdown documents join `doc_files` in the root `meson.build`. Generated publications are updated through their owning generators, as described in README.md and (`[PLAN-rule:labels-index:update]`).

The house label is a colon-separated triple of kind, area and name, governed by (`[ADR019-lang:labels:label-language]`) and the repository's area amendment (`[ADR019-rule:labels:areas]`): lowercase letters and digits, with hyphenated words permitted in area and name. A bare single-backtick span mints; parentheses around it cite within the same owner; an imported citation encloses the target owner's prefix and label in square brackets inside the backticks. These displayed forms are examples, not mints:

```text
mint:                 `sec:contributing:text`
same-owner citation:  (`sec:contributing:text`)
imported citation:    (`[ADR019-inv:labels:unique-mint]`)
```

Owners and prefixes are fixed by (`[ADR019-rule:labels:owner-prefixes]`) and (`[ADR019-rule:labels:owner-partition]`), read with (`[ADR023-dec:scripts:carrier]`): this guide is DOC-owned; plans use PLAN, realization RZ, specification papers A, numbered ADRs their ADR prefix, and code its package owner. Code-comment occurrences use acute delimiters in place of backticks. Fenced examples and double-backtick spans display spelling without participating; scanned regions are specified by (`[ADR019-rule:labels:scanned-regions]`) and its ADR-023 amendment. Mint a label once, cite it elsewhere, and update its citations when it moves between owners or retires. Resolution and warrant rules belong to ADR-019, including its implementation limits (`[ADR019-rem:labels:implementation-standing]`).

Kinds and their meanings belong to (`[ADR020-dec:kinds:adoption]`); using an existing kind does not license a new sense for it. Identity admission and construction belong to (`[ADR021-dec:identity:adoption]`); use labels or paths for citations, not Git object names (`[ADR021-rule:identity:hash-citation-audit]`). Externally consumed documents follow (`[ADR022-dec:interchange:adoption]`), with first-party streams kept under their separate owner by (`[ADR022-rule:interchange:boundary]`). Interchange namespace labels and documentation labels are different languages (`[ADR022-rem:interchange:alignment]`).

## Invariant routes · `sec:contributing:invariants`

Each row routes one statement of kind `inv` to the document that mints it. Read its statement, argument, scope and adoption status there. This table does not assert a new invariant, strengthen an existing one or confer authority on planning text. Properties declared under other kinds remain written authority without an entry here. Whoever adds or moves an `inv` statement updates its route here in the same change.

### Adopted documents and chartered planning · `sec:contributing:adopted-routes`

The ADR rows retain their recorded adoption and implementation boundaries. The realization rows are its conformance clauses. Guide 15 is chartered execution planning subject to the authority order above.

| Statement | Minting document |
|---|---|
| (`[ADR019-inv:labels:warrant-totality]`) | [ADR-019](adr/019-label-calculus.md) |
| (`[ADR019-inv:labels:unique-mint]`) | [ADR-019](adr/019-label-calculus.md) |
| (`[ADR019-inv:labels:total-resolution]`) | [ADR-019](adr/019-label-calculus.md) |
| (`[ADR019-inv:labels:generated-compliance]`) | [ADR-019](adr/019-label-calculus.md) |
| (`[ADR019-inv:labels:inventory]`) | [ADR-019](adr/019-label-calculus.md) |
| (`[ADR019-inv:labels:two-pass]`) | [ADR-019](adr/019-label-calculus.md) |
| (`[ADR020-inv:kinds:one-kind]`) | [ADR-020](adr/020-environment-kinds.md) |
| (`[ADR020-inv:kinds:distinctness]`) | [ADR-020](adr/020-environment-kinds.md) |
| (`[ADR020-inv:kinds:totality]`) | [ADR-020](adr/020-environment-kinds.md) |
| (`[ADR020-inv:kinds:catalogued-pairs]`) | [ADR-020](adr/020-environment-kinds.md) |
| (`[ADR020-inv:kinds:attestation-coverage]`) | [ADR-020](adr/020-environment-kinds.md) |
| (`[ADR022-inv:interchange:restraint]`) | [ADR-022](adr/022-interchange-conventions.md) |
| (`[ADR022-inv:interchange:permanence]`) | [ADR-022](adr/022-interchange-conventions.md) |
| (`[ADR022-inv:interchange:patch-identity]`) | [ADR-022](adr/022-interchange-conventions.md) |
| (`[ADR022-inv:interchange:minor-inclusion]`) | [ADR-022](adr/022-interchange-conventions.md) |
| (`[RZ-inv:invariant:identity]`) | [Realization](docs/attestation/realization.md) |
| (`[RZ-inv:invariant:domains]`) | [Realization](docs/attestation/realization.md) |
| (`[RZ-inv:invariant:rate-floor]`) | [Realization](docs/attestation/realization.md) |
| (`[RZ-inv:invariant:no-trap]`) | [Realization](docs/attestation/realization.md) |
| (`[RZ-inv:invariant:backing]`) | [Realization](docs/attestation/realization.md) |
| (`[RZ-inv:invariant:no-starve]`) | [Realization](docs/attestation/realization.md) |
| (`[RZ-inv:invariant:escrow-receipts]`) | [Realization](docs/attestation/realization.md) |
| (`[RZ-inv:invariant:accounting]`) | [Realization](docs/attestation/realization.md) |
| (`[RZ-inv:invariant:consensus-value]`) | [Realization](docs/attestation/realization.md) |
| (`[RZ-inv:invariant:succession]`) | [Realization](docs/attestation/realization.md) |
| (`[RZ-inv:invariant:maturity]`) | [Realization](docs/attestation/realization.md) |
| (`[PLAN-inv:guide15-exec:clear-census]`) | [Guide 15](plans/guides/guide_fifteen.md) |

### Retained proposal routes · `sec:contributing:proposal-routes`

The destruction-lineage study is superseded as the Wave-1 contract by the Keyed Provenance Covenant paper. Its seven statements below remain reachable as the record of that study, not current contract requirements; its status and scope are (`[PLAN-abst:lineage:scope]`). Routing them does not adopt them.

| Statement | Minting document |
|---|---|
| (`[PLAN-inv:lineage:unique-successor]`) | [Destruction-lineage study](plans/drafts/destruction-lineage-exemplar-study.md) |
| (`[PLAN-inv:lineage:set-growth]`) | [Destruction-lineage study](plans/drafts/destruction-lineage-exemplar-study.md) |
| (`[PLAN-inv:lineage:exactly-once]`) | [Destruction-lineage study](plans/drafts/destruction-lineage-exemplar-study.md) |
| (`[PLAN-inv:lineage:immutable-records]`) | [Destruction-lineage study](plans/drafts/destruction-lineage-exemplar-study.md) |
| (`[PLAN-inv:lineage:accepted-sets]`) | [Destruction-lineage study](plans/drafts/destruction-lineage-exemplar-study.md) |
| (`[PLAN-inv:lineage:replacement-history]`) | [Destruction-lineage study](plans/drafts/destruction-lineage-exemplar-study.md) |
| (`[PLAN-inv:lineage:inherited-anchors]`) | [Destruction-lineage study](plans/drafts/destruction-lineage-exemplar-study.md) |

The routes cover 34 Markdown mints in six documents. No additional `inv` mint occurs in the code-comment or LaTeX carriers. The invariant identifier strings in `packages/architecture/src/ids.rs` and semantic tags in `packages/architecture/src/spec.rs` refer to realization clauses; they are data, not additional source mints. Their routes are the RZ rows above. Generated copies likewise introduce no additional statement.

## Verify and report · `sec:contributing:verification`

Follow AGENTS.md for routine checks, the full batch gate and the real document build's separate coverage. Documentation-only work needs the label and census checks, not a Rust build; the existing checker binaries read edited subjects at runtime. Run the brief's lane gate on its named compute route, correct failures and repeat. A passing lane gate supplies review evidence; the loop still gates the imported landing. Report a full gate not run as deferred, never as passed. README.md describes the `lint` target and generated publications, and [the labels package](packages/labels/README.md) describes the checking tools.

The working brief fixes the step's report contract. Include what changed and why, its owning labels and files, commit and lane tip provenance, the commands run with exit status and wall time, failures and deferred checks, deviations or NONE, and unresolved questions or missing rules. Distinguish editing, transport and server time. For every server round give its start and wait form and completion marker; after an interrupted wait, recover the same round rather than assuming completion. Finish with the brief's ready or stop form. The loop's gate of record and landing remain separate from that report.
