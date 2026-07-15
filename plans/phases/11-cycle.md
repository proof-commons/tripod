# Phase 11 — Cycle · `phase:roadmap:cycle`

> **Status:** Planned
> **Entry:** (`gate:phase6:exit`), (`gate:phase8:exit`),
> (`gate:phase9:exit`), (`gate:phase10:exit`)
> **Operation:** `cycle`
> **Reason for ordering:** combines nearly every prior seam

## Goal · `sec:phase11:goal`

Implement the cycle relation after STATE continuity, cadence, wide arithmetic,
issuance, distribution construction, maturity conversion, transaction ABI, and
resource measurement are established independently.

## Deliverables · `sec:phase11:deliverables`

### Root relations

Implement succession of:

- STATE;
- RESV;
- PACE;
- distribution authority.

No entitlement-authority use.

### Cadence band

Enforce:

```text
age < MIN:
    invalid

MIN <= age < MAX:
    operator-authorized

age >= MAX:
    permissionless
```

Every cycle consumes and recreates PACE, including empty cycles.

### State arithmetic

Implement:

```text
Ω' = Ω + Q
Q' = 0
cycle' = cycle + 1
ΔY = floor(Q * Y / Ω)
```

Require active-backing cap and floor preservation.

### Class and fee split

Before maturity:

```text
ΔY_L = floor(ζ * ΔY)
ΔY_T = ΔY - ΔY_L
```

After or at conversion:

```text
ΔY_L = ΔY
ΔY_T = 0
```

Apply the formula-fixed operator fee separately to each active class.

### Issuance

Implement:

- exact `U` issuance under PACE;
- exact `DIST_CTL` issuance under distribution authority when `Q>0`;
- no issuance when inactive;
- destination exhaustion;
- no undeclared minted output.

### Distribution creation

When `Q>0`:

- create one control;
- create vault iff contributor allocation is positive;
- initialize exact principal, allocations, and remainders.

When `Q=0`:

- create no control or vault;
- issue no `U` or `DIST_CTL`;
- still advance roots and cycle.

### Maturity conversion

When the next cycle equals announced maturity:

```text
Y_L' = Y_L + Y_T + ΔY
Y_T' = 0
maturity' = complete
```

Conversion occurs atomically, including an empty cycle.

Create the configured zero-value CPFP anchor only under the maturity condition.

### Open flows

Enforce:

- RESV carry;
- optional isolated sponsor;
- exact chain fee;
- no sponsor effect on reserve or issuance.

## Required vectors · `sec:phase11:vectors`

### Cadence

- before minimum;
- at minimum with/without operator;
- one below maximum;
- at maximum permissionless;
- invalid sequence/version;
- hidden operator gate on delayed branch.

### Empty cycle

- roots advance;
- cycle advances;
- no issuance/control/vault;
- empty maturity cycle converts atomically.

### Nonempty cycle

- exact issuance;
- wrong quotient;
- wrong class split;
- wrong fee split;
- wrong operator receipt class;
- wrong issuance destination;
- missing authority;
- wrong control/vault;
- active-backing boundary.

### Maturity

- before maturity;
- exact maturity;
- post-maturity all-live issuance;
- conversion omitted;
- conversion repeated;
- wrong anchor presence/absence.

### Root and open flow

- missing/wrong root;
- wrong RESV carry;
- sponsor interference;
- stale PACE;
- mixed cadence programs;
- wrong successor constructor.

## Resource evidence · `sec:phase11:resources`

Measure separate likely maxima for:

- operator-band branch;
- delayed permissionless branch;
- empty cycle;
- nonempty pre-maturity cycle;
- maturity conversion cycle;
- post-maturity cycle;
- maximum sponsor candidate;
- full issuance/control/vault output set.

## Target dependency evidence · `sec:phase11:target-evidence`

Require verified reports for:

- relative timelock behavior;
- selected sighash profile;
- issuance/reissuance and introspection;
- explicit value/asset introspection;
- constructor continuity;
- target resource/policy limits;
- package relay for the CPFP deployment claim.

## Exit gate · `gate:phase11:exit`

Phase 11 exits when:

- cadence implements all three regimes;
- delayed cycle is genuinely permissionless;
- every required root succeeds exactly once;
- empty and nonempty cycles match the model;
- wide issuance arithmetic is exact;
- issuance amounts and destinations exhaust;
- distribution creation is exact;
- maturity conversion is atomic;
- anchor condition is exact;
- complete transaction resources fit calibrated candidates;
- target dependency reports pass;
- relation coverage is complete and all outputs reproduce cleanly.
