# `tripod-linker`

Resolves typed relocatable backend bundles into three candidate links: `CandidateLinkedBundle` for compact ASH (Guide-12 §14), `CandidateLinkedLiveTransferBundle` for live transfer (Guide-13 §11), and `CandidateLinkedMaturityBundle` for the maturity announcement (Guide-14 Wave 6).

The linker owns symbol resolution, typed reference graphs and cycle policy, structured relocation, deterministic taptree assembly, relation-carrier closure, and linked resource projection; it constructs no complete transaction and decides no semantics, since its semantic facts arrive in the validated operation plans carried by the bundles.

Nothing here is final. `CandidateLinkedBundle`, `CandidateLinkedLiveTransferBundle`, and `CandidateLinkedMaturityBundle` expose read-only candidate status and carry their respective outstanding obligations; linking alone supplies no final deployment bundle.

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

## The maturity constructor's tree (Guide-14 Wave 6)

`state_static_taptree_input` and `assemble_state_static` build the static subtree through the shared deterministic construction rather than a second one, with unit weights because no execution-frequency data exists, and the outer pair above it is fixed: the metadata leaf left, the static root right. The admitted recipe carries one static leaf, so the subtree has no topology to choose at all and the only decisions left here are policy, duplicate semantics, depth and binding.

The policy is the strict one, `SubsetOracleOnly`: every static leaf set this vocabulary expresses sits far inside the exhaustive oracle's budget, so the route that ranges over the whole tree space always answers and nothing weaker is admitted.

The costs are therefore two figures established two ways. The static cost is what the construction chose and the oracle checked — zero for the singleton tree, which has no topology. The complete cost is arithmetic over a pair nobody selected, one unit of depth for every static leaf plus one for the metadata leaf, which is two for the singleton tree; the closed-form and enumerated figures the tests compare against are a second opinion recorded beside the proof, never the route to it.

`StateLinkedTaptree::bind` binds the construction to one constructor's exact leaf bytes: the leaf sets and versions must agree, every committed path must be the depth the construction chose, the announcement's path must end at the metadata leaf, and the deployment's depth cap is enforced over the complete tree rather than assumed. The evidence retains the constructor's own hashes, so two trees whose declared roles agree and whose programs differ by one instruction are two different roots here.

## The maturity link's symbol census (Guide-14 Wave 6)

`collect_state_definitions` and `resolve_state_census` settle thirteen keys in two passes: the six the composed announcement program pushes, derived from the structural, semantic and operator family enums, and the seven kinds of constructor reference, whose values are stored beside the key rather than inside it, because the reference type embeds its value and a key carrying one could not say that a dependency had been defined twice.

Pass one collects one definition per key from five typed origins — the deployment, the candidate constructor, the reviewed target, the architecture's announcement lead bounds and the architecture's asset declaration — recording which layer answers for each and refusing a second claim on one key. Pass two compares the census against what actually consumes it: every consumer has a definition, every definition is the type its key declares, and every definition has a consumer. The last of those is the closure statement, because an enum variant does not disappear when a push site does, so a record that stopped consuming a key is exactly what it catches.

The structural and semantic families name one asset and one amount between them, so both spellings canonicalize onto one key each: they are one deployment fact, and a census with two keys for it could define it twice and inconsistently. The amount is the architecture's declared issuance rather than deployment data, because the asset declaration fixes the whole issuance at one unit and forbids reissuance, and a link that took the number from deployment data could push a value the chain can never carry into a program that then refuses every spend.

The census keeps a closed refusal root of its own, which is what lets a test enumerate it and hold every variant to being either exercised or declared unreachable with its reason; the shared root stays open, because it is the vocabulary of the engines every generation shares.

The authenticated graph, relocation, the tree, the carrier comparison and the candidate bundle are later steps of the same wave. This census resolves no program bytes, rewrites no leaf and builds no graph.

## The maturity link's authenticated graph (Guide-14 Wave 6)

`state_graph_from_sources` makes every reference the maturity link resolves a typed node — the thirteen census keys, the leaf programs the constructor commits, and the constructor's output as one node because its merkle root and output key are one commitment — and carries every dependency as an edge whose binding time says where the referent comes from: written into the leaf's bytes at link time, fixed by the constructor's recipe, introspected at spend time, witnessed and then authenticated, or reconstructed in the program from what it has already authenticated.

Only the three binding times that survive the commitment can cut a cycle, and a cut is validated and removed rather than flagged. Each is checked first against the record's own declared witness schedule and components, through a table that names, for each of them, the witness roles it needs and the components that read them — every witness a row names is read by a component the row names. The residual graph is then proved acyclic as a whole and reduced to a canonical dependency order, because accepting a component as soon as one of its edges could break a cycle leaves that component's other cycles unexamined: a component carrying two edge-disjoint cycles with one cut between them is refused, by the name of the cycle that is left.

The static root is never a link-time constant beneath a program that commits to it. A leaf cannot contain the root of the tree that commits to it, so the root arrives as a witness and the tweak equation against the internal key and the authenticated metadata is what binds it; the literal form is refused outright rather than handed to the cycle analysis, because it is not a cycle a better cut could resolve. The frozen constructor graph is retained beside all of this as a projection that must agree, because it is the constructor's own statement of the inputs it consumed.

Relocation, the linked resource figures, the carrier comparison and the candidate bundle are later steps of the same wave. This graph relocates nothing, rewrites no leaf and measures no resource.

## The maturity link's relocations and resources (Guide-14 Wave 6)

`discover_state_relocations` writes one record per mandatory occurrence of a resolved key, read off the census's own absolute sites and cross-checked by rebuilding the program with one key's value at a time: a byte search alone finds coincidences, and a rebuild alone says nothing about whether the census named what it moved, so each catches what the other misses. Each record names the component whose range holds its site, and a site inside more than one range is still one record, because the census counts instructions rather than memberships.

`substitute_state` then builds the linked program once from the pristine one with every value together, and holds it to four questions afterwards: every site carries its linked value, nothing outside the sites moved, the bytes decode back to the same program, and the walk from the record's own precondition reaches the record's own execution. The witnessed static root has no relocation at all — it is bound at spend time by the tweak equation, and a literal beneath the program committing to it would be a self-commitment with no authenticated cut.

`measure_state_resources` measures the linked program rather than inheriting the record's saturating projection, which is diagnostic and could not establish closure: a program whose cost overflowed would report the figure a program that did not reports. The checked totals stand beside that projection, beside the abstract schedule and the final-stack walk, and beside native observations that state what the program's primitives do not observe as explicitly as what they do. Initial-argument admission compares the walked program's starting stack with the reviewed relay policy, separately from the execution element bound: relay judges those arguments before execution, while the item constructor enforces the latter bound and the leaf can form larger legal intermediates. The historical replay schedule retains its measured policy refusal, while an emitted schedule with a refused argument fails the link. What the measurement leaves open is named — the streaming hash primitives carry no capability and no length-dependent cost, a budgeted primitive's figure is the most it can charge, the witness ABI reads its own declared widths, and no target observation is established here — because covering a gap quietly would be inventing a measurement.

The carrier comparison and the candidate bundle are later steps of the same wave. This sweep compares no carriers and assembles no bundle.

## The maturity link's carrier closure (Guide-14 Wave 6)

`close_state_carriers` puts three sides of every relation beside each other: what the compiler requires and at which boundary it discharges it, what the composed record says discharges it, and where the linked leaf actually does. None of the three is believed — the emitted table is recomputed from the record, the boundaries and cases are read from the plan the link's own bridge carries, and each emitted component is located at its actual range inside the linked program, held to being the instructions the composition wrote outside the sites the link substituted at, and checked against a tree that commits those linked bytes rather than the pre-link ones. A missing row, an extra one, a class the boundaries do not admit, a duplicate key and a leaf the tree does not commit are refused by name in both directions.

The row set is the discharge table rather than a placement census, because the reduced announcement leaf carries no positional sponsor carriers at all: ten of the twenty-six relations have nothing in the leaf to point at, and a closure over placements would have to omit them. Twenty-six relations, in two representations and two execution cases, give one hundred and four rows over four classes — thirteen relations carried by an emitted component, one resting on a named deployment fact, ten evaluated over the whole observed transaction and enforced by no leaf, and two left as the realization's own outstanding premises. The placement question survives inside the table: where the analysis does raise a runtime carrier obligation and the leaf carries it, the row records which alternative the announcement leaf answers to.

Vacuity and external requirements are preserved rather than absorbed. A relation with no content in a case carries nothing and refuses a component claiming otherwise, because a row reporting it carried would claim enforcement nothing provides. A premise the plan leaves open must still be visible afterwards — as the emitted carrier, as the linked one, or as an obligation on the side that does not exist yet — because an open premise that disappears into a comparison has been converted into a result. The ten model-scope rows stay published for the same reason: until the realization re-scopes those relations to the region an operation claims, a transaction composing several operations is accepted on-chain and outside the model, and a closure that dropped those rows would read as though the model already covered it.

The fourth side is stated as an exact contract and never as a claim. `StateAbiObligation` lists what an ABI must supply — each item of the record's own witness schedule, the input and successor slots, the operator's witness, the deployment facts the bridge recorded, the premises the plan leaves open, and the sponsor envelope — keyed by the same relation and representation as the rows, so the wave that builds the ABI answers one relation at a time instead of starting from prose. Nothing here counts a listed obligation as met.

The candidate bundle is where the linked tree and the linked leaf travel together, and what remains of the wave after it is the independent closure tests over linked bytes and the wave's own closure record. This closure assembles no bundle and supplies no ABI.

## The maturity link's candidate bundle (Guide-14 Wave 6)

`link_state_candidate` assembles everything the link established into one aggregate and carries each of it once: the bound deployment with its validated operator binding, the constructor policy, the composed record and the linked program, the committed tree with its control recipes, the resolved census, the authenticated graph and its cut proof, the relocations, the checked resources, the carrier closure with its outstanding ABI contract, the open premises the plan leaves, the retained constructor instances and the candidate status. Carried once is a rule rather than an aspiration: a value another field already holds is reached by delegation — the plan, the revision and the lifecycle closure through the deployment, the static subtree through the committed tree, the relocations through the linked program, the definitions as a projection of the resolved census — because two fields for one fact would let a reader compare a bundle against itself and find a disagreement. There is no content digest anywhere in it and no field that would hold one, because an identity minted before a consumer of one exists is an identity this generation refuses to mint.

The constructor is applied over the linked static subtree and never over the pre-link one. A deployment publishes the tree that commits the bytes a spend runs, and the composed record still carries the composition's fixtures at every site the census resolves, so a constructor derived over it commits a program nobody spends — which the carrier closure already refuses from the other side. Running the application after substitution is sound because of a fixed point the link asserts rather than assumes: five of the six keys the leaf pushes resolve from the deployment, the architecture's declarations and the reviewed target; the sixth, the internal key, resolves from the constructor's key policy, which is an input to the derivation rather than a function of the subtree; and the static root, the one definition that does move, is witnessed at spend time and pushed nowhere. The census re-collected over the applied constructor is held to the first on every pushed key, and the applied constructor's metadata-independent references are held to the subtree it was handed and to the recipe it came from.

A concrete constructor is metadata- and nonce-specific, so it is an application rather than a field. The same subtree under two semantic metadata values is two constructors with two metadata leaves, two merkle roots and two output keys, so a constructor field would have to pick one metadata value and carry the result as though it were a property of the link. `apply_constructor` is that application, fallible and under the bundle's own policy and budget; `retain` keeps a supplied instance only with its exact encoded metadata and nonce, holds it to the continuity equality first, and refuses a second retention of one semantic metadata, because a constructor kept without the metadata it is of would be a claim rather than an artifact.

`state_bundle_continuity` is the continuity equality, written once: exact equality of a predecessor and a successor static subtree, refusing with both roots and reconciling nothing. It is one named function rather than an equality repeated wherever two subtrees meet because no migration between static subtrees is implemented and this is the single assertion a migration relation would replace — a later generation has one place to point at, and a reader asking what forbids migration today has one place to read.

The outstanding set is structurally non-empty, its least member a field of its own, so a linked bundle owing nothing has no representation. Five obligations, each with the layer that owes it: current-state validation and the successor nonce search against a real predecessor, and finalization and witness population, all belong to the wave that builds a transaction against a chain; the ABI is the fourth side the carrier closure states as thirty-eight exact obligations; and the relations the realization evaluates over a whole observed transaction are enforced by no leaf and are the realization's own refit. This link validates no current state, searches no nonce against an observed predecessor, finalizes nothing, supplies no ABI and scopes no region.
