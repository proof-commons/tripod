# D005: Value-Parametric, Closed-Asset-Identity-Rigid · `dec:representation:value-parametric`

> **Status:** Accepted
> **Class:** Representation
> **Depends on:** (`dec:architecture:realization-layer`),
> (`dec:backend:tapscript-first`),
> (`dec:assurance:translation-validation`)
> **Imports:** (`[RZ-sec:realization:representation]`),
> (`[RZ-obl:oracle:representation]`),
> (`[RZ-obl:oracle:disclosure]`)
> **Supersedes:** none

## Choice · `rule:representation:choice`

Treat value as a target-independent semantic amount whose concrete proof may
vary.

For the initial Elements backend:

```text
value representation:
    may vary where a supported proof preserves semantics

closed protocol asset identity:
    explicit and rigid at every protocol seam
```

Conceptual value modes are:

| Mode | Amount public | Commitment algebra retained |
|---|---:|---:|
| private committed | no | yes |
| public committed | yes, through authenticated opening | yes |
| explicit | yes, directly encoded | no value blinding term |

These modes denote the same semantic value.

## Closed asset rule · `rule:representation:closed-assets`

The initial Elements backend must explicitly classify:

```text
U
ENT
DIST_CTL
PID
PACE
ENT_AUTH
DIST_AUTH
```

No confidential or unclassified asset output may silently carry one of these
assets.

Confidential value conservation does not prove object-family or recipient
closure.

## Proof alternatives · `rule:representation:proofs`

A semantic relation may admit target-independent proof alternatives such as:

- explicit arithmetic;
- confidential value conservation;
- commitment equality;
- authenticated opening;
- owner-authorized normalization.

A backend may select an alternative only when:

- target capability exists;
- semantic relation is preserved;
- closed assets remain classified;
- required witness is available;
- permissionless construction remains possible;
- lifecycle exits remain reachable;
- resource and deployment policy permit it.

No proof alternative is invented merely because a target lacks the preferred
one.

## Public availability · `rule:representation:availability`

A value required by:

- public state;
- public event or audit;
- public interface;
- permissionless construction

must be publicly and authentically available.

Public availability may use:

- explicit encoding;
- public commitment plus authenticated public opening.

Unauthenticated metadata amount is never sufficient.

## Permissionless rule · `rule:representation:permissionless`

A permissionless operation cannot require:

- another owner’s private opening;
- another owner’s blinding factor;
- an operator secret;
- unpublished metadata;
- a private database.

A target program being able to verify a secret does not make that secret
publicly constructible.

## Lifecycle rule · `rule:representation:lifecycle`

Every supported representation retains required exits.

Examples:

```text
live receipt:
    transfer, burn, redeem

time-locked receipt:
    transfer, post-maturity relabel

ASH:
    compact, clear

request:
    cancel, admit

entitlement:
    settle
```

A representation that creates a valid object but strands one required exit is
unsupported.

Normalization must be an explicit authorized value-preserving path, not a
wallet convention.

## Disclosure rule · `rule:representation:disclosure`

Semantic disclosure derives from typed relation dependencies.

Compiler analysis distinguishes:

1. semantic disclosure;
2. target-safety disclosure;
3. deployment-policy disclosure.

Safety and minimality remain separate claims.

- Rejecting unauthorized transactions supports safety.
- Accepting a supported lower-disclosure representation supports minimality.

Neither substitutes for the other.

## Initial operation posture · `tbl:representation:initial`

| Operation | Initial posture |
|---|---|
| compact ASH | public/openable ASH value; explicit closed `U` |
| live transfer | explicit or private committed value; explicit closed `U` |
| burn | public fresh ASH aggregate; source/change privacy prototype-dependent |
| clear | public/openable ASH and public state |
| redemption | public amount-dependent effects; direct opening or normalization prototype-dependent |
| settlement/cycle | public initial profile |

This table guides implementation. Typed realization and accepted research
results remain authoritative over the final backend plan.

## Does not authorize · `sec:representation:limits`

This decision does not authorize:

- confidential closed protocol asset identity;
- blanket confidential values;
- blanket explicit values while claiming minimality;
- unauthenticated public openings;
- owner-assisted permissionless maintenance;
- arbitrary foreign sponsor assets;
- changing recipient, authorization, formula, observable, or lifecycle through
  representation;
- claiming universal transaction privacy.

## Supersession · `rule:representation:supersession`

Confidential closed-asset identity requires a new decision proving:

- exact asset-class authentication;
- complete object closure;
- issuance and authority integrity;
- no hidden closed-asset escape;
- constructibility;
- lifecycle;
- target-native safety evidence.

## Verification · `gate:representation:evidence`

The decision is demonstrated when live transfer accepts a valid
confidential-value representation with the same semantic projection as the
explicit form, while every confidential or unclassified closed-asset escape
rejects independently.
