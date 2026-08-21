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

## Discipline · `ref:upstream:frictions-discipline`

A friction earns an entry when it cost an investigation, forced an
adaptation, or blocks a capability — not merely when upstream made a
choice this project would not have made. Entries are never deleted;
a resolved entry keeps its identifier at its owning register with a
closed status, and this index drops to one line saying so. When a new
external dependency joins the project, its frictions join this index.
