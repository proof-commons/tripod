# Attestation Realization

**An Elements/Liquid covenant realizing the attestation contract: a reserve-backed, two-class, conserved, burnable claim — specified as a typed architecture manifest, a conformance contract of invariants and oracle obligations, and a translation discipline for reaching script.**

*The realization document of the* Attestation *specification. Self-contained, with one declared upward dependency: the abstract object it enforces is defined in* Attestation *(**v1.0.0**), cited throughout by ``[A-...]`` anchors under the consumer prefix `A-`. The dependency is machine-checked, not prose — the attached manifest's document block binds the specification version, and release validation refuses an unpinned anchor set (`rem:overview:anchor-pin`). The manifest itself —* `architecture.toml`*, architecture schema 17, semantic hash* `3974e4d7860d2cddad97c6ab63f5c358e303505a2529bf757d47db462285aa68`*, behavioural hash* `756ea65ce3dc370e76ec70dc001231693facd58e53ebca315d549967f03cf206` *— is attached as the closing appendix (`app:realization:architecture`) and is authoritative on every enumerated fact.*

> **Release envelope.** The attached manifest carries `publication_status = "final"` and `realization_version = "0.3.0-dev"` — the tracked binding of (`def:versioning:denotation-law`), which moves with the compiler line's minor and signals nothing about content; the behavioural hash printed above is the denotation's sole stability witness. The semantic hash printed above is the *release* hash, minted at the pin ceremony that set the specification anchor-set hash (`b0cafaa48ac2ed388984f6a2d757a011a9c8654acf224f5f64516f26a3f7ee35`) for the specification's v1.0.0 release. The prior release identities were re-measured, not redefined: the ceremony renamed two anchor names with the specification's own-division label area, the retired anchor-set value was reproduced before the new one was taken, and the behavioural hash — unchanged across the ceremony — witnesses that the denotation did not move. The earlier recipe migration that domain-separated both identities is recorded in ADR-021.

---

## §0 What this document is · `sec:realization:overview`

This document specifies an on-chain **covenant** — spending conditions enforced by Elements/Liquid tapscript — that realizes **the attestation contract**. The economic content — a reserve pool of backing collateral, a live class redeemable against it, a time-locked class that matures once, a per-cycle issuance and distribution cadence — is fixed upstream by the *Attestation* specification (`[A-def:model:classes]`, `[A-def:model:ratefloor]`); this document makes the on-chain machinery **correct, trust-minimized, auditable, and scalable**, and says *exactly* what any conforming implementation must demonstrate.

It uses no operator zero-knowledge proofs, no off-chain authority over correctness, and no trusted oracle. Where trust is unavoidable it is named (`sec:realization:trust`). The reserve asset (`[A-def:model:reserve-asset]`) is instantiated as **L-BTC**, whose settlement consortium is a named dependency, exactly as the specification permits (`[A-rem:model:instantiations]`).

### §0.0 The four registers · `sec:overview:registers`

Every sentence belongs to exactly one of four registers and inherits its authority.

*The four registers · `tab:overview:registers`*

| register | role | authoritative on | lives in |
|---|---|---|---|
| **trap boxes** | *why* | that the structure realizes the economics without a leak | primarily (`sec:realization:architecture`), (`sec:realization:kernel`), (`sec:realization:operations`), (`sec:realization:ledger`); also (`sec:realization:arithmetic`), (`sec:realization:invariant`), (`sec:realization:translation`) |
| **`architecture.toml`** | structural *what* | every enumerated set and relation; the semantic hash | (`app:realization:architecture`) |
| **invariants + oracle** | *goal* | the properties any realization MUST keep, and the acceptance test | (`sec:realization:invariant`), (`sec:realization:oracle`) |
| **script** | implicit *how* | nothing a reader must trust — it is produced, not specified | (absent) |

> **The register-authority rule · `rem:overview:register-authority`**
>
> On any enumerated fact — which assets, roots, objects, operations, quantities, tags, bounds, decisions, and dependencies exist, and their declared relations — the appendix **wins**. A listing in the body that shows structure is illustrative of the manifest, never a second source of truth. Prose never contradicts the manifest; where it appears to, the manifest is correct and the prose is defective.

> **The conformance claim · `rem:overview:conformance-claim`**
>
> An independent implementer reconstructs the executable model and the emitted script **against the attached manifest**, and accepts via the oracle of (`sec:realization:oracle`). Conformance is **behavioural, not byte-identical**: two realizations whose scripts differ leaf-for-leaf are equally conformant iff both maintain every clause of (`sec:realization:invariant`) under the obligations of (`sec:realization:oracle`), and both separately discharge the `{verify}` surface of (`sec:realization:trust`). The semantic hash binds *which* manifest a candidate claims to realize; the obligations bind the candidate's behaviour to it.

The document refuses to specify script bytes because the goal register *witnesses* any conforming realization: an implementer who computes a wrong quantity fails a clause, and the oracle catches it. That refusal is licensed only to the depth the witness reaches — it fails at exactly two depths, the four arithmetic gadgets below the amount abstraction (`sec:arithmetic:gadgets`) and the emitted script above the model (`sec:oracle:boundary`), and those are the two places the *how* surfaces explicitly.

One divergence is deliberate: the document is **written** kernel-first — (`sec:realization:kernel`) is its centre of gravity — but **reads** in the order printed, objects before machinery.

### §0.1 Position beside the specification; the one-directional rule · `sec:overview:position`

*The coupling · `fig:overview:stack`*

```
Attestation Realization (THIS DOCUMENT) ─realizes─▶  ATTESTATION (the specification) ──I₀──▶ A : 𝔸 → ℝ≥0  (to the consumer)
                    ──monetary surface──▶ operations & audit quantities
```

> **Cross-document references — one direction only · `rem:overview:cross-references`**
>
> Anchors of the form ``[A-<type>:<section>:<name>]`` are references **into** the *Attestation* specification, using its exact label schema under the consumer prefix `A-`. These are upward cites and are never reciprocated: the specification cites nothing outside itself — it forward-references no document and names no realization or consumer concept. There is **no** reverse-citation table on the specification side and **no** "(cited by …)" annotation anywhere here. This document consumes the specification's published anchors, and that is the *entire* coupling. **This is the canonical statement of the one-directional rule; every other mention points here.** The upward cites this document makes are indexed, one row each, at (`sec:realization:anchors`).

> **The anchor pin is a machine-checked field · `rem:overview:anchor-pin`**
>
> The masthead's version claim is backed by the manifest, not by this sentence: the document block of (`app:realization:architecture`) carries the specification version binding, and release validation rejects a manifest whose specification anchor-set hash is unpinned. A draft manifest may leave the anchor-set hash unset; a releasable one may not. The version-incoherence failure class is closed at the seam where it would occur — in the artifact, under the hash.

### §0.2 The citation contract and the reading devices · `sec:overview:contract`

> **The three-tier citation contract · `rem:overview:citation-contract`**
>
> **Tier 1 — the prose glyph.** A numbered, spoken name — `G6`, `𝗜₈`, `O3`, `T12`, `P-issue`, `L-floor`, `R-conv` — appears bare in running text, is never bracketed, never backticked, and is never itself a label. The ordinal lives on the glyph and **only** there.
> **Tier 2 — the citation.** A cross-reference is the round-bracketed, backticked, number-free label: `` (`inv:invariant:accounting`) ``. Square brackets appear inside the backticks iff the label is external — a specification cite `` (`[A-…]`) `` or a code-register cite `` (`[rule:verification:pure-transition]`) ``. No bracket, round or square, ever contains a number.
> **Tier 3 — the definition (a mint).** The same number-free label rides bare in its one home — a heading suffix, a blockquote head, or a box foot — with no round brackets, because position already marks it. Text search resolves it to exactly one place.
>
> Section numbers (`§5.4`) are locators for human navigation and are never cited; if a section moves, its number changes and its label does not. One namespace note: the trap-box family under `trap:branches:*` is a published API whose middle segment records the boxes' conceptual home; it is frozen and does not chase the section family it now lives beside.

Every non-obvious design choice is presented as a **naïve-trap box** — the *why* register:

> **The trap template · `rem:overview:trap-template`**
>
> **⚠ Naïve construction.** *What a direct implementer reaches for.*
> **Why it fails ({a named guarantee or a named attack}).** *The specific break.*
> **What we do.** *The construction.* `[enforced: P-…]` *or* `[invariant: 𝗜ₙ]` — the bare trap label.
>
> Two rules make the boxes load-bearing rather than narrative: every box terminates in a **named** `Gn` or a **named** attack, and every "what we do" terminates in a **pin** (`sec:realization:pins`) or an **invariant clause** (`sec:realization:invariant`) — something a reviewer greps. Eight boxes are *canonical*: they carry their argument once, and every later occurrence of the same idea cites rather than restates.

> **Status tags · `rem:overview:status-tags`**
>
> A distinct, already-bracketed family, read in place — not cross-references: always backticked (so the harvester audits each in its own grammar class) but never round-wrapped, so they cannot collide with the round-bracket cite rule:
>
> | tag | meaning |
> |---|---|
> | `[enforced: P-…]` | guaranteed by covenant, or covenant ∧ consensus, with a build pin |
> | `[invariant: 𝗜ₙ]` | discharged by the named invariant clause |
> | `[accepted residual]` | tolerated; never reaches a bad state; catalogued in (`sec:realization:trust`) |
> | `[liveness, not safety]` | affects progress, never the invariant |
> | `[design property]` | an incentive or structural fact |
> | `[honesty note]` | a candid scope limitation |
> | `{verify}` | a claim about emitted script or substrate — **the absence of a proof, never a proof**; the witness is owed at the deployment profile (`sec:oracle:boundary`) |

### §0.3 The two neutralities · `sec:overview:neutrality`

> **What "neutral" means here, and what it does not · `rem:overview:two-neutralities`**
>
> Throughout this document **neutral** names exactly two properties of the exported map `A`: it **declines to price** (the specification confers no claim on the attestation, so it is fit as a common reference (`[A-def:interface:attestation]`)), and it is **identity-blind** (no oracle, consortium, or party resolves *who* an address is). Neutral does **not** mean wealth-blind. `A` is wealth-**sensitive by construction**: attestation is `δ = x·φ_c`, anchored to forfeited reserve, and that cost *is* the Sybil resistance. Costly-and-unforgeable and wealth-blind are the same object negated — you cannot have both.
>
> The propagation rule is one-directional: `A` is the **seed** the consumer reads. Wealth enters the consumer only through that seed; whether the consumer's standing field is wealth-blind in its *dynamics* is a property of the consumer's combination operator, out of scope here (`rem:overview:cross-references`). This document claims exactly: `A` is identity-neutral and wealth-sensitive. It claims nothing about wealth-neutrality at any layer, and the absence of such a claim is **not** its presence. **Every other mention of "neutral" points here.**

### §0.4 The versioning law · `sec:realization:versioning`

(`rem:overview:register-authority`) settles who wins on an enumerated fact; this settles what the recorded version means and how a change to the system is classified. Classify on the **denotation** of $\mathcal S = (\mathsf{Obj},\mathsf{State},\mathsf{Step},\mathsf{Inv},\mathsf{Obs})$ (`rem:oracle:thesis`), never on which arrays were touched.

> **The Denotation Law · `def:versioning:denotation-law`**
>
> `realization_version` binds this document to the compiler line it is realized through: it reads `major.minor.0` for the compiler line `major.minor.patch`, the prerelease marker carried verbatim. The bound object is the development line, not a release of it, so the binding is patch-blind and moves with every compiler minor **whether or not one word of this document changes**. The version therefore signals nothing about content. Content stability has exactly one witness: the behavioural hash. A change is a **presentation change** iff it holds all five denotations fixed, altering only presentation — schema, vocabulary, prose, labels, encoding — or the *conforming-implementation set* — pins, obligations, representation latitude. It is a **denotation change** iff any denotation moves. Equivalently: $\mathcal S$-preserving $\iff$ the behavioural hash is stable — and nothing that moves the hash is presentational, however small it reads.

> **⚠ Naïve construction.** Tightening only removes behaviours, so it never moves the denotation.
>
> **Why it fails (the register confusion).** **Latitude-tightening** forces realizations to enforce what $\mathcal S$ already forbade — $\llbracket\mathsf{Step}\rrbracket,\llbracket\mathsf{Inv}\rrbracket$ unchanged, only the implementation set shrinks: presentation. **Denotation-tightening** removes an abstract behaviour the system previously permitted — $\llbracket\mathsf{Inv}\rrbracket$ shrinks as a set: a denotation change, however "stricter" it reads.
>
> **What we do.** Ask: did the set of valid *abstract* behaviours change, or only the fidelity with which implementations must track an unchanged set? The first moves the denotation, the second does not. The same cut governs optimizing: re-*encoding* a value is presentation; changing a *formula's form* — floor, split, fee, payout, issuance — moves the denotation. `[enforced: P-denotation]` `trap:versioning:tightening`

> **Decision procedure · `rule:versioning:decision`**
>
> Any one verdict decides that the denotation moved: (1) an *abstract* transition becomes accepted that the standing denotation rejected; (2) one becomes rejected as lost *capability* rather than lost implementation freedom; (3) an economy-determining formula re-values the same inputs; (4) a consumer of the specification's exported interface sees a different I₀/surface for the same history, representation aside (`sec:realization:representation`); (5) an object's recognition criterion changes; (6) any SP or G changes *meaning* rather than gaining a pin. None firing, with (2) only latitude-loss and the conforming-implementation set non-empty, is a presentation change. Two memberships are read, not adjudicated: `requires_deployment_calibration = true` marks a magnitude outside $\mathcal S$ (its draft default never moves the denotation; `amount_limits`, unflagged, is $\mathcal S$); `dependencies`/`decisions` are normative-descriptive (`sec:trust:verify`) and sit outside the behavioural hash.

> **P-denotation · `pin:pins:denotation`** The behavioural hash is computed over the behavioural arrays and relations only — assets, roots, objects, operations, quantities, witnesses, clauses, bounds, amount_limits, tags — excluding dependencies, decisions, envelope, and prose. Bounds enter as a projection: a bound flagged `requires_deployment_calibration = true` contributes its identity, cardinality references, and calibration classification, but **not** its draft `default_value`, which lives outside $\mathcal S$ (the membership read above); an unflagged bound's value, like `amount_limits`, is $\mathcal S$ and remains a hash input. Witnesses and clauses likewise enter as projections: their stable codes and the witness semantic id are $\mathcal S$; the document citation labels they carry (witness `semantic_tag`, the clause registry's frozen labels) are presentation guarded by the weld (`rem:manifest:weld`), **not** hash inputs. The attached envelope names the algorithm that computes it; retired algorithms stay retired, each one's contradiction of this law and its pinned value recorded with the migration in the versioning-gate test. **Two gates enforce the law mechanically.** The *binding gate* derives the expected `realization_version` from the compiler's own version and fails when the recorded binding lags — a compiler minor bump is completed only by re-recording the binding here and in the envelope, so every version movement is a deliberate, recorded event even when no content changed. The *denotation gate* pins the behavioural hash: the pinned value and this masthead's stated value must both equal the measured value, so moving the denotation takes two deliberate edits plus a denotation record naming the deciding test — a moved hash discovered rather than declared fails the build. While the denotation stands, obligations grow monotonically — a later revision of this document conforms to every earlier one; across denotation changes, neither direction holds. Monotonicity applies to valid obligations: it does not preserve an accidental implementation restriction shown to inspect a fact outside the denotation ((`rule:representation:inspection-necessity`)). Removing such a restriction — the sponsor-positivity erratum — corrects conformance latitude and reintroduces no abstract behaviour. Checklist-and-CI protected (`rem:pins:weld-scope`). Goal: (`def:versioning:denotation-law`).

---

## §1 What the covenant must guarantee · `sec:realization:requirements`

The pool holds reserve collateral `Ω` (in the reserve asset, instantiated as L-BTC) against a circulating claim supply `Y = Y_L + Y_T`. The redemption-rate floor `φ = Ω/Y` is the redemption price (`[A-def:model:ratefloor]`). The covenant makes the following true *by construction*, for any sequence of spends any party can attempt. The third column is the specified obligation each requirement discharges — an upward cite; the fourth is why the property cannot be left to good behaviour.

*What the covenant must guarantee · `tab:requirements:guarantees`*

| # | Requirement | Discharges (L0) | Why it cannot be left to good behaviour |
|---|---|---|---|
| **G1 Floor monotone** `req:requirements:rate-monotone` | `φ` never decreases; every operation rounds in the pool's favour. | (`[A-thm:invariants:floor-non-decrease]`) | A spender-chosen rounding that lowers `φ` extracts backing from every other holder. A checked relation on every transition. |
| **G2 Backing integrity** `req:requirements:backing-integrity` | Burning claim units never removes backing; redeeming removes exactly the floored payout. | (`[A-prop:invariants:burn-increases-floor]`, `[A-prop:invariants:redemption-preserves-floor]`) | Burning must be costly, yet must let nobody siphon `Ω`. |
| **G3 Costliness is real** `req:requirements:costliness` | Reaching a burn total requires destroying claim units redeemable for that value at the instant of destruction; attestation is anchored to unforgeable destruction under a dual anchor (`sec:ledger:authentication`), never to free-standing chain data. | (`[A-thm:interface:instantaneous-costliness]`) | The consumer's commitment thesis rests on burns being genuinely expensive — including at the accounting layer, where unauthenticated data would grant free attestation. |
| **G4 Irrevocable record** `req:requirements:irrevocable-record` | The burn record `(a, x)` rides a burn-typed transition as consensus-immutable data outputs, contiguously indexed; it only ever grows. The *derived* valuation `δ = x·φ_c` is checkpoint-relative (`sec:ledger:reorg`); the immutable object is the record, never the valuation. | (`[A-lem:costliness:attestation-append-only]`, `[A-postc:interface:monotonicity]`) | A rewindable record would let an address un-spend attestation. A free-standing payload with no authenticated transition beneath it is not a record at all. |
| **G5 No forgery / no inflation** `req:requirements:no-forgery-inflation` | A closed-asset unit exists only if its issuance authority created it; issuance of `U` happens only in a sanctioned cycle and only in the sanctioned amount `ΔY`. | (`[A-def:model:supply]`, `[A-prop:invariants:issuance-preserves-floor]`) | Forgery is consensus (asset-id derivation). Inflation is **not** free: `U` is reissuable, and its supply discipline is covenant ∧ consensus (`sec:architecture:one-asset`). |
| **G6 Two classes, correct asymmetry** `req:requirements:two-classes` | The time-locked class is non-redeemable and non-burnable until the one-time maturity conversion; the live class is always redeemable. Structural — unrepresentable, not discouraged. | (`[A-def:model:classes]`) | The asymmetry *is* the time-locked class. It must be impossible, not rejected at runtime. |
| **G7 No discretion in arithmetic** `req:requirements:no-discretion` | Every economy-determining quantity (`ΔY`, payout, fee share, split, allocation) is a deterministic function of public state; the operator chooses nothing numeric. | (`[A-def:model:fee-and-split]`, `[A-alg:operations:cycle-processing]`) | Operator numeric discretion is an attack surface and a trust assumption. |
| **G8 Liveness without ransom** `req:requirements:liveness` | The cadence ceiling, admission, settlement, relabel, compaction, and clearing are drivable by *anyone* once their conditions hold; maturity conversion needs no trigger at all — it is the cycle's own atomic step. No operator secret stalls the system. | (`[A-def:maturity:conversion]`) | A permissionless safety valve gated by an operator signature lets the operator ransom the system. |
| **G9 Public auditability** `req:requirements:auditability` | Anyone recomputes `Ω`, `Y`, the floor facts, every address's attestation, and the **full root history** from the chain, at cost `O(burns + clearings + records)` per address for attestation over a pre-indexed canonical event stream. | (`[A-postc:interface:auditability]`) | Trust-minimization requires the public computation to authenticate provenance, not trust attacker-chosen data. History replay is part of the public audit, not an internal test. |
| **G10 Self-custody of value** `req:requirements:self-custody` | No spend routes value to an unintended recipient — including every spend *anyone* may trigger. | realization-level; supports (`[A-prop:interface:bootstrap-capacity]`) | The permissionless paths are the dangerous ones: a random triggerer must *advance* the system and never *capture* from it. |

The remainder of the document is the construction that discharges G1–G10, and the contract any other construction must satisfy to claim the same. Each requirement is re-derived as a corollary of the invariant at (`sec:invariant:corollaries`); the requirements table and the residual catalogue of (`sec:realization:trust`) **partition** the property space — a fact appears in exactly one of them, never both.

### §1.1 The I₀ discharge map · `sec:requirements:discharge-map`

The specification exports one computational object — the attestation map `A : 𝔸 → ℝ≥0` with six specified properties — plus a monetary surface of operations and audit quantities. One row per exported obligation; every "discharged at" cell is a label in this document.

*The I₀ discharge map · `tab:requirements:discharge`*

| I₀ obligation | L0 anchor | Discharged at |
|---|---|---|
| Address space `𝔸`, opaque, consent-free | (`[A-def:interface:address-space]`) | burn records as data outputs (`sec:operations:burn`); addresses are 32-byte strings, distinct from owner keys (`sec:realization:identity`) |
| Attestation map `A` | (`[A-def:interface:attestation]`) | the dual-anchor gate (`sec:ledger:authentication`); the identity-neutral, wealth-sensitive seed (`rem:overview:two-neutralities`) |
| SP1 non-negativity | (`[A-postc:interface:non-negativity]`) | gate positivity — `x > 0`, `φ_c > 0`; credits only add (`sec:ledger:authentication`) |
| SP2 settlement-pinned monotonicity | (`[A-postc:interface:monotonicity]`) | clearings are this realization's designated settlement-event family — a sanctioned instance of the specification's settlement-pinned clearing reduction (`[A-prop:invariants:burn-order-residual]`); the three-layer record/valuation split (`sec:ledger:model`); reorg as reprojection (`sec:ledger:reorg`). `A`'s valuation is monotone only relative to a checkpoint policy; between checkpoints it reprojects. |
| SP3 public auditability | (`[A-postc:interface:auditability]`) | G9 (`req:requirements:auditability`); the reader matrix (`sec:ledger:reader-matrix`); cost form (`rem:ledger:cost`) |
| SP4 instantaneous costliness | (`[A-thm:interface:instantaneous-costliness]`) | the event-type + value dual anchor (`sec:ledger:authentication`); the ash roach-motel (`sec:operations:burn`) |
| SP5 bootstrap capacity | (`[A-prop:interface:bootstrap-capacity]`) | the genesis ζ-split (`sec:invariant:genesis`) + time-locked unburnability (G6) + floor rise on clear (G1/G2) |
| SP6 conservative valuation | (`[A-prop:interface:conservative-valuation]`) | last-clearing valuation (`sec:ledger:order`); the lazy φ-rise and both terminals (`sec:invariant:terminals`) |
| Net live share `(1−f)ζ` | (`[A-def:interface:net-live-share]`) | the cycle's split and fee laws (`sec:operations:cycle`) |
| Burn / Deposit / Redemption / Transfer | (`[A-def:operations:burn]`, `[A-def:operations:deposit]`, `[A-def:operations:redemption]`, `[A-def:operations:transfer]`) | (`sec:operations:burn`) / (`sec:operations:request`) + (`sec:operations:admit`) / (`sec:operations:redeem`) / (`sec:operations:transfer`) |
| Floor invariants, per operation | (`[A-thm:invariants:floor-non-decrease]`) and per-op props | L-floor (`lem:invariant:rate`); each operation's preservation note (`sec:realization:operations`) |
| Burn-order residual | (`[A-prop:invariants:burn-order-residual]`) | the canonical order and frozen per-clearing floor (`sec:ledger:order`) |
| Maturity announcement / conversion | (`[A-postc:maturity:announcement]`, `[A-def:maturity:conversion]`) | both lead bounds native (`sec:operations:announce`); atomic conversion in the cycle ((`sec:operations:cycle`), 𝗜₁₁) |
| Cycle processing | (`[A-alg:operations:cycle-processing]`) | (`sec:operations:cycle`) + the cadence band (`sec:realization:authorization`) |
| Genesis | (`[A-postc:model:genesis]`) | (`sec:invariant:genesis`) |

---

## §2 Architecture — the bound UTXO set · `sec:realization:architecture`

The system is a small set of UTXO kinds bound by covenant rules. The manifest (`app:realization:architecture`) is authoritative on the full enumeration — eight assets, five roots, fourteen objects, thirteen operations, six tags — and this section states *why* the graph has the shape it has. The figure is orientation, not enumeration; on any count, the appendix wins (`rem:overview:register-authority`).

*The bound UTXO set · `fig:architecture:cluster`*

```
pool      STATE(PID, state-committed)  ⟷  RESV(L-BTC, Ω+Q)     ← welded pair, spent together (clear excepted)
clock     PACE(U-reissuance authority + cadence)                ← cycle-exclusive
auth      ENT_AUTH(→ENT)   DIST_AUTH(→DIST_CTL)                 ← genesis-unique, amount-one, off STATE
money     RECEIPT_L / RECEIPT_T (U, per-owner)                  ← the two covenant classes
deposit   DEPOSIT_REQUEST (open L-BTC offer) → DEPOSIT_ENTITLEMENT (ENT)
dist      DISTRIBUTION_CONTROL (DIST_CTL counters) + DISTRIBUTION_VAULT (U)
burn      ASH (U; destroyed-pending-clear)                      ← exit only via clear
open      PLAIN_LBTC (wallets/sponsors)   CPFP_ANCHOR (zero-value fee hook)
```

### §2.1 Open vs closed — the recognition spine · `sec:architecture:open-closed`

Two asset families exist. **Closed** assets — `U`, `ENT`, `DIST_CTL`, `PID`, `PACE`, `ENT_AUTH`, `DIST_AUTH` — can be created only by the covenant's own issuance rules; their scarcity is consensus fact. **Open** assets — `L-BTC`, anything foreign — can be shaped into *anything* by *anyone*: a request-shaped junk UTXO, a decoy reserve, a fake state carrying any committed fields.

> **⚠ Naïve construction.** A safety predicate rejects a malformed object wherever it appears — the invariant refuses a request-shaped UTXO whose fields are inconsistent, a reserve-shaped output at the wrong value, a state-shaped object with a bad counter.
>
> **Why it fails (the invariant denial-of-service).** On a permissionless chain, anyone can place an object shaped like anything into the UTXO set. A predicate that can *fail* on an externally-placeable object is adversary-haltable: one junk UTXO and the audit reports violation forever, at the cost of dust.
>
> **What we do.** A safety predicate may fail only on **closed** objects — those the covenant alone could have produced, whose existence is already a consensus-scarce fact. Objects built from **open** assets are **inert until consumed under a rule**: they are recognized only at the moment an operation spends them under that operation's validation, never as standing state. Closed strictness is *licensed by* native-asset scarcity; open inertness is *required* by open-asset forgeability. `[enforced: P-open]` `[invariant: 𝗜₁]` — **canonical: every later "inert until consumed" points here.** `trap:architecture:open-closed`

> **⚠ Naïve construction.** Recognize a protocol object by its program shape: "has the pool script," "has the ash code," "carries a reserve-shaped output," "commits plausible fields."
>
> **Why it fails (identity forgery).** A public script is copyable and a committed field is attacker-chosen. Any recognition keyed on a property the attacker can choose recognizes the attacker.
>
> **What we do.** Recognition keys only on unforgeable properties: a canonical **asset id** (the closed family), a **consumed predecessor** (the tracked root cursors), a **transition lineage** (the derived history of (`sec:kernel:certificate`)), or **actual consensus value** — never a committed scalar standing alone, never a bare script match on a payload-bearing key. Its four load-bearing applications all point here: ash value read from consensus, addresses attested only as payloads of authenticated burns, the pool recognized by `PID` at its cursor, the reserve recognized by asset ∧ program ∧ provenance. `[enforced: P-ident, P-open]` — **canonical.** `trap:architecture:recognize-unforgeable`

### §2.2 Two closed domains, one bridge · `sec:architecture:domains`

The closed assets partition into two accounting domains: **money** (`U`, held in receipts, the vault, and ash) and **entitlement** (`ENT` in deposit entitlements; `DIST_CTL` in distribution controls). The domains are disjoint, each with its own invariant fold ((`inv:invariant:accounting`), (`inv:invariant:escrow-receipts`)), and they meet at exactly **one** operation: settlement (`sec:operations:settle`), which destroys entitlement and routes money. Every cross-domain preservation argument in the document localizes to that bridge; every other operation is provably domain-internal.

### §2.3 One asset, two covenant classes · `sec:architecture:one-asset`

The claim is **one** native asset, `U`. The live and time-locked classes are **covenant classes** — the `RECEIPT_L` and `RECEIPT_T` object kinds — not two assets. This realizes the specification's "one conserved receipt, two redemption classes — classes, not assets" (`[A-def:model:classes]`): the single floor over the combined supply (`[A-def:model:ratefloor]`) is the single asset's conservation, and maturity conversion is a relabel touching neither supply nor backing (`[A-def:maturity:conversion]`).

> **⚠ Naïve construction.** `U` is a native asset, so consensus conservation already guarantees no inflation — the covenant need do nothing.
>
> **Why it fails (G5).** `U` is **reissuable**: every non-empty cycle creates `ΔY` new units. For a reissuable asset, consensus permits whoever exercises the reissuance authority to mint *any* amount. Conservation gives no-forgery free; it gives no-inflation not at all.
>
> **What we do.** `PACE` is `U`'s **sole** reissuance authority, consumable only by the cycle operation, with the issued amount pinned to `ΔY` and every minted output enumerated to fixed destinations. `[enforced: P-issue, P-mint]` `trap:architecture:conservation-inflation`

### §2.4 The pool: STATE and RESV, welded · `sec:architecture:soul-vault`

*The welded pair · `listing:architecture:weld-shape`*

```
STATE : asset PID (genesis-unique, amount 1); commits S = (Ω, Y_L, Y_T, Q, k, Mat)
RESV  : asset L-BTC (explicit, unblinded); value = Ω + Q; program RESV_SPK
```

Identity must be unforgeable, and the only unforgeable on-chain identity is a genesis-unique asset — so identity is `PID`. Backing must be actual reserve units (`[A-def:model:reserve-pool]`). An Elements UTXO carries one asset; the pool is therefore a **pair**, inseparable in both directions: RESV moves only alongside its STATE, and every operation that touches either recreates the reserve at the correct value under the correct program. The weld is *operationally conditioned* — its full statement, including the one legitimate exit (the sealing redemption) and the one exempt operation (clear, which touches neither `Ω` nor `Q`), is the certificate section's `trap:branches:sealing-terminal`; the standing property is 𝗜₅ (`inv:invariant:backing`). `[enforced: P-value, P-ident]` There is no read-only access to the pool: **every STATE read is a succession edge** — the state is authenticated because the STATE input is consumed and recreated, a derived fact of (`sec:kernel:certificate`), not a leaf convention.

### §2.5 Authorities off state · `sec:architecture:authorities`

Three closed assets are reissuable — `U`, `ENT`, `DIST_CTL` — and each has its own genesis-unique, amount-one authority token: `PACE`, `ENT_AUTH`, `DIST_AUTH`. None of them is `PID`, and none lives on the pool pair.

> **⚠ Naïve construction.** Reuse the pool identity `PID` as an issuance authority — a genesis-unique token already exists; why mint three more?
>
> **Why it fails (G5).** Binding issuance to `PID` makes **every STATE-spending operation** — including the least-exercised — a surface on which the asset could be minted. Safety would rest on each such operation *remembering* to assert "issued = 0": fail-open, forever.
>
> **What we do.** Each reissuable asset's authority is a distinct token held **off** STATE and consumed only by the one operation that issues it (`PACE` → cycle; `ENT_AUTH` → admission; `DIST_AUTH` → cycle). An operation that does not hold the authority **cannot** mint the asset — unauthorized issuance is impossible by absence, never by remembered suppression. `[enforced: P-issue, P-distinct]` `trap:architecture:authority-off-state`

> **⚠ Naïve construction.** Use two tokens for the cycle — one as the cadence clock, one as the mint authority.
>
> **Why it fails (desync).** The cadence needs a consensus-honest "blocks since last cycle" read; issuance needs an authority usable only at a cycle boundary. Two tokens can be spent in different transactions and drift apart.
>
> **What we do.** One token. `PACE` is simultaneously the clock — its confirmation depth *is* the cadence age (`rule:translation:timelocks`) — and `U`'s reissuance authority, so `reissuance ⟺ PACE-spend ⟺ cycle` holds by construction. The consumer's own clock is decoupled; `A` flows continuously and is sampled at the consumer's boundary (`[A-rem:interface:clocks]`). `[enforced: P-issue, P-distinct]` `trap:architecture:two-tokens`

### §2.6 The deposit pipeline · `sec:architecture:deposit-pipe`

*The pipeline · `fig:architecture:pipeline`*

```
create-request ──▶ admit-deposits ──▶ cycle ──▶ settle-distribution
 (open offer;       (mint ENT,          (mint ΔY U + control      (destroy ENT; pay
  cancelable)        one per request;    + vault; escrow Q → Ω)     floored pro-rata)
                     Q += Σδ)
```

A deposit (`[A-def:operations:deposit]`) begins life as an **open offer** — a request-shaped L-BTC UTXO committing `(pool, refund key, receipt owner, principal)` — inert until admission consumes it under the request rule (`trap:architecture:open-closed`), cancelable by its refund key meanwhile. Admission converts offers into **entitlements**; the cycle mints the supply and escrows the contributors' share; settlement pays it out.

> **⚠ Naïve construction.** A deposit issues a covenant-keyed claim ticket whose `(owner, amount, target)` are committed fields, redeemed later against the cycle's escrow.
>
> **Why it fails (G5, G10 — the entitlement-forgery attack).** A ticket that carries no scarce asset has fields forgeable for dust: an attacker commits one naming themselves for a cohort's whole remaining share and co-spends it against the genuine escrow through a permissionless settlement, minting real escrowed `U` to themselves — while any checker that folds over "all tickets" *assumes* each is genuine, an assumption nothing on-chain enforces. The forgery is invisible to exactly the predicate meant to catch it.
>
> **What we do.** The entitlement **is a consensus-scarce asset**: `ENT`, issued only by admission, only against a real deposit, in the deposit's exact amount, under `ENT_AUTH`. Holding an entitlement *means* having deposited; forging one requires the reserve it represents — forgery is indistinguishable from participation, and no rule remains to get wrong. `[enforced: P-ent, P-admit]` `[invariant: 𝗜₇]` — **canonical: the deposit-forgery closure.** `trap:architecture:entitlement-scarcity`

### §2.7 Distribution objects: control and vault · `sec:architecture:dist`

Each non-empty cycle creates one **distribution control** — `DIST_CTL`, amount one, carrying the counters: principal, per-class allocations, remainders — and one **vault** — `U`, the escrowed value itself. Counters and value live in different objects of different assets, bound by the payability bijection 𝗜₆ (`inv:invariant:no-starve`); the two-counter argument is the settlement section's `trap:branches:control-vault`. Settlement is permissionless and owner-preserving — the reasons are `trap:branches:settlement-sig`.

### §2.8 Burn machinery: ash, records, and no accumulator · `sec:architecture:burn-ledger`

Burning is the highest-frequency, widest-fan-out operation the consumer drives (`[A-def:operations:burn]`), and the datum it produces — an address `a ∈ 𝔸` — is attacker-chosen and asset-unbacked, exactly as the specification's addresses-not-actors clause demands (`[A-def:interface:address-space]`). The machinery is shaped by scale first and authentication second.

> **⚠ Naïve construction.** Route every burn through the pool pair, and commit the cumulative burn history in a succinct on-chain accumulator so any verifier reads attestation from one root.
>
> **Why it fails (scale · no real verifier).** Welding the busiest operation to the single STATE UTXO serializes the system's hottest path behind its scarcest resource. A succinct root serves only verifiers that do not track the chain — a population that does not exist here: full-chain verifiers recompute `A` from history, which is strictly more trustless, and ordinary users trust their Layer-1 view regardless. A naïve root is not even self-verifying without ZK or fraud proofs, both excluded. Maintaining the root is itself the unbounded fold script cannot perform (`rule:translation:fold-elimination`).
>
> **What we do.** There is **no accumulator** — an attestation-root role is *not representable*: the root set is exactly the five declared roots, checked structurally at build and by 𝗜₁ at every state, and the decision is closed with recorded rationale (`app:realization:architecture`). Burn splits into a **share-nothing hot path** (burn: receipts → ash, touching no pool object) and a **permissionless batched cold path** (clear: ash → supply decrement). The record is the consensus-immutable payload of an authenticated burn transition; the map `A` is an off-chain recomputation, `O(burns + clearings + records)` per address over a pre-indexed canonical event stream. `[enforced: P-distinct, P-ledger]` `[invariant: 𝗜₁]` — **canonical: the no-accumulator decision.** `trap:architecture:accumulator`

The lag between a burn and the clearing that reflects it into `φ` is carried by the ash term of 𝗜₈ (`inv:invariant:accounting`): `φ` is **understated** in the interim — **the lazy φ-rise**, conservative throughout, inside the upper-bound form the specification states SP6 in (`[A-prop:interface:conservative-valuation]`). **This is its canonical introduction; later mentions point at (`res:trust:locked-residue`).**

> **⚠ Naïve construction.** Record the burned amount as a committed scalar on a marker UTXO — a "burn flag" carrying the number `x`.
>
> **Why it fails (G2 — φ-inflation).** Clearing lowers `Y`, raising `φ`. A committed, forgeable amount is a direct floor-inflation vector: write a large `x` you never backed, clear it, and over-draw the reserve against every other holder.
>
> **What we do.** Ash carries its **consensus-true `U` value** — the destroyed units *are* the ash output's amount, read from consensus at clear. You cannot fund an ash with `U` you never held (`trap:architecture:recognize-unforgeable`). `[enforced: P-clear]` `trap:architecture:burn-flag`

> **⚠ Naïve construction.** The attestation ledger *is* the set of tagged record payloads; an indexer sums the records carrying each address.
>
> **Why it fails (G3 — the free-attestation exploit).** Record emission is permissionless: anyone can emit `(a, x)` payloads in a cheap transaction that spends no receipt, creates no ash, destroys nothing. A scanning indexer attests them for free — (`[A-thm:interface:instantaneous-costliness]`) breaks at precisely the layer where the cost is supposed to be paid.
>
> **What we do.** A record is attestation-bearing only as the payload of an **authenticated burn transition**, under the dual anchor of (`sec:ledger:authentication`): the transaction must *be* a burn (event-type anchor) and its records are capped by its destruction — `Σx ≤ F(T)`, fail-closed. `[enforced: P-burn, P-ledger]` `trap:architecture:opreturn-ledger`

> **⚠ Naïve construction (the over-correction).** Close the hole on-chain instead: validate records in script — one record per ash, forbid change, forbid multi-input burns.
>
> **Why it fails.** It puts application data (`a`) into consensus where nothing consumes it, and it forbids useful shapes for no gain: the real risk is the *inequality* `Σ attestation > Σ destroyed` — a per-transaction sum, not a cardinality.
>
> **What we do.** Records stay off the on-chain validation surface entirely; the inequality is enforced off-chain against the unforgeable ash total. Multi-input, multi-output, and live-class change are all free and safe (`trap:branches:burn-change`). `[enforced: P-burn, P-ledger]` `trap:architecture:over-correction`

> **The tag register · `rem:architecture:tag-register`**
>
> Six domain-separated destruction/record tags exist — `tag-burn`, `tag-recon` (clear), `tag-redeem`, `tag-entitlement` (settlement's `ENT` destruction), `tag-distribution-control-close`, `tag-distribution-residue` — pairwise distinct, each welded one-to-one to its declaring operation's destruction delta (`sec:kernel:canonical-partition`). **Exactly one participates in attestation** (`tag-burn`), and participation is a manifest field, not prose. `[enforced: P-tag, P-distinct]`

> **Scale — initiation unbounded, clearing demand-driven · `rem:architecture:scale`** `[honesty note]`
>
> Distinct burns spend distinct receipts and share no UTXO: **initiation throughput is unbounded**, across funders and across addresses. **Clearing throughput is not**: clear rejoins the single STATE, so reflection into `φ` proceeds in batches at the STATE-serialization rate. The asymmetry is safety-neutral and demand-driven (`sec:ledger:convergence`): clearing happens when some agent finds the φ-rise worth a fee, not on a schedule the system must meet.

### §2.9 The floor is never materialized · `sec:architecture:rate-not-materialized`

`φ = Ω/Y` is never computed as a number on-chain. Every floor fact is an **integer cross-multiplication** — "the floor does not fall" is `Ω'·Y ≥ Ω·Y'` (`lem:invariant:rate`). `φ = Ω/Y` is rational whenever `Ω` and `Y` are integers, but is generally non-integral and need not have an exact finite representation in the target's selected fixed-width encoding. Materializing it therefore requires an explicit representation and rounding policy, and any spender-selected approximation is an extraction vector (G1).

> **The one exception, off-chain · `rem:architecture:offchain-rate`**
>
> The attestation ledger materializes `φ_c = Ω_c/Y_c` at clearing events, reading both terms from the clearing's committed state, purely to value attestation increments (`sec:ledger:order`) — the conservative valuation SP6 licenses (`[A-prop:interface:conservative-valuation]`). This never feeds a covenant decision; it is an indexer computation over public data.

### §2.10 Two clocks · `sec:architecture:two-clocks`

> **⚠ Naïve construction.** Protocol schedules — cadence and maturity — are denominated in block height, the unit consensus timelocks speak.
>
> **Why it fails (G7/G8 — the two-clocks confusion).** Height is the wrong clock for maturity: the specification denominates the maturity schedule in **cycles** (`[A-postc:maturity:announcement]`), and a height-denominated latch can express only a maximum lead — it silently drops the minimum lead and imports a height↔cycle conversion constant nobody audits.
>
> **What we do.** Two clocks, each native to its consumer. **Cadence** is consensus-relative — blocks since the last cycle, a confirmation-depth fact read by relative timelock — because rate-limiting is a chain property. **Maturity** is committed-`k` arithmetic — both lead bounds checked against the spent STATE's own cycle counter (`sec:operations:announce`) — because the schedule is a protocol property. No conversion constant between the clocks exists anywhere in the covenant, and the maturity path carries **no timelock at all**. `[enforced: P-flavor]` `[invariant: 𝗜₁₁]` `trap:architecture:two-clocks`

---

## §3 The manifest, and how to read it · `sec:realization:manifest`

The appendix (`app:realization:architecture`) is a typed declaration, generated from a normative source whose type system and validators constrain what is representable, exported deterministically, and bound by a semantic hash. This section is the reading key: what each field means, what a *well-formed* manifest guarantees, and how the hash and the label weld bind the appendix to this document. It explains; it does not reproduce (`rem:overview:register-authority`).

### §3.1 Field semantics · `sec:manifest:fields`

*The field glossary · `tab:manifest:fields`*

| Family | Fields (per entry) | Meaning |
|---|---|---|
| `assets` | `code, id, class, role, fixed_amount, reissuable, authority, issue_operation, destruction_operations` | `class` is open/closed (`trap:architecture:open-closed`); a reissuable asset names its **authority asset** and its **sole issuing operation**; `destruction_operations` is the exact set of operations declaring a destruction delta for it — **derived, not asserted** |
| `roots` | `code, id, asset, role, fixed_amount` | the five constant-cardinality roots; every non-reserve root is a closed amount-one asset |
| `objects` | `code, id, asset, lifecycle, accounting_domain, allocators, mutators, deallocators, authorization_paths, witnesses, consensus_value_authoritative` | the fourteen recognized UTXO kinds; `allocators`/`mutators`/`deallocators` name every operation that may create/alter/consume the object; `authorization_paths` are **derived** from operation inputs, not stored |
| `operations` | `code, id, kind, authorization, roots, issuances, inputs, outputs, canonical_deltas, data_outputs, open_flows, reads, writes, witnesses, value_flows, bounds, projections` | the thirteen transitions; each input carries its own authorization mode; each destruction delta names its tag; `projections` declare which derived events (certificate, burn, clear, residue) the operation must/may induce |
| `quantities` | `code, id, kind, reads, writers, readers` | the eight named quantities and the reader matrix (`sec:ledger:reader-matrix`); `kind` ∈ monetary / interface / audit-only / derived |
| `witnesses` | `code, id, semantic_tag` | the fourteen proof families; `semantic_tag` is a **document label of this document** — the register-1 → document weld (`rem:manifest:weld`) |
| `clauses` | `code, id` | the eleven invariant clauses, exported so the clause table of (`sec:invariant:clauses`) is generated, never re-typed |
| `dependencies` | `code, id, verification_required, rationale` | the standing `{verify}` surface (`sec:trust:verify`) — a *requirement* for evidence, never a completed status |
| `decisions` | `code, id, status, rationale` | the seven closed architecture decisions with recorded rationale (`sec:trust:decisions`) |
| `bounds` | `code, id, default_value, requires_deployment_calibration` | the ten finite batch/script bounds; every `default_value` is a **calibration placeholder**, a draft default, never a shipped constant |
| `amount_limits` | `code, id, asset, value, unit, rationale` | fixed protocol maxima; presently one, the active-backing cap (`trap:domains:active-backing`) |
| `tags` | `code, id, participates_in_attestation` | the six domain tags (`rem:architecture:tag-register`); exactly one participates |
| evidence tables | `input_authorization_evidence`, `operation_authorization_evidence` | generated: the evidence class backing each authorization mode at model, compiler, and deployment layers (`sec:realization:authorization`) |

Every `code` is a stable numeric discriminant; every `id` is a stable string. Both are published API (`rem:manifest:discriminants`).

### §3.2 Well-formedness and closure · `sec:manifest:wellformed`

A manifest is well-formed only if it passes draft validation — a body of cross-declaration rules that make the appendix self-checking rather than a listing. A reader auditing the toml applies the same rules by hand; a conforming model derives its policy from the declarations and checks the relations **bidirectionally** (`obl:oracle:manifest`).

*The closure rules · `tab:manifest:closure`*

| Rule | Statement |
|---|---|
| **complete coverage** | every declared identifier family is present exactly once, with no duplicates and no omissions — assets, roots, objects, operations, quantities, witnesses, clauses, dependencies, decisions, bounds, limits, tags |
| **issuance ↔ delta** | every issuance declaration has exactly one matching issuance delta, and conversely; the issuing operation consumes and recreates the named authority; a singleton root asset is never issued or destroyed |
| **destruction ↔ data output** | every destruction delta names a tag and is matched by exactly one data output of that tag and asset, and conversely |
| **root-use ↔ cardinality** | an operation's declared root use (forbidden / succession / succession-or-termination) agrees exactly with its input/output cardinalities for that root's object |
| **authorization legality** | every root input is a **covenant companion** (`trap:branches:companion-auth`); no non-root object claims companion status; refund-key authorization only on the request; sponsor authorization only on plain reserve inputs; owner authorization only on owner-bearing objects — where owner-bearing means *consent gates consumption*, not merely *carries an owner field* |
| **permission coherence** | owner-authorized operations declare at least one owner-signed input; refund-key operations exactly one refund input; cadence-band authorization is reserved to the cycle and requires its clock root; permissionless and operator classes declare no owner or refund inputs |
| **bound coverage** | the bounds an operation declares are exactly the bounds its cardinalities reference; every minimum is satisfiable under the bound's draft default |
| **witness completeness** | canonical deltas require the delta witness; root use requires the corresponding succession witness; receipt input/output closure requires the class-closure witness; every operation carries the value-flow-closure witness; an open fee-sponsor flow requires the sponsor-envelope value flow |
| **reader firewall** | audit-only quantities have no operation readers and no writers; the residue quantities are readable by the invariant checker and external auditor **only** — wiring them into any monetary, interface, or consumer reader is a validation failure (`trap:ledger:residue-decrement`) |
| **object lifecycle** | every allocator produces the object, every mutator and deallocator consumes it, and — conversely — every operation consuming an object holds a mutator or deallocator role for it, every producer an allocator or mutator role (external-wallet objects excepted, as ordinary wallet activity) |
| **decision closure** | the seven named decisions are present and closed; the root set is exactly the five roots (no accumulator representable); only `tag-burn` participates in attestation |

### §3.3 Stable identifiers, the hash, the schemas, and the weld · `sec:manifest:hash`

> **Discriminants and the semantic hash · `rem:manifest:discriminants`**
>
> Published `code` values are never reordered or renumbered; new identifiers arrive only under a deliberate schema-version bump, and the schema version rides in every export. The semantic hash is computed over a **canonicalized body** — object keys sorted, set-like arrays sorted at export, unknown fields rejected — under the algorithm the attached envelope names, which prefixes that body with a domain separator, so it binds *semantics*: formatting, comments, declaration order, and presentation encoding cannot move it. A retired algorithm is never reused: each retirement, with the measurement it made and the value it pinned, is recorded with its migration rather than folded into the standing identifier. The behavioural hash (`pin:pins:denotation`) is separate and gates versioning. Publication status, the schema version, and the realization version are envelope metadata, deliberately **excluded** — changing any of them does not change what the architecture *is*. The deployment profile carries its own hash under a domain-separated algorithm; the two are never interchangeable (`sec:oracle:boundary`).
>
> **The three schemas, co-located.** This document lives at three independent schema versions, distinguished once here and read at their use sites: the **architecture** manifest's, carried in the attached envelope and in every export; the **attestation wire** format's, carried in the canonical query context (`sec:ledger:model`); the **deployment profile**'s, carried in the profile a deployment release binds (`sec:trust:verify`). They advance independently; conflating them is the drift this co-location exists to prevent.

> **The label weld · `rem:manifest:weld`**
>
> The manifest carries document labels in two places: every witness's `semantic_tag`, and the exported clause registry. The distinct welded set is **sixteen strings** — the eleven clause labels of (`sec:invariant:clauses`) plus (`lem:invariant:delta`), (`lem:invariant:rate`), (`lem:invariant:flow`), (`sec:ledger:authentication`), and (`res:trust:op`) — and a conformance test holds each **verbatim** against this document: the manifest is generated, the document is hand-written, so the weld guards the hand-written side. Witnesses may cite any frozen label, not only goal labels — the attestation-authentication witness cites the ledger's dual-anchor home, and the fee-auction witness cites the operational residual whose model content is exactly "every valid contender is safe." Everything else in this document's label API is guarded by the ship checklist, not by CI; the weld's scope is exactly these sixteen strings, stated so the boundary is honest (`rem:pins:weld-scope`).

---

## §4 Domains and arithmetic · `sec:realization:arithmetic`

### §4.1 Newtypes and bounds · `sec:arithmetic:domains`

*Amounts and the master bound · `listing:domains:sat` — model-normative*

```rust
pub const TWO_51: u64 = 1 << 51;                           // master arithmetic bound
pub const ACTIVE_BACKING_MAX: u64 = 2_100_000_000_000_000; // Ω + Q ceiling; < 2^51 (build-checked)

/// Reserve-unit amount. Newtype invariant: 0 ≤ v < 2^51.
pub struct Sat(u64);
impl Sat {
    pub fn new(v: u64) -> Result<Sat, Guard>            // rejects v ≥ 2^51    (rule:translation:sat)
    pub fn positive(v: u64) -> Result<Sat, Guard>       // additionally rejects 0
    pub fn checked_add(self, o: Sat) -> Result<Sat, Guard> // checked; result re-bounded
    pub fn checked_sub(self, o: Sat) -> Result<Sat, Guard> // checked; underflow rejected
}
```

*Baked rationals · `listing:domains:ratio` — model-normative*

```rust
/// Newtype invariant: 0 < num < den < 2^10 — strict, so ratio 1 is
/// unrepresentable. Realizes the published constants ζ, f
/// (A-def:model:fee-and-split), whose magnitudes are calibration
/// outputs, jointly chosen (A-rem:model:calibration) — the 1/2 values
/// used in examples are placeholders, never specified values.
pub struct Ratio { pub num: u16, pub den: u16 }
```

The strict open interval `0 < num < den` transcribes the specification's constants `f, ζ ∈ (0,1)` exactly (`[A-def:model:fee-and-split]`): a zero rate and a unit rate are unrepresentable by construction, not by convention.

> **Why these two bounds are the entire overflow proof · `rem:domains:bounds`**
>
> `Sat < 2⁵¹` and `Ratio < 2¹⁰` together keep every product in the system below the signed-64-bit trap on the small-constant path (`2⁵¹·2¹⁰ = 2⁶¹ < 2⁶³`) and delimit the wide path (`2⁵¹·2⁵¹ = 2¹⁰²`), which the model computes in 128-bit arithmetic and an emitted script must realize by limb decomposition (`sec:arithmetic:gadgets`). Every witnessed or minted amount re-enters through `Sat::new`, so the bound is re-established at every boundary rather than assumed to persist.

> **⚠ Naïve construction.** The per-field bounds already imply the pair is safe: `Ω` and `Q` are each capped, so `Ω + Q` is fine, and `2⁵¹` is the only ceiling backing needs.
>
> **Why it fails (the misread bound).** Two field caps do not bound a sum: `2 · ACTIVE_BACKING_MAX > 2⁵¹`. And `2⁵¹` is the *arithmetic-safety* trap, not an economic cap — nothing about multiplication limits how much reserve the pool may simultaneously hold.
>
> **What we do.** Carry the **active-backing cap** as an explicit invariant term: `Ω + Q ≤ ACTIVE_BACKING_MAX < 2⁵¹`, enforced at genesis, at admission, at the cycle, and by the invariant. It is a perpetual bound on *simultaneously active* reserve plus admitted escrow — **not** on cumulative deposit volume: redemption restores headroom, and an admission batch over the cap fails closed with every request left independently re-admissible. The economic cap subsumes the arithmetic trap for the pair; both are stated because they fail for different reasons. `[enforced: P-cap, P-arith]` `[invariant: 𝗜₂]` `trap:domains:active-backing`

The guard vocabulary below is a **machine-held projection** of the model's `Guard` and `InvariantError` enums — pasted from the checked fixture, never hand-authored. Inventing a variant, or omitting one, is a build failure caught by the projection test, not a prose defect; the listing therefore names the *guard that fired*, while the eleven clauses name the *property violated*, and the exported clause registry maps the first onto the second (`rem:invariant:reasons`).

*The guard and invariant-error vocabulary · `listing:domains:guard` — a checked projection of the model enums*

```rust
pub enum Guard {                               // why a *transition* was rejected
    // recognition & shape
    NoSuch, WrongAsset, WrongShape, WrongPool, WrongTarget, WrongClass,
    // domains & arithmetic
    Domain, BadConstant, Overflow, Underflow, CycleOverflow,
    ActiveBackingCapExceeded,                  // the 𝗜₂ cap, by name
    // construction exactness
    DuplicateInput, DuplicateOutputIndex, MissingOutputIndex,
    // authorization
    BadSignature, BadAuthorization,
    // terminal & progress discipline
    Sealed, NoTrap, ZeroProgress,              // sealing, no-trap, and zero-progress paths
    // value & recipient pins
    OverDraw, ValuePin, PartitionPin, RecipientPin, ClassCross,
    // issuance & destruction
    MissingAuthority, BadIssuance, BadDestruction,
    // the two exact partitions
    CanonicalDeltaMismatch, OpenFlowMismatch,
    // fee & sponsor
    SponsorMismatch, FeeMismatch,
    // roots & welds
    RootMultiplicity, RootSuccession, ResvWeld, ControlVaultWeld,
    // cadence & maturity
    CadenceTooEarly, CadenceOperatorOnly,
    MaturityAlreadyAnnounced, MaturityLeadTooShort,
    MaturityLeadTooLong, MaturityNotComplete,
    // history, checkpoint, codec
    WrongCheckpoint, UnsupportedSchema, HistoryOrder, DuplicateEvent,
    // wrapper
    InvariantFailure,
}

pub enum InvariantError {                      // which *state/audit* clause failed → clause_of
    IdentityAuthority, CanonicalClosure,       // 𝗜₁
    Domains, ActiveBackingCap,                 // 𝗜₂
    Floor,                                     // 𝗜₃ — Y > Ω lives here, not in Guard
    SealedTerminal,                            // 𝗜₄
    Backing,                                   // 𝗜₅
    DistributionPayability,                    // 𝗜₆
    EntitlementLifecycle,                      // 𝗜₇
    ReceiptAccountingPreMaturity,
    ReceiptAccountingPostMaturity,             // 𝗜₈
    ConsensusValueAuthority,                   // 𝗜₉ — declared, never produced
    StateSuccession, ResvSuccession,
    HistoryProjection,                         // 𝗜₁₀
    MaturityCoherence,                         // 𝗜₁₁
}
```

The vocabulary is deliberately finer than the eleven clauses; `Y > Ω` is `InvariantError::Floor` (𝗜₃), an invariant error rather than a transition `Guard`, and `ConsensusValueAuthority` is declared but never produced — its clause is discharged structurally (`inv:invariant:consensus-value`). This partition is why O3 binds to **named reasons** and not merely to clauses (`obl:oracle:reductions`).

### §4.2 The four gadgets — witnessed division, and the sole explicit low-level how · `sec:arithmetic:gadgets`

Every protocol quotient in the system — and there are exactly four families: the cycle's issuance `ΔY`, the redemption payout `p`, the settlement draws `m_c`, and the cycle's split/fee shares — is **witnessed and verified, never computed**. The spender supplies the quotient; the script pins it uniquely.

> **⚠ Naïve construction.** Verify a witnessed quotient with the lower bound only: accept `q` whenever `q·D ≤ N`.
>
> **Why it fails (G1 — the under-quotient extraction).** A spender free to witness a *too-small* quotient chooses where the difference goes. On a user-facing payout the spender pockets the remainder; on an issuance the recipient set is quietly shorted. One missing inequality converts every division site into a discretion point.
>
> **What we do.** Pin the quotient from both sides — the **sandwich** — together with the wrap guard `q < 2⁵¹` so `(q+1)·D` cannot overflow the signed trap, which fixes `q = ⌊N/D⌋` **uniquely**: no witness other than the true floor satisfies both bounds. Flooring every user-facing quantity keeps the remainder in the pool — the rounding direction *is* G1's discharge at each site. `[enforced: P-arith]` `trap:translation:lower-bound`

$$q·D ≤ N < (q+1)·D$$

*The two model-normative division forms · `listing:arithmetic:floor`*

```rust
/// ⌊a·b/d⌋ for a, b < 2^51 — the wide path (product < 2^102, computed in u128).
/// d = 0 aborts: a divisor of zero is unreachable on a live pool (Ω ≥ Y ≥ 1)
/// and rejected outright rather than reasoned away.
pub fn floor_mul_div(a: Sat, b: Sat, d: Sat) -> Result<Sat, Guard>

/// ⌊n·num/den⌋ — the small-constant path (product < 2^61).
pub fn floor_ratio(n: Sat, r: Ratio) -> Result<Sat, Guard>
```

Call sites: `floor_mul_div` at the cycle (`ΔY = ⌊QY/Ω⌋`), redemption (`p = ⌊xΩ/Y⌋`), and settlement (`m_c = ⌊δ·alloc_c/principal⌋`); `floor_ratio` at the cycle's class split and fee share. **Burn, clear, transfer, relabel, and compaction perform no division** — the hot paths are division-free by design.

> **The hand-audit surface · `rem:arithmetic:hand-audit`** ◆
>
> This is the one place the *how* surfaces below the model, because it is the one place the goal register cannot reach: the invariant speaks in `Sat`-level relations, and no clause can witness the **carry, limb, and borrow correctness** of the multi-precision machinery an emitted script needs to realize the wide path — a wide multiply, a borrow-propagating compare, and the two sandwich verifiers (the quotient form and the ratio form). Those four gadgets are specified here as **input→output contracts**: each MUST implement exactly its model function's relation, pinned by test vectors, with every product below the trap under the (`rem:domains:bounds`) bounds — and their internal construction is `{verify}`, discharged by hand-audit en route to a machine-checked port (`res:trust:hand-audit`). The exception is load-bearing for the whole document: everywhere else the how is implicit because a clause witnesses it; here the witness is a vector suite and a named residual, stated as such.

---

## §5 The kernel and the transition certificate · `sec:realization:kernel`

The kernel is the model-level relation between a declared operation and an accepted transition. An operation presents the objects it consumes, the objects and data outputs it creates, the closed-asset movements and issuances it claims, the open-value flows it uses, and its requested canonical order. The kernel accepts only if those declarations form one exact, manifest-conforming transaction; it then **derives** the transition certificate from the actual consumed and created objects. Operation code does not author provenance.

The kernel proves **structural transaction validity** at the model layer: exact conservation and partitioning, manifest-derived shape, root use, event provenance, and branch postconditions. It does not independently prove cryptographic authorization. Operation constructors establish the model's signer relation (`sec:realization:authorization`); exact signature bytes, sighash behaviour, covenant predicates, and faithful script emission remain `{verify}` obligations (`sec:oracle:boundary`).

A conforming script need not reproduce any particular kernel implementation. It MUST enforce a transaction relation equivalent to the accepted relation defined here and preserve every clause of (`sec:realization:invariant`). The translation of each derived certificate relation into emitted leaves is T13 (`rule:translation:certificate-leaf`) and is witnessed by P-weld (`pin:pins:weld`) `{verify}`.

### §5.1 Generic transaction construction · `sec:kernel:builder`

A conforming transition model MUST stage all effects before commit:

1. consume a distinct set of existing inputs;
2. emit a finite ordered set of candidate outputs and non-spendable data outputs;
3. declare every closed-asset flow and issuance;
4. declare the open-flow membership of every ordinary L-BTC input and output;
5. validate the exact canonical partition (`sec:kernel:canonical-partition`);
6. validate root-input policy and closed-asset conservation;
7. validate the exact open-flow partition (`sec:kernel:open-flow`);
8. validate manifest-derived input/output/data-output cardinalities;
9. derive the transition certificate (`sec:kernel:certificate`);
10. validate branch-specific semantic postconditions;
11. update root cursors from the derived certificate;
12. append the certificate to history and commit atomically.

If any step fails, the predecessor state and history remain bit-for-bit unchanged. This is L-atomic (`lem:invariant:atomic`): rejection is a pure result, not a partial mutation. A conforming model MUST make the property structural — for example, by computing over a fresh successor and publishing it only after validation — rather than relying on rollback discipline.

> **Kernel proof boundary · `rem:kernel:proof-boundary`**
>
> The kernel proves structural validity, not authorization. Its low-level construction path MUST NOT be a public path by which an external caller can assemble a structurally valid transition while bypassing the declared owner, refund-key, operator, or cadence authorization. The public transition surface is the closed operation set declared by (`app:realization:architecture`) — a sealed set, unextendable from outside the model (`[rule:verification:pure-transition]`) — and each operation validates its abstract authorization before invoking the kernel. Model signer membership is proven there; compiler `CHECKSIG` placement and deployment sighash semantics remain `{verify}` (`rem:authorization:evidence`).

> **Uniform construction rules · `rem:kernel:uniform-rules`**
>
> The following rules apply once, here, rather than being repeated per operation:
>
> - an input outpoint is consumed at most once; duplication is rejected before any value is counted;
> - every recognized closed-asset value-bearing output is positive;
> - the first-party constructor emits no explicit zero-valued ordinary L-BTC output — canonical construction policy, not a protocol predicate ((`rule:representation:sponsor-canonicality`)); the declared `CPFP_ANCHOR` carries value zero as its own semantic role and is recognized by family, never by testing an ordinary output for zero ((`rule:representation:anchor-identity`));
> - every output has one canonical output index, and no output index is cited twice by a witness;
> - at most one generic fee-sponsor envelope appears in a transaction;
> - any rejected construction leaves the predecessor unchanged.
>
> These are model requirements. Their enforcement by emitted script and consensus transaction shape is `{verify}` under P-weld, P-flow, and P-sponsor ((`pin:pins:weld`), (`pin:pins:flow`), (`pin:pins:sponsor`)).

### §5.2 The exact canonical partition · `sec:kernel:canonical-partition`

The *canonical value assets* are `U`, `ENT`, and `DIST_CTL`. Every consumed input and every created output of those assets belongs to exactly one declared issuance or flow. Singleton identity and authority assets are governed separately by root succession and closed-asset conservation.

A canonical flow names one canonical asset; one non-empty, distinct set of consumed source outpoints of that asset; a distinct set of created destination outputs of that asset; zero or more positive, domain-tagged destruction legs; and a movement kind if and only if positive current-state value remains. Each flow satisfies the exact value relation:

$$Σ value(sources) = Σ value(destinations) + Σ amount(destructions)$$

An issuance names the issued asset, its authority asset and consumed authority outpoint, the positive issued amount, and the complete set of destination outputs. It satisfies:

$$issued = Σ value(issuance destinations)$$

No source may witness two flows. No output may be funded by two flows, or by both a flow and an issuance. No consumed canonical object may be omitted, and no created canonical object may remain unwitnessed.

> **⚠ Naïve construction.** A branch that changes closed-asset supply proves it with a balanced aggregate, `Σ inputs + issued = Σ outputs + destroyed`, while issuance is licensed by the presence of exactly one authority token.
>
> **Why it fails (G5 — the aggregate-alibi attack).** A balanced total authorizes no *particular* movement. It cannot distinguish issuance from theft, a lateral move from destruction, or one party's units from another's; two wrong flows can offset exactly. A static count of one authority token proves only that the token was neither lost nor duplicated — not that it was consumed to authorize this transaction's issuance.
>
> **What we do.** Every canonical source and destination belongs to exactly one witness, and every delta is authorized **by kind**: **issuance** consumes the named authority and exhausts the issued amount into its named outputs; **destruction** carries a domain-separated tag and exactly one matching non-spendable data output; **lateral** preserves value into current-state outputs; **ownerless-lateral** preserves value among ownerless protocol objects. The kernel derives the certificate's actual deltas from these exact witnesses; the resulting active delta-family set MUST equal the manifest's expected set for the operation and its activation conditions. An aggregate total never authorizes a canonical delta. `[enforced: P-delta]` `[invariant: 𝗜₈]` — **canonical.** `trap:branches:canonical-delta`

A destruction data output is evidence of a destruction leg, not a spendable UTXO. Its tag and asset MUST match exactly one destruction delta. `tag-burn` is the sole exception in *kind*: it is a burn-record family rather than a destruction data output, because the burn operation moves `U` laterally into `ASH`; actual `U` destruction occurs at clear (`rem:operations:burn-lateral`).

> **Stable fault ordering · `rem:kernel:fault-order`**
>
> The exact canonical partition is validated before root-input policy. Consequently, a consumed or created canonical object that is missing, duplicated, or cited by two witnesses reports a canonical-partition reason regardless of which roots the operation touches. Failure reasons remain stable under composition; O3 binds to those reasons (`obl:oracle:reductions`), while the document maps them onto clauses through (`rem:invariant:reasons`).

### §5.3 The exact open-flow partition · `sec:kernel:open-flow`

Open assets are not protocol-issued and remain inert outside an accepted operation (`trap:architecture:open-closed`). When an operation consumes or creates ordinary (`PLAIN_LBTC`) value, however, every such input and output — zero-valued members included — belongs to **exactly one** declared open flow; membership is decided by the declared family, never by the amount ((`sec:kernel:sponsor-opacity`)). Each flow balances independently:

$$Σ value(sources) = Σ value(destinations) + fee$$

*The open-flow roles · `tab:kernel:flow-roles`*

| Role | Sources | Permitted destinations | Purpose |
|---|---|---|---|
| `request-creation` | ordinary owner-authorized L-BTC inputs | one deposit request + optional owner change | creates the open offer; carries its own explicit chain fee |
| `request-refund` | one deposit request | full-value refund to the committed refund key | cancellation never funds its own fee from the refund |
| `deposit-admission` | active RESV + admitted requests | successor RESV + optional admission reward | moves principal into active backing; request budgets fund reward + fee |
| `reserve-carry` | active RESV | successor RESV of equal value | cycle preserves backing while moving escrow `Q` into `Ω` in committed state |
| `redemption` | active RESV | formula-bound payout + optional successor RESV | reserve release; this flow itself carries zero fee |
| `fee-sponsor` | ordinary sponsor-owned L-BTC inputs | optional sponsor change | generic explicit chain-fee envelope |

The sum of all open-flow fees equals the transaction's chain fee. An ordinary L-BTC input in no flow, an output in no flow, or any source/destination cited twice is rejected. Foreign open assets are conserved if an operation happens to carry them, but they have no protocol flow role and cannot satisfy any L-BTC obligation.

A `CPFP_ANCHOR` does not enter the partition: it is excluded by its declared family — it is a package-fee hook, not a semantic value flow — never by a zero-value test on ordinary outputs (`res:trust:op`).

> **⚠ Naïve construction.** L-BTC conservation is sufficient: if the transaction balances, its fee, refund, payout, reserve successor, and sponsor change are all "somewhere in the outputs."
>
> **Why it fails (G10 — the fee-reroute attack).** Conservation proves the total, not the purpose. An unpartitioned fee can silently shrink a formula-bound redemption payout; sponsor value can be mixed into reserve accounting; a request refund can be redirected while the transaction still balances exactly.
>
> **What we do.** Every ordinary L-BTC input and output belongs to exactly one declared role, each role balances internally, and branch postconditions pin every formula-bound or immutable destination. The generic sponsor envelope is one distinct role: sponsor-owned inputs may produce only sponsor change plus the explicit fee, and **at most one** such envelope appears in a transaction. Sponsor value cannot alter another role's payout because it is witnessed separately. `[enforced: P-flow, P-sponsor]` (`lem:invariant:flow`) `trap:branches:sponsor-envelope`

> **Primary-flow fees and admission discretion · `rem:kernel:inline-fees`** `[honesty note]`
>
> `create-request` and `admit-deposits` do not use the generic sponsor envelope. Request creation's own owner-funded flow pays its fee. Admission consumes request values partitioned into principal and preauthorized service budget; the admitter chooses a reward no greater than the sum of those budgets, and the remainder is the chain fee. No pool quantity depends on that reward/fee split — `Ω`, `Q`, entitlements, cycle issuance, and receipt allocations depend only on principal — so the choice is service compensation inside a depositor-authorized envelope, not G7 discretion over the economy (`req:requirements:no-discretion`).

### §5.4 The transition certificate and separable root edges · `sec:kernel:certificate`

The kernel derives one certificate for every accepted transition from the transaction's actual consumed and created objects. The certificate records provenance; it is not an object operation code is permitted to author.

*The transition certificate · `listing:kernel:certificate` — model-normative; emitted enforcement is `{verify}` (`pin:pins:weld`)*

```rust
enum RootEdge {
    Succ { input: OutPoint, output: OutPoint },
    Term { input: OutPoint },
}

struct TransitionCertificate {
    txid: TxId,
    order: CanonicalOrder,
    branch: BranchKind,

    consumed: Set<OutPoint>,
    created: Set<OutPoint>,

    state_edge: Option<RootEdge>,
    resv_edge: Option<RootEdge>,
    pace_edge: Option<RootEdge>,
    entitlement_authority_edge: Option<RootEdge>,
    distribution_authority_edge: Option<RootEdge>,

    canonical_deltas: Vec<CanonicalDelta>,
    open_flows: Vec<OpenFlowProjection>,
    chain_fee: Sat,

    burn: Option<BurnProjection>,
    clear: Option<ClearProjection>,
    distribution_residue: Option<DistributionResidueProjection>,
}
```

For each root, the manifest declares the operation's root-use policy: **forbidden** — the active root is not consumed and no successor of its root shape is created; **succession** — the active root is consumed and exactly one valid successor is created; **succession-or-termination** — the active root is consumed and either one successor is created or the root chain terminates. `STATE`, `PACE`, `ENT_AUTH`, and `DIST_AUTH` are non-terminating. `RESV` alone may terminate, and only through the sealing redemption (`sec:operations:redeem`).

The kernel derives an edge by selecting the actual current root input and the actual created successor of the required canonical shape. A branch cannot claim a successor it did not create, omit a required consumed root, or create a forbidden root output. Root cursors update **only** from the certificate after derivation and postcondition validation.

> **⚠ Naïve construction.** Pool provenance is one paired STATE↔RESV weld event reproduced by every pool operation.
>
> **Why it fails (G2, G9 — the false-weld gap).** A single paired event cannot represent an operation that moves STATE but not RESV (`clear`, `receipt-relabel`, `announce-maturity`), nor the sealing redemption that consumes RESV without recreating it. Forcing the pair serializes state-only operations behind collateral for no safety gain; exempting them ad hoc leaves their STATE reads witnessed by nothing.
>
> **What we do.** The certificate carries **separable root edges**, one per root, derived from actual inputs and outputs. Every STATE read is a real succession edge: the active STATE input is consumed, the successor is recreated, and the kernel derives that fact. RESV alone admits termination. State-only operations have a STATE edge and no RESV edge by declared policy — no special "read-only state" primitive exists. `[enforced: P-cert]` `[invariant: 𝗜₁₀]` `trap:branches:cert-single-weld`

> **⚠ Naïve construction.** Backing integrity is the unconditional identity `RESV = Ω + Q`, required to recreate RESV on every pool transition.
>
> **Why it fails (G10 — the forbidden exit).** The sealing redemption — the final live claim with no pending escrow — pays the entire reserve and intentionally consumes RESV. An unconditional weld forbids the exact-dead terminal and strands the final holder.
>
> **What we do.** The weld is **operationally conditioned**. While `Y > 0`, an active RESV exists at the tracked cursor with value `Ω + Q`; the sealing redemption emits a RESV termination edge and a sealed STATE successor with `Y = Ω = Q = 0`. After termination, STATE is a tombstone and no later pool transition may recreate RESV. `[enforced: P-value, P-cert]` `[invariant: 𝗜₄]` `[invariant: 𝗜₅]` (`trap:branches:sealing-terminal`)

Three specialized event projections are derived alongside the generic certificate: a **burn projection** exists only for the burn operation, with live-receipt canonical inputs, no ASH input, exactly one fresh ASH output, all remaining `U` outputs live receipt change, and canonically indexed records; a **clear projection** exists only for clear and is derived from the predecessor/successor STATE edge plus the authenticated `tag-recon` destruction amount; a **distribution-residue projection** exists only at terminal settlement, derived from the consumed control/vault, destroyed entitlements, created receipts, control closure, and residue destruction. Tags are consistency aids; event type comes from the whole derived transition, never from a free-standing tag (`trap:ledger:event-anchor`).

Every certificate relation described here is model-proven. Its realization as an emitted covenant predicate is `{verify}` under T13 (`rule:translation:certificate-leaf`) and P-weld (`pin:pins:weld`): the deployment witness is the emitted-script hash plus relation-indexed differential script vectors, not this document.

### §5.5 Full root-history replay · `sec:kernel:replay`

A current cursor proves only an endpoint. A conforming auditor MUST replay the entire root history from the trusted genesis projection:

1. transitions are strictly ordered and transaction identifiers unique;
2. each certificate's consumed and created sets are disjoint;
3. no outpoint is consumed twice or created twice;
4. no outpoint is created after it was already consumed;
5. each root edge is checked against the root active **immediately before** that transition;
6. forbidden roots remain untouched;
7. a termination edge records the sealed STATE successor;
8. after RESV termination, no transition may spend STATE or recreate RESV;
9. replayed root cursors equal the current tracked cursors.

> **Why the path matters · `rem:kernel:intermediate-corruption`**
>
> Comparing only the final cursor misses an intermediate bad edge if a later certificate restores the expected endpoint. Full replay witnesses the **path**, not merely its last pointer: corrupting an earlier STATE, RESV, or PACE input is detected even when every current cursor appears correct. Replay intentionally does not re-execute every non-root UTXO semantic — the transition kernel remains authoritative for full input/output validity — but it independently authenticates the succession history that current state alone cannot witness. This is 𝗜₁₀ (`inv:invariant:succession`).

### §5.6 Recipient safety is not conservation · `sec:kernel:recipient`

> **⚠ Naïve construction.** A branch is value-safe once each asset conserves in aggregate.
>
> **Why it fails (G10 — the balanced-theft attack).** Conservation does not protect recipients. Alice's `5 → 4` and Bob's `7 → 8` preserve a total of twelve while transferring one unit without Alice's authorization; a formula-bound payout can be shortened while another output grows by the same amount.
>
> **What we do.** Every value-moving operation declares its authorization class, and a closure lemma checks **every** asset flow against one of eight value-flow classes: **owner-consented** — all consumed owners authorize the complete output set; **immutable-destination** — the destination is committed before the triggerer acts; **formula-bound payout** — both recipient and amount are fixed by public state; **preauthorized service budget** — the source committed the maximum spend beforehand; **ownerless terminal sink** — value is destroyed under a domain-separated tag; **sponsor envelope** — sponsor-owned inputs fund only sponsor change plus fee; **ownerless bound sink** — value enters an ownerless object whose exits are closed; **ownerless bound movement** — value moves only among such ownerless objects. The taxonomy is a completeness register, not a proof by naming: branch-specific postconditions and the exact partitions are the witnesses. A permissionless operation may advance, settle, relabel, consolidate, or clear, but it may never choose an economic recipient or capture value. `[enforced: P-flow]` (`lem:invariant:flow`) — **canonical.** `trap:branches:recipient-safety`

### §5.7 Consensus value is authoritative · `sec:kernel:consensus-value`

The model carries one value per UTXO; a committed-versus-consensus divergence is structurally unrepresentable. That does not make the corresponding deployment property automatic: an emitted covenant reads actual explicit values through substrate introspection, and `{verify}` must establish that the values it validates are the values consensus enforces.

> **⚠ Naïve construction.** A recognized object's committed metadata amount is the amount used by accounting; consensus conservation of its asset handles the rest.
>
> **Why it fails (G2, G9 — reality-blind accounting).** A receipt can commit face `100` while carrying one actual `U`; another output can carry the remaining `99`. The committed sums balance and consensus sums balance, but the correspondence between them is false. The checker becomes complete over committed space and blind to reality — exactly the failure safety accounting exists to catch.
>
> **What we do.** Actual consensus value is authoritative. A receipt, entitlement, ASH, vault, or RESV amount is its real asset value, not a parallel metadata scalar. Metadata may **partition** a value only where every leg is pinned to a real movement whose parts sum exactly to that value: request `gross = principal + service budget`; distribution vault `value = remaining_live + remaining_timelocked`. It may never impersonate value. `[enforced: P-value, P-explicit]` `[invariant: 𝗜₉]` `trap:branches:committed-value`

In the model, 𝗜₉ is structural evidence: the one value field is the object's value, and no runtime comparison exists because there is nothing distinct to compare (`inv:invariant:consensus-value`). On-chain discharge is explicit-value availability plus issuance/value introspection `{verify}` under P-explicit (`pin:pins:explicit`); O3 exercises the partition seams through 𝗜₆/𝗜₇ corruption, never the unrepresentable identity core.

---

### §5.8 Sponsor-value opacity · `sec:kernel:sponsor-opacity`

Partition all L-BTC inputs and outputs of an accepted operation into disjoint protocol and sponsor regions:

$$I_{\mathrm{LBTC}} = I_P \uplus I_S,\qquad O_{\mathrm{LBTC}} = O_P \uplus O_S.$$

Let $f_P$ and $f_S$ be the fee contributions assigned to the protocol and sponsor regions. Target-wide L-BTC conservation establishes $V(I_P)+V(I_S)=V(O_P)+V(O_S)+f_P+f_S$; the branch-specific protocol relation independently establishes $V(I_P)=V(O_P)+f_P$. Subtracting the authenticated protocol relation from target-wide conservation gives:

$$V(I_S)=V(O_S)+f_S.$$

The derivation requires no individual sponsor amount to be decoded or opened. For confidential values, $V$ denotes the exact consensus-enforced commitment relation; the subtraction is commitment cancellation, not public scalar inspection.

Because sponsor references are disjoint from protocol references ((`sec:kernel:open-flow`)), sponsor value cannot satisfy, shorten, enlarge, or redirect any protocol payout, refund, reserve successor, issuance destination, state assignment, destruction, or event. Because every sponsor input owner authorizes the final transaction, sponsor value cannot be consumed without sponsor consent. Therefore protocol safety is independent of the individual values of $I_S$ and $O_S$, provided the complete transaction remains target-valid. In particular, adding or removing an authorized zero-valued sponsor member changes transaction shape and resources but leaves the protocol semantic projection ((`def:representation:sponsor-erasure`)) unchanged. The proof holds for Pedersen commitments or any additive confidential-value relation; it does not assume public amounts.

> **⚠ Naïve construction.** Requiring every sponsor amount to be strictly positive is a cheap extra safety check.
>
> **Why it fails (positivity is not the boundary).** Suppose a redemption payout is reduced by one and sponsor change enlarged by one: $R+S=(R-p)+(p-1)+(c+1)+f$ still conserves, and $c+1>0$, so positivity never fires. The attack is rejected because the protocol relation pins $p_{\mathrm{actual}}=\lfloor x\Omega/Y\rfloor$ ((`sec:kernel:recipient`)); once the payout and RESV successor are authenticated, sponsor conservation follows as the residual relation. Positivity detects nothing the exact protocol relations do not already pin, and it costs a value read the protocol has no right to ((`rule:representation:inspection-necessity`)).
>
> **What we do.** Authenticate sponsor *role* — asset, family, exact membership, source/destination uniqueness, sponsor/protocol disjointness, owner authorization, at-most-one envelope — and sponsor *conservation* through the substrate's exact value or commitment relation. Individual sponsor amounts stay outside the protocol read-set on both sides; a zero-valued ordinary sponsor member is accepted whenever that role structure is exact. The balanced-theft vector is a permanent regression. `trap:kernel:sponsor-positivity`

---

## §6 The operations · `sec:realization:operations`

The appendix (`app:realization:architecture`) is authoritative on the operation set and on each operation's structural declaration — roots, authorization, inputs, outputs, issuances, deltas, data outputs, flow roles, witnesses, bounds, and projections. This section supplies what the manifest does not: each operation's **semantic delta**, its irreducible arithmetic, and the postcondition that joins it to the kernel lemmas of (`sec:realization:kernel`).

### §6.0 The operation contract · `sec:operations:generic`

For every operation, a conforming realization MUST:

1. accept only the manifest-declared input and output families within their calibrated cardinality bounds;
2. satisfy the manifest-declared root-use, issuance, canonical-delta, data-output, open-flow, and projection relations;
3. establish the operation-specific state and recipient postconditions stated here;
4. preserve every invariant clause not modified by the semantic delta;
5. satisfy L-floor, L-atomic, L-delta, L-flow, and L-record where applicable.

The generic obligations — exact partitions, root derivation, conservation, shape, certificate construction, atomic rejection — are stated once in (`sec:realization:kernel`) and are **not** repeated per operation.

> **The non-compressible semantic core · `rem:operations:noncompressible`** ◆
>
> The manifest carries each operation's *shape*, not its economic arithmetic. The cycle issuance `ΔY = ⌊QY/Ω⌋`, redemption payout `p = ⌊xΩ/Y⌋`, settlement allocation `m_c = ⌊δ·alloc_c/principal⌋`, clear clamp, and sealing-terminal condition are therefore irreducible content of this document. Each is stated with the specified law, invariant clause, and flooring direction that force it. A density pass MUST NOT remove one on the ground that the operation appears in the appendix.
>
> Physical partitioning of an output set is otherwise implementation-flexible: where the goal fixes only an owner/class/value multiset, a conforming realization may split or aggregate outputs within the manifest's calibrated bounds. Conformance is semantic, not byte-identical (`rem:overview:conformance-claim`).

### §6.1 Create a deposit request · `sec:operations:request`

`create-request` is an owner-authorized client operation over open L-BTC. It touches no pool root and changes no closed-asset state. It consumes one or more ordinary owner-authorized L-BTC inputs and emits exactly one `DEPOSIT_REQUEST`, committing the target pool, refund key, receipt owner, and deposit principal; optional ordinary L-BTC change to an authorized owner; and an explicit chain fee carried by the request-creation flow.

For request value `W` and principal `δ`, a conforming request satisfies:

$$0 < δ < W,  b = W − δ > 0$$

where `b` is the **preauthorized admission budget**. The request's full consensus value remains open L-BTC; `δ` and `b` partition that real value and do not impersonate it (`trap:branches:committed-value`).

A request is not standing pool state. It remains inert until admission consumes it under the rules of (`sec:operations:admit`), and may be canceled by its refund key meanwhile (`trap:architecture:open-closed`). Creating malformed or cross-pool request-shaped L-BTC does not violate the invariant; such an object simply fails admission.

Preservation follows from open-flow exactness and the absence of closed-state effects. The request-creation flow itself funds the fee; no generic sponsor envelope appears (`rem:kernel:inline-fees`).

### §6.2 Cancel a deposit request · `sec:operations:cancel`

`cancel-request` consumes exactly one deposit request under its committed refund-key authorization. It returns the request's **full gross value** to that refund key:

$$W_refund = W_request$$

Any chain fee is funded by a separate sponsor envelope; it MUST NOT reduce the refund. The request-refund flow contains exactly one request source and exactly one formula- and destination-pinned refund output. The consumed request is not recreated, so cancellation cannot replay.

The receipt owner carried by the request is a routing destination for future settlement, not the cancellation authority. A receipt-owner signature without the refund-key signature is insufficient (`sec:realization:authorization`). No pool root or closed asset is touched; preservation follows from L-flow (`lem:invariant:flow`) and the request's open-object lifecycle.

### §6.3 Admit deposits · `sec:operations:admit`

`admit-deposits` converts a non-empty bounded batch of valid open requests into consensus-scarce deposit entitlements. The cardinality bound `ADMISSION_BATCH_MAX` is a deployment-calibrated limit, not a protocol constant.

For requests with principals `δ_i`, let `D = Σ δ_i`. Admission preserves `Ω`, `Y_L`, `Y_T`, `k`, and maturity status, while updating:

$$Q' = Q + D,  RESV' = RESV + D = Ω + Q'$$

Admission MUST enforce the active-backing cap:

$$Ω + Q' ≤ ACTIVE_BACKING_MAX$$

A batch exceeding the cap fails atomically, leaving every request unspent and independently admissible later — for example after redemption restores headroom (`trap:domains:active-backing`).

For each request, admission issues **exactly one** `ENT` output: value `δ_i`, owner equal to the request's committed receipt owner, target cycle `k + 1`. Issuance consumes and recreates `ENT_AUTH`, and the total issued `ENT` equals `D`. The entitlement owner is a future receipt-routing destination; it is not a consent gate on settlement.

> **⚠ Naïve construction.** Merge admitted requests by owner or target before issuing entitlements, reducing the number of outputs.
>
> **Why it fails (G7, G10 — the aggregation-discretion attack).** Floored allocation is superadditive — `⌊δ₁D_c/Q⌋ + ⌊δ₂D_c/Q⌋ ≤ ⌊(δ₁+δ₂)D_c/Q⌋`. An admitter allowed to choose whether requests merge can move receipt value by choosing the grouping before a nonlinear operation.
>
> **What we do.** Admission emits **one entitlement per request**. Every entitlement's principal and owner remain independently recognizable until settlement, and each floor is applied per entitlement before any physical output aggregation. Never aggregate before a rounded or nonlinear operation unless aggregation-invariance is proved. `[enforced: P-admit, P-ent]` `[invariant: 𝗜₇]` `trap:branches:admit-merge`

> **Admission budget and reward · `rem:operations:admission-budget`** `[honesty note]`
>
> Each request's preauthorized budget is `b_i = W_i − δ_i`. Admission may pay an admitter-chosen reward `R` only within `0 ≤ R ≤ Σ b_i`, with `F_chain = Σ b_i − R`. The choice affects only service compensation versus chain fee inside an envelope each depositor authorized beforehand. No pool quantity — `Ω`, `Q`, `Y`, entitlement value, cycle issuance, or receipt allocation — depends on `R`. This is not G7 discretion over the economy (`req:requirements:no-discretion`); it is a bounded choice over a preauthorized service budget (`lem:invariant:flow`).

**Dust-deposit boundary.** If a later cycle has `Q > 0` but issues `ΔY = 0`, the admitted entitlements settle for zero and their principal is absorbed into `Ω`. This is conservative and permissionlessly retired, but it is a real depositor loss, catalogued as R-dust (`res:trust:dust`); it is not hidden as a failed admission.

### §6.4 Run a cycle · `sec:operations:cycle`

`cycle` is the only operation that advances `k`, moves admitted escrow into settled reserve, exercises the `U` reissuance authority, and creates a per-cycle distribution. Its operation authorization is the cadence band (`sec:realization:authorization`); its root inputs remain covenant companions.

Let the predecessor state be `S = (Ω, Y_L, Y_T, Q, k, Mat)` with `Y = Y_L + Y_T`. The successor reserve and cycle are:

$$Ω' = Ω + Q,  Q' = 0,  k' = k + 1$$

The active RESV's actual L-BTC value does not change during the cycle: before the transition it is `Ω + Q`; after the state update it is `Ω' + Q' = Ω + Q`.

**Cycle issuance law.** The receipt issuance is:

$$ΔY = ⌊QY/Ω⌋$$

This relation is forced by issuing admitted reserve at the predecessor floor (`[A-prop:invariants:issuance-preserves-floor]`), with flooring in the pool's favour. Since `Y ≤ Ω`, `ΔY ≤ QY/Ω ≤ Q`, so `Y + ΔY ≤ Ω + Q = Ω'`. Equivalently, the floor does not fall:

$$(Ω+Q)·Y ≥ Ω·(Y+ΔY)$$

This is the cycle instance of L-floor (`lem:invariant:rate`).

An empty cycle (`Q = 0`) issues nothing but still consumes and recreates the cadence/issuance and distribution-authority roots, advances `k`, re-arms cadence, and executes maturity conversion if `k'` is the announced maturity cycle. The clock cannot reset without exercising the cycle transition, including in the zero-issuance case.

**Class split and fee share.** Before maturity conversion, issuance splits by the published live-share parameter, flooring the live part:

$$ΔY_L = ⌊ζ·ΔY⌋,  ΔY_T = ΔY − ΔY_L$$

From the conversion cycle onward, all issuance is live: `ΔY_L = ΔY`, `ΔY_T = 0`. The formula-fixed mint fee is floored separately from each class:

$$O_c = ⌊f·ΔY_c⌋,  C_c = ΔY_c − O_c$$

where `O_c` is the operator share and `C_c` the contributor allocation. The operator receives ordinary receipts in the matching class; there is no separate fee escrow or claim path.

> **Post-maturity fee class · `rem:operations:fee-class`**
>
> The class split — **including the fee share's class** — exists only in the bootstrapping phase. From the conversion cycle onward, `ΔY_T = 0`, so the operator's entire fee share is live-class. This follows from the normal-phase law (`[A-def:model:normal-phase]`); it is not a special operator exception.

**Distribution creation.** If `Q > 0`, cycle issues one `DIST_CTL` under `DIST_AUTH` and creates one distribution control for cycle `k'` with `principal = remaining_principal = Q`, `live_allocation = remaining_live = C_L`, `timelocked_allocation = remaining_timelocked = C_T`. A distribution vault is created iff `C_L + C_T > 0`, with actual `U` value equal to that sum. The operator's ordinary receipt outputs and the vault exhaust the issued `U` exactly. If `Q > 0` but `ΔY = 0`, the control still exists with zero allocations and no vault, so the entitlements are permissionlessly retired at zero draw — R-dust (`res:trust:dust`). If `Q = 0`, the cycle creates no control, no vault, and no `DIST_CTL` issuance.

At the maturity-conversion cycle, a zero-value `CPFP_ANCHOR` is also created as a package-fee hook. It carries no economic value and does not enter an open flow; its usability depends on package-relay deployment evidence `{verify}` (`sec:trust:verify`). The manifest's output row for the anchor is unconditioned; the maturity-only condition is enforced by the cycle branch postcondition — a branch-semantic register fact, not a structural one.

**Atomic maturity conversion.** Let `at_maturity ⟺ Mat = Announced(k')`. If false, class state updates normally: `Y_L' = Y_L + ΔY_L`, `Y_T' = Y_T + ΔY_T`. If true, conversion is part of the same cycle:

$$Y_L' = Y_L + Y_T + ΔY,  Y_T' = 0,  Mat' = Complete$$

No announced-but-unconverted state exists. Physical `RECEIPT_T` objects and pre-existing distribution time-locked allocations may persist; after conversion they account on the live side and migrate permissionlessly through (`sec:operations:relabel`).

> **⚠ Naïve construction.** Let whoever fires a permissionless cycle direct the fresh issuance, or allow the operator to choose the issued amount.
>
> **Why it fails (G5, G10 — the cycle-capture attack).** Reissuable native-asset conservation permits any amount authorized by the reissuance token. Without an amount pin and an exhaustive recipient set, a triggerer can issue excess `U` or route valid issuance to themselves.
>
> **What we do.** The cycle alone consumes `PACE`, the sole `U` authority; the transaction carries no `U` inputs; issued `U` is exactly `ΔY`; and every issued output is exhausted into the fixed set `{operator receipts, distribution vault}` with class and amount pinned by the equations above. `DIST_CTL` issuance likewise consumes `DIST_AUTH` and equals exactly one when `Q > 0`. `[enforced: P-issue, P-mint, P-delta]` `[invariant: 𝗜₈]` `trap:branches:cycle-capture`

> **⚠ Naïve construction.** An empty cycle issues nothing, so it need not consume the cadence and issuance authority.
>
> **Why it fails (G7, G8 — the free-clock-reset attack).** With no issued asset for consensus to challenge, a zero-escrow branch could advance `k` and reset cadence without spending the object whose age defines the cadence. The protocol and consensus clocks diverge.
>
> **What we do.** Every cycle, empty or not, consumes and recreates `PACE` and advances `k` exactly once. Empty cycles carry the same root-succession and cadence obligations as non-empty ones. `[enforced: P-ident, P-issue]` `[invariant: 𝗜₁₀]` `trap:branches:empty-cycle`

> **⚠ Naïve construction.** Put an operator signature check on the cycle operation, since the operator paces cycles inside the early cadence band.
>
> **Why it fails (G8 — cycle ransom).** The same operation must become permissionless after the anti-stall ceiling. A branch-wide operator signature re-ransoms the forced path.
>
> **What we do.** The operation's authorization is the composite cadence band: before the minimum nobody; inside the band the operator; at and after the maximum anyone. Root inputs remain covenant companions, and the generic cycle semantics contain no unconditional operator signature (`sec:realization:authorization`). `[enforced: P-ransom, P-companion]` `trap:branches:cycle-opkey`

### §6.5 Settle a distribution · `sec:operations:settle`

`settle-distribution` is the **sole bridge** between the entitlement and receipt-accounting domains (`sec:architecture:domains`). It is permissionless and consumes exactly one distribution control; its vault iff the control's remaining class value is positive; a non-empty bounded batch of entitlements targeting the control's cycle; and optionally one sponsor envelope. `SETTLEMENT_BATCH_MAX` is deployment-calibrated.

For one entitlement of principal `δ` and class `c ∈ {L,T}`, the draw is:

$$m_c(δ) = ⌊δ·D_c/Q_k⌋$$

where `Q_k` is the control's original principal and `D_c` its original class allocation. The floor applies **per entitlement before any output aggregation**. For the batch, `P = Σ δ_i` and `M_c = Σ m_c(δ_i)`. The successor counters are:

$$R_Q' = R_Q − P,  R_c' = R_c − M_c$$

Every consumed entitlement is destroyed under `tag-entitlement`, in total amount `P`. Receipt outputs route `M_L` and `M_T` to the entitlements' committed owners in the corresponding classes. Outputs may be physically aggregated by owner and class within manifest bounds; the owner/class/value multiset is the semantic fact. The floor-superadditivity relation

$$Σ ⌊δ_i·D_c/Q_k⌋ ≤ ⌊(Σ δ_i)·D_c/Q_k⌋$$

is what makes the control's class remainders dominate all future draws, preserving 𝗜₆ (`inv:invariant:no-starve`).

**Continuing settlement.** If `R_Q' > 0`, settlement recreates the control with its original principal and allocations and updated remainders, and one vault iff `R_L' + R_T' > 0` with actual `U` value exactly that sum. `DIST_CTL` moves laterally; the vault `U` moves laterally into receipt outputs plus the successor vault.

**Terminal settlement.** If `R_Q' = 0`, settlement creates no successor control and no successor vault. It destroys the control's amount-one `DIST_CTL` under `tag-distribution-control-close`, and any `U` residue `H_L = R_L'`, `H_T = R_T'` under `tag-distribution-residue`. The terminal residue projection records the two class components separately for audit. Recorded `Y` is **not** decremented: the residue enters the historical terms of 𝗜₈, keeping `φ` understated in the safe direction (`trap:ledger:residue-decrement`).

> **⚠ Naïve construction.** Store only one mutable distribution balance and let both settlement payouts and terminal cleanup decrement it.
>
> **Why it fails (G6, G10 — the claim-against-residue attack).** One balance does not witness how much principal remains, which class future claims draw from, or whether a cleanup has consumed value still owed. An interleaving can destroy headroom required by outstanding entitlements, or pay the same headroom twice.
>
> **What we do.** The control carries **remaining principal and per-class allocation remainders**; the vault carries the real `U`. Their bijection is invariant: one vault exists iff the class sum is positive, and its value equals that sum. Settlement alone updates both, per-entitlement floors keep all future claims payable, and terminal closure occurs only at remaining principal zero. `[enforced: P-counter, P-settle]` `[invariant: 𝗜₆]` — **canonical.** (`trap:branches:control-vault`)

> **⚠ Naïve construction.** Require every entitlement owner to sign settlement, because the value belongs to them.
>
> **Why it fails (G8, G10 — the lost-key stuck-state leak).** Settlement is a shared-state hygiene operation. One lost owner key would retain the control and vault forever, blocking cleanup and converting a self-custody safeguard into ransom.
>
> **What we do.** Settlement is **permissionless and owner-preserving**: no entitlement-owner signature is required, while every receipt recipient and amount is fixed by the consumed entitlement and public control arithmetic. A random triggerer can advance settlement but cannot choose a recipient or capture value. `[enforced: P-settle, P-ransom, P-flow]` (`lem:invariant:flow`) (`trap:branches:settlement-sig`)

> **Committed classes survive delayed settlement · `rem:operations:committed-classes`** `[honesty note]`
>
> Settlement always pays the distribution's original class allocations, even after global maturity conversion. A pre-conversion entitlement settled later may therefore receive a physical `RECEIPT_T`; it then migrates through permissionless relabel. The logical conversion (`Y_T := 0` at the maturity cycle) and physical migration of outstanding objects are distinct facts. Post-conversion receipt accounting includes both physical classes on the live side (`inv:invariant:accounting`).

### §6.6 Transfer receipts · `sec:operations:transfer`

Two operations implement transfer — one for live receipts, one for time-locked receipts. Their shapes are disjoint and class-closed. Each transfer consumes a non-empty bounded set of receipts of exactly one class, with every consumed owner represented in the signer set, and emits a non-empty bounded set of positive receipt outputs of the **same** class:

$$Σ value(inputs) = Σ value(outputs)$$

`TRANSFER_INPUT_MAX` and `TRANSFER_OUTPUT_MAX` are deployment-calibrated. The destination owners and denominations may differ from the inputs; aggregate class value may not. No STATE or RESV root is touched; the canonical `U` delta is lateral. A generic sponsor envelope may fund the chain fee without entering the receipt flow.

> **⚠ Naïve construction.** Use one generic receipt-transfer operation that recognizes either class at input and any receipt-shaped output at destination.
>
> **Why it fails (G6 — class crossing in both directions).** A time-locked holder can route value into live receipts before maturity; or route it into ASH and simulate a pre-maturity burn. The first creates immediately redeemable value; the second lets the structurally unburnable class influence the burn ledger and live-supply clearing.
>
> **What we do.** The manifest declares two disjoint operations. Live transfer admits only `RECEIPT_L` inputs and outputs; time-locked transfer admits only `RECEIPT_T` inputs and outputs. Neither admits ASH, reserve, distribution, or bare-key `U` destinations. A mixed-class transfer is unrepresentable because the two closure predicates cannot be satisfied by one output set. `[enforced: P-transfer]` (`req:requirements:two-classes`) `trap:branches:transfer-generic`

### §6.7 Redeem a live receipt · `sec:operations:redeem`

`redeem` consumes exactly one positive live receipt under its owner's authorization, plus STATE and the active RESV as covenant companions. Time-locked receipts are structurally absent from the operation.

For receipt value `x`, total supply `Y`, and reserve `Ω`, the payout is:

$$p = ⌊xΩ/Y⌋$$

The receipt's `U` is destroyed under `tag-redeem`; it is not routed to ASH and never participates in attestation. The payout is an explicit L-BTC output to the receipt owner, and the redemption flow itself carries zero chain fee. A sponsor envelope may fund the fee separately.

For a non-sealing redemption:

$$Ω' = Ω − p,  Y_L' = Y_L − x,  Y_T' = Y_T,  Q' = Q$$

Because `p ≤ xΩ/Y`, we have `Ω − p ≥ Ω(Y−x)/Y`, so without division:

$$(Ω−p)·Y ≥ Ω·(Y−x)$$

This is the redemption instance of L-floor (`lem:invariant:rate`); flooring preserves or raises the floor in the pool's favour (`[A-prop:invariants:redemption-preserves-floor]`).

**Sealing redemption.** The unique terminal case is `x = Y ∧ Q = 0`. Then `p = ⌊YΩ/Y⌋ = Ω`, the payout exhausts RESV, the receipt exhausts supply, the successor STATE commits `Ω' = Y_L' = Y_T' = Q' = 0`, and the certificate carries a RESV termination edge. STATE remains as the sealed tombstone required by 𝗜₁₀ (`inv:invariant:succession`). Any `x ≥ Y` outside that exact case MUST be rejected; a receipt value greater than recorded live supply is likewise rejected.

> **⚠ Naïve construction.** Let a holder redeem the last unit whenever they possess it, even if admitted deposits remain pending.
>
> **Why it fails (G10 — stranded escrow).** Draining `Ω` while `Q > 0` leaves the admitted request principal in RESV with no live supply against which the next cycle can issue.
>
> **What we do.** The exact-dead path requires `x = Y ∧ Q = 0`; otherwise redemption requires `x < Y`. RESV termination is derived only in the exact-dead case, and no later pool transition may revive it. `[enforced: P-notrap, P-value, P-cert]` `[invariant: 𝗜₄]` `[invariant: 𝗜₅]` `trap:branches:redeem-drain`

### §6.8 Relabel time-locked receipts · `sec:operations:relabel`

`receipt-relabel` is permissionless, owner-preserving, and available only after maturity is complete. It consumes STATE as a succession root and a non-empty bounded batch of time-locked receipts; it recreates STATE byte-identically and emits live receipts preserving every input owner and value. `RELABEL_BATCH_MAX` is deployment-calibrated.

The model-level semantic requirement is an exact multiset bijection:

$$⦃(owner, value)⦄ over RECEIPT_T inputs = ⦃(owner, value)⦄ over RECEIPT_L outputs$$

No value is issued, destroyed, or redistributed; the `U` delta is lateral. Global `Y_L`, `Y_T`, `Ω`, `Q`, `k`, and maturity state remain unchanged — logical conversion already occurred in the maturity cycle. A sponsor envelope may fund the fee.

> **The relabel evidence split · `rem:operations:relabel-split`** `[honesty note]`
>
> The **model-proven half** is the owner/value multiset bijection and the `Mat = Complete` gate. The **script-side half** `{verify}` is a canonical positional layout that makes the relation locally checkable: STATE successor first, one live receipt per consumed time-locked receipt in input order, optional sponsor change afterward. The positional rule is not the safety property; it is one efficient emitted realization of it. A conforming alternative script may use a different locally verifiable layout if it proves the same multiset relation and passes the differential obligations of P-relabel (`pin:pins:relabel`).

An owner signature is deliberately absent. Requiring it would turn lost-key time-locked receipts into permanent shared-state residue; permissionless owner preservation is the safety property (`trap:branches:settlement-sig`).

### §6.9 Burn live receipts · `sec:operations:burn`

`burn` is an owner-authorized, share-nothing operation over live receipts. It touches no root, no reserve, no pool state, and performs no division.

A burn consumes a non-empty bounded set of live receipts, with every consumed owner in the signer set, and emits exactly one positive ASH output; optional positive live-receipt change outputs; zero or more canonically indexed burn records `(record_index, a, x)`; and optional sponsor change through a separate fee envelope. Let `U_in = Σ value(live receipt inputs)`, `F(T) = value(the fresh ASH output)`, and `C = Σ value(live receipt change)`. The operation requires:

$$U_in = F(T) + C$$

Every spendable `U` output is either the one ASH or recognized live change. Time-locked change, a bare-key `U` output, a distribution object, or any other `U` destination is forbidden.

> **Burn is lateral; destruction happens at clear · `rem:operations:burn-lateral`**
>
> The burn operation's canonical `U` delta is **lateral**: live receipt value moves into ownerless ASH while recorded `Y` is unchanged. Actual `U` destruction occurs later at clear under `tag-recon`, when `Y_L` decreases by the same amount. The economic burn is therefore the authenticated burn→clear lineage; the ash term of 𝗜₈ carries the lag exactly (`inv:invariant:accounting`).

> **Signer commitment · `rem:operations:burn-sighash`**
>
> The model proves that every consumed receipt owner belongs to the signer set and that the complete modeled output set satisfies burn closure. The emitted signature MUST commit the full output set `{verify}`: otherwise a third party could append records and force the off-chain `Σx ≤ F(T)` gate to reject the burner's genuine records. Exact sighash semantics are a deployment dependency (`sec:trust:verify`), not a model theorem. `[honesty note]` Because every signature commits the full output set, a multi-owner burn is an interactive multi-party signing session over a final output set — a real client-protocol burden for collaborative burns, not a covenant complication.

> **⚠ Naïve construction.** Forbid receipt change or require one receipt input per ASH so an indexer can tell how much each input destroyed.
>
> **Why it fails (scale and unnecessary rigidity).** The ledger never attributes ASH per input. It needs only the per-transaction total `F(T)`. Forbidding change forces a split-then-burn transaction and serializes shapes without improving the security inequality.
>
> **What we do.** Permit multi-input burns, multiple live change outputs, and arbitrary record partitioning, subject only to `U_in = F(T) + C` and the ledger gate `Σx ≤ F(T)` (`sec:ledger:authentication`). Splitting or merging records at one floor leaves total attestation unchanged (`[A-prop:costliness:splitting-invariance]`). `[enforced: P-burn, P-ledger]` `trap:branches:burn-change`

Burn records are positive and indexed contiguously from zero in transaction-output order. A non-contiguous or duplicate index makes the transition non-canonical. The transaction remains valid when records under-claim or over-claim the ASH; record acceptance is an off-chain verdict distinct from burn provenance (`rem:ledger:overclaim`).

> **Clear-then-burn · `rem:operations:clear-then-burn`** `[design property]`
>
> A burner seeking maximum attestation clears pending ASH first, then burns at the newly committed floor. The sequence is two transactions: the burn may spend a change output of the clear to enforce parent-before-child ordering by consensus. A competing valid STATE operation may invalidate the parent and therefore the child, requiring resubmission; this is liveness contention under R-op (`res:trust:op`), never a safety loss. Either the chain includes the pair in order or it does not include the dependent burn.

### §6.10 Compact ASH · `sec:operations:compact`

`compact-ash` is permissionless and root-free. It consumes at least two and at most `ASH_BATCH_MAX` positive ASH objects, plus an optional sponsor envelope, and emits exactly one positive ASH whose value is their sum:

$$ASH_out = Σ ASH_i$$

`ASH_BATCH_MAX` is deployment-calibrated. The `U` delta is ownerless-lateral; no `U` is destroyed, no supply counter changes, and no burn projection or burn record is emitted. Compaction is therefore **attestation-silent**. The event-type anchor (`trap:ledger:event-anchor`) is what prevents a conforming indexer from treating compacted ASH as fresh destruction.

### §6.11 Clear ASH · `sec:operations:clear`

`clear` is permissionless and consumes STATE plus a non-empty bounded ASH batch. It does **not** consume RESV: `Ω` and `Q` remain unchanged, so the standing backing identity is preserved by non-modification. `ASH_BATCH_MAX` is deployment-calibrated.

Let `B = Σ value(ASH_i)` and `Y = Y_L + Y_T`. The clear amount is the clamp:

$$X = min(B, Y_L, Y−1)$$

A conforming clear requires `X > 0`, then updates `Y_L' = Y_L − X`, `Y_T' = Y_T`, `Ω' = Ω`, `Q' = Q`. It destroys exactly `X` units of `U` under `tag-recon`. If `R = B − X > 0`, it re-emits exactly one residual ASH of value `R`; otherwise it emits none.

The clamp is a safety property, not an arithmetic accident: `X ≤ Y_L` prevents a live-supply underflow; `X ≤ Y−1` makes `Y = 0` unreachable from clear **by construction**; `X ≤ B` makes destruction no greater than the presented ASH; and any oversized ASH partially clears and leaves a cleanable residual — no ASH is stranded by size alone. A zero-progress clear (`X = 0`) is rejected atomically.

The floor strictly rises whenever `X > 0`:

$$Ω'·Y = Ω·Y > Ω·(Y−X) = Ω'·Y'$$

This is the realized burn increase (`[A-prop:invariants:burn-increases-floor]`).

> **⚠ Naïve construction.** Every STATE operation must co-spend RESV, so clear should consume the welded pair.
>
> **Why it fails (unnecessary serialization).** Clear changes only recorded live supply. It touches neither `Ω` nor `Q`; forcing RESV through the transition adds collateral contention without strengthening the backing equation.
>
> **What we do.** Clear has a STATE succession edge and no RESV edge by manifest policy. It MUST preserve `Ω`, `Q`, `Y_T`, `k`, and maturity exactly, derive its decrement solely from the consumed ASH values, and leave the active RESV cursor and value untouched. The certificate witnesses the STATE transition; no unanchored read exists. `[enforced: P-clear, P-cert]` `[invariant: 𝗜₅]` `trap:branches:clear-weld`

> **⚠ Naïve construction.** Read each ASH's amount from committed metadata, or accept a declared aggregate burn amount from the clearer.
>
> **Why it fails (G2 — φ-inflation).** Clear lowers `Y` and raises `φ`. A forgeable decrement lets a clearer claim more destruction than the transaction actually presents, inflating every redemption claim against unchanged backing.
>
> **What we do.** Each ASH amount is its actual consensus `U` value; the clear amount derives from the sum of **all presented ASH inputs**, and every consumed `U` input must be ASH. Exact-value availability and emitted introspection are `{verify}` under P-explicit; the state relation is model-proven. `[enforced: P-clear, P-explicit]` `[invariant: 𝗜₈]` `trap:branches:clear-committed`

> **Who pays · `rem:operations:clear-payer`** `[accepted residual]`
>
> Clear captures nothing. It is funded when an agent internalizes enough of the floor rise to justify the fee: a redeemer improves their own payout; an attestation-seeking burner improves their own next `δ`. For backlog `B` against recorded `Y`, the first-order private gain to stake `V` is approximately `V·B/Y`; clearing is privately worthwhile when that exceeds the fee. The resulting convergence is demand-driven R-conv (`res:trust:conv`), not a safety precondition.

### §6.12 Announce maturity · `sec:operations:announce`

`announce-maturity` is operator-authorized, consumes and recreates STATE, leaves RESV untouched, and is valid only while maturity is unannounced.

Let the committed predecessor cycle be `k`. The announced maturity cycle `k_m` MUST satisfy both specified lead bounds:

$$k + Δk_min ≤ k_m ≤ k + Δk_max$$

The successor changes only `Mat: Unannounced → Announced(k_m)`; `Ω`, `Y_L`, `Y_T`, `Q`, and `k` remain unchanged. The announcement is set-once and irrevocable; a second announcement is invalid.

The operation carries **no maturity timelock**. Both lead bounds are checked directly against the cycle counter committed by the consumed STATE. Conversion occurs automatically inside the cycle that reaches `k_m` (`sec:operations:cycle`). The distinct clocks and the absence of any height↔cycle conversion are (`trap:architecture:two-clocks`).

Announcement is also unavailable on a sealed pool: a sealed STATE has `Y = 0`, and the operation requires a live pool, so an attempt to announce after sealing is rejected. At the manifest level this is the sealed-terminal tombstone ((`inv:invariant:no-trap`), (`inv:invariant:succession`)) making no further STATE-advancing pool transition valid; at the model level the harness, offered a sealed predecessor, rejects with a named `Sealed` reason. Both facts hold, and neither is the other: unrepresentability is the manifest's, the named rejection is the harness's.

> **Both lead bounds are native · `rem:operations:lead-bounds`**
>
> The maturity schedule is protocol arithmetic, not wall-clock arithmetic. Checking the lower and upper leads against the committed `k` realizes (`[A-postc:maturity:announcement]`) directly. The bounds constrain cycle distance after announcement; they do not guarantee wall-clock progress or compel the operator to announce — exactly the specified scope (`[A-rem:maturity:announcement-timing]`).

---

## §7 Authorization · `sec:realization:authorization`

Authorization has two distinct levels:

1. **input authorization** — why each consumed object may participate;
2. **operation authorization** — who may select the semantic operation.

Conflating them is a category error. A root input may carry no independent signature while the operation remains operator-authorized; a sponsor input may require its owner while the operation remains permissionless. The manifest exports the two evidence classes separately, and this section renders both.

*Input-authorization evidence · `tab:manifest:input-authorization-evidence`*

| Input mode | Model evidence | Compiler evidence | Deployment evidence |
|---|---|---|---|
| covenant-companion | branch shape | covenant predicate `{verify}` | emitted-script semantics `{verify}` |
| input-owner | signer set | signature predicate `{verify}` | sighash semantics `{verify}` |
| refund-key | signer set | signature predicate `{verify}` | sighash semantics `{verify}` |
| sponsor-owner | signer set | signature predicate `{verify}` | sighash semantics `{verify}` |
| permissionless | none | none | none |

*Operation-authorization evidence · `tab:manifest:operation-authorization-evidence`*

| Class | Operations | Model evidence | Compiler evidence | Deployment evidence |
|---|---|---|---|---|
| client-authorized | create-request | signer set | signature predicate `{verify}` | sighash semantics `{verify}` |
| refund-key | cancel-request | signer set | signature predicate `{verify}` | sighash semantics `{verify}` |
| receipt-owners | transfer ×2, redeem, burn | every consumed owner in signer set | per-input signature predicates `{verify}` | full output-commitment semantics `{verify}` |
| operator | announce-maturity | operator in signer set | operator signature predicate `{verify}` | sighash semantics `{verify}` |
| cadence-band | cycle | cadence-band state relation | cadence leaves `{verify}` | relative-timelock semantics `{verify}` |
| permissionless | admit, settle, relabel, compact, clear | none required | none required | none required |

Both tables are transcriptions of the manifest's generated evidence arrays; on any cell the appendix wins (`rem:overview:register-authority`). The covenant-companion row's evidence triple — branch shape, covenant predicate, emitted-script semantics — is exactly *not* a signature, which is the whole point of the next box.

> **⚠ Naïve construction.** Every consumed input either carries its own owner signature or is declared permissionless.
>
> **Why it fails (mis-scoped authorization).** A root input — STATE, RESV, PACE, `ENT_AUTH`, `DIST_AUTH` — has no independent owner whose signature licenses it, yet it is not spendable in isolation. Calling it permissionless erases the branch shape and weld that actually authorize its participation; inventing an owner reintroduces a ransom key.
>
> **What we do.** Every root input is a **covenant companion**: it carries no independent signature, and its participation is authorized by the enclosing operation's root-use policy, co-spend weld, succession predicate, and transaction shape. A companion may name only a root the operation is declared to use; no non-root object may claim companion status. Input participation and operation authorization remain separate claims with separate evidence. `[enforced: P-companion, P-cert]` `[invariant: 𝗜₁₀]` `trap:branches:companion-auth`

The same distinction fixes helper placement: the operator's early-cycle authorization belongs to the cadence-band operation predicate, **not** to every companion input and not to an unconditional cycle-wide gate. Adding an operator signature to the forced path is a G8 failure (`trap:branches:cycle-opkey`).

### §7.1 The cadence band · `sec:authorization:cadence`

Let `a` be the consensus-relative age of the active `PACE` root — blocks since the last accepted cycle. It is not a stored protocol counter.

*The cadence band · `tab:authorization:band`*

| PACE age `a` | Valid caller | Property |
|---|---|---|
| `0 ≤ a < MIN` | nobody | anti-rush floor |
| `MIN ≤ a < MAX` | operator | pacing window |
| `a ≥ MAX` | anyone | anti-stall ceiling, G8 |

The cycle always consumes and recreates `PACE`, resetting the age. `MIN` and `MAX` are deployment-calibrated cadence constants and MUST satisfy `MIN < MAX`.

> **The cadence leaves · `leaf:authorization:cadence-band`** `{verify}`
>
> The emitted realization of the band is a shape obligation on script, not a signature class. A conforming covenant emits leaves whose spending conditions are: below `MIN`, unsatisfiable; in `[MIN, MAX)`, an operator-signature leaf gated by a relative timelock at `MIN`; at and above `MAX`, a permissionless leaf gated by a relative timelock at `MAX`. The three-regime *relation* is the model evidence; the emitted early/operator and delayed/permissionless leaves are compiler evidence `{verify}`; relative-timelock semantics and the transaction-version preconditions they require are deployment evidence `{verify}` (`rem:authorization:csv`). The leaf layout is one realization; any emission preserving the three-regime relation and passing the differential vectors conforms.

> **Authorization evidence boundary · `rem:authorization:evidence`**
>
> A model signer set means the named key authorizes the complete modeled transaction output set. It does not prove signature bytes, opcode placement, or sighash flags. Signer-backed authorization therefore has three layers: model signer membership; emitted signature predicate `{verify}`; deployment sighash semantics `{verify}`. A green model test never upgrades the latter two. Cadence-band authorization is not an ordinary signature class — its three layers are the band relation, the emitted leaves, and relative-timelock semantics (`leaf:authorization:cadence-band`). Permissionless authorization claims no secret at any layer — its safety comes from L-flow's no-capture postconditions (`lem:invariant:flow`), not from a missing check.

> **CSV dependency is fail-closed · `rem:authorization:csv`** `[liveness, not safety]`
>
> The emitted cadence predicate depends on consensus relative-timelock semantics `{verify}`. A transaction that cannot satisfy the substrate's version and sequence preconditions cannot spend the cadence path at all; it does not bypass the floor or ceiling. Misconfiguration produces a non-confirming cycle attempt — a liveness failure, never a safety transition. This is R-CSV (`res:trust:csv-version`).

A conforming emission contains **no signature predicate on a permissionless operation path**. P-ransom (`pin:pins:ransom`) applies to admission, permissionless settlement, relabel, compaction, clear, and the delayed cycle path. The operation's fixed recipients and exact partitions are what make that absence safe.

---

## §8 The global invariant 𝗜 — the conformance contract · `sec:realization:invariant`

Correctness is one predicate over the current recognized state `Γ` and its canonical transition history `H`: `𝗜(Γ, H)`. The invariant is the **goal**, not an implementation. A conforming realization MAY organize UTXOs, scripts, proofs, indexes, and local caches differently from any reference model, but it MUST establish the same state clauses at genesis and after every accepted transition, and MUST satisfy the transition lemmas for every step.

The invariant is maintained **inductively**: genesis establishes every clause (`sec:invariant:genesis`); every accepted transition re-establishes every clause; every rejected transition leaves `(Γ, H)` unchanged (`lem:invariant:atomic`). The invariant is never computed by covenant script. Its accounting and history checks fold over unbounded sets; those folds belong to an auditor and to the oracle (`sec:realization:oracle`), not to an on-chain branch.

> **⚠ Naïve construction.** Have every operation recompute the whole invariant — scan all receipts, entitlements, controls, vaults, ASH, residues, and history — and reject if the global identities fail.
>
> **Why it fails (the unbounded-fold trap).** A transaction cannot inspect an unbounded UTXO set or replay an unbounded history. Attempting to do so either makes the covenant unrealizable or quietly replaces the intended global predicate with an incomplete local scan.
>
> **What we do.** Maintain the clauses **inductively** through exact local transitions, recognized objects, running state, and kernel-derived certificates. The proof-only invariant checker folds the whole recognized state and history; each operation proves only its local semantic delta plus the generic kernel lemmas. `[enforced: P-account, P-cert]` `[invariant: 𝗜₇]` `[invariant: 𝗜₈]` `trap:invariant:compute-onchain`

> **Failure reasons are finer than clauses · `rem:invariant:reasons`**
>
> The eleven clauses name stable safety properties; the model's failure reasons identify the narrower check that failed. Several reasons may map to one clause — malformed closed-object shape and duplicate singleton authority both violate 𝗜₁, while state-edge, reserve-edge, and history-set failures all violate 𝗜₁₀. The exported clause registry fixes this many-to-one correspondence. O3 binds seeded corruptions to their **named reasons** (`obl:oracle:reductions`); this document binds those reasons to the clauses below. One clause, 𝗜₉, has a reason declared but never produced — the divergence it would catch is unrepresentable in a one-value-per-object model, so its discharge is structural. The guard vocabulary carries the same pattern once: `RootMultiplicity` is declared for the emitted-script vocabulary but never produced at model level, because duplicate root outputs are unrepresentable in the model's map-keyed world.

*The proof-only invariant projection · `listing:invariant:check` — goal-level pseudocode; never compiled*

```text
check_invariant(Γ, H):
    require exact singleton roots and canonical closed-object closure       // 𝗜₁
    require scalar domains and active-backing cap                           // 𝗜₂
    require Y ≤ Ω                                                           // 𝗜₃
    require Y = 0 ⇒ Ω = Q = 0                                               // 𝗜₄
    require operational RESV value = Ω + Q; sealed RESV absent              // 𝗜₅

    fold recognized current objects and history-derived residue:
        require distribution-control ↔ vault payability                     // 𝗜₆
        require entitlement ↔ Q/control lifecycle                            // 𝗜₇
        require pre-/post-maturity receipt-accounting identities             // 𝗜₈

    require consensus value authoritative at every value-bearing seam        // 𝗜₉
    replay every root edge from genesis and compare final cursors            // 𝗜₁₀
    require maturity state coherent with k, k_m, Y_T                          // 𝗜₁₁
```

### §8.1 State clauses · `sec:invariant:clauses`

*The invariant-clause registry · `tab:invariant:clauses`*

| Glyph | Clause | Secures |
|---|---|---|
| 𝗜₁ | identity, authority, and canonical closure | G5, G9 |
| 𝗜₂ | domains and active-backing cap | arithmetic safety |
| 𝗜₃ | redemption-rate floor | G1 |
| 𝗜₄ | sealed terminal / no trap | G10 |
| 𝗜₅ | operational backing weld | G2, G9 |
| 𝗜₆ | distribution payability | G6, G10 |
| 𝗜₇ | entitlement lifecycle | running-counter correctness |
| 𝗜₈ | receipt accounting | G5, G9 |
| 𝗜₉ | consensus-value authority | G2, G9 |
| 𝗜₁₀ | root succession and history integrity | G5, G9 |
| 𝗜₁₁ | maturity coherence | G6, G8 |

This table is a rendering of the manifest's exported `clauses` array; the eleven labels below are welded verbatim (`rem:manifest:weld`).

> **𝗜₁ — Identity, authority, and canonical closure · `inv:invariant:identity`** *(secures G5, G9)*
>
> A conforming realization MUST maintain: exactly one current `STATE` root carrying the genesis-unique amount-one `PID`; exactly one current amount-one `PACE`, `ENT_AUTH`, and `DIST_AUTH` root; each singleton root at the cursor obtained by replaying its succession history; at most one active `RESV`, at the replayed RESV cursor; and every closed-asset UTXO classified as a recognized canonical object of (`app:realization:architecture`). A closed asset under an undeclared or malformed shape is a violation. An open-asset look-alike is inert rather than a violation until an operation attempts to consume it (`trap:architecture:open-closed`). The clause is covenant ∧ consensus: the model checks exactness and closure; scarcity of the native asset identifiers is a substrate property `{verify}`.

> **𝗜₂ — Domains and the active-backing cap · `inv:invariant:domains`** *(secures arithmetic safety)*
>
> A conforming realization MUST maintain every protocol amount in the `Sat` domain `0 ≤ v < 2⁵¹`; every baked ratio in its declared positive bounded domain; every distribution control with `0 < R_Q ≤ Q_k`, `0 ≤ R_L ≤ D_L`, `0 ≤ R_T ≤ D_T`; and the active-backing cap `Ω + Q ≤ ACTIVE_BACKING_MAX < 2⁵¹`. The cap bounds **simultaneously active** settled reserve plus admitted escrow, not cumulative historical deposits. Redemption may restore headroom; admission above the cap MUST fail atomically and leave requests available (`trap:domains:active-backing`).

> **𝗜₃ — Redemption-rate floor · `inv:invariant:rate-floor`** *(secures G1)*
>
> A conforming realization MUST maintain `Y = Y_L + Y_T ≤ Ω`. Thus the redemption-rate floor satisfies `φ = Ω/Y ≥ 1` whenever `Y > 0`. The ratio is never materialized on-chain; transition preservation is the cross-multiplied relation L-floor (`lem:invariant:rate`).

> **𝗜₄ — Sealed terminal · `inv:invariant:no-trap`** *(secures G10)*
>
> A conforming realization MUST maintain `Y = 0 ⟹ Ω = 0 ∧ Q = 0`. The dead state is reachable only through the exact sealing redemption (`sec:operations:redeem`). Clear cannot reach it because its decrement is clamped by `Y − 1` (`sec:operations:clear`). A state with zero supply and positive reserve or pending escrow is invalid.

> **𝗜₅ — Operational backing weld · `inv:invariant:backing`** *(secures G2, G9)*
>
> A conforming realization MUST maintain: while `Y > 0`, an active `RESV` exists at the replayed cursor and its actual L-BTC value is exactly `value(RESV) = Ω + Q`; while `Y = 0`, `RESV` is absent and `Ω = Q = 0`. The first form is the operational weld; the second is its sealed terminal. RESV succession or termination is derived from actual transition inputs and outputs (`trap:branches:sealing-terminal`), never inferred from a reserve-shaped scan.

> **𝗜₆ — Distribution payability · `inv:invariant:no-starve`** *(secures G6, G10)*
>
> For every live distribution control, a conforming realization MUST maintain: exactly one matching vault iff `R_L + R_T > 0`; no matching vault iff `R_L + R_T = 0`; the vault's actual `U` value equals `value(vault) = R_L + R_T`; and for each class `c ∈ {L,T}`, the dominance bound `R_c ≥ ⌊R_Q·D_c/Q_k⌋`. The final inequality says the remaining class value dominates the largest aggregate draw the remaining principal can still require. It follows from per-entitlement flooring and floor superadditivity (`sec:operations:settle`), via the floor-difference lemma: `⌊A⌋ − ⌊A−B⌋ ≥ ⌊B⌋` for reals `A ≥ B ≥ 0`, so each settlement subtracts from `R_c` no more than the dominance bound releases. No entitlement can be starved by an earlier settlement or by terminal cleanup.

> **𝗜₇ — Entitlement lifecycle · `inv:invariant:escrow-receipts`** *(secures running-counter correctness)*
>
> Let `k+1` be the next cycle. A conforming realization MUST maintain `Q = Σ value(e)` over entitlements `e` targeting `k+1`; and for every live distribution cycle `j`, `R_Q(j) = Σ value(e)` over entitlements targeting `j`. No live control targets a cycle later than the current committed `k`. Every entitlement either targets the pending next cycle or has exactly one matching live distribution control. Consequently `U_ENT = Q + Σ_j R_Q(j)`. These equalities are the proof-side folds whose on-chain counterparts are the locally maintained `Q` and control counters. Entitlement scarcity itself is supplied by `ENT` and `ENT_AUTH` (`trap:architecture:entitlement-scarcity`).

> **𝗜₈ — Receipt accounting · `inv:invariant:accounting`** *(secures G5, G9; carries the burn→clear lag)*
>
> Before maturity conversion, a conforming realization MUST maintain **two** class identities:
>
> $$Y_L = U^L_circ + U_ash + U^L_dist + H_L$$
> $$Y_T = U^T_circ + U^T_dist + H_T$$
>
> Here `U^c_circ` is the actual `U` value in current receipt objects of class `c`; `U_ash` is the actual `U` value in current ASH; `U^c_dist` is the class remainder committed by live controls, whose physical vault value is tied to the class sum by 𝗜₆; and `H_L, H_T` are history-derived terminal-settlement residues. After maturity conversion, a conforming realization MUST maintain `Y_T = 0` and the single live-side identity:
>
> $$Y_L = U^L_circ + U^T_circ + U_ash + U^L_dist + U^T_dist + H_L + H_T$$
>
> Both historical residue terms migrate into the live-side identity; neither is dropped merely because the time-locked accounting class no longer exists. Physical `RECEIPT_T` and time-locked distribution allocations may persist post-conversion and are likewise counted on the live side until permissionless migration completes. `U_ash` carries the transient burn→clear lag; `H_L + H_T` carries permanently destroyed distribution residue. Both make recorded `Y` over-state current circulating `U`, so `φ` is understated — the safe direction (`trap:ledger:residue-decrement`).

> **𝗜₉ — Consensus-value authority · `inv:invariant:consensus-value`** *(secures G2, G9)*
>
> A conforming realization MUST treat the actual consensus value of every recognized value-bearing object as authoritative. A receipt, entitlement, ASH, vault, control amount, or active RESV MUST NOT carry an independent metadata amount that can diverge from its real asset value. Metadata MAY partition an actual value only where all parts are pinned to real flows whose sum equals that value — request `gross = principal + service budget`, and distribution-vault `value = R_L + R_T`. Metadata MUST NOT impersonate value (`trap:branches:committed-value`). In a model where each UTXO has one value field, committed-versus-consensus divergence is unrepresentable; this clause's model evidence is structural — the reason is declared but never produced — rather than a runtime check. Its on-chain discharge is explicit values plus value and issuance introspection `[enforced: P-explicit]` `{verify}`. The oracle exercises the partition seams through 𝗜₆ and 𝗜₇ corruption, not the unrepresentable identity core.

> **𝗜₁₀ — Root succession and history integrity · `inv:invariant:succession`** *(secures G5, G9)*
>
> A conforming realization MUST maintain a canonical history that replays from genesis: transition order strictly increasing; transaction identifiers unique; each outpoint created at most once and consumed at most once; a certificate's consumed and created sets disjoint; no outpoint created after it was previously consumed; every root edge checked against the root active immediately before that transition; every succession output among the transition's created set; every terminating input among its consumed set; replayed root cursors equal to the current tracked cursors. Once RESV terminates, the resulting sealed STATE is a tombstone: no later operation may spend STATE or recreate RESV. Root-free operations that remain meaningful — transferring or compacting already-existing off-pool objects — do not revive the pool. Full replay witnesses the path rather than only the endpoint (`rem:kernel:intermediate-corruption`).

> **𝗜₁₁ — Maturity coherence · `inv:invariant:maturity`** *(secures G6, G8)*
>
> A conforming realization MUST maintain: `Unannounced` carries no maturity cycle; `Announced(k_m)` satisfies `k_m > k`; `Complete` satisfies `Y_T = 0`. The cycle reaching `k_m` performs conversion atomically and moves the latch to `Complete`; no accepted state has `k ≥ k_m` while maturity remains announced. Once complete, maturity never reverses. Physical time-locked receipt objects may remain, but 𝗜₈ counts them on the live side and (`sec:operations:relabel`) migrates them permissionlessly.

### §8.2 Transition lemmas · `sec:invariant:transitions`

The state clauses describe settled states. Five transition lemmas describe every accepted or rejected step.

> **L-floor · `lem:invariant:rate`** ⚓ *(secures G1)*
>
> For predecessor `(Ω, Y)` and successor `(Ω', Y')`, every accepted transition MUST satisfy `Ω'·Y ≥ Ω·Y'`. Whenever both supplies are positive, this is equivalent to `Ω'/Y' ≥ Ω/Y`. The inequality is exact and integer-valued; no on-chain floor ratio is materialized (`sec:architecture:rate-not-materialized`).

> **L-atomic · `lem:invariant:atomic`** *(structural)*
>
> A rejected transition leaves `(Γ, H)` bit-for-bit unchanged. An error MUST NOT consume an input, emit an output, advance a cursor, alter history, or mutate cadence state. A conforming model makes this structural by publishing a successor only after all validation succeeds. Therefore preservation needs proof only for accepted transitions.

> **L-delta · `lem:invariant:delta`** ⚓ *(secures G5)*
>
> Every accepted closed-asset change MUST have exactly one witness by kind — issuance, destruction, lateral, or ownerless-lateral — and every consumed or created canonical value object MUST belong to exactly one flow or issuance. The derived active delta-family set MUST equal the manifest's expected set for the operation and its activation conditions (`trap:branches:canonical-delta`).

> **L-flow · `lem:invariant:flow`** ⚓ *(secures G8, G10)*
>
> Every accepted value movement MUST conserve its asset and satisfy one declared authorization class from (`sec:kernel:recipient`). Open L-BTC MUST also belong to exactly one declared flow role. Permissionless operations MUST have immutable or formula-bound recipients, or ownerless sinks/movements; a triggerer MUST NOT capture value.

> **L-record · `lem:invariant:rec`** *(secures G4)*
>
> A raw burn record `(a, x)` carried by an accepted burn transition is append-only in canonical history. No operation deletes or rewrites it. The record's derived attestation value `δ = x·φ_c` is a checkpoint-relative view and MAY change under a reorg that changes the sampled clearing (`sec:ledger:reorg`); record immutability is not valuation immutability.

### §8.3 Genesis establishes 𝗜 · `sec:invariant:genesis`

Genesis is the disclosed trusted setup of one pool. It creates `STATE` — amount-one `PID`, committing `Ω = E_0`, `Q = 0`, `k = 0`, `Mat = Unannounced`; `RESV` — explicit L-BTC of value `E_0`; amount-one `PACE`, `ENT_AUTH`, and `DIST_AUTH`; `U = E_0` split into `Y_L = ⌊ζ·E_0⌋` and `Y_T = E_0 − Y_L` with both parts positive; one live and one time-locked receipt carrying those values; and a genesis history projection that identifies every initial root and seeds the ledger's clearing zero with `(Ω, Y) = (E_0, E_0)` (`rem:ledger:genesis-clear`). The setup requires `0 < E_0 ≤ ACTIVE_BACKING_MAX` and `0 < Y_L < E_0`. No entitlement, distribution, vault, ASH, or historical residue exists.

Then: 𝗜₁ holds by exact genesis issuance and root placement; 𝗜₂ by the bounds above; 𝗜₃ with `Y = Ω = E_0`, so `φ = 1`; 𝗜₄ vacuously because `Y > 0`; 𝗜₅ because `RESV = E_0 = Ω + Q`; 𝗜₆ and 𝗜₇ over empty distribution and entitlement sets; 𝗜₈ because all `U` is current receipt value; 𝗜₉ structurally in the model and by the genesis explicit-value obligation `{verify}`; 𝗜₁₀ from the trusted genesis projection; 𝗜₁₁ because maturity is unannounced.

The live/time-locked split is what makes the bootstrap capacity property SP5 a theorem: only the live portion is burnable before external deposits, while the time-locked portion remains in recorded supply (`[A-prop:interface:bootstrap-capacity]`).

### §8.4 G1–G10 are corollaries · `sec:invariant:corollaries`

- **G1** follows from L-floor and the operation arithmetic of (`sec:realization:operations`).
- **G2** follows from 𝗜₅, the redemption payout law, and the fact that burn moves `U` to ASH while clear lowers `Y_L` by exactly the `U` it destroys.
- **G3** follows from live-only burn, the ownerless ASH lineage, the clear destruction, and the ledger's dual anchor (`sec:ledger:authentication`).
- **G4** follows from L-record plus checkpoint-relative reprojection (`sec:ledger:model`).
- **G5** follows from 𝗜₁, 𝗜₈, L-delta, authority-off-state, and native-asset conservation `{verify}`.
- **G6** follows from the absent time-locked burn/redeem operations, class-closed transfer, atomic logical conversion, 𝗜₈'s post-conversion identity, and permissionless owner-preserving relabel.
- **G7** follows from the forced arithmetic relations at cycle, settlement, and redemption, and from the absence of any operator-selected economic quantity outside preauthorized service budgets.
- **G8** follows from the cadence ceiling and the permissionless admission, settlement, relabel, compaction, and clear paths, each safe under L-flow.
- **G9** follows from canonical recognition, 𝗜₅–𝗜₁₀, full history replay, the quantity-reader matrix, and the ledger recomputation.
- **G10** follows from L-flow, 𝗜₄–𝗜₇, formula-bound payouts, owner-preserving permissionless paths, and the sealed-terminal rule.

### §8.5 Conservative resting states · `sec:invariant:terminals`

Recorded `Y` deliberately includes destroyed-but-not-decremented terms — `U_ash` before clear; `H_L + H_T` permanently. Consequently, valid states may retain reserve for which no spendable receipt exists. These are theorems of 𝗜₈, not invariant exceptions, and always bias `φ` downward.

> **Terminal A — historical-residue locked reserve · `rem:invariant:terminal-a`** `[accepted residual]`
>
> Per-entitlement flooring may leave distribution residue at terminal settlement. The residue `U` is destroyed under `tag-distribution-residue`, enters `H_L, H_T`, and does not decrement `Y`. If all remaining spendable receipts later exit, recorded supply may consist wholly or partly of historical residue. The corresponding reserve cannot be claimed because no receipt carries that portion; it is permanently un-redeemable and uncapturable.

> **Terminal B — full-burn locked reserve · `rem:invariant:terminal-b`** `[accepted residual]`
>
> If essentially all live receipts move into ASH, clear decrements supply only to the `Y − 1` floor and re-emits the residual ASH. In the exhausted form `Y_L = U_ash + H_L + H_T ≥ 1`, while no spendable live receipt remains. RESV may remain positive, but the residual recorded supply is ownerless ASH and/or historical residue, so nobody can present a redemption claim against it. The reserve is locked, never capturable.

Both terminals understate `φ` because the denominator includes non-circulating terms. This is the permanent face of the same conservatism as the lazy φ-rise (`res:trust:locked-residue`) and lies within SP6's upper-bound form (`[A-prop:interface:conservative-valuation]`).

---

## §9 Recognition and identity · `sec:realization:identity`

Recognition is the projection from the global UTXO environment into protocol state. It must distinguish three categories without ambiguity:

1. **canonical protocol objects** — closed-asset objects whose asset, value domain, metadata, and lineage satisfy the manifest;
2. **the active RESV** — the one open L-BTC root selected by replayed provenance;
3. **inert external objects** — all other open-asset UTXOs.

A fourth verdict, **canonical violation**, exists only for a closed asset under a malformed or undeclared object shape. This is the executable form of (`trap:architecture:open-closed`): open junk is inert, while malformed closed state is an invariant failure.

*The recognition projection · `listing:identity:recognize` — model-normative; emitted introspection is `{verify}`*

```text
classify(outpoint, utxo, replayed_roots):

    if outpoint == active_RESV_cursor:
        require asset = L-BTC and object = RESV
        return ActiveResv

    if asset is open:
        return InertExternal

    match closed asset and declared object shape:
        PID       + STATE(amount 1)                  → Canonical(State)
        PACE      + PACE(amount 1)                   → Canonical(Pace)
        ENT_AUTH  + ENTITLEMENT_AUTHORITY(amount 1)  → Canonical(EntitlementAuthority)
        DIST_AUTH + DISTRIBUTION_AUTHORITY(amount 1) → Canonical(DistributionAuthority)

        U + RECEIPT_L(owner, positive value)          → Canonical(LiveReceipt)
        U + RECEIPT_T(owner, positive value)          → Canonical(TimeLockedReceipt)
        U + DISTRIBUTION_VAULT(cycle, positive value) → Canonical(Vault)
        U + ASH(positive value)                       → Canonical(Ash)

        ENT      + DEPOSIT_ENTITLEMENT(owner,target,positive value)
                                                     → Canonical(Entitlement)
        DIST_CTL + DISTRIBUTION_CONTROL(amount 1, valid counters)
                                                     → Canonical(Control)

        otherwise                                    → CanonicalViolation
```

The classifier does not locate the active reserve by scanning for `RESV` shape. It first obtains the RESV cursor from trusted genesis plus full root-history replay (`inv:invariant:succession`); only the object at that cursor may be active backing. Every other RESV-shaped L-BTC output is inert external state.

> **The recognition lesson · `intuit:identity:lesson`**
>
> The same rule applies at every seam: recognize the **unforgeable thing**, never an attacker-choosable field (`trap:architecture:recognize-unforgeable`). The pool is the replayed `PID` root, not any STATE-shaped script; backing is the L-BTC object at the replayed RESV cursor, not any RESV-shaped output; an entitlement is scarce `ENT`, not committed `(owner, amount, target)` fields alone; an ASH amount is its actual `U` value, not metadata; a burn record is attestation-bearing only through a burn-typed certificate, not through a tag alone; and an owner field routes or authorizes only where the manifest explicitly says which — carrying an owner is not itself an authorization rule. The model's typed classifier establishes these distinctions structurally. Their emitted realization depends on asset/value/issuance introspection and covenant welds `{verify}` under P-ident, P-explicit, and P-weld.

### §9.1 Owner, recipient, and attestation address are distinct · `sec:identity:three-keys`

Three 32-byte concepts MUST remain distinct: an **owner key** authorizes consumption of an owner-bearing receipt, request refund, sponsor input, or operator path; an **entitlement owner** is a routing destination for settlement outputs and does **not** gate entitlement consumption; an **attestation address** `a ∈ 𝔸` receives a monotone record and carries no spend authority at this layer.

The deposit entitlement therefore derives no external-spend authorization path: its sole consumption path is permissionless settlement, whose outputs preserve the committed owner (`trap:branches:settlement-sig`). Conversely, the request's refund key, not its receipt owner, authorizes cancellation. "Owner-bearing" in the manifest's authorization-legality rule (`sec:manifest:wellformed`) means *consent gates consumption*, which is true of receipts, requests, and plain reserve, and false of the entitlement despite its owner field.

### §9.2 Structural class asymmetry · `sec:identity:structural-nonredeem`

> **Time-locked non-redeemability is structural · `rem:identity:structural-nonredeem`**
>
> The time-locked receipt object participates only in owner-authorized time-locked transfer and permissionless owner-preserving receipt relabel after maturity. It is absent from the burn and redemption input families. No generic transfer operation accepts both classes, and no time-locked circulation path admits ASH or live-receipt outputs.
>
> Two facts hold here, and neither is the other. At the **manifest and emitted level**, "a time-locked receipt cannot burn or redeem" is unrepresentable: the operation the manifest cannot express for that object simply does not exist, so there is no branch to reach. At the **model-harness level**, a constructor offered a time-locked outpoint where a live receipt is required necessarily rejects with a *named* reason — `ClassCross` — because the harness accepts arbitrary outpoints and must classify the mismatch. The first is the strongest realization of G6 (`req:requirements:two-classes`) and the specification's structural class clause (`[A-def:model:classes]`); the second is what an O3 auditor observes when it probes the seam. Unrepresentability is the manifest's; the named rejection is the harness's.

### §9.3 STATE reads are succession, never reference · `sec:identity:state-succession`

No read-only STATE primitive exists. An operation that depends on committed pool state consumes the current STATE root and recreates its successor, even when the semantic state is byte-identical (receipt relabel). The kernel derives the `StateSuccession` edge from those actual objects (`sec:kernel:certificate`); full replay authenticates the path (`sec:kernel:replay`).

This eliminates an unwitnessed category: a branch cannot read maturity, supply, escrow, or reserve counters from a convenient pool-shaped UTXO. It either consumes the replayed `PID` root under the declared succession rule or it has no authenticated STATE input and cannot make a state-dependent claim.

---

## §10 Representation conformance and opacity · `sec:realization:representation`

The manifest is the canonical finite presentation of a **semantic theory**: an object's value is a semantic fact, not a mandated encoding. A backend may discharge a relation by explicit introspection, authenticated opening, commitment equality, or consensus Confidential-Transaction conservation — provided every accepted transaction refines the same abstract transition (`rem:oracle:thesis`).

> **Value-parametric, asset-rigid.** Parametricity holds on the value axis only. Closed-asset *identity* remains explicit at every protocol seam: a confidential asset commitment could carry `U`, and the closed-asset induction ((`trap:architecture:open-closed`), G5) fails at the first blinded escape. `[enforced: P-asset-boundary]`

> **Three value modes.** `PrivateCommitted` (no public opening) ≺ `PublicCommitted` (commitment-encoded but amount and opening published) ≺ `Explicit` (8-byte LE). This is a leakage order, not a value order — all three denote the same semantic amount. A declassification that fully consumes a confidential input must route the residual blinding factor into a **public-committed** output; `Explicit` is insufficient wherever an input is wholly spent. `[enforced: P-declassify]`

> **The disclosure frontier.** A value becomes public iff $D_{\text{state}} \lor D_{\text{live}} \lor D_{\text{audit}}$: it changes a public STATE field; or a **permissionless** actor must *construct* the transition from public data plus own sponsor funds ($D_{\text{live}}$ is availability, not verifiability); or it is itself a public event or interface quantity. Everything else — lateral owner-to-owner movement — may stay `PrivateCommitted`. `[enforced: P-opacity]`

> **Two axes, not one.** *Safety* = no illicit acceptance, witnessed by the reject/closure vectors (`trap:architecture:open-closed`). *Minimality* = no unnecessary demand, witnessed by accept-under-blinding vectors. A green blinded-accept is a **minimality** witness and MUST NOT be read as safety. `trap:representation:two-axes`

> **Declassification is model-derived.** Each operation's declassification function $\mathcal D_o$ is the set of semantic facts its model transition actually reads — extracted from the transition read-sets, never authored (`declassification.json`, (`sec:pins:codegen`)). The metamorphic oracle's per-cell verdict is a function of $\mathcal D_o$; an authored $\mathcal D_o$ would certify the wrong thing.

Worked membrane: transfer reveals nothing ($\mathcal D=\varnothing$); relabel reveals nothing (commitment equality, stronger than decode-and-compare); burn reveals the fresh-ASH aggregate $F(T)$, not source denominations; redeem reveals $(x,p)$; deposit reveals principal $\delta$. ASH is always publicly openable — it feeds STATE and is permissionlessly swept.

> **The inspection-necessity rule · `rule:representation:inspection-necessity`**
>
> A conforming realization inspects, opens, decodes, or compares a value only when that exact value is required by a named semantic or security relation. Implementation convenience, uniform object handling, diagnostic usefulness, canonical wallet policy, and target-policy preference do not create a protocol read. For every exact value inspection, the realization MUST identify: (1) the named invariant, transition relation, authorization relation, public observable, or constructibility requirement that consumes the value; (2) the security failure possible if that relation is not established; (3) the authentic source from which the value is obtained; (4) the disclosure reason introduced by making the value public; (5) a focused negative vector demonstrating the omitted check. If a relation can instead be discharged by commitment equality, Confidential-Transaction conservation, a range proof, a zero-knowledge relation, or another approved opaque proof, that proof does not authorize disclosure of the exact value. A fact absent from the minimal semantic read-set is opaque to protocol predicates; a backend MUST NOT add a read merely because its target exposes an introspection primitive.

> **Inspection burden · `rule:representation:inspection-burden`**
>
> A proposed inspection of fact $x$ is security-justified only if there exist two otherwise equivalent candidate transitions $T_{\mathrm{good}}$ and $T_{\mathrm{bad}}$ such that every already-authenticated fact is equal, the protocol projections differ only through $x$, $T_{\mathrm{bad}}$ violates a named security relation, and inspecting or proving the required property of $x$ distinguishes the invalid transition. If no such focused counterexample exists, exact inspection of $x$ is not part of the realization relation. Worked boundary: RESV value — a bad transition can commit $\Omega+Q$ in STATE while carrying less real reserve, so exact authentication is required; ASH value — a bad transition can claim destruction larger than actual ASH, fraudulently lowering $Y$; redemption payout — a bad transition can shorten the owner's formula-bound payout while aggregate value still balances; sponsor change — changing its amount moves only sponsor-local residual value and fee once protocol flows are independently pinned ((`sec:kernel:sponsor-opacity`)), so no counterexample exists and no inspection is licensed.

> **Sponsor erasure · `def:representation:sponsor-erasure`**
>
> Let $\pi_P(T)$ be the protocol projection of target transaction $T$. It retains: protocol objects and roots; protocol assets and authenticated protocol amounts; protocol authorization; protocol state assignments; canonical issuance and destruction; protocol recipients; public records and event projections; and sponsor-region existence, membership, authorization, and balance verdict. It erases: individual sponsor amounts; sponsor openings and blinding factors; sponsor-local denominations; sponsor-local change allocation; and any zero-valued sponsor member whose presence has no protocol effect. Two target transactions are protocol-equivalent when their protocol projections are equal; the denotation of (`sec:realization:versioning`) is taken over $\pi_P(T)$, not over sponsor denominations. A protocol predicate, compiler-selected proof, emitted target program, canonical public report, or release identity MUST NOT require an individual sponsor amount to be explicitly encoded for protocol use, decoded, opened, compared with zero, proved strictly positive, aggregated as a public integer, or emitted in a public diagnostic or evidence field — on either side of the transaction. The sponsor constructor MAY use its own private amount, opening, blinding factor, or wallet state to construct a valid transaction; those remain sponsor-local capabilities, never protocol facts.

> **Anchor identity is structural · `rule:representation:anchor-identity`**
>
> `CPFP_ANCHOR` is recognized by its declared constructor, object family, operation condition, and ABI role — not by testing whether an ordinary L-BTC output has value zero. A zero-valued `PLAIN_LBTC` output remains an ordinary sponsor output; it cannot satisfy an anchor family, anchor constructor, anchor slot, or anchor-specific deployment claim. If value zero were needed to distinguish the anchor from sponsor change, object recognition would be under-specified, and the fix is stronger constructor/family authentication — never sponsor-value inspection. The anchor itself keeps its exact value-zero requirement because "zero-value fee hook" is its declared semantic role; that requirement does not authorize reading unrelated sponsor values.

> **Canonical sponsor construction · `rule:representation:sponsor-canonicality`**
>
> A first-party builder SHOULD omit a known explicit zero-valued sponsor-change output: it carries no value and consumes transaction resources. This is wallet construction policy, not a protocol predicate — a covenant, semantic validator, or compiler proof plan MUST NOT inspect the sponsor value merely to enforce the wallet's preferred canonical form, and a deployment policy MAY separately reject a nonstandard zero output as policy, not semantics. For confidential sponsor values, output presence may reveal whether the private residual is zero; an ABI or builder claiming disclosure minimality MUST account for that shape leakage explicitly and MUST NOT require private-value-dependent omission while describing the construction as value-opaque.

---

## §11 Translation discipline — model relation ⇒ tapscript · `sec:realization:translation`

This section is the contract for translating a conforming model into an Elements/Liquid covenant. It constrains **behaviour**, not emitted bytes. An implementation may organize leaves, witnesses, stack values, and helper fragments differently, provided its accepted transaction relation is equivalent to (`sec:realization:kernel`), preserves (`sec:realization:invariant`), and passes (`sec:realization:oracle`).

Every emission claim in this section carries `{verify}`. The model establishes the source relation; emitted-script hashes, differential script vectors, and substrate tests establish that a particular script enforces it (`sec:oracle:boundary`). No translation rule is itself a proof.

> **T1 — Predicate and atomic failure · `rule:translation:predicate`** `{verify}`
>
> Each declared operation compiles to a fail-closed transaction predicate: model acceptance corresponds to successful script evaluation; model rejection corresponds to script failure; failure creates no partial successor. The emitted transaction is atomic by consensus, while the model establishes the same relation structurally through L-atomic (`lem:invariant:atomic`). A translator MUST NOT compile an error-returning model path into a branch that accepts with a partially updated output set.

> **T2 — Amount domain · `rule:translation:sat`** `{verify}`
>
> Every protocol amount realizes the `Sat` domain `0 ≤ v < 2⁵¹`. Every externally witnessed, issued, introspected, summed, or subtracted amount MUST re-enter through the domain check at the point where the model constructs a `Sat`. Arithmetic failure is fail-closed. The emitted relation MUST also enforce the active-backing cap independently of the arithmetic domain (`trap:domains:active-backing`).

> **T3 — Witnessed and verified division · `rule:translation:division`** `{verify}`
>
> Every model quotient is supplied as a witness and pinned by `q·D ≤ N < (q+1)·D`, `q < 2⁵¹`, `D > 0`. This applies to `floor_mul_div` and `floor_ratio` call sites. The emitted relation MUST implement the same exact floor as the model; a lower-bound-only check is non-conforming (`trap:translation:lower-bound`). Internal limb, carry, and borrow correctness belongs to the gadget `{verify}` surface (`rem:arithmetic:hand-audit`).

> **T4 — Payload enums · `rule:translation:enum-payload`** `{verify}`
>
> A model enum carrying data compiles to a tagged or sentinel-disjoint committed field. In particular, maturity has three states: `Unannounced`, `Announced(k_m)`, `Complete`. The encoding MUST make the two sentinels disjoint from every admissible cycle index `k_m`, and branch selection MUST preserve the model's state-transition relation. The concrete byte encoding is implementation-specific; sentinel collision is forbidden.

> **T5 — Operation selection is structural · `rule:translation:leaf-selection`** `{verify}`
>
> The operation enum identifies distinct accepted transaction relations. An emitted covenant SHOULD realize those relations as distinct tapleaves rather than as attacker-selected in-script dispatch. Regardless of physical arrangement, it MUST preserve the structural absences declared by the manifest: time-locked receipts have no burn or redemption path; ASH has no owner-recovery path; roots participate only in operations whose root-use declarations permit them; an operation cannot smuggle inputs from a different semantic family through a generic branch. Separate script bytes are not the property; mutually exclusive accepted relations are.

> **T6 — Kernel checks are part of every affected operation · `rule:translation:kernel-inline`** `{verify}`
>
> The generic kernel of (`sec:realization:kernel`) has no on-chain runtime and need not compile to a reusable script subroutine. Its obligations MUST nevertheless appear in every emitted operation relation they govern: exact input/output shape; root-use and succession; canonical and open-flow partitions; issuance authority and amount; event-projection shape; branch semantic postcondition. A translator MUST NOT emit only the operation-specific arithmetic while treating the generic kernel as off-chain validation.

> **T7 — Committed state · `rule:translation:struct`** `{verify}`
>
> The pool state `S = (Ω, Y_L, Y_T, Q, k, Mat)` is encoded under the unique STATE identity in a fixed, unambiguous commitment. Reading one field authenticates the whole predecessor state; recreating STATE commits the complete successor state. The encoding MUST reject omitted fields, alternate encodings of one logical state, field-order ambiguity, and a successor that copies an unauthenticated predecessor value. The exact byte layout is not normative; the one-to-one state commitment is.

> **T8 — Collections are UTXO sets · `rule:translation:collections`** `{verify}`
>
> Model vectors and maps denote sets of independently spendable UTXOs, not serialized containers held in one output: lookup means recognizing a presented input; insertion means emitting a recognized output; removal means consuming without recreating; mutation means consuming and recreating under the object's declared relation. Cardinality is bounded only where the manifest names a finite calibrated bound. A translator MUST NOT collapse an independently spendable collection into one shared object unless it proves equivalent liveness, authorization, and contention properties.

> **T8a — Fold elimination · `rule:translation:fold-elimination`** `{verify}`
>
> An unbounded fold over the current UTXO set or complete history is a **proof-only** operation and MUST NOT appear in an emitted covenant. Its locally maintained counterpart is an inductive counter or object relation whose equality to the global fold is asserted by (`sec:realization:invariant`). Bounded folds over inputs explicitly presented by one transaction are permitted — summing an admission batch, settlement batch, receipt inputs, ASH inputs, or sponsor inputs within manifest-calibrated cardinalities. The off-chain fold computing `A` is likewise permitted because it is not covenant execution (`sec:realization:ledger`).
>
> > **⚠ Naïve construction.** Compile the invariant checker itself: every transition scans all current receipts, entitlements, distributions, ASH, and history.
> >
> > **Why it fails (unbounded-fold unrealizability).** The branch cannot inspect objects the transaction does not present, and a global accumulator reintroduces the serialization the design removes (`trap:architecture:accumulator`).
> >
> > **What we do.** Only bounded presented-input folds compile. Global equalities remain invariant and oracle obligations, maintained inductively by local transitions. `[enforced: P-account]` `[invariant: 𝗜₇]` `[invariant: 𝗜₈]` `trap:translation:fold`

> **T9 — Identity binding · `rule:translation:bind`** `{verify}`
>
> Every recognized protocol object MUST bind to an unforgeable identity source: a closed native-asset identifier; the replayed predecessor root; a covenant-key commitment derived from declared object metadata; or actual explicit consensus value where value is semantic. The emitted predicate MUST reconstruct and verify the object's identity rather than accepting a bare public script shape or attacker-supplied field (`trap:architecture:recognize-unforgeable`). The active RESV additionally binds to its STATE transition relation; a decoy RESV-shaped L-BTC output remains inert.

> **T10 — Timelocks and the two clocks · `rule:translation:timelocks`** `{verify}`
>
> Exactly one protocol condition is implemented with a consensus timelock: **cycle cadence**, expressed as relative age of `PACE`. Maturity uses no timelock; it is checked against committed cycle indices, `k + Δk_min ≤ k_m ≤ k + Δk_max`. A translator that emits a maturity CLTV condition or a height↔cycle conversion is non-conforming (`trap:architecture:two-clocks`). Cadence's relative-timelock semantics and transaction-version requirements are deployment evidence (`rem:authorization:csv`).

> **T11 — Signatures and their absence · `rule:translation:signatures`** `{verify}`
>
> A model signer requirement compiles to a signature predicate for the named authorization mode. The signature MUST commit the complete economic output set required by the operation; whether an input-set extension mode is permitted is a separate policy and MUST NOT weaken output commitment. The **absence** of a signature on a permissionless operation is equally load-bearing. Admission, settlement, receipt relabel, ASH compaction, clear, and the delayed cycle path MUST contain no hidden operator or owner signature gate. Their safety comes from immutable/formula-bound recipients and exact flow closure, not from a secret (`pin:pins:ransom`).

> **T12 — Cross-UTXO and transaction-shape obligations · `rule:translation:cross-utxo`** `{verify}`
>
> Some requirements are relations among several objects and do not live inside any one operation body: exact root input/output cardinality; STATE↔RESV and authority succession; control↔vault correspondence; entitlement↔control target agreement; burn event-type shape; clear's all-presented-ASH fold; issuance destination exhaustion; sponsor-envelope isolation; owner/value preservation during relabel. These relations MUST be explicit compiler obligations. Auditing only the arithmetic bodies is insufficient.
>
> > **⚠ Naïve construction.** Audit each operation's local function and assume every object it references is the correct companion.
> >
> > **Why it fails (the cross-object substitution attack).** A locally correct state update can consume the wrong root, pair a control with the wrong vault, read maturity from a fabricated state, or route issuance into an unlisted output while every local equation remains true.
> >
> > **What we do.** Compile every cross-object relation as a named transaction-shape obligation, indexed by the corresponding manifest relation and pin. `[enforced: P-companion, P-cert, P-flow]` `[invariant: 𝗜₁₀]` `trap:translation:bodies-sufficient`

> **T13 — Certificate relation ⇒ emitted predicate · `rule:translation:certificate-leaf`** `{verify}`
>
> Each relation represented in the model's transition certificate MUST have a corresponding emitted enforcement obligation: each root edge; each canonical delta; each open-flow balance; each event projection; each consumed/created membership fact needed by the branch postcondition. The script need not materialize or serialize a certificate. It MUST enforce a relation from which an independent verifier derives the same certificate. Differential script vectors are organized **relation by relation**, not only operation by operation, so a missing edge or flow check cannot hide inside a broadly successful branch test (`pin:pins:weld`).

*Translation master table · `tab:translation:master`*

| Model construct | Emitted obligation | Rule |
|---|---|---|
| total operation returning acceptance/rejection | fail-closed transaction predicate | T1 |
| `Sat` construction and checked arithmetic | `< 2⁵¹` domain and failure checks | T2 |
| floored quotient | witnessed multiply-back sandwich | T3 |
| payload enum | unambiguous tagged/sentinel encoding | T4 |
| operation family | mutually exclusive accepted relation | T5 |
| generic kernel | relation inlined into every affected operation | T6 |
| committed state struct | one-to-one predecessor/successor commitment | T7 |
| vector/map | recognized UTXO set with bounded presented inputs | T8 |
| unbounded fold | inductive counter + invariant; never on-chain | T8a |
| object/root identity | asset/provenance/key/value binding | T9 |
| cadence/maturity clocks | relative timelock / committed-cycle arithmetic | T10 |
| signer set or permissionless absence | output-committing signature / no secret gate | T11 |
| multi-object relation | explicit transaction-shape obligation | T12 |
| transition certificate | equivalent per-relation enforcement | T13 |

---

## §12 The off-chain attestation ledger and reader matrix · `sec:realization:ledger`

The ledger is not covenant script. It is the independent, publicly reproducible computation of the one object the specification exports: `A : 𝔸 → ℝ≥0`. A conforming indexer reads the canonical public history, authenticates burn provenance, groups records against clearing states, and emits an exact, checkpoint-bound result. A non-conforming indexer can mis-attest; conformance is established by O6 (`obl:oracle:ledger`), not by trusting an index operator.

> **Audit cost · `rem:ledger:cost`**
>
> For one address over a pre-indexed canonical event stream, a conforming indexer performs one ordered traversal of burn and clear events and their records: `O(burns + clearings + records)`, excluding big-integer arithmetic and output allocation. All-address materialization may add associative-map or sorting cost and never defines the single-address query claim. Recognition of each candidate burn is provenance verification, not a grep over tagged payloads. Implementations MAY use indexed lookup with equivalent asymptotic behaviour; a per-burn scan of all prior clearings is not the reference cost claim.

### §12.1 The dual-anchor gate · `sec:ledger:authentication` ⚓

For a burn-typed transaction `T`, define `F(T) = value(the fresh ASH output of T)`. Let its canonically ordered records be `R(T) = {(i, a_i, x_i)}` for `i = 0 … n−1`, where indices are contiguous from zero, each `x_i > 0`, and `a_i ∈ 𝔸`. The record set is **accepted** iff:

$$Σ x_i ≤ F(T)$$

If the inequality fails, **all** records of `T` contribute zero. The underlying burn event remains part of the raw history.

> **⚠ Naïve construction.** Treat any transaction creating ASH as a burn and credit its tagged records up to the ASH amount.
>
> **Why it fails (G3 — the false-burn attestation).** ASH is also created by compaction and by clear's residual re-emission. Neither represents fresh receipt forfeiture. An amount anchor alone re-attests previously accounted ASH whenever it moves.
>
> **What we do.** Require two independent anchors: an **event-type anchor** — the kernel derives a burn projection only from the burn operation, with live receipt inputs, no ASH input, exactly one fresh ASH output, and only live receipt change; and a **value anchor** — accepted records satisfy `Σx ≤ F(T)`. The event anchor proves *what happened*; the value anchor proves *how much*. Neither substitutes for the other. `[enforced: P-burn, P-ledger]` (`req:requirements:costliness`) `trap:ledger:event-anchor`

> **⚠ Naïve construction.** Scan for `tag-burn` payloads and compute `A(a)` by summing every `x` naming `a`.
>
> **Why it fails (G3 — the free-attestation exploit).** Anyone can emit the bytes `(tag-burn, a, x)` without spending a receipt or creating ASH. A tag is attacker-chosen data, not provenance (`trap:architecture:opreturn-ledger`).
>
> **What we do.** Read records only from kernel-authenticated burn projections and apply the value gate above. A free-standing payload belongs to no accepted burn projection and contributes nothing. `[enforced: P-ledger]` `trap:ledger:scan-sum`

Under-claim is valid: `Σ x_i < F(T)` means the difference is donated — it still enters ASH and raises the floor when cleared, but creates no attestation. Record partitioning is additive: at one sampled floor, splitting one amount among several records for the same address leaves `A(a)` unchanged (`[A-prop:costliness:splitting-invariance]`).

> **Burn provenance versus record acceptance · `rem:ledger:overclaim`**
>
> A burn whose records over-claim remains a genuine burn in the raw log. Only the record set fails. Dropping the entire transaction would conflate two independent verdicts: **provenance** — did a valid burn transition create this ASH? — and **credit** — do its records fit beneath `F(T)`? The separation is load-bearing for append-only history and for independent re-indexing.

> **Canonical record ordinal · `rem:ledger:ordinal`**
>
> Records are indexed contiguously from zero in transaction-output order. Duplicate, skipped, or reordered ordinals make the transition non-canonical and are rejected before indexing. The ordinal gives independent indexers one stable ordering without assigning economic meaning to record position.

### §12.2 Three layers and a hash-bound checkpoint · `sec:ledger:model`

A conforming ledger has three logical layers:

1. **raw burn log** — every provenance-authenticated burn, including burns whose records fail the value gate; stores transaction id, canonical order, block hash, `F(T)`, and raw records; stores **no sampled-clear identifier**;
2. **clear table** — genesis clearing zero plus every kernel-authenticated clear, each carrying canonical order and committed `(Ω_c, Y_c)`;
3. **grouped-credit projection** — derived relative to one canonical chain checkpoint by assigning each accepted record to its most recent earlier clearing.

The raw objects are append-only relative to one canonical history. Clear assignment is not a raw fact: it depends on canonical order and therefore belongs only to the derived projection. A reorg reconstructs layer 3 from the replacement burn and clear sets; it does not mutate a supposedly immutable sampled-clear field. The candidate indexer against which O6 compares is a separately implemented tool of this shape (`[def:verification:reference-indexer]`).

Three roles must not be conflated. The **reference model projection** derives its event and query results from a history assumed to have been produced by the executable-model kernel; it is the trusted *expected* result the O6 differentials compare a candidate against, not an independent recognizer, and no public interface promotes a caller-authored history into it. The **independent deployment event derivation** required by O6 (`obl:oracle:ledger`) recognizes events from validated target transactions and is the only path that authenticates burn provenance from consensus data. The **hash-bound checkpoint** is a consistency-preserving cache of an already-projected reference index: restoring it re-answers the same queries, but it carries no provenance the projection did not already assume and is never independent event evidence.

> **Genesis is clearing zero · `rem:ledger:genesis-clear`**
>
> The clear table begins with the trusted genesis projection, carrying `Ω_0 = Y_0 = E_0` and `φ_0 = 1`, ordered strictly before every ordinary burn. Therefore "the most recent clearing before `b`" is total even for burns preceding the first ordinary clear.

The index is bound to a context `C = (network id, genesis id, architecture hash, checkpoint hash, checkpoint height, wire schema)`.

> **Manifest-hash binding · `rem:ledger:hash-bound`**
>
> The architecture-hash field MUST equal the semantic hash of (`app:realization:architecture`). Canonical query encoding and decoding reject any other hash. Register 1 is therefore bound at the public wire boundary, not only by prose. The three schemas this document lives at — architecture, attestation wire, and deployment-profile — are co-located at (`rem:manifest:discriminants`); the attestation wire schema is the one this context field carries, and it is independent of the other two.

A validated chain view MUST be contiguous and parent-linked from the realization genesis through the checkpoint. Events after the checkpoint do not enter the index. Duplicate event identifiers or non-increasing event order invalidate the view.

The canonical query contains the context `C`, one attestation address, and an ordered list of grouped terms, each carrying clear identity/order, aggregate burn amount, `Ω_c`, and `Y_c`. Terms are strictly ordered by `(clear order, clear id)`, carry no duplicate clear id, and require positive aggregate amount, positive `Ω_c`, and positive `Y_c`.

Canonical serialization MUST enforce a fixed domain separator; fixed-width context fields; minimal variable integers; minimal arbitrary-precision integers; strict term order; no duplicate terms; no term after the checkpoint; and no trailing bytes. Encoding and decoding MUST share the same semantic validator. Same context and semantic result imply byte-identical output.

### §12.3 Valuation and canonical event order · `sec:ledger:order`

Clearings are this realization's designated settlement-event family under the specification's settlement-pinned monotonicity (`[A-postc:interface:monotonicity]`): every burn settles at the floor committed by the most recent clearing, a sanctioned instance of the settlement-pinned clearing reduction (`[A-prop:invariants:burn-order-residual]`) rather than a third valuation scheme. For each accepted record `b = (a, x)` at canonical order `o_b`, let `c(b)` be the clearing of maximal order strictly less than `o_b`. The attestation increment is:

$$δ_b = x·φ_{c(b)} = x·Ω_{c(b)}/Y_{c(b)}$$

A query groups all accepted record amounts sharing `(a, c)` into `X_{a,c}` and emits the exact term `X_{a,c}·Ω_c/Y_c`. The normative export is the ordered term list. Optional reduction to one rational uses arbitrary-precision integers and exact greatest-common-divisor reduction; floating point is never part of conformance.

The canonical event order is `order(e) = (block height(e), transaction index(e))`. Same-block events therefore have a total order. A burn dependent on an output of a clear cannot precede that clear by consensus; a miner may exclude the pair but cannot invert it. An unchained clear and burn remain miner-orderable. A same-transaction clear+burn is unrepresentable: clear consumes STATE and ASH while destroying `U`; burn is root-free, consumes live receipts, and creates fresh ASH — their manifest-derived input/output and projection requirements cannot be satisfied by one transition.

> **⚠ Naïve construction.** Value each burn at a continuously replayed floor reconstructed at its exact block position, or store the sampled floor inside the raw burn row.
>
> **Why it fails (SP2/SP6 — expensive and mutable valuation).** Exact replay couples every burn to the entire preceding pool history, while storing a sampled clearing makes an append-only row depend on a reorg-sensitive canonical order. The data model confuses immutable record with mutable valuation.
>
> **What we do.** Value at the last authenticated clearing's committed floor and keep the assignment in the derived projection. The valuation is a conservative lower bound — since the floor never falls, `φ_{c(b)} ≤ φ_burn`, so `δ_b` never exceeds the redeemable value destroyed at the burn. `[enforced: P-ledger]` (`[A-prop:interface:conservative-valuation]`) `trap:ledger:replay`

### §12.4 Reorg sensitivity · `sec:ledger:reorg`

The raw record `(a, x)` is immutable inside any retained canonical history. The derived value is not chain-independent: a reorg may move the burn across a clearing boundary, changing `c(b)` and therefore `δ_b`, while leaving the burn transaction and its payload unchanged.

A conforming indexer responds by replacing the canonical burn and clear sets with those of the new checkpoint; recomputing grouped clear assignments; and emitting a result under the new checkpoint context. It MUST NOT rewrite the raw payload or claim that valuation stability is record immutability. This is the specification's settlement-pinned monotonicity realized at the data-model boundary (`[A-postc:interface:monotonicity]`): a consumer that samples beyond its own reorg-depth threshold establishes the observed non-decrease; this layer exports the checkpoint-relative facts.

A chained clear→burn pair preserves relative order on every history containing both, because the child consumes the parent. Its absolute sampled state may still change if earlier canonical history changes.

### §12.5 Demand-driven convergence · `sec:ledger:convergence`

Clear captures no value. It is funded when an agent internalizes enough of the floor increase to justify its fee: a redeemer increases their own formula-bound payout; an attestation-seeking burner increases their next record valuation through clear-then-burn. For ash backlog `B` against recorded `Y`, clearing changes the denominator from `Y` to approximately `Y − B`. For a private position of scale `V`, the first-order benefit is approximately `V·B/Y`, and clearing is privately attractive when that exceeds the chain fee.

Consequences: larger positions clear larger backlogs sooner; small backlogs may remain as conservative `U_ash`; several beneficiaries may each prefer another to pay; a burner may enforce clear-before-burn with a dependent transaction pair; competing real STATE operations may invalidate a pending clear, requiring resubmission. These are liveness and fee-market properties, not invariant assumptions. Safety is independent of who clears or when: uncleared ASH keeps `Y` overstated and `φ` understated. This is R-conv (`res:trust:conv`); STATE contention is R-op (`res:trust:op`).

### §12.6 The quantity-reader firewall · `sec:ledger:reader-matrix`

The manifest declares every quantity's readers. The matrix is not documentation around the interface; it **is** the boundary that prevents audit residue and monetary internals from leaking into the exported computation.

*The quantity-reader matrix · `tab:ledger:readers`*

| Quantity | Kind | Permitted readers |
|---|---|---|
| floor `φ` terms | monetary | cycle, redeem, external auditor |
| redemption payout | monetary | redeem |
| cycle issuance | monetary | cycle |
| attestation delta | derived | attestation indexer |
| attestation map `A` | **interface** | attestation indexer, external auditor, **consumer formula** |
| historical live residue | **audit-only** | invariant checker, external auditor |
| historical time-locked residue | **audit-only** | invariant checker, external auditor |
| receipt-accounting audit | **audit-only** | invariant checker, external auditor |

The attestation map is the **only** realization quantity a consumer formula may read. Historical residue is read by neither covenant operation, nor the attestation indexer, nor a consumer formula.

> **⚠ Naïve construction.** When terminal distribution residue is destroyed, decrement `Y` to keep recorded supply equal to current circulating `U`; or expose residue as a correction term to whoever computes the floor.
>
> **Why it fails (G1, G9 — residue-driven over-payment).** Decrementing `Y` raises `φ`; a consumer computing `Ω/(Y − H)` raises it again. Any missed, duplicated, or offsetting residue recognition then changes monetary payouts. Audit history becomes an economic input whose provenance the covenant cannot locally re-evaluate.
>
> **What we do.** Residue destruction does **not** decrement recorded `Y`. The history-derived `H_L, H_T` terms explain the difference in 𝗜₈ but feed no monetary, interface, covenant, indexer, or consumer computation. Recorded supply overstates circulating `U`; `φ` is understated, the safe direction, **independently of any residue quantity**. The reader firewall is checked statically from the manifest and dynamically by O5 residue non-interference. `[enforced: P-residue, P-account]` `[invariant: 𝗜₈]` — **canonical.** `trap:ledger:residue-decrement`

> **Differential boundary · `rem:ledger:differential-boundary`** `[honesty note]`
>
> Byte-identical query comparison detects every recognition error that changes `A`. It cannot, by itself, detect an extra and a missing equal burn whose grouped terms cancel exactly. O6 therefore requires two independent comparisons: an **event/projection differential** — recognized burn, clear, and residue event sets agree with an independent derivation, detecting offsetting event-set errors — and a **query differential** — canonical query bytes agree for every tested address and context. Neither comparison subsumes the other (`obl:oracle:ledger`).

---

## §13 The oracle — the conformance obligation · `sec:realization:oracle`

The oracle is the acceptance contract for a realization model. It does not prescribe implementation architecture, storage layout, code organization, or emitted bytes. It states what a candidate MUST demonstrate before claiming conformance to this document.

> **Conformance thesis · `rem:oracle:thesis`**
>
> A candidate is conformant at the model layer iff: it is bound to the exact manifest in (`app:realization:architecture`); its accepted transition relation preserves every clause of (`sec:realization:invariant`); its rejected transitions satisfy L-atomic (`lem:invariant:atomic`); and it discharges obligations O1–O9 below. Byte-identical script is not required. A conforming deployment separately supplies the evidence named at (`sec:oracle:boundary`). The oracle proves a model against the manifest; it does not prove the emitted script or the substrate beneath it.

### §13.1 The nine obligations · `sec:oracle:obligations`

> **O1 — Inductive preservation · `obl:oracle:inductive`**
>
> A candidate MUST execute arbitrarily long traces from genesis over the complete action family and establish: every accepted transition re-establishes 𝗜₁–𝗜₁₁; every accepted transition satisfies L-floor, L-delta, L-flow, and L-record where applicable; every rejected transition satisfies L-atomic; every reachable accepted state remains a valid input to the next action. The action family MUST cover every declared operation, cadence advancement, and adversarial open-object injection. Action materialization MUST be state-aware: a decoy reserve uses the current reserve value; an entitlement-shaped payload may target the current pending cycle; a maturity action reads the current latch. A static fixture set is insufficient. O1 is the mechanized induction of (`sec:realization:invariant`): genesis is the base case, accepted transitions are the inductive step, and rejected transitions preserve the hypothesis by atomicity.

> **O2 — Adversarial reachability under scarcity · `obl:oracle:adversary`**
>
> The oracle's adversary MUST carry a real asset budget: it MAY inject open L-BTC and foreign assets only up to its available external budget; it MUST NOT inject `U`, `ENT`, `DIST_CTL`, `PID`, `PACE`, `ENT_AUTH`, or `DIST_AUTH` without acquiring them through an accepted transition; closed-asset forgery is therefore **unconstructible**, not merely rejected; open-asset look-alikes remain inert until consumed under an operation-specific rule and MUST NOT alter canonical state merely by existing. The injection family MUST exercise malformed and cross-pool requests, decoy RESV-shaped outputs, zero-value anchor-shaped outputs, and foreign assets carrying state-, entitlement-, or receipt-shaped metadata. Each accepted injection preserves 𝗜; each attempted consumption is then governed by the consuming operation's recognition rule (`trap:architecture:open-closed`). An honest-operation-only fuzzer does not discharge O2: it cannot reach the adversarial states the property exists to classify.

> **O3 — Load-bearing guards and seeded reductions · `obl:oracle:reductions`**
>
> Every safety-critical guard MUST have at least one deliberate corruption that reaches the guard; is rejected for the intended **named reason**; leaves the predecessor unchanged; and changes the semantic hash when it mutates the manifest rather than runtime state. O3 binds to failure reasons, not merely to the broader invariant clause to which a reason maps (`rem:invariant:reasons`). The minimum required reduction set is (`sec:oracle:reductions-list`).

> **O4 — Manifest conformance and non-drift · `obl:oracle:manifest`**
>
> A candidate MUST derive, rather than independently re-type: branch/root policy; input/output/data-output cardinalities; open-flow allow-lists; value-flow classes; projection permissions; finite-bound references; quantity reader and writer permissions. Cross-declaration relations MUST be checked bidirectionally: operation reads ↔ quantity readers; operation writes ↔ quantity writers; outputs ↔ object allocators or mutators; inputs ↔ object mutators or deallocators; issuance declarations ↔ issuance deltas; destruction deltas ↔ tagged data outputs; control counters ↔ entitlement sums ↔ vault value. A semantic mutation MUST change the architecture semantic hash. Presentation changes and publication-envelope status changes MUST NOT. Unknown export fields MUST be rejected. Existing stable discriminants MUST remain fixed.

> **O5 — Reader firewall and residue non-interference · `obl:oracle:firewall`**
>
> The reader matrix of (`sec:ledger:reader-matrix`) MUST hold both statically and dynamically: `attestation-map` is the only realization quantity available to a consumer formula; historical live/time-locked residue is readable only by the invariant checker and external auditor; no covenant operation reads historical residue; the attestation indexer does not read historical residue. A candidate MUST perturb residue history while holding protocol state and all non-residue history fixed, then demonstrate: floor terms unchanged; redemption payout unchanged; cycle issuance unchanged; branch-local accepted/rejected behaviour unchanged; attestation event recognition and query bytes unchanged; and the independent receipt-accounting audit **does** detect the perturbation. Thus residue remains explanatory audit state, never an economic input. The conservatism of (`trap:ledger:residue-decrement`) does not depend on a convention that future readers remember.

> **O6 — Independent projection and indexer conformance · `obl:oracle:ledger`**
>
> O6 has three independent parts: an **attestation-event differential** — an independent implementation derives the canonical burn and clear event sequence and agrees exactly on event identity, order, block context, ASH value, burn records, and clear `(Ω, Y)`; an **attestation-query differential** — from those events, an independent implementation emits semantically valid canonical query bytes identical for the same address and context; and a **receipt-accounting differential** — under the external-auditor role, an independent implementation derives the exact residue-event sequence and accounting totals.
>
> The comparisons MUST remain distinct: query equality does not prove event-set equality, because an extra and missing equal event may cancel in one grouped term; aggregate residue equality does not prove residue-event equality; and event-set equality does not prove correct clear assignment, exact-rational grouping, or canonical encoding. A conforming attestation query uses one canonical ordered traversal of the pre-indexed burn and clear events — `O(burns + clearings + records)` for one address, excluding arbitrary-precision arithmetic and output allocation — and MUST NOT scan every clearing separately for every burn. The differential suite MUST detect over-recognition, under-recognition, and offsetting recognition, and MUST demonstrate that the attestation indexer remains residue-blind while the separate receipt-accounting audit sees residue.

> **O7 — Determinism, canonical encoding, and closure · `obl:oracle:determinism`**
>
> A candidate MUST demonstrate: genesis establishes every invariant clause and seeds clearing zero; the declared asset/root/object/operation/quantity/witness/clause/tag/bound families are complete and duplicate-free; no attestation-accumulator root or object is representable; canonical query encoding round-trips byte-identically; non-minimal integers and variable integers are rejected; duplicate or out-of-order terms are rejected; zero aggregate, zero clear reserve, and zero denominator terms are rejected; terms after the checkpoint and trailing bytes are rejected; wrong architecture hash and unsupported wire schema are rejected; and canonical chain views are contiguous and parent-linked through the selected checkpoint. Determinism is always context-relative: same network, genesis, manifest hash, checkpoint, and schema imply same bytes. Same height alone is not a canonical context.

> **O8 — Representation-parametric conformance · `obl:oracle:representation`**
>
> For every supported value-representation mode, target acceptance MUST imply the same authorized semantic transition and invariant-preserving successor. A representation change to a fact the operation does not observe MUST preserve acceptance and public projection (metamorphic equality); hiding a semantically-required value with no approved alternative proof MUST be rejected. Asset identity of closed assets remains explicit.

> **O9 — Disclosure minimality and declassification-robustness · `obl:oracle:disclosure`**
>
> Every explicit field and every opening MUST trace to a named obligation in `declassification.json`. For the target capability set, no strictly less-disclosing supported proof plan may exist. Each $\mathcal D_o$ MUST be non-launderable: the choice of what to declassify carries no covert dependency on a private fact. Confidential-allowed seams get acceptance vectors; reveal-required seams get blinded-rejection vectors.

### §13.2 Seeded reductions · `sec:oracle:reductions-list`

*The minimum witnessed corruption set · `listing:oracle:reductions`* ◆

Each entry names a corruption and the required verdict. A conforming implementation MAY expose different internal error types, but its conformance harness MUST preserve an equally precise failure classification and MUST NOT accept the corrupted state. Where an entry reads "structurally unavailable," two facts hold: the operation is unrepresentable at the manifest level, and a harness offered the corrupt outpoint rejects with a named reason (`rem:identity:structural-nonredeem`).

**Identity, closure, and open-asset immunity.** Duplicate an amount-one singleton root → identity/authority failure · place `U` under an undeclared foreign shape → canonical-closure failure · place `ENT` under ASH metadata → canonical-closure failure · inject malformed L-BTC junk → accepted as inert, invariant unchanged · inject a decoy RESV matching the active reserve value → active cursor unchanged · relink the active RESV cursor to the decoy → succession failure.

**Domains and active backing.** Genesis at `ACTIVE_BACKING_MAX` → accepted · genesis one unit above it → active-backing-cap failure · admission exactly to the cap → accepted · admission one unit beyond the cap → rejected atomically, requests unspent · cycle at the cap → cap preserved · redemption from a capped pool → active headroom increases · construct `Ω + Q ≥ 2⁵¹` → arithmetic-domain failure.

**Canonical partitions and issuance.** Omit a consumed canonical source from every flow → canonical-delta failure · cite one source in two flows → canonical-delta failure · cite one destination twice → duplicate-output/canonical-delta failure · emit a canonical output claimed by no flow or issuance → canonical-delta failure · claim one output through both issuance and lateral movement → canonical-delta failure · declare issuance without consuming the authority → missing-authority failure · issue an amount not equal to the destination total → issuance failure · declare destruction without the operation's permitted tag → authorization/destruction failure · remove settlement from `U`'s declared destruction lifecycle → manifest validation failure · mutate one operation's active delta family → postcommit manifest-conformance failure.

**Open flows and sponsorship.** Consume positive L-BTC in no open flow → open-flow failure · claim one L-BTC output through two open flows → open-flow failure · claim one L-BTC reference through both a protocol flow and the generic sponsor region → open-flow failure · provide two generic sponsor envelopes → sponsor-envelope failure · omit a sponsor owner signature → authorization failure · emit a zero-valued ordinary sponsor member with exact role structure → accepted by the semantic relation · ask a first-party builder for a known zero-valued sponsor change → omitted by canonical construction policy, not by a protocol predicate · substitute an ordinary sponsor output for the CPFP anchor → family/constructor/ABI-role failure, never a value-zero test · emit the zero-value CPFP anchor in its declared cycle condition → accepted · reduce a formula-bound redemption payout by one while enlarging sponsor change by one, leaving whole-transaction conservation intact and every sponsor amount positive → flow/postcondition failure on the protocol payout, never on sponsor positivity.

**Root succession and history.** Substitute a wrong STATE successor → succession/history failure · omit a required authority edge → succession failure · terminate RESV while the successor STATE is operational → reserve-succession failure · corrupt an intermediate STATE edge and later restore the final cursor → replay detects failure · corrupt an intermediate RESV or PACE edge and later restore the final cursor → replay detects failure · reuse a transaction identifier → history failure · use non-increasing canonical order → history failure · let an off-pool operation advance a root it forbids → succession failure · attempt a pool transition after sealing → tombstone failure.

**Requests and admission.** Request with `principal = gross` → partition failure on admission, inert before consumption · request for another pool → wrong-pool failure on admission · merge two requests before per-entitlement floors → manifest/semantic-conformance failure · issue fewer or more than one entitlement per request → shape/recipient failure · pay an admission reward exactly equal to the preauthorized budget → accepted · pay one unit more → partition failure · admit beyond active-backing headroom → cap failure, requests remain unspent · cycle with `Q > 0` and `ΔY = 0` → accepted R-dust case, entitlements retire at zero draw.

**Cycle and cadence.** Cycle before `MIN` → cadence-too-early failure · permissionless cycle at `MAX − 1` → operator-only failure · permissionless cycle at `MAX` → accepted · operator-band cycle without operator signer → authorization failure · cycle without the required issuance/cadence authority → root/issuance failure · issue `U ≠ ΔY` → issuance failure · route minted `U` outside operator receipts and the distribution vault → mint/flow failure · empty cycle → advances `k`, recreates roots, issues nothing · empty maturity cycle → performs atomic conversion · post-maturity cycle with `Q > 0` → all issuance, including the operator fee share, is live-class.

**Settlement and payability.** Orphan a distribution vault → payability failure · duplicate a vault for one control → payability failure · change vault value without changing control remainders → payability failure · remove an entitlement while leaving its control remainder unchanged → lifecycle failure · settle an entitlement against a control for another cycle → target failure · partial settlement → successor control/vault remains payable · terminal settlement → control and vault close, residue projection exact · zero-draw entitlements → retired permissionlessly · settle after maturity → committed physical classes preserved, then relabelable.

**Transfer, relabel, and authorization.** Transfer one class into the other → class-crossing failure · route receipt `U` to an undeclared/bare destination → closure failure · combine owners while omitting one signer → authorization failure · redeem with the wrong owner → authorization failure · cancel with the receipt owner but not the refund key → authorization failure · relabel before maturity completes → maturity failure · relabel while redistributing owner/value pairs → recipient failure · swap distinct owner/value pairs → recipient failure · merge several relabel inputs into one output where the declared relation requires one-to-one layout → shape failure · correct owner/value multiset → accepted · every permissionless operation accepts without an owner/operator signer.

**Burn, ASH, and clear.** Burn with an ASH input → event-type/shape failure · burn a time-locked receipt → structurally unavailable · burn with live change reclassified time-locked → class failure · burn while routing `U` to a bare key → closure failure · duplicate or skip burn-record ordinal → shape failure · records under-claim `F(T)` → accepted donation · records over-claim `F(T)` → burn accepted, records contribute zero · bare `tag-burn` payload with no burn projection → contributes zero · compact several ASH objects → value preserved, no new burn event · clear a positive ASH batch → `Y_L` and destroyed ASH decrease by the same amount · clear an ASH larger than the available decrement → partial clear, residual ASH re-emitted · clear to zero total supply → impossible by clamp, `Y ≥ 1` · repeat clear at zero progress → rejected · attempt ASH recovery to a spendable owner → structurally unavailable.

**Maturity.** Announce below `Δk_min` → lead-too-short failure · announce above `Δk_max` → lead-too-long failure · announce a cycle not strictly in the future → maturity-coherence failure · announce twice → already-announced failure · announce on a sealed pool → sealed failure · reach `k_m` → conversion executes atomically · post-conversion `Y_T ≠ 0` → maturity/accounting failure · physical `RECEIPT_T` after conversion → valid, accounted live-side, permissionlessly relabelable.

**Residue firewall.** Perturb historical residue while keeping protocol state fixed: floor unchanged · redemption payout unchanged · cycle issuance unchanged · branch-local results unchanged · attestation events and query bytes unchanged · receipt-accounting audit changes and fails the expected accounting relation.

**Event and query differential.** Add a false burn event → event comparison fails · omit a genuine burn event → event comparison fails · replace two burns of `40 + 60` with one false burn of `100` at the same address and clear → query aggregate may match, event comparison MUST fail · omit a clear while querying an address with no burns → empty query may match, event comparison MUST fail · preserve event set but assign a burn to the wrong clear → event comparison may pass, query comparison MUST fail · alter ASH value, record sequence, clear `Ω`, clear `Y`, order, identity, or block context → event comparison fails · same context and semantics → byte-identical query · wrong manifest hash → encoder and decoder reject · reorg across a clear boundary → `δ` reprojects, raw `(a, x)` unchanged · event after checkpoint → excluded · offsetting residue events with equal totals → accounting-event comparison fails even though aggregate totals match.

**Canonical encoding and chain context.** Non-minimal variable integer → rejected · non-minimal arbitrary-precision integer → rejected · duplicate clear term → rejected · out-of-order term → rejected · zero aggregate, clear reserve, or denominator → rejected · term after checkpoint → rejected · trailing bytes → rejected · unsupported wire schema → rejected · checkpoint-hash mismatch → rejected · non-contiguous chain view → rejected · parent-hash mismatch → rejected.

### §13.3 Boundary of the oracle · `sec:oracle:boundary`

The oracle establishes the manifest-bound model relation. It does **not** establish that emitted tapscript faithfully enforces that relation; that signature bytes and sighash flags match model authorization; that Elements exposes the assumed explicit-value and issuance introspection semantics; that relative timelocks and package relay behave as assumed; that provably unspendable tagged outputs are excluded as required; that finite bounds fit real script weight and witness limits; that L-BTC settlement behaves as reserve-asset settlement; or that the four low-level arithmetic gadgets implement their `Sat`-level contracts.

> **The next recognition class · `rem:oracle:next-class`**
>
> The model-to-script seam is the same correspondence problem at a lower layer: *does the emitted artifact correspond to the asserted model?* Internal model consistency cannot answer. A conforming deployment binds the exact architecture semantic hash; normative model source; compiler configuration; emitted script bundle; independent indexer and accounting implementations; canonical wire vectors; and unit, property, event-differential, query-differential, accounting-differential, and script-integration reports. Each is hashed in the deployment profile. Architecture release and deployment release are distinct assurance boundaries (`sec:trust:verify`).

> **One line · `rem:oracle:one-line`**
>
> A candidate is conformant when its model keeps every clause of (`sec:realization:invariant`) under O1–O9 against the attached manifest — regardless of its script bytes — and its deployment separately discharges the `{verify}` surface. **The invariants are the specification; script is one realization of them.**

---

## §14 Trust surface, residuals, dependencies, and decisions · `sec:realization:trust`

The G-table (`tab:requirements:guarantees`) and the residual catalogue below partition the property space. A guarantee names something a conforming realization prevents. A residual names something that may occur, remains inside 𝗜, and is tolerated with its scope stated. No fact appears in both.

*The trust surface · `tab:trust:surface`*

| Claim | Evidence |
|---|---|
| branch semantics preserve 𝗜 | model + oracle O1–O3, O8 |
| manifest and model do not drift | typed manifest + O4 + semantic hash |
| closed-asset scarcity/no-forgery | substrate native-asset conservation `{verify}` |
| authority-gated issuance | model L-delta + issuance introspection `{verify}` |
| root and object welds | model certificate + emitted covenant enforcement `{verify}` |
| owner/operator authorization | model signer set + signature/sighash evidence `{verify}` |
| attestation event recognition | independent raw-event differential, O6 |
| attestation query computation | independent query-byte differential, O6 |
| historical residue/accounting | independent external-auditor differential, O5/O6 |
| arithmetic above `Sat` | model and property tests |
| arithmetic below `Sat` | gadget vectors + hand-audit/machine-checked port `{verify}` |
| reserve-asset settlement | Liquid functionary consortium `{verify}` |
| liveness in a fee market | not guaranteed; only safety of each valid contender is model-proven |

> **Favourable structural facts · `intuit:trust:favorable-facts`**
>
> Four facts follow from the architecture rather than from participant virtue: **full active reserve** — while operational, the actual RESV value is `Ω + Q`, welded to STATE; **no promised return** — the floor rises only through voluntary forfeiture and conservative rounding; **no operator custody path** — reserve leaves only through formula-bound owner redemption, and the operator cannot redirect it; **no economic numeric discretion** — issuance, splits, fees, settlement draws, and payouts are public-state functions. These statements do not erase the named dependencies or residuals below.

> **Guarantees and residuals are disjoint · `rem:trust:partition`**
>
> A self-inflicted, publicly visible, conservative loss is still a residual, not a guarantee violation. Conversely, a property that depends on substrate evidence is still a guarantee if deployment release requires that evidence. Classification follows *what may happen in a conforming deployment*, not whether the event is pleasant.

### §14.1 Named residuals · `sec:trust:residuals`

> **R-arith — low-level arithmetic surface · `res:trust:hand-audit`** `[accepted residual]`
>
> The invariant witnesses `Sat`-level relations, not limb carries and borrows. The wide multiply and sandwich gadgets therefore retain a hand-audit surface until replaced by a machine-checked implementation. A failed gadget would be a deployment failure, not an accepted state; the residual is evidence burden (`rem:arithmetic:hand-audit`).

> **R-CSV — cadence substrate dependence · `res:trust:csv-version`** `[liveness, not safety]`
>
> Relative-timelock realization depends on substrate sequence/version semantics. Failure to meet them makes the cycle transaction non-confirming; it does not bypass the cadence floor or ceiling. Deployment evidence must show the emitted leaves implement the band (`leaf:authorization:cadence-band`).

> **R-lock — conservative locked reserve · `res:trust:locked-residue`** `[accepted residual]`
>
> `U_ash` before clearing and `H_L + H_T` permanently remain in recorded `Y`, understating `φ`. In wind-down states, reserve may remain with no spendable receipt claim. The value is permanently un-redeemable and uncapturable, described by (`sec:invariant:terminals`). This is the permanent face of the lazy φ-rise introduced at (`sec:architecture:burn-ledger`).

> **R-issue — issuance dependency · `res:trust:issue`** `[accepted residual]`
>
> No-inflation depends on `PACE` being the sole `U` reissuance authority, `ENT_AUTH` and `DIST_AUTH` being unique, and emitted issuance introspection pinning amounts and destinations. The model proves the intended relation; the substrate/compiler correspondence is deployment evidence.

> **R-conv — demand-driven clearing · `res:trust:conv`** `[liveness, not safety]`
>
> Clear captures nothing and is funded only when some beneficiary internalizes enough of the floor increase. Large backlogs and positions clear sooner; small backlogs may persist; beneficiaries may wait for each other. Uncleared ASH remains conservative and never breaks 𝗜 (`sec:ledger:convergence`).

> **R-lag — issuance at the stale floor · `res:trust:lag-issuance`** `[accepted residual]`
>
> A cycle executed while ASH is pending mints `⌊QY/Ω⌋` against the recorded (overstated) `Y`, allocating part of the uncleared burns' latent floor rise to the incoming cohort. Safety-neutral: the recorded floor is preserved exactly and 𝗜₈ carries the lag. Incumbents and burners therefore hold a clear-before-cycle incentive symmetric to the documented clear-then-burn incentive; ordering on the contended STATE is fee-market-resolved (`res:trust:op`).

> **R-cred — off-chain attestation computation · `res:trust:cred`** `[accepted residual]`
>
> The covenant does not enforce `A`. Conformance depends on the dual-anchor indexing rule, canonical context, exact rational arithmetic, and independent event/query differentials. A user may trust a particular indexer; a verifier may recompute from history. The public rule is deterministic, but it remains off-chain.

> **R-op — serialized shared state and fee competition · `res:trust:op`** ⚓ `[liveness, not safety]`
>
> Operations spending STATE, RESV, PACE, or a distribution control contend on those UTXOs. The model proves that **every valid contender is safe** and that only one conflicting transaction can win; it proves nothing about inclusion, fee-market selection, or latency. Burns and root-free transfers remain parallel. A dependent clear→burn pair may require resubmission after a competing valid STATE operation, but its relative order cannot invert. Contention includes **deliberate minimal-progress churn**: a griefer may feed 1-unit clears or single-receipt relabels to occupy STATE cheaply. All such churn is safety-neutral, but it is a griefing amplifier, not merely competition among legitimate operations. A progress floor on clear was considered and rejected: it would strand dust ash below the floor, contradicting the no-stranding property (`rem:trust:dissolved`).

> **R-dust — sub-floor deposit absorption · `res:trust:dust`** `[accepted residual]`
>
> If a cycle has `Q > 0` but `⌊QY/Ω⌋ = 0`, no receipt value is issued. The principal enters `Ω`, the corresponding entitlements settle at zero, and incumbents receive the floor increase. This is publicly predictable, conservative, and permissionlessly retired, but it is a real fund loss for a dust depositor. Deposit clients SHOULD refuse or aggregate values that cannot produce positive issuance under current public state. Relatedly, redemption consumes exactly one receipt per transaction, so dust receipts must be consolidated via transfer before redemption is economical; receipt clients SHOULD surface consolidation alongside redemption.

> **R-audit-verify — computational audit under confidentiality · `res:trust:audit-verify`** `[accepted residual]`
>
> With confidential receipt values, part of G9 moves from information-theoretic recomputation to computational proof-chain verification under the CT binding assumption. Lateral transitions prove zero net change to the public aggregate; boundary transitions declassify their delta. A trust-surface *shift*, not an invariant weakening (`rem:trust:partition`).

> **Structurally absent candidates · `rem:trust:dissolved`**
>
> Two tempting residuals are not residuals of this architecture: a decoy RESV-shaped output cannot substitute for the active reserve because provenance selects the cursor and open look-alikes are inert; and an oversized ASH cannot become permanently unclearable merely because of size, because clear clamps the decrement and re-emits the remainder. They are stated here only to prevent them being catalogued elsewhere as tolerated behaviour; the structure removes them.

### §14.2 Deployment verification surface · `sec:trust:verify`

*Required deployment dependencies · `tab:trust:dependencies`*

| Dependency | Required evidence |
|---|---|
| native-asset conservation | closed assets cannot be forged outside declared issuance |
| issuance introspection | emitted predicates bind asset, authority, amount, and destinations |
| explicit-value introspection | explicit-arithmetic and public-delta seams read the same values consensus enforces |
| confidential-value conservation | blind lateral movements conserve value through consensus CT balancing |
| value-commitment equality | amount-blind relabel preserves per-object value by commitment equality |
| value-commitment opening | reveal-at-spend paths, if implemented, authenticate openings on-chain |
| sighash profile | every owner/operator signature commits the required outputs; input-extension policy cannot weaken them |
| package relay | maturity-cycle CPFP anchor can fund the intended package |
| unspendable-output exclusion | tagged destruction outputs do not remain spendable current state |
| weld enforcement | emitted scripts enforce root, control/vault, and companion relations |
| L-BTC settlement | reserve units settle under the named functionary consortium |
| script-emission fidelity | emitted script implements the model transition relation |

Every listed dependency except value-commitment opening is `verification_required` in (`app:realization:architecture`) — opening evidence becomes required only when a reveal-at-spend path ships. That field expresses an obligation, never a completed status. The four value rows are an earlier revision's split of the former single `explicit-values` row into its proof-method components (`sec:realization:representation`); dependencies are normative-descriptive, so the split is a letter under (`rule:versioning:decision`).

A final deployment profile — at the deployment schema of the three co-located at (`rem:manifest:discriminants`) — MUST bind: the final, anchor-pinned architecture semantic hash; nonzero network and genesis identities; every deployment-calibrated finite bound and its measurement evidence; script limits and measurements within them; verified evidence for every dependency above; hashes of normative model source, compiler configuration, emitted script bundle, reference indexer, generated manifest artifacts, and canonical wire vectors; and separate report hashes for model tests, property tests, raw event differential, canonical query differential, receipt-accounting differential, and script integration. The three independent-evidence report hashes are committed separately, so no one differential can stand in for another. Architecture finality is necessary and insufficient; deployment release is a separate validator and a separately domain-separated hash.

### §14.3 Closed architecture decisions · `sec:trust:decisions`

*Closed decisions · `tab:trust:decisions`*

| Decision | Status | Rationale |
|---|---|---|
| no on-chain attestation accumulator | closed | full-chain verifiers recompute; a root is not self-verifying; a shared root serializes burn |
| full-chain attestation verification | closed | verifier parties maintain canonical history; ordinary users may trust their Layer-1 view |
| historical residue audit-only | closed | residue explains conservative supply; no monetary, interface, covenant, indexer, or consumer computation reads it |
| one entitlement per request | closed | flooring is not aggregation-invariant; admission has no payout-affecting grouping discretion |
| settlement permissionless | closed | lost owner keys must not retain shared distribution state; outputs are owner-preserving and formula-bound |
| atomic maturity cycle | closed | no announced-but-unconverted limbo; conversion occurs in the cycle reaching `k_m` |
| active-backing cap | closed | simultaneously active reserve plus escrow remains within the declared maximum; cumulative historical deposits remain uncapped |

### §14.4 Out of scope · `sec:trust:out-of-scope`

The following are intentionally not covenant guarantees: positive **net economic** cost when one holder controls all supply — the specification's conservation-identity market regime (`[A-thm:costliness:conservation-identity]`); what incentivizes external deposits; whether and when the operator announces maturity before any schedule exists (`[A-rem:maturity:announcement-timing]`); operator versus non-operator deposit disambiguation (`[A-open:attestation:operator-disambiguation]`); the consumer's sampling/frozen-factor half of cross-layer burn timing (`[A-open:attestation:leverage-timing]`); calibration of `ζ`, `f`, maturity leads, cadence, and finite bounds (`[A-rem:model:calibration]`); functionary behaviour beyond the named L-BTC settlement dependency; and the consumer's wealth-propagation dynamics (`rem:overview:two-neutralities`). Out of scope means the property belongs to another layer or evidence boundary; it does not mean the opposite is claimed.

One reservation is structural rather than semantic: the model's world state carries a `wallets` map reserved for future per-owner L-BTC bookkeeping. In this realization it is written by no transition and read by no invariant — external funds enter only through the explicit external-budget and plain-L-BTC paths — and its inertness is itself witnessed by a model test. It is retained to avoid a state-shape break when a later version activates it.

---

The remainder of the document is (`sec:realization:pins`) — the weld-scope boundary and all 38 build-blocking pins — followed by the codegen checklist, (`sec:realization:reviewer`), the generated (`sec:realization:anchors`), and the appendix. (A thirty-ninth pin, P-denotation, is homed with the versioning law at (`sec:realization:versioning`).)

---

## §15 Build pins and code-generation obligations · `sec:realization:pins`

Pins bind the manifest and model to a concrete realization. They do not prescribe one byte sequence; each states a relation that an emitted covenant, indexer, compiler, or deployment bundle MUST enforce. A script-facing pin carries `{verify}` because its model relation is established here while faithful emission is deployment evidence (`sec:oracle:boundary`).

> **Scope of the automatic document weld · `rem:pins:weld-scope`**
>
> CI holds sixteen manifest-carried labels verbatim against this document: the eleven invariant-clause labels, L-delta, L-floor, L-flow, the ledger authentication subsection, and R-op. Every other section, trap, obligation, rule, pin, residual, listing, figure, table, and leaf label is protected by the publication checklist rather than by the manifest. Two further machine-held adjuncts sit beside the weld, each with its own guard, not the sixteen-string one: the guard/invariant-error vocabulary is a checked projection of the model enums (`listing:domains:guard`), and the model register's own labels are shape-tested and indexed in the code repository, cited from this document only as bracketed model-typed spans in boundary notes. Machine harvesters key on the bare backticked label token; the backtick form is therefore part of the frozen convention, not typography. The distinction is deliberate and exact: the automatic weld guards labels the manifest carries; the projection test guards the failure vocabulary; the model-register harness guards the code-owned labels; and the document's remaining API is frozen and reviewed at ship. The versioning pin (`pin:pins:denotation`) and the representation pins ((`pin:pins:asset-boundary`) through (`pin:pins:ct`)) are checklist-and-CI protected, not sixteen-label welded.

### §15.1 Arithmetic and value pins · `sec:pins:arith-value`

> **P-arith · `pin:pins:arith`** `{verify}`
>
> Every amount entering protocol arithmetic MUST satisfy `0 ≤ v < 2⁵¹`; every quotient site MUST enforce `D > 0`, `q < 2⁵¹`, and both sides of `q·D ≤ N < (q+1)·D`. Small-ratio products and wide products MUST satisfy the bounds of (`rem:domains:bounds`). Witness: arithmetic vectors and emitted gadget tests. Why: (`trap:translation:lower-bound`). Goal: 𝗜₂ and G1.

> **P-cap · `pin:pins:cap`** `{verify}`
>
> Genesis, admission, cycle, and invariant validation MUST enforce `Ω + Q ≤ ACTIVE_BACKING_MAX`. The value is a cap on simultaneous active backing, not cumulative volume. An admission over the cap MUST leave every request unspent. Why: (`trap:domains:active-backing`). Goal: 𝗜₂.

> **P-explicit · `pin:pins:explicit`** `{verify}`
>
> Every value consumed by covenant arithmetic MUST be explicit and introspected as the same value consensus enforces. A blinded or independently committed amount MUST NOT satisfy a receipt, entitlement, vault, ASH, reserve, payout, or issuance check. Witness: deployment explicit-value and introspection tests. Why: (`trap:branches:committed-value`). Goal: 𝗜₉ (`inv:invariant:consensus-value`).

> **P-value · `pin:pins:value`** `{verify}`
>
> Actual consensus value is authoritative. Every metadata partition MUST exhaust the real source value; every formula-bound payout MUST equal the model result; operational RESV MUST equal `Ω + Q`; sealing redemption MUST exhaust both supply and reserve. Why: (`trap:branches:committed-value`), (`trap:branches:sealing-terminal`). Goal: 𝗜₅ and 𝗜₉.

### §15.2 Identity, openness, and authority pins · `sec:pins:identity`

> **P-ident · `pin:pins:ident`** `{verify}`
>
> Every root input and successor MUST bind to its canonical asset identifier, tracked predecessor, and declared object relation. STATE MUST carry amount-one `PID`; every state-dependent operation MUST consume the replayed STATE root and recreate its declared successor. No pool-shaped or reserve-shaped substitute may authenticate a read. Why: (`trap:architecture:recognize-unforgeable`). Goal: 𝗜₁ and 𝗜₁₀.

> **P-distinct · `pin:pins:distinct`** `{verify}`
>
> Asset identifiers, singleton authorities, root identities, object commitments, and the six domain tags MUST be pairwise distinct wherever aliasing would let one family satisfy another. Exactly one `U` reissuance authority exists and is `PACE`; exactly one tag participates in attestation and is `tag-burn`. Why: (`trap:architecture:authority-off-state`), (`trap:architecture:two-tokens`). Goal: 𝗜₁ and G5.

> **P-open · `pin:pins:open`**
>
> Open-asset objects MUST be inert until consumed by an operation-specific validator. Standing invariant classification MUST fail only on malformed closed objects or corrupted tracked roots, never on arbitrary L-BTC or foreign junk. Why: (`trap:architecture:open-closed`). Goal: 𝗜₁ and O2.

> **P-ent · `pin:pins:ent`** `{verify}`
>
> A deposit entitlement MUST be actual `ENT` issued under `ENT_AUTH`, with consensus value equal to principal, owner and target fixed by the admitted request, and no owner-signature consumption path. Forging an entitlement without acquiring `ENT` MUST be impossible. Why: (`trap:architecture:entitlement-scarcity`). Goal: 𝗜₇ and G5.

> **P-admit · `pin:pins:admit`** `{verify}`
>
> Admission MUST issue exactly one entitlement per request, preserve each request's principal and receipt owner, target `k+1`, update `Q` and RESV by the exact principal sum, and reject any reward above the preauthorized budget. No pre-floor request merge is permitted. Why: (`trap:branches:admit-merge`). Goal: 𝗜₂ and 𝗜₇.

> **P-issue · `pin:pins:issue`** `{verify}`
>
> `U`, `ENT`, and `DIST_CTL` issuance MUST consume their declared amount-one authorities and occur only in their declared operations: `U` under `PACE` in cycle; `ENT` under `ENT_AUTH` in admission; `DIST_CTL` under `DIST_AUTH` in a non-empty cycle. Singleton root assets MUST never be reissued or destroyed. Why: (`trap:architecture:authority-off-state`), (`trap:architecture:conservation-inflation`). Goal: 𝗜₁ and L-delta.

> **P-mint · `pin:pins:mint`** `{verify}`
>
> A cycle MUST carry no `U` inputs, issue exactly `ΔY`, and exhaust that issuance into the model-pinned operator receipt outputs and optional distribution vault — no fourth destination, no amount slack, no alternate class. Empty cycles issue zero. Witness: issuance-introspection and emitted-output differential vectors. Why: (`trap:branches:cycle-capture`). Goal: 𝗜₈ and G5.

### §15.3 Delta, tag, certificate, and weld pins · `sec:pins:delta-cert`

> **P-delta · `pin:pins:delta`** `{verify}`
>
> Every consumed and created canonical value object MUST belong to exactly one issuance or flow. Every active delta family MUST match the manifest's operation and activation conditions; no source or destination may be omitted, duplicated, or offset by another wrong flow. Why: (`trap:branches:canonical-delta`). Goal: L-delta.

> **P-tag · `pin:pins:tag`** `{verify}`
>
> Every destruction MUST use its declared tag and exactly one matching non-spendable data output. The six tags are pairwise distinct. Only `tag-burn` records participate in attestation; `tag-recon`, `tag-redeem`, `tag-entitlement`, `tag-distribution-control-close`, and `tag-distribution-residue` are attestation-silent. Why: (`trap:branches:canonical-delta`). Goal: L-delta and G3.

> **P-cert · `pin:pins:cert`**
>
> The model MUST derive every root edge, canonical delta, open-flow projection, and specialized event projection from actual consumed and created objects. Operation code MUST NOT author provenance. Every current cursor MUST equal full-history replay. Why: (`trap:branches:cert-single-weld`). Goal: 𝗜₁₀.

> **P-weld · `pin:pins:weld`** `{verify}`
>
> Emitted covenant predicates MUST enforce every kernel-derived certificate relation: root succession or termination; canonical and open-flow exactness; authority consumption; event-type shape; consumed/created membership needed by each branch postcondition. Witness: emitted-script hash and relation-indexed differential script vectors in the deployment profile — not this document. Why: (`trap:translation:bodies-sufficient`). Goal: T13 (`rule:translation:certificate-leaf`) and 𝗜₁₀.

### §15.4 Terminal, distribution, and permissionless-path pins · `sec:pins:terminal`

> **P-notrap · `pin:pins:notrap`** `{verify}`
>
> The only transition reaching `Y = 0` is sealing redemption with `x = Y ∧ Q = 0`. Clear MUST clamp its decrement by `Y − 1` and reject zero progress. Admission into a sealed pool and later pool transitions after sealing MUST fail. Why: (`trap:branches:redeem-drain`). Goal: 𝗜₄ and 𝗜₁₀.

> **P-settle · `pin:pins:settle`** `{verify}`
>
> Settlement MUST be permissionless and owner-preserving; consume only matching-cycle entitlements; apply each floor before output aggregation; route each class draw to the committed owner; and close control/vault only when remaining principal reaches zero. Why: (`trap:branches:settlement-sig`). Goal: 𝗜₆, 𝗜₇, and L-flow.

> **P-counter · `pin:pins:counter`** `{verify}`
>
> Every live distribution MUST maintain the control/vault bijection — `value(vault) = R_L + R_T` and `R_Q = Σ value(ENT targeting the cycle)`. A continuing settlement recreates both sides consistently; a terminal settlement destroys control and residue under their declared tags. Why: (`trap:branches:control-vault`). Goal: 𝗜₆ and 𝗜₇.

> **P-ransom · `pin:pins:ransom`** `{verify}`
>
> No permissionless path may contain an owner or operator signature requirement. Admission, settlement, receipt relabel, ASH compaction, clear, and the delayed cycle path remain secret-free. Their recipients and state deltas are fixed by L-flow and branch postconditions. Why: (`trap:branches:cycle-opkey`), (`trap:branches:settlement-sig`). Goal: G8.

### §15.5 Burn, ASH, clearing, and class pins · `sec:pins:burn`

> **P-burn · `pin:pins:burn`** `{verify}`
>
> An accepted burn MUST consume only positive live receipts; require every consumed owner; consume no ASH and no root; create exactly one positive fresh ASH; route every other spendable `U` output into positive live receipt change; conserve `U_in = F(T) + C`; and emit only canonically indexed positive records. Burn-record acceptance remains off-chain under P-ledger. Why: (`trap:ledger:event-anchor`), (`trap:branches:burn-change`). Goal: G3 and 𝗜₈.

> **P-roach · `pin:pins:roach`** `{verify}`
>
> ASH MUST be ownerless and closed under exactly two semantic movements: ownerless compaction into ASH; and clear into `tag-recon` destruction plus optional ASH residual. No owner recovery, live receipt, time-locked receipt, reserve, or arbitrary `U` destination is permitted. Actual destruction occurs only at clear. Why: (`trap:ledger:event-anchor`). Goal: G3 and 𝗜₈.

> **P-clear · `pin:pins:clear`** `{verify}`
>
> Clear MUST consume the replayed STATE root and a non-empty bounded ASH batch; consume no RESV; derive `B` from every presented ASH's actual value; compute `X = min(B, Y_L, Y − 1)`; require `X > 0`; destroy exactly `X` under `tag-recon`; re-emit exactly one ASH residual iff `B − X > 0`; and change only `Y_L` in pool state. Why: (`trap:branches:clear-weld`), (`trap:branches:clear-committed`). Goal: 𝗜₄, 𝗜₅, 𝗜₈, and L-floor.

> **P-transfer · `pin:pins:transfer`** `{verify}`
>
> Live and time-locked transfers MUST be separate class-closed relations. Every consumed owner authorizes; every output is positive, recognized, and of the same class; total class value is preserved. No ASH, reserve, vault, control, or bare-key `U` destination is permitted. Why: (`trap:branches:transfer-generic`). Goal: G6 and L-flow.

> **P-relabel · `pin:pins:relabel`**
>
> **Model half:** the consumed time-locked set and emitted live set have identical owner/value multisets, maturity is complete, and STATE is recreated byte-identically. **Emission half `{verify}`:** the emitted relation MUST make that bijection locally checkable — for example, by a canonical positional layout — and pass differential script vectors. The physical layout is not the safety property; owner/value preservation is. Why: (`trap:branches:settlement-sig`). Goal: 𝗜₈ and G8.

### §15.6 Maturity, clock, and companion pins · `sec:pins:maturity`

> **P-atomic · `pin:pins:atomic`** `{verify}`
>
> The cycle reaching `k_m` MUST atomically set `Y_L := Y_L + Y_T`, `Y_T := 0`, and maturity `Complete`, including when `Q = 0`. No separate trigger, delayed conversion state, reverse transition, or second conversion is permitted. Why: (`trap:architecture:two-clocks`). Goal: 𝗜₁₁.

> **P-flavor · `pin:pins:flavor`** `{verify}`
>
> Cadence is relative block age of `PACE`; maturity is committed-cycle arithmetic. Announcement MUST enforce both `Δk_min` and `Δk_max` against predecessor `k` and carry no maturity timelock. No height↔cycle conversion exists. Why: (`trap:architecture:two-clocks`). Goal: G7, G8, and 𝗜₁₁.

> **P-companion · `pin:pins:companion`** `{verify}`
>
> Every root input is covenant-companion authorized by the enclosing relation and only by an operation whose manifest root-use permits it. No root requires an independent owner signature; no non-root object may claim companion status. Why: (`trap:branches:companion-auth`). Goal: 𝗜₁₀.

### §15.7 Accounting, reader, flow, sponsor, and ledger pins · `sec:pins:ledger`

> **P-account · `pin:pins:account`**
>
> The proof-side accounting fold MUST include both circulating receipt classes; ASH; both live distribution class remainders; both historical residue classes; and pre-/post-maturity class treatment exactly as in 𝗜₈. Dropping `H_T` at conversion or omitting ASH is an accounting failure. Why: (`trap:invariant:compute-onchain`). Goal: 𝗜₈ and O1.

> **P-residue · `pin:pins:residue`**
>
> Historical residue MUST remain audit-only. It MUST NOT feed floor, payout, issuance, branch semantics, attestation indexing, or consumer formulas. Static reader validation and dynamic non-interference MUST both pass; the independent receipt-accounting audit MUST detect residue corruption. Why: (`trap:ledger:residue-decrement`). Goal: 𝗜₈ and O5.

> **P-flow · `pin:pins:flow`**
>
> **Model half:** every ordinary L-BTC input/output belongs to exactly one declared open flow; each role balances; formula-bound recipients and amounts satisfy branch postconditions; at most one sponsor envelope appears. **Emission half `{verify}`:** emitted leaves MUST introspect and pin the same source/destination partition, payout, refund, and sponsor isolation. Witness: differential script vectors and sighash profile. Why: (`trap:branches:recipient-safety`). Goal: L-flow.

> **P-sponsor · `pin:pins:sponsor`** `{verify}`
>
> A generic sponsor envelope consumes only ordinary sponsor-owned L-BTC, emits at most one sponsor change output, and contributes exactly its declared fee. It MUST NOT fund or alter a formula-bound payout, request refund, reserve carry, or admission principal, and no emitted sponsor path may consume an individual sponsor amount — positivity included — as a protocol fact ((`sec:kernel:sponsor-opacity`), (`def:representation:sponsor-erasure`)). Why: (`trap:branches:sponsor-envelope`). Goal: G10 and L-flow.

> **P-ledger · `pin:pins:ledger`**
>
> The shipped indexing rule MUST implement burn/clear event recognition from kernel-authenticated projections; the dual anchor and all-record fail-closed gate; contiguous record ordinals; genesis clearing zero; canonical `(height, tx_index)` order; manifest-hash-bound context; one-pass clear/burn merge; exact rational terms and canonical encoding; and separate event, query, and receipt-accounting differentials. It MUST remain residue-blind. Why: (`trap:ledger:event-anchor`), (`trap:ledger:scan-sum`), (`trap:ledger:residue-decrement`). Goal: SP2–SP4, SP6, O5, and O6.

### §15.8 Representation and disclosure pins · `sec:pins:representation`

> **P-asset-boundary · `pin:pins:asset-boundary`** `{verify}`
>
> Every closed-asset seam exposes an explicit asset ID; no confidential output may carry a closed asset unclassified. Why: (`trap:architecture:open-closed`). Goal: G5, 𝗜₈.

> **P-opacity · `pin:pins:opacity`**
>
> A leaf inspects or opens only facts in its operation's dependency graph. Why: (`sec:realization:representation`). Goal: O8.

> **P-discharge · `pin:pins:discharge`**
>
> Every semantic relation names ≥1 target proof method; every emitted check traces to a relation. Goal: O8, O9.

> **P-declassify · `pin:pins:declassify`**
>
> An operation reveals exactly its $\mathcal D_o$ synchronization quantities; residual blinding routes to a public-committed output. Why: (`sec:realization:representation`). Goal: O9.

> **P-representation-liveness · `pin:pins:repr-liveness`**
>
> Every representation mode retains an authorized path to each lifecycle exit; permissionless operations require no private witness ($D_{\text{live}}$). Goal: G8.

> **P-ct · `pin:pins:ct`** `{verify}`
>
> Commitment-equality and CT-conservation semantics used as witnesses are deployment-tested against the pinned target. Goal: O8.

### §15.9 Code-generation checklist · `sec:pins:codegen`

*The compiler and deployment checklist · `tab:codegen:checklist`* ◆

A conforming toolchain MUST complete every item:

1. **Bind the manifest.** Ingest (`app:realization:architecture`), verify schema, semantic hash, publication envelope, stable identifiers, and the pinned specification anchor set before release.
2. **Resolve deployment calibration.** Replace every `requires_deployment_calibration` draft bound with a measured positive value compatible with manifest minima and script limits; bind the exact values in the deployment profile.
3. **Generate operation shapes.** Derive every allowed input, output, data-output, root-use, projection, flow, and bound from the manifest — no parallel policy tables.
4. **Separate authorizations.** Compile input participation and operation permission independently; root inputs are companions, owner/refund/sponsor inputs require their own modes, cadence remains composite.
5. **Emit root and certificate relations.** Enforce each root edge, succession/termination condition, cursor identity, and sealed tombstone under T12/T13.
6. **Emit exact closed-asset partitions.** Enforce one witness per canonical source/destination, authority-bound issuance, tagged destruction, and active delta-family conformance.
7. **Emit exact open-flow partitions.** Enforce role-local balances, immutable/formula-bound destinations, chain fee equality, and one sponsor envelope.
8. **Emit arithmetic relations.** Enforce `Sat` domains, active-backing cap, quotient sandwiches, class splits, cycle issuance, settlement draws, redemption payout, and clear clamp.
9. **Bind consensus value.** Require explicit values and introspection wherever the model reads value; reject any metadata/consensus divergence.
10. **Emit event projections.** Make burn, clear, and terminal-residue event types independently derivable from transaction shape; tags never substitute for event provenance.
11. **Preserve permissionless no-capture.** Grep and test every permissionless path for hidden signatures and variable recipients; run P-ransom and L-flow vectors.
12. **Generate and compare vectors.** Produce canonical wire vectors, model transition vectors, relation-indexed script vectors, event snapshots, query outputs, and receipt-accounting projections; compare with independent implementations.
13. **Publish one bound bundle.** Hash the model source, compiler configuration, emitted script bundle, indexers/auditors, manifest artifacts, vectors, and all reports into the final deployment profile; run architecture-release and deployment-release validators separately.
14. **Emit the declassification map.** Emit `declassification.json` derived from model read-sets; assert every emitted disclosure traces to it (O9).

---

## §16 Reviewer's guide · `sec:realization:reviewer`

The fastest path to confidence is not to read every operation in source order. It is to audit the eight load-bearing seams below, then run the oracle.

> **1 — Arithmetic gadgets · `rem:reviewer:item-arithmetic`**
>
> Confirm the four low-level gadgets implement their `Sat`-level contracts: wide multiply, borrow-propagating comparison, quotient sandwich, and ratio sandwich. Check `q < 2⁵¹`, strict upper bounds, zero divisors, limb/carry limits, and vectors at every boundary. This is the only explicit low-level how (`rem:arithmetic:hand-audit`).

> **2 — Kernel certificate and model↔script boundary · `rem:reviewer:item-kernel`**
>
> This is the weightiest seam. Confirm operation code cannot author provenance; each certificate edge, delta, flow, and event derives from actual objects; replay catches intermediate corruption; and T13 maps every relation to an emitted obligation. Then verify the deployment, not the prose, supplies the emitted-script hash and relation-indexed differential vectors (`pin:pins:weld`). The model's public transition surface is a sealed set (`[rule:verification:pure-transition]`); confirm no low-level construction path bypasses it.

> **3 — Issuance · `rem:reviewer:item-issuance`**
>
> Confirm each reissuable asset has exactly one authority and issuing operation; the authority is consumed; the amount exhausts its destinations; the cycle carries no `U` input; `U` issuance equals `ΔY`; and an empty cycle still advances `PACE` without issuance. A clean aggregate total is not issuance authorization (`trap:branches:canonical-delta`).

> **4 — Identity and recognition · `rem:reviewer:item-identity`**
>
> Confirm exact singleton roots, cursor-selected RESV, open-object inertness, closed-object closure, actual-value authority, and absence of any pool-shaped or tag-shaped substitute. Verify every STATE-dependent operation carries a real succession edge — no read-only state primitive exists (`sec:identity:state-succession`).

> **5 — Deposit pipeline · `rem:reviewer:item-pipeline`**
>
> Walk request → admission → cycle → settlement. Confirm principal/budget partition, one scarce entitlement per request, floors before aggregation, control/vault bijection, owner-preserving permissionless settlement, zero-draw retirement, R-dust disclosure, and committed-class settlement after maturity.

> **6 — Burn, ASH, and clear · `rem:reviewer:item-burn`**
>
> Confirm burn is live-only and lateral, creates one fresh ASH, and shares no root; compaction is ownerless and attestation-silent; clear derives actual value, clamps at `Y − 1`, destroys under `tag-recon`, and re-emits residual; and the indexer requires both event and value anchors. Check all six tags and contiguous record ordinals.

> **7 — Permissionless no-capture · `rem:reviewer:item-permissionless`**
>
> Enumerate admission, delayed cycle, settlement, relabel, compaction, and clear. Confirm no hidden secret gate and no triggerer-selected economic recipient. Permissionless safety is recipient closure, not the absence of a signature by itself.

> **8 — Ledger and reader firewall · `rem:reviewer:item-ledger`**
>
> Confirm genesis clearing zero, manifest-hash-bound context, one-pass burn/clear merge, exact rational terms, reorg reprojection, over-claim retention, and the three independent differentials. Confirm historical residue is invisible to the attestation indexer and all monetary consumers while visible to the accounting auditor: the reference indexer (`[def:verification:reference-indexer]`) is the candidate shape O6 compares against.

> **Fastest route · `rem:reviewer:fastest-route`**
>
> Run O1–O9 first. Then hand-check what the oracle cannot establish: low-level gadget internals; model-to-script emission fidelity; signature, issuance, explicit-value, timelock, package-relay, and unspendable-output substrate semantics; deployment calibration; and the named reserve-asset settlement dependency. A green model is necessary and insufficient. Architecture release and deployment release remain separate.

> **Cross-document boundary · `rem:reviewer:cross-document`**
>
> This document realizes the specification but does not redefine it. The reviewer checks only that the realization discharges the cited interface and monetary-surface obligations without contradiction. Market net-cost discipline (`[A-thm:costliness:conservation-identity]`), whether the operator announces maturity (`[A-rem:maturity:announcement-timing]`), operator-deposit disambiguation (`[A-open:attestation:operator-disambiguation]`), the consumer's sampling half of burn timing (`[A-open:attestation:leverage-timing]`), and the consumer's wealth-propagation dynamics remain owned by their respective layers (`sec:trust:out-of-scope`).

> **One line · `rem:reviewer:one-line`**
>
> The Attestation Realization is a manifest-bound transition relation over a full-reserve, two-class receipt: closed assets and replayed roots authenticate state, exact canonical and open-flow partitions authenticate every movement, a kernel-derived certificate witnesses provenance, local transitions preserve one state-and-history invariant, burn remains share-nothing until permissionless clearing, and an independently reproducible checkpoint-bound ledger derives the identity-neutral, wealth-sensitive attestation map — while emitted script, substrate semantics, calibration, and reserve settlement remain separately evidenced deployment obligations.

<div align="right">∎</div>

---

## §17 Upward-citation index · `sec:realization:anchors`

The references below are one-directional upward cites into the *Attestation* specification, under the canonical rule (`rem:overview:cross-references`). The table is generated from body occurrences against the specification at v1.0.0: every body cite appears once, and every row is used. The distinct anchor set harvested here is the same set from which the manifest's `anchor_set_hash` is computed here and pinned at release, so the printed index and the pinned hash cannot diverge (`rem:overview:anchor-pin`). There is no reverse column.

*Specification anchor index · `tab:anchors:index`*

| specification anchor | Realized or referenced at |
|---|---|
| `[A-def:model:reserve-asset]` | (`sec:realization:overview`) |
| `[A-rem:model:instantiations]` | (`sec:realization:overview`) |
| `[A-def:model:reserve-pool]` | (`sec:architecture:soul-vault`) |
| `[A-def:model:ratefloor]` | ((`sec:realization:overview`), (`sec:realization:requirements`), (`sec:architecture:one-asset`)) |
| `[A-def:model:classes]` | ((`sec:realization:overview`), (`req:requirements:two-classes`), (`sec:architecture:one-asset`), (`sec:identity:structural-nonredeem`)) |
| `[A-def:model:supply]` | (`req:requirements:no-forgery-inflation`) |
| `[A-def:model:fee-and-split]` | ((`req:requirements:no-discretion`), (`sec:arithmetic:domains`)) |
| `[A-rem:model:calibration]` | ((`sec:arithmetic:domains`), (`sec:trust:out-of-scope`)) |
| `[A-def:model:normal-phase]` | (`rem:operations:fee-class`) |
| `[A-postc:model:genesis]` | (`sec:requirements:discharge-map`) |
| `[A-def:operations:deposit]` | ((`sec:requirements:discharge-map`), (`sec:architecture:deposit-pipe`)) |
| `[A-def:operations:burn]` | ((`sec:requirements:discharge-map`), (`sec:architecture:burn-ledger`)) |
| `[A-def:operations:redemption]` | (`sec:requirements:discharge-map`) |
| `[A-def:operations:transfer]` | (`sec:requirements:discharge-map`) |
| `[A-alg:operations:cycle-processing]` | ((`req:requirements:no-discretion`), (`sec:requirements:discharge-map`)) |
| `[A-def:interface:address-space]` | ((`sec:requirements:discharge-map`), (`sec:architecture:burn-ledger`)) |
| `[A-def:interface:attestation]` | ((`rem:overview:two-neutralities`), (`sec:requirements:discharge-map`)) |
| `[A-def:interface:net-live-share]` | (`sec:requirements:discharge-map`) |
| `[A-postc:interface:non-negativity]` | (`sec:requirements:discharge-map`) |
| `[A-postc:interface:monotonicity]` | ((`req:requirements:irrevocable-record`), (`sec:requirements:discharge-map`), (`sec:ledger:order`), (`sec:ledger:reorg`)) |
| `[A-postc:interface:auditability]` | ((`req:requirements:auditability`), (`sec:requirements:discharge-map`)) |
| `[A-thm:interface:instantaneous-costliness]` | ((`req:requirements:costliness`), (`sec:requirements:discharge-map`), (`sec:architecture:burn-ledger`)) |
| `[A-prop:interface:bootstrap-capacity]` | ((`req:requirements:self-custody`), (`sec:requirements:discharge-map`), (`sec:invariant:genesis`)) |
| `[A-prop:interface:conservative-valuation]` | ((`sec:requirements:discharge-map`), (`sec:architecture:burn-ledger`), (`rem:architecture:offchain-rate`), (`sec:ledger:order`), (`sec:invariant:terminals`)) |
| `[A-thm:invariants:floor-non-decrease]` | ((`req:requirements:rate-monotone`), (`sec:requirements:discharge-map`)) |
| `[A-prop:invariants:burn-increases-floor]` | ((`req:requirements:backing-integrity`), (`sec:operations:clear`)) |
| `[A-prop:invariants:redemption-preserves-floor]` | ((`req:requirements:backing-integrity`), (`sec:operations:redeem`)) |
| `[A-prop:invariants:issuance-preserves-floor]` | ((`req:requirements:no-forgery-inflation`), (`sec:operations:cycle`)) |
| `[A-prop:invariants:burn-order-residual]` | ((`sec:requirements:discharge-map`), (`sec:ledger:order`)) |
| `[A-lem:costliness:attestation-append-only]` | (`req:requirements:irrevocable-record`) |
| `[A-prop:costliness:splitting-invariance]` | ((`sec:operations:burn`), (`sec:ledger:authentication`)) |
| `[A-thm:costliness:conservation-identity]` | ((`sec:trust:out-of-scope`), (`rem:reviewer:cross-document`)) |
| `[A-postc:maturity:announcement]` | ((`sec:requirements:discharge-map`), (`sec:architecture:two-clocks`), (`rem:operations:lead-bounds`)) |
| `[A-def:maturity:conversion]` | ((`req:requirements:liveness`), (`sec:requirements:discharge-map`), (`sec:architecture:one-asset`)) |
| `[A-rem:maturity:announcement-timing]` | ((`rem:operations:lead-bounds`), (`sec:trust:out-of-scope`), (`rem:reviewer:cross-document`)) |
| `[A-rem:interface:clocks]` | (`sec:architecture:authorities`) |
| `[A-open:attestation:operator-disambiguation]` | ((`sec:trust:out-of-scope`), (`rem:reviewer:cross-document`)) |
| `[A-open:attestation:leverage-timing]` | ((`sec:trust:out-of-scope`), (`rem:reviewer:cross-document`)) |

---

## Appendix — Typed architecture manifest · `app:realization:architecture`

The complete `architecture.toml` is attached here **verbatim**. It is architecture schema 17 and is authoritative on every enumerated structural fact under (`rem:overview:register-authority`).

```toml
publication_status = "final"
architecture_schema_version = 17
realization_version = "0.3.0-dev"
semantic_hash_algorithm = "sha256-canonical-json-v2"
semantic_hash = "3974e4d7860d2cddad97c6ab63f5c358e303505a2529bf757d47db462285aa68"
behavioural_hash_algorithm = "sha256-canonical-json-behavioural-v3"
behavioural_hash = "756ea65ce3dc370e76ec70dc001231693facd58e53ebca315d549967f03cf206"

[architecture]
target_network = "liquid"

[architecture.specification]
version = "1.0.0"
anchor_set_hash = "b0cafaa48ac2ed388984f6a2d757a011a9c8654acf224f5f64516f26a3f7ee35"

[[architecture.assets]]
code = 1
id = "L-BTC"
class = "open"
role = "reserve-asset"
reissuable = false
destruction_operations = []

[[architecture.assets]]
code = 2
id = "U"
class = "closed"
role = "monetary-receipt"
reissuable = true
authority = "PACE"
issue_operation = "cycle"
destruction_operations = [
    "clear",
    "redeem",
    "settle-distribution",
]

[[architecture.assets]]
code = 3
id = "ENT"
class = "closed"
role = "deposit-entitlement"
reissuable = true
authority = "ENT_AUTH"
issue_operation = "admit-deposits"
destruction_operations = ["settle-distribution"]

[[architecture.assets]]
code = 4
id = "DIST_CTL"
class = "closed"
role = "distribution-control"
reissuable = true
authority = "DIST_AUTH"
issue_operation = "cycle"
destruction_operations = ["settle-distribution"]

[[architecture.assets]]
code = 5
id = "PID"
class = "closed"
role = "identity"
fixed_amount = 1
reissuable = false
destruction_operations = []

[[architecture.assets]]
code = 6
id = "PACE"
class = "closed"
role = "issuance-authority"
fixed_amount = 1
reissuable = false
destruction_operations = []

[[architecture.assets]]
code = 7
id = "ENT_AUTH"
class = "closed"
role = "issuance-authority"
fixed_amount = 1
reissuable = false
destruction_operations = []

[[architecture.assets]]
code = 8
id = "DIST_AUTH"
class = "closed"
role = "issuance-authority"
fixed_amount = 1
reissuable = false
destruction_operations = []

[[architecture.roots]]
code = 1
id = "STATE"
asset = "PID"
role = "state-identity"
fixed_amount = 1

[[architecture.roots]]
code = 2
id = "RESV"
asset = "L-BTC"
role = "active-reserve"

[[architecture.roots]]
code = 3
id = "PACE"
asset = "PACE"
role = "cadence-and-issuance"
fixed_amount = 1

[[architecture.roots]]
code = 4
id = "ENT_AUTH"
asset = "ENT_AUTH"
role = "entitlement-authority"
fixed_amount = 1

[[architecture.roots]]
code = 5
id = "DIST_AUTH"
asset = "DIST_AUTH"
role = "distribution-authority"
fixed_amount = 1

[[architecture.objects]]
code = 1
id = "STATE"
asset = "PID"
lifecycle = "constant-root"
accounting_domain = "none"
mutators = [
    "admit-deposits",
    "announce-maturity",
    "clear",
    "cycle",
    "receipt-relabel",
    "redeem",
]
witnesses = ["state-succession"]
consensus_value_authoritative = true

[[architecture.objects.allocators]]
kind = "genesis"

[[architecture.objects.deallocators]]
kind = "none"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "admit-deposits"
authorization = "covenant-companion"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "announce-maturity"
authorization = "covenant-companion"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "clear"
authorization = "covenant-companion"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "cycle"
authorization = "covenant-companion"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "receipt-relabel"
authorization = "covenant-companion"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "redeem"
authorization = "covenant-companion"

[[architecture.objects]]
code = 2
id = "RESV"
asset = "L-BTC"
lifecycle = "constant-root"
accounting_domain = "backing"
mutators = [
    "admit-deposits",
    "cycle",
    "redeem",
]
witnesses = ["resv-succession"]
consensus_value_authoritative = true

[[architecture.objects.allocators]]
kind = "genesis"

[[architecture.objects.deallocators]]
kind = "operation"
operation = "redeem"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "admit-deposits"
authorization = "covenant-companion"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "cycle"
authorization = "covenant-companion"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "redeem"
authorization = "covenant-companion"

[[architecture.objects]]
code = 3
id = "PACE"
asset = "PACE"
lifecycle = "constant-root"
accounting_domain = "none"
mutators = ["cycle"]
witnesses = [
    "canonical-delta",
    "native-fee-auction",
]
consensus_value_authoritative = true

[[architecture.objects.allocators]]
kind = "genesis"

[[architecture.objects.deallocators]]
kind = "none"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "cycle"
authorization = "covenant-companion"

[[architecture.objects]]
code = 4
id = "ENTITLEMENT_AUTHORITY"
asset = "ENT_AUTH"
lifecycle = "constant-root"
accounting_domain = "none"
mutators = ["admit-deposits"]
witnesses = ["canonical-delta"]
consensus_value_authoritative = true

[[architecture.objects.allocators]]
kind = "genesis"

[[architecture.objects.deallocators]]
kind = "none"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "admit-deposits"
authorization = "covenant-companion"

[[architecture.objects]]
code = 5
id = "DISTRIBUTION_AUTHORITY"
asset = "DIST_AUTH"
lifecycle = "constant-root"
accounting_domain = "none"
mutators = ["cycle"]
witnesses = ["canonical-delta"]
consensus_value_authoritative = true

[[architecture.objects.allocators]]
kind = "genesis"

[[architecture.objects.deallocators]]
kind = "none"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "cycle"
authorization = "covenant-companion"

[[architecture.objects]]
code = 6
id = "RECEIPT_L"
asset = "U"
lifecycle = "user-position"
accounting_domain = "receipt"
mutators = [
    "burn",
    "redeem",
    "transfer-live-receipts",
]
witnesses = [
    "canonical-delta",
    "receipt-class-closure",
    "receipt-owner-routing",
]
consensus_value_authoritative = true

[[architecture.objects.allocators]]
kind = "genesis"

[[architecture.objects.allocators]]
kind = "operation"
operation = "cycle"

[[architecture.objects.allocators]]
kind = "operation"
operation = "receipt-relabel"

[[architecture.objects.allocators]]
kind = "operation"
operation = "settle-distribution"

[[architecture.objects.allocators]]
kind = "operation"
operation = "transfer-live-receipts"

[[architecture.objects.deallocators]]
kind = "operation"
operation = "burn"

[[architecture.objects.deallocators]]
kind = "operation"
operation = "redeem"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "burn"
authorization = "input-owner"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "redeem"
authorization = "input-owner"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "transfer-live-receipts"
authorization = "input-owner"

[[architecture.objects]]
code = 7
id = "RECEIPT_T"
asset = "U"
lifecycle = "user-position"
accounting_domain = "receipt"
mutators = [
    "receipt-relabel",
    "transfer-time-locked-receipts",
]
witnesses = [
    "canonical-delta",
    "receipt-class-closure",
    "receipt-owner-routing",
]
consensus_value_authoritative = true

[[architecture.objects.allocators]]
kind = "genesis"

[[architecture.objects.allocators]]
kind = "operation"
operation = "cycle"

[[architecture.objects.allocators]]
kind = "operation"
operation = "settle-distribution"

[[architecture.objects.allocators]]
kind = "operation"
operation = "transfer-time-locked-receipts"

[[architecture.objects.deallocators]]
kind = "operation"
operation = "receipt-relabel"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "receipt-relabel"
authorization = "permissionless"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "transfer-time-locked-receipts"
authorization = "input-owner"

[[architecture.objects]]
code = 8
id = "DEPOSIT_REQUEST"
asset = "L-BTC"
lifecycle = "open-offer"
accounting_domain = "none"
mutators = []
witnesses = [
    "request-closure",
    "value-flow-closure",
]
consensus_value_authoritative = true

[[architecture.objects.allocators]]
kind = "external"

[[architecture.objects.allocators]]
kind = "operation"
operation = "create-request"

[[architecture.objects.deallocators]]
kind = "operation"
operation = "admit-deposits"

[[architecture.objects.deallocators]]
kind = "operation"
operation = "cancel-request"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "admit-deposits"
authorization = "permissionless"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "cancel-request"
authorization = "refund-key"

[[architecture.objects]]
code = 9
id = "DEPOSIT_ENTITLEMENT"
asset = "ENT"
lifecycle = "cycle-queue"
accounting_domain = "entitlement"
mutators = []
witnesses = [
    "distribution-closure",
    "entitlement-closure",
]
consensus_value_authoritative = true

[[architecture.objects.allocators]]
kind = "operation"
operation = "admit-deposits"

[[architecture.objects.deallocators]]
kind = "operation"
operation = "settle-distribution"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "settle-distribution"
authorization = "permissionless"

[[architecture.objects]]
code = 10
id = "DISTRIBUTION_CONTROL"
asset = "DIST_CTL"
lifecycle = "cycle-queue"
accounting_domain = "entitlement"
mutators = ["settle-distribution"]
witnesses = [
    "distribution-closure",
    "utxo-lifecycle",
]
consensus_value_authoritative = true

[[architecture.objects.allocators]]
kind = "operation"
operation = "cycle"

[[architecture.objects.deallocators]]
kind = "operation"
operation = "settle-distribution"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "settle-distribution"
authorization = "permissionless"

[[architecture.objects]]
code = 11
id = "DISTRIBUTION_VAULT"
asset = "U"
lifecycle = "cycle-queue"
accounting_domain = "receipt"
mutators = ["settle-distribution"]
witnesses = [
    "canonical-delta",
    "distribution-closure",
]
consensus_value_authoritative = true

[[architecture.objects.allocators]]
kind = "operation"
operation = "cycle"

[[architecture.objects.deallocators]]
kind = "operation"
operation = "settle-distribution"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "settle-distribution"
authorization = "permissionless"

[[architecture.objects]]
code = 12
id = "ASH"
asset = "U"
lifecycle = "backlog"
accounting_domain = "receipt"
mutators = [
    "clear",
    "compact-ash",
]
witnesses = [
    "ash-lineage",
    "canonical-delta",
    "utxo-lifecycle",
]
consensus_value_authoritative = true

[[architecture.objects.allocators]]
kind = "operation"
operation = "burn"

[[architecture.objects.allocators]]
kind = "operation"
operation = "clear"

[[architecture.objects.allocators]]
kind = "operation"
operation = "compact-ash"

[[architecture.objects.deallocators]]
kind = "operation"
operation = "clear"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "clear"
authorization = "permissionless"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "compact-ash"
authorization = "permissionless"

[[architecture.objects]]
code = 13
id = "PLAIN_LBTC"
asset = "L-BTC"
lifecycle = "external-wallet"
accounting_domain = "fee"
mutators = []
witnesses = ["value-flow-closure"]
consensus_value_authoritative = true

[[architecture.objects.allocators]]
kind = "external"

[[architecture.objects.deallocators]]
kind = "external-spend"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "announce-maturity"
authorization = "sponsor-owner"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "burn"
authorization = "sponsor-owner"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "cancel-request"
authorization = "sponsor-owner"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "clear"
authorization = "sponsor-owner"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "compact-ash"
authorization = "sponsor-owner"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "create-request"
authorization = "input-owner"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "cycle"
authorization = "sponsor-owner"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "receipt-relabel"
authorization = "sponsor-owner"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "redeem"
authorization = "sponsor-owner"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "settle-distribution"
authorization = "sponsor-owner"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "transfer-live-receipts"
authorization = "sponsor-owner"

[[architecture.objects.authorization_paths]]
kind = "operation"
operation = "transfer-time-locked-receipts"
authorization = "sponsor-owner"

[[architecture.objects.authorization_paths]]
kind = "external-spend"
authorization = "input-owner"

[[architecture.objects]]
code = 14
id = "CPFP_ANCHOR"
asset = "L-BTC"
lifecycle = "bounded-retained"
accounting_domain = "none"
mutators = []
witnesses = ["native-fee-auction"]
consensus_value_authoritative = true

[[architecture.objects.allocators]]
kind = "operation"
operation = "cycle"

[[architecture.objects.deallocators]]
kind = "external-spend"

[[architecture.objects.authorization_paths]]
kind = "external-spend"
authorization = "permissionless"

[[architecture.operations]]
code = 1
id = "create-request"
kind = "client-protocol"
authorization = "client-authorized"
open_flows = ["request-creation"]
reads = []
writes = []
witnesses = [
    "request-closure",
    "value-flow-closure",
]
value_flows = [
    "immutable-destination",
    "owner-consented",
]
bounds = ["FEE_SPONSOR_INPUT_MAX"]
roots = []
issuances = []
canonical_deltas = []
data_outputs = []

[[architecture.operations.inputs]]
object = "PLAIN_LBTC"
minimum = 1
authorization = "input-owner"

[architecture.operations.inputs.maximum]
kind = "bound"
bound = "FEE_SPONSOR_INPUT_MAX"

[[architecture.operations.outputs]]
object = "DEPOSIT_REQUEST"
minimum = 1

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "PLAIN_LBTC"
minimum = 0

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.projections]]
projection = "transition-certificate"
rule = "required"

[[architecture.operations]]
code = 2
id = "cancel-request"
kind = "covenant-branch"
authorization = "refund-key"
open_flows = [
    "fee-sponsor",
    "request-refund",
]
reads = []
writes = []
witnesses = [
    "request-closure",
    "value-flow-closure",
]
value_flows = [
    "immutable-destination",
    "owner-consented",
    "sponsor-envelope",
]
bounds = ["FEE_SPONSOR_INPUT_MAX"]
roots = []
issuances = []
canonical_deltas = []
data_outputs = []

[[architecture.operations.inputs]]
object = "DEPOSIT_REQUEST"
minimum = 1
authorization = "refund-key"

[architecture.operations.inputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.inputs]]
object = "PLAIN_LBTC"
minimum = 0
authorization = "sponsor-owner"

[architecture.operations.inputs.maximum]
kind = "bound"
bound = "FEE_SPONSOR_INPUT_MAX"

[[architecture.operations.outputs]]
object = "PLAIN_LBTC"
minimum = 1

[architecture.operations.outputs.maximum]
kind = "exact"
value = 2

[[architecture.operations.projections]]
projection = "transition-certificate"
rule = "required"

[[architecture.operations]]
code = 3
id = "admit-deposits"
kind = "covenant-branch"
authorization = "permissionless"
open_flows = ["deposit-admission"]
reads = []
writes = []
witnesses = [
    "canonical-delta",
    "entitlement-closure",
    "resv-succession",
    "state-succession",
    "value-flow-closure",
]
value_flows = [
    "immutable-destination",
    "preauthorized-service-budget",
]
bounds = ["ADMISSION_BATCH_MAX"]
data_outputs = []

[[architecture.operations.roots]]
root = "ENT_AUTH"
use_kind = "succession"

[[architecture.operations.roots]]
root = "RESV"
use_kind = "succession"

[[architecture.operations.roots]]
root = "STATE"
use_kind = "succession"

[[architecture.operations.issuances]]
asset = "ENT"
authority = "ENT_AUTH"
condition = "positive-admitted-principal"

[[architecture.operations.inputs]]
object = "DEPOSIT_REQUEST"
minimum = 1
authorization = "permissionless"

[architecture.operations.inputs.maximum]
kind = "bound"
bound = "ADMISSION_BATCH_MAX"

[[architecture.operations.inputs]]
object = "ENTITLEMENT_AUTHORITY"
minimum = 1
authorization = "covenant-companion"

[architecture.operations.inputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.inputs]]
object = "RESV"
minimum = 1
authorization = "covenant-companion"

[architecture.operations.inputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.inputs]]
object = "STATE"
minimum = 1
authorization = "covenant-companion"

[architecture.operations.inputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "DEPOSIT_ENTITLEMENT"
minimum = 1

[architecture.operations.outputs.maximum]
kind = "bound"
bound = "ADMISSION_BATCH_MAX"

[[architecture.operations.outputs]]
object = "ENTITLEMENT_AUTHORITY"
minimum = 1

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "PLAIN_LBTC"
minimum = 0

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "RESV"
minimum = 1

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "STATE"
minimum = 1

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.canonical_deltas]]
asset = "ENT"
kind = "issuance"
condition = "positive-admitted-principal"

[[architecture.operations.projections]]
projection = "transition-certificate"
rule = "required"

[[architecture.operations]]
code = 4
id = "cycle"
kind = "covenant-branch"
authorization = "cadence-band"
open_flows = [
    "fee-sponsor",
    "reserve-carry",
]
reads = [
    "cycle-issuance",
    "floor-phi",
]
writes = []
witnesses = [
    "canonical-delta",
    "distribution-closure",
    "floor-nondecrease",
    "native-fee-auction",
    "resv-succession",
    "state-succession",
    "value-flow-closure",
]
value_flows = [
    "immutable-destination",
    "sponsor-envelope",
]
bounds = ["FEE_SPONSOR_INPUT_MAX"]
data_outputs = []

[[architecture.operations.roots]]
root = "DIST_AUTH"
use_kind = "succession"

[[architecture.operations.roots]]
root = "PACE"
use_kind = "succession"

[[architecture.operations.roots]]
root = "RESV"
use_kind = "succession"

[[architecture.operations.roots]]
root = "STATE"
use_kind = "succession"

[[architecture.operations.issuances]]
asset = "DIST_CTL"
authority = "DIST_AUTH"
condition = "positive-cycle-principal"

[[architecture.operations.issuances]]
asset = "U"
authority = "PACE"
condition = "positive-cycle-issuance"

[[architecture.operations.inputs]]
object = "DISTRIBUTION_AUTHORITY"
minimum = 1
authorization = "covenant-companion"

[architecture.operations.inputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.inputs]]
object = "PACE"
minimum = 1
authorization = "covenant-companion"

[architecture.operations.inputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.inputs]]
object = "PLAIN_LBTC"
minimum = 0
authorization = "sponsor-owner"

[architecture.operations.inputs.maximum]
kind = "bound"
bound = "FEE_SPONSOR_INPUT_MAX"

[[architecture.operations.inputs]]
object = "RESV"
minimum = 1
authorization = "covenant-companion"

[architecture.operations.inputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.inputs]]
object = "STATE"
minimum = 1
authorization = "covenant-companion"

[architecture.operations.inputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "CPFP_ANCHOR"
minimum = 0

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "DISTRIBUTION_AUTHORITY"
minimum = 1

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "DISTRIBUTION_CONTROL"
minimum = 0

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "DISTRIBUTION_VAULT"
minimum = 0

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "PACE"
minimum = 1

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "PLAIN_LBTC"
minimum = 0

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "RECEIPT_L"
minimum = 0

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "RECEIPT_T"
minimum = 0

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "RESV"
minimum = 1

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "STATE"
minimum = 1

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.canonical_deltas]]
asset = "DIST_CTL"
kind = "issuance"
condition = "positive-cycle-principal"

[[architecture.operations.canonical_deltas]]
asset = "U"
kind = "issuance"
condition = "positive-cycle-issuance"

[[architecture.operations.projections]]
projection = "transition-certificate"
rule = "required"

[[architecture.operations]]
code = 5
id = "settle-distribution"
kind = "covenant-branch"
authorization = "permissionless"
open_flows = ["fee-sponsor"]
reads = []
writes = []
witnesses = [
    "canonical-delta",
    "distribution-closure",
    "entitlement-closure",
    "receipt-owner-routing",
    "utxo-lifecycle",
    "value-flow-closure",
]
value_flows = [
    "immutable-destination",
    "ownerless-terminal-sink",
    "sponsor-envelope",
]
bounds = [
    "FEE_SPONSOR_INPUT_MAX",
    "SETTLEMENT_BATCH_MAX",
]
roots = []
issuances = []

[[architecture.operations.inputs]]
object = "DEPOSIT_ENTITLEMENT"
minimum = 1
authorization = "permissionless"

[architecture.operations.inputs.maximum]
kind = "bound"
bound = "SETTLEMENT_BATCH_MAX"

[[architecture.operations.inputs]]
object = "DISTRIBUTION_CONTROL"
minimum = 1
authorization = "permissionless"

[architecture.operations.inputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.inputs]]
object = "DISTRIBUTION_VAULT"
minimum = 0
authorization = "permissionless"

[architecture.operations.inputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.inputs]]
object = "PLAIN_LBTC"
minimum = 0
authorization = "sponsor-owner"

[architecture.operations.inputs.maximum]
kind = "bound"
bound = "FEE_SPONSOR_INPUT_MAX"

[[architecture.operations.outputs]]
object = "DISTRIBUTION_CONTROL"
minimum = 0

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "DISTRIBUTION_VAULT"
minimum = 0

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "PLAIN_LBTC"
minimum = 0

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "RECEIPT_L"
minimum = 0

[architecture.operations.outputs.maximum]
kind = "bound"
bound = "SETTLEMENT_BATCH_MAX"

[[architecture.operations.outputs]]
object = "RECEIPT_T"
minimum = 0

[architecture.operations.outputs.maximum]
kind = "bound"
bound = "SETTLEMENT_BATCH_MAX"

[[architecture.operations.canonical_deltas]]
asset = "DIST_CTL"
kind = "destruction"
condition = "distribution-terminates"
destruction_tag = "tag-distribution-control-close"

[[architecture.operations.canonical_deltas]]
asset = "DIST_CTL"
kind = "lateral"
condition = "distribution-continues"

[[architecture.operations.canonical_deltas]]
asset = "ENT"
kind = "destruction"
condition = "always"
destruction_tag = "tag-entitlement"

[[architecture.operations.canonical_deltas]]
asset = "U"
kind = "destruction"
condition = "positive-distribution-residue"
destruction_tag = "tag-distribution-residue"

[[architecture.operations.canonical_deltas]]
asset = "U"
kind = "lateral"
condition = "positive-settlement-u-output"

[[architecture.operations.data_outputs]]
kind = "destruction"
tag = "tag-distribution-control-close"
asset = "DIST_CTL"
minimum = 1
condition = "distribution-terminates"

[architecture.operations.data_outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.data_outputs]]
kind = "destruction"
tag = "tag-distribution-residue"
asset = "U"
minimum = 1
condition = "positive-distribution-residue"

[architecture.operations.data_outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.data_outputs]]
kind = "destruction"
tag = "tag-entitlement"
asset = "ENT"
minimum = 1
condition = "always"

[architecture.operations.data_outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.projections]]
projection = "distribution-residue"
rule = "optional"

[[architecture.operations.projections]]
projection = "transition-certificate"
rule = "required"

[[architecture.operations]]
code = 6
id = "transfer-live-receipts"
kind = "covenant-branch"
authorization = "receipt-owners"
open_flows = ["fee-sponsor"]
reads = []
writes = []
witnesses = [
    "canonical-delta",
    "receipt-class-closure",
    "receipt-owner-routing",
    "value-flow-closure",
]
value_flows = [
    "owner-consented",
    "sponsor-envelope",
]
bounds = [
    "FEE_SPONSOR_INPUT_MAX",
    "TRANSFER_INPUT_MAX",
    "TRANSFER_OUTPUT_MAX",
]
roots = []
issuances = []
data_outputs = []

[[architecture.operations.inputs]]
object = "PLAIN_LBTC"
minimum = 0
authorization = "sponsor-owner"

[architecture.operations.inputs.maximum]
kind = "bound"
bound = "FEE_SPONSOR_INPUT_MAX"

[[architecture.operations.inputs]]
object = "RECEIPT_L"
minimum = 1
authorization = "input-owner"

[architecture.operations.inputs.maximum]
kind = "bound"
bound = "TRANSFER_INPUT_MAX"

[[architecture.operations.outputs]]
object = "PLAIN_LBTC"
minimum = 0

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "RECEIPT_L"
minimum = 1

[architecture.operations.outputs.maximum]
kind = "bound"
bound = "TRANSFER_OUTPUT_MAX"

[[architecture.operations.canonical_deltas]]
asset = "U"
kind = "lateral"
condition = "always"

[[architecture.operations.projections]]
projection = "transition-certificate"
rule = "required"

[[architecture.operations]]
code = 7
id = "transfer-time-locked-receipts"
kind = "covenant-branch"
authorization = "receipt-owners"
open_flows = ["fee-sponsor"]
reads = []
writes = []
witnesses = [
    "canonical-delta",
    "receipt-class-closure",
    "receipt-owner-routing",
    "value-flow-closure",
]
value_flows = [
    "owner-consented",
    "sponsor-envelope",
]
bounds = [
    "FEE_SPONSOR_INPUT_MAX",
    "TRANSFER_INPUT_MAX",
    "TRANSFER_OUTPUT_MAX",
]
roots = []
issuances = []
data_outputs = []

[[architecture.operations.inputs]]
object = "PLAIN_LBTC"
minimum = 0
authorization = "sponsor-owner"

[architecture.operations.inputs.maximum]
kind = "bound"
bound = "FEE_SPONSOR_INPUT_MAX"

[[architecture.operations.inputs]]
object = "RECEIPT_T"
minimum = 1
authorization = "input-owner"

[architecture.operations.inputs.maximum]
kind = "bound"
bound = "TRANSFER_INPUT_MAX"

[[architecture.operations.outputs]]
object = "PLAIN_LBTC"
minimum = 0

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "RECEIPT_T"
minimum = 1

[architecture.operations.outputs.maximum]
kind = "bound"
bound = "TRANSFER_OUTPUT_MAX"

[[architecture.operations.canonical_deltas]]
asset = "U"
kind = "lateral"
condition = "always"

[[architecture.operations.projections]]
projection = "transition-certificate"
rule = "required"

[[architecture.operations]]
code = 8
id = "redeem"
kind = "covenant-branch"
authorization = "receipt-owners"
open_flows = [
    "fee-sponsor",
    "redemption",
]
reads = [
    "floor-phi",
    "redemption-payout",
]
writes = []
witnesses = [
    "canonical-delta",
    "floor-nondecrease",
    "receipt-owner-routing",
    "resv-succession",
    "state-succession",
    "value-flow-closure",
]
value_flows = [
    "formula-bound-payout",
    "owner-consented",
    "ownerless-terminal-sink",
    "sponsor-envelope",
]
bounds = ["FEE_SPONSOR_INPUT_MAX"]
issuances = []

[[architecture.operations.roots]]
root = "RESV"
use_kind = "succession-or-termination"

[[architecture.operations.roots]]
root = "STATE"
use_kind = "succession"

[[architecture.operations.inputs]]
object = "PLAIN_LBTC"
minimum = 0
authorization = "sponsor-owner"

[architecture.operations.inputs.maximum]
kind = "bound"
bound = "FEE_SPONSOR_INPUT_MAX"

[[architecture.operations.inputs]]
object = "RECEIPT_L"
minimum = 1
authorization = "input-owner"

[architecture.operations.inputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.inputs]]
object = "RESV"
minimum = 1
authorization = "covenant-companion"

[architecture.operations.inputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.inputs]]
object = "STATE"
minimum = 1
authorization = "covenant-companion"

[architecture.operations.inputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "PLAIN_LBTC"
minimum = 1

[architecture.operations.outputs.maximum]
kind = "exact"
value = 2

[[architecture.operations.outputs]]
object = "RESV"
minimum = 0

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "STATE"
minimum = 1

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.canonical_deltas]]
asset = "U"
kind = "destruction"
condition = "always"
destruction_tag = "tag-redeem"

[[architecture.operations.data_outputs]]
kind = "destruction"
tag = "tag-redeem"
asset = "U"
minimum = 1
condition = "always"

[architecture.operations.data_outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.projections]]
projection = "transition-certificate"
rule = "required"

[[architecture.operations]]
code = 9
id = "receipt-relabel"
kind = "covenant-branch"
authorization = "permissionless"
open_flows = ["fee-sponsor"]
reads = []
writes = []
witnesses = [
    "canonical-delta",
    "receipt-class-closure",
    "receipt-owner-routing",
    "state-succession",
    "utxo-lifecycle",
    "value-flow-closure",
]
value_flows = [
    "immutable-destination",
    "sponsor-envelope",
]
bounds = [
    "FEE_SPONSOR_INPUT_MAX",
    "RELABEL_BATCH_MAX",
]
issuances = []
data_outputs = []

[[architecture.operations.roots]]
root = "STATE"
use_kind = "succession"

[[architecture.operations.inputs]]
object = "PLAIN_LBTC"
minimum = 0
authorization = "sponsor-owner"

[architecture.operations.inputs.maximum]
kind = "bound"
bound = "FEE_SPONSOR_INPUT_MAX"

[[architecture.operations.inputs]]
object = "RECEIPT_T"
minimum = 1
authorization = "permissionless"

[architecture.operations.inputs.maximum]
kind = "bound"
bound = "RELABEL_BATCH_MAX"

[[architecture.operations.inputs]]
object = "STATE"
minimum = 1
authorization = "covenant-companion"

[architecture.operations.inputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "PLAIN_LBTC"
minimum = 0

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "RECEIPT_L"
minimum = 1

[architecture.operations.outputs.maximum]
kind = "bound"
bound = "RELABEL_BATCH_MAX"

[[architecture.operations.outputs]]
object = "STATE"
minimum = 1

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.canonical_deltas]]
asset = "U"
kind = "lateral"
condition = "always"

[[architecture.operations.projections]]
projection = "transition-certificate"
rule = "required"

[[architecture.operations]]
code = 10
id = "burn"
kind = "covenant-branch"
authorization = "receipt-owners"
open_flows = ["fee-sponsor"]
reads = []
writes = []
witnesses = [
    "ash-lineage",
    "attestation-authentication",
    "canonical-delta",
    "receipt-class-closure",
    "receipt-owner-routing",
    "value-flow-closure",
]
value_flows = [
    "owner-consented",
    "ownerless-bound-sink",
    "sponsor-envelope",
]
bounds = [
    "BURN_CHANGE_MAX",
    "BURN_INPUT_MAX",
    "BURN_RECORD_MAX",
    "FEE_SPONSOR_INPUT_MAX",
]
roots = []
issuances = []

[[architecture.operations.inputs]]
object = "PLAIN_LBTC"
minimum = 0
authorization = "sponsor-owner"

[architecture.operations.inputs.maximum]
kind = "bound"
bound = "FEE_SPONSOR_INPUT_MAX"

[[architecture.operations.inputs]]
object = "RECEIPT_L"
minimum = 1
authorization = "input-owner"

[architecture.operations.inputs.maximum]
kind = "bound"
bound = "BURN_INPUT_MAX"

[[architecture.operations.outputs]]
object = "ASH"
minimum = 1

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "PLAIN_LBTC"
minimum = 0

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "RECEIPT_L"
minimum = 0

[architecture.operations.outputs.maximum]
kind = "bound"
bound = "BURN_CHANGE_MAX"

[[architecture.operations.canonical_deltas]]
asset = "U"
kind = "lateral"
condition = "always"

[[architecture.operations.data_outputs]]
kind = "burn-record"
tag = "tag-burn"
minimum = 0
condition = "always"

[architecture.operations.data_outputs.maximum]
kind = "bound"
bound = "BURN_RECORD_MAX"

[[architecture.operations.projections]]
projection = "burn-event"
rule = "required"

[[architecture.operations.projections]]
projection = "transition-certificate"
rule = "required"

[[architecture.operations]]
code = 11
id = "compact-ash"
kind = "covenant-branch"
authorization = "permissionless"
open_flows = ["fee-sponsor"]
reads = []
writes = []
witnesses = [
    "ash-lineage",
    "canonical-delta",
    "utxo-lifecycle",
    "value-flow-closure",
]
value_flows = [
    "ownerless-bound-movement",
    "sponsor-envelope",
]
bounds = [
    "ASH_BATCH_MAX",
    "FEE_SPONSOR_INPUT_MAX",
]
roots = []
issuances = []
data_outputs = []

[[architecture.operations.inputs]]
object = "ASH"
minimum = 2
authorization = "permissionless"

[architecture.operations.inputs.maximum]
kind = "bound"
bound = "ASH_BATCH_MAX"

[[architecture.operations.inputs]]
object = "PLAIN_LBTC"
minimum = 0
authorization = "sponsor-owner"

[architecture.operations.inputs.maximum]
kind = "bound"
bound = "FEE_SPONSOR_INPUT_MAX"

[[architecture.operations.outputs]]
object = "ASH"
minimum = 1

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "PLAIN_LBTC"
minimum = 0

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.canonical_deltas]]
asset = "U"
kind = "ownerless-lateral"
condition = "always"

[[architecture.operations.projections]]
projection = "transition-certificate"
rule = "required"

[[architecture.operations]]
code = 12
id = "clear"
kind = "covenant-branch"
authorization = "permissionless"
open_flows = ["fee-sponsor"]
reads = []
writes = []
witnesses = [
    "ash-lineage",
    "canonical-delta",
    "floor-nondecrease",
    "native-fee-auction",
    "state-succession",
    "value-flow-closure",
]
value_flows = [
    "ownerless-terminal-sink",
    "sponsor-envelope",
]
bounds = [
    "ASH_BATCH_MAX",
    "FEE_SPONSOR_INPUT_MAX",
]
issuances = []

[[architecture.operations.roots]]
root = "STATE"
use_kind = "succession"

[[architecture.operations.inputs]]
object = "ASH"
minimum = 1
authorization = "permissionless"

[architecture.operations.inputs.maximum]
kind = "bound"
bound = "ASH_BATCH_MAX"

[[architecture.operations.inputs]]
object = "PLAIN_LBTC"
minimum = 0
authorization = "sponsor-owner"

[architecture.operations.inputs.maximum]
kind = "bound"
bound = "FEE_SPONSOR_INPUT_MAX"

[[architecture.operations.inputs]]
object = "STATE"
minimum = 1
authorization = "covenant-companion"

[architecture.operations.inputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "ASH"
minimum = 0

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "PLAIN_LBTC"
minimum = 0

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "STATE"
minimum = 1

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.canonical_deltas]]
asset = "U"
kind = "destruction"
condition = "always"
destruction_tag = "tag-recon"

[[architecture.operations.canonical_deltas]]
asset = "U"
kind = "ownerless-lateral"
condition = "positive-ash-residual"

[[architecture.operations.data_outputs]]
kind = "destruction"
tag = "tag-recon"
asset = "U"
minimum = 1
condition = "always"

[architecture.operations.data_outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.projections]]
projection = "clear-event"
rule = "required"

[[architecture.operations.projections]]
projection = "transition-certificate"
rule = "required"

[[architecture.operations]]
code = 13
id = "announce-maturity"
kind = "covenant-branch"
authorization = "operator"
open_flows = ["fee-sponsor"]
reads = []
writes = []
witnesses = [
    "native-fee-auction",
    "state-succession",
    "value-flow-closure",
]
value_flows = [
    "owner-consented",
    "sponsor-envelope",
]
bounds = ["FEE_SPONSOR_INPUT_MAX"]
issuances = []
canonical_deltas = []
data_outputs = []

[[architecture.operations.roots]]
root = "STATE"
use_kind = "succession"

[[architecture.operations.inputs]]
object = "PLAIN_LBTC"
minimum = 0
authorization = "sponsor-owner"

[architecture.operations.inputs.maximum]
kind = "bound"
bound = "FEE_SPONSOR_INPUT_MAX"

[[architecture.operations.inputs]]
object = "STATE"
minimum = 1
authorization = "covenant-companion"

[architecture.operations.inputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "PLAIN_LBTC"
minimum = 0

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.outputs]]
object = "STATE"
minimum = 1

[architecture.operations.outputs.maximum]
kind = "exact"
value = 1

[[architecture.operations.projections]]
projection = "transition-certificate"
rule = "required"

[[architecture.quantities]]
code = 1
id = "floor-phi"
kind = "monetary"
reads = [
    "state.omega",
    "state.y-live",
    "state.y-time-locked",
]
writers = []

[[architecture.quantities.readers]]
kind = "operation"
operation = "cycle"

[[architecture.quantities.readers]]
kind = "operation"
operation = "redeem"

[[architecture.quantities.readers]]
kind = "external-auditor"

[[architecture.quantities]]
code = 2
id = "redemption-payout"
kind = "monetary"
reads = [
    "state.omega",
    "state.y-live",
    "state.y-time-locked",
]
writers = []

[[architecture.quantities.readers]]
kind = "operation"
operation = "redeem"

[[architecture.quantities]]
code = 3
id = "cycle-issuance"
kind = "monetary"
reads = [
    "state.omega",
    "state.q",
    "state.y-live",
    "state.y-time-locked",
]
writers = []

[[architecture.quantities.readers]]
kind = "operation"
operation = "cycle"

[[architecture.quantities]]
code = 4
id = "attestation-delta"
kind = "derived"
reads = [
    "burn-record.amount",
    "clear.omega",
    "clear.y",
]
writers = []

[[architecture.quantities.readers]]
kind = "attestation-indexer"

[[architecture.quantities]]
code = 5
id = "attestation-map"
kind = "interface"
reads = ["attestation.terms"]
writers = []

[[architecture.quantities.readers]]
kind = "attestation-indexer"

[[architecture.quantities.readers]]
kind = "external-auditor"

[[architecture.quantities.readers]]
kind = "consumer-formula"

[[architecture.quantities]]
code = 6
id = "historical-live-residue"
kind = "audit-only"
reads = ["history.residue-live"]
writers = []

[[architecture.quantities.readers]]
kind = "invariant-checker"

[[architecture.quantities.readers]]
kind = "external-auditor"

[[architecture.quantities]]
code = 7
id = "historical-time-locked-residue"
kind = "audit-only"
reads = ["history.residue-time-locked"]
writers = []

[[architecture.quantities.readers]]
kind = "invariant-checker"

[[architecture.quantities.readers]]
kind = "external-auditor"

[[architecture.quantities]]
code = 8
id = "receipt-accounting-audit"
kind = "audit-only"
reads = [
    "history.residue-live",
    "history.residue-time-locked",
    "state.y-live",
    "state.y-time-locked",
]
writers = []

[[architecture.quantities.readers]]
kind = "invariant-checker"

[[architecture.quantities.readers]]
kind = "external-auditor"

[[architecture.witnesses]]
code = 1
id = "canonical-delta"
semantic_tag = "lem:invariant:delta"

[[architecture.witnesses]]
code = 2
id = "ash-lineage"
semantic_tag = "inv:invariant:accounting"

[[architecture.witnesses]]
code = 3
id = "state-succession"
semantic_tag = "inv:invariant:succession"

[[architecture.witnesses]]
code = 4
id = "resv-succession"
semantic_tag = "inv:invariant:succession"

[[architecture.witnesses]]
code = 5
id = "receipt-owner-routing"
semantic_tag = "lem:invariant:flow"

[[architecture.witnesses]]
code = 6
id = "value-flow-closure"
semantic_tag = "lem:invariant:flow"

[[architecture.witnesses]]
code = 7
id = "request-closure"
semantic_tag = "lem:invariant:flow"

[[architecture.witnesses]]
code = 8
id = "entitlement-closure"
semantic_tag = "inv:invariant:escrow-receipts"

[[architecture.witnesses]]
code = 9
id = "distribution-closure"
semantic_tag = "inv:invariant:no-starve"

[[architecture.witnesses]]
code = 10
id = "receipt-class-closure"
semantic_tag = "inv:invariant:accounting"

[[architecture.witnesses]]
code = 11
id = "attestation-authentication"
semantic_tag = "sec:ledger:authentication"

[[architecture.witnesses]]
code = 12
id = "native-fee-auction"
semantic_tag = "res:trust:op"

[[architecture.witnesses]]
code = 13
id = "utxo-lifecycle"
semantic_tag = "inv:invariant:succession"

[[architecture.witnesses]]
code = 14
id = "floor-nondecrease"
semantic_tag = "lem:invariant:rate"

[[architecture.clauses]]
code = 1
id = "inv:invariant:identity"

[[architecture.clauses]]
code = 2
id = "inv:invariant:domains"

[[architecture.clauses]]
code = 3
id = "inv:invariant:rate-floor"

[[architecture.clauses]]
code = 4
id = "inv:invariant:no-trap"

[[architecture.clauses]]
code = 5
id = "inv:invariant:backing"

[[architecture.clauses]]
code = 6
id = "inv:invariant:no-starve"

[[architecture.clauses]]
code = 7
id = "inv:invariant:escrow-receipts"

[[architecture.clauses]]
code = 8
id = "inv:invariant:accounting"

[[architecture.clauses]]
code = 9
id = "inv:invariant:consensus-value"

[[architecture.clauses]]
code = 10
id = "inv:invariant:succession"

[[architecture.clauses]]
code = 11
id = "inv:invariant:maturity"

[[architecture.dependencies]]
code = 1
id = "native-asset-conservation"
verification_required = true
rationale = "Closed-asset scarcity and no-forgery depend on Elements native-asset conservation."

[[architecture.dependencies]]
code = 2
id = "issuance-introspection"
verification_required = true
rationale = "Receipt, entitlement, and control issuance are pinned through issuance introspection."

[[architecture.dependencies]]
code = 3
id = "explicit-value-introspection"
verification_required = true
rationale = "Explicit-arithmetic and public-delta seams read explicit consensus values."

[[architecture.dependencies]]
code = 4
id = "sighash-profile"
verification_required = true
rationale = "Owner signatures must commit all outputs; ANYONECANPAY remains a separate input-set policy."

[[architecture.dependencies]]
code = 5
id = "package-relay"
verification_required = true
rationale = "The maturity-cycle CPFP anchor depends on package selection."

[[architecture.dependencies]]
code = 6
id = "unspendable-utxo-exclusion"
verification_required = true
rationale = "Tagged destruction outputs must not remain in the current UTXO set."

[[architecture.dependencies]]
code = 7
id = "weld-enforcement"
verification_required = true
rationale = "Emitted script must enforce STATE/RESV and control/vault welds represented by the model."

[[architecture.dependencies]]
code = 8
id = "lbtc-settlement"
verification_required = true
rationale = "L-BTC settlement is provided by the Liquid functionary consortium."

[[architecture.dependencies]]
code = 9
id = "script-emission-fidelity"
verification_required = true
rationale = "The compiler must emit script matching the typed transition model."

[[architecture.dependencies]]
code = 10
id = "confidential-value-conservation"
verification_required = true
rationale = "Blind lateral movements discharge conservation through consensus CT balancing."

[[architecture.dependencies]]
code = 11
id = "value-commitment-equality"
verification_required = true
rationale = "Amount-blind relabel discharges per-object preservation by commitment equality."

[[architecture.dependencies]]
code = 12
id = "value-commitment-opening"
verification_required = false
rationale = "Reveal-at-spend paths, if implemented, authenticate openings on-chain."

[[architecture.decisions]]
code = 1
id = "no-onchain-attestation-accumulator"
status = "closed"
rationale = [
    "A root would not make the unbounded fold self-verifying.",
    "A shared root would serialize the burn hot path.",
    "Full-chain verifiers recompute attestation from public history.",
]

[[architecture.decisions]]
code = 2
id = "full-chain-attestation-verification"
status = "closed"
rationale = [
    "Ordinary users may trust their Layer-1 system.",
    "Verifier parties maintain canonical history.",
]

[[architecture.decisions]]
code = 3
id = "historical-residue-audit-only"
status = "closed"
rationale = [
    "Historical residue explains conservative recorded supply.",
    "No monetary, interface, covenant, or consumer formula reads residue.",
]

[[architecture.decisions]]
code = 4
id = "one-entitlement-per-request"
status = "closed"
rationale = [
    "Admission must not possess payout-affecting aggregation discretion.",
    "Integer floor allocation is not additive across merged requests.",
]

[[architecture.decisions]]
code = 5
id = "settlement-permissionless"
status = "closed"
rationale = [
    "Lost owner keys must not retain shared distribution state.",
    "Settlement outputs are owner-preserving and formula-bound.",
]

[[architecture.decisions]]
code = 6
id = "atomic-maturity-cycle"
status = "closed"
rationale = [
    "Maturity conversion occurs in the cycle reaching the maturity cycle index.",
    "No announced-but-unconverted limbo state is permitted.",
]

[[architecture.decisions]]
code = 7
id = "active-backing-cap"
status = "closed"
rationale = [
    "The cap does not limit cumulative historical deposit volume.",
    "The cap is enforced at genesis, admission, cycle, and invariant validation.",
    "The pool never holds more active L-BTC backing plus admitted escrow than the maximum L-BTC supply.",
]

[[architecture.bounds]]
code = 1
id = "ADMISSION_BATCH_MAX"
default_value = 32
requires_deployment_calibration = true

[[architecture.bounds]]
code = 2
id = "SETTLEMENT_BATCH_MAX"
default_value = 32
requires_deployment_calibration = true

[[architecture.bounds]]
code = 3
id = "RELABEL_BATCH_MAX"
default_value = 32
requires_deployment_calibration = true

[[architecture.bounds]]
code = 4
id = "ASH_BATCH_MAX"
default_value = 64
requires_deployment_calibration = true

[[architecture.bounds]]
code = 5
id = "BURN_INPUT_MAX"
default_value = 64
requires_deployment_calibration = true

[[architecture.bounds]]
code = 6
id = "BURN_CHANGE_MAX"
default_value = 32
requires_deployment_calibration = true

[[architecture.bounds]]
code = 7
id = "BURN_RECORD_MAX"
default_value = 64
requires_deployment_calibration = true

[[architecture.bounds]]
code = 8
id = "TRANSFER_INPUT_MAX"
default_value = 64
requires_deployment_calibration = true

[[architecture.bounds]]
code = 9
id = "TRANSFER_OUTPUT_MAX"
default_value = 64
requires_deployment_calibration = true

[[architecture.bounds]]
code = 10
id = "FEE_SPONSOR_INPUT_MAX"
default_value = 16
requires_deployment_calibration = true

[[architecture.amount_limits]]
code = 1
id = "ACTIVE_BACKING_MAX"
asset = "L-BTC"
value = 2100000000000000
unit = "lbtc-atomic-unit"
rationale = "Maximum simultaneous settled reserve plus admitted deposit escrow."

[[architecture.tags]]
code = 1
id = "tag-burn"
participates_in_attestation = true

[[architecture.tags]]
code = 2
id = "tag-recon"
participates_in_attestation = false

[[architecture.tags]]
code = 3
id = "tag-redeem"
participates_in_attestation = false

[[architecture.tags]]
code = 4
id = "tag-entitlement"
participates_in_attestation = false

[[architecture.tags]]
code = 5
id = "tag-distribution-control-close"
participates_in_attestation = false

[[architecture.tags]]
code = 6
id = "tag-distribution-residue"
participates_in_attestation = false

[[architecture.input_authorization_evidence]]
input_authorization = "covenant-companion"
model_evidence = "model-branch-shape"
compiler_evidence = "compiler-covenant-predicate"
deployment_evidence = "deployment-script-semantics"

[[architecture.input_authorization_evidence]]
input_authorization = "input-owner"
model_evidence = "model-signer-set"
compiler_evidence = "compiler-checksig"
deployment_evidence = "deployment-sighash"

[[architecture.input_authorization_evidence]]
input_authorization = "refund-key"
model_evidence = "model-signer-set"
compiler_evidence = "compiler-checksig"
deployment_evidence = "deployment-sighash"

[[architecture.input_authorization_evidence]]
input_authorization = "sponsor-owner"
model_evidence = "model-signer-set"
compiler_evidence = "compiler-checksig"
deployment_evidence = "deployment-sighash"

[[architecture.input_authorization_evidence]]
input_authorization = "permissionless"
model_evidence = "none-permissionless"
compiler_evidence = "none-permissionless"
deployment_evidence = "none-permissionless"

[[architecture.operation_authorization_evidence]]
permission_class = "client-authorized"
model_evidence = "model-signer-set"
compiler_evidence = "compiler-checksig"
deployment_evidence = "deployment-sighash"

[[architecture.operation_authorization_evidence]]
permission_class = "refund-key"
model_evidence = "model-signer-set"
compiler_evidence = "compiler-checksig"
deployment_evidence = "deployment-sighash"

[[architecture.operation_authorization_evidence]]
permission_class = "permissionless"
model_evidence = "none-permissionless"
compiler_evidence = "none-permissionless"
deployment_evidence = "none-permissionless"

[[architecture.operation_authorization_evidence]]
permission_class = "cadence-band"
model_evidence = "model-cadence-band"
compiler_evidence = "compiler-cadence-leaves"
deployment_evidence = "deployment-csv-semantics"

[[architecture.operation_authorization_evidence]]
permission_class = "receipt-owners"
model_evidence = "model-signer-set"
compiler_evidence = "compiler-checksig"
deployment_evidence = "deployment-sighash"

[[architecture.operation_authorization_evidence]]
permission_class = "operator"
model_evidence = "model-signer-set"
compiler_evidence = "compiler-checksig"
deployment_evidence = "deployment-sighash"
```

The publication pipeline MUST verify that the appended bytes parse as the supported `PublishedArchitecture` envelope, that the embedded semantic hash verifies over the canonical architecture body, that the embedded behavioural hash verifies over the behavioural arrays (`pin:pins:denotation`), and that the body is byte-for-byte the generated artifact selected for the release (`sec:pins:codegen`).

<div align="right">∎</div>
