# Draft: Confidential Test Materialization and Funding Protocol Concept

> **Status:** Concept draft; not an execution guide; unnumbered
> **Phase:** Phase 5 — Live Receipt Transfer; this work completes the arc's private half
> **Entry:** the reproduced `NoConfidentialPredecessorCanBeFunded` candidate-pipeline blocker and an accepted charter decision on test-material custody
> **Primary semantic operation:** materialize and fund confidential candidate test receipts
> **Affected packages:** `target-elements-conformance`, `transaction`, `vectors`; the native Elements executor where it implements the first-party funding wire protocol
> **Evidence support:** the first-party target executor and validated-report boundary, the Guide-13 evidence registers, and Elements consensus behavior verified at reviewed tip `b7fc5d080a`
> **May affect after acceptance:** funding schema, executor capabilities, candidate transaction construction, output-witness serialization, test-opening custody, canonical report exclusions, evidence plans, package contracts, Meson graph, and dependency graph
> **Supersedes as concept direction:** scalar-only funding and per-output confidential commitment materialization for Guide-13 private candidate evidence
> **Does not implement:** the owner sighash, production wallets, production opening custody, production blinding, production signing or blinding coordination, privacy guarantees, final calibration, production deployment, or release
> **Required result:** one reviewed candidate-only protocol that creates, mines, reads back, spends, and evidences explicit-protocol-asset/confidential-value receipts with valid rangeproofs, empty surjection proofs, transaction-wide balanced value blinders, proof-bearing output witnesses finalized before owner signing, and honest evidence boundaries
> **Authority:** Attestation, the typed architecture, implemented ADRs, and accepted decisions take precedence over this concept; the project owner resolves every charter decision recorded below
> **Trust boundary:** repository source and selected executors are executable; untrusted contributions run only in an externally established, credential-free environment under ADR-015, and any selected secret-bearing interface must pass ADR-015's separate design gate before implementation

---

## Mission · `sec:guide-ctf:mission`

This concept charters the work needed to make the Guide-13 private receipt form creatable, spendable, and evidencable end to end in the candidate pipeline while preserving honest separation between funding evidence, transfer acceptance, disclosure minimality, and production readiness.

The required target form is exact:

```text
asset:
    explicit protocol asset U

value:
    33-byte confidential commitment with prefix 0x08 or 0x09

output witness:
    valid rangeproof
    empty surjection proof
```

The work begins at predecessor funding and ends only when the candidate path consumes the verified opening, balances and proves all confidential outputs, freezes them before owner signing, submits a complete successor, and reports only the boundary exercised. Mined funding proves materialization; an accepted successor additionally requires the separate owner sighash; safety and minimality rows additionally require their own complete relations and evidence.

---

## One-line thesis · `rem:guide-ctf:thesis`

> The confidential-funding gap closes when one owner-selected custody and reproducibility model drives a fail-closed tagged funding protocol and one transaction-wide deterministic materializer, the target mines and reads back the exact explicit-asset/confidential-value form, proof-bearing outputs become immutable before the separate owner-sighash step, and every report refuses to claim transfer or minimality evidence before those independent conditions are met.

---

## Scope and ownership · `sec:guide-ctf:scope`

The primary defect owner is the first-party funding boundary in `target-elements-conformance` and its native adapter. `TargetFundingSubject` and `FundedOutput` speak only integer amounts, while the adapter constructs and reads back explicit values; the protocol cannot faithfully request or report a confidential predecessor.

Ownership remains divided by existing responsibilities:

- `target-elements-conformance` and the native adapter own protocol negotiation, construction, target readback, response validation, and funding evidence;
- `transaction` owns candidate serialization, confidential output finalization, and the replacement for per-output commitment construction;
- `vectors` owns ceremony fixtures, typed blockers, and evidence restart; a pre-target refusal is never a target verdict;
- `tapscript` already recognizes both parities, and Elements Core already defines the target rules; neither owns this funding gap.

The existing `NoConfidentialPredecessorCanBeFunded` value remains the honest blocker until the new predecessor slice is mined and read back. `OwnerSighashNotComputable` remains independently authoritative for private positive rows until the parallel digest work closes. Neither blocker may be deleted merely because a new type or helper exists.

The concept does not reopen Guide-13 receipt semantics, explicit `U`, live-class closure, target CT conservation, sponsor isolation, or evidence-role separation.

---

## Verified target boundary · `sec:guide-ctf:target-boundary`

The target-side claims in this concept were checked against Elements tip `b7fc5d080a` rather than inferred from RPC names.

Elements consensus accepts the selected hybrid form. In `src/confidential_validation.cpp:368-403`, a confidential value paired with an explicit asset uses the unblinded asset generator, requires a rangeproof, and requires the surjection-proof field to be empty. `src/script/sigcache.cpp:131-163` rejects an empty or invalid rangeproof. `src/primitives/confidential.h:13-84` and `src/primitives/confidential.h:129-154` admit value-commitment prefixes `0x08` and `0x09`.

No reviewed stock RPC produces that exact form. The common path in `src/blind.cpp:557-627` draws fresh value and asset blinders, blinds the asset, creates a fresh ephemeral nonce, generates a rangeproof, and generates a surjection proof. Both wallet and raw blinding RPCs enter that common path; none of their reviewed parameters selects confidential value with an explicit asset or supplies deterministic blinding and nonce inputs.

The existing generic confidential-conservation workload is reference material, not the solution: it uses the fully confidential RPC path, returns disposable wallet openings, and reports `FixtureInputsOnly`. Its typed outcome vocabulary is reusable, but it cannot fund explicit-`U` receipts or preserve byte-identical ceremony evidence.

These findings require a reviewed deterministic materializer. Calling `blindrawtransaction`, retrying ordinary wallet blinding until an output happens to look useful, or mutating a fully blinded asset back to explicit after proof construction does not meet the charter.

---

## First charter decision: custody · `rule:guide-ctf:custody-decision`

The project owner selects the custody model before wire fields, process boundaries, diagnostics, or materializer placement are designed. This is the first chartered decision because it determines whether the interface carries public disposable fixtures or secrets and therefore whether ADR-015's future-secret design gate applies.

| Model | What crosses the construction boundary | Consequence |
|---|---|---|
| Deterministic central public fixture | Reproducible labels, digests, and public test openings | Fits the current model and byte-identical ceremony; proves target feasibility, not constructor privacy. |
| Disposable wallet-held | A development wallet retains openings | Native custody and strong randomness, but selects the external-wallet model, still lacks the stock hybrid form, and revises reproducibility. |
| Adapter-local opaque handles | The adapter privately retains openings | Hides raw openings from the wire but adds lifecycle, erasure, diagnostics, and process authority; secret treatment invokes ADR-015. |
| Cooperative or distributed | Owners exchange openings or jointly blind | Adds a multi-party secret protocol with trust, liveness, abort, and transcript rules; ADR-015 applies. |

**Recommendation.** Select deterministic central public fixtures. This matches Guide 13, permits independent recomputation, preserves byte identity, and keeps disposable scalars honestly public. Record the domain separator, derivation recipe, labels, scalar rules, lifetime, and destruction with the chain.

**Ruling required.** The ruling names opening ownership, lifetime, lookup authority, diagnostics, production separation, and ADR-015 disposition. Any nonrecommended model blocks wire implementation until its process boundary is accepted.

---

## Second charter decision: determinism · `rule:guide-ctf:determinism-decision`

The current ceremony has a proven field-identical rerun. Stock blinding uses fresh blinder and nonce randomness, so it cannot satisfy that contract. The owner must choose one of two explicit reproducibility contracts:

- **Preserve byte identity.** Derive each public-fixture blinder, nonce input, proof input, order, and retry counter deterministically; equal inputs produce equal funding and successor bytes and report `MaterializedBytes`.
- **Revise reproducibility.** Permit wallet or adapter randomness and compare semantic fixtures, retained per-run bytes, verified openings, and target projections; explicitly revise the ceremony, evidence schema, and field-identity claims.

**Recommendation.** Preserve byte identity. This keeps the proven ceremony claim, makes parity fixtures reviewable, and avoids retrying nondeterministic RPC output.

The recipe is domain-separated by role and case. It derives all but one output blinder and solves the final blinder from transaction-wide balance. Degenerate scalars, identity commitments, proof failure, and unavailable parity cause typed refusal or bounded deterministic search, never hidden randomness.

Revised reproducibility must precede wire work and name every retired byte-identity assertion; implementation may not downgrade silently.

---

## Representation-tagged funding wire · `sec:guide-ctf:wire`

The funding request becomes a representation-tagged union. The concept fixes its meaning, while the execution guide fixes exact Rust and serialized field names.

```text
funding request
    explicit
        issue-or-name explicit asset
        output program and count
        amount per output

    confidential_value
        issue-or-name explicit asset
        one or more destination programs
        public fixture identity and bound fixture digest
        selected materializer and determinism profiles
        required value form: commitment
        required asset form: explicit
        required proof form: rangeproof present, surjection proof absent
```

The confidential member is not a boolean beside `amount_per_output`: a distinct arm prevents scalar/private ambiguity and contradictory members.

The response becomes a matching union:

```text
funded output
    explicit
        outpoint, explicit asset, amount, script

    confidential_value
        outpoint, explicit asset, value commitment, nonce, script
        transaction and output-witness binding
        rangeproof binding and empty-surjection-proof observation
        fixture identity and digest, never its opening
```

The response binds witness-bearing transaction identity, output index, explicit asset, commitment, nonce, script, rangeproof, and absent surjection proof to raw mined-transaction readback rather than trusting a request echo.

The schema revision and capability rule fail closed:

- old executors receive only their old explicit schema; confidential requests require the new schema and advertised capability;
- unsupported profiles refuse before construction, and unknown tags, profiles, or members fail strict framing;
- request and response arms must match; explicit fallback from a confidential request is a protocol error;
- wrong value or asset form, proof shape, count, script, asset, or readback causes typed response refusal;
- compatibility may translate old explicit requests only to the explicit arm, never confidential requests backward.

Typed refusals distinguish at least: confidential capability absent; hybrid representation unsupported; fixture unknown; fixture digest mismatch; deterministic materialization refused; response arm mismatch; confidential asset returned; explicit value returned; rangeproof missing or invalid; surjection proof unexpected; and mined readback mismatch.

Construction, infrastructure, consensus, relay policy, and accepted-and-mined results remain distinct layers.

---

## Third charter decision: canonical request evidence · `rule:guide-ctf:canonical-request-decision`

Guide-13 validated reports currently bind exact sent requests, while private evidence excludes exact receipt amounts and openings from canonical reports. Reusing the scalar request violates the exclusion; redacting that request after execution violates exact-request evidence. The charter must resolve the tension before the schema is implemented.

The available choices are:

- **Canonical public-fixture handle and digest.** Send a stable non-semantic case identity, profile, and digest binding a separate fixture; retain the exact request and response but exclude the fixture attachment.
- **Canonical redaction with a binding envelope.** Send construction material, replace it in the report with a verified commitment, and revise exact-request semantics.
- **Noncanonical exact attachment.** Retain only a request digest canonically and keep exact material under separate retention and access rules.

**Recommendation.** Use the public-fixture handle and digest. The adapter resolves only registered test fixtures; validation binds handle, digest, profile, exact request and response, and mined transaction without serializing an amount or opening.

This is a public deterministic test identity, not an adapter-local secret handle; its spelling encodes no amount, its digest detects drift, and lookup failure is a construction refusal.

Any alternative ruling names its schema change, validation proof, retention, and disclosure effect before wire work.

---

## First implementation slice: one predecessor transaction · `rule:guide-ctf:predecessor-slice`

The first implementation wave proves only that the target-required predecessor form can be deterministically created and mined. It contains one funding transaction with exactly two protocol receipt outputs:

- both outputs carry the same explicit protocol asset `U` and positive confidential values whose semantic sum equals the explicit input amount;
- both outputs use zero asset blinders and therefore carry no surjection proofs;
- an explicit predecessor asset input contributes a zero value blinder, and the two confidential output value blinders are deterministic additive inverses so their sum is zero;
- a bounded deterministic fixture-counter search chooses an accepted scalar pair whose serialized value commitments include one `0x08` prefix and one `0x09` prefix;
- each output has its deterministic nonce field and a valid rangeproof bound to its value commitment, explicit asset generator, and output program;
- the transaction is submitted, accepted, mined, and read back from the target as raw proof-bearing bytes.

Any policy-asset fee or change belongs to an explicitly classified non-protocol region and must not alter the two-`U` commitment equation. Output order is fixed by the fixture, not by commitment prefix, amount, or a nondeterministic retry result.

Independent recomputation and adapter readback agree on asset, commitment, parity, nonce, program, proof shape, outpoint, and witness transaction identity.

The slice reports only:

- confidential funding capability and schema negotiation;
- deterministic materialization under the selected custody profile;
- exact hybrid representation of both mined outputs;
- both accepted commitment parities;
- valid proof-bearing predecessor outputs and target readback;
- a stable opening reference usable by the later transaction-wide materializer.

The slice reports no transfer, authorization, CT transfer conservation, Guide-13 acceptance, minimality, production privacy, or matrix discharge. It clears only `NoConfidentialPredecessorCanBeFunded` after validated readback, never `OwnerSighashNotComputable`.

---

## Transaction-wide private materializer · `def:guide-ctf:materializer`

The current per-output capability sees no predecessor opening, cannot balance destinations, returns only commitments, fixes null nonces, and cannot carry proofs. Private construction replaces that role with one transaction-wide capability.

The capability consumes the complete candidate construction intent at once:

- all input outpoints, target-observed fields, confidential openings, and explicit amounts with zero blinders;
- all destination amounts, explicit assets, programs, fixture identities, sponsor, fee, and change roles;
- the selected deterministic materializer, proof, nonce, order, and retry profiles.

It validates before construction:

- predecessor openings recompute target commitments, and each fixture identity and digest binds uniquely to the case;
- all protocol inputs and outputs carry explicit `U`, semantic amounts conserve, and complete target families are classified;
- destination values satisfy target proof policy and the selected construction recipe matches the charter.

It materializes the transaction as one balance problem:

1. derive deterministic output value blinders for all but the balancing output;
2. solve the final output value blinder so the output sum equals the confidential input blinder sum for the explicit asset generator;
3. keep every protocol asset blinder zero and reject any confidential-asset result;
4. construct every value commitment and independently compare it with the public-fixture expectation;
5. derive deterministic nonce material and generate a valid rangeproof for each confidential value;
6. serialize the complete output-witness vector with each rangeproof and an empty surjection-proof field;
7. freeze inputs, outputs, nonce fields, output witnesses, version, locktime, sponsor region, and fee region into one materialized candidate.

The typed result carries exact proof-finalized bytes, fixture and materializer profiles, an opening-binding census, and signer inputs without exposing openings.

Encoding must round-trip nonempty rangeproofs and empty surjection proofs exactly; dropped or refused proof bytes and per-output-only results fail the boundary.

Typed refusals distinguish missing or mismatched openings, semantic or blinder imbalance, invalid scalars or commitments, bounded parity exhaustion, nonce or proof failure, serialization mismatch, and post-finalization mutation.

The execution guide names cryptographic ownership, an independent commitment check, and dependency direction; one computation cannot count as observation and expectation.

---

## Owner-sighash boundary and handoff · `rule:guide-ctf:sighash-boundary`

This guide does not implement, select, or declare accepted the owner sighash. That work proceeds under its own review and typed blocker.

The interaction is nevertheless fixed. Elements `SIGHASH_ALL` incorporates both the serialized output set and the hash of the output-witness vector at `src/script/interpreter.cpp:2414-2425`, `src/script/interpreter.cpp:2568-2578`, and `src/script/interpreter.cpp:2741-2744`. The exact rangeproof and surjection-proof fields covered by the selected profile must therefore be final before any receipt owner signs.

The handoff order is mandatory:

```text
resolve and verify predecessor openings
    ↓
materialize all inputs, outputs, commitments, nonces, and output witnesses
    ↓
freeze one proof-finalized candidate transaction
    ↓
hand the exact protected transaction to the separately reviewed owner-sighash component
    ↓
collect every required owner authorization over that same candidate
    ↓
submit without changing any protected byte
```

Digest review and deterministic materializer development should proceed in parallel. Integration waits until proof finalization and the selected owner-sighash profile are both independently accepted. An explicit-only signing success does not establish proof-bearing signing, and a valid confidential funding transaction does not establish any owner digest.

The handoff binds exact proof-finalized bytes, rejects responses for another candidate, and supplies target spent-output data without openings; this concept imposes no digest need for openings.

Any protected mutation after signing starts is a construction refusal; no proof is repaired, reblinded, or regenerated after a signature.

---

## Evidence contract · `sec:guide-ctf:evidence`

The guide produces separate typed evidence for funding materialization and for the later candidate transfer. The former is new; the latter resumes the existing Guide-13 registers only after all of their prerequisites are real.

A validated funding record binds target, deployment, handshake, capability, schema, selected profiles, exact request and response, fixture handle and digest, witness-bearing transaction, submission and mining, raw readback, both output fields and proof shapes, independent commitment results, observed outcome layer, exclusions, and non-claims. It carries no amount or opening.

Validation recomputes arm agreement, fixture binding, output order, representation, prefix mask, proof shape, readback, and summary; raw DTOs and caller-authored outcomes cannot discharge evidence.

Canonical reports exclude private amounts, openings, blinders, nonce and proof inputs, wallet data, credentials, environment values, and diagnostics. Public fixture data remains in its chartered attachment, not a schema that excludes it.

Funding evidence, CT conservation evidence, owner-signature evidence, safety evidence, minimality evidence, resource evidence, and lifecycle status do not substitute for one another. The summary must preserve that separation even when one ceremony records several of them.

After sighash integration, evidence restarts with one accepted sponsorless private one-to-one control, then both parities, CT balance, proof and blinder faults, remaining accepted shapes, signer-ready sponsor cases, and finally minimality pairs.

A proof-negative mutates one field of a balance-valid control; a transaction already failing commitment balance cannot attribute its verdict to the rangeproof.

---

## Quantified unblocking and non-claims · `sec:guide-ctf:nonclaims`

Confidential funding alone clears one carried residual: `NoConfidentialPredecessorCanBeFunded`. It establishes that a guide-shaped predecessor can exist on chain and that the ceremony can retain a verified opening reference. It moves zero Guide-13 matrix rows because all positive private rows remain blocked by the independent owner-sighash component, and sponsor cases retain their signer dependency where claimed.

The materializer makes ten private-positive fixtures constructible and enables CT/proof mutations, but moves zero positive rows before owner authorization and target acceptance.

This guide never claims:

- production opening, key, nonce, seed, blinder, wallet, or credential custody;
- production-quality randomness, blinding, proof generation, signing, erasure, or side-channel resistance;
- a production multi-owner blinding or signing protocol;
- owner anonymity, graph privacy, count privacy, timing privacy, wallet privacy, or universal transaction confidentiality;
- that public deterministic fixture openings are secret from an observer;
- that the stock Elements RPC surface supports the selected hybrid representation;
- that funding evidence proves a live transfer, safety relation, disclosure-minimality relation, or resource result;
- that malformed private rejections substitute for an accepted positive private transaction;
- that the owner-sighash profile is implemented or reviewed by this guide;
- that a candidate interface is final, stable, production-capable, or released.

The result remains candidate-only; a secret-bearing selection stops at ADR-015 rather than weakening the boundary.

---

## Wave 0 — Decide custody and reproducibility · `task:guide-ctf:wave0`

**Deliverables**

- project-owner ruling selecting one of the four custody models;
- opening owner, lifetime, process boundary, lookup authority, diagnostics, and ADR-015 disposition;
- project-owner ruling preserving byte identity or explicitly revising reproducibility;
- reviewed fixture domain separators, derivation inputs, bounded retry rules, and determinism level;
- canonical-request evidence option selected with its disclosure and retention consequences;
- affected package and dependency boundary recorded without implementation by implication.

**Suggested commit**

```text
plans: decide confidential test-material custody
```

---

## Wave 1 — Revise the funding wire · `task:guide-ctf:wave1`

**Deliverables**

- representation-tagged request and response unions;
- confidential-funding capability and schema migration;
- exact hybrid-form profile and public-fixture binding;
- fail-closed legacy compatibility;
- typed request, construction, response-shape, and readback refusals;
- canonical report validation for exact handle-and-digest requests or the accepted alternative;
- mock executor and protocol contract cases for every union arm and mismatch.

**Suggested commit**

```text
target-elements-conformance: type confidential funding
```

---

## Wave 2 — Prove one confidential predecessor · `task:guide-ctf:wave2`

**Deliverables**

- one deterministic funding transaction with two explicit-`U`, confidential-value outputs;
- opposite-sum value blinders and zero asset blinders;
- one `0x08` and one `0x09` commitment under the bounded fixture recipe;
- deterministic nonce fields, valid rangeproofs, and empty surjection proofs;
- target acceptance, mining, raw readback, and independent commitment comparison;
- validated funding-only evidence carrying all non-claims;
- `NoConfidentialPredecessorCanBeFunded` cleared only at its owning boundary.

**Suggested commit**

```text
vectors: record confidential predecessor funding
```

---

## Wave 3 — Materialize private transactions transaction-wide · `task:guide-ctf:wave3`

**Deliverables**

- transaction-wide capability consuming every selected input opening and destination;
- predecessor-opening verification and complete family classification;
- deterministic balanced output blinders with explicit assets;
- commitment, nonce, and rangeproof materialization;
- proof-bearing output-witness serialization and exact decode round-trip;
- proof-finalized candidate type and post-finalization mutation refusals;
- independent commitment checks and focused valid, imbalance, blinder, proof, and serialization cases;
- old per-output commitment role removed from private complete-transaction claims.

**Suggested commit**

```text
transaction: finalize confidential candidate witnesses
```

---

## Wave 4 — Close the external sighash handoff · `task:guide-ctf:wave4`

**Entry condition**

- the separately reviewed owner-sighash profile has reached its own accepted result.

**Deliverables**

- proof-finalized candidate bytes enter the signing request unchanged;
- output-witness commitment is independently tested by the sighash work;
- valid, wrong-owner, wrong-candidate, post-proof-mutation, and post-signing-mutation cases;
- every required owner signs the same protected candidate;
- one complete sponsorless private candidate becomes submit-ready;
- explicit statement that the digest implementation belongs to the parallel sighash work, not this guide.

**Suggested commit**

```text
transaction: bind private candidates to reviewed owner signing
```

---

## Wave 5 — Restart Guide-13 evidence · `task:guide-ctf:wave5`

**Deliverables**

- one accepted sponsorless private one-to-one control before negative cases;
- both predecessor commitment parities exercised in complete successors;
- target CT conservation, wrong-blinder, missing-rangeproof, and malformed-rangeproof cases with attributable layers;
- remaining positive private shapes only where each accepts;
- sponsor cases only after their independent signer dependency closes;
- semantic projections and exact report exclusions validated;
- disclosure-minimality pairs only after both sides accept;
- Guide-13 blockers and matrix rows updated from observed evidence, never from construction capability alone.

**Suggested commit**

```text
vectors: restart private live-transfer evidence
```

---

## Acceptance and rejection · `sec:guide-ctf:acceptance`

The guide is acceptable for translation into an execution guide only when:

- the owner has recorded custody, determinism, canonical-request, sequencing, and any ADR-015 rulings;
- the wire union has one exact hybrid meaning and a fail-closed migration;
- the first slice specifies deterministic opposite-sum blinders, zero asset blinders, both parities, valid rangeproofs, empty surjection proofs, mining, and raw readback;
- the transaction-wide capability consumes all openings, balances blinders, serializes proofs, and freezes the complete protected candidate before signing;
- the owner-sighash boundary is external, parallel, and ordered after proof finalization;
- funding-only evidence and zero-row movement are explicit;
- the six waves preserve the required dependency order;
- production and privacy non-claims are carried by the result, not left to implication.

Implementation acceptance requires one mined predecessor, one accepted sponsorless private one-to-one successor after external sighash closure, independently validated observations, and no stronger claim.

Reject or stop the guide when:

- custody is implicit, ADR-015 is bypassed, or nondeterminism lands without an accepted contract revision;
- stock blinding is claimed to produce the hybrid form, or the confidential request exposes an amount or opening;
- legacy fallback, an unbound mined response, or per-output-only materialization is accepted;
- any protocol asset blinder is nonzero, the asset is confidential, the rangeproof is absent, or the surjection proof is present;
- protected bytes change after signing, or funding capability is counted as a matrix pass;
- malformed rejection replaces a positive control, one computation supplies observation and expectation, or production/privacy claims appear.

---

## Identity, schema, security, and handoff · `sec:guide-ctf:impact`

**Identity.** This unnumbered concept adds no architecture operation, phase, release identity, or digest. At charter time the owner sequences it against the queued concept and parallel sighash work; all new types remain subordinate to existing identities.

**Schema.** Tagged request and response semantics require deliberate migration; report versioning follows if the selected evidence policy needs it. Old explicit records retain their original schema.

**Security.** Deterministic fixtures are public, disposable, test-only, unrelated to production, and destroyed with the chain. Real secret retention or transport requires ADR-015; diagnostics and canonical reports always exclude openings.

**Dependencies.** The execution guide names materializer library, package, process, independent checker, and graph effect; this concept does not approve a dependency by naming the need.

**Handoff.** The execution guide translates rulings, unions, refusals, evidence, finalization, and waves into APIs and tests. Guide-13 evidence resumes only when predecessor, materializer, and external sighash meet in one finalized candidate.

The final report records selected models, mined target facts, consumed sighash result, cleared blockers, moved rows, and remaining non-claims. A typed stopped result is valid; an overstated one is not.
