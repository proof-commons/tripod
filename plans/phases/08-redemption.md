# Phase 8 — Wide Arithmetic and Redemption · `phase:roadmap:redemption`

> **Status:** Planned
> **Entry:** (`gate:phase7:exit`) and accepted wide-arithmetic decision
> **Packages:** tapscript, linker, transaction, vectors
> **Operation:** `redeem`

## Goal · `sec:phase8:goal`

Integrate exact wide floor arithmetic into a formula-bound owner redemption and
the RESV succession/termination relation.

## Deliverables · `sec:phase8:deliverables`

Implement:

```text
p = floor(x * Ω / Y)
```

with:

- canonical STATE and active RESV inputs;
- one live receipt input;
- receipt owner authorization;
- authenticated amount `x`;
- exact wide arithmetic proof;
- formula-bound L-BTC payout to the owner;
- receipt `U` destruction under `tag-redeem`;
- exact STATE decrement;
- RESV successor for nonterminal case;
- RESV termination for exact sealing case;
- optional isolated sponsor flow;
- transition-certificate root edges.

## Terminal rule · `rule:phase8:terminal`

Sealing is valid only when:

```text
x = Y
Q = 0
p = Ω
```

Then:

- STATE becomes sealed;
- RESV is exhausted and terminates;
- no RESV successor exists.

Otherwise:

```text
x < Y
```

and RESV must succeed.

## Required vectors · `sec:phase8:vectors`

### Arithmetic

- exact division;
- nonzero remainder;
- quotient one below;
- quotient one above;
- zero divisor;
- malformed width/limb/carry;
- near-domain maximum;
- result overflow;
- wrong semantic operand binding.

### Authorization and class

- valid owner;
- wrong/missing owner;
- time-locked receipt;
- wrong receipt asset/object;
- confidential closed asset.

### Recipient and flow

- correct payout;
- payout redirected;
- payout shortened while another output grows;
- sponsor value masking payout mismatch;
- wrong reserve successor value;
- wrong state decrement;
- missing/duplicate destruction.

### Terminal

- exact sealing;
- `x=Y` with pending `Q`;
- RESV termination while state remains operational;
- RESV successor in sealed case;
- later STATE/RESV revival attempt.

## Representation · `sec:phase8:representation`

Support one accepted path for every enabled live-receipt representation:

- direct authenticated opening; or
- owner-authorized normalization; or
- explicit-only initial profile.

Release and client documentation must name the actual path.

## Resource evidence · `sec:phase8:resources`

Measure:

- readable and optimized arithmetic patterns;
- nonterminal redemption;
- sealing redemption;
- representation/opening or normalization path;
- maximum sponsor candidate;
- complete transaction target/policy costs.

## Exit gate · `gate:phase8:exit`

Phase 8 exits when:

- wide arithmetic pattern matches independent reference results;
- target-native boundary/property vectors pass;
- authenticated operands feed the correct payout and state relations;
- recipient and sponsor isolation are exact;
- terminal and nonterminal edges match the model;
- post-sealing revival rejects;
- enabled representations retain redemption lifecycle;
- complete transaction resources pass;
- relation coverage and deterministic reports are complete.
