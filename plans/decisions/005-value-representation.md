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

## Initial operation posture · `tab:representation:initial`

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

## Sponsor-value opacity · `rule:representation:sponsor-value-opacity`

Decided 2026-07-24 (closing finding F2-006); the load-bearing statement lives
in the realization — the inspection-necessity rule, the inspection-burden
counterexample criterion, the sponsor-erasure projection, and the sponsor
opacity proof in the kernel section.

The exact value of an ordinary sponsor L-BTC input or output is sponsor-local
data, not a protocol-readable fact. A protocol predicate, compiler-selected
proof, emitted target program, canonical public report, or release identity
must not require an individual sponsor amount to be explicitly encoded for
protocol use, decoded, opened, compared with zero, proved strictly positive,
aggregated as a public integer, or emitted in a public diagnostic or evidence
field — on either side of the transaction.

Sponsor-value safety derives from exact L-BTC asset authentication; ordinary
sponsor-family recognition; exact sponsor-region membership (zero-valued
members included); source and destination reference uniqueness;
sponsor/protocol reference disjointness; authorization by every sponsor input
owner; output-committing signature semantics where required; at most one
generic sponsor envelope; substrate-enforced exact value conservation,
commitment balance, or another approved exact proof; and independent
enforcement of every protocol payout, refund, reserve, issuance, state,
destruction, recipient, and event relation. A zero-valued sponsor member is
accepted by the target-independent semantic relation when those conditions
hold; its presence may affect transaction shape, resources, wallet policy, or
deployment-policy acceptance, never a protocol semantic result.

Positivity was shown not to be a security boundary: balanced theft (protocol
value short one unit, sponsor change up one, all sponsor amounts positive,
totals conserved) passes any positivity check and is rejected by the pinned
protocol relation; the vector is a permanent regression in both pilots.

Model and realization state one rule: the model kernel's open-flow membership
is family-based (every non-anchor L-BTC member claimed exactly once,
zero-valued included) and it neither requires nor forbids a zero-valued
ordinary output — zero value is only meaningful for CPFP anchoring or another
colored-output scheme outside this protocol, and is otherwise pointless
(weight, dust, likely nonstandard under default relay policy), which is
wallet economics recorded as a comment, not a check. The first-party builder
normalizes: a requested zero sponsor change is omitted rather than emitted. A
deployment policy may separately reject nonstandard zero outputs as policy.
CPFP_ANCHOR remains recognized by its declared family — it stands outside the
open-flow partition and can neither satisfy nor join a sponsor role — never
by testing an ordinary output for zero. The realization enforces the read-set
structurally: deriving a scoped realization fails if any expression,
disclosure node, declassification entry, or constructibility fact names a
PLAIN_LBTC family amount.

## Does not authorize · `sec:representation:limits`

This decision does not authorize:

- confidential closed protocol asset identity;
- blanket confidential values;
- blanket explicit values while claiming minimality;
- unauthenticated public openings;
- owner-assisted permissionless maintenance;
- arbitrary foreign sponsor assets;
- reading, opening, or comparing an individual sponsor amount as a protocol
  fact;
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
