# Phase 9 — Requests, Cancellation, and Admission · `phase:roadmap:admission`

> **Status:** Planned
> **Entry:** (`gate:phase6:exit`), (`gate:phase8:exit`)
> **Packages:** realization, compiler, tapscript, linker, transaction, vectors
> **Operations:** `create-request`, `cancel-request`, `admit-deposits`

## Goal · `sec:phase9:goal`

Implement the open request lifecycle and convert valid deposits into scarce
entitlements and active backing without adding depositor-dependent shared-state
liveness.

## Deliverables · `sec:phase9:deliverables`

### Request construction

Provide a canonical client-policy builder for:

- owner-authorized L-BTC inputs;
- one request output;
- pool ID;
- refund key;
- receipt owner;
- principal;
- service budget;
- optional owner change;
- explicit chain fee.

Malformed request-shaped open outputs remain possible and inert.

### Cancellation

Implement:

- one request input;
- refund-key authorization;
- full request value returned to refund key;
- no receipt-owner substitution;
- separate sponsor-funded fee;
- no pool root.

### Admission

Implement:

- nonempty bounded valid request family;
- request validation at consumption;
- target pool match;
- exact principal and budget partition;
- one entitlement per request;
- `ENT_AUTH` succession;
- exact `ENT` issuance;
- STATE and RESV succession;
- exact `Q` and reserve updates;
- active-backing cap;
- bounded reward and chain-fee split;
- permissionless construction;
- no request-owner signature.

## One-entitlement rule · `rule:phase9:one-entitlement`

Admission emits one entitlement per request before any later floored operation.

Pre-floor request aggregation is prohibited because flooring is not
aggregation-invariant.

## Required vectors · `sec:phase9:vectors`

### Request

- canonical valid request;
- malformed principal equal to gross;
- zero principal;
- cross-pool request;
- wrong funder/owner authorization;
- request change and fee variants.

### Cancellation

- correct refund key;
- receipt owner but not refund key;
- partial refund;
- redirected refund;
- sponsor fee reducing refund;
- duplicate request input.

### Admission

- one and several requests;
- one entitlement per request;
- wrong owner or target cycle;
- merged entitlement;
- missing/extra entitlement;
- wrong `ENT` issuance;
- missing authority;
- wrong STATE/RESV update;
- exact active-backing cap;
- one above cap with requests unspent;
- reward equal to budget;
- reward one above budget;
- private/unavailable request facts on permissionless path;
- sponsor/open-flow interference.

### Dust boundary

- positive `Q` with later `ΔY=0`;
- entitlements retire at zero draw;
- principal enters reserve;
- no false receipt output.

This accepted residual must remain visible to clients.

## Representation · `sec:phase9:representation`

Initial admission facts are public or publicly authenticated.

A request representation is supported only if it retains both:

```text
cancel
admit
```

without hidden participant secrets on admission.

## Resource evidence · `sec:phase9:resources`

Measure:

- maximum request creation candidates;
- cancellation with sponsor;
- admission batches under candidate bound;
- authority and root constructor paths;
- entitlement outputs;
- reward/fee variants.

## Exit gate · `gate:phase9:exit`

Phase 9 exits when:

- canonical client builder works without pretending all requests are valid;
- cancellation refunds full value to the correct key;
- admission remains permissionless;
- one entitlement per request is exact;
- authority issuance and STATE/RESV succession match the model;
- active-backing cap and reward envelope are enforced;
- malformed open requests remain inert before consumption;
- representation lifecycle for request cancel/admit is complete;
- target resources and relation coverage pass;
- artifacts and reports remain deterministic and clean.
