# Research Question: Exact Wide Floor Arithmetic in Elements Tapscript

> **Status:** PROTOTYPE REQUIRED / MEASUREMENT REQUIRED
> **Blocks:** production redemption arithmetic; cycle issuance; settlement
> allocation; final wide-arithmetic target capability; target resource formulas
> for operations using `floor_mul_div`; release evidence for the wide arithmetic
> pattern
> **Does not directly block:** admission's own principal/cap arithmetic, which
> uses checked addition and exact issuance rather than `floor_mul_div`; it does
> block the later cycle that processes admitted `Q`
> **Affected packages:** `realization`, `compiler`, `target-elements`,
> `tapscript`, `transaction`, `vectors`, `release`; the linker consumes the
> accepted pattern's resource and witness artifacts
> **Depends on decisions:**
> [D001](../decisions/001-typed-rust-source.md),
> [D002](../decisions/002-realization-layer.md),
> [D003](../decisions/003-tapscript-first.md),
> [D004](../decisions/004-translation-validation.md),
> [D005](../decisions/005-value-representation.md),
> [D006](../decisions/006-transaction-abi.md)
> **Related normative constraints:** checked protocol amounts below `2^51`;
> exact floor arithmetic; witnessed quotient verification; fail-closed overflow;
> pool-favouring rounding; economy-determining formulas; target arithmetic
> evidence; resource calibration in
> `docs/attestation/realization.md`
> **Expected decision output:** an accepted or rejected Elements tapscript
> wide-floor proof pattern, including operand representation, witness schema,
> limb base, normalization/carry/borrow rules, target stack contract, resource
> formula, permanent vector family, and formal-evidence status
> **Machine-consumed by the toolchain:** no

---

## 1. Question

Which exact Elements tapscript construction can prove:

```text
q = floor(a * b / d)
```

for nonnegative protocol amounts satisfying:

```text
0 <= a < 2^51
0 <= b < 2^51
0 <  d < 2^51
0 <= q < 2^51
```

while also:

1. enforcing the exact same floor relation as the target-independent
   realization and executable model;
2. rejecting a quotient one below or one above the true floor;
3. rejecting zero divisor, malformed values, negative values, overflow, and
   noncanonical witness representations;
4. operating entirely within the exact pinned Elements target's signed 64-bit,
   stack, witness, crypto, script, transaction-weight, and policy limits;
5. exposing a deterministic typed witness ABI;
6. remaining suitable for redemption, cycle issuance, and per-entitlement
   settlement;
7. producing a resource formula that agrees with target-native measurement;
8. supporting relation-indexed positive and negative vectors;
9. permitting a clearly scoped hand audit and, if practical, a machine-checked
   bit-vector equivalence proof?

The leading proof relation is quotient/remainder verification:

```text
a * b = q * d + r
0 <= r < d
```

where `r` is an additional witness.

This relation is mathematically equivalent to:

```text
q * d <= a * b < (q + 1) * d
```

and therefore proves the exact floor quotient.

The target construction must establish the equality and remainder bound without
performing an overflowing native 64-bit multiplication.

---

## 2. Why the answer matters

### 2.1 Protocol products exceed signed 64-bit range

Every semantic amount is bounded below `2^51`.

A product of two protocol amounts can reach nearly:

```text
2^102
```

The target's signed 64-bit multiplication cannot directly represent that
product.

Using a native multiply and relying on a success flag would reject valid
protocol inputs whose product is larger than the signed 64-bit range.

### 2.2 The relation determines monetary outputs

Wide floor arithmetic appears in economy-determining formulas.

#### Cycle issuance

```text
ΔY = floor(Q * Y / Ω)
```

#### Redemption payout

```text
p = floor(x * Ω / Y)
```

#### Settlement class draw

For entitlement principal `δ`, class allocation `D_c`, and cycle principal
`Q_k`:

```text
m_c = floor(δ * D_c / Q_k)
```

A quotient error can:

- overissue receipts;
- underissue receipts;
- overpay redemption;
- underpay redemption;
- starve later settlement claims;
- redirect rounding residue;
- violate floor monotonicity;
- break accounting;
- create numeric discretion.

The arithmetic pattern is therefore a core safety boundary rather than an
optimization detail.

### 2.3 Lower-bound-only verification is insufficient

Checking only:

```text
q * d <= a * b
```

allows any quotient below the true floor.

A spender could select a smaller quotient and redirect the difference through
another output or state field.

The target must establish a unique quotient.

### 2.4 Host arithmetic is not target proof

The transaction builder can compute the correct quotient with Rust wide
arithmetic.

That helps produce a witness. It does not prove to the target that the witness
is correct.

The target program must independently verify the exact relation.

### 2.5 Settlement multiplies the cost

Redemption and cycle use one principal wide quotient each.

Settlement applies the floor per entitlement and per class.

A settlement batch may therefore execute many wide arithmetic proofs in one
transaction.

A pattern that is correct but too expensive for even a small settlement batch
cannot support the selected deployment.

### 2.6 The answer affects several package interfaces

The selected construction affects:

- realization proof-alternative declarations;
- compiler target requirements;
- target-elements capability declarations;
- tapscript pattern types;
- stack scheduling;
- witness ABI;
- transaction proof generation;
- relation vectors;
- resource formulas;
- calibrated settlement bounds;
- release evidence.

The design must be prototyped before those interfaces freeze.

---

## 3. Existing constraints

### 3.1 Semantic amount domain

The executable model defines:

```text
Sat < 2^51
```

The target-independent realization must preserve this checked domain.

All arithmetic is nonnegative at the semantic level.

The target uses signed 64-bit arithmetic opcodes, so the backend must:

- require exact 8-byte operand width where the target does;
- prove or enforce nonnegativity;
- avoid crossing `2^63 - 1`;
- verify every arithmetic success result;
- reject negative target encodings where semantic values are nonnegative.

### 3.2 Result domain

The semantic result is also a protocol amount:

```text
0 <= q < 2^51
```

A generic mathematical quotient can exceed `2^51` when:

```text
d
```

is small.

The target relation must reject such a result even if the mathematical floor
exists.

For the intended protocol call sites, state and operation constraints should
keep the result in domain. That fact must be represented and tested, not
assumed informally.

### 3.3 Divisor domain

The divisor must satisfy:

```text
d > 0
```

A zero divisor rejects.

The target proof must not rely solely on target division failure if the selected
wide relation no longer executes native division.

### 3.4 Exact floor semantics

The target relation must establish exactly:

```text
q = floor(a * b / d)
```

It may use an equivalent witness relation such as quotient/remainder.

Using an equivalent proof is acceptable only if:

- the target-independent relation is unchanged;
- mathematical equivalence is documented and tested;
- every target witness component is constrained;
- the resulting target pattern is accepted by implementation decision.

### 3.5 Initial representation policy

The first wide arithmetic implementation is expected to operate on:

- explicit semantic amount inputs; or
- publicly authenticated values converted to exact fixed-width target
  operands.

It does not need to perform arithmetic directly over hidden commitments.

A confidential object may require authenticated opening or normalization before
using this pattern.

Closed protocol asset identity remains explicit under D005.

### 3.6 Exact target semantics

The pattern may rely only on typed target capabilities supplied by
`target-elements`.

Likely primitives include:

- 8-byte signed addition;
- subtraction;
- multiplication;
- division/remainder;
- comparisons;
- script-number/fixed-width conversion;
- byte operations;
- stack manipulation;
- conditionals;
- target success flags.

The exact operand/result stack order must match the pinned target.

### 3.7 No general loop assumption

The pattern must use fixed, statically emitted limb operations.

No general script loop is assumed.

### 3.8 Witness ABI

Every witness component must have:

- typed role;
- exact width;
- canonical encoding;
- semantic domain;
- secrecy/public status;
- availability;
- source relation;
- malformed-input behavior.

For permissionless operations such as settlement, arithmetic witnesses must be
computable from public facts.

### 3.9 Fail-closed behavior

Any:

- malformed width;
- negative value;
- decomposition mismatch;
- carry overflow;
- borrow mismatch;
- quotient mismatch;
- remainder mismatch;
- divisor zero;
- target arithmetic failure;
- stack mismatch

must reject.

---

## 4. Definitions and terminology

### 4.1 Semantic floor relation

Define:

```text
FloorMulDiv(a, b, d, q)
```

to mean:

```text
0 <= a,b,q < 2^51
0 < d < 2^51
q = floor(a*b/d)
```

### 4.2 Quotient/remainder witness

The constructor supplies:

```text
q
r
```

such that:

```text
a*b = q*d + r
0 <= r < d
```

Because `d > 0`, this uniquely determines:

```text
q = floor(a*b/d)
```

### 4.3 Limb base

A limb representation uses base:

```text
B = 2^w
```

For the leading candidate:

```text
w = 26
B = 2^26
```

A value below `2^51` decomposes into two limbs:

```text
a = a_0 + a_1*B
```

with:

```text
0 <= a_0 < 2^26
0 <= a_1 < 2^25
```

The product fits four 26-bit limbs because:

```text
a*b < 2^102 < 2^104 = B^4
```

### 4.4 Canonical decomposition

A decomposition is canonical when:

```text
a_0 = a mod B
a_1 = floor(a/B)
```

and both limb bounds hold.

A witness-supplied decomposition must be checked by recomposition or equivalent
target arithmetic.

### 4.5 Normalized product limbs

A normalized product representation is:

```text
p = p_0 + p_1*B + p_2*B^2 + p_3*B^3
```

with:

```text
0 <= p_i < B
```

for every limb.

### 4.6 Carry

A carry transfers high bits from one coefficient to the next:

```text
coefficient_i = limb_i + carry_{i+1}*B
```

The target must constrain each carry's exact value and domain.

### 4.7 Borrow

A borrow is used for multi-limb subtraction/comparison.

The target must constrain each borrow to its allowed small domain, usually:

```text
0 or 1
```

or use another exact comparison relation.

### 4.8 Pattern-level versus operation-level result

A pattern-level result proves:

```text
FloorMulDiv(a,b,d,q)
```

for one set of operands.

An operation-level result proves that the pattern receives the correct semantic
operands and that `q` feeds the correct state/output relation.

Both are required.

---

## 5. Required properties

A production wide-floor pattern must satisfy all of the following.

### 5.1 Mathematical exactness

For every valid semantic input:

```text
accept iff q = floor(a*b/d)
```

under the selected witness schema.

No quotient below or above the floor may pass.

### 5.2 Result-domain enforcement

The pattern must enforce:

```text
0 <= q < 2^51
```

even when the unconstrained mathematical quotient is larger.

### 5.3 Remainder-domain enforcement

The pattern must enforce:

```text
0 <= r < d
```

and:

```text
r < 2^51
```

where the latter follows from `r < d < 2^51` but should remain explicit in the
typed proof.

### 5.4 Exact product equality

The pattern must establish full-width equality:

```text
a*b = q*d + r
```

No high limb may be omitted.

No aggregate checksum or truncated low-word equality is sufficient.

### 5.5 Canonical operands

The target input values and witnesses must have exactly one accepted encoding
under the selected target ABI, or accepted alternative encodings must be
explicit and semantically identical.

### 5.6 No signed-domain ambiguity

Negative fixed-width values reject.

Signed comparison behavior must not accept a large unsigned value interpreted
as negative.

### 5.7 Target arithmetic success

Every target arithmetic opcode capable of overflow/failure must have its
success result checked.

The pattern must account for failure-path stack behavior exactly.

### 5.8 Constructibility

For every operation using public operands, any authorized or permissionless
constructor can compute:

```text
q
r
```

off-chain from public facts.

No owner/operator secret is needed merely to produce the arithmetic witness.

### 5.9 Deterministic witness generation

Given identical semantic operands:

```text
q
r
```

and any additional canonical limb/carry witness are deterministic.

The first-party builder must not choose arbitrary equivalent carry
representations.

### 5.10 Target feasibility

The pattern must fit:

- stack element size;
- stack plus altstack count;
- initial witness item limit/policy;
- script/program limits;
- transaction weight;
- operation-specific complete-transaction limits;
- target standardness required by deployment.

### 5.11 Evidence and identity

The pattern must have:

- stable pattern identity;
- target identity;
- exact script/program identity;
- witness schema identity;
- resource formula;
- reference evaluator;
- positive/negative vectors;
- target-native report;
- formal evidence identity if adopted.

---

## 6. Call sites and operand mapping

### 6.1 Redemption

```text
a = receipt amount x
b = reserve Ω
d = total supply Y
q = payout p
```

Required semantic facts:

- `Y > 0`;
- `x <= Y_L`;
- result in `Sat` domain;
- payout feeds formula-bound owner output and reserve decrement.

### 6.2 Cycle issuance

```text
a = queued principal Q
b = total supply Y
d = reserve Ω
q = issued supply ΔY
```

Required semantic facts:

- operational pool;
- `Ω > 0`;
- `Y <= Ω`;
- `Q` within active-backing cap;
- result in `Sat` domain;
- issuance destinations exhaust `ΔY`.

### 6.3 Settlement

For each entitlement and class:

```text
a = entitlement principal δ
b = original class allocation D_c
d = original cycle principal Q_k
q = class draw m_c
```

Required semantic facts:

- `Q_k > 0`;
- `δ <= remaining principal`;
- `D_c` in domain;
- per-entitlement flooring occurs before aggregation;
- result feeds fixed owner/class receipt output;
- repeated proofs fit the calibrated batch.

### 6.4 Narrow-ratio distinction

The following formulas may use a separate narrow target pattern when one
operand is a small fixed ratio:

```text
live/time-locked issuance split
operator fee split
```

They are not the primary subject of this research note.

The backend should not force narrow ratio arithmetic through the wide pattern
unless measurement and simplicity justify it.

---

## 7. Candidate approaches

No candidate is accepted until the prototype and decision process complete.

---

### Candidate A — Quotient/remainder equality with script-derived limbs

#### Relation

Witness supplies:

```text
q
r
```

The target script derives canonical limbs for:

```text
a
b
q
d
r
```

using exact division/remainder by:

```text
B = 2^26
```

It computes normalized wide products:

```text
a*b
q*d
```

adds `r` to `q*d`, and compares all normalized limbs.

It separately checks:

```text
0 <= r < d
0 <= q < 2^51
d > 0
```

#### Advantages

- witness is small;
- limb decomposition is derived, not trusted;
- deterministic canonical decomposition;
- equality plus remainder bound proves exact floor;
- fewer wide products than direct sandwich;
- target relation is clear.

#### Risks

- many stack operations;
- target DIV64 result ordering/failure behavior complicates decomposition;
- storing two products and inputs may create stack pressure;
- carry normalization must be exact;
- repeated settlement proofs may be expensive;
- script size may be large.

#### Current status

Leading candidate.

---

### Candidate B — Quotient/remainder equality with witness-supplied limbs and carries

#### Relation

Witness supplies:

- `q`;
- `r`;
- operand limbs;
- product limbs;
- carry values.

The script verifies:

- every operand recomposes exactly;
- every limb/carry is in range;
- product coefficients and carries are correct;
- equality and remainder bound hold.

#### Advantages

- may reduce script stack manipulation;
- expensive decomposition or carry calculation moves off-chain;
- target script verifies only local equations;
- witness computation remains public.

#### Risks

- much larger witness;
- many additional domain checks;
- more malformed-witness surface;
- witness/carry schema may become complex;
- incorrect or underconstrained carry equations can admit false products;
- repeated settlement proofs increase witness size significantly.

#### Current status

Viable comparison candidate.

---

### Candidate C — Direct sandwich verification

#### Relation

Verify:

```text
q*d <= a*b
a*b < (q+1)*d
```

using wide products and comparisons.

#### Advantages

- directly mirrors the realization document's sandwich form;
- no remainder witness;
- mathematical condition is familiar.

#### Risks

- requires at least:
  - `a*b`;
  - `q*d`;
  - `(q+1)*d`;
- requires wide comparisons;
- requires checked `q+1`;
- likely more target work than quotient/remainder;
- greater stack/resource cost.

#### Current status

Reference candidate for comparison. Expected to be more expensive.

---

### Candidate D — Witness full product and quotient relation

#### Relation

Witness supplies full product limbs for:

```text
N = a*b
```

and proves:

```text
N = q*d+r
0 <= r<d
```

The script verifies multiplication and quotient relation separately.

#### Advantages

- can reuse product limbs;
- may simplify operation-specific integration;
- clear decomposition between multiplication and division proof.

#### Risks

- larger witness;
- still requires complete multiplication verification;
- still requires second multiplication `q*d`;
- may not improve stack/resource use compared with Candidate B.

#### Current status

Variant to evaluate if stack scheduling benefits.

---

### Candidate E — Repeated division or bitwise long division in script

#### Relation

Compute the quotient directly from the wide numerator through a statically
unrolled long-division algorithm.

#### Advantages

- no quotient/remainder witness trust;
- target derives quotient directly.

#### Risks

- very large script;
- many comparison/subtraction steps;
- high stack/resource cost;
- difficult hand audit;
- poor settlement scaling.

#### Current status

Expected to be rejected for the initial tapscript backend unless measurements
show an unexpected advantage.

---

### Candidate F — Target-specific cryptographic proof of arithmetic

#### Relation

Use a proof system or target primitive to prove the wide arithmetic relation
without explicit limb verification.

#### Advantages

- potentially smaller target program;
- may preserve private values;
- may generalize to more arithmetic.

#### Risks

- no currently approved proof pattern;
- target capability and witness construction uncertain;
- may require new cryptographic assumptions;
- significant implementation scope;
- permissionless witness availability may fail.

#### Current status

Not an initial Elements candidate. May be relevant to a future backend.

---

### Candidate G — Simplicity backend arithmetic

#### Relation

Use Simplicity expressions/jets to prove the same semantic floor relation.

#### Advantages

- potentially stronger formal semantics;
- may avoid tapscript stack choreography;
- possible jet support.

#### Risks

- second backend is parked;
- does not solve the first tapscript deployment;
- target and resource details remain unpinned.

#### Current status

Future alternative, not a substitute for this research.

---

## 8. Leading limb candidate

This section defines the Candidate-A/B limb arithmetic to prototype.

It is not yet production design.

### 8.1 Base

Use:

```text
B = 2^26
```

Reasons:

- every semantic amount below `2^51` fits in two limbs;
- low limbs are below `2^26`;
- high limbs are below `2^25`;
- individual partial products fit signed 64-bit;
- four limbs cover products below `2^104`;
- two unused high bits remain above the maximum `2^102` product.

The prototype should compare nearby bases such as `2^25` or `2^27` only if
measurement indicates a meaningful advantage.

### 8.2 Two-limb decomposition

For:

```text
x < 2^51
```

derive:

```text
x_0 = x mod B
x_1 = floor(x/B)
```

and enforce:

```text
0 <= x_0 < B
0 <= x_1 < 2^25
```

Recomposition:

```text
x = x_0 + x_1*B
```

must hold exactly in signed 64-bit range:

```text
x_1*B < 2^51
```

### 8.3 Raw multiplication coefficients

For:

```text
a = a_0 + a_1*B
b = b_0 + b_1*B
```

compute:

```text
c_0 = a_0*b_0
c_1 = a_0*b_1 + a_1*b_0
c_2 = a_1*b_1
```

Bounds:

```text
c_0 < 2^52
a_0*b_1 < 2^51
a_1*b_0 < 2^51
c_1 < 2^52
c_2 < 2^50
```

All are below signed 64-bit maximum with substantial headroom.

Every addition/multiplication success flag remains mandatory.

### 8.4 Carry normalization

Normalize:

```text
p_0 = c_0 mod B
carry_1 = floor(c_0/B)

t_1 = c_1 + carry_1
p_1 = t_1 mod B
carry_2 = floor(t_1/B)

t_2 = c_2 + carry_2
p_2 = t_2 mod B
p_3 = floor(t_2/B)
```

Enforce:

```text
0 <= p_i < B
```

and derive bounds for each temporary and carry before writing target code.

The prototype report must include exact symbolic maximum values.

### 8.5 Product representation

The normalized product is:

```text
P(a,b) = [p_0, p_1, p_2, p_3]
```

in little-limb order.

The ABI/report must state limb order explicitly.

### 8.6 Adding remainder

Compute:

```text
P(q,d) + r
```

where `r` has two limbs:

```text
r = r_0 + r_1*B
```

Add with carry across all four limbs.

The result must be normalized and compared limb-by-limb with:

```text
P(a,b)
```

### 8.7 Equality

Require exact equality of all four limbs.

A low-limb-only comparison is invalid.

### 8.8 Remainder comparison

Verify:

```text
r < d
```

Options include:

1. compare original 8-byte values with target signed 64-bit comparison after
   enforcing nonnegative `<2^51`;
2. compare two-limb representations.

The first is likely simpler and should be preferred if exact target semantics
support it.

### 8.9 Divisor positivity

Verify:

```text
d > 0
```

using target signed comparison over validated nonnegative 8-byte values.

### 8.10 Result domain

Verify:

```text
q < 2^51
```

and nonnegative.

### 8.11 Operand domain

Verify every operand:

```text
a,b,d,q,r
```

has:

- exact target width;
- nonnegative signed interpretation;
- `<2^51` where required.

The target source values `a,b,d` may already be protocol-domain checked by
other relation carriers. The arithmetic pattern must state whether it relies
on those checks or duplicates them.

For reusable pattern safety, prefer explicit preconditions represented in the
typed pattern contract even when the caller proves them elsewhere.

---

## 9. Stack and witness design

### 9.1 Minimum witness

Candidate A:

```text
q : 8-byte LE nonnegative amount
r : 8-byte LE nonnegative amount
```

Additional target-program/context witnesses may exist outside the arithmetic
pattern.

### 9.2 Expanded witness

Candidate B may add:

- operand limbs;
- product limbs;
- carry values.

Every item requires:

- fixed encoding;
- domain;
- canonicality;
- source relation;
- public availability;
- malformed vectors.

### 9.3 Witness availability

For all initial call sites:

- semantic operands are public or publicly opened under the selected profile;
- any party can compute `q` and `r`;
- no private owner/operator secret is needed merely for arithmetic.

This is especially important for permissionless settlement and delayed cycle.

### 9.4 Stack contract

The pattern must define exact:

- input stack items;
- retained semantic operands;
- consumed witness items;
- output stack value;
- cleanup behavior;
- altstack behavior;
- failure-path behavior.

Likely output:

```text
success leaves q or consumes q according to the operation pattern
```

The production API should choose one convention and apply it consistently.

### 9.5 Operand reuse

Operations need `q` for output/state construction checks.

The arithmetic pattern should avoid discarding the authenticated quotient if
that forces an unverified duplicate witness later.

Options:

- retain verified `q` on stack;
- duplicate before proof and preserve one copy;
- recompute from the same authenticated witness with equality check.

The stack contract must state this explicitly.

### 9.6 Stack scheduling

Prototype both:

- straightforward readable schedule;
- deterministic greedy optimized schedule.

Do not start with opaque hand-minimized bytecode.

The readable schedule is the audit/reference implementation.

Any optimized schedule must be shown equivalent through pattern tests or formal
analysis.

### 9.7 Altstack policy

Use altstack only when:

- target semantics are exact and typed;
- it materially reduces stack pressure;
- branch behavior remains clear;
- resource formulas account for combined stack+altstack limit.

---

## 10. Threat and failure model

The attacker controls or influences:

- quotient witness `q`;
- remainder witness `r`;
- any witness-supplied limbs/carries;
- target input/output amount encodings;
- stack item widths;
- signs;
- transaction values and metadata where not independently pinned;
- operation branch and witness order;
- target program/leaf selection;
- layout positions;
- confidential/explicit representation where ABI permits a choice.

The attacker cannot break:

- target consensus arithmetic semantics;
- target script execution;
- cryptographic commitments/signatures;
- linked program identity.

### 10.1 Under-quotient

Supply:

```text
q = floor(a*b/d) - 1
```

with manipulated remainder/carries.

Required result:

```text
reject
```

### 10.2 Over-quotient

Supply:

```text
q = floor(a*b/d) + 1
```

Required result:

```text
reject
```

### 10.3 Equality with out-of-range remainder

Supply:

```text
a*b = q*d+r
r >= d
```

Required result:

```text
reject
```

### 10.4 Truncated high product

Supply correct low limbs but wrong/missing high limb.

Required result:

```text
reject
```

### 10.5 Noncanonical decomposition

Supply limbs that recompose incorrectly or exceed limb bounds.

Required result:

```text
reject
```

### 10.6 Carry manipulation

Supply or derive a carry inconsistent with the coefficient.

Required result:

```text
reject
```

### 10.7 Signed interpretation attack

Supply an 8-byte value with high signed bit set or negative target
interpretation.

Required result:

```text
reject
```

### 10.8 Width attack

Supply:

- 7-byte amount;
- 9-byte amount;
- nonminimal script number where relevant;
- wrong-endian value.

Required result:

```text
reject
```

### 10.9 Divisor zero

Required result:

```text
reject
```

### 10.10 Result overflow

Choose valid `a,b,d` whose floor quotient is `>=2^51`.

Required result:

```text
reject
```

### 10.11 Target arithmetic overflow

Attempt to force one partial product or temporary above signed 64-bit through
invalid limb values.

Required result:

```text
limb-domain or arithmetic-success check rejects
```

### 10.12 Witness reorder

Place quotient/remainder/limbs in wrong witness positions.

Required result:

```text
reject
```

### 10.13 Wrong semantic operand

Use a mathematically valid floor proof over an amount not authenticated as the
operation's intended input/state field.

Required result:

```text
operation-level target relation rejects
```

Pattern-level arithmetic success is insufficient when fact-source binding is
wrong.

---

## 11. Prototype design

### 11.1 Prototype location

Preferred:

- experimental/private modules under `tapscript::arithmetic`;
- reference evaluator in realization/vectors or a narrow test module;
- target-native runner in vectors.

Production release must reject prototype pattern status.

### 11.2 Stage A — Mathematical reference

Implement a clear Rust reference using wide or arbitrary-precision arithmetic:

```rust
fn reference_floor_mul_div(
    a: SatLike,
    b: SatLike,
    d: SatLike,
) -> Result<(q, r), ReferenceError>;
```

The reference must:

- reject `d = 0`;
- compute exact `a*b`;
- compute quotient and remainder;
- reject quotient outside `Sat`;
- expose normalized limb vectors for test comparison.

Avoid reusing the target limb implementation as the only expected result.

### 11.3 Stage B — Limb-bound proof

Before script implementation, derive and document exact maximum bounds for:

- operand limbs;
- every partial product;
- every coefficient sum;
- every carry;
- every remainder addition temporary;
- every wide subtraction/comparison temporary;
- every signed target operation.

This may be checked through:

- Rust exhaustive bound formulas;
- symbolic assertions;
- a small formal arithmetic note;
- SMT as a supplement.

No target operation should rely on an unstated “seems below `i64::MAX`”
assumption.

### 11.4 Stage C — Target typed pattern

Implement the readable Candidate-A target pattern:

- domain checks;
- decomposition;
- product normalization;
- remainder addition;
- equality;
- remainder comparison;
- quotient preservation;
- cleanup.

Validate stack effects statically.

### 11.5 Stage D — Pattern target-native vectors

Execute the standalone pattern through a synthetic target program/context.

Test exact and malformed witnesses.

### 11.6 Stage E — Candidate-B comparison

Implement witness-supplied limb/carry candidate only if Candidate A is too
large or stack-heavy, or if measurement suggests a clear benefit.

Do not implement every candidate by default.

### 11.7 Stage F — Operation integration: redemption

Integrate the accepted/readable pattern into a synthetic or real redemption
leaf after STATE constructor support exists.

Redemption is a useful first operation integration because it uses one wide
floor proof and has a clear formula-bound payout.

If STATE constructor work is not yet ready, use a synthetic operation program
whose operand bindings are explicit and separately authenticated, then repeat
with real redemption later.

### 11.8 Stage G — Settlement batch-size-2 integration

Run two entitlements and two class floors in one complete settlement prototype.

This is required before claiming the pattern scales to settlement.

### 11.9 Stage H — Cycle integration

Run one cycle issuance proof in a representative complete cycle transaction
after the relevant operation exists.

### 11.10 Stage I — Optimization

If necessary:

- apply deterministic stack scheduling;
- apply verified peephole rewrites;
- compare exact target behavior;
- preserve the readable reference pattern.

### 11.11 Stage J — Formal-equivalence spike

Attempt a bounded bit-vector model for the exact selected pattern.

This is optional for initial feasibility but should be attempted if the opcode
subset is tractable.

---

## 12. Test and vector plan

### 12.1 Deterministic fixed vectors

Include at least:

```text
a=0, b=0, d=1
a=1, b=1, d=1
a=1, b=max, d=max
a=max, b=max, d=max
a=max, b=max, d=1 with result overflow
a=max, b=max, d=max-1
a=max, b=1, d=2
a=2^26-1, b=2^26-1, d=...
a=2^26, b=2^26, d=...
a=2^51-1, b=2^51-1, d=2^51-1
```

where:

```text
max = 2^51 - 1
```

### 12.2 Limb boundaries

Exercise values around:

```text
0
1
B-1
B
B+1
2B-1
2^25*B - 1
2^51 - 2
2^51 - 1
```

### 12.3 Carry boundaries

Create cases where:

- `c_0` is exactly below/at/above a multiple of `B`;
- `t_1` creates zero/minimum/maximum carry;
- `t_2` creates nonzero top limb;
- remainder addition propagates through one, two, three, or four limbs;
- final equality uses the highest possible nonzero limb.

### 12.4 Remainder boundaries

For several divisors:

```text
r = 0
r = 1
r = d-1
r = d
r = d+1
```

The last two reject.

### 12.5 Quotient mutation

For every positive fixture where possible:

```text
q-1
q
q+1
```

Require only `q` to pass.

Handle `q=0` and `q=max` boundaries explicitly.

### 12.6 Divisor boundaries

```text
d = 0
d = 1
d = 2
d = B-1
d = B
d = max
```

### 12.7 Width/sign/endian vectors

- exact 8-byte LE;
- 7-byte;
- 9-byte;
- negative signed 8-byte;
- high bit set;
- byte-reversed value;
- script-number encoding supplied instead of LE64;
- empty vector.

### 12.8 Witness-limb/carry vectors

If Candidate B is used:

- limb equal to `B`;
- negative limb;
- incorrect recomposition;
- carry too large;
- inconsistent carry;
- duplicate/missing limb;
- nonnormalized product.

### 12.9 Randomized/property vectors

Generate many valid:

```text
a,b,d
```

within domain.

For each:

- derive `q,r` from independent reference;
- target accepts;
- mutate `q`, `r`, one limb, and one carry;
- target rejects where applicable.

Use explicit deterministic seeds for canonical regression sets.

### 12.10 Operation vectors

#### Redemption

- exact payout;
- rounding down by one remainder;
- quotient below/above;
- wrong state operand;
- payout output mismatch;
- reserve decrement mismatch;
- sealing and nonsealing boundary.

#### Cycle

- `Q=0`;
- positive issuance;
- maximum active-backing case;
- wrong issuance amount;
- wrong `Ω`, `Y`, or `Q` operand binding.

The operation may omit the wide proof entirely when `Q=0`; activation coverage
must include both cases.

#### Settlement

- one entitlement;
- two entitlements;
- both classes;
- zero class draw;
- nonzero remainder;
- per-entitlement floor versus aggregate floor difference;
- wrong entitlement principal;
- wrong allocation/control operand;
- wrong output amount.

### 12.11 Differential target vectors

For every accepted operation transaction:

- compare target result with model/realization semantic projection;
- ensure the arithmetic relation carrier executes;
- record target program identity;
- record witness schema identity.

---

## 13. Optional formal-equivalence plan

### 13.1 Goal

Prove or search for a counterexample to:

```text
selected target arithmetic pattern accepts
iff
FloorMulDiv(a,b,d,q)
```

for all bounded input and witness values under the modeled target opcode
semantics.

### 13.2 Scope

Model only the target opcode subset used by the selected pattern:

- fixed-width arithmetic;
- comparisons;
- stack operations;
- conditionals;
- exact-width checks;
- any byte operations required by decomposition.

Do not model the complete Elements script interpreter initially.

### 13.3 QF_BV encoding

Use fixed-width bit vectors for:

- semantic operands;
- limbs;
- carries;
- target signed arithmetic;
- success flags;
- stack values where tractable.

Model malformed-width cases separately or with tagged values if needed.

### 13.4 Trusted base

The formal result depends on:

- correctness of the opcode semantics model;
- correspondence between modeled pattern and emitted bytes;
- solver/proof checker;
- bounded domain encoding.

The report must state those dependencies.

### 13.5 Acceptance role

Possible outcomes:

1. **successful equivalence proof**
   Strong supplemental evidence.

2. **counterexample**
   Candidate pattern rejected or corrected.

3. **intractable/incomplete encoding**
   Retain hand audit, property vectors, and target-native evidence; do not
   block implementation automatically unless policy later requires formal
   proof.

### 13.6 Proof artifact

If adopted, bind:

- exact target identity;
- exact pattern/program identity;
- semantics-model version;
- solver/proof-checker version;
- theorem/query hash;
- proof/result artifact hash.

---

## 14. Measurement plan

### 14.1 Pattern-level resources

Measure:

- script bytes;
- witness bytes for `q,r` and additional limbs/carries;
- initial stack item count;
- peak stack;
- peak altstack;
- maximum element size;
- executed opcode count/project cost;
- crypto budget, expected to be zero for arithmetic pattern itself;
- target execution time as noncanonical diagnostic only.

### 14.2 Readable versus optimized pattern

Measure both:

- readable reference schedule;
- deterministic optimized schedule.

The optimized schedule is accepted only if:

- target behavior identical across vectors;
- stack contract remains validated;
- formal/pattern identity updated;
- resource improvement is material.

### 14.3 Complete redemption transaction

Measure:

- STATE and RESV constructor/control paths;
- receipt input;
- payout output;
- successor outputs;
- arithmetic witness;
- signatures;
- sponsor maximum candidate;
- full transaction weight/policy.

This measurement depends on other prototypes and may occur after pattern
acceptance.

### 14.4 Complete settlement batch-size-2 transaction

Measure:

- control;
- vault if present;
- two entitlements;
- per-class arithmetic proofs;
- receipt outputs;
- successor or terminal outputs;
- sponsor;
- all witnesses/programs.

This is the minimum realistic settlement feasibility test.

### 14.5 Complete cycle transaction

Measure one nonempty cycle with:

- STATE;
- RESV;
- PACE;
- distribution authority;
- issuance;
- control/vault;
- arithmetic witnesses;
- operator/delayed branch as appropriate;
- sponsor and anchor cases.

### 14.6 Predicted versus observed

Compare:

- backend pattern formula;
- linker linked-program formula;
- transaction prediction;
- target-native observed result.

Any mismatch requires correction before accepted production use.

### 14.7 Bound implications

The arithmetic research does not select final protocol bounds.

It reports:

- per-proof cost;
- complete operation candidate costs;
- monotonicity observations;
- likely calibration constraints.

Final bounds are selected by the release calibration runner.

---

## 15. Acceptance criteria

The selected tapscript wide-floor pattern is accepted only when every mandatory
criterion passes.

### 15.1 Mathematical correctness

- [ ] exact target relation is documented;
- [ ] quotient/remainder proof is shown equivalent to floor;
- [ ] exact wide equality includes every high limb;
- [ ] `0 <= r < d` is enforced;
- [ ] `d > 0` is enforced;
- [ ] `0 <= q < 2^51` is enforced;
- [ ] every semantic operand has correct domain;
- [ ] quotient one below rejects;
- [ ] quotient one above rejects;
- [ ] malformed carry/limb witnesses reject.

### 15.2 Target arithmetic safety

- [ ] every partial product bound is documented;
- [ ] every temporary/carry bound is documented;
- [ ] no valid path crosses signed 64-bit maximum;
- [ ] invalid oversized limb cannot induce unchecked overflow;
- [ ] every target arithmetic success flag is checked;
- [ ] target failure-path stack behavior is modeled correctly;
- [ ] exact-width encoding is enforced;
- [ ] negative values reject.

### 15.3 Stack and witness safety

- [ ] stack contract validates;
- [ ] branch joins validate;
- [ ] initial stack count fits;
- [ ] peak stack plus altstack fits;
- [ ] maximum element size fits;
- [ ] witness ABI is canonical;
- [ ] verified `q` remains available where operation integration needs it;
- [ ] permissionless witnesses are publicly computable.

### 15.4 Target evidence

- [ ] exact target identity is pinned;
- [ ] every used opcode comes from typed target semantics;
- [ ] all fixed vectors pass target-native execution;
- [ ] randomized/property vectors pass;
- [ ] all mandatory negative vectors reject;
- [ ] target execution errors are distinguished from infrastructure failures.

### 15.5 Operation integration

- [ ] at least one operation-level target program binds the arithmetic pattern
      to authenticated semantic operands;
- [ ] wrong semantic operand binding rejects;
- [ ] output/state use of `q` is exact;
- [ ] target accepted projection matches realization/model;
- [ ] settlement batch-size-2 feasibility is measured before claiming
      settlement support.

### 15.6 Resource feasibility

- [ ] standalone pattern fits hard target limits;
- [ ] representative redemption transaction fits;
- [ ] batch-size-2 settlement transaction fits or settlement use is explicitly
      rejected/deferred;
- [ ] predicted and observed resources agree;
- [ ] no target policy uncertainty remains for selected deployment;
- [ ] final operation bounds remain subject to calibration.

### 15.7 Assurance handoff

- [ ] permanent pattern identity exists;
- [ ] production pattern tests are implemented;
- [ ] vector/report schemas bind exact program and target;
- [ ] implementation decision is created or updated;
- [ ] package plans are updated;
- [ ] release evidence requirements are defined;
- [ ] hand-audit residual is stated honestly;
- [ ] optional formal-evidence status is recorded.

---

## 16. Rejection criteria

Reject a candidate pattern for the initial tapscript backend if any unresolved
condition below holds.

### 16.1 Semantic rejection

- under-quotient passes;
- over-quotient passes;
- `r >= d` passes;
- quotient outside `Sat` passes;
- zero divisor passes;
- high product limb is not checked;
- target result differs from semantic floor.

### 16.2 Arithmetic-domain rejection

- valid input can overflow an unchecked signed target operation;
- invalid limb can bypass an overflow check;
- negative encoding can be interpreted as valid semantic amount;
- arithmetic success flag is ignored;
- temporary bounds are undocumented or false.

### 16.3 Witness rejection

- multiple uncontrolled witness representations pass;
- witness-supplied carry is underconstrained;
- permissionless caller cannot compute witness;
- witness size exceeds target/policy limits;
- quotient authentication is lost before output/state checks.

### 16.4 Target rejection

- required opcode unavailable or inactive;
- target behavior differs from typed stack contract;
- mandatory negative target vector accepts;
- exact target-native test cannot execute the pattern.

### 16.5 Resource rejection

- standalone pattern exceeds target hard limits;
- representative redemption cannot fit;
- settlement batch-size-2 cannot fit and no accepted alternative settlement
  design exists;
- resource formula cannot match observed execution;
- only a nondeterministic or noncanonical schedule fits.

### 16.6 Assurance rejection

- expected arithmetic is generated only by the same target pattern code;
- no independent reference exists;
- prototype bytes differ from production bytes without revalidation;
- report cannot bind exact target/program/witness identity;
- secrets appear in canonical reports.

A rejected tapscript candidate may remain valid for Simplicity or another
target.

---

## 17. Result

Pending.

When populated, this section must contain:

- exact repository revision;
- exact target identity;
- exact pattern/program identity;
- selected candidate;
- limb base;
- complete symbolic bound table;
- witness schema;
- stack contract;
- readable and optimized script sizes;
- predicted/observed resource table;
- fixed vector summary;
- randomized/property vector summary;
- operation integration summary;
- settlement batch-size-2 result;
- formal-equivalence result, if attempted;
- canonical report hashes;
- acceptance/rejection verdict.

Do not summarize the result only as:

```text
tests passed
```

The report must identify the exact tested artifact and scope.

---

## 18. Decision and implementation handoff

Pending.

Expected handoff if Candidate A or B succeeds:

1. create a new accepted decision, tentatively:

   ```text
   D008: Use Quotient/Remainder Multi-Limb Verification for Elements
   Wide Floor Arithmetic
   ```

2. stabilize target-independent proof-alternative mapping;
3. advertise one complete wide-floor proof pattern in tapscript;
4. add exact target capability dependencies;
5. stabilize witness ABI;
6. add linker/resource formula support;
7. add transaction witness generation;
8. promote fixed and generated vectors;
9. add release evidence requirements;
10. update roadmap phases for redemption, settlement, and cycle.

If no tapscript candidate succeeds:

- record the measured failure;
- evaluate whether another target proof exists;
- determine whether settlement/cycle bounds or transaction layouts can change
  without changing semantics;
- consider Simplicity timing;
- do not weaken the floor relation;
- begin normative review only if implementation evidence demonstrates the
  current abstract capability cannot be realized on the selected target.

---

## 19. Residual risks

Even an accepted pattern will retain residual risk.

### 19.1 Hand-audited stack implementation

Unless the complete emitted pattern is formally proved against exact target
semantics, limb/carry/stack correctness retains a hand-audit surface.

### 19.2 Target semantics model

Target-native tests depend on the exact Elements implementation and test
environment.

### 19.3 Finite vectors

Boundary/property vectors do not universally prove the relation.

Optional SMT/formal evidence can strengthen this.

### 19.4 Compiler placement

A correct arithmetic pattern can still receive the wrong semantic operands.

Operation-level relation carrier and fact-source vectors remain required.

### 19.5 Resource coupling

A pattern that fits one operation may fail in another due to:

- repeated proofs;
- deeper taptrees;
- additional signatures;
- larger witnesses;
- different layout.

Final per-operation calibration remains required.

### 19.6 Public arithmetic

The initial pattern requires public or publicly opened values. It does not
provide private arithmetic over commitments.

### 19.7 Future target revisions

Opcode semantics, policy, or resource behavior may change under another target
revision.

Target identity and evidence must be regenerated.

### 19.8 Reference implementation correlation

The Rust reference and realization/model may share conceptual formulas.

Independent mathematical vectors and target evidence mitigate but do not
eliminate correlated misunderstanding.

---

## 20. References

### Normative and current source

- [`../../docs/attestation/realization.md`](../../docs/attestation/realization.md)
- [`../../packages/model/src/scalar.rs`](../../packages/model/src/scalar.rs)
- [`../../packages/model/src/queries.rs`](../../packages/model/src/queries.rs)
- [`../../packages/model/src/ops/cycle.rs`](../../packages/model/src/ops/cycle.rs)
- [`../../packages/model/src/ops/redeem.rs`](../../packages/model/src/ops/redeem.rs)
- [`../../packages/model/src/ops/settlement.rs`](../../packages/model/src/ops/settlement.rs)
- [`../../packages/model/src/shape.rs`](../../packages/model/src/shape.rs)
- [`../../packages/model/src/tests/cycle_tests.rs`](../../packages/model/src/tests/cycle_tests.rs)
- [`../../packages/model/src/tests/redemption_tests.rs`](../../packages/model/src/tests/redemption_tests.rs)
- [`../../packages/model/src/tests/settlement_tests.rs`](../../packages/model/src/tests/settlement_tests.rs)

### Decisions

- [D001](../decisions/001-typed-rust-source.md)
- [D002](../decisions/002-realization-layer.md)
- [D003](../decisions/003-tapscript-first.md)
- [D004](../decisions/004-translation-validation.md)
- [D005](../decisions/005-value-representation.md)
- [D006](../decisions/006-transaction-abi.md)

### Package plans

- [`../packages/realization.md`](../packages/realization.md)
- [`../packages/compiler.md`](../packages/compiler.md)
- [`../packages/target-elements.md`](../packages/target-elements.md)
- [`../packages/tapscript.md`](../packages/tapscript.md)
- [`../packages/linker.md`](../packages/linker.md)
- [`../packages/transaction.md`](../packages/transaction.md)
- [`../packages/vectors.md`](../packages/vectors.md)
- [`../packages/release.md`](../packages/release.md)

### Target reference

- [`../reference/elements-tapscript.md`](../reference/elements-tapscript.md)

### Related research

- [`settlement-layout.md`](settlement-layout.md)

### Roadmap

- [`../roadmap.md`](../roadmap.md)

---

## 21. One-line research contract

> Determine, against one exact Elements target, whether a deterministic
> quotient/remainder multi-limb tapscript pattern can enforce
> `q = floor(a*b/d)` for all valid `<2^51` protocol amounts with exact wide
> equality, canonical witnesses, checked signed-64-bit operations, public
> constructibility, complete relation and operation vectors, and acceptable
> whole-transaction resources—and do not permit redemption, settlement, or
> cycle to use a production wide-floor proof until that exact pattern,
> witness ABI, resource formula, and evidence scope are accepted.
