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

## The frozen STATE reference graph (Guide-14 Wave 4)

`FrozenStateReferenceGraph` consumes the constructor's typed declarations
and freezes canonical nodes, dependent-to-dependency edges, and strongly
connected components. Its limit of 64 distinct references matches the
constructor-local census, so the constructor and linker admit the same
reference inputs. A cycle is retained for inspection and then refused
without selecting or accepting a cut (`0.6.176-dev`).

Wave 6 owns binding-time resolution, authenticated cuts, the proof that the
residual graph is acyclic, canonical ordering after cuts, and integration
into the link pipeline. This Wave-4 graph is the frozen handoff artifact,
not a claim that any of those later obligations has been discharged.

## The maturity link's sources (Guide-14 Wave 6)

`StateLinkDeploymentParameters` binds what a maturity announcement link is given: the compiler's validated announcement plan, the realization's validated lead window, the reviewed target's revision, a candidate deployment identity and an established operator binding, each arriving as the type that validated it rather than as an entry in a map of names to values. A map would erase where each value came from, and origin is the whole of what distinguishes a deployment's answer from a stand-in that happens to be the right width.

The three external evidence roles that are facts about a deployment — what the substrate conserves, the singleton's non-reissuable declaration, and the issuance that placed its whole amount under the constructor — are read off the composed record and recorded beside the binding. The reduced announcement leaf rests on them and checks none of them, so naming them is how a reader learns which deployment records to go to; the binding does not verify them, because a past issuance and a deployed declaration are not things this process observes. The report-layer freshness role stays out, being a fact about chain context at the moment of spending rather than about the deployment.

The lead magnitudes travel with a `StateLeadBoundOrigin`, because the architecture declares both lead bounds with no default and requires deployment calibration: the numbers the fixtures use are test material, and a type that could not say so would let them be read as the contract's own.

Symbol resolution, the authenticated graph, relocation, the tree, the carrier comparison and the candidate bundle are later steps of the same wave. This bridge resolves no symbol, carries no internal key and builds no graph.
