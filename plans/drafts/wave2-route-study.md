# Research: routes for typed burn and clear realization

**Abstract (Proposal status)** · `abst:w2route:scope`

This draft proposes the semantic routes for Wave 2; it accepts no position and reports no implementation result. The execution guide supplies the delivery obligation, the concept supplies its semantic reference, and accepted Phase-7 rulings constrain both. The companion strategy orders reviewable work (`task:guide15-exec:wave2`), (`gate:guide15-exec:wave2-gate`), (`task:guide15:wave1`), (`sec:wave2:execution`), (`rule:drafts:authority`).

**Convention (Sources and proposed names)** · `conv:w2route:sources`

Path-and-line citations describe the inspected tree; proposed Rust names and file homes are design suggestions, not existing interfaces. Counts below are obtained from the listed members. A complete declaration means a complete declaration of one operation, not a complete realization of all architecture operations. No source text, generated publication or report becomes a semantic input.

## 1. Dispatch, ownership and the architecture weld · `sec:w2route:dispatch`

**Fact (The missing boundary is specific)** · `fact:w2route:dispatch`

The attestation dispatch supports AnnounceMaturity, CompactAsh and TransferLive and refuses other identifiers (`packages/realization/src/declarations/mod.rs:12`). Burn and Clear already have complete architecture operation rows (`packages/architecture/src/spec.rs:1391`, `packages/architecture/src/spec.rs:1524`). The sealed declaration trait separately dispatches the keyed profile (`packages/realization/src/declarations/mod.rs:47–61`). Wave 1 supplies that owner boundary, not attestation declarations (`plans/guides/guide_fifteen.md:1013`); its retained declaration packet preserves the attestation publication and pilot captures (`plans/phases/07-burn-and-clear.md:448`).

**Table (Dispatch routes)** · `tab:w2route:dispatch`

| Proposed route | Cost and useful result | What it would not establish | Recommendation |
|---|---|---|---|
| Add Burn and Clear arms to the attestation owner, with operation-local modules and welds | Two declarations; reuse sealed owner, typed identifiers, scoped derivation and graph validation | Compiler plans, target programs and complete architecture-wide scope | Select: these operations already belong to this architecture |
| Add a new semantic profile or crate | New bindings, identity, dependency and cross-profile lifecycle work | Agreement with existing Burn/Clear identifiers without another weld | Reject absent an independent ownership requirement |
| Treat model transitions or generated architecture text as declarations | Small apparent declaration effort, but imports execution or parsing authority into the wrong layer | Target-independent typed semantics and independent conformance | Reject: violates the typed-source and independence boundaries |

**Proposal (Exact per-operation weld)** · `proposal:w2route:weld`

Add `validate_burn_architecture` and `validate_clear_architecture` beside the existing per-operation functions, called before graph construction, using the owner-entry pattern at `packages/realization/src/validate.rs:797`, `:982` and `:1172`. Their set comparisons collapse duplicates (`:824–842`, `:890–894`, `:909–930`, `:956–965`); proposed duplicate exclusion is an additional check, not an inherited guarantee. Compare canonical collections with exact multiplicity, preserving permutation equivalence for set-like fields. Each weld checks operation identity/kind, authorization, issuance absence, input/output families and cardinalities, each input authorization, root policy, empty interface reads/writes, canonical deltas including conditions and tags, data-output schema/count/asset/tag, open flows, value-flow classes, witness census, bounds and projection policy. Bind object lifecycle and required exits as well. A changed coordinate must return an operation-specific architecture mismatch; a coherently removed declaration must fail a separate declaration-completeness check. Copying a plausible relation list without its architecture comparison is insufficient.

| Weld coordinate | Burn expected value | Clear expected value |
|---|---|---|
| Authorization and roots | Receipt owners; every root forbidden | Permissionless protocol; STATE succession only, no RESV |
| Protocol inputs | Live receipts, 1 through BurnInputMax, InputOwner authorization | Exactly one STATE with CovenantCompanion authorization; ASH, 1 through AshBatchMax, Permissionless authorization |
| Protocol outputs | Exactly one ASH; live change, 0 through BurnChangeMax | Exactly one STATE; ASH, 0 or 1 subject to residual law |
| Sponsor envelope | PlainLbtc inputs bounded by FeeSponsorInputMax; at most one PlainLbtc output; SponsorOwner | Same isolated envelope; no sponsor operand in clear arithmetic |
| Canonical flow | U lateral, always, no destruction tag | U destruction, always, Recon; ownerless lateral only on PositiveAshResidual |
| Data output | BurnRecord, Burn tag, no asset, 0 through BurnRecordMax | Exactly one Destruction, Recon tag, asset U |
| Witnesses | CanonicalDelta, AshLineage, ReceiptOwnerRouting, ReceiptClassClosure, AttestationAuthentication, ValueFlowClosure | CanonicalDelta, AshLineage, StateSuccession, FloorNondecrease, ValueFlowClosure, NativeFeeAuction |
| Value flows | OwnerConsented, OwnerlessBoundSink, SponsorEnvelope | OwnerlessTerminalSink, SponsorEnvelope |
| Projections | TransitionCertificate and BurnEvent required; ClearEvent and DistributionResidue forbidden | TransitionCertificate and ClearEvent required; BurnEvent and DistributionResidue forbidden |

The rows derive from `packages/architecture/src/spec.rs:1391–1468` and `:1524–1610`; the inherited compact weld checks projection exclusion explicitly at `packages/realization/src/validate.rs:938–950`. Bounds remain architecture-keyed parameters, not new semantic constants. Empty interface reads/writes do not mean that Clear cannot read committed STATE: the architecture explains that distinction at `packages/architecture/src/spec.rs:1590`. No architecture publication change is proposed. Compact names OwnerlessBoundMovement, while Clear names OwnerlessTerminalSink and SponsorEnvelope (`packages/architecture/src/spec.rs:1501–1520`, `:1574–1579`, `:1604–1607`). Read Clear's single flow as a terminal sink with a conditional residual, not a separate bound-movement operation; weld that set unchanged.

## 2. What makes each declaration complete · `sec:w2route:complete`

**Fact (The shell is larger than relations)** · `fact:w2route:shell`

`OperationRealization` lists operation, expressions, relations, relation dependencies, constructibility nodes and edges, lifecycle nodes and edges, disclosure nodes and edges, and disclosure seeds (`packages/realization/src/operation.rs:13–25`). Compact constructs all graph families after its weld (`packages/realization/src/declarations/compact_ash.rs:23–47`). Its expressions are empty, as are live transfer's and maturity announcement's (`packages/realization/src/declarations/transfer_live.rs:35`, `packages/realization/src/declarations/announce_maturity.rs:47`). This is a fact about those declarations, not a prohibition on expressions.

**Proposal (Operation-owned completeness census)** · `proposal:w2route:complete`

Freeze a separately reviewed semantic-clause-to-relation census before enabling each dispatch arm. List every relation identity/body, dependency endpoint, constructibility alternative, lifecycle edge and disclosure seed; compare exact canonical sets at validation, and retain independent literal expectations in tests. Each required law must be evaluated or name an explicit external-evidence requirement; an empty graph is justified by a proved absence, never by unfinished work. Scoped support for Burn/Clear must not make `try_complete` succeed for the remaining undeclared architecture operations.

| Declaration component | Burn proposal | Clear proposal |
|---|---|---|
| Relations | Bounded live inputs and change, one positive fresh ASH, exact U recognition and closed families, owner authorization, side-qualified representation admission, I=A+C, lateral delta, typed data-output record structure, event binding, root absence, sponsor isolation/opacity, substrate premise | Bounded public ASH inputs, one predecessor/successor STATE, six-field assignment, checked clamp, positive progress and independently derived nonzero floor Y′≥1, exact Recon destruction and conditional residual, no RESV, permissionlessness, event binding, sponsor isolation/opacity, substrate premise |
| Dependencies | Recognition and bounded family membership precede aggregation; authorization binds accepted split; event anchor precedes record-credit interpretation | STATE and ASH recognition precede aggregate/clamp; clamp precedes assignment/destruction/residual; these precede clear event and accounting comparison |
| Constructibility | Owner authorization is required for live receipts; fresh ASH value and future protocol exits require public inputs, not creator secrets; fee capability remains isolated | Public ASH amounts, public STATE metadata, public constructor facts; no owner/operator/creator signature on protocol path |
| Lifecycle | Consume live receipts; allocate one ASH and any live change; preserve CompactAsh and Clear exits on ASH and inherited receipt exits on change | Succeed STATE, consume ASH; either deallocate all ASH or allocate one residual with both future exits; preserve required STATE exits without claiming linked generation closure |
| Disclosure | Explicit public ASH value and authenticated record association; do not reveal sponsor amounts or turn input-owner secrets into public requirements | Public ASH operands and semantic STATE facts; sponsor values erased; no private opening or secret seed accepted as public availability |
| Expressions | Only expressions with carried facts and an evaluated declaration role | Existing STATE facts and Equal for copy laws; choose the clamp form at 2.3a; no duplicate assignment census |

**Table (Arithmetic and declaration routes)** · `tab:w2route:law-routes`

| Route | Cost | Limit | Proposed disposition |
|---|---|---|---|
| Existing policies only | Least vocabulary work | Conservation, roots and projection presence do not bind assignment/data/payloads | Insufficient alone |
| Accepted maturity precedent | Typed transition, independent function-equivalence and executed-successor tests; compiler symbolic laws later, declaration relations unchanged | Proves transition, not binding of observed Clear STATE to destruction/residual | Take this foundation in 2.1 |
| Existing typed STATE facts plus primitive carrier | Reuse FactId::StateField and Equal for five copy laws; add carrier and bind decrement | Clamp still needs closed relation or minimal expression | Recommend hybrid with transition; no second assignment-description type |
| General expression extension | New arithmetic nodes and all type/evaluation/consumer support | Still needs observation bindings and role checks | Only minimal clamp extension if justified at 2.3a |
| Assignment descriptor plus closed relation | Explicit law and evaluator | Duplicates StateField::ALL and risks a second census | Reject duplicate descriptor; closed clamp remains a candidate |

The accepted maturity precedent keeps execution in the typed transition and executable declaration relations unchanged (`packages/realization/README.md:455`, `packages/compiler/src/maturity_announcement_requirements.rs:1–5`). Independent function-equivalence and executed-successor tests define the comparison domain and refusal agreement (`packages/model/src/tests/state_metadata_conformance_tests.rs:1–23`); the vectors pin distinguishes the transition law from a published expression relation (`packages/vectors/src/maturity_safety.rs:5551`). Existing `FactId::StateField { operation, side, field }` (`packages/realization/src/identity.rs:128–134`) is typed at `expression.rs:198–223` and declared at `declarations/announce_maturity.rs:541–552`. Its evaluation currently refuses because neither STATE side has a primitive carrier (`packages/realization/src/evaluate.rs:708–713`). This is a carrier gap, not absent field vocabulary.

ExpressionNode lacks subtraction/minimum (`packages/realization/src/expression.rs:31–67`). Recommend transition plus primitive STATE carrier and existing equality expressions; the 2.3a packet chooses a closed clamp relation versus minimal expression addition against the frozen law census. Every declared law must be decidable and exactly bound.

**Table (Consumer census for new relation vocabulary)** · `tab:w2route:consumers`

| Consumer | Cited sites | Proposed handling |
|---|---|---|
| Realization identity/evaluation | `packages/realization/src/validate.rs:130–262`, `evaluate.rs:568–605` | Exact kind/subject/body validation and carried-fact evaluation |
| Compiler engine keys | `packages/compiler/src/engine_attestation.rs:306–341`, `:343–361` | Injective namespace codes for new RelationKind/RelationSubject and any new RelationEdge; preserve existing codes under ruling 172 (`plans/phases/07-burn-and-clear.md:420`) |
| Compiler coverage | `packages/compiler/src/coverage.rs:576`, `:600`, `:648`, `:697`, `:897` | Audit all matches, including wildcard paths; no invented coverage |
| Layout and placement | `packages/compiler/src/layout.rs:449`; `placement.rs:307`, `:561`, `:628` | Explicit unsupported behavior until Wave 3 |
| Requirements and source | `packages/compiler/src/engine_attestation/requirement.rs:388`, `:408`, `:476`; `engine_attestation/source.rs:95–116` | Preserve old requirements; no empty substitute for unsupported laws |
| Silent Representation consumers | `packages/realization/src/validate.rs:647–659` (wildcard at `:658`); `packages/compiler/src/engine_attestation/lifecycle.rs:40–58`; `packages/compiler/src/engine_attestation/proof.rs:27–29` with wildcard `:65` | In 2.3b, feed the side-qualified body's input ∪ change modes into the object-level lifecycle census; list the compiler sites as Wave-3 inputs, unreachable in Wave 2 behind the `bind_input` refusal |
| Conditional ExpressionNode consumer | `packages/compiler/src/fold.rs:67` | Only if 2.3a selects an expression-node clamp; then 2.3b adds it |
| Conditional FactId consumers | `packages/compiler/src/engine_attestation/constructibility.rs:339–372`; `packages/vectors/src/maturity_mutants/native_staging.rs:2938–2951` | None is proposed; any new member allocates its key code in 2.3b |
| Vectors relation match | `packages/vectors/src/maturity_mutants/native_staging.rs:2875–2912` | Adapt exhaustive match without changing admitted candidate identity |
| Vectors candidate/dispatch pins | `packages/vectors/src/maturity_mutants/native_staging.rs:2793`, `:2958–2985` | Separate candidate leaf census from supported operation census; revisit at Burn and Clear landings |

Reuse existing RelationEdge meanings where exact, with StaticRequirement for additional operation-owned binding/assignment/projection dependencies. No new edge member is currently proposed. If this cannot express the census honestly, 2.3a adds its precise ledger and 2.3b allocates its engine code before landing; no overloaded meaning or unencoded member is acceptable.

## 3. Stored Clear assignment and checked arithmetic · `sec:w2route:state`

**Fact (One stored decrement)** · `fact:w2route:state`

The six stored semantic fields are omega, y_l, y_t, q, cycle and maturity (`packages/realization/src/state.rs:53–71`). The existing transition is `announce_maturity` (`packages/realization/src/state.rs:315`); its copy-through helper deliberately names every field (`:73–90`). The model computes X=min(B,y_l,Y−1), refuses zero, and changes y_l (`packages/model/src/ops/ash.rs:72–151`); its postcondition independently checks the other five fields (`packages/model/src/shape.rs:2040–2097`). The concept's Y′=Y−X and Y_L′=Y_L−X are expressly subject to semantic confirmation (`plans/guides/guide_fifteen_concept.md:1506–1520`). Disposition 7 and ruling 140 supply that confirmation: Y is derived, not another stored write (`plans/guides/guide_fifteen.md:186–192`, `plans/phases/07-burn-and-clear.md:203`).

**Proposal (Typed assignment beside maturity)** · `proposal:w2route:state`

Add a total Clear transition beside announce_maturity over predecessor StateMetadata and the authenticated bounded ASH family, returning X, residual and complete successor or a closed refusal. Use `StateField::ALL` (`packages/realization/src/identity.rs:48–55`) as the six-field census for the transition, its tests and any binding, so that no second field list exists. Retain field-by-field construction; add no stored Y, nonce, codec field or caller-authoritative X. Bind observations through the operation-specific primitive carrier; copy laws use Equal and 2.3a fixes decrement/clamp representation.

Recommend model-aligned precedence rather than a different order concealed by broad error equivalence. At the complete operation boundary, domain/batch checks and distinct references precede STATE lookup and checked Y derivation; Y=0 refuses as Sealed before ASH recognition/aggregation (`packages/model/src/ops/ash.rs:76–108`). The typed function's authenticated-family precondition does not claim world lookup or recognition; those adapter checks retain their model order. Within its domain it checks Y before summing supplied authenticated amounts. Publish the refusal-to-Guard agreement and explicit domain as in `packages/model/src/tests/state_metadata_conformance_tests.rs:17–23`, with two-fault tests including Sealed versus ASH-sum overflow.

| Ordered check proposed | Controls and negatives | Reason |
|---|---|---|
| Bounded nonempty family and distinct references | One/multiple inputs; empty, oversized, duplicate | Model structural precedence |
| Obtain STATE, derive checked Y=y_l+y_t, require Y>0 | Interior/overflow/zero total; zero total beside overflowing B | Sealed before ASH sum; no wrapping |
| Authenticate actual ASH sources and checked-sum B | Correct set; wrong family, omitted/unavailable amount, overflow | No caller-authoritative aggregate; recognition is a distinct boundary |
| Derive Y−1 and X=min(B,y_l,Y−1)>0 | Each limiter, ties, zero y_l and Y=1 | No witness chooses X |
| Derive residual and successor y_l; copy remaining StateField::ALL members | Full/partial clear; one mutation per field | Exact assignment/destruction/residual join |
| Independently derive Y′≥1 | Floor-limited control; zero-total successor mutation | Named nonzero-floor obligation separate from positive X |

The maturity precedent suffices for a typed transition proved by independent equivalence; it does not itself bind observed Clear STATE, destruction and residual together. The hybrid adds that missing binding rather than declaring all helpers without a relation inadequate. Land function-equivalence and executed-successor model tests with 2.1, and observation/declaration comparison with 2.7. Disposition 7 and rulings 140/145 already settle the semantic law; the new decisions are totality, carrier binding and refusal order. The nonzero floor is independently projected after acceptance (`plans/guides/guide_fifteen_concept.md:1522–1530`, `:3958`), so retain its own named comparison rather than inferring coverage from positive X. Wave-7 arithmetic/generation receivers are `plans/guides/guide_fifteen.md:270`, `:278`, `:280`, `:282`.

## 4. Events, records and accounting have distinct acceptance roles · `sec:w2route:projections`

**Fact (Existing payloads are deliberately small)** · `fact:w2route:projections`

The model's BurnProjection stores ash_output, ash_value and records; a BurnRecord stores record_index, address and amount; ClearProjection stores omega and y (`packages/model/src/history.rs:317–337`). Clear certification binds the STATE succession and Recon destruction before deriving that payload (`packages/model/src/certify.rs:794–859`). The concept's richer event report additionally binds the accepted operation context (`plans/guides/guide_fifteen_concept.md:2420–2452`). `OperationObservation` currently carries a set of projection identifiers (`packages/realization/src/observation.rs:223–231`), so presence alone cannot compare these payloads. Ruling 147 expressly separates roles (`plans/phases/07-burn-and-clear.md:217`).

**Proposal (Bound semantic projections, internal only)** · `proposal:w2route:projections`

Use separate typed Burn event, record-credit, Clear event, accounting and STATE-edge values, bound through an operation-specific extension to exact region/certificate/object references. Context plus payload forms the comparison subject; do not enlarge the model's ClearProjection just to duplicate facts already authenticated by its certificate. A projection-presence policy stays useful but is not the payload validator. Keep record schema/ordinal validity in burn structural conformance and record aggregate credit in a separate verdict, so “no credit” is not a failed Burn operation. Choose the concept's full accounting list (`plans/guides/guide_fifteen_concept.md:2470–2483`) as typed projection subjects, explicitly widening ruling 147's Clear-accounting row to Burn totals and compact conservation. Clear-only is cheaper but leaves cross-operation noninterference without a named comparison subject; the full list is therefore recommended. Wave 2 delivers typed projections and binding checks only; event/accounting report producers remain owed in Waves 10–12 (`plans/phases/07-burn-and-clear.md:217`). No external serialization or public realization hash follows from these internal types; disposition 12 remains the default (`plans/guides/guide_fifteen.md:210–216`).

| Separately validated role | Exact subject proposed | Focused disagreement |
|---|---|---|
| Operation acceptance | Bound model execution and independently validated declaration observations | Wrong branch/family/root effect; an equal-total changed split remains a valid standalone observation and disagrees only with its bound request/execution |
| Burn event | Burn identity, consumed live family, unique fresh ASH identity/value, ordered records, change, no STATE/clear/residue effects | Copy the records onto compact or change the fresh ASH reference while preserving payload value |
| Record structure and value credit | Zero through configured maximum; contiguous zero-based ordinals; positive individual amounts; transaction/ordinal identity; repeated addresses permitted; aggregate claim at most fresh ASH | Duplicate/noncanonical ordinal or zero amount is structural refusal; aggregate overclaim voids the whole record set, not the burn |
| Compact silence | No burn records, burn event, clear event, STATE, destruction or issuance | Copied burn payload or fabricated clear projection |
| Clear event | Exact STATE pair, ASH sources, B, X, U/Recon destruction, residual, no RESV/burn provenance, certificate binding; payload omega and derived successor Y | Correct payload attached to the wrong STATE edge or certificate |
| Accounting | Burn record totals and fresh value; compact conservation; Clear B, X, decrement, destruction/tag, residual and event type; residue noninterference | Event equality with altered accounting value must still fail accounting |
| STATE succession | Exactly one predecessor/successor edge and exhaustive assignment | Right amounts attached to another predecessor or a termination edge |

These proposed subjects instantiate the concept's separate accounting list (`plans/guides/guide_fifteen_concept.md:2468–2483`) and burn/clear contracts (`:1294–1325`, `:1579–1593`). Malformed record structure and excessive aggregate claim are different cases: the model explicitly keeps a valid overclaiming burn and voids all its records, including aggregates beyond the individual amount domain (`packages/model/src/shape.rs:1993–2003`; ruling 143 at `plans/phases/07-burn-and-clear.md:209`). Use checked wide aggregation or early comparison for credit, not a ProtocolAmount sum that rejects the operation. Include zero records, underclaim/donation, exact claim, overclaim and beyond-domain overclaim controls. An optional-fields universal report would cost less code but permit one role's success to masquerade as another's; separate typed results cost binding tests and make that confusion rejectable.

## 5. Exact destruction and conditional residual · `sec:w2route:destruction`

**Fact (The reusable flow already has the right shape)** · `fact:w2route:flow`

`ObservedCanonicalFlow` shares one source family between movement and destruction, with typed tag/amount legs (`packages/realization/src/observation.rs:131–151`). Its evaluator checks source=destination+destruction, positive destruction legs and movement kind iff destination value is positive (`packages/realization/src/evaluate.rs:825–917`, especially `:908`), plus exact partition membership without repeated references (`:833–836`, `:958`); delta families are extracted into a set. A set of U/Recon delta kinds alone cannot distinguish two positive Recon legs from one leg. Architecture declares a single Destruction data output and conditional residual (`packages/architecture/src/spec.rs:1562–1588`). Realization has no data-output observation (`packages/realization/src/observation.rs:41–49`, `:222–233`) or relation (`packages/realization/src/relation.rs:126–198`); its existing welds only require data outputs absent (`packages/realization/src/validate.rs:952`, `:1129`, `:1329`). A new typed carrier and relation are required.

**Proposal (Reuse partition, add exact operation law)** · `proposal:w2route:destruction`

Retain the existing flow representation. Clear's closed law requires exactly one U/Recon destruction leg of X, no other destruction or issuance, B=X+R, and no duplicated source across flows. If R>0 require exactly one ASH destination of R and ownerless-lateral movement; if R=0 require no ASH destination and no movement kind. A zero-valued placeholder is not absence. Reuse partition exactness and movement-kind rules; the new law adds leg multiplicity, X binding and data-output association. Add extension-owned BurnRecord values with ordinal, realization-domain 32-byte address, positive amount and Burn tag, zero through BurnRecordMax; add exactly one Destruction with Recon, U and X. The model address is a semantic 32-byte value (`packages/model/src/scalar.rs:65`, `object.rs:110–119`), never a target type imported into realization. Check the data-output role separately from the flow leg, with an exact association; neither a matching tag string nor aggregate conservation supplies that association. Burn is lateral and destroys no U; compact remains ownerless lateral without destruction.

Controls cover full and partial clear. One-coordinate subjects change tag, asset, amount, leg count, destination count, residual presence, residual amount and movement kind. A split of one destruction into two equal-total legs tests the weakness of a set-only policy. Omitting the residual, duplicating it or replacing absence by zero must fail the exact Clear relation. Adding a generic multiplicity field to every delta is an alternative, but costs all-operation migration; the operation law is sufficient until another operation demonstrates that shared need. No target encoding or actual destruction script is established by these semantic checks (`plans/guides/guide_fifteen_concept.md:1532–1569`).

## 6. Public ASH, provenance and lifecycle · `sec:w2route:public-ash`

**Fact (One class, three creating operations)** · `fact:w2route:ash`

Architecture has one ASH object row, with Burn, CompactAsh and Clear allocators, CompactAsh/Clear mutators and Clear deallocation (`packages/architecture/src/spec.rs:2244–2260`). The concept allows one class while keeping fresh, aggregate and residual provenance in event/history facts (`plans/guides/guide_fifteen_concept.md:1374–1391`). The accepted boundary is explicit-only, normalized-first, with semantic representation vocabulary unchanged (`plans/phases/07-burn-and-clear.md:205`). The public construction contract excludes creator secrets and assigns the complete construction evidence to later waves (`:219`).

**Proposal (Public availability as a graph obligation)** · `proposal:w2route:public-ash`

Keep one ObjectId::Ash and authenticate provenance through the creating operation and bound event; never introduce an enduring “fresh” bit that lets residual or compacted ASH earn burn credit. Declare public positive authenticated amounts and public constructor dependencies, including both compact and clear exits for fresh and residual ASH. Burn's receipt-owner authorization remains distinct from the later owner's absence. Clear consumes public STATE metadata as well as public ASH; fee sponsorship contributes only its isolated capability. Test removal of the public amount source, insertion of a creator secret or protocol signature dependency, missing exit and false provenance. Burn and Clear ASH outputs admit compact's semantic {Explicit, PublicCommitted} vocabulary (`packages/realization/src/declarations/compact_ash.rs:365–366`), with ruling 141's explicit-only target selection. Preserve both meanings.

Separate ASH classes would require architecture and constructor migration with no present semantic need. Treating public availability as prose would avoid graph work but establish nothing about hidden dependencies. The recommended graph plus conformance establishes target-independent availability only; fresh-process construction, linked-byte equality and actual Clear leaves remain Waves 5, 7, 8 and 11 obligations, not Wave-2 results (rulings 142, 146 and 148 at `plans/phases/07-burn-and-clear.md:207–219`; Wave-7 receivers at `plans/guides/guide_fifteen.md:270`, `:278`, `:280`, `:282`).

**Proposal (Burn representation and inherited private exit)** · `proposal:w2route:burn-representation`

Normalized-first is mandatory with private change permitted (`plans/guides/guide_fifteen.md:162–164`; `plans/phases/07-burn-and-clear.md:205`); Burn inputs must authenticate representation (`plans/guides/guide_fifteen_concept.md:3924`). Representation currently selects object alone (`packages/realization/src/relation.rs:178–181`), and lifecycle representations are unioned across operations (`validate.rs:642–664`) for the lifecycle weld, while runtime admission is per region and per object (`packages/realization/src/evaluate.rs:597–603`). Live transfer declares PrivateCommitted live-receipt RequiresExit to Burn (`packages/realization/src/declarations/transfer_live.rs:559–572`), pinned at `validate.rs:699–718`. The union cannot be used to admit direct-private Burn.

| Route | Cost and limit | Recommendation |
|---|---|---|
| Side-qualified Burn admission | Add relation/identity/evaluation: ReceiptLive inputs Explicit only; live change Explicit or PrivateCommitted; preserve older object-wide relations | Select: complete admission now while actual target forms remain Wave 4 |
| Explicit Wave-2 representation non-claim | Avoid new vocabulary now; qualify deliverable 1's completeness and leave selection to Wave 4 (`plans/guides/guide_fifteen.md:256`) | Reject: the accepted normalized-first disposition already constrains the declaration |

Read the inherited PrivateCommitted live-receipt Burn exit as the lifecycle requirement of the deferred direct-private route, not as reachability through normalization. The typed lifecycle contract has one edge kind, `RequiresExit`, from a representation directly to a required exit of the same object; it rejects representation-to-representation edges and is not satisfied through generic reachability (`packages/realization/src/lifecycle.rs:31–34`, `:126–166`), and live transfer builds that direct edge from every mode (`packages/realization/src/declarations/transfer_live.rs:559–572`, `:713–739`). Disposition 3 and ruling 141 defer direct-private burn rather than abolish it, with semantic representation vocabulary unchanged. Preserve the edge, its pins and the contract; Wave 2's Burn admits explicit inputs only and does not discharge this exit, whose receiver is the separately admitted direct-private route. Test that Burn refuses a PrivateCommitted input while the inherited edge remains, and hand Wave 3 the rule that Burn in compiler scope does not discharge the private-mode exit, since compiler exits are tracked by operation (`packages/compiler/src/live_transfer_plan.rs:132`). Actual normalized/private-change target assessment remains Wave 4.

## 7. Independent model conformance · `sec:w2route:model`

**Fact (Execution precedes observation)** · `fact:w2route:model`

The conformance module explicitly does not participate in model acceptance (`packages/model/src/conformance.rs:1–5`). Its three adapters observe bound AnnounceMaturity, CompactAsh and TransferReceipts executions (`:122`, `:185`, `:215`); Burn and Clear adapters are absent from that list. Burn and Clear already execute through their own model code (`packages/model/src/ops/burn.rs:44–120`, `packages/model/src/ops/ash.rs:72–151`). The conformance wrapper's established evidence is model-side only (`packages/model/src/conformance.rs:74–90`).

**Proposal (Observe after independent acceptance)** · `proposal:w2route:model`

Add `observe_burn` and `observe_clear` taking the corresponding `ExecutedTransition` wrappers. Project primitive before/after objects, exact claimed region, request association, certificate, signer facts, canonical flow, record family, STATE fields and event payloads; do not call realization to decide the transition or manufacture its expected successor. Independently compare realized assignment and projection with the model's checked result. Keep substrate and authorization evidence explicitly model-side; it cannot discharge target signatures or native conservation. Reuse region projection only after checking exact source membership and sponsor erasure.

| Proposed evidence family | Accepted controls | Focused refusal or disagreement |
|---|---|---|
| Burn transition and observation | One/multiple owners, repeated owner, change/no change, sponsor/no sponsor, zero/exact/under/overclaimed records | Missing signer/class/event/ordinal/binding at their declared boundaries; equal-total changed split gives bound-execution conformance disagreement, not standalone realization refusal; Wave-9 signing separately binds the split (`plans/guides/guide_fifteen.md:261`) |
| Clear clamp and assignment | Each limiter and ties; full/partial residual; sponsor/no sponsor; varied unchanged metadata | Zero progress, arithmetic bounds, one mutated stored field, wrong STATE edge, wrong Recon leg, missing/extra residual |
| Composition and independence | Burn or Clear observed beside an unrelated valid region; sequential burn→compact→clear | Foreign object claimed into the region, reused source, sponsor value in an operand, copied burn provenance on compact/residual |
| Projection roles | Burn stands with refused record credit; Clear event and accounting separately agree | Corrupt event only, accounting only, record credit only or STATE edge only; another role's success supplies no credit |
| Existing consumers | Compact, live transfer, maturity and keyed owner controls | Any changed old relation projection, refusal boundary or owner identity |

Negative observations are conformance subjects, not claims that the model accepted an invalid transition. A model refusal is retained separately from a realization disagreement. Replaying the realization evaluator inside the model would make agreement circular; deriving all expected values from the production declaration would likewise leave coherent omissions untested. Independent literal clause inventories, model transition tests and one-coordinate observation mutations cost more tests but address these distinct failure modes.

## 8. Reopening and later consumers · `sec:w2route:handoff`

**Fact (Clear landing invalidates an inherited premise)** · `fact:w2route:reopening`

The compact keys `missing-clear-exit-emitted-sponsorless` and `missing-clear-exit-emitted-sponsored` currently record declaration retention/removal, not an emitted Clear path (`plans/backlog.md:2902–2903`). Ruling 138 requires their disposition and evidence compatibility to reopen when Clear realization lands (`plans/phases/07-burn-and-clear.md:199`). The baseline repeats that receiver (`plans/backlog.md:3301`). Ruling 133 also reopens the announced-state premise, finality of `16.6 / wrong-control-recipe` and the RETIRED successor-recipe part of conjunct 5 (`plans/phases/07-burn-and-clear.md:189`; `plans/backlog.md:1974`, `:2470`). Executable subjects are `candidate_leaf_and_transition_censuses_have_no_announced_state_spend` and `realization_operation_census_retains_only_the_announcement_state_transition` (`packages/vectors/src/maturity_mutants/native_staging.rs:2793`, `:2958`). Ruling 138 invokes this candidate-bound form under (`cond:wave0:baseline-freeze`).

**Proposal (Revalidate without inventing emission)** · `proposal:w2route:reopening`

At Clear's declaration landing, reopen both compact keys plus ruling 133's premise, Safety final and RETIRED conjunct part. Rerun both compact control/removal pairs and both named ruling-133 subjects, separating the growing semantic dispatch census from the admitted leaf census. Record whether each still reaches its declared boundary; preserve historical observations and identify the absent emitted control. If a downstream test assumes Clear is globally unsupported, repair that fixture's explicitly partial scope or record a newly required boundary; never turn an unsupported-plan refusal into an emitted-exit observation. Wave 3 owns plan closure and Wave 7 owns Clear lowering; Wave 2 may retain an explicit emitted-path obligation with a named receiver. Both rulings require performed candidate-bound dispositions. The likely outcome is that the admitted bundle still has no Clear leaf, but this must be checked; losing a Safety final or retirement ground is a baseline regression disposed before advancement.

Keep the provisional carrier as an ordering/association obligation. Its order is already decided by ruling 157 (`plans/phases/07-burn-and-clear.md:241`) and `plans/guides/guide_fifteen.md:54`, `:1115`, `:1215`: Wave-2 declarations precede Wave-3 plans, Wave 4 measures the complete carrier and Wave 6 consumes those measurements. Ruling 144's Wave-4 ruling is conditional on size or policy refusal (`plans/phases/07-burn-and-clear.md:211`); it names affected Wave-2 declarations and Wave-3 plans reopened before target declarations. Proposed ruling 190 is withdrawn as redundant; no new timing reading needs adoption.

**Proposal (Derived Wave-3 input packet)** · `proposal:w2route:wave3`

The guide's Wave-3 assumption names only Waves 0 and 1 (`plans/guides/guide_fifteen.md:1063–1065`). Derive the missing Wave-2 receiver from Wave 3's eight deliverables at `:1031–1061`: validated Burn/Clear declarations and architecture binding; canonical relation and dependency inventories; public construction/lifecycle/disclosure obligations; six-field assignment and arithmetic/refusal domain; typed event/record/accounting subjects; exact destruction/residual alternatives; independent conformance controls; reopened compact compatibility record and explicit target-only obligations, and the rule that an explicit-only Burn does not discharge the PrivateCommitted live-receipt Burn exit. These inputs enable Burn/Clear plans, compact revalidation, cross-operation scope, relation/carrier/layout/coverage/lifecycle censuses, ASH closure, validator corruptions and small-instance oracles. They are not those plans or their proofs. Recommend an explicit inherited-input sentence in adoption so no consumer silently substitutes Wave-1 keyed evidence for attestation semantics.

## 9. Inherited interfaces: the smallest sufficient changes · `sec:w2route:interfaces`

**Proposal (Default no change, with enumerated interface additions)** · `proposal:w2route:interfaces`

Preserve architecture publications, sealed owners, model acceptance, target ABI, STATE codec/generation and external format. Adopt these six eight-item ledgers at 2.0 against (`tab:w2route:consumers`), completing exact variant names and code allocations before implementation (`plans/guides/guide_fifteen_concept.md:74–111`). Compiler adaptation is refusal-only until Wave 3.

**Table (Observation-carrier routes)** · `tab:w2route:observation-routes`

| Route | Cost and limit | Recommendation |
|---|---|---|
| Optional carriers on OperationObservation | Fifteen literal-construction files across five packages migrate, including production maturity ABI construction at `packages/transaction/src/state_abi.rs:930`; every legacy path must leave absent carriers unread | Valid alternative, but unnecessary risk to a pinned production path |
| Separate operation-specific extension | Exact binding into common operation/region/reference identities, extension-aware evaluation and refusal of missing required facts | Select: keep OperationObservation and all old initializers unchanged, paying only new binding/entry checks |

The extension authenticates STATE pair, data-output ownership and payload/context against the normalized common observation; it is not an unbound side table. Its validated entry supplies existing FactId::StateField values. Legacy maturity retains its missing-carrier behavior and its production constructor remains unchanged. Burn/Clear cannot succeed through the old evaluator without required extension facts. If shared fields become necessary, stop and adopt the alternative with its complete initializer census; do not silently change transaction production code.

| Change | 1 Missing fact; 2 inherited type | 3 Smallest sufficient generalization | 4 Regression controls | 5 Identity/schema impact | 6 Evidence impact | 7 Dependency impact | 8 Positive/negative tests |
|---|---|---|---|---|---|---|---|
| Clear transition/STATE carrier | Clear result and carried fields; StateMetadata, FactId::StateField and MissingFactValue path | Total transition, extension-backed facts, StateField::ALL and Equal copy laws; clamp fixed at 2.3a | Compact/live unchanged; maturity function/codec; refusal-to-Guard precedence | No stored/codec change or second census | Independent function equivalence in 2.1, observed binding in 2.7 | Own realization domains; post-execution model comparison | All limiters/ties, fields, full/partial, zero/overflow and competing faults |
| Relation vocabulary/engine keys | Exact assignment/flow/payload laws; Relation, RelationKind, RelationSubject and consumers | Minimal decidable bodies; existing edges including StaticRequirement where exact | Pilot captures/refusals and keyed isolation; vectors pins | New injective owner kind/subject codes in engine_attestation.rs under ruling 172; old codes preserved; new edge only with explicit ledger/code | Clause outcomes, no target credit | Realization/compiler/vectors exhaustive census | Removed/weakened law, duplicate/wrong identity, code collision, unsupported plan refusal |
| Bound operation extension | Missing STATE/data/payload carriers, not missing field vocabulary; OperationObservation/reference model | Separate bound Burn/Clear extension, common struct unchanged; compare optional fields above | Legacy observations including production state_abi.rs:930 remain unchanged/unread for new facts | Internal types only, old projections preserved; no external format | Distinct operation/event/credit/accounting/edge binding | Realization evaluator entry and model adapters; no transaction initializer migration | Missing extension, correct payload on wrong region/certificate/edge, legacy maturity control |
| Model adapters | Post-execution Burn/Clear facts; three-adapter API | Two bound adapters with validated extensions and errors | Compact/live/maturity comparisons and raw model acceptance | No model law/identity change | Model-only premises; request split compared at conformance boundary | No realization call from kernel | Independent executions; substituted execution/region/payload |
| Data outputs | No observed data-output type or relation | Extension-owned BurnRecord (ordinal, 32-byte realization address, positive amount, Burn tag, 0..BurnRecordMax) and Destruction (exactly one Recon/U/X), exact association relation | Existing operations' data-output absence; overclaim keeps Burn valid | Semantic types and relation/key codes only; no wire schema | Record structure separate from credit; destruction output separate from leg | Realization domain/evaluator, independent model projection, compiler/vectors compatibility | Empty/max/over-bound records; wrong ordinal/address association/tag/asset/count; split destruction |
| Burn representation | Object-only Representation and unioned lifecycle modes cannot distinguish input/change | Side-qualified Burn admission; retain old relation; private-mode Burn exit retained as the undischarged requirement of the deferred direct-private route | Preserve live-transfer edge/pins; private mode union cannot admit Burn input; compact/maturity unchanged | New relation/subject side encoding, old identities stable | Explicit input/private-capable change semantics; target-form assessment still Wave 4 | Realization declaration/validator/evaluator, object-level lifecycle census, compiler keys/refusal, vectors census | Explicit-input/private-change control, direct-private input refusal, retained private exit |

Every additional interface need stops for its own eight-item ledger. 2.3a may settle the minimal clamp representation against the approved census; it may not create another field census or an undecidable declaration.

**Table (Package-contract questions retained at this boundary)** · `tab:w2route:package-questions`

The package card still lists these five questions (`plans/packages/realization.md:346–353`); the following are Wave-2 recommendations, not a claim to close every future use of them. The publication boundary remains typed values without a public realization hash (`packages/realization/README.md:449–453`).

| Question | Recommended route and reason | Alternative cost and limit |
|---|---|---|
| Partial scope | Keep existing explicit operation scopes; complete each new operation, leave missing architecture operations missing | Declaring the whole architecture complete would require unrelated declarations and cannot be justified by adding two arms |
| Amount-domain ownership | Keep realization's semantic domain and independent conversion from model Sat; use a distinct checked aggregate for record credit | Moving the domain to a shared crate adds dependency work without a new need; sharing the model's arithmetic implementation would weaken independent comparison |
| Fact/expression/relation identity | Extend explicit operation-owned identities and exact body validation only | Structural/public hashes require schema policy and consumer evidence; they do not prove semantic completeness |
| Shared relation provenance | Share code only while retaining distinct operation identities and exact region/certificate bindings | A common unqualified relation key risks cross-operation evidence reuse; deduplicating implementations does not authorize deduplicating claims |
| Public realization identity | Defer: no external consumer is named, and internal projection comparison suffices | Publishing an identity incurs schema, compatibility and ADR-022 boundary work without supplying the missing Burn/Clear law |

The package card also requires later declarations to import accepted STATE-constructor, wide-arithmetic, public-declassification and settlement-layout results without target opcodes (`plans/packages/realization.md:328–340`). The adopted Wave-0 census and rulings are the input packet for those boundaries; any newly required research result is a stop for a named reading, not permission to import a target type. This is a package-contract update need at adoption, because its open-question and pilot-era text does not itself describe complete Burn/Clear declarations.

## 10. Readings needing adoption before work · `sec:w2route:readings`

**Table (Merits support a recommended answer)** · `tab:w2route:readings`

| Reading or decision | Recommended answer and reason | Proposed ruling |
|---|---|---|
| Dispatch and completeness | Attestation arms with exact weld and complete per-operation graph census; existing ownership already names Burn/Clear | 182 |
| STATE-law representation | Typed transition/equivalence plus existing STATE facts and primitive carrier; minimal clamp form selected at 2.3a; no second assignment census | 183 |
| Transition/refusal contract | Inherit settled law; choose total transition, model guard order and no caller-supplied X | 184 |
| Accounting scope | Concept's full typed projection list, explicitly broader than Clear-accounting wording; record law inherited and report producers remain later | 185 |
| Destruction/residual exactness | One U/Recon leg of X; one positive residual iff needed, never zero placeholder; aggregate/family sets are insufficient | 186 |
| Public ASH/provenance | One object class, event-derived provenance, public dependencies and retained exits; no new class or target proof | 187 |
| Independent conformance | Execute model first; project primitive facts afterward; no realization-dependent acceptance | 188 |
| Candidate reopenings | Both compact keys, ruling 133's premise, Safety final and RETIRED conjunct part; preserve exact candidate meanings | 189 |
| Carrier order, already settled | Follow ruling 157 and guide :1115; refusal reopens named artifacts | 190 withdrawn |
| Interface and consumer handoff | Adopt six completed ledgers and explicit Wave-3 receiver, including data outputs, engine keys and vectors | 191 |
| Burn representation | Side-qualified admission; private exit is the deferred direct-private route's requirement, not discharged by Wave 2 | 191 |
| Observation carrier | Separate bound extension avoids shared initializer migration and preserves production maturity path | 191 |
| Execution/closure carrier | Adopt ordered packets and comparable baseline records before opening code work | 192 |

**Table (Merits alone cannot supply these facts)** · `tab:w2route:open`

| Unsettled subject | Why a paper decision cannot settle it | Proposed receiver |
|---|---|---|
| Actual record-carrier size and policy acceptance | Requires full encoded candidate and target observation | Wave 4; preserve provisional standing |
| Exact constructor equality and fresh-process public recovery | Requires linked Burn/compact/residual and expanded STATE subjects | Waves 5, 7, 8 and 11; preserve their separate gates |
| Whether gate wall becomes a regression threshold | The guide explicitly leaves policy open, and shared-load wall alone supplies no threshold (`plans/guides/guide_fifteen.md:1686–1688`) | Continue recording time and runner context; no invented threshold |

There is no unresolved semantic contradiction that prevents this proposal. Before implementation, adoption must approve the representation/lifecycle reading, complete interface scope and typed projection boundary; carrier order is already settled. Experiments then answer the empirical rows. The draft establishes none of their results.
