# Typed Realization Package · `pkg:realization:contract`

> **Status:** Active — `compact-ash`, `transfer-live-receipts`, and `announce-maturity` have typed attestation declarations; keyed SOURCE and MERGE have stable typed declarations; `RealizationScope::phase1_pilots()` retains the two-operation pilot scope and `RealizationScope::from_operations` reaches the announcement through `derive`; the bite-1.5 conformance evidence packet is closed at (`rec:w1b15:closed`), without a full-wave realization claim
> **Phase:** [Phase 1](../phases/01-realization.md)
> **Package:** `tripod-realization`
> **Library:** `realization`
> **Direct dependencies:** `architecture`, `petgraph`, `thiserror`
> **Decisions:** [D001](../decisions/001-typed-rust-source.md),
> [D002](../decisions/002-realization-layer.md),
> [D005](../decisions/005-value-representation.md),
> [D007](../decisions/007-petgraph-graph-substrate.md)

## Purpose · `sec:realization:purpose`

`realization` declares the target-independent semantic relation shared by model
conformance and compiler analysis.

```text
architecture
    ↓
realization
   ├────────▶ model conformance
   └────────▶ compiler
```

It fills the gap between the architecture’s finite registries and the formulas,
dependencies, constructibility, lifecycle, representation, and observables
needed below them.

The package implements typed semantics corresponding to
(`[RZ-rem:oracle:thesis]`) and the representation boundary of
(`[RZ-sec:realization:representation]`).

## Keyed declaration contract · `sec:realization:keyed-declarations`

Under (`rule:phase7:wave1-home-rulings`), the stable `keyed` and `relation` APIs declare the Keyed Provenance Covenant independently of the attestation census, without a declaration-side feature. `KeyedOperationRealization` is `OperationRealization<SemanticOperationId, KeyedRelationDeclaration>`; each `KeyedRelationId` contains the namespace-qualified operation, `SemanticObjectId::Keyed(KeyedObjectId::Handle)` and relation kind. `KeyedRealizationDraft::derive` consumes `architecture::keyed::KeyedManifest::contract()` through sealed declaration ownership; `validate` re-derives the complete declaration, verifies its architecture binding, refuses foreign/duplicate/misidentified rows and compares the complete shell. The canonical project orders validated relation identities independently of insertion order. There is one object, two operations, two covenant clauses, five relation variants, six kinds and nine relation rows.

**Table (Normative keyed paper weld)** · `tab:realization:keyed-paper-weld`

| Operation | Exact relation | Paper obligation |
|---|---|---|
| SOURCE | `HandleFamily { side: Input, cardinality: Zero }` | Complete authenticated same-key input classification under (`[A-op:kpc-operations:source]`), (`[A-def:kpc-model:keyed-membership]`) and the Retrieval assumption in (`[A-sec:kpc:interface]`); unrelated inputs remain open because no keyed covenant is evaluated at SOURCE. |
| SOURCE | `HandleFamily { side: Output, cardinality: One }` | Same-key output-0 handle under (`[A-def:kpc-model:keyed-membership]`) and (`[A-op:kpc-operations:source]`); later shaped outputs are not handles. |
| SOURCE | `SourceClassification` | The empty consumed family classifies SOURCE under (`[A-def:kpc-operations:classification]`), not an invented spending condition. |
| SOURCE | `MembershipReading { recurrence: SourceSingleton }` | Reader-derived singleton under (`[A-def:kpc-interface:membership-reading]`), (`[A-def:kpc-model:keyed-membership]`) and (`[A-op:kpc-operations:source]`); no stored record field. |
| MERGE | `HandleFamily { side: Input, cardinality: OneOrMore }` | Nonempty authenticated same-key consumed family under (`[A-op:kpc-operations:merge]`), with no covenant maximum. |
| MERGE | `HandleFamily { side: Output, cardinality: One }` | Same-key successor at output 0 under (`[A-def:kpc-model:first-output]`); unrelated outputs remain open. |
| MERGE | `BornFirst` | Each consumed handle's authenticated birth position is zero under (`[A-def:kpc-model:first-output]`). |
| MERGE | `ContinueFirst` | Each consumed handle requires an authenticated same-key first successor under (`[A-def:kpc-model:first-output]`). |
| MERGE | `MembershipReading { recurrence: ConsumedUnion }` | Reader-derived union under (`[A-def:kpc-interface:membership-reading]`), (`[A-def:kpc-model:keyed-membership]`) and (`[A-op:kpc-operations:merge]`), without a fresh record or emitted history traversal. |

Table spellings are `KeyedRelation` variants; sides are `TransactionSide`, cardinalities are `KeyedCardinality` and recurrences are `RecordRecurrence`. The remaining paper anchors complete the ten-anchor independent binding: (`[A-def:kpc-model:history]`) supplies immutable history; (`[A-sec:kpc:interface]`), (`[A-def:kpc-interface:membership-reading]`), (`[A-rem:kpc-interface:record-evidence]`) and (`[A-thm:kpc-interface:membership-correctness]`) keep retrieval, inclusion, acceptance and finality premises explicit evidence obligations. SOURCE has four evidence obligations and no synthetic execution carrier; MERGE has three evidence obligations and two per-member runtime obligations. These declarations do not implement a trusted history reader or discharge those premises.

All nine ancillary fields are explicitly empty for both declarations: expressions, relation dependencies, constructibility nodes/edges, lifecycle nodes/edges and disclosure nodes/edges/seeds. Nonempty fields, changed cardinality or recurrence, and coherently removed clauses refuse as `DeclarationCensus`; duplicate relations refuse as `DuplicateRelation`, mismatched operation/kind identities as `RelationIdentity`, foreign operations as `ForeignOperation`, and changed or old experimental bindings as `ArchitectureBindingMismatch`. A coherent foreign-object substitution also fails complete declaration comparison. Each refusal is paired with an accepted declaration in `src/tests/keyed_profile_tests.rs`; the exact nine-row control is independent of the declaration constructors. A reader runtime assignment remains the shared compiler's `PlacementCensusMismatch` control, not an excuse to add a carrier here.

The default compiler library provides keyed preparation/publication, all seven shared stages and independent fixed-owner validation. Opaque identity/adapter work, complete eligibility comparison and the scheduled measurements are recorded at (`rec:w1b12:gate`), with its explicit later-bite obligations; no compiler feature is needed. Ruling 180 assigns the envelope-specific 1,564/1,565 construction-policy boundary to the bite-1.4 transaction constructor; `OneOrMore` stays unbounded in the paper declaration. Stable declarations and symbolic compiler plans accept no realization or deployment; the bite-1.2 gate evidence is (`rec:w1b12:gate`).

## Dependencies · `sec:realization:dependencies`

Allowed first-party direct dependency:

```text
architecture
```

Graph-shaped realization internals use the workspace-owned Petgraph dependency
directly under [D007](../decisions/007-petgraph-graph-substrate.md). Petgraph
indices remain local graph handles, not semantic identity.
All expression, relation, constructibility, lifecycle, and disclosure graphs are
concrete Petgraph graph values used directly. The package introduces no graph
wrapper, stable realization hash, or Phase-1 publication; model conformance is a
post-execution observation check and does not decide model acceptance.

Forbidden dependencies:

```text
model
compiler
target-elements
tapscript
simplicity
linker
transaction
vectors
release
artifacts
```

No target or evidence package may become necessary to construct a realization.

## Typed inputs · `sec:realization:inputs`

The package consumes:

- a validated `architecture::Architecture` for attestation;
- an `architecture::keyed::KeyedManifest`, validated by `KeyedRealizationDraft::derive`, for SOURCE and MERGE;
- realization-owned typed Rust semantic declarations;
- architecture IDs and bound references.

It does not consume concrete deployment values during Phase 1.

## Typed outputs · `sec:realization:outputs`

The principal output is a validated, explicitly scoped realization containing:

- architecture binding;
- semantic facts;
- typed expressions;
- semantic relations;
- operation declarations;
- authorization requirements;
- state and root effects;
- public observables;
- constructibility requirements;
- witness availability;
- lifecycle requirements;
- representation capabilities;
- proof alternatives;
- derived declassification.

Exact Rust fields remain provisional until the two pilots pass.

> Illustrative boundary; names and exact fields are not frozen.

```rust
pub fn derive(
   architecture: &architecture::Architecture,
   scope: RealizationScope,
) -> Result<RealizationSpec, RealizationError>;
```

## Forbidden inputs · `sec:realization:forbidden`

The package must not parse or scrape:

- generated architecture JSON or TOML;
- generated declassification;
- model source or tests;
- realization Markdown;
- Attestation LaTeX;
- plans or ADR prose;
- target references;
- target programs;
- environment variables;
- filesystem state.

This follows [D001](../decisions/001-typed-rust-source.md).

## Identity ownership · `sec:realization:identity`

Architecture continues to own:

```text
AssetId
RootId
ObjectId
OperationId
QuantityId
WitnessId
InvariantClauseId
BoundId
TagId
```

Realization may own typed IDs for:

```text
Fact
Expression
Relation
Observable
Constructibility requirement
Lifecycle requirement
Proof alternative
```

IDs must be deterministic and independent of:

- source paths;
- line numbers;
- insertion order where order is not semantic;
- target selection;
- deployment values;
- wall-clock time.

No stable realization hash is published until its canonical projection and
migration policy receive a separate decision.

## Semantic domains · `sec:realization:domains`

The type system must distinguish at least:

- boolean;
- count;
- protocol amount;
- asset;
- object;
- owner;
- input reference;
- output reference.

Later operations may add cycle, block age, ratio, root, address, tag, and
canonical-order domains.

A count is not an amount merely because both are integers.

An operation region explicitly claims object references and owns its canonical partition and open flows; claims are disjoint, their union is the transaction's claimed portion, and catalog objects claimed by no region contribute to no operation's relations.

Arithmetic is checked and fail-closed.

## Derivation · `rule:realization:derive`

The package exposes one pure deterministic derivation from architecture and
typed declarations.

Derivation performs:

1. architecture validation;
2. deterministic indexing;
3. fact construction;
4. expression typing and dependency analysis;
5. relation construction;
6. authorization, observability, constructibility, lifecycle, and
   representation construction;
7. declassification derivation;
8. bidirectional validation;
9. explicit scope validation.

A pilot-scoped value must not be accepted as a full realization.

Transaction-side relation evaluation and primitive fact selection use the same selected operation region, with membership supplied by the observation producer rather than inferred from family or amount.

## Pilot A — compact ASH · `op:realization:compact-ash`

The declaration must cover:

- ASH input minimum and `ASH_BATCH_MAX`;
- exactly one ASH output;
- explicit semantic `U` conservation;
- permissionless authorization;
- optional isolated sponsor flow;
- no root use;
- transition-certificate projection;
- no burn, clear, or residue projection;
- public constructibility;
- future ASH compaction and clear lifecycle.

## Pilot B — live transfer · `op:realization:live-transfer`

The declaration must cover:

- nonempty bounded live-receipt inputs and outputs;
- owner authorization for every input;
- live-class closure;
- aggregate semantic `U` conservation;
- caller-selected authorized destinations;
- optional isolated sponsor flow;
- no root use;
- transition-certificate projection;
- explicit and private-committed value alternatives;
- future transfer, burn, and redemption lifecycle.

## Declassification · `rule:realization:declassification`

Declassification derives from typed dependencies.

Seed facts are those required by:

- public state;
- public events or interface output;
- permissionless construction.

The result records reason and provenance.

The compiler consumes the typed result. A JSON publication, if retained, is
derivative only.

## Assurance boundary · `sec:realization:assurance`

This package establishes:

- typed semantic completeness for its declared scope;
- architecture binding;
- expression and relation validity;
- dependency-derived disclosure;
- declared constructibility and lifecycle.

It does not establish:

- model behavior;
- target capability;
- backend correctness;
- transaction construction;
- deployment evidence.

Model conformance is a separate test boundary.

Conformance over combined observations establishes the relations of their selected regions, without establishing combined execution or target acceptance.

## Milestones · `tab:realization:milestones`

| Label | Deliverable |
|---|---|
| `milestone:realization:crate` | Crate and architecture dependency |
| `milestone:realization:identity` | Typed IDs and domains |
| `milestone:realization:expressions` | Expression arena and evaluator |
| `milestone:realization:relations` | Pilot relation vocabulary |
| `milestone:realization:derive` | Deterministic derivation and validation |
| `milestone:realization:compact-ash` | First pilot |
| `milestone:realization:live-transfer` | Second pilot |
| `milestone:realization:declassification` | Typed derived disclosure |
| `milestone:realization:model-check` | Model conformance |

## Exit gate · `gate:realization:exit`

Phase 1 exits when:

- both pilots are complete;
- architecture and realization coverage agree bidirectionally;
- model transitions satisfy the declarations;
- focused semantic mutations fail;
- declassification is derived;
- pilot scope is explicit;
- output is deterministic;
- no target type or generated-file input appears;
- workspace checks pass without modifying tracked files.

## Research boundary · `sec:realization:research`

Phase 1 is not blocked by target research.

Later operation declarations must import accepted results from:

- [STATE constructor](../research/state-constructor.md);
- [wide arithmetic](../research/wide-arithmetic.md);
- [public declassification](../research/public-declassification.md);
- [settlement layout](../research/settlement-layout.md).

Research may refine proof alternatives and lifecycle requirements. It must not
insert target opcodes into realization types.

## Error vocabulary · `sec:realization:errors`

See [`errors/realization.md`](errors/realization.md).

## Open questions · `sec:realization:open`

- How is partial pilot scope represented?
- Does the amount domain remain realization-owned until a second consumer exists?
- Are fact, expression, and relation identities explicit keys, structural hashes, or both?
- How do structurally shared relations retain operation-specific provenance?
- When is a public realization identity safe to mint?


## Keyed reader and evidence premises · `rule:realization:keyed-reading-premises`

The reader rule adopted under (`rec:w1b15:adoption`) is implemented in the vectors conformance boundary, not in declaration evaluation. Every authenticated SOURCE is eligible under the all-record rule, regardless of amount, with attribution unassigned. An optional explicit SOURCE-output annotation names its creating bytes; it denotes the SOURCE value at creation, not a reading of its successor value or record credit; for a directly queried SOURCE the outputs coincide. The fixed asset/value predicate has immutable parameters in its rule identity; a changing predicate has no growth promise. Only extensions of one valid history, under fixed premises and rule parameters and complete evidence, support fixed-rule accepted-set growth (`[A-rem:kpc-interface:record-evidence]`), (`[A-cor:kpc-interface:path-reading]`).

Inclusion, unspentness, retrieval and finality stay per-input external assumptions. The per-key limit is explicit: the absence of a consumed handle of an unsupplied key rests on inclusion in one valid history, not on local bytes proving its absence. Requested complete readings refuse missing evidence at the reader-evidence boundary; later non-handles have no set. Handle-closure completeness and whole-supplied-history completeness differ as (`rec:w1b15:reading-review`) records. Non-demanding reports carry incomplete or non-handle outcomes with no record-set field; unrelated keyed and unexamined incompleteness remains visible. The finality declaration form names deployment, policy version, history anchor, depth or replacement rule, inclusion/retrieval sources, unspentness observation scope and acceptance provenance. Its anchor is an opaque external chain position, matched to caller provenance rather than authenticated against transaction bytes. Its hypothetical fixture form establishes no deployment premise and refuses a required-final request before coordinate checks; the deployment type has no public producer until activation is specified. Inclusion/retrieval sources match the reading. Depth zero is a deployment choice meaning final at inclusion. Activation owes the actual declaration. Nothing here selects a confirmation count or discharges the remaining bite-1.6 obligations.

Internal versioned Rust reports follow ruling 165. Their framing and recomputing validators bind claims to exact subject bytes and evidence, with explicit absent quantities; all six evidence-family validators now exist as the offline conformance layer recorded at (`rec:w1b15:families`). Their shared derivations are checked against the graph-only model and independent literals; no family verdict replaces a premise. No external consumer, stable descriptor or maintained interchange format is introduced. This completes the realization-owned reader-rule and report-premise documentation deferred by (`rec:w1b15:reader-review`), without changing a declaration, compiler plan or covenant predicate.

The six-family packet is closed at (`rec:w1b15:closed`). Its eight-row paper trace supplements the unchanged declarations; MERGE unions SOURCE records without a new record, carried accumulator or covenant clause. The separate reader-rule and premise forms do not add realization evaluators or owner authorization. No deployment declaration exists or is claimed. Hypothetical declarations exercise matching only under NotRequested, while Required refuses them; birth, continuation and resources make no finality claim. Per-instance parent assumptions, final-byte transaction validity, native execution and policy observations stay open for the exact subjects handed to bite 1.6 at (`obl:guide15-exec:measurement-home`).

The final-subject hand-off and retained per-instance/deployment limits are accounted for at (`rec:w1b16:packet`).

## Burn and Clear declaration contract · `sec:realization:burn-clear`

The attestation census is adopted by (`rule:phase7:wave2-rulings`); (`rec:backlog:wave2-closure`) joins implementation and later evidence. This contract claims neither implementation nor architecture-wide completeness. Architecture owns operation/object/witness/bound/tag identities; realization owns laws, primitive carriers and graphs; model acceptance stays independent. Wave 3 plans receive every identity; target forms, linked constructors, signatures and reports keep their later-wave owners.

**Convention (Declaration identities)** · `conv:realization:burn-clear-identities`

B/C denote OperationId::Burn/Clear; R(o,k,s)=RelationId::new(o,k,s), with RelationKind/RelationSubject members. F(side,object)=ObjectFamily { side, object }; A(asset)=Asset { asset }; T(side)=TransactionSide { side }; L(object,exit)=LifecycleExit { object, exit }; P(object)=Representation { object }. Expand every listed set member literally, retaining operation and side. The owner validator independently checks body, identity, exact multiplicity and complete shell; coherent removal and duplicates fail, set permutations pass. New names allocate implementation, not existing interfaces.

**Table (Public vocabulary allocations)** · `tab:realization:burn-clear-vocabulary`

| Body / kind | Exact subject | Attestation kind code | Meaning and staging |
|---|---|---|---|
| SideRepresentation { side, object, allowed } / SideRepresentation | SideRepresentation { side, object } | 16 | Side-specific admission, preserving existing object-wide Representation; Burn input and change have separate keys. |
| DataOutputs { expected } / DataOutputs | Operation | 17 | Exact typed data-output structure and association for the owning operation; expected is the architecture-keyed set of kind, tag, asset, minimum, maximum and condition that the weld compares, as CanonicalDeltaPolicy { expected } does; the operation is the identity's, never a body field. |
| BurnBinding / BurnBinding | Operation | 18 | Fresh ASH, source/change, event and certificate binding; no STATE, issuance or destruction. |
| ClearClamp (body fields fixed at 2.3a under ruling 183) / ClearClamp | Operation | 19 | Checked B and Y, X=min(B,y_l,Y−1)>0 and residual; no caller-authoritative aggregate or X. |
| ClearAssignment (YL form fixed at 2.3a; copy bodies { field, expression }) / ClearAssignment | StateField { field } | 20 | One StateField::ALL field per relation; five Equal copies and exact y_l decrement. |
| ClearFloor / ClearFloor | Operation | 21 | Independent successor Y′≥1, unchanged omega, nonincreasing Y. |
| ClearFlow / ClearFlow | Operation | 22 | One U/Recon destruction leg X; one positive residual iff B−X>0. |
| ClearBinding / ClearBinding | Operation | 23 | Exact STATE pair, sources, residual, event payload, authenticated context. |

In packages/compiler/src/engine_attestation.rs preserve kind codes 0–15, subject prefixes 0–8 and edge codes 0–12. SideRepresentation uses prefix 9 then existing side/object codes; StateField uses prefix 10 then Omega=0, YL=1, YT=2, Q=3, Cycle=4, Maturity=5. Compiler encodings add no public hash/wire schema or RelationEdge/FactId/StateField/RepresentationMode member. StaticRequirement carries exact additional operation-owned dependencies. Bite 2.3a fixes the ClearClamp implementation as a closed body or minimal expression form and freezes its evaluated operand graph; either form keeps this semantic identity and census. Under a closed body X is no expression value and no FactId carries it, so 2.3a fixes the YL ClearAssignment's evaluated form together with the clamp; only the identities R(C,ClearClamp,Operation) and R(C,ClearAssignment,StateField { field: YL }) and kind codes 19 and 20 are fixed here. Declare the eight RelationKind members after ExpressionPredicate in code order and the two RelationSubject members after Representation, SideRepresentation first, so codes keep following derived ordering (packages/compiler/src/engine_attestation.rs:304–305). One shared state_field_code encoder there supplies subject/fact keys, replacing the cast at packages/compiler/src/engine_attestation/constructibility.rs:355 under ruling 176; the additional path stops the code bite for a revised packet. Expression additions require complete consumer dispositions before 2.3b; added paths require a revised packet.

**Table (Complete relation and body census)** · `tab:realization:burn-clear-relations`

| Operation / identities | Bodies / clauses | Producer / receiver |
|---|---|---|
| B: R(B,Cardinality,F(Input,ReceiptLive)), R(B,Recognition,F(Input,ReceiptLive)) | 1..BurnInputMax recognized live U receipts; distinct sources. | 2.4; model 2.5; Wave-3 Burn plan |
| B: R(B,Cardinality,F(Output,ReceiptLive)), R(B,Recognition,F(Output,ReceiptLive)) | 0..BurnChangeMax live U change, exact owner/request routing. | 2.4/2.5; Wave 3 and finalized signing in Wave 9 |
| B: R(B,Cardinality,F(Output,Ash)), R(B,Recognition,F(Output,Ash)) | Exactly one fresh positive U ASH, authenticated creating operation. | 2.4/2.5; Waves 3, 5 and 8 |
| C: R(C,Cardinality,F(Input,Ash)), R(C,Recognition,F(Input,Ash)), R(C,Cardinality,F(Output,Ash)), R(C,Recognition,F(Output,Ash)) | 1..AshBatchMax inputs and 0..1 output, all public positive U ASH; the residual law fixes output presence. | 2.6/2.7; Waves 3 and 7 |
| C: R(C,Cardinality,F(Input,State)), R(C,Recognition,F(Input,State)), R(C,Cardinality,F(Output,State)), R(C,Recognition,F(Output,State)) | One PID STATE per side; exact companion/succession, no termination. | 2.6/2.7; Waves 3 and 7 |
| Each o∈{B,C}, side∈{Input,Output}: R(o,Cardinality,F(side,PlainLbtc)), R(o,Recognition,F(side,PlainLbtc)) | Recognized L-BTC sponsor inputs 0..FeeSponsorInputMax, output 0..1; no protocol amount operands. | 2.4/2.6; 2.8; Waves 3 and 11 |
| Each o∈{B,C}, each side: R(o,AllowedObjectFamilies,T(side)) | B input {ReceiptLive,PlainLbtc}, output {Ash,ReceiptLive,PlainLbtc}; C both sides {State,Ash,PlainLbtc}; exact selected region, no foreign family. | 2.4/2.6; 2.8; Wave 3 |
| R(B,Authorization,F(Input,ReceiptLive)); R(C,Authorization,Operation) | OwnerAuthorization { object: ReceiptLive } for every Burn owner; PermissionlessAuthorization for Clear. Request comparison is independent of finalized-byte signatures. | 2.4–2.7; Wave 9 |
| R(B,Conservation,A(U)) | AmountConservation: ReceiptLive→{ReceiptLive,Ash}, I=A+C, fresh A>0; equal totals do not bind the authorized split. | 2.4/2.5; Wave 3 |
| Each o: R(o,CanonicalDeltaPolicy,Operation) | Burn U lateral without tag. Clear U/Recon destruction always and ownerless lateral exactly on positive residual (architecture DeltaCondition::PositiveAshResidual). The existing body cannot state that condition: ExpectedCanonicalDelta carries no condition and the evaluator requires the observed family set to equal one fixed set, so any fixed Clear set refuses either the full or the partial clear. Bite 2.3a fixes the conditional-family form within the relation ledger, with full and partial clear both accepted and a lateral family without residual or a residual without lateral refused, before 2.3b activation; a change to ExpectedCanonicalDelta changes captured bytes containing its values and stops 2.3a for its own eight-item ledger. Preserve exact partition membership and no reused source; ClearFlow adds multiplicity. | 2.3a/2.3b, 2.4/2.6; Waves 3 and 7 |
| Each o: R(o,OpenFlowPolicy,Operation), R(o,SponsorEnvelopeMultiplicity,Sponsor), R(o,SponsorIsolation,Sponsor) | Only FeeSponsor, at most one envelope, no U escape, isolated signer/capability and sponsor opacity. | 2.4/2.6; 2.8; Wave 11 |
| Each o: R(o,SubstrateConservation,A(Lbtc)) | ExternalEvidenceRequirement::SubstrateConservation { operation: o, asset: Lbtc }; sponsor-erased evaluation never yields Passed. | Model-only discharge 2.5/2.7; target discharge remains later evidence |
| Each o: R(o,RootPolicy,Operation) | Burn roots Forbidden; Clear STATE succession only, all others Forbidden, particularly RESV. | 2.4/2.6; Wave 7 |
| Each o: R(o,ProjectionPolicy,Operation) | TransitionCertificate required; B BurnEvent required and ClearEvent/DistributionResidue forbidden; C ClearEvent required and BurnEvent/DistributionResidue forbidden. | 2.4/2.6; separate typed bindings and Waves 10–12 |
| R(B,Constructibility,Operation); R(C,Constructibility,Operation) | OwnersOf { object: ReceiptLive } for Burn; PublicPermissionless for Clear; owner authorization never becomes an ASH creator secret. | 2.4/2.6; 2.8; Waves 5, 7, 8 and 11 |
| R(B,SideRepresentation,SideRepresentation { side: Input, object: ReceiptLive }); R(B,SideRepresentation,SideRepresentation { side: Output, object: ReceiptLive }) | Input {Explicit}; change {Explicit,PrivateCommitted}. Object-level union feeds lifecycle only, never input admission. | 2.3b/2.4; Wave-3 case semantics and Wave-4 target assessment |
| R(B,Representation,P(Ash)); R(C,Representation,P(Ash)); R(C,Representation,P(State)) | ASH {Explicit,PublicCommitted}; STATE retains {Explicit,PublicCommitted}; explicit-only selected target policy remains separate. | 2.4/2.6; Waves 3 and 4 |
| Each o: R(o,DataOutputs,Operation) | B 0..BurnRecordMax Burn-tagged records: contiguous zero-based ordinal, 32-byte realization address, positive amount, exact transaction/ordinal/fresh-ASH association, repeated addresses permitted. C exactly one Destruction, Recon/U/X, associated with its sole destruction leg. | 2.2/2.3b, 2.4/2.6; Waves 4 and 6 |
| R(B,BurnBinding,Operation) | Bind actual live sources, every owner, exact change/request split, fresh ASH and ordered event records to normalized region/certificate; no STATE, Clear event, issuance or destruction. | 2.4/2.5; Wave 9 and event reports |
| R(C,ClearClamp,Operation) | Checked B=sum(authenticated ASH), Y=y_l+y_t; helper rejects Y=0 before summing ASH; X=min(B,y_l,Y−1)>0; R=B−X. Model owns whole-operation batch/distinct/lookup/recognition order. | 2.1/2.3a/2.3b; 2.7 and Wave-7 arithmetic |
| For every field in StateField::ALL: R(C,ClearAssignment,StateField { field }) | YL output is predecessor YL−X; Omega, YT, Q, Cycle and Maturity copy exactly. Existing FactId::StateField { operation: C, side, field } supplies both sides; no stored Y, nonce or codec field. | 2.1/2.3b/2.6/2.7; Wave 7 |
| R(C,ClearFloor,Operation) | Derive Y′ independently from actual successor fields; require Y′≥1, unchanged omega and Y′≤Y for FloorNondecrease, separately from X>0 and the rate witness's external target premise. | 2.6/2.7; Waves 3 and 7 |
| R(C,ClearFlow,Operation) | Exactly one U/Recon destruction leg X, no other destruction/issuance; B=X+R; exactly one ASH destination R with ownerless-lateral movement iff R>0, otherwise no destination/movement. No zero placeholder or split equal-total legs. | 2.3b/2.6/2.7; Wave 7 |
| R(C,ClearBinding,Operation) | Bind STATE pair, source ASH, X, residual, data output and certificate to one region; payload omega/derived successor Y; no RESV, Burn credit or residue effect. | 2.6/2.7; Waves 7 and 10–12 |
| Each o, each exit in {CompactAsh,Clear}: R(o,Lifecycle,L(Ash,exit)) | Fresh/residual ASH keeps both exits regardless of provenance; no emitted-exit proof. | 2.4/2.6; Waves 3, 5, 7, 8 and 11 |
| Each exit in {TransferLive,Burn,Redeem}: R(B,Lifecycle,L(ReceiptLive,exit)) | Live change keeps inherited exits in both modes; PrivateCommitted→Burn remains the deferred direct-private route's undischarged requirement. | 2.4; Wave 3; separately admitted direct-private route |
| Each exit in {AdmitDeposits,Cycle,Redeem,ReceiptRelabel,Clear,AnnounceMaturity}: R(C,Lifecycle,L(State,exit)) | Preserve six STATE exits and constructor/generation boundary; Clear availability proves no emitted leaf. | 2.6; 2.9; Waves 3 and 7 |

**Table (Graph and expression census)** · `tab:realization:burn-clear-graphs`

| Shell field | Required expansion | Receiver/refusal |
|---|---|---|
| Operation/weld | Exact OperationSpec identity/kind, operation/input authorization, roots, no issuance, input/output lists/counts/bounds, delta conditions/tags, data schema, FeeSponsor, empty interface reads/writes, six witnesses, flow classes, bounds/projections and ASH/live/STATE lifecycles. Clear keeps OwnerlessTerminalSink and SponsorEnvelope. | 2.4/2.6 independent coordinate and coherent-removal tests. |
| Expressions | C: ExprId::fact(StateField fact) for both sides × StateField::ALL. Five copy ClearAssignments own ExprId::relation(its ID,Predicate) Equal predicates; YL/clamp evaluated form fixed at 2.3a. B has no unused expression. | 2.3a/2.3b exact nodes/edges and evaluation; no duplicate census or missing-fact success. |
| Family dependencies | Each Recognition→its Cardinality: RecognitionBeforeCardinality. B live input/change and ASH output Recognition/Cardinality→Conservation: RecognitionBeforeValue/CardinalityBeforeValue. CanonicalDeltaPolicy→Conservation: CanonicalDeltaBeforeValue. | 2.4/2.5; Wave 3 exact endpoints. |
| Authorization/sponsor dependencies | Authorization→input AllowedObjectFamilies/Constructibility: AuthorizationBeforeClosure/AuthorizationBeforeConstructibility. OpenFlowPolicy→SponsorIsolation: OpenFlowPolicyBeforeSponsor. OpenFlowPolicy→SponsorEnvelopeMultiplicity→SponsorIsolation and sponsor Recognition→SponsorIsolation: StaticRequirement. SponsorIsolation→Constructibility/SubstrateConservation: SponsorBeforeConstructibility/SponsorBeforeOperation. RootPolicy/ProjectionPolicy→Constructibility: corresponding BeforeOperation edges. | Both declarations; 2.8 isolation; no sponsor-value operand. |
| Operation-law dependencies | StaticRequirement: B recognition/cardinality/authorization/Conservation→BurnBinding→DataOutputs. C STATE/ASH input recognition/cardinality→ClearClamp→all ClearAssignment/ClearFlow/DataOutputs; assignments→ClearFloor; ClearFloor/ClearFlow/DataOutputs and output recognition/cardinality→ClearBinding. | 2.3a expanded endpoints; 2.4/2.6 complete comparison; credit never authorizes Burn. |
| Representation dependencies | Constructibility/output Recognition→relevant Representation or output SideRepresentation: StaticRequirement. Each representation→its object Lifecycle relations: RepresentationBeforeLifecycle. Burn input SideRepresentation→BurnBinding: StaticRequirement. | 2.4/2.6; private input refused, private change retained; Wave 3 cannot discharge private exit by scope membership. |
| Constructibility | Operation node, facts/witnesses below. Fact→Operation: RequiredFact/Required; Burn owner: RequiredWitness/Required; sponsor: SponsorOnly/Optional. No Clear owner/operator/creator witness or protocol sponsor amount. | 2.4/2.6/2.8 public and secret-insertion controls; linked/fresh-process evidence later. |
| Lifecycle | Each censused representation × exit: Representation { object,mode }→RequiredExit { object,operation: exit } through RequiresExit. ASH/STATE two public modes; live Explicit/PrivateCommitted. No representation-to-representation edge. | 2.4/2.6 exact nodes/edges and private exit; Waves 3/7 discharge only owned routes. |
| Disclosure | Facts below; constructibility/representation/event-binding relation nodes. Fact→Constructibility: ConstructibilityInput; output amount→Representation and event operand→binding: PublicObservableInput; arithmetic/assignment operands: RelationOperand. Seeds: STATE/PublicState, events/PublicEvent, ASH representation/PublicInterface, Clear constructibility/PermissionlessConstructibility. | 2.4/2.6/2.8 source-removal/secret-insertion; owner signatures never ASH dependencies; sponsor amounts erased; no linked-byte claim. |

Packets retain expanded canonical IDs and literal graph expectations for these endpoints. Bite 2.3a may refine expression nodes, but removing clauses, renaming obligations or adding primitive facts reopens the interface ledger. Constructor availability uses existing facts and explicit later evidence, never a target-proof Boolean.

ProofAlternativeId::new(relation ID, kind) keys alternatives: ManifestShape for structural policies/DataOutputs/bindings; PublicArithmetic for Conservation/clamp/assignments/floor/flow; SignerMembership for Burn authorization/constructibility; PublicConstructibility for Clear authorization/constructibility. SubstrateConservation has only its own alternative; Representation/SideRepresentation/Lifecycle constrain modes/exits without alternatives. PublicArithmetic excludes PrivateCommitted (packages/compiler/src/lifecycle.rs:130–139); explicit/public ASH with private change is refused as mixed conservation (packages/compiler/src/case.rs:116–150) under (`rule:guide13-exec:mixed-representation`). Wave 3 plans and Wave 4 target assessment own that unapproved proof route; 2.4's admission grants no proof coverage. Bite 2.3a freezes alternatives beside bodies and Wave 3 selects plans.

Required constructor facts are operation-qualified except BoundValue. B: counts Input/ReceiptLive, Output/ReceiptLive, Output/Ash; public amounts Input/ReceiptLive and Output/Ash; FactId::FamilyAmount { operation: Burn, side: Output, object: ReceiptLive } with AvailabilityClass::InputOwners { object: ReceiptLive } (zero for no change); bounds BurnInputMax/BurnChangeMax/BurnRecordMax. C: counts Input/State, Output/State, Input/Ash, Output/Ash; public Input/Ash amount, AshBatchMax and all predecessor StateField facts. Both: public FeeSponsorInputMax, optional SponsorAuthorization/SponsorLocal witness; no sponsor amount. Burn adds ProtocolOwnerAuthorization/InputOwners { ReceiptLive }; Clear adds no private witness. Disclosure: both STATE sides, public ASH amounts, these bounds/counts; no absent-residual placeholder or private change/sponsor amount seed. Extension/binding authenticates event context without Boolean facts.

**Table (Adopted witness-to-obligation map)** · `tab:realization:burn-clear-witnesses`

| Witness and operation | Exact declaration identities | External evidence / receiver |
|---|---|---|
| CanonicalDelta, B/C | R(o,CanonicalDeltaPolicy,Operation), R(o,AllowedObjectFamilies,T(Input/Output)); B Conservation/U and BurnBinding; C ClearClamp/ClearFlow/DataOutputs | SubstrateConservation { operation: o, asset: Lbtc } is model-only in 2.5/2.7 and separately target-owned; Wave 3 plans and Wave 7 Clear destruction. |
| AshLineage, B/C | B BurnBinding, R(B,Conservation,A(U)) and Ash output Recognition; C ClearClamp (owning B), ClearBinding, ClearFlow and Ash input/output Recognition; both ASH Lifecycle exits | LinkedConstructorEquality and PublicRecovery evidence remain Waves 5, 8 and 11; residual/compact receives no fresh credit. |
| ReceiptOwnerRouting, B | Authorization/F(Input,ReceiptLive), Conservation/U, BurnBinding and live input/output Recognition | FinalizedBurnAuthorization: exact all-input/all-output signature binding in Wave 9; model request comparison is 2.5, not signature proof. |
| ReceiptClassClosure, B | Both ReceiptLive Recognition, both AllowedObjectFamilies and both SideRepresentation identities | Wave 4 normalized/private-change target assessment; direct-private input remains refused. |
| AttestationAuthentication, B | BurnBinding, DataOutputs, ProjectionPolicy and Ash output Recognition | AuthenticatedBurnCarrier: event-type/value anchors and canonical carrier authentication in Waves 6/9/10; typed binding alone supplies no target authentication. |
| ValueFlowClosure, B/C | Both AllowedObjectFamilies, CanonicalDeltaPolicy, OpenFlowPolicy, SponsorIsolation, SponsorEnvelopeMultiplicity; B Conservation/BurnBinding, C ClearFlow | SubstrateConservation as above; 2.8 independent disjoint-region/foreign-source checks, Wave-3 coverage. |
| StateSuccession, C | State input/output Cardinality/Recognition, RootPolicy, ClearBinding and all six ClearAssignment/StateField identities | LinkedStateSuccession: expanded constructor/generation/history proof in Wave 7 and later continuity reports; no migration claim. |
| FloorNondecrease, C | ClearClamp, ClearFloor and ClearAssignment for every field; omega copy and independently derived Y′≥1 | TargetFloorNondecrease: Wave-7 arithmetic and authenticated STATE operands; rate invariant is distinct from nonzero supply and no native witness is fabricated. |
| NativeFeeAuction, C | OpenFlowPolicy, SponsorIsolation, SponsorEnvelopeMultiplicity, Constructibility and optional sponsor witness | NativeFeeAuctionPremise: architecture-owned auction/economic and target fee evidence in Waves 7/11/12; no semantic predicate reads sponsor amounts. |

External requirements name evidence, not new ExternalEvidenceRequirement variants; a later owning packet must justify a typed carrier. The substrate variant keeps its identity. Witness coverage adds no seventh interface.

**Table (Six completed interface ledgers)** · `tab:realization:burn-clear-interfaces`

| Interface | 1 Missing fact / 2 inherited type | 3 Addition | 4 Regression | 5 Identity/schema | 6 Evidence | 7 Dependency impact | 8 Controls/refusals |
|---|---|---|---|---|---|---|---|
| Clear transition/STATE | Result/carrier; StateMetadata, StateField::ALL, FactId::StateField, MissingFactValue | Total transition, X/residual/successor or closed refusal; bound STATE pair and allocated assignment/clamp/floor | Compact and live unchanged; maturity transition and codec; six fields and Guard precedence | No stored-field/codec/census change | 2.1 independent equivalence, 2.7 actual successor; helper domain excludes world lookup | state/expression/evaluate; model tests; unchanged STATE consumers in closure census | Limiters/ties, each field, residual branches, zero/overflow, Sealed versus overflowing B, independent floor |
| Relations/engine keys | Exact laws, including Clear's residual-conditional delta family; Relation/Kind/Subject, ExpectedCanonicalDelta and encoders | Eight body/kind families, two subjects above; existing StaticRequirement | Pilot captures, old codes, keyed isolation, vector pins | Kind 16–23; subject 9/10; no public hash | Exact clause/body/shell outcomes, no target credit | 2.3a stages public/reachable bodies; 2.3b atomically integrates all censused consumers | Coherent omission, wrong/duplicate ID, collision, absent facts, typed unsupported plans |
| Bound extension | STATE/data/payload/context; OperationObservation/references | BurnClearObservation binds normalized common observation, exact members, certificate and primitive fields; checked construction/evaluation | Fourteen literal-initializer files plus observation.rs's struct definition, including state_abi.rs:930; legacy missing facts | Internal only; common struct/projections unchanged | Separate operation/event/credit/accounting/edge verdicts | realization extension/evaluate and model adapters; no transaction migration | Missing extension; right payload on wrong region/certificate/ASH/STATE; foreign object; maturity control |
| Model adapters | Post-execution Burn/Clear; ModelConformanceObservation/ConformanceProjectionError | observe_burn/observe_clear from bound ExecutedTransition; primitive extension and model-only evidence | Compact/live/maturity and raw acceptance | No model law/identity change | Independent expected execution, never realization-dependent acceptance | conformance/lib/tests; independent domain conversion | Accepted controls; substituted execution/source/region/payload; equal-total split disagrees only with bound request; substituted-region subject is received by bite 2.8 and was not built at 2.5 |
| Data outputs | Missing record/destruction observation/relation; OperationObservation (packages/realization/src/observation.rs:222–233), Relation (packages/realization/src/relation.rs:126–198) | Extension BurnRecord { ordinal,address,amount,tag }, Destruction { asset,amount,tag }, association; 32-byte address, wide credit | Old absence; overclaim never voids Burn | DataOutputs allocation above, no wire format | Structure versus credit; leg versus output binding | domain/projection/laws/evaluate; model; compiler/vectors; carrier Waves 4/6 | Empty/max/over-bound, repeated address, bad ordinal/tag/asset/count/association, beyond-domain overclaim, split destruction |
| Burn representation | Object-only Representation and unioned lifecycle modes | SideRepresentation input Explicit, change Explicit/PrivateCommitted; preserve old relation/private exit | Live edge/pins, compact/maturity, public ASH | Side-qualified key only; old identities stable | Admission is not target-form/private-exit discharge | realization owners; 2.3b direct case/lifecycle/proof dispositions; Wave 3 case meaning | Explicit input/private change, private-input refusal despite union, retained private exit, direct case subject |

Scope is complete only per declared operation. Realization owns amounts; model conversion stays independent and credit arithmetic wider. Fact/expression/relation identities stay operation-qualified; shared bodies cannot share evidence across regions/certificates. No external consumer or public realization identity is added. These bounded answers leave future uses of (`sec:realization:open`) open.


## Fixed-form keyed track · `sec:realization:fixed-form`

Under (`rule:phase7:fixed-form-rulings`), the joined gate remains (`rec:backlog:fixed-form-closure`). Architecture owns profile code 3 and the schema-3 symbolic K,A,v manifest; its actual contract receiver is packages/architecture/README.md at S3. Realization owns fixed-form SOURCE/MERGE declarations, exact explicit-pair membership and completeness, in sibling keyed_fixed/keyed_fixed_relation/declarations modules under ruling 208. No legacy semantic object/operation union or relation body is extended. Existing sealed entry traits register the new manifest type; the shared compiler consumes the complete validated declaration, never a reader or fixture verdict. S3 freezes and tests the exact relation census after S1’s proof/anchors and S2’s realization contract; ruling 211 assigns fixed-form capture-freeze admission to S5 and consumer activation to S8 before capture. Authenticated Retrieval must supply fields/forms, and malformed payments are nonmembers with no SOURCE record; native unspendability remains a separate observation. The exact source/registration packet and Wave-2 boundary are (`tab:backlog:fixed-form-paths`), (`tab:backlog:fixed-form-consumers`).
