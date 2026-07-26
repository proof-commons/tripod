# Research Question: Lifetime Attestation and Floor Bounds · `q:attestation:floor-bounds`

> **Status:** Open; formal analysis required
> **Blocks:** Layer-0 containment and seigniorage claims, the SP5 capacity export, and every importing-layer cost projection derived from them
> **Affected packages:** `papers/attestation`, model, realization
> **Depends on:** an accepted burn-settlement convention (`q:attestation:burn-settlement`)
> **Decisions:** none yet
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

## Sub-question: burn settlement convention · `q:attestation:burn-settlement`

The document uses two settlement conventions and does not name either.

```text
atomic     the whole burn settles at the prevailing pre-burn floor
           used in operations, seigniorage, and the verification walkthrough

continuous the burn settles in infinitesimal increments at the appreciating floor
           used in the SP5 capacity proof to obtain its supremum
```

They disagree numerically on the same action: the capacity proof's integral is
strictly larger than the atomic valuation of the same burn. SP5 is stated as a
supremum, so it is safe under both, but no other result declares which
convention it assumes, and the certificates above give different magnitudes
under each. Every candidate below must state its convention before it can be
evaluated.

## Candidate matrix · `tbl:attestation-floor-bounds:candidates`

| Mint | Candidate | Strength | Main risk |
|---|---|---|---|
| `candidate:attestation:no-lifetime-bound` | Prove no lifetime bound exists; export only the pre-deposit capacity and the per-round ratio | Honest and provable from the log identity | Removes an exported guarantee importing layers may already assume |
| `candidate:attestation:permanent-supply-floor` | Retain a permanent minimum supply, so the log denominator stays bounded below | Restores a finite bound by the same mechanism that made SP5 finite | Changes the maturity contract; a residual locked tranche never converts |
| `candidate:attestation:conditional-maturity` | Make maturity conversion partial or conditional rather than total | Keeps a bound without a permanent lock | Complicates the one-time relabel and its value-neutrality proof |
| `candidate:attestation:explicit-floor-cap` | Cap the floor directly as a protocol constant | Simple to state and check | A capped floor is no longer the pool over supply; breaks the redemption identity |
| `candidate:attestation:narrow-to-immediate` | Restrict the seigniorage result to immediate burning and delete the lifetime envelope | Minimal, provably correct today | The section loses its distinct result; says nothing about a patient operator |

Candidates are not exclusive. The narrowing candidate is the minimum
correctness action for the published theorem and may be combined with any
structural candidate.

## Required analysis · `sec:attestation-floor-bounds:analysis`

1. Fix the burn settlement convention, or carry both explicitly through every
   affected statement.
2. Prove or refute a lifetime bound on cumulative attestation per address under
   the accepted convention, with external deposits admitted.
3. Prove or refute a lifetime bound on the operator's attestation specifically,
   given that its inflow is a fee share rather than a free choice of holdings.
4. Determine whether the per-round multiplier result generalizes to a closed
   form for the floor under an arbitrary deposit and burn schedule.
5. Establish what the time-locked class actually bounds, stated so that the
   pre-deposit and post-deposit regimes are visibly different claims.
6. Restate the calibration coupling in (`[A-rem:model:calibration]`) once the
   surviving bounds are known; it currently couples ζ and f through the
   envelope, which is one of the failing claims.

## Vectors · `sec:attestation-floor-bounds:vectors`

Required vectors, each with expected values derived before any prose is
written:

- the two certificates above, as executable fixtures;
- the n-round iteration driving the floor toward φ_max^(n+1), for n up to a
  bound where floating error is still negligible;
- atomic and continuous valuation of one identical burn sequence, asserting
  they differ;
- a pre-maturity sequence with no external deposit, asserting the floor does
  attain and does not exceed φ_max;
- an operator that never burns until after every other holder has finished,
  asserting the envelope's failure magnitude;
- a sequence exercising SP5 alone, asserting the published `A_max` is attained
  as a supremum and not exceeded before external deposits;
- redemption-only sequences, asserting the floor is preserved and neither bound
  moves.

The executable model owns these fixtures once the convention is accepted;
until then they are analysis artefacts, not evidence.

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

Pending.

## Handoff · `sec:attestation-floor-bounds:handoff`

A resolution updates the containment and seigniorage sections, the interface
capacity export, the verification suite's invariant and bounds tables, the
calibration remark, and the macro contract and symbol index entries for the
floor-ceiling symbol. The notation-side repair is owned separately by
(`q:notation:semantic-census`), which is what allowed one corrected proposition
to coexist with four uncorrected restatements of it.

If no candidate yields a lifetime bound, record that result explicitly and
export the absence. Do not retain an envelope whose only support is a ceiling
that the same document declares unbounded.
