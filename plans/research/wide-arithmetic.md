# Research Question: Exact Wide Floor Arithmetic · `q:arithmetic:wide-floor`

> **Status:** Accepted prototype — see (`sec:wide-arithmetic:result`)
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

## Candidate matrix · `tab:wide-arithmetic:candidates`

| Mint | Candidate | Witness | Main trade-off |
|---|---|---|---|
| (`candidate:arithmetic:derived-limbs`) | Quotient/remainder with script-derived limbs | `q,r` | Small witness; more script and stack work |
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

Accepted as a prototype. The selected candidate is
(`candidate:arithmetic:derived-limbs`) at base `B = 2^26`, resting on the
reviewed Euclidean `Div64`, which returns remainder, quotient, and a success
flag with the remainder normalized non-negative.

### Decisive design results · `sec:wide-arithmetic:decisive`

**Nothing that could be corrupted is witnessed.** The witness is exactly the
five amounts `a`, `b`, `d`, `q`, `r`. Every limb, every partial product, and
every carry is derived from those five inside the program. The wrong-limb and
wrong-carry rows of (`sec:wide-arithmetic:threats`) therefore have no witness
to mutate under Candidate A: they are refused by construction rather than by a
check, and the matrix records them as such rather than pretending to test them.

Candidates B and C were carried through to typed lower bounds rather than
dropped by argument — B costs 27 witness items where A costs 5, and C costs
three wide products where A computes one. Both are dominated on every measured
axis, and the comparison is emitted so that the domination is a figure rather
than a claim.

**The bound census is machine-checked.** Every intermediate the schedule
produces is proven below `2^53`, so no target arithmetic relies on an informal
overflow argument.

**One documented deviation from the staged form.** The quotient-side carry
cascade is folded, giving three divisions where the staged §11.5 shape gives
six. The folded form was cross-checked two independent ways: against the staged
form itself, and against arbitrary-precision digit extraction. The deviation is
recorded here because a reader comparing the emitted schedule against the guide
will otherwise find three divisions missing and be right to ask.

Layout is a single packed byte-string accumulator read by constant-width
slices, which is what keeps the schedule inside the reach bound: no primitive
reads below the third stack item, and there is no `OP_PICK`, no `OP_ROLL`, and
no altstack.

### Resources · `tab:wide-arithmetic:resources`

Measured on the emitted program.

| Quantity | Value |
|---|---:|
| script bytes | 523 |
| witness bytes (5 items) | 606 |
| peak main stack | 7 |
| widest element | 216 |
| instructions | 304 |
| multiplications | 8 |
| divisions | 11 |
| hash, curve, budget | 0 |

The pattern is pure arithmetic: it hashes nothing, touches no curve, and spends
none of the per-check validation budget.

### Native evidence · `sec:wide-arithmetic:evidence`

| Fact | Value |
|---|---|
| matrix | 39 of 39 rows agreed |
| claims | 11 of 11 covered |
| completeness | `complete_for_wide_floor_prototype` |
| determinism | two gated runs byte-identical |

Node and workspace provenance are the same run recorded in the constructor
research, and so is the reason no report digest appears here: the table records
a run that happened, and is not a locator for a report the repository does not
retain.

### Residuals · `sec:wide-arithmetic:residuals`

Enumerated by the matrix's own `residual_threats`, not by prose. The
load-bearing three are the enclosing-binding substitutions: swapping the
semantic source of `a`, of `b`, or of `d` is correct arithmetic over a wrong
fact, and this pattern has no standalone boundary that could catch it. Operand
binding is the enclosing operation's claim and stays there — which is the same
separation (`sec:wide-arithmetic:threats`) already draws in its final row.

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

### What this acceptance is · `sec:wide-arithmetic:scope`

The accepted object is a prototype. No operation emits it, no ABI is frozen
around its five witness items, no bundle contains it, no identity was minted
for it, and no bound was calibrated from its figures. Stage 6 operation
integration was not performed, so no settlement claim rests on this: the
batch-size-2 measurement that (`gate:wide-arithmetic:accept`) requires before
settlement feasibility may be claimed has not been taken, and settlement
feasibility is accordingly not claimed.

Promotion to a production backend pattern is a separate reviewed act. What the
acceptance settles is the question the pattern was blocked on — whether the
target can verify the exact floor relation over the `2^51` domain at a cost an
operation could afford — and the answer is yes, at 523 script bytes.

Evidence lives in `target-elements-conformance` and in the Guide-10 gate record
in [the backlog](../backlog.md) (§2.14).
