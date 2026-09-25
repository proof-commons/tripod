# `tripod-transaction`

Derives the compact-ASH candidate transaction ABI from a candidate linked bundle and the exact reviewed target (Guide-12 §15), and the maturity-announcement candidate ABI from a validated current-STATE view of its linked bundle and the exact reviewed target (Guide-14 §12), then constructs ABI-valid target transactions from those ABIs.

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

Nothing here is final. Compact ASH yields `CandidateTransactionAbi` and `CandidateCompleteTransaction`, both with read-only candidate status and structurally non-empty obligations; the maturity announcement yields `CandidateMaturityAnnouncementAbi` and `MaturityConstruction` within its candidate construction path. Guide-12 §1.9 keeps candidate and final states distinct, and the taproot output key on which the compact-ASH control blocks depend is pinned rather than recomputed.

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

A positive result makes no genesis or global-origin claim. In particular, a synthetic origin is not protocol genesis, trusted setup, earlier history, or production STATE; accepted-byte continuity evidence is admitted on `0.6.245-dev`, its row classification belongs to `T11-099`, and branch-history evidence remains with Wave 10.

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

## The candidate maturity-announcement ABI (§12.1, §12.2, §12.6, §12.9)

`derive_maturity_announcement_abi` takes the reviewed target and a *validated* view rather than a bare bundle, because the view's type is the evidence that the reconstruction half of the link's current-state obligation was computed rather than asserted, and the prototype bundle is read through it. The result is a `CandidateMaturityAnnouncementAbi`: a single-variant status read and never written, no digest field and no accessor that would return one, and nothing final anywhere. Its two layouts carry no shape index — one STATE input and one STATE successor is the whole family, and the optional sponsor suffix is stated as the index it begins at rather than as a range no count exists for — and they are the only place §12.1 and §12.2 are true, since the reduced leaf pins input zero and output zero and states none of the rest. The seven witness roles carry §12.6's seven attributes with four of them read rather than restated: the declared encoding, its minimum and maximum width and the canonical position come from the composed record's own schedule, and the consuming component comes from the linker's carrier attribution, so the successor nonce reports both of its two true facts — the type the copy-through component declares and the successor reconstruction that reads it. The relay verdict is the reviewed target's own, delegated rather than copied: consensus admits both schedules; the whole-item schedule's eighty-six-byte metadata item exceeds the eighty-byte initial-item bound and remains refused by default relay, while the variable-region schedule carries fifty-three bytes at that position and the form review admits it under the reviewed argument-width policy. The fresh variable-schedule submission is accepted in the disclosed zero-fee-floor environment on `0.6.245-dev`. The link's five obligations are partitioned into two discharged and three carried by two named sets checked against the bundle's own, so an obligation a later wave adds is refused rather than dropped, and each ABI's structurally non-empty outstanding set carries the freshness residual as its least member, the historical whole-item schedule's relay-witness residual or the variable schedule's linked-width-measurement residual beside it, and one member per carried obligation in the link's own words.

## Constructing the maturity announcement (§12.5, §12.7)

`construct_maturity_announcement` takes the reviewed target, the candidate ABI, a *validated* view, a typed request and a curve capability, and runs the first seven of §12.5's nine steps in order: it refuses the sponsored form by name because no leaf and no model constrains those bytes yet, holds the ABI against the target's revision and the bundle against the prototype status, checks the view's stated asset and amount against the deployment's own resolved definitions, derives the successor semantic metadata through the realization's one maturity transition over the deployment's lead window, applies the linked bundle's constructor — the single owner of the nonce search, its budget and its leastness evidence, which is also why steps three to five are one call — holds the continuity equality against the application the link itself retained, and fixes output zero. Each borrowed refusal travels whole rather than flattened, so an exhausted search, a nonce reconstructing the wrong program and a later admissible nonce beyond the budget stay three distinct findings, the last being the residual of a search that succeeded. Output zero carries the deployment's asset and declared issuance rather than the caller's statement of them, its program is the successor constructor's and its nonce field is the null one an unblinded output carries; the sponsorless candidate is therefore exactly one input and one output, conserving the singleton amount with no fee output; consensus admits both schedules, the whole-item form retains its default-relay width refusal, and the variable-region form is admitted under the reviewed argument-width policy and has an accepted zero-fee-floor run on `0.6.245-dev`. The result is `MaturityConstruction`, §12.7's first state, in which a caller-supplied nonce result or output program is unrepresentable — no field, no argument, no setter — and which stops where finalization begins: freezing the protected bytes and creating the operator signing request are the next state's steps, and this construction is what that state finalizes.

## Finalizing the maturity announcement (§12.7, §13.1, §13.2)

`finalize_maturity_announcement` takes a construction and returns `FinalizedMaturityAnnouncement`, §12.7's second state, taking the protected bytes exactly once and comparing against them ever after rather than recomputing them; it is infallible, and that is not a lost refusal, because every value it settles is a read off something the construction already fixed and what can still refuse is the signing request. The value owns the construction whole, so the successor constructor, the metadata, the request and the validated view stay readable through it and the protected transaction is read rather than held a second time. Three censuses hang off it: an output census holding the settled outputs beside the transaction, with the successor at zero and the sponsor-change and fee positions absent because the form has no sponsor region and needs no fee output to conserve the singleton amount; a spent-output record carrying the view's outpoint and its exact asset, value and program, which derives the census entry §13.1 binds rather than storing a second copy of it; and an eleven-member settled-fact census over §12.7's regions, eleven for twelve because the executing leaf data is not something these bytes carry — it is a fact about the predecessor's committed tree, and the signing request is what fixes it. `check_offered` names the region that moved where it can, reusing the three post-signing refusals for the input census and the outputs and naming the version and the lock time by their own regions rather than as an output position that did not move, and closes over the exact protected encoding, which in this form catches the input's sequence field, since inputs are compared by outpoint and a sequence can move without the census changing size; the comparison is over the witnessless encoding, which for this candidate is byte for byte the full one, so the check keeps holding once the next state inserts the operator witness. `signing_request` builds the request through `OperatorSigningRequest::freeze` from this one value, so all nine terms of §13.2 derive from one finalized candidate: the candidate structure and its bytes are the finalized ones, the spent-output census is the record's own three fields, the input index is the record's position, the leaf hash, leaf version and control block come from the predecessor constructor's control recipe for the announcement role — the committed static subtree's leaf, not the link's relocated program, because the tapleaf hash the control block authenticates was taken over the committed bytes — the code-separator position is the operator profile's, and the deployment seed is the binding's own genesis in the byte order the message uses. There is no parameter, argument or setter through which a leaf, a byte string or a spent output could arrive from anywhere else, which is a stronger statement than refusing one that did; and the chain stops here, with no signature computed, nothing submitted and no state past this one named.

## Signing the maturity announcement (§12.7, §13.3, §13.4)

`OperatorSigningStarted::open` freezes the request once through the finalized form's own builder, and the three states after it each borrow that one finalized candidate, because a request borrows the operator binding inside it and a second freeze would give one rule two places to disagree — which also makes the last state's standing structural, since a submit-ready value cannot outlive the candidate it is a state of. The chain is STATE-specific and shaped on the live generation's rather than extending it: the live types carry owner plurality and confidential-proof operations with no STATE subject here, and an extension would have had to represent the absence of all of them. `MaturityProtectedRegion` is §12.7's twelve regions in the guide's order, and the started state spells one operation per member, each taking the value a caller would offer and refusing by naming its region before reading it, because a mutation that is unrepresentable cannot be *shown* to reject and a census readable only in prose stops being evidence the first time a member arrives without an entry. Authorization runs through `authorize_operator_under_right` and nothing else, so every accepted signature is one the registry observed; a refusal returns its token beside the refusal, since the registry leaves a refused scope outstanding precisely so it can be consumed again and a signature that dropped the token would let one malformed response lock a scope for good. `check_offered` names §13.4's four post-signing mutations — the successor metadata, the successor program, a sponsor input and the fee role — most specific first and then falls back to the finalized form's own comparison, taking the offered transaction together with the successor metadata it claims to commit to, because semantic metadata reaches the bytes only through the constructor and a moved metadata and a swapped program are otherwise one differing program; they are named rather than left to the exact echo because an echo that disagrees is evidence that something moved, not that a named mutation rejects. `bind_for_submission` populates the STATE input's witness by walking the ABI's own witness records, so the stack carries the record's deepest-first order by construction — the successor's output-key prefix and nonce, the requested cycle, the static subtree root, the canonical predecessor metadata, the predecessor's output-key prefix and the operator signature — and appends the executing leaf's script and control block; the two prefix bytes are compressed-key prefixes taken from the successor constructor and from the retained predecessor constructor rather than from the view, which states the predecessor program as the target does, an x-only key with no prefix in it and no parity beside it. `TransactionRefusal::WitnessItemWidthMismatch` refuses a populated item whose width differs from its schedule's declared role, including the fifty-three-byte variable-region slice; the tests hold each item against that declaration. `SubmitReadyMaturityAnnouncement` is a state and not a promotion: its bytes are the finalized bytes with the one witness inserted, its protected bytes are read off the finalized value, its status is a single variant read and never written, and it carries no digest, no field one could be put in, and no method that submits, encodes for a wire or produces a record.
