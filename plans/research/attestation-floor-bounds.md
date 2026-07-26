# Research Question: Lifetime Attestation and Floor Bounds · `q:attestation:floor-bounds`

> **Status:** Analysis accepted; normative correction and permanent regressions pending
> **Blocks:** Layer-0 containment and seigniorage claims, the SP5 capacity export, and every importing-layer cost projection derived from them
> **Affected packages:** `papers/attestation`, model, realization
> **Depends on:** the burn-settlement convention (`q:attestation:burn-settlement`), resolved below
> **Decisions:** settlement-pinned batch valuation adopted; SP5 retained and generalized
> **Imports:** (`[A-prop:containment:floor-ceiling]`),
> (`[A-eq:containment:floor-ceiling]`),
> (`[A-prop:interface:bootstrap-capacity]`),
> (`[A-eq:containment:max-attestation]`),
> (`[A-thm:seigniorage:lifetime-envelope]`),
> (`[A-eq:seigniorage:lifetime]`),
> (`[A-thm:seigniorage:bootstrapping-bound]`),
> (`[A-prop:containment:seigniorage-suppression]`),
> (`[A-def:model:ratefloor]`),
> (`[A-def:model:classes]`),
> (`[A-def:maturity:conversion]`),
> (`[A-alg:operations:cycle-processing]`),
> (`[A-prop:interface:conservative-valuation]`),
> (`[A-app:attestation:verification]`)
> **Expected handoff:** corrected containment and seigniorage statements, plus either a proved lifetime attestation bound or an accepted statement that none exists

## Question · `sec:attestation-floor-bounds:question`

Over the receipt lifetime, what — if anything — bounds the redemption-rate
floor and cumulative attestation once external deposits are admitted?

Attestation currently exports three finite bounds and treats them as a family. Two
of the three are established only at genesis, before any external deposit, and
the document generalizes them to the whole bootstrapping phase and to the
receipt lifetime. Under the model's own operations that generalization is
false, and one published theorem is false with it.

This note is opened because the defect is not notational. The affected claims
are the layer's exported capacity story and the calibration coupling that
depends on it.

## Established algebra · `sec:attestation-floor-bounds:algebra`

Burning changes no reserve (`[A-def:operations:burn]`): the pool is unchanged
and supply falls. With the floor defined as the pool over total supply
(`[A-def:model:ratefloor]`), a burn sequence carrying supply from `Y_start`
down to `Y_end` at constant pool yields cumulative attestation

```text
A = ∫ φ dx = ∫ (Ω/Y) dY  =  Ω · ln(Y_start / Y_end)
```

This single identity organizes every result below.

The SP5 capacity bound (`[A-eq:containment:max-attestation]`) is exactly this
identity instantiated at

```text
Y_start = E_0
Y_end   = Y_T = (1-ζ)·E_0
A_max   = E_0 · ln(1/(1-ζ))
```

so SP5 is finite **for one reason only**: the time-locked class holds supply
away from zero. The logarithm is bounded because its denominator is bounded
below, and the time-locked class is the whole of that lower bound.

Maturity conversion (`[A-def:maturity:conversion]`) sets the time-locked
supply to zero. It therefore removes the only term that made the bound finite.

## Resolved: settlement-pinned batch valuation · `q:attestation:burn-settlement`

The document used two conventions and named neither: an atomic one in
operations, seigniorage, and the walkthrough, and a continuous one in the SP5
capacity proof. They disagree numerically on the same action.

The accepted convention is **settlement-pinned and batch-linear**. A burn
record of amount x is credited at the most recent preceding authenticated
settlement floor:

```text
ΔA = x · φ_c
```

Every record assigned to one settlement interval uses the same committed
floor. A burn does not internally appreciate while being valued. The
continuous integral is retained only as the supremum obtained from arbitrarily
fine burn/settlement interleaving; it is not the semantics of a single burn.

This matches the implemented realization, where a burn produces an ASH, a
clear commits the supply decrement and the new floor, and record valuation
reads the most recent preceding authenticated clearing.

The convention is what makes the potential argument below conservative in the
intended direction: because the floor ratchets
(`[A-thm:invariants:floor-non-decrease]`), the committed floor never exceeds
the current one, so `ΔA = x·φ_c ≤ xΩ/Y`.

## Accepted potential argument · `sec:attestation-floor-bounds:potential`

Pre-maturity, write T for time-locked supply, c = 1-ζ, D for cumulative gross
reserve deposits counting every deposit event, and A for aggregate accepted
attestation credit. Define

```text
Ψ(Ω, Y, T) = Ω · ln(Y / T)
```

which is nonnegative pre-maturity because 0 < T ≤ Y.

**The class rules give T ≥ cY.** This holds with equality at genesis. The
quantity T - cY is *invariant* under deposits — with u = T/Y and r = δ/Ω,

```text
T' - cY' = Y(u + cr) - cY(1 + r) = Yu - cY = T - cY
```

— and non-decreasing under live burns and live redemptions, both of which
reduce Y and leave T untouched. Starting at zero, it never goes negative.

**Burns consume the potential.** For a settled batch x, settlement-pinning
gives ΔA ≤ xΩ/Y, while

```text
Ψ_before - Ψ_after = Ω·ln(Y / (Y-x)) ≥ Ω·x/Y
```

by -ln(1-s) ≥ s. Hence ΔA ≤ Ψ_before - Ψ_after. Equality is approached by
arbitrarily fine interleaving; a coarse batch credits strictly less, which is
the intended conservative direction.

**Redemptions cannot increase it.** A floor-preserving live redemption has
Ω' = αΩ, Y' = αY, T' = T for some 0 < α ≤ 1, so both factors of Ψ are
nonnegative and non-increasing and Ψ' ≤ Ψ.

**Deposits add boundedly.** Since deposits mint at the current floor,

```text
Ψ' = Ω(1+r) · ln((1+r) / (u + cr))
```

The increase is maximized at the boundary u = c. This needs the derivative
sign, not merely the stationary point:

```text
d/du [Ψ' - Ψ] = Ω · r(c-u) / [u(u+cr)]
```

which is positive iff u < c. The admissible domain is u ≥ c, so Ψ' - Ψ is
decreasing there and attains its maximum at u = c, giving exactly

```text
Ψ' - Ψ ≤ δ · ln(1/c) = δ · ln(1/(1-ζ))
```

**Telescoping** from Ψ₀ = E₀·ln(1/(1-ζ)) yields the tight pre-maturity theorem
recorded in the result below. It is attained as a supremum by depositing the
whole of D at genesis, which preserves u = c, then burning the entire live
class through arbitrarily fine settlements.

## Candidate matrix · `tbl:attestation-floor-bounds:candidates`

| Mint | Candidate | Strength | Main risk |
|---|---|---|---|
| `candidate:attestation:no-lifetime-bound` | Prove no lifetime bound exists; export only the pre-deposit capacity and the per-round ratio | Honest and provable from the log identity | Removes an exported guarantee importing layers may already assume |
| `candidate:attestation:permanent-supply-floor` | Retain a permanent minimum supply, so the log denominator stays bounded below | Restores a finite bound by the same mechanism that made SP5 finite | Changes the maturity contract; a residual locked tranche never converts |
| `candidate:attestation:conditional-maturity` | Make maturity conversion partial or conditional rather than total | Keeps a bound without a permanent lock | Complicates the one-time relabel and its value-neutrality proof |
| `candidate:attestation:explicit-floor-cap` | Cap the floor directly as a protocol constant | Simple to state and check | A capped floor is no longer the pool over supply; breaks the redemption identity |
| `candidate:attestation:narrow-to-immediate` | Restrict the seigniorage result to immediate burning and delete the lifetime envelope | Minimal, provably correct today | The section loses its distinct result; says nothing about a patient operator |

**Selected:** (`candidate:attestation:no-lifetime-bound`) together with
(`candidate:attestation:narrow-to-immediate`). The potential argument supplies
what the first needed — a proof rather than an assertion that no lifetime bound
exists — and the second is the minimum correctness action for the published
theorem.

**Not selected:** the permanent supply floor, conditional maturity, and
explicit floor cap. Each would restore a finite lifetime bound by changing the
operations, and none is required once the surviving bounds are stated with
their real scopes. They remain recorded because a future decision to want a
finite lifetime bound must change an operation; the log identity forecloses
obtaining one by restatement.

## Remaining analysis · `sec:attestation-floor-bounds:analysis`

The bounds question is answered. What remains before the normative correction
lands:

1. Restate the calibration coupling in (`[A-rem:model:calibration]`), which
   currently couples ζ and f through the withdrawn envelope.
2. Decide where the permanent regressions live. These are Layer-0 real-valued
   claims; the realization model crate is integer-domained and cites Layer 0
   once, in a constants comment, so it is not their natural owner.
3. Record how the realization's integer domain and its clear clamp interact
   with the negative lifetime result: a deployment maximum exists but depends
   on atomic-unit scale and current reserve, and is not a scale-independent
   protocol bound.

## Vectors · `sec:attestation-floor-bounds:vectors`

Required permanent regressions:

1. SP5 at D = 0 — the bound is approached and never exceeded.
2. The floor-ceiling certificate — a valid pre-maturity state with φ above
   1/(1-ζ).
3. The lifetime-envelope certificate — atomic batches exceed fD/(1-ζ).
4. Potential monotonicity under burn.
5. Potential non-increase under redemption.
6. Potential increase bound under deposit.
7. Generated valid pre-maturity traces satisfying the boxed inequality.

**Exactness rule.** The logarithmic inequality must not be certified with raw
`f64`. The split is by content, not by convenience:

```text
exact rational, no logarithm needed
    the two certificates (2, 3)
    T - cY invariance under deposit
    ΔA ≤ xΩ/Y from settlement pinning and the ratchet
    α ≤ 1 for a floor-preserving redemption
    the deposit derivative sign r(c-u)/[u(u+cr)]

requires a certified logarithm bound
    SP5 attainment (1) and the end-to-end trace inequality (7)
```

The first implementation prefers symbolic transition lemmas and uses floating
evaluation only for illustrative vectors. Where a logarithm must be bounded,
use rational interval enclosures with a stated error bound, never a bare `f64`
comparison. `num-bigint` is already a workspace dependency; exact rational
comparison by cross-multiplication needs no new dependency, and adding one
would require its own ADR-011 entry.

Ownership is unresolved — see the remaining-analysis section. These are
Layer-0 real-valued claims, and the realization model crate is integer-domained.

## Acceptance · `gate:attestation-floor-bounds:accept`

Accept a resolution only when:

- one burn settlement convention is named and used by every affected statement;
- every published bound carries its own scope in its own statement, with no
  bound relying on a scope stated elsewhere;
- the pre-deposit and lifetime regimes are separate claims with separate
  proofs;
- each surviving bound has a vector that attains it and a vector that
  demonstrates the adjacent regime where it fails;
- no claim summing instantaneous SP6 increments into a total remains;
- the calibration coupling cites only surviving bounds;
- the certificates in this note run as regressions and fail against the current
  published statements.

## Rejection · `gate:attestation-floor-bounds:reject`

Reject a candidate if:

- it restores a finite lifetime bound without changing any operation, which
  would contradict the log identity;
- it relies on the floor being bounded during bootstrapping;
- it narrows a claim by qualifying its prose while leaving the symbol and its
  macro contract asserting the wider scope;
- it states a bound whose scope must be recovered from a different section;
- it treats the maturity conversion as bound-preserving without proving what
  replaces the time-locked supply floor.

## Result · `sec:attestation-floor-bounds:result`

Accepted. The analysis is settled; the normative correction and the permanent
regressions are not yet landed.

```text
1. Burn valuation is settlement-pinned and batch-linear:
   ΔA = x·φ_c for the preceding authenticated settlement c.

2. Continuous integration is an upper-envelope/supremum argument,
   not the semantics of one burn.

3. SP5 remains valid exactly before external deposits.

4. Before maturity, with cumulative gross deposits D:

       A + Ω·ln(Y/Y_T) ≤ (E₀ + D)·ln(1/(1-ζ))

   and therefore

       A ≤ (E₀ + D)·ln(1/(1-ζ))

   attained as a supremum by depositing D at genesis, then burning
   the whole live class through arbitrarily fine settlements.

5. The always-valid pre-maturity floor bound is φ ≤ Ω/Y_T, a dynamic
   standing bound. A deposit-parameterized form is

       φ ≤ (E₀ + D) / ((1-ζ)·E₀)

   from Ω ≤ E₀ + D and Y_T ≥ (1-ζ)·E₀.

6. No deposit-independent finite bootstrapping floor ceiling exists.
   The constant 1/(1-ζ) is the genesis/no-external-deposit value and
   the limiting per-round multiplier — not a ceiling.

7. No finite lifetime attestation bound and no finite lifetime
   operator-seigniorage envelope exist under the current real-valued
   operations once maturity removes Y_T.

8. The immediate-burning operator result A_op = fζD is retained.
   The lifetime envelope fD/(1-ζ) is withdrawn.
```

Point 4 subsumes the old SP5 statement rather than replacing it: SP5 is the
D = 0 corollary, so its substance survives unchanged. What fails is only the
description of that fixed value as a bound holding across the whole
bootstrapping phase after arbitrary external deposits.

Point 7 is a negative result and is stated as one. The realization's integer
domain and its clear clamp make any particular deployment's maximum finite,
but that maximum depends on atomic-unit scale and current reserve. It is
neither the published envelope nor a scale-independent protocol bound, and it
must not be presented as rescuing the withdrawn theorem.

The rejection of the old envelope needs no convention argument: it fails under
atomic settlement-pinned batches, which is the convention now adopted.

## Handoff · `sec:attestation-floor-bounds:handoff`

The accepted result is handed off in three series, in order.

**Permanent regressions** land first, so the corrected statements are welded
before the prose that states them is rewritten. Ownership is the open decision
recorded above.

**Normative Attestation correction** then updates the containment and seigniorage
sections, the interface capacity export, the verification suite's invariant and
bounds tables, the calibration remark, and the macro contract and symbol index
entries for the floor-ceiling symbol. Existing anchor keys are kept wherever
possible, so the anchor-set hash is unchanged if no anchor name moves.

**Identity and realization weld refresh** follows only if the Attestation version
moves. The version bump deserves deliberate review rather than a silent edit:
executable transition behavior is unchanged, but a materially false theorem is
being withdrawn, which argues for a visible pre-1.0 correction release. Expect
the architecture semantic hash to move and the behavioural hash to stay stable
if no behavioural array changes.

The notation-side repair is owned separately by
(`q:notation:semantic-census`), which is what allowed one corrected proposition
to coexist with four uncorrected restatements of it. That note's scope-marker
weld exists precisely to stop the corrected scope from drifting again, so the
two questions land together rather than the maths alone.
