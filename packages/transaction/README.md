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

## The state-thread anchor (Guide-14 Wave 4)

`StateThreadAnchor` is typed provenance for one caller-anchored
predecessor: deployment and branch context, starting outpoint, asset or
issuance provenance, tapscript constructor generation, checkpoint policy,
continuity-evidence standing, and an explicit synthetic or observed origin.
Its fields are private and construction checks their shape (`0.6.177-dev`).

`check_continuations` states and tests the conditional theorem: given one
uniquely anchored predecessor, each selected branch has at most one accepted
continuation, and two accepted continuations on the same branch are refused
as equivocation. Acceptance remains caller-supplied evidence rather than a
target observation verified here.

A positive result makes no genesis or global-origin claim. In particular, a
synthetic origin is not protocol genesis, trusted setup, earlier history, or
production STATE; accepted continuity and history evidence remain for Waves
9 and 10.

## The typed maturity-announcement request (Guide-14 §12.3)

`MaturityAnnouncementRequest` is three closed fields — the announced cycle, the requested transaction form, and the inherited sponsor-change choice — and two walkable censuses that are its contract: three selectable facets, and fifteen unselectable ones drawn from the union of the execution and semantic request sections, each naming where its value is actually settled, whether that is the public view, the linked bundle, the deployment, the constructor search, or the target's own verdict. Those fifteen are absent structurally rather than present and refused, so no caller can name a successor program, an internal key, a control block, or a root cursor; both transaction forms stay representable, because a request that could not ask for a sponsor region could not be refused for asking, while the one pair that contradicts itself — a sponsorless form taking sponsor change — is refused by the request itself.

## Authenticated operator membership (Guide-14 Wave 5)

`produce_operator_membership` consumes an explicit
`OperatorMembershipMapping`, a frozen `OperatorMembershipRequest`, the verified
`OperatorAuthorizedCandidate`, the affine `OperatorRightRegistry`, the reviewed
target revision, and a nonempty run description. The mapping states one
deployment binding, one established operator profile, and exactly one
realization operation; the request carries the matching frozen signing request,
operation assignment, and still-live construction right (`0.6.181-dev`).

The producer first checks that the operation, authorization, revision, right
scope, and run agree, then consumes the right. A deployment, key, profile, or
revision mapping disagreement produces a checked non-member decision only after
that authenticated consumption; it never turns a bad or stale authorization
into evidence. `OperatorMembershipRefusal` names operation scope or mismatch,
authorization mismatch or revision, right scope or registry failure, empty run,
and provenance failure, while `OperatorMappingRefusal` names the four checked
non-membership grounds.

The emitted `ObservedOperatorMembership` names
`transaction::produce_operator_membership` and the supplied run in its
provenance. Its operation assignment remains a caller assertion until Wave 7's
observation boundary binds it to finalized bytes. The transaction package card
admits the direct realization dependency for this vocabulary without admitting
a parallel semantic table: the producer compares authenticated identities and
evaluates no realization formula.

## The public current-STATE view (§12.4)

`PublicMaturityStateView` carries §12.4's ten entries about one current STATE output — the outpoint, its exact asset and amount, the canonical predecessor metadata, the representation nonce, the branch and checkpoint the current root is bound at, the actual predecessor program, the current cycle, the accepted linked maturity bundle, the operator's committed public identity and the deployment's resolved symbols — of which seven are stated as a stream of `MaturityViewStatement` values and three are read from the entries they are facts of, so a second statement of a stated entry is refused by name through `MaturityViewEntry` while the derived three have no second statement to disagree with; there is no setter, so nothing overwrites a stated entry afterwards. `validate` checks the one relation a first party can check without observing a chain: the supplied metadata and nonce, encoded canonically and committed under the accepted bundle's own constructor policy over the linked static subtree, must reproduce the supplied predecessor program, which is the announcement leaf's own authentication equation computed here rather than taken on the caller's word — a pair that commits to nothing refuses carrying the constructor's refusal, and a pair that commits elsewhere refuses with both program lengths and no key bytes. What validation does not establish it names: `ValidatedMaturityStateView::residual` is `CurrentStateRootFreshness`, because whether that outpoint is still the current root is a fact about a chain that nothing in this crate observes, and the linked bundle's own outstanding obligations are unchanged by what this layer computed.
