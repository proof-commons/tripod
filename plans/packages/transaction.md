# Transaction and Witness ABI · `pkg:transaction:contract`

> **Status:** Planned
> **Phase:** [Phase 4](../phases/04-compact-ash.md) onward
> **Package:** `tripod-transaction`
> **Library:** `transaction`
> **Direct dependencies:** `linker`, `target-elements`
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
linker
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

The package establishes ABI-consistent construction and witness materialization.

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
| maturity announcement | STATE constructor, operator signing |
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

- Which Rust Elements library owns transaction, sighash, and CT proof types?
- How are linked layout types finalized without a linker/transaction cycle?
- Does construction evaluate realization expressions directly or use linked recipes?
- What signer-capability interface supports hardware and remote signers?
- Where does fee-market policy end and ABI-valid sponsor construction begin?
- Which package owns trusted-setup or genesis transaction construction?
