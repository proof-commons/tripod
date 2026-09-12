# Phase 6 — STATE Constructor and Maturity Announcement · `phase:roadmap:state-maturity`

> **Status:** Active — opened by the owner 2026-08-27 as the roadmap's
> current phase when Phase 5 exited; both entry conditions hold — the
> amended phase-5 exit gate is met on its countersigned assessment, and
> the STATE-constructor research decision is accepted. The owner
> supplied Guide 14, archived verbatim (`T6-001`); its Wave 0 — the
> tenth-review disposition and the Phase-5 handoff revalidation — ran
> and closed out (`T6-002`, `T6-031`); the owner-issued conceptual
> preflight register binds Waves 1–13 (`T8-001`). Wave 1, typed STATE
> metadata and semantic transition, is the next wave.
> **Entry:** (`gate:phase5:exit`) and accepted STATE-constructor decision
> **Packages:** tapscript, linker, transaction, vectors
> **Operation:** `announce-maturity`

## Goal · `sec:phase6:goal`

Integrate authenticated metadata-dependent STATE succession in the smallest
STATE-changing operation.

The implementation must preserve STATE metadata and static code continuity
under the accepted constructor design.

## Deliverables · `sec:phase6:deliverables`

### STATE constructor

Implement the accepted constructor recipe:

- canonical STATE metadata schema;
- deterministic metadata commitment;
- linked static operation-program identity;
- predecessor constructor authentication;
- successor constructor reconstruction;
- internal-key policy;
- metadata path unspendability;
- target control-path recipe;
- constructor resource formula.

### Operation relation

Implement `announce-maturity`:

- canonical STATE predecessor;
- maturity currently unannounced;
- announced cycle within minimum and maximum leads;
- successor maturity announced;
- all economic state fields unchanged;
- operator authorization;
- no RESV use;
- optional isolated sponsor flow;
- STATE succession projection.

### ABI

Define:

- STATE input slot;
- STATE successor slot;
- operator witness role;
- constructor/static-root metadata;
- sponsor region;
- target transaction version/sequence constraints.

## Required vectors · `sec:phase6:vectors`

Positive:

- minimum valid lead;
- maximum valid lead;
- valid operator;
- sponsorless/sponsored;
- exact successor metadata.

Negative:

- below minimum;
- above maximum;
- second announcement;
- sealed predecessor;
- wrong operator;
- wrong STATE asset or amount;
- wrong predecessor metadata;
- changed unaffected state field;
- wrong successor metadata;
- successor under another static root;
- extra/missing operation leaf;
- wrong internal key;
- wrong parity/control path;
- metadata-leaf spend;
- unexpected RESV;
- missing STATE successor;
- stale constructor from another bundle.

## History evidence · `sec:phase6:history`

Extend constructor and root-history fixtures to reject:

- intermediate constructor substitution;
- final cursor restored after an invalid intermediate STATE edge;
- old-bundle predecessor/new-bundle successor without an approved migration;
- post-sealing STATE revival.

## Resource evidence · `sec:phase6:resources`

Measure complete announce-maturity transactions with:

- representative and maximum metadata;
- deepest control path;
- operator signature;
- maximum sponsor candidate;
- constructor verification.

Success here validates the constructor mechanism for this operation, not
resource feasibility of every later STATE operation.

## Wave-0 ground truth · `sec:phase6:ground-truth`

Guide 14 §3.1 asks Wave 0 to record the existing owners of the STATE semantics. The Wave-0 rows recorded the tenth-review disposition and the archival arithmetic but not this census, so it is recorded here from the tree as it stands at the Wave-1 opening, each entry naming its owner file and line. Every line below was read at that tip before it was written; where a cited line had moved, the line here is the one the tree holds.

| Subject | Owner | Exact spelling |
|---|---|---|
| STATE operation identifier | `packages/architecture/src/ids.rs:112` | `AnnounceMaturity = 13 => "announce-maturity"`, inside `OperationId` at `ids.rs:99`. The identifier §2.1 expects already exists, so Wave 1 reuses it rather than minting a lookalike. |
| STATE operation spec | `packages/architecture/src/spec.rs:1607-1654` | One `ObjectId::State` in and one out, each `minimum: 1` and `maximum: MaxCount::Exact(1)`; an `ObjectId::PlainLbtc` sponsor input from `0` to `MaxCount::Bound(BoundId::FeeSponsorInputMax)`; `roots: ROOTS_STATE_ONLY`; `canonical_deltas: NO_DELTAS`; `projections: PROJECTION_TRANSITION`; witnesses `StateSuccession`, `NativeFeeAuction`, `ValueFlowClosure`. |
| STATE singleton asset | `packages/architecture/src/spec.rs:2049-2050`, `packages/architecture/src/ids.rs:71` | The `OBJECTS` entry `id: ObjectId::State` carries `asset: AssetId::Pid`, and `AssetId::Pid = 5 => "PID"`. |
| STATE singleton amount | `packages/model/src/ops/maturity.rs:70` | `Sat::ONE`, in the successor emission. `ObjectSpec` carries no amount field at all, so the architecture states no amount and the model is its only owner. |
| cycle domain | `packages/model/src/scalar.rs:14` | `pub type Cycle = u64;` — a bare alias, with no ordinal law attached. The realization crate had no cycle domain before Wave 1; the `Cycle` occurrences in its error vocabulary name dependency-graph cycles, a different sense. |
| announcement lead bounds | `packages/model/src/constants.rs:17-18` | `pub min_maturity_lead: Cycle` and `pub max_maturity_lead: Cycle` on `Constants`, read per world at `packages/model/src/ops/maturity.rs:44-60` through `checked_add`. No `BoundId` in the architecture names them (`packages/architecture/src/ids.rs:252-263`). |
| maturity variants | `packages/model/src/asset.rs:64-69` | `Unannounced`, `Announced { cycle: Cycle }`, `Complete`. No explicit discriminants, so no encoding is fixed by the type. |
| complete STATE semantic field census | `packages/model/src/pool.rs:11-19` | `omega`, `y_l`, `y_t`, `q`, `cycle`, `maturity` — six fields on `PoolState`. The §6.1 clue holds field for field, so §3.1's blocker condition does not fire. |
| architecture data census of STATE | `packages/architecture/src/ids.rs:193-204` | `StateOmega`, `StateYLive`, `StateYTimeLocked`, `StateQ` — four of the six. `DataId` has no entry for cycle or for maturity. |
| announce-maturity relation in realization | `packages/realization/src/declarations/mod.rs:7-18` | Absent before Wave 1. `compact_ash` and `transfer_live` are the declared operations, and every other `OperationId` falls to `RealizationError::UnsupportedOperationDeclaration`. |
| maturity refusal vocabulary | `packages/model/src/guard.rs:74-77` | `MaturityAlreadyAnnounced`, `MaturityLeadTooShort`, `MaturityLeadTooLong`, `MaturityNotComplete`. The branch also reaches `Sealed` (`guard.rs:48`), `CycleOverflow` (`guard.rs:39`) and `BadSignature` (`guard.rs:45`, raised by `require_signer` at `packages/model/src/signer.rs:28`). `BadAuthorization` exists at `guard.rs:46` but no path in this branch reaches it. |
| transition certificate | `packages/model/src/certify.rs:46-59` derives it; `packages/architecture/src/ids.rs:269` names `ProjectionId::TransitionCertificate` | `derive_transition_certificate` takes the before and after worlds and the consumed and emitted maps; the request supplies none of it, so §2.6's "the request does not author the certificate" holds against the tree. |
| first-party codec precedent | `packages/model/src/ledger.rs:1307`, `:1480`, `:1642-1664`, `:1742-1743` | Domain string `b"tripod/query/v13"`; big-endian integers via `to_be_bytes`; a strict prologue that refuses a wrong domain before reading anything else, over a fixed-width cursor; and `DecodeError::TrailingBytes` when the cursor does not land on the end. |
| prototype metadata codec | `packages/target-elements-conformance/src/constructor/metadata.rs:30-44`, `:70`, `:108-112` | A 48-byte prototype object with a `u32` representation nonce, written little-endian via `to_le_bytes`. Its own module header is titled "Not an ABI": it is evidence-package prototype code, never promoted, so its choices bind nothing. |

### Wave-1 rulings · `rule:phase6:wave1-rulings`

1. The realization crate gains a `Cycle` domain. Its semantic-domains section already anticipates exactly this for later operations ((`sec:realization:domains`)), so the domain is an expected extension rather than a new kind of fact in that crate. The same section fixes the classification: a count is not an amount merely because both are integers, and a cycle is neither — it is an ordinal, and only comparison and checked offset are meaningful on it.

2. The pipeline-facing typed STATE metadata — `Maturity`, `StateMetadata`, `AnnouncementLeadBounds`, the total `announce_maturity` transition, and later its canonical codec — lives in the realization crate. The library direction runs architecture, realization, compiler, target-elements, tapscript, linker, transaction (§3.4), and no pipeline crate names the model: `packages/realization/Cargo.toml:16-18` names `architecture`, `petgraph` and `thiserror` only, and the compiler, tapscript, linker and transaction manifests name no model either. A type owned by the model would therefore be unreachable to the constructor and ABI waves, which is the decisive constraint. The projection law against the model's `PoolState` — lossless round trip, field-for-field agreement, and agreement of `announce_maturity` with the model branch on every refusal — is proved in the model's conformance tests, which see both crates because `packages/model/Cargo.toml:19` and `:23` name `architecture` and `realization`. That test is the drift tripwire: it is what keeps one owner per fact while the type sits in the crate the pipeline can reach.

3. Lead bounds enter the transition as a typed parameter carrying the model's validity law — the minimum at least one, and the minimum not above the maximum — so an invalid bound pair cannot be constructed and the transition need not re-check it. No architecture bound is minted in Wave 1. `BoundId` is a published discriminant set whose header forbids reordering or renumbering after publication and admits new variants only under a new architecture schema version (`packages/architecture/src/ids.rs:1-5`), so naming a maturity-lead bound there is an architecture schema change, not a realization change, and Wave 1 is not the place to make one.

4. Refusals are distinct closed sums per layer, spelled as §6.6 spells them: a transition sum, and, when the codec bite lands, a separate decode sum. A decoder that could return a semantic refusal, or a transition that could return a byte-level one, would make each layer a partial owner of the other's vocabulary and leave the composed precedence undefined. The model keeps its `Guard` names and maps onto the transition sum in conformance, which is the same discharge shape `G14C-06` states for Waves 1 and 4.

5. The canonical metadata encoding is big-endian, with a `u32` representation nonce. Big-endian is the first-party precedent already load-bearing in the model's own codec (`packages/model/src/ledger.rs:1480`), and matching it means one byte-order convention to reason about across first-party encodings rather than two. The little-endian prototype in the conformance package is not counter-evidence: it is unpromoted evidence code whose own header refuses ABI standing. The encoding is an ABI-identity component — D006 binds constructor and metadata schemas and representation requirements into ABI identity ((`rule:abi:identity`)) — so it is recorded here, before any bytes exist, rather than left to be discovered from whatever the first implementation happened to write.

### Handed up for a ruling · `rem:phase6:wave1-questions`

(a) Candidate R-6 reads `StateMetadata` as a projection of the model's canonical `PoolState`. Ruling 2 realizes that reading as a realization-owned type whose projection law is proved in model conformance, because the verified dependency direction admits no other placement that the constructor and ABI waves can reach. Under the register's candidate governance, evidence contradicting a candidate is escalated before any adoption or overturn, so `G14C-01` and R-6 stay OPEN until this reading is confirmed, or until a dependency admission of the model crate into the pipeline is directed instead — which would require the backlog's full dependency-admission record under §3.4.

(b) Guide §3.1 expects the announcement lead bounds in the typed architecture. The tree owns them in the model's `Constants`, with no `BoundId` naming them. Whether the architecture gains that bound — a schema bump under the header rule at `packages/architecture/src/ids.rs:1-5` — is needed before the Wave-2 compiler plan needs values to calibrate against, and is not needed for Wave 1 to close.

(c) The architecture data census names four of the six STATE fields. `G14C-14`'s field-slicing law, that field slicing commutes with semantic projection, will need architecture names for cycle and maturity, or an explicit statement that the law ranges over the four declared quantities only and that the remaining two are model-owned.

(d) Wave 0's remaining deliverable, the §5.1 carrier-sufficiency proof, was not recorded. §5.1 accepts the witness carrier only if Wave 0 proves that an unrelated process can reconstruct the successor from the accepted transaction alone, and directs that implementation stop for a focused carrier decision if that proof fails. Neither outcome is on record, so the proof is owed before the constructor wave and is carried here as Wave-0 debt.

## Exit gate · `gate:phase6:exit`

Phase 6 exits when:

- accepted constructor research is implemented exactly;
- metadata encoding is canonical;
- predecessor and successor share one authenticated static code identity;
- no key-path or metadata path bypass remains;
- announce-maturity matches the model relation;
- all wrong-code-subtree vectors reject;
- root-history continuity tests pass;
- target and policy resources fit;
- bundle/ABI/report identities are deterministic;
- relation coverage is complete and the repository remains clean.
