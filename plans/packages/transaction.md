# Transaction and Witness ABI · `pkg:transaction:contract`

> **Status:** Candidate — the compact-ASH ABI and the maturity-announcement `CandidateMaturityAnnouncementAbi` with `MaturityConstruction`, finalization and operator-signing states are delivered under Guide-12 §15 and Guide-14 §12
> **Phase:** [Phase 4](../phases/04-compact-ash.md) onward
> **Package:** `tripod-transaction`
> **Library:** `transaction`
> **Direct dependencies:** `architecture`, `linker`, `realization`, `sha2`, `tapscript`, `target-elements`
> **Decision:** [D006](../decisions/006-transaction-abi.md)

## Purpose · `sec:transaction:purpose`

`transaction` derives the final target transaction/witness ABI from a linked
bundle and constructs ABI-valid target transactions.

It owns:

- final operation layouts;
- typed operation requests;
- canonical family ordering;
- metadata encoding;
- constructor instantiation;
- target program and control selection;
- transaction assembly;
- representation and blinding construction;
- signing requests;
- witness assembly;
- valid worst-case calibration transactions;
- ABI and construction identities.

It does not own private-key custody or network submission.

## Dependencies · `sec:transaction:dependencies`

Allowed direct dependencies:

```text
architecture
linker
realization
tapscript
target-elements
```

Forbidden dependencies:

```text
model
compiler
vectors
release
artifacts
```

A direct `realization` dependency is added only if linked construction recipes
cannot provide formula evaluation without creating a parallel semantic table.

The direct `architecture` dependency supplies the operation, object, projection and tag identifiers used by maturity ABI classification; the entry also reads the architecture's object table for each claimed family's asset. Transaction authenticates offered positions and reads realization verdicts by relation identity without defining a parallel family policy.

The direct `tapscript` dependency lets `StateThreadAnchor` name constructor generation by `tapscript::StateConstructorGeneration`, so provenance cannot drift from the constructor's typed recipe identity; the direction transaction → linker → tapscript already holds, so this admission adds no crate to the dependency closure and leaves linked construction recipes authoritative, as required by ruling 27 at (`rule:phase6:wave4-rulings`).

The direct `realization` dependency supplies the membership witness in realization's vocabulary: transaction is the boundary that verifies the frozen message and the deployment before producing it. The direction realization → tapscript → linker → transaction already holds in the dependency closure, and this admission leaves the parallel-semantic-table clause untouched because the producer evaluates no formula, as required by ruling 35 at (`rule:phase6:wave5-rulings`).

## Typed inputs · `sec:transaction:inputs`

The package consumes:

- candidate or final linked bundle;
- exact typed target;
- typed operation request;
- typed public chain/input view;
- authorized caller witness capabilities;
- explicit construction policy;
- explicit randomness for confidential construction.

The library performs no RPC or wallet lookup.

## Typed outputs · `sec:transaction:outputs`

Stages include:

1. candidate or final transaction ABI;
2. semantic construction plan;
3. unsigned/unblinded target template;
4. representation-complete signing-ready transaction;
5. signing requests;
6. finalized target transaction and witnesses;
7. construction report;
8. valid worst-case fixtures.

Candidate and final bundle/ABI statuses remain distinct.

> Illustrative boundary; names and exact fields are not frozen.

```rust
pub fn derive_abi(
	bundle: &linker::LinkedBundle,
	target: &target_elements::ElementsTarget,
) -> Result<TransactionAbi, TransactionError>;
```

## Operation requests · `rule:transaction:requests`

A request contains only choices the caller is semantically authorized to make.

The caller may choose, where permitted:

- input outpoints;
- destination owners and denominations;
- burn records;
- sponsor inputs/change;
- supported representation preference.

The caller may not choose:

- formula-bound payout;
- state successor fields;
- issuance amount;
- fixed recipient;
- target program;
- layout index;
- witness order;
- closed asset type.

Those derive from the linked ABI.

## Public construction view · `rule:transaction:view`

The construction view contains typed:

- outpoints;
- target asset/value/program data;
- authenticated public metadata;
- public openings;
- root/state facts;
- confirmation/age facts;
- checkpoint binding.

Owner-private openings or blinding data are supplied separately by authorized
owner adapters.

Permissionless construction tests use only the public view plus sponsor-local
data.

## Canonical ordering · `rule:transaction:ordering`

Top-level family order is ABI-owned.

Within a set-like family, use a declared deterministic rule, normally canonical
outpoint order.

Preserve caller order only when order is semantic or the ABI uses positional
correspondence.

Reject duplicate or overlapping input assignments before construction.

## Metadata and constructors · `rule:transaction:constructors`

Metadata schemas define:

- field order;
- typed domain;
- byte order;
- schema version;
- object/domain separation;
- canonical zero and tags;
- invalid encoding behavior.

Concrete output programs derive from:

```text
linked constructor recipe
+
typed metadata
+
public deployment constants
```

Caller-supplied arbitrary target programs or control paths are rejected.

## Construction stages · `rule:transaction:stages`

The initial staged pipeline is:

1. validate request and view;
2. derive semantic outputs and state effects;
3. instantiate constructors and canonical layout;
4. assemble unblinded transaction template;
5. apply selected explicit/confidential representation;
6. generate target proofs;
7. finalize signature-committed fields;
8. issue signing requests;
9. collect and validate signatures;
10. assemble witnesses/control data;
11. perform ABI-local preflight.

Signing requests are generated only after all protected outputs are fixed.

## Signing · `rule:transaction:signing`

The package holds no production private key.

A signing request binds:

- target;
- bundle;
- ABI;
- operation;
- transaction identity;
- input;
- signer role and public identity;
- sighash profile;
- target message.

Multi-owner operations require every owner to sign the same finalized output
set.

Secret values never enter canonical reports.

## Confidential construction · `rule:transaction:confidential`

Initial support may include:

- confidential live-receipt values;
- confidential sponsor values.

Closed protocol asset identity remains explicit.

Permissionless operations cannot require private unblinding data.

Public-committed outputs and direct private-to-public transitions remain gated
by [public declassification research](../research/public-declassification.md).

Production randomness is injected from a secure source. Canonical tests use
fixed, clearly test-only randomness.

## Client-policy construction · `rule:transaction:client-policy`

`create-request` is client-policy construction, not a covenant-enforced
creation operation.

The package may provide a canonical valid request builder, but malformed open
request-shaped outputs remain possible and are rejected only when consumed by
admission.

The ABI marks this distinction explicitly.

## Worst-case fixtures · `rule:transaction:worst-case`

Calibration fixtures are complete valid transactions targeting one resource
objective.

One operation may require different fixtures for:

- weight;
- witness size;
- stack;
- crypto budget;
- control-path depth;
- policy.

A changed bundle, ABI, bound, target, or representation invalidates fixture
reuse.

## Identity · `rule:transaction:identity`

ABI identity binds:

- schema;
- target;
- bundle;
- calibrated bounds;
- layouts;
- constructors and metadata schemas;
- witness schemas;
- representation support;
- target transaction constraints.

Construction identity excludes private keys and secret blinders.

Given identical explicit inputs—including randomness and signatures—final
bytes are identical.

## Assurance boundary · `sec:transaction:assurance`

The maturity ABI classification and role entries authenticate offered positions, validate the coordinator and placements against the ABI layouts and read the STATE cardinalities from the retained realization report; their acceptance admits nothing to safe construction and calibrates no sponsor-input maximum.

The package establishes ABI-consistent construction and witness materialization; `abi::validate_compact_abi_fields` separately classifies compact input-count, version and sequence disagreements before submission, without admitting raw input to safe construction or asserting witness validity.

It does not establish:

- backend correctness;
- target consensus acceptance;
- current input unspentness at broadcast time;
- relay or confirmation;
- final calibration;
- deployment release.

## Initial operations · `tab:transaction:pilots`

| Operation | Construction requirement |
|---|---|
| compact ASH | Public view only; one derived ASH output; no protocol signature |
| live transfer | Multi-owner signing; explicit/confidential value plans; explicit `U` |
| maturity announcement | STATE constructor, operator signing, explicit sponsored construction with complete spent-output census and per-input sponsor completion; a separate balanced confidential route admits one committed sponsor coin and freezes its change proof before signing, with selected-policy observation admitted at (`rec:backlog:wave0-confidential-route-admission`) |
| later operations | Roadmap and research order |

## Exit gate · `gate:transaction:compact-ash`

Compact-ASH transaction support exits when:

- candidate ABI binds exact candidate bundle;
- canonical ranges and coordinator derive from ABI;
- ASH output derives from authenticated public inputs;
- no owner/operator secret is required;
- sponsor region remains isolated;
- constructor and control data match the bundle;
- worst-case fixtures are valid and deterministic;
- vectors can consume the result without bypassing the safe API.

## Error vocabulary · `sec:transaction:errors`

See [`errors/transaction.md`](errors/transaction.md).

## Open questions · `sec:transaction:open`

Three of these are settled for the compact-ASH candidate and are recorded
rather than deleted, because each was settled for one operation and not in
general.

- Which Rust Elements library owns transaction, sighash, and CT proof types?
  Settled for the candidate: none. The reviewed substrate decision selected
  first-party explicit-field structures, and this package owns the encoder,
  the decoder, the script-path witness assembly, and the canonical role
  layout. Blinding, signing digests, and issuance stay delegated, and each is
  a recorded trigger that would reopen the decision.
- How are linked layout types finalized without a linker/transaction cycle?
  Settled: the linker re-exports the backend types its own accessors return,
  so this package names them through the linker rather than depending on the
  backend, and the dependency stays one-way.
- Does construction evaluate realization expressions directly or use linked
  recipes? Settled for compact ASH: linked recipes only. No `realization`
  dependency was added.
- What signer-capability interface supports hardware and remote signers?
- Where does fee-market policy end and ABI-valid sponsor construction begin?
  The candidate puts the fee amount on the sponsor's side of the boundary: a
  signing capability declares what it is paying, and the ABI states only the
  fee *role*.
- Which package owns trusted-setup or genesis transaction construction?

## Candidate residuals · `rem:transaction:candidate-residuals`

For the existing compact/live/STATE paths, the taproot output key is pinned rather than recomputed. The merkle root and every control-path element are computed here, because the witness is made of them; the tweak that would turn that root and the internal key into an output key is not, and the equality between the pinned program and the committed tree is carried as an outstanding obligation. The arithmetic that would discharge it belongs to the independent host oracle this package's output is compared against, and a builder that called that oracle would be counting one opinion twice.

Sponsor change uses the construction default when no destination is supplied; an explicit ordinary destination is honored and fixed by the sponsor's all-inputs-all-outputs signature, while the leaf retains asset and exact-role checks without destination equality.

## Keyed production curve · `rule:transaction:keyed-curve`

Under (`rec:w1b14:adoption`), `packages/transaction/src/keyed_curve.rs` implements tapscript's supplied `KeyedCurveCapability` through the workspace `secp256k1-zkp` safe wrapper. This changes the wrapper's role from reference oracle only to production public commitment arithmetic; it binds the same C-library family the node uses, so agreement with the node is not independent evidence. The first-party arithmetic in `target-elements-conformance` remains the independent oracle and is never a production dependency. There is no direct `secp256k1-zkp-sys` edge and no second hash implementation.

The workspace pins the safe wrapper to `=0.11.2`, disables defaults and requests `std` on this edge. That request is not a claim about a unified workspace build: `elements` adds its features. The adapter's verification-only context and absence of secret inputs establish its no-signing boundary; signing is not a Cargo feature. The adapter uses `XOnlyPublicKey::from_slice` for even-y lift, `Scalar::from_be_bytes` for unreduced t below n (zero valid), `Secp256k1::verification_only`, `XOnlyPublicKey::add_tweak`, `serialize` and `Parity::Odd`. Transaction's existing `tagged_hash` computes `TapTweak/elements`. An invalid x returns `InternalKeyNotAPoint`, t at or above n returns `TweakAboveGroupOrder`, and the point sum at infinity returns `TweakedPointIsIdentity`; after validated point/scalar inputs, the wrapper's later conversion/parity failure branches cannot arise for a finite library output. Capability tests use raw scalar bytes solely to reach overflow and infinity without finding hash preimages. These are arithmetic/constructor refusals, never covenant execution verdicts. The untraced capability cores run beneath the public commitment boundary.

The node's read-only `cargo tree --offline --locked -p tripod-transaction -e features -i secp256k1-zkp` resolved `secp256k1-zkp 0.11.2` with `std` only; `cargo tree --offline --locked --workspace -e features -i secp256k1-zkp` resolved `actual-rand`, `default`, `global-context`, `hashes`, `rand-std` and `std`. `elements 0.27.0` requests default/global-context/hashes; global-context enables rand-std, which enables actual-rand. Transaction and conformance also request std. The resolved dependencies are `secp256k1 0.29.1`, `secp256k1-zkp-sys 0.10.1` and `secp256k1-sys 0.10.1`. In those registry sources, `secp256k1-sys-0.10.1/depend/secp256k1/CMakeLists.txt:10-14` reports C library `0.4.1`; `secp256k1-zkp-sys-0.10.1/depend/secp256k1/configure.ac:6-9` reports `0.1.0-dev`. Each sys package retains its vendor revision in `depend/secp256k1-HEAD-revision.txt`. The wrapper reexports the `secp256k1` point API, so this adapter's point operations use the former C implementation; the ZKP fork is also in the resolved wrapper graph. These are observed vendored versions, not a claim of identical node revision or independent arithmetic. Only the wrapper's version is pinned by this change; the transitive versions are the observed graph. The recorded trees are at (`tab:w1b14:curve-provenance`).

## Keyed commitment and local authentication · `rule:transaction:keyed-commitment`

`packages/transaction/src/keyed_taproot.rs` computes only the fixed schema-1 recipe in `packages/tapscript/src/keyed_constructor.rs`, from the sealed linked material. The sole leaf hash uses transaction's existing `leaf_hash` (`version || CompactSize(script length) || script`, tagged `TapLeaf/elements`), the complete root equals that hash, and the existing tagged hash computes the big-endian unreduced `TapTweak/elements` scalar. The production curve adapter supplies Q = P + tG and Q's parity. The output script goes through `witness_program_script`; a separate keyed depth-zero control builder writes `(leaf version | parity) || internal x`. The ASH-specific control helper is unchanged. Public SEC1 generator coordinates and the NUMS constant are reused as numerical constants only, never as STATE semantics. No reference-oracle dependency, second hash implementation, transaction construction or funding policy is introduced.

Only `commit_keyed_bundle` produces the private-field `KeyedCommitment`; public `authenticate_keyed_predecessor` returns unit after comparing the complete supplied predecessor program. The sealed borrowed `AuthenticatedKeyedPredecessor` has only a crate-private, untraced equality producer; step 3 must mint and retain it only after canonical parent decoding and selected-outpoint checks, together with that provenance. The public equality entry cannot mint a receipt, and comparison clones no linked bundle. `authenticate_keyed_path` authenticates the canonical script/control pair against that sealed value. These local checks do not bind a parent transaction or outpoint and assert no inclusion, unspentness, retrieval or finality: step 3 owns the former, later recognition supplies the latter. Constructor/arithmetic refusals and commitment mismatches are separate types, and neither claims covenant execution. The complete path census is one script path containing both clauses plus an unavoidable key path conditional on NUMS/discrete-log and preimage resistance; alternate-description exclusion requires second-preimage resistance, and adversarial pairs require collision resistance.

The reviewed API and controls are inventoried at (`rec:w1b14:commitment`). Public entries have one DEBUG span and one completion event on success and refusal; hash, curve and control cores are untraced. Invalid recipe/internal-key derivation are defensive internal consistency guards, not caller-selectable policy. Overflow/infinity and unliftable-key mapping are tested at the real safe arithmetic core and at a private capability seam without searching for hash preimages. Existing attestation construction, emitted bytes and expected values remain unchanged.

### Keyed serialized requests · `rule:transaction:keyed-inputs`

The request entries in `keyed_request` produce sealed SOURCE or nonempty MERGE structural requests from canonical parent bytes, selected outpoints, an independently supplied request key and a sealed keyed commitment. `keyed_inputs` reproduces txids from stripped bytes, reads asset/value/script from the decoded output and retains that provenance with the crate-private authenticated predecessor receipt. Issuance and peg-in parents are unsupported forms, not covenant-invalid parents. The request checks index/duplicate/role/member constraints and derives the unchanged member binder's coverage; it adds no covenant law or revalidation entry. SOURCE uses a SOURCE-only emission; MERGE may use a MERGE or joint emission. The successor position is structurally zero, with materialization still owed.

Collateral is bound to its canonical parent output with authorization deferred. An optional existing sponsor capability must offer that exact outpoint; this does not prove signing authority or spendability. Same-program keyed spends cannot be hidden as collateral. Other scripts and offered sequences remain structurally preserved without being admitted. Every nonzero spending-input flag is refused: bits outside `0xc0` are invalid offers, and in-domain nonzero flags are unsupported spending-input forms for either role. Step 4 must derive actual serialized flags and compare them with the retained zero offer, never clear them silently. Step 4 freezes non-witness bytes, requests and verifies the one admitted P2WPKH sponsor’s signature and admits only the adopted measured envelope; all other collateral remains structurally preserved, unadmitted and unauthorized. Inclusion, unspentness, retrieval, finality, funding and policy admission remain obligations. Public types, refusals, controls and source-lock limits are inventoried at (`rec:w1b14:inputs`).

Input roles do not establish spending authority: members have local commitment equality, birth position zero and an accepted zero flag offer, while covenant execution remains to be evidenced on the materialized transaction. Capabilities on members refuse. Collateral keeps the exact `SponsorOffer` snapshot checked at binding, and copied parent buffers need not outlive the request. Another key’s different-program handle can bind as collateral, but step 4 never admits it and spending it with this request’s output-0 successor violates that key’s continuation clause. Fee outputs and other provably unspendable outputs are likewise structurally preserved, not spendable. Four request obligations concern external parent facts for every input; collateral authorization and funding/policy admission belong to step 4. Per-relation evidence remains in `members().evidence()`. The binder’s `Sponsor` role merely means non-member. Bare predecessor receipts are never step-4 authority: use a sealed request or bound input, and reconcile retained offer coverage/member overlap, exact materialized census, actual flags, signer role and checked index width as recorded at (`rec:w1b14:inputs`).
