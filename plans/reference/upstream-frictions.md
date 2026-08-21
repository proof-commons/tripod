# Upstream Friction Register · `ref:upstream:frictions`

> **Status:** Human review reference
> **Scope:** every complaint or friction this project holds against
> software outside this repository
> **Machine-consumed:** no

This is the cross-reference index for everything this project has found
wrong, misleading, or missing in the software it builds against. The
primary records live where the friction was established — the Elements
observation register lives in the upstream workspace per ADR-018, and
the evidence behind each entry lives in this repository's gate records —
and this index exists so no complaint has to be rediscovered by reading
a campaign's history. A new friction is filed at its owning register
first and indexed here in the same change.

## Elements observation register · `ref:upstream:elements-gripes`

The register of record is `doc/local-notes/gripes.md` on the `notes`
branch of the ADR-018 Elements workspace. Twenty-one entries as of
2026-08-21. Dispositions: patch (a local fix branch), report-upstream
(worth an upstream issue), feature-request (a consensus-affecting
capability proposal), live-with-it (adapted to downstream).

| ID | Disposition | One line |
|---|---|---|
| EG-001 | patch | Tapscript EC opcodes lacked stack-size checks; local fix branch exists. |
| EG-002 | report-upstream | No C++ unit test covers the tapscript opcodes. |
| EG-003 | report-upstream | Arithmetic overflow and division by zero share one script error string. |
| EG-004 | report-upstream | Script-number decode failures collapse to an unnamed "unknown error". |
| EG-005 | live-with-it | CHECKSEQUENCEVERIFY reads five-byte operands where ordinary numerics read four. |
| EG-006 | live-with-it | Minimal-push, unknown leaf versions, and OP_SUCCESS reject at relay policy only. |
| EG-007 | report-upstream | generateblock mines only what it is handed and ignores the mempool. |
| EG-008 | live-with-it | Unrecognized public-key types pass signature checks without verification. |
| EG-009 | live-with-it | Nonce introspection pushes one stack item where asset and value push two. |
| EG-010 | live-with-it | A rebuilt binary can keep a stale dirty-version string; version output is not provenance. |
| EG-011 | live-with-it | No RPC observes interpreter stack state; conformance sees verdicts only. |
| EG-012 | report-upstream | BIP-341 wallet vectors half-regenerated: every leaf-hash entry is stale Bitcoin-tagged data. |
| EG-013 | report-upstream | secp256k1-zkp fixtures named for 2G actually encode its negation; only the names are false. |
| EG-014 | report-upstream | Consensus call sites assert on generator-generation failure instead of rejecting. |
| EG-015 | report-upstream | Every CT conservation failure collapses to one reject code; the block layer loses even that. |
| EG-016 | live-with-it | No interface accepts a blinding seed; byte-reproducible confidential transactions are unconstructible. |
| EG-017 | live-with-it | Blinded inputs cannot be consumed to a fully explicit output set through any supported path. |
| EG-018 | live-with-it | The functional test framework has no confidential-transaction primitives to test against. |
| EG-019 | patch | The wallet signs a taproot digest the wire form cannot reproduce and calls it complete; local fix branch exists. |
| EG-020 | feature-request | Tapscript cannot verify a Pedersen commitment opening; three upstream capabilities would each close it. |
| EG-021 | report-upstream | An issuance with no witness section is refused as a balance failure, and rawissueasset builds transactions that can never be accepted. |

EG-013 and EG-014 originate in the secp256k1-zkp library that Elements
vendors, so their true upstream is the BlockstreamResearch repository
rather than Elements itself; they are filed in the Elements register
because that vendored copy is the reviewed target source.

The diagnostic-collapse family — EG-003, EG-004, EG-015, EG-021 —
recurs as a practical cost: the Guide-12 campaign twice attributed a
failure to the wrong layer because the target's verdict vocabulary
could not carry the distinction, once in the issuance bisect and once
in the negative-coverage adapter, and both times the repair was
downstream engineering around an upstream answer that says less than
the node knows.

## Toolchain and environment frictions · `ref:upstream:toolchain-frictions`

Frictions with the build and verification environment, each with its
adaptation. These have no upstream register of their own; this table is
their record.

| Friction | Adaptation |
|---|---|
| The freedesktop SDK rust-nightly extension bundles a clippy older than its own cargo, so the lint set the gate enforces trails the current toolchain and new lints surface only elsewhere. | Lint currency comes from a rustup nightly outside the SDK; two batch-gate failures during Guide 12 were exactly lints the SDK clippy could not see. |
| fontconfig does not index the TeX Live texmf-dist font tree, so XeLaTeX finds a font that fontconfig-based lookup then denies exists. | The document toolchain symlinks the texmf-dist opentype and truetype trees into the system font path and refreshes the cache; carried in the shared-instance bootstrap. |
| Distribution-packaged meson can sit far below this repository's declared floor. | meson is pinned in a virtual environment wherever the distribution package is too old. |
| An Elements regtest daemon defaults to validating pegins against a mainchain daemon that does not exist in a test environment. | Every harness invocation disables pegin validation explicitly. |

## Friction labels and adaptation sites · `ref:upstream:friction-labels`

Every friction above carries a label, minted in the table below and
cited at each place in this repository that adapts to it. The label is
the friction's name in the corpus: a reader who meets one of those
adaptations follows the citation to the entry, and a reader who wants
to know what an upstream correction would cost here reads the citation
list instead of a campaign's history. The Elements labels take the
owning register's own identifier as their name, so that a mint and the
entry it indexes cannot drift apart.

A citation is the mechanism; the third column is only its index. Where
that column says the register alone, this repository holds no citation
the label check can reach, for one of three reasons: nothing here is
shaped by the friction, the adaptation lives in the Python adapter
under `scripts/`, which the label census does not scan, or the
adaptation is not in this repository at all. Those places are named in
words and are not citations, and the difference matters, because only a
citation fails when its mint is removed.

| Friction | Label | Where this repository adapts |
|---|---|---|
| EG-001 | `obs:upstream:eg-001` | The register alone: the missing checks are upstream's, and no shape here was taken because of them. |
| EG-002 | `obs:upstream:eg-002` | The register alone: the gap is upstream's own coverage. |
| EG-003 | `obs:upstream:eg-003` | `packages/target-elements-conformance/src/protocol.rs`, the observed class for a refusal the target states with one coarse arithmetic string. |
| EG-004 | `obs:upstream:eg-004` | `packages/target-elements-conformance/src/protocol.rs`, the script-number class that carries both causes the unnamed error hides. |
| EG-005 | `obs:upstream:eg-005` | `packages/target-elements/src/encoding.rs` and `packages/target-elements-conformance/src/census/numeric.rs`, which model the wider operand apart from the ordinary one. |
| EG-006 | `obs:upstream:eg-006` | `packages/vectors/src/matrix.rs`, the relay-policy evidence boundary, and `packages/target-elements-conformance/src/claim.rs`, the nonminimal-push claim. |
| EG-007 | `obs:upstream:eg-007` | The register alone: the funding and confirmation path in `scripts/elements-native-executor.py` hands every transaction to the miner, and that file is outside the label census. |
| EG-008 | `obs:upstream:eg-008` | `packages/target-elements/src/success.rs` and `packages/target-elements/src/authorization.rs`, which model an unrecognized key type as a success. |
| EG-009 | `obs:upstream:eg-009` | `packages/target-elements/src/opcode.rs`, the nonce introspection contract, the one asymmetric case among the reviewed primitives. |
| EG-010 | `obs:upstream:eg-010` | `packages/target-elements-conformance/src/provenance.rs`, which compares an embedded revision against the tip the operator intended. |
| EG-011 | `obs:upstream:eg-011` | `packages/target-elements-conformance/src/protocol.rs`, the resource observation whose interpreter figures are optional rather than zero. |
| EG-012 | `obs:upstream:eg-012` | `packages/target-elements-conformance/src/tests/constructor_tests.rs`, which transcribes the sound array of the vector file and no other. |
| EG-013 | `obs:upstream:eg-013` | `packages/target-elements-conformance/src/disposition.rs` and `packages/target-elements-conformance/src/tests/commitment_oracle_tests.rs`, which read the prefix rather than the fixture's name. |
| EG-014 | `obs:upstream:eg-014` | `packages/target-elements-conformance/src/commitment_oracle/generator.rs`, which returns a defect where the consensus call site asserts. |
| EG-015 | `obs:upstream:eg-015` | `packages/vectors/src/matrix.rs`, where the one reject code fixes a class's boundary and leaves its relation undischarged. |
| EG-016 | `obs:upstream:eg-016` | `packages/target-elements-conformance/src/conservation.rs`, the determinism level that stops at the fixture inputs. |
| EG-017 | `obs:upstream:eg-017` | `packages/target-elements-conformance/src/normalization.rs`, which claims the constructible shape and reports the other as a finding. |
| EG-018 | `obs:upstream:eg-018` | `packages/target-elements-conformance/src/conservation.rs` and `packages/target-elements-conformance/src/commitment_oracle/mod.rs`, the first-party oracle that exists because there is nothing to differentially test against. |
| EG-019 | `obs:upstream:eg-019` | `packages/transaction/src/bytes.rs`, whose serializer grows both witness vectors exactly where the wallet's signer does not. |
| EG-020 | `obs:upstream:eg-020` | `packages/target-elements-conformance/src/disposition.rs`, the typed blocker register, with `plans/research/public-declassification.md` and `plans/reference/elements-tapscript.md`. |
| EG-021 | `obs:upstream:eg-021` | `packages/transaction/src/bytes.rs` and `packages/transaction/src/tests/encoding_tests.rs`, which refuse to write the section the target asserts on. |
| The SDK clippy trailing its cargo | `obs:upstream:sdk-clippy-currency` | The register alone: lint currency is a property of the toolchain the gate is run with, not of anything in the tree. |
| The texmf font tree unindexed | `obs:upstream:texmf-font-lookup` | The register alone: the symlink and cache refresh are carried in the shared-instance bootstrap. |
| Distribution meson below the floor | `obs:upstream:meson-version-floor` | The register alone: the pinned environment is provisioned outside this repository. |
| Regtest pegin validation on by default | `obs:upstream:regtest-pegin-validation` | The register alone: the flag is passed by `scripts/elements-native-executor.py`, outside the label census. |

## Discipline · `ref:upstream:frictions-discipline`

A friction earns an entry when it cost an investigation, forced an
adaptation, or blocks a capability — not merely when upstream made a
choice this project would not have made. Entries are never deleted;
a resolved entry keeps its identifier at its owning register with a
closed status, and this index drops to one line saying so. When a new
external dependency joins the project, its frictions join this index.

Removing a label is how a correction is collected. When upstream fixes
a friction, the entry at its owning register flips to closed and the
mint above is deleted in the same change. The label check then fails at
every citation that outlived it, one diagnostic per site, and that list
is the worklist: each site is a place this repository shaped itself
around the correction's absence and must now be read again. The
failures cannot be cleared by deleting the citations, because a
citation goes only with the adaptation it explains — clearing them is
the work. A friction whose row above says the register alone yields no
such list, which is the standing cost of an adaptation the checker
cannot reach.
