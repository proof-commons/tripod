# Research Question: Bounded Settlement Layout and Obligation Placement

> **Status:** OPEN / PROTOTYPE REQUIRED / MEASUREMENT REQUIRED
> **Blocks:** production `settle-distribution` backend support; final settlement
> transaction and witness ABI; settlement obligation placement; calibrated
> `SETTLEMENT_BATCH_MAX`; complete distribution lifecycle evidence; final cycle
> deployment path because cycle-created controls, vaults, and entitlements must
> remain settleable
> **Affected packages:** `realization`, `compiler`, `target-elements`,
> `tapscript`, `linker`, `transaction`, `vectors`, and `release`
> **Depends on decisions:**
> [D001](../decisions/001-typed-rust-source.md),
> [D002](../decisions/002-realization-layer.md),
> [D003](../decisions/003-tapscript-first.md),
> [D004](../decisions/004-translation-validation.md),
> [D005](../decisions/005-value-representation.md),
> [D006](../decisions/006-transaction-abi.md)
> **Depends on research:**
> [`wide-arithmetic.md`](wide-arithmetic.md) for exact per-entitlement floors;
> target/transaction foundations from
> [`state-object-constructor.md`](state-object-constructor.md) where linked
> metadata-dependent constructors are reused; public-declassification results
> only if a future settlement representation uses confidential values
> **Related normative constraints:** permissionless settlement; one control;
> vault iff remaining class value is positive; nonempty bounded entitlement
> batch; exact matching target cycle; one entitlement per request; floor applied
> per entitlement before aggregation; owner- and class-preserving receipt
> routing; exact `ENT`, `DIST_CTL`, and `U` canonical partitions; continuing
> control/vault succession; terminal control closure and residue destruction;
> audit-only residue; sponsor isolation; exact event projection; loopless target
> lowering; relation-indexed evidence in
> `docs/attestation/realization.md`
> **Expected decision output:** an accepted or rejected canonical settlement
> transaction layout and obligation-placement strategy, initially demonstrated
> at batch size 2, together with a typed witness ABI, target proof pattern,
> resource formula, target-native vector set, and guidance for deployment
> calibration
> **Machine-consumed by the toolchain:** no

---

## 1. Question

Which canonical, bounded Elements transaction layout and target obligation
placement can enforce `settle-distribution` exactly while preserving
permissionless construction and fitting target limits?

The construction must support a nonempty bounded entitlement batch and prove,
for every consumed entitlement:

1. the entitlement is a canonical explicit-`ENT` object;
2. it targets the consumed distribution control's cycle;
3. its semantic principal is its actual consensus value;
4. the live and time-locked draws are computed independently by exact
   per-entitlement floor formulas;
5. every positive draw is routed to the entitlement's committed owner in the
   correct receipt class;
6. zero draws create no zero-valued receipt output;
7. the entitlement is destroyed exactly once.

Across the whole transaction, the construction must also prove:

1. exactly one distribution control is consumed;
2. the control is canonical explicit `DIST_CTL` of amount one;
3. a matching distribution vault is consumed iff the control's remaining live
   plus time-locked value is positive;
4. the vault is canonical explicit `U`, targets the same cycle, and has exact
   value equal to the control's class remainders;
5. settled principal and class draws are summed exactly;
6. successor control counters are exact in the continuing case;
7. successor vault presence and value are exact in the continuing case;
8. no successor control or vault exists in the terminal case;
9. the control is destroyed exactly once in the terminal case;
10. positive terminal `U` residue is destroyed exactly once and projected into
    separate live and time-locked audit components;
11. every canonical input/output belongs to exactly one flow, issuance, or
    destruction relation;
12. sponsor inputs and change are isolated from protocol value;
13. the operation requires no entitlement-owner or operator secret;
14. every target-enforced relation has an executable carrier;
15. the complete transaction fits the selected target's consensus, policy,
    stack, witness, crypto, and weight limits.

The first decisive prototype uses:

```text
settlement batch size = 2
```

A batch-size-2 prototype is large enough to expose:

- repeated entitlement arithmetic;
- output mapping;
- zero/nonzero class combinations;
- duplicate-recipient aggregation questions;
- local versus global obligation placement;
- continuing versus terminal behavior;
- uniqueness and completeness requirements.

No final general settlement ABI or calibrated batch default is accepted before
this prototype resolves the design.

---

## 2. Why the answer matters

### 2.1 Settlement is the sole bridge between two accounting domains

Settlement destroys entitlement-domain value:

```text
ENT
DIST_CTL at terminal closure
```

and routes receipt-domain value:

```text
U vault
    →
live receipts
time-locked receipts
successor vault
terminal residue destruction
```

An error can therefore break both:

- entitlement lifecycle;
- receipt accounting.

### 2.2 The operation is permissionless

Settlement deliberately requires no entitlement-owner signature.

Any party must be able to advance the shared distribution state while every
receipt remains routed to its committed owner.

A target construction that needs:

- entitlement-owner secret;
- entitlement-owner private opening;
- operator signature;
- operator database;
- private vault data

changes the abstract capability and is not a valid implementation-only choice.

### 2.3 Flooring occurs before aggregation

For one entitlement principal `δ_i`, original control principal `P`, and
original class allocation `D_c`:

```text
m_c(i) = floor(δ_i * D_c / P)
```

The operation must apply the floor to each entitlement independently.

It may not compute:

```text
floor(sum(δ_i) * D_c / P)
```

because floor is not aggregation-invariant.

The target layout must preserve the per-entitlement relation even if physical
receipt outputs are later aggregated by owner and class.

### 2.4 The executable model aggregates outputs conveniently

The current executable model may aggregate physical receipt outputs by:

```text
(owner, class)
```

after computing per-entitlement floors.

That aggregation is semantically valid because the per-entitlement amounts are
computed first.

It is not necessarily the simplest loopless target layout.

A target backend may prefer one receipt output per entitlement and class,
subject to:

- positive-value output rules;
- architecture cardinality bounds;
- owner/class/value correctness;
- target resource limits.

The research must distinguish:

```text
semantic recipient/value multiset
```

from:

```text
one concrete target positional encoding
```

### 2.5 Target programs execute per input

The distribution control is the natural coordinator because it authenticates
the global counters and continuing/terminal branch.

Each entitlement input also authenticates local:

- owner;
- target cycle;
- principal;
- object constructor.

The target construction must decide where to place:

- each floor proof;
- each output mapping;
- aggregate sums;
- uniqueness checks;
- control/vault succession;
- terminal residue;
- output closure.

Putting everything on the control input may be large.

Distributing checks across entitlement inputs may create consistency and
uniqueness problems.

### 2.6 Optional outputs complicate positional mapping

Each entitlement may produce:

- no receipt output;
- live only;
- time-locked only;
- both classes.

Zero-value protocol outputs are not used as placeholders.

Therefore a simple fixed two-output layout per entitlement is unavailable
unless the target construction uses another canonical absent-value
representation that preserves semantic and target constraints.

The layout must map variable positive output families unambiguously.

### 2.7 Settlement likely sets one of the tightest deployment bounds

A batch may require, per entitlement:

- two wide floor proofs;
- constructor checks;
- owner and target metadata checks;
- one or two receipt-output checks;
- destruction accounting.

The control may require:

- aggregate arithmetic;
- control successor;
- vault successor;
- terminal branch;
- residue;
- complete output closure.

`SETTLEMENT_BATCH_MAX` must therefore be measured rather than inherited from
its draft default.

---

## 3. Existing semantic constraints

Let the consumed distribution control contain:

```text
cycle                  = k
principal              = P
live_allocation        = D_L
time_locked_allocation = D_T
remaining_principal    = R_P
remaining_live_value   = R_L
remaining_tl_value     = R_T
```

For consumed entitlements:

```text
e_i = (owner_i, target_i, principal_i = δ_i)
```

with:

```text
target_i = k
δ_i > 0
```

Define per-entitlement draws:

```text
l_i = floor(δ_i * D_L / P)
t_i = floor(δ_i * D_T / P)
```

Define batch totals:

```text
B_P = sum_i δ_i
B_L = sum_i l_i
B_T = sum_i t_i
```

Require:

```text
B_P <= R_P
B_L <= R_L
B_T <= R_T
```

Successor remainders:

```text
R_P' = R_P - B_P
R_L' = R_L - B_L
R_T' = R_T - B_T
```

### 3.1 Continuing case

When:

```text
R_P' > 0
```

require:

- exactly one successor control;
- same `cycle`, `principal`, `D_L`, and `D_T`;
- successor remainders exactly `R_P'`, `R_L'`, and `R_T'`;
- successor `DIST_CTL` amount one;
- exactly one successor vault iff:

  ```text
  R_L' + R_T' > 0
  ```

- successor vault explicit `U`;
- successor vault target cycle `k`;
- successor vault value:

  ```text
  R_L' + R_T'
  ```

- no control-close destruction;
- no terminal residue projection.

### 3.2 Terminal case

When:

```text
R_P' = 0
```

require:

- no successor control;
- no successor vault;
- control `DIST_CTL` amount one destroyed under the declared close tag;
- remaining live residue:

  ```text
  H_L = R_L'
  ```

- remaining time-locked residue:

  ```text
  H_T = R_T'
  ```

- positive total residue:

  ```text
  H_L + H_T
  ```

  destroyed as `U` under the declared residue tag;
- no residue output when total residue is zero;
- residue event projection carries separate `H_L` and `H_T`;
- recorded receipt supply is not decremented by residue;
- no attestation credit from residue.

### 3.3 Entitlement destruction

For every accepted settlement:

- all consumed entitlement `ENT` is destroyed;
- exactly one declared entitlement-destruction data-output family is present
  under the operation's semantic aggregation rule;
- destroyed total equals:

  ```text
  B_P
  ```

- no entitlement successor exists.

### 3.4 Receipt routing

For every entitlement:

```text
l_i > 0
    ⇒ owner_i receives live receipt value l_i

t_i > 0
    ⇒ owner_i receives time-locked receipt value t_i
```

Receipt outputs may be physically aggregated only after these values are
derived.

The semantic owner/class/value multiset must equal the per-entitlement result
multiset after any permitted aggregation.

### 3.5 Vault input rule

The predecessor vault exists iff:

```text
R_L + R_T > 0
```

When it exists:

```text
value(vault input) = R_L + R_T
cycle(vault input) = k
```

When the sum is zero, no vault input is permitted.

### 3.6 Permissionless construction

The constructor needs no entitlement-owner or operator secret.

The initial profile therefore expects public/explicit:

- entitlement principal;
- entitlement owner and target cycle;
- control counters;
- vault value;
- receipt-output amounts;
- successor counters;
- residue amounts.

Private entitlement arithmetic is outside the first settlement prototype.

### 3.7 Sponsor isolation

Optional sponsor inputs and change form one separate L-BTC open flow.

Sponsor value cannot alter:

- `B_P`;
- `B_L`;
- `B_T`;
- receipt outputs;
- successor vault;
- residue;
- control counters;
- data-output amounts.

---

## 4. Existing architecture constraints

The layout must derive from the typed architecture declaration.

### 4.1 Inputs

Settlement includes:

- exactly one distribution control;
- zero or one distribution vault according to semantic condition;
- one to `SETTLEMENT_BATCH_MAX` entitlements;
- zero to `FEE_SPONSOR_INPUT_MAX` sponsor inputs.

### 4.2 Outputs

Settlement includes:

- zero or one successor distribution control;
- zero or one successor vault;
- zero to `SETTLEMENT_BATCH_MAX` live receipt outputs;
- zero to `SETTLEMENT_BATCH_MAX` time-locked receipt outputs;
- zero or one plain L-BTC sponsor change;
- declared nonspendable destruction-output families under activation
  conditions.

### 4.3 Roots

Settlement uses no global root.

The distribution control acts as operation coordinator but is not one of the
five global roots.

### 4.4 Authorization

Settlement operation authorization is permissionless.

Input authorization:

- control: permissionless;
- vault: permissionless;
- entitlements: permissionless;
- sponsor inputs: sponsor-owner.

### 4.5 Canonical deltas

Depending on branch:

```text
ENT destruction
DIST_CTL lateral or destruction
U lateral when receipt/successor-vault outputs exist
U destruction when positive terminal residue exists
```

### 4.6 Projections

Always:

```text
transition certificate
```

Terminal only:

```text
distribution residue projection
```

No burn or clear projection.

---

## 5. Definitions and terminology

### 5.1 Semantic entitlement order

The semantic relation treats the entitlement batch as a set or bounded family
except where per-entitlement output mapping introduces a target order.

The target ABI may impose canonical order, expected initially to be:

```text
ascending canonical entitlement outpoint
```

The selected order must not change the semantic per-entitlement values.

### 5.2 Physical receipt aggregation

Physical aggregation means combining two or more semantically derived receipt
amounts with equal:

```text
owner
class
```

into one target receipt output.

Aggregation is permitted only after individual floors are computed.

### 5.3 Output mapping

An output mapping identifies which target receipt output(s) satisfy one or more
per-entitlement owner/class/value obligations.

### 5.4 Local carrier

A local carrier is the target program executed by one entitlement input.

It can naturally authenticate:

- its own outpoint/program;
- its own owner;
- its own target cycle;
- its own value;
- its current input index;
- target transaction outputs through introspection.

It cannot directly share its ordinary witness stack with other inputs.

### 5.5 Global carrier

The distribution-control program is the natural global carrier.

It can authenticate:

- complete transaction layout;
- control counters;
- vault relation;
- entitlement family/range;
- aggregate totals;
- continuing/terminal branch;
- successor control/vault;
- residue;
- output closure.

### 5.6 Mapping witness

A mapping witness is untrusted data proposing:

- output index;
- output range;
- owner/class group;
- positive-output bitmask;
- prefix count/rank;
- another entitlement-to-output relation.

The target must authenticate every mapping witness against the canonical
layout and semantic values.

### 5.7 Positive-output mask

A positive-output mask indicates which entitlement/class pairs have nonzero
draws.

It may assist compact layout computation but cannot replace verification that:

```text
bit set iff derived draw > 0
```

### 5.8 Continuing and terminal layouts

The ABI may define two distinct target program/leaf layouts:

- continuing settlement;
- terminal settlement.

Both realize one semantic operation under different activation conditions.

---

## 6. Required properties

Any accepted settlement design must satisfy all properties below.

### 6.1 Exact object authentication

Authenticate:

- control;
- optional vault;
- every entitlement;
- every receipt output;
- optional successor control;
- optional successor vault;
- destruction outputs;
- sponsor region.

### 6.2 Complete entitlement census

Every entitlement in the canonical family range is:

- checked exactly once;
- included in `B_P`;
- destroyed exactly once;
- mapped to its positive receipt obligations.

No entitlement input is omitted or counted twice.

### 6.3 Exact per-entitlement floors

For each entitlement/class, target enforcement establishes exact:

```text
floor(δ_i * D_c / P)
```

using the accepted wide-arithmetic pattern.

No aggregate-before-floor substitution is accepted.

### 6.4 Exact routing

Every positive class draw reaches the correct owner/class.

A triggerer cannot:

- redirect;
- swap owners;
- swap classes;
- merge distinct owners;
- move one unit between owners;
- drop a positive draw;
- create an extra receipt.

### 6.5 Output uniqueness and completeness

Every receipt output is claimed by exactly one semantic routing relation or one
explicitly permitted aggregation group.

No output is:

- unclaimed;
- claimed twice;
- funded by both vault flow and another closed-asset source;
- outside the authenticated receipt ranges.

### 6.6 Exact aggregate counters

The control and vault successor relations use exact batch totals derived from
the same per-entitlement proofs.

The global carrier must not trust caller-supplied `B_P`, `B_L`, or `B_T`
without proving them from the family.

### 6.7 Exact branch selection

Continuing branch iff:

```text
R_P' > 0
```

Terminal branch iff:

```text
R_P' = 0
```

No caller-selected branch flag may override this relation.

### 6.8 Vault/control bijection

Predecessor and successor vault presence/value must match control counters
exactly.

### 6.9 Terminal residue integrity

Terminal residue projection and destruction must use the exact remaining class
values.

No residue is:

- omitted;
- duplicated;
- moved to a receipt;
- credited to attestation;
- fed into monetary calculations.

### 6.10 Exact closed-asset partitions

For each closed asset:

#### `ENT`

```text
all entitlement input value
    =
entitlement destruction
```

#### `DIST_CTL`

Continuing:

```text
control input amount one
    =
successor control amount one
```

Terminal:

```text
control input amount one
    =
control-close destruction amount one
```

#### `U`

```text
vault input value
    =
live receipt outputs
    +
time-locked receipt outputs
    +
successor vault value
    +
terminal residue destruction
```

under the branch's active terms.

### 6.11 Permissionless constructibility

An unrelated actor can construct settlement using:

- public control;
- public vault;
- public entitlements;
- public arithmetic witnesses;
- public linked bundle/ABI;
- actor's own sponsor funds.

No owner/operator secret.

### 6.12 Canonical target layout

One deterministic ABI maps:

- input families;
- output families;
- data-output families;
- local/global programs;
- witness roles.

### 6.13 Relation carrier completeness

Every settlement relation has at least one reachable target carrier.

### 6.14 Resource feasibility

At minimum batch size 2 must fit the target.

A useful larger batch is preferred, but final maximum is calibration output.

### 6.15 Determinism

Given identical typed inputs, public witnesses, target, bundle, ABI, and
explicit test randomness, the constructed transaction and reports are
identical.

---

## 7. Core layout questions

The prototype must answer all of the following.

### 7.1 Input order

A likely initial order is:

```text
input 0:
    distribution control coordinator

input 1, conditional:
    distribution vault

next range:
    entitlement inputs in canonical outpoint order

final range:
    optional sponsor inputs
```

Questions:

- Does optional vault shift the entitlement start index?
- Should the ABI use separate continuing/zero-vault programs?
- Can one canonical count derive every range without ambiguity?
- How does each entitlement input prove its rank?

### 7.2 Output order

A likely family order is:

```text
optional successor control
optional successor vault
live receipt range
time-locked receipt range
optional sponsor change
nonspendable data-output ranges
```

Questions:

- Should data outputs precede sponsor change to simplify ordinal rules?
- Should receipt outputs be grouped by class or entitlement?
- How are zero draws omitted canonically?
- How are repeated owners aggregated, if at all?
- How does the terminal branch alter preceding ranges?

### 7.3 Data-output order

Required canonical order among:

```text
entitlement destruction
control-close destruction
residue destruction
```

must follow the linked ABI and activation conditions.

### 7.4 Program selection

Likely target programs include:

- control continuing coordinator;
- control terminal coordinator;
- vault local participation;
- entitlement local participation;
- sponsor local participation.

Questions:

- Does each entitlement program need separate live/time-locked output mapping
  variants?
- Can one program verify both optional outputs through a mask/rank witness?
- Are zero-draw classes represented through branch conditions?

### 7.5 Relation placement

Determine exactly which checks belong on:

- control;
- vault;
- each entitlement;
- sponsor.

No relation may remain only in transaction builder policy.

---

## 8. Candidate approaches

No candidate is accepted until batch-size-2 target-native prototypes and
measurements complete.

---

### Candidate A — Global control coordinator verifies complete settlement

#### Layout

The control input's target program:

- authenticates complete input/output ranges;
- inspects every entitlement input;
- reads owner, target, and principal;
- computes both class floors per entitlement;
- checks all receipt outputs;
- sums batch totals;
- checks control/vault branch;
- checks closed-asset partitions;
- checks data outputs and residue;
- checks sponsor boundary.

Entitlement and vault inputs perform only local object/operation participation
checks.

#### Output mapping

The coordinator owns one canonical receipt-output layout.

Possible initial policy:

```text
live outputs in entitlement order for positive live draws
then
time-locked outputs in entitlement order for positive time-locked draws
```

The coordinator derives output ranks by statically unrolled prefix sums over
positive-draw conditions.

#### Advantages

- one global proof of completeness;
- one place authenticates all ranges and sums;
- uniqueness and branch consistency are straightforward;
- easier relation-carrier attribution;
- local input programs remain small;
- no cross-input witness consistency beyond ABI and current input checks.

#### Risks

- very large coordinator program;
- repeated wide arithmetic concentrated in one input;
- high stack pressure;
- many inspected metadata fields;
- large witness for all quotients/remainders;
- control input crypto/resource budget may be limiting;
- scaling beyond small batch may be poor.

#### Current status

Primary baseline candidate for batch size 2.

---

### Candidate B — Entitlement-local floor and routing; control verifies aggregates

#### Layout

Each entitlement input program:

- authenticates its own entitlement;
- authenticates control input/reference;
- computes its own two class floors;
- verifies its own receipt output(s);
- contributes public draw values to a transaction-global structure.

The control program:

- verifies all entitlement inputs belong to the family;
- verifies aggregate principal/live/time-locked totals;
- verifies successor/terminal counters and vault;
- verifies output closure and uniqueness.

#### Possible aggregate mechanisms

1. witness-supplied totals checked against each local relation through shared
   target output commitments;
2. canonical coordinator data output containing per-entitlement terms;
3. positional receipt-output layout from which the control recomputes totals;
4. target transaction commitments authenticated by all inputs.

#### Advantages

- wide arithmetic distributed across entitlement inputs;
- lower control-program stack pressure;
- naturally local owner/target/value relation;
- target crypto/operation budget distributed per input;
- may scale better in script size per input.

#### Risks

- ordinary witnesses are not shared across inputs;
- local programs may accept inconsistent global totals;
- output uniqueness may be difficult;
- one receipt output may be claimed by two entitlement inputs;
- control must still recompute or authenticate all local results;
- target data-output commitment may become another global object;
- relation placement and evidence are more complex.

#### Current status

Strong candidate if a sound global consistency mechanism exists.

---

### Candidate C — Canonical per-entitlement output slots with no aggregation

#### Layout

For each entitlement in canonical input order, reserve conceptual slots:

```text
live receipt if l_i > 0
time-locked receipt if t_i > 0
```

Only positive outputs are emitted.

The target layout derives compact positive-output indexes using a verified
mask or prefix-rank calculation.

No owner/class aggregation occurs.

#### Advantages

- direct entitlement-to-output mapping;
- each positive receipt has one semantic source;
- no same-owner aggregation ambiguity;
- output count per class remains at most batch size;
- aligns with architecture maxima;
- local entitlement checks become plausible.

#### Risks

- variable output positions;
- mask/rank witness must be authenticated;
- coordinator may need statically unrolled prefix sums;
- two class ranges complicate mapping;
- zero-draw cases still require careful canonical omission;
- output count/ordering proof may remain expensive.

#### Current status

Likely output policy paired with Candidate A or B.

---

### Candidate D — Aggregate physical outputs by owner and class

#### Layout

Compute every per-entitlement draw, then aggregate:

```text
sum live draws by owner
sum time-locked draws by owner
```

Emit one receipt per `(owner, class)` group in canonical owner/class order.

#### Advantages

- fewer outputs when owners repeat;
- matches current executable model's convenient output construction;
- lower transaction weight in some cases;
- semantic aggregation is permitted after individual floors.

#### Risks

- target must sort/group owners;
- target must prove group completeness and uniqueness;
- variable number of owner groups;
- duplicated owners across entitlements;
- expensive loopless aggregation;
- complex witness and output mapping;
- difficult focused mutations;
- target owner ordering and equality logic add cost.

#### Current status

Semantically valid but unlikely to be the initial tapscript layout.

May remain a future optimization after per-entitlement layout works.

---

### Candidate E — Receipt outputs carry entitlement provenance metadata

#### Construction

Each settlement-created receipt commits an entitlement-derived provenance
identifier, such as:

- consumed entitlement outpoint;
- entitlement ordinal;
- settlement operation commitment.

Local entitlement programs verify their own provenance-tagged outputs.

The control verifies aggregate counters and output census.

#### Advantages

- local mapping and uniqueness become easier;
- output ownership is explicit;
- global coordinator may avoid complex output-reference witnesses;
- independent audit can trace each payout.

#### Risks

- changes receipt constructor metadata;
- provenance may persist across later transfers unless stripped or normalized;
- may alter semantic object recognition or behavioural hash;
- could create a new public observable;
- target constructor and transaction ABI become larger;
- may require normative architecture revision rather than implementation-only
  metadata;
- one provenance tag does not automatically prove aggregate uniqueness.

#### Current status

Research candidate only.

Before implementation, perform denotation/versioning review.

Do not treat provenance metadata as backend-only automatically.

---

### Candidate F — Fixed two receipt outputs per entitlement including zero values

#### Construction

Reserve:

```text
one live slot
one time-locked slot
```

per entitlement and emit explicit zero-valued protocol outputs when a draw is
zero.

#### Advantages

- trivial fixed mapping;
- no masks or prefix ranks;
- easy local checking.

#### Risks

- zero-value protocol outputs are invalid under current object recognition and
  target layout policy;
- creates unrecognized or inert outputs;
- may violate architecture/object cardinality semantics;
- increases transaction size;
- could create dust/junk or unexpected lifecycle.

#### Current status

Rejected under current semantic/shape rules.

Reconsideration requires normative object/shape review.

---

### Candidate G — Settlement proof accumulator or off-chain proof

#### Construction

Replace repeated direct checks with a proof/accumulator attesting the complete
batch relation.

#### Advantages

- potentially compact target program;
- may scale to larger batches.

#### Risks

- no approved target proof system;
- new cryptographic assumptions;
- witness/prover availability;
- target verification cost;
- changes evidence and trust surface;
- may conflict with no-trusted-proof/operator assumptions.

#### Current status

Out of scope for initial Elements tapscript.

---

## 9. Preferred initial prototype matrix

The batch-size-2 prototype should implement and compare at least:

### Prototype A1

```text
global control coordinator
+
canonical per-entitlement positive output layout
+
no owner aggregation
```

### Prototype B1

```text
entitlement-local floor/routing checks
+
control global aggregate/closure checks
+
same canonical per-entitlement positive output layout
```

If B1 cannot establish sound aggregate/uniqueness linkage without repeating
the full work on control, reject or revise it.

Candidate D aggregation and Candidate E provenance need not be implemented
unless A1/B1 fail or measurements show a compelling need.

---

## 10. Canonical batch-size-2 fixture families

The prototype must include fixtures covering distinct semantic branches.

### 10.1 Continuing, both classes positive

Two entitlements:

```text
e_0 owner A
e_1 owner B
```

Both produce positive live and time-locked draws.

Require:

- four receipt outputs under per-entitlement layout;
- successor control;
- successor vault;
- no residue projection.

### 10.2 Continuing, mixed zero draws

One entitlement has:

```text
live draw = 0
time-locked draw > 0
```

Another has:

```text
live draw > 0
time-locked draw = 0
```

Require exactly two positive receipt outputs and canonical omission of zero
outputs.

### 10.3 Continuing, repeated owner

Both entitlements route to the same owner.

Under no-aggregation candidate:

- outputs remain separate and ordered by entitlement.

Under aggregation candidate:

- one per class group, after proving per-entitlement floors.

### 10.4 Terminal, zero residue

Batch consumes all remaining principal and exactly exhausts class remainders.

Require:

- no successor control;
- no successor vault;
- control close;
- no residue destruction/projection amount beyond zero;
- exact receipt outputs.

### 10.5 Terminal, positive live residue

Require:

- separate live residue in event projection;
- `U` residue destruction;
- no successor vault;
- time-locked residue zero.

### 10.6 Terminal, positive time-locked residue

Mirror the prior case.

### 10.7 Terminal, both residues positive

Require exact class split in projection and aggregate `U` destruction.

### 10.8 Zero-value predecessor vault

Control has:

```text
R_L + R_T = 0
```

No vault input exists.

Entitlements may still be retired at zero draw.

Require exact zero receipt output behavior and continuing/terminal control
behavior according to principal.

### 10.9 Partial principal

Batch total is less than remaining principal.

Require exact successor remainders.

### 10.10 Exact principal boundary

Batch total equals remaining principal.

Require terminal branch.

---

## 11. Candidate A1 prototype design

### 11.1 Input layout

Proposed canonical layout:

```text
input 0:
    distribution control coordinator

input 1 if predecessor vault exists:
    distribution vault local program

next N inputs:
    entitlements in ascending outpoint order

remaining inputs:
    sponsor inputs in ascending outpoint order
```

The coordinator derives:

```text
vault_present = (R_L + R_T > 0)
entitlement_start = 1 + vault_present
entitlement_end = entitlement_start + N
sponsor_start = entitlement_end
```

Every range and transaction input count is checked.

### 11.2 Output layout

Proposed family order:

```text
output 0 if continuing:
    successor control

next output if continuing and successor vault value > 0:
    successor vault

next range:
    positive live receipt outputs in entitlement input order

next range:
    positive time-locked receipt outputs in entitlement input order

next optional output:
    sponsor change

final range:
    nonspendable data outputs in canonical semantic tag order
```

Alternative placement of data outputs before sponsor change should be measured
if it simplifies ordinal authentication.

### 11.3 Positive-output indexes

For batch size 2, statically derive:

```text
live_0_present = (l_0 > 0)
live_1_present = (l_1 > 0)
tl_0_present   = (t_0 > 0)
tl_1_present   = (t_1 > 0)
```

Then derive each positive output's index through fixed branch arithmetic.

Do not trust caller-supplied indexes without verifying the presence conditions.

For a general bound, the backend would statically unroll prefix counts up to
the calibrated maximum.

### 11.4 Coordinator witness

Expected witness roles:

- entitlement batch count `N`;
- two quotient/remainder pairs per entitlement;
- constructor/static-root data;
- target control path;
- optional sponsor-related layout facts;
- any public value openings under the selected explicit/public profile.

All arithmetic witnesses are public/computable.

### 11.5 Coordinator program phases

Suggested readable order:

1. authenticate control and target operation branch;
2. read/validate control metadata;
3. derive predecessor vault presence;
4. authenticate transaction input ranges/count;
5. inspect/authenticate vault if required;
6. inspect/authenticate every entitlement input;
7. validate entitlement target cycles;
8. derive/verify per-entitlement floors;
9. derive positive-output presence/ranks;
10. inspect/authenticate receipt outputs;
11. derive batch totals;
12. validate principal and class underflow bounds;
13. derive successor counters;
14. choose continuing/terminal branch;
15. verify successor control/vault or terminal absence;
16. verify ENT destruction;
17. verify DIST_CTL lateral/destruction;
18. verify U flow/residue;
19. verify data outputs/projection shape;
20. verify sponsor region;
21. leave canonical success.

This order is for auditability, not yet optimized.

### 11.6 Local entitlement programs

Each entitlement program should at least verify:

- current input is canonical entitlement;
- explicit `ENT` asset;
- current input lies in canonical entitlement range;
- target cycle matches the control or a target-authenticated operation
  commitment;
- operation program corresponds to settlement;
- no owner signature is required.

If the control coordinator already authenticates all local facts, determine
which local checks are still necessary to prevent the entitlement input from
being spent under a mismatched transaction/leaf combination.

### 11.7 Vault local program

Verify:

- canonical distribution vault;
- explicit `U`;
- target cycle;
- canonical vault slot;
- settlement operation;
- no owner signature.

Coordinator retains the global value/counter relation.

---

## 12. Candidate B1 prototype design

### 12.1 Local entitlement proof

Each entitlement input receives:

- control input index, expected fixed at 0;
- its own quotient/remainder pairs;
- its output mapping witness;
- linked constructor data.

It verifies:

- control constructor/program and metadata;
- own target cycle;
- own principal;
- own live/time-locked floors;
- own positive receipt outputs;
- own owner/class routing.

### 12.2 Global consistency challenge

The control must prove:

- every entitlement input executes the correct local program;
- every entitlement is included exactly once;
- every output mapping is unique;
- every positive receipt output is claimed;
- local `l_i,t_i` values sum to global `B_L,B_T`;
- no local program uses a different control metadata interpretation.

Possible mechanisms to prototype:

#### B1.1 Control recomputes all floors

This restores soundness but duplicates arithmetic.

Likely defeats the resource benefit.

#### B1.2 Control reads receipt outputs and recomputes only totals

Local programs prove floors; control sums output values and principal inputs.

Need to prove output values correspond to every local proof exactly once.

#### B1.3 Canonical positional mapping

Local programs use fixed entitlement rank and output rank.

Control authenticates complete family layout and sums receipt outputs.

This may be sound if local program identity and rank are unforgeable and every
entitlement is canonical.

#### B1.4 Committed local result vector

A target data structure commits each local `(δ_i,l_i,t_i,owner_i)` term.

Control validates aggregate commitment/output.

This adds a new target structure and may not reduce complexity.

### 12.3 Acceptance requirement

Candidate B1 is accepted only if the control can establish global completeness
and uniqueness without reproducing essentially all local arithmetic and without
introducing a new unsupported proof assumption.

---

## 13. Threat and failure model

The attacker can choose or influence:

- entitlement input set and order;
- entitlement values;
- entitlement owners and target cycles where malformed objects are attempted;
- control and vault input;
- batch count;
- quotient/remainder witnesses;
- output mapping witnesses;
- receipt output order;
- receipt output values;
- receipt owners/classes;
- successor control/vault values;
- continuing/terminal branch witness;
- residue values;
- data-output order;
- sponsor inputs/change;
- local/global target program combination;
- target witness order;
- constructor/static-root witness.

The attacker cannot break:

- target consensus;
- closed native asset conservation;
- cryptographic commitments;
- linked constructor identity;
- valid owner metadata commitment.

### 13.1 Cross-cycle entitlement

Use entitlement targeting another control cycle.

Required result:

```text
reject
```

### 13.2 Entitlement omitted from aggregate

Consume two entitlements but include one in `B_P` or class totals.

Required result:

```text
reject
```

### 13.3 Entitlement counted twice

Required result:

```text
reject
```

### 13.4 Aggregate-before-floor substitution

Use receipt totals corresponding to:

```text
floor((δ_0+δ_1)*D_c/P)
```

instead of:

```text
floor(δ_0*D_c/P)+floor(δ_1*D_c/P)
```

where they differ.

Required result:

```text
reject
```

### 13.5 Owner swap

Swap receipt owners while preserving aggregate class value.

Required result:

```text
reject
```

### 13.6 Class swap

Swap live and time-locked outputs.

Required result:

```text
reject
```

### 13.7 One-unit redistribution

Subtract one unit from owner A and add one to owner B.

Required result:

```text
reject
```

### 13.8 Missing positive output

Omit one nonzero receipt draw.

Required result:

```text
reject
```

### 13.9 Extra receipt output

Create an additional receipt not derived from an entitlement.

Required result:

```text
reject
```

### 13.10 Zero-valued placeholder

Insert zero-valued receipt in a supposedly absent slot.

Required result:

```text
reject
```

### 13.11 Mapping collision

Two entitlements claim the same receipt output.

Required result:

```text
reject
```

### 13.12 Unclaimed receipt output

Required result:

```text
reject
```

### 13.13 Wrong vault

Use vault from another cycle or with wrong value.

Required result:

```text
reject
```

### 13.14 Missing required vault

Control remainder sum positive but no vault input.

Required result:

```text
reject
```

### 13.15 Unexpected vault

Control remainder sum zero but vault supplied.

Required result:

```text
reject
```

### 13.16 Wrong successor counters

Preserve receipt outputs but alter successor control remainder.

Required result:

```text
reject
```

### 13.17 Wrong successor vault

Required result:

```text
reject
```

### 13.18 Wrong branch

Choose continuing when `R_P'=0` or terminal when `R_P'>0`.

Required result:

```text
reject
```

### 13.19 Residue omission or substitution

Required result:

```text
reject
```

### 13.20 Residue class swap

Aggregate residue destruction remains equal but event class components are
swapped.

Required result:

```text
reject
```

### 13.21 Sponsor interference

Use sponsor value to compensate for a missing receipt or vault amount.

Required result:

```text
reject
```

### 13.22 Mixed target programs

Use entitlement-local program from a different settlement/control or another
operation.

Required result:

```text
reject
```

### 13.23 Confidential/private entitlement

Supply an entitlement whose required principal is not publicly available under
the initial profile.

Required result:

```text
compiler/ABI unsupported or target rejection
```

No owner-private opening may be requested by permissionless settlement.

---

## 14. Prototype design

### 14.1 Prototype location

Preferred:

- experimental settlement modules under `tapscript`;
- candidate layouts under `transaction`;
- target vectors under `vectors`;
- candidate bundle under linker;
- no final release path.

### 14.2 Stage A — Semantic batch-size-2 fixture generator

Build deterministic model/realization fixtures for every family in Section 10.

Project:

- exact inputs;
- exact per-entitlement floors;
- expected owner/class/value multiset;
- expected control/vault successor;
- expected residue.

### 14.3 Stage B — Canonical ABI A1

Implement the proposed control/vault/entitlement/sponsor ranges and
per-entitlement positive output layout.

Derive candidate ABI identity.

### 14.4 Stage C — Global coordinator A1

Implement a readable unoptimized coordinator for batch size 2.

Use accepted wide arithmetic pattern.

Add local vault/entitlement participation programs.

### 14.5 Stage D — A1 complete transactions

Link candidate bundle, derive candidate ABI, construct target-native
transactions, and run all positive/negative fixtures.

### 14.6 Stage E — A1 measurement

Measure pattern, per-input, and complete transaction resources.

### 14.7 Stage F — Distributed B1

Implement enough B1 to answer whether local floor/routing plus global
aggregation is sound and materially cheaper.

If no sound consistency mechanism emerges without duplicating arithmetic,
record rejection and stop.

### 14.8 Stage G — Candidate comparison

Compare:

- correctness;
- carrier completeness;
- witness availability;
- stack complexity;
- transaction weight;
- per-input resource use;
- target policy;
- vector locality;
- scalability formula;
- implementation/audit complexity.

### 14.9 Stage H — Generalized bound formula

For the selected candidate, derive symbolic formulas over:

```text
N = settlement batch size
```

Validate against measured `N=1` and `N=2`, then additional candidate counts if
resources permit.

### 14.10 Stage I — Calibration handoff

Provide candidate operation/resource formulas and worst-case fixture generator
to the calibration runner.

Do not select final `SETTLEMENT_BATCH_MAX` inside this research note.

---

## 15. Test and vector plan

### 15.1 Positive vectors

- continuing, both classes positive;
- continuing, mixed zero draws;
- repeated owner;
- no predecessor vault;
- continuing with zero successor vault;
- terminal zero residue;
- terminal live residue;
- terminal time-locked residue;
- terminal both residues;
- sponsorless;
- sponsored;
- canonical input ordering permutation normalized by builder.

### 15.2 Arithmetic vectors

Per entitlement/class:

- exact division;
- nonzero remainder;
- zero draw;
- quotient below;
- quotient above;
- wrong divisor/allocation/principal binding;
- aggregate-before-floor mismatch.

### 15.3 Routing vectors

- owner swap;
- class swap;
- one-unit redistribution;
- missing output;
- extra output;
- mapping collision;
- unclaimed output;
- zero-valued placeholder;
- wrong receipt constructor;
- confidential closed asset receipt.

### 15.4 Control/vault vectors

- wrong predecessor vault;
- missing/unexpected vault;
- wrong control counters;
- wrong successor control;
- wrong successor vault;
- duplicate successor control;
- duplicate successor vault;
- wrong cycle;
- wrong explicit asset;
- confidential asset.

### 15.5 Terminal vectors

- wrong branch flag;
- omitted control close;
- control close in continuing branch;
- wrong control-close amount;
- omitted residue;
- excess residue;
- residue when zero;
- class components swapped;
- wrong residue tag;
- residue credited as receipt;
- false residue event projection.

### 15.6 Canonical-partition vectors

- entitlement not destroyed;
- entitlement destroyed twice;
- `DIST_CTL` both lateral and destroyed;
- vault input claimed by two flows;
- receipt output funded twice;
- unclassified `U` output;
- sponsor L-BTC compensates U mismatch;
- destruction data output missing or duplicated.

### 15.7 Permissionless vectors

Construct complete settlement in a fresh process/view with:

- public control;
- public vault;
- public entitlements;
- public arithmetic witnesses;
- no entitlement-owner keys;
- no operator key;
- optional constructor-owned sponsor.

Require success.

Attempt a private entitlement opening requirement and require compiler/ABI
rejection.

### 15.8 Layout vectors

- wrong input range count;
- vault-presence shift error;
- entitlement range overlap with sponsor;
- wrong entitlement order/rank;
- output range overlap;
- wrong positive-output mask;
- wrong output prefix rank;
- data-output order mutation;
- wrong coordinator;
- wrong local program;
- continuing ABI used for terminal case or vice versa.

### 15.9 Mixed-program vectors

- entitlement local program paired with another control;
- one entitlement uses another settlement instance;
- one input uses transfer/relabel program;
- sponsor input placed in entitlement range;
- vault local program from another bundle;
- local programs valid individually but global control relation inconsistent.

### 15.10 Model/target projection vectors

For every accepted fixture compare:

- consumed/created object sets;
- canonical deltas;
- receipt owner/class/value multiset;
- successor control counters;
- successor vault;
- destruction outputs;
- residue projection;
- chain fee/open-flow projection.

---

## 16. Measurement plan

### 16.1 Per-input measurements

Measure separately:

- control coordinator continuing;
- control coordinator terminal;
- entitlement local program;
- vault local program;
- sponsor local program;
- receipt/control/vault constructor control paths.

### 16.2 Complete batch-size-2 transactions

For every fixture class, measure:

- transaction weight;
- witness bytes;
- number of inputs/outputs;
- data-output bytes;
- initial stack per input;
- peak stack/altstack;
- maximum stack element;
- crypto budget per input;
- executed opcode/project cost;
- target consensus result;
- policy/mempool result.

### 16.3 Worst-case candidate

At batch size 2, identify separate likely maxima for:

- continuing branch;
- terminal branch;
- all four receipt outputs;
- maximum residue/data outputs;
- maximum sponsor family;
- deepest target control paths;
- largest arithmetic witness set.

### 16.4 Candidate A1 versus B1

Produce one deterministic comparison table:

| Dimension | A1 global coordinator | B1 distributed | Notes |
|---|---:|---:|---|
| Control script bytes | | | |
| Entitlement script bytes per input | | | |
| Total witness bytes | | | |
| Peak stack | | | |
| Crypto budget | | | |
| Transaction weight | | | |
| Positive vector count | | | |
| Negative vector locality | | | |
| Implementation complexity | | | |
| Generalized cost in N | | | |

### 16.5 Scaling measurements

If batch 2 fits, measure candidate counts:

```text
N = 1
N = 2
N = 4
```

and additional values only as resources permit.

Compare measurements with symbolic formulas.

Do not infer linear scaling without checking branch and output-count effects.

### 16.6 Final calibration

This research reports feasibility and formulas.

The release calibration runner determines the final bound across the exact final
bundle and target.

---

## 17. Acceptance criteria

A settlement layout is accepted for production only when every mandatory
criterion passes.

### 17.1 Semantic correctness

- [ ] every entitlement is authenticated and counted once;
- [ ] every entitlement targets the consumed control cycle;
- [ ] per-entitlement floors are exact for both classes;
- [ ] aggregate-before-floor mutation rejects;
- [ ] every positive draw routes to the exact owner/class;
- [ ] zero draws create no protocol output;
- [ ] output uniqueness and completeness hold;
- [ ] entitlement destruction is exact;
- [ ] control/vault predecessor relation is exact;
- [ ] successor counters are exact;
- [ ] continuing/terminal branch is derived, not caller-selected;
- [ ] terminal control closure is exact;
- [ ] terminal residue and class projection are exact;
- [ ] sponsor flow is isolated;
- [ ] all canonical partitions close.

### 17.2 Permissionless constructibility

- [ ] no entitlement-owner signature;
- [ ] no operator signature;
- [ ] no private entitlement opening;
- [ ] every arithmetic witness is public/computable;
- [ ] fresh independent constructor can build the transaction;
- [ ] only sponsor-local secrets are required for sponsor funds.

### 17.3 Target enforcement

- [ ] every relation has a reachable carrier;
- [ ] local/global program consistency holds;
- [ ] wrong-control local programs reject;
- [ ] mapping collisions reject;
- [ ] unclaimed outputs reject;
- [ ] mixed-program vectors reject;
- [ ] target-native positive and negative vectors pass.

### 17.4 ABI canonicality

- [ ] input ranges are deterministic;
- [ ] output ranges are deterministic;
- [ ] optional vault/successor/residue rules are canonical;
- [ ] zero outputs are omitted canonically;
- [ ] entitlement rank/output mapping is authenticated;
- [ ] witness order is canonical;
- [ ] data-output order is canonical;
- [ ] candidate ABI derives entirely from linked typed artifacts.

### 17.5 Resource feasibility

- [ ] batch-size-2 complete transactions fit target hard limits;
- [ ] required policy/standardness passes;
- [ ] predicted and observed resources agree;
- [ ] a scaling/resource formula exists;
- [ ] at least one operationally useful calibrated bound appears feasible;
- [ ] final value remains release calibration output.

### 17.6 Evidence handoff

- [ ] selected candidate and rejected alternatives documented;
- [ ] implementation decision created or updated;
- [ ] compiler placement/layout requirements updated;
- [ ] tapscript patterns stabilized;
- [ ] linker constructor/program roles stabilized;
- [ ] transaction ABI stabilized for settlement;
- [ ] permanent vector families implemented;
- [ ] resource/calibration requirements updated;
- [ ] release evidence census updated;
- [ ] normative review completed for any added receipt metadata/provenance.

---

## 18. Rejection criteria

Reject a candidate layout for the initial backend if any unresolved criterion
below holds.

### 18.1 Semantic rejection

- aggregate-before-floor can pass;
- owner/class misrouting can preserve totals and pass;
- entitlement can be omitted or counted twice;
- output mapping can collide;
- unclaimed receipt output can pass;
- successor counters can diverge from routed outputs;
- terminal residue can be omitted or class-swapped;
- sponsor value can mask protocol mismatch.

### 18.2 Permissionless rejection

- owner/operator secret required;
- private entitlement opening required;
- public constructor cannot obtain required mapping/proof;
- lost owner can block settlement.

### 18.3 Placement rejection

- one required relation has no carrier;
- local programs can use inconsistent control facts;
- control cannot prove local output uniqueness/completeness;
- distributed candidate requires duplicating all arithmetic with no benefit;
- optional branch makes an unconditional relation unreachable.

### 18.4 ABI rejection

- several uncontrolled output mappings pass;
- zero-valued placeholder is required;
- caller can choose arbitrary branch/layout;
- target indexes are unauthenticated;
- output count/range ambiguity remains.

### 18.5 Target rejection

- required target primitive unavailable;
- exact target accepts mandatory negative vector;
- complete transaction cannot be executed under the pinned target;
- wrong local/global program combination passes.

### 18.6 Resource rejection

- batch size 2 exceeds hard target limits;
- batch size 2 cannot pass required policy;
- predicted/observed resources cannot be reconciled;
- only a noncanonical or secret-dependent layout fits;
- no useful bound appears feasible under the selected target.

### 18.7 Normative rejection

- candidate requires new semantic receipt provenance;
- candidate changes owner/class/value observables;
- candidate removes permissionless settlement;
- candidate changes architecture cardinality or object family;
- candidate changes residue accounting

without completed normative versioning review.

A rejected tapscript settlement layout may remain viable on a future backend.

---

## 19. Result

Pending.

When populated, this section must include:

- exact repository revision;
- exact target identity;
- exact wide-arithmetic pattern identity;
- candidate linked bundles and ABIs;
- A1/B1 program and transaction identities;
- fixture/vector census;
- positive/negative result summary;
- relation-carrier matrix;
- predicted/observed resource table;
- scaling formula and measurements;
- permissionless fresh-process construction result;
- failed candidate reasons;
- selected/rejected verdict;
- canonical report hashes.

Required branch summary:

| Fixture | A1 result | B1 result | Expected semantic result |
|---|---|---|---|
| Continuing, both classes positive | | | |
| Continuing, mixed zero draws | | | |
| Repeated owner | | | |
| Terminal, zero residue | | | |
| Terminal, live residue | | | |
| Terminal, time-locked residue | | | |
| Terminal, both residues | | | |
| No predecessor vault | | | |

Do not claim a general batch bound from batch-size-2 success alone.

---

## 20. Decision and implementation handoff

Pending.

Expected handoff if A1 succeeds:

```text
D010: Use a Distribution-Control Coordinator with Canonical
Per-Entitlement Receipt Output Ranges for Elements Settlement
```

Expected handoff if B1 succeeds:

```text
D010: Distribute Settlement Floor and Routing Checks Across Entitlement
Inputs with Control-Coordinator Aggregate Closure
```

The accepted decision must define:

- input family order;
- output family order;
- continuing/terminal target programs;
- coordinator role;
- entitlement rank rule;
- positive-output mapping rule;
- local/global relation placement;
- witness ABI;
- closed-asset partition;
- data-output order;
- resource formula;
- calibration handoff;
- permanent vector requirements.

If Candidate E provenance is selected, first complete normative denotation and
versioning review.

If no candidate succeeds:

- record exact resource/safety failure;
- evaluate another target/backend;
- evaluate whether a different target proof primitive exists;
- do not silently reduce semantic settlement guarantees;
- do not accept owner-assisted settlement;
- begin normative review only if implementation evidence demonstrates a true
  target incompatibility rather than ordinary engineering difficulty.

---

## 21. Residual risks

Even an accepted layout will retain residual risks.

### 21.1 Target serialization pressure

Settlement may remain one of the largest and most complex transaction
families.

### 21.2 Coordinator contention

Distribution control serializes settlement for one cycle by design.

This is shared-state liveness, not an invariant failure.

### 21.3 Conservative calibrated bound

The final batch bound may be much smaller than the architecture draft default.

This is acceptable when deployment calibration preserves semantic minima and
liveness remains sufficient.

### 21.4 Complex independent implementation

External wallets and auditors must implement the canonical settlement ABI
correctly.

Canonical vectors and typed schemas reduce but do not remove this risk.

### 21.5 Arithmetic hand-audit surface

Repeated wide-floor proofs retain the accepted arithmetic pattern's residual
assurance limits.

### 21.6 Public entitlement values

The initial profile provides no entitlement amount privacy.

This is an accepted representation limitation, not a hidden privacy claim.

### 21.7 Output fragmentation

A per-entitlement no-aggregation layout may create more receipt outputs than an
aggregated model implementation.

This affects weight and UTXO growth but not semantic correctness.

### 21.8 Future optimization migration

Later owner/class aggregation or another backend may use a different ABI.

Migration must preserve semantic relation and evidence while changing bundle
and ABI identities.

---

## 22. References

### Normative and current source

- [`../../docs/attestation/realization.md`](../../docs/attestation/realization.md)
- [`../../packages/architecture/src/spec.rs`](../../packages/architecture/src/spec.rs)
- [`../../packages/model/src/object.rs`](../../packages/model/src/object.rs)
- [`../../packages/model/src/recognition.rs`](../../packages/model/src/recognition.rs)
- [`../../packages/model/src/kernel.rs`](../../packages/model/src/kernel.rs)
- [`../../packages/model/src/certify.rs`](../../packages/model/src/certify.rs)
- [`../../packages/model/src/shape.rs`](../../packages/model/src/shape.rs)
- [`../../packages/model/src/invariant.rs`](../../packages/model/src/invariant.rs)
- [`../../packages/model/src/ops/settlement.rs`](../../packages/model/src/ops/settlement.rs)
- [`../../packages/model/src/tests/settlement_tests.rs`](../../packages/model/src/tests/settlement_tests.rs)
- [`../../packages/model/src/tests/distribution_bijection_tests.rs`](../../packages/model/src/tests/distribution_bijection_tests.rs)
- [`../../packages/model/src/tests/distribution_fixtures.rs`](../../packages/model/src/tests/distribution_fixtures.rs)
- [`../../packages/model/src/tests/residue_noninterference_tests.rs`](../../packages/model/src/tests/residue_noninterference_tests.rs)

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

### Related research

- [`wide-arithmetic.md`](wide-arithmetic.md)
- [`state-object-constructor.md`](state-object-constructor.md)
- [`public-declassification.md`](public-declassification.md)

### Target reference

- [`../reference/elements-tapscript.md`](../reference/elements-tapscript.md)

### Roadmap

- [`../roadmap.md`](../roadmap.md)

---

## 23. One-line research contract

> Build and compare exact batch-size-2 Elements settlement candidates that
> authenticate every control, vault, entitlement, receipt, destruction, and
> sponsor family; apply each class floor per entitlement before aggregation;
> preserve owner/class routing and control/vault counters; derive continuing or
> terminal closure and residue exactly; remain permissionlessly constructible;
> assign every relation to a reachable local or global carrier; and fit whole
> target transactions—then accept no general settlement ABI or calibrated batch
> bound until one canonical layout passes the full semantic, adversarial,
> constructibility, target-native, and resource gates.
