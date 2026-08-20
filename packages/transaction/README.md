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
