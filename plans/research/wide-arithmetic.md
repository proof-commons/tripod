# Research Question: Exact Wide Floor Arithmetic · `q:arithmetic:wide-floor`

> **Status:** Prototype and measurement required
> **Blocks:** redemption; settlement floors; cycle issuance
> **Affected packages:** realization, compiler, target-elements, tapscript,
> transaction, vectors, release
> **Decisions:** D003, D004, D006
> **Imports:** (`[RZ-sec:arithmetic:gadgets]`),
> (`[RZ-rule:translation:division]`),
> (`[RZ-pin:pins:arith]`)
> **Expected handoff:** accepted tapscript floor-proof pattern or documented target rejection

## Question · `sec:wide-arithmetic:question`

Which Elements tapscript construction proves:

```text
q = floor(a*b/d)
```

for:

```text
0 <= a,b,q < 2^51
0 < d < 2^51
```

while rejecting malformed operands, zero divisor, quotient error, overflow,
noncanonical witnesses, and incorrect semantic operand binding?

The leading relation is:

```text
a*b = q*d + r
0 <= r < d
```

with witness:

```text
q, r
```

This is equivalent to the floor relation when `d > 0`.

The target program must establish full-width equality without overflowing the
target’s signed 64-bit operations.

## Fixed constraints · `sec:wide-arithmetic:constraints`

- semantic amounts use the checked `<2^51` domain;
- target operands use exact reviewed fixed-width encodings;
- negative target values reject;
- every target arithmetic success flag is checked;
- no general target loop is assumed;
- quotient/remainder witnesses are publicly computable for initial call sites;
- the verified quotient must feed the exact operation output/state relation;
- complete transaction feasibility matters, especially for repeated
  settlement floors.

Call sites are:

| Operation | Mapping |
|---|---|
| redemption | `a=x`, `b=Ω`, `d=Y`, `q=payout` |
| cycle | `a=Q`, `b=Y`, `d=Ω`, `q=issuance` |
| settlement | `a=δ`, `b=class allocation`, `d=cycle principal`, `q=draw` |

Small fixed-ratio splits may use a separate narrow pattern.

## Candidate matrix · `tbl:wide-arithmetic:candidates`

| Mint | Candidate | Witness | Main trade-off |
|---|---|---|---|
| `candidate:arithmetic:derived-limbs` | Quotient/remainder with script-derived limbs | `q,r` | Small witness; more script and stack work |
| `candidate:arithmetic:witnessed-limbs` | Witnessed limbs and carries, all recomposed and checked | `q,r,limbs,carries` | Larger witness; simpler local equations |
| `candidate:arithmetic:sandwich` | Verify `q*d <= a*b < (q+1)*d` | `q` | Direct form; three wide products/comparisons |
| `candidate:arithmetic:long-division` | Statically unrolled division | none or partial | Likely large and difficult to audit |
| `candidate:arithmetic:cryptographic-proof` | Target proof system | target-specific | No approved initial proof |

The leading candidate is
(`candidate:arithmetic:derived-limbs`).

## Leading limb design · `candidate:arithmetic:derived-limbs`

Use provisional base:

```text
B = 2^26
```

A semantic amount decomposes into:

```text
x = x_0 + x_1*B
0 <= x_0 < 2^26
0 <= x_1 < 2^25
```

A product fits four 26-bit limbs because:

```text
x*y < 2^102 < 2^104 = B^4
```

For:

```text
a = a_0 + a_1*B
b = b_0 + b_1*B
```

derive raw coefficients:

```text
c_0 = a_0*b_0
c_1 = a_0*b_1 + a_1*b_0
c_2 = a_1*b_1
```

Then normalize through checked carries into:

```text
P(a,b) = [p_0,p_1,p_2,p_3]
```

in explicitly documented limb order.

Compute:

```text
P(q,d) + r
```

with checked carry propagation and compare all four limbs with `P(a,b)`.

Separately verify:

```text
d > 0
0 <= r < d
0 <= q < 2^51
```

The prototype must derive exact symbolic maxima for every partial product,
coefficient, carry, temporary, and sum before emitting target arithmetic.

## Witness ABI · `rule:wide-arithmetic:witness`

Candidate A witness:

```text
q: exact target 8-byte amount
r: exact target 8-byte amount
```

Candidate B may add canonical limb/carry items only if measurement justifies
them.

Every item defines:

- width and byte order;
- nonnegative domain;
- public availability;
- canonical encoding;
- source relation;
- malformed-input behavior.

The authenticated `q` must remain available to the enclosing operation.

## Threat model · `sec:wide-arithmetic:threats`

Required failures:

| Mutation | Required result |
|---|---|
| `q = floor-1` | reject |
| `q = floor+1` | reject |
| equality with `r >= d` | reject |
| zero divisor | reject |
| quotient outside amount domain | reject |
| wrong or truncated high limb | reject |
| wrong carry/borrow | reject |
| noncanonical limb decomposition | reject |
| negative fixed-width value | reject |
| 7-byte or 9-byte amount | reject |
| byte-reversed amount | reject |
| witness item reorder | reject |
| correct arithmetic over wrong semantic operand | operation-level rejection |

Pattern correctness and operation fact-source binding are separate claims.

## Prototype · `sec:wide-arithmetic:prototype`

### Stage 1 — independent reference

Implement exact host-side reference using wide or arbitrary-precision
arithmetic.

Return:

```text
q
r
canonical operand limbs
canonical product limbs
```

The target pattern implementation is not the sole expected-result source.

### Stage 2 — bound proof

Document and test exact upper bounds for:

- limbs;
- partial products;
- coefficient sums;
- carries;
- remainder addition;
- comparison temporaries;
- signed target operations.

No target instruction may rely on an informal overflow claim.

### Stage 3 — readable target pattern

Implement Candidate A without premature byte optimization.

Statically validate:

- stack contract;
- branch joins;
- success/failure stack behavior;
- altstack use;
- target limits.

### Stage 4 — standalone target vectors

Execute the pattern in the exact development target.

### Stage 5 — compare Candidate B if needed

Implement witnessed limbs/carries only if Candidate A fails resource or stack
goals.

### Stage 6 — operation integration

Integrate in this order:

1. synthetic authenticated operands;
2. redemption;
3. settlement batch size 2;
4. nonempty cycle.

Settlement batch size 2 is mandatory before claiming settlement feasibility.

### Stage 7 — optional formal check

Attempt a bounded bit-vector equivalence check over the selected opcode subset.

A formal result must bind:

- exact target semantics model;
- exact emitted pattern identity;
- solver/proof checker;
- theorem/query hash.

Failure to complete this optional proof does not automatically reject an
otherwise accepted finite-evidence pattern unless release policy later requires
it.

## Vectors · `sec:wide-arithmetic:vectors`

Fixed vectors cover:

- zero and one;
- exact division;
- remainder one and `d-1`;
- divisor one and maximum;
- amount maximum `2^51-1`;
- quotient zero;
- result-domain overflow;
- values around `B-1`, `B`, and `B+1`;
- every carry boundary;
- nonzero top limb;
- carry propagation from remainder through several limbs.

For every eligible positive fixture, test:

```text
q-1
q
q+1
```

Property vectors use explicit deterministic seeds and compare with the
independent reference.

Operation vectors additionally mutate authenticated operands, payout/state
outputs, and target programs.

## Measurements · `sec:wide-arithmetic:measurements`

Measure:

- readable pattern bytes;
- optimized pattern bytes, if any;
- witness bytes;
- initial and peak stack;
- altstack;
- maximum element;
- target operation cost;
- complete redemption transaction;
- complete settlement batch-size-2 transaction;
- representative nonempty cycle transaction.

Compare backend, linker, and transaction predictions with target observations.

The research reports per-proof and operation costs. Final bounds remain release
calibration outputs.

## Acceptance · `gate:wide-arithmetic:accept`

Accept only when:

- exact floor equivalence is documented;
- all quotient/remainder constraints are target-enforced;
- every partial target operation is proven in signed range;
- every success flag is checked;
- malformed widths/signs/limbs/carries reject;
- standalone and operation-level target vectors pass;
- authenticated operands feed exact state/output relations;
- permissionless witnesses are public;
- settlement batch 2 is measured before settlement support is claimed;
- predicted and observed resources agree;
- pattern and witness identities are deterministic;
- hand-audit and formal-evidence scope are stated honestly.

## Rejection · `gate:wide-arithmetic:reject`

Reject a candidate if:

- under- or over-quotient passes;
- any high limb is unchecked;
- invalid limb can cause unchecked target overflow;
- target stack behavior differs from the contract;
- witness representation is underconstrained;
- operation can bind the proof to the wrong fact;
- representative redemption cannot fit;
- settlement batch 2 cannot fit and no accepted settlement alternative exists;
- only mock execution passes;
- production pattern differs from tested bytes.

## Result · `sec:wide-arithmetic:result`

Pending.

## Handoff · `sec:wide-arithmetic:handoff`

A successful result creates an arithmetic-pattern decision and updates:

- realization proof alternatives;
- compiler target requirements;
- typed target capability status;
- tapscript pattern and resource formulas;
- transaction witness ABI;
- permanent vectors;
- settlement and cycle phase readiness;
- release evidence requirements.

A failed result does not weaken the exact floor relation.
