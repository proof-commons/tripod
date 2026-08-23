# `tripod-transaction`

Derives the candidate compact-ASH transaction ABI from a candidate
linked bundle and the exact reviewed target, and constructs ABI-valid
target transactions from it (Guide-12 §15).

The package owns the canonical role layout, the typed operation request
and the public construction view, sponsor signing *requests*, witness
and control-block assembly, and a first-party encoder and decoder for
explicit-field target transactions. That last one is owned here because
the reviewed substrate decision selected first-party structures over a
third-party library, and named exactly these capabilities as the ones
this workspace would then have to own.

The package holds no private key, signs nothing, blinds nothing, and
performs no RPC, wallet lookup, or network submission. A sponsor
supplies signatures through a capability adapter, and every signing
request names a finalized transaction rather than a template.

Nothing here is final. The output is a `CandidateTransactionAbi` and a
`CandidateCompleteTransaction`, both of which carry a read-only
candidate status and a structurally non-empty obligation set, because
Guide-12 §1.9 keeps the candidate and final states distinct and the
taproot output key this crate's control blocks depend on is pinned
rather than recomputed.

## The live-transfer surface (Guide-13 §12)

The same shape one operation later: a typed `LiveTransferRequest` with
six closed fields, a `CandidateLiveTransferAbi` derived from a linked
live bundle, and `finalize_live_transfer`, which settles §12.6's ten
items and refuses at whichever stage a fault reaches.

What the request *cannot* say is the load-bearing part. Fourteen facets
are structurally absent — no field, no argument, no setter — rather than
present and refused, and the crate's tests census all fourteen. A caller
cannot name a program, a constructor, a control path, an internal key, or
a spending route, which is why a mixed-operation program is not an input
this pipeline can be handed.

Before those ten items, the ABI must carry the plan the request selected.
A one-representation link is legitimate, and the request selects its plan
without ever seeing the ABI, so the two can disagree; that disagreement
is its own refusal rather than being met four stages later as a receipt
that fails to be recognized. The owner-specific refusal beside it keeps
its own case: the plan is linked and one destination owner has no
constructor in it.

The private form is central public-fixture construction and records that
in its own model. It consumes randomness the caller has already
published, generates none, stores none, and offers no interface a
production secret could arrive through.
