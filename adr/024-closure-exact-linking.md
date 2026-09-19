# ADR-024: Closure-Exact Emission and Linking

**Status:** Decided and adopted for the STATE generation; implementation lands with the Wave-6 reduction bite; earlier generations under audit.
**Scope:** What an emitted target program and its linked form are permitted to check, in every generation this repository emits.
**Supersedes:** For the STATE generation, the census entries `StateCardinalityV1`, `StateSponsorIsolationV1` and `StateIssuanceAbsenceV1` at (`[PLAN-tab:guide14-exec:pattern-census]`), and the exact-input-count, exact-output-count, sponsor-suffix, sponsor-change-role, fee-role and no-unclassified-position clauses at (`[PLAN-rule:guide14-exec:coordinator]`). The guide text stays archived verbatim and is not edited; the supersession is recorded on the Phase-6 card and in the conceptual register by the wave's opening record.
**Does not establish:** No change to the compact or live generations, which are audited against this record by their own backlog row; no constructor migration, and no generations model. What it does change is the scope of five relations of this operation — `AllowedObjectFamilies` on each side, `SponsorIsolation`, `CanonicalDeltaPolicy`, `OpenFlowPolicy` and `SponsorEnvelopeMultiplicity` — which become statements about the region the operation claims rather than about the whole observed transaction, by a staged refit; no other relation changes, and no relation's content changes.

---

## Context · `sec:closure:context`

The maturity-announcement leaf pins the whole transaction shape. `counts` in `packages/tapscript/src/state_pattern.rs` writes the exact input count and the exact output count as literals and then compares the shape's own sponsor count against the `FeeSponsorInputMax` symbol; `partition` checks the asset at every sponsor position, the witness version and witness program of the change output, and the digest of the fee program at the last output; `issuance` sweeps every input for an empty issuance; `StateCoordinatorRoleV1` adds the self-position pin, the current input index compared against zero. `StateStructuralEvidence` names what the emitter believes it buys: no second STATE object, no position for another object family, no output for another family, no destruction or specialized event role — and its own documentation says the inspected counts and the asset partition are what leave no additional position.

The question all of that answers is what a sponsor could do to the singleton. Three facts already in the tree answer it without a single one of those checks.

The executing input is pinned to index zero. A second STATE object cannot be spent anywhere else in the same transaction, because its own covenant refuses to run at any other index, and the constructor admits no other way in: the production static subtree is exactly one leaf under ruling 34 at (`[PLAN-rule:phase6:wave5-rulings]`), and `StateInternalKeyPolicy` in `packages/tapscript/src/state_constructor.rs` fixes a publicly derived internal key with `KeyPathPolicy::NoAcceptedEscape`. That policy states its own residual rather than overclaiming — discrete-log hardness and preimage resistance, with no proof that a key path is impossible — and the argument here inherits exactly that residual and no more.

The singleton cannot be minted. `packages/architecture/src/spec.rs` declares the PID asset with a fixed amount of one and `reissuable: false`, so no input of any transaction can issue more of it.

Target consensus conserves every asset per transaction. With one PID input and output zero receiving its exact amount, every other output of that transaction carries zero PID, whatever else it carries. This is not a new assumption: `StateExternalEvidenceRole::SubstrateConservation` already names it as the role that establishes reserve conservation without sponsor-value reads, and realization's sponsor-isolation evaluator in `packages/realization/src/evaluate.rs` already declines to re-derive it, on the ground that Elements validates that every transaction balances and that re-deriving it would duplicate the base layer's own job while reading exactly the amounts sponsor erasure removes from the protocol read-set.

What the shape pin still buys, once those three facts are counted, is the exclusion of other operations from the same transaction. That is model fidelity, not singleton safety, and it is worth what it costs only if composition is something the system means to forbid.

The exact counts are also not a property of the contract. The announce-maturity declaration in `packages/realization/src/declarations/announce_maturity.rs` says exactly one STATE object on each side, at most `FeeSponsorInputMax` sponsor inputs, and at most one sponsor change output. It never says how many inputs a transaction has. On a target with positional introspection and no loop primitive and no aggregate, a universal over positions can only be written by fixing the domain and unrolling it, so the exact count is the quantifier's encoding, not the thing being said. The comparison against `FeeSponsorInputMax` shows the same seam from the other side: both of its operands are fixed before the transaction exists, one by the shape and one by the link, so the runtime bytes decide a question the linker could have decided.

That the domain has to be fixed is not a claim about this operation. `packages/tapscript/src/shape.rs` states it from the compact side: the reviewed target has no general loop primitive and the guide assumes no general conditional dispatch, so a backend cannot emit one program that handles a range of input counts — it emits one program per shape, and the shape is what selects the program. A shape is therefore a typed key into a family of emitted programs, and reading the key back out of the emitted bytes as a contract clause confuses the index with the thing indexed.

The compiler already models sponsorship without a count. `SponsorCase` in `packages/compiler/src/case.rs` has two values and says why — one sponsor input and three sponsor inputs with change share every active relation and every active source row, so they are one placement case — and `packages/compiler/src/sponsor_region.rs` treats sponsorship as a flow role rather than an object family, with individual amounts erased inside the region.

The earlier generation reached the same conclusion for the constructor's own program. `apply_cycle_policy` in `packages/linker/src/graph.rs` refuses a self-committing literal outright as `ImpossibleStaticFixedPoint`; `SelfCommitmentStrategy::IdentityIntrospection` in `packages/linker/src/deployment.rs` records the resolution — a program that introspects the input it is spending has no need of a literal committing to itself, so the edge disappears rather than being resolved — and `require_program_matches_this_input` in `packages/tapscript/src/pattern.rs` is that resolution emitted. The live generation records the same shape of argument for a different reason: `packages/tapscript/src/live_pattern.rs` explains that recognition is local because a literal for another input's program would be a mutual fixed point no deployment can supply. In each case a check was removed because something else already established what it claimed.

---

## Decision · `dec:closure:exact-linking`

An emitted program and its linked form carry exactly the checks the contract's closure requires, and nothing more.

A check beyond closure is a defect of the same kind as a missing one. The covenant's accepted set must equal the contract's permitted set: a missing check widens it and a surplus check narrows it, and a narrowed covenant refuses transactions the contract permits. The surplus is not free in any other dimension either — it adds symbols to resolve, push sites to bind, bytes to the leaf and weight to the resource projection, and it turns an encoding choice into a deployment constraint, because a leaf specialized to a shape must be re-linked to admit a shape the contract already allowed.

Closure is compositional. Every family's leaf is locally sound: it constrains its own input, its own successor and its own asset, and nothing else. Consensus conserves assets per transaction. Any transaction that satisfies every involved leaf therefore satisfies every involved family's invariant, and the composition of several operations in one transaction is permitted rather than excluded. Local soundness is the stated obligation of every later family's leaf, and a family whose leaf is not locally sound is the defect, not the transaction that composes with it.

Permitting composition has a consequence for the model, and this record carries it rather than leaving it to be discovered. Five of the announcement's relations are evaluated over the whole observed transaction and not over the positions the operation claims: `AllowedObjectFamilies` on each side, which requires every observed object on that side to belong to an allowed family; `SponsorIsolation`, which reads the observation's flows entire; `CanonicalDeltaPolicy`, which compares the transaction's canonical partition against the operation's expected set; `OpenFlowPolicy`, which requires every open flow the transaction carries to be an allowed kind; and `SponsorEnvelopeMultiplicity`, which counts the transaction's sponsor flows. Left as written while composition is permitted, each of them refuses a transaction the covenant accepts, so the accepted set exceeds the permitted set — this record's own defect, mirrored. An operation's transaction-side relations therefore quantify over the region the operation claims, region membership enters the observation, and a transaction is the union of its operations' regions.

Adoption is staged, because the leaf reduction is safe without the refit while the refit is not needed for safety. The reduction lands first: every check it removes is one the singleton's safety argument does not need, and until the refit lands the model still describes single-operation transactions exactly as before. The discharge table classes those five relations MODEL-SCOPE — evaluated by the realization over the whole observed transaction, enforced by no leaf, retired by the refit — so the gap is visible on every published closure instead of resting in prose. The region-scoping refit of the realization is its own backlog row, opened by the wave's opening record. Until that row lands, a composed transaction is accepted on-chain and outside the model, and this record says so rather than implying that the model already covers it.

"Required for closure" is derived, never asserted. The linking of a generation publishes a discharge table with one row per realization relation of the operation, each row naming what discharges it: a positional check in the leaf, the self-position pin, consensus conservation, a named external evidence role, a semantic component, or MODEL-SCOPE. The carrier closure formalizes that table. The mechanism is not new, only sharper: `project_maturity_carriers` in `packages/tapscript/src/maturity_assessment.rs` already maps one carrier to every relation, emitted component or external requirement, and refuses an unmapped relation. What the discharge table adds is the other direction — a check in the emitted bytes that appears in no row is surplus and comes out.

The deployment facts the argument rests on are named external evidence roles beside substrate conservation, not silent assumptions: the singleton's non-reissuable declaration, and the fact that its issuance placed the whole amount under the constructor, which is the induction base for output closure. Naming them is what keeps the reduction honest, because each is a fact about a deployment rather than a fact the leaf checks.

No count bound is carried in the leaf. A bound over the whole transaction's inputs is itself a surplus check once composition is permitted, because it refuses a transaction whose further sponsor positions belong to another operation's region, and replacing an unrolled sweep with one inequality would only make the surplus cheaper rather than remove it. `SponsorEnvelopeMultiplicity` is therefore MODEL-SCOPE with the other four until the refit re-scopes it to the operation's own region or retires it: it is resource policy, and its retention is the realization's question, not the linker's.

---

## Consequences · `rem:closure:consequences`

For the STATE announcement, the Wave-6 reduction bite removes the exact input and output counts, the sponsor-suffix asset sweep, the committed change program and version, the fee-program digest, the shape-literal comparison against the sponsor maximum, and the per-input issuance sweep.

Recognition keeps the self-position pin, input zero's asset, explicit amount and script version, and output zero's asset, explicit amount and authenticated successor program — the output-zero three already established by the `SuccessorReconstruction` component in `packages/tapscript/src/state_announcement.rs`, not by the partition, whose output-zero asset check is a duplicate of it. The semantic components and operator authorization are untouched.

The predecessor-program literal goes with the rest, and it goes by this record's own principle rather than as an extra economy. It is surplus: the semantic `MetadataAuthentication` component already establishes the consumed program, by introspecting input zero's own witness program and verifying the tweak against the internal key and the authenticated metadata, and that is the strictly stronger check — it binds the program to what commits to it, where an equality against a literal compares bytes and nothing else. It is also unlinkable, for the reason the compact generation recorded before it: the literal's bytes would have to appear inside the tree that commits to them, which is the fixed point `apply_cycle_policy` refuses outright rather than search for.

The four absence facts of `StateStructuralEvidence` become one. That no second STATE object occurs is discharged by the self-position pin, the non-reissuable declaration and consensus conservation. The other three said that no position remains for another family, and they were true only because the leaf had claimed every position; under compositional closure they are not facts about this operation at all, and the emitter says so rather than continuing to claim them.

The symbol and site census is recounted after the change. The pre-change figures recorded at (`[PLAN-sec:phase6:wave5-findings]`) — twelve distinct unresolved symbols over 46 fixture push sites, of which eight structural symbols over 33 sites — are the baseline the recount is read against, and the reduction is expected to show in both the symbol count and the site count rather than in bytes alone.

One over-approximation disappears with the checks that carried it. Realization's `SponsorIsolation` asks that a sponsor destination be a declared ordinary-L-BTC object with an erased value, which is a statement about family and role; the leaf asked instead that output one carry one exact witness program. Equality with a single program is a far stronger demand than the relation makes, and it fixes where a sponsor's change may go as a condition of announcing maturity.

For the guide, the three census entries of §10.1 and the six coordinator clauses named in the header are superseded for the STATE generation only. The guide's four-sided carrier wording is unaffected, and so is §9.7's prohibition on searching for a fixed point by repeated hashing at (`[PLAN-rule:guide14-exec:constructor-graph]`) and §17.3's refusal of constructor migration at (`[PLAN-rule:guide14-exec:bundle-continuity]`).

For earlier generations, the compact and live shapes keep their region counts. Compact ASH aggregates, so its emitter unrolls a sum over a fixed batch and the count is what makes the unrolling writable; a live transfer admits split, merge and redistribution, so its output count is a second free axis its multi-destination logic reads. Those counts are required for closure on their own terms. Their sponsor-region checks are a separate question and are audited against this record by a backlog row the wave's opening record opens; no such row exists yet, and this record states the obligation rather than citing an artifact.

The reduction inherits the residuals of the facts it leans on and states them rather than absorbing them. The self-position argument is sound up to the internal-key policy's own residual, and conservation is external evidence the leaf does not verify, which is why it is a named role. A reduced leaf therefore claims no more target enforcement than the maturity assessment already claims: fewer bytes, the same honesty about what the bytes do not establish.

For the estate, a "strange sponsor" is not a threat class. The threat classes are a second family object at a position the leaf does not own, and a mint of the singleton. Both are closed locally, by the pin and by the asset declaration, and neither is closed any better by counting positions.

---

## Rejected alternatives · `rem:closure:rejected`

**Keep the shape pin as composition exclusion.** It buys model fidelity that the compositional argument already provides, and it charges symbols, push sites, bytes and resource weight for it, plus a deployment constraint on every shape the contract permits and the leaf does not. It also decides, as a side effect of excluding other operations, where a sponsor's change output may go — a restriction on the sponsor that no relation asks for.

**Dispatch between shapes inside the one leaf at runtime.** Every branch's checks would sit in the bytes of every spend, so the cost is the sum of the shapes rather than the maximum. It needs a branching construct no emitter in this repository has, and the target has no loop primitive to fold the branches back together. Once the shape pin is gone there is also nothing left to dispatch on.

**Emit one leaf per sponsor shape.** This contradicts the one-production-leaf ruling at (`[PLAN-rule:phase6:wave5-rulings]`), and it multiplies the control recipes, the resource projection and the golden evidence by the size of the shape set. It is the same over-specification the exact counts already are, spread across more leaves.

---

## Adoption gate · `gate:closure:adoption`

Adoption holds when:

- the reduction bite's discharge table names a discharge for every realization relation of the announcement, and the carrier-closure tests check the table rather than restating it;
- the composed leaf's golden evidence is re-run over the reduced leaf, and the closure tests show that a transaction carrying additional inputs and outputs of any asset other than the singleton is accepted;
- the same tests show that the singleton at any position other than input zero or output zero is refused, by the self-position pin and by conservation rather than by a count;
- the deployment binding carries the two external evidence roles, the non-reissuable declaration and the issuance base, as named roles beside substrate conservation;
- the discharge table's MODEL-SCOPE rows name the region-scoping refit row, and the published closure record states that a composed transaction is accepted on-chain and outside the model until that row lands;
- the repository gate passes, with the recounted symbol and site census reported against the pre-change baseline.

Until that bite lands this record is decided and adopted but unimplemented, and the index row says so. The earlier generations' sponsor-region audit is a separate row and does not gate this one.
