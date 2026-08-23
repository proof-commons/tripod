# `tripod-linker`

Resolves the typed relocatable backend bundle `tapscript` emits into a
deterministic candidate linked bundle (Guide-12 §14).

The linker owns symbol resolution, the typed reference graph and its
cycle policy, structured relocation, deterministic taptree assembly,
relation-carrier closure, and the linked resource projection. It does
not construct complete transactions, and it does not decide semantics:
every semantic fact it uses arrives inside the validated operation plan
the bundle carries.

Nothing here is final. The output is a `CandidateLinkedBundle`, whose
status is read-only and whose outstanding obligations are structurally
non-empty, because Guide-12 §1.9 keeps the candidate and final states
distinct and the evidence a final bundle would bind does not exist.

## The live-transfer link (Guide-13 §11)

`link_live_candidate` takes relocatable live bundles for one deployment
and produces a `CandidateLinkedLiveTransferBundle`: owner-parameterized
symbol resolution, exact relocations checked for application and
round-trip, a deterministic taptree that refuses mixed-representation
declarations, duplicates, emptiness, and depth violations, and per-plan
carrier closure.

The linker validates link-time composition and nothing earlier. It never
recomputes a constructor's leaf set from its shapes, because a sealed
constructor arriving here has already had that schema validated by the
derivation that owns it. A `LinkedConstructorPlacement` is a placement:
it is not a constructor digest and not a replacement identity.

A candidate carrying one representation is a link rather than a defect,
and the bundle reports which plans it actually carries so that a consumer
can tell an absent plan from a broken one.
